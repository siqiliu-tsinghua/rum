use rum_core::SharedArgs;
use rum_syntax::{LexedSource, Span, SyntaxError, SyntaxTokenKind, TriviaKind, lex_with_trivia};

use super::patterns::{
    StringPattern, StringPatternMatchConfig, parse_string_pattern, string_pattern_match_end,
};
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct TokenizeRule {
    class: String,
    pattern: StringPattern,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TokenSpan {
    class: String,
    start: usize,
    end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ByteTokenSpan {
    kind: String,
    start: usize,
    end: usize,
}

struct ByteToCharOffsetCursor<'a> {
    source: &'a str,
    byte_position: usize,
    char_position: usize,
}

impl<'a> ByteToCharOffsetCursor<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            byte_position: 0,
            char_position: 0,
        }
    }

    fn advance_to(&mut self, byte_offset: usize) -> usize {
        debug_assert!(self.byte_position <= byte_offset);
        self.char_position += self.source[self.byte_position..byte_offset].chars().count();
        self.byte_position = byte_offset;
        self.char_position
    }
}

pub(crate) fn eval_tokenize_with(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [rules_expr, source_expr] = args else {
        return Ok(fail_tokenizer(
            session,
            "TokenizeWith",
            "TokenizeWith expects a rule list and a string.",
        ));
    };

    let rules = match parse_tokenize_rules(rules_expr) {
        Ok(rules) => rules,
        Err(message) => return Ok(fail_tokenizer(session, "TokenizeWith", message)),
    };

    let Some(source) = expr_as_string(source_expr) else {
        return Ok(fail_tokenizer(
            session,
            "TokenizeWith",
            "TokenizeWith expects its second argument to be a string.",
        ));
    };

    Ok(tokenize_with_expr(&rules, source))
}

pub(crate) fn eval_rum_tokens(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [source_expr] = args else {
        return Ok(fail_tokenizer(
            session,
            "RumTokens",
            "RumTokens expects exactly one string argument.",
        ));
    };

    let Some(source) = expr_as_string(source_expr) else {
        return Ok(fail_tokenizer(
            session,
            "RumTokens",
            "RumTokens expects a string argument.",
        ));
    };

    Ok(rum_tokens_expr(source))
}

fn parse_tokenize_rules(expr: &Expr) -> Result<Vec<TokenizeRule>, &'static str> {
    let Expr::Call { head, args, .. } = expr else {
        return Err("TokenizeWith expects its first argument to be a List of string-class rules.");
    };
    if !is_system_symbol_name(head, "List") {
        return Err("TokenizeWith expects its first argument to be a List of string-class rules.");
    }

    args.iter()
        .map(parse_tokenize_rule)
        .collect::<Result<Vec<_>, _>>()
}

fn parse_tokenize_rule(expr: &Expr) -> Result<TokenizeRule, &'static str> {
    let Expr::Call { head, args, .. } = expr else {
        return Err("TokenizeWith rules must have the form \"class\" -> pattern.");
    };
    if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
        return Err("TokenizeWith rules must have the form \"class\" -> pattern.");
    }

    let Some(class) = expr_as_string(&args[0]) else {
        return Err("TokenizeWith rule classes must be strings.");
    };
    let Some(pattern) = parse_string_pattern(&args[1]) else {
        return Err("TokenizeWith rule patterns must use the documented string-pattern language.");
    };

    Ok(TokenizeRule {
        class: class.to_string(),
        pattern,
    })
}

fn tokenize_with_expr(rules: &[TokenizeRule], source: &str) -> Expr {
    list_expr(
        tokenize_with_spans(rules, source)
            .into_iter()
            .map(token_span_expr)
            .collect(),
    )
}

fn tokenize_with_spans(rules: &[TokenizeRule], source: &str) -> Vec<TokenSpan> {
    let chars = source.chars().collect::<Vec<_>>();
    let mut spans = Vec::new();
    let mut text_start = None;
    let mut index = 0usize;

    while index < chars.len() {
        if let Some((rule, end)) = best_tokenize_rule_at(rules, &chars, index) {
            if let Some(start) = text_start.take() {
                push_token_span(&mut spans, "text", start, index);
            }
            push_token_span(&mut spans, &rule.class, index, end);
            index = end;
            continue;
        }

        text_start.get_or_insert(index);
        index += 1;
    }

    if let Some(start) = text_start {
        push_token_span(&mut spans, "text", start, chars.len());
    }

    spans
}

fn best_tokenize_rule_at<'a>(
    rules: &'a [TokenizeRule],
    chars: &[char],
    start: usize,
) -> Option<(&'a TokenizeRule, usize)> {
    rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| {
            string_pattern_match_end(
                &rule.pattern,
                chars,
                start,
                StringPatternMatchConfig::default(),
            )
            .filter(|end| *end > start)
            .map(|end| (index, rule, end))
        })
        .max_by_key(|(index, _, end)| (*end, usize::MAX - *index))
        .map(|(_, rule, end)| (rule, end))
}

fn push_token_span(spans: &mut Vec<TokenSpan>, class: &str, start: usize, end: usize) {
    if start < end {
        spans.push(TokenSpan {
            class: class.to_string(),
            start,
            end,
        });
    }
}

fn token_span_expr(span: TokenSpan) -> Expr {
    list_expr(vec![
        Expr::string(span.class),
        inclusive_span_expr(span.start, span.end),
    ])
}

fn rum_tokens_expr(source: &str) -> Expr {
    let spans = rum_token_spans(source);
    let mut offset_cursor = ByteToCharOffsetCursor::new(source);

    list_expr(
        spans
            .into_iter()
            .map(|span| {
                debug_assert!(
                    offset_cursor.byte_position <= span.start && span.start <= span.end,
                    "RumTokens spans must be ordered and non-overlapping"
                );
                byte_token_span_expr(&mut offset_cursor, span)
            })
            .collect(),
    )
}

fn rum_token_spans(source: &str) -> Vec<ByteTokenSpan> {
    let mut spans = Vec::new();
    let mut base = 0usize;

    while base < source.len() {
        match lex_with_trivia(&source[base..]) {
            Ok(lexed) => {
                push_lexed_spans(&mut spans, base, &lexed);
                break;
            }
            Err(error) => {
                let error_span = error
                    .span()
                    .unwrap_or_else(|| Span::new(0, source.len() - base));
                let prefix_end = error_span.start.min(source.len() - base);

                if prefix_end > 0 {
                    if let Ok(prefix) = lex_with_trivia(&source[base..base + prefix_end]) {
                        push_lexed_spans(&mut spans, base, &prefix);
                    }
                    base += prefix_end;
                    continue;
                }

                let error_end = nonempty_error_end(&source[base..], &error, error_span);
                push_byte_span(&mut spans, "error", base, base + error_end);
                base += error_end;
            }
        }
    }

    spans
}

fn push_lexed_spans(spans: &mut Vec<ByteTokenSpan>, base: usize, lexed: &LexedSource) {
    for (index, token) in lexed.tokens().iter().enumerate() {
        if let Some(leading_trivia) = lexed.leading_trivia(index) {
            for trivia in leading_trivia {
                push_byte_span(
                    spans,
                    trivia_kind_label(trivia.kind),
                    base + trivia.span.start,
                    base + trivia.span.end,
                );
            }
        }

        if matches!(token.kind, SyntaxTokenKind::End) {
            continue;
        }

        push_byte_span(
            spans,
            token.kind.label(),
            base + token.span.start,
            base + token.span.end,
        );
    }
}

fn push_byte_span(spans: &mut Vec<ByteTokenSpan>, kind: &str, start: usize, end: usize) {
    if start < end {
        spans.push(ByteTokenSpan {
            kind: kind.to_string(),
            start,
            end,
        });
    }
}

fn nonempty_error_end(source: &str, error: &SyntaxError, span: Span) -> usize {
    let end = span.end.min(source.len());
    if end > 0 {
        return end;
    }

    match error {
        SyntaxError::UnexpectedCharacter { found, .. } => found.len_utf8(),
        _ => source
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or_default(),
    }
}

fn byte_token_span_expr(
    offset_cursor: &mut ByteToCharOffsetCursor<'_>,
    span: ByteTokenSpan,
) -> Expr {
    list_expr(vec![
        Expr::string(span.kind),
        inclusive_span_expr(
            offset_cursor.advance_to(span.start),
            offset_cursor.advance_to(span.end),
        ),
    ])
}

fn inclusive_span_expr(start: usize, end: usize) -> Expr {
    list_expr(vec![
        Expr::integer(BigInt::from(start + 1)),
        Expr::integer(BigInt::from(end)),
    ])
}

fn trivia_kind_label(kind: TriviaKind) -> &'static str {
    match kind {
        TriviaKind::Whitespace => "whitespace",
        TriviaKind::Comment => "comment",
    }
}

fn fail_tokenizer(session: &mut Session, symbol: &str, message: impl Into<String>) -> Expr {
    session.push_message(symbol, "tok", message);
    system_expr("$Failed")
}
