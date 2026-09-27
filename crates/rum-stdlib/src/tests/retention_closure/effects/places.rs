//! Checked locals and field projections. Moving one field leaves the rest live.
//! Like MIR, a projection starts from a local; names never imply a receipt.

use super::*;

pub(in super::super) fn place(expr: &syn::Expr) -> Option<(String, Vec<String>)> {
    match expr {
        syn::Expr::Path(p) if p.qself.is_none() && p.attrs.is_empty() => {
            p.path.get_ident().map(|n| (n.to_string(), Vec::new()))
        }
        syn::Expr::Field(f) if f.attrs.is_empty() => {
            let (name, mut fields) = place(&f.base)?;
            fields.push(match &f.member {
                syn::Member::Named(n) => n.to_string(),
                syn::Member::Unnamed(i) => i.index.to_string(),
            });
            Some((name, fields))
        }
        syn::Expr::Paren(p) if p.attrs.is_empty() => place(&p.expr),
        _ => None,
    }
}

pub(in super::super) fn moved(ty: &Ty) -> bool {
    match ty {
        Ty::Moved => true,
        Ty::Fields(fs) => fs.values().any(moved),
        Ty::Tuple(ts) => ts.iter().any(moved),
        Ty::Variants(vs) => vs.values().flat_map(|fs| fs.values()).any(moved),
        _ => false,
    }
}

pub(in super::super) fn child<'a>(state: &'a mut Ty, field: &str) -> Option<&'a mut Ty> {
    match state {
        Ty::Fields(fs) => fs.get_mut(field),
        Ty::Tuple(ts) => field.parse::<usize>().ok().and_then(|n| ts.get_mut(n)),
        _ => None,
    }
}

impl Checker<'_> {
    pub(super) fn cleanup(
        &self,
        f: &Function,
        span: proc_macro2::Span,
        env: &Environment,
    ) -> Checked<()> {
        for local in env.values().filter(|l| l.live) {
            self.retain(f, span, &local.value)?;
        }
        Ok(())
    }

    pub(super) fn place_value(
        &self,
        f: &Function,
        expr: &syn::Expr,
        env: &mut Environment,
        take: bool,
    ) -> Checked<Value> {
        let (name, fields) = place(expr).ok_or_else(|| {
            self.unsupported(f, expr.span(), "borrow/place must start at a checked local")
        })?;
        let local = env.get(&name).filter(|l| l.live).ok_or_else(|| {
            self.unsupported(
                f,
                expr.span(),
                "use after ownership transfer or unbound place",
            )
        })?;
        if local.value.provenance.is_none() {
            return Err(self.unsupported(f, expr.span(), "place lacks checked provenance"));
        }
        let mut value = local.value.clone();
        let mut borrowed = false;
        for field in &fields {
            while let ExactTy::Reference(inner, _) = &value.exact {
                value.ty = types::ownership(self.source, inner);
                value.exact = (**inner).clone();
                borrowed = true;
                value
                    .provenance
                    .as_mut()
                    .unwrap()
                    .projections
                    .push("deref".into());
            }
            let schema = types::fields(self.source, &value.exact, None)
                .map_err(|r| self.unsupported(f, expr.span(), r))?;
            value.exact = schema.get(field).cloned().ok_or_else(|| {
                self.unsupported(f, expr.span(), format!("unknown named field {field}"))
            })?;
            value.ty = child(&mut value.ty, field).cloned().ok_or_else(|| {
                self.unsupported(
                    f,
                    expr.span(),
                    "field ownership is opaque or already transferred",
                )
            })?;
            value
                .provenance
                .as_mut()
                .unwrap()
                .projections
                .push(format!("field {field}"));
        }
        if moved(&value.ty) {
            return Err(self.unsupported(f, expr.span(), "use after partial ownership transfer"));
        }
        if take && !value.ty.copyable() {
            if borrowed {
                return Err(self.unsupported(f, expr.span(), "move through a borrowed place"));
            }
            let local = env.get_mut(&name).unwrap();
            if fields.is_empty() {
                local.live = false;
            } else {
                let mut state = &mut local.value.ty;
                for field in &fields {
                    state = child(state, field).unwrap();
                }
                *state = Ty::Moved; // MUTATION:partial-move-siblings
            }
        }
        Ok(value)
    }

    pub(super) fn bind_pattern(
        &self,
        f: &Function,
        pat: &syn::Pat,
        mut value: Value,
        env: &mut Environment,
    ) -> Checked<()> {
        match pat {
            syn::Pat::Ident(p) if p.by_ref.is_none() && p.subpat.is_none() => {
                let name = p.ident.to_string();
                let path: syn::Path = syn::parse_str(&name).unwrap();
                if resolve::path(self.source, &f.module, &path)
                    .ok()
                    .and_then(|id| resolve::standard_variant(&id))
                    .is_some_and(|(_, v)| v == "None")
                {
                    return self.bind_variant(f, pat, &path, Vec::new(), value, env);
                }
                if env.contains_key(&name) {
                    return Err(self.unsupported(
                        f,
                        pat.span(),
                        "shadowing or implicit binding borrow",
                    ));
                }
                value.provenance.get_or_insert(Provenance {
                    source: Site::new(&f.site.file, pat.span()),
                    basis: "checked pattern".into(),
                    projections: Vec::new(),
                });
                env.insert(name, Local { value, live: true });
                Ok(())
            }
            syn::Pat::Wild(_) => self.retain(f, pat.span(), &value),
            syn::Pat::Tuple(tuple) => {
                let ExactTy::Tuple(ts) = &value.exact else {
                    return Err(self.unsupported(f, pat.span(), "tuple pattern type mismatch"));
                };
                let Ty::Tuple(states) = &value.ty else {
                    return Err(self.unsupported(f, pat.span(), "tuple pattern state mismatch"));
                };
                if ts.len() != tuple.elems.len() {
                    return Err(self.unsupported(f, pat.span(), "tuple pattern arity mismatch"));
                }
                for ((p, t), s) in tuple.elems.iter().zip(ts).zip(states) {
                    let mut field = Value::typed(t.clone(), s.clone());
                    field.origin = value.origin.clone();
                    field.provenance = value.provenance.clone();
                    self.bind_pattern(f, p, field, env)?;
                }
                Ok(())
            }
            syn::Pat::TupleStruct(p) => self.bind_variant(
                f,
                pat,
                &p.path,
                p.elems
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (i.to_string(), p))
                    .collect(),
                value,
                env,
            ),
            syn::Pat::Path(p) => self.bind_variant(f, pat, &p.path, Vec::new(), value, env),
            syn::Pat::Struct(p) if p.rest.is_none() => self.bind_variant(
                f,
                pat,
                &p.path,
                p.fields
                    .iter()
                    .map(|field| (tokens(&field.member), &*field.pat))
                    .collect(),
                value,
                env,
            ),
            _ => Err(self.unsupported(f, pat.span(), "binding pattern has unmodeled moves")),
        }
    }

    pub(super) fn variant_name(
        &self,
        f: &Function,
        path: &syn::Path,
        exact: &ExactTy,
        span: proc_macro2::Span,
    ) -> Checked<String> {
        let target = resolve::path(self.source, &f.module, path)
            .map_err(|r| self.unsupported(f, span, r))?;
        let mut base = exact;
        while let ExactTy::Reference(t, _) = base {
            base = t;
        }
        let ExactTy::Nominal(id, _) = base else {
            return Err(self.unsupported(f, span, "variant pattern has no exact nominal base"));
        };
        target
            .strip_prefix(&format!("{id}::"))
            .map(str::to_owned)
            .filter(|v| !v.contains("::"))
            .ok_or_else(|| {
                self.unsupported(
                    f,
                    span,
                    "pattern variant resolves to a different nominal definition",
                )
            })
    }

    fn bind_variant(
        &self,
        f: &Function,
        pat: &syn::Pat,
        path: &syn::Path,
        patterns: Vec<(String, &syn::Pat)>,
        mut value: Value,
        env: &mut Environment,
    ) -> Checked<()> {
        let variant = self.variant_name(f, path, &value.exact, pat.span())?;
        let mut borrowed = None;
        while let ExactTy::Reference(t, mutable) = &value.exact {
            borrowed = Some(borrowed.unwrap_or(true) && *mutable);
            value.exact = (**t).clone();
            value.ty = types::ownership(self.source, &value.exact);
        }
        let schema = types::fields(self.source, &value.exact, Some(&variant))
            .map_err(|r| self.unsupported(f, pat.span(), r))?;
        if schema.len() != patterns.len() {
            return Err(self.unsupported(
                f,
                pat.span(),
                "pattern must account for every variant field",
            ));
        }
        let Ty::Variants(vs) = &value.ty else {
            return Err(self.unsupported(f, pat.span(), "pattern variant state is opaque"));
        };
        let states = vs
            .get(&variant)
            .ok_or_else(|| self.unsupported(f, pat.span(), "unreachable variant pattern"))?;
        for (name, p) in patterns {
            let exact = schema
                .get(&name)
                .ok_or_else(|| self.unsupported(f, p.span(), "unknown pattern field"))?
                .clone();
            let state = states
                .get(&name)
                .ok_or_else(|| self.unsupported(f, p.span(), "missing pattern field state"))?
                .clone();
            let mut field = if let Some(mutable) = borrowed {
                Value::typed(
                    ExactTy::Reference(Box::new(exact), mutable),
                    Ty::Borrow(Box::new(state), mutable),
                )
            } else {
                Value::typed(exact, state)
            };
            field.origin = value.origin.clone();
            field.provenance = value.provenance.clone();
            if let Some(provenance) = &mut field.provenance {
                provenance
                    .projections
                    .push(format!("variant {variant} field {name}"));
            }
            self.bind_pattern(f, p, field, env)?;
        }
        Ok(())
    }
}
