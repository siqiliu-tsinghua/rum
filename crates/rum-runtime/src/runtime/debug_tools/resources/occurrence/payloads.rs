use super::*;
use crate::runtime::scoping::packages::SourceUnit;

pub(in crate::runtime) fn definition(value: &SymbolDefinition) -> Bytes {
    Bytes::Exact(size_of::<SymbolDefinition>())
        .add(optional_expr(value.own_value.as_ref()))
        .add(optional_expr(value.delayed_own_value.as_ref()))
        .add(sum(value.own_value_guards().iter().map(expression)))
        .add(sum(value.up_values.iter().map(|rule| {
            rule_payload(size_of::<UpValue>(), &rule.lhs, &rule.rhs, &rule.guards)
        })))
        .add(sum(value.down_values.iter().map(|rule| {
            rule_payload(size_of::<DownValue>(), &rule.lhs, &rule.rhs, &rule.guards)
        })))
        .add(sum(value.sub_values.iter().map(|rule| {
            rule_payload(size_of::<SubValue>(), &rule.lhs, &rule.rhs, &rule.guards)
        })))
        .add(value.occurrence_dispatch_bytes())
}

fn rule_payload(inline: usize, lhs: &Expr, rhs: &Expr, guards: &[Expr]) -> Bytes {
    // Inline Expr slots are deliberately charged again. Matcher payloads and
    // unused rule Vec capacity are historical omissions, not zero-sized data.
    Bytes::Exact(inline)
        .add(expression(lhs))
        .add(expression(rhs))
        .add(sum(guards.iter().map(expression)))
}

pub(super) fn optional_expr(value: Option<&Expr>) -> Bytes {
    value.map_or(Bytes::Exact(0), expression)
}

pub(super) fn options(value: &SymbolOptions) -> Bytes {
    Bytes::Exact(size_of::<SymbolOptions>())
        .add(slots::<SymbolOptionEntry>(value.capacity()))
        .add(sum(value.iter().map(|(name, value, _)| {
            Bytes::Exact(name.to_string().len()).add(expression(value))
        })))
}

pub(super) fn option_scope(scope: &OptionValueScope) -> Bytes {
    Bytes::Exact(size_of::<OptionValueScope>())
        .add(Bytes::Exact(scope.target().to_string().len()))
        .add(options(scope.explicit()))
}

pub(super) fn context_frame(frame: &ContextFrame) -> Bytes {
    Bytes::Exact(size_of::<ContextFrame>())
        .add(Bytes::Exact(frame.previous_context.capacity()))
        .add(strings(&frame.previous_context_path))
        .add(Bytes::Exact(frame.opened_context.capacity()))
}

pub(super) fn evaluation_entry(entry: &EvaluationStackEntry) -> Bytes {
    Bytes::Exact(size_of::<EvaluationStackEntry>()).add(expression(&entry.expr))
}

pub(super) fn dynamic_frame(frame: &DynamicOwnValueFrame) -> Bytes {
    Bytes::Exact(size_of::<DynamicOwnValueFrame>())
        .add(symbol_text(&frame.symbol))
        .add(optional_expr(frame.value.as_ref()))
}

pub(super) fn activation(value: &ModuleActivation) -> Bytes {
    Bytes::Exact(size_of::<ModuleActivation>()).add(sequence(&value.gensyms, symbol))
}

pub(super) fn source_unit(unit: &SourceUnit) -> Bytes {
    Bytes::Exact(size_of::<SourceUnit>())
        .add(Bytes::Exact(unit.package_context.capacity()))
        .add(expression(&unit.held_wrapper))
        .add(sequence(&unit.public, symbol))
        .add(sequence(&unit.package_scope, symbol))
        .add(sum(unit.private_index.iter().map(|(name, value)| {
            Bytes::Exact(size_of::<(String, Symbol)>())
                .add(Bytes::Exact(name.capacity()))
                .add(symbol_text(value))
        })))
}

pub(super) fn selector(value: &MessageSelector) -> Bytes {
    Bytes::Exact(size_of::<MessageSelector>())
        .add(Bytes::Exact(value.symbol.capacity()))
        .add(Bytes::Exact(value.tag.as_ref().map_or(0, String::capacity)))
}

pub(super) fn message(value: &RuntimeMessage) -> Bytes {
    Bytes::Exact(size_of::<RuntimeMessage>())
        .add(Bytes::Exact(value.symbol.capacity()))
        .add(Bytes::Exact(value.tag.capacity()))
        .add(Bytes::Exact(value.text.capacity()))
}

pub(super) fn trace(value: &TraceEvent) -> Bytes {
    let payload = match value {
        TraceEvent::EvaluationPass { before, after } => expression(before).add(expression(after)),
        TraceEvent::DefinitionStored { symbol, value, .. } => {
            Bytes::Exact(symbol.capacity()).add(expression(value))
        }
        TraceEvent::RuleApplied {
            input,
            rule,
            result,
            ..
        } => expression(input)
            .add(expression(rule))
            .add(expression(result)),
    };
    Bytes::Exact(size_of::<TraceEvent>()).add(payload)
}

pub(super) fn reap(scope: &ReapScope) -> Bytes {
    Bytes::Exact(size_of::<ReapScope>())
        .add(sum(scope.patterns.iter().map(expression)))
        .add(optional_expr(scope.handler.as_ref()))
        .add(sum(scope.buckets_by_pattern.iter().flat_map(|group| {
            group.iter().map(|bucket| {
                expression(&bucket.tag).add(sum(bucket.values.iter().map(expression)))
            })
        })))
}

pub(super) fn monitor(scope: &MonitorScope) -> Bytes {
    Bytes::Exact(size_of::<MonitorScope>())
        .add(expression(&scope.display_expr))
        .add(optional_expr(scope.evaluation_monitor.as_ref()))
        .add(optional_expr(scope.step_monitor.as_ref()))
        .add(Bytes::Exact(
            scope.last_display.as_ref().map_or(0, String::capacity),
        ))
}
