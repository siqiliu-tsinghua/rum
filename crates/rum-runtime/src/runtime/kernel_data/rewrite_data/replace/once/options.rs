use super::*;
use crate::runtime::kernel_api::{FrozenOptionReadContinuation, OptionReadContinuation};

pub(super) struct ReplaceOptionWorking {
    pub(super) target: Expr,
    pub(super) rule_plan: RewriteRulePlan,
    pub(super) levels_expr: Option<Expr>,
    pub(super) heads_option_exprs: Vec<Expr>,
    pub(super) call_args: Vec<Expr>,
}
#[derive(FrameRootTrace)]
pub(super) struct ReplaceOptionRead {
    target: Expr,
    rule_plan: RewriteRulePlan,
    levels_expr: Option<Expr>,
    heads_option_exprs: SharedArgs,
    call_args: SharedArgs,
}
impl OptionReadContinuation<bool> for ReplaceOptionWorking {
    type Frozen = ReplaceOptionRead;
    fn finish(self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        session.finish_replace_option(self, value)
    }
    fn freeze(self) -> Self::Frozen {
        ReplaceOptionRead {
            target: self.target,
            rule_plan: self.rule_plan,
            levels_expr: self.levels_expr,
            heads_option_exprs: SharedArgs::new(self.heads_option_exprs),
            call_args: SharedArgs::new(self.call_args),
        }
    }
}
impl FrozenOptionReadContinuation<bool> for ReplaceOptionRead {
    fn finish(&self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        session.finish_replace_option(
            ReplaceOptionWorking {
                target: self.target.clone(),
                rule_plan: self.rule_plan.clone(),
                levels_expr: self.levels_expr.clone(),
                heads_option_exprs: self.heads_option_exprs.as_ref().to_vec(),
                call_args: self.call_args.as_ref().to_vec(),
            },
            value,
        )
    }
}
impl Session {
    fn finish_replace_option(
        &mut self,
        next: ReplaceOptionWorking,
        default_include_heads: bool,
    ) -> Result<Expr, EvalError> {
        let ReplaceOptionWorking {
            target,
            rule_plan,
            levels_expr,
            heads_option_exprs,
            call_args,
        } = next;
        let heads_result =
            self.resolve_replace_heads_options_owned(heads_option_exprs, default_include_heads);
        match heads_result {
            Ok(include_heads) => self.finish_replace_heads_resolution(
                target,
                rule_plan,
                levels_expr,
                call_args,
                include_heads,
            ),
            Err(EvalError::InputRequired { request }) => {
                let call_args = SharedArgs::new(call_args);
                let next = ContinueReplaceSetupFrame {
                    rule_plan: Some(rule_plan.clone()),
                };
                crate::resume_or_suspend!(insert self, ResumeReplaceAfterRuleExprsSuspensionFrame { target, rule_plan, levels_expr, call_args });
                crate::resume_or_suspend!(insert self, next);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }
}
