//! `gen-animated-icons`: find, create, build and install animated icons in web, iOS and
//! Android apps, and maintain the icon library.
//!
//! [`run`] is the whole command line. Each command returns a [`output::Report`] (a
//! JSON value plus a text rendering) or a [`error::CliError`] with a stable code, so
//! other front ends can call commands directly through [`execute`].

pub mod cli;
pub mod commands;
pub mod config;
pub mod descriptor;
pub mod embedded;
pub mod engine;
pub mod error;
pub mod js;
pub mod kit;
pub mod library;
pub mod local_set;
pub mod lock;
pub mod managed;
pub mod math;
pub mod mcp;
pub mod output;
pub mod project;
pub mod scene_checks;
pub mod search;
pub mod weight;

use clap::Parser;
use clap::error::{ContextKind, ContextValue, ErrorKind};
use cli::{Cli, Command};
use error::{CliError, ErrorCode, Result};
use library::LibrarySource;
use output::{Format, Output, Report};
use project::Platform;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Settings shared by every command.
#[derive(Debug, Clone)]
pub struct Context {
    /// The app folder, canonicalized.
    pub cwd: PathBuf,
    /// `--platform`.
    pub platform: Option<Platform>,
    /// `--library` or `GEN_ANIMATED_ICONS_LIBRARY`.
    pub library: Option<String>,
    /// `--offline`.
    pub offline: bool,
    /// `--library-root` or `GAI_LIBRARY_ROOT`.
    pub library_root: Option<PathBuf>,
    /// `--engine-url` or `GEN_ANIMATED_ICONS_ENGINE_URL`.
    pub engine_url: Option<String>,
    /// Where logs go.
    pub out: Output,
}

impl Context {
    /// Settings for commands run in `cwd` with nothing else given.
    pub fn new(cwd: PathBuf, out: Output) -> Context {
        Context { cwd, platform: None, library: None, offline: false, library_root: None, engine_url: None, out }
    }

    /// The library source for this run: `--library`, else the `--library-root`
    /// checkout, else the nearest library above the working folder or the hosted one.
    pub fn library(&self) -> Result<Box<dyn LibrarySource>> {
        let source: Box<dyn LibrarySource> = match (&self.library, &self.library_root) {
            (None, Some(root)) => Box::new(library::LocalDir::open(&self.cwd.join(root))?),
            (explicit, _) => library::resolve(explicit.as_deref(), &self.cwd, self.offline)?,
        };
        self.out.log(&format!("library: {}", source.location()));
        Ok(source)
    }

    /// The checkout for library mode: `--library-root`, else the nearest folder at or
    /// above the working one that holds the catalog and the family contract.
    pub fn checkout(&self) -> Result<Option<PathBuf>> {
        match &self.library_root {
            Some(root) => {
                let root = self.cwd.join(root);
                if !library::is_checkout(&root) {
                    return Err(CliError::new(
                        ErrorCode::ProjectNotFound,
                        format!("{} is not a library checkout", root.display()),
                    )
                    .with_hint("Point --library-root at the folder holding prototype/icons and motion/icon-sets"));
                }
                Ok(Some(root.canonicalize().map_err(|error| CliError::io("open", &root, &error))?))
            }
            None => Ok(self.cwd.ancestors().find(|dir| library::is_checkout(dir)).map(Path::to_path_buf)),
        }
    }

    /// The library repository for maintainer commands: `--library-root`, else the nearest
    /// folder at or above the working one that holds `motion/icon-sets`.
    pub fn library_repo(&self) -> Result<PathBuf> {
        let found = match &self.library_root {
            Some(root) => Some(self.cwd.join(root)).filter(|root| root.join("motion/icon-sets").is_dir()),
            None => self.cwd.ancestors().find(|dir| dir.join("motion/icon-sets").is_dir()).map(Path::to_path_buf),
        };
        let root = found.ok_or_else(|| {
            CliError::new(ErrorCode::ProjectNotFound, "Not in a library checkout (no motion/icon-sets)")
                .with_hint("Run this in the library repository, or pass --library-root <checkout>")
        })?;
        root.canonicalize().map_err(|error| CliError::io("open", &root, &error))
    }
}

/// Runs the command line and returns the process exit status.
pub fn run(args: impl IntoIterator<Item = OsString>) -> u8 {
    let args: Vec<OsString> = args.into_iter().collect();
    let cli = match Cli::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(error) => {
            let raw: Vec<String> = args.iter().skip(1).map(|a| a.to_string_lossy().into_owned()).collect();
            return usage_error(error, Format::from_raw_args(&raw));
        }
    };
    let global = &cli.global;
    let out = Output::new(Format::choose(global.json, global.format), global.quiet, global.verbose, !global.no_color);
    // The MCP server outlives any one call, so it runs each one as a command of its own.
    if !matches!(cli.command, Command::Mcp) {
        protect(&out);
    }
    let result = open_folder(global.cwd.as_deref()).and_then(|cwd| {
        let ctx = Context {
            cwd,
            platform: global.platform,
            library: global.library.clone(),
            offline: global.offline,
            library_root: global.library_root.clone(),
            engine_url: global.engine_url.clone(),
            out: out.clone(),
        };
        if matches!(cli.command, Command::Mcp) {
            return mcp::serve(&ctx).map(|()| None);
        }
        execute(&ctx, &cli.command).map(Some)
    });
    match result {
        Ok(Some(report)) => out.report(&report),
        Ok(None) => 0,
        Err(error) => {
            out.error(&error);
            error.code.exit_code()
        }
    }
}

/// Runs one parsed command.
pub fn execute(ctx: &Context, command: &Command) -> Result<Report> {
    match command {
        Command::Init(args) => commands::init::run(ctx, args),
        Command::Info => commands::info::run(ctx),
        Command::Search(args) => commands::search::run(ctx, args),
        Command::Add(args) => commands::add::run(ctx, args),
        Command::Diff => commands::diff::run(ctx),
        Command::Remove(args) => commands::remove::run(ctx, args),
        Command::Doctor => commands::doctor::run(ctx),
        Command::Login(args) => commands::login::login(ctx, args),
        Command::Logout => commands::login::logout(ctx),
        Command::Whoami => commands::login::whoami(ctx),
        Command::Create(args) => commands::create::run(ctx, args),
        Command::Context(args) => commands::context::run(ctx, args),
        Command::Check(args) => commands::check::run(ctx, args),
        Command::Build(args) => commands::build::run(ctx, args),
        Command::Preview(args) => commands::preview::run(ctx, args),
        Command::Draft(args) => commands::v2::draft(ctx, args),
        Command::Submit(args) => commands::submit::run(ctx, args),
        Command::Mcp => Err(CliError::usage("mcp serves stdio; run gen-animated-icons mcp")),
        Command::Library(command) => commands::library::run(ctx, command),
        Command::VerifyAction(args) => commands::verify_action::run(args),
    }
}

/// Ends this process when icon authoring code runs past its time limit in native code
/// that never checks it (gai-js's [`watchdog`](gai_js::watchdog)): the call's
/// `SOURCE_ERROR` is printed as any error is, and the process exits with its status. Run
/// once per one-shot command; a process that must survive a stuck call (the MCP server)
/// runs its calls as commands of their own instead.
pub fn protect(out: &Output) {
    let report = out.clone();
    let installed = gai_js::watchdog::install(move |error| {
        let error = commands::check::source_error(error);
        report.error(&error);
        error.code.exit_code()
    });
    if let Err(error) = installed {
        // Calls still stop at their time limit; only native code that never checks it runs on.
        out.warning(&format!("{error}; icon code stuck in native code will not be stopped"));
    }
}

/// Canonicalizes `--cwd` (or the current folder); symlinked roots are allowed.
fn open_folder(cwd: Option<&Path>) -> Result<PathBuf> {
    let not_found = |path: &Path, reason: String| {
        CliError::new(ErrorCode::ProjectNotFound, format!("Cannot open {}: {reason}", path.display()))
            .with_path(path.display().to_string())
    };
    let current = std::env::current_dir().map_err(|e| not_found(Path::new("."), e.to_string()))?;
    let path = cwd.map_or_else(|| current.clone(), |cwd| current.join(cwd));
    let path = path.canonicalize().map_err(|e| not_found(&path, e.to_string()))?;
    if !path.is_dir() {
        return Err(not_found(&path, "not a folder".into()));
    }
    Ok(path)
}

/// Prints a parse failure (help and version are not failures) and returns its status.
fn usage_error(error: clap::Error, format: Format) -> u8 {
    if matches!(error.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
        let _ = error.print();
        return 0;
    }
    if format == Format::Text {
        let _ = error.print();
        return ErrorCode::Usage.exit_code();
    }
    let missing_value =
        matches!(error.get(ContextKind::InvalidValue), Some(ContextValue::String(value)) if value.is_empty());
    let code = match error.kind() {
        ErrorKind::InvalidSubcommand => ErrorCode::UnknownCommand,
        ErrorKind::InvalidValue if missing_value => ErrorCode::MissingValue,
        _ => ErrorCode::Usage,
    };
    let message = match error.kind() {
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => "A command is required".to_string(),
        _ => {
            let rendered = error.render().to_string();
            let first = rendered.lines().next().unwrap_or_default();
            first.strip_prefix("error: ").unwrap_or(first).to_string()
        }
    };
    let error = CliError::new(code, message).with_hint("Run gen-animated-icons --help");
    Output::new(Format::Json, false, false, false).error(&error);
    code.exit_code()
}
