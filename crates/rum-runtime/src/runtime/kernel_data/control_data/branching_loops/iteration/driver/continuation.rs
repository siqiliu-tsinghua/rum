use super::*;
use crate::runtime::suspension::{result_from_continuation_expr, result_into_continuation_expr};

impl SuspendedLoopIteratorSymbolState {
    pub(super) fn into_continuation_expr(
        self,
        symbol: Symbol,
        outcome: Result<Expr, EvalError>,
    ) -> Expr {
        let saved_state = Arc::try_unwrap(self.0)
            .unwrap_or_else(|_| panic!("completed loop-iterator state must be uniquely owned"));
        Expr::call(
            system_expr("InternalContinuationLoopIteratorScope"),
            vec![
                Expr::Atom(Atom::Symbol(symbol)),
                saved_state.into_block_continuation_expr(),
                result_into_continuation_expr(outcome),
            ],
        )
    }

    pub(super) fn from_continuation_expr(
        expr: Expr,
    ) -> (Symbol, BlockSymbolState, Result<Expr, EvalError>) {
        let Expr::Call { head, args, .. } = expr else {
            panic!("unexpected loop-iterator continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationLoopIteratorScope"),
            "unexpected loop-iterator continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let Expr::Atom(Atom::Symbol(symbol)) = args
            .next()
            .expect("loop-iterator continuation stores its symbol")
        else {
            panic!("loop-iterator continuation symbol should be a symbol")
        };
        let saved_state = BlockSymbolState::from_block_continuation_expr(
            args.next()
                .expect("loop-iterator continuation stores its saved state"),
        );
        let outcome = result_from_continuation_expr(
            args.next()
                .expect("loop-iterator continuation stores its evaluation result"),
        );
        debug_assert!(args.next().is_none());
        (symbol, saved_state, outcome)
    }
}
