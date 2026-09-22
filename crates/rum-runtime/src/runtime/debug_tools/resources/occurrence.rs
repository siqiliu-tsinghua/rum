//! Independent, test-only specification of the occurrence memory proxy.
//!
//! Never call production `*_memory_proxy_bytes` helpers from this tree, and
//! never use this oracle to implement a maintained total. Sharing structural
//! access (including FrameRootTrace) is intentional; sharing accounting is not.
//! Reading the oracle neither samples the peak nor prunes/initializes caches.

use super::*;
use std::mem::{size_of, size_of_val};

mod arithmetic;
pub(in crate::runtime) mod association;
pub(in crate::runtime) mod dispatch;
mod expr;
mod payloads;
mod schema;
pub(in crate::runtime) mod suspension;
pub(in crate::runtime) mod tagged;

pub(in crate::runtime) use arithmetic::Bytes;
pub(in crate::runtime) use expr::expression;
pub(in crate::runtime) use payloads::definition;
pub(in crate::runtime) use schema::{
    SESSION_MEMORY_FIELD_DECISIONS, SessionMemoryFieldDecision,
    assert_session_memory_decisions_are_exhaustive, session_breakdown,
};

use arithmetic::{slots, sum};
use expr::{shared_args, symbol, symbol_text};
use payloads::*;

#[derive(Debug, PartialEq, Eq)]
pub(in crate::runtime) struct FieldContribution {
    pub field: &'static str,
    pub bytes: Bytes,
}

#[derive(Debug, PartialEq, Eq)]
pub(in crate::runtime) struct MemoryBreakdown {
    /// Covers all inline fields, including the recorded exemptions.
    pub session_inline: Bytes,
    pub fields: Vec<FieldContribution>,
}

impl MemoryBreakdown {
    pub fn total(&self) -> Bytes {
        self.session_inline
            .add(sum(self.fields.iter().map(|field| field.bytes)))
    }

    pub fn field(&self, name: &str) -> Bytes {
        self.fields
            .iter()
            .find(|field| field.field == name)
            .unwrap_or_else(|| panic!("unclassified Session field: {name}"))
            .bytes
    }
}

fn spare<T>(values: &Vec<T>) -> Bytes {
    // This subtraction is of real container sizes, not a release from a
    // saturated total: Vec guarantees capacity >= length. An invalid synthetic
    // capacity must fail closed too, rather than silently becoming zero.
    Bytes::Exact(values.capacity())
        .release(Bytes::Exact(values.len()))
        .multiply(size_of::<T>())
}

fn sequence<T>(values: &Vec<T>, item: impl Fn(&T) -> Bytes) -> Bytes {
    spare(values).add(sum(values.iter().map(item)))
}

fn strings(values: &Vec<String>) -> Bytes {
    slots::<String>(values.capacity()).add(sum(values
        .iter()
        .map(|value| Bytes::Exact(value.capacity()))))
}

fn paths(values: &Vec<PathBuf>) -> Bytes {
    slots::<PathBuf>(values.capacity()).add(sum(values
        .iter()
        .map(|path| Bytes::Exact(path.as_os_str().len()))))
}

fn symbol_map<V, S>(values: &HashMap<Symbol, V, S>) -> Bytes {
    slots::<(Symbol, V)>(values.capacity()).add(sum(values.keys().map(symbol_text)))
}

fn symbol_set<S>(values: &std::collections::HashSet<Symbol, S>) -> Bytes {
    slots::<Symbol>(values.capacity()).add(sum(values.iter().map(symbol_text)))
}

fn arc_object<T: ?Sized>(value: &T) -> Bytes {
    slots::<usize>(2).add(Bytes::Exact(size_of_val(value)))
}
