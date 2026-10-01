//! `compare` expressions (verify.md §8.5): a JSON AST evaluated in tree order, left operand
//! first, so legacy float expressions reproduce bit for bit. It is a superset of the
//! contracts expression syntax.
//!
//! ```text
//! EXPR := number | [EXPR, …]                            array literal (a point)
//!       | {"pts"|"flat"|"vis"|"width"|"radius": PART}   sampled coordinates, polyline,
//!                                                       visibility, stroke width, paint radius
//!       | {"box": PART, "of"?: "flat"|"pts", "paint"?: bool}   [l, t, r, b]; default flat, painted
//!       | {"at": [EXPR, i]} | {"slice": [EXPR, s, e?]}  JS .at() / .slice()
//!       | {"xs"|"ys": EXPR}                              every other coordinate, or each point's x/y
//!       | {"minOf"|"maxOf": EXPR}                        Math.min(...arr) / Math.max(...arr)
//!       | {"add"|"sub"|"mul"|"div": [EXPR, EXPR]} | {"neg": EXPR} | {"abs": EXPR}
//!       | {"hypot"|"min"|"max": [EXPR, …]}               V8 Math.hypot / Math.min / Math.max
//!       | {"segdist": [P, A, B]} | {"polydist": [P, POLY]} | {"inside": [P, POLY]}   inside → 1 | 0
//!       | {"center": BOX}                                [(l + r) / 2, (t + b) / 2]
//!       | {"mapMin"|"mapMax": {"over": EXPR, "expr": EXPR}}   fold EXPR over each item, bound to $p
//!       | {"$p": 0|1|null}                               a coordinate of the bound point, or the item
//!       | {"gt"|"ge"|"lt"|"le"|"eq": [EXPR, EXPR]} | {"and"|"or": [EXPR, …]} | {"not": EXPR}   → 1 | 0
//!       | {"t": _} | {"f": _} | {"rest": _}              seconds, frame, the action's rest frame
//! PART := "name" | {"part": "name", "pick": "bold"|"base"|"prefix"|"boldOrPrefix"|"exact"}
//! ```

use crate::contracts::geom::{Point, bounds, inside_poly, pad_box, seg_dist};
use crate::contracts::jsnum::{at_index, hypot, max_of, min_of, slice_range};
use crate::verify::VerifyError;
use crate::verify::geom::{Pick, coords_box, poly_distance};
use serde_json::{Map, Value};
use std::rc::Rc;

/// A part reference: a name (prefixed per the dialect) and an optional picker override.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Part {
    /// The part name; `@` is the node of the enclosing `each` rule.
    pub name: String,
    /// The picker; `None` uses the dialect's.
    pub pick: Option<Pick>,
}

impl Part {
    /// Parses `"name"` or `{"part": "name", "pick": …}`.
    pub fn parse(v: &Value) -> Result<Part, VerifyError> {
        match v {
            Value::String(name) => Ok(Part {
                name: name.clone(),
                pick: None,
            }),
            Value::Object(map) => {
                if let Some(extra) = map.keys().find(|k| !matches!(k.as_str(), "part" | "pick")) {
                    return Err(bad(format!("unknown part field {extra}")));
                }
                let name = map
                    .get("part")
                    .and_then(Value::as_str)
                    .ok_or_else(|| bad("a part needs a name"))?;
                let pick = map
                    .get("pick")
                    .map(|p| {
                        serde_json::from_value(p.clone())
                            .map_err(|_| bad(format!("unknown picker {p}")))
                    })
                    .transpose()?;
                Ok(Part {
                    name: name.to_string(),
                    pick,
                })
            }
            other => Err(bad(format!("not a part: {other}"))),
        }
    }
}

/// A parsed expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A number literal.
    Num(f64),
    /// An array literal of numbers.
    List(Vec<Expr>),
    /// A part property at the current time.
    Part(PartValue, Part),
    /// Bounds `[l, t, r, b]` of a part.
    Box {
        /// The part.
        part: Part,
        /// Bounds of the flattened polyline (`true`) or of the control coordinates.
        flat: bool,
        /// Grow by the paint radius.
        paint: bool,
    },
    /// JS `.at(i)`.
    At(Box<Expr>, i64),
    /// JS `.slice(start, end)`.
    Slice(Box<Expr>, i64, Option<i64>),
    /// `xy(arr, axis)`, or each point's coordinate.
    Axis(Box<Expr>, usize),
    /// `Math.min(...arr)` or `Math.max(...arr)`.
    Spread(Fold, Box<Expr>),
    /// Binary arithmetic.
    Arith(Arith, Box<Expr>, Box<Expr>),
    /// Unary minus.
    Neg(Box<Expr>),
    /// `Math.abs`.
    Abs(Box<Expr>),
    /// `Math.hypot`, `Math.min` or `Math.max` of the arguments in order.
    Fold(Fold, Vec<Expr>),
    /// Distance from a point to a segment.
    SegDist(Box<Expr>, Box<Expr>, Box<Expr>),
    /// Distance from a point to a polyline.
    PolyDist(Box<Expr>, Box<Expr>),
    /// Even-odd point-in-polygon, as 1 or 0.
    Inside(Box<Expr>, Box<Expr>),
    /// The centre of a box.
    Center(Box<Expr>),
    /// A fold of `expr` over the items of `over`.
    Map {
        /// `Math.min` or `Math.max`.
        fold: Fold,
        /// Points or numbers.
        over: Box<Expr>,
        /// Evaluated with each item bound to `$p`.
        expr: Box<Expr>,
    },
    /// The bound item (`None`) or one of its coordinates.
    Bound(Option<usize>),
    /// A comparison, as 1 or 0 (`false` for NaN operands).
    Cmp(Cmp, Box<Expr>, Box<Expr>),
    /// Short-circuit conjunction, as 1 or 0.
    And(Vec<Expr>),
    /// Short-circuit disjunction, as 1 or 0.
    Or(Vec<Expr>),
    /// Negation, as 1 or 0.
    Not(Box<Expr>),
    /// `t`, `f` or the action's rest frame.
    Clock(Clock),
}

/// Per-part properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartValue {
    /// Sampled flat control coordinates.
    Pts,
    /// Flattened polyline.
    Flat,
    /// Sampled visibility.
    Vis,
    /// Sampled stroke width, whatever the paint type.
    Width,
    /// Paint radius: half the width for strokes, 0 for fills.
    Radius,
}

/// Binary arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arith {
    /// `a + b`
    Add,
    /// `a - b`
    Sub,
    /// `a * b`
    Mul,
    /// `a / b`
    Div,
}

/// Variadic folds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fold {
    /// V8 `Math.hypot`.
    Hypot,
    /// `Math.min`.
    Min,
    /// `Math.max`.
    Max,
}

/// Comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmp {
    /// `a > b`
    Gt,
    /// `a >= b`
    Ge,
    /// `a < b`
    Lt,
    /// `a <= b`
    Le,
    /// `a === b`
    Eq,
}

/// Frame atoms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clock {
    /// Seconds, `f / 60`.
    Time,
    /// The frame number.
    Frame,
    /// The action's `rest` (frames).
    Rest,
}

/// An evaluated value.
#[derive(Debug, Clone)]
pub enum Val {
    /// A number.
    Num(f64),
    /// Flat numbers: coordinates, a point or bounds.
    Nums(Rc<[f64]>),
    /// A polyline.
    Points(Rc<[Point]>),
}

/// Scene access for evaluation at one frame.
pub trait Env {
    /// Evaluates a part property.
    fn part(&mut self, what: PartValue, part: &Part) -> Result<Val, VerifyError>;
    /// A frame atom.
    fn clock(&self, which: Clock) -> f64;
    /// Whether one-point polylines measure to the point (the 49 dialect).
    fn single_point(&self) -> bool;
}

fn bad(message: impl Into<String>) -> VerifyError {
    VerifyError::Rules(message.into())
}

fn int(v: &Value, what: &str) -> Result<i64, VerifyError> {
    v.as_f64()
        .filter(|x| x.fract() == 0.0)
        .map(|x| x as i64)
        .ok_or_else(|| bad(format!("{what} must be an integer")))
}

fn args<'v>(v: &'v Value, op: &str, n: Option<usize>) -> Result<&'v [Value], VerifyError> {
    let items = v
        .as_array()
        .ok_or_else(|| bad(format!("{op} takes an array")))?;
    match n {
        Some(n) if items.len() != n => Err(bad(format!("{op} takes {n} arguments"))),
        _ => Ok(items),
    }
}

fn eval_error(message: impl Into<String>) -> VerifyError {
    VerifyError::Geometry(message.into())
}

impl Expr {
    /// Parses an expression from its JSON form.
    pub fn parse(v: &Value) -> Result<Expr, VerifyError> {
        match v {
            Value::Number(n) => Ok(Expr::Num(n.as_f64().unwrap_or(f64::NAN))),
            Value::Array(items) => Ok(Expr::List(
                items.iter().map(Expr::parse).collect::<Result<_, _>>()?,
            )),
            Value::Object(map) => Self::parse_op(map),
            other => Err(bad(format!("not an expression: {other}"))),
        }
    }

    fn parse_op(map: &Map<String, Value>) -> Result<Expr, VerifyError> {
        if let Some(p) = map.get("box") {
            if let Some(extra) = map
                .keys()
                .find(|k| !matches!(k.as_str(), "box" | "of" | "paint"))
            {
                return Err(bad(format!("unknown box field {extra}")));
            }
            let flat = match map.get("of").map(Value::as_str) {
                None | Some(Some("flat")) => true,
                Some(Some("pts")) => false,
                _ => return Err(bad("box \"of\" must be \"flat\" or \"pts\"")),
            };
            let paint = match map.get("paint") {
                None => true,
                Some(v) => v
                    .as_bool()
                    .ok_or_else(|| bad("box \"paint\" must be a boolean"))?,
            };
            return Ok(Expr::Box {
                part: Part::parse(p)?,
                flat,
                paint,
            });
        }
        if let Some(m) = map.get("mapMin").or_else(|| map.get("mapMax")) {
            let fold = if map.contains_key("mapMin") {
                Fold::Min
            } else {
                Fold::Max
            };
            let spec = m
                .as_object()
                .filter(|o| o.len() == 2)
                .ok_or_else(|| bad("mapMin/mapMax take {over, expr}"))?;
            let (over, expr) = (spec.get("over"), spec.get("expr"));
            let (Some(over), Some(expr)) = (over, expr) else {
                return Err(bad("mapMin/mapMax take {over, expr}"));
            };
            if map.len() != 1 {
                return Err(bad("an expression object has exactly one operator"));
            }
            return Ok(Expr::Map {
                fold,
                over: Box::new(Expr::parse(over)?),
                expr: Box::new(Expr::parse(expr)?),
            });
        }
        let mut entries = map.iter();
        let (op, v) = match (entries.next(), entries.next()) {
            (Some(entry), None) => entry,
            _ => return Err(bad("an expression object has exactly one operator")),
        };
        let e = |v: &Value| Expr::parse(v).map(Box::new);
        let two = |v: &Value| -> Result<(Box<Expr>, Box<Expr>), VerifyError> {
            let a = args(v, op, Some(2))?;
            Ok((e(&a[0])?, e(&a[1])?))
        };
        let many = |v: &Value| -> Result<Vec<Expr>, VerifyError> {
            args(v, op, None)?.iter().map(Expr::parse).collect()
        };
        let part = |what| Ok::<_, VerifyError>(Expr::Part(what, Part::parse(v)?));
        Ok(match op.as_str() {
            "pts" => part(PartValue::Pts)?,
            "flat" => part(PartValue::Flat)?,
            "vis" => part(PartValue::Vis)?,
            "width" => part(PartValue::Width)?,
            "radius" => part(PartValue::Radius)?,
            "at" => {
                let a = args(v, op, Some(2))?;
                Expr::At(e(&a[0])?, int(&a[1], "at index")?)
            }
            "slice" => {
                let a = args(v, op, None)?;
                if !(2..=3).contains(&a.len()) {
                    return Err(bad("slice takes 2 or 3 arguments"));
                }
                let end = a.get(2).map(|x| int(x, "slice end")).transpose()?;
                Expr::Slice(e(&a[0])?, int(&a[1], "slice start")?, end)
            }
            "xs" => Expr::Axis(e(v)?, 0),
            "ys" => Expr::Axis(e(v)?, 1),
            "minOf" => Expr::Spread(Fold::Min, e(v)?),
            "maxOf" => Expr::Spread(Fold::Max, e(v)?),
            "add" | "sub" | "mul" | "div" => {
                let arith = match op.as_str() {
                    "add" => Arith::Add,
                    "sub" => Arith::Sub,
                    "mul" => Arith::Mul,
                    _ => Arith::Div,
                };
                let (a, b) = two(v)?;
                Expr::Arith(arith, a, b)
            }
            "neg" => Expr::Neg(e(v)?),
            "abs" => Expr::Abs(e(v)?),
            "not" => Expr::Not(e(v)?),
            "center" => Expr::Center(e(v)?),
            "hypot" => Expr::Fold(Fold::Hypot, many(v)?),
            "min" => Expr::Fold(Fold::Min, many(v)?),
            "max" => Expr::Fold(Fold::Max, many(v)?),
            "and" => Expr::And(many(v)?),
            "or" => Expr::Or(many(v)?),
            "segdist" => {
                let a = args(v, op, Some(3))?;
                Expr::SegDist(e(&a[0])?, e(&a[1])?, e(&a[2])?)
            }
            "polydist" => {
                let (a, b) = two(v)?;
                Expr::PolyDist(a, b)
            }
            "inside" => {
                let (a, b) = two(v)?;
                Expr::Inside(a, b)
            }
            "gt" | "ge" | "lt" | "le" | "eq" => {
                let cmp = match op.as_str() {
                    "gt" => Cmp::Gt,
                    "ge" => Cmp::Ge,
                    "lt" => Cmp::Lt,
                    "le" => Cmp::Le,
                    _ => Cmp::Eq,
                };
                let (a, b) = two(v)?;
                Expr::Cmp(cmp, a, b)
            }
            "$p" => match v {
                Value::Null => Expr::Bound(None),
                v => Expr::Bound(Some(
                    usize::try_from(int(v, "$p")?).map_err(|_| bad("$p takes 0, 1 or null"))?,
                )),
            },
            "t" => Expr::Clock(Clock::Time),
            "f" => Expr::Clock(Clock::Frame),
            "rest" => Expr::Clock(Clock::Rest),
            other => return Err(bad(format!("unknown expression operator {other}"))),
        })
    }

    /// Evaluates to a number.
    pub fn number(&self, env: &mut dyn Env, bound: Option<&Val>) -> Result<f64, VerifyError> {
        match self.eval(env, bound)? {
            Val::Num(x) => Ok(x),
            _ => Err(eval_error("expected a number, got an array")),
        }
    }

    /// JS truthiness of a numeric result: non-zero and not NaN.
    pub fn truthy(&self, env: &mut dyn Env, bound: Option<&Val>) -> Result<bool, VerifyError> {
        self.number(env, bound).map(|x| x != 0.0 && !x.is_nan())
    }

    fn nums(&self, env: &mut dyn Env, bound: Option<&Val>) -> Result<Rc<[f64]>, VerifyError> {
        match self.eval(env, bound)? {
            Val::Nums(p) => Ok(p),
            _ => Err(eval_error("expected an array of numbers")),
        }
    }

    fn point(&self, env: &mut dyn Env, bound: Option<&Val>) -> Result<Point, VerifyError> {
        let p = self.nums(env, bound)?;
        Ok([
            p.first().copied().unwrap_or(f64::NAN),
            p.get(1).copied().unwrap_or(f64::NAN),
        ])
    }

    fn polyline(&self, env: &mut dyn Env, bound: Option<&Val>) -> Result<Rc<[Point]>, VerifyError> {
        match self.eval(env, bound)? {
            Val::Points(p) => Ok(p),
            _ => Err(eval_error("expected a polyline")),
        }
    }

    /// Evaluates the expression with `bound` as `$p`.
    pub fn eval(&self, env: &mut dyn Env, bound: Option<&Val>) -> Result<Val, VerifyError> {
        let flag = |b: bool| Val::Num(if b { 1.0 } else { 0.0 });
        Ok(match self {
            Expr::Num(x) => Val::Num(*x),
            Expr::List(items) => Val::Nums(
                items
                    .iter()
                    .map(|e| e.number(env, bound))
                    .collect::<Result<Rc<[f64]>, _>>()?,
            ),
            Expr::Part(what, part) => env.part(*what, part)?,
            Expr::Box { part, flat, paint } => {
                let b = if *flat {
                    match env.part(PartValue::Flat, part)? {
                        Val::Points(p) => bounds(p.iter().copied()),
                        _ => return Err(eval_error("expected a polyline")),
                    }
                } else {
                    match env.part(PartValue::Pts, part)? {
                        Val::Nums(c) => coords_box(&c),
                        _ => return Err(eval_error("expected coordinates")),
                    }
                };
                let r = if *paint {
                    match env.part(PartValue::Radius, part)? {
                        Val::Num(r) => r,
                        _ => return Err(eval_error("expected a radius")),
                    }
                } else {
                    0.0
                };
                Val::Nums(Rc::new(pad_box(b, r)))
            }
            Expr::At(e, i) => match e.eval(env, bound)? {
                Val::Nums(v) => Val::Num(at_index(v.len(), *i).map_or(f64::NAN, |k| v[k])),
                Val::Points(v) => {
                    let k = at_index(v.len(), *i)
                        .ok_or_else(|| eval_error(format!("point index {i} out of range")))?;
                    Val::Nums(Rc::new(v[k]))
                }
                Val::Num(_) => return Err(eval_error("at() needs an array")),
            },
            Expr::Slice(e, s, end) => match e.eval(env, bound)? {
                Val::Nums(v) => Val::Nums(v[slice_range(v.len(), *s, *end)].into()),
                Val::Points(v) => Val::Points(v[slice_range(v.len(), *s, *end)].into()),
                Val::Num(_) => return Err(eval_error("slice() needs an array")),
            },
            Expr::Axis(e, axis) => match e.eval(env, bound)? {
                Val::Nums(v) => Val::Nums(v.iter().skip(*axis).step_by(2).copied().collect()),
                Val::Points(v) => Val::Nums(v.iter().map(|p| p[*axis]).collect()),
                Val::Num(_) => return Err(eval_error("xs/ys need an array")),
            },
            Expr::Spread(fold, e) => {
                let v = e.nums(env, bound)?;
                Val::Num(if *fold == Fold::Min {
                    min_of(v.iter().copied())
                } else {
                    max_of(v.iter().copied())
                })
            }
            Expr::Arith(op, a, b) => {
                let (x, y) = (a.number(env, bound)?, b.number(env, bound)?);
                Val::Num(match op {
                    Arith::Add => x + y,
                    Arith::Sub => x - y,
                    Arith::Mul => x * y,
                    Arith::Div => x / y,
                })
            }
            Expr::Neg(e) => Val::Num(-e.number(env, bound)?),
            Expr::Abs(e) => Val::Num(e.number(env, bound)?.abs()),
            Expr::Fold(op, items) => {
                let xs = items
                    .iter()
                    .map(|e| e.number(env, bound))
                    .collect::<Result<Vec<_>, _>>()?;
                Val::Num(match op {
                    Fold::Hypot => hypot(&xs),
                    Fold::Min => min_of(xs),
                    Fold::Max => max_of(xs),
                })
            }
            Expr::SegDist(p, a, b) => {
                let (p, a, b) = (
                    p.point(env, bound)?,
                    a.point(env, bound)?,
                    b.point(env, bound)?,
                );
                Val::Num(seg_dist(p, a, b))
            }
            Expr::PolyDist(p, line) => {
                let p = p.point(env, bound)?;
                let line = line.polyline(env, bound)?;
                Val::Num(poly_distance(p, &line, env.single_point()))
            }
            Expr::Inside(p, poly) => {
                let p = p.point(env, bound)?;
                flag(inside_poly(p, &poly.polyline(env, bound)?))
            }
            Expr::Center(b) => {
                let b = b.nums(env, bound)?;
                let at = |i: usize| b.get(i).copied().unwrap_or(f64::NAN);
                Val::Nums(Rc::new([(at(0) + at(2)) / 2.0, (at(1) + at(3)) / 2.0]))
            }
            Expr::Map { fold, over, expr } => {
                let items: Vec<Val> = match over.eval(env, bound)? {
                    Val::Points(p) => p.iter().map(|q| Val::Nums(Rc::new(*q))).collect(),
                    Val::Nums(v) => v.iter().map(|x| Val::Num(*x)).collect(),
                    Val::Num(_) => return Err(eval_error("mapMin/mapMax need an array")),
                };
                let values = items
                    .iter()
                    .map(|item| expr.number(env, Some(item)))
                    .collect::<Result<Vec<_>, _>>()?;
                Val::Num(if *fold == Fold::Min {
                    min_of(values)
                } else {
                    max_of(values)
                })
            }
            Expr::Bound(coordinate) => match (bound, coordinate) {
                (Some(item), None) => item.clone(),
                (Some(Val::Nums(p)), Some(i)) => Val::Num(p.get(*i).copied().unwrap_or(f64::NAN)),
                _ => return Err(eval_error("$p is not bound to a point")),
            },
            Expr::Cmp(op, a, b) => {
                let (x, y) = (a.number(env, bound)?, b.number(env, bound)?);
                flag(match op {
                    Cmp::Gt => x > y,
                    Cmp::Ge => x >= y,
                    Cmp::Lt => x < y,
                    Cmp::Le => x <= y,
                    Cmp::Eq => x == y,
                })
            }
            Expr::And(items) => {
                for e in items {
                    if !e.truthy(env, bound)? {
                        return Ok(flag(false));
                    }
                }
                flag(true)
            }
            Expr::Or(items) => {
                for e in items {
                    if e.truthy(env, bound)? {
                        return Ok(flag(true));
                    }
                }
                flag(false)
            }
            Expr::Not(e) => flag(!e.truthy(env, bound)?),
            Expr::Clock(which) => Val::Num(env.clock(*which)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct Fixed;

    impl Env for Fixed {
        fn part(&mut self, what: PartValue, part: &Part) -> Result<Val, VerifyError> {
            if part.name == "missing" {
                return Err(VerifyError::Assertion("missing".into()));
            }
            Ok(match what {
                PartValue::Pts => Val::Nums(Rc::new([1.0, 2.0, 5.0, 8.0])),
                PartValue::Flat => Val::Points(Rc::new([
                    [0.0, 0.0],
                    [4.0, 0.0],
                    [4.0, 4.0],
                    [0.0, 4.0],
                    [0.0, 0.0],
                ])),
                PartValue::Vis => Val::Num(0.5),
                PartValue::Width => Val::Num(2.5),
                PartValue::Radius => Val::Num(1.25),
            })
        }
        fn clock(&self, which: Clock) -> f64 {
            match which {
                Clock::Time => 0.5,
                Clock::Frame => 30.0,
                Clock::Rest => 62.0,
            }
        }
        fn single_point(&self) -> bool {
            false
        }
    }

    fn eval(v: Value) -> f64 {
        Expr::parse(&v).unwrap().number(&mut Fixed, None).unwrap()
    }

    #[test]
    fn evaluates_in_tree_order() {
        assert_eq!(
            eval(json!({"sub": [{"sub": [24, 1.3]}, {"at": [{"box": "token"}, 2]}]})),
            22.7 - 5.25
        );
        assert_eq!(
            eval(json!({"at": [{"box": "p", "of": "pts", "paint": false}, 3]})),
            8.0
        );
        assert_eq!(eval(json!({"at": [{"at": [{"flat": "p"}, -2]}, 1]})), 4.0);
        assert_eq!(eval(json!({"neg": {"hypot": [3, 4]}})), -5.0);
        assert_eq!(
            eval(json!({"polydist": [[2, 1], {"slice": [{"flat": "p"}, 0, 2]}]})),
            1.0
        );
        assert_eq!(eval(json!({"inside": [[2, 2], {"flat": "p"}]})), 1.0);
        assert!(eval(json!({"at": [{"pts": "p"}, 9]})).is_nan());
        assert_eq!(eval(json!({"maxOf": {"ys": {"pts": "p"}}})), 8.0);
        assert_eq!(eval(json!({"minOf": {"xs": {"flat": "p"}}})), 0.0);
        assert_eq!(
            eval(json!({"at": [{"center": {"box": "p", "paint": false}}, 1]})),
            2.0
        );
        assert_eq!(eval(json!({"segdist": [[3, 4], [0, 0], [0, 0]]})), 5.0);
        assert_eq!(eval(json!({"ge": [{"f": 1}, {"rest": 1}]})), 0.0);
        assert_eq!(eval(json!({"width": {"part": "p", "pick": "exact"}})), 2.5);
    }

    #[test]
    fn maps_bind_points_and_numbers() {
        let over_points = json!({"mapMax": {"over": {"flat": "p"}, "expr": {"hypot": [{"sub": [{"$p": 0}, 2]}, {"sub": [{"$p": 1}, 2]}]}}});
        assert_eq!(eval(over_points), 8f64.sqrt());
        let over_numbers = json!({"mapMin": {"over": {"xs": {"pts": "p"}}, "expr": {"min": [{"sub": [{"$p": null}, 5]}, {"sub": [18, {"$p": null}]}]}}});
        assert_eq!(eval(over_numbers), -4.0);
    }

    #[test]
    fn logic_short_circuits_and_errors_propagate() {
        assert_eq!(
            eval(json!({"and": [{"gt": [{"vis": "a"}, 0.9]}, {"vis": "missing"}]})),
            0.0
        );
        assert_eq!(
            eval(json!({"or": [{"lt": [1, 2]}, {"vis": "missing"}]})),
            1.0
        );
        let err = Expr::parse(&json!({"and": [1, {"vis": "missing"}]}))
            .unwrap()
            .number(&mut Fixed, None);
        assert!(err.is_err());
        for v in [
            json!({"add": [1]}),
            json!({"foo": 1}),
            json!({"at": [{"pts": "a"}, 1.5]}),
            json!("x"),
            json!({"$p": 3.5}),
        ] {
            assert!(Expr::parse(&v).is_err(), "{v}");
        }
    }
}
