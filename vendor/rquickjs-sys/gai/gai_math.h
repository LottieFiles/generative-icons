/* V8-exact Math for QuickJS-NG (gen-animated-icons patch).
 *
 * quickjs.c calls these instead of the platform libm so Math.* and the **
 * operator return the same doubles as Node's V8 14.6 on arm64. */
#ifndef GAI_MATH_H
#define GAI_MATH_H
#ifdef __cplusplus
extern "C" {
#endif

/* V8 src/base/ieee754.cc (fdlibm), compiled with FMA contraction. */
double gai_sin(double x);
double gai_cos(double x);
double gai_tan(double x);
double gai_asin(double x);
double gai_acos(double x);
double gai_atan(double x);
double gai_atan2(double y, double x);
double gai_exp(double x);
double gai_log(double x);
double gai_cosh(double x);
double gai_sinh(double x);
double gai_tanh(double x);
double gai_acosh(double x);
double gai_asinh(double x);
double gai_atanh(double x);
double gai_expm1(double x);
double gai_log1p(double x);
double gai_log2(double x);
double gai_log10(double x);
double gai_cbrt(double x);

/* V8 math::pow special cases over a correctly rounded pow (CORE-MATH). */
double gai_pow(double x, double y);

/* What these functions are, for version reports: a NUL-terminated string with static
 * storage. Linking against it also proves the patched engine is the one in use. */
const char *gai_math_description(void);

#ifdef __cplusplus
}
#endif
#endif
