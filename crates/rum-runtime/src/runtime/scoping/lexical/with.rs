use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct PersistentWithBindings {
    base: Arc<SymbolExprMap>,
    tail: Option<Arc<WithBindingNode>>,
}

#[derive(FrameRootTrace)]
struct WithBindingNode {
    previous: Option<Arc<WithBindingNode>>,
    symbol: Symbol,
    value: Expr,
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedWithBindings {
    items: Arc<[(Symbol, Expr)]>,
    body: Expr,
    bindings: PersistentWithBindings,
    cursor: usize,
}

impl PersistentWithBindings {
    fn freeze(bindings: SymbolExprMap) -> Self {
        Self {
            base: Arc::new(bindings),
            tail: None,
        }
    }

    fn inserted(&self, symbol: Symbol, value: Expr) -> Self {
        Self {
            base: self.base.clone(),
            tail: Some(Arc::new(WithBindingNode {
                previous: self.tail.clone(),
                symbol,
                value,
            })),
        }
    }

    fn materialize(&self) -> SymbolExprMap {
        let mut delta = Vec::new();
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            delta.push((current.symbol, current.value.clone()));
            node = current.previous.as_deref();
        }
        delta.reverse();

        let mut bindings = self.base.as_ref().clone();
        bindings.extend(delta);
        bindings
    }
}

impl Session {
    pub(in crate::runtime) fn eval_with(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            if args.len() >= 3 {
                return self.eval_staged_with(args);
            }
            return Ok(self.invalid_call_with_exact_arg_count("With", args, 2));
        }

        let config = match self.compile_with_call_from_raw_args(args) {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("With", args, error.message()));
            }
        };

        self.resume_with_bindings(config.items, args[1].clone(), SymbolExprMap::default(), 0)
    }

    fn eval_staged_with(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        for declarations in &args[..args.len() - 1] {
            if let Err(error) =
                self.compile_with_call_from_raw_args(std::slice::from_ref(declarations))
            {
                return Ok(self.invalid_call_with_arg_form("With", args, error.message()));
            }
        }

        let mut nested = args[args.len() - 1].clone();
        for declarations in args[..args.len() - 1].iter().rev() {
            nested = Expr::call(system_expr("With"), vec![declarations.clone(), nested]);
        }

        self.evaluate(nested)
    }

    fn resume_with_bindings(
        &mut self,
        items: Vec<(Symbol, Expr)>,
        body: Expr,
        mut bindings: SymbolExprMap,
        mut index: usize,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some((symbol, rhs)) = items.get(index) else {
                let body = substitute_lexical_bindings(&body, &bindings, &BTreeSet::new());
                return self.evaluate(body);
            };

            let symbol = *symbol;
            let substituted_rhs = substitute_lexical_bindings(rhs, &bindings, &BTreeSet::new());
            match self.evaluate(substituted_rhs) {
                Ok(value) => {
                    bindings.insert(symbol, value);
                    index += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    let state = SuspendedWithBindings {
                        items: Arc::from(items),
                        body,
                        bindings: PersistentWithBindings::freeze(bindings),
                        cursor: index + 1,
                    };
                    crate::resume_or_suspend!(insert self, ResumeWithBindingsSuspensionFrame { state, symbol });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_with_binding(
        &mut self,
        mut state: SuspendedWithBindings,
        symbol: Symbol,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        state.bindings = state.bindings.inserted(symbol, value);

        loop {
            let Some((symbol, rhs)) = state.items.get(state.cursor) else {
                let bindings = state.bindings.materialize();
                let body = substitute_lexical_bindings(&state.body, &bindings, &BTreeSet::new());
                return self.evaluate(body);
            };

            let symbol = *symbol;
            let bindings = state.bindings.materialize();
            let substituted_rhs = substitute_lexical_bindings(rhs, &bindings, &BTreeSet::new());
            match self.evaluate(substituted_rhs) {
                Ok(value) => {
                    state.bindings = state.bindings.inserted(symbol, value);
                    state.cursor += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    state.cursor += 1;
                    crate::resume_or_suspend!(insert self, ResumeSuspendedWithBindingSuspensionFrame { state, symbol });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<PersistentWithBindings>();
    assert_traces::<SuspendedWithBindings>();
};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeWithBindingsSuspensionFrame {
    pub(super) state: crate::runtime::scoping::lexical::with::SuspendedWithBindings,
    pub(super) symbol: rum_core::Symbol,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeWithBindingsSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut symbol = self.symbol.clone();
        let resumed = (|| {
            let value = outcome?;
            session.resume_suspended_with_binding(state, symbol, value)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct ResumeSuspendedWithBindingSuspensionFrame {
    pub(super) state: crate::runtime::scoping::lexical::with::SuspendedWithBindings,
    pub(super) symbol: rum_core::Symbol,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeSuspendedWithBindingSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut symbol = self.symbol.clone();
        let resumed = (|| {
            let value = outcome?;
            session.resume_suspended_with_binding(state, symbol, value)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
