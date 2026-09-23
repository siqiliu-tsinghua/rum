//! Scope cleanup, explicit returns, exhaustive variants, and ownership joins.
//! Exhaustiveness and retention are separate checks: every discarded payload
//! and every live sibling still needs its cleanup/retention proof.

use super::*;

fn join(mut left: Value, right: Value) -> Option<Value> {
    if matches!(left.ty, Ty::Moved) {
        return Some(right);
    }
    if matches!(right.ty, Ty::Moved) {
        return Some(left);
    }
    if left.exact != right.exact {
        return None;
    }
    match (&mut left.ty, right.ty) {
        (Ty::Variants(a), Ty::Variants(b)) => {
            for (v, fields) in b {
                if let Some(old) = a.get(&v) {
                    if old != &fields {
                        return None;
                    }
                }
                a.insert(v, fields);
            }
        }
        (a, b) if *a == b => {}
        _ => return None,
    }
    if left.origin != right.origin {
        left.origin = None;
    }
    Some(left)
}

impl Checker<'_> {
    pub(super) fn block_expected(
        &mut self,
        f: &Function,
        block: &syn::Block,
        env: &mut Environment,
        expected: Option<&ExactTy>,
    ) -> Checked<Value> {
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
            if matches!(result.ty, Ty::Moved) {
                return Err(self.unsupported(
                    f,
                    stmt.span(),
                    "unreachable statement after explicit return",
                ));
            }
            result = Value::new(Ty::Unit);
            match stmt {
                syn::Stmt::Local(local) => {
                    types::attributes(&local.attrs)
                        .map_err(|r| self.unsupported(f, local.span(), r))?;
                    let (pat, declared) = match &local.pat {
                        syn::Pat::Type(p) => (&*p.pat, Some(&*p.ty)),
                        p => (p, None),
                    };
                    let declared = declared
                        .map(|t| {
                            types::in_function(self.source, f, t)
                                .map_err(|r| self.unsupported(f, t.span(), r))
                        })
                        .transpose()?;
                    let init = local
                        .init
                        .as_ref()
                        .ok_or_else(|| self.unsupported(f, local.span(), "uninitialized local"))?;
                    let mut value = self.expr_expected(f, &init.expr, env, declared.as_ref())?;
                    self.retain(f, init.expr.span(), &value)?;
                    if let Some(declared) = declared {
                        if value.exact != declared {
                            return Err(self.unsupported(
                                f,
                                local.pat.span(),
                                "local type differs from resolved initializer",
                            ));
                        }
                        value.provenance = Some(Provenance {
                            source: Site::new(&f.site.file, local.pat.span()),
                            basis: "annotation checked against initializer".into(),
                            projections: Vec::new(),
                        });
                    }
                    if let Some((_, otherwise)) = &init.diverge {
                        // Only a single variant with a returning failure arm is
                        // modeled; no guards, destructuring chains or diverging calls.
                        let path = match pat {
                            syn::Pat::TupleStruct(p) => &p.path,
                            syn::Pat::Path(p) => &p.path,
                            _ => {
                                return Err(self.unsupported(
                                    f,
                                    pat.span(),
                                    "let-else requires one exact variant pattern",
                                ));
                            }
                        };
                        let name = self.variant_name(f, path, &value.exact, pat.span())?;
                        let mut failure = value.clone();
                        if let Ty::Variants(vs) = &mut failure.ty {
                            vs.remove(&name);
                        } else {
                            return Err(self.unsupported(
                                f,
                                pat.span(),
                                "let-else variant ownership unresolved",
                            ));
                        }
                        self.retain(f, otherwise.span(), &failure)?;
                        let mut other = env.clone();
                        let result = self.expr_expected(f, otherwise, &mut other, expected)?;
                        if !matches!(result.ty, Ty::Moved) {
                            return Err(self.unsupported(
                                f,
                                otherwise.span(),
                                "let-else failure arm must explicitly return",
                            ));
                        }
                    } else if matches!(pat, syn::Pat::TupleStruct(_) | syn::Pat::Path(_)) {
                        return Err(self.unsupported(
                            f,
                            pat.span(),
                            "refutable local pattern without let-else",
                        ));
                    }
                    self.bind_pattern(f, pat, value, env)?;
                }
                syn::Stmt::Expr(expr, semi) => {
                    let value = self.expr_expected(
                        f,
                        expr,
                        env,
                        if semi.is_none() { expected } else { None },
                    )?;
                    self.retain(f, expr.span(), &value)?;
                    if semi.is_none() || matches!(value.ty, Ty::Moved) {
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
                syn::Stmt::Item(_) => unreachable!("preflight rejects local items"),
            }
        }
        for (name, local) in env
            .iter()
            .filter(|(name, local)| !outer.contains(*name) && local.live)
        {
            let _ = name;
            self.retain(f, block.span(), &local.value)?;
        }
        env.retain(|name, _| outer.contains(name));
        Ok(result)
    }

    pub(super) fn if_expr(
        &mut self,
        f: &Function,
        branch: &syn::ExprIf,
        env: &mut Environment,
        expected: Option<&ExactTy>,
    ) -> Checked<Value> {
        let condition = self.expr(f, &branch.cond, env)?;
        if condition.ty != Ty::Scalar("bool".into()) {
            return Err(self.unsupported(f, branch.span(), "nonprimitive branch condition"));
        }
        let mut other = env.clone();
        let yes = self.block_expected(f, &branch.then_branch, env, expected)?;
        let no = match &branch.else_branch {
            Some((_, e)) => self.expr_expected(f, e, &mut other, expected)?,
            None => Value::new(Ty::Unit),
        };
        if matches!(yes.ty, Ty::Moved) {
            *env = other;
        } else if !matches!(no.ty, Ty::Moved) && !same_state(env, &other) {
            return Err(self.unsupported(f, branch.span(), "incompatible branch ownership join"));
        }
        join(yes, no)
            .ok_or_else(|| self.unsupported(f, branch.span(), "incompatible branch ownership join"))
    }

    pub(super) fn match_expr(
        &mut self,
        f: &Function,
        branch: &syn::ExprMatch,
        env: &mut Environment,
        expected: Option<&ExactTy>,
    ) -> Checked<Value> {
        if branch
            .arms
            .iter()
            .any(|a| a.guard.is_some() || !a.attrs.is_empty())
        {
            return Err(self.unsupported(
                f,
                branch.span(),
                "match guards/attributed arms are outside admitted ownership",
            ));
        }
        let value = self.expr(f, &branch.expr, env)?;
        let mut base = &value.exact;
        while let ExactTy::Reference(t, _) = base {
            base = t;
        }
        let variants = types::variants(self.source, base)
            .map_err(|r| self.unsupported(f, branch.span(), r))?;
        if variants.is_empty() {
            return Err(self.unsupported(f, branch.span(), "match needs an exact enum"));
        }
        let mut remaining: BTreeSet<_> = variants.into_iter().collect();
        let outer: BTreeSet<_> = env.keys().cloned().collect();
        let mut result = None;
        let mut continuing = None;
        for arm in &branch.arms {
            let selected = match &arm.pat {
                syn::Pat::Ident(p) if p.by_ref.is_none() && p.subpat.is_none() => {
                    let path: syn::Path = syn::parse_str(&p.ident.to_string()).unwrap();
                    BTreeSet::from([self.variant_name(f, &path, &value.exact, p.span())?])
                }
                syn::Pat::TupleStruct(p) => {
                    BTreeSet::from([self.variant_name(f, &p.path, &value.exact, p.span())?])
                }
                syn::Pat::Path(p) => {
                    BTreeSet::from([self.variant_name(f, &p.path, &value.exact, p.span())?])
                }
                syn::Pat::Struct(p) => {
                    BTreeSet::from([self.variant_name(f, &p.path, &value.exact, p.span())?])
                }
                syn::Pat::Wild(_) => remaining.clone(),
                _ => {
                    return Err(self.unsupported(
                        f,
                        arm.pat.span(),
                        "match pattern is outside exhaustive variant grammar",
                    ));
                }
            };
            if selected.is_empty() || !selected.is_subset(&remaining) {
                return Err(self.unsupported(
                    f,
                    arm.pat.span(),
                    "duplicate/unreachable variant arm",
                ));
            }
            for variant in &selected {
                remaining.remove(variant);
            }
            let mut projected = value.clone();
            if let Ty::Variants(vs) = &mut projected.ty {
                vs.retain(|v, _| selected.contains(v));
            }
            let mut state = env.clone();
            self.bind_pattern(f, &arm.pat, projected, &mut state)?;
            let arm_value = self.expr_expected(f, &arm.body, &mut state, expected)?;
            self.retain(f, arm.body.span(), &arm_value)?;
            for (_, local) in state.iter().filter(|(n, l)| !outer.contains(*n) && l.live) {
                self.retain(f, arm.body.span(), &local.value)?;
            }
            state.retain(|n, _| outer.contains(n));
            if !matches!(arm_value.ty, Ty::Moved) {
                if continuing
                    .as_ref()
                    .is_some_and(|prior| !same_state(prior, &state))
                {
                    return Err(self.unsupported(
                        f,
                        arm.span(),
                        "incompatible branch ownership join",
                    ));
                }
                continuing = Some(state);
            }
            result = Some(match result {
                None => arm_value,
                Some(old) => join(old, arm_value).ok_or_else(|| {
                    self.unsupported(f, arm.span(), "incompatible branch ownership join")
                })?,
            });
        }
        if !remaining.is_empty() {
            return Err(self.unsupported(
                f,
                branch.span(),
                "nonexhaustive variant ownership branches",
            ));
        }
        if let Some(state) = continuing {
            *env = state;
        }
        result.ok_or_else(|| self.unsupported(f, branch.span(), "empty match"))
    }
}
