# Third-party notices

Original icon artwork, authoring code and documentation are provided under the [MIT License](https://github.com/LottieFiles/lottie-generative-icons/blob/main/LICENSE). Third-party components keep their own licenses.

Components bundled in this repository, its preview site and the `gen-animated-icons` binary:

| Component | Where | License |
| --- | --- | --- |
| dotLottie Web runtime and WASM 0.80.0 | `prototype/vendor/` | MIT, [dotlottie-LICENSE.txt](https://github.com/LottieFiles/lottie-generative-icons/blob/main/prototype/vendor/dotlottie-LICENSE.txt) |
| DM Sans Regular and SemiBold fonts | `prototype/assets/` | SIL Open Font License 1.1, [FONT-LICENSE.txt](https://github.com/LottieFiles/lottie-generative-icons/blob/main/prototype/assets/FONT-LICENSE.txt) |
| QuickJS-NG 0.8.0 JavaScript engine, patched | `vendor/rquickjs-sys/quickjs/` | MIT, [LICENSE](https://github.com/LottieFiles/lottie-generative-icons/blob/main/vendor/rquickjs-sys/quickjs/LICENSE) |
| rquickjs-sys 0.9.0 (QuickJS bindings for rquickjs), patched | `vendor/rquickjs-sys/` | MIT, [LICENSE](https://github.com/LottieFiles/lottie-generative-icons/blob/main/vendor/rquickjs-sys/LICENSE) |
| V8 14.6.202.34 `ieee754.cc` and `ieee754.h` | `vendor/rquickjs-sys/gai/v8/` | BSD-3-Clause, [LICENSE](https://github.com/LottieFiles/lottie-generative-icons/blob/main/vendor/rquickjs-sys/gai/v8/LICENSE) |
| fdlibm, as carried in V8's `ieee754.cc` | `vendor/rquickjs-sys/gai/v8/` | Sun Microsystems notice, [LICENSE.fdlibm](https://github.com/LottieFiles/lottie-generative-icons/blob/main/vendor/rquickjs-sys/gai/v8/LICENSE.fdlibm) |
| CORE-MATH binary64 `pow` (commit 708e86ef) | `vendor/rquickjs-sys/gai/core-math/` | MIT, [LICENSE](https://github.com/LottieFiles/lottie-generative-icons/blob/main/vendor/rquickjs-sys/gai/core-math/LICENSE) |
| svgpath 2.6.0 | `crates/gai-js/js/svgpath.mjs` | MIT, [LICENSE.svgpath](https://github.com/LottieFiles/lottie-generative-icons/blob/main/crates/gai-js/js/LICENSE.svgpath) |
| DejaVu Sans Mono 2.37, printable ASCII subset | `crates/gai-render/fonts/` | Bitstream Vera and Arev font licenses, DejaVu changes in the public domain, [LICENSE-DejaVu.txt](https://github.com/LottieFiles/lottie-generative-icons/blob/main/crates/gai-render/fonts/LICENSE-DejaVu.txt) |

npm dependencies are installed, not bundled. Exact versions are pinned in `package-lock.json` and `motion/package-lock.json`; each package carries its own license.

Rust crates other than `rquickjs-sys` are fetched by Cargo, not vendored. Exact versions are pinned in `Cargo.lock`, and each crate carries its own license. The `gen-animated-icons` binary links them statically, including rquickjs and rquickjs-core 0.9.0 (MIT) and dotlottie-rs with ThorVG (MIT).

Native Lottie export runs as a service on LottieFiles' hosted engine. No engine code is part of this repository or the `gen-animated-icons` binary.

LottieFiles names and logos identify the project. The MIT License does not grant trademark rights.
