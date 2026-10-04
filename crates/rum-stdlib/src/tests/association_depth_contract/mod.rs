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
