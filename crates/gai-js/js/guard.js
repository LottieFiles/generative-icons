// Evaluated as a script before any module: authoring code must be deterministic.
// The rest of Math is V8-exact at C level (vendor/rquickjs-sys/PATCHED.md).
Math.random = () => {
  throw Error('gen-animated-icons: Math.random is non-deterministic and not allowed in icon definitions');
};
globalThis.Date = function Date() {
  throw Error('gen-animated-icons: Date is not allowed in icon definitions');
};
// QuickJS-NG's performance.now() and timeOrigin read the clock.
delete globalThis.performance;
