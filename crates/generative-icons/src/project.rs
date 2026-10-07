//! Project detection for web, iOS and Android app folders.
//!
//! Web detection ports `inspectProject` from `packages/cli/src/project.mjs` exactly,
//! plus `bun.lockb` and `pnpm-workspace.yaml`. iOS and Android are detected from the
//! Xcode and Gradle files at the folder root.

use crate::error::{CliError, ErrorCode, Result};
use crate::js::truthy;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Supported app platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Platform {
    /// A JavaScript app with `package.json`.
    Web,
    /// An Xcode project or Swift package.
    Ios,
    /// A Gradle project with an Android app module.
    Android,
}

impl Platform {
    /// The config and JSON spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Ios => "ios",
            Self::Android => "android",
        }
    }

    /// Parses the config spelling.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "web" => Some(Self::Web),
            "ios" => Some(Self::Ios),
            "android" => Some(Self::Android),
            _ => None,
        }
    }

    /// Native platforms play `.lottie` files with the dotLottie players directly.
    pub fn is_native(self) -> bool {
        self != Self::Web
    }
}

/// What detection found in an app folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detection {
    /// The platform.
    pub platform: Platform,
    /// `next`, `react`, `vue`, `svelte`, `vanilla`; `swiftui`, `uikit`; `compose`, `views`.
    pub framework: String,
    /// `npm`, `pnpm`, `yarn`, `bun`; `spm`, `cocoapods`; `gradle`.
    pub package_manager: String,
    /// Where app sources live, relative to the root (`src`, `.`, `<Target>`, `app/src/main`).
    pub source_directory: String,
    /// Default folder for installed `.lottie` files.
    pub assets: String,
}

/// Detects the app in `root`. `requested` (from `--platform` or the config) picks one
/// platform when several match; `module` picks the Android app module.
pub fn detect(root: &Path, requested: Option<Platform>, module: Option<&str>) -> Result<Detection> {
    let web = web_signal(root)?;
    let ios = root_has(root, |name, is_dir| {
        (is_dir && (name.ends_with(".xcodeproj") || name.ends_with(".xcworkspace")))
            || (!is_dir && name == "Package.swift")
    });
    let android = android_modules(root);
    let platform = match requested {
        Some(platform) => platform,
        None => {
            let found: Vec<Platform> =
                [(web.is_some(), Platform::Web), (ios, Platform::Ios), (!android.is_empty(), Platform::Android)]
                    .into_iter()
                    .filter_map(|(present, platform)| present.then_some(platform))
                    .collect();
            match found.as_slice() {
                [] => {
                    return Err(CliError::new(
                        ErrorCode::ProjectNotFound,
                        format!("No web, iOS or Android app in {}", root.display()),
                    )
                    .with_hint("Select an app folder with --cwd <app-folder>"));
                }
                [only] => *only,
                several => {
                    let names: Vec<&str> = several.iter().map(|p| p.as_str()).collect();
                    return Err(CliError::new(
                        ErrorCode::AmbiguousPlatform,
                        format!("This folder holds several app types: {}", names.join(", ")),
                    )
                    .with_hint(format!("Choose one with --platform {}", names.join("|"))));
                }
            }
        }
    };
    match platform {
        Platform::Web => match web {
            Some(WebSignal::App(pkg)) => Ok(web_detection(root, &pkg)),
            Some(WebSignal::WorkspaceRoot) => Err(CliError::new(
                ErrorCode::WorkspaceRoot,
                "This is a workspace root. Select an app with --cwd <app-folder>",
            )),
            Some(WebSignal::ReactNative) => Err(CliError::new(
                ErrorCode::UnsupportedPlatform,
                "React Native and Expo projects are not supported yet",
            )
            .with_hint("Select a web, iOS or Android app folder with --cwd <app-folder>")),
            None => Err(CliError::new(
                ErrorCode::ProjectNotFound,
                "Select an app folder containing package.json with --cwd <app>",
            )),
        },
        Platform::Ios if ios => Ok(ios_detection(root)),
        Platform::Ios => Err(CliError::new(
            ErrorCode::ProjectNotFound,
            format!("No Xcode project (*.xcodeproj, *.xcworkspace or Package.swift) in {}", root.display()),
        )
        .with_hint("Select the folder that holds the .xcodeproj with --cwd <app-folder>")),
        Platform::Android if !android.is_empty() => android_detection(root, &android, module),
        Platform::Android => Err(CliError::new(
            ErrorCode::ProjectNotFound,
            format!(
                "No Android Gradle project (settings.gradle and <module>/src/main/AndroidManifest.xml) in {}",
                root.display()
            ),
        )
        .with_hint("Select the folder that holds settings.gradle with --cwd <app-folder>")),
    }
}

enum WebSignal {
    App(Value),
    WorkspaceRoot,
    ReactNative,
}

fn web_signal(root: &Path) -> Result<Option<WebSignal>> {
    let file = root.join("package.json");
    let bytes = match fs::read(&file) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(CliError::io("read", &file, &error)),
    };
    let pkg: Value = serde_json::from_slice(&bytes).map_err(|error| {
        CliError::new(ErrorCode::InvalidConfig, format!("Cannot parse package.json: {error}")).with_path("package.json")
    })?;
    if pkg.get("workspaces").is_some_and(truthy) || root.join("pnpm-workspace.yaml").exists() {
        return Ok(Some(WebSignal::WorkspaceRoot));
    }
    if dependency(&pkg, "react-native") || dependency(&pkg, "expo") {
        return Ok(Some(WebSignal::ReactNative));
    }
    Ok(Some(WebSignal::App(pkg)))
}

/// `{...pkg.dependencies, ...pkg.devDependencies}[name]` is truthy.
fn dependency(pkg: &Value, name: &str) -> bool {
    let lookup = |key| pkg.get(key).and_then(Value::as_object).and_then(|deps| deps.get(name));
    // devDependencies spread last, so its entry wins.
    lookup("devDependencies").or_else(|| lookup("dependencies")).is_some_and(truthy)
}

fn web_detection(root: &Path, pkg: &Value) -> Detection {
    let framework =
        ["next", "react", "vue", "svelte"].into_iter().find(|name| dependency(pkg, name)).unwrap_or("vanilla");
    let lockfile = ["pnpm-lock.yaml", "yarn.lock", "bun.lock", "bun.lockb", "package-lock.json"]
        .into_iter()
        .find(|name| root.join(name).exists());
    let declared = pkg
        .get("packageManager")
        .and_then(Value::as_str)
        .and_then(|value| value.split('@').next())
        .filter(|name| !name.is_empty());
    let package_manager = declared.unwrap_or(match lockfile {
        Some("pnpm-lock.yaml") => "pnpm",
        Some("yarn.lock") => "yarn",
        Some("bun.lock" | "bun.lockb") => "bun",
        _ => "npm",
    });
    let source_directory = if root.join("src").exists() { "src" } else { "." };
    Detection {
        platform: Platform::Web,
        framework: framework.to_string(),
        package_manager: package_manager.to_string(),
        source_directory: source_directory.to_string(),
        assets: "public/generative-icons".to_string(),
    }
}

/// Sorted names of the entries directly in `dir` with whether each is a directory.
fn entries(dir: &Path) -> Vec<(String, bool)> {
    let mut names: Vec<(String, bool)> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            Some((name, entry.path().is_dir()))
        })
        .collect();
    names.sort();
    names
}

fn root_has(root: &Path, matches: impl Fn(&str, bool) -> bool) -> bool {
    entries(root).iter().any(|(name, is_dir)| matches(name, *is_dir))
}

fn ios_detection(root: &Path) -> Detection {
    // The app target is the folder beside `<Name>.xcodeproj` named `<Name>`.
    let listing = entries(root);
    let target = [".xcodeproj", ".xcworkspace"].iter().find_map(|suffix| {
        listing.iter().find_map(|(name, is_dir)| {
            let stem = name.strip_suffix(suffix).filter(|_| *is_dir)?;
            root.join(stem).is_dir().then(|| stem.to_string())
        })
    });
    let source_directory = target.clone().unwrap_or_else(|| ".".to_string());
    let framework = if swift_imports_swiftui(&root.join(&source_directory), 4) { "swiftui" } else { "uikit" };
    let package_manager = if root.join("Podfile").exists() { "cocoapods" } else { "spm" };
    Detection {
        platform: Platform::Ios,
        framework: framework.to_string(),
        package_manager: package_manager.to_string(),
        assets: match &target {
            Some(target) => format!("{target}/GenerativeIcons"),
            None => "GenerativeIcons".to_string(),
        },
        source_directory,
    }
}

fn swift_imports_swiftui(dir: &Path, depth: usize) -> bool {
    entries(dir).into_iter().any(|(name, is_dir)| {
        let path = dir.join(&name);
        if is_dir {
            let skipped = name.starts_with('.') || matches!(name.as_str(), "Pods" | "build" | "DerivedData");
            depth > 0 && !skipped && swift_imports_swiftui(&path, depth - 1)
        } else {
            name.ends_with(".swift")
                && fs::read_to_string(&path).is_ok_and(|source| source.lines().any(|l| l.trim() == "import SwiftUI"))
        }
    })
}

/// Modules (sorted) holding `src/main/AndroidManifest.xml`, when `settings.gradle(.kts)` exists.
fn android_modules(root: &Path) -> Vec<String> {
    if !root.join("settings.gradle").is_file() && !root.join("settings.gradle.kts").is_file() {
        return Vec::new();
    }
    entries(root)
        .into_iter()
        .filter(|(name, is_dir)| *is_dir && root.join(name).join("src/main/AndroidManifest.xml").is_file())
        .map(|(name, _)| name)
        .collect()
}

fn gradle_build_file(root: &Path, module: &str) -> String {
    ["build.gradle.kts", "build.gradle"]
        .iter()
        .find_map(|name| fs::read_to_string(root.join(module).join(name)).ok())
        .unwrap_or_default()
}

fn android_detection(root: &Path, modules: &[String], requested: Option<&str>) -> Result<Detection> {
    let is_application = |module: &&String| {
        let build = gradle_build_file(root, module);
        build.contains("com.android.application") || build.contains("android.application")
    };
    let applications: Vec<&String> = modules.iter().filter(is_application).collect();
    let module = match requested {
        Some(name) if modules.iter().any(|m| m == name) => name.to_string(),
        Some(name) => {
            return Err(CliError::new(
                ErrorCode::ProjectNotFound,
                format!("No Android module {name} with src/main/AndroidManifest.xml"),
            )
            .with_hint(format!("Modules found: {}", modules.join(", "))));
        }
        None => match (applications.as_slice(), modules) {
            ([only], _) => only.to_string(),
            (_, _) if modules.iter().any(|m| m == "app") => "app".to_string(),
            (_, [only]) => only.clone(),
            _ => {
                return Err(CliError::new(
                    ErrorCode::AmbiguousPlatform,
                    format!("Several Android modules could hold the app: {}", modules.join(", ")),
                )
                .with_hint("Choose one with --module <name>"));
            }
        },
    };
    let framework = if gradle_build_file(root, &module).contains("compose") { "compose" } else { "views" };
    Ok(Detection {
        platform: Platform::Android,
        framework: framework.to_string(),
        package_manager: "gradle".to_string(),
        source_directory: format!("{module}/src/main"),
        assets: format!("{module}/src/main/assets"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn web_matches_inspect_project() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "package.json",
            r#"{"dependencies":{"react":"^19","next":""},"devDependencies":{"vue":"3"}}"#,
        );
        write(dir.path(), "yarn.lock", "");
        let found = detect(dir.path(), None, None).unwrap();
        // An empty version string is falsy, so `next` does not count.
        assert_eq!(
            (found.framework.as_str(), found.package_manager.as_str(), found.source_directory.as_str()),
            ("react", "yarn", ".")
        );
        write(dir.path(), "package.json", r#"{"packageManager":"pnpm@9.1.0","devDependencies":{"svelte":"5"}}"#);
        fs::create_dir(dir.path().join("src")).unwrap();
        let found = detect(dir.path(), None, None).unwrap();
        assert_eq!(
            (found.framework.as_str(), found.package_manager.as_str(), found.source_directory.as_str()),
            ("svelte", "pnpm", "src")
        );
    }

    #[test]
    fn workspace_roots_and_react_native_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "package.json", r#"{"workspaces":[]}"#);
        assert_eq!(detect(dir.path(), None, None).unwrap_err().code, ErrorCode::WorkspaceRoot);
        write(dir.path(), "package.json", "{}");
        write(dir.path(), "pnpm-workspace.yaml", "packages: []");
        assert_eq!(detect(dir.path(), None, None).unwrap_err().code, ErrorCode::WorkspaceRoot);
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "package.json", r#"{"dependencies":{"expo":"51"}}"#);
        assert_eq!(detect(dir.path(), None, None).unwrap_err().code, ErrorCode::UnsupportedPlatform);
    }

    #[test]
    fn ios_uses_the_target_folder() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "Demo.xcodeproj/project.pbxproj", "");
        write(dir.path(), "Demo/ContentView.swift", "import SwiftUI\n");
        let found = detect(dir.path(), None, None).unwrap();
        assert_eq!(found.platform, Platform::Ios);
        assert_eq!((found.framework.as_str(), found.package_manager.as_str()), ("swiftui", "spm"));
        assert_eq!(found.assets, "Demo/GenerativeIcons");
    }

    #[test]
    fn android_prefers_the_single_application_module() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "settings.gradle.kts", "include(\":mobile\", \":core\")");
        write(dir.path(), "mobile/build.gradle.kts", "plugins { id(\"com.android.application\") }");
        write(dir.path(), "mobile/src/main/AndroidManifest.xml", "<manifest/>");
        write(dir.path(), "core/build.gradle.kts", "plugins { id(\"com.android.library\") }");
        write(dir.path(), "core/src/main/AndroidManifest.xml", "<manifest/>");
        let found = detect(dir.path(), None, None).unwrap();
        assert_eq!((found.assets.as_str(), found.framework.as_str()), ("mobile/src/main/assets", "views"));
        let found = detect(dir.path(), None, Some("core")).unwrap();
        assert_eq!(found.assets, "core/src/main/assets");
    }

    #[test]
    fn several_platforms_need_a_choice() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "package.json", "{}");
        write(dir.path(), "Package.swift", "");
        let error = detect(dir.path(), None, None).unwrap_err();
        assert_eq!(error.code, ErrorCode::AmbiguousPlatform);
        assert_eq!(detect(dir.path(), Some(Platform::Ios), None).unwrap().assets, "GenerativeIcons");
        let empty = tempfile::tempdir().unwrap();
        assert_eq!(detect(empty.path(), None, None).unwrap_err().code, ErrorCode::ProjectNotFound);
    }
}
