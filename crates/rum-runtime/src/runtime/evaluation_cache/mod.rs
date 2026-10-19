use super::*;

// This brief deliberately lands these tested primitives one step before their
// first production caller.
#[cfg_attr(not(test), allow(dead_code))]
mod mutation_proofs;
mod profile;
pub(in crate::runtime) mod stability;

#[cfg(test)]
mod tests;

pub(super) use profile::{
    evaluation_cache_profile_source_timing_start, evaluation_profile_current_source_symbol,
    evaluation_profile_definition_source, print_evaluation_cache_profile_snapshot,
    record_profile_definition_rhs_eval_by_source, record_profile_eval_module_by_source,
    record_profile_module_body_substitution_by_source,
    record_profile_module_init_substitution_by_source, reset_evaluation_cache_profile_counters,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum CallStabilityHeadShape {
    AtomSymbol(Symbol),
    AtomOther,
    CallHeadSymbol(Symbol),
    CallOther,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct CallStabilitySourceHeadKey {
    source: Symbol,
    head: CallStabilityHeadShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CallStabilityProfileOutcome {
    Hit,
    MissAbsent,
    MissGeneration,
    MissDirty,
    MissPointer,
}

fn symbol_family_name(symbol: &Symbol) -> String {
    let name = symbol.name();
    let base_name = name
        .rsplit_once('$')
        .filter(|(_, suffix)| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        })
        .map(|(base_name, _)| base_name)
        .unwrap_or(name);
    format!("{}{base_name}", symbol.context())
}

fn symbol_has_generated_dollar_name(symbol: &Symbol) -> bool {
    let Some((_, suffix)) = symbol.name().rsplit_once('$') else {
        return false;
    };
    !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(any(test, feature = "test-continuation-slots"))]
pub(crate) use profile::observe_logical_profile;
