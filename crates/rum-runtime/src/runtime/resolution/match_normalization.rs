use crate::runtime::{
    Attribute, AttributeSet, Expr, HoldAttribute, Session, SharedArgs, SharedExpr,
};

// This follows `RECURSIVE_TRANSFORM_CALL_DEPTH` for `Map`/`Apply` and the
// bounded recursive front path for `Level`. Sixteen is precedent, not a
// measured optimum for match normalization.
const MATCH_NORMALIZER_RECURSIVE_CALL_DEPTH: usize = 16;

enum MatchNormalizationWork<'a> {
    Visit(&'a Expr),
    FinishCall {
        head: &'a SharedExpr,
        args: &'a SharedArgs,
    },
}

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

        let Expr::Call { head, args, .. } = expr else {
            return expr.clone();
        };
        let normalized_head = self.normalize_match_expr_changed(head);
        let mut normalized_args: Option<Vec<Expr>> = None;
        for (index, arg) in args.iter().enumerate() {
            let normalized_arg = self.normalize_match_expr_changed(arg);
            append_normalized_arg(args, index, arg, normalized_arg, &mut normalized_args);
        }

        finish_normalized_match_call(head, args, normalized_head, normalized_args)
            .unwrap_or_else(|| expr.clone())
    }

    fn normalize_match_expr_changed(&self, expr: &Expr) -> Option<Expr> {
        self.normalize_match_expr_changed_with_call_depth(
            expr,
            MATCH_NORMALIZER_RECURSIVE_CALL_DEPTH,
        )
    }

    fn normalize_match_expr_changed_with_call_depth(
        &self,
        expr: &Expr,
        remaining_call_depth: usize,
    ) -> Option<Expr> {
        match expr {
            Expr::Atom(_) => None,
            Expr::Call { head, args, .. } => self.normalize_match_call_changed_recursive_bounded(
                head,
                args,
                remaining_call_depth,
            ),
        }
    }

    fn normalize_match_call_changed_recursive_bounded(
        &self,
        head: &SharedExpr,
        args: &SharedArgs,
        remaining_call_depth: usize,
    ) -> Option<Expr> {
        if remaining_call_depth == 0 {
            let expr = Expr::call_shared(head.clone(), args.clone());
            return self.normalize_match_expr_changed_iterative(&expr);
        }
        let child_call_depth = remaining_call_depth - 1;
        let normalized_head = match head.as_ref() {
            Expr::Atom(_) => None,
            Expr::Call {
                head: child_head,
                args: child_args,
                ..
            } => self.normalize_match_call_changed_recursive_bounded(
                child_head,
                child_args,
                child_call_depth,
            ),
        };
        let head_for_attributes = normalized_head.as_ref().unwrap_or(head);
        let normalized_args = self.normalize_match_arg_list_changed_recursive_bounded(
            args,
            head_for_attributes,
            child_call_depth,
        );

        finish_normalized_match_call(head, args, normalized_head, normalized_args)
    }

    fn normalize_match_arg_list_changed_recursive_bounded(
        &self,
        args: &SharedArgs,
        normalized_head: &Expr,
        remaining_call_depth: usize,
    ) -> Option<Vec<Expr>> {
        let mut normalized_args: Option<Vec<Expr>> = None;

        for (index, arg) in args.iter().enumerate() {
            let normalized_arg = match arg {
                Expr::Atom(_) => None,
                Expr::Call { head, args, .. } => self
                    .normalize_match_call_changed_recursive_bounded(
                        head,
                        args,
                        remaining_call_depth,
                    ),
            };
            append_normalized_arg(args, index, arg, normalized_arg, &mut normalized_args);
        }

        self.finish_match_arg_list_changed(args, normalized_head, normalized_args)
    }

    #[cold]
    #[inline(never)]
    fn normalize_match_expr_changed_iterative(&self, expr: &Expr) -> Option<Expr> {
        let mut work = vec![MatchNormalizationWork::Visit(expr)];
        let mut completed = Vec::<Option<Expr>>::new();

        while let Some(task) = work.pop() {
            match task {
                MatchNormalizationWork::Visit(Expr::Atom(_)) => completed.push(None),
                MatchNormalizationWork::Visit(Expr::Call { head, args, .. }) => {
                    work.push(MatchNormalizationWork::FinishCall { head, args });
                    for arg in args.iter().rev() {
                        work.push(MatchNormalizationWork::Visit(arg));
                    }
                    work.push(MatchNormalizationWork::Visit(head));
                }
                MatchNormalizationWork::FinishCall { head, args } => {
                    let child_count = args
                        .len()
                        .checked_add(1)
                        .expect("match normalization child count remains representable");
                    let first_child = completed
                        .len()
                        .checked_sub(child_count)
                        .expect("match normalization child results remain balanced");
                    let mut child_results = completed.drain(first_child..);
                    let normalized_head = child_results
                        .next()
                        .expect("a match-normalization call completes its head");
                    let mut normalized_args: Option<Vec<Expr>> = None;
                    for index in 0..args.len() {
                        let normalized_arg = child_results
                            .next()
                            .expect("a match-normalization call completes each argument");
                        append_normalized_arg(
                            args,
                            index,
                            &args[index],
                            normalized_arg,
                            &mut normalized_args,
                        );
                    }
                    debug_assert!(child_results.next().is_none());
                    drop(child_results);

                    let head_for_attributes = normalized_head.as_ref().unwrap_or(head);
                    let normalized_args = self.finish_match_arg_list_changed(
                        args,
                        head_for_attributes,
                        normalized_args,
                    );
                    completed.push(finish_normalized_match_call(
                        head,
                        args,
                        normalized_head,
                        normalized_args,
                    ));
                }
            }
        }

        let normalized = completed
            .pop()
            .expect("match normalization completes its requested root");
        debug_assert!(completed.is_empty());
        normalized
    }

    fn finish_match_arg_list_changed(
        &self,
        args: &SharedArgs,
        normalized_head: &Expr,
        normalized_args: Option<Vec<Expr>>,
    ) -> Option<Vec<Expr>> {
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

    #[cfg(test)]
    pub(in crate::runtime) fn normalize_match_expr_changed_for_test(
        &self,
        expr: &Expr,
    ) -> Option<Expr> {
        self.normalize_match_expr_changed(expr)
    }

    #[cfg(test)]
    pub(in crate::runtime) fn normalize_match_expr_changed_with_call_depth_for_test(
        &self,
        expr: &Expr,
        remaining_call_depth: usize,
    ) -> Option<Expr> {
        self.normalize_match_expr_changed_with_call_depth(expr, remaining_call_depth)
    }

    #[cfg(test)]
    pub(in crate::runtime) fn normalize_match_expr_changed_recursive_bounded_for_test(
        &self,
        expr: &Expr,
        remaining_call_depth: usize,
    ) -> Option<Expr> {
        self.normalize_match_expr_changed_with_call_depth(expr, remaining_call_depth)
    }

    #[cfg(test)]
    pub(in crate::runtime) fn normalize_match_expr_changed_iterative_for_test(
        &self,
        expr: &Expr,
    ) -> Option<Expr> {
        self.normalize_match_expr_changed_iterative(expr)
    }
}

#[inline]
fn append_normalized_arg(
    args: &SharedArgs,
    index: usize,
    original_arg: &Expr,
    normalized_arg: Option<Expr>,
    normalized_args: &mut Option<Vec<Expr>>,
) {
    if let Some(normalized_arg) = normalized_arg {
        normalized_args
            .get_or_insert_with(|| args[..index].to_vec())
            .push(normalized_arg);
    } else if let Some(normalized_args) = normalized_args {
        normalized_args.push(original_arg.clone());
    }
}

fn finish_normalized_match_call(
    head: &SharedExpr,
    args: &SharedArgs,
    normalized_head: Option<Expr>,
    normalized_args: Option<Vec<Expr>>,
) -> Option<Expr> {
    match (normalized_head, normalized_args) {
        (None, None) => None,
        (Some(head), None) => Some(Expr::call_with_shared_args(head, args.clone())),
        (None, Some(args)) => Some(Expr::call_shared(head.clone(), SharedArgs::new(args))),
        (Some(head), Some(args)) => Some(Expr::call(head, args)),
    }
}
