//! Only immediate literal FnOnce closures at scoped/suspend, and the exact
//! CallbackValue::adopt function item at Result::and_then. No callable locals,
//! coercions to fn pointers, dyn calls, or escaping closures are admitted.
use super::*;

impl Lint<'_> {
    pub(super) fn scoped(&mut self, c: &syn::ExprCall, path: &syn::ExprPath) -> Option<ExactTy> {
        let syn::PathArguments::AngleBracketed(g) = &path.path.segments.last()?.arguments else {
            return self.no(
                c.span(),
                "scoped requires one explicit sealed mode and a literal closure",
            );
        };
        let args: Vec<_> = g.args.iter().collect();
        let [
            syn::GenericArgument::Type(mode),
            syn::GenericArgument::Type(syn::Type::Infer(_)),
        ] = args.as_slice()
        else {
            return self.no(
                c.span(),
                "scoped generic arguments differ from finite mode plus literal closure",
            );
        };
        let mode = types::in_function(self.source(), self.f, mode).ok()?;
        if !types::is_mode(&mode) || c.args.len() != 2 {
            return self.no(c.span(), "scoped mode/arity is not concrete");
        }
        let session = protocol::borrow(protocol::nominal(super::super::regions::SESSION), true);
        if self.expr(&c.args[0], Some(&session)) != Some(session) {
            return self.no(c.span(), "scoped session type mismatch");
        }
        let output = protocol::result(
            protocol::instance(protocol::RESULT, &mode),
            protocol::nominal(protocol::ERROR),
        );
        self.literal_callback(
            &c.args[1],
            &[
                protocol::instance(protocol::CONTEXT, &mode),
                protocol::instance(protocol::VALUES, &mode),
            ],
            &output,
        )?;
        self.call_mark(c.span(), &protocol::key(protocol::SCOPED, &mode));
        Some(protocol::result(
            protocol::nominal(protocol::EXPR),
            protocol::nominal(protocol::ERROR),
        ))
    }

    fn literal_callback(
        &mut self,
        expr: &syn::Expr,
        parameters: &[ExactTy],
        output: &ExactTy,
    ) -> Option<ExactTy> {
        let syn::Expr::Closure(c) = expr else {
            return self.no(expr.span(), "callback must be a literal FnOnce closure");
        };
        if c.asyncness.is_some()
            || c.movability.is_some()
            || c.capture.is_some()
            || c.lifetimes.is_some()
            || !c.attrs.is_empty()
            || c.inputs.len() != parameters.len()
            || !matches!(c.output, syn::ReturnType::Default)
        {
            return self.no(
                c.span(),
                "callback closure has unsupported capture/signature",
            );
        }
        let old = self.locals.clone();
        let old_output = self.output.replace(output.clone());
        let old_diverges = self.diverges;
        self.diverges = false;
        for (pat, ty) in c.inputs.iter().zip(parameters) {
            self.bind(pat, Some(ty.clone()));
        }
        let result = self.expr(&c.body, Some(output));
        self.locals = old;
        self.output = old_output;
        self.diverges = old_diverges;
        if result.as_ref() != Some(output) {
            return self.no(
                c.span(),
                "literal callback return differs from contracted output",
            );
        }
        // Unique closure identity. No nominal or function-pointer type claim is
        // forged for a Rust closure's anonymous type.
        self.report.facts.insert(
            Site::new(&self.f.site.file, c.span()),
            ExactTy::Function(format!(
                "literal@{}",
                Site::new(&self.f.site.file, c.span()).display()
            )),
        );
        Some(output.clone())
    }

    pub(super) fn adopt_call(
        &mut self,
        c: &syn::ExprMethodCall,
        receiver: &ExactTy,
    ) -> Option<ExactTy> {
        let ExactTy::Nominal(id, args) = receiver else {
            return self.no(c.span(), "and_then requires concrete callback Result");
        };
        let Some(ExactTy::Nominal(owner, modes)) = args.first() else {
            return self.no(c.span(), "and_then success must be CallbackValue");
        };
        if id != "std::result::Result"
            || owner != protocol::CALLBACK
            || modes.len() != 1
            || !types::is_mode(&modes[0])
            || args.len() != 2
            || args[1] != protocol::instance(protocol::FAILURE, &modes[0])
            || c.args.len() != 1
            || c.turbofish.is_some()
        {
            return self.no(
                c.span(),
                "and_then requires exact CallbackValue/CallbackFailure mode",
            );
        }
        let syn::Expr::Path(p) = &c.args[0] else {
            return self.no(
                c.span(),
                "and_then callback must be exact adopt function item",
            );
        };
        let target = resolve::path(self.source(), &self.f.module, &p.path).ok()?;
        if p.qself.is_some()
            || target != format!("{}::adopt", protocol::CALLBACK)
            || p.path.segments.iter().any(|s| !s.arguments.is_empty())
        {
            return self.no(
                p.span(),
                "and_then function item is not exact CallbackValue::adopt",
            );
        }
        self.call_mark(p.span(), &protocol::key(&target, &modes[0]));
        self.call_mark(
            c.span(),
            &protocol::key(
                "std::result::Result::and_then::<CallbackValue::adopt>",
                &modes[0],
            ),
        );
        let f = protocol::signature(self.source(), &target)?;
        let f = types::with_mode(self.source(), f, Some(&modes[0])).ok()?;
        types::exact_output(self.source(), &f).ok()
    }

    pub(super) fn suspend_call(
        &mut self,
        c: &syn::ExprMethodCall,
        receiver: &ExactTy,
    ) -> Option<ExactTy> {
        let ExactTy::Nominal(id, modes) = receiver else {
            return self.no(c.span(), "suspend needs concrete CallbackFailure");
        };
        if id != protocol::FAILURE
            || modes.len() != 1
            || !types::is_mode(&modes[0])
            || c.args.len() != 1
            || c.turbofish.is_some()
        {
            return self.no(
                c.span(),
                "suspend receiver/arguments outside concrete instance",
            );
        }
        self.literal_callback(&c.args[0], &[], &protocol::nominal(protocol::PROBE))?;
        self.call_mark(
            c.span(),
            &protocol::key(
                &format!(
                    "{}::suspend::<{},literal>",
                    protocol::FAILURE,
                    protocol::PROBE
                ),
                &modes[0],
            ),
        );
        Some(protocol::instance(protocol::RESULT, &modes[0]))
    }
}
