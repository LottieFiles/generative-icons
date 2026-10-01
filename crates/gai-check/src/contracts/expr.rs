//! `compare` expressions (verify.md §8.5): a JSON AST evaluated in tree order, left operand
//! first, so legacy float expressions reproduce bit for bit.
//!
//! ```text
//! EXPR := number | [EXPR, …]                       array literal (a point)
//!       | {"pts": PART} | {"flat": PART} | {"vis": PART}
//!       | {"box": PART, "of"?: "flat"|"pts", "paint"?: bool}   [l, t, r, b]; default flat, painted
//!       | {"at": [EXPR, i]} | {"slice": [EXPR, s, e?]}           JS .at() / .slice()
//!       | {"add"|"sub"|"mul"|"div": [EXPR, EXPR]} | {"neg": EXPR} | {"abs": EXPR}
//!       | {"hypot"|"min"|"max": [EXPR, …]}                      V8 Math.hypot / Math.min / Math.max
//!       | {"polydist": [POINT, POLY]} | {"inside": [POINT, POLY]}   inside → 1 | 0
//!       | {"gt"|"ge"|"lt"|"le": [EXPR, EXPR]} | {"and"|"or": [EXPR, …]} | {"not": EXPR}   → 1 | 0
//! PART := "name"   (resolved as "<icon>-<name>" with the dialect's picker)
//! ```

use super::ContractError;
use super::geom::{Point, bounds, inside_poly, pad_box, poly_dist};
use super::jsnum::{at_index, hypot, max_of, min_of, slice_range};
use serde_json::{Map, Value};
use std::rc::Rc;

/// A parsed expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A number literal.
    Num(f64),
    /// An array literal; every item must evaluate to a number.
    List(Vec<Expr>),
    /// The part's sampled flat control coordinates.
    Pts(String),
    /// The part's flattened polyline.
    Flat(String),
    /// The part's sampled visibility.
    Vis(String),
    /// Bounds `[l, t, r, b]` of the part.
    Box {
        /// Part name.
        part: String,
        /// Bounds of the flattened polyline (`true`) or of the control coordinates.
        flat: bool,
        /// Grow by the paint radius.
        paint: bool,
    },
    /// JS `.at(i)`.
    At(Box<Expr>, i64),
    /// JS `.slice(start, end)`.
    Slice(Box<Expr>, i64, Option<i64>),
    /// Binary arithmetic.
    Arith(Arith, Box<Expr>, Box<Expr>),
    /// Unary minus.
    Neg(Box<Expr>),
    /// `Math.abs`.
    Abs(Box<Expr>),
    /// `Math.hypot`, `Math.min` or `Math.max` over the arguments in order.
    Fold(FoldOp, Vec<Expr>),
    /// Distance from a point to a polyline.
    PolyDist(Box<Expr>, Box<Expr>),
    /// Even-odd point-in-polygon, as 1 or 0.
    Inside(Box<Expr>, Box<Expr>),
    /// A comparison, as 1 or 0.
    Cmp(CmpOp, Box<Expr>, Box<Expr>),
    /// Short-circuit conjunction, as 1 or 0.
    And(Vec<Expr>),
    /// Short-circuit disjunction, as 1 or 0.
    Or(Vec<Expr>),
    /// Negation, as 1 or 0.
    Not(Box<Expr>),
}

/// Binary arithmetic operators.
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

/// Variadic numeric folds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldOp {
    /// V8 `Math.hypot`.
    Hypot,
    /// `Math.min`.
    Min,
    /// `Math.max`.
    Max,
}

/// Comparison operators (`false` for NaN operands, as in JS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    /// `a > b`
    Gt,
    /// `a >= b`
    Ge,
    /// `a < b`
    Lt,
    /// `a <= b`
    Le,
}

impl CmpOp {
    fn test(self, a: f64, b: f64) -> bool {
        match self {
            CmpOp::Gt => a > b,
            CmpOp::Ge => a >= b,
            CmpOp::Lt => a < b,
            CmpOp::Le => a <= b,
        }
    }
}

/// An evaluated value.
#[derive(Debug, Clone)]
pub enum Val {
    /// A number.
    Num(f64),
    /// Flat numbers: sampled coordinates, a point or bounds.
    Nums(Rc<[f64]>),
    /// A polyline.
    Points(Rc<[Point]>),
}

/// Scene access for expression evaluation at one frame.
pub trait Env {
    /// Sampled flat control coordinates of a part.
    fn coords(&mut self, part: &str) -> Result<Rc<[f64]>, ContractError>;
    /// Flattened polyline of a part.
    fn flat(&mut self, part: &str) -> Result<Rc<[Point]>, ContractError>;
    /// Sampled visibility of a part.
    fn visibility(&mut self, part: &str) -> Result<f64, ContractError>;
    /// Paint radius of a part (half the stroke width; 0 for fills).
    fn radius(&mut self, part: &str) -> Result<f64, ContractError>;
}

fn bad(message: impl Into<String>) -> ContractError {
    ContractError::Rules(message.into())
}

fn int(v: &Value, what: &str) -> Result<i64, ContractError> {
    v.as_f64()
        .filter(|x| super::jsnum::is_integer(*x))
        .map(|x| x as i64)
        .ok_or_else(|| bad(format!("{what} must be an integer")))
}

fn part(v: &Value, op: &str) -> Result<String, ContractError> {
    v.as_str()
        .map(str::to_string)
        .ok_or_else(|| bad(format!("{op} takes a part name")))
}

fn args<'v>(v: &'v Value, op: &str, n: Option<usize>) -> Result<&'v [Value], ContractError> {
    let items = v
        .as_array()
        .ok_or_else(|| bad(format!("{op} takes an array")))?;
    match n {
        Some(n) if items.len() != n => Err(bad(format!("{op} takes {n} arguments"))),
        _ => Ok(items),
    }
}

impl Expr {
    /// Parses an expression from its JSON form.
    pub fn parse(v: &Value) -> Result<Expr, ContractError> {
        match v {
            Value::Number(n) => Ok(Expr::Num(n.as_f64().unwrap_or(f64::NAN))),
            Value::Array(items) => Ok(Expr::List(
                items.iter().map(Expr::parse).collect::<Result<_, _>>()?,
            )),
            Value::Object(map) => Self::parse_op(map),
            other => Err(bad(format!("not an expression: {other}"))),
        }
    }

    fn parse_op(map: &Map<String, Value>) -> Result<Expr, ContractError> {
        if let Some(p) = map.get("box") {
            if let Some(extra) = map
                .keys()
                .find(|k| !matches!(k.as_str(), "box" | "of" | "paint"))
            {
                return Err(bad(format!("unknown box field {extra}")));
            }
            let flat = match map.get("of").map(|v| v.as_str()) {
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
                part: part(p, "box")?,
                flat,
                paint,
            });
        }
        let mut entries = map.iter();
        let (op, v) = match (entries.next(), entries.next()) {
            (Some(entry), None) => entry,
            _ => return Err(bad("an expression object has exactly one operator")),
        };
        let e = |v: &Value| Expr::parse(v).map(Box::new);
        let two = |v: &Value| -> Result<(Box<Expr>, Box<Expr>), ContractError> {
            let a = args(v, op, Some(2))?;
            Ok((e(&a[0])?, e(&a[1])?))
        };
        let many = |v: &Value| -> Result<Vec<Expr>, ContractError> {
            args(v, op, None)?.iter().map(Expr::parse).collect()
        };
        Ok(match op.as_str() {
            "pts" => Expr::Pts(part(v, op)?),
            "flat" => Expr::Flat(part(v, op)?),
            "vis" => Expr::Vis(part(v, op)?),
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
            "hypot" => Expr::Fold(FoldOp::Hypot, many(v)?),
            "min" => Expr::Fold(FoldOp::Min, many(v)?),
            "max" => Expr::Fold(FoldOp::Max, many(v)?),
            "and" => Expr::And(many(v)?),
            "or" => Expr::Or(many(v)?),
            "polydist" => {
                let (a, b) = two(v)?;
                Expr::PolyDist(a, b)
            }
            "inside" => {
                let (a, b) = two(v)?;
                Expr::Inside(a, b)
            }
            "gt" | "ge" | "lt" | "le" => {
                let cmp = match op.as_str() {
                    "gt" => CmpOp::Gt,
                    "ge" => CmpOp::Ge,
                    "lt" => CmpOp::Lt,
                    _ => CmpOp::Le,
                };
                let (a, b) = two(v)?;
                Expr::Cmp(cmp, a, b)
            }
            other => return Err(bad(format!("unknown expression operator {other}"))),
        })
    }

    /// Evaluates to a number.
    pub fn number(&self, env: &mut dyn Env) -> Result<f64, ContractError> {
        match self.eval(env)? {
            Val::Num(x) => Ok(x),
            _ => Err(ContractError::Expr(
                "expected a number, got an array".into(),
            )),
        }
    }

    /// JS truthiness of a numeric result: non-zero and not NaN.
    pub fn truthy(&self, env: &mut dyn Env) -> Result<bool, ContractError> {
        self.number(env).map(|x| x != 0.0 && !x.is_nan())
    }

    fn point(&self, env: &mut dyn Env) -> Result<Point, ContractError> {
        match self.eval(env)? {
            Val::Nums(p) if p.len() >= 2 => Ok([p[0], p[1]]),
            _ => Err(ContractError::Expr("expected a point".into())),
        }
    }

    fn polyline(&self, env: &mut dyn Env) -> Result<Rc<[Point]>, ContractError> {
        match self.eval(env)? {
            Val::Points(p) => Ok(p),
            _ => Err(ContractError::Expr("expected a polyline".into())),
        }
    }

    /// Evaluates the expression.
    pub fn eval(&self, env: &mut dyn Env) -> Result<Val, ContractError> {
        let flag = |b: bool| Val::Num(if b { 1.0 } else { 0.0 });
        Ok(match self {
            Expr::Num(x) => Val::Num(*x),
            Expr::List(items) => Val::Nums(
                items
                    .iter()
                    .map(|e| e.number(env))
                    .collect::<Result<Rc<[f64]>, _>>()?,
            ),
            Expr::Pts(p) => Val::Nums(env.coords(p)?),
            Expr::Flat(p) => Val::Points(env.flat(p)?),
            Expr::Vis(p) => Val::Num(env.visibility(p)?),
            Expr::Box { part, flat, paint } => {
                let b = if *flat {
                    bounds(env.flat(part)?.iter().copied())
                } else {
                    let c = env.coords(part)?;
                    let xs = c.iter().step_by(2).copied();
                    let ys = c.iter().skip(1).step_by(2).copied();
                    [
                        min_of(xs.clone()),
                        min_of(ys.clone()),
                        max_of(xs),
                        max_of(ys),
                    ]
                };
                let r = if *paint { env.radius(part)? } else { 0.0 };
                Val::Nums(Rc::new(pad_box(b, r)))
            }
            Expr::At(e, i) => match e.eval(env)? {
                Val::Nums(v) => Val::Num(at_index(v.len(), *i).map_or(f64::NAN, |k| v[k])),
                Val::Points(v) => {
                    let k = at_index(v.len(), *i).ok_or_else(|| {
                        ContractError::Expr(format!("point index {i} out of range"))
                    })?;
                    Val::Nums(Rc::new(v[k]))
                }
                Val::Num(_) => return Err(ContractError::Expr("at() needs an array".into())),
            },
            Expr::Slice(e, s, end) => match e.eval(env)? {
                Val::Nums(v) => Val::Nums(v[slice_range(v.len(), *s, *end)].into()),
                Val::Points(v) => Val::Points(v[slice_range(v.len(), *s, *end)].into()),
                Val::Num(_) => return Err(ContractError::Expr("slice() needs an array".into())),
            },
            Expr::Arith(op, a, b) => {
                let (x, y) = (a.number(env)?, b.number(env)?);
                Val::Num(match op {
                    Arith::Add => x + y,
                    Arith::Sub => x - y,
                    Arith::Mul => x * y,
                    Arith::Div => x / y,
                })
            }
            Expr::Neg(e) => Val::Num(-e.number(env)?),
            Expr::Abs(e) => Val::Num(e.number(env)?.abs()),
            Expr::Fold(op, items) => {
                let xs = items
                    .iter()
                    .map(|e| e.number(env))
                    .collect::<Result<Vec<_>, _>>()?;
                Val::Num(match op {
                    FoldOp::Hypot => hypot(&xs),
                    FoldOp::Min => min_of(xs),
                    FoldOp::Max => max_of(xs),
                })
            }
            Expr::PolyDist(p, line) => {
                let p = p.point(env)?;
                Val::Num(poly_dist(p, &line.polyline(env)?))
            }
            Expr::Inside(p, poly) => {
                let p = p.point(env)?;
                flag(inside_poly(p, &poly.polyline(env)?))
            }
            Expr::Cmp(op, a, b) => {
                let (x, y) = (a.number(env)?, b.number(env)?);
                flag(op.test(x, y))
            }
            Expr::And(items) => {
                for e in items {
                    if !e.truthy(env)? {
                        return Ok(flag(false));
                    }
                }
                flag(true)
            }
            Expr::Or(items) => {
                for e in items {
                    if e.truthy(env)? {
                        return Ok(flag(true));
                    }
                }
                flag(false)
            }
            Expr::Not(e) => flag(!e.truthy(env)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct Fixed;

    impl Env for Fixed {
        fn coords(&mut self, _: &str) -> Result<Rc<[f64]>, ContractError> {
            Ok(Rc::new([1.0, 2.0, 5.0, 8.0]))
        }
        fn flat(&mut self, _: &str) -> Result<Rc<[Point]>, ContractError> {
            Ok(Rc::new([
                [0.0, 0.0],
                [4.0, 0.0],
                [4.0, 4.0],
                [0.0, 4.0],
                [0.0, 0.0],
            ]))
        }
        fn visibility(&mut self, part: &str) -> Result<f64, ContractError> {
            if part == "missing" {
                Err(ContractError::MissingPart(part.into()))
            } else {
                Ok(0.5)
            }
        }
        fn radius(&mut self, _: &str) -> Result<f64, ContractError> {
            Ok(1.25)
        }
    }

    fn eval(v: Value) -> f64 {
        Expr::parse(&v).unwrap().number(&mut Fixed).unwrap()
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
        assert_eq!(eval(json!({"max": [{"abs": -3}, 2]})), 3.0);
    }

    #[test]
    fn logic_short_circuits() {
        assert_eq!(
            eval(json!({"and": [{"gt": [{"vis": "a"}, 0.9]}, {"vis": "missing"}]})),
            0.0
        );
        assert_eq!(
            eval(json!({"or": [{"lt": [1, 2]}, {"vis": "missing"}]})),
            1.0
        );
        assert_eq!(eval(json!({"not": {"ge": [1, 2]}})), 1.0);
        let err = Expr::parse(&json!({"and": [1, {"vis": "missing"}]}))
            .unwrap()
            .number(&mut Fixed);
        assert!(matches!(err, Err(ContractError::MissingPart(_))));
    }

    #[test]
    fn rejects_malformed_expressions() {
        for v in [
            json!({"add": [1]}),
            json!({"foo": 1}),
            json!({"at": [{"pts": "a"}, 1.5]}),
            json!({"add": [1, 2], "sub": [1, 2]}),
            json!("x"),
        ] {
            assert!(Expr::parse(&v).is_err(), "{v}");
        }
    }
}
