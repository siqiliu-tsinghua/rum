//! Test-only declarations for the association depth class.
//!
//! This stays separate from `BuiltinSpec`, so it adds no dispatch field,
//! normal-build allocation, or other default-path work.

use super::DeclaredRecursiveEdge;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mechanism {
    NoProportionalDescent,
    BoundedHeap,
    GuardedNative,
    UnguardedNative,
    EvaluatorPolled,
}

const MECHANISM_VOCABULARY: &[Mechanism] = &[
    Mechanism::NoProportionalDescent,
    Mechanism::BoundedHeap,
    Mechanism::GuardedNative,
    Mechanism::UnguardedNative,
    Mechanism::EvaluatorPolled,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContractStatus {
    MeetsContract,
    ShortOfContract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FailureDirection {
    FailsOpen,
    FailsClosed,
    Unmeasured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AssociationDepthRow {
    surface: &'static str,
    catalog_name: Option<&'static str>,
    mechanism: Mechanism,
    contract_status: ContractStatus,
    failure_direction: Option<FailureDirection>,
    owed_by: Option<&'static str>,
    recursion_declarations: &'static [&'static str],
}

const NO_RECURSION: &[&str] = &[];

const CROSS_FAMILY_AXES: &[&str] = &[
    "Apply (association axis)",
    "Map (association axis)",
    "MapIndexed (association axis)",
    "Part (keyed association axis)",
    "Scan (association axis)",
    "Set (nested association key-path axis)",
];

const ASSOCIATION_DEPTH_REGISTRY: &[AssociationDepthRow] = &[
    AssociationDepthRow {
        surface: "Apply (association axis)",
        catalog_name: None,
        mechanism: Mechanism::BoundedHeap,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::Unmeasured),
        owed_by: None,
        recursion_declarations: &["apply-bounded-front"],
    },
    AssociationDepthRow {
        surface: "AssociateTo",
        catalog_name: Some("AssociateTo"),
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::FailsOpen),
        owed_by: Some("association Part-target conversion queue"),
        recursion_declarations: &["held-association-part-exact-paths"],
    },
    AssociationDepthRow {
        surface: "Association",
        catalog_name: Some("Association"),
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::Unmeasured),
        owed_by: Some("residual association-owned shapes queue"),
        recursion_declarations: &[
            "runtime-association-deep-parser",
            "stdlib-association-deep-parser",
        ],
    },
    AssociationDepthRow {
        surface: "AssociationKeyDeflatten",
        catalog_name: Some("AssociationKeyDeflatten"),
        mechanism: Mechanism::BoundedHeap,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::Unmeasured),
        owed_by: None,
        recursion_declarations: &["association-key-deflatten"],
    },
    AssociationDepthRow {
        surface: "AssociationKeyFlatten",
        catalog_name: Some("AssociationKeyFlatten"),
        mechanism: Mechanism::BoundedHeap,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::Unmeasured),
        owed_by: None,
        recursion_declarations: &["association-key-flatten"],
    },
    AssociationDepthRow {
        surface: "AssociationMap",
        catalog_name: Some("AssociationMap"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "AssociationQ",
        catalog_name: Some("AssociationQ"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "AssociationThread",
        catalog_name: Some("AssociationThread"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "Catenate",
        catalog_name: Some("Catenate"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "Counts",
        catalog_name: Some("Counts"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "CountsBy",
        catalog_name: Some("CountsBy"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "DeleteMissing",
        catalog_name: Some("DeleteMissing"),
        mechanism: Mechanism::BoundedHeap,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::Unmeasured),
        owed_by: None,
        recursion_declarations: &[
            "delete-missing-bounded-front",
            "delete-missing-target-bounded-front",
        ],
    },
    AssociationDepthRow {
        surface: "GroupBy",
        catalog_name: Some("GroupBy"),
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::ShortOfContract,
        failure_direction: Some(FailureDirection::FailsOpen),
        owed_by: Some("multi-spec GroupBy queue"),
        recursion_declarations: &["groupby-multi-spec"],
    },
    AssociationDepthRow {
        surface: "JoinAcross",
        catalog_name: Some("JoinAcross"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "Key",
        catalog_name: Some("Key"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeyComplement",
        catalog_name: Some("KeyComplement"),
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::Unmeasured),
        owed_by: Some("residual association-owned shapes queue"),
        recursion_declarations: &["key-complement-nested-options"],
    },
    AssociationDepthRow {
        surface: "KeyDrop",
        catalog_name: Some("KeyDrop"),
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::ShortOfContract,
        failure_direction: Some(FailureDirection::FailsOpen),
        owed_by: Some("deferred collection-shape queue"),
        recursion_declarations: &["key-take-drop-nested-lists"],
    },
    AssociationDepthRow {
        surface: "KeyDropFrom",
        catalog_name: Some("KeyDropFrom"),
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::FailsOpen),
        owed_by: Some("association Part-target conversion queue"),
        recursion_declarations: &["held-association-part-exact-paths"],
    },
    AssociationDepthRow {
        surface: "KeyExistsQ",
        catalog_name: Some("KeyExistsQ"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeyFreeQ",
        catalog_name: Some("KeyFreeQ"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeyGroupBy",
        catalog_name: Some("KeyGroupBy"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeyIntersection",
        catalog_name: Some("KeyIntersection"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeyMap",
        catalog_name: Some("KeyMap"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeyMemberQ",
        catalog_name: Some("KeyMemberQ"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeySelect",
        catalog_name: Some("KeySelect"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeySort",
        catalog_name: Some("KeySort"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeySortBy",
        catalog_name: Some("KeySortBy"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeyTake",
        catalog_name: Some("KeyTake"),
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::ShortOfContract,
        failure_direction: Some(FailureDirection::FailsOpen),
        owed_by: Some("deferred collection-shape queue"),
        recursion_declarations: &["key-take-drop-nested-lists"],
    },
    AssociationDepthRow {
        surface: "KeyUnion",
        catalog_name: Some("KeyUnion"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeyValueMap",
        catalog_name: Some("KeyValueMap"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "KeyValuePattern",
        catalog_name: Some("KeyValuePattern"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "Keys",
        catalog_name: Some("Keys"),
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::ShortOfContract,
        failure_direction: Some(FailureDirection::FailsOpen),
        owed_by: Some("deferred collection-shape queue"),
        recursion_declarations: &["keys-nested-lists"],
    },
    AssociationDepthRow {
        surface: "Lookup",
        catalog_name: Some("Lookup"),
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::Unmeasured),
        owed_by: Some("residual association-owned shapes queue"),
        recursion_declarations: &["lookup-nested-key-specifications"],
    },
    AssociationDepthRow {
        surface: "Map (association axis)",
        catalog_name: None,
        mechanism: Mechanism::BoundedHeap,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::Unmeasured),
        owed_by: None,
        recursion_declarations: &["map-bounded-front"],
    },
    AssociationDepthRow {
        surface: "MapIndexed (association axis)",
        catalog_name: None,
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::FailsOpen),
        owed_by: Some("association MapIndexed conversion queue"),
        recursion_declarations: &["map-indexed-recursive-walker"],
    },
    AssociationDepthRow {
        surface: "Merge",
        catalog_name: Some("Merge"),
        mechanism: Mechanism::GuardedNative,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::FailsClosed),
        owed_by: None,
        recursion_declarations: &["merge-collection-parser"],
    },
    AssociationDepthRow {
        surface: "MergeNested",
        catalog_name: Some("MergeNested"),
        mechanism: Mechanism::BoundedHeap,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::Unmeasured),
        owed_by: None,
        recursion_declarations: &["merge-nested-bounded-front"],
    },
    AssociationDepthRow {
        surface: "Missing",
        catalog_name: Some("Missing"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "Normal",
        catalog_name: Some("Normal"),
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::Unmeasured),
        owed_by: Some("residual association-owned shapes queue"),
        recursion_declarations: &["normal-wrapper-descent"],
    },
    AssociationDepthRow {
        surface: "Part (keyed association axis)",
        catalog_name: None,
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "PositionIndex",
        catalog_name: Some("PositionIndex"),
        mechanism: Mechanism::NoProportionalDescent,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: None,
        owed_by: None,
        recursion_declarations: NO_RECURSION,
    },
    AssociationDepthRow {
        surface: "Scan (association axis)",
        catalog_name: None,
        mechanism: Mechanism::GuardedNative,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::FailsClosed),
        owed_by: None,
        recursion_declarations: &["scan-association-guarded-native"],
    },
    AssociationDepthRow {
        surface: "Set (nested association key-path axis)",
        catalog_name: None,
        mechanism: Mechanism::GuardedNative,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::FailsClosed),
        owed_by: None,
        recursion_declarations: &["set-nested-key-path-guarded-native"],
    },
    AssociationDepthRow {
        surface: "Values",
        catalog_name: Some("Values"),
        mechanism: Mechanism::UnguardedNative,
        contract_status: ContractStatus::MeetsContract,
        failure_direction: Some(FailureDirection::Unmeasured),
        owed_by: Some("residual association-owned shapes queue"),
        recursion_declarations: &["values-nested-lists"],
    },
];

fn validate_registry(
    rows: &[AssociationDepthRow],
    recursion: &[DeclaredRecursiveEdge],
) -> Result<(), String> {
    if !rows
        .windows(2)
        .all(|entries| entries[0].surface < entries[1].surface)
    {
        return Err("the association depth registry must stay sorted and unique".to_string());
    }

    for row in rows {
        match row.mechanism {
            Mechanism::NoProportionalDescent => {
                if row.failure_direction.is_some() {
                    return Err(format!(
                        "{} declares NoProportionalDescent but also has a failure direction",
                        row.surface
                    ));
                }
                if let Some(edge) = recursion
                    .iter()
                    .find(|edge| row.recursion_declarations.contains(&edge.declaration))
                {
                    return Err(format!(
                        "{} declares NoProportionalDescent but recursive edge ({:?}, {:?}, {:?}, {}) implements {:?}",
                        row.surface,
                        edge.edge.path,
                        edge.edge.caller,
                        edge.edge.callee,
                        edge.edge.count,
                        edge.declaration
                    ));
                }
            }
            _ if row.failure_direction.is_none() => {
                return Err(format!(
                    "{} must record its failure direction above the ceiling",
                    row.surface
                ));
            }
            _ => {}
        }

        if matches!(row.contract_status, ContractStatus::ShortOfContract) && row.owed_by.is_none() {
            return Err(format!(
                "{} is short of the contract but has no owning brief or queue item",
                row.surface
            ));
        }
        if matches!(row.mechanism, Mechanism::UnguardedNative) && row.owed_by.is_none() {
            return Err(format!(
                "{} is unguarded but has no owning brief or queue item",
                row.surface
            ));
        }
    }

    let declared = rows
        .iter()
        .flat_map(|row| row.recursion_declarations.iter().copied())
        .collect::<std::collections::BTreeSet<_>>();
    if let Some(edge) = recursion
        .iter()
        .find(|edge| !declared.contains(edge.declaration))
    {
        return Err(format!(
            "recursive edge ({:?}, {:?}, {:?}, {}) names absent declaration {:?}",
            edge.edge.path, edge.edge.caller, edge.edge.callee, edge.edge.count, edge.declaration
        ));
    }

    Ok(())
}

#[test]
fn association_depth_registry_is_sorted_and_exhaustive() {
    assert_eq!(MECHANISM_VOCABULARY.len(), 5);

    let mut catalog_names = crate::catalog::ASSOCIATION_CORE_BUILTINS
        .iter()
        .chain(crate::catalog::ASSOCIATION_TRANSFORM_BUILTINS)
        .map(|spec| spec.name)
        .collect::<Vec<_>>();
    catalog_names.sort_unstable();

    let registry_catalog_names = ASSOCIATION_DEPTH_REGISTRY
        .iter()
        .filter_map(|row| row.catalog_name)
        .collect::<Vec<_>>();
    let registry_cross_family_axes = ASSOCIATION_DEPTH_REGISTRY
        .iter()
        .filter(|row| row.catalog_name.is_none())
        .map(|row| row.surface)
        .collect::<Vec<_>>();

    let catalog_set = catalog_names
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let registry_set = registry_catalog_names
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let missing = catalog_set
        .difference(&registry_set)
        .copied()
        .collect::<Vec<_>>();
    let extra = registry_set
        .difference(&catalog_set)
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty()
            && extra.is_empty()
            && catalog_names.len() == registry_catalog_names.len(),
        "association depth registry/catalog mismatch; catalog entries without rows: {missing:?}; rows without catalog entries: {extra:?}"
    );
    assert_eq!(registry_cross_family_axes, CROSS_FAMILY_AXES);
    validate_registry(
        ASSOCIATION_DEPTH_REGISTRY,
        &super::recursion_inventory::declared_production_recursion(),
    )
    .unwrap_or_else(|error| panic!("{error}"));
}

#[test]
#[should_panic(
    expected = "AssociationKeyFlatten declares NoProportionalDescent but recursive edge (\"crates/rum-stdlib/src/associations/key_reshape/traversal.rs\", \"flatten_association_entries_with_front_depth\", \"flatten_association_entries_with_front_depth\", 1)"
)]
fn real_catalog_misdeclaration_names_association_key_flatten_recursive_edge() {
    let mut rows = ASSOCIATION_DEPTH_REGISTRY.to_vec();
    let row = rows
        .iter_mut()
        .find(|row| row.catalog_name == Some("AssociationKeyFlatten"))
        .expect("AssociationKeyFlatten should have a registry row");
    row.mechanism = Mechanism::NoProportionalDescent;
    row.failure_direction = None;

    validate_registry(
        &rows,
        &super::recursion_inventory::declared_production_recursion(),
    )
    .unwrap_or_else(|error| panic!("{error}"));
}
