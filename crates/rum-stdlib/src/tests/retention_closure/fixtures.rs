//! These modules are compiled by Rust even though the six bad roots never run.

#[allow(dead_code)]
#[path = "fixtures/accepted.rs"]
pub(super) mod accepted;
#[allow(dead_code)]
#[path = "fixtures/aliased_collect.rs"]
mod aliased_collect;
#[allow(dead_code)]
#[path = "fixtures/bare_vec_loop.rs"]
mod bare_vec_loop;
#[allow(dead_code)]
#[path = "fixtures/constructors.rs"]
mod constructors;
#[allow(dead_code)]
#[path = "fixtures/extracted_allocator.rs"]
mod extracted_allocator;
#[allow(dead_code)]
#[path = "fixtures/formatted_string.rs"]
mod formatted_string;
#[allow(dead_code)]
#[path = "fixtures/map_entry_callback.rs"]
mod map_entry_callback;
#[allow(dead_code)]
#[path = "fixtures/methods.rs"]
mod methods;
#[allow(dead_code)]
#[path = "fixtures/unleased_callback_result.rs"]
mod unleased_callback_result;

use super::{model::Source, source};

#[path = "fixtures/imports.rs"]
mod imports;

pub(super) const FILES: &[(&str, &str, &str)] = &[
    (
        "methods",
        "crates/rum-stdlib/src/tests/retention_closure/fixtures/methods.rs",
        include_str!("fixtures/methods.rs"),
    ),
    (
        "constructors",
        "crates/rum-stdlib/src/tests/retention_closure/fixtures/constructors.rs",
        include_str!("fixtures/constructors.rs"),
    ),
    (
        "accepted",
        "crates/rum-stdlib/src/tests/retention_closure/fixtures/accepted.rs",
        include_str!("fixtures/accepted.rs"),
    ),
    (
        "aliased_collect",
        "crates/rum-stdlib/src/tests/retention_closure/fixtures/aliased_collect.rs",
        include_str!("fixtures/aliased_collect.rs"),
    ),
    (
        "bare_vec_loop",
        "crates/rum-stdlib/src/tests/retention_closure/fixtures/bare_vec_loop.rs",
        include_str!("fixtures/bare_vec_loop.rs"),
    ),
    (
        "extracted_allocator",
        "crates/rum-stdlib/src/tests/retention_closure/fixtures/extracted_allocator.rs",
        include_str!("fixtures/extracted_allocator.rs"),
    ),
    (
        "formatted_string",
        "crates/rum-stdlib/src/tests/retention_closure/fixtures/formatted_string.rs",
        include_str!("fixtures/formatted_string.rs"),
    ),
    (
        "map_entry_callback",
        "crates/rum-stdlib/src/tests/retention_closure/fixtures/map_entry_callback.rs",
        include_str!("fixtures/map_entry_callback.rs"),
    ),
    (
        "unleased_callback_result",
        "crates/rum-stdlib/src/tests/retention_closure/fixtures/unleased_callback_result.rs",
        include_str!("fixtures/unleased_callback_result.rs"),
    ),
];

pub(super) fn program() -> Source {
    source::fixture(FILES)
}

use super::model::{Diagnostic, Kind, Site};

pub(super) const SIX: &[&str] = &[
    "bare_vec_loop",
    "aliased_collect",
    "formatted_string",
    "map_entry_callback",
    "extracted_allocator",
    "unleased_callback_result",
];

pub(super) fn expected(name: &str) -> Diagnostic {
    let (kind, start, end, target, reason) = match name {
        "bare_vec_loop" => (
            Kind::RET001,
            (2, 22),
            (2, 41),
            Some("std::vec::Vec::new"),
            "unleased owned payload",
        ),
        "aliased_collect" => (
            Kind::RET002,
            (4, 31),
            (4, 52),
            Some("std::iter::Iterator::collect"),
            "not a resolved function definition: std::iter::Iterator::collect",
        ),
        "formatted_string" => (
            Kind::RET003,
            (5, 17),
            (5, 31),
            None,
            "unsupported macro expansion",
        ),
        "map_entry_callback" => (
            Kind::RET002,
            (2, 5),
            (2, 38),
            None,
            "method dispatch requires autoderef/autoref/trait resolution",
        ),
        "extracted_allocator" => (
            Kind::RET001,
            (8, 19),
            (8, 38),
            Some("std::vec::Vec::new"),
            "unleased owned payload",
        ),
        "unleased_callback_result" => (
            Kind::RET004,
            (2, 16),
            (2, 55),
            Some("fixture::accepted::substrate::callback"),
            "callback payload has no retention transfer",
        ),
        _ => panic!("not one of the six falsifiers"),
    };
    let mut witness = vec![format!("fixture::{name}::root")];
    if name == "extracted_allocator" {
        witness.push("fixture::extracted_allocator::allocate".into());
    }
    Diagnostic {
        kind,
        site: Site {
            file: format!("crates/rum-stdlib/src/tests/retention_closure/fixtures/{name}.rs"),
            start,
            end,
        },
        target: target.map(str::to_owned),
        witness,
        reason: reason.into(),
    }
}
