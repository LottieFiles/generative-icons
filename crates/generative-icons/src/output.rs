//! Where results, warnings and errors go.
//!
//! JSON (`JSON.stringify(value, null, 2)` plus one newline) goes to stdout when `--json`
//! is given or stdout is not a terminal; people at a terminal get text. Warnings, logs
//! and errors always go to stderr, so stdout stays parseable.

use crate::error::CliError;
use serde_json::{Value, json};
use std::io::{IsTerminal, Write};

/// Output format for results and errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    /// Machine-readable JSON.
    Json,
    /// Human-readable text.
    Text,
}

impl Format {
    /// `--json` wins, then `--format`, then JSON unless stdout is a terminal.
    pub fn choose(json: bool, format: Option<Format>) -> Format {
        match (json, format) {
            (true, _) => Format::Json,
            (false, Some(format)) => format,
            (false, None) if std::io::stdout().is_terminal() => Format::Text,
            (false, None) => Format::Json,
        }
    }

    /// The format requested by raw arguments, for errors raised before parsing succeeds.
    pub fn from_raw_args(args: &[String]) -> Format {
        let mut format = None;
        let mut json = false;
        let mut iter = args.iter().peekable();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--" => break,
                "--json" => json = true,
                "--format" => format = iter.peek().and_then(|value| parse_format(value)),
                other => {
                    if let Some(value) = other.strip_prefix("--format=") {
                        format = parse_format(value);
                    }
                }
            }
        }
        Format::choose(json, format)
    }
}

fn parse_format(value: &str) -> Option<Format> {
    match value {
        "json" => Some(Format::Json),
        "text" => Some(Format::Text),
        _ => None,
    }
}

/// A finished command: its JSON value, a text rendering, and whether a gate failed.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// The JSON result printed in JSON mode.
    pub value: Value,
    /// The rendering printed in text mode.
    pub text: String,
    /// A gate ran and failed; the process exits 1 after printing the report.
    pub failed: bool,
    /// Non-fatal findings, printed to stderr.
    pub warnings: Vec<String>,
}

impl Report {
    /// A successful report.
    pub fn new(value: Value, text: String) -> Self {
        Self { value, text, failed: false, warnings: Vec::new() }
    }
}

/// Prints reports, warnings, logs and errors in the chosen format.
#[derive(Debug, Clone)]
pub struct Output {
    format: Format,
    quiet: bool,
    verbose: bool,
    color: bool,
}

impl Output {
    /// `color` applies to text-mode stderr labels and is dropped when stderr is not a terminal.
    pub fn new(format: Format, quiet: bool, verbose: bool, color: bool) -> Self {
        let color = color && std::env::var_os("NO_COLOR").is_none() && std::io::stderr().is_terminal();
        Self { format, quiet, verbose, color }
    }

    /// A progress or diagnostic line, shown with `--verbose` only.
    pub fn log(&self, message: &str) {
        if self.verbose {
            self.stderr_line("note", "2", message);
        }
    }

    /// A warning line on stderr, hidden by `--quiet`.
    pub fn warning(&self, message: &str) {
        if !self.quiet {
            self.stderr_line("warning", "33", message);
        }
    }

    /// Prints a report and returns the process exit status (0, or 1 when a gate failed).
    pub fn report(&self, report: &Report) -> u8 {
        for warning in &report.warnings {
            self.warning(warning);
        }
        let mut stdout = std::io::stdout().lock();
        let written = match self.format {
            Format::Json => writeln!(stdout, "{}", gai_core::to_js_json_pretty(&report.value, 2)),
            Format::Text if report.text.is_empty() => Ok(()),
            Format::Text => writeln!(stdout, "{}", report.text.trim_end()),
        };
        // A closed pipe (`| head`) is not an error worth reporting.
        drop(written);
        u8::from(report.failed)
    }

    /// Prints an error to stderr: one compact JSON line, or `error:` and `hint:` lines.
    pub fn error(&self, error: &CliError) {
        match self.format {
            Format::Json => {
                let _ = writeln!(std::io::stderr().lock(), "{}", gai_core::to_js_json(&error.to_json()));
            }
            Format::Text => {
                self.stderr_line("error", "31", &error.message);
                if let Some(hint) = &error.hint {
                    self.stderr_line("hint", "2", hint);
                }
            }
        }
    }

    fn stderr_line(&self, label: &str, ansi: &str, message: &str) {
        let mut stderr = std::io::stderr().lock();
        let _ = match self.format {
            Format::Json => writeln!(stderr, "{}", gai_core::to_js_json(&json!({ label: message }))),
            Format::Text if self.color => writeln!(stderr, "\x1b[{ansi}m{label}:\x1b[0m {message}"),
            Format::Text => writeln!(stderr, "{label}: {message}"),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn raw_args_pick_the_requested_format() {
        assert_eq!(Format::from_raw_args(&args(&["search", "--json"])), Format::Json);
        assert_eq!(Format::from_raw_args(&args(&["--format", "text", "x"])), Format::Text);
        assert_eq!(Format::from_raw_args(&args(&["--format=text"])), Format::Text);
        assert_eq!(Format::from_raw_args(&args(&["--format=text", "--json"])), Format::Json);
    }
}
