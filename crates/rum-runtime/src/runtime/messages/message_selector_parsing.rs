use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum MessageSelectorNormalizationError {
    InvalidSelectorSpec,
}

impl MessageSelectorNormalizationError {
    pub(in crate::runtime) fn message(self) -> &'static str {
        match self {
            Self::InvalidSelectorSpec => {
                "expected the optional message selector to be a symbol, a \"symbol::tag\" string, or a list of those"
            }
        }
    }
}

impl Session {
    pub(in crate::runtime) fn normalize_optional_message_selectors(
        &self,
        selector_expr: Option<&Expr>,
    ) -> Result<Option<Vec<MessageSelector>>, MessageSelectorNormalizationError> {
        match selector_expr {
            Some(expr) => parse_message_selectors(expr)
                .map(Some)
                .ok_or(MessageSelectorNormalizationError::InvalidSelectorSpec),
            None => Ok(None),
        }
    }
}

pub(in crate::runtime) fn parse_message_selectors(expr: &Expr) -> Option<Vec<MessageSelector>> {
    match expr {
        Expr::Call { head, args, .. } if is_system_symbol_name(head, "List") => args
            .iter()
            .map(parse_message_selector_item)
            .collect::<Option<Vec<_>>>(),
        _ => Some(vec![parse_message_selector_item(expr)?]),
    }
}

pub(in crate::runtime) fn parse_message_selector_item(expr: &Expr) -> Option<MessageSelector> {
    if let Expr::Call { .. } = expr {
        let key = message_name_key_from_expr(expr)?;
        return Some(MessageSelector {
            symbol: key.symbol.to_string(),
            tag: Some(key.tag),
        });
    }

    match expr {
        Expr::Atom(Atom::String(spec)) => parse_message_selector_string(spec),
        Expr::Atom(Atom::Symbol(symbol)) => Some(MessageSelector {
            symbol: symbol.to_string(),
            tag: None,
        }),
        _ => None,
    }
}

pub(in crate::runtime) fn parse_message_selector_string(spec: &str) -> Option<MessageSelector> {
    if spec.is_empty() {
        return None;
    }

    if let Some((symbol, tag)) = spec.split_once("::") {
        if symbol.is_empty() || tag.is_empty() {
            return None;
        }

        return Some(MessageSelector {
            symbol: symbol.to_string(),
            tag: Some(tag.to_string()),
        });
    }

    Some(MessageSelector {
        symbol: spec.to_string(),
        tag: None,
    })
}

pub(in crate::runtime) fn message_name_key_from_expr(expr: &Expr) -> Option<MessageNameKey> {
    let Expr::Call { head, args, .. } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "MessageName") || args.len() != 2 {
        return None;
    }

    let symbol = match &args[0] {
        Expr::Atom(Atom::Symbol(symbol)) => symbol.clone(),
        Expr::Atom(Atom::String(raw)) => symbol_from_raw_name(raw),
        _ => return None,
    };

    let tag = match &args[1] {
        Expr::Atom(Atom::String(tag)) => tag.clone(),
        Expr::Atom(Atom::Symbol(tag)) => tag.name().to_string(),
        _ => return None,
    };

    Some(MessageNameKey { symbol, tag })
}

pub fn truthiness_symbol(expr: &Expr) -> Option<bool> {
    match expr_as_symbol(expr) {
        Some(symbol) if symbol.context() == SYSTEM_CONTEXT && symbol.name() == "True" => Some(true),
        Some(symbol) if symbol.context() == SYSTEM_CONTEXT && symbol.name() == "False" => {
            Some(false)
        }
        _ => None,
    }
}
