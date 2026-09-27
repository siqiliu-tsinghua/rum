//! Resolved calls and exact, individually audited trusted instantiations.

use super::*;

impl Checker<'_> {
    pub(super) fn call(
        &mut self,
        f: &Function,
        call: &syn::ExprCall,
        env: &mut Environment,
        expected: Option<&ExactTy>,
    ) -> Checked<Value> {
        let syn::Expr::Path(path) = &*call.func else {
            return Err(self.unsupported(f, call.func.span(), "runtime callable value"));
        };
        if path.qself.is_some() || !path.attrs.is_empty() {
            return Err(self.unsupported(f, path.span(), "trait projection or attributed callee"));
        }
        if path
            .path
            .get_ident()
            .is_some_and(|name| env.contains_key(&name.to_string()))
        {
            return Err(self.unsupported(f, path.span(), "callable local shadows the function"));
        }
        let target = resolve::path(self.source, &f.module, &path.path)
            .map_err(|r| self.unsupported(f, path.span(), r))?;
        if resolve::standard_variant(&target).is_some_and(|(_, variant)| variant == "None") {
            return Err(self.unsupported(f, call.span(), "unit variant is not callable"));
        }
        if resolve::standard_variant(&target).is_some() {
            let arguments: Vec<_> = call.args.iter().collect();
            return self.construct(f, &path.path, &arguments, call.span(), env, expected);
        }
        if types::OWNED_CONTAINERS.iter().any(|owner| {
            target
                .strip_prefix(owner)
                .is_some_and(|rest| rest.starts_with("::"))
        }) {
            return Err(self.error(
                f,
                call.span(),
                Kind::RET001,
                Some(target),
                "unleased owned payload",
            )); // MUTATION:raw-allocation
        }
        let callee = match resolve::function(self.source, &target) {
            Ok(callee) => callee.clone(),
            Err(reason) => {
                return Err(self.error(f, call.span(), Kind::RET002, Some(target), reason)); // MUTATION:external-dispatch
            }
        };
        let instance = types::arguments(self.source, &f.module, &path.path)
            .map_err(|r| self.unsupported(f, path.span(), r))?;
        let key = LeafKey {
            id: target.clone(),
            instance: instance.clone(),
        };
        let leaf = self
            .contracts
            .iter()
            .find(|leaf| leaf.key() == key)
            .cloned();
        if !instance.is_empty() && leaf.is_none() {
            return Err(self.unsupported(
                f,
                path.span(),
                "generic call substitution has no exact exercised contract",
            ));
        }
        let (exact_parameters, exact_output) = if instance.is_empty() {
            types::signature(self.source, &callee)
                .map_err(|r| self.unsupported(f, call.span(), r))?;
            (
                types::exact_parameters(self.source, &callee),
                types::exact_output(self.source, &callee),
            )
        } else {
            let substitutions = callee
                .sig
                .generics
                .params
                .iter()
                .zip(&instance)
                .map(|(p, t)| match p {
                    syn::GenericParam::Type(p) => Ok((p.ident.to_string(), t.clone())),
                    _ => Err("unsupported generic parameter".to_string()),
                })
                .collect::<Result<BTreeMap<_, _>, _>>()
                .map_err(|r| self.unsupported(f, call.span(), r))?;
            if substitutions.len() != instance.len()
                || instance.len() != callee.sig.generics.params.len()
            {
                return Err(self.unsupported(
                    f,
                    call.span(),
                    "exact function instance arity mismatch",
                ));
            }
            let parameters = callee
                .sig
                .inputs
                .iter()
                .map(|arg| {
                    let syn::FnArg::Typed(arg) = arg else {
                        return Err("generic receiver not modeled".into());
                    };
                    let syn::Pat::Ident(p) = &*arg.pat else {
                        return Err("generic argument pattern not modeled".into());
                    };
                    Ok((
                        p.ident.to_string(),
                        types::substitute(self.source, &callee.module, &arg.ty, &substitutions)?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>();
            let output = match &callee.sig.output {
                syn::ReturnType::Default => Ok(ExactTy::Unit),
                syn::ReturnType::Type(_, t) => {
                    types::substitute(self.source, &callee.module, t, &substitutions)
                }
            };
            (parameters, output)
        };
        let parameters = exact_parameters.map_err(|r| self.unsupported(f, call.span(), r))?;
        let exact_output = exact_output.map_err(|r| self.unsupported(f, call.span(), r))?;
        if parameters.len() != call.args.len() {
            return Err(self.unsupported(f, call.span(), "exact call arity mismatch"));
        }
        let mut arguments = Vec::new();
        for (arg, (_, expected)) in call.args.iter().zip(&parameters) {
            let value = self.expr_expected(f, arg, env, Some(expected))?;
            if value.exact != *expected {
                return Err(self.unsupported(f, arg.span(), "argument ownership/type mismatch"));
            }
            if leaf.as_ref().is_none_or(|l| l.operation != "adopt") {
                self.retain(f, arg.span(), &value)?;
            }
            arguments.push(value);
        }
        let mut value = if leaf.as_ref().is_some_and(|l| l.operation == "keep") {
            arguments[0].clone()
        } else {
            Value::typed(
                exact_output.clone(),
                types::ownership(self.source, &exact_output),
            )
        };
        value.origin = Some(target.clone());
        self.fact(
            f,
            &call.func,
            &Value::typed(ExactTy::Function(target.clone()), Ty::Fixed(target.clone())),
            "resolved-function-item",
        );
        if let Some(marks) = self.local_marks {
            marks
                .get(&target)
                .cloned()
                .unwrap_or_else(|| Err(format!("callee has no local mark: {target}")))
                .map_err(|r| self.unsupported(f, call.span(), r))?;
        } else if let Some(leaf) = leaf {
            let used = BTreeSet::from([key.clone()]);
            leaves::validate(self.source, std::slice::from_ref(&leaf), &used)
                .map_err(|r| self.unsupported(f, call.span(), r))?;
            if leaf.operation == "adopt" {
                self.proof.adoptions += 1;
            }
            if leaf.operation == "send" {
                self.proof.transfers += 1;
            }
            self.proof.leaves.insert(key);
        } else {
            // Body closure is automatic, including unmarked helper extraction.
            self.body(&target)?; // MUTATION:helper-closure
        }
        Ok(value)
    }
}
