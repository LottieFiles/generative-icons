//! `library new-set <slug>`: scaffolds an empty source set (`icon-set.py new`): the same
//! validation, numbering and all-or-nothing staging. The default templates are the
//! files a Rust-built set needs; `--legacy-templates` writes exactly what `icon-set.py`
//! wrote, including the frozen `build.mjs`, `verify.mjs`, `package.py` and `verify.html`.

use super::sets_dir;
use crate::cli::NewSetArgs;
use crate::error::{CliError, ErrorCode, Result};
use crate::local_set::is_icon_name;
use crate::managed::write_inside;
use crate::output::Report;
use serde_json::json;
use std::path::{Path, PathBuf};

const DEFINITIONS: &str = "// Author named vector parts and custom Light/Strong motion here.\n// An empty set intentionally fails preflight; scaffolding is not a design.\nexport const icons = [];\n";
const DESIGN_REVIEW: &str = "# Design and review record\n\nRecord meaning, reference IDs, named parts and pivots, Light/Strong timing, protected gaps, observed failures and corrections. Keep source checks, native rendering, agent visual review and user approval separate. Link final evidence to current package hashes.\n";
const PACKAGE_PY: &str = "from pathlib import Path\nimport subprocess,sys\nroot=Path(__file__).resolve().parent\nsubprocess.run([sys.executable,str(root.parent/\"library/package-set.py\"),str(root)],check=True)\n";
/// The renderer lines `icon-set.py` removes from the baseline page.
const COUNTER_START: &str = "                  if (size >= 32 && size <= 48 && style !== 'duotone')";
const COUNTER_END: &str = "                  const b = pixels(buffer, size);";

/// `(path relative to the set folder, contents)` of each scaffolded file.
type Files = Vec<(String, String)>;

/// Runs `library new-set`.
pub fn run(repo: &Path, args: &NewSetArgs) -> Result<Report> {
    let slug = args.slug.as_str();
    let invalid = |message: &str| CliError::usage(message);
    if !is_icon_name(slug) {
        return Err(invalid("Use a lowercase set slug, such as landscaping. Do not include a path or number."));
    }
    // Python `name or default`: an empty --name falls back too.
    let name = match args.name.as_deref().filter(|name| !name.is_empty()) {
        Some(name) => name.to_string(),
        None => capitalize(&slug.replace('-', " ")),
    };
    if name.chars().all(py_isspace) || name.contains(['\n', '\r']) {
        return Err(invalid("The display name must be a nonempty single line."));
    }
    let sets = sets_dir(repo);
    let pages = repo.join("prototype/icons");
    let folders = super::set_folders(repo)?;
    if folders.iter().any(|folder| folder[3..] == *slug) {
        return Err(CliError::new(ErrorCode::Exists, format!("A source set named {slug} already exists.")));
    }
    let mut used: Vec<u32> = folders.iter().filter_map(|folder| folder[..2].parse().ok()).collect();
    for entry in std::fs::read_dir(&pages).map_err(|error| CliError::io("read", &pages, &error))?.flatten() {
        let page = entry.file_name().to_string_lossy().into_owned();
        if page.len() == 2 && page.bytes().all(|b| b.is_ascii_digit()) {
            used.push(page.parse().unwrap_or(0));
        }
    }
    let number = used.into_iter().max().unwrap_or(0) + 1;
    if number > 99 {
        return Err(invalid("The current asset layout supports two-digit set numbers only."));
    }
    let set_id = format!("{number:02}-{slug}");
    let (source_files, page) = match args.legacy_templates {
        true => legacy_files(repo, &set_id, &name, slug, number)?,
        false => (current_files(&set_id, &name, slug), None),
    };

    // Stage both trees before moving either into place; never replace a set.
    let staging = tempfile::Builder::new()
        .prefix(".new-set-")
        .tempdir_in(&sets)
        .map_err(|error| CliError::io("create", &sets, &error))?;
    let write = |relative: &str, content: &str| write_inside(staging.path(), relative, content.as_bytes()).map(drop);
    for (relative, content) in &source_files {
        write(&format!("source/{relative}"), content)?;
    }
    if let Some(page) = &page {
        write("page/verify.html", page)?;
    }
    let (source, page_dir) = (sets.join(&set_id), pages.join(format!("{number:02}")));
    if source.exists() || (page.is_some() && page_dir.exists()) {
        return Err(CliError::new(
            ErrorCode::FileChanged,
            "The selected set number was taken; retry with the current checkout.",
        ));
    }
    let moved =
        |from: PathBuf, to: &Path| std::fs::rename(&from, to).map_err(|error| CliError::io("move", &from, &error));
    moved(staging.path().join("source"), &source)?;
    if page.is_some() {
        moved(staging.path().join("page"), &page_dir)?;
    }

    let files: Vec<String> =
        source_files.iter().map(|(relative, _)| format!("motion/icon-sets/{set_id}/{relative}")).collect();
    let next = [
        format!("author motion/icon-sets/{set_id}/definitions.mjs"),
        format!("gen-animated-icons library build {set_id}"),
        format!("gen-animated-icons library verify {set_id}"),
        format!("gen-animated-icons library register {set_id}"),
    ];
    let value = json!({
        "setId": set_id,
        "source": format!("motion/icon-sets/{set_id}"),
        "page": page.as_ref().map(|_| format!("prototype/icons/{number:02}/verify.html")),
        "files": files,
        "next": next,
    });
    let text = format!(
        "Created motion/icon-sets/{set_id}. Author definitions.mjs, then build and verify.\nRegister after a successful build: gen-animated-icons library register {set_id}\nThe catalog is unchanged. Scaffolding does not generate or approve artwork.\n"
    );
    Ok(Report::new(value, text))
}

/// `set-config.json` (PY-JSON, ensure_ascii).
fn set_config(set_id: &str, name: &str, shared: &[&str]) -> String {
    let config = json!({ "id": set_id, "name": name, "category": name, "designReview": format!("{set_id}/evidence"), "sharedSources": shared });
    gai_core::to_py_json(&config, Some(2), true) + "\n"
}

fn download_readme(name: &str) -> String {
    format!(
        "{name}\n\nNative dotLottie files with Light/Strong motion and Outline/Solid/Duotone styles. Editable source is under source/. Original work is MIT licensed; see LICENSE and THIRD_PARTY_NOTICES.md.\n"
    )
}

/// The files a set built by this tool starts with.
fn current_files(set_id: &str, name: &str, slug: &str) -> Files {
    let readme = format!(
        "# {name}

Author custom artwork in definitions.mjs using the family skill. No artwork is generated by the scaffold. Once the set is registered, replace this README with its icon list (see any registered set).

From the repository root:

```sh
gen-animated-icons doctor
gen-animated-icons library build {set_id}
gen-animated-icons library verify {set_id}
gen-animated-icons library verify-visual {set_id}
gen-animated-icons library register {set_id}
```

Build before registration. Inspect the gallery at /icons/01/#category-{slug}.
Add drawing-specific rules (verify.rules.json) and clearance contracts (geometry-contract.json); generic checks cannot establish visual quality.
"
    );
    vec![
        ("set-config.json".into(), set_config(set_id, name, &["drawing.mjs", "direction.mjs", "ai-authoring.mjs"])),
        ("definitions.mjs".into(), DEFINITIONS.into()),
        ("batches/.gitkeep".into(), String::new()),
        ("DOWNLOAD_README.txt".into(), download_readme(name)),
        ("evidence/design-review.md".into(), DESIGN_REVIEW.into()),
        ("README.md".into(), readme),
    ]
}

/// Exactly the files `icon-set.py new` wrote, from the set-34 baseline, plus its page.
fn legacy_files(repo: &Path, set_id: &str, name: &str, slug: &str, number: u32) -> Result<(Files, Option<String>)> {
    let read = |path: PathBuf| std::fs::read_to_string(&path).map_err(|error| CliError::io("read", &path, &error));
    let baseline = sets_dir(repo).join("34-scheduling-time");
    let builder = read(baseline.join("build.mjs"))?;
    let verifier = read(baseline.join("verify.mjs"))?.replace("Stale scheduling-time builder", "Stale set builder");
    let renderer = read(repo.join("prototype/icons/34/verify.html"))?
        .replace("Scheduling & time tools", &html_escape(name))
        .replace("set:\"34\"", &format!("set:\"{number:02}\""))
        .replace("location.port===\"4191\"", "new URLSearchParams(location.search).get(\"save\")===\"1\"");
    let not_found = || CliError::new(ErrorCode::InvalidConfig, "substring not found");
    let start = renderer.find(COUNTER_START).ok_or_else(not_found)?;
    let end = renderer[start..].find(COUNTER_END).ok_or_else(not_found)? + start;
    let renderer = format!("{}{}", &renderer[..start], &renderer[end..]);
    let readme = format!(
        "# {name}

Author custom artwork in definitions.mjs using the family skill. No artwork is generated by the scaffold. Once the set is registered, replace this README with its icon list (see any registered set).

From the repository root:

```sh
npm run doctor
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/{set_id}
node motion/icon-sets/{set_id}/verify.mjs
npm run icons:register -- {set_id}
npm run dev
```

Build before registration. Inspect the gallery at /icons/01/#category-{slug}.
For native frame checks, run the verification server described in docs/verification.md and open /icons/{number:02}/verify.html?save=1. Add drawing-specific contact and clearance checks; generic checks cannot establish visual quality.
"
    );
    let files = vec![
        ("build.mjs".into(), builder),
        ("verify.mjs".into(), verifier),
        ("set-config.json".into(), set_config(set_id, name, &["drawing.mjs", "direction.mjs"])),
        ("definitions.mjs".into(), DEFINITIONS.into()),
        ("batches/.gitkeep".into(), String::new()),
        ("package.py".into(), PACKAGE_PY.into()),
        ("DOWNLOAD_README.txt".into(), download_readme(name)),
        ("evidence/design-review.md".into(), DESIGN_REVIEW.into()),
        ("README.md".into(), readme),
    ];
    Ok((files, Some(renderer)))
}

/// Python `str.capitalize()`: first character upper case, the rest lower case.
fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars.flat_map(char::to_lowercase)).collect(),
        None => String::new(),
    }
}

/// Python `str.isspace` for one character: Unicode White_Space plus U+001C–U+001F.
fn py_isspace(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// Python `html.escape(text, quote=True)`.
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_string_semantics() {
        assert_eq!(capitalize("food drink"), "Food drink");
        assert_eq!(capitalize("q a"), "Q a");
        assert!(py_isspace('\u{1f}') && py_isspace('\u{85}') && !py_isspace('x'));
        assert_eq!(html_escape("Q&A <\"tools\">'"), "Q&amp;A &lt;&quot;tools&quot;&gt;&#x27;");
    }

    #[test]
    fn config_is_python_json() {
        assert_eq!(
            set_config("35-landscaping", "Landscaping", &["drawing.mjs", "direction.mjs"]),
            "{\n  \"id\": \"35-landscaping\",\n  \"name\": \"Landscaping\",\n  \"category\": \"Landscaping\",\n  \"designReview\": \"35-landscaping/evidence\",\n  \"sharedSources\": [\n    \"drawing.mjs\",\n    \"direction.mjs\"\n  ]\n}\n"
        );
    }
}
