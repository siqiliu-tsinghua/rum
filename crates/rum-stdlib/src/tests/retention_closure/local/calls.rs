//! Call result types come from signatures. Selection is A1/A2; marks are a
//! separate obligation. A return signature never grants a retention receipt.

use super::*;

impl Lint<'_> {
    pub(super) fn call(
        &mut self,
        c: &syn::ExprCall,
        expected: Option<&ExactTy>,
    ) -> Option<ExactTy> {
        let syn::Expr::Path(p) = &*c.func else {
            self.reject(
                c.span(),
                Kind::RET002,
                None,
                "runtime callable is not admitted",
            );
            self.expr(&c.func, None);
            for arg in &c.args {
                self.expr(arg, None);
            }
            return None;
        };
        if p.qself.is_some()
            || p.path
                .get_ident()
                .is_some_and(|n| self.locals.contains_key(&n.to_string()))
        {
            self.reject(
                p.span(),
                Kind::RET002,
                None,
                "trait projection or callable local is not admitted",
            );
            for arg in &c.args {
                self.expr(arg, None);
            }
            return None;
        }
        let target = match resolve::path(self.source(), &self.f.module, &p.path) {
            Ok(id) => id,
            Err(reason) => {
                self.reject(p.span(), Kind::RET002, None, reason);
                for arg in &c.args {
                    self.expr(arg, None);
                }
                // A1 expands type aliases even when value-path resolution is
                // rejected. Preserve the exact diagnostic Self return fact.
                let mut owner = p.path.clone();
                let member = owner.segments.pop()?.into_value().ident.to_string();
                owner.segments.pop_punct();
                let ty = types::exact(
                    self.source(),
                    &self.f.module,
                    &syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: owner,
                    }),
                )
                .ok()?;
                if matches!(&ty, ExactTy::Nominal(id,_) if id=="std::vec::Vec") && member == "new" {
                    return Some(ty);
                }
                return None;
            }
        };
        if resolve::standard_variant(&target).is_some() {
            self.report
                .calls
                .insert(Site::new(&self.f.site.file, c.span()), false);
            let exact = expected.cloned().or_else(|| {
                self.constructors
                    .get(&Site::new(&self.f.site.file, c.span()))
                    .cloned()
            });
            let (_, variant) = resolve::standard_variant(&target).unwrap();
            let schema = exact
                .as_ref()
                .and_then(|t| types::fields(self.source(), t, Some(variant)).ok());
            for (i, arg) in c.args.iter().enumerate() {
                self.expr(arg, schema.as_ref().and_then(|s| s.get(&i.to_string())));
            }
            return exact;
        }
        if target == protocol::SCOPED {
            return self.scoped(c, p);
        }
        if [option_boundaries::SNAPSHOT, option_boundaries::DRIVE].contains(&target.as_str()) {
            return self.option_boundary(c, p, &target);
        }
        if target == format!("{}::string", protocol::EXPR) {
            if c.args.len() != 1 || p.path.segments.iter().any(|s| !s.arguments.is_empty()) {
                return self.no(
                    c.span(),
                    "Expr::string requires one String and no explicit generic dispatch",
                );
            }
            let ty = self.expr(&c.args[0], None);
            if ty != Some(protocol::nominal("std::string::String")) {
                self.reject(
                    c.span(),
                    Kind::RET002,
                    None,
                    "Expr::string contract requires exactly String, no Into dispatch",
                );
                return None;
            }
            self.call_mark(c.span(), &format!("{target}::<std::string::String>"));
            return Some(protocol::nominal(protocol::EXPR));
        }
        let mut mode = self
            .f
            .substitutions
            .values()
            .find(|t| types::is_mode(t))
            .cloned();
        // Inference is restricted to the protocol's positional context/value
        // parameters; all arguments are checked against this same instance.
        if mode.is_none() {
            for arg in &c.args {
                if let syn::Expr::Path(p) = arg {
                    if let Some(Some(ExactTy::Nominal(id, args))) = p
                        .path
                        .get_ident()
                        .and_then(|n| self.locals.get(&n.to_string()))
                    {
                        if [protocol::CONTEXT, protocol::VALUES].contains(&id.as_str())
                            && args.len() == 1
                            && types::is_mode(&args[0])
                        {
                            mode = Some(args[0].clone());
                            break;
                        }
                    }
                }
            }
        }
        self.call_mark(c.span(), &target);
        let signature = resolve::function(self.source(), &target)
            .ok()
            .map(|callee| {
                types::with_mode(self.source(), callee, mode.as_ref())
                    .unwrap_or_else(|_| callee.clone())
            });
        let output = if let Some(callee) = &signature {
            if p.path.segments.iter().any(|s| !s.arguments.is_empty())
                || c.args.len() != callee.sig().inputs.len()
            {
                return self.no(
                    c.span(),
                    "free call generic arguments or arity differ from the concrete signature",
                );
            }
            self.report.facts.insert(
                Site::new(&self.f.site.file, p.span()),
                ExactTy::Function(target.clone()),
            );
            if !callee.sig().generics.params.is_empty() || !callee.impl_generics().params.is_empty()
            {
                self.reject(
                    c.span(),
                    Kind::RET002,
                    Some(target.clone()),
                    "generic free call not admitted",
                );
            }
            types::exact_output(self.source(), callee).ok()
        } else {
            self.std_output(p, &target, expected)
        };
        for (i, arg) in c.args.iter().enumerate() {
            let parameter =
                signature
                    .as_ref()
                    .and_then(|f| match f.sig().inputs.iter().nth(i)? {
                        syn::FnArg::Typed(a) => types::in_function(self.source(), f, &a.ty).ok(),
                        _ => None,
                    });
            let previous = self.argument_hazard.take();
            if !self.inert.contains(&target) {
                self.argument_hazard = Some(Site::new(&self.f.site.file, c.span()));
            }
            let value = self.expr(arg, parameter.as_ref());
            self.argument_hazard = previous;
            if parameter.is_some()
                && value != parameter
                && !self.shared_expr_coercion(arg, value.as_ref(), parameter.as_ref())
            {
                self.reject(
                    arg.span(),
                    Kind::RET002,
                    None,
                    "call argument differs from concrete parameter (coercion is not admitted)",
                );
            }
        }
        output
    }

    pub(super) fn method(&mut self, c: &syn::ExprMethodCall) -> Option<ExactTy> {
        let claim = self.methods.get(&methods::site(self.f, c)).cloned();
        let receiver = self.expr(&c.receiver, None);
        if protocol::competing_trait(self.source(), self.f, &c.method.to_string()) {
            return self.no(
                c.span(),
                "concrete method contract cannot exclude an imported trait candidate",
            );
        }
        if let Some(ty) = &receiver {
            if c.method == "and_then" {
                return self.adopt_call(c, ty);
            }
            if c.method == "suspend" {
                return self.suspend_call(c, ty);
            }
        }
        if let Some(call) = receiver
            .as_ref()
            .and_then(|t| protocol::method(self.source(), t, &c.method.to_string()))
        {
            if c.turbofish.is_some() || c.args.len() != call.inputs.len() {
                self.reject(
                    c.span(),
                    Kind::RET002,
                    None,
                    "concrete contract call arity or generic arguments differ",
                );
                return None;
            }
            if matches!(receiver.as_ref().map(protocol::base), Some(ExactTy::Nominal(id, _)) if id == protocol::ARGS)
                && c.method == "len"
            {
                self.call_mark(c.span(), &contracts::target("I01"));
            }
            self.call_mark(c.span(), &call.target);
            for (arg, parameter) in c.args.iter().zip(&call.inputs) {
                let value = self.expr(arg, Some(parameter));
                if value.as_ref() != Some(parameter) {
                    self.reject(
                        arg.span(),
                        Kind::RET002,
                        None,
                        "contract argument differs from exact parameter",
                    );
                }
            }
            return Some(call.output);
        }
        let signature = match claim {
            Some(Ok(selection)) => {
                self.call_mark(c.span(), &selection.function.id);
                Some(selection.function)
            }
            Some(Err(reason)) => {
                self.reject(
                    c.span(),
                    Kind::RET002,
                    None,
                    format!("unresolved call: {reason}"),
                );
                None
            }
            None => {
                self.reject(
                    c.span(),
                    Kind::RET002,
                    None,
                    "call outside A2 checked scope (including rejected macro/closure)",
                );
                None
            }
        };
        // Independent signature facts may diagnose nested raw construction in
        // an otherwise rejected generic call. They cannot grant a call mark.
        let diagnostic_signature = signature.clone().or_else(|| {
            let syn::Expr::Path(p) = &*c.receiver else {
                return None;
            };
            let mut ty = self
                .locals
                .get(&p.path.get_ident()?.to_string())?
                .as_ref()?;
            while let ExactTy::Reference(t, _) = ty {
                ty = t;
            }
            let ExactTy::Nominal(owner, _) = ty else {
                return None;
            };
            let id = format!("{owner}::{}", c.method);
            resolve::function(self.source(), &id).ok().cloned()
        });
        for (i, arg) in c.args.iter().enumerate() {
            let parameter = diagnostic_signature.as_ref().and_then(|f| {
                match f.sig().inputs.iter().nth(i + 1)? {
                    syn::FnArg::Typed(a) => types::in_function(self.source(), f, &a.ty).ok(),
                    _ => None,
                }
            });
            self.expr(arg, parameter.as_ref());
        }
        signature.and_then(|f| types::exact_output(self.source(), &f).ok())
    }

    fn std_output(
        &self,
        p: &syn::ExprPath,
        target: &str,
        expected: Option<&ExactTy>,
    ) -> Option<ExactTy> {
        // A compiler-checked diagnostic signature, NOT an admission or a list
        // of calls to reject. No mark exists for this operation. Renamed calls
        // and arbitrary helpers returning Vec go through the same type check.
        const _: fn() -> Vec<u8> = Vec::<u8>::new;
        if target != "std::vec::Vec::new" {
            return None;
        }
        let mut owner = p.path.clone();
        owner.segments.pop();
        owner.segments.pop_punct();
        types::exact(
            self.source(),
            &self.f.module,
            &syn::Type::Path(syn::TypePath {
                qself: None,
                path: owner,
            }),
        )
        .ok()
        .or_else(|| {
            expected
                .filter(|t| matches!(t, ExactTy::Nominal(id,_) if id == "std::vec::Vec"))
                .cloned()
        })
    }
}
