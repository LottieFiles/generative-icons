//! Module resolution (js-runtime §2.3). Module identity is the canonical path, as in Node,
//! so a kit file reached as `./drawing.mjs`, `../library/drawing.mjs` or
//! `../../library/drawing.mjs` is one module instance.

use crate::ModuleError;
use rquickjs::{Ctx, Module, loader, module::Declared};
use std::{
    cell::RefCell,
    fs,
    path::{Component, Path, PathBuf},
    rc::Rc,
};

pub(crate) const GUARD: &str = include_str!("../js/guard.js");
pub(crate) const HOST: &str = "gai:host";

/// Builtin modules: the bootstrap, the svgpath 2.6.0 ES-module bundle and the 01/02 shims.
fn builtin(name: &str) -> Option<&'static str> {
    Some(match name {
        HOST => include_str!("../js/host.mjs"),
        "svgpath" => include_str!("../js/svgpath.mjs"),
        "gai:legacy/01" => include_str!("../js/legacy01.mjs"),
        "gai:legacy/02" => include_str!("../js/legacy02.mjs"),
        _ => return None,
    })
}

/// The first refused import, kept for the error report (QuickJS only sees a message).
#[derive(Debug, Clone)]
pub(crate) struct Refusal {
    pub specifier: String,
    pub importer: String,
    pub reason: ModuleError,
}

pub(crate) type Refusals = Rc<RefCell<Option<Refusal>>>;

/// The canonical folders relative imports may reach.
#[derive(Debug)]
pub(crate) struct Roots(Vec<PathBuf>);

impl Roots {
    pub fn new(dirs: &[&Path]) -> Self {
        Roots(dirs.iter().filter_map(|dir| fs::canonicalize(dir).ok()).collect())
    }

    fn contains(&self, path: &Path) -> bool {
        self.0.iter().any(|root| path.starts_with(root))
    }

    /// The canonical path of the module file at `path`, which must be absolute: an `.mjs`
    /// file inside a root once symlinks are followed.
    pub fn check(&self, path: &Path) -> Result<String, ModuleError> {
        let canonical = fs::canonicalize(path).map_err(|_| ModuleError::NotFound(normalize(path)))?;
        if canonical.extension().is_none_or(|ext| ext != "mjs") {
            return Err(ModuleError::NotModule(canonical));
        }
        if !self.contains(&canonical) {
            // A path inside a root before symlinks are followed and outside after is a
            // symlink leaving the root.
            let lexical = normalize(path);
            return Err(if self.contains(&lexical) {
                ModuleError::Symlink { path: lexical, target: canonical }
            } else {
                ModuleError::OutsideRoots(canonical)
            });
        }
        canonical.into_os_string().into_string().map_err(|path| ModuleError::NotUtf8(path.into()))
    }
}

pub(crate) struct Resolver {
    roots: Roots,
    /// The canonical `definitions.mjs` that Rust imports, already checked by [`Roots::check`].
    entry: Option<String>,
    refusals: Refusals,
}

impl Resolver {
    pub fn new(roots: Roots, entry: Option<String>, refusals: Refusals) -> Self {
        Resolver { roots, entry, refusals }
    }

    fn resolve_path(&self, base: &str, name: &str) -> Result<String, ModuleError> {
        if builtin(name).is_some() {
            // Only Rust (an empty base) and the builtins themselves may import gai:*.
            let from_rust = base.is_empty() || builtin(base).is_some();
            return if name == "svgpath" || from_rust { Ok(name.to_owned()) } else { Err(ModuleError::Unsupported) };
        }
        if self.entry.as_deref() == Some(name) && base.is_empty() {
            return Ok(name.to_owned());
        }
        if !(name.starts_with("./") || name.starts_with("../")) {
            return Err(ModuleError::Unsupported);
        }
        if builtin(base).is_some() || base.is_empty() {
            return Err(ModuleError::FromBuiltin);
        }
        // The importer's path is canonical, so the joined path is absolute.
        self.roots.check(&Path::new(base).parent().unwrap_or(Path::new("/")).join(name))
    }
}

impl loader::Resolver for Resolver {
    fn resolve<'js>(&mut self, _ctx: &Ctx<'js>, base: &str, name: &str) -> rquickjs::Result<String> {
        self.resolve_path(base, name).map_err(|reason| {
            let message = reason.to_string();
            refuse(&self.refusals, name, base, reason);
            rquickjs::Error::new_resolving_message(base, name, message)
        })
    }
}

/// The canonical paths of the file modules loaded so far, in load order.
pub(crate) type Loaded = Rc<RefCell<Vec<PathBuf>>>;

pub(crate) struct Loader {
    refusals: Refusals,
    loaded: Loaded,
}

impl Loader {
    pub fn new(refusals: Refusals) -> Self {
        Loader { refusals, loaded: Loaded::default() }
    }

    /// The list this loader appends each file module it reads to.
    pub fn loaded(&self) -> Loaded {
        Rc::clone(&self.loaded)
    }
}

impl loader::Loader for Loader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, name: &str) -> rquickjs::Result<Module<'js, Declared>> {
        if let Some(source) = builtin(name) {
            return Module::declare(ctx.clone(), name, source);
        }
        match read_module(Path::new(name)) {
            Ok(source) => {
                self.loaded.borrow_mut().push(PathBuf::from(name));
                Module::declare(ctx.clone(), name, source)
            }
            Err(reason) => {
                let message = reason.to_string();
                refuse(&self.refusals, name, "", reason);
                Err(rquickjs::Error::new_loading_message(name, message))
            }
        }
    }
}

fn refuse(refusals: &Refusals, specifier: &str, importer: &str, reason: ModuleError) {
    refusals.borrow_mut().get_or_insert(Refusal {
        specifier: specifier.to_owned(),
        importer: importer.to_owned(),
        reason,
    });
}

/// Reads a module as UTF-8. Node would strip a byte order mark; none exists in the corpus,
/// so one is refused rather than silently handled differently.
fn read_module(path: &Path) -> Result<String, ModuleError> {
    let bytes =
        fs::read(path).map_err(|error| ModuleError::Read { path: path.to_owned(), message: error.to_string() })?;
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(ModuleError::ByteOrderMark(path.to_owned()));
    }
    String::from_utf8(bytes).map_err(|_| ModuleError::NotUtf8(path.to_owned()))
}

/// Removes `.` and resolves `..` without touching the file system.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_resolves_dot_segments() {
        assert_eq!(normalize(Path::new("/a/b/../c/./d.mjs")), Path::new("/a/c/d.mjs"));
    }

    #[test]
    fn builtins_are_reserved_for_rust_and_other_builtins() {
        let resolver = Resolver::new(Roots::new(&[]), None, Refusals::default());
        assert_eq!(resolver.resolve_path("", "gai:host").unwrap(), "gai:host");
        assert_eq!(resolver.resolve_path("gai:host", "svgpath").unwrap(), "svgpath");
        assert_eq!(resolver.resolve_path("/set/definitions.mjs", "svgpath").unwrap(), "svgpath");
        assert_eq!(resolver.resolve_path("/set/definitions.mjs", "gai:host"), Err(ModuleError::Unsupported));
        assert_eq!(resolver.resolve_path("/set/definitions.mjs", "node:fs"), Err(ModuleError::Unsupported));
        assert_eq!(resolver.resolve_path("gai:host", "./x.mjs"), Err(ModuleError::FromBuiltin));
    }
}
