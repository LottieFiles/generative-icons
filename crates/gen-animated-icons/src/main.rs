use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::from(gen_animated_icons::run(std::env::args_os()))
}
