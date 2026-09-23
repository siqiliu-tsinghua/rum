//! Fail-closed expression grammar and ownership transfer interpreter.
//!
//! Every accepted expression gets a fact, including callee path expressions.
//! A loop is admitted only when its complete abstract back edge equals its
//! entry state: this proves a fixed point, not a sampled execution. Recursive
//! bodies use their exact signatures, and all SCC members must finish checking;
//! no raw local, literal allocation allowance or pending callback can cross a
//! recursive edge. There is no partial-success certificate.

use std::collections::{BTreeMap, BTreeSet};

use syn::spanned::Spanned;

use super::model::*;
use super::{leaves, resolve, types};

#[path = "effects/branches.rs"]
mod branches;
#[path = "effects/call.rs"]
mod call;
#[path = "effects/construct.rs"]
mod construct;
#[path = "effects/method.rs"]
mod method;
#[path = "effects/places.rs"]
mod places;

type Environment = BTreeMap<String, Local>;

pub(super) fn check(source: &Source, root: &str, contracts: &[leaves::Leaf]) -> Checked<Proof> {
    let mut checker = Checker {
        source,
        contracts,
        proof: Proof::default(),
        stack: Vec::new(),
        active: BTreeSet::new(),
    };
    checker.body(root)?;
    Ok(checker.proof)
}

struct Checker<'a> {
    source: &'a Source,
    contracts: &'a [leaves::Leaf],
    proof: Proof,
    stack: Vec<Id>,
    active: BTreeSet<LeafKey>,
}

impl Checker<'_> {
    fn error(
        &self,
        f: &Function,
        span: proc_macro2::Span,
        kind: Kind,
        target: Option<Id>,
        reason: impl Into<String>,
    ) -> Diagnostic {
        Diagnostic {
            kind,
            site: Site::new(&f.site.file, span),
            target,
            witness: self.stack.clone(),
            reason: reason.into(),
        }
    }

    fn unsupported(
        &self,
        f: &Function,
        span: proc_macro2::Span,
        reason: impl Into<String>,
    ) -> Diagnostic {
        self.error(f, span, Kind::RET002, None, reason)
    }

    fn retain(&self, f: &Function, span: proc_macro2::Span, value: &Value) -> Checked<()> {
        if let Some(kind) = types::retention(&value.ty) {
            return Err(self.error(
                f,
                span,
                kind,
                value.origin.clone(),
                match kind {
                    Kind::RET004 => "callback payload has no retention transfer",
                    Kind::RET002 => {
                        return Err(self.unsupported(
                            f,
                            span,
                            format!("unclassified cleanup: {:?}", value.ty),
                        ));
                    }
                    _ => "unleased owned payload",
                },
            )); // MUTATION:callback-retention
        }
        Ok(())
    }

    fn body(&mut self, id: &str) -> Checked<()> {
        let f = resolve::function(self.source, id)
            .map_err(|reason| Diagnostic {
                kind: Kind::RET002,
                site: Site::new("<root>", proc_macro2::Span::call_site()),
                target: Some(id.into()),
                witness: self.stack.clone(),
                reason,
            })?
            .clone();
        self.body_function(f)
    }

    fn body_function(&mut self, f: Function) -> Checked<()> {
        let key = super::methods::key(&f);
        if self.proof.instances.contains(&key) || self.active.contains(&key) {
            return Ok(());
        }
        self.stack.push(f.id.clone());
        self.active.insert(key.clone());
        let parameters = types::signature(self.source, &f)
            .map_err(|reason| self.unsupported(&f, f.sig.span(), reason))?;
        if !self.source.modules[&f.module].hazards.is_empty() {
            return Err(self.unsupported(
                &f,
                f.sig.span(),
                "module contains an unclassified item-generating expansion",
            ));
        }
        let mut env = Environment::new();
        let exact = types::exact_parameters(self.source, &f)
            .map_err(|r| self.unsupported(&f, f.sig.span(), r))?;
        for ((name, ty), (_, exact)) in parameters.into_iter().zip(exact) {
            let mut value = Value::typed(exact, ty);
            value.provenance = Some(Provenance {
                source: f.site.clone(),
                basis: format!("parameter {name}"),
                projections: Vec::new(),
            });
            self.retain(&f, f.sig.span(), &value)?;
            env.insert(name, Local { value, live: true });
        }
        let output = types::exact_output(self.source, &f)
            .map_err(|r| self.unsupported(&f, f.sig.span(), r))?;
        let value = self.block_expected(&f, &f.body, &mut env, Some(&output))?;
        self.cleanup(&f, f.body.span(), &env)?;
        self.retain(&f, f.body.span(), &value)?;
        if value.exact != output && !matches!(value.ty, Ty::Moved) {
            return Err(self.unsupported(
                &f,
                f.body.span(),
                format!(
                    "return ownership/type mismatch: {:?} versus {output:?}",
                    value.exact
                ),
            ));
        }
        self.active.remove(&key);
        self.stack.pop();
        self.proof.bodies.insert(f.id.clone());
        self.proof.instances.insert(key);
        Ok(())
    }

    fn fact(&mut self, f: &Function, expr: &syn::Expr, value: &Value, effect: &'static str) {
        self.proof.facts.push(Fact {
            site: Site::new(&f.site.file, expr.span()),
            ty: value.ty.clone(),
            exact: (!matches!(value.ty, Ty::Moved)).then(|| value.exact.clone()),
            provenance: value.provenance.clone(),
            effect,
        });
    }

    fn expr(&mut self, f: &Function, expr: &syn::Expr, env: &mut Environment) -> Checked<Value> {
        self.expr_expected(f, expr, env, None)
    }

    fn expr_expected(
        &mut self,
        f: &Function,
        expr: &syn::Expr,
        env: &mut Environment,
        expected: Option<&ExactTy>,
    ) -> Checked<Value> {
        let (mut value, effect) = match expr {
            syn::Expr::Lit(lit) if lit.attrs.is_empty() => {
                let ty = match &lit.lit {
                    syn::Lit::Bool(_) => Ty::Scalar("bool".into()),
                    syn::Lit::Int(i) if matches!(i.suffix(), "usize" | "u8" | "i32") => {
                        Ty::Scalar(i.suffix().into())
                    }
                    _ => {
                        return Err(self.unsupported(
                            f,
                            expr.span(),
                            "literal requires an explicit admitted primitive type",
                        ));
                    }
                };
                (Value::new(ty), "primitive")
            }
            syn::Expr::Path(path) if path.attrs.is_empty() && path.qself.is_none() => {
                if path
                    .path
                    .get_ident()
                    .is_some_and(|n| env.contains_key(&n.to_string()))
                {
                    (
                        self.place_value(f, expr, env, true)?,
                        "move-or-primitive-copy",
                    )
                } else {
                    (
                        self.construct(f, &path.path, &[], expr.span(), env, expected)?,
                        "unit-variant",
                    )
                }
            }
            syn::Expr::Field(_) => (self.place_value(f, expr, env, true)?, "field-projection"),
            syn::Expr::Call(call) if call.attrs.is_empty() => {
                (self.call(f, call, env, expected)?, "resolved-call")
            }
            syn::Expr::Match(branch) if branch.attrs.is_empty() => (
                self.match_expr(f, branch, env, expected)?,
                "exhaustive-match",
            ),
            syn::Expr::Return(ret) if ret.attrs.is_empty() => {
                let output = types::exact_output(self.source, f)
                    .map_err(|r| self.unsupported(f, ret.span(), r))?;
                let value = match &ret.expr {
                    Some(e) => self.expr_expected(f, e, env, Some(&output))?,
                    None => Value::new(Ty::Unit),
                };
                self.retain(f, ret.span(), &value)?;
                if value.exact != output {
                    return Err(self.unsupported(f, ret.span(), "explicit return type mismatch"));
                }
                self.cleanup(f, ret.span(), env)?;
                (Value::typed(output, Ty::Moved), "explicit-return")
            }
            syn::Expr::MethodCall(call) => (self.method(f, call, env)?, "proved-inherent-method"),
            syn::Expr::Macro(m) => {
                return Err(self.error(
                    f,
                    m.mac.span(),
                    Kind::RET003,
                    None,
                    "unsupported macro expansion",
                )); // MUTATION:macro-expansion
            }
            syn::Expr::Tuple(tuple) if tuple.attrs.is_empty() => {
                let mut fields = Vec::new();
                let mut exact = Vec::new();
                let mut origin = None;
                for (i, field) in tuple.elems.iter().enumerate() {
                    let expected = match expected {
                        Some(ExactTy::Tuple(ts)) => ts.get(i),
                        _ => None,
                    };
                    let value = self.expr_expected(f, field, env, expected)?;
                    exact.push(value.exact);
                    fields.push(value.ty);
                    origin = origin.or(value.origin);
                }
                let mut value = Value::typed(
                    if fields.is_empty() {
                        ExactTy::Unit
                    } else {
                        ExactTy::Tuple(exact)
                    },
                    if fields.is_empty() {
                        Ty::Unit
                    } else {
                        Ty::Tuple(fields)
                    },
                );
                value.origin = origin;
                (value, "closed-tuple")
            }
            syn::Expr::Paren(p) if p.attrs.is_empty() => {
                (self.expr_expected(f, &p.expr, env, expected)?, "group")
            }
            syn::Expr::Block(b) if b.attrs.is_empty() && b.label.is_none() => (
                self.block_expected(f, &b.block, env, expected)?,
                "scope-and-cleanup",
            ),
            syn::Expr::Binary(binary) if binary.attrs.is_empty() => {
                let left = self.expr(f, &binary.left, env)?;
                let right = self.expr(f, &binary.right, env)?;
                if left.ty != right.ty || !matches!(left.ty, Ty::Scalar(_)) {
                    return Err(self.unsupported(
                        f,
                        expr.span(),
                        "operator needs equal proved primitive operands",
                    ));
                }
                let ty = match binary.op {
                    syn::BinOp::Add(_) | syn::BinOp::Sub(_) | syn::BinOp::Mul(_)
                        if left.ty != Ty::Scalar("bool".into()) =>
                    {
                        left.ty
                    }
                    syn::BinOp::Eq(_)
                    | syn::BinOp::Ne(_)
                    | syn::BinOp::Lt(_)
                    | syn::BinOp::Le(_)
                    | syn::BinOp::Gt(_)
                    | syn::BinOp::Ge(_) => Ty::Scalar("bool".into()),
                    _ => {
                        return Err(self.unsupported(
                            f,
                            expr.span(),
                            "operator/desugaring not modeled",
                        ));
                    }
                };
                (Value::new(ty), "primitive-operator;panic-private-excluded")
            }
            syn::Expr::Assign(assign) if assign.attrs.is_empty() => {
                let syn::Expr::Path(path) = &*assign.left else {
                    return Err(self.unsupported(
                        f,
                        expr.span(),
                        "only a whole local can be assigned",
                    ));
                };
                let Some(name) = path.path.get_ident().map(ToString::to_string) else {
                    return Err(self.unsupported(f, expr.span(), "assignment place is not local"));
                };
                let old = env
                    .get(&name)
                    .ok_or_else(|| self.unsupported(f, expr.span(), "assignment to unbound local"))?
                    .value
                    .clone();
                let value = self.expr(f, &assign.right, env)?;
                self.retain(f, assign.right.span(), &value)?;
                if value.exact != old.exact || value.ty != old.ty {
                    return Err(self.unsupported(
                        f,
                        expr.span(),
                        "assignment changes the ownership class",
                    ));
                }
                self.fact(f, &assign.left, &old, "whole-place;classified-old-Drop");
                env.insert(name, Local { value, live: true });
                (Value::new(Ty::Unit), "whole-place-transfer")
            }
            syn::Expr::While(loop_) if loop_.attrs.is_empty() && loop_.label.is_none() => {
                let entry = env.clone();
                let condition = self.expr(f, &loop_.cond, env)?;
                if condition.ty != Ty::Scalar("bool".into()) || !same_state(&entry, env) {
                    return Err(self.unsupported(
                        f,
                        expr.span(),
                        "loop condition changes ownership",
                    ));
                }
                let result = self.block_expected(f, &loop_.body, env, Some(&ExactTy::Unit))?;
                if result.ty != Ty::Unit || !same_state(&entry, env) {
                    return Err(self.unsupported(
                        f,
                        expr.span(),
                        "loop ownership back edge has no fixed point",
                    ));
                }
                self.proof.loop_invariants += 1;
                (Value::new(Ty::Unit), "loop-fixed-point")
            }
            syn::Expr::If(branch) if branch.attrs.is_empty() => {
                (self.if_expr(f, branch, env, expected)?, "branch-join")
            }
            syn::Expr::Reference(reference) if reference.attrs.is_empty() => {
                let local = self.place_value(f, &reference.expr, env, false)?;
                if reference.mutability.is_some() && !matches!(local.ty, Ty::Retained(_)) {
                    return Err(self.unsupported(
                        f,
                        expr.span(),
                        "mutable borrow has no exact charged-owner contract",
                    ));
                }
                self.fact(f, &reference.expr, &local, "borrow-place");
                let mut value = Value::typed(
                    ExactTy::Reference(Box::new(local.exact), reference.mutability.is_some()),
                    Ty::Borrow(Box::new(local.ty), reference.mutability.is_some()),
                );
                value.provenance = local.provenance;
                (value, "checked-borrow;receipt-not-inferred")
            }
            // This is the admitted syntax specification. In particular, no
            // default child visit can classify Index/Try/For/Closure/Unsafe,
            // aggregate field moves, casts, generators or new syn variants.
            _ => {
                return Err(self.unsupported(
                    f,
                    expr.span(),
                    format!("unmodeled expression: {}", tokens(expr)),
                ));
            }
        };
        if value.provenance.is_none() {
            value.provenance = Some(Provenance {
                source: Site::new(&f.site.file, expr.span()),
                basis: effect.into(),
                projections: Vec::new(),
            });
        }
        self.fact(f, expr, &value, effect);
        Ok(value)
    }
}

fn same_state(left: &Environment, right: &Environment) -> bool {
    left.len() == right.len()
        && left.iter().all(|(name, a)| {
            right.get(name).is_some_and(|b| {
                a.live == b.live && a.value.ty == b.value.ty && a.value.exact == b.value.exact
            })
        })
}
