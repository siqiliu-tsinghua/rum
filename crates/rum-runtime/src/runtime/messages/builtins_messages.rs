use super::super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

impl Session {
    pub(in crate::runtime) fn eval_message(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let Some(spec) = args.first() else {
            return Ok(self.invalid_call_with_message(
                "Message",
                args,
                "argx",
                "expected at least 1 argument",
            ));
        };

        let key = match self.normalize_message_emit_target(spec) {
            Ok(key) => key,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Message", args, error.message()));
            }
        };

        let rendered_text = self.render_message_text(&key, &args[1..]);
        let symbol_name = key.symbol.to_string();
        let tag = key.tag.clone();

        crate::resume_or_suspend!(
            self,
            rendered_text,
            ok(text) => { let session = self; (|| {
                    session.push_message_unchecked(&symbol_name, &tag, text);
                    Ok(system_expr("Null"))
                })() },
            suspend => EvalMessageSuspensionFrame {
                symbol_name: std::sync::Arc::new(symbol_name),
                tag: std::sync::Arc::new(tag),
            },
        )
    }

    pub(in crate::runtime) fn eval_message_name(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("MessageName", args, 2));
        }

        let Some(key) = self.resolve_message_name_args(&args[0], &args[1]) else {
            return Ok(self.invalid_call_with_arg_form(
                "MessageName",
                args,
                "expected a symbol or symbol-name string, plus a message tag string or symbol",
            ));
        };

        Ok(message_name_expr(key.symbol, key.tag))
    }

    pub(in crate::runtime) fn eval_messages(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(self.message_names_expr_for_symbol(None)),
            [expr] => match self.normalize_message_query_target(expr) {
                Ok(MessageQueryTarget::Key(key)) => {
                    let sealed_hidden = !self.sealed_symbols_visible()
                        && crate::runtime::scoping::packages::symbol_is_sealed_private(&key.symbol);
                    let items = if !sealed_hidden && self.message_definitions.contains_key(&key) {
                        vec![message_name_expr(key.symbol, key.tag)]
                    } else {
                        Vec::new()
                    };
                    Ok(Expr::call(system_expr("List"), items))
                }
                Ok(MessageQueryTarget::Symbol(symbol)) => {
                    Ok(self.message_names_expr_for_symbol(Some(&symbol)))
                }
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("Messages", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_message(
                "Messages",
                args,
                "argx",
                format!("expected 0 or 1 argument(s), got {}", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn eval_message_definitions(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(self.message_definitions_expr_for_symbol(None)),
            [expr] => match self.normalize_message_query_target(expr) {
                Ok(MessageQueryTarget::Key(key)) => Ok(self.message_definitions_expr_for_key(&key)),
                Ok(MessageQueryTarget::Symbol(symbol)) => {
                    Ok(self.message_definitions_expr_for_symbol(Some(&symbol)))
                }
                Err(error) => Ok(self.invalid_call_with_arg_form(
                    "MessageDefinitions",
                    args,
                    error.message(),
                )),
            },
            _ => Ok(self.invalid_call_with_message(
                "MessageDefinitions",
                args,
                "argx",
                format!("expected 0 or 1 argument(s), got {}", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn eval_message_tags(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(self.message_tags_expr_for_symbol(None)),
            [expr] => match self.normalize_message_query_target(expr) {
                Ok(MessageQueryTarget::Key(key)) => Ok(self.message_tags_expr_for_key(&key)),
                Ok(MessageQueryTarget::Symbol(symbol)) => {
                    Ok(self.message_tags_expr_for_symbol(Some(&symbol)))
                }
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("MessageTags", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_message(
                "MessageTags",
                args,
                "argx",
                format!("expected 0 or 1 argument(s), got {}", args.len()),
            )),
        }
    }
}

#[derive(rum_derive::FrameRootTrace, rum_derive::PureFrameAbandon)]
struct EvalMessageSuspensionFrame {
    symbol_name: std::sync::Arc<std::string::String>,
    tag: std::sync::Arc<std::string::String>,
}

impl crate::runtime::suspension::SuspensionFrame for EvalMessageSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut symbol_name = self.symbol_name.clone();
        #[allow(unused_mut)]
        let mut tag = self.tag.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let text = <std::string::String as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            session.push_message_unchecked(symbol_name.as_str(), tag.as_str(), text);
            Ok(system_expr("Null"))
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
