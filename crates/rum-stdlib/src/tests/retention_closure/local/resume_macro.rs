//! Source adapter for the two exact resume_or_suspend arms. Recognizing the
//! expansion is separate from B12's guarded-frame caller precondition.
use super::*;
use syn::parse::{Parse, ParseStream};

const FILE: &str = "crates/rum-runtime/src/runtime/suspension/core.rs";

pub(super) fn definition(source: &Source) -> Result<(), String> {
    let file = source
        .files
        .get(FILE)
        .ok_or("missing canonical resume macro")?;
    let parsed = syn::parse_file(file).map_err(|e| e.to_string())?;
    let definitions: Vec<_> = parsed
        .items
        .iter()
        .filter_map(|i| match i {
            syn::Item::Macro(m) if m.ident.as_ref().is_some_and(|n| n == "resume_or_suspend") => {
                Some(m)
            }
            _ => None,
        })
        .collect();
    let expected: syn::ItemMacro = syn::parse_quote! {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! resume_or_suspend {
            (
                $session:expr,
                $result:expr,
                ok($value:pat) => $continue:expr,
                suspend => $frame:expr $(,)?
            ) => {{
                match $result {
                    Ok($value) => $continue,
                    Err($crate::kernel_api::EvalError::InputRequired { request }) => {
                        $session.insert_suspension_frame($frame);
                        Err($crate::kernel_api::EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }};
            (insert $session:expr, $frame:expr $(,)?) => {{ $session.insert_suspension_frame($frame) }};
        }
    };
    let [found] = definitions.as_slice() else {
        return Err("ambiguous resume macro definition".into());
    };
    let mut found = (*found).clone();
    // The ordinary doc comment is not part of expansion or hygiene.
    found
        .attrs
        .retain(|a| !a.path().is_ident("doc") || tokens(a) == "# [doc (hidden)]");
    if tokens(&found) != tokens(&expected) {
        return Err("resume macro definition/arm tokens changed".into());
    }
    Ok(())
}

pub(super) enum Arm {
    Insert {
        session: syn::Expr,
        frame: syn::Expr,
    },
    Result {
        session: syn::Expr,
        result: syn::Expr,
        value: syn::Ident,
        success: syn::Expr,
        frame: syn::Expr,
    },
}

impl Parse for Arm {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let insert = input
            .fork()
            .parse::<syn::Ident>()
            .is_ok_and(|i| i == "insert");
        if insert {
            input.parse::<syn::Ident>()?;
        }
        let session = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let result = if insert {
            Self::Insert {
                session,
                frame: input.parse()?,
            }
        } else {
            let result = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            if input.parse::<syn::Ident>()? != "ok" {
                return Err(input.error("expected ok arm"));
            }
            let binding;
            syn::parenthesized!(binding in input);
            let value = binding.parse()?;
            if !binding.is_empty() {
                return Err(binding.error("only one plain binding is admitted"));
            }
            input.parse::<syn::Token![=>]>()?;
            let success = input.parse()?;
            input.parse::<syn::Token![,]>()?;
            if input.parse::<syn::Ident>()? != "suspend" {
                return Err(input.error("expected suspend arm"));
            }
            input.parse::<syn::Token![=>]>()?;
            Self::Result {
                session,
                result,
                value,
                success,
                frame: input.parse()?,
            }
        };
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }
        if !input.is_empty() {
            return Err(input.error("extra resume macro tokens"));
        }
        Ok(result)
    }
}

pub(super) fn adapt(
    source: &Source,
    f: &Function,
    mac: &syn::Macro,
    definition: &Result<(), String>,
) -> Result<Arm, String> {
    definition.clone()?;
    if tokens(&mac.path) != "crate :: resume_or_suspend" || !f.module.starts_with("rum_runtime::") {
        return Err("resume macro path/hygiene is outside the exact crate invocation".into());
    }
    for name in ["Ok", "Err"] {
        let path = syn::parse_str::<syn::Path>(name).unwrap();
        if resolve::path(source, &f.module, &path).ok()
            != Some(format!("std::result::Result::{name}"))
        {
            return Err("resume macro Ok/Err hygiene changed".into());
        }
    }
    struct Bindings(bool);
    impl<'a> Visit<'a> for Bindings {
        fn visit_pat_ident(&mut self, p: &'a syn::PatIdent) {
            if p.ident == "Ok" || p.ident == "Err" {
                self.0 = true;
            }
        }
    }
    let mut bindings = Bindings(false);
    bindings.visit_signature(&f.sig());
    bindings.visit_block(&f.body());
    if bindings.0 {
        return Err("resume macro local Ok/Err shadow".into());
    }
    syn::parse2(mac.tokens.clone()).map_err(|e| e.to_string())
}

impl Arm {
    /// The expansion is diagnostic source, never an automatic admission. The
    /// three branches keep lazy frame construction and the actual Err payload.
    pub(super) fn expansion(&self) -> syn::Expr {
        match self {
            Self::Insert { session, frame } => {
                syn::parse_quote!({ #session.insert_suspension_frame(#frame) })
            }
            Self::Result {
                session,
                result,
                value,
                success,
                frame,
            } => syn::parse_quote!({
                match #result {
                    Ok(#value) => #success,
                    Err(crate::kernel_api::EvalError::InputRequired { request }) => {
                        #session.insert_suspension_frame(#frame);
                        Err(crate::kernel_api::EvalError::InputRequired { request })
                    }
                    Err(error) => Err(error),
                }
            }),
        }
    }
}

pub(super) fn guarded_insert(index: &Interfaces, f: &Function, arm: &Arm) -> Result<(), String> {
    let Arm::Insert { frame, .. } = arm else {
        return Err("B12 excludes the raw result-arm frame; use the charged typed bridge".into());
    };
    rules::caller_domain(index, f, &contracts::target("B12"))?;
    struct Invocations(Vec<syn::Macro>);
    impl<'a> Visit<'a> for Invocations {
        fn visit_macro(&mut self, mac: &'a syn::Macro) {
            if tokens(&mac.path) == "crate :: resume_or_suspend" {
                self.0.push(mac.clone());
            }
        }
    }
    let mut invocations = Invocations(vec![]);
    invocations.visit_block(&f.body());
    if !invocations.0.iter().any(|mac| {
        syn::parse2::<Arm>(mac.tokens.clone())
            .is_ok_and(|original| tokens(&original.expansion()) == tokens(&arm.expansion()))
    }) {
        return Err("B12 invocation differs from the indexed guarded insertion".into());
    }
    let syn::Expr::Struct(frame) = frame else {
        return Err("B12 needs private RetentionFrame data".into());
    };
    let target = resolve::path(&index.signatures, &f.module, &frame.path)?;
    if target != "rum_runtime::runtime::accounting_mode::frozen::RetentionFrame" {
        return Err("B12 excludes raw frames even at insert_suspension_frame".into());
    }
    Ok(())
}
