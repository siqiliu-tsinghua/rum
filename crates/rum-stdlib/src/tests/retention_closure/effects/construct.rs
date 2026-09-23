//! Four standard variants, selected by namespace before applying any rule.

use super::*;

impl Checker<'_> {
    pub(super) fn construct(
        &mut self,
        f: &Function,
        path: &syn::Path,
        args: &[&syn::Expr],
        span: proc_macro2::Span,
        env: &mut Environment,
        expected: Option<&ExactTy>,
    ) -> Checked<Value> {
        let target = resolve::path(self.source, &f.module, path)
            .map_err(|r| self.unsupported(f, span, r))?;
        let (family, variant) = resolve::standard_variant(&target).ok_or_else(|| {
            self.unsupported(
                f,
                span,
                "not a local value; stored function/constant values are not admitted",
            )
        })?;
        if args.len() != usize::from(variant != "None") {
            return Err(self.unsupported(f, span, "variant constructor arity mismatch"));
        }
        let explicit = types::arguments(self.source, &f.module, path)
            .map_err(|r| self.unsupported(f, span, r))?;
        // Arguments on an alias instantiate that alias before selecting its
        // variant (e.g. Swap<A,B> = Result<B,A>), never the target's raw args.
        let alias_instance =
            if path.segments.len() > 1 && path.segments.last().unwrap().arguments.is_empty() {
                let mut base = path.clone();
                base.segments.pop();
                base.segments.pop_punct();
                types::exact(
                    self.source,
                    &f.module,
                    &syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: base,
                    }),
                )
                .ok()
            } else {
                None
            };
        let mut instance = alias_instance.or_else(|| {
            if !explicit.is_empty() {
                Some(ExactTy::Nominal(family.into(), explicit))
            } else {
                expected.cloned()
            }
        });
        if let Some(ty) = &instance {
            if !matches!(ty, ExactTy::Nominal(id, arguments) if id == family && arguments.len() == if family == "std::option::Option" {1} else {2})
            {
                return Err(self.unsupported(
                    f,
                    span,
                    "constructor instance differs from expected nominal type",
                ));
            }
            if expected.is_some_and(|e| e != ty) {
                return Err(self.unsupported(
                    f,
                    span,
                    "explicit constructor instance differs from context",
                ));
            }
        }
        let payload_expected = instance.as_ref().and_then(|t| match t {
            ExactTy::Nominal(_, ts) => ts.get(usize::from(variant == "Err")),
            _ => None,
        });
        let payload = args
            .first()
            .map(|arg| self.expr_expected(f, arg, env, payload_expected))
            .transpose()?;
        if instance.is_none() && variant == "Some" {
            instance = Some(ExactTy::Nominal(
                family.into(),
                vec![payload.as_ref().unwrap().exact.clone()],
            ));
        }
        let exact = instance.ok_or_else(|| {
            self.unsupported(
                f,
                span,
                "constructor needs an exact expected type or explicit instantiation",
            )
        })?;
        if let Some(payload) = &payload {
            let declared = types::fields(self.source, &exact, Some(variant))
                .map_err(|r| self.unsupported(f, span, r))?;
            if declared.get("0") != Some(&payload.exact) {
                return Err(self.unsupported(f, span, "constructor payload exact type mismatch"));
            }
        }
        let mut fields = BTreeMap::new();
        let mut origin = None;
        if let Some(payload) = payload {
            fields.insert("0".into(), payload.ty); // MUTATION:constructor-payload
            origin = payload.origin;
        }
        let mut value = Value::typed(
            exact,
            Ty::Variants(BTreeMap::from([(variant.into(), fields)])),
        );
        value.origin = origin;
        value.provenance = Some(Provenance {
            source: Site::new(&f.site.file, span),
            basis: format!("constructor {target}"),
            projections: Vec::new(),
        });
        Ok(value)
    }
}
