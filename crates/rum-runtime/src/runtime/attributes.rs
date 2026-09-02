use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
struct AttributeUpdateConfig {
    attributes: AttributeSet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AttributeUpdateNormalizationError {
    InvalidTargets,
    InvalidAttributes,
}

impl AttributeUpdateNormalizationError {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidTargets => {
                "expected a symbol, symbol-name string, or list of those as the first argument"
            }
            Self::InvalidAttributes => {
                "expected an attribute symbol or list of attribute symbols as the second argument"
            }
        }
    }
}

impl Session {
    pub(super) fn eval_attributes(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => {
                if let Some(symbol) = self.normalize_symbol_target_expr(expr, false) {
                    Ok(self.attribute_set_for_symbol(&symbol).expr())
                } else {
                    Ok(self.invalid_call_with_arg_form(
                        "Attributes",
                        args,
                        "expected a symbol or symbol-name string argument",
                    ))
                }
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("Attributes", args, 1)),
        }
    }

    pub(super) fn eval_set_attributes(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        self.eval_update_attributes("SetAttributes", args, AttributeUpdate::Insert)
    }

    pub(super) fn eval_clear_attributes(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        self.eval_update_attributes("ClearAttributes", args, AttributeUpdate::Remove)
    }

    pub(super) fn eval_protect(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_protect_like("Protect", args, AttributeUpdate::Insert)
    }

    pub(super) fn eval_unprotect(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_protect_like("Unprotect", args, AttributeUpdate::Remove)
    }

    pub(super) fn eval_update_attributes(
        &mut self,
        symbol: &str,
        args: &SharedArgs,
        update: AttributeUpdate,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if args_slice.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count(symbol, args_slice, 2));
        }

        let mut targets = match self.normalize_attribute_update_targets(&args_slice[0]) {
            Ok(targets) => targets,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(symbol, args_slice, error.message()));
            }
        };

        let symbol_name = symbol.to_string();
        let resolved_result = self.evaluate(args_slice[1].clone());
        let resolved_attributes = crate::resume_or_suspend!(
            self,
            resolved_result,
            ok(value) => Ok(value),
            suspend => {
        let targets = Arc::<[Symbol]>::from(std::mem::take(&mut targets));
                    let args = args.clone();
        EvalUpdateAttributesSuspensionFrame {
            symbol_name: std::sync::Arc::new(symbol_name),
            args,
            targets,
            update,
        }
        },
        )?;

        self.finish_update_attributes_after_resolved_attributes(
            symbol,
            args_slice,
            &targets,
            update,
            resolved_attributes,
        )
    }

    fn finish_update_attributes_after_resolved_attributes(
        &mut self,
        symbol: &str,
        original_args: &[Expr],
        targets: &[Symbol],
        update: AttributeUpdate,
        resolved_attributes: Expr,
    ) -> Result<Expr, EvalError> {
        let config = match self.compile_attribute_update_from_resolved_args(&resolved_attributes) {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(symbol, original_args, error.message()));
            }
        };

        for target in targets {
            let removes_protected = update == AttributeUpdate::Remove
                && config.attributes.contains(Attribute::Protected);
            if self.symbol_is_protected(target) && !removes_protected {
                self.push_protected_write_message(symbol, target);
                continue;
            }

            let mut current = self.attribute_set_for_symbol(target);

            for attribute in config.attributes.attributes() {
                match update {
                    AttributeUpdate::Insert => current.insert(attribute),
                    AttributeUpdate::Remove => current.remove(attribute),
                }
            }

            self.set_attribute_set_for_symbol(*target, current);
        }

        Ok(system_expr("Null"))
    }

    fn eval_protect_like(
        &mut self,
        symbol: &str,
        args: &[Expr],
        update: AttributeUpdate,
    ) -> Result<Expr, EvalError> {
        let targets = match self.normalize_multi_symbol_targets(args) {
            Ok(targets) => targets,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(symbol, args, error.message()));
            }
        };

        for target in &targets {
            let mut current = self.attribute_set_for_symbol(target);
            match update {
                AttributeUpdate::Insert => current.insert(Attribute::Protected),
                AttributeUpdate::Remove => current.remove(Attribute::Protected),
            }
            self.set_attribute_set_for_symbol(target.clone(), current);
        }

        Ok(Expr::call(
            system_expr("List"),
            targets
                .into_iter()
                .map(|symbol| Expr::string(symbol.name().to_string()))
                .collect(),
        ))
    }

    fn normalize_attribute_update_targets(
        &self,
        expr: &Expr,
    ) -> Result<Vec<Symbol>, AttributeUpdateNormalizationError> {
        self.normalize_symbol_target_exprs(expr, true)
            .ok_or(AttributeUpdateNormalizationError::InvalidTargets)
    }

    fn compile_attribute_update_from_resolved_args(
        &self,
        resolved_attributes: &Expr,
    ) -> Result<AttributeUpdateConfig, AttributeUpdateNormalizationError> {
        let attributes = parse_attribute_set_expr(resolved_attributes)
            .ok_or(AttributeUpdateNormalizationError::InvalidAttributes)?;

        Ok(AttributeUpdateConfig { attributes })
    }

    pub(super) fn normalize_multi_symbol_targets(
        &self,
        args: &[Expr],
    ) -> Result<Vec<Symbol>, DefinitionTargetNormalizationError> {
        self.normalize_clear_targets(args)
    }

    pub(super) fn symbol_is_protected(&self, symbol: &Symbol) -> bool {
        self.attribute_set_for_symbol(symbol)
            .contains(Attribute::Protected)
    }

    pub(super) fn push_protected_write_message(&mut self, operator: &str, symbol: &Symbol) {
        self.push_message(operator, "wrsym", format!("Symbol {symbol} is Protected."));
    }

    /// Checks protection before a write and emits the standard write message.
    pub fn ensure_symbol_not_protected_for_write(
        &mut self,
        operator: &str,
        symbol: &Symbol,
    ) -> bool {
        if self.symbol_is_protected(symbol) {
            self.push_protected_write_message(operator, symbol);
            false
        } else {
            true
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalUpdateAttributesSuspensionFrame {
    symbol_name: std::sync::Arc<std::string::String>,
    args: rum_core::SharedArgs,
    targets: std::sync::Arc<[rum_core::Symbol]>,
    update: crate::runtime::session::session_state::AttributeUpdate,
}

impl crate::runtime::suspension::SuspensionFrame for EvalUpdateAttributesSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut symbol_name = self.symbol_name.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut targets = self.targets.clone();
        #[allow(unused_mut)]
        let mut update = self.update.clone();
        let resumed = match outcome {
            Ok(resolved) => (|| {
                session.finish_update_attributes_after_resolved_attributes(
                    symbol_name.as_str(),
                    args.as_ref(),
                    targets.as_ref(),
                    update,
                    resolved,
                )
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
