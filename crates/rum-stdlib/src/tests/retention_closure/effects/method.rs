//! Selection is a Rust fact. It cannot substitute for body/effect closure.

use super::super::methods;
use super::*;

impl Checker<'_> {
    pub(super) fn method(
        &mut self,
        f: &Function,
        call: &syn::ExprMethodCall,
        env: &mut Environment,
    ) -> Checked<Value> {
        let selection = methods::claims(self.source, f)
            .remove(&methods::site(f, call))
            .ok_or_else(|| {
                self.unsupported(f, call.span(), "method lies outside checked lexical scope")
            })?
            .map_err(|reason| {
                self.unsupported(
                    f,
                    call.span(),
                    if matches!(
                        reason.as_str(),
                        "method receiver lacks checked nominal/place provenance"
                            | "method receiver lacks an exact nominal type"
                    ) {
                        "method dispatch requires autoderef/autoref/trait resolution".into()
                    } else {
                        reason
                    },
                )
            })?;
        let callee = selection.function;
        types::signature(self.source, &callee).map_err(|r| self.unsupported(f, call.span(), r))?;
        let parameters = types::exact_parameters(self.source, &callee)
            .map_err(|r| self.unsupported(f, call.span(), r))?;
        if parameters.len() != call.args.len() + 1 {
            return Err(self.unsupported(f, call.span(), "method argument arity mismatch"));
        }
        let borrowed = matches!(parameters[0].1, ExactTy::Reference(..));
        let receiver = match &*call.receiver {
            syn::Expr::Path(_) | syn::Expr::Field(_) => {
                self.place_value(f, &call.receiver, env, !borrowed)?
            }
            _ => self.expr(f, &call.receiver, env)?,
        };
        if receiver.exact != selection.receiver.exact {
            return Err(self.unsupported(
                f,
                call.receiver.span(),
                "receiver provenance disagrees with effects place",
            ));
        }
        self.retain(f, call.receiver.span(), &receiver)?;
        self.fact(
            f,
            &call.receiver,
            &receiver,
            if borrowed {
                "method-reborrow-place"
            } else {
                "method-owned-receiver"
            },
        );
        for (arg, (_, expected)) in call.args.iter().zip(&parameters[1..]) {
            let value = self.expr_expected(f, arg, env, Some(expected))?;
            if value.exact != *expected {
                return Err(self.unsupported(f, arg.span(), "method argument type mismatch"));
            }
            self.retain(f, arg.span(), &value)?;
        }
        let exact = types::exact_output(self.source, &callee)
            .map_err(|r| self.unsupported(f, call.span(), r))?;
        let mut result = Value::typed(exact.clone(), types::ownership(self.source, &exact));
        result.origin = Some(callee.id.clone());
        self.body_function(callee)?; // MUTATION:method-body-closure
        Ok(result)
    }
}
