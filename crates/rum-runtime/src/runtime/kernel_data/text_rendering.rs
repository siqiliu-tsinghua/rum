use super::super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToStringFormat {
    FullForm,
    InputForm,
    OutputLike,
    CanonicalHoldFormTransparent,
}

const PREC_LOWEST: u8 = 0;
const PREC_RULE: u8 = 10;
const PREC_STRING_EXPRESSION: u8 = 15;
const PREC_PLUS: u8 = 20;
const PREC_TIMES: u8 = 30;
const PREC_POWER: u8 = 40;
const PREC_CALL: u8 = 90;

/// Renders an expression in the kernel's current input-like text form.
pub fn render_input_like_expr(expr: &Expr) -> String {
    render_to_string_expr(expr, ToStringFormat::OutputLike)
}

/// Renders an expression using the requested current kernel text format.
pub fn render_to_string_expr(expr: &Expr, format: ToStringFormat) -> String {
    if format == ToStringFormat::CanonicalHoldFormTransparent {
        return transparent_hold_form_expr(expr)
            .map_or_else(|| expr.to_string(), |expr| expr.to_string());
    }

    if matches!(expr, Expr::Atom(_)) || is_flat_generic_call(expr) {
        return expr.to_string();
    }

    render_to_string_expr_with_precedence(expr, format, PREC_LOWEST)
}

fn is_flat_generic_call(expr: &Expr) -> bool {
    let Expr::Call { head, args, .. } = expr else {
        return false;
    };
    expr_as_symbol(head).is_some_and(|symbol| symbol.context() != SYSTEM_CONTEXT)
        && args.iter().all(|arg| matches!(arg, Expr::Atom(_)))
}

fn render_to_string_expr_with_precedence(
    expr: &Expr,
    format: ToStringFormat,
    parent_precedence: u8,
) -> String {
    if format == ToStringFormat::FullForm {
        return expr.to_string();
    }

    let mut rendered = String::new();
    let mut work = vec![RenderTask::Expr(expr.clone(), parent_precedence)];
    while let Some(task) = work.pop() {
        match task {
            RenderTask::Expr(expr, parent_precedence) => {
                schedule_render_expr(&mut work, expr, format, parent_precedence, &mut rendered);
            }
            RenderTask::Text(text) => rendered.push_str(text),
            RenderTask::OwnedText(text) => rendered.push_str(&text),
        }
    }
    rendered
}

enum RenderTask {
    Expr(Expr, u8),
    Text(&'static str),
    OwnedText(String),
}

fn schedule_render_expr(
    work: &mut Vec<RenderTask>,
    expr: Expr,
    format: ToStringFormat,
    parent_precedence: u8,
    rendered: &mut String,
) {
    let Expr::Call { head, args, .. } = &expr else {
        rendered.push_str(&expr.to_string());
        return;
    };

    if is_system_symbol_name(head, heads::messages::MESSAGE_NAME) && args.len() == 2 {
        work.push(RenderTask::OwnedText(expr.to_string()));
        return;
    }

    if let Some(entries) = association_entries_from_expr(&expr) {
        let wrap = PREC_CALL < parent_precedence;
        push_close(work, wrap);
        work.push(RenderTask::Text("|>"));
        for (index, (key, value)) in entries.into_iter().enumerate().rev() {
            work.push(RenderTask::Expr(value, PREC_RULE + 1));
            work.push(RenderTask::Text(" -> "));
            work.push(RenderTask::Expr(key, PREC_RULE + 1));
            if index > 0 {
                work.push(RenderTask::Text(", "));
            }
        }
        work.push(RenderTask::Text("<|"));
        push_open(work, wrap);
        return;
    }

    if is_system_symbol_name(head, heads::structure::LIST) {
        schedule_delimited(work, args, PREC_LOWEST, ", ", "{", "}", false);
        return;
    }

    if is_system_symbol_name(head, heads::functions::SLOT) && args.len() == 1 {
        if let Some(index) = expr_as_non_negative_usize(&args[0]) {
            work.push(RenderTask::OwnedText(if index == 1 {
                "#".to_string()
            } else {
                format!("#{index}")
            }));
            return;
        }
    }

    if is_system_symbol_name(head, heads::functions::SLOT_SEQUENCE) && args.len() == 1 {
        if let Some(index) = expr_as_non_negative_usize(&args[0]) {
            work.push(RenderTask::OwnedText(if index == 1 {
                "##".to_string()
            } else {
                format!("##{index}")
            }));
            return;
        }
    }

    if is_system_symbol_name(head, heads::functions::FUNCTION) && args.len() == 1 {
        schedule_unary_suffix(
            work,
            args[0].clone(),
            PREC_RULE,
            " &",
            PREC_RULE,
            parent_precedence,
        );
        return;
    }

    if format == ToStringFormat::OutputLike && transparent_hold_form_arg(&expr).is_some() {
        work.push(RenderTask::Expr(args[0].clone(), parent_precedence));
        return;
    }

    if (is_system_symbol_name(head, "HoldCompleteForm")
        || is_system_symbol_name(head, "Defer")
        || is_system_symbol_name(head, "Unevaluated"))
        && args.len() == 1
    {
        work.push(RenderTask::Expr(args[0].clone(), parent_precedence));
        return;
    }

    if is_system_symbol_name(head, "Part") && args.len() >= 2 {
        let wrap = PREC_CALL < parent_precedence;
        push_close(work, wrap);
        work.push(RenderTask::Text("]]"));
        push_joined_exprs(work, &args[1..], PREC_LOWEST, ", ");
        work.push(RenderTask::Text("[["));
        work.push(RenderTask::Expr(args[0].clone(), PREC_CALL));
        push_open(work, wrap);
        return;
    }

    if is_system_symbol_name(head, heads::arithmetic::PLUS) && !args.is_empty() {
        schedule_joined(work, args, PREC_PLUS, " + ", PREC_PLUS, parent_precedence);
        return;
    }

    if is_system_symbol_name(head, heads::arithmetic::TIMES) && !args.is_empty() {
        schedule_joined(work, args, PREC_TIMES, " * ", PREC_TIMES, parent_precedence);
        return;
    }

    if is_system_symbol_name(head, heads::arithmetic::POWER) && args.len() == 2 {
        schedule_binary(
            work,
            args[0].clone(),
            args[1].clone(),
            PREC_POWER,
            " ^ ",
            PREC_POWER,
            parent_precedence,
        );
        return;
    }

    let binary = if is_system_symbol_name(head, heads::rules::SET) && args.len() == 2 {
        Some((" = ", PREC_RULE + 1))
    } else if is_system_symbol_name(head, heads::rules::SET_DELAYED) && args.len() == 2 {
        Some((" := ", PREC_RULE + 1))
    } else if is_system_symbol_name(head, heads::rules::RULE) && args.len() == 2 {
        Some((" -> ", PREC_RULE + 1))
    } else if is_system_symbol_name(head, heads::rules::RULE_DELAYED) && args.len() == 2 {
        Some((" :> ", PREC_RULE + 1))
    } else if is_system_symbol_name(head, "Map") && args.len() == 2 {
        Some((" /@ ", PREC_RULE))
    } else if is_system_symbol_name(head, "Apply") && args.len() == 2 {
        Some((" @@ ", PREC_RULE))
    } else if is_system_symbol_name(head, "MapApply") && args.len() == 2 {
        Some((" @@@ ", PREC_RULE))
    } else if is_system_symbol_name(head, heads::rules::REPLACE_ALL) && args.len() == 2 {
        Some((" /. ", PREC_RULE))
    } else if is_system_symbol_name(head, heads::rules::REPLACE_REPEATED) && args.len() == 2 {
        Some((" //. ", PREC_RULE))
    } else if is_system_symbol_name(head, heads::patterns::CONDITION) && args.len() == 2 {
        Some((" /; ", PREC_RULE))
    } else {
        None
    };
    if let Some((separator, child_precedence)) = binary {
        schedule_binary(
            work,
            args[0].clone(),
            args[1].clone(),
            child_precedence,
            separator,
            PREC_RULE,
            parent_precedence,
        );
        return;
    }

    if is_system_symbol_name(head, heads::rules::UNSET) && args.len() == 1 {
        schedule_unary_suffix(
            work,
            args[0].clone(),
            PREC_RULE + 1,
            " =.",
            PREC_RULE,
            parent_precedence,
        );
        return;
    }

    if is_system_symbol_name(head, heads::patterns::ALTERNATIVES) && !args.is_empty() {
        schedule_joined(work, args, PREC_RULE, " | ", PREC_RULE, parent_precedence);
        return;
    }

    if is_system_symbol_name(head, "StringExpression") && !args.is_empty() {
        schedule_joined(
            work,
            args,
            PREC_STRING_EXPRESSION,
            " ~~ ",
            PREC_STRING_EXPRESSION,
            parent_precedence,
        );
        return;
    }

    if is_system_symbol_name(head, "Repeated") && args.len() == 1 {
        schedule_unary_suffix(
            work,
            args[0].clone(),
            PREC_CALL,
            "..",
            PREC_CALL,
            parent_precedence,
        );
        return;
    }

    if is_system_symbol_name(head, "RepeatedNull") && args.len() == 1 {
        schedule_unary_suffix(
            work,
            args[0].clone(),
            PREC_CALL,
            "...",
            PREC_CALL,
            parent_precedence,
        );
        return;
    }

    if is_system_symbol_name(head, heads::patterns::PATTERN_TEST) && args.len() == 2 {
        schedule_binary(
            work,
            args[0].clone(),
            args[1].clone(),
            PREC_CALL,
            "?",
            PREC_CALL,
            parent_precedence,
        );
        return;
    }

    if is_system_symbol_name(head, heads::patterns::PATTERN) && args.len() == 2 {
        if let Some((name, prefix, constraint)) = named_blank_pattern_parts(&expr) {
            if let Some(constraint) = constraint {
                work.push(RenderTask::Expr(constraint, PREC_CALL));
            }
            work.push(RenderTask::Text(prefix));
            work.push(RenderTask::Expr(name, PREC_CALL));
            return;
        }
        schedule_binary(
            work,
            args[0].clone(),
            args[1].clone(),
            PREC_RULE + 1,
            " : ",
            PREC_RULE,
            parent_precedence,
        );
        return;
    }

    if let Some((prefix, constraint)) = blank_pattern_parts(head, args) {
        if let Some(constraint) = constraint {
            work.push(RenderTask::Expr(constraint, PREC_CALL));
        }
        work.push(RenderTask::Text(prefix));
        return;
    }

    if is_system_symbol_name(head, heads::patterns::OPTIONAL) && !args.is_empty() {
        match args.as_slice() {
            [pattern] => schedule_unary_suffix(
                work,
                pattern.clone(),
                PREC_CALL,
                ".",
                PREC_RULE,
                parent_precedence,
            ),
            [pattern, default] => schedule_binary(
                work,
                pattern.clone(),
                default.clone(),
                PREC_RULE + 1,
                " : ",
                PREC_RULE,
                parent_precedence,
            ),
            _ => schedule_display_fallback(work, expr, PREC_RULE, parent_precedence),
        }
        return;
    }

    if is_system_symbol_name(head, heads::structure::SPAN) {
        schedule_span(work, expr.clone(), args, parent_precedence);
        return;
    }

    if is_system_symbol_name(head, "CompoundExpression") && !args.is_empty() {
        schedule_joined(work, args, PREC_RULE, "; ", PREC_RULE, parent_precedence);
        return;
    }

    work.push(RenderTask::Text("]"));
    push_joined_exprs(work, args, PREC_LOWEST, ", ");
    work.push(RenderTask::Text("["));
    work.push(RenderTask::Expr(head.as_ref().clone(), PREC_CALL));
}

fn transparent_hold_form_expr(expr: &Expr) -> Option<Expr> {
    enum Work {
        Visit(Expr),
        FinishCall(usize),
    }

    let mut work = vec![Work::Visit(expr.clone())];
    let mut completed = Vec::new();
    let mut changed = false;
    while let Some(task) = work.pop() {
        match task {
            Work::Visit(expr) => {
                if let Some(arg) = transparent_hold_form_arg(&expr) {
                    changed = true;
                    work.push(Work::Visit(arg.clone()));
                    continue;
                }
                match expr {
                    Expr::Atom(_) => completed.push(expr),
                    Expr::Call { head, args, .. } => {
                        work.push(Work::FinishCall(args.len()));
                        for arg in args.iter().rev() {
                            work.push(Work::Visit(arg.clone()));
                        }
                        work.push(Work::Visit(head.into_owned()));
                    }
                }
            }
            Work::FinishCall(arg_count) => {
                let first = completed
                    .len()
                    .checked_sub(arg_count + 1)
                    .expect("transparent HoldForm traversal remains balanced");
                let mut children = completed.split_off(first).into_iter();
                let head = children
                    .next()
                    .expect("transparent HoldForm call retains its head");
                completed.push(Expr::call(head, children.collect()));
            }
        }
    }

    if !changed {
        return None;
    }
    completed.pop()
}

fn transparent_hold_form_arg(expr: &Expr) -> Option<&Expr> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    (is_system_symbol_name(head, "HoldForm") && args.len() == 1).then(|| &args[0])
}

fn named_blank_pattern_parts(expr: &Expr) -> Option<(Expr, &'static str, Option<Expr>)> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, heads::patterns::PATTERN) || args.len() != 2 {
        return None;
    }

    let Expr::Call {
        head: blank_head,
        args: blank_args,
        ..
    } = &args[1]
    else {
        return None;
    };

    let prefix = if is_system_symbol_name(blank_head, heads::patterns::BLANK) {
        "_"
    } else if is_system_symbol_name(blank_head, heads::patterns::BLANK_SEQUENCE) {
        "__"
    } else if is_system_symbol_name(blank_head, heads::patterns::BLANK_NULL_SEQUENCE) {
        "___"
    } else {
        return None;
    };

    let constraint = match blank_args.as_slice() {
        [] => None,
        [constraint] => Some(constraint.clone()),
        _ => return None,
    };

    Some((args[0].clone(), prefix, constraint))
}

fn blank_pattern_parts(head: &Expr, args: &[Expr]) -> Option<(&'static str, Option<Expr>)> {
    let prefix = if is_system_symbol_name(head, heads::patterns::BLANK) {
        "_"
    } else if is_system_symbol_name(head, heads::patterns::BLANK_SEQUENCE) {
        "__"
    } else if is_system_symbol_name(head, heads::patterns::BLANK_NULL_SEQUENCE) {
        "___"
    } else {
        return None;
    };

    let constraint = match args {
        [] => None,
        [constraint] => Some(constraint.clone()),
        _ => return None,
    };

    Some((prefix, constraint))
}

fn schedule_binary(
    work: &mut Vec<RenderTask>,
    left: Expr,
    right: Expr,
    child_precedence: u8,
    separator: &'static str,
    own_precedence: u8,
    parent_precedence: u8,
) {
    let wrap = own_precedence < parent_precedence;
    push_close(work, wrap);
    work.push(RenderTask::Expr(right, child_precedence));
    work.push(RenderTask::Text(separator));
    work.push(RenderTask::Expr(left, child_precedence));
    push_open(work, wrap);
}

fn schedule_unary_suffix(
    work: &mut Vec<RenderTask>,
    expr: Expr,
    child_precedence: u8,
    suffix: &'static str,
    own_precedence: u8,
    parent_precedence: u8,
) {
    let wrap = own_precedence < parent_precedence;
    push_close(work, wrap);
    work.push(RenderTask::Text(suffix));
    work.push(RenderTask::Expr(expr, child_precedence));
    push_open(work, wrap);
}

fn schedule_joined(
    work: &mut Vec<RenderTask>,
    args: &[Expr],
    child_precedence: u8,
    separator: &'static str,
    own_precedence: u8,
    parent_precedence: u8,
) {
    let wrap = own_precedence < parent_precedence;
    push_close(work, wrap);
    push_joined_exprs(work, args, child_precedence, separator);
    push_open(work, wrap);
}

fn schedule_delimited(
    work: &mut Vec<RenderTask>,
    args: &[Expr],
    child_precedence: u8,
    separator: &'static str,
    open: &'static str,
    close: &'static str,
    wrap: bool,
) {
    push_close(work, wrap);
    work.push(RenderTask::Text(close));
    push_joined_exprs(work, args, child_precedence, separator);
    work.push(RenderTask::Text(open));
    push_open(work, wrap);
}

fn push_joined_exprs(
    work: &mut Vec<RenderTask>,
    args: &[Expr],
    child_precedence: u8,
    separator: &'static str,
) {
    for index in (0..args.len()).rev() {
        work.push(RenderTask::Expr(args[index].clone(), child_precedence));
        if index > 0 {
            work.push(RenderTask::Text(separator));
        }
    }
}

fn schedule_span(work: &mut Vec<RenderTask>, expr: Expr, args: &[Expr], parent_precedence: u8) {
    let wrap = PREC_RULE < parent_precedence;
    push_close(work, wrap);
    match args {
        [start] if is_system_symbol_name(start, heads::structure::ALL) => {
            work.push(RenderTask::Text(";;"));
        }
        [start] => {
            work.push(RenderTask::Text(" ;;"));
            work.push(RenderTask::Expr(start.clone(), PREC_RULE + 1));
        }
        [start, end] if is_system_symbol_name(start, heads::structure::ALL) => {
            work.push(RenderTask::Expr(end.clone(), PREC_RULE + 1));
            work.push(RenderTask::Text(";; "));
        }
        [start, end] if is_system_symbol_name(end, heads::structure::ALL) => {
            work.push(RenderTask::Text(" ;;"));
            work.push(RenderTask::Expr(start.clone(), PREC_RULE + 1));
        }
        [start, end] => {
            work.push(RenderTask::Expr(end.clone(), PREC_RULE + 1));
            work.push(RenderTask::Text(" ;; "));
            work.push(RenderTask::Expr(start.clone(), PREC_RULE + 1));
        }
        [start, end, step]
            if is_system_symbol_name(start, heads::structure::ALL)
                && is_system_symbol_name(end, heads::structure::ALL) =>
        {
            work.push(RenderTask::Expr(step.clone(), PREC_RULE + 1));
            work.push(RenderTask::Text(";; ;; "));
        }
        [start, end, step] => {
            work.push(RenderTask::Expr(step.clone(), PREC_RULE + 1));
            work.push(RenderTask::Text(" ;; "));
            work.push(RenderTask::Expr(end.clone(), PREC_RULE + 1));
            work.push(RenderTask::Text(" ;; "));
            work.push(RenderTask::Expr(start.clone(), PREC_RULE + 1));
        }
        _ => work.push(RenderTask::OwnedText(expr.to_string())),
    }
    push_open(work, wrap);
}

fn schedule_display_fallback(
    work: &mut Vec<RenderTask>,
    expr: Expr,
    own_precedence: u8,
    parent_precedence: u8,
) {
    let wrap = own_precedence < parent_precedence;
    push_close(work, wrap);
    work.push(RenderTask::OwnedText(expr.to_string()));
    push_open(work, wrap);
}

fn push_close(work: &mut Vec<RenderTask>, wrap: bool) {
    if wrap {
        work.push(RenderTask::Text(")"));
    }
}

fn push_open(work: &mut Vec<RenderTask>, wrap: bool) {
    if wrap {
        work.push(RenderTask::Text("("));
    }
}
