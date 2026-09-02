use super::super::*;
use super::definition_lhs_preparation::{
    DefinitionLhsPreparationContinuation, PreparedDefinitionLhs,
};
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn eval_up_set(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        let [lhs, rhs] = args.as_ref() else {
            return Ok(self.invalid_call_with_exact_arg_count("UpSet", args.as_ref(), 2));
        };

        let lhs = lhs.clone();
        let rhs_result = self.evaluate(rhs.clone());
        let rhs = crate::resume_or_suspend!(
            self,
            rhs_result,
            ok(value) => Ok(value),
            suspend => {
                let lhs = lhs.clone();
                EvalUpSetSuspensionFrame { lhs }
            },
        )?;

        self.prepare_up_set_assignment(lhs, rhs, false)
    }

    pub(in crate::runtime) fn eval_up_set_delayed(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let [lhs, rhs] = args.as_ref() else {
            return Ok(self.invalid_call_with_exact_arg_count("UpSetDelayed", args.as_ref(), 2));
        };

        self.prepare_up_set_assignment(lhs.clone(), rhs.clone(), true)
    }

    pub(in crate::runtime) fn eval_tag_set(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() != 3 {
            return Ok(self.invalid_call_with_exact_arg_count("TagSet", args_slice, 3));
        }

        // The explicit tag is normalized before any user code in the rhs or
        // lhs, preserving TagSet's established sequencing contract.
        let tag = match self.normalize_tag_target(&args_slice[0]) {
            Ok(tag) => tag,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("TagSet", args_slice, error.message()));
            }
        };

        let rhs_result = self.evaluate(args_slice[2].clone());
        let rhs = crate::resume_or_suspend!(
            self,
            rhs_result,
            ok(value) => Ok(value),
            suspend => {
                let args = args.clone();
                let tag = tag;
                EvalTagSetSuspensionFrame { args, tag }
            },
        )?;
        self.prepare_tag_set_assignment(tag, args.clone(), rhs, false)
    }

    pub(in crate::runtime) fn eval_tag_set_delayed(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() != 3 {
            return Ok(self.invalid_call_with_exact_arg_count("TagSetDelayed", args_slice, 3));
        }

        let tag = match self.normalize_tag_target(&args_slice[0]) {
            Ok(tag) => tag,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "TagSetDelayed",
                    args_slice,
                    error.message(),
                ));
            }
        };

        self.prepare_tag_set_assignment(tag, args.clone(), args_slice[2].clone(), true)
    }

    pub(in crate::runtime) fn eval_tag_unset(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("TagUnset", args_slice, 2));
        }

        let tag = match self.normalize_tag_target(&args_slice[0]) {
            Ok(tag) => tag,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "TagUnset",
                    args_slice,
                    error.message(),
                ));
            }
        };

        if !self.ensure_symbol_not_protected_for_write("TagUnset", &tag) {
            return Ok(system_expr("Null"));
        }

        self.prepare_definition_lhs(
            args_slice[1].clone(),
            DefinitionLhsPreparationContinuation::TagUnset {
                tag,
                args: args.clone(),
            },
        )
    }

    fn prepare_up_set_assignment(
        &mut self,
        lhs: Expr,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        self.prepare_definition_lhs(
            lhs,
            DefinitionLhsPreparationContinuation::UpSet { rhs, delayed },
        )
    }

    fn prepare_tag_set_assignment(
        &mut self,
        tag: Symbol,
        args: SharedArgs,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        let lhs = args
            .get(1)
            .expect("validated TagSet arguments retain their lhs")
            .clone();
        self.prepare_definition_lhs(
            lhs,
            DefinitionLhsPreparationContinuation::TagSet {
                tag,
                args,
                rhs,
                delayed,
            },
        )
    }

    pub(super) fn finish_prepared_tag_set(
        &mut self,
        prepared: PreparedDefinitionLhs,
        tag: Symbol,
        args: SharedArgs,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        let symbol = if delayed { "TagSetDelayed" } else { "TagSet" };
        match prepared.into_tagged_store(&tag) {
            Ok(store) => {
                if !self.ensure_symbol_not_protected_for_write(symbol, store.symbol()) {
                    return Ok(if delayed { system_expr("Null") } else { rhs });
                }
                self.store_prepared_definition(store, rhs.clone(), delayed);
                Ok(if delayed { system_expr("Null") } else { rhs })
            }
            Err(error) => Ok(self.invalid_tagged_definition_call(symbol, args.as_ref(), error)),
        }
    }

    pub(super) fn finish_prepared_tag_unset(
        &mut self,
        prepared: PreparedDefinitionLhs,
        tag: Symbol,
        args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        match prepared.into_tagged_removal(&tag) {
            Ok(removal) => {
                // Preparation can run user code, including Protect, so the
                // original early check is repeated at the mutation boundary.
                if self.ensure_symbol_not_protected_for_write("TagUnset", removal.symbol()) {
                    self.remove_prepared_definition(removal);
                }
                Ok(system_expr("Null"))
            }
            Err(error) => Ok(self.invalid_tagged_definition_call("TagUnset", args.as_ref(), error)),
        }
    }

    pub(super) fn finish_prepared_up_set(
        &mut self,
        prepared: PreparedDefinitionLhs,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        let symbol = if delayed { "UpSetDelayed" } else { "UpSet" };
        let stores = match prepared.into_upset_stores() {
            Ok(stores) => stores,
            Err(lhs) => {
                self.push_message(
                    symbol,
                    "nosym",
                    format!("No valid tag symbol occurs in the assignment lhs {lhs}."),
                );
                return Ok(if delayed { system_expr("Null") } else { rhs });
            }
        };

        for store in stores {
            if !self.ensure_symbol_not_protected_for_write(symbol, store.symbol()) {
                continue;
            }
            self.store_prepared_definition(store, rhs.clone(), delayed);
        }

        Ok(if delayed { system_expr("Null") } else { rhs })
    }

    pub(in crate::runtime) fn invalid_tagged_definition_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        error: TagDefinitionError,
    ) -> Expr {
        match error {
            TagDefinitionError::TagNotFound => self.invalid_call_with_message(
                symbol,
                args,
                "tagnf",
                "tag symbol does not occur in the assignment lhs",
            ),
            TagDefinitionError::TagPositionInvalid => self.invalid_call_with_message(
                symbol,
                args,
                "tagpos",
                "tag symbol must appear within the outer two levels of the assignment lhs",
            ),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalUpSetSuspensionFrame {
    lhs: Expr,
}

impl crate::runtime::suspension::SuspensionFrame for EvalUpSetSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(rhs) => session.prepare_up_set_assignment(self.lhs.clone(), rhs, false),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalTagSetSuspensionFrame {
    args: SharedArgs,
    tag: Symbol,
}

impl crate::runtime::suspension::SuspensionFrame for EvalTagSetSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(rhs) => session.prepare_tag_set_assignment(self.tag, self.args.clone(), rhs, false),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
