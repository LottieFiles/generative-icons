use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::from(generative_icons::run(std::env::args_os()))
}
