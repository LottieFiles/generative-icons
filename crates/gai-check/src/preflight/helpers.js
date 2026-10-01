// The JS operations preflight applies to author values (preflight.mjs:47-90), so reads,
// typeof, truthiness and coercions keep their exact semantics. The checks themselves are Rust.
(() => {
  const ids = new Set();
  return {
    get: (value, key) => value[key],
    typeOf: value => typeof value,
    truthy: value => !!value,
    isArray: value => Array.isArray(value),
    has: (value, key) => key in value,
    toNumber: value => +value,
    kebab: name => /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/.test(name),
    // ids.has(name), then ids.add(name) (SameValueZero).
    seen: name => {
      const had = ids.has(name);
      ids.add(name);
      return had;
    },
  };
})()
