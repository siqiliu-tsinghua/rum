//! Condition 1 is temporal. A1's place/partial-move domain is reused; a raw
//! value is legal until it crosses a call not proved inert. Previously
//! evaluated arguments and borrowed temporaries remain live until the call.
use super::super::effects::places::{child, moved, place};
use super::*;

type Env = BTreeMap<String, Local>;

pub(super) fn check(source: &Source, f: &Function, report: &mut Report) {
    let mut flow = Flow {
        source,
        f,
        report,
        env: Env::new(),
        pending: vec![],
        terminated: false,
    };
    for input in &f.sig.inputs {
        let mut one = f.clone();
        one.sig.inputs = [input.clone()].into_iter().collect();
        if let Ok(parameters) = types::exact_parameters(source, &one) {
            for (name, ty) in parameters {
                let v = flow.value(ty, input.span());
                flow.env.insert(
                    name,
                    Local {
                        value: v,
                        live: true,
                    },
                );
            }
        }
    }
    flow.block(&f.body);
}

struct Flow<'a> {
    source: &'a Source,
    f: &'a Function,
    report: &'a mut Report,
    env: Env,
    pending: Vec<Value>,
    terminated: bool,
}

impl Flow<'_> {
    fn value(&self, exact: ExactTy, span: proc_macro2::Span) -> Value {
        let mut v = Value::typed(exact.clone(), cleanup::ownership(self.source, &exact));
        v.provenance = Some(Provenance {
            source: Site::new(&self.f.site.file, span),
            basis: "local ownership".into(),
            projections: vec![],
        });
        v
    }
    fn unknown(&self) -> Value {
        Value::typed(ExactTy::Unit, Ty::Unchecked("rejected syntax/type".into()))
    }
    fn fact(&self, e: &syn::Expr) -> Value {
        self.report
            .facts
            .get(&Site::new(&self.f.site.file, e.span()))
            .map(|t| self.value(t.clone(), e.span()))
            .unwrap_or_else(|| self.unknown())
    }
    fn error(&mut self, site: Site, kind: Kind, reason: String) {
        self.report.rejected.insert(Diagnostic {
            kind,
            site,
            target: None,
            witness: vec![self.f.id.clone()],
            reason,
        });
    }
    fn hazard(&mut self, span: proc_macro2::Span, arguments: &[Value]) {
        let site = Site::new(&self.f.site.file, span);
        if self.report.calls.get(&site) == Some(&false) {
            return;
        }
        let live: Vec<_> = self
            .env
            .values()
            .filter(|l| l.live)
            .map(|l| l.value.clone())
            .chain(self.pending.iter().cloned())
            .chain(arguments.iter().cloned())
            .collect();
        for v in live {
            if types::retention(&v.ty) == Some(Kind::RET001) {
                let origin = v
                    .provenance
                    .as_ref()
                    .map(|p| p.source.clone())
                    .unwrap_or_else(|| site.clone());
                self.error(origin, Kind::RET001, format!("raw owned container produced: {} is live across call at {} (callee not proved inert)", v.exact.spelling(), site.display()));
            }
        }
    }
    fn read(&mut self, e: &syn::Expr, take: bool) -> Option<Value> {
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
    fn bind(&mut self, pat: &syn::Pat, value: Value) {
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
    fn block(&mut self, b: &syn::Block) -> Value {
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
                        if matches!(&*init.expr, syn::Expr::Reference(r) if place(&r.expr).is_none())
                        {
                            self.error(
                                Site::new(&self.f.site.file, l.span()),
                                Kind::RET002,
                                "extended lifetime of a borrowed temporary is not admitted".into(),
                            );
                        }
                        self.bind(&l.pat, v);
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
    fn join(&mut self, span: proc_macro2::Span, branches: Vec<Env>) {
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
    fn expr(&mut self, e: &syn::Expr, take: bool) -> Value {
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

fn same(a: &Env, b: &Env) -> bool {
    a.len() == b.len()
        && a.iter().all(|(n, a)| {
            b.get(n).is_some_and(|b| {
                a.live == b.live && a.value.ty == b.value.ty && a.value.exact == b.value.exact
            })
        })
}
