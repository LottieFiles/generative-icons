// Stand-in for the parts of V8's src/base/macros.h that ieee754.cc uses.
#pragma once
#include <cstdint>
#include <cstring>

#define V8_INLINE inline
#define V8_WARN_UNUSED_RESULT

namespace v8 {
namespace base {
template <class Dest, class Source>
inline Dest bit_cast(const Source& source) {
  static_assert(sizeof(Dest) == sizeof(Source), "bit_cast needs equal sizes");
  Dest dest;
  std::memcpy(&dest, &source, sizeof(dest));
  return dest;
}
}  // namespace base
}  // namespace v8
