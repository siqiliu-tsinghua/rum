use super::status::is_failure_q_expr;
use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
enum ConfirmQuietSelectors {
    All,
    Exact(Vec<MessageSelector>),
}

#[derive(Clone, FrameRootTrace)]
enum SuspendedConfirmQuietSelectors {
    All,
    Exact(Arc<[MessageSelector]>),
}

#[derive(Clone, Copy)]
enum ConfirmQuietSelectorsView<'a> {
    All,
    Exact(&'a [MessageSelector]),
}

impl ConfirmQuietSelectors {
    fn view(&self) -> ConfirmQuietSelectorsView<'_> {
        match self {
            Self::All => ConfirmQuietSelectorsView::All,
            Self::Exact(selectors) => ConfirmQuietSelectorsView::Exact(selectors),
        }
    }
}

impl SuspendedConfirmQuietSelectors {
    fn freeze(selectors: ConfirmQuietSelectors) -> Self {
        match selectors {
            ConfirmQuietSelectors::All => Self::All,
            ConfirmQuietSelectors::Exact(selectors) => Self::Exact(Arc::from(selectors)),
        }
    }

    fn view(&self) -> ConfirmQuietSelectorsView<'_> {
        match self {
            Self::All => ConfirmQuietSelectorsView::All,
            Self::Exact(selectors) => ConfirmQuietSelectorsView::Exact(selectors),
        }
    }
}

mod builtins;

fn normalize_confirm_quiet_selectors(
    selector_expr: Option<&Expr>,
) -> Result<ConfirmQuietSelectors, MessageSelectorNormalizationError> {
    match selector_expr {
        None => Ok(ConfirmQuietSelectors::All),
        Some(expr) if is_system_symbol_name(expr, "All") => Ok(ConfirmQuietSelectors::All),
        Some(expr) => parse_message_selectors(expr)
            .map(ConfirmQuietSelectors::Exact)
            .ok_or(MessageSelectorNormalizationError::InvalidSelectorSpec),
    }
}

fn confirm_quiet_matched_message(
    messages: &[RuntimeMessage],
    selectors: ConfirmQuietSelectorsView<'_>,
) -> Option<RuntimeMessage> {
    messages
        .iter()
        .find(|message| match selectors {
            ConfirmQuietSelectorsView::All => true,
            ConfirmQuietSelectorsView::Exact(selectors) => {
                selectors.iter().any(|selector| selector.matches(message))
            }
        })
        .cloned()
}

fn suppress_confirm_quiet_messages(
    session: &mut Session,
    checkpoint: usize,
    selectors: ConfirmQuietSelectorsView<'_>,
) {
    match selectors {
        ConfirmQuietSelectorsView::All => session.messages.truncate(checkpoint),
        ConfirmQuietSelectorsView::Exact(selectors) => {
            session.suppress_messages_since(checkpoint, selectors);
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<SuspendedConfirmQuietSelectors>();
};

fn evaluate_optional_confirmation_info(
    session: &mut Session,
    info_expr: Option<Expr>,
) -> Result<Option<Expr>, EvalError> {
    match info_expr {
        None => Ok(None),
        Some(expr) if is_internal_confirmation_auto_info(&expr) => Ok(None),
        Some(expr) => session.evaluate(expr).map(Some),
    }
}

fn confirmation_tag_from_args(args: &[Expr], index: usize) -> Expr {
    args.get(index)
        .cloned()
        .unwrap_or_else(default_confirmation_tag_expr)
}

fn default_confirmation_tag_expr() -> Expr {
    system_expr("InternalConfirmTag")
}

fn fresh_internal_enclose_tag_expr(session: &mut Session) -> Expr {
    let seed = Symbol::system("InternalEncloseTag");
    Expr::Atom(Atom::Symbol(session.allocate_fresh_symbol(&seed)))
}

fn is_internal_confirmation_auto_info(expr: &Expr) -> bool {
    is_system_symbol_name(expr, "InternalConfirmAutomaticInfo")
}

fn build_confirm_failure(raw_expr: Expr, value: Expr, info: Option<Expr>) -> Expr {
    build_confirmation_failure(vec![
        confirmation_entry("MessageTemplate", Expr::string("`` encountered.")),
        confirmation_delayed_entry("MessageParameters", list_expr(vec![value])),
        confirmation_entry("ConfirmationType", Expr::string("Confirm")),
        confirmation_delayed_entry("Expression", raw_expr),
        confirmation_entry("Information", confirmation_info_expr(info)),
    ])
}

fn build_confirm_failure_from_failure(raw_expr: Expr, message: Expr, info: Option<Expr>) -> Expr {
    build_confirmation_failure(vec![
        confirmation_entry("MessageTemplate", message),
        confirmation_entry("MessageParameters", list_expr(Vec::new())),
        confirmation_entry("ConfirmationType", Expr::string("Confirm")),
        confirmation_delayed_entry("Expression", raw_expr),
        confirmation_entry("Information", confirmation_info_expr(info)),
    ])
}

fn build_confirm_by_failure(raw_expr: Expr, raw_function: Expr, info: Option<Expr>) -> Expr {
    build_confirmation_failure(vec![
        confirmation_entry(
            "MessageTemplate",
            Expr::string("``[``] did not return True."),
        ),
        confirmation_delayed_entry(
            "MessageParameters",
            list_expr(vec![raw_function.clone(), raw_expr.clone()]),
        ),
        confirmation_entry("ConfirmationType", Expr::string("ConfirmBy")),
        confirmation_delayed_entry("Expression", raw_expr),
        confirmation_entry("Information", confirmation_info_expr(info)),
        confirmation_entry("Function", raw_function),
    ])
}

fn build_confirm_match_failure(raw_expr: Expr, raw_pattern: Expr, info: Option<Expr>) -> Expr {
    build_confirmation_failure(vec![
        confirmation_entry("MessageTemplate", Expr::string("`` does not match ``.")),
        confirmation_delayed_entry(
            "MessageParameters",
            list_expr(vec![raw_expr.clone(), raw_pattern.clone()]),
        ),
        confirmation_entry("ConfirmationType", Expr::string("ConfirmMatch")),
        confirmation_delayed_entry("Expression", raw_expr),
        confirmation_entry("Information", confirmation_info_expr(info)),
        confirmation_entry("Pattern", raw_pattern),
    ])
}

fn build_confirm_quiet_failure(
    raw_expr: Expr,
    info: Option<Expr>,
    message: RuntimeMessage,
) -> Expr {
    let held_expr = hold_expr(raw_expr.clone());
    let held_message_name = hold_expr(runtime_message_name_expr(message));
    build_confirmation_failure(vec![
        confirmation_entry(
            "MessageTemplate",
            Expr::string("`` produced a message on evaluation."),
        ),
        confirmation_delayed_entry(
            "MessageParameters",
            list_expr(vec![hold_form_expr(raw_expr.clone())]),
        ),
        confirmation_entry("ConfirmationType", Expr::string("ConfirmQuiet")),
        confirmation_delayed_entry("Expression", held_expr.clone()),
        confirmation_entry("Information", confirmation_info_expr(info)),
        confirmation_entry("HeldExpression", held_expr),
        confirmation_entry("HeldMessage", held_message_name.clone()),
        confirmation_entry("HeldMessageName", held_message_name),
    ])
}

fn build_confirm_assert_failure(raw_test: Expr, info: Option<Expr>) -> Expr {
    build_confirmation_failure(vec![
        confirmation_entry("MessageTemplate", Expr::string("Assertion `` failed.")),
        confirmation_delayed_entry(
            "MessageParameters",
            list_expr(vec![hold_form_expr(raw_test.clone())]),
        ),
        confirmation_entry("ConfirmationType", Expr::string("ConfirmAssert")),
        confirmation_delayed_entry("Expression", raw_test.clone()),
        confirmation_entry("Information", confirmation_info_expr(info)),
        confirmation_delayed_entry("Test", raw_test.clone()),
        confirmation_entry("HeldTest", hold_expr(raw_test)),
    ])
}

fn build_confirmation_failure(fields: Vec<AssociationEntryForm>) -> Expr {
    Expr::call(
        system_expr("Failure"),
        vec![
            Expr::string("ConfirmationFailed"),
            render_association_entry_forms_preserving_heads(fields),
        ],
    )
}

fn confirmation_entry(key: &str, value: Expr) -> AssociationEntryForm {
    confirmation_entry_form(key, value, false)
}

fn confirmation_delayed_entry(key: &str, value: Expr) -> AssociationEntryForm {
    confirmation_entry_form(key, value, true)
}

fn confirmation_entry_form(key: &str, value: Expr, delayed: bool) -> AssociationEntryForm {
    AssociationEntryForm {
        key: Expr::string(key),
        value,
        delayed,
    }
}

fn confirmation_info_expr(info: Option<Expr>) -> Expr {
    info.unwrap_or_else(|| system_expr("Null"))
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr(heads::structure::LIST), items)
}

fn hold_expr(expr: Expr) -> Expr {
    Expr::call(system_expr("Hold"), vec![expr])
}

fn hold_form_expr(expr: Expr) -> Expr {
    Expr::call(system_expr("HoldForm"), vec![expr])
}

pub(super) fn build_failsafe_failure(_function: Expr, _test: Expr, args: Vec<Expr>) -> Expr {
    let held_args = Expr::call(system_expr("Hold"), args.clone());
    let held_message_parameters = Expr::call(
        system_expr("List"),
        vec![Expr::call(system_expr("HoldForm"), args.clone())],
    );

    Expr::call(
        system_expr("Failure"),
        vec![
            Expr::string("FailsafeFailed"),
            Expr::call(
                system_expr("Association"),
                vec![
                    Expr::call(
                        system_expr("Rule"),
                        vec![
                            Expr::string("MessageTemplate"),
                            Expr::string("The test failed on the arguments `1`."),
                        ],
                    ),
                    Expr::call(
                        system_expr("RuleDelayed"),
                        vec![Expr::string("MessageParameters"), held_message_parameters],
                    ),
                    Expr::call(
                        system_expr("Rule"),
                        vec![Expr::string("Arguments"), held_args],
                    ),
                ],
            ),
        ],
    )
}

pub(super) fn is_confirmation_failure_like(expr: &Expr) -> bool {
    is_failure_q_expr(expr) || is_system_symbol_name(expr, "$Canceled") || is_missing_expr(expr)
}

fn rewrite_untagged_confirms(expr: Expr, tag: &Expr) -> Expr {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(&head, "Enclose") => {
            Expr::call_shared(head, args)
        }
        Expr::Call { head, args, .. } => {
            let rewritten_head = rewrite_untagged_confirms(head.as_ref().clone(), tag);
            let rewritten_args = args
                .into_iter()
                .map(|arg| rewrite_untagged_confirms(arg, tag))
                .collect::<Vec<_>>();

            let name = expr_as_symbol(&rewritten_head).map(|symbol| symbol.name().to_string());
            match (name.as_deref(), rewritten_args.as_slice()) {
                (Some("Confirm"), [expr]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("Confirm"), [expr, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmBy"), [expr, function]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        function.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmBy"), [expr, function, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), function.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmMatch"), [expr, pattern]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        pattern.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmMatch"), [expr, pattern, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), pattern.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmQuiet"), [expr]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        system_expr("All"),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmQuiet"), [expr, selectors]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        selectors.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmQuiet"), [expr, selectors, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), selectors.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmAssert"), [test]) => Expr::call(
                    rewritten_head,
                    vec![
                        test.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmAssert"), [test, info]) => Expr::call(
                    rewritten_head,
                    vec![test.clone(), info.clone(), tag.clone()],
                ),
                _ => Expr::call(rewritten_head, rewritten_args),
            }
        }
        other => other,
    }
}
