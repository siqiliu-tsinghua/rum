use super::*;
use crate::runtime::kernel_api::{FrozenOptionReadContinuation, OptionReadContinuation};
pub(super) struct TraceOptionWorking<'a> {
    pub(super) args: &'a [Expr],
    pub(super) name: &'static str,
    pub(super) output: TraceOutput,
}

#[derive(FrameRootTrace)]
pub(super) struct TraceOptionRead {
    args: SharedArgs,
    name: &'static str,
    output: TraceOutput,
}

impl OptionReadContinuation<bool> for TraceOptionWorking<'_> {
    type Frozen = TraceOptionRead;
    fn finish(self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        session.finish_trace_option(self.args, self.name, self.output, value)
    }
    fn freeze(self) -> Self::Frozen {
        TraceOptionRead {
            args: SharedArgs::new(self.args.to_vec()),
            name: self.name,
            output: self.output,
        }
    }
}

impl FrozenOptionReadContinuation<bool> for TraceOptionRead {
    fn finish(&self, session: &mut Session, value: bool) -> Result<Expr, EvalError> {
        session.finish_trace_option(&self.args, self.name, self.output, value)
    }
}

impl Session {
    fn finish_trace_option(
        &mut self,
        args: &[Expr],
        name: &'static str,
        output: TraceOutput,
        default_include_heads: bool,
    ) -> Result<Expr, EvalError> {
        if let [expr] = args {
            return self.eval_structured_trace_capture_expr(TraceEvalState {
                original: expr.clone(),
                pattern: None,
                levels: None,
                include_heads: default_include_heads,
                output,
            });
        }

        let (expr, pattern, levels, include_heads) = match args {
            [expr, pattern] => (expr, Some(pattern.clone()), None, default_include_heads),
            [expr, pattern, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (expr, Some(pattern.clone()), None, include_heads)
                } else {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return Ok(self.invalid_call_with_arg_form(
                            name,
                            args,
                            "expected a standard levelspec or Heads -> True|False in the current structured-trace subset",
                        ));
                    };
                    (
                        expr,
                        Some(pattern.clone()),
                        Some(levels),
                        default_include_heads,
                    )
                }
            }
            [expr, pattern, levels_expr, option] => {
                let Some(levels) = parse_standard_level_spec(levels_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        name,
                        args,
                        "expected a standard levelspec as the third argument in the current structured-trace subset",
                    ));
                };
                let Some(include_heads) =
                    parse_heads_option_with_default(option, default_include_heads)
                else {
                    return Ok(self.invalid_call_with_arg_form(
                        name,
                        args,
                        "expected Heads -> True|False as the optional fourth argument in the current structured-trace subset",
                    ));
                };
                (expr, Some(pattern.clone()), Some(levels), include_heads)
            }
            _ => return Ok(self.invalid_call_with_arg_range(name, args, 2, 4)),
        };

        self.eval_structured_trace_capture_expr(TraceEvalState {
            original: expr.clone(),
            pattern,
            levels,
            include_heads,
            output,
        })
    }
}
