//! `library verify-visual <set>`: the checks `verify.html` ran in a browser, on the
//! players' own renderer. Each action runs in a child process (`__verify-action`), up
//! to `--jobs` at once, so a renderer fault fails one action instead of the run; the
//! results are recorded in set order and saved where `serve-verification.py` saved them.

use super::{resolve_set, sets_dir};
use crate::Context;
use crate::cli::VerifyVisualArgs;
use crate::error::{CliError, ErrorCode, Result};
use crate::output::Report;
use gai_render::{ActionRun, EvidenceWriter, RUNTIME, SetManifest, SetRun, Verdict, iso_date};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::fmt::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::SystemTime;

/// A running child and where it writes.
struct Pending {
    name: String,
    child: std::io::Result<Child>,
    out: tempfile::TempDir,
}

/// Runs `library verify-visual`.
pub fn run(ctx: &Context, repo: &Path, args: &VerifyVisualArgs) -> Result<Report> {
    let render = |error: gai_render::RenderError| CliError::new(ErrorCode::Internal, error.to_string());
    gai_render::probe::require_browser_parity().map_err(|error| {
        render(error).with_hint("Build with CXXFLAGS=-ffp-contract=off (the workspace's .cargo/config.toml sets it)")
    })?;
    let set = resolve_set(repo, &args.set)?;
    let number = &set[..2];
    let assets = repo.join("prototype/icons").join(number).join("assets");
    let evidence = sets_dir(repo).join(&set).join("evidence");
    let set_file = assets.join("set.json");
    let bytes = std::fs::read(&set_file).map_err(|error| CliError::io("read", &set_file, &error))?;
    let manifest = SetManifest::from_slice(&bytes).map_err(render)?;
    let icons = (!args.icons.is_empty()).then_some(args.icons.as_slice());
    let names: Vec<String> = manifest.actions(icons, None).into_iter().map(|meta| meta.name.clone()).collect();
    if names.is_empty() {
        return Err(
            CliError::new(ErrorCode::UnknownIcon, format!("No actions of {set} match")).with_hint("Check --icons")
        );
    }
    let exe = std::env::current_exe()
        .map_err(|error| CliError::new(ErrorCode::Internal, format!("Cannot find this executable: {error}")))?;
    let spawn = |name: &str| -> Result<Pending> {
        let out = tempfile::Builder::new()
            .prefix("gai-verify-")
            .tempdir()
            .map_err(|error| CliError::io("create", Path::new("temp"), &error))?;
        let mut command = Command::new(&exe);
        command
            .args(["__verify-action", "--set", number, "--action", name])
            .arg("--assets")
            .arg(&assets)
            .arg("--out")
            .arg(out.path());
        if args.no_sheets {
            command.arg("--no-sheets");
        }
        let child = command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn();
        Ok(Pending { name: name.to_string(), child, out })
    };

    let mut run = SetRun::new(iso_date(SystemTime::now()), String::new(), RUNTIME.into());
    let mut writer = EvidenceWriter::new(evidence.clone(), Some(assets.clone()));
    let mut written = Vec::new();
    let mut save = |run: &SetRun| -> Result<()> {
        written.extend(writer.save(run, SystemTime::now()).map_err(render)?);
        Ok(())
    };
    let mut queue: VecDeque<Pending> = VecDeque::new();
    let mut next = 0;
    for name in &names {
        while next < names.len() && queue.len() < args.jobs.max(1) {
            queue.push_back(spawn(&names[next])?);
            next += 1;
        }
        run.start_action(name);
        save(&run)?;
        let pending = queue.pop_front().expect("the action's child was spawned");
        let action = finish(pending);
        ctx.out.log(&format!(
            "{name}: {}",
            if matches!(action.outcome, gai_render::Outcome::Passed(_)) { "pass" } else { "fail" }
        ));
        run.record(action);
        save(&run)?;
    }
    if icons.is_some() {
        let previous = evidence.join("browser-verification.json");
        if let Some(report) = std::fs::read(&previous).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()) {
            run.merge_previous(&gai_render::Report::from_value(&report).map_err(render)?, &manifest);
        }
    }
    run.finish();
    save(&run)?;

    let report = run.report();
    let passed = report.result == Some(Verdict::Pass);
    let errors: Vec<Value> = report.errors.clone();
    let value = json!({
        "set": set,
        "result": if passed { "PASS" } else { "FAIL" },
        "renderer": gai_render::RENDERER,
        "actions": names.len(),
        "assets": report.assets.len(),
        "errors": errors,
        "evidence": written.iter().map(|path| super::super::preview::relative(repo, path)).collect::<Vec<_>>(),
    });
    let mut text = String::new();
    let _ = writeln!(
        text,
        "{} {set}: {} actions checked with {}",
        if passed { "PASS" } else { "FAIL" },
        names.len(),
        gai_render::RENDERER
    );
    for error in &errors {
        let _ = writeln!(text, "  {}", error.get("message").and_then(Value::as_str).unwrap_or_default());
    }
    Ok(Report { failed: !passed, ..Report::new(value, text) })
}

/// Waits for a child and reads its result; a child that died is a renderer crash.
fn finish(pending: Pending) -> ActionRun {
    let Pending { name, child, out } = pending;
    let output = match child.and_then(Child::wait_with_output) {
        Ok(output) => output,
        Err(error) => return ActionRun::crashed(&name, &format!("could not run: {error}")),
    };
    if output.status.success() {
        return ActionRun::read_from(out.path()).unwrap_or_else(|error| ActionRun::crashed(&name, &error.to_string()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = output.status.signal() {
            return ActionRun::crashed(&name, &format!("signal {signal}"));
        }
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let detail = stderr.lines().last().unwrap_or("no output");
    ActionRun::crashed(&name, &format!("exit {}: {detail}", output.status.code().unwrap_or(-1)))
}
