//! Parity with the Python/JS packaging pipeline. Oracle tests read the committed outputs of
//! the library checkout and skip when it is missing; whole-catalog and differential tests
//! are `#[ignore]` (run with `cargo test -p gai-package -- --ignored`). Nothing is written
//! into the checkout: every output goes to a temporary folder.

mod common;
mod library;
mod lottie;
mod packages;
mod set_zip;
