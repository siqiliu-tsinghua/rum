//! Live catalog routes, including explicit rejected macro/conditional edges.

use super::model::*;
use super::resolve;
use std::collections::{BTreeMap, BTreeSet};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Attachment {
    pub site: Site,
    pub spelling: String,
    pub target: Result<Id, String>,
}

impl Attachment {
    pub fn route(&self) -> String {
        match &self.target {
            Ok(id) => id.clone(),
            Err(reason) => format!(
                "REJECTED {} {}: {reason}",
                self.site.display(),
                self.spelling
            ),
        }
    }
}

pub(super) fn catalog_routes(source: &Source) -> BTreeMap<String, BTreeSet<Id>> {
    attachments(source)
        .into_iter()
        .map(|(name, roots)| (name, roots.iter().map(Attachment::route).collect()))
        .collect()
}

pub(super) fn attachments(source: &Source) -> BTreeMap<String, BTreeSet<Attachment>> {
    struct Catalog<'a> {
        source: &'a Source,
        module: &'a str,
        file: &'a str,
        routes: &'a mut BTreeMap<String, BTreeSet<Attachment>>,
    }
    impl<'ast> Visit<'ast> for Catalog<'_> {
        fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
            if let syn::Expr::Path(p) = &*call.func {
                if resolve::path(self.source, self.module, &p.path)
                    .ok()
                    .is_some_and(|id| {
                        id.starts_with("rum_runtime::runtime::metadata::BuiltinSpec::")
                    })
                {
                    if let Some(syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(name),
                        ..
                    })) = call.args.first()
                    {
                        let entry = self.routes.entry(name.value()).or_default();
                        let constructor = resolve::path(self.source, self.module, &p.path).unwrap();
                        let definition = resolve::function(self.source, &constructor).unwrap();
                        for (parameter, arg) in definition.sig().inputs.iter().zip(&call.args) {
                            let syn::FnArg::Typed(parameter) = parameter else {
                                continue;
                            };
                            let syn::Pat::Ident(parameter) = &*parameter.pat else {
                                continue;
                            };
                            if parameter.ident != "handler" {
                                continue;
                            }
                            let result = match arg {
                                syn::Expr::Path(p) => {
                                    resolve::path(self.source, self.module, &p.path).and_then(
                                        |id| resolve::function(self.source, &id).map(|_| id),
                                    )
                                }
                                _ => Err("catalog handler is not a static function path".into()),
                            };
                            entry.insert(Attachment {
                                site: Site::new(self.file, arg.span()),
                                spelling: tokens(arg),
                                target: result,
                            });
                        }
                    }
                }
            }
            visit::visit_expr_call(self, call);
        }
    }
    let mut routes = BTreeMap::new();
    for (module, file, expr) in &source.constants {
        Catalog {
            source,
            module,
            file,
            routes: &mut routes,
        }
        .visit_expr(&ast::<syn::Expr>(expr));
    }
    // The installed accounting companion is a distinct source route, checked
    // against the actual install_core body, not inferred from the builtin name.
    let install = resolve::function(source, "rum_stdlib::install_core").unwrap();
    struct Companion<'a> {
        source: &'a Source,
        module: &'a str,
        file: &'a str,
        found: BTreeSet<Attachment>,
    }
    impl<'ast> Visit<'ast> for Companion<'_> {
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            if call.method == "register_accounting_builtin" {
                if let Some(syn::Expr::Path(p)) = call.args.last() {
                    let id = resolve::path(self.source, self.module, &p.path)
                        .expect("resolve installed companion");
                    resolve::function(self.source, &id).expect("companion body exists");
                    self.found.insert(Attachment {
                        site: Site::new(self.file, p.span()),
                        spelling: tokens(p),
                        target: Ok(id),
                    });
                }
            }
            visit::visit_expr_method_call(self, call);
        }
    }
    let mut companion = Companion {
        source,
        module: &install.module,
        file: &install.site.file,
        found: BTreeSet::new(),
    };
    companion.visit_block(&install.body());
    assert_eq!(
        companion
            .found
            .iter()
            .map(Attachment::route)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["rum_stdlib::control_flow::eval_accounted_compound_expression".into()])
    );
    routes
        .entry("CompoundExpression".into())
        .or_default()
        .extend(companion.found);
    routes
}

pub(super) fn installed() {
    let mut expected = BTreeMap::new();
    for spec in rum_runtime::core_builtin_specs()
        .iter()
        .chain(crate::tests::stdlib_catalog_specs())
    {
        expected.insert(spec.name, *spec);
    }
    let mut session = rum_runtime::Session::new();
    crate::install_core(&mut session);
    let actual: BTreeMap<_, _> = session
        .registered_builtin_specs()
        .map(|s| (s.name, *s))
        .collect();
    assert_eq!(
        actual, expected,
        "installed handler routes and full specs must equal catalog records"
    );
}
