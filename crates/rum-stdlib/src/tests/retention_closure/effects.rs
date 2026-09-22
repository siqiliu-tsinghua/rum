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
    active: BTreeSet<Id>,
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
        if self.proof.bodies.contains(id) || self.active.contains(id) {
            return Ok(());
        }
        self.stack.push(id.into());
        self.active.insert(id.into());
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
        for (name, ty) in parameters {
            let value = Value::new(ty);
            self.retain(&f, f.sig.span(), &value)?;
            env.insert(name, Local { value, live: true });
        }
        let value = self.block(&f, &f.body, &mut env)?;
        self.retain(&f, f.body.span(), &value)?;
        let output = types::output(self.source, &f)
            .map_err(|reason| self.unsupported(&f, f.sig.span(), reason))?;
        if output != value.ty {
            return Err(self.unsupported(
                &f,
                f.body.span(),
                format!(
                    "return ownership/type mismatch: {:?} versus {output:?}",
                    value.ty
                ),
            ));
        }
        self.active.remove(id);
        self.stack.pop();
        self.proof.bodies.insert(id.into());
        Ok(())
    }

    fn fact(&mut self, f: &Function, expr: &syn::Expr, value: &Value, effect: &'static str) {
        self.proof.facts.push(Fact {
            site: Site::new(&f.site.file, expr.span()),
            ty: value.ty.clone(),
            effect,
        });
    }

    fn block(&mut self, f: &Function, block: &syn::Block, env: &mut Environment) -> Checked<Value> {
        // Items have block-wide scope in Rust, including before their textual
        // declaration. Reject the whole scope before resolving any expression.
        for stmt in &block.stmts {
            if matches!(stmt, syn::Stmt::Item(_)) {
                return Err(self.unsupported(
                    f,
                    stmt.span(),
                    "block-local items/imports are outside the admitted surface",
                ));
            }
        }
        let outer: BTreeSet<_> = env.keys().cloned().collect();
        let mut result = Value::new(Ty::Unit);
        for stmt in &block.stmts {
            result = Value::new(Ty::Unit);
            match stmt {
                syn::Stmt::Local(local) => {
                    types::attributes(&local.attrs)
                        .map_err(|r| self.unsupported(f, local.span(), r))?;
                    let (pat, declared) = match &local.pat {
                        syn::Pat::Type(p) => (&*p.pat, Some(&*p.ty)),
                        pat => (pat, None),
                    };
                    let syn::Pat::Ident(name) = pat else {
                        return Err(self.unsupported(
                            f,
                            pat.span(),
                            "binding pattern has unmodeled moves",
                        ));
                    };
                    if name.by_ref.is_some()
                        || name.subpat.is_some()
                        || env.contains_key(&name.ident.to_string())
                    {
                        return Err(self.unsupported(
                            f,
                            pat.span(),
                            "shadowing or implicit binding borrow",
                        ));
                    }
                    let Some(init) = &local.init else {
                        return Err(self.unsupported(f, local.span(), "uninitialized local"));
                    };
                    if init.diverge.is_some() {
                        return Err(self.unsupported(
                            f,
                            local.span(),
                            "let-else ownership join is not modeled",
                        ));
                    }
                    let value = self.expr(f, &init.expr, env)?;
                    self.retain(f, init.expr.span(), &value)?;
                    if let Some(declared) = declared {
                        let t = types::ty(self.source, &f.module, declared)
                            .map_err(|r| self.unsupported(f, declared.span(), r))?;
                        if t != value.ty {
                            return Err(self.unsupported(
                                f,
                                declared.span(),
                                "local type differs from resolved initializer",
                            ));
                        }
                    }
                    env.insert(name.ident.to_string(), Local { value, live: true });
                }
                syn::Stmt::Expr(expr, semi) => {
                    let value = self.expr(f, expr, env)?;
                    self.retain(f, expr.span(), &value)?;
                    if semi.is_none() {
                        result = value;
                    }
                }
                syn::Stmt::Macro(m) => {
                    return Err(self.error(
                        f,
                        m.mac.span(),
                        Kind::RET003,
                        None,
                        "unsupported macro expansion",
                    ));
                }
                syn::Stmt::Item(_) => unreachable!("preflight rejects all local items"),
            }
        }
        // All admitted owned types have checked allocation-free destruction.
        // Rust compilation supplies the additional borrow/lifetime proof.
        env.retain(|name, _| outer.contains(name));
        Ok(result)
    }

    fn expr(&mut self, f: &Function, expr: &syn::Expr, env: &mut Environment) -> Checked<Value> {
        let (value, effect) = match expr {
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
            syn::Expr::Path(path)
                if path.attrs.is_empty()
                    && path.qself.is_none()
                    && path.path.segments.len() == 1 =>
            {
                let name = path.path.segments[0].ident.to_string();
                let Some(local) = env.get_mut(&name) else {
                    return Err(self.unsupported(
                        f,
                        expr.span(),
                        "not a local value; stored function/constant values are not admitted",
                    ));
                };
                if !local.live {
                    return Err(self.unsupported(f, expr.span(), "use after ownership transfer"));
                }
                let value = local.value.clone();
                if !value.ty.copyable() {
                    local.live = false;
                }
                (value, "move-or-primitive-copy")
            }
            syn::Expr::Call(call) if call.attrs.is_empty() => {
                (self.call(f, call, env)?, "resolved-call")
            }
            syn::Expr::MethodCall(_) => {
                return Err(self.unsupported(
                    f,
                    expr.span(),
                    "method dispatch requires autoderef/autoref/trait resolution",
                )); // MUTATION:method-dispatch
            }
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
                for field in &tuple.elems {
                    let value = self.expr(f, field, env)?;
                    self.retain(f, field.span(), &value)?;
                    fields.push(value.ty);
                }
                (
                    Value::new(if fields.is_empty() {
                        Ty::Unit
                    } else {
                        Ty::Tuple(fields)
                    }),
                    "closed-tuple",
                )
            }
            syn::Expr::Paren(p) if p.attrs.is_empty() => (self.expr(f, &p.expr, env)?, "group"),
            syn::Expr::Block(b) if b.attrs.is_empty() && b.label.is_none() => {
                (self.block(f, &b.block, env)?, "scope-and-cleanup")
            }
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
                if value.ty != old.ty {
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
                let result = self.block(f, &loop_.body, env)?;
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
                let condition = self.expr(f, &branch.cond, env)?;
                if condition.ty != Ty::Scalar("bool".into()) {
                    return Err(self.unsupported(f, expr.span(), "nonprimitive branch condition"));
                }
                let mut other = env.clone();
                let yes = self.block(f, &branch.then_branch, env)?;
                let no = match &branch.else_branch {
                    Some((_, e)) => self.expr(f, e, &mut other)?,
                    None => Value::new(Ty::Unit),
                };
                if yes.ty != no.ty || !same_state(env, &other) {
                    return Err(self.unsupported(
                        f,
                        expr.span(),
                        "incompatible branch ownership join",
                    ));
                }
                (yes, "branch-join")
            }
            syn::Expr::Reference(reference) if reference.attrs.is_empty() => {
                let syn::Expr::Path(path) = &*reference.expr else {
                    return Err(self.unsupported(
                        f,
                        expr.span(),
                        "borrow must name a live whole local",
                    ));
                };
                let Some(name) = path.path.get_ident().map(ToString::to_string) else {
                    return Err(self.unsupported(f, expr.span(), "borrow place unresolved"));
                };
                let local = env
                    .get(&name)
                    .filter(|v| v.live)
                    .ok_or_else(|| self.unsupported(f, expr.span(), "borrow outlives owner"))?;
                self.retain(f, expr.span(), &local.value)?;
                self.fact(f, &reference.expr, &local.value, "borrow-place");
                (
                    Value::new(Ty::Borrow(
                        Box::new(local.value.ty.clone()),
                        reference.mutability.is_some(),
                    )),
                    "charged-borrow",
                )
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
        self.fact(f, expr, &value, effect);
        Ok(value)
    }

    fn call(
        &mut self,
        f: &Function,
        call: &syn::ExprCall,
        env: &mut Environment,
    ) -> Checked<Value> {
        let syn::Expr::Path(path) = &*call.func else {
            return Err(self.unsupported(f, call.func.span(), "runtime callable value"));
        };
        if path.qself.is_some() || !path.attrs.is_empty() {
            return Err(self.unsupported(f, path.span(), "trait projection or attributed callee"));
        }
        if path
            .path
            .get_ident()
            .is_some_and(|name| env.contains_key(&name.to_string()))
        {
            return Err(self.unsupported(f, path.span(), "callable local shadows the function"));
        }
        let target = resolve::path(self.source, &f.module, &path.path)
            .map_err(|r| self.unsupported(f, path.span(), r))?;
        if [
            "std::vec::Vec::",
            "std::boxed::Box::",
            "std::string::String::",
            "std::collections::BTreeMap::",
            "std::collections::HashMap::",
            "std::collections::HashSet::",
        ]
        .iter()
        .any(|prefix| target.starts_with(prefix))
        {
            return Err(self.error(
                f,
                call.span(),
                Kind::RET001,
                Some(target),
                "unleased owned payload",
            )); // MUTATION:raw-allocation
        }
        let callee = match resolve::function(self.source, &target) {
            Ok(callee) => callee.clone(),
            Err(reason) => {
                return Err(self.error(f, call.span(), Kind::RET002, Some(target), reason)); // MUTATION:external-dispatch
            }
        };
        if path.path.segments.iter().any(|s| !s.arguments.is_empty()) {
            return Err(self.unsupported(f, path.span(), "generic call substitution not modeled"));
        }
        let leaf = self
            .contracts
            .iter()
            .find(|leaf| leaf.id == target)
            .cloned();
        let parameters = types::signature(self.source, &callee)
            .map_err(|r| self.unsupported(f, call.span(), r))?;
        if parameters.len() != call.args.len() {
            return Err(self.unsupported(f, call.span(), "exact call arity mismatch"));
        }
        let mut arguments = Vec::new();
        for (arg, (_, expected)) in call.args.iter().zip(&parameters) {
            let value = self.expr(f, arg, env)?;
            if value.ty != *expected {
                return Err(self.unsupported(f, arg.span(), "argument ownership/type mismatch"));
            }
            if leaf.as_ref().is_none_or(|l| l.operation != "adopt") {
                self.retain(f, arg.span(), &value)?;
            }
            arguments.push(value);
        }
        let mut value = Value::new(
            types::output(self.source, &callee).map_err(|r| self.unsupported(f, call.span(), r))?,
        );
        value.origin = Some(target.clone());
        self.fact(
            f,
            &call.func,
            &Value::new(Ty::Fixed(target.clone())),
            "resolved-function-item",
        );
        if let Some(leaf) = leaf {
            let used = BTreeSet::from([target.clone()]);
            leaves::validate(self.source, std::slice::from_ref(&leaf), &used)
                .map_err(|r| self.unsupported(f, call.span(), r))?;
            if leaf.operation == "adopt" {
                self.proof.adoptions += 1;
            }
            if leaf.operation == "send" {
                self.proof.transfers += 1;
            }
            self.proof.leaves.insert(target);
        } else {
            // Body closure is automatic, including unmarked helper extraction.
            self.body(&target)?; // MUTATION:helper-closure
        }
        Ok(value)
    }
}

fn same_state(left: &Environment, right: &Environment) -> bool {
    left.len() == right.len()
        && left.iter().all(|(name, a)| {
            right
                .get(name)
                .is_some_and(|b| a.live == b.live && a.value.ty == b.value.ty)
        })
}
