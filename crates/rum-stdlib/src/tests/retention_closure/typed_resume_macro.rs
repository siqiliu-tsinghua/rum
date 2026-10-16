//! Checked frontier adapter, not a retention admission or a compiler expansion.
//! The local legacy adapter intentionally keeps its narrower binding grammar.
use super::{
    model::{Function, Site, Source, tokens},
    resolve,
};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::visit::Visit;

const FILE: &str = "crates/rum-runtime/src/runtime/suspension/typed/witness.rs";

// Exhaustive accounting of the checked expansion, apart from the invocation's
// five syntax inputs visited by each production view. Generic witness/adapter effects
// are not certified by this frontier, so each operation is a located rejection.
// The cold helper captures and moves the suspension inputs, invokes FnOnce,
// and drops that closure. Account for those operations independently of the
// delegated insertion and error reconstruction; no return type earns coverage.
pub(super) const EXPANSION_REJECTIONS: [&str; 5] = [
    "typed macro expansion: owned producer binding and slot_witness(&produced) borrow/call unresolved",
    "typed macro expansion: owned Result match, Ok/InputRequired/error bindings, Err(error) forwarding and local drops unresolved",
    "typed macro expansion: cold helper closure capture/move of witness, session, request and frame inputs; FnOnce call and closure drop unresolved",
    "typed macro expansion: return through witness.suspend; typed insertion and frame ownership transfer unresolved",
    "typed macro expansion: delegated InputRequired reconstruction and request/error ownership unresolved",
];

fn recognizes(mac: &syn::Macro) -> bool {
    mac.path
        .segments
        .last()
        .is_some_and(|s| s.ident == "resume_typed_or_suspend")
}

fn definition(source: &Source) -> Result<(), String> {
    let file = source
        .files
        .get(FILE)
        .ok_or("missing typed resume macro definition")?;
    let parsed = syn::parse_file(file).map_err(|e| e.to_string())?;
    let definitions: Vec<_> = parsed
        .items
        .iter()
        .filter_map(|i| match i {
            syn::Item::Macro(m)
                if m.ident
                    .as_ref()
                    .is_some_and(|n| n == "resume_typed_or_suspend") =>
            {
                Some(m)
            }
            _ => None,
        })
        .collect();
    let expected: syn::ItemMacro = syn::parse_quote! {
        #[macro_export]
        macro_rules! resume_typed_or_suspend {
            (
                $session:expr, $result:ident,
                ok($value:pat) => $continue:expr,
                evaluation_return => $frame:expr $(,)?
            ) => {{
                let witness = $crate::kernel_api::slot_witness(&($result));
                match $result {
                    Ok($value) => $continue,
                    Err($crate::kernel_api::EvalError::InputRequired { request }) => {
                        #[cold]
                        #[inline(never)]
                        fn suspend<R>(make: impl ::core::ops::FnOnce() -> R) -> R {
                            make()
                        }
                        return suspend(move || {
                            witness.suspend_evaluation_return($session, request, $frame)
                        });
                    }
                    Err(error) => Err(error),
                }
            }};
            (
                $session:expr, $result:ident,
                ok($value:pat) => $continue:expr,
                suspend => $frame:expr $(,)?
            ) => {{
                let witness = $crate::kernel_api::slot_witness(&($result));
                match $result {
                    Ok($value) => $continue,
                    Err($crate::kernel_api::EvalError::InputRequired { request }) => {
                        #[cold]
                        #[inline(never)]
                        fn suspend<R>(make: impl ::core::ops::FnOnce() -> R) -> R {
                            make()
                        }
                        return suspend(move || witness.suspend($session, request, $frame));
                    }
                    Err(error) => Err(error),
                }
            }};
            (
                $session:expr, $result:expr,
                ok($value:pat) => $continue:expr,
                suspend => $frame:expr $(,)?
            ) => {{
                let produced = $result;
                let witness = $crate::kernel_api::slot_witness(&produced);
                match produced {
                    Ok($value) => $continue,
                    Err($crate::kernel_api::EvalError::InputRequired { request }) => {
                        #[cold]
                        #[inline(never)]
                        fn suspend<R>(make: impl ::core::ops::FnOnce() -> R) -> R {
                            make()
                        }
                        return suspend(move || witness.suspend($session, request, $frame));
                    }
                    Err(error) => Err(error),
                }
            }};
        }
    };
    let [found] = definitions.as_slice() else {
        return Err("ambiguous typed resume macro definition".into());
    };
    let mut found = (*found).clone();
    // Documentation text is not expansion syntax or hygiene.
    found.attrs.retain(|a| !a.path().is_ident("doc"));
    if tokens(&found) != tokens(&expected) {
        return Err("typed resume macro definition changed".into());
    }
    Ok(())
}

pub(super) struct Arm {
    pub session: syn::Expr,
    pub result: syn::Expr,
    pub value: syn::Pat,
    pub success: syn::Expr,
    pub frame: syn::Expr,
    evaluation_return: bool,
}

impl Arm {
    pub(super) fn expansion_rejections(&self) -> Vec<&'static str> {
        let mut operations = EXPANSION_REJECTIONS.to_vec();
        // Only a bare identifier selects the first Rust macro arm. Parentheses,
        // blocks and qualified paths select the expression arm and its binding.
        if matches!(
            &self.result,
            syn::Expr::Path(path)
                if path.qself.is_none()
                    && path.path.leading_colon.is_none()
                    && path.path.segments.len() == 1
                    && matches!(path.path.segments[0].arguments, syn::PathArguments::None)
        ) {
            operations[0] = "typed macro expansion: caller producer place and slot_witness(&($result)) borrow/call unresolved";
        }
        if self.evaluation_return {
            operations[3] = "typed macro expansion: return through private Expr witness.suspend_evaluation_return; frame ownership transfer unresolved";
            operations.push("typed macro expansion: inline evaluation-return event insertion before witnessed consumer; parent context and resource epilogue unresolved");
        }
        operations
    }
}

impl Parse for Arm {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let session = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let result = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        if input.parse::<syn::Ident>()? != "ok" {
            return Err(input.error("expected ok arm"));
        }
        let binding;
        syn::parenthesized!(binding in input);
        let value = syn::Pat::parse_multi_with_leading_vert(&binding)?;
        if !binding.is_empty() {
            return Err(binding.error("extra ok pattern tokens"));
        }
        input.parse::<syn::Token![=>]>()?;
        let success = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let selector = input.parse::<syn::Ident>()?;
        let evaluation_return = selector == "evaluation_return";
        if selector != "suspend" && !evaluation_return {
            return Err(input.error("expected suspend or evaluation_return arm"));
        }
        if evaluation_return
            && !matches!(&result, syn::Expr::Path(path)
            if path.qself.is_none() && path.path.leading_colon.is_none()
                && path.path.segments.len() == 1
                && matches!(path.path.segments[0].arguments, syn::PathArguments::None))
        {
            return Err(input.error("evaluation_return requires a producer identifier"));
        }
        input.parse::<syn::Token![=>]>()?;
        let frame = input.parse()?;
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }
        if !input.is_empty() {
            return Err(input.error("extra typed resume macro tokens"));
        }
        Ok(Self {
            session,
            result,
            value,
            success,
            frame,
            evaluation_return,
        })
    }
}

fn adapt(source: &Source, f: &Function, mac: &syn::Macro) -> Result<Arm, String> {
    definition(source)?;
    let path = tokens(&mac.path);
    let canonical = (path == "crate :: resume_typed_or_suspend"
        && f.module.starts_with("rum_runtime::"))
        || (path == "rum_runtime :: resume_typed_or_suspend"
            && source
                .dependencies
                .get(f.module.split("::").next().unwrap())
                .is_some_and(|d| d.contains("rum_runtime")));
    if !canonical {
        return Err("typed resume macro path/hygiene is outside the canonical invocation".into());
    }
    // Ok/Err are intentionally unqualified in the checked definition. Resolve
    // them in the invocation's enclosing module exactly as the legacy adapter.
    for name in ["Ok", "Err"] {
        let path = syn::parse_str::<syn::Path>(name).unwrap();
        if resolve::path(source, &f.module, &path).ok()
            != Some(format!("std::result::Result::{name}"))
        {
            return Err("typed resume macro Ok/Err hygiene changed".into());
        }
    }
    let arm: Arm = syn::parse2(mac.tokens.clone())
        .map_err(|e| format!("typed resume macro arguments: {e}"))?;
    struct Bindings(bool);
    impl<'ast> Visit<'ast> for Bindings {
        fn visit_pat_ident(&mut self, pat: &'ast syn::PatIdent) {
            self.0 |= pat.ident == "Ok" || pat.ident == "Err";
            syn::visit::visit_pat_ident(self, pat);
        }
    }
    let mut bindings = Bindings(false);
    bindings.visit_signature(&f.sig);
    bindings.visit_block(&f.body);
    bindings.visit_pat(&arm.value);
    for expression in [&arm.session, &arm.result, &arm.success, &arm.frame] {
        bindings.visit_expr(expression);
    }
    if bindings.0 {
        return Err("typed resume macro local Ok/Err shadow".into());
    }
    Ok(arm)
}

// Every production view uses this entry point: an unrecognized macro is opaque,
// but a recognized typed invocation may never silently become opaque on drift.
pub(super) fn checked(source: &Source, f: &Function, mac: &syn::Macro) -> Option<Arm> {
    if !recognizes(mac) {
        return None;
    }
    Some(adapt(source, f, mac).unwrap_or_else(|error| {
        panic!("{}: {error}", Site::new(&f.site.file, mac.span()).display())
    }))
}
