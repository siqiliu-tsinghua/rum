//! Independent promotion baseline, installed obligations, and rejected frontier.

use std::collections::BTreeSet;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use super::model::*;
use super::{effects, leaves, resolve, types};

pub(super) const DEMO: &str = "fixture::accepted::root";
pub(super) const SESSION: &str = "rum_runtime::runtime::model::session::Session";

pub(super) fn proposed_boundaries(source: &Source) {
    for line in include_str!("inventory/proposed-boundaries.tsv")
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 8);
        assert!(fields.iter().all(|f| !f.is_empty()));
        assert_eq!(fields[1], "ProposedPending");
        let anchor = resolve::function(source, fields[2]).expect("live boundary anchor");
        println!("PROPOSED_BOUNDARY\t{line}\t{}", anchor.site.display());
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Region {
    pub witness: String,
    pub root: Id,
    pub covered: bool,
    pub entry: Vec<Ty>,
    pub exit: Ty,
}

pub(super) fn baseline() -> Vec<Region> {
    vec![Region {
        witness: "demo".into(),
        root: DEMO.into(),
        covered: true,
        entry: vec![Ty::Scalar("usize".into())],
        exit: Ty::Outgoing(format!("{}::Outgoing", leaves::PREFIX)),
    }]
}

pub(super) fn verify(
    source: &Source,
    records: &[Region],
    promoted: &[Region],
    leaves: &[leaves::Leaf],
) -> Result<Proof, String> {
    for expected in promoted {
        let Some(actual) = records.iter().find(|r| r.witness == expected.witness) else {
            return Err(format!("coverage record removed: {}", expected.witness));
        };
        if actual != expected {
            return Err(format!(
                "promotion changed or downgraded: {}",
                expected.witness
            ));
        }
    }
    let mut proof = Proof::default();
    for region in records.iter().filter(|r| r.covered) {
        let f = resolve::function(source, &region.root)?;
        let marker = format!("retention-region:{}", region.witness);
        let marked: Vec<_> = source
            .functions
            .values()
            .flatten()
            .filter(|f| {
                f.attrs
                    .iter()
                    .any(|a| a.path().is_ident("doc") && tokens(&a.meta).contains(&marker))
            })
            .map(|f| f.id.clone())
            .collect();
        if marked != [region.root.clone()] {
            return Err(format!(
                "root marker missing or redirected: {}",
                region.witness
            ));
        }
        proof = effects::check(source, &region.root, leaves).map_err(|e| e.to_string())?;
        let entry: Vec<_> = types::signature(source, f)?
            .into_iter()
            .map(|(_, t)| t)
            .collect();
        if entry != region.entry || types::output(source, f)? != region.exit {
            return Err(format!("entry/exit contract changed: {}", region.witness));
        }
    }
    Ok(proof)
}

#[derive(Default)]
pub(super) struct Frontier {
    pub bodies: BTreeSet<Id>,
    pub rejected: BTreeSet<Diagnostic>,
}

// Diagnostic enumeration only. It cannot return a coverage certificate and
// never treats visiting children of a rejected parent as admitting that parent.
pub(super) fn frontier(source: &Source, roots: &[Id]) -> Frontier {
    let mut report = Frontier::default();
    let mut pending: Vec<_> = roots
        .iter()
        .map(|id| (id.clone(), vec![id.clone()]))
        .collect();
    while let Some((id, witness)) = pending.pop() {
        if !report.bodies.insert(id.clone()) {
            continue;
        }
        let f = resolve::function(source, &id).expect("exact frontier root/callee");
        let mut visitor = Edges {
            source,
            function: f,
            witness,
            report: &mut report,
            pending: &mut pending,
            locals: BTreeSet::new(),
        };
        visitor.visit_block(&f.body);
    }
    report
}

struct Edges<'a> {
    source: &'a Source,
    function: &'a Function,
    witness: Vec<Id>,
    report: &'a mut Frontier,
    pending: &'a mut Vec<(Id, Vec<Id>)>,
    locals: BTreeSet<String>,
}

impl Edges<'_> {
    fn reject(
        &mut self,
        span: proc_macro2::Span,
        kind: Kind,
        target: Option<Id>,
        reason: impl Into<String>,
    ) {
        self.report.rejected.insert(Diagnostic {
            kind,
            site: Site::new(&self.function.site.file, span),
            target,
            witness: self.witness.clone(),
            reason: reason.into(),
        });
    }
}

impl<'ast> Visit<'ast> for Edges<'_> {
    fn visit_pat_ident(&mut self, pat: &'ast syn::PatIdent) {
        self.locals.insert(pat.ident.to_string());
        visit::visit_pat_ident(self, pat);
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let result = if let syn::Expr::Path(path) = &*call.func {
            if path.qself.is_some()
                || path
                    .path
                    .get_ident()
                    .is_some_and(|p| self.locals.contains(&p.to_string()))
            {
                Err("trait projection or runtime callable local".into())
            } else {
                resolve::path(self.source, &self.function.module, &path.path)
            }
        } else {
            Err("runtime callable expression".into())
        };
        match result {
            Ok(target) => match resolve::function(self.source, &target) {
                Ok(_) => {
                    let mut witness = self.witness.clone();
                    witness.push(target.clone());
                    self.pending.push((target, witness));
                }
                Err(reason) => self.reject(
                    call.span(),
                    if target.starts_with("std::vec::Vec::") {
                        Kind::RET001
                    } else {
                        Kind::RET002
                    },
                    Some(target),
                    reason,
                ),
            },
            Err(reason) => self.reject(call.span(), Kind::RET002, None, reason),
        }
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let target = if matches!(&*call.receiver, syn::Expr::Path(p) if p.path.is_ident("self")) {
            self.function
                .owner
                .as_ref()
                .map(|owner| format!("{owner}::{}", call.method))
                .filter(|id| resolve::function(self.source, id).is_ok())
        } else {
            None
        };
        self.reject(
            call.span(),
            Kind::RET002,
            target,
            "method syntax: receiver/autoref/trait effects are outside admitted resolution",
        );
        visit::visit_expr_method_call(self, call);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        self.reject(
            mac.span(),
            Kind::RET003,
            None,
            "unexpanded macro; contents are not resolved Rust",
        );
    }

    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        let reason = match expr {
            syn::Expr::ForLoop(_) => Some("implicit IntoIterator/next/iterator Drop not modeled"),
            syn::Expr::Try(_) => Some("implicit Try/FromResidual/error ownership not modeled"),
            syn::Expr::Index(_) => {
                Some("implicit Index/Deref/bounds exceptional effects not proved")
            }
            syn::Expr::Binary(_) | syn::Expr::Unary(_) | syn::Expr::Cast(_) => {
                Some("primitive operand proof or exact operator/conversion impl required")
            }
            syn::Expr::Closure(_) => {
                self.reject(expr.span(), Kind::RET002, None, "closure capture/callback effects unresolved; closure body is a rejected boundary");
                return;
            }
            _ => None,
        };
        if let Some(reason) = reason {
            self.reject(expr.span(), Kind::RET002, None, reason);
        }
        visit::visit_expr(self, expr);
    }
}
