//! Command-line surface.

use crate::output::Format;
use crate::project::Platform;
use clap::{Args, Parser, Subcommand, ValueEnum};
use gai_render::{Style, Theme};
use std::path::PathBuf;

/// Find, create and install animated icons in web, iOS and Android apps.
///
/// Prints JSON when stdout is not a terminal or with --json. Exit codes: 0 success,
/// 1 a check failed, 2 usage, 3 project or file conflict, 4 sign-in, 5 network or
/// engine, 6 integrity, 70 internal.
#[derive(Debug, Parser)]
#[command(name = "gen-animated-icons", version, arg_required_else_help = true, max_term_width = 100)]
pub struct Cli {
    /// Options every command accepts.
    #[command(flatten)]
    pub global: Global,
    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Options every command accepts.
#[derive(Debug, Args)]
#[command(next_help_heading = "Global options")]
pub struct Global {
    /// App folder to work in [default: current folder]
    #[arg(long, global = true, value_name = "DIR")]
    pub cwd: Option<PathBuf>,
    /// Print JSON (same as --format json)
    #[arg(long, global = true)]
    pub json: bool,
    /// Output format [default: json unless stdout is a terminal]
    #[arg(long, global = true, value_enum, value_name = "FORMAT")]
    pub format: Option<Format>,
    /// Hide warnings
    #[arg(long, short, global = true)]
    pub quiet: bool,
    /// Log progress and where data comes from, on stderr
    #[arg(long, short, global = true, conflicts_with = "quiet")]
    pub verbose: bool,
    /// Never color stderr labels
    #[arg(long, global = true)]
    pub no_color: bool,
    /// App platform when a folder holds several
    #[arg(long, global = true, value_enum)]
    pub platform: Option<Platform>,
    /// Icon library: a package dist folder, a library checkout or a URL
    #[arg(long, global = true, value_name = "DIR|URL", env = "GEN_ANIMATED_ICONS_LIBRARY")]
    pub library: Option<String>,
    /// Library checkout for maintainer commands and library-mode context
    #[arg(long, global = true, value_name = "DIR", env = "GAI_LIBRARY_ROOT")]
    pub library_root: Option<PathBuf>,
    /// Hosted engine endpoint [default: GAI_ENGINE_URL, then the config's engine.url (LottieFiles engines only), else the LottieFiles engine]
    #[arg(long, global = true, value_name = "URL", env = "GEN_ANIMATED_ICONS_ENGINE_URL")]
    pub engine_url: Option<String>,
    /// Use only local and cached data; never the network
    #[arg(long, global = true)]
    pub offline: bool,
}

/// Commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Set up this app: config, agent skills, the authoring kit and the lock
    Init(InitArgs),
    /// Show the platform, config, library, sign-in and managed files
    Info,
    /// Find library icons by meaning
    Search(SearchArgs),
    /// Install icons: the .lottie asset and its descriptor
    Add(AddArgs),
    /// List managed files and whether they were edited
    Diff,
    /// Uninstall icons whose files are unedited
    Remove(RemoveArgs),
    /// Check the setup: project, skills, kit, lock, sign-in, engine and renderer
    Doctor,
    /// Sign in to the hosted engine that exports icons
    Login(LoginArgs),
    /// Sign out of the hosted engine
    Logout,
    /// Show the engine account and check the session
    Whoami,
    /// Scaffold a new local icon as a spec (optionally from a library icon's spec)
    Create(CreateArgs),
    /// Print the reference packet an AI reads before drawing an icon
    Context(ContextArgs),
    /// Run the offline checks on the project and its local icons
    Check(CheckArgs),
    /// Export, package, verify, preview and install local icons
    Build(BuildArgs),
    /// Render an icon's motions to contact sheets (PNG)
    Preview(PreviewArgs),
    /// Render a v2 icon's review sheet in one engine call, before any build (nothing stored)
    Draft(DraftArgs),
    /// Send local icons for review (or write the bundle with --dry-run)
    Submit(SubmitArgs),
    /// Run an MCP server on stdio for coding agents
    ///
    /// Add it to an agent's MCP settings as {"command": "npx", "args": ["gen-animated-icons",
    /// "mcp"]}. Until the gen-animated-icons npm package is published, use this binary's
    /// absolute path as the command, with args ["mcp"]; `gen-animated-icons init` prints both
    /// entries (`mcp` and `mcpLocal`).
    Mcp,
    /// Maintainer commands for the library checkout
    #[command(subcommand)]
    Library(LibraryCommand),
    /// Verify one action in isolation (used by library verify-visual)
    #[command(name = "__verify-action", hide = true)]
    VerifyAction(VerifyActionArgs),
}

/// Arguments of `init`.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Agents to install the skill for (claude, agents), or none
    #[arg(long, value_enum, value_delimiter = ',', value_name = "AGENTS")]
    pub agents: Option<Vec<Agent>>,
    /// Move a lottie-icons project to gen-animated-icons
    #[arg(long)]
    pub migrate: bool,
    /// Replace an edited kit copy in the local library folder
    #[arg(long)]
    pub update_kit: bool,
    /// Android module that holds the app [default: the application module]
    #[arg(long, value_name = "NAME")]
    pub module: Option<String>,
}

/// Agents whose skill folders `init` writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Agent {
    /// Claude Code: `.claude/skills`
    Claude,
    /// Codex and other agents: `.agents/skills`
    Agents,
    /// No skills
    None,
}

/// Arguments of `search`.
#[derive(Debug, Args)]
pub struct SearchArgs {
    /// What the icon should mean, e.g. "notification bell"
    #[arg(value_name = "MEANING")]
    pub words: Vec<String>,
    /// Most results to show [default: 20 when ranked, all otherwise]
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    pub limit: Option<u32>,
    /// Search only this project's own icons
    #[arg(long, conflicts_with = "library_only")]
    pub local_only: bool,
    /// Search only the released library
    #[arg(long)]
    pub library_only: bool,
    /// ranked: scored with synonyms; all-words: every word as a substring, catalog order
    #[arg(long = "match", value_enum, default_value_t = Match::Ranked)]
    pub mode: Match,
}

/// Search algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Match {
    /// Token scoring with vocabulary expansion.
    Ranked,
    /// The legacy substring filter.
    AllWords,
}

/// Arguments of `add`.
#[derive(Debug, Args)]
pub struct AddArgs {
    /// Icon ids from search, or local icons you built
    #[arg(value_name = "ID", required_unless_present = "installed")]
    pub ids: Vec<String>,
    /// Re-add every installed library icon (upgrade)
    #[arg(long, conflicts_with = "ids")]
    pub installed: bool,
    /// iOS and Android: bake this outline weight (1-3) into the files
    #[arg(long, value_name = "WEIGHT")]
    pub bake_weight: Option<f64>,
    /// Show what would change without writing
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments of `remove`.
#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Installed icon ids
    #[arg(value_name = "ID", required = true)]
    pub ids: Vec<String>,
}

/// Arguments of `login`.
#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Print the sign-in address and read the redirect address from stdin instead of
    /// opening a browser (automatic in CI and over SSH)
    #[arg(long)]
    pub no_browser: bool,
}

/// Arguments of `create`.
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// New icon id, kebab-case, not a library id
    #[arg(value_name = "ID")]
    pub id: String,
    /// Start from this library icon's spec (an icon reanimated as one; `search` marks them
    /// in a checkout), or with --batch from its set's JS source. Needs a library checkout
    #[arg(long, value_name = "LIBRARY_ICON")]
    pub from: Option<String>,
    /// Display label [default: from the id]
    #[arg(long)]
    pub label: Option<String>,
    /// What the motion communicates
    #[arg(long)]
    pub intent: Option<String>,
    /// Reference icons for the saved context packet (up to four, as `context --neighbor`)
    /// [default: the --from icon, else the nearest by label and intent]
    #[arg(long, value_name = "ID")]
    pub neighbor: Vec<String>,
    /// Write a v1 JS batch (`icons/batches/<ID>.mjs`) instead of a spec, for what specs
    /// cannot express yet (continuous loaders, parts resting at partial opacity)
    #[arg(long)]
    pub batch: bool,
    /// A spec icon (`icons/<ID>.icon.json`), the default; kept so older scripts still run
    #[arg(long, hide = true, conflicts_with = "batch")]
    pub spec: bool,
}

/// Arguments of `draft`.
#[derive(Debug, Args)]
pub struct DraftArgs {
    /// A local spec v2 icon (`gen-animated-icons/icons/<ID>.icon.json`)
    #[arg(value_name = "ID")]
    pub id: String,
    /// Write the engine call (creator_exec arguments) to FILE instead of sending it
    #[arg(long, value_name = "FILE")]
    pub emit_call: Option<PathBuf>,
}

/// Arguments of `context`.
#[derive(Debug, Args)]
pub struct ContextArgs {
    /// What the icon means and how it moves
    #[arg(value_name = "QUERY")]
    pub words: Vec<String>,
    /// Read the query from a file (UTF-8)
    #[arg(long, value_name = "FILE")]
    pub query_file: Option<PathBuf>,
    /// Use these icons as references (up to four)
    #[arg(long, value_name = "ID")]
    pub neighbor: Vec<String>,
    /// Print only the ranked candidates
    #[arg(long)]
    pub search: bool,
    /// Candidates with --search [default: 6]
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..))]
    pub limit: Option<u32>,
    /// Print the design prompt that embeds the packet
    #[arg(long)]
    pub prompt: bool,
    /// The user's brief, for the prompt (also the query when none is given)
    #[arg(long, value_name = "FILE")]
    pub brief: Option<PathBuf>,
    /// Extra reference text for the prompt
    #[arg(long, value_name = "FILE")]
    pub context: Vec<PathBuf>,
    /// Also write generation-context.json (and prompt.md) into this new folder
    #[arg(long, value_name = "DIR")]
    pub output: Option<PathBuf>,
}

/// Arguments of `check`.
#[derive(Debug, Args)]
pub struct CheckArgs {
    /// Limit to these local icons
    #[arg(value_name = "ID")]
    pub ids: Vec<String>,
    /// Preflight the local icon source
    #[arg(long)]
    pub source: bool,
    /// Validate installed descriptors
    #[arg(long)]
    pub descriptors: bool,
    /// Validate the config and the lock
    #[arg(long)]
    pub project: bool,
    /// Verify built icons
    #[arg(long)]
    pub native: bool,
}

/// Arguments of `build`.
#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Local icons to build [default: all]
    #[arg(value_name = "ID")]
    pub ids: Vec<String>,
    /// Build every local icon
    #[arg(long, conflicts_with = "ids")]
    pub all: bool,
    /// Build and verify without installing
    #[arg(long)]
    pub no_install: bool,
    /// Engine exports in flight at once (1-4)
    #[arg(long, default_value_t = 2, value_parser = parse_jobs)]
    pub jobs: usize,
    /// Spec v2: write the engine call (creator_exec arguments) to FILE and stop
    #[arg(long, value_name = "FILE", hide = true)]
    pub emit_call: Option<PathBuf>,
    /// Spec v2: finish the build from a bundle the engine exported elsewhere (no engine call)
    #[arg(long, value_name = "FILE", hide = true, conflicts_with = "emit_call")]
    pub from_bundle: Option<PathBuf>,
}

/// Which motions `preview` renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MotionChoice {
    /// Light only.
    Light,
    /// Strong only.
    Strong,
    /// Light and Strong.
    Both,
    /// The reverse (reset) action of each motion of a stateful icon.
    Reverse,
    /// Light, Strong and their reverse actions.
    All,
}

impl MotionChoice {
    /// The option's spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Strong => "strong",
            Self::Both => "both",
            Self::Reverse => "reverse",
            Self::All => "all",
        }
    }
}

/// Arguments of `preview`.
#[derive(Debug, Args)]
pub struct PreviewArgs {
    /// Icon id: a built local icon, an installed icon or a library icon
    #[arg(value_name = "ID")]
    pub id: String,
    /// Motions to render
    #[arg(long, value_enum, default_value_t = MotionChoice::Both)]
    pub motion: MotionChoice,
    /// Render these actions by name instead, e.g. bell-reset (as verify failures name them)
    #[arg(long, value_delimiter = ',', value_name = "NAME", conflicts_with = "motion")]
    pub animation: Option<Vec<String>>,
    /// Column times in seconds, e.g. 0,0.1,0.2 (at most 64)
    #[arg(long, value_delimiter = ',', conflicts_with = "frames", value_name = "SECONDS")]
    pub times: Option<Vec<f64>>,
    /// Evenly spaced columns from the start to the rest frame (reverse actions get their own)
    #[arg(long, value_name = "N", value_parser = parse_columns)]
    pub frames: Option<usize>,
    /// Rows: styles (outline, solid, duotone)
    #[arg(long, value_delimiter = ',', value_parser = parse_style)]
    pub styles: Option<Vec<Style>>,
    /// Rows: outline weights from 1 to 3
    #[arg(long, value_delimiter = ',')]
    pub weights: Option<Vec<f64>>,
    /// Rows: themes (dark, light)
    #[arg(long, value_delimiter = ',', value_parser = parse_theme)]
    pub themes: Option<Vec<Theme>>,
    /// Cell size in pixels
    #[arg(long, default_value_t = 96, value_parser = clap::value_parser!(u32).range(16..=512))]
    pub cell: u32,
    /// Folder for the sheets [default: .gen-animated-icons/previews/ID]
    #[arg(long, value_name = "DIR")]
    pub out: Option<PathBuf>,
}

/// Most columns a preview sheet takes, from `--times` or `--frames`.
pub const MAX_COLUMNS: usize = 64;

/// Engine exports in flight: 1 to 4 (the engine's document limit).
fn parse_jobs(value: &str) -> Result<usize, String> {
    parse_in(value, 1..=4)
}

/// Preview columns: 1 to [`MAX_COLUMNS`].
fn parse_columns(value: &str) -> Result<usize, String> {
    parse_in(value, 1..=MAX_COLUMNS)
}

fn parse_in(value: &str, range: std::ops::RangeInclusive<usize>) -> Result<usize, String> {
    match value.parse::<usize>() {
        Ok(n) if range.contains(&n) => Ok(n),
        _ => Err(format!("expected a whole number from {} to {}", range.start(), range.end())),
    }
}

fn parse_style(value: &str) -> Result<Style, String> {
    match value {
        "outline" => Ok(Style::Outline),
        "solid" => Ok(Style::Solid),
        "duotone" => Ok(Style::Duotone),
        other => Err(format!("unknown style {other}; use outline, solid or duotone")),
    }
}

fn parse_theme(value: &str) -> Result<Theme, String> {
    match value {
        "dark" => Ok(Theme::Dark),
        "light" => Ok(Theme::Light),
        other => Err(format!("unknown theme {other}; use dark or light")),
    }
}

/// Licenses a submission can be offered under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum License {
    /// The library's own license.
    #[value(name = "MIT")]
    Mit,
    /// Creative Commons Attribution 4.0.
    #[value(name = "CC-BY-4.0")]
    CcBy4,
}

impl License {
    /// The SPDX id.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mit => "MIT",
            Self::CcBy4 => "CC-BY-4.0",
        }
    }
}

/// Arguments of `submit`.
#[derive(Debug, Args)]
pub struct SubmitArgs {
    /// Built local icons to submit
    #[arg(value_name = "ID", required = true)]
    pub ids: Vec<String>,
    /// Write the bundle without sending it
    #[arg(long)]
    pub dry_run: bool,
    /// Where to write the bundle (a new file)
    #[arg(long, value_name = "FILE")]
    pub output: Option<PathBuf>,
    /// A note for the reviewers
    #[arg(long)]
    pub message: Option<String>,
    /// License offered for the icons
    #[arg(long, value_enum, default_value_t = License::Mit)]
    pub license: License,
    /// Submit icons whose imports cannot be traced (an import of a computed name, or an
    /// escape in a specifier), with every module their builds loaded, other icons' batches
    /// included
    #[arg(long)]
    pub include_untraced: bool,
    /// Submissions endpoint [default: the LottieFiles review service]
    #[arg(long, value_name = "URL", env = "GEN_ANIMATED_ICONS_SUBMIT_URL")]
    pub endpoint: Option<String>,
}

/// Maintainer commands.
#[derive(Debug, Subcommand)]
pub enum LibraryCommand {
    /// Scaffold an empty source set with the next free number
    NewSet(NewSetArgs),
    /// Screen set sources before export (preflight.mjs)
    Preflight(LibraryPreflightArgs),
    /// Preflight, build and package sets
    Build(LibraryBuildArgs),
    /// Replace library icons with spec v2 builds: spec, assets and set.json entry, then package
    Adopt(LibraryAdoptArgs),
    /// Run the set gates and write their evidence
    Verify(LibraryVerifyArgs),
    /// Run the renderer checks of verify.html and write the browser evidence
    VerifyVisual(VerifyVisualArgs),
    /// Measure source motion and write evidence/motion-measurements.json (sets 22-39)
    Measure(LibraryMeasureArgs),
    /// Package sets (.lottie files, set.json, set zip) or icon-library.zip
    Package(LibraryPackageArgs),
    /// Build the npm packages' dist folders and the search sidecar
    Packages(PackagesArgs),
    /// Add a built, verified set to the catalog
    Register(RegisterArgs),
    /// Build the packages and pack the release tarballs (never publishes)
    Release(ReleaseArgs),
}

/// Arguments of `library new-set`.
#[derive(Debug, Args)]
pub struct NewSetArgs {
    /// Set slug, e.g. landscaping
    pub slug: String,
    /// Display name [default: from the slug]
    #[arg(long)]
    pub name: Option<String>,
    /// Also write the frozen JS builder, verifier, package.py and verify.html
    #[arg(long)]
    pub legacy_templates: bool,
}

/// Arguments of `library preflight`.
#[derive(Debug, Args)]
pub struct LibraryPreflightArgs {
    /// Set folder, number or path
    #[arg(value_name = "SET")]
    pub set: Option<String>,
    /// Every set with definitions.mjs
    #[arg(long)]
    pub all: bool,
}

/// How `library build` exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    /// Sample, export through the hosted engine, package.
    Full,
    /// Reuse the committed natives, then package.
    ReuseNative,
    /// Write scenes only.
    OfflineScenes,
}

/// Arguments of `library build`.
#[derive(Debug, Args)]
pub struct LibraryBuildArgs {
    /// Set folders, numbers or paths
    #[arg(value_name = "SET")]
    pub sets: Vec<String>,
    /// Every set with definitions.mjs
    #[arg(long)]
    pub all: bool,
    /// Rebuild from the natives in native/ instead of exporting
    #[arg(long, conflicts_with = "offline_scenes")]
    pub reuse_native: bool,
    /// Write scenes only; no export, no package
    #[arg(long)]
    pub offline_scenes: bool,
    /// Engine exports in flight at once (1-4)
    #[arg(long, default_value_t = 2, value_parser = parse_jobs)]
    pub jobs: usize,
}

impl LibraryBuildArgs {
    /// The build mode the flags select.
    pub fn mode(&self) -> BuildMode {
        match (self.reuse_native, self.offline_scenes) {
            (true, _) => BuildMode::ReuseNative,
            (_, true) => BuildMode::OfflineScenes,
            _ => BuildMode::Full,
        }
    }
}

/// Arguments of `library adopt`.
#[derive(Args, Debug)]
pub struct LibraryAdoptArgs {
    /// Set folder, number or path
    #[arg(value_name = "SET")]
    pub set: String,
    /// Build folders that `gen-animated-icons build <id>` wrote (`.gen-animated-icons/build/<id>`)
    #[arg(value_name = "BUILD", required = true)]
    pub builds: Vec<std::path::PathBuf>,
}

/// Arguments of `library verify`.
#[derive(Debug, Args)]
pub struct LibraryVerifyArgs {
    /// Set folders, numbers or paths
    #[arg(value_name = "SET")]
    pub sets: Vec<String>,
    /// Every built set, plus the paint audit
    #[arg(long)]
    pub all: bool,
    /// Audit every catalog paint and write its evidence
    #[arg(long)]
    pub paint: bool,
}

/// Arguments of `library measure`.
#[derive(Debug, Args)]
pub struct LibraryMeasureArgs {
    /// Set folders, numbers or paths
    #[arg(value_name = "SET")]
    pub sets: Vec<String>,
    /// Every set with a measurement (22-39)
    #[arg(long)]
    pub all: bool,
}

/// Arguments of `library verify-visual`.
#[derive(Debug, Args)]
pub struct VerifyVisualArgs {
    /// Set folder, number or path
    #[arg(value_name = "SET")]
    pub set: String,
    /// Only these icons; earlier records of the others are kept
    #[arg(long, value_delimiter = ',', value_name = "ICON")]
    pub icons: Vec<String>,
    /// Child processes at once [default: available cores]
    #[arg(long, default_value_t = default_jobs(), hide_default_value = true)]
    pub jobs: usize,
    /// Skip the rest, styles and poses sheets
    #[arg(long)]
    pub no_sheets: bool,
}

fn default_jobs() -> usize {
    std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
}

/// Arguments of `library package`.
#[derive(Debug, Args)]
pub struct LibraryPackageArgs {
    /// Set folders, numbers or paths
    #[arg(value_name = "SET")]
    pub sets: Vec<String>,
    /// Build icon-library.zip and delivery.json instead
    #[arg(long)]
    pub icon_library: bool,
}

/// Arguments of `library packages`.
#[derive(Debug, Args)]
pub struct PackagesArgs {
    /// Folder holding each package's dist folder [default: packages]
    #[arg(long, value_name = "DIR")]
    pub output: Option<PathBuf>,
}

/// Arguments of `library register`.
#[derive(Debug, Args)]
pub struct RegisterArgs {
    /// Set id, e.g. 64-landscaping
    pub set: String,
}

/// Arguments of `library release`.
#[derive(Debug, Args)]
pub struct ReleaseArgs {
    /// New folder for the tarballs [default: dist/packages]
    #[arg(long, value_name = "DIR")]
    pub output: Option<PathBuf>,
    /// Pack a build with uncommitted changes (status prerelease-dev)
    #[arg(long)]
    pub allow_dirty: bool,
}

/// Arguments of `__verify-action`.
#[derive(Debug, Args)]
pub struct VerifyActionArgs {
    /// The set's assets folder (set.json and .lottie files)
    #[arg(long)]
    pub assets: PathBuf,
    /// Two-digit set number
    #[arg(long)]
    pub set: String,
    /// Action name
    #[arg(long)]
    pub action: String,
    /// Folder for run.json and the sheets
    #[arg(long)]
    pub out: PathBuf,
    /// Skip the sheets
    #[arg(long)]
    pub no_sheets: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn definition_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn engine_url_help_states_the_precedence() {
        let help = Cli::command().render_long_help().to_string();
        let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            help.contains("[default: GAI_ENGINE_URL, then the config's engine.url (LottieFiles engines only), else the LottieFiles engine]"),
            "{help}"
        );
    }

    #[test]
    fn global_flags_work_after_the_command() {
        let cli = Cli::try_parse_from(["gen-animated-icons", "search", "bell", "--cwd=app", "--json"]).unwrap();
        assert_eq!(cli.global.cwd, Some(PathBuf::from("app")));
        assert!(cli.global.json);
    }

    #[test]
    fn library_build_modes() {
        let cli = Cli::try_parse_from(["gen-animated-icons", "library", "build", "63", "--reuse-native"]).unwrap();
        let Command::Library(LibraryCommand::Build(args)) = cli.command else { panic!("library build") };
        assert_eq!((args.mode(), args.sets.as_slice()), (BuildMode::ReuseNative, ["63".to_string()].as_slice()));
        assert!(
            Cli::try_parse_from(["gen-animated-icons", "library", "build", "63", "--reuse-native", "--offline-scenes"])
                .is_err()
        );
    }

    #[test]
    fn preview_lists_parse() {
        let cli = Cli::try_parse_from([
            "gen-animated-icons",
            "preview",
            "heart",
            "--times",
            "0,0.5",
            "--styles",
            "solid,outline",
        ])
        .unwrap();
        let Command::Preview(args) = cli.command else { panic!("preview") };
        assert_eq!(args.times, Some(vec![0.0, 0.5]));
        assert_eq!(args.styles, Some(vec![Style::Solid, Style::Outline]));
    }
}
