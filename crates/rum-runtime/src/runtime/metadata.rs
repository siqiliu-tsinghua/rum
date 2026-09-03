use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

pub(super) const DEFAULT_REPLACE_REPEATED_LIMIT: usize = 256;
pub(super) const DEFAULT_RECURSION_LIMIT: usize = 1024;
/// Smallest declared thread stack supported by the default resource profile.
///
/// This is a caller-declared contract; the runtime does not discover the
/// current thread's stack bounds.
pub const MIN_SUPPORTED_NATIVE_STACK_BYTES: usize = 2 * 1024 * 1024;
/// Native-stack allowance for one evaluation level between byte-budget polls.
///
/// The 128 KiB reserve is more than six times the measured 20.6 KiB debug
/// user-recursion cost.
pub const NATIVE_STACK_UNPOLLED_LEVEL_RESERVE_BYTES: usize = 128 * 1024;
/// Number of evaluation-depth increments between native-stack byte-budget polls.
pub const NATIVE_STACK_DEPTH_POLL_STRIDE: usize = 4;
/// Native-stack headroom reserved for evaluation levels skipped between polls.
pub const NATIVE_STACK_POLL_HEADROOM_BYTES: usize =
    NATIVE_STACK_DEPTH_POLL_STRIDE * NATIVE_STACK_UNPOLLED_LEVEL_RESERVE_BYTES;
/// Native-stack headroom reserved outside the measured evaluator descent.
pub const NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES: usize = 512 * 1024;
/// Default native-stack byte budget available to evaluator descent.
///
/// This is derived from the minimum supported declared stack after reserving
/// poll headroom and non-evaluator capacity.
pub const DEFAULT_NATIVE_STACK_LIMIT: usize = MIN_SUPPORTED_NATIVE_STACK_BYTES
    - NATIVE_STACK_POLL_HEADROOM_BYTES
    - NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES;
pub(super) const DEFAULT_EVALUATION_PASS_LIMIT: usize = 256;
pub(super) const DEFAULT_GENERATED_ELEMENT_LIMIT: usize = 100_000;
pub(super) const DEFAULT_INTEGER_WORK_LIMIT: usize = 100_000;
pub(super) const DEFAULT_ITERATION_WORK_LIMIT: usize = 100_000;

/// Derives an evaluator byte budget from a caller-declared thread stack size.
///
/// This does not discover or verify the current thread's actual stack size;
/// safe Rust has no portable API for doing so. Callers must pass the stack size
/// they provisioned and lower the result if evaluation begins beneath
/// substantial caller frames. Declared stacks below
/// [`MIN_SUPPORTED_NATIVE_STACK_BYTES`] are unsupported by this calibration and
/// return `None`.
pub const fn native_stack_limit_for_declared_stack(stack_bytes: usize) -> Option<usize> {
    if stack_bytes < MIN_SUPPORTED_NATIVE_STACK_BYTES {
        return None;
    }

    match stack_bytes.checked_sub(NATIVE_STACK_POLL_HEADROOM_BYTES) {
        Some(remaining) => remaining.checked_sub(NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES),
        None => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub enum HoldAttribute {
    None,
    HoldFirst,
    HoldRest,
    HoldAll,
}

impl HoldAttribute {
    pub(super) fn holds_argument(self, index: usize) -> bool {
        match self {
            Self::None => false,
            Self::HoldFirst => index == 0,
            Self::HoldRest => index > 0,
            Self::HoldAll => true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attribute {
    HoldAllComplete,
    HoldAll,
    HoldFirst,
    HoldRest,
    SubValuesHoldAll,
    SequenceHold,
    Listable,
    NHoldAll,
    NumericFunction,
    Orderless,
    Flat,
    OneIdentity,
    NonThreadable,
    Protected,
    ReadProtected,
    Temporary,
}

impl Attribute {
    pub(super) const fn bit(self) -> u16 {
        match self {
            Self::HoldAllComplete => 1 << 0,
            Self::HoldAll => 1 << 1,
            Self::HoldFirst => 1 << 2,
            Self::HoldRest => 1 << 3,
            // Bit 15 is the last free u16 bit; another attribute needs a wider mask.
            Self::SubValuesHoldAll => 1 << 15,
            Self::SequenceHold => 1 << 4,
            Self::Listable => 1 << 5,
            Self::NHoldAll => 1 << 14,
            Self::NumericFunction => 1 << 6,
            Self::Orderless => 1 << 7,
            Self::Flat => 1 << 8,
            Self::OneIdentity => 1 << 9,
            Self::NonThreadable => 1 << 13,
            Self::Protected => 1 << 10,
            Self::ReadProtected => 1 << 11,
            Self::Temporary => 1 << 12,
        }
    }

    pub(super) fn symbol_name(self) -> &'static str {
        match self {
            Self::HoldAllComplete => "HoldAllComplete",
            Self::HoldAll => "HoldAll",
            Self::HoldFirst => "HoldFirst",
            Self::HoldRest => "HoldRest",
            Self::SubValuesHoldAll => "SubValuesHoldAll",
            Self::SequenceHold => "SequenceHold",
            Self::Listable => "Listable",
            Self::NHoldAll => "NHoldAll",
            Self::NumericFunction => "NumericFunction",
            Self::Orderless => "Orderless",
            Self::Flat => "Flat",
            Self::OneIdentity => "OneIdentity",
            Self::NonThreadable => "NonThreadable",
            Self::Protected => "Protected",
            Self::ReadProtected => "ReadProtected",
            Self::Temporary => "Temporary",
        }
    }

    pub(super) fn from_expr(expr: &Expr) -> Option<Self> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) if symbol.context() == SYSTEM_CONTEXT => {
                Self::from_name(symbol.name())
            }
            Expr::Atom(Atom::String(name)) => Self::from_name(name),
            _ => None,
        }
    }

    pub(super) fn from_name(name: &str) -> Option<Self> {
        match name {
            "HoldAllComplete" => Some(Self::HoldAllComplete),
            "HoldAll" => Some(Self::HoldAll),
            "HoldFirst" => Some(Self::HoldFirst),
            "HoldRest" => Some(Self::HoldRest),
            "SubValuesHoldAll" => Some(Self::SubValuesHoldAll),
            "SequenceHold" => Some(Self::SequenceHold),
            "Listable" => Some(Self::Listable),
            "NHoldAll" => Some(Self::NHoldAll),
            "NumericFunction" => Some(Self::NumericFunction),
            "Orderless" => Some(Self::Orderless),
            "Flat" => Some(Self::Flat),
            "OneIdentity" => Some(Self::OneIdentity),
            "NonThreadable" => Some(Self::NonThreadable),
            "Protected" => Some(Self::Protected),
            "ReadProtected" => Some(Self::ReadProtected),
            "Temporary" => Some(Self::Temporary),
            _ => None,
        }
    }

    pub(super) fn expr(self) -> Expr {
        system_expr(self.symbol_name())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FrameRootTrace)]
pub struct AttributeSet {
    bits: u16,
}

impl AttributeSet {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn from_attribute(attribute: Attribute) -> Self {
        Self {
            bits: Self::normalized_bits(attribute.bit()),
        }
    }

    pub const fn from_hold(hold: HoldAttribute) -> Self {
        match hold {
            HoldAttribute::None => Self::empty(),
            HoldAttribute::HoldFirst => Self::from_attribute(Attribute::HoldFirst),
            HoldAttribute::HoldRest => Self::from_attribute(Attribute::HoldRest),
            HoldAttribute::HoldAll => Self::from_attribute(Attribute::HoldAll),
        }
    }

    pub const fn with(self, attribute: Attribute) -> Self {
        Self {
            bits: Self::normalized_bits(self.bits | attribute.bit()),
        }
    }

    pub(super) fn contains(self, attribute: Attribute) -> bool {
        self.bits & attribute.bit() != 0
    }

    pub(super) fn insert(&mut self, attribute: Attribute) {
        self.bits = Self::normalized_bits(self.bits | attribute.bit());
    }

    pub(super) fn remove(&mut self, attribute: Attribute) {
        self.bits = Self::normalized_bits(self.bits & !attribute.bit());
    }

    const fn normalized_bits(bits: u16) -> u16 {
        let hold_all = Attribute::HoldAll.bit();
        let hold_first = Attribute::HoldFirst.bit();
        let hold_rest = Attribute::HoldRest.bit();
        let has_hold_all = bits & hold_all != 0;
        let has_hold_first = bits & hold_first != 0;
        let has_hold_rest = bits & hold_rest != 0;

        if has_hold_all || (has_hold_first && has_hold_rest) {
            (bits | hold_all) & !hold_first & !hold_rest
        } else {
            bits
        }
    }

    pub(super) fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub(super) fn len(self) -> usize {
        self.attributes().len()
    }

    #[doc(hidden)]
    pub fn iter(self) -> impl Iterator<Item = Attribute> {
        ALL_ATTRIBUTES
            .iter()
            .copied()
            .filter(move |attribute| self.contains(*attribute))
    }

    pub(super) fn attributes(self) -> Vec<Attribute> {
        self.iter().collect()
    }

    pub(super) fn hold_attribute(self) -> HoldAttribute {
        let holds_first = self.contains(Attribute::HoldFirst);
        let holds_rest = self.contains(Attribute::HoldRest);

        if self.contains(Attribute::HoldAllComplete)
            || self.contains(Attribute::HoldAll)
            || (holds_first && holds_rest)
        {
            HoldAttribute::HoldAll
        } else if holds_first {
            HoldAttribute::HoldFirst
        } else if holds_rest {
            HoldAttribute::HoldRest
        } else {
            HoldAttribute::None
        }
    }

    pub(super) fn sequence_hold(self) -> bool {
        self.contains(Attribute::HoldAllComplete) || self.contains(Attribute::SequenceHold)
    }

    pub(super) fn blocks_evaluate_override(self) -> bool {
        self.contains(Attribute::HoldAllComplete)
    }

    pub(super) fn expr(self) -> Expr {
        Expr::call(
            system_expr("List"),
            self.attributes()
                .into_iter()
                .map(Attribute::expr)
                .collect::<Vec<_>>(),
        )
    }
}

pub(super) const ALL_ATTRIBUTES: [Attribute; 16] = [
    Attribute::HoldAllComplete,
    Attribute::HoldAll,
    Attribute::HoldFirst,
    Attribute::HoldRest,
    Attribute::SubValuesHoldAll,
    Attribute::SequenceHold,
    Attribute::Listable,
    Attribute::NHoldAll,
    Attribute::NumericFunction,
    Attribute::Orderless,
    Attribute::Flat,
    Attribute::OneIdentity,
    Attribute::NonThreadable,
    Attribute::Protected,
    Attribute::ReadProtected,
    Attribute::Temporary,
];

pub(super) fn parse_attribute_set_expr(expr: &Expr) -> Option<AttributeSet> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut attributes = AttributeSet::empty();

            for item in args {
                // Preserve the user-written spec for ClearAttributes; storage normalizes later.
                attributes.bits |= Attribute::from_expr(item)?.bit();
            }

            Some(attributes)
        }
        _ => Some(AttributeSet::from_attribute(Attribute::from_expr(expr)?)),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinDispatch {
    None,
    SpecialForm,
    Ordinary,
}
pub type BuiltinFn = fn(&mut Session, &Expr, &SharedArgs) -> Result<Expr, EvalError>;

/// Declares how a builtin handler can retain or return memory beyond its inputs.
///
/// `Preflight` is the attachment point for the projector functions added by the
/// subsequent memory-accounting work. This declaration intentionally has no
/// default: every builtin construction must select a strategy explicitly.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryGrowthContract {
    /// The result is no more than a fixed allowance larger than the arguments.
    NonAmplifying = 0,
    /// A conservative result-size projection can run before construction.
    Preflight = 1,
    /// Retained growth must be charged as data-dependent construction proceeds.
    Incremental = 2,
    /// Growth is owned by another classified evaluator or host boundary.
    Delegated = 3,
}

impl MemoryGrowthContract {
    pub(in crate::runtime) const fn suspended_code(self) -> u8 {
        self as u8
    }

    pub(in crate::runtime) const fn from_suspended_code(code: u8) -> Self {
        match code {
            0 => Self::NonAmplifying,
            1 => Self::Preflight,
            2 => Self::Incremental,
            3 => Self::Delegated,
            _ => panic!("invalid suspended memory-growth contract"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinMessageSpec {
    pub tag: &'static str,
    pub template: &'static str,
}

// Specs compare catalog-origin function pointers; copies come from one catalog entry.
#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub attributes: AttributeSet,
    pub dispatch: BuiltinDispatch,
    pub handler: Option<BuiltinFn>,
    pub capability: Capability,
    pub memory_growth: MemoryGrowthContract,
    pub messages: &'static [BuiltinMessageSpec],
}

impl BuiltinSpec {
    pub const fn new(
        name: &'static str,
        summary: &'static str,
        memory_growth: MemoryGrowthContract,
    ) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::empty(),
            dispatch: BuiltinDispatch::None,
            handler: None,
            capability: Capability::Pure,
            memory_growth,
            messages: &[],
        }
    }

    pub const fn with_hold(
        name: &'static str,
        summary: &'static str,
        hold: HoldAttribute,
        memory_growth: MemoryGrowthContract,
    ) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::from_hold(hold),
            dispatch: BuiltinDispatch::None,
            handler: None,
            capability: Capability::Pure,
            memory_growth,
            messages: &[],
        }
    }

    pub const fn with_attributes(
        name: &'static str,
        summary: &'static str,
        attributes: AttributeSet,
        memory_growth: MemoryGrowthContract,
    ) -> Self {
        Self {
            name,
            summary,
            attributes,
            dispatch: BuiltinDispatch::None,
            handler: None,
            capability: Capability::Pure,
            memory_growth,
            messages: &[],
        }
    }

    pub const fn special(
        name: &'static str,
        summary: &'static str,
        handler: BuiltinFn,
        memory_growth: MemoryGrowthContract,
    ) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::empty(),
            dispatch: BuiltinDispatch::SpecialForm,
            handler: Some(handler),
            capability: Capability::Pure,
            memory_growth,
            messages: &[],
        }
    }

    pub const fn special_with_hold(
        name: &'static str,
        summary: &'static str,
        hold: HoldAttribute,
        handler: BuiltinFn,
        memory_growth: MemoryGrowthContract,
    ) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::from_hold(hold),
            dispatch: BuiltinDispatch::SpecialForm,
            handler: Some(handler),
            capability: Capability::Pure,
            memory_growth,
            messages: &[],
        }
    }

    pub const fn special_with_attributes(
        name: &'static str,
        summary: &'static str,
        attributes: AttributeSet,
        handler: BuiltinFn,
        memory_growth: MemoryGrowthContract,
    ) -> Self {
        Self {
            name,
            summary,
            attributes,
            dispatch: BuiltinDispatch::SpecialForm,
            handler: Some(handler),
            capability: Capability::Pure,
            memory_growth,
            messages: &[],
        }
    }

    pub const fn ordinary(
        name: &'static str,
        summary: &'static str,
        handler: BuiltinFn,
        memory_growth: MemoryGrowthContract,
    ) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::empty(),
            dispatch: BuiltinDispatch::Ordinary,
            handler: Some(handler),
            capability: Capability::Pure,
            memory_growth,
            messages: &[],
        }
    }

    pub const fn ordinary_with_hold(
        name: &'static str,
        summary: &'static str,
        hold: HoldAttribute,
        handler: BuiltinFn,
        memory_growth: MemoryGrowthContract,
    ) -> Self {
        Self {
            name,
            summary,
            attributes: AttributeSet::from_hold(hold),
            dispatch: BuiltinDispatch::Ordinary,
            handler: Some(handler),
            capability: Capability::Pure,
            memory_growth,
            messages: &[],
        }
    }

    pub const fn ordinary_with_attributes(
        name: &'static str,
        summary: &'static str,
        attributes: AttributeSet,
        handler: BuiltinFn,
        memory_growth: MemoryGrowthContract,
    ) -> Self {
        Self {
            name,
            summary,
            attributes,
            dispatch: BuiltinDispatch::Ordinary,
            handler: Some(handler),
            capability: Capability::Pure,
            memory_growth,
            messages: &[],
        }
    }

    pub const fn protected(self) -> Self {
        let mut spec = self;
        spec.attributes = spec.attributes.with(Attribute::Protected);
        spec
    }

    pub const fn with_capability(mut self, capability: Capability) -> Self {
        self.capability = capability;
        self
    }

    pub const fn with_messages(mut self, messages: &'static [BuiltinMessageSpec]) -> Self {
        self.messages = messages;
        self
    }

    pub(super) fn with_fallback_dispatch(mut self, fallback: Self) -> Self {
        if self.dispatch == BuiltinDispatch::None {
            self.dispatch = fallback.dispatch;
        }

        if self.handler.is_none() {
            self.handler = fallback.handler;
            self.capability = fallback.capability;
            self.memory_growth = fallback.memory_growth;
        }

        if self.messages.is_empty() {
            self.messages = fallback.messages;
        }

        self
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<AttributeSet>();
};
