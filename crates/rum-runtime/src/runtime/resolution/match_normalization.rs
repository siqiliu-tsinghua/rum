use crate::runtime::{Attribute, AttributeSet, Expr, HoldAttribute, Session, SharedArgs};

impl Session {
    pub(in crate::runtime) fn normalize_match_expr(&self, expr: &Expr) -> Expr {
        self.normalize_match_expr_changed(expr)
            .unwrap_or_else(|| expr.clone())
    }

    pub(in crate::runtime) fn normalize_definition_identity_for_matching(
        &self,
        expr: &Expr,
        attributes: AttributeSet,
    ) -> Expr {
        if matches!(attributes.hold_attribute(), HoldAttribute::None) {
            return expr.clone();
        }

        let Expr::Call { head, args } = expr else {
            return expr.clone();
        };
        let normalized_head = self.normalize_match_expr_changed(head);
        let mut normalized_args: Option<Vec<Expr>> = None;
        for (index, arg) in args.iter().enumerate() {
            if let Some(normalized_arg) = self.normalize_match_expr_changed(arg) {
                normalized_args
                    .get_or_insert_with(|| args[..index].to_vec())
                    .push(normalized_arg);
            } else if let Some(normalized_args) = &mut normalized_args {
                normalized_args.push(arg.clone());
            }
        }

        match (normalized_head, normalized_args) {
            (None, None) => expr.clone(),
            (Some(head), None) => Expr::call_with_shared_args(head, args.clone()),
            (None, Some(args)) => Expr::call_shared(head.clone(), SharedArgs::new(args)),
            (Some(head), Some(args)) => Expr::call(head, args),
        }
    }

    fn normalize_match_expr_changed(&self, expr: &Expr) -> Option<Expr> {
        match expr {
            Expr::Atom(_) => None,
            Expr::Call { head, args } => {
                let normalized_head = self.normalize_match_expr_changed(head);
                let head_for_attributes = normalized_head.as_ref().unwrap_or(head);
                let normalized_args =
                    self.normalize_match_arg_list_changed(args, head_for_attributes);

                match (normalized_head, normalized_args) {
                    (None, None) => None,
                    (Some(head), None) => Some(Expr::call_with_shared_args(head, args.clone())),
                    (None, Some(args)) => {
                        Some(Expr::call_shared(head.clone(), SharedArgs::new(args)))
                    }
                    (Some(head), Some(args)) => Some(Expr::call(head, args)),
                }
            }
        }
    }

    #[cfg(test)]
    pub(in crate::runtime) fn normalize_match_expr_changed_for_test(
        &self,
        expr: &Expr,
    ) -> Option<Expr> {
        self.normalize_match_expr_changed(expr)
    }

    fn normalize_match_arg_list_changed(
        &self,
        args: &SharedArgs,
        normalized_head: &Expr,
    ) -> Option<Vec<Expr>> {
        let mut normalized_args: Option<Vec<Expr>> = None;

        for (index, arg) in args.iter().enumerate() {
            if let Some(normalized_arg) = self.normalize_match_expr_changed(arg) {
                normalized_args
                    .get_or_insert_with(|| args[..index].to_vec())
                    .push(normalized_arg);
            } else if let Some(normalized_args) = &mut normalized_args {
                normalized_args.push(arg.clone());
            }
        }

        let attributes = self.attribute_set_for_head(normalized_head);
        if !attributes.contains(Attribute::Flat) && !attributes.contains(Attribute::Orderless) {
            return normalized_args;
        }

        let args_for_attribute_normalization = normalized_args.unwrap_or_else(|| args.to_vec());
        let attribute_normalized_args = self.normalize_call_args_by_attributes(
            normalized_head,
            args_for_attribute_normalization,
            attributes,
        );

        if attribute_normalized_args.as_slice() == args.as_ref() {
            None
        } else {
            Some(attribute_normalized_args)
        }
    }
}
