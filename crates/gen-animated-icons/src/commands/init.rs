//! `init`: writes the config, the agent skills, the kit copy, the state `.gitignore`
//! and the lock in one batch. Never touches `AGENTS.md`, `CLAUDE.md` or MCP settings; the
//! MCP settings entry is only printed: `mcp` through npx, and `mcpLocal` through this
//! binary's absolute path, which works before the npm package is published.

use super::{lock_project, text_list};
use crate::Context;
use crate::cli::{Agent, InitArgs};
use crate::config::{CONFIG_FILE, Config, DEFAULT_SKILL_ROOTS, LEGACY_CONFIG_FILE, Layout};
use crate::embedded::{LEGACY_SKILL_FILES, SKILL_FILES, SKILL_NAME, STATE_GITIGNORE};
use crate::error::{CliError, ErrorCode, Result};
use crate::js::json_file;
use crate::kit;
use crate::lock::Lock;
use crate::managed::{Batch, inside, prune_empty_dirs, read_optional};
use crate::output::Report;
use crate::project::{Platform, detect};
use gai_core::sha256_hex;
use serde_json::{Value, json};
use std::fmt::Write;

/// Where `lottie-icons init` installed its skill.
const LEGACY_SKILL_DIR: &str = ".agents/skills/lottie-icons";

/// Runs `init`.
pub fn run(ctx: &Context, args: &InitArgs) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let layout = Layout::find(root);
    match (layout, args.migrate) {
        (Some(Layout::Legacy), false) => {
            return Err(CliError::new(
                ErrorCode::Exists,
                format!("This project was set up by lottie-icons ({LEGACY_CONFIG_FILE})"),
            )
            .with_path(LEGACY_CONFIG_FILE)
            .with_hint("Run gen-animated-icons init --migrate to move it to gen-animated-icons.json"));
        }
        (Some(Layout::Current), true) => {
            return Err(CliError::new(ErrorCode::Exists, format!("{CONFIG_FILE} already exists; nothing to migrate"))
                .with_path(CONFIG_FILE));
        }
        (None, true) => return Err(CliError::usage(format!("--migrate needs a project with {LEGACY_CONFIG_FILE}"))),
        _ => {}
    }
    let existing = Config::load(root, ctx.platform)?;
    if existing.is_some() && !args.migrate && args.agents.is_some() {
        return Err(CliError::usage(format!(
            "--agents applies when init creates {CONFIG_FILE}; edit its skills list instead"
        )));
    }
    let detection = detect(root, existing.as_ref().map(|c| c.platform).or(ctx.platform), args.module.as_deref())?;
    let _guard = lock_project(ctx, Layout::Current)?;

    // The config to write, if any, and the validated config the batch installs for.
    let (config_bytes, config) = match existing {
        Some(config) if !args.migrate => (None, config),
        existing => {
            let value = match existing {
                Some(legacy) => migrated_config(legacy),
                None => Config::initial(&detection, &skill_roots(args.agents.as_deref())?),
            };
            let bytes = json_file(&value);
            let config = Config::parse(root, Layout::Current, &bytes, ctx.platform)?;
            (Some(bytes), config)
        }
    };

    let lock = match args.migrate {
        true => Lock::load(root, Layout::Legacy)?,
        false => Lock::load(root, Layout::Current)?,
    };
    let mut expected = lock.expected();
    let mut next = lock.moved_to(Layout::Current);
    next.stamp(Layout::Current, None);

    let mut batch = Batch::new();
    if let Some(bytes) = config_bytes {
        batch.write(CONFIG_FILE, bytes);
    }
    // Managed installs, recorded in the lock so later versions can replace them.
    let mut installs: Vec<(String, &'static str)> = Vec::new();
    let mut skills = Vec::new();
    for skill_root in &config.skills {
        for (relative, content) in SKILL_FILES {
            installs.push((format!("{skill_root}/{SKILL_NAME}/{relative}"), content));
        }
        skills.push(format!("{skill_root}/{SKILL_NAME}/SKILL.md"));
    }
    let kit_dir = config.kit_dir();
    for file in kit::files() {
        let path = format!("{kit_dir}/{}", file.name);
        if args.update_kit {
            // Accept whatever is on disk so an edited copy is replaced.
            if let Some(bytes) = read_optional(&inside(root, &path)?)? {
                expected.insert(path.clone(), sha256_hex(bytes));
            }
        }
        installs.push((path, file.text));
    }
    installs.push((".gen-animated-icons/.gitignore".to_string(), STATE_GITIGNORE));
    let files = next.object_mut("files");
    for (path, content) in &installs {
        files.insert(path.clone(), json!(sha256_hex(content)));
    }
    for (path, content) in installs {
        batch.write(path, content);
    }

    let mut warnings = Vec::new();
    let mut legacy_dirs = Vec::new();
    if args.migrate {
        for relative in [LEGACY_CONFIG_FILE, Layout::Legacy.lock_file()] {
            if let Some(bytes) = read_optional(&inside(root, relative)?)? {
                expected.insert(relative.to_string(), sha256_hex(bytes));
                batch.delete(relative);
            }
        }
        for (relative, sha) in LEGACY_SKILL_FILES {
            let path = format!("{LEGACY_SKILL_DIR}/{relative}");
            match read_optional(&inside(root, &path)?)? {
                Some(bytes) if sha256_hex(&bytes) == sha => {
                    expected.insert(path.clone(), sha.to_string());
                    batch.delete(path);
                }
                Some(_) => warnings.push(format!(
                    "Kept {path}: it differs from the lottie-icons skill; remove it when you no longer need it"
                )),
                None => {}
            }
        }
        legacy_dirs =
            vec![".lottie-icons".to_string(), format!("{LEGACY_SKILL_DIR}/references"), LEGACY_SKILL_DIR.to_string()];
    }
    batch.write(next.path, next.to_bytes());

    let changed = batch.plan(root, &expected)?.apply()?;
    for dir in legacy_dirs {
        prune_empty_dirs(root, &root.join(dir));
    }

    let next_steps = [
        "gen-animated-icons search <meaning>",
        "gen-animated-icons add <id>",
        "gen-animated-icons create <id> (for a missing icon)",
    ];
    let value = json!({
        "platform": detection.platform.as_str(),
        "framework": detection.framework,
        "packageManager": detection.package_manager,
        "sourceDirectory": detection.source_directory,
        "config": CONFIG_FILE,
        "changed": changed,
        "skills": skills,
        "next": next_steps,
        "mcp": crate::mcp::client_config(),
        "mcpLocal": crate::mcp::local_client_config(),
    });
    let mut text = String::new();
    let _ = writeln!(
        text,
        "gen-animated-icons is set up for this {} app ({}, {}).",
        platform_name(detection.platform),
        detection.framework,
        detection.package_manager
    );
    text_list(&mut text, "Changed", &changed);
    text_list(&mut text, "Skills", &skills);
    text_list(&mut text, "Next", &next_steps.map(String::from));
    let _ = writeln!(
        text,
        "Agents with MCP can use it as tools; add this to their MCP settings if you want that:\n  {}",
        gai_core::to_js_json(&value["mcp"])
    );
    if !value["mcpLocal"].is_null() {
        let _ = writeln!(
            text,
            "Until the gen-animated-icons npm package is published, point the entry at this binary instead:\n  {}",
            gai_core::to_js_json(&value["mcpLocal"])
        );
    }
    Ok(Report { warnings, ..Report::new(value, text) })
}

fn platform_name(platform: Platform) -> &'static str {
    match platform {
        Platform::Web => "web",
        Platform::Ios => "iOS",
        Platform::Android => "Android",
    }
}

/// The legacy config with `platform: "web"` after `schemaVersion`; paths are kept.
fn migrated_config(legacy: Config) -> Value {
    let mut raw = legacy.raw;
    let index = raw.keys().position(|key| key == "schemaVersion").map_or(0, |i| i + 1);
    raw.shift_insert(index, "platform".into(), json!("web"));
    Value::Object(raw)
}

/// Skill roots for `--agents` (default: all).
fn skill_roots(agents: Option<&[Agent]>) -> Result<Vec<String>> {
    let Some(agents) = agents else { return Ok(DEFAULT_SKILL_ROOTS.map(String::from).to_vec()) };
    match agents {
        [Agent::None] => return Ok(Vec::new()),
        _ if agents.contains(&Agent::None) => {
            return Err(CliError::usage("--agents none cannot be combined with other agents"));
        }
        _ => {}
    }
    let mut roots = Vec::new();
    for agent in agents {
        let root = match agent {
            Agent::Claude => DEFAULT_SKILL_ROOTS[0],
            Agent::Agents => DEFAULT_SKILL_ROOTS[1],
            Agent::None => continue,
        };
        if !roots.iter().any(|r| r == root) {
            roots.push(root.to_string());
        }
    }
    Ok(roots)
}
