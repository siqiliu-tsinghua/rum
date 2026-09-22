//! Standard library for `rum` sessions.
//!
//! This crate owns builtin family implementations and catalog specs, and
//! installs the core builtin set into runtime sessions through [`install_core`].

use rum_core::Expr;
use rum_runtime::kernel_api::TrustedSystemDownValueLhs;
use rum_runtime::{Session, core_builtin_specs};

mod adaptive_set;
mod arithmetic;
mod associations;
mod bytes;
mod calculus;
mod catalog;
mod character_classes;
mod combinatorics;
mod control_flow;
mod csv;
mod datetime;
mod functional_ops;
mod ini;
mod integer_digits;
mod json;
mod lazy;
mod logic;
mod number_theory;
mod polynomials;
mod predicates;
mod random;
mod rewrite_builtins;
mod schema;
mod strings;
mod structure_ops;
mod toml;
mod xml;
mod yaml;

#[doc(hidden)]
pub mod docgen;

pub fn install_core(session: &mut Session) {
    for spec in core_builtin_specs() {
        session.register_builtin(*spec);
    }
    for spec in catalog::ARITHMETIC_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::LOGIC_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::PREDICATE_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::ASSOCIATION_CORE_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::ASSOCIATION_TRANSFORM_BUILTINS {
        session.register_builtin(*spec);
    }
    #[cfg(feature = "hash")]
    for spec in catalog::DIGEST_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::BYTE_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::CHARACTER_CLASS_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::COMBINATORICS_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::CONTROL_BUILTINS {
        if spec.name == "CompoundExpression" {
            session.register_accounting_builtin(
                *spec,
                control_flow::eval_accounted_compound_expression,
            );
        } else {
            session.register_builtin(*spec);
        }
    }
    for spec in catalog::CSV_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::DATETIME_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::FUNCTIONAL_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::INI_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::INTEGER_DIGITS_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::JSON_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::LAZY_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::NUMBER_THEORY_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::CALCULUS_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::POLYNOMIAL_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::RANDOM_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::REWRITE_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::SCHEMA_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::STRING_CORE_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::STRING_TEXT_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::STRING_MATCH_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::STRUCTURE_CORE_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::STRUCTURE_ORDERING_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::TOML_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::XML_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::YAML_BUILTINS {
        session.register_builtin(*spec);
    }
    character_classes::install(session);

    session.install_system_down_value(
        TrustedSystemDownValueLhs::default_for_plus(),
        Expr::small_integer(0),
    );
    session.install_system_down_value(
        TrustedSystemDownValueLhs::default_for_times(),
        Expr::small_integer(1),
    );
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
