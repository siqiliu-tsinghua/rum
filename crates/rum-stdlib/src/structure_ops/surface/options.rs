use super::*;
use rum_runtime::kernel_api::{
    FrameRootTrace, FrameRootVisitor, FrozenOptionReadContinuation, OptionReadContinuation,
};

pub(super) struct DepthOptionWorking<'a> {
    pub(super) args: &'a SharedArgs,
}
#[derive(FrameRootTrace)]
pub(super) struct DepthOptionRead {
    args: SharedArgs,
}

impl OptionReadContinuation<bool> for DepthOptionWorking<'_> {
    type Frozen = DepthOptionRead;
    fn finish(self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        finish_depth_option(session, self.args, value)
    }
    fn freeze(self) -> Self::Frozen {
        DepthOptionRead {
            args: self.args.clone(),
        }
    }
}
impl FrozenOptionReadContinuation<bool> for DepthOptionRead {
    fn finish(&self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        finish_depth_option(session, &self.args, value)
    }
}
fn finish_depth_option(
    session: &mut Session,
    args: &[Expr],
    default_include_heads: bool,
) -> Result<Expr, EvalError> {
    let (expr, include_heads) = match args {
        [expr] => (expr, default_include_heads),
        [expr, option] => {
            let classification = classify_heads_option_expr(option);
            let HeadsOptionClassification::Value(include_heads) = classification else {
                return Ok(session.invalid_heads_option_call(
                    "Depth",
                    args,
                    option,
                    1,
                    classification,
                    "expected a Heads -> True/False option as the second argument",
                ));
            };
            (expr, include_heads.unwrap_or(default_include_heads))
        }
        _ => return Ok(session.invalid_call_with_arg_range("Depth", args, 1, 2)),
    };

    Ok(Expr::integer(BigInt::from(structural_depth(
        expr,
        include_heads,
    ))))
}
