//! `init`: writes the config, the agent skills, the kit copy, the state `.gitignore`
//! and the lock in one batch. Never touches `AGENTS.md`, `CLAUDE.md` or MCP settings; the
//! MCP settings entry is only printed: `mcp` through npx, and `mcpLocal` through this
//! binary's absolute path, which works without the npm package.

use super::{lock_project, text_list};
use crate::Context;
use crate::cli::{Agent, InitArgs};
use crate::config::{CONFIG_FILE, Config, DEFAULT_SKILL_ROOTS};
use crate::embedded::{SKILL_FILES, SKILL_NAME, STATE_GITIGNORE};
use crate::error::{CliError, Result};
use crate::js::json_file;
use crate::kit;
use crate::lock::Lock;
use crate::managed::{Batch, inside, read_optional};
use crate::output::Report;
use crate::project::{Platform, detect};
use gai_core::sha256_hex;
use serde_json::json;
use std::fmt::Write;

/// Runs `init`.
pub fn run(ctx: &Context, args: &InitArgs) -> Result<Report> {
    let root = ctx.cwd.as_path();
    let existing = Config::load(root, ctx.platform)?;
    if existing.is_some() && args.agents.is_some() {
        return Err(CliError::usage(format!(
            "--agents applies when init creates {CONFIG_FILE}; edit its skills list instead"
        )));
    }
    let detection = detect(root, existing.as_ref().map(|c| c.platform).or(ctx.platform), args.module.as_deref())?;
    let _guard = lock_project(ctx)?;

    // The config to write, if any, and the validated config the batch installs for.
    let (config_bytes, config) = match existing {
        Some(config) => (None, config),
        None => {
            let bytes = json_file(&Config::initial(&detection, &skill_roots(args.agents.as_deref())?));
            let config = Config::parse(root, &bytes, ctx.platform)?;
            (Some(bytes), config)
        }
    };

    let mut next = Lock::load(root)?;
    let mut expected = next.expected();
    next.stamp(None);

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
    installs.push((".generative-icons/.gitignore".to_string(), STATE_GITIGNORE));
    let files = next.object_mut("files");
    for (path, content) in &installs {
        files.insert(path.clone(), json!(sha256_hex(content)));
    }
    for (path, content) in installs {
        batch.write(path, content);
    }
    batch.write(next.path, next.to_bytes());
    let changed = batch.plan(root, &expected)?.apply()?;

    let next_steps = [
        "generative-icons search <meaning>",
        "generative-icons add <id>",
        "generative-icons create <id> (for a missing icon)",
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
        "generative-icons is set up for this {} app ({}, {}).",
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
            "To run this binary rather than the npm package, point the entry at it instead:\n  {}",
            gai_core::to_js_json(&value["mcpLocal"])
        );
    }
    Ok(Report::new(value, text))
}

fn platform_name(platform: Platform) -> &'static str {
    match platform {
        Platform::Web => "web",
        Platform::Ios => "iOS",
        Platform::Android => "Android",
    }
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
