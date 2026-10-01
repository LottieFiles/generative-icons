// Stand-in for the helpers of V8's src/base/overflowing-math.h (tag 14.6.202.34)
// that ieee754.cc uses, with the same semantics.
#pragma once
#include <cmath>
#include <limits>
#include <type_traits>

namespace v8 {
namespace base {
template <typename T>
inline T NegateWithWraparound(T a) {
  static_assert(std::is_integral_v<T> && std::is_signed_v<T>);
  if (a == std::numeric_limits<T>::min()) return a;
  return -a;
}

template <typename T>
inline T SubWithWraparound(T a, T b) {
  static_assert(std::is_integral_v<T> && std::is_signed_v<T>);
  using U = std::make_unsigned_t<T>;
  return static_cast<T>(static_cast<U>(a) - static_cast<U>(b));
}

template <typename T>
inline T Divide(T x, T y) {
  if (y != 0) return x / y;
  if (x == 0 || x != x) return std::numeric_limits<T>::quiet_NaN();
  if ((x >= 0) == (std::signbit(y) == 0)) return std::numeric_limits<T>::infinity();
  return -std::numeric_limits<T>::infinity();
}
}  // namespace base
}  // namespace v8
