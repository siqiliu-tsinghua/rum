//! Injective component encoding. Ordering is over these bytes, not Symbol ids.
use super::super::{AggregateKey, synchronous::EventKey};
use crate::runtime::Symbol;

fn escape(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        match ch {
            '%' => out.push_str("%25"),
            '|' => out.push_str("%7C"),
            ',' => out.push_str("%2C"),
            '(' => out.push_str("%28"),
            ')' => out.push_str("%29"),
            '#' => out.push_str("%23"),
            _ => out.push(ch),
        }
    }
    out
}

pub(super) fn symbol(symbol: Symbol) -> String {
    format!(
        "s({},{},{})",
        escape(symbol.context()),
        escape(symbol.name()),
        symbol
            .identity()
            .map_or_else(|| "n".into(), |id| id.to_string()),
    )
}

fn source(value: Option<Symbol>) -> String {
    value.map_or_else(|| "-".into(), symbol)
}

pub(super) fn native(key: AggregateKey) -> String {
    match key {
        AggregateKey::Runtime { phase, source: s } => format!("runtime|{phase}|{}", source(s)),
        AggregateKey::Builtin {
            symbol: s,
            source: src,
            arity,
        } => {
            format!("builtin|{}|{}|{arity}", symbol(s), source(src))
        }
        AggregateKey::Definition {
            kind,
            symbol: s,
            index,
        } => {
            format!("definition-rule|{kind}|{}|{index}", symbol(s))
        }
        AggregateKey::RuleRhs { phase, source: s } => format!("rule-rhs|{phase}|{}", source(s)),
    }
}

pub(super) fn event(key: EventKey) -> (String, usize) {
    match key {
        EventKey::Runtime { phase, source: s } => (format!("runtime|{phase}|{}", source(s)), 7),
        EventKey::Rhs { phase, source: s } => (format!("rule-rhs|{phase}|{}", source(s)), 3),
        EventKey::Dispatch {
            kind,
            symbol: s,
            source: src,
        } => (
            format!("definition-dispatch|{kind}|{}|{}", symbol(s), source(src)),
            6,
        ),
        EventKey::Counter(name) => (format!("counter|{name}"), 1),
        EventKey::Cache(name) => (format!("cache|{name}"), 1),
        EventKey::CacheSymbol { event, symbol: s } => {
            (format!("cache-symbol|{event}|{}", symbol(s)), 1)
        }
        EventKey::CacheSource { event, symbol: s } => {
            (format!("cache-source|{event}|{}", symbol(s)), 1)
        }
    }
}
