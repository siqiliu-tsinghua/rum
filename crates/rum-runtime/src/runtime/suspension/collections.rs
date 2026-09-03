use super::*;

impl ContinuationValue for CasesCollectionOutcome {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationCasesCollection"),
            vec![
                Expr::call(system_expr("List"), self.matches),
                Expr::integer(BigInt::from(self.depth)),
                system_expr(if self.exhausted { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationCasesCollection")
                    && args.len() == 3 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let exhausted_expr = args
                    .pop()
                    .expect("cases-collection payload stores exhausted");
                let depth_expr = args.pop().expect("cases-collection payload stores depth");
                let matches_expr = args.pop().expect("cases-collection payload stores matches");

                let matches = match matches_expr {
                    Expr::Call { head, args, .. } if is_system_symbol_name(&head, "List") => {
                        args.into_vec()
                    }
                    other => vec![other],
                };

                Self {
                    matches,
                    depth: expr_as_non_negative_usize(&depth_expr)
                        .expect("cases-collection depth should be non-negative"),
                    exhausted: is_system_symbol_name(&exhausted_expr, "True"),
                }
            }
            other => {
                panic!("unexpected continuation payload for CasesCollectionOutcome: {other:?}")
            }
        }
    }
}

impl ContinuationValue for PositionCollectionOutcome {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationPositionCollection"),
            vec![
                Expr::call(system_expr("List"), self.positions),
                Expr::integer(BigInt::from(self.depth)),
                system_expr(if self.exhausted { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationPositionCollection")
                    && args.len() == 3 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let exhausted_expr = args
                    .pop()
                    .expect("position-collection payload stores exhausted");
                let depth_expr = args
                    .pop()
                    .expect("position-collection payload stores depth");
                let positions_expr = args
                    .pop()
                    .expect("position-collection payload stores positions");

                let positions = match positions_expr {
                    Expr::Call { head, args, .. } if is_system_symbol_name(&head, "List") => {
                        args.into_vec()
                    }
                    other => vec![other],
                };

                Self {
                    positions,
                    depth: expr_as_non_negative_usize(&depth_expr)
                        .expect("position-collection depth should be non-negative"),
                    exhausted: is_system_symbol_name(&exhausted_expr, "True"),
                }
            }
            other => {
                panic!("unexpected continuation payload for PositionCollectionOutcome: {other:?}")
            }
        }
    }
}

impl ContinuationValue for TraversalCountOutcome {
    fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationTraversalCount"),
            vec![
                Expr::integer(BigInt::from(self.count)),
                Expr::integer(BigInt::from(self.depth)),
                system_expr(if self.exhausted { "True" } else { "False" }),
            ],
        )
    }

    fn from_continuation_expr(expr: Expr) -> Self {
        match expr {
            Expr::Call { head, args, .. }
                if is_system_symbol_name(&head, "InternalContinuationTraversalCount")
                    && args.len() == 3 =>
            {
                let mut args = Expr::unwrap_shared_args(args);
                let exhausted_expr = args
                    .pop()
                    .expect("traversal-count payload stores exhausted");
                let depth_expr = args.pop().expect("traversal-count payload stores depth");
                let count_expr = args.pop().expect("traversal-count payload stores count");

                Self {
                    count: expr_as_non_negative_usize(&count_expr)
                        .expect("traversal-count count should be non-negative"),
                    depth: expr_as_non_negative_usize(&depth_expr)
                        .expect("traversal-count depth should be non-negative"),
                    exhausted: is_system_symbol_name(&exhausted_expr, "True"),
                }
            }
            other => {
                panic!("unexpected continuation payload for TraversalCountOutcome: {other:?}")
            }
        }
    }
}
