use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::fmt;
use std::path::{Path, PathBuf};

use rum_core::{
    Atom, BigInt, Expr, GLOBAL_CONTEXT, Rational, SYSTEM_CONTEXT, SharedArgs, SharedExpr, Symbol,
    WeakSharedArgs, heads,
};
use rum_syntax::{
    BinaryOp, BlankKind, ParsedSource, PostfixOp, PrefixOp, SlotSpec, StructuredSourceDiagnostic,
    SyntaxError, SyntaxExpr, SyntaxExprKind, parse_expr, parse_with_diagnostics,
};

mod accounting_mode;
mod accumulator_chain;
mod arithmetic;
mod attributes;
mod builtin_adapters;
mod builtin_catalog;
mod builtins_context_loader;
mod builtins_input;
mod call_preparation;
mod capabilities;
mod context_and_load_helpers;
mod debug_tools;
mod definitions;
mod eval_call_prepare;
mod eval_core;
mod eval_dispatch;
mod evaluation_cache;
mod expr_helpers;
mod expr_runtime_helpers;
mod file_io;
mod hold_thread;
pub mod kernel_api;
mod kernel_data;
mod listable_dispatch;
mod lowering;
mod matcher_and_symbols;
mod messages;
mod metadata;
mod model;
mod namespace_tools;
mod native_stack_guard;
mod options;
mod package_tools;
mod pattern_matching;
#[cfg(test)]
mod pattern_matching_interpreted;
mod pattern_syntax;
mod print_sink;
mod profiling;
mod random;
mod resolution;
mod scoping;
mod session;
mod substitution;
mod suspension;
#[cfg(test)]
mod test_builtins;
mod tree_walkers;

#[allow(unused_imports)]
use self::{
    arithmetic::*, attributes::*, builtin_adapters::*, builtin_catalog::*,
    builtins_context_loader::*, builtins_input::*, call_preparation::*, capabilities::*,
    context_and_load_helpers::*, debug_tools::*, definitions::*, eval_call_prepare::*,
    eval_core::*, eval_dispatch::*, evaluation_cache::*, expr_helpers::*, expr_runtime_helpers::*,
    file_io::*, hold_thread::*, kernel_data::*, listable_dispatch::*, lowering::*,
    matcher_and_symbols::*, messages::*, metadata::*, model::*, namespace_tools::*, options::*,
    package_tools::*, pattern_matching::*, pattern_syntax::*, profiling::*, random::*,
    resolution::*, scoping::*, session::*, substitution::*, suspension::*, tree_walkers::*,
};
pub use self::{
    builtin_catalog::core_builtin_specs, capabilities::*, file_io::*, lowering::lower_syntax_expr,
    metadata::*, model::*, print_sink::*, random::*, session::session_state::EvalError,
};

impl Session {
    pub(in crate::runtime) fn eval_missing_q(
        &mut self,
        _head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let [expr] = args.as_ref() else {
            return Ok(self.invalid_call_with_exact_arg_count("MissingQ", args, 1));
        };

        Ok(system_expr(
            if matches!(
                expr,
                Expr::Call { head, .. } if is_system_symbol_name(head, "Missing")
            ) {
                heads::structure::TRUE
            } else {
                heads::structure::FALSE
            },
        ))
    }
}

mod host_payload;

#[cfg(test)]
mod tests;
