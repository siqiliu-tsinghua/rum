use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
pub(super) struct EvalOnceBuiltinSuspensionFrame {
    pub(super) cache_key: std::sync::Arc<std::string::String>,
}

impl crate::runtime::suspension::SuspensionFrame for EvalOnceBuiltinSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut cache_key = self.cache_key.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let value = <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            session.once_values.insert(cache_key.as_ref().clone(), value.clone());
            session.refresh_memory_owner(crate::runtime::debug_tools::resources::MemoryOwner::OnceValues);
            Ok(value)
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
