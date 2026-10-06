mod recursion_inventory;
mod registry;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RecursiveEdge {
    path: String,
    caller: String,
    callee: String,
    count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeclaredRecursiveEdge {
    edge: RecursiveEdge,
    declaration: &'static str,
}

mod lazy_depth_contract;
mod rule_pattern_depth_contract;
mod schema_depth_contract;

mod g7_continuation_contract;
