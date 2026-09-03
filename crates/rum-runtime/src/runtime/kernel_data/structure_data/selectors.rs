use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use crate::runtime::kernel_data::evaluated_part_selectors::EvaluatedPartSelectors;

mod apply;
mod diagnostics;
mod extract;
mod parse;
mod spans;

pub(super) use apply::apply_part_selectors;
pub(super) use extract::parse_single_position_spec;
pub(super) use parse::{
    invalid_part_selector_for_part, parse_part_selectors, parse_part_selectors_for_part,
};
pub(super) use spans::select_positioned_parts_for_span;

use apply::{collect_missing_key_selections, selector_mixes_key_and_non_key};
use diagnostics::{
    diagnose_part_selection_failure, invalid_span_message_spec, span_take_message_range,
};
use extract::{extract_operator_form, parse_extract_position_spec, wrap_extracted_value};
use parse::{
    first_invalid_span_expr, is_invalid_numeric_upto_limit, is_invalid_span_expr,
    parse_part_selector_for_part,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) enum AtomicPartSpec {
    Index(isize),
    All,
    Span(PartSpanSpec),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) struct PartSpanSpec {
    pub(super) start: SpanEndpoint,
    pub(super) end: SpanEndpoint,
    pub(super) step: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) enum SpanEndpoint {
    Index(isize),
    UpTo(Option<isize>),
    All,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) enum PartSelector {
    Single(AtomicPartSpec),
    Key(Expr),
    Multi(Vec<PartSelector>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SelectedPartShape {
    Leaf(Expr),
    Branch(Vec<SelectedPartShape>),
    AssociationBranch(Vec<(Expr, SelectedPartShape)>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExtractPositionSpec {
    position_specs: Vec<Vec<PartSelector>>,
    multiple: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PartSelectionFailure {
    PartW { part: Expr, target: Expr },
    PartD { spec: Option<Expr> },
    Take { span: PartSpanSpec, target: Expr },
    Pspec1 { spec: Expr },
}

impl Session {
    pub(in crate::runtime) fn push_existing_part_selection_failure_messages(
        &mut self,
        target: &Expr,
        selector_exprs: &EvaluatedPartSelectors,
    ) -> bool {
        let selector_exprs = selector_exprs.as_slice();
        let Some(selectors) = parse_part_selectors_for_part(selector_exprs) else {
            return false;
        };
        let Some(failure) = diagnose_part_selection_failure(self, target, &selectors) else {
            return false;
        };

        let mut args = Vec::with_capacity(selector_exprs.len() + 1);
        args.push(target.clone());
        args.extend_from_slice(selector_exprs);
        let original = Expr::call(system_expr("Part"), args);
        self.push_part_selection_failure("Part", failure, &original);
        true
    }

    pub(in crate::runtime) fn eval_part(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(self.invalid_call_with_message(
                "Part",
                args,
                "argm",
                "Part called with 0 arguments; 1 or more arguments are expected.",
            ));
        }

        if let [
            Expr::Call {
                head: lazy_head, ..
            },
            index_expr,
        ] = args.as_ref()
        {
            if is_system_symbol_name(lazy_head, "LazyList") {
                if let Some(index) = expr_as_exact_integer(index_expr) {
                    if index > BigInt::from(0_u8) {
                        let lazy_drop = Expr::call(
                            system_expr("LazyDrop"),
                            vec![args[0].clone(), Expr::integer(index - BigInt::from(1_u8))],
                        );
                        return self.evaluate(Expr::call(system_expr("First"), vec![lazy_drop]));
                    }
                }
            }
        }

        if let Some(value) = fast_exact_integer_part_path(args) {
            return Ok(value);
        }

        let Some(selectors) = parse_part_selectors_for_part(&args[1..]) else {
            if let Some(offender) = args[1..]
                .iter()
                .find(|selector| parse_part_selector_for_part(selector).is_none())
            {
                if is_invalid_span_expr(offender) {
                    self.push_invalid_span_selector_messages("Part", offender);
                } else {
                    self.push_message(
                        "Part",
                        "pkspec1",
                        format!(
                            "The expression {} cannot be used as a part specification.",
                            offender
                        ),
                    );
                }
            }
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if selectors.iter().any(selector_mixes_key_and_non_key)
            && self.association_entry_cursor(&args[0]).is_some()
        {
            self.push_message(
                "Part",
                "pmix",
                "Integer and Key specifications cannot be mixed in the same part specification.",
            );
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        let original = Expr::call(head.clone(), args.to_vec());
        let Some(value) = apply_part_selectors(self, &args[0], &selectors) else {
            if let Some(failure) = diagnose_part_selection_failure(self, &args[0], &selectors) {
                self.push_part_selection_failure("Part", failure, &original);
            }
            return Ok(original);
        };

        Ok(value)
    }

    pub(in crate::runtime) fn eval_extract(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        if let [position_spec] = args.as_ref() {
            return Ok(extract_operator_form(position_spec.clone()));
        }

        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_message(
                "Extract",
                args,
                "argt",
                format!(
                    "Extract called with {} arguments; 2 or 3 arguments are expected.",
                    args.len()
                ),
            ));
        }

        let Some(position_spec) = parse_extract_position_spec(&args[1]) else {
            if let Some(span_expr) = first_invalid_span_expr(&args[1]) {
                self.push_invalid_span_selector_messages("Extract", span_expr);
            } else {
                self.push_message(
                    "Extract",
                    "psl1",
                    format!(
                        "Position specification {} in {} is not applicable.",
                        args[1],
                        Expr::call(head.clone(), args.to_vec())
                    ),
                );
            }
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let extracted = if position_spec.multiple {
            let mut items = Vec::new();
            for selectors in &position_spec.position_specs {
                let Some(value) = apply_part_selectors(self, &args[0], selectors) else {
                    if let Some(failure) =
                        diagnose_part_selection_failure(self, &args[0], selectors)
                    {
                        let original = Expr::call(head.clone(), args.to_vec());
                        self.push_part_selection_failure("Extract", failure, &original);
                    }
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                self.push_extract_missing_key_messages(&args[0], selectors);
                items.push(wrap_extracted_value(value, args.get(2)));
            }
            Expr::call(system_expr("List"), items)
        } else {
            let Some(value) =
                apply_part_selectors(self, &args[0], &position_spec.position_specs[0])
            else {
                if let Some(failure) = diagnose_part_selection_failure(
                    self,
                    &args[0],
                    &position_spec.position_specs[0],
                ) {
                    let original = Expr::call(head.clone(), args.to_vec());
                    self.push_part_selection_failure("Extract", failure, &original);
                }
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            self.push_extract_missing_key_messages(&args[0], &position_spec.position_specs[0]);
            wrap_extracted_value(value, args.get(2))
        };

        Ok(extracted)
    }

    fn push_extract_missing_key_messages(&mut self, expr: &Expr, selectors: &[PartSelector]) {
        let mut missing_keys = Vec::new();
        collect_missing_key_selections(self, expr, selectors, &mut missing_keys);
        for (key, target) in missing_keys {
            self.push_message(
                "Extract",
                "keyw",
                format!("Key {key} does not exist in {target}."),
            );
        }
    }

    fn push_part_selection_failure(
        &mut self,
        symbol: &str,
        failure: PartSelectionFailure,
        original: &Expr,
    ) {
        match failure {
            PartSelectionFailure::PartW { part, target } => self.push_message(
                symbol,
                "partw",
                format!("Part {} of {} does not exist.", part, target),
            ),
            PartSelectionFailure::PartD { spec } => {
                let rendered_spec = if symbol == "Extract" {
                    spec.as_ref().unwrap_or(original)
                } else {
                    original
                };
                self.push_message(
                    symbol,
                    "partd",
                    format!(
                        "Part specification {} is longer than depth of object.",
                        rendered_spec
                    ),
                );
            }
            PartSelectionFailure::Take { span, target } => {
                self.push_take_selection_failure(symbol, span, target);
            }
            PartSelectionFailure::Pspec1 { spec } => self.push_message(
                symbol,
                "pspec1",
                format!("Part specification {} is not applicable.", spec),
            ),
        }
    }

    fn push_take_selection_failure(&mut self, symbol: &str, span: PartSpanSpec, target: Expr) {
        let (start, end) = span_take_message_range(span);
        let text = format!(
            "Cannot take positions {} through {} in {}.",
            start, end, target
        );
        if self.messages.last().is_some_and(|message| {
            message.symbol == symbol && message.tag == "take" && message.text == text
        }) {
            return;
        }

        self.push_message(symbol, "take", text);
    }

    fn push_invalid_span_selector_messages(&mut self, symbol: &str, span_expr: &Expr) {
        self.push_invalid_upto_messages_in_span(span_expr);
        let text = format!(
            "{} is not a valid Span specification. A Span specification should be 1, 2 or 3 machine-sized integers separated by ;;. (Any of the integers can be omitted or replaced with All.)",
            invalid_span_message_spec(span_expr)
        );
        if self.has_message(symbol, "span", &text) {
            return;
        }

        self.push_message(symbol, "span", text);
    }

    fn has_message(&self, symbol: &str, tag: &str, text: &str) -> bool {
        self.messages
            .iter()
            .any(|message| message.symbol == symbol && message.tag == tag && message.text == text)
    }

    fn push_invalid_upto_message_once(&mut self, tag: &str, text: String) {
        if self.has_message("UpTo", tag, &text) {
            return;
        }

        self.push_message("UpTo", tag, text);
    }

    fn push_invalid_upto_messages_in_span(&mut self, expr: &Expr) {
        let Expr::Call { head, args, .. } = expr else {
            return;
        };

        if is_system_symbol_name(head, "UpTo") {
            match args.as_slice() {
                [limit] if is_invalid_numeric_upto_limit(limit) => self
                    .push_invalid_upto_message_once(
                        "innf",
                        format!(
                            "Non-negative integer or Infinity expected at position 1 in {expr}."
                        ),
                    ),
                [_] => {}
                _ => self.push_invalid_upto_message_once(
                    "argx",
                    format!(
                        "UpTo called with {} arguments; 1 argument is expected.",
                        args.len()
                    ),
                ),
            }
            return;
        }

        for arg in args {
            self.push_invalid_upto_messages_in_span(arg);
        }
    }
}

fn fast_exact_integer_part_path(args: &[Expr]) -> Option<Expr> {
    let [_target, selector_exprs @ ..] = args else {
        return None;
    };
    if selector_exprs.is_empty() {
        return None;
    }

    let mut selected = None;
    for selector in selector_exprs {
        let current = selected.as_ref().unwrap_or(&args[0]);
        if has_association_head(current) {
            return None;
        }
        let index = expr_as_signed_isize(selector)?;
        selected = Some(select_exact_part(current, index)?);
    }
    selected
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<AtomicPartSpec>();
    assert_traces::<PartSelector>();
    assert_traces::<PartSpanSpec>();
    assert_traces::<SpanEndpoint>();
};
