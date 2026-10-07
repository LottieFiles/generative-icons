// V8-exact Math for QuickJS-NG (generative-icons patch).
//
// The fdlibm functions come from V8's src/base/ieee754.cc (tag 14.6.202.34),
// built with -ffp-contract=on like Node's official clang build. gai_pow keeps
// V8's math::pow special cases (src/numbers/ieee754.cc, --use-std-math-pow)
// and replaces the platform std::pow with CORE-MATH's correctly rounded pow,
// so results do not depend on the host libm.
#include <cmath>
#include <limits>

#include "gai_math.h"
#include "src/base/ieee754.h"

extern "C" double gai_cr_pow(double x, double y);

namespace ie = v8::base::ieee754;

extern "C" {
const char *gai_math_description(void) {
  return "V8 14.6.202.34 fdlibm with FMA contraction, V8 Math.hypot, CORE-MATH pow 708e86ef";
}

double gai_sin(double x) { return ie::sin(x); }
double gai_cos(double x) { return ie::cos(x); }
double gai_tan(double x) { return ie::tan(x); }
double gai_asin(double x) { return ie::asin(x); }
double gai_acos(double x) { return ie::acos(x); }
double gai_atan(double x) { return ie::atan(x); }
double gai_atan2(double y, double x) { return ie::atan2(y, x); }
double gai_exp(double x) { return ie::exp(x); }
double gai_log(double x) { return ie::log(x); }
double gai_cosh(double x) { return ie::cosh(x); }
double gai_sinh(double x) { return ie::sinh(x); }
double gai_tanh(double x) { return ie::tanh(x); }
double gai_acosh(double x) { return ie::acosh(x); }
double gai_asinh(double x) { return ie::asinh(x); }
double gai_atanh(double x) { return ie::atanh(x); }
double gai_expm1(double x) { return ie::expm1(x); }
double gai_log1p(double x) { return ie::log1p(x); }
double gai_log2(double x) { return ie::log2(x); }
double gai_log10(double x) { return ie::log10(x); }
double gai_cbrt(double x) { return ie::cbrt(x); }

double gai_pow(double x, double y) {
  // Exponent NaN, and (+-1) ** (+-Infinity), are NaN in ECMAScript.
  if (std::isnan(y)) return std::numeric_limits<double>::quiet_NaN();
  if (std::isinf(y) && (x == 1 || x == -1)) return std::numeric_limits<double>::quiet_NaN();
  if (std::isnan(x)) x = std::numeric_limits<double>::quiet_NaN();
  // V8's optimizing compilers turn these into x*x and sqrt; the runtime matches.
  if (y == 2) return x * x;
  if (y == 0.5) {
    if (std::isinf(x)) return std::numeric_limits<double>::infinity();
    return std::sqrt(x + 0);  // +0 so that -0 ** 0.5 is +0
  }
  return gai_cr_pow(x, y);
}
}
