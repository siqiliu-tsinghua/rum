use super::*;

impl<'a, const SPLIT_FORMS: bool> Parser<'a, SPLIT_FORMS> {
    pub(super) fn parse_slot(&mut self) -> Result<SyntaxExpr, SyntaxError> {
        let start_span = self.bump();
        let mut end_span = start_span;
        let is_sequence = matches!(self.current().kind, TokenKind::Hash)
            && self.current().leading_trivia.is_empty();

        if is_sequence {
            end_span = self.bump();
        }

        let spec = match (&self.current().kind, is_sequence) {
            (TokenKind::Integer(value), false) if self.current().leading_trivia.is_empty() => {
                let value = value.clone();
                end_span = self.bump();
                SlotSpec::Number(value)
            }
            (TokenKind::Symbol(value), false) if self.current().leading_trivia.is_empty() => {
                let value = value.clone();
                end_span = self.bump();
                SlotSpec::Named(value)
            }
            (TokenKind::Integer(value), true) if self.current().leading_trivia.is_empty() => {
                let value = value.clone();
                end_span = self.bump();
                SlotSpec::SequenceNumber(value)
            }
            _ if is_sequence => SlotSpec::Sequence,
            _ => SlotSpec::Anonymous,
        };

        Ok(SyntaxExpr::slot(spec, start_span.merge(end_span)))
    }

    pub(super) fn parse_blank_tail(
        &mut self,
        kind: BlankKind,
        start_span: Span,
    ) -> Result<SyntaxExpr, SyntaxError> {
        // The head parse re-enters `parse_postfix_expr`, whose postfix loop
        // can consume another blank (`a_b_c_...`), so a blank chain is a
        // right-associative chain and shares the operator-chain depth guard.
        let head = if self.token_starts_primary() && !self.at_form_boundary() {
            Some(self.with_operator_chain_depth(start_span, |parser| parser.parse_postfix_expr())?)
        } else {
            None
        };

        let span = head
            .as_ref()
            .map(|head| start_span.merge(head.span()))
            .unwrap_or(start_span);

        Ok(SyntaxExpr::blank(kind, head, span))
    }

    pub(super) fn parse_expression_list(
        &mut self,
        end_kind: TokenKind,
        end_label: &'static str,
        opening_span: Span,
        opening: &'static str,
    ) -> Result<(Vec<SyntaxExpr>, Span), SyntaxError> {
        let mut items = Vec::new();

        if self.current().kind == end_kind {
            let end_span = self.bump();
            return Ok((items, end_span));
        }

        if self.at_end() {
            return Err(SyntaxError::UnclosedDelimiter {
                expected: end_label,
                span: self.current().span,
                opening_span,
                opening,
            });
        }

        loop {
            items.push(self.parse_function_expr()?);

            if self.current().kind == end_kind {
                let end_span = self.bump();
                return Ok((items, end_span));
            }

            if self.at_end() {
                return Err(SyntaxError::UnclosedDelimiter {
                    expected: end_label,
                    span: self.current().span,
                    opening_span,
                    opening,
                });
            }

            self.expect_comma()?;

            if self.current().kind == end_kind {
                return Err(SyntaxError::UnexpectedToken {
                    expected: "expression",
                    found: self.current_token_label(),
                    span: self.current().span,
                });
            }

            if self.at_end() {
                return Err(SyntaxError::UnclosedDelimiter {
                    expected: end_label,
                    span: self.current().span,
                    opening_span,
                    opening,
                });
            }
        }
    }

    pub(super) fn token_starts_primary(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Integer(_)
                | TokenKind::String(_)
                | TokenKind::Symbol(_)
                | TokenKind::Hash
                | TokenKind::Underscore
                | TokenKind::DoubleUnderscore
                | TokenKind::TripleUnderscore
                | TokenKind::LParen
                | TokenKind::LBrace
                | TokenKind::LAssociation
        )
    }

    pub(super) fn token_starts_expression(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Plus
                | TokenKind::PlusPlus
                | TokenKind::Minus
                | TokenKind::MinusMinus
                | TokenKind::Bang
                | TokenKind::Span
        ) || self.token_starts_primary()
    }

    pub(super) fn current_binary_op(&self) -> Option<BinaryOp> {
        match self.current().kind {
            TokenKind::Semicolon => Some(BinaryOp::CompoundExpression),
            TokenKind::MessageName => Some(BinaryOp::MessageName),
            TokenKind::TagSet => Some(BinaryOp::TagSet),
            TokenKind::Set => Some(BinaryOp::Set),
            TokenKind::CaretEqual => Some(BinaryOp::UpSet),
            TokenKind::PlusEqual => Some(BinaryOp::AddTo),
            TokenKind::MinusEqual => Some(BinaryOp::SubtractFrom),
            TokenKind::StarEqual => Some(BinaryOp::TimesBy),
            TokenKind::SlashEqual => Some(BinaryOp::DivideBy),
            TokenKind::SlashSlashEqual => Some(BinaryOp::ApplyTo),
            TokenKind::SetDelayed => Some(BinaryOp::SetDelayed),
            TokenKind::CaretColonEqual => Some(BinaryOp::UpSetDelayed),
            TokenKind::ReplaceAll => Some(BinaryOp::ReplaceAll),
            TokenKind::ReplaceRepeated => Some(BinaryOp::ReplaceRepeated),
            TokenKind::Arrow => Some(BinaryOp::Rule),
            TokenKind::RuleDelayed => Some(BinaryOp::RuleDelayed),
            TokenKind::Condition => Some(BinaryOp::Condition),
            TokenKind::Or => Some(BinaryOp::Or),
            TokenKind::And => Some(BinaryOp::And),
            TokenKind::Same => Some(BinaryOp::SameQ),
            TokenKind::Unsame => Some(BinaryOp::UnsameQ),
            TokenKind::Equal => Some(BinaryOp::Equal),
            TokenKind::Unequal => Some(BinaryOp::Unequal),
            TokenKind::Less => Some(BinaryOp::Less),
            TokenKind::Greater => Some(BinaryOp::Greater),
            TokenKind::LessEqual => Some(BinaryOp::LessEqual),
            TokenKind::GreaterEqual => Some(BinaryOp::GreaterEqual),
            TokenKind::Colon => Some(BinaryOp::Pattern),
            TokenKind::Bar => Some(BinaryOp::Alternatives),
            TokenKind::Question => Some(BinaryOp::PatternTest),
            TokenKind::TildeTilde => Some(BinaryOp::StringExpression),
            TokenKind::StringJoin => Some(BinaryOp::StringJoin),
            TokenKind::At => Some(BinaryOp::Apply),
            TokenKind::SlashSlash => Some(BinaryOp::PostfixApply),
            TokenKind::SlashAt => Some(BinaryOp::Map),
            TokenKind::SlashSlashAt => Some(BinaryOp::MapAll),
            TokenKind::AtAt => Some(BinaryOp::ApplyHead),
            TokenKind::AtAtAt => Some(BinaryOp::MapApply),
            TokenKind::AtStar => Some(BinaryOp::Composition),
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Subtract),
            TokenKind::Star => Some(BinaryOp::Times),
            TokenKind::Dot => Some(BinaryOp::Dot),
            TokenKind::StarStar => Some(BinaryOp::NonCommutativeMultiply),
            TokenKind::Slash => Some(BinaryOp::Divide),
            TokenKind::SlashStar => Some(BinaryOp::RightComposition),
            TokenKind::Caret => Some(BinaryOp::Power),
            TokenKind::BarMinusGreater => Some(BinaryOp::Function),
            _ => None,
        }
    }

    pub(super) fn consume_ampersand(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::Ampersand) && !self.at_form_boundary() {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_increment(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::PlusPlus) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_decrement(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::MinusMinus) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_factorial2(&mut self) -> Option<Span> {
        if !matches!(self.current().kind, TokenKind::Bang)
            || !self.current().leading_trivia.is_empty()
        {
            return None;
        }
        if !self.tokens.get(self.index + 1).is_some_and(|next| {
            matches!(next.kind, TokenKind::Bang) && next.leading_trivia.is_empty()
        }) {
            return None;
        }

        let first = self.bump();
        let second = self.bump();
        Some(first.merge(second))
    }

    pub(super) fn consume_factorial(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::Bang)
            && self.current().leading_trivia.is_empty()
        {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_derivative(&mut self) -> Option<(u32, Span)> {
        if !matches!(self.current().kind, TokenKind::Prime)
            || !self.current().leading_trivia.is_empty()
        {
            return None;
        }

        let mut count = 1_u32;
        let start_span = self.bump();
        let mut end_span = start_span;

        while matches!(self.current().kind, TokenKind::Prime)
            && self.current().leading_trivia.is_empty()
        {
            count += 1;
            end_span = self.bump();
        }

        Some((count, start_span.merge(end_span)))
    }

    pub(super) fn consume_repeated(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::Repeated) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_repeated_null(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::RepeatedNull) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_dot(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::Dot) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_unset(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::Unset) && !self.at_form_boundary() {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_lbracket(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::LBracket) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn consume_double_lbracket(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::DoubleLBracket) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn current_starts_double_rbracket(&self) -> bool {
        matches!(self.current().kind, TokenKind::RBracket)
            && self
                .tokens
                .get(self.index + 1)
                .is_some_and(|next| matches!(next.kind, TokenKind::RBracket))
            // The `]]` pair is the one lookahead consumption with no
            // adjacency requirement, so a full-stream parse would pair two
            // brackets across a line boundary that the line-driven splitter
            // never tests together (its line prefix ends between them and
            // hard-errors on the lone `]`). Script-form splitting must not
            // pair them either. Compiles away when `SPLIT_FORMS` is false.
            && !(SPLIT_FORMS && self.line_breaks[self.index + 1])
    }

    pub(super) fn consume_double_rbracket(&mut self) -> Option<Span> {
        if !self.current_starts_double_rbracket() {
            return None;
        }

        let first = self.bump();
        let second = self.bump();
        Some(first.merge(second))
    }

    pub(super) fn consume_span(&mut self) -> Option<Span> {
        if matches!(self.current().kind, TokenKind::Span) {
            Some(self.bump())
        } else {
            None
        }
    }

    pub(super) fn expect_tilde(&mut self) -> Result<Span, SyntaxError> {
        if matches!(self.current().kind, TokenKind::Tilde) {
            Ok(self.bump())
        } else if self.at_end() {
            Err(SyntaxError::Incomplete {
                expected: "~",
                span: self.current().span,
            })
        } else {
            Err(SyntaxError::UnexpectedToken {
                expected: "~",
                found: self.current_token_label(),
                span: self.current().span,
            })
        }
    }

    pub(super) fn consume_blank_kind(&mut self) -> Option<(BlankKind, Span)> {
        let kind = match self.current().kind {
            TokenKind::Underscore => BlankKind::Blank,
            TokenKind::DoubleUnderscore => BlankKind::BlankSequence,
            TokenKind::TripleUnderscore => BlankKind::BlankNullSequence,
            _ => return None,
        };

        let span = self.bump();
        Some((kind, span))
    }

    pub(super) fn expect_comma(&mut self) -> Result<(), SyntaxError> {
        if matches!(self.current().kind, TokenKind::Comma) {
            self.bump();
            Ok(())
        } else if self.at_end() {
            Err(SyntaxError::Incomplete {
                expected: ",",
                span: self.current().span,
            })
        } else {
            Err(SyntaxError::UnexpectedToken {
                expected: ",",
                found: self.current_token_label(),
                span: self.current().span,
            })
        }
    }

    pub(super) fn expect_rparen(
        &mut self,
        opening_span: Span,
        opening: &'static str,
    ) -> Result<Span, SyntaxError> {
        if matches!(self.current().kind, TokenKind::RParen) {
            Ok(self.bump())
        } else if self.at_end() {
            Err(SyntaxError::UnclosedDelimiter {
                expected: ")",
                span: self.current().span,
                opening_span,
                opening,
            })
        } else {
            Err(SyntaxError::UnexpectedToken {
                expected: ")",
                found: self.current_token_label(),
                span: self.current().span,
            })
        }
    }

    pub(super) fn current_token_label(&self) -> String {
        self.current().kind.label().to_string()
    }

    pub(super) fn double_rbracket_label(&self) -> String {
        "]]".to_string()
    }

    pub(super) fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    pub(super) fn bump(&mut self) -> Span {
        let span = self.current().span;

        if !self.at_end() {
            self.index += 1;
        }

        span
    }

    pub(super) fn at_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::End)
    }
}
