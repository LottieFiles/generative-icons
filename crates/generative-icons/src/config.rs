//! The project config: `generative-icons.json`.
//!
//! Paths, colors and names must be JSON strings. Unknown keys are kept and ignored.

use crate::error::{CliError, ErrorCode, Result};
use crate::managed::{inside, read_optional};
use crate::project::{Detection, Platform, detect};
use serde_json::{Map, Value, json};
use std::path::Path;

/// Config file of a project.
pub const CONFIG_FILE: &str = "generative-icons.json";
/// Project-relative folder of the lock and the tool's state.
pub const STATE_DIR: &str = ".generative-icons";
/// Project-relative lock path.
pub const LOCK_FILE: &str = ".generative-icons/lock.json";
/// The tool named in generated file headers.
pub const TOOL_NAME: &str = "generative-icons";
/// Skill roots installed by default: Claude Code, then Codex and other agents.
pub const DEFAULT_SKILL_ROOTS: [&str; 2] = [".claude/skills", ".agents/skills"];
/// Default folder of a project's own icon source.
pub const DEFAULT_LOCAL_SOURCE: &str = "generative-icons/icons";

/// A validated project config.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// The config as written, unknown keys included.
    pub raw: Map<String, Value>,
    /// Declared, requested or detected platform.
    pub platform: Platform,
    /// Folder for installed `.lottie` files.
    pub assets: String,
    /// Folder for installed descriptors.
    pub descriptors: String,
    /// URL path the web app serves `assets` from; `None` on native platforms.
    pub public_path: Option<String>,
    /// The `defaults` object as written.
    pub defaults: Value,
    /// `defaults.weight`.
    pub weight: f64,
    /// Folder of the project's own icon source set.
    pub local_source: String,
    /// Skill roots, e.g. `.claude/skills`.
    pub skills: Vec<String>,
}

impl Config {
    /// Loads and validates the project config. `None` when the project has none.
    /// `requested` (`--platform`) overrides a declared platform.
    pub fn load(root: &Path, requested: Option<Platform>) -> Result<Option<Config>> {
        let file = inside(root, CONFIG_FILE)?;
        let Some(bytes) = read_optional(&file)? else { return Ok(None) };
        Self::parse(root, &bytes, requested).map(Some)
    }

    /// Loads the config, failing with `PROJECT_NOT_FOUND` when there is none.
    pub fn require(root: &Path, requested: Option<Platform>) -> Result<Config> {
        Self::load(root, requested)?.ok_or_else(|| {
            CliError::new(ErrorCode::ProjectNotFound, format!("No {CONFIG_FILE} in {}", root.display()))
                .with_hint("Run generative-icons init in the app folder, or pass --cwd <app-folder>")
        })
    }

    /// Validates config bytes.
    pub fn parse(root: &Path, bytes: &[u8], requested: Option<Platform>) -> Result<Config> {
        let file = CONFIG_FILE;
        let invalid = |message: &str| CliError::new(ErrorCode::InvalidConfig, message).with_path(file);
        let value: Value =
            serde_json::from_slice(bytes).map_err(|error| invalid(&format!("Cannot parse {file}: {error}")))?;
        let Value::Object(raw) = value else { return Err(invalid("Unsupported project configuration")) };
        if raw.get("schemaVersion").and_then(Value::as_f64) != Some(1.0) {
            return Err(invalid("Unsupported project configuration"));
        }
        let assets = project_path(root, raw.get("assets"))?;
        let descriptors = project_path(root, raw.get("descriptors"))?;

        let declared = match raw.get("platform") {
            None => None,
            Some(value) => Some(
                value
                    .as_str()
                    .and_then(Platform::parse)
                    .ok_or_else(|| invalid("platform must be web, ios or android"))?,
            ),
        };
        let platform = match requested.or(declared) {
            Some(platform) => platform,
            None => detect(root, None, None)?.platform,
        };

        let public_path = match platform {
            Platform::Web => {
                let path = raw.get("publicPath").and_then(Value::as_str).filter(|path| valid_public_path(path));
                Some(path.ok_or_else(|| invalid("publicPath must be a local URL path"))?.to_string())
            }
            Platform::Ios | Platform::Android => None,
        };

        let defaults = raw.get("defaults").cloned().unwrap_or(Value::Null);
        let weight = valid_defaults(&defaults).ok_or_else(|| invalid("Invalid project defaults"))?;

        let local_source = match raw.get("local") {
            None => DEFAULT_LOCAL_SOURCE.to_string(),
            Some(Value::Object(local)) => match local.get("source") {
                None => DEFAULT_LOCAL_SOURCE.to_string(),
                source => project_path(root, source)?,
            },
            Some(_) => return Err(invalid("local must be an object")),
        };
        let skills = match raw.get("skills") {
            None => DEFAULT_SKILL_ROOTS.iter().map(|s| s.to_string()).collect(),
            Some(Value::Array(items)) => {
                items.iter().map(|item| project_path(root, Some(item))).collect::<Result<_>>()?
            }
            Some(_) => return Err(invalid("skills must be a list of project-relative folders")),
        };

        Ok(Config { raw, platform, assets, descriptors, public_path, defaults, weight, local_source, skills })
    }

    /// The config `init` writes for a detected project, in its documented key order.
    pub fn initial(detection: &Detection, skills: &[String]) -> Value {
        let mut config = Map::new();
        config.insert("schemaVersion".into(), json!(1));
        config.insert("platform".into(), json!(detection.platform.as_str()));
        config.insert("framework".into(), json!(detection.framework));
        config.insert("packageManager".into(), json!(detection.package_manager));
        config.insert("sourceDirectory".into(), json!(detection.source_directory));
        config.insert("assets".into(), json!(detection.assets));
        let descriptors = match (detection.platform, detection.source_directory.as_str()) {
            (Platform::Web, ".") => "generative-icons".to_string(),
            (Platform::Web, source) => format!("{source}/generative-icons"),
            _ => "generative-icons/descriptors".to_string(),
        };
        config.insert("descriptors".into(), json!(descriptors));
        if detection.platform == Platform::Web {
            config.insert("publicPath".into(), json!("/generative-icons"));
        }
        config.insert(
            "defaults".into(),
            json!({ "color": "#008c76", "style": "outline", "weight": 2, "motion": "light" }),
        );
        config.insert("local".into(), json!({ "source": DEFAULT_LOCAL_SOURCE, "license": "UNLICENSED" }));
        config.insert("skills".into(), json!(skills));
        Value::Object(config)
    }

    /// Folder of the managed kit copy: `library/` beside the local source folder.
    pub fn kit_dir(&self) -> String {
        match self.local_source.rsplit_once('/') {
            Some((parent, _)) => format!("{parent}/library"),
            None => "library".to_string(),
        }
    }
}

/// A project-relative path value, validated like `inside()`.
fn project_path(root: &Path, value: Option<&Value>) -> Result<String> {
    match value {
        Some(Value::String(path)) => inside(root, path).map(|_| path.clone()),
        other => {
            let shown = other.map_or_else(|| "undefined".to_string(), gai_core::to_js_json);
            Err(CliError::new(ErrorCode::InvalidPath, format!("Expected a project-relative path: {shown}")))
        }
    }
}

/// `/^\/[a-zA-Z0-9_/-]+$/` and no `..`.
fn valid_public_path(path: &str) -> bool {
    path.len() > 1
        && path.starts_with('/')
        && path.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'/' | b'-'))
        && !path.contains("..")
}

/// Checks `defaults` and returns its weight.
fn valid_defaults(defaults: &Value) -> Option<f64> {
    let color = defaults.get("color")?.as_str()?;
    let hex = color.strip_prefix('#')?;
    let color_ok = hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit());
    let style_ok = matches!(defaults.get("style")?.as_str()?, "outline" | "solid" | "duotone");
    let motion_ok = matches!(defaults.get("motion")?.as_str()?, "light" | "strong");
    let weight = defaults.get("weight")?.as_f64()?;
    (color_ok && style_ok && motion_ok && (1.0..=3.0).contains(&weight)).then_some(weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Result<Config> {
        let dir = tempfile::tempdir().unwrap();
        Config::parse(dir.path(), json.as_bytes(), None)
    }

    const WEB: &str = r##"{"schemaVersion":1,"platform":"web","assets":"public/i","descriptors":"src/i","publicPath":"/i","defaults":{"color":"#00AAbb","style":"solid","weight":1.75,"motion":"strong"},"extra":true}"##;

    #[test]
    fn accepts_a_valid_web_config() {
        let config = parse(WEB).unwrap();
        assert_eq!(config.public_path.as_deref(), Some("/i"));
        assert_eq!(config.weight, 1.75);
        assert_eq!(config.skills, [".claude/skills", ".agents/skills"]);
        assert_eq!(config.kit_dir(), "generative-icons/library");
        assert!(config.raw.contains_key("extra"));
    }

    #[test]
    fn rejects_what_read_config_rejects() {
        let cases = [
            (WEB.replace("\"schemaVersion\":1", "\"schemaVersion\":2"), "Unsupported project configuration"),
            (WEB.replace("\"/i\"", "\"/../i\""), "publicPath must be a local URL path"),
            (WEB.replace("\"/i\"", "[\"/i\"]"), "publicPath must be a local URL path"),
            (WEB.replace("1.75", "3.5"), "Invalid project defaults"),
            (WEB.replace("\"#00AAbb\"", "\"#00AAb\""), "Invalid project defaults"),
            (WEB.replace("\"web\"", "\"tv\""), "platform must be web, ios or android"),
            (WEB.replace("\"src/i\"", "\"../i\""), "Expected a project-relative path: ../i"),
        ];
        for (json, message) in cases {
            assert_eq!(parse(&json).unwrap_err().message, message, "{json}");
        }
    }

    #[test]
    fn native_configs_need_no_public_path() {
        let json = r##"{"schemaVersion":1,"platform":"ios","assets":"App/GenerativeIcons","descriptors":"generative-icons/descriptors","defaults":{"color":"#008c76","style":"outline","weight":2,"motion":"light"},"local":{"source":"icons"},"skills":[".claude/skills"]}"##;
        let config = parse(json).unwrap();
        assert_eq!(config.public_path, None);
        assert_eq!(config.kit_dir(), "library");
        assert_eq!(config.skills, [".claude/skills"]);
    }
}
