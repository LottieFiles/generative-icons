# rquickjs-sys 0.9.0, patched for V8-exact Math and bounded builtins

This is the crates.io package `rquickjs-sys 0.9.0` (QuickJS-NG 0.8.0). The root
`Cargo.toml` substitutes it through `[patch.crates-io]`. gen-animated-icons runs icon
authoring code in QuickJS, and its committed outputs were produced by Node (V8 14.6,
arm64). Stock QuickJS calls the platform libm, which differs from V8 by up to 20 ULP in
`Math.sin`, `cos`, `tan`, `asin`, `acos`, `exp`, `atan2`, `hypot` and `**`, and that
changed 83 of 3,094 scenes. With this patch all 3,094 are byte-identical.

## What changed

- `quickjs/`: only the files the build compiles, plus `LICENSE`. `quickjs.c` is patched as
  in `patches/quickjs-v8-math.patch`:
  - the 20 libm-backed `Math` functions call `gai_*` (`gai/gai_math.h`);
  - `js_math_pow`, which also serves the `**` operator, calls `gai_pow`;
  - `Math.hypot` uses V8's algorithm (`src/builtins/math.tq`): convert every argument,
    then Kahan-sum the squares normalized by the largest magnitude.
- `build.rs` compiles `quickjs.c` with `-ffp-contract=off` (hypot's `n * n - comp` must
  stay unfused, as in V8's CSA code; clang's default fuses it and gcc's GNU default is
  `fast`) and `-O2` in every profile. It links in the objects below.
- `gai/`:
  - `v8/src/base/ieee754.cc` and `ieee754.h`: V8 tag 14.6.202.34, unmodified (sha256
    `badd30a2…`). Compiled by clang with `-ffp-contract=on`, as Node's official build is,
    and `-mfma` on x86_64. The other headers under `v8/src/base/` are minimal stand-ins.
    cc puts `CXXFLAGS` after a build's own flags, and the workspace sets
    `CXXFLAGS=-ffp-contract=off` for ThorVG (`.cargo/config.toml`), so `build.rs` also
    appends `-ffp-contract=on` to `CXXFLAGS_<target>`, the variable cc reads last.
  - `gai_math.cc`: `gai_*` wrappers, and `gai_pow`, which keeps V8's `math::pow` special
    cases (`src/numbers/ieee754.cc`: NaN exponent, `(±1) ** ±Infinity`, `y == 2 → x*x`,
    `y == 0.5 → sqrt(x + 0)`) over a correctly rounded pow. It also defines
    `gai_math_description()`, which returns the string gai-js reports in `--version`,
    so gai-js cannot link against a stock rquickjs-sys.
  - `core-math/`: CORE-MATH `src/binary64/pow` at commit
    `708e86ef634414431070336099bb5745f061dc21`, unmodified. `cr_pow` is renamed
    `gai_cr_pow` at compile time.
- `quickjs/` is also patched as in `patches/quickjs-interrupts.patch`, applied after the
  Math patch, so the interrupt handler (gai-js's time limits) stops native code that
  authoring code can keep busy, not just bytecode. The handler is only polled, so no
  result changes before a deadline passes.
  - `libregexp.c`, `libregexp.h`: regexp execution polls `lre_check_timeout()` every
    10,000 backtracking steps and fails with `LRE_RET_TIMEOUT` (QuickJS 2024-01-13's
    change, backported). `quickjs.c` implements it with the interrupt handler and throws
    the uncatchable "interrupted" error, as the interpreter does.
  - `JS_GetPropertyInt64`, `JS_TryGetPropertyInt64` (their non-fast-array paths) and
    `JS_SetPropertyInt64` poll, so `Array.prototype` builtins stop on array-likes such as
    `{ length: 2 ** 53 - 1 }`. The default-order `sort` comparator polls too.
  - `String.prototype.indexOf`, `lastIndexOf`, `includes`, and `string_indexof` (used by
    `split` and `replace`) poll in proportion to the needle's length, since their search
    is O(haystack × needle).
  - Not covered: other native code runs to completion; see the watchdog below.
- `quickjs/` is also patched as in `patches/quickjs-linear.patch`, applied third, so two
  builtins that were quadratic, and never polled, are linear. A one-line batch kept
  `check --source` busy for 269 s against a 120 s limit with the first. Results are
  unchanged; a differential fuzz of upstream against the patch (`normalize` in all four
  forms, `localeCompare`, 120,000 regexps with named groups) matched exactly.
  - `libunicode.c`: `sort_cc` orders a run of combining marks by insertion, which is
    quadratic in the run's length (`normalize` and `localeCompare` of 2^17 marks took
    minutes). A run longer than 32 marks is now sorted by counting, in linear time. Both
    sorts are stable, so the order is the same. The new allocation can fail, so
    `unicode_normalize` frees its buffer on failure (upstream leaked it), and
    `String.prototype.normalize` throws out of memory (`js_realloc_rt` does not throw).
  - `libregexp.c`: a forward named back reference (`\k<b>` before `(?<b>…)`) rescanned
    the whole pattern each time. The capture names are now scanned once per compile,
    by the same walk, and looked up from that list.
- `quickjs/` is also patched as in `patches/quickjs-compile-interrupt.patch`, applied
  fourth, so compiling source and regexps polls the interrupt handler, not only running
  them. Several of these paths are quadratic, so a short source could keep `check` busy
  for seconds to hours past the limit. Polling only; no result changes before a deadline.
  - `quickjs.c`: `next_token` polls once per token, so parsing and name resolution stop
    (each declaration scans the scope's earlier ones, and each reference scans the
    function's declarations and closures — both quadratic). `resolve_variables` polls per
    bytecode; its `fail` label already finishes the copy that keeps atom refcounts right.
  - `libregexp.c`: `lre_compile` polls (`re_poll_timeout`) where compilation is quadratic
    — a disjunction's alternatives, a lookbehind's reversed terms, a character class's
    merged atoms — weighting each poll by the bytecode or class it moved. On a timeout it
    returns `NULL` with `*plen == LRE_RET_TIMEOUT`, and `js_compile_regexp` throws the
    uncatchable "interrupted" error instead of a `SyntaxError`. A back reference in
    `lre_exec` also polls in proportion to the captured length it compares.
- `quickjs/` is also patched as in `patches/quickjs-interrupt-request.patch`, applied
  fifth, so a call stops at its deadline even when it polls rarely. QuickJS calls the
  interrupt handler once per 10,000 polls, and a builtin call is one poll however long it
  runs: a loop of `normalize` on a 4 MiB string ran 742 s past a 200 ms limit.
  - `quickjs.h`, `quickjs.c`: `JS_RequestInterrupt(rt)`, safe from any thread, sets an
    atomic flag on the runtime (relaxed; it is only a hint). `js_poll_interrupts` and
    `js_poll_interrupts_cost` call the handler when the flag is set, as when the counter
    runs out, and `__js_poll_interrupts` clears it. gai-js's timer thread calls it once
    a call's deadline has passed, so the call stops within one builtin call. The extra
    load per poll costs about 1% on the slowest set.
  - `JS_RegExpDelete` (`replace` of a global regexp with `''`) polls before each
    execution: an execution polls only past 10,000 steps, so many short ones never did.
- `quickjs/` is also patched as in `patches/quickjs-module-free.patch`, applied last, so
  freeing a runtime is safe when a module's promise never settled. The callbacks waiting
  on it hold the module, and `JS_FreeContext` freed the module before them, so freeing
  them aborted the process (`js_free_value_rt` aborts on a module) or wrote to freed
  memory. `export const icons = []; await new Promise(() => {})` did it, and so did a time
  limit that stopped the call settling a module's promise. `js_free_module_def` now keeps
  the module's memory while values still name it, and the last of them frees it. (A module
  read from bytecode is dup'ed twice by `JS_ReadModule`, so its few bytes now leak; gai-js
  never reads bytecode.)
- What still runs to completion past a deadline: one call into libbf's BigInt arithmetic
  (`toString` of a 10^8-bit BigInt takes about 5 s, of a 10^9-bit one minutes), and any
  builtin not found yet. `Atomics.wait` cannot block, because rquickjs never enables
  blocking. gai-js's watchdog (`crates/gai-js/src/watchdog.rs`) is the hard stop: a
  one-shot process installs it, and it ends the process with the call's timeout error
  once a call runs `Limits::grace` past its deadline.

Node's `Math.pow` calls the platform `pow`, which differs between macOS arm64, x86_64 and
glibc. The correctly rounded pow gives the same result everywhere. It reproduces every
committed output; of the 18,557 distinct `Math.pow` and `**` inputs captured from the corpus under
Node on macOS arm64, 19 differ from it by one ULP, and none of them reaches an output.

## Requirements

- `ieee754.cc` must be compiled by clang (set `CXX=clang++` where the default is gcc): the
  choice of fused operations is made by clang's frontend.
- x86_64 needs FMA hardware. gai-js checks for it before running any JS.

## Licences

- QuickJS-NG: MIT (`quickjs/LICENSE`); rquickjs-sys: MIT (`LICENSE`, from the rquickjs repository).
- V8: BSD-3-Clause (`gai/v8/LICENSE`); fdlibm: Sun Microsystems notice
  (`gai/v8/LICENSE.fdlibm`, also at the top of `ieee754.cc`).
- CORE-MATH: MIT (`gai/core-math/LICENSE`, also at the top of `pow.c`).
