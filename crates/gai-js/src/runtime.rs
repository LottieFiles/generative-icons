use crate::{
    Error, Location, ModernProfile, ModuleError, Result, ReverseName, SetSource,
    describe::{self, Description},
    loader::{self, Refusal, Refusals, Roots},
    probe::{Packed, ProbeRequest, ProbeResult, ProbeWidth},
    project::project,
    samples::{self, VariantSamples},
    watchdog,
};
use rquickjs::{
    Array, Coerced, Context, Ctx, FromJs, Function, Module, Object, Persistent, Runtime, TypedArray, Value,
};
use serde_json::Value as Json;
use std::{
    cell::Cell,
    fs,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

/// Resource limits for one set's runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limits {
    /// QuickJS heap limit.
    pub memory: usize,
    /// QuickJS stack limit, applied on threads started by [`run_on_worker`](crate::run_on_worker)
    /// or [`map_parallel`](crate::map_parallel), whose native stack leaves room for it. Other
    /// threads get at most [`UNSIZED_THREAD_STACK`](crate::UNSIZED_THREAD_STACK).
    pub stack: usize,
    /// Time limit for each call into the set (module evaluation, one animation, one probe).
    pub call_timeout: Duration,
    /// Time limit for all calls into the set together, so a set cannot take many
    /// `call_timeout`s. Time between calls does not count.
    pub budget: Duration,
    /// How long a call may run past its deadline, in native code that never checks it,
    /// before the [`watchdog`](crate::watchdog) (once installed) ends the process.
    pub grace: Duration,
}

impl Default for Limits {
    /// 1 GiB, 8 MiB, 120 s per call, 600 s per set and 10 s of grace. The slowest set
    /// spends about 25 s sampling in total.
    fn default() -> Self {
        Limits {
            memory: 1 << 30,
            stack: 8 << 20,
            call_timeout: Duration::from_secs(120),
            budget: Duration::from_secs(600),
            grace: Duration::from_secs(10),
        }
    }
}

impl Limits {
    /// The longest a set's calls run in total once the [`watchdog`](crate::watchdog) is
    /// installed: the [`budget`](Self::budget), plus the [`grace`](Self::grace) by which
    /// the last call may overrun it. Time between calls does not count.
    pub fn hard_deadline(&self) -> Duration {
        self.budget.saturating_add(self.grace)
    }
}

/// Fails on CPUs that cannot run the V8-exact Math (x86_64 without FMA). Every
/// [`SetRuntime`] checks this before running JS.
pub fn check_cpu() -> Result<()> {
    #[cfg(target_arch = "x86_64")]
    if !std::arch::is_x86_feature_detected!("fma") {
        return Err(Error::UnsupportedCpu);
    }
    Ok(())
}

/// A failure inside a `Context::with` closure. The runtime turns it into an [`Error`].
#[derive(Debug)]
pub(crate) enum Fail {
    /// A JS exception is pending (`Error::Exception`), or rquickjs failed otherwise.
    Js(rquickjs::Error),
    /// The host broke its own data contract.
    Protocol(String),
}

impl From<rquickjs::Error> for Fail {
    fn from(error: rquickjs::Error) -> Self {
        Fail::Js(error)
    }
}

/// How long a stopped call has to unwind before the interrupt handler stops it again.
const RESTOP: Duration = Duration::from_millis(50);

/// The deadline the interrupt handler enforces, whether it fired, and the time the
/// set's calls have used.
struct Clock {
    deadline: Cell<Instant>,
    fired: Cell<bool>,
    /// When the handler last stopped the call.
    stopped_at: Cell<Option<Instant>>,
    spent: Cell<Duration>,
}

impl Clock {
    /// Sets the deadline `limit` from now and returns it.
    fn arm(&self, limit: Duration) -> Instant {
        let deadline = Instant::now() + limit;
        self.deadline.set(deadline);
        self.fired.set(false);
        self.stopped_at.set(None);
        deadline
    }

    /// Whether the interrupt handler stops the call: once its deadline has passed, and then
    /// again only [`RESTOP`] after the last stop. A stop throws while QuickJS unwinds and
    /// settles the call's promises; a second stop in between (the watchdog's poll request
    /// landing just after the handler's own poll stopped the call) throws over the first
    /// error and loses where the call stopped.
    fn expired(&self, now: Instant) -> bool {
        let expired = now > self.deadline.get() && self.stopped_at.get().is_none_or(|at| now - at >= RESTOP);
        if expired {
            self.fired.set(true);
            self.stopped_at.set(Some(now));
        }
        expired
    }
}

/// The module Rust imports for a set's definitions.
enum Entry {
    /// A builtin `gai:legacy/NN` shim, exporting `definitions`.
    Shim(&'static str),
    /// The canonical path of `definitions.mjs`, exporting `icons`.
    File(String),
}

/// The profile `install()` hands the host.
enum Profile {
    Legacy(Persistent<Object<'static>>),
    Modern(ModernProfile),
}

/// One set's authoring code loaded into its own QuickJS runtime.
///
/// Use one runtime per set and never reuse it for another: `install()` mutates the
/// definitions, and module singletons (such as the `keyframes` icon set 06 pushes) must not
/// be shared. A runtime is `!Send`: create it on the thread that uses it, ideally one from
/// [`run_on_worker`](crate::run_on_worker) or [`map_parallel`](crate::map_parallel), which
/// get the full [`Limits::stack`].
pub struct SetRuntime {
    // JS handles come first so they are released before the context and runtime.
    host: Persistent<Object<'static>>,
    icons: Persistent<Value<'static>>,
    profile: Profile,
    context: Context,
    _runtime: Runtime,
    guard: Guard,
    legacy: bool,
    /// Icon and variant names once installed, for error locations.
    names: Vec<(String, Vec<String>)>,
    /// The file modules the loader has read.
    loaded: loader::Loaded,
}

impl SetRuntime {
    /// Creates a runtime for `source`, evaluates its definitions (after a guard that
    /// disables `Math.random`, `Date` and `performance`) and binds the exported `icons`.
    /// [`probe`](Self::probe) works from here on; sampling needs [`install`](Self::install).
    pub fn load(source: &SetSource, limits: &Limits) -> Result<Self> {
        check_cpu()?;
        let guard = Guard {
            clock: Rc::new(Clock {
                deadline: Cell::new(Instant::now()),
                fired: Cell::new(false),
                stopped_at: Cell::new(None),
                spent: Cell::new(Duration::ZERO),
            }),
            refusals: Refusals::default(),
            label: source.label(),
            limits: limits.clone(),
        };
        let runtime = Runtime::new().map_err(|error| guard.engine(error.to_string()))?;
        runtime.set_memory_limit(limits.memory);
        runtime.set_max_stack_size(crate::worker::usable_stack(limits));
        let clock = Rc::clone(&guard.clock);
        runtime.set_interrupt_handler(Some(Box::new(move || clock.expired(Instant::now()))));
        let (entry, roots) = match source {
            SetSource::Legacy(set) => (Entry::Shim(set.module()), Roots::new(&[])),
            SetSource::Definitions { set_dir, library_dir, .. } => {
                let file = set_dir.join("definitions.mjs");
                let roots = Roots::new(&[set_dir.as_path(), library_dir.as_path()]);
                // The entry gets the checks of every import. Joined to the canonical set
                // folder, a symlinked definitions.mjs that leaves the roots reads as one.
                let entry = fs::canonicalize(set_dir)
                    .map_err(|_| ModuleError::NotFound(file.clone()))
                    .and_then(|dir| roots.check(&dir.join("definitions.mjs")))
                    .map_err(|reason| Error::Module {
                        set: guard.label.clone(),
                        specifier: file.display().to_string(),
                        importer: String::new(),
                        reason: Box::new(reason),
                    })?;
                (Entry::File(entry), roots)
            }
        };
        let entry_file = match &entry {
            Entry::File(path) => Some(path.clone()),
            Entry::Shim(_) => None,
        };
        let file_loader = loader::Loader::new(Rc::clone(&guard.refusals));
        let loaded = file_loader.loaded();
        runtime.set_loader(loader::Resolver::new(roots, entry_file, Rc::clone(&guard.refusals)), file_loader);
        let context = Context::full(&runtime).map_err(|error| guard.engine(error.to_string()))?;

        let (host, icons, shim) = guard.run(&context, Location::default(), |ctx| {
            ctx.eval::<(), _>(loader::GUARD)?;
            let host: Object = Module::import(&ctx, loader::HOST)?.finish()?;
            let (icons, shim) = match &entry {
                Entry::Shim(module) => {
                    let shim: Object = Module::import(&ctx, *module)?.finish()?;
                    (shim.get::<_, Value>("definitions")?, Some(shim))
                }
                Entry::File(path) => {
                    let namespace: Object = Module::import(&ctx, path.as_str())?.finish()?;
                    (namespace.get::<_, Value>("icons")?, None)
                }
            };
            host.get::<_, Function>("bind")?.call::<_, ()>((icons.clone(),))?;
            Ok((
                Persistent::save(&ctx, host),
                Persistent::save(&ctx, icons),
                shim.map(|shim| Persistent::save(&ctx, shim)),
            ))
        })?;
        let profile = match (source, shim) {
            (SetSource::Definitions { profile, .. }, _) => Profile::Modern(*profile),
            (SetSource::Legacy(_), Some(shim)) => Profile::Legacy(shim),
            (SetSource::Legacy(_), None) => return Err(guard.engine("legacy shim missing".into())),
        };
        Ok(SetRuntime {
            host,
            icons,
            legacy: matches!(profile, Profile::Legacy(_)),
            profile,
            context,
            _runtime: runtime,
            guard,
            names: Vec::new(),
            loaded,
        })
    }

    /// The set's name in messages: its folder name.
    pub fn label(&self) -> &str {
        &self.guard.label
    }

    /// The canonical path of every file module loaded so far (`definitions.mjs`, the
    /// modules it imports and the kit files they reach), in load order. Builtins are not
    /// listed. Modules a later call imports dynamically are added when they load.
    pub fn modules(&self) -> Vec<PathBuf> {
        self.loaded.borrow().clone()
    }

    /// Applies the sampling profile (the duplicate-name and duotone checks where the old
    /// builder had them, then the weight expansion) and describes the set. Call it once.
    pub fn install(&mut self) -> Result<Description> {
        let description = self.run(Location::default(), |ctx, host| {
            let profile = Object::new(ctx.clone())?;
            match &self.profile {
                Profile::Legacy(shim) => {
                    profile.set("kind", "legacy")?;
                    profile.set("shim", shim.clone().restore(ctx)?)?;
                }
                Profile::Modern(modern) => {
                    profile.set("kind", "modern")?;
                    profile.set("passWidth", modern.pass_width)?;
                    profile.set("checkDuotone", modern.check_duotone)?;
                    let reverse_name = match modern.reverse_name {
                        ReverseName::Suffix => "suffix",
                        ReverseName::Essentials => "essentials",
                    };
                    profile.set("reverseName", reverse_name)?;
                }
            }
            host.get::<_, Function>("install")?.call::<_, ()>((profile,))?;
            let described: Value = host.get::<_, Function>("describe")?.call(())?;
            let json = project(host, described)?.ok_or_else(|| Fail::Protocol("describe() returned nothing".into()))?;
            Description::from_json(&json)
        })?;
        self.names = description
            .icons
            .iter()
            .map(|icon| (icon.name.clone(), icon.variants.iter().map(|v| v.name.clone()).collect()))
            .collect();
        Ok(description)
    }

    /// Samples one animation in the builder's call order: for each part `d(0)`, every
    /// frame (`d`, then `pose` per point), the rigid transform, then visibility.
    pub fn sample_variant(&self, icon: usize, variant: usize) -> Result<VariantSamples> {
        let known = self.names.get(icon).is_some_and(|(_, variants)| variant < variants.len());
        if !known {
            return Err(self.guard.engine(format!("no animation {variant} of icon {icon}; install() first")));
        }
        let with_visibility = !self.legacy;
        self.run(self.location(icon, Some(variant)), |_, host| {
            let out: Object = host.get::<_, Function>("sampleVariant")?.call((icon as i32, variant as i32))?;
            let header: TypedArray<i32> = out.get("header")?;
            let commands: TypedArray<u8> = out.get("commands")?;
            let data: TypedArray<f64> = out.get("data")?;
            samples::unpack(header.as_ref(), commands.as_ref(), data.as_ref(), with_visibility)
        })
    }

    /// Evaluates one authored part at `request.times`; see [`ProbeSample`](crate::ProbeSample).
    /// Works before and after [`install`](Self::install), always on unexpanded parts.
    pub fn probe(&self, request: &ProbeRequest<'_>) -> Result<ProbeResult> {
        self.run(self.location(request.icon, None), |ctx, host| {
            let times = TypedArray::<f64>::new_copy(ctx.clone(), request.times)?;
            let (width, authored) = match request.width {
                ProbeWidth::Authored => (f64::NAN, true),
                ProbeWidth::Value(width) => (width, false),
            };
            let out: Object = host.get::<_, Function>("probe")?.call((
                request.icon as i32,
                request.part as i32,
                request.strong,
                request.reverse,
                width,
                authored,
                times,
            ))?;
            let callbacks = project(host, out.get("callbacks")?)?
                .ok_or_else(|| Fail::Protocol("probe() returned no callbacks".into()))?;
            let messages: Array = out.get("messages")?;
            let messages = (0..messages.len()).map(|k| project(host, messages.get(k)?)).collect::<Result<_, Fail>>()?;
            let packed = Packed {
                callbacks: describe::callbacks(&callbacks)?,
                static_d: out.get("staticD")?,
                status: typed(&out, "status")?,
                stage: typed(&out, "stage")?,
                messages,
                geo_error: strings(&out, "geoError")?,
                svg_err: strings(&out, "svgErr")?,
                topology: typed(&out, "topology")?,
                topologies: strings(&out, "topologies")?,
                point_offsets: typed(&out, "pointOffsets")?,
                points: typed(&out, "points")?,
                posed: typed(&out, "posed")?,
                posed_ok: typed(&out, "posedOk")?,
                posed_pairs: typed(&out, "posedPairs")?,
                visibility: typed(&out, "visibility")?,
                vis_flags: typed(&out, "visFlags")?,
                tr_flags: typed(&out, "trFlags")?,
                tr_nullish: typed(&out, "trNullish")?,
                pivot: typed(&out, "pivot")?,
                pivot_json: strings(&out, "pivotJson")?,
                tr_or: typed(&out, "trOr")?,
            };
            packed.unpack(request.times)
        })
    }

    /// Runs `f` with this set's context and its raw `icons` export (weight-expanded once
    /// installed), for checks that must inspect author values with JS semantics. The call
    /// is time-limited like any other.
    pub fn with_icons<R>(&self, f: impl for<'js> FnOnce(&JsScope<'js>) -> rquickjs::Result<R>) -> Result<R> {
        self.run(Location::default(), |ctx, host| {
            let scope = JsScope { ctx: ctx.clone(), host: host.clone(), icons: self.icons.clone().restore(ctx)? };
            Ok(f(&scope)?)
        })
    }

    fn location(&self, icon: usize, variant: Option<usize>) -> Location {
        let names = self.names.get(icon);
        Location {
            icon: names.map(|(name, _)| name.clone()),
            variant: variant.and_then(|v| names.and_then(|(_, variants)| variants.get(v).cloned())),
            part: None,
        }
    }

    fn run<R>(
        &self,
        location: Location,
        f: impl for<'js> FnOnce(&Ctx<'js>, &Object<'js>) -> Result<R, Fail>,
    ) -> Result<R> {
        self.guard.run(&self.context, location, |ctx| {
            let host = self.host.clone().restore(&ctx)?;
            f(&ctx, &host)
        })
    }
}

/// A set's context during [`SetRuntime::with_icons`].
pub struct JsScope<'js> {
    ctx: Ctx<'js>,
    host: Object<'js>,
    icons: Value<'js>,
}

impl<'js> JsScope<'js> {
    /// The set's context.
    pub fn ctx(&self) -> &Ctx<'js> {
        &self.ctx
    }

    /// The definitions' `icons` export.
    pub fn icons(&self) -> &Value<'js> {
        &self.icons
    }

    /// `serde_json::from_str(JSON.stringify(value))` as V8 would print it: numbers are taken
    /// as exact doubles, never as QuickJS text. `None` when `JSON.stringify` returns
    /// `undefined`.
    pub fn project(&self, value: Value<'js>) -> rquickjs::Result<Option<Json>> {
        project(&self.host, value).map_err(|fail| match fail {
            Fail::Js(error) => error,
            Fail::Protocol(message) => rquickjs::Error::new_from_js_message("value", "JSON", message),
        })
    }
}

/// Runs calls under the time limits and turns failures into located [`Error`]s.
struct Guard {
    clock: Rc<Clock>,
    refusals: Refusals,
    label: String,
    limits: Limits,
}

impl Guard {
    fn run<R>(
        &self,
        context: &Context,
        location: Location,
        f: impl for<'js> FnOnce(Ctx<'js>) -> Result<R, Fail>,
    ) -> Result<R> {
        let left = self.limits.budget.saturating_sub(self.clock.spent.get());
        if left.is_zero() {
            return Err(self.over_budget(location, None));
        }
        let started = Instant::now();
        let budget_bound = left < self.limits.call_timeout;
        let result = context.with(|ctx| {
            let deadline = self.clock.arm(left.min(self.limits.call_timeout));
            let _watch = watchdog::watch(watchdog::Runtime::of(&ctx), deadline, self.limits.grace, || {
                self.stopped(location.clone(), budget_bound, None)
            });
            f(ctx.clone()).map_err(|fail| self.error(&ctx, fail, location, budget_bound))
        });
        self.clock.spent.set(self.clock.spent.get() + started.elapsed());
        result
    }

    fn over_budget(&self, location: Location, stack: Option<String>) -> Error {
        Error::Budget { set: self.label.clone(), location: Box::new(location), limit: self.limits.budget, stack }
    }

    /// The error of a call stopped at its deadline, with the stack where it stopped.
    /// `budget_bound` is whether the deadline came from the budget.
    fn stopped(&self, location: Location, budget_bound: bool, stack: Option<String>) -> Error {
        if budget_bound {
            return self.over_budget(location, stack);
        }
        let (set, location, limit) = (self.label.clone(), Box::new(location), self.limits.call_timeout);
        Error::Timeout { set, location, limit, stack }
    }

    fn engine(&self, message: String) -> Error {
        Error::Engine { set: self.label.clone(), message }
    }

    /// `budget_bound` is whether the call's deadline came from the budget.
    fn error(&self, ctx: &Ctx<'_>, fail: Fail, mut location: Location, budget_bound: bool) -> Error {
        let error = match fail {
            Fail::Protocol(message) => return self.engine(message),
            Fail::Js(error) => error,
        };
        // A pending exception belongs to this call whatever failed, so the next call
        // starts clean.
        let thrown = ctx.catch();
        // Stopped calls fail in many ways: a thrown "interrupted", or a module promise
        // the interrupt left pending (`WouldBlock`).
        if self.clock.fired.get() {
            let stack = thrown.as_exception().and_then(|exception| exception.stack());
            return self.stopped(location, budget_bound, stack);
        }
        let set = self.label.clone();
        match error {
            rquickjs::Error::Exception => {}
            rquickjs::Error::WouldBlock => {
                let message = "a top-level await never finished: it waits on a promise that nothing settles".into();
                return Error::Script { set, location: Box::new(location), message, stack: None };
            }
            error => return self.engine(error.to_string()),
        }
        if let Some(Refusal { specifier, importer, reason }) = self.refusals.borrow_mut().take() {
            return Error::Module { set, specifier, importer, reason: Box::new(reason) };
        }
        location.part = current_part(ctx);
        let (message, stack) = match thrown.as_exception() {
            Some(exception) => (exception.message().unwrap_or_default(), exception.stack()),
            None => {
                (thrown.get::<Coerced<String>>().map_or_else(|_| "a value that is not an Error".into(), |s| s.0), None)
            }
        };
        Error::Script { set, location: Box::new(location), message, stack }
    }
}

/// The part the host was evaluating when it threw.
fn current_part(ctx: &Ctx<'_>) -> Option<String> {
    let part = (|| -> rquickjs::Result<Option<String>> {
        let host: Object = Module::import(ctx, loader::HOST)?.finish()?;
        let part: Value = host.get::<_, Function>("currentPart")?.call(())?;
        part.as_string().map(|s| s.to_string()).transpose()
    })();
    part.unwrap_or_else(|_| {
        ctx.catch();
        None
    })
}

fn typed<'js, T: Copy>(object: &Object<'js>, key: &str) -> rquickjs::Result<Vec<T>>
where
    TypedArray<'js, T>: FromJs<'js> + AsRef<[T]>,
{
    let array: TypedArray<T> = object.get(key)?;
    Ok(array.as_ref().to_vec())
}

/// A JS array of strings (`String(v)` for anything else).
fn strings(object: &Object<'_>, key: &str) -> rquickjs::Result<Vec<String>> {
    Ok(object.get::<_, Vec<Coerced<String>>>(key)?.into_iter().map(|s| s.0).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stopped_call_has_time_to_unwind_before_the_next_stop() {
        let clock = Clock {
            deadline: Cell::new(Instant::now()),
            fired: Cell::new(false),
            stopped_at: Cell::new(None),
            spent: Cell::new(Duration::ZERO),
        };
        let deadline = clock.arm(Duration::from_millis(200));
        let after = |ms| deadline + Duration::from_millis(ms);
        assert!(!clock.expired(deadline));
        assert!(clock.expired(after(1)));
        assert!(clock.fired.get());
        // The watchdog's request can make QuickJS poll again while the first stop unwinds.
        assert!(!clock.expired(after(2)));
        assert!(!clock.expired(after(50)));
        // Code that still runs after that is stopped again.
        assert!(clock.expired(after(51)));
        // A new call starts clean.
        clock.arm(Duration::from_millis(200));
        assert!(!clock.fired.get() && clock.stopped_at.get().is_none());
    }
}
