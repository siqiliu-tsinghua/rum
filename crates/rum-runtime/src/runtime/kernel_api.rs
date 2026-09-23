//! Kernel API contract between the evaluator kernel and builtin families.
//!
//! Builtin families import only from this module, plus `rum_core`,
//! `rum_syntax`, and standard library modules. This surface grows one family
//! conversion at a time. Items exported here become semver-relevant when the
//! crate split ships.

// Explicit accounting route; only the evaluation-stack owner is maintained.
pub use super::accounting_mode::{
    AccountingBuiltinFn, AccountingContext, AccountingShadow, AccountingSnapshot,
};

// accumulator links (stdlib-only bridge)
#[doc(hidden)]
pub use super::accumulator_chain::{AccumulatorChain, AccumulatorChainNode};
#[doc(hidden)]
pub use rum_derive::AccumulatorChainNode;

// capabilities
pub use super::capabilities::Capability;

// builtin_catalog
pub use super::builtin_catalog::{BuiltinAttributeSpec, stdlib_builtin_attribute_specs};

// control_data
pub use super::kernel_data::control_data::ControlDataBuiltin;

// assignment Part selectors
#[doc(hidden)]
pub use super::kernel_data::evaluated_part_selectors::{
    EvaluatedPartSelectors, EvaluatedPartSelectorsContinuation, EvaluatedPartSelectorsEvaluation,
};

// rewrite_data
pub use super::kernel_data::rewrite_data::RewriteDataBuiltin;

// arithmetic
pub use super::arithmetic::{ArithmeticDataBuiltin, expr_as_exact_integer, expr_as_exact_rational};

// association_data
pub use super::kernel_data::association_data::{
    AssociationEntryForm, association_entries_from_expr, association_entry_forms_from_expr,
    is_missing_expr, list_items, missing_key_absent_expr, parse_association_entry_form,
    render_association, render_association_entry_forms_preserving_heads,
};

// expr_helpers
pub use super::expr_helpers::{
    expr_as_non_negative_usize, expr_as_signed_isize, expr_as_string, expr_as_string_list,
    expr_as_symbol, is_negative_one_expr, system_expr,
};

// expr_runtime_helpers
pub use super::expr_runtime_helpers::{
    base64_encode, call_items_with_head, compare_expr_canonical, is_system_symbol_name,
};

// file_io
pub use super::file_io::{FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode};

// entropy/random
pub use super::random::{EntropyError, EntropyProvider, EntropySource};

// host payload destruction
pub use super::host_payload::BoundedDestruction;

// print sinks
pub use super::print_sink::{ClosurePrintSink, PrintSink};

// definition rhs guards (stdlib-only bridge)
#[doc(hidden)]
pub use super::definitions::rhs_guards::{
    evaluate_definition_rhs_guards_for_match, split_definition_rhs_guards,
};
#[doc(hidden)]
pub use super::definitions::{PreparedDefinitionLhs, PreparedDefinitionLhsContinuation};

// lexical_substitution
pub use super::substitution::lexical_substitution::substitute_lexical_bindings;

// message_selector_parsing
pub use super::messages::message_selector_parsing::truthiness_symbol;

// metadata
pub use super::metadata::{
    Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec, HoldAttribute, MemoryGrowthContract,
};

// numeric traits
pub use num_integer::{Integer, Roots};
pub use num_traits::{One, Signed, ToPrimitive, Zero};

// model
pub use super::model::{
    CancellationObserver, CasesLevelSpec, CasesTraversalConfig, RewriteRule, RuleApplicationSource,
    Session, SymbolExprMap,
};
#[doc(hidden)]
pub use super::session::TrustedSystemDownValueLhs;

// native_stack_guard (stdlib-only bridge)
#[doc(hidden)]
pub use super::native_stack_guard::{NativeStackDepth, run_with_native_stack_guard};

// parse_and_message_helpers
pub use super::messages::parse_and_message_helpers::{
    HeadsOptionClassification, classify_heads_option_expr, parse_heads_option,
    parse_heads_option_with_default, parse_standard_level_spec,
};

// session_state
pub use super::session::session_state::{EvalError, ThreadArgSpec, ThreadingDispatch};

// structure_data
#[doc(hidden)]
pub use super::kernel_data::structure_data::invalid_part_selector_expr;
pub use super::kernel_data::structure_data::{
    exact_expr_at_path, exact_position_paths_from_expr,
    exact_position_paths_from_part_selector_exprs, replace_exact_path,
};

// tree_walkers
pub use super::tree_walkers::collect_level_parts_owned;

// suspension/core (stdlib-only bridge)
#[doc(hidden)]
pub use super::suspension::{
    AbandonmentContext, ContinuationValue, FrameAbandon, FrameAbandonAction, FrameAction,
    PureFrameFinalizer, SuspensionFrame,
};
#[doc(hidden)]
pub use crate::resume_or_suspend;

// suspension/trace (stdlib-only bridge)
pub use super::options::OptionReentry;
#[doc(hidden)]
pub use super::suspension::trace::{FrameRootTrace, FrameRootVisitor};
#[doc(hidden)]
pub use rum_derive::FrameRootTrace;
#[doc(hidden)]
pub use rum_derive::PureFrameAbandon;

// text_rendering
pub use super::kernel_data::text_rendering::{
    ToStringFormat, render_input_like_expr, render_to_string_expr,
};
