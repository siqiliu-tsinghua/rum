//! Place ownership, borrowed bindings, scoped cleanup and branch joins.
use super::*;

impl Flow<'_> {
    pub(super) fn read(&mut self, e: &syn::Expr, take: bool) -> Option<Value> {
        let (name, fields) = place(e)?;
        let local = self.env.get(&name)?;
        if !local.live {
            self.error(
                Site::new(&self.f.site.file, e.span()),
                Kind::RET002,
                "use after ownership transfer".into(),
            );
            return Some(self.unknown());
        }
        let mut value = local.value.clone();
        let mut borrowed = false;
        for field in &fields {
            while let ExactTy::Reference(t, _) = &value.exact {
                value.exact = (**t).clone();
                value.ty = cleanup::ownership(self.source, &value.exact);
                borrowed = true;
            }
            value.exact = types::fields(self.source, &value.exact, None)
                .ok()?
                .remove(field)?;
            value.ty = child(&mut value.ty, field)?.clone();
        }
        if moved(&value.ty) {
            self.error(
                Site::new(&self.f.site.file, e.span()),
                Kind::RET002,
                "use after partial ownership transfer".into(),
            );
        }
        if take && !value.ty.copyable() {
            if borrowed {
                self.error(
                    Site::new(&self.f.site.file, e.span()),
                    Kind::RET002,
                    "move through borrowed place".into(),
                );
            } else if fields.is_empty() {
                self.env.get_mut(&name)?.live = false;
            } else {
                let mut state = &mut self.env.get_mut(&name)?.value.ty;
                for field in &fields {
                    state = child(state, field)?;
                }
                *state = Ty::Moved;
            }
        }
        Some(value)
    }
    pub(super) fn bind(&mut self, pat: &syn::Pat, value: Value) {
        match pat {
            syn::Pat::Ident(p) if p.by_ref.is_none() && p.subpat.is_none() => {
                if self.env.contains_key(&p.ident.to_string()) {
                    self.error(
                        Site::new(&self.f.site.file, p.span()),
                        Kind::RET002,
                        "shadowed ownership place is outside the admitted surface".into(),
                    );
                }
                self.env
                    .insert(p.ident.to_string(), Local { value, live: true });
            }
            syn::Pat::Type(p) => self.bind(&p.pat, value),
            syn::Pat::Tuple(p) => {
                if let (ExactTy::Tuple(ts), Ty::Tuple(states)) = (&value.exact, &value.ty) {
                    for ((p, t), s) in p.elems.iter().zip(ts).zip(states) {
                        let mut v = value.clone();
                        v.exact = t.clone();
                        v.ty = s.clone();
                        self.bind(p, v);
                    }
                }
            }
            syn::Pat::TupleStruct(p) => {
                if let Ok(id) = resolve::path(self.source, &self.f.module, &p.path) {
                    if let Some((_, v)) = resolve::standard_variant(&id) {
                        if let Ok(fields) = types::fields(self.source, &value.exact, Some(v)) {
                            for (i, p) in p.elems.iter().enumerate() {
                                if let Some(t) = fields.get(&i.to_string()) {
                                    let mut payload = self.value(t.clone(), p.span());
                                    if let Ty::Variants(states) = &value.ty {
                                        if let Some(state) =
                                            states.get(v).and_then(|f| f.get(&i.to_string()))
                                        {
                                            payload.ty = state.clone();
                                            payload.provenance = value.provenance.clone();
                                        }
                                    }
                                    self.bind(p, payload);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pub(super) fn block(&mut self, b: &syn::Block) -> Value {
        let names: BTreeSet<_> = self.env.keys().cloned().collect();
        let mut result = Value::new(Ty::Unit);
        for stmt in &b.stmts {
            if self.terminated {
                break;
            }
            let pending = self.pending.len();
            match stmt {
                syn::Stmt::Local(l) => {
                    if let Some(init) = &l.init {
                        // `_` does not move a place. Keep the whole owner for
                        // patterns with wildcards; this is conservative for
                        // mixed patterns, never a premature cleanup receipt.
                        struct Wild(bool);
                        impl<'a> Visit<'a> for Wild {
                            fn visit_pat_wild(&mut self, _: &'a syn::PatWild) {
                                self.0 = true;
                            }
                        }
                        let mut wild = Wild(false);
                        wild.visit_pat(&l.pat);
                        let v = self.expr(&init.expr, !wild.0);
                        if let Some((_, otherwise)) = &init.diverge {
                            let success = self.env.clone();
                            let pending = self.pending.clone();
                            let terminated = self.terminated;
                            self.expr(otherwise, true);
                            // Failure diverges (checked by typing); its moves
                            // and cleanup cannot alter the success environment.
                            self.env = success;
                            self.pending = pending;
                            self.terminated = terminated;
                            if let Ok(bindings) =
                                reads::borrowed_call(self.source, self.f, &l.pat, Some(&v.exact))
                            {
                                for (pat, ty) in bindings {
                                    self.bind(pat, self.value(ty, pat.span()));
                                }
                            }
                        }
                        if matches!(&*init.expr, syn::Expr::Reference(r) if place(&r.expr).is_none())
                        {
                            self.error(
                                Site::new(&self.f.site.file, l.span()),
                                Kind::RET002,
                                "extended lifetime of a borrowed temporary is not admitted".into(),
                            );
                        }
                        if init.diverge.is_none() {
                            self.bind(&l.pat, v);
                        }
                    }
                }
                syn::Stmt::Expr(e, semi) => {
                    result = self.expr(e, true);
                    if semi.is_some() {
                        result = Value::new(Ty::Unit);
                    }
                }
                _ => {}
            }
            self.pending.truncate(pending);
        }
        self.env.retain(|n, _| names.contains(n));
        result
    }
    pub(super) fn join(&mut self, span: proc_macro2::Span, branches: Vec<Env>) {
        if let Some(first) = branches.first() {
            if branches.iter().skip(1).any(|b| !same(first, b)) {
                self.error(
                    Site::new(&self.f.site.file, span),
                    Kind::RET002,
                    "branch ownership states differ".into(),
                );
            }
            self.env = first.clone();
            self.terminated = false;
        } else {
            self.terminated = true;
        }
    }
}
