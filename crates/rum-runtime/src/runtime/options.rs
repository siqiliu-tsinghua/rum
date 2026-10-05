use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod arguments;
mod builtins;
mod core;
mod defaults;
mod patterns;
mod read_phase;
#[cfg(test)]
#[path = "options/tests/read_phase.rs"]
mod read_phase_tests;
mod reentry;
#[cfg(test)]
#[path = "options/tests/compile.rs"]
mod reentry_tests;
mod resume_frames;

pub use read_phase::{FrozenOptionReadContinuation, OptionReadContinuation};
pub use reentry::{OptionReentry, OptionReentryDeclaration};
pub(in crate::runtime) use resume_frames::MergedOptionsSuspensionFrame;
pub(in crate::runtime) use resume_frames::RuntimeOptionSuspensionFrame;

pub(super) use defaults::{default_system_option_snapshot, default_system_options};
pub(super) use patterns::{is_options_pattern_expr, options_pattern_sequence_matcher};

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) struct SymbolOptionEntry {
    name: Symbol,
    value: Expr,
    delayed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, FrameRootTrace)]
pub(super) struct SymbolOptions {
    entries: Vec<SymbolOptionEntry>,
}

impl SymbolOptions {
    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    pub(super) fn contains_key(&self, key: &Symbol) -> bool {
        self.entries.iter().any(|entry| entry.name == *key)
    }

    pub(super) fn contains_name(&self, key: &str) -> bool {
        self.entries.iter().any(|entry| entry.name.name() == key)
    }

    pub(super) fn get(&self, key: &Symbol) -> Option<&Expr> {
        self.entry(key).map(|entry| &entry.value)
    }

    pub(super) fn get_by_name(&self, key: &str) -> Option<&Expr> {
        self.entry_by_name(key).map(|entry| &entry.value)
    }

    fn entry(&self, key: &Symbol) -> Option<&SymbolOptionEntry> {
        self.entries.iter().find(|entry| entry.name == *key)
    }

    fn entry_by_name(&self, key: &str) -> Option<&SymbolOptionEntry> {
        self.entries.iter().find(|entry| entry.name.name() == key)
    }

    fn set_existing(&mut self, key: &Symbol, value: Expr, delayed: bool) -> bool {
        if let Some(current) = self.entries.iter_mut().find(|entry| entry.name == *key) {
            current.value = value;
            current.delayed = delayed;
            true
        } else {
            false
        }
    }

    fn set_existing_by_name(&mut self, key: &str, value: Expr, delayed: bool) -> bool {
        if let Some(current) = self
            .entries
            .iter_mut()
            .find(|entry| entry.name.name() == key)
        {
            current.value = value;
            current.delayed = delayed;
            true
        } else {
            false
        }
    }

    pub(super) fn from_entries(entries: Vec<SymbolOptionEntry>) -> Self {
        let mut normalized = Vec::new();

        for entry in entries {
            if let Some(existing) = normalized
                .iter_mut()
                .find(|existing: &&mut SymbolOptionEntry| existing.name == entry.name)
            {
                *existing = entry;
            } else {
                normalized.push(entry);
            }
        }

        Self {
            entries: normalized,
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (&Symbol, &Expr, bool)> {
        self.entries
            .iter()
            .map(|entry| (&entry.name, &entry.value, entry.delayed))
    }

    pub(super) fn rule_exprs(&self) -> Vec<Expr> {
        self.entries
            .iter()
            .map(|entry| {
                Expr::call(
                    system_expr(if entry.delayed { "RuleDelayed" } else { "Rule" }),
                    vec![
                        Expr::Atom(Atom::Symbol(entry.name.clone())),
                        entry.value.clone(),
                    ],
                )
            })
            .collect()
    }

    /// Consume the private snapshot into its final shared slots. The original
    /// session options stay untouched; no temporary rule Vec or second clone
    /// of the snapshot's values is needed before the option loop starts.
    pub(super) fn into_shared_rule_exprs(self, spare: usize) -> std::sync::Arc<[Expr]> {
        self.entries
            .into_iter()
            .map(|entry| {
                Expr::call(
                    system_expr(if entry.delayed { "RuleDelayed" } else { "Rule" }),
                    vec![Expr::Atom(Atom::Symbol(entry.name)), entry.value],
                )
            })
            .chain(std::iter::repeat_with(|| Expr::small_integer(0)).take(spare))
            .collect()
    }

    /// A stored option table stays borrowed until the independent snapshot is
    /// complete. No guest runs while this borrow is live.
    #[inline(always)]
    pub(super) fn shared_rule_exprs(&self, spare: usize) -> std::sync::Arc<[Expr]> {
        self.entries
            .iter()
            .map(|entry| {
                Expr::call(
                    system_expr(if entry.delayed { "RuleDelayed" } else { "Rule" }),
                    vec![
                        Expr::Atom(Atom::Symbol(entry.name.clone())),
                        entry.value.clone(),
                    ],
                )
            })
            .chain(std::iter::repeat_with(|| Expr::small_integer(0)).take(spare))
            .collect()
    }

    pub(super) fn expr(&self) -> Expr {
        Expr::call(system_expr("List"), self.rule_exprs())
    }

    pub(super) fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationBlockSymbolOptions"),
            self.entries
                .into_iter()
                .map(|entry| {
                    Expr::call(
                        system_expr("InternalContinuationBlockSymbolOption"),
                        vec![
                            Expr::Atom(Atom::Symbol(entry.name)),
                            entry.value,
                            entry.delayed.into_continuation_expr(),
                        ],
                    )
                })
                .collect(),
        )
    }

    pub(super) fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args } = expr else {
            panic!("unexpected Block options continuation payload")
        };
        assert!(is_system_symbol_name(
            &head,
            "InternalContinuationBlockSymbolOptions"
        ));
        Self::from_entries(
            args.into_iter()
                .map(|entry| {
                    let Expr::Call { head, args } = entry else {
                        panic!("unexpected Block option continuation payload")
                    };
                    assert!(is_system_symbol_name(
                        &head,
                        "InternalContinuationBlockSymbolOption"
                    ));
                    let mut args = args.into_iter();
                    let name_expr = args.next().expect("Block option stores its name");
                    let Expr::Atom(Atom::Symbol(name)) = name_expr else {
                        panic!("Block option name should be a symbol: {name_expr:?}")
                    };
                    let value = args.next().expect("Block option stores its value");
                    let delayed = bool::from_continuation_expr(
                        args.next().expect("Block option stores its delayed flag"),
                    );
                    debug_assert!(args.next().is_none());
                    SymbolOptionEntry {
                        name,
                        value,
                        delayed,
                    }
                })
                .collect(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) struct OptionValueScope {
    target: Symbol,
    explicit: SymbolOptions,
}

impl OptionValueScope {
    pub(super) fn target(&self) -> &Symbol {
        &self.target
    }

    pub(super) fn explicit(&self) -> &SymbolOptions {
        &self.explicit
    }

    pub(super) fn into_continuation_expr(self) -> Expr {
        Expr::call(
            system_expr("InternalContinuationOptionValueScope"),
            vec![
                Expr::Atom(Atom::Symbol(self.target)),
                self.explicit.into_continuation_expr(),
            ],
        )
    }

    pub(super) fn from_continuation_expr(expr: Expr) -> Self {
        let Expr::Call { head, args } = expr else {
            panic!("unexpected option-value-scope continuation payload")
        };
        assert!(
            is_system_symbol_name(&head, "InternalContinuationOptionValueScope"),
            "unexpected option-value-scope continuation tag: {head:?}"
        );
        let mut args = args.into_iter();
        let target = match args
            .next()
            .expect("option-value scope stores its target symbol")
        {
            Expr::Atom(Atom::Symbol(symbol)) => symbol,
            other => panic!("option-value-scope target should be a symbol: {other:?}"),
        };
        let explicit = SymbolOptions::from_continuation_expr(
            args.next()
                .expect("option-value scope stores its explicit options"),
        );
        debug_assert!(args.next().is_none());
        Self { target, explicit }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OptionsNormalizationError {
    InvalidTarget,
    InvalidRules,
    InvalidOptionName,
}

impl OptionsNormalizationError {
    pub(super) fn message(self) -> &'static str {
        match self {
            Self::InvalidTarget => "expected a symbol or symbol-name string argument",
            Self::InvalidRules => {
                "expected a rule or list of rules using symbol or symbol-name-string option names"
            }
            Self::InvalidOptionName => "expected an option symbol or symbol-name string",
        }
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<OptionValueScope>();
    assert_traces::<SymbolOptionEntry>();
    assert_traces::<SymbolOptions>();
};
