use super::*;
use crate::runtime::kernel_api::{
    FrameRootTrace, FrameRootVisitor, FrozenOptionReadContinuation, OptionReadContinuation,
};

pub(super) struct LevelOptionWorking<'a> {
    pub(super) args: &'a SharedArgs,
}

#[derive(FrameRootTrace)]
pub(super) struct LevelOptionRead {
    args: SharedArgs,
}

impl OptionReadContinuation<bool> for LevelOptionWorking<'_> {
    type Frozen = LevelOptionRead;
    fn finish(self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        session.finish_level_option(self.args, value)
    }
    fn freeze(self) -> Self::Frozen {
        LevelOptionRead {
            args: self.args.clone(),
        }
    }
}
impl FrozenOptionReadContinuation<bool> for LevelOptionRead {
    fn finish(&self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        session.finish_level_option(&self.args, value)
    }
}
impl Session {
    fn finish_level_option(
        &mut self,
        args: &SharedArgs,
        default_include_heads: bool,
    ) -> Result<Expr, EvalError> {
        let Some(levels) = parse_level_level_spec(&args[1]) else {
            return Ok(self.invalid_level_spec_call("Level", args, &args[1]));
        };

        let (apply_head, include_heads) = match args.as_ref() {
            [_, _, third] => match classify_heads_option_expr(third) {
                HeadsOptionClassification::Value(include_heads) => {
                    (None, include_heads.unwrap_or(default_include_heads))
                }
                HeadsOptionClassification::NotOptionSyntax => {
                    (Some(third.clone()), default_include_heads)
                }
                classification => {
                    return Ok(self.invalid_heads_option_call(
                        "Level",
                        args,
                        third,
                        2,
                        classification,
                        "expected a Heads -> True/False option or apply head as the third argument",
                    ));
                }
            },
            [_, _, third, fourth] => {
                let classification = classify_heads_option_expr(fourth);
                let HeadsOptionClassification::Value(include_heads) = classification else {
                    return Ok(self.invalid_heads_option_call(
                        "Level",
                        args,
                        fourth,
                        3,
                        classification,
                        "expected a Heads -> True/False option as the fourth argument",
                    ));
                };

                (
                    Some(third.clone()),
                    include_heads.unwrap_or(default_include_heads),
                )
            }
            _ => (None, default_include_heads),
        };

        let traversal = CasesTraversalConfig {
            levels,
            max_matches: None,
            include_heads,
        };
        let parts = collect_level_parts_owned(&args[0], traversal, 0);
        Ok(Expr::call(
            apply_head.unwrap_or_else(|| system_expr("List")),
            parts,
        ))
    }
}
