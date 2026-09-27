//! Temporal expression evaluation, including conditional calls and captures.
use super::*;

impl Flow<'_> {
    fn callback(&mut self, c: &syn::ExprClosure) {
        let old = self.env.clone();
        let terminated = self.terminated;
        // Only shared borrows and scalar captures occur in the probe. No
        // implicit owning capture, mutable capture, or escaping storage rule.
        struct Uses(BTreeSet<String>);
        impl<'a> Visit<'a> for Uses {
            fn visit_expr_path(&mut self, p: &'a syn::ExprPath) {
                if let Some(n) = p.path.get_ident() {
                    self.0.insert(n.to_string());
                }
            }
        }
        let mut uses = Uses(BTreeSet::new());
        uses.visit_expr(&c.body);
        let parameters: BTreeSet<_> = c
            .inputs
            .iter()
            .filter_map(|p| {
                if let syn::Pat::Ident(p) = p {
                    Some(p.ident.to_string())
                } else {
                    None
                }
            })
            .collect();
        for name in uses.0.difference(&parameters) {
            if let Some(l) = old.get(name) {
                if !l.live || !l.value.ty.copyable() {
                    self.error(
                        Site::new(&self.f.site.file, c.span()),
                        Kind::RET002,
                        "closure capture must be a live scalar or shared borrow".into(),
                    );
                }
            }
        }
        for p in &c.inputs {
            if let Some(t) = self
                .report
                .bindings
                .get(&Site::new(&self.f.site.file, p.span()))
                .cloned()
            {
                self.bind(p, self.value(t, p.span()));
            }
        }
        self.terminated = false;
        self.expr(&c.body, true);
        self.env = old;
        self.terminated = terminated;
    }
    pub(super) fn expr(&mut self, e: &syn::Expr, take: bool) -> Value {
        if let Some(v) = self.read(e, take) {
            return v;
        }
        match e {
            syn::Expr::Call(c) => {
                let begin = self.pending.len();
                let mut args = vec![];
                for arg in &c.args {
                    let v = self.expr(arg, true);
                    self.pending.push(v.clone());
                    args.push(v);
                }
                self.pending.truncate(begin);
                self.hazard(c.span(), &args);
                let mut output = self.fact(e);
                if let syn::Expr::Path(p) = &*c.func {
                    if let Ok(target) = resolve::path(self.source, &self.f.module, &p.path) {
                        if target == format!("{}::string", protocol::EXPR) && args.len() == 1 {
                            output.ty = args[0].ty.clone();
                            output.provenance = args[0].provenance.clone();
                        } else if let Some((_, variant)) = resolve::standard_variant(&target) {
                            output.ty = Ty::Variants(BTreeMap::from([(
                                variant.into(),
                                args.iter()
                                    .enumerate()
                                    .map(|(i, v)| (i.to_string(), v.ty.clone()))
                                    .collect(),
                            )]));
                        }
                    }
                }
                return output;
            }
            syn::Expr::MethodCall(c) => {
                let receiver_ty = self
                    .report
                    .facts
                    .get(&Site::new(&self.f.site.file, c.receiver.span()));
                let borrowed = receiver_ty
                    .and_then(|t| protocol::method(self.source, t, &c.method.to_string()))
                    .is_some_and(|c| c.borrow_receiver);
                let recv = self.expr(&c.receiver, !borrowed);
                let begin = self.pending.len();
                self.pending.push(recv.clone());
                let mut args = vec![recv];
                for arg in &c.args {
                    let v = self.expr(arg, true);
                    self.pending.push(v.clone());
                    args.push(v);
                }
                self.pending.truncate(begin);
                let targets = self
                    .report
                    .targets
                    .get(&Site::new(&self.f.site.file, c.span()))
                    .cloned()
                    .unwrap_or_default();
                // Only these exact, validated lease/guest-input transfers may
                // consume a freshly produced unleased Expr before the call.
                if targets.iter().any(|t| {
                    ["push", "evaluate"]
                        .iter()
                        .any(|m| t.starts_with(&format!("{}::{m}::<", protocol::CONTEXT)))
                }) && args.len() == 3
                {
                    args[2].ty = Ty::Moved;
                }
                self.hazard(c.span(), &args);
                let mut output = self.fact(e);
                if targets.iter().any(|t| {
                    [protocol::EXPR, protocol::ARGS]
                        .iter()
                        .any(|owner| t == &format!("<{owner} as std::clone::Clone>::clone"))
                }) {
                    output.ty = Ty::Raw("new unleased shared occurrence".into());
                }
                return output;
            }
            syn::Expr::Reference(r) => {
                let v = self.expr(&r.expr, false);
                if r.mutability.is_some()
                    && !matches!(protocol::base(&v.exact),ExactTy::Nominal(id,_) if id==protocol::VALUES || id==protocol::CONTEXT)
                {
                    self.error(
                        Site::new(&self.f.site.file, e.span()),
                        Kind::RET002,
                        "mutable borrow has no exact charged-owner contract".into(),
                    );
                }
                if place(&r.expr).is_none() {
                    self.pending.push(v);
                }
            }
            syn::Expr::Index(i) => {
                self.expr(&i.expr, false);
                self.expr(&i.index, true);
                self.hazard(i.span(), &[]);
            }
            syn::Expr::Binary(b) => {
                let a = self.expr(&b.left, false);
                if matches!(b.op, syn::BinOp::And(_)) {
                    if self.terminated {
                        return self.fact(e);
                    }
                    let mut branches = vec![self.env.clone()]; // RHS skipped
                    let pending = self.pending.len();
                    self.expr(&b.right, false);
                    self.pending.truncate(pending);
                    if !self.terminated {
                        branches.push(self.env.clone());
                    }
                    self.join(b.span(), branches);
                    return self.fact(e);
                }
                let n = self.pending.len();
                self.pending.push(a);
                self.expr(&b.right, false);
                self.pending.truncate(n);
                if self
                    .report
                    .calls
                    .contains_key(&Site::new(&self.f.site.file, b.span()))
                {
                    self.hazard(b.span(), &[]);
                }
            }
            syn::Expr::Paren(p) => return self.expr(&p.expr, take),
            syn::Expr::Block(b) => return self.block(&b.block),
            syn::Expr::Return(r) => {
                let value = r
                    .expr
                    .as_ref()
                    .map(|e| self.expr(e, true))
                    .unwrap_or_else(|| Value::new(Ty::Unit));
                self.terminated = true;
                return value;
            }
            syn::Expr::Try(t) => {
                let input = self.expr(&t.expr, true);
                // Err returns immediately and drops every live local and
                // pending temporary (all have checked cleanup). Only Ok's
                // field continues; its ownership is moved, never reconstructed.
                let mut output = self.fact(e);
                if let Ty::Variants(vs) = input.ty {
                    if let Some(t) = vs.get("Ok").and_then(|fs| fs.get("0")) {
                        output.ty = t.clone();
                        output.provenance = input.provenance;
                    }
                }
                return output;
            }
            syn::Expr::Assign(a) => {
                let v = self.expr(&a.right, true);
                if let Some((n, fs)) = place(&a.left) {
                    if fs.is_empty() {
                        self.env.insert(
                            n,
                            Local {
                                value: v,
                                live: true,
                            },
                        );
                    }
                }
            }
            syn::Expr::If(i) => {
                self.expr(&i.cond, true);
                let entry = self.env.clone();
                let mut branches = vec![];
                let mut outputs = vec![];
                let value = self.block(&i.then_branch);
                if !self.terminated {
                    branches.push(self.env.clone());
                    outputs.push(value);
                }
                self.env = entry;
                self.terminated = false;
                let value = i
                    .else_branch
                    .as_ref()
                    .map(|(_, e)| self.expr(e, true))
                    .unwrap_or_else(|| Value::new(Ty::Unit));
                if !self.terminated {
                    branches.push(self.env.clone());
                    outputs.push(value);
                }
                self.join(i.span(), branches);
                return self.output(e, outputs);
            }
            syn::Expr::Match(m) => {
                let value = self.expr(&m.expr, true);
                let entry = self.env.clone();
                let mut branches = vec![];
                let mut outputs = vec![];
                for arm in &m.arms {
                    self.env = entry.clone();
                    self.terminated = false;
                    self.bind(&arm.pat, value.clone());
                    let value = self.expr(&arm.body, true);
                    self.env.retain(|n, _| entry.contains_key(n));
                    if !self.terminated {
                        branches.push(self.env.clone());
                        outputs.push(value);
                    }
                }
                self.join(m.span(), branches);
                return self.output(e, outputs);
            }
            syn::Expr::ForLoop(f) => {
                self.expr(&f.expr, true);
                let entry = self.env.clone();
                self.bind(&f.pat, self.value(protocol::scalar("usize"), f.pat.span()));
                self.block(&f.body);
                self.env.retain(|n, _| entry.contains_key(n));
                if !self.terminated && !same(&entry, &self.env) {
                    self.error(
                        Site::new(&self.f.site.file, f.span()),
                        Kind::RET002,
                        "loop ownership back edge has no fixed point".into(),
                    );
                }
                self.env = entry;
                self.terminated = false;
            }
            syn::Expr::Range(r) => {
                if let Some(a) = &r.start {
                    self.expr(a, true);
                }
                if let Some(b) = &r.end {
                    self.expr(b, true);
                }
            }
            syn::Expr::Closure(c) => self.callback(c),
            syn::Expr::Tuple(t) => {
                let begin = self.pending.len();
                let mut vs = vec![];
                for e in &t.elems {
                    let v = self.expr(e, true);
                    self.pending.push(v.clone());
                    vs.push(v);
                }
                self.pending.truncate(begin);
                let mut result = self.fact(e);
                result.ty = Ty::Tuple(vs.into_iter().map(|v| v.ty).collect());
                return result;
            }
            syn::Expr::Struct(s) => {
                for f in &s.fields {
                    self.expr(&f.expr, true);
                }
            }
            _ => {}
        }
        self.fact(e)
    }

    fn output(&self, e: &syn::Expr, values: Vec<Value>) -> Value {
        // A branch cannot wash away an owned payload wrapped in Expr/Result.
        // If any continuing branch is raw, conservatively retain that state.
        values
            .into_iter()
            .find(|v| types::retention(&v.ty) == Some(Kind::RET001))
            .unwrap_or_else(|| self.fact(e))
    }
}
