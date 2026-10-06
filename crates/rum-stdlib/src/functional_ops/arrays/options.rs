use super::*;
use rum_runtime::kernel_api::FrozenOptionReadContinuation;

pub(super) struct ArrayFilterOptionWorking<'a> {
    pub(super) args: &'a SharedArgs,
    pub(super) operator: &'a Expr,
    pub(super) shape: Vec<usize>,
    pub(super) flat: Vec<Expr>,
    pub(super) spec: ArrayFilterSpec,
}

#[derive(FrameRootTrace)]
pub(super) struct ArrayFilterOptionRead {
    args: SharedArgs,
}

impl OptionReadContinuation<Option<Expr>> for ArrayFilterOptionWorking<'_> {
    type Frozen = ArrayFilterOptionRead;
    #[inline(always)]
    fn finish(
        self,
        session: &mut Session,
        effective_padding_expr: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        let padding = match effective_padding_expr.as_ref() {
            None => ArrayFilterPadding::Uniform(ArrayFilterPaddingLeaf::Fixed),
            Some(padding_expr) => {
                let Some(parsed) = parse_array_filter_padding(padding_expr, self.shape.len())
                else {
                    return Ok(session.invalid_call_with_arg_form(
                        "ArrayFilter",
                        self.args,
                        "expected Padding -> value, Padding -> None, Padding -> \"Fixed\", Padding -> \"Periodic\", a cyclic padding list, or matching per-axis padding pairs as the fourth argument",
                    ));
                };
                parsed
            }
        };

        apply_array_filter(
            session,
            self.operator,
            &self.shape,
            &self.flat,
            &self.spec,
            &padding,
        )
    }
    fn freeze(self) -> Self::Frozen {
        ArrayFilterOptionRead {
            args: self.args.clone(),
        }
    }
}

impl FrozenOptionReadContinuation<Option<Expr>> for ArrayFilterOptionRead {
    fn finish(&self, session: &mut Session, value: Option<Expr>) -> Result<Expr, EvalError> {
        // These parsers only inspect the already-evaluated, validated arguments.
        // Rebuild their pure shape/radius data only on this cold resume path.
        let (shape, flat) =
            parse_rectangular_list_array(&self.args[1]).expect("validated ArrayFilter array");
        let spec = parse_array_filter_spec(&self.args[2], shape.len())
            .expect("validated ArrayFilter radius");
        ArrayFilterOptionWorking {
            args: &self.args,
            operator: &self.args[0],
            shape,
            flat,
            spec,
        }
        .finish(session, value)
    }
}
