use super::*;
use crate::{MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS, ast::SyntaxExprKind};

const SPAN_PRECEDENCE: u8 = 45;
const INFIX_APPLICATION_PRECEDENCE: u8 = 67;
/// A parser-specific guard kept below the debug and release native stack limits.
/// The recursive-descent parser uses multiple Rust frames per delimiter level,
/// so this is intentionally lower than the runtime evaluator's recursion limit.
/// Delimiter rather than "bracket" because `( ... )` is guarded too.
pub(crate) const MAX_DELIMITER_NESTING_DEPTH: usize = 128;
/// Guards binary-operator-chain recursion (right-associative spines such as
/// `^` chains, prefix-operator runs, and postfix blank chains like
/// `a_b_c_...`, whose blank-tail head parse re-enters the postfix loop).
/// This cap exists to fail gracefully
/// *before* the native stack runs out, so it is sized by the stack budget,
/// not by semantic analogy with the evaluator's `DEFAULT_RECURSION_LIMIT`:
/// the parser spends ~4.6 KiB of native stack per chain link, and a 2 MiB
/// thread — Rust's default for a spawned thread, and the realistic worst
/// case for an embedder — aborts at 452 links, so a cap of 1024 could never
/// fire there. 128 keeps roughly a 3.5× margin on that measurement and
/// matches the delimiter cap, which `2a23d5e` chose the same way.
pub(crate) const MAX_OPERATOR_CHAIN_DEPTH: usize = 128;
/// Bounds how many `;`-separated statements one parsed form may hold.
/// Counted along the whole spine — including empty statements, which return
/// to the climbing loop one `;` at a time — because the chain helper itself
/// holds at most two operands while the spine grows without bound.
/// Splitting the statements across lines sidesteps the cap: scripts are
/// split into forms line by line before parsing.
pub(crate) const MAX_STATEMENT_CHAIN_DEPTH: usize = 256;

pub fn parse_expr(input: &str) -> Result<SyntaxExpr, SyntaxError> {
    let tokens = lex(input)?;
    super::parse_tokens(tokens)
}

/// Parses a source slice whose byte zero starts at `source_offset` in its
/// containing file. Every returned syntax or error span uses file coordinates.
#[doc(hidden)]
pub fn parse_expr_with_offset(
    input: &str,
    source_offset: usize,
) -> Result<SyntaxExpr, SyntaxError> {
    if source_offset == 0 {
        return parse_expr(input);
    }

    let mut tokens = lex(input).map_err(|error| error.offset_by(source_offset))?;
    for token in &mut tokens {
        token.span = token.span.offset_by(source_offset);
    }
    super::parse_tokens(tokens)
}

pub(super) struct Parser<'a, const SPLIT_FORMS: bool = false> {
    pub(super) tokens: &'a [Token],
    pub(super) index: usize,
    delimiter_depth: usize,
    operator_chain_depth: usize,
    /// Script-form splitting mode: `line_breaks[i]` says token `i` is the
    /// first token on its source line. Empty unless `SPLIT_FORMS`.
    pub(super) line_breaks: &'a [bool],
    /// Depth of sub-parses inside a committed construct that a form can
    /// never end within (the `~`-infix head, whose closing `~` is still
    /// required) — the boundary guard stays off there.
    boundary_suspension_depth: usize,
}

impl<'a> Parser<'a, false> {
    pub(super) fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            index: 0,
            delimiter_depth: 0,
            operator_chain_depth: 0,
            line_breaks: &[],
            boundary_suspension_depth: 0,
        }
    }
}

impl<'a> Parser<'a, true> {
    pub(super) fn new_with_line_breaks(tokens: &'a [Token], line_breaks: &'a [bool]) -> Self {
        Self {
            tokens,
            index: 0,
            delimiter_depth: 0,
            operator_chain_depth: 0,
            line_breaks,
            boundary_suspension_depth: 0,
        }
    }
}

impl<'a, const SPLIT_FORMS: bool> Parser<'a, SPLIT_FORMS> {
    /// Script-form boundary rule: at delimiter depth 0, an optional
    /// continuation decision stops before a token that starts a new line.
    /// Compiles to a constant `false` when `SPLIT_FORMS` is false, so the
    /// single-expression parse path (`parse_expr`) pays nothing for the
    /// splitter it never uses.
    pub(super) fn at_form_boundary(&self) -> bool {
        SPLIT_FORMS
            && self.delimiter_depth == 0
            && self.boundary_suspension_depth == 0
            && self.line_breaks[self.index]
    }

    fn with_boundary_suspended<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<T, SyntaxError>,
    ) -> Result<T, SyntaxError> {
        self.boundary_suspension_depth += 1;
        let result = parse(self);
        self.boundary_suspension_depth -= 1;
        result
    }

    fn with_delimiter_depth<T>(
        &mut self,
        span: Span,
        parse: impl FnOnce(&mut Self) -> Result<T, SyntaxError>,
    ) -> Result<T, SyntaxError> {
        if self.delimiter_depth >= MAX_DELIMITER_NESTING_DEPTH {
            return Err(SyntaxError::ParseDepthExceeded {
                max: MAX_DELIMITER_NESTING_DEPTH,
                span,
            });
        }

        self.delimiter_depth += 1;
        let result = parse(self);
        self.delimiter_depth -= 1;
        result
    }

    pub(super) fn with_operator_chain_depth<T>(
        &mut self,
        span: Span,
        parse: impl FnOnce(&mut Self) -> Result<T, SyntaxError>,
    ) -> Result<T, SyntaxError> {
        if self.operator_chain_depth >= MAX_OPERATOR_CHAIN_DEPTH {
            return Err(SyntaxError::ParseDepthExceeded {
                max: MAX_OPERATOR_CHAIN_DEPTH,
                span,
            });
        }

        self.operator_chain_depth += 1;
        let result = parse(self);
        self.operator_chain_depth -= 1;
        result
    }

    pub(super) fn parse_root(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        if self.at_end() {
            return Err(SyntaxError::EmptyInput);
        }

        let expr = self.parse_function_expr()?;

        if self.at_end() || self.at_form_boundary() {
            Ok(expr)
        } else {
            Err(SyntaxError::TrailingInput {
                span: self.current().span,
            })
        }
    }

    pub(super) fn parse_function_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let mut expr = self.parse_binary_expr(0)?;

        loop {
            while let Some(ampersand_span) = self.consume_ampersand() {
                let span = expr.span().merge(ampersand_span);
                expr = SyntaxExpr::postfix(PostfixOp::Function, expr, span);
            }

            let Some(op) = self.current_binary_op() else {
                break;
            };

            if !can_follow_function_postfix(op) || self.at_form_boundary() {
                break;
            }

            self.bump();
            let rhs = self.parse_nested_binary_expr(next_min_precedence_for_binary_op(op))?;
            let span = expr.span().merge(rhs.span());
            expr = SyntaxExpr::binary(op, expr, rhs, span);
        }

        Ok(expr)
    }

    pub(super) fn parse_binary_expr(
        &mut self,
        min_precedence: u8,
    ) -> Result<SyntaxExpr, SyntaxError> {
        self.parse_binary_expr_internal(min_precedence, true, true)
    }

    fn parse_nested_binary_expr(&mut self, min_precedence: u8) -> Result<SyntaxExpr, SyntaxError> {
        let span = self.current().span;
        self.with_operator_chain_depth(span, |parser| parser.parse_binary_expr(min_precedence))
    }

    fn parse_nested_binary_expr_internal(
        &mut self,
        min_precedence: u8,
        allow_function_postfix: bool,
        allow_infix_application: bool,
    ) -> Result<SyntaxExpr, SyntaxError> {
        let span = self.current().span;
        self.with_operator_chain_depth(span, |parser| {
            parser.parse_binary_expr_internal(
                min_precedence,
                allow_function_postfix,
                allow_infix_application,
            )
        })
    }

    fn parse_binary_expr_internal(
        &mut self,
        min_precedence: u8,
        allow_function_postfix: bool,
        allow_infix_application: bool,
    ) -> Result<SyntaxExpr, SyntaxError> {
        let mut lhs = if matches!(self.current().kind, TokenKind::Span)
            && SPAN_PRECEDENCE >= min_precedence
        {
            self.parse_span_sugar(None)?
        } else {
            self.parse_prefix_expr()?
        };

        // `;` links consumed by this climbing loop and the chain helper it
        // calls, trailing-`Null` branch included. The count is local to one
        // call because this loop owns exactly one spine: an independent
        // spine elsewhere (`f[a;b]` vs `g[c;d]`) parses through a nested
        // call with its own fresh count.
        let mut statement_chain_links: usize = 0;
        // Every explicit left-associative operator and implicit `Times` link
        // consumed by this loop extends the same lhs spine. Nested and rhs
        // parses enter another call and therefore start a fresh count.
        let mut left_associative_chain_links: usize = 0;

        loop {
            if allow_function_postfix && PostfixOp::Function.precedence() >= min_precedence {
                if let Some(ampersand_span) = self.consume_ampersand() {
                    let span = lhs.span().merge(ampersand_span);
                    lhs = SyntaxExpr::postfix(PostfixOp::Function, lhs, span);
                    continue;
                }
            }

            if matches!(self.current().kind, TokenKind::Span)
                && SPAN_PRECEDENCE >= min_precedence
                && !self.at_form_boundary()
            {
                lhs = self.parse_span_sugar(Some(lhs))?;
                continue;
            }

            if matches!(self.current().kind, TokenKind::Tilde)
                && allow_infix_application
                && INFIX_APPLICATION_PRECEDENCE >= min_precedence
                && !self.at_form_boundary()
            {
                lhs = self.parse_infix_application(lhs, allow_function_postfix)?;
                continue;
            }

            let op = match self.current_binary_op() {
                Some(op) => op,
                None => {
                    // Implicit multiplication (juxtaposition): `a b` == Times[a, b].
                    let times = BinaryOp::Times;
                    if self.token_starts_primary()
                        && times.precedence() >= min_precedence
                        && !self.at_form_boundary()
                    {
                        count_left_associative_chain_link(
                            &mut left_associative_chain_links,
                            self.current().span,
                        )?;
                        let next_min_precedence = next_min_precedence_for_binary_op(times);
                        let rhs = self.parse_nested_binary_expr_internal(
                            next_min_precedence,
                            allow_function_postfix,
                            allow_infix_application,
                        )?;
                        let span = lhs.span().merge(rhs.span());
                        lhs = SyntaxExpr::binary(times, lhs, rhs, span);
                        continue;
                    }
                    break;
                }
            };

            if op.precedence() < min_precedence || self.at_form_boundary() {
                break;
            }

            if op.is_comparison() {
                lhs = self.parse_comparison_chain(
                    lhs,
                    allow_function_postfix,
                    allow_infix_application,
                    &mut left_associative_chain_links,
                )?;
                continue;
            }

            let operator_span = self.current().span;
            if !op.is_right_associative() {
                count_left_associative_chain_link(
                    &mut left_associative_chain_links,
                    operator_span,
                )?;
            }
            self.bump();

            if matches!(op, BinaryOp::CompoundExpression) {
                statement_chain_links += 1;
                // The spine starts at one statement (the lhs), so the
                // `MAX_STATEMENT_CHAIN_DEPTH`-th `;` would append statement
                // `MAX_STATEMENT_CHAIN_DEPTH + 1`.
                if statement_chain_links >= MAX_STATEMENT_CHAIN_DEPTH {
                    return Err(SyntaxError::StatementChainTooLong {
                        max: MAX_STATEMENT_CHAIN_DEPTH,
                        span: self.current().span,
                    });
                }

                if !self.token_starts_expression() || self.at_form_boundary() {
                    // Trailing `;`: the missing statement yields `Null`.
                    let rhs = SyntaxExpr::symbol("Null", self.current().span);
                    let span = lhs.span().merge(rhs.span());
                    lhs = SyntaxExpr::binary(op, lhs, rhs, span);
                    continue;
                }

                lhs = self.parse_compound_expression_chain(
                    lhs,
                    allow_function_postfix,
                    allow_infix_application,
                    &mut statement_chain_links,
                )?;
                continue;
            }

            let next_min_precedence = next_min_precedence_for_binary_op(op);

            let rhs = if matches!(op, BinaryOp::Rule | BinaryOp::RuleDelayed) {
                self.parse_nested_binary_expr_internal(
                    next_min_precedence,
                    false,
                    allow_infix_application,
                )?
            } else {
                self.parse_nested_binary_expr_internal(
                    next_min_precedence,
                    allow_function_postfix,
                    allow_infix_application,
                )?
            };
            let span = lhs.span().merge(rhs.span());
            lhs = SyntaxExpr::binary(op, lhs, rhs, span);
        }

        while let Some(unset_span) = self.consume_unset() {
            let span = lhs.span().merge(unset_span);
            lhs = SyntaxExpr::postfix(PostfixOp::Unset, lhs, span);
        }

        Ok(lhs)
    }

    fn parse_comparison_chain(
        &mut self,
        lhs: SyntaxExpr,
        allow_function_postfix: bool,
        allow_infix_application: bool,
        left_associative_chain_links: &mut usize,
    ) -> Result<SyntaxExpr, SyntaxError> {
        let mut operands = vec![lhs];
        let mut operators = Vec::new();

        while let Some(op) = self.current_binary_op() {
            if !op.is_comparison() || self.at_form_boundary() {
                break;
            }

            count_left_associative_chain_link(left_associative_chain_links, self.current().span)?;
            self.bump();
            let rhs = self.parse_nested_binary_expr_internal(
                next_min_precedence_for_binary_op(op),
                allow_function_postfix,
                allow_infix_application,
            )?;
            operators.push(op);
            operands.push(rhs);
        }

        let span = operands
            .first()
            .expect("comparison chain has a left operand")
            .span()
            .merge(
                operands
                    .last()
                    .expect("comparison chain has a right operand")
                    .span(),
            );
        Ok(SyntaxExpr::comparison_chain(operands, operators, span))
    }

    /// Collects the rest of a `;`-separated statement chain iteratively, so
    /// the chain links no longer consume recursion depth in either guard.
    /// The caller has consumed the leading `;` and verified that an
    /// expression follows. Each statement still parses through the ordinary
    /// operator-chain guard; only the `;` links are hoisted out of it. Every
    /// `;` consumed here increments the caller's spine-wide link count, so
    /// the statement cap bounds the whole spine, not one collection loop.
    fn parse_compound_expression_chain(
        &mut self,
        lhs: SyntaxExpr,
        allow_function_postfix: bool,
        allow_infix_application: bool,
        statement_chain_links: &mut usize,
    ) -> Result<SyntaxExpr, SyntaxError> {
        let mut operands = vec![lhs];

        loop {
            if !self.token_starts_expression() || self.at_form_boundary() {
                // An empty statement yields `Null` and ends this chain here:
                // breaking (rather than continuing) keeps an empty statement
                // locally left-leaning — `a; ; b` still parses as
                // `(a; Null); b`, with the remaining `;` links handled by the
                // climbing loop that called us.
                operands.push(SyntaxExpr::symbol("Null", self.current().span));
                break;
            }

            operands.push(self.parse_nested_binary_expr_internal(
                BinaryOp::CompoundExpression.precedence() + 1,
                allow_function_postfix,
                allow_infix_application,
            )?);

            if !matches!(self.current_binary_op(), Some(BinaryOp::CompoundExpression))
                || self.at_form_boundary()
            {
                break;
            }

            self.bump();
            *statement_chain_links += 1;
            if *statement_chain_links >= MAX_STATEMENT_CHAIN_DEPTH {
                return Err(SyntaxError::StatementChainTooLong {
                    max: MAX_STATEMENT_CHAIN_DEPTH,
                    span: self.current().span,
                });
            }
        }

        // Fold from the right so the AST shape is unchanged: `a; b; c` is
        // `a; (b; c)`, exactly what the recursive form produced.
        let mut expr = operands
            .pop()
            .expect("compound expression chain has at least one operand");
        while let Some(operand) = operands.pop() {
            let span = operand.span().merge(expr.span());
            expr = SyntaxExpr::binary(BinaryOp::CompoundExpression, operand, expr, span);
        }

        Ok(expr)
    }

    fn parse_infix_application(
        &mut self,
        lhs: SyntaxExpr,
        allow_function_postfix: bool,
    ) -> Result<SyntaxExpr, SyntaxError> {
        self.bump();

        if !self.token_starts_expression() {
            return Err(if self.at_end() {
                SyntaxError::Incomplete {
                    expected: "expression",
                    span: self.current().span,
                }
            } else {
                SyntaxError::UnexpectedToken {
                    expected: "expression",
                    found: self.current_token_label(),
                    span: self.current().span,
                }
            });
        }

        // The head of a `~`-infix application is a committed construct: the
        // closing `~` is still required, so no form can end inside it — and
        // stopping the head early at a line-first token would turn that
        // required `~` check into a hard error the line-driven splitter never
        // produces. Suspend the boundary guard for the head parse.
        let head = self.with_boundary_suspended(|parser| {
            parser.parse_nested_binary_expr_internal(0, allow_function_postfix, false)
        })?;
        self.expect_tilde()?;
        let rhs = self.parse_nested_binary_expr_internal(
            INFIX_APPLICATION_PRECEDENCE + 1,
            allow_function_postfix,
            true,
        )?;
        let span = lhs.span().merge(rhs.span());

        Ok(SyntaxExpr::call(head, vec![lhs, rhs], span))
    }

    pub(super) fn parse_prefix_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let token = self.current().clone();

        match &token.kind {
            TokenKind::Plus => {
                self.bump();
                let expr = self.parse_nested_binary_expr(PrefixOp::Plus.precedence())?;
                let span = token.span.merge(expr.span());
                Ok(SyntaxExpr::prefix(PrefixOp::Plus, expr, span))
            }
            TokenKind::PlusPlus => {
                self.bump();
                let expr = self.parse_nested_binary_expr(PrefixOp::PreIncrement.precedence())?;
                let span = token.span.merge(expr.span());
                Ok(SyntaxExpr::prefix(PrefixOp::PreIncrement, expr, span))
            }
            TokenKind::Minus => {
                self.bump();
                let expr = self.parse_nested_binary_expr(PrefixOp::Minus.precedence())?;
                let span = token.span.merge(expr.span());
                Ok(SyntaxExpr::prefix(PrefixOp::Minus, expr, span))
            }
            TokenKind::Bang => {
                self.bump();
                let expr = self.parse_nested_binary_expr(PrefixOp::Not.precedence())?;
                let span = token.span.merge(expr.span());
                Ok(SyntaxExpr::prefix(PrefixOp::Not, expr, span))
            }
            TokenKind::MinusMinus => {
                self.bump();
                let expr = self.parse_nested_binary_expr(PrefixOp::PreDecrement.precedence())?;
                let span = token.span.merge(expr.span());
                Ok(SyntaxExpr::prefix(PrefixOp::PreDecrement, expr, span))
            }
            _ => self.parse_postfix_expr(),
        }
    }

    pub(super) fn parse_postfix_expr(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.at_form_boundary() {
                break;
            }

            if let Some(open_span) = self.consume_double_lbracket() {
                let (mut specs, close_span) = self.with_delimiter_depth(open_span, |parser| {
                    parser.parse_double_bracket_expression_list(open_span)
                })?;
                let span = expr.span().merge(close_span);
                let mut args = vec![expr];
                args.append(&mut specs);
                expr = SyntaxExpr::call(SyntaxExpr::symbol("Part", open_span), args, span);
                continue;
            }

            if let Some(open_span) = self.consume_lbracket() {
                let (args, close_span) = self.with_delimiter_depth(open_span, |parser| {
                    parser.parse_expression_list(TokenKind::RBracket, "]", open_span, "[")
                })?;
                let span = expr.span().merge(close_span);
                expr = SyntaxExpr::call(expr, args, span);
                continue;
            }

            if let Some((kind, blank_span)) = self.consume_blank_kind() {
                let pattern = self.parse_blank_tail(kind, blank_span)?;
                let span = expr.span().merge(pattern.span());
                expr = SyntaxExpr::pattern(expr, pattern, span);
                continue;
            }

            if let Some(increment_span) = self.consume_increment() {
                let span = expr.span().merge(increment_span);
                expr = SyntaxExpr::postfix(PostfixOp::Increment, expr, span);
                continue;
            }

            if let Some(decrement_span) = self.consume_decrement() {
                let span = expr.span().merge(decrement_span);
                expr = SyntaxExpr::postfix(PostfixOp::Decrement, expr, span);
                continue;
            }

            if let Some(factorial2_span) = self.consume_factorial2() {
                let span = expr.span().merge(factorial2_span);
                expr = SyntaxExpr::postfix(PostfixOp::Factorial2, expr, span);
                continue;
            }

            if let Some(factorial_span) = self.consume_factorial() {
                let span = expr.span().merge(factorial_span);
                expr = SyntaxExpr::postfix(PostfixOp::Factorial, expr, span);
                continue;
            }

            if let Some((order, derivative_span)) = self.consume_derivative() {
                let span = expr.span().merge(derivative_span);
                expr = SyntaxExpr::postfix(PostfixOp::Derivative(order), expr, span);
                continue;
            }

            if let Some(repeated_null_span) = self.consume_repeated_null() {
                let span = expr.span().merge(repeated_null_span);
                expr = SyntaxExpr::postfix(PostfixOp::RepeatedNull, expr, span);
                continue;
            }

            if let Some(repeated_span) = self.consume_repeated() {
                let span = expr.span().merge(repeated_span);
                expr = SyntaxExpr::postfix(PostfixOp::Repeated, expr, span);
                continue;
            }

            if is_optional_pattern_target(&expr) {
                if let Some(optional_span) = self.consume_dot() {
                    let span = expr.span().merge(optional_span);
                    expr = SyntaxExpr::postfix(PostfixOp::Optional, expr, span);
                    continue;
                }
            }

            break;
        }

        Ok(expr)
    }

    fn parse_double_bracket_expression_list(
        &mut self,
        opening_span: Span,
    ) -> Result<(Vec<SyntaxExpr>, Span), SyntaxError> {
        let mut items = Vec::new();

        if let Some(close_span) = self.consume_double_rbracket() {
            return Ok((items, close_span));
        }

        if self.at_end() {
            return Err(SyntaxError::UnclosedDelimiter {
                expected: "]]",
                span: self.current().span,
                opening_span,
                opening: "[[",
            });
        }

        loop {
            items.push(self.parse_function_expr()?);

            if let Some(close_span) = self.consume_double_rbracket() {
                return Ok((items, close_span));
            }

            if self.at_end() {
                return Err(SyntaxError::UnclosedDelimiter {
                    expected: "]]",
                    span: self.current().span,
                    opening_span,
                    opening: "[[",
                });
            }

            self.expect_comma()?;

            if self.current_starts_double_rbracket() {
                return Err(SyntaxError::UnexpectedToken {
                    expected: "expression",
                    found: self.double_rbracket_label(),
                    span: self.current().span.merge(self.tokens[self.index + 1].span),
                });
            }

            if self.at_end() {
                return Err(SyntaxError::UnclosedDelimiter {
                    expected: "]]",
                    span: self.current().span,
                    opening_span,
                    opening: "[[",
                });
            }
        }
    }

    fn parse_span_sugar(&mut self, start: Option<SyntaxExpr>) -> Result<SyntaxExpr, SyntaxError> {
        let first_span = self
            .consume_span()
            .expect("parse_span_sugar only called at a span token");

        let start_span = start.as_ref().map(SyntaxExpr::span).unwrap_or(first_span);
        let mut end_span = first_span;
        let end = if matches!(self.current().kind, TokenKind::Span)
            || !self.token_starts_expression()
            || self.at_form_boundary()
        {
            None
        } else {
            let expr = self.parse_nested_binary_expr(SPAN_PRECEDENCE + 1)?;
            end_span = expr.span();
            Some(expr)
        };

        let step = if !self.at_form_boundary() && self.consume_span().is_some() {
            if !self.token_starts_expression() {
                return Err(if self.at_end() {
                    SyntaxError::Incomplete {
                        expected: "expression",
                        span: self.current().span,
                    }
                } else {
                    SyntaxError::UnexpectedToken {
                        expected: "expression",
                        found: self.current_token_label(),
                        span: self.current().span,
                    }
                });
            }

            let expr = self.parse_nested_binary_expr(SPAN_PRECEDENCE + 1)?;
            end_span = expr.span();
            Some(expr)
        } else {
            None
        };

        let args = match (start, end, step) {
            (Some(start), Some(end), Some(step)) => vec![start, end, step],
            (Some(start), Some(end), None) => vec![start, end],
            (Some(start), None, Some(step)) => {
                vec![start, SyntaxExpr::symbol("All", first_span), step]
            }
            (Some(start), None, None) => vec![start, SyntaxExpr::symbol("All", first_span)],
            (None, Some(end), Some(step)) => {
                vec![SyntaxExpr::symbol("All", first_span), end, step]
            }
            (None, Some(end), None) => vec![SyntaxExpr::symbol("All", first_span), end],
            (None, None, Some(step)) => vec![
                SyntaxExpr::symbol("All", first_span),
                SyntaxExpr::symbol("All", first_span),
                step,
            ],
            (None, None, None) => vec![SyntaxExpr::symbol("All", first_span)],
        };

        let span = start_span.merge(end_span);
        Ok(SyntaxExpr::call(
            SyntaxExpr::symbol("Span", first_span),
            args,
            span,
        ))
    }

    pub(super) fn parse_primary(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let token = self.current().clone();

        match token.kind {
            TokenKind::Integer(value) => {
                self.bump();
                Ok(SyntaxExpr::integer(value, token.span))
            }
            TokenKind::String(value) => {
                self.bump();
                Ok(SyntaxExpr::string(value, token.span))
            }
            TokenKind::Symbol(value) => {
                self.bump();
                Ok(SyntaxExpr::symbol(value, token.span))
            }
            TokenKind::Hash => self.parse_slot(),
            TokenKind::Underscore => {
                self.bump();
                self.parse_blank_tail(BlankKind::Blank, token.span)
            }
            TokenKind::DoubleUnderscore => {
                self.bump();
                self.parse_blank_tail(BlankKind::BlankSequence, token.span)
            }
            TokenKind::TripleUnderscore => {
                self.bump();
                self.parse_blank_tail(BlankKind::BlankNullSequence, token.span)
            }
            TokenKind::LParen => {
                self.bump();
                self.with_delimiter_depth(token.span, |parser| {
                    let expr = parser.parse_function_expr()?;
                    let close_span = parser.expect_rparen(token.span, "(")?;
                    Ok(expr.with_span(token.span.merge(close_span)))
                })
            }
            TokenKind::LBrace => {
                self.bump();
                self.with_delimiter_depth(token.span, |parser| {
                    let (items, close_span) =
                        parser.parse_expression_list(TokenKind::RBrace, "}", token.span, "{")?;
                    Ok(SyntaxExpr::list(items, token.span.merge(close_span)))
                })
            }
            TokenKind::LAssociation => {
                self.bump();
                self.with_delimiter_depth(token.span, |parser| {
                    let (items, close_span) = parser.parse_expression_list(
                        TokenKind::RAssociation,
                        "|>",
                        token.span,
                        "<|",
                    )?;
                    Ok(SyntaxExpr::association(items, token.span.merge(close_span)))
                })
            }
            TokenKind::End => Err(SyntaxError::Incomplete {
                expected: "expression",
                span: token.span,
            }),
            _ => Err(SyntaxError::UnexpectedToken {
                expected: "expression",
                found: self.current_token_label(),
                span: token.span,
            }),
        }
    }
}

fn is_optional_pattern_target(expr: &SyntaxExpr) -> bool {
    match &expr.kind {
        SyntaxExprKind::Blank { .. } => true,
        SyntaxExprKind::Pattern { pattern, .. } => {
            matches!(&pattern.kind, SyntaxExprKind::Blank { .. })
        }
        _ => false,
    }
}

fn next_min_precedence_for_binary_op(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::ReplaceAll | BinaryOp::ReplaceRepeated => BinaryOp::Rule.precedence(),
        BinaryOp::Rule | BinaryOp::RuleDelayed => BinaryOp::Set.precedence(),
        _ if op.is_right_associative() => op.precedence(),
        _ => op.precedence() + 1,
    }
}

fn can_follow_function_postfix(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Apply | BinaryOp::Map | BinaryOp::ApplyHead | BinaryOp::MapApply
    )
}

fn count_left_associative_chain_link(links: &mut usize, span: Span) -> Result<(), SyntaxError> {
    if *links >= MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS {
        return Err(SyntaxError::LeftAssociativeChainTooLong {
            max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
            span,
        });
    }

    *links += 1;
    Ok(())
}
