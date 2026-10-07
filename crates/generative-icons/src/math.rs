//! A self-test of the embedded engine's `Math` against values captured from V8 (Node).
//!
//! Icons must sample exactly as the library's builder did, which needs V8's fdlibm built
//! with FMA contraction. A binary built with other compiler flags or another libm still
//! runs, but its scenes differ; these vectors catch it. Most are inputs where a platform
//! libm differs from V8 (taken from gai-js's T-math fixtures), plus the `sin` that broke
//! when compiler flags dropped contraction.

use crate::error::{CliError, ErrorCode, Result};
use gai_js::rquickjs::{Context, Runtime};

/// `(expression, the bits of V8's result)`.
const VECTORS: [(&str, u64); 23] = [
    ("Math.sin(0.22836509603277938)", 0x3fcc_fa32_1d05_e797),
    ("Math.sin(61.82188779520061)", 0xbfeb_1918_5d84_3b2c),
    ("Math.sin(-1.2550951820197235e+264)", 0x3fe5_033f_e0ba_f136),
    ("Math.cos(-2.0744333352481268e+165)", 0xbfe4_171c_a631_e3cc),
    ("Math.cos(-2.1257032713867222e+291)", 0xbfda_7edb_dd37_1783),
    ("Math.tan(1)", 0x3ff8_eb24_5cbe_e3a6),
    ("Math.tan(-1)", 0xbff8_eb24_5cbe_e3a6),
    ("Math.atan(0.5)", 0x3fdd_ac67_0561_bb4f),
    ("Math.exp(-120.83102513683346)", 0x3509_97d1_0a33_bddc),
    ("Math.exp(233.14612858675267)", 0x54f4_8468_ac98_fe32),
    ("Math.log(3)", 0x3ff1_93ea_7aad_030a),
    ("Math.cbrt(3.141592653589793)", 0x3ff7_6ef7_e731_04b7),
    ("Math.cbrt(-3.141592653589793)", 0xbff7_6ef7_e731_04b7),
    ("Math.expm1(1)", 0x3ffb_7e15_1628_aed2),
    ("Math.expm1(1.0000000000000002)", 0x3ffb_7e15_1628_aed6),
    ("Math.sinh(2)", 0x400d_03cf_63b6_e1a0),
    ("Math.atan2(2.5, -1)", 0x3fff_3889_2f1e_0c89),
    ("Math.atan2(-2.5, -1)", 0xbfff_3889_2f1e_0c89),
    ("Math.pow(-0.003531269022447244, 5)", 0xbd63_51e1_eab2_f658),
    ("(-0.003531269022447244) ** 5", 0xbd63_51e1_eab2_f658),
    ("Math.hypot(-47.65107596297621, 9.35893500324496)", 0x4048_47dd_b008_fc16),
    ("Math.hypot(-0.2874788462144139, -0.7443557845988457)", 0x3fe9_88bb_46ec_0cbd),
    ("Math.sqrt(2)", 0x3ff6_a09e_667f_3bcd),
];

/// A vector the engine computed differently.
#[derive(Debug, Clone, PartialEq)]
pub struct Mismatch {
    /// The JS expression.
    pub expression: &'static str,
    /// What V8 returns.
    pub expected: f64,
    /// What this binary returned (`None` when evaluation failed).
    pub actual: Option<f64>,
}

impl std::fmt::Display for Mismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let expected = gai_core::js_number_to_string(self.expected);
        match self.actual {
            Some(actual) => {
                write!(f, "{} is {}, V8 gives {expected}", self.expression, gai_core::js_number_to_string(actual))
            }
            None => write!(f, "{} failed to evaluate", self.expression),
        }
    }
}

/// Evaluates every vector in a fresh QuickJS context. Returns how many were checked, or the
/// first that differs. The CPU must support the engine (see [`gai_js::check_cpu`]).
pub fn self_test() -> std::result::Result<usize, Mismatch> {
    let failed = |expression, actual| Mismatch { expression, expected: f64::from_bits(0), actual };
    let runtime = Runtime::new().map_err(|_| failed(VECTORS[0].0, None))?;
    let context = Context::full(&runtime).map_err(|_| failed(VECTORS[0].0, None))?;
    context.with(|ctx| {
        for (expression, bits) in VECTORS {
            let expected = f64::from_bits(bits);
            let actual = ctx.eval::<f64, _>(expression).ok();
            if actual.map(f64::to_bits) != Some(bits) {
                return Err(Mismatch { expression, expected, actual });
            }
        }
        Ok(VECTORS.len())
    })
}

/// Fails with `INTERNAL` when the CPU cannot run the engine or its `Math` differs from V8,
/// before any icon source is evaluated.
pub fn verify() -> Result<()> {
    gai_js::check_cpu().map_err(super::commands::check::cpu_error)?;
    self_test().map(drop).map_err(|mismatch| {
        CliError::new(
            ErrorCode::Internal,
            format!("This generative-icons binary computes Math differently from V8 ({mismatch}), so its icons would not match the library"),
        )
        .with_hint("Reinstall generative-icons from the official release, then run generative-icons doctor")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_engine_matches_v8() {
        assert_eq!(self_test(), Ok(VECTORS.len()));
        verify().unwrap();
    }

    #[test]
    fn mismatches_name_the_expression() {
        let mismatch = Mismatch { expression: "Math.sin(1)", expected: 0.5, actual: Some(0.25) };
        assert_eq!(mismatch.to_string(), "Math.sin(1) is 0.25, V8 gives 0.5");
    }
}
