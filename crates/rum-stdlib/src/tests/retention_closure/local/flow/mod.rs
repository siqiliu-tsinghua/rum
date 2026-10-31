//! Condition 1 is temporal. A1's place/partial-move domain is reused; a raw
//! value is legal until it crosses a call not proved inert. Previously
//! evaluated arguments and borrowed temporaries remain live until the call.
use super::super::effects::places::{child, moved, place};
use super::*;

mod expressions;
mod places;

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
    for input in &f.sig().inputs {
        let mut one = f.clone();
        let mut signature = one.sig();
        signature.inputs = [input.clone()].into_iter().collect();
        one.sig = tokens(&signature);
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
    flow.block(&f.body());
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
}

fn same(a: &Env, b: &Env) -> bool {
    a.len() == b.len()
        && a.iter().all(|(n, a)| {
            b.get(n).is_some_and(|b| {
                a.live == b.live && a.value.ty == b.value.ty && a.value.exact == b.value.exact
            })
        })
}
