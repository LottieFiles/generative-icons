//! T-math and T-toFixed: the embedded engine's `Math`, `**` and `toFixed` against V8.
//!
//! Rows are evaluated through QuickJS (`Math.sin(x)`, `a ** b`, …) and must be bit-equal,
//! any NaN matching any NaN. Expectations for `pow` and `**` are correctly rounded: Node
//! calls the platform `pow`, which is not (see `vendor/rquickjs-sys/PATCHED.md`).
//!
//! - `fixtures/math/*.f64`: synthetic vectors from Node for every patched function and
//!   2–6-argument `Math.hypot`, including the functions the corpus never calls (`generate.mjs`).
//! - The corpus capture (`js-runtime-prototype/mathcap/*.f64`, about 280k rows), read from
//!   `GAI_MATHCAP_DIR` or `.local/parity-oracles/mathcap`; skipped when absent. Its 19 rows
//!   where Node's `pow` is not correctly rounded are in `fixtures/mathcap-pow-cr.f64` as
//!   `[x, y, node, correctly rounded]`.

use gai_js::rquickjs::{Context, Function, Runtime, TypedArray};
use std::{
    fs,
    path::{Path, PathBuf},
};

const UNARY: [&str; 21] = [
    "sin", "cos", "tan", "asin", "acos", "atan", "exp", "log", "cosh", "sinh", "tanh", "acosh", "asinh", "atanh",
    "expm1", "log1p", "log2", "log10", "cbrt", "sqrt", "atan2",
];

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_f64(path: &Path) -> Vec<f64> {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    bytes.chunks_exact(8).map(|b| f64::from_le_bytes(b.try_into().unwrap())).collect()
}

fn same(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}

/// Evaluates `Math[name]` (or `**` for `powop`) on each row's arguments in QuickJS.
fn evaluate(name: &str, rows: &[f64]) -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>) {
    let (mut args, mut expected) = (Vec::new(), Vec::new());
    let mut i = 0;
    while i < rows.len() {
        let argc = match name {
            "hypot" => {
                i += 1;
                rows[i - 1] as usize
            }
            "atan2" | "pow" | "powop" => 2,
            _ => 1,
        };
        args.push(rows[i..i + argc].to_vec());
        expected.push(rows[i + argc]);
        i += argc + 1;
    }
    let runtime = Runtime::new().unwrap();
    let context = Context::full(&runtime).unwrap();
    let flat: Vec<f64> = args.iter().flatten().copied().collect();
    let counts: Vec<i32> = args.iter().map(|a| a.len() as i32).collect();
    let actual = context.with(|ctx| {
        let run: Function = ctx
            .eval(
                "(name, flat, counts) => { const out = new Float64Array(counts.length); let at = 0;
                   for (let i = 0; i < counts.length; i++) {
                     const a = Array.from(flat.subarray(at, at + counts[i])); at += counts[i];
                     out[i] = name === 'powop' ? a[0] ** a[1] : Math[name](...a);
                   }
                   return out; }",
            )
            .unwrap();
        let flat = TypedArray::<f64>::new_copy(ctx.clone(), &flat).unwrap();
        let counts = TypedArray::<i32>::new_copy(ctx.clone(), &counts).unwrap();
        let out: TypedArray<f64> = run.call((name, flat, counts)).unwrap();
        AsRef::<[f64]>::as_ref(&out).to_vec()
    });
    (args, expected, actual)
}

/// Mismatching rows, each as `(args, expected, actual)`.
fn mismatches(name: &str, rows: &[f64]) -> (usize, Vec<(Vec<f64>, f64, f64)>) {
    let (args, expected, actual) = evaluate(name, rows);
    let bad = args
        .into_iter()
        .zip(expected.iter().zip(&actual))
        .filter(|(_, (e, a))| !same(**e, **a))
        .map(|(args, (e, a))| (args, *e, *a))
        .collect();
    (expected.len(), bad)
}

#[test]
fn synthetic_vectors_match_v8() {
    let mut total = 0;
    for name in UNARY.iter().chain(&["pow", "powop", "hypot"]) {
        let (count, bad) = mismatches(name, &read_f64(&fixtures().join(format!("math/{name}.f64"))));
        assert!(bad.is_empty(), "{name}: {} of {count} rows differ, e.g. {:?}", bad.len(), &bad[..bad.len().min(3)]);
        total += count;
    }
    assert_eq!(total, 9108, "fixture rows");
}

#[test]
fn hypot_keeps_v8_compensated_sum_for_many_arguments() {
    let rows = read_f64(&fixtures().join("math/hypot.f64"));
    let (args, _, _) = evaluate("hypot", &rows);
    for argc in 3..=6 {
        assert!(args.iter().filter(|a| a.len() == argc).count() >= 100, "{argc} arguments");
    }
}

#[test]
fn corpus_math_matches_v8() {
    let dir = std::env::var_os("GAI_MATHCAP_DIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/parity-oracles/mathcap"),
        PathBuf::from,
    );
    if !dir.is_dir() {
        eprintln!("skipped: no math capture at {}", dir.display());
        return;
    }
    let exceptions: Vec<[f64; 4]> =
        read_f64(&fixtures().join("mathcap-pow-cr.f64")).chunks_exact(4).map(|r| [r[0], r[1], r[2], r[3]]).collect();
    let mut total = 0;
    let mut correctly_rounded = 0;
    for name in ["sin", "cos", "tan", "asin", "acos", "atan", "exp", "sqrt", "atan2", "pow", "powop", "hypot"] {
        let (count, bad) = mismatches(name, &read_f64(&dir.join(format!("{name}.f64"))));
        total += count;
        for (args, expected, actual) in &bad {
            let known = exceptions
                .iter()
                .any(|r| same(r[0], args[0]) && same(r[1], args[1]) && same(r[2], *expected) && same(r[3], *actual));
            assert!(known && name.starts_with("pow"), "{name}{args:?}: V8 {expected:e}, engine {actual:e}");
            correctly_rounded += 1;
        }
    }
    eprintln!("{total} captured rows bit-equal to V8, except {correctly_rounded} correctly rounded pow rows");
    assert_eq!(correctly_rounded, exceptions.len());
}

#[test]
fn to_fixed_matches_v8() {
    let text = fs::read_to_string(fixtures().join("to-fixed.txt")).unwrap();
    let (values, expected): (Vec<f64>, Vec<&str>) = text
        .lines()
        .map(|line| {
            let (bits, fixed) = line.split_once(' ').unwrap();
            (f64::from_bits(u64::from_str_radix(bits, 16).unwrap()), fixed)
        })
        .unzip();
    let runtime = Runtime::new().unwrap();
    let context = Context::full(&runtime).unwrap();
    let actual: Vec<String> = context.with(|ctx| {
        let run: Function = ctx.eval("values => Array.from(values, v => v.toFixed(3))").unwrap();
        run.call((TypedArray::<f64>::new_copy(ctx.clone(), &values).unwrap(),)).unwrap()
    });
    let bad: Vec<_> = values.iter().zip(expected.iter().zip(&actual)).filter(|(_, (e, a))| *e != a).collect();
    assert!(bad.is_empty(), "{} of {} differ, e.g. {:?}", bad.len(), values.len(), &bad[..bad.len().min(3)]);
}
