//! Closed source proof for S5's producer-to-cold-helper insertion route.
use super::scan::{Facts, Function, Use, text};
use std::collections::BTreeMap;
use syn::{
    Expr, Pat, Stmt,
    spanned::Spanned,
    visit::{self, Visit},
};

fn ident(expr: &Expr) -> Option<String> {
    let Expr::Path(path) = expr else { return None };
    path.path.get_ident().map(ToString::to_string)
}

fn local(stmt: &Stmt) -> Option<(&syn::Ident, &Expr)> {
    let Stmt::Local(local) = stmt else {
        return None;
    };
    let Pat::Ident(pat) = &local.pat else {
        return None;
    };
    Some((&pat.ident, &local.init.as_ref()?.expr))
}

fn witness_source(expr: &Expr) -> Option<String> {
    let Expr::Call(call) = expr else { return None };
    let Expr::Path(path) = call.func.as_ref() else {
        return None;
    };
    if path.path.segments.last()?.ident != "slot_witness" || call.args.len() != 1 {
        return None;
    }
    let Expr::Reference(reference) = &call.args[0] else {
        return None;
    };
    ident(&reference.expr)
}

fn request(pat: &Pat) -> Option<String> {
    let Pat::TupleStruct(error) = pat else {
        return None;
    };
    if !error.path.is_ident("Err") || error.elems.len() != 1 {
        return None;
    }
    let Pat::Struct(input) = &error.elems[0] else {
        return None;
    };
    if text(&input.path).replace(' ', "") != "EvalError::InputRequired" {
        return None;
    }
    input.fields.iter().find_map(|field| {
        if text(&field.member) != "request" {
            return None;
        }
        let Pat::Ident(binding) = field.pat.as_ref() else {
            return None;
        };
        Some(binding.ident.to_string())
    })
}

// Refuse aliases shadowed or reassigned before the terminal insertion.
#[derive(Default)]
struct Rebindings(Vec<String>);
impl<'a> Visit<'a> for Rebindings {
    fn visit_pat_ident(&mut self, pat: &'a syn::PatIdent) {
        self.0.push(pat.ident.to_string());
        visit::visit_pat_ident(self, pat);
    }
    fn visit_expr_assign(&mut self, expr: &'a syn::ExprAssign) {
        if let Some(name) = ident(&expr.left) {
            self.0.push(name);
        }
        visit::visit_expr_assign(self, expr);
    }
}

struct Calls<'a>(Vec<&'a Expr>);
impl<'a> Visit<'a> for Calls<'a> {
    fn visit_expr(&mut self, expr: &'a Expr) {
        if matches!(expr, Expr::Call(_) | Expr::MethodCall(_)) {
            self.0.push(expr);
        }
        visit::visit_expr(self, expr);
    }
    // An uncalled closure is not an insertion edge.
    fn visit_expr_closure(&mut self, _: &'a syn::ExprClosure) {}
}

fn parts(expr: &Expr) -> Option<(String, Vec<&Expr>, proc_macro2::Span)> {
    match expr {
        Expr::Call(call) => {
            let Expr::Path(path) = call.func.as_ref() else {
                return None;
            };
            Some((
                path.path.segments.last()?.ident.to_string(),
                call.args.iter().collect(),
                call.func.span(),
            ))
        }
        Expr::MethodCall(call) => Some((
            call.method.to_string(),
            call.args.iter().collect(),
            call.method.span(),
        )),
        _ => None,
    }
}

// Follow only named cold helpers with the witness and request passed unchanged.
// A terminal route.suspend or witness.suspend must consume those same parameters.
fn reaches_insertion(
    facts: &Facts,
    caller: &Function,
    expr: &Expr,
    witness: &str,
    request: &str,
    seen: &mut Vec<String>,
) -> bool {
    let Some((name, args, _)) = parts(expr) else {
        return false;
    };
    if let Expr::MethodCall(call) = expr {
        if name == "suspend" {
            return if ident(&call.receiver).as_deref() == Some(witness) {
                args.len() == 3 && ident(args[1]).as_deref() == Some(request)
            } else {
                ident(&call.receiver).as_deref() == Some("route")
                    && args.len() == 4
                    && ident(args[1]).as_deref() == Some(witness)
                    && ident(args[2]).as_deref() == Some(request)
            };
        }
    }
    let Some(index) = args
        .iter()
        .position(|arg| ident(arg).as_deref() == Some(witness))
    else {
        return false;
    };
    if args.get(index + 1).and_then(|arg| ident(arg)).as_deref() != Some(request) {
        return false;
    }
    let target_scope = match expr {
        Expr::Call(call) => {
            let Expr::Path(path) = call.func.as_ref() else {
                return false;
            };
            let segments: Vec<_> = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect();
            if segments.len() == 2 {
                format!("{}::{}", caller.scope, segments[0])
            } else if segments.len() == 1 {
                caller.scope.clone()
            } else {
                return false;
            }
        }
        Expr::MethodCall(call) if ident(&call.receiver).as_deref() == Some("self") => {
            caller.scope.clone()
        }
        _ => return false,
    };
    let targets: Vec<_> = facts
        .functions
        .iter()
        .filter(|f| f.scope == target_scope && f.name == name && f.cold)
        .collect();
    let [target] = targets.as_slice() else {
        return false;
    };
    if seen.contains(&target.identity) {
        return false;
    }
    let parameters: Vec<_> = target
        .signature_ast
        .inputs
        .iter()
        .filter_map(|p| {
            let syn::FnArg::Typed(p) = p else { return None };
            let Pat::Ident(pat) = p.pat.as_ref() else {
                return None;
            };
            Some(pat.ident.to_string())
        })
        .collect();
    let (Some(witness), Some(request)) = (parameters.get(index), parameters.get(index + 1)) else {
        return false;
    };
    let mut rebound = Rebindings::default();
    rebound.visit_block(&target.body);
    if rebound
        .0
        .iter()
        .any(|name| name == witness || name == request || name == "route")
    {
        return false;
    }
    seen.push(target.identity.clone());
    let mut calls = Calls(Vec::new());
    calls.visit_block(&target.body);
    let result = calls
        .0
        .into_iter()
        .any(|expr| reaches_insertion(facts, target, expr, witness, request, seen));
    seen.pop();
    result
}

struct Producer<'a> {
    facts: &'a Facts,
    function: &'a Function,
    site: &'a Use,
    witnesses: BTreeMap<String, String>,
    proved: bool,
}
impl<'a> Visit<'a> for Producer<'a> {
    fn visit_block(&mut self, block: &'a syn::Block) {
        let saved = self.witnesses.clone();
        let mut previous = None;
        for stmt in &block.stmts {
            if let Some((binding, expr)) = local(stmt) {
                let binding = binding.to_string();
                self.witnesses
                    .retain(|w, source| w != &binding && source != &binding);
                if let Some(source) = witness_source(expr) {
                    if previous.as_ref() == Some(&source) {
                        self.witnesses.insert(binding.clone(), source);
                    }
                }
                previous = Some(binding);
            } else {
                previous = None;
            }
            self.visit_stmt(stmt);
        }
        self.witnesses = saved;
    }
    fn visit_expr_match(&mut self, matched: &'a syn::ExprMatch) {
        if let Some(source) = ident(&matched.expr) {
            for arm in &matched.arms {
                if let Some(request) = request(&arm.pat) {
                    let mut rebound = Rebindings::default();
                    rebound.visit_expr(&arm.body);
                    let mut calls = Calls(Vec::new());
                    calls.visit_expr(&arm.body);
                    for expr in calls.0 {
                        let Some((_, _, span)) = parts(expr) else {
                            continue;
                        };
                        if span.start().line != self.site.site.line
                            || span.start().column + 1 != self.site.site.column
                        {
                            continue;
                        }
                        self.proved |= self.witnesses.iter().any(|(witness, binding)| {
                            binding == &source
                                && !rebound.0.iter().any(|name| {
                                    name == witness || name == binding || name == &request
                                })
                                && reaches_insertion(
                                    self.facts,
                                    self.function,
                                    expr,
                                    witness,
                                    &request,
                                    &mut Vec::new(),
                                )
                        });
                    }
                }
            }
        }
        visit::visit_expr_match(self, matched);
    }
    fn visit_expr_closure(&mut self, _: &'a syn::ExprClosure) {}
}

pub(super) fn insertion(facts: &Facts, site: &Use) -> bool {
    let Some(function) = facts.functions.iter().find(|f| f.identity == site.caller) else {
        return false;
    };
    let mut proof = Producer {
        facts,
        function,
        site,
        witnesses: BTreeMap::new(),
        proved: false,
    };
    proof.visit_block(&function.body);
    proof.proved
}
