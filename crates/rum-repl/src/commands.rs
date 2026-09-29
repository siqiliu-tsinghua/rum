use rum_host::{HostCompletionRequest, HostEvalSuccess, HostSession, HostTyped};
use rum_syntax::{
    FormatterOptions, SyntaxExprKind, format_parsed_source, parse_format_command,
    parse_with_diagnostics,
};
use rum_terminal::{render_host_failure, render_host_inspect, render_host_message_catalog};

const DEFAULT_TRACE_LINE_LIMIT: usize = 40;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ReplCommand {
    Help,
    Quit,
    Parse {
        expr: Option<String>,
    },
    Format {
        expr: String,
        options: FormatterOptions,
    },
    Lower {
        expr: Option<String>,
    },
    Trace {
        expr: Option<String>,
        form: Option<String>,
        depth: Option<usize>,
        rules: bool,
        full: bool,
        form_was_rewritten: bool,
    },
    History {
        limit: Option<usize>,
    },
    Complete {
        prefix: Option<String>,
    },
    Inspect {
        name: Option<String>,
    },
    Messages {
        query: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CommandExecution {
    Exit,
    Output(String),
    Trace {
        source: String,
        full: bool,
        form_notice: Option<String>,
    },
}

pub(super) fn parse_repl_command(line: &str) -> Result<Option<ReplCommand>, String> {
    match line {
        ":quit" | ":q" => Ok(Some(ReplCommand::Quit)),
        ":help" => Ok(Some(ReplCommand::Help)),
        _ if line == ":parse" || line.starts_with(":parse ") => Ok(Some(ReplCommand::Parse {
            expr: parse_trailing_argument(line, ":parse"),
        })),
        _ if line == ":format" || line.starts_with(":format ") => {
            let input = line.trim_start_matches(":format").trim();
            let (expr, options) = parse_format_command(input, ":format")?;
            Ok(Some(ReplCommand::Format {
                expr: expr.to_string(),
                options,
            }))
        }
        _ if line == ":lower" || line.starts_with(":lower ") => Ok(Some(ReplCommand::Lower {
            expr: parse_trailing_argument(line, ":lower"),
        })),
        _ if line == ":trace" || line.starts_with(":trace ") => {
            let input = line.trim_start_matches(":trace").trim();
            let (expr, form, depth, rules, full, form_was_rewritten) = parse_trace_command(input)?;
            Ok(Some(ReplCommand::Trace {
                expr,
                form,
                depth,
                rules,
                full,
                form_was_rewritten,
            }))
        }
        _ if line == ":history" || line.starts_with(":history ") => {
            Ok(Some(ReplCommand::History {
                limit: parse_history_limit(line)?,
            }))
        }
        _ if line == ":complete" || line.starts_with(":complete ") => {
            Ok(Some(ReplCommand::Complete {
                prefix: parse_trailing_argument(line, ":complete"),
            }))
        }
        _ if line == ":inspect" || line.starts_with(":inspect ") => {
            Ok(Some(ReplCommand::Inspect {
                name: parse_trailing_argument(line, ":inspect"),
            }))
        }
        _ if line == ":messages" || line.starts_with(":messages ") => {
            Ok(Some(ReplCommand::Messages {
                query: parse_trailing_argument(line, ":messages"),
            }))
        }
        _ => Ok(None),
    }
}

pub(super) fn execute_repl_command(
    session: &mut HostSession,
    history: &[String],
    command: ReplCommand,
) -> CommandExecution {
    match command {
        ReplCommand::Quit => CommandExecution::Exit,
        ReplCommand::Help => CommandExecution::Output(help_text().to_string()),
        ReplCommand::Parse { expr: None } => {
            CommandExecution::Output("! :parse expects an expression".to_string())
        }
        ReplCommand::Parse { expr: Some(expr) } => {
            CommandExecution::Output(match parse_with_diagnostics(&expr) {
                Ok(parsed) => parsed.debug_dump(),
                Err(diagnostic) => diagnostic.render(),
            })
        }
        ReplCommand::Format { expr, options } => {
            CommandExecution::Output(match parse_with_diagnostics(&expr) {
                Ok(parsed) => format_parsed_source(&parsed, options),
                Err(diagnostic) => diagnostic.render(),
            })
        }
        ReplCommand::Lower { expr: None } => {
            CommandExecution::Output("! :lower expects an expression".to_string())
        }
        ReplCommand::Lower { expr: Some(expr) } => {
            CommandExecution::Output(match session.lower_dump(&expr) {
                Ok(rendered) => rendered,
                Err(failure) => render_host_failure(&failure),
            })
        }
        ReplCommand::Trace { expr: None, .. } => {
            CommandExecution::Output("! :trace expects an expression".to_string())
        }
        ReplCommand::Trace {
            expr: Some(expr),
            form,
            depth,
            rules,
            full,
            form_was_rewritten,
        } => CommandExecution::Trace {
            source: trace_source(&expr, form.as_deref(), depth, rules),
            full,
            form_notice: form.filter(|_| form_was_rewritten),
        },
        ReplCommand::History { limit } => CommandExecution::Output(render_history(history, limit)),
        ReplCommand::Complete { prefix: None } => {
            CommandExecution::Output("! :complete expects a prefix".to_string())
        }
        ReplCommand::Complete {
            prefix: Some(prefix),
        } => {
            let cursor_offset = prefix.len();
            let completion = session.complete(HostCompletionRequest::new(&prefix, cursor_offset));
            let rendered = if completion.matches.is_empty() {
                "(no completions)".to_string()
            } else {
                completion.matches.join("\n")
            };
            CommandExecution::Output(rendered)
        }
        ReplCommand::Inspect { name: None } => {
            CommandExecution::Output("! :inspect expects a symbol name".to_string())
        }
        ReplCommand::Inspect { name: Some(name) } => {
            let response = session.inspect_symbol(&name);
            let rendered = if response.found {
                render_host_inspect(&response)
            } else {
                format!("! unknown symbol `{name}`")
            };
            CommandExecution::Output(rendered)
        }
        ReplCommand::Messages { query } => {
            let rendered = match session.message_catalog(query.as_deref()) {
                None => {
                    let name = query.as_deref().unwrap_or("<invalid>");
                    format!("! invalid message selector `{name}`")
                }
                Some(entries) if entries.is_empty() => match query {
                    Some(name) => format!("! no messages for `{name}`"),
                    None => "(no message definitions)".to_string(),
                },
                Some(entries) => render_host_message_catalog(&entries),
            };
            CommandExecution::Output(rendered)
        }
    }
}

fn parse_trailing_argument(line: &str, command: &str) -> Option<String> {
    line.strip_prefix(command)
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_history_limit(line: &str) -> Result<Option<usize>, String> {
    let raw_limit = line
        .strip_prefix(":history")
        .map(str::trim)
        .filter(|raw| !raw.is_empty());

    match raw_limit {
        Some(raw) => raw
            .parse::<usize>()
            .map(Some)
            .map_err(|_| format!("invalid history count `{raw}`")),
        None => Ok(None),
    }
}

fn parse_trace_command(
    input: &str,
) -> Result<
    (
        Option<String>,
        Option<String>,
        Option<usize>,
        bool,
        bool,
        bool,
    ),
    String,
> {
    let mut remainder = input.trim();
    let mut form = None;
    let mut depth = None;
    let mut rules = false;
    let mut full = false;
    let mut form_was_rewritten = false;

    while let Some((flag, rest)) = split_first_token(remainder) {
        if flag == "--rules" {
            rules = true;
            remainder = rest;
            continue;
        }

        if flag == "--full" {
            full = true;
            remainder = rest;
            continue;
        }

        if let Some(raw_form) = flag.strip_prefix("--form=") {
            let (normalized, rewritten) = normalize_trace_form(raw_form)?;
            form = Some(normalized);
            form_was_rewritten = rewritten;
            remainder = rest;
            continue;
        }

        if flag == "--form" {
            let Some((raw_form, next)) = split_first_token(rest) else {
                return Err("--form expects a pattern".to_string());
            };
            if raw_form.starts_with("--") {
                return Err("--form expects a pattern".to_string());
            }
            let (normalized, rewritten) = normalize_trace_form(raw_form)?;
            form = Some(normalized);
            form_was_rewritten = rewritten;
            remainder = next;
            continue;
        }

        if let Some(raw_depth) = flag.strip_prefix("--depth=") {
            depth = Some(parse_trace_depth(raw_depth)?);
            remainder = rest;
            continue;
        }

        if flag == "--depth" {
            let Some((raw_depth, next)) = split_first_token(rest) else {
                return Err("--depth expects a non-negative integer".to_string());
            };
            depth = Some(parse_trace_depth(raw_depth)?);
            remainder = next;
            continue;
        }

        if flag.starts_with("--") {
            return Err(format!("unknown flag `{flag}`"));
        }

        break;
    }

    let expr = (!remainder.is_empty()).then(|| remainder.to_string());
    Ok((expr, form, depth, rules, full, form_was_rewritten))
}

fn normalize_trace_form(raw: &str) -> Result<(String, bool), String> {
    if raw.is_empty() {
        return Err("--form expects a pattern".to_string());
    }

    let is_bare_symbol = parse_with_diagnostics(raw).ok().is_some_and(|parsed| {
        matches!(parsed.syntax().kind, SyntaxExprKind::Symbol(_))
            && parsed.syntax().span.start == 0
            && parsed.syntax().span.end == raw.len()
    });

    if is_bare_symbol {
        Ok((format!("_{raw}"), true))
    } else {
        Ok((raw.to_string(), false))
    }
}

fn parse_trace_depth(raw: &str) -> Result<usize, String> {
    raw.parse::<usize>()
        .map_err(|_| "--depth expects a non-negative integer".to_string())
}

fn split_first_token(input: &str) -> Option<(&str, &str)> {
    let trimmed = input.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    match trimmed.find(char::is_whitespace) {
        Some(index) => Some((&trimmed[..index], trimmed[index..].trim_start())),
        None => Some((trimmed, "")),
    }
}

pub(super) fn render_trace_response(
    response: &HostEvalSuccess,
    full: bool,
    form_notice: Option<&str>,
) -> String {
    let trace = render_trace_tree(&response.value.typed, full);
    match form_notice {
        Some(form) if trace.is_empty() => format!("filtering on {form}"),
        Some(form) => format!("filtering on {form}\n{trace}"),
        None => trace,
    }
}

fn trace_source(expr: &str, form: Option<&str>, depth: Option<usize>, rules: bool) -> String {
    let head = if rules { "TraceDetail" } else { "Trace" };
    match (form, depth) {
        (None, None) => format!("{head}[{expr}]"),
        (Some(form), None) => format!("{head}[{expr}, {form}]"),
        (form, Some(depth)) => format!("{head}[{expr}, {}, {depth}]", form.unwrap_or("_")),
    }
}

fn render_trace_tree(value: &HostTyped, full: bool) -> String {
    let mut lines = Vec::new();
    match value {
        HostTyped::List(items) => collect_trace_lines(items, 0, &mut lines),
        other => lines.push(render_trace_leaf(other, 0)),
    }

    if !full && lines.len() > DEFAULT_TRACE_LINE_LIMIT {
        let omitted = lines.len() - DEFAULT_TRACE_LINE_LIMIT;
        lines.truncate(DEFAULT_TRACE_LINE_LIMIT);
        lines.push(format!("… {omitted} more steps (:trace --full to see all)"));
    }

    lines.join("\n")
}

fn collect_trace_lines(items: &[HostTyped], depth: usize, lines: &mut Vec<String>) {
    for item in items {
        match item {
            HostTyped::List(nested) => collect_trace_lines(nested, depth + 1, lines),
            leaf => lines.push(render_trace_leaf(leaf, depth)),
        }
    }
}

fn render_trace_leaf(value: &HostTyped, depth: usize) -> String {
    let text = match value {
        HostTyped::Association(entries) => render_trace_step(entries),
        HostTyped::Other { head, text } if head == "Hold" => peel_trace_hold(text),
        _ => render_typed_text(value),
    };
    format!("{}{text}", "  ".repeat(depth))
}

fn render_trace_step(entries: &[(HostTyped, HostTyped)]) -> String {
    match trace_text_field(entries, "Step").as_deref() {
        Some("RuleApplied") => render_rule_applied_step(entries),
        Some("EvaluationPass") => render_evaluation_pass_step(entries),
        Some("DefinitionStored") => render_definition_stored_step(entries),
        Some(step) => render_unknown_trace_step(Some(step), entries),
        None => render_unknown_trace_step(None, entries),
    }
}

fn render_rule_applied_step(entries: &[(HostTyped, HostTyped)]) -> String {
    let mut line = render_trace_transition(
        "RuleApplied",
        "Input",
        trace_expression_field(entries, "Input"),
        "Result",
        trace_expression_field(entries, "Result"),
    );
    let identity = [
        trace_text_field(entries, "Source"),
        trace_expression_field(entries, "RuleLeft"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");

    if !identity.is_empty() {
        line.push_str(&format!("   [{identity}]"));
    }
    line
}

fn render_evaluation_pass_step(entries: &[(HostTyped, HostTyped)]) -> String {
    render_trace_transition(
        "EvaluationPass",
        "Before",
        trace_expression_field(entries, "Before"),
        "After",
        trace_expression_field(entries, "After"),
    )
}

fn render_definition_stored_step(entries: &[(HostTyped, HostTyped)]) -> String {
    let symbol = trace_text_field(entries, "Symbol");
    let value = trace_expression_field(entries, "Value");
    let mut line = match (symbol, value) {
        (Some(symbol), Some(value)) => format!("{symbol} = {value}"),
        (Some(symbol), None) => format!("Symbol: {symbol}"),
        (None, Some(value)) => format!("Value: {value}"),
        (None, None) => "DefinitionStored".to_string(),
    };

    if let Some(kind) = trace_text_field(entries, "Kind") {
        line.push_str(&format!("   [{kind}]"));
    }
    line
}

fn render_trace_transition(
    step: &str,
    left_name: &str,
    left: Option<String>,
    right_name: &str,
    right: Option<String>,
) -> String {
    match (left, right) {
        (Some(left), Some(right)) => format!("{left} → {right}"),
        (Some(left), None) => format!("{left_name}: {left}"),
        (None, Some(right)) => format!("{right_name}: {right}"),
        (None, None) => step.to_string(),
    }
}

fn render_unknown_trace_step(step: Option<&str>, entries: &[(HostTyped, HostTyped)]) -> String {
    let fields = entries
        .iter()
        .filter(|(key, _)| trace_key(key) != Some("Step"))
        .map(|(key, value)| {
            format!(
                "{}: {}",
                render_trace_metadata(key),
                render_trace_expression(value)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let label = step.unwrap_or("TraceStep");

    if fields.is_empty() {
        label.to_string()
    } else {
        format!("{label}: {fields}")
    }
}

fn trace_expression_field(entries: &[(HostTyped, HostTyped)], name: &str) -> Option<String> {
    trace_field(entries, name).map(render_trace_expression)
}

fn trace_text_field(entries: &[(HostTyped, HostTyped)], name: &str) -> Option<String> {
    trace_field(entries, name).map(render_trace_metadata)
}

fn trace_field<'a>(entries: &'a [(HostTyped, HostTyped)], name: &str) -> Option<&'a HostTyped> {
    entries
        .iter()
        .find_map(|(key, value)| (trace_key(key) == Some(name)).then_some(value))
}

fn trace_key(value: &HostTyped) -> Option<&str> {
    match value {
        HostTyped::Str(value) | HostTyped::Symbol(value) => Some(value),
        _ => None,
    }
}

fn render_trace_expression(value: &HostTyped) -> String {
    match value {
        HostTyped::Other { head, text } if head == "Hold" => peel_trace_hold(text),
        _ => render_typed_text(value),
    }
}

fn render_trace_metadata(value: &HostTyped) -> String {
    match value {
        HostTyped::Str(value) | HostTyped::Symbol(value) => value.clone(),
        _ => render_typed_text(value),
    }
}

fn peel_trace_hold(text: &str) -> String {
    text.strip_prefix("Hold[")
        .and_then(|contents| contents.strip_suffix(']'))
        .unwrap_or(text)
        .to_string()
}

fn render_typed_text(value: &HostTyped) -> String {
    match value {
        HostTyped::Integer(value) => value.to_string(),
        HostTyped::BigInteger(value) | HostTyped::Symbol(value) => value.clone(),
        HostTyped::Rational {
            numerator,
            denominator,
        } => format!("Rational[{numerator}, {denominator}]"),
        HostTyped::Bool(true) => "True".to_string(),
        HostTyped::Bool(false) => "False".to_string(),
        HostTyped::Str(value) => format!("{value:?}"),
        HostTyped::List(items) => format!(
            "List[{}]",
            items
                .iter()
                .map(render_typed_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        HostTyped::Association(entries) => format!(
            "Association[{}]",
            entries
                .iter()
                .map(|(key, value)| format!(
                    "Rule[{}, {}]",
                    render_typed_text(key),
                    render_typed_text(value)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        HostTyped::Rule { lhs, rhs } => {
            format!(
                "Rule[{}, {}]",
                render_typed_text(lhs),
                render_typed_text(rhs)
            )
        }
        HostTyped::Other { text, .. } => text.clone(),
    }
}

fn render_history(history: &[String], limit: Option<usize>) -> String {
    if history.is_empty() {
        return "(no history)".to_string();
    }

    let limit = limit.unwrap_or(history.len());
    let start = history.len().saturating_sub(limit);

    history[start..]
        .iter()
        .enumerate()
        .map(|(index, entry)| format!("{:>4}: {}", start + index + 1, entry.replace('\n', "\\n")))
        .collect::<Vec<_>>()
        .join("\n")
}

fn help_text() -> &'static str {
    ":help           Show REPL commands.\n:parse EXPR     Parse and print the syntax tree.\n:format [--line-width N] EXPR  Format syntax into the current canonical style.\n:lower EXPR     Lower syntax into core Expr form.\n:trace [--rules] [--form=PAT] [--depth=N] [--full] EXPR  Trace evaluation as an indented tree.\n:complete PREF  Show symbol and context completions.\n:history [N]    Show recent input history.\n:inspect NAME   Show basic symbol metadata.\n:messages [NAME]   Show message definitions globally or for a symbol/message.\n:quit           Exit the REPL."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_history_reports_invalid_counts() {
        let error = parse_repl_command(":history nope")
            .expect_err("invalid history counts should surface a parse error");
        assert_eq!(error, "invalid history count `nope`");
    }

    #[test]
    fn parse_format_returns_a_structured_command() {
        let command = parse_repl_command(":format f[x] --line-width=60")
            .expect("format command should parse")
            .expect("format command should be recognized");

        match command {
            ReplCommand::Format { expr, options } => {
                assert_eq!(expr, "f[x]");
                assert_eq!(options.line_width, 60);
            }
            other => panic!("expected format command, got {other:?}"),
        }
    }

    #[test]
    fn execute_history_renders_numbered_entries() {
        let history = vec!["1 + 2".to_string(), "f[\n1\n]".to_string()];
        let rendered = match execute_repl_command(
            &mut HostSession::new(),
            &history,
            ReplCommand::History { limit: None },
        ) {
            CommandExecution::Output(rendered) => rendered,
            other => panic!("expected command output, got {other:?}"),
        };

        assert!(rendered.contains("   1: 1 + 2"));
        assert!(rendered.contains(r"   2: f[\n1\n]"));
    }

    #[test]
    fn trace_step_renderer_degrades_for_missing_and_unknown_fields() {
        let missing_result = HostTyped::Association(vec![
            (
                HostTyped::Str("Step".to_string()),
                HostTyped::Str("RuleApplied".to_string()),
            ),
            (
                HostTyped::Str("Input".to_string()),
                HostTyped::Other {
                    head: "Hold".to_string(),
                    text: "Hold[Hold[x]]".to_string(),
                },
            ),
        ]);
        assert_eq!(render_trace_leaf(&missing_result, 1), "  Input: Hold[x]");

        let future_step = HostTyped::Association(vec![
            (
                HostTyped::Str("Step".to_string()),
                HostTyped::Str("FutureStep".to_string()),
            ),
            (
                HostTyped::Str("Payload".to_string()),
                HostTyped::Other {
                    head: "Hold".to_string(),
                    text: "Hold[Plus[1, 1]]".to_string(),
                },
            ),
            (
                HostTyped::Str("Detail".to_string()),
                HostTyped::Str("kept".to_string()),
            ),
        ]);
        assert_eq!(
            render_trace_leaf(&future_step, 0),
            "FutureStep: Payload: Plus[1, 1], Detail: \"kept\""
        );
    }
}
