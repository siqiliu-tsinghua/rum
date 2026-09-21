use std::fmt::{self, Write};

use crate::parser::parse_tokens;
use crate::{LexedSource, Span, SyntaxError, SyntaxExpr, lex_with_trivia};

const MULTILINE_CONTEXT_LIMIT: usize = 3;

#[derive(Clone, Debug)]
pub struct SourceText {
    source: String,
    line_starts: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceDiagnosticSpan {
    pub span: Span,
    pub start: SourceLocation,
    pub end: SourceLocation,
    pub label: Option<String>,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructuredSourceDiagnostic {
    pub message: String,
    pub rendered: String,
    pub is_incomplete: bool,
    pub spans: Vec<SourceDiagnosticSpan>,
}

#[derive(Clone, Debug)]
pub struct SourceDiagnostic {
    source: SourceText,
    error: SyntaxError,
}

#[derive(Clone, Debug)]
pub struct ParsedSource {
    source: SourceText,
    lexed: LexedSource,
    syntax: SyntaxExpr,
}

impl SourceText {
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into();
        let line_starts = collect_line_starts(&source);

        Self {
            source,
            line_starts,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn text(&self, span: Span) -> Option<&str> {
        self.source.get(span.start..span.end)
    }

    pub fn location(&self, offset: usize) -> Option<SourceLocation> {
        if offset > self.source.len() {
            return None;
        }

        let line_index = self.line_starts.partition_point(|start| *start <= offset);
        let line_index = line_index.saturating_sub(1);
        let line_start = *self.line_starts.get(line_index)?;

        Some(SourceLocation {
            line: line_index + 1,
            column: offset.saturating_sub(line_start) + 1,
        })
    }

    pub fn span_locations(&self, span: Span) -> Option<(SourceLocation, SourceLocation)> {
        Some((self.location(span.start)?, self.location(span.end)?))
    }

    pub fn render_syntax_error(&self, error: &SyntaxError) -> String {
        let mut output = String::new();
        writeln!(&mut output, "error: {error}").unwrap();

        let Some(span) = error.span() else {
            return output.trim_end().to_string();
        };

        let Some((start, end)) = self.span_locations(span) else {
            return output.trim_end().to_string();
        };

        writeln!(&mut output, " --> {}:{}", start.line, start.column).unwrap();
        writeln!(&mut output, "  |").unwrap();
        self.render_span_excerpt(&mut output, span, None).unwrap();

        if let Some(related_span) = error.related_span() {
            writeln!(&mut output, "  |").unwrap();
            self.render_span_excerpt(&mut output, related_span, error.related_label().as_deref())
                .unwrap();
        }

        if start.line != end.line {
            writeln!(
                &mut output,
                "  = note: span covers lines {} through {}",
                start.line, end.line
            )
            .unwrap();
        }

        output.trim_end().to_string()
    }

    fn render_span_excerpt(
        &self,
        output: &mut String,
        span: Span,
        label: Option<&str>,
    ) -> fmt::Result {
        let Some((start, end)) = self.span_locations(span) else {
            return Ok(());
        };

        if start.line == end.line {
            return self.render_single_line_span(output, start.line, span, label);
        }

        let last_line = end.line;
        let first_line = start.line;
        let middle_lines = last_line.saturating_sub(first_line + 1);
        let show_last_directly = middle_lines <= MULTILINE_CONTEXT_LIMIT.saturating_sub(2);

        self.render_single_line_span(
            output,
            first_line,
            Span::new(
                span.start,
                self.line_bounds(first_line)
                    .map_or(span.end, |(_, line_end)| line_end),
            ),
            label.or(Some("span begins here")),
        )?;

        if middle_lines > 0 {
            if show_last_directly {
                for line in (first_line + 1)..last_line {
                    self.render_continuation_line(output, line)?;
                }
            } else {
                writeln!(output, "   | ...")?;
            }
        }

        self.render_single_line_span(
            output,
            last_line,
            Span::new(
                self.line_bounds(last_line)
                    .map_or(span.start, |(line_start, _)| line_start),
                span.end,
            ),
            Some("span ends here"),
        )
    }

    fn render_single_line_span(
        &self,
        output: &mut String,
        line: usize,
        span: Span,
        label: Option<&str>,
    ) -> fmt::Result {
        let line_text = self.line_text(line).unwrap_or("");
        let gutter_width = line.to_string().len();
        let highlight = self.highlight_line_span(line, span);

        writeln!(
            output,
            "{:>width$} | {}",
            line,
            line_text,
            width = gutter_width
        )?;

        if let Some(label) = label {
            writeln!(
                output,
                "{:>width$} | {} {}",
                "",
                highlight,
                label,
                width = gutter_width
            )
        } else {
            writeln!(
                output,
                "{:>width$} | {}",
                "",
                highlight,
                width = gutter_width
            )
        }
    }

    fn render_continuation_line(&self, output: &mut String, line: usize) -> fmt::Result {
        let line_text = self.line_text(line).unwrap_or("");
        let gutter_width = line.to_string().len();

        writeln!(
            output,
            "{:>width$} | {}",
            line,
            line_text,
            width = gutter_width
        )?;
        writeln!(output, "{:>width$} | ^^^", "", width = gutter_width)
    }

    fn line_bounds(&self, line: usize) -> Option<(usize, usize)> {
        let line_index = line.checked_sub(1)?;
        let start = *self.line_starts.get(line_index)?;
        let end = self
            .line_starts
            .get(line_index + 1)
            .copied()
            .map(|next| next.saturating_sub(1))
            .unwrap_or(self.source.len());

        Some((start, end))
    }

    fn line_text(&self, line: usize) -> Option<&str> {
        let (start, end) = self.line_bounds(line)?;
        Some(self.source.get(start..end)?.trim_end_matches('\r'))
    }

    fn highlight_line_span(&self, line: usize, span: Span) -> String {
        let Some((line_start, line_end)) = self.line_bounds(line) else {
            return "^".to_string();
        };

        let line_text = self.source.get(line_start..line_end).unwrap_or("");
        let line_text = line_text.trim_end_matches('\r');
        let visible_line_end = line_start + line_text.len();

        let highlight_start = span.start.max(line_start).min(visible_line_end);
        let highlight_end = span.end.max(highlight_start).min(visible_line_end);
        let start_column = self.location(highlight_start).map_or(1, |loc| loc.column);
        let end_column = if highlight_end > highlight_start {
            self.location(highlight_end)
                .map_or(start_column + 1, |loc| loc.column)
        } else {
            start_column + 1
        };

        format!(
            "{}{}",
            " ".repeat(start_column.saturating_sub(1)),
            "^".repeat(end_column.saturating_sub(start_column).max(1))
        )
    }
}

impl SourceDiagnostic {
    pub fn new(source: SourceText, error: SyntaxError) -> Self {
        Self { source, error }
    }

    pub fn source(&self) -> &SourceText {
        &self.source
    }

    pub fn error(&self) -> &SyntaxError {
        &self.error
    }

    pub fn into_error(self) -> SyntaxError {
        self.error
    }

    pub fn render(&self) -> String {
        self.source.render_syntax_error(&self.error)
    }

    pub fn structured(&self) -> StructuredSourceDiagnostic {
        let mut spans = Vec::new();

        if let Some(span) = self.error.span() {
            if let Some(span) = self.source.diagnostic_span(span, None) {
                spans.push(span);
            }
        }

        if let Some(related_span) = self.error.related_span() {
            if let Some(span) = self
                .source
                .diagnostic_span(related_span, self.error.related_label().as_deref())
            {
                spans.push(span);
            }
        }

        StructuredSourceDiagnostic {
            message: self.error.to_string(),
            rendered: self.render(),
            is_incomplete: self.error.is_incomplete(),
            spans,
        }
    }
}

impl fmt::Display for SourceDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

pub fn parse_with_diagnostics(source: impl Into<String>) -> Result<ParsedSource, SourceDiagnostic> {
    let source = SourceText::new(source);
    let lexed = lex_with_trivia(source.source())
        .map_err(|error| SourceDiagnostic::new(source.clone(), error))?;
    let syntax = parse_tokens(lexed.tokens().to_vec())
        .map_err(|error| SourceDiagnostic::new(source.clone(), error))?;

    Ok(ParsedSource {
        source,
        lexed,
        syntax,
    })
}

impl ParsedSource {
    pub fn parse(source: impl Into<String>) -> Result<Self, SyntaxError> {
        parse_with_diagnostics(source).map_err(SourceDiagnostic::into_error)
    }

    pub fn source_text(&self) -> &SourceText {
        &self.source
    }

    pub fn source(&self) -> &str {
        self.source.source()
    }

    pub fn lexed(&self) -> &LexedSource {
        &self.lexed
    }

    pub fn syntax(&self) -> &SyntaxExpr {
        &self.syntax
    }

    pub fn text(&self, span: Span) -> Option<&str> {
        self.source.text(span)
    }

    pub fn location(&self, offset: usize) -> Option<SourceLocation> {
        self.source.location(offset)
    }

    pub fn span_locations(&self, span: Span) -> Option<(SourceLocation, SourceLocation)> {
        self.source.span_locations(span)
    }

    pub fn render_syntax_error(&self, error: &SyntaxError) -> String {
        self.source.render_syntax_error(error)
    }

    pub fn debug_dump(&self) -> String {
        let mut output = String::new();
        writeln!(&mut output, "syntax:").unwrap();
        writeln!(&mut output, "{}", syntax_dump::Dump(&self.syntax)).unwrap();
        writeln!(&mut output, "tokens:").unwrap();

        for (index, token) in self.lexed.tokens().iter().enumerate() {
            let text = self.text(token.span).map(debug_text).unwrap_or_default();
            let location = self
                .location(token.span.start)
                .map(|location| format!("{}:{}", location.line, location.column))
                .unwrap_or_else(|| "?".to_string());

            writeln!(
                &mut output,
                "  [{index}] {} @ {} ({location}) text=\"{}\"",
                token.kind.label(),
                token.span,
                text
            )
            .unwrap();

            if let Some(trivia) = self.lexed.leading_trivia(index) {
                for piece in trivia {
                    let text = debug_text(&piece.text);
                    writeln!(
                        &mut output,
                        "       trivia {} @ {} text=\"{}\"",
                        piece.kind.label(),
                        piece.span,
                        text
                    )
                    .unwrap();
                }
            }
        }

        output
    }
}

impl SourceText {
    fn diagnostic_span(&self, span: Span, label: Option<&str>) -> Option<SourceDiagnosticSpan> {
        let (start, end) = self.span_locations(span)?;

        Some(SourceDiagnosticSpan {
            span,
            start,
            end,
            label: label.map(str::to_owned),
            text: self.text(span).unwrap_or_default().to_string(),
        })
    }
}

fn collect_line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];

    for (index, ch) in source.char_indices() {
        if ch == '\n' {
            starts.push(index + ch.len_utf8());
        }
    }

    starts
}

fn debug_text(text: &str) -> String {
    text.chars()
        .flat_map(|ch| ch.escape_debug())
        .collect::<String>()
}

#[path = "syntax_dump.rs"]
mod syntax_dump;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsed_source_preserves_source_slices_locations_and_trivia() {
        let parsed = ParsedSource::parse("f[\n  (* note *) x\n]").unwrap();

        assert_eq!(
            parsed.text(parsed.syntax.span()),
            Some("f[\n  (* note *) x\n]")
        );
        assert_eq!(
            parsed.location(3),
            Some(SourceLocation { line: 2, column: 1 })
        );
        assert_eq!(
            parsed.span_locations(Span::new(0, 1)),
            Some((
                SourceLocation { line: 1, column: 1 },
                SourceLocation { line: 1, column: 2 },
            ))
        );

        let trivia = parsed.lexed.leading_trivia(2).unwrap();
        assert_eq!(trivia.len(), 3);
        assert_eq!(trivia[1].text, "(* note *)");
    }

    #[test]
    fn parsed_source_debug_dump_includes_syntax_tokens_and_trivia() {
        let parsed = ParsedSource::parse("f[(* note *) x]").unwrap();
        let dump = parsed.debug_dump();

        assert!(dump.contains("syntax:"));
        assert!(dump.contains("tokens:"));
        assert!(dump.contains("symbol @ 0..1"));
        assert!(dump.contains("trivia comment"));
        assert!(dump.contains("(* note *)"));
    }

    #[test]
    fn debug_dump_matches_alternate_debug_across_front_and_tail() {
        // The call-head edge takes two traversal steps. These depths straddle
        // the 16-step front and exercise its iterative continuation.
        for depth in [0, 1, 7, 8, 9, 16, 17, 32] {
            let parsed = ParsedSource::parse(format!("f{}", "[1]".repeat(depth))).unwrap();
            let expected = format!("syntax:\n{:#?}\ntokens:\n", parsed.syntax);
            assert!(parsed.debug_dump().starts_with(&expected), "depth {depth}");
        }
    }

    #[test]
    fn debug_dump_preserves_every_syntax_variant_and_leaf_layout() {
        let source = r#"(* dump layout *) { {}, <||>, {123, "a\n\t\"\\", café},
            <|"key" -> {x_, y__Integer, z___, _Real}, a :> b|>,
            f[][x_Integer, y:(_|_Real), #, #2, #name, ##, ##3],
            -x + y z/2^3, !p, ++x, x++, f'''[x],
            a < b <= c != d, (x; y;), (# + #2)& }"#;
        let mut syntax = ParsedSource::parse(source).unwrap().syntax;
        for depth in 0..=9 {
            assert_eq!(
                format!("{}", syntax_dump::Dump(&syntax)),
                format!("{syntax:#?}"),
                "shape fixture at head depth {depth}",
            );
            let span = syntax.span();
            syntax = SyntaxExpr::call(syntax, Vec::new(), span);
        }

        let syntax = SyntaxExpr::string("\0\r\t\n\\\"\u{7f}\u{85}\u{200d}é", Span::new(1, 99));
        assert_eq!(
            format!("{}", syntax_dump::Dump(&syntax)),
            format!("{syntax:#?}"),
        );
    }

    #[test]
    fn debug_dump_deep_head_chain_completes_on_small_stack() {
        let depth = 1_000;
        let parsed = ParsedSource::parse(format!("f{}", "[1]".repeat(depth))).unwrap();
        let (parsed, expressions, spans, has_tokens) = std::thread::Builder::new()
            .stack_size(512 * 1024)
            .spawn(move || {
                let dump = parsed.debug_dump();
                let expressions = dump.matches("SyntaxExpr {").count();
                let spans = dump.matches("span: Span {").count();
                let has_tokens = dump.contains("\ntokens:\n");
                // Drop the AST on the parent stack: its independent recursive
                // destruction is outside the dump emitter's contract.
                (parsed, expressions, spans, has_tokens)
            })
            .unwrap()
            .join()
            .unwrap();
        assert_eq!(expressions, 2 * depth + 1);
        assert_eq!(spans, expressions);
        assert!(has_tokens);
        drop(parsed);
    }

    #[test]
    fn source_diagnostic_renders_line_column_and_snippet() {
        let diagnostic = parse_with_diagnostics("f[1 ! 2]").unwrap_err();
        let rendered = diagnostic.render();

        assert!(rendered.contains("error: expected ,"));
        assert!(rendered.contains("--> 1:5"));
        assert!(rendered.contains("1 | f[1 ! 2]"));
        assert!(rendered.contains("^"));
    }

    #[test]
    fn source_diagnostic_renders_related_opening_span() {
        let diagnostic = parse_with_diagnostics("f[\n  1").unwrap_err();
        let rendered = diagnostic.render();

        assert!(rendered.contains("opening `[` for expected `]`"));
        assert!(rendered.contains("1 | f["));
        assert!(rendered.contains("2 |   1"));
    }

    #[test]
    fn source_diagnostic_renders_multiline_spans() {
        let source = SourceText::new("\"a\nb\nc");
        let rendered = source.render_syntax_error(&SyntaxError::UnterminatedString {
            span: Span::new(0, 6),
        });

        assert!(rendered.contains("span begins here"));
        assert!(rendered.contains("span ends here"));
        assert!(rendered.contains("span covers lines 1 through 3"));
    }

    #[test]
    fn source_diagnostic_exposes_structured_span_data() {
        let diagnostic = parse_with_diagnostics("f[1 ! 2]").unwrap_err().structured();

        assert!(diagnostic.message.starts_with("expected ,"));
        assert!(!diagnostic.is_incomplete);
        assert_eq!(diagnostic.spans.len(), 1);
        assert_eq!(
            diagnostic.spans[0].start,
            SourceLocation { line: 1, column: 5 }
        );
        assert_eq!(diagnostic.spans[0].text, "!");
    }
}
