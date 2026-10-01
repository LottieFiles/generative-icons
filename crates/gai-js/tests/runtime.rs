//! T-resolution, T-errors and the probe/project contracts, on small sets written to a temp dir.

use gai_js::{
    Error, FRAMES, JsType, Kind, Limits, Location, ModernProfile, ModuleError, ProbeOutcome, ProbeRequest, ProbeResult,
    ProbeStage, ProbeWidth, SetRuntime, SetSource, run_on_worker,
};
use serde_json::json;
use std::{fs, path::Path, time::Duration};
use tempfile::TempDir;

/// A kit file importing svgpath, as `library/drawing.mjs` does.
const KIT: &str = "import svgpath from 'svgpath';
globalThis.kitLoads = (globalThis.kitLoads || 0) + 1;
export const flip = d => svgpath(d).scale(-1, 1).translate(32, 0).toString();
export const icon = (name, parts, extra = {}) => ({ name, label: name, heading: name, description: name, intent: name,
  timings: { light: { active: .5, rest: .54, keyPoses: [0, .5] }, strong: { active: .5, rest: .54, keyPoses: [0, .5] } }, parts, ...extra });
";

/// `icon-sets/99-test/` with `definitions` and a `library/` kit, plus extra files.
fn set(definitions: &str, extra: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let files = [("99-test/definitions.mjs", definitions), ("library/kit.mjs", KIT)];
    for (path, text) in files.iter().chain(extra) {
        let path = dir.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    dir
}

fn source(dir: &TempDir) -> SetSource {
    SetSource::library_set(dir.path().join("99-test")).unwrap()
}

fn with_runtime<T: Send>(dir: &TempDir, limits: Limits, f: impl FnOnce(gai_js::Result<SetRuntime>) -> T + Send) -> T {
    let source = source(dir);
    run_on_worker(&limits.clone(), move || f(SetRuntime::load(&source, &limits)))
}

fn load_error(definitions: &str, extra: &[(&str, &str)]) -> Error {
    let dir = set(definitions, extra);
    with_runtime(&dir, Limits::default(), |runtime| runtime.err().expect("load fails"))
}

fn module_reason(error: Error) -> ModuleError {
    match error {
        Error::Module { reason, .. } => *reason,
        other => panic!("not a module error: {other}"),
    }
}

#[test]
fn kit_modules_load_once_whatever_path_reaches_them() {
    let dir = set(
        "import { icon } from '../library/kit.mjs';
         import { parts } from './batches/a.mjs';
         export const icons = [icon('one', parts)];",
        &[
            (
                "99-test/batches/a.mjs",
                "import { flip } from '../../library/kit.mjs'; import { shape } from './helpers.mjs';
              export const parts = [{ name: 'one-shape', d: flip(shape), style: 'solid' }];",
            ),
            (
                "99-test/batches/helpers.mjs",
                "import { flip } from '../../library/kit.mjs'; export const shape = flip('M1 1 L4 1 L4 4 Z');",
            ),
        ],
    );
    let loads = with_runtime(&dir, Limits::default(), |runtime| {
        let runtime = runtime.unwrap();
        runtime.with_icons(|scope| scope.ctx().eval::<i32, _>("globalThis.kitLoads")).unwrap()
    });
    assert_eq!(loads, 1);
}

#[test]
fn modules_lists_every_file_module_once_in_load_order() {
    let dir = set(
        "import { icon } from '../library/kit.mjs';
         import { parts } from './batches/a.mjs';
         export const icons = [icon('one', parts)];",
        &[
            ("99-test/batches/a.mjs", "import { shape } from './shared/b.mjs'; export const parts = [shape];"),
            (
                "99-test/batches/shared/b.mjs",
                "import { flip } from '../../../library/kit.mjs';
                 export const shape = { name: 'one-shape', d: flip('M1 1 L4 1 L4 4 Z'), style: 'solid' };",
            ),
            ("99-test/batches/unused.mjs", "export const unused = 1;"),
        ],
    );
    let modules = with_runtime(&dir, Limits::default(), |runtime| runtime.unwrap().modules());
    let root = dir.path().canonicalize().unwrap();
    let relative: Vec<_> = modules.iter().map(|path| path.strip_prefix(&root).unwrap().to_path_buf()).collect();
    let expected =
        ["99-test/definitions.mjs", "library/kit.mjs", "99-test/batches/a.mjs", "99-test/batches/shared/b.mjs"];
    assert_eq!(relative, expected.map(Path::new));
}

#[test]
fn imports_outside_the_set_and_kit_are_refused() {
    let outside = load_error("import '../98-other/x.mjs'; export const icons = [];", &[("98-other/x.mjs", "")]);
    assert!(matches!(module_reason(outside), ModuleError::OutsideRoots(_)));
    for specifier in ["node:fs", "lodash", "gai:host", "/etc/hosts"] {
        let error = load_error(&format!("import '{specifier}'; export const icons = [];"), &[]);
        assert_eq!(module_reason(error), ModuleError::Unsupported, "{specifier}");
    }
    let missing = load_error("import './nope.mjs'; export const icons = [];", &[]);
    assert!(matches!(module_reason(missing), ModuleError::NotFound(_)));
    let not_module = load_error("import './data.json'; export const icons = [];", &[("99-test/data.json", "{}")]);
    assert!(matches!(module_reason(not_module), ModuleError::NotModule(_)));
    let bom = load_error("import './bom.mjs'; export const icons = [];", &[("99-test/bom.mjs", "\u{feff}export {};")]);
    assert!(matches!(module_reason(bom), ModuleError::ByteOrderMark(_)));
}

#[cfg(unix)]
#[test]
fn symlinked_kit_files_that_leave_the_kit_are_refused() {
    let dir = set("import '../library/linked.mjs'; export const icons = [];", &[("elsewhere/linked.mjs", "")]);
    std::os::unix::fs::symlink(dir.path().join("elsewhere/linked.mjs"), dir.path().join("library/linked.mjs")).unwrap();
    let error = with_runtime(&dir, Limits::default(), |runtime| runtime.err().unwrap());
    assert!(matches!(module_reason(error), ModuleError::Symlink { .. }));
}

#[cfg(unix)]
#[test]
fn a_symlinked_definitions_file_is_checked_like_any_import() {
    let secret = "API_TOKEN=sk_live_EXAMPLESECRET";
    let dir = set(
        "",
        &[
            ("elsewhere/ci-secrets.env", secret),
            ("elsewhere/definitions.mjs", "export const icons = [];"),
            ("library/definitions.mjs", "export const icons = [];"),
        ],
    );
    let entry = dir.path().join("99-test/definitions.mjs");
    let link_to = |target: &str| {
        fs::remove_file(&entry).unwrap();
        std::os::unix::fs::symlink(dir.path().join(target), &entry).unwrap();
        with_runtime(&dir, Limits::default(), |runtime| runtime.err())
    };
    let error = link_to("elsewhere/ci-secrets.env").expect("a non-module entry is refused");
    assert!(!error.to_string().contains("sk_live"), "{error}");
    assert!(matches!(module_reason(error), ModuleError::NotModule(_)));
    let error = link_to("elsewhere/definitions.mjs").expect("an entry outside the roots is refused");
    assert!(matches!(module_reason(error), ModuleError::Symlink { .. }));
    assert!(link_to("library/definitions.mjs").is_none(), "a link inside the roots loads");
}

#[test]
fn missing_definitions_are_reported() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("99-test")).unwrap();
    let error = with_runtime(&dir, Limits::default(), |runtime| runtime.err().unwrap());
    assert!(matches!(module_reason(error), ModuleError::NotFound(_)));
}

fn script_error(error: Error) -> (Location, String) {
    match error {
        Error::Script { location, message, .. } => (*location, message),
        other => panic!("not a script error: {other}"),
    }
}

#[test]
fn top_level_throws_and_nondeterminism_fail_the_load() {
    let (_, message) = script_error(load_error("throw Error('icons out of order'); export const icons = [];", &[]));
    assert_eq!(message, "icons out of order");
    let (_, message) = script_error(load_error("export const icons = [Math.random()];", &[]));
    assert!(message.contains("Math.random is non-deterministic"), "{message}");
    let (_, message) = script_error(load_error("export const icons = [Date.now()];", &[]));
    assert!(message.contains("not a function"), "{message}");
    let (_, message) = script_error(load_error("export const icons = [new Date()];", &[]));
    assert!(message.contains("Date is not allowed"), "{message}");
    let (_, message) = script_error(load_error("export const icons = [performance.now()];", &[]));
    assert_eq!(message, "performance is not defined");
}

/// A module promise that never settles held its module after the module was freed, and
/// freeing the runtime aborted the process (or wrote to freed memory).
#[test]
fn a_top_level_await_that_never_settles_fails_the_load() {
    let forever = "await new Promise(() => {}); export const icons = [];";
    let imported = "import { parts } from './batches/a.mjs'; export const icons = [];";
    let dynamic = "export const icons = [...(await import('./batches/a.mjs')).parts];";
    let batch = [("99-test/batches/a.mjs", "await new Promise(() => {}); export const parts = [];")];
    for (definitions, extra) in [(forever, &[][..]), (imported, &batch[..]), (dynamic, &batch[..])] {
        let (_, message) = script_error(load_error(definitions, extra));
        assert_eq!(message, "a top-level await never finished: it waits on a promise that nothing settles");
    }
}

/// A call stopped as its module finishes can leave the module's promise pending (the
/// interrupt also stops the call that settles it). That is a timeout, and freeing the
/// runtime afterwards is safe.
#[test]
fn a_load_stopped_at_any_point_is_a_timeout() {
    let definitions = "Array.from({ length: 2 ** 19 }, (_, i) => ({ i })); export const icons = [];";
    let dir = set(definitions, &[]);
    for millis in (5..=120).step_by(5) {
        let limits = Limits { call_timeout: Duration::from_millis(millis), ..Limits::default() };
        match with_runtime(&dir, limits, |runtime| runtime.err()) {
            None | Some(Error::Timeout { .. }) => {}
            Some(other) => panic!("{millis} ms: {other}"),
        }
    }
}

/// Installs `definitions` and samples every variant of the first icon.
fn sample_error(definitions: &str, limits: Limits) -> Error {
    let dir = set(definitions, &[]);
    with_runtime(&dir, limits, |runtime| {
        let mut runtime = runtime.unwrap();
        let description = runtime.install()?;
        (0..description.icons[0].variants.len()).try_for_each(|v| runtime.sample_variant(0, v).map(drop))
    })
    .expect_err("sampling fails")
}

#[test]
fn sampling_errors_name_the_icon_variant_and_part() {
    let topology = "import { icon } from '../library/kit.mjs';
      export const icons = [icon('grow', [{ name: 'grow-bar', style: 'solid', d: t => t > 1 ? 'M0 0 L1 1 L2 2' : 'M0 0 L1 1' }])];";
    let (location, message) = script_error(sample_error(topology, Limits::default()));
    assert_eq!(message, "Changing topology: grow-bar");
    let expected = Location { icon: Some("grow".into()), variant: Some("grow".into()), part: Some("grow-bar".into()) };
    assert_eq!(location, expected);

    let pivot = "import { icon } from '../library/kit.mjs';
      const pose = (x, y) => [x, y]; pose.transform = t => ({ angle: t, pivot: [16, t > 1 ? 17 : 16] });
      export const icons = [icon('spin', [{ name: 'spin-arm', style: 'solid', d: 'M0 0 L1 1', pose }])];";
    let (location, message) = script_error(sample_error(pivot, Limits::default()));
    assert_eq!((message.as_str(), location.part.as_deref()), ("Moving rigid pivot", Some("spin-arm")));

    let pose = "import { icon } from '../library/kit.mjs';
      export const icons = [icon('wave', [{ name: 'wave-line', style: 'solid', d: 'M0 0 L1 1', pose: (x, y) => [x, y, 0] }])];";
    let (_, message) = script_error(sample_error(pose, Limits::default()));
    assert_eq!(message, "Pose must return [x, y]: wave-line");
}

#[test]
fn runaway_calls_stop_at_the_time_limit() {
    let definitions = "import { icon } from '../library/kit.mjs';
      export const icons = [icon('hang', [{ name: 'hang-part', style: 'solid', d: t => { for (;;) {} } }])];";
    let limits = Limits { call_timeout: Duration::from_millis(200), ..Limits::default() };
    match sample_error(definitions, limits) {
        Error::Timeout { location, limit, .. } => {
            assert_eq!(limit, Duration::from_millis(200));
            assert_eq!(location.variant.as_deref(), Some("hang"));
        }
        other => panic!("{other}"),
    }
}

/// Samples an icon whose `d` runs `body` before returning a path, under a 200 ms call limit.
fn runaway(body: &str) -> Error {
    let definitions = format!(
        "import {{ icon }} from '../library/kit.mjs';
      export const icons = [icon('hang', [{{ name: 'hang-part', style: 'solid', d: t => {{ {body}; return 'M0 0 L1 1'; }} }}])];"
    );
    sample_error(&definitions, Limits { call_timeout: Duration::from_millis(200), ..Limits::default() })
}

#[test]
fn native_loops_stop_at_the_time_limit() {
    let cases = [
        // Catastrophic backtracking, also when the author catches errors.
        "/(a+)+$/.test('a'.repeat(40) + 'b')",
        "try { /(a|a)*b/.exec('a'.repeat(40)) } catch (error) { return 'M0 0 L1 1' }",
        // Builtins looping over an array-like's length.
        "Array.prototype.includes.call({ length: 2 ** 53 - 1 }, 1)",
        "Array.prototype.lastIndexOf.call({ length: 2 ** 53 - 1 }, 1)",
        "Array.prototype.join.call({ length: 2 ** 53 - 1 }, '')",
        "Array.prototype.forEach.call({ length: 2 ** 53 - 1 }, () => {})",
        "Array.prototype.fill.call(Object.create(new Uint8Array(0), { length: { value: 2 ** 53 - 1 } }), 0)",
        // Naive string search.
        "'a'.repeat(2 ** 24).indexOf('a'.repeat(2 ** 23) + 'b')",
        "'a'.repeat(2 ** 24).includes('a'.repeat(2 ** 23) + 'b')",
        "'a'.repeat(2 ** 24).split('a'.repeat(2 ** 23) + 'b')",
        // Once quadratic and never polled: ordering combining marks, and resolving
        // forward named back references. Each took minutes for one frame.
        r"('\u0345'.repeat(2 ** 17) + '\u0334'.repeat(2 ** 17)).normalize('NFD')",
        r"('\u0345'.repeat(2 ** 17) + '\u0334'.repeat(2 ** 17)).localeCompare('a')",
        r"new RegExp('\\k<b>'.repeat(2 ** 17) + '(?<b>x)')",
    ];
    for body in cases {
        let started = std::time::Instant::now();
        match runaway(body) {
            Error::Timeout { limit, .. } => assert_eq!(limit, Duration::from_millis(200)),
            other => panic!("{body}: {other}"),
        }
        assert!(started.elapsed() < Duration::from_secs(5), "{body}: took {:?}", started.elapsed());
    }
}

/// QuickJS polls its interrupt handler every so many steps, and a builtin call counts as
/// one, so a loop of builtins that each take milliseconds polled minutes apart. The
/// runtime is now asked to poll at the deadline, so these stop within one builtin call.
#[test]
fn loops_of_costly_builtins_stop_at_the_time_limit() {
    let cases = [
        r"const s = '\u0345'.repeat(2 ** 22); for (;;) s.normalize('NFD')",
        "const s = 'A'.repeat(2 ** 24); for (;;) s.toLowerCase()",
        // Many short regexp executions in one builtin call.
        "'a'.repeat(2 ** 17).replace(/(?=a{0,9000}b)?a/g, '')",
    ];
    for body in cases {
        let started = std::time::Instant::now();
        match runaway(body) {
            Error::Timeout { limit, .. } => assert_eq!(limit, Duration::from_millis(200)),
            other => panic!("{body}: {other}"),
        }
        assert!(started.elapsed() < Duration::from_secs(5), "{body}: took {:?}", started.elapsed());
    }
}

/// Compiling source and regexps is quadratic in places and ignored the time limit; the
/// interrupt handler now stops it too. Each payload is a short string that builds the
/// large input at runtime.
#[test]
fn compilation_stops_at_the_time_limit() {
    let cases = [
        // Parsing and name resolution: each of 2**20 references scans the function's
        // 60,000 declarations. (More than 65,534 declarations is a SyntaxError, not a
        // loop, so the references carry the cost.)
        "const src = '(function(){' + Array.from({ length: 60000 }, (_, i) => `var v${i}=0;`).join('')
           + 'return ()=>[' + Array.from({ length: 2 ** 20 }, (_, i) => `v${i % 60000}`).join(',') + ']})';
         (0, eval)(src)",
        // Regexp compilation: an alternation moves every earlier branch, and a character
        // class merges each atom into the whole class.
        "new RegExp('a|'.repeat(2 ** 16) + 'a')",
        "new RegExp('[' + Array.from({ length: 2 ** 15 }, (_, i) => String.fromCodePoint(0x100 + 2 * i)).join('') + ']', 'u')",
    ];
    for body in cases {
        let started = std::time::Instant::now();
        match runaway(body) {
            Error::Timeout { limit, .. } => assert_eq!(limit, Duration::from_millis(200)),
            other => panic!("{body}: {other}"),
        }
        assert!(started.elapsed() < Duration::from_secs(5), "{body}: took {:?}", started.elapsed());
    }
}

#[test]
fn a_set_stops_once_its_calls_use_up_the_budget() {
    let definitions = "import { icon } from '../library/kit.mjs';
      export const icons = [icon('hang', [{ name: 'hang-part', style: 'solid', d: t => { for (;;) {} } }])];";
    let dir = set(definitions, &[]);
    let limits = Limits {
        call_timeout: Duration::from_millis(100),
        budget: Duration::from_millis(250),
        grace: Duration::from_millis(50),
        ..Limits::default()
    };
    let started = std::time::Instant::now();
    assert_eq!(limits.hard_deadline(), Duration::from_millis(300));
    let (probes, after) = with_runtime(&dir, limits, |runtime| {
        let runtime = runtime.unwrap();
        let request = ProbeRequest {
            icon: 0,
            part: 0,
            strong: false,
            reverse: false,
            width: ProbeWidth::Authored,
            times: &[0.0],
        };
        let probes: Vec<Error> = (0..4).map(|_| runtime.probe(&request).unwrap_err()).collect();
        (probes, runtime.with_icons(|_| Ok(())).unwrap_err())
    });
    let budget = |error: &Error| match error {
        Error::Budget { limit, location, .. } => {
            assert_eq!(*limit, Duration::from_millis(250));
            assert_eq!(location.icon, None, "probe() runs before install()");
            true
        }
        Error::Timeout { limit, .. } => {
            assert_eq!(*limit, Duration::from_millis(100));
            false
        }
        other => panic!("{other}"),
    };
    let kinds: Vec<bool> = probes.iter().map(budget).collect();
    assert!(!kinds[0] && kinds[3] && kinds.is_sorted(), "per-call timeouts, then the budget: {probes:?}");
    assert!(budget(&after), "nothing runs once the budget is used");
    assert!(started.elapsed() < Duration::from_secs(2), "{:?}", started.elapsed());
}

/// Long runs of combining marks are sorted by counting, not insertion, and forward named
/// back references are resolved from one scan of the pattern. Results are unchanged.
#[test]
fn normalization_and_named_references_are_linear_and_unchanged() {
    const CHECKS: &str = r#"(() => {
      const failed = [];
      const N = 2 ** 17;
      const high = 'ͅ'.repeat(N), low = '̴'.repeat(N);
      for (const form of ['NFC', 'NFD', 'NFKC', 'NFKD'])
        if ((high + low).normalize(form) !== low + high) failed.push(form);
      if ((high + low).localeCompare(low + high) !== 0) failed.push('localeCompare');
      if (new RegExp('\\k<b>'.repeat(N) + '(?<b>x)').exec('x')?.groups.b !== 'x') failed.push('forward reference');

      // Two marks are ordered by the unchanged insertion sort; long runs must agree with it.
      const marks = ['̴', 'ְ', 'ཱ', '̧', '̛', '〪', '̖', '॒',
        '̀', '́', '̇', 'ͅ'];
      const before = (a, b) => a !== b && (b + a).normalize('NFD') === a + b;
      if (!before('̴', 'ͅ') || before('̀', '́') || before('́', '̀'))
        failed.push('pairs');
      const sortMarks = run => run.map((mark, i) => [mark, i])
        .sort((a, b) => (before(a[0], b[0]) ? -1 : before(b[0], a[0]) ? 1 : a[1] - b[1]))
        .map(([mark]) => mark).join('');
      let seed = 7;
      const next = () => (seed = (seed * 1103515245 + 12345) % 2147483648);
      for (const length of [2, 31, 32, 33, 34, 65, 1000, 5000]) {
        const run = Array.from({ length }, () => marks[next() % marks.length]);
        // U+1E9B decomposes to U+017F U+0307, which joins the run that follows.
        const text = 'a' + run.join('') + 'b' + run.join('') + 'ẛ' + run.join('');
        const nfd = text.normalize('NFD');
        if (nfd !== 'a' + sortMarks(run) + 'b' + sortMarks(run) + 'ſ' + sortMarks(['̇', ...run]))
          failed.push(`NFD ${length}`);
        if (text.normalize('NFC').normalize('NFD') !== nfd) failed.push(`NFC ${length}`);
      }

      // Named references resolve as before, forward or backward.
      const g = Array.from({ length: 10 }, (_, i) => `(?<g${i}>${i})`).join('');
      const matches = [
        ['\\k<a>(?<a>x)\\k<a>', 'xx', 'xx'],
        ['(?<a>x)(?<b>y)\\k<b>\\k<a>', 'xyyx', 'xyyx'],
        ['\\k<b>(x)(?<b>z)\\k<b>', 'xzz', 'xzz'],
        ['\\k<g9>' + g + '\\k<g9>', '01234567899', '01234567899'],
        ['\\k<a>', 'k<a>', 'k<a>'],
      ];
      for (const [source, input, match] of matches)
        if (new RegExp(source).exec(input)?.[0] !== match) failed.push(source);
      if (new RegExp('\\k<a>(?<a>x)', 'u').exec('x')?.[0] !== 'x') failed.push('unicode');
      for (const source of ['\\k<zz>(?<a>x)', '(?<a>x)(?<a>y)\\k<a>'])
        try { new RegExp(source); failed.push(source) } catch (error) { if (!(error instanceof SyntaxError)) failed.push(source) }
      return failed;
    })()"#;
    let dir = set("export const icons = [];", &[]);
    let started = std::time::Instant::now();
    let failed: Vec<String> = with_runtime(&dir, Limits::default(), |runtime| {
        runtime.unwrap().with_icons(|scope| scope.ctx().eval(CHECKS)).unwrap()
    });
    assert!(failed.is_empty(), "{failed:?}");
    assert!(started.elapsed() < Duration::from_secs(20), "took {:?}", started.elapsed());
}

#[test]
fn atomics_cannot_block() {
    let (_, message) = script_error(runaway("Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0)"));
    assert!(message.contains("block"), "{message}");
}

#[test]
fn install_applies_the_shared_builders_checks() {
    let duotone = "import { icon } from '../library/kit.mjs';
      export const icons = [icon('mark', [{ name: 'mark-fill', style: 'solid', d: 'M0 0 L1 1' }], { duotoneFill: true })];";
    let duplicate = "import { icon } from '../library/kit.mjs';
      export const icons = [icon('mark', []), icon('mark', [])];";
    for (definitions, expected) in
        [(duotone, "Declared Duotone policy disagrees with drawing: mark"), (duplicate, "Duplicate icon name")]
    {
        let dir = set(definitions, &[]);
        let error = with_runtime(&dir, Limits::default(), |runtime| runtime.unwrap().install().unwrap_err());
        assert_eq!(script_error(error).1, expected);
    }
}

const SAMPLED: &str = "import { icon } from '../library/kit.mjs';
  const rigid = (x, y) => [x, y]; rigid.transform = t => ({ angle: 90 * t, dx: 1 });
  export const icons = [
    icon('toggle', [
      { name: 'toggle-track', style: 'outline', type: 'stroke', width: 2, d: (t, s, r, w) => `M2 16 L${30 - w} 16` },
      { name: 'toggle-knob', style: 'solid', d: 'M10 12 L14 12 L14 16 Z', pose: rigid, visibility: t => t < 1 ? 1 : .5 },
      { name: 'toggle-glow', style: 'tint', d: 'M0 0 H4 V4 Z' },
    ], { stateful: true, reverseSuffix: '-off', number: 7,
         reverseTimings: { light: { active: .3, rest: .31, keyPoses: [0] }, strong: { active: .3, rest: .31, keyPoses: [0] } } }),
  ];";

#[test]
fn install_expands_weights_and_describes_variants() {
    let dir = set(SAMPLED, &[]);
    let (description, samples) = with_runtime(&dir, Limits::default(), |runtime| {
        let mut runtime = runtime.unwrap();
        let description = runtime.install().unwrap();
        let samples = runtime.sample_variant(0, 1).unwrap();
        (description, samples)
    });
    assert_eq!(description.kind, Kind::Modern);
    let icon = &description.icons[0];
    let names: Vec<&str> = icon.variants.iter().map(|v| v.name.as_str()).collect();
    assert_eq!(names, ["toggle", "toggle-off", "toggle-strong", "toggle-off-strong"]);
    assert_eq!(icon.variants[0].rest(), Some(33.0 / 60.0));
    assert_eq!(icon.variants[1].rest(), Some(19.0 / 60.0));
    assert_eq!(icon.meta["number"], json!(7));
    let nodes: Vec<&str> = icon.parts.iter().map(|p| p.node.name.as_str()).collect();
    assert_eq!(
        nodes,
        [
            "toggle-track-light__color.primary__outline.light",
            "toggle-track-regular__color.primary__outline.regular",
            "toggle-track-bold__color.primary__outline.bold",
            "toggle-knob__color.primary__solid",
            "toggle-glow__color.primary__tint",
        ]
    );
    assert_eq!(icon.parts.iter().map(|p| p.source_index).collect::<Vec<_>>(), [0, 0, 0, 1, 2]);
    assert_eq!(icon.parts[1].node.opacity, 1.0);
    assert_eq!((icon.parts[0].node.width.clone(), icon.parts[3].node.width.clone()), (json!(1.5), json!(1)));
    assert_eq!(icon.parts[0].node.r#type, Some(json!("stroke")));
    assert_eq!(icon.parts[4].node.r#type, Some(json!("fill")));
    assert_eq!(icon.source_parts[1].callbacks.pose, JsType::Function);
    assert!(icon.source_parts[1].callbacks.transform);

    // d receives each weight's width; H and V expand to pairs.
    assert_eq!(samples.parts[0].frame(0), [2.0, 16.0, 28.5, 16.0]);
    assert_eq!(samples.parts[2].frame(FRAMES - 1), [2.0, 16.0, 27.5, 16.0]);
    assert_eq!(samples.parts[4].commands, [0, 1, 1, 3]);
    assert_eq!(samples.parts[4].frame(0), [0.0, 0.0, 4.0, 0.0, 4.0, 4.0]);
    let knob = &samples.parts[3];
    let rigid = knob.rigid.as_ref().expect("static d with a rigid pose");
    assert_eq!(rigid.anchor, [16.0, 16.0]);
    assert_eq!(rigid.position[0], [17.0, 16.0]);
    assert_eq!(rigid.rotation[60], -(90.0f64 * 1.0) * std::f64::consts::PI / 180.0);
    assert_eq!(knob.visibility.as_ref().unwrap()[60], 0.5);
}

/// Probes the first part of the first icon at t = 0 and t = 1.
/// Replaces, once the definitions have run, builtins the host calls that neither svgpath nor
/// the fixtures use. The replacements throw, so any call the host still makes shows.
const MONKEYPATCH: &str = "
  const E = Error, forged = name => () => { throw E('forged ' + name); };
  const typed = Object.getPrototypeOf(Int32Array.prototype);
  for (const name of ['every', 'some', 'join', 'includes', 'fill']) Array.prototype[name] = forged(name);
  for (const name of ['set', 'fill', 'copyWithin']) typed[name] = forged(name);
  Object.defineProperty(typed, 'length', { get: forged('length') });
  Float64Array.from = Int32Array.from = Uint8Array.from = forged('from');
  Map.prototype.get = Map.prototype.set = Set.prototype.add = forged('collection');
  JSON.stringify = forged('stringify');
  Object.entries = Object.assign = Object.fromEntries = forged('Object');
  Number.isFinite = forged('isFinite');
  Math.round = forged('round');
  Object.defineProperty(Number, Symbol.hasInstance, { value: forged('instanceof') });
  for (const name of ['Error', 'Boolean', 'Map', 'Set', 'Float64Array', 'Int32Array', 'Uint8Array']) globalThis[name] = forged(name);
";

#[test]
fn replaced_builtins_do_not_change_what_the_host_reports() {
    let evaluate = |definitions: &str| {
        let dir = set(definitions, &[]);
        with_runtime(&dir, Limits::default(), |runtime| {
            let mut runtime = runtime.unwrap();
            let description = runtime.install().unwrap();
            let variants = description.icons[0].variants.len();
            let samples: Vec<_> = (0..variants).map(|v| runtime.sample_variant(0, v).unwrap()).collect();
            (description, samples)
        })
    };
    assert_eq!(evaluate(&format!("{SAMPLED}{MONKEYPATCH}")), evaluate(SAMPLED));

    let nonfinite = format!(
        "import {{ icon }} from '../library/kit.mjs'; {MONKEYPATCH}
         export const icons = [icon('p', [{{ name: 'p-x', style: 'solid', d: 'M1e400 0 L1 1' }}])];"
    );
    let result = probe(&nonfinite, false, ProbeWidth::Value(2.0));
    let message = "Nonfinite path coordinates".into();
    assert_eq!(result.samples[0].outcome, ProbeOutcome::InvalidGeometry { message, svg_error: String::new() });

    let topology = format!(
        "import {{ icon }} from '../library/kit.mjs'; {MONKEYPATCH}
         export const icons = [icon('grow', [{{ name: 'grow-bar', style: 'solid', d: t => t > 1 ? 'M0 0 L1 1 L2 2 Z' : 'M0 0 L1 1 Z M2 2' }}])];"
    );
    let (_, message) = script_error(sample_error(&topology, Limits::default()));
    assert_eq!(message, "Changing topology: grow-bar");
}

fn probe(definitions: &str, strong: bool, width: ProbeWidth) -> ProbeResult {
    let dir = set(definitions, &[]);
    let request = ProbeRequest { icon: 0, part: 0, strong, reverse: strong, width, times: &[0.0, 1.0] };
    with_runtime(&dir, Limits::default(), |runtime| runtime.unwrap().probe(&request).unwrap())
}

fn part_probe(part: &str) -> ProbeResult {
    let definitions =
        format!("import {{ icon }} from '../library/kit.mjs'; export const icons = [icon('p', [{part}])];");
    probe(&definitions, false, ProbeWidth::Value(2.0))
}

#[test]
fn probe_reports_preflight_geometry_errors() {
    for (d, message) in [
        ("' '", "Path must be a nonempty SVG path string"),
        ("'M0 0 Q1 1 2 2'", "Unsupported native command Q"),
        ("'M1e400 0 L1 1'", "Nonfinite path coordinates"),
        ("'M0 0 Z'", "Path has no drawable segment"),
    ] {
        let result = part_probe(&format!("{{ name: 'p-x', style: 'solid', d: {d} }}"));
        assert_eq!(
            result.samples[0].outcome,
            ProbeOutcome::InvalidGeometry { message: message.into(), svg_error: String::new() },
            "{d}"
        );
    }
    let result = part_probe("{ name: 'p-x', style: 'solid', d: 'M0 0 L' }");
    match &result.samples[0].outcome {
        ProbeOutcome::InvalidGeometry { message, svg_error } => assert!(!svg_error.is_empty() && message == svg_error),
        other => panic!("{other:?}"),
    }
}

#[test]
fn probe_reports_which_callback_failed() {
    let threw = |stage, message: Option<serde_json::Value>| ProbeOutcome::Threw { stage, message };
    let cases = [
        ("d: () => { throw Error('no path'); }", threw(ProbeStage::D, Some(json!("no path")))),
        ("d: 'M0 0 L1 1', visibility: () => { throw 'plain'; }", threw(ProbeStage::Visibility, None)),
        ("d: 'M0 0 L1 1', pose: () => { throw null; }", ProbeOutcome::ThrewNullish { stage: ProbeStage::Pose }),
        (
            "d: 'M0 0 L1 1', pose: Object.assign((x, y) => [x, y], { transform: () => undefined })",
            ProbeOutcome::TransformReturnedNullish { null: false },
        ),
        (
            "d: 'M0 0 L1 1', pose: Object.assign((x, y) => [x, y], { transform: () => null })",
            ProbeOutcome::TransformReturnedNullish { null: true },
        ),
    ];
    for (fields, outcome) in cases {
        let result = part_probe(&format!("{{ name: 'p-x', style: 'solid', {fields} }}"));
        assert_eq!(result.samples[1].outcome, outcome, "{fields}");
    }
}

#[test]
fn probe_keeps_uncoerced_judgements() {
    let result = part_probe(
        "{ name: 'p-x', style: 'solid', d: (t, s, r, w) => `M0 0 L${w} ${t}`, visibility: t => t ? null : '0.5',
           pose: Object.assign((x, y, t) => t ? [x, NaN] : [x, y], { transform: t => ({ dx: null, angle: '5', pivot: t ? [16] : undefined }) }) }",
    );
    let [first, second] = [&result.samples[0], &result.samples[1]];
    assert_eq!((result.static_d, result.callbacks.d), (false, JsType::Function));
    assert_eq!(first.outcome, ProbeOutcome::Complete);
    assert_eq!(first.geometry.as_ref().unwrap().points, [0.0, 0.0, 2.0, 0.0]);
    assert_eq!(first.geometry.as_ref().unwrap().topology, "ML");
    let visibility = first.visibility.unwrap();
    assert_eq!((visibility.value, visibility.is_number, visibility.nullish), (0.5, false, false));
    assert!(second.visibility.unwrap().nullish);
    assert!(first.posed.iter().all(|p| p.ok));
    assert!(!second.posed[0].ok && second.posed[0].x.is_nan());
    let transform = first.transform.as_ref().unwrap();
    assert_eq!(transform.nullish, [0.0, 0.0, 5.0]);
    assert_eq!(transform.or, [5.0, 0.0, 0.0]);
    assert_eq!((transform.pivot, transform.ok, transform.pivot_json.as_str()), (Some([16.0, 16.0]), false, "[16,16]"));
    let transform = second.transform.as_ref().unwrap();
    assert_eq!((transform.pivot, transform.pivot_json.as_str()), (None, "[16]"));
}

#[test]
fn probe_passes_the_authored_width_when_asked() {
    let definitions = "import { icon } from '../library/kit.mjs';
      export const icons = [icon('p', [{ name: 'p-x', style: 'solid', width: 2.6, d: function (t, s, r, w) { return `M0 0 L${w} ${arguments.length}`; } }])];";
    let result = probe(definitions, true, ProbeWidth::Authored);
    assert_eq!(result.samples[0].geometry.as_ref().unwrap().points, [0.0, 0.0, 2.6, 4.0]);
}

#[test]
fn project_follows_json_stringify() {
    let dir = set("export const icons = [];", &[]);
    let projected = with_runtime(&dir, Limits::default(), |runtime| {
        let runtime = runtime.unwrap();
        runtime
            .with_icons(|scope| {
                let value = scope.ctx().eval(
                    "({ b: 1, a: [undefined, () => 1, 0.5, -0, 1e21, 2 ** -24], skip: undefined, fn() {},
                        '\\u0000raw': '\\u0000text', nested: { toJSON: () => 'custom' }, 2: 'int key', big: 2 ** 70 })",
                )?;
                scope.project(value)
            })
            .unwrap()
            .unwrap()
    });
    assert_eq!(
        gai_core::to_js_json(&projected),
        r#"{"2":"int key","b":1,"a":[null,null,0.5,0,1e+21,5.960464477539063e-8],"\u0000raw":"\u0000text","nested":"custom","big":1.1805916207174113e+21}"#
    );
    let error = with_runtime(&dir, Limits::default(), |runtime| {
        let runtime = runtime.unwrap();
        runtime.with_icons(|scope| scope.project(scope.ctx().eval("({ n: 1n })")?)).unwrap_err()
    });
    assert!(matches!(error, Error::Script { .. }), "{error}");
}

#[test]
fn user_projects_use_the_shared_profile() {
    let dir = set("export const icons = [];", &[]);
    let source = SetSource::user_project(dir.path().join("99-test"));
    assert!(matches!(source, SetSource::Definitions { profile: ModernProfile::SHARED, .. }));
    let limits = Limits::default();
    let description = run_on_worker(&limits, || SetRuntime::load(&source, &limits)?.install()).unwrap();
    assert!(description.icons.is_empty());
    let _: &Path = dir.path();
}

#[test]
fn runaway_recursion_throws_on_any_thread() {
    let dir = set("const down = n => down(n + 1) + 1; export const icons = [down(0)];", &[]);
    // The test thread (2 MiB) and a worker (16 MiB, full limit) both get a JS error, not an abort.
    let direct = SetRuntime::load(&source(&dir), &Limits::default()).err().expect("load fails");
    let worker = with_runtime(&dir, Limits::default(), |runtime| runtime.err().expect("load fails"));
    for error in [direct, worker] {
        let (_, message) = script_error(error);
        assert_eq!(message, "Maximum call stack size exceeded");
    }
}
