use std::collections::HashMap;

use super::*;

const NESTED_PATH_INDEX_THRESHOLD: usize = 16;
const RECURSIVE_INSERT_PATH_DEPTH: usize = 16;
const RECURSIVE_RENDER_NESTED_PATH_DEPTH: usize = 16;
const RECURSIVE_FLATTEN_ASSOCIATION_DEPTH: usize = 16;

enum BoundedTraversal<T> {
    Complete(T),
    Exhausted(T),
}

impl<T> BoundedTraversal<T> {
    fn into_value(self) -> T {
        match self {
            Self::Complete(value) | Self::Exhausted(value) => value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum NestedPathNode {
    Branch(NestedPathNodes),
    Leaf { value: Expr, delayed: bool },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NestedPathNodes {
    entries: Vec<(Expr, NestedPathNode)>,
    positions: Option<HashMap<Expr, usize>>,
}

impl NestedPathNodes {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            positions: None,
        }
    }

    fn position(&self, key: &Expr) -> Option<usize> {
        self.positions.as_ref().map_or_else(
            || {
                self.entries
                    .iter()
                    .position(|(existing, _)| existing == key)
            },
            |positions| positions.get(key).copied(),
        )
    }

    fn get_mut(&mut self, key: &Expr) -> Option<&mut NestedPathNode> {
        let position = self.position(key)?;
        Some(&mut self.entries[position].1)
    }

    fn push(&mut self, key: Expr, node: NestedPathNode) {
        let position = self.entries.len();
        if let Some(positions) = self.positions.as_mut() {
            positions.insert(key.clone(), position);
        }
        self.entries.push((key, node));
        if self.positions.is_none() && self.entries.len() > NESTED_PATH_INDEX_THRESHOLD {
            self.positions = Some(
                self.entries
                    .iter()
                    .enumerate()
                    .map(|(index, (key, _))| (key.clone(), index))
                    .collect(),
            );
        }
    }
}

pub(super) fn association_key_deflatten_entries(
    entries: Vec<AssociationEntryForm>,
) -> Option<Expr> {
    let mut nested = NestedPathNodes::new();

    for entry in entries {
        let Some(path) = path_from_key(entry.key) else {
            drop_nested_path_nodes_iterative(nested);
            return None;
        };
        if insert_path(&mut nested, &path, entry.value, entry.delayed).is_none() {
            drop_nested_path_nodes_iterative(nested);
            return None;
        }
    }

    Some(render_nested_path_entries(nested))
}

fn path_from_key(key: Expr) -> Option<Vec<Expr>> {
    match list_items(&key) {
        Some(items) if items.is_empty() => None,
        Some(items) => Some(items.to_vec()),
        None => Some(vec![key]),
    }
}

fn insert_path(
    nodes: &mut NestedPathNodes,
    path: &[Expr],
    value: Expr,
    delayed: bool,
) -> Option<()> {
    insert_path_with_front_depth(nodes, path, value, delayed, RECURSIVE_INSERT_PATH_DEPTH)
        .into_value()
}

fn insert_path_with_front_depth(
    nodes: &mut NestedPathNodes,
    path: &[Expr],
    value: Expr,
    delayed: bool,
    remaining_path_depth: usize,
) -> BoundedTraversal<Option<()>> {
    if remaining_path_depth == 0 {
        return BoundedTraversal::Exhausted(insert_path_iterative(nodes, path, value, delayed));
    }

    let (key, remaining) = match path.split_first() {
        Some(path) => path,
        None => return BoundedTraversal::Complete(None),
    };

    if remaining.is_empty() {
        if let Some(node) = nodes.get_mut(key) {
            if matches!(node, NestedPathNode::Branch(_)) {
                return BoundedTraversal::Complete(None);
            }
            *node = NestedPathNode::Leaf { value, delayed };
        } else {
            nodes.push(key.clone(), NestedPathNode::Leaf { value, delayed });
        }
        return BoundedTraversal::Complete(Some(()));
    }

    let child_path_depth = remaining_path_depth - 1;
    if let Some(node) = nodes.get_mut(key) {
        let NestedPathNode::Branch(children) = node else {
            return BoundedTraversal::Complete(None);
        };
        return insert_path_with_front_depth(children, remaining, value, delayed, child_path_depth);
    }

    let mut children = NestedPathNodes::new();
    match insert_path_with_front_depth(&mut children, remaining, value, delayed, child_path_depth) {
        BoundedTraversal::Complete(Some(())) => {
            nodes.push(key.clone(), NestedPathNode::Branch(children));
            BoundedTraversal::Complete(Some(()))
        }
        BoundedTraversal::Exhausted(Some(())) => {
            nodes.push(key.clone(), NestedPathNode::Branch(children));
            BoundedTraversal::Exhausted(Some(()))
        }
        BoundedTraversal::Complete(None) => BoundedTraversal::Complete(None),
        BoundedTraversal::Exhausted(None) => BoundedTraversal::Exhausted(None),
    }
}

#[cold]
#[inline(never)]
fn insert_path_iterative(
    nodes: &mut NestedPathNodes,
    path: &[Expr],
    value: Expr,
    delayed: bool,
) -> Option<()> {
    let mut current = nodes;
    let mut index = 0;

    loop {
        let key = path.get(index)?;
        if index + 1 == path.len() {
            if let Some(position) = current.position(key) {
                let node = &mut current.entries[position].1;
                if matches!(node, NestedPathNode::Branch(_)) {
                    return None;
                }
                *node = NestedPathNode::Leaf { value, delayed };
            } else {
                current.push(key.clone(), NestedPathNode::Leaf { value, delayed });
            }
            return Some(());
        }

        let Some(position) = current.position(key) else {
            current.push(
                key.clone(),
                nested_path_node_for_missing_suffix(&path[index + 1..], value, delayed),
            );
            return Some(());
        };
        let NestedPathNode::Branch(children) = &mut current.entries[position].1 else {
            return None;
        };
        current = children;
        index += 1;
    }
}

fn nested_path_node_for_missing_suffix(
    remaining: &[Expr],
    value: Expr,
    delayed: bool,
) -> NestedPathNode {
    let mut node = NestedPathNode::Leaf { value, delayed };
    for key in remaining.iter().rev() {
        let mut children = NestedPathNodes::new();
        children.push(key.clone(), node);
        node = NestedPathNode::Branch(children);
    }
    node
}

#[cold]
#[inline(never)]
fn drop_nested_path_nodes_iterative(mut nodes: NestedPathNodes) {
    let mut work = Vec::new();
    work.append(&mut nodes.entries);
    while let Some((_key, node)) = work.pop() {
        if let NestedPathNode::Branch(mut children) = node {
            work.append(&mut children.entries);
        }
    }
}

fn render_nested_path_entries(nodes: NestedPathNodes) -> Expr {
    render_nested_path_entries_with_front_depth(nodes, RECURSIVE_RENDER_NESTED_PATH_DEPTH)
        .into_value()
}

fn render_nested_path_entries_with_front_depth(
    nodes: NestedPathNodes,
    remaining_path_depth: usize,
) -> BoundedTraversal<Expr> {
    if remaining_path_depth == 0 {
        return BoundedTraversal::Exhausted(render_nested_path_entries_iterative(nodes));
    }

    let child_path_depth = remaining_path_depth - 1;
    let mut exhausted = false;
    let entries = nodes
        .entries
        .into_iter()
        .map(
            |entry| match render_nested_path_entry_with_front_depth(entry, child_path_depth) {
                BoundedTraversal::Complete(entry) => entry,
                BoundedTraversal::Exhausted(entry) => {
                    exhausted = true;
                    entry
                }
            },
        )
        .collect();
    let rendered = render_association_entry_forms_preserving_heads(entries);
    if exhausted {
        BoundedTraversal::Exhausted(rendered)
    } else {
        BoundedTraversal::Complete(rendered)
    }
}

fn render_nested_path_entry_with_front_depth(
    (key, node): (Expr, NestedPathNode),
    remaining_path_depth: usize,
) -> BoundedTraversal<AssociationEntryForm> {
    match node {
        NestedPathNode::Branch(children) => {
            match render_nested_path_entries_with_front_depth(children, remaining_path_depth) {
                BoundedTraversal::Complete(value) => {
                    BoundedTraversal::Complete(AssociationEntryForm {
                        key,
                        value,
                        delayed: false,
                    })
                }
                BoundedTraversal::Exhausted(value) => {
                    BoundedTraversal::Exhausted(AssociationEntryForm {
                        key,
                        value,
                        delayed: false,
                    })
                }
            }
        }
        NestedPathNode::Leaf { value, delayed } => {
            BoundedTraversal::Complete(AssociationEntryForm {
                key,
                value,
                delayed,
            })
        }
    }
}

enum RenderNestedPathWork {
    VisitEntries(NestedPathNodes),
    VisitEntry(Expr, NestedPathNode),
    FinishEntries { entry_count: usize },
    FinishBranch { key: Expr },
}

enum RenderNestedPathResult {
    Entries(Expr),
    Entry(AssociationEntryForm),
}

#[cold]
#[inline(never)]
fn render_nested_path_entries_iterative(nodes: NestedPathNodes) -> Expr {
    let mut work = vec![RenderNestedPathWork::VisitEntries(nodes)];
    let mut completed = Vec::new();

    while let Some(task) = work.pop() {
        match task {
            RenderNestedPathWork::VisitEntries(nodes) => {
                let entry_count = nodes.entries.len();
                work.push(RenderNestedPathWork::FinishEntries { entry_count });
                work.extend(
                    nodes
                        .entries
                        .into_iter()
                        .rev()
                        .map(|(key, node)| RenderNestedPathWork::VisitEntry(key, node)),
                );
            }
            RenderNestedPathWork::VisitEntry(key, node) => match node {
                NestedPathNode::Branch(children) => {
                    work.push(RenderNestedPathWork::FinishBranch { key });
                    work.push(RenderNestedPathWork::VisitEntries(children));
                }
                NestedPathNode::Leaf { value, delayed } => {
                    completed.push(RenderNestedPathResult::Entry(AssociationEntryForm {
                        key,
                        value,
                        delayed,
                    }));
                }
            },
            RenderNestedPathWork::FinishEntries { entry_count } => {
                let first_entry = completed
                    .len()
                    .checked_sub(entry_count)
                    .expect("nested-path entry results remain balanced");
                let entries = completed
                    .split_off(first_entry)
                    .into_iter()
                    .map(|result| match result {
                        RenderNestedPathResult::Entry(entry) => entry,
                        RenderNestedPathResult::Entries(_) => {
                            panic!("nested-path entries finish only rendered entries")
                        }
                    })
                    .collect();
                completed.push(RenderNestedPathResult::Entries(
                    render_association_entry_forms_preserving_heads(entries),
                ));
            }
            RenderNestedPathWork::FinishBranch { key } => {
                let value = match completed.pop() {
                    Some(RenderNestedPathResult::Entries(value)) => value,
                    _ => panic!("nested-path branch result remains balanced"),
                };
                completed.push(RenderNestedPathResult::Entry(AssociationEntryForm {
                    key,
                    value,
                    delayed: false,
                }));
            }
        }
    }

    match completed.pop() {
        Some(RenderNestedPathResult::Entries(value)) if completed.is_empty() => value,
        _ => panic!("nested-path rendering completes with one association"),
    }
}

pub(super) fn flatten_association_entries(
    prefix: Vec<Expr>,
    entries: Vec<AssociationEntryForm>,
) -> Vec<AssociationEntryForm> {
    flatten_association_entries_with_front_depth(
        prefix,
        entries,
        RECURSIVE_FLATTEN_ASSOCIATION_DEPTH,
    )
    .into_value()
}

fn flatten_association_entries_with_front_depth(
    prefix: Vec<Expr>,
    entries: Vec<AssociationEntryForm>,
    remaining_association_depth: usize,
) -> BoundedTraversal<Vec<AssociationEntryForm>> {
    if remaining_association_depth == 0 {
        return BoundedTraversal::Exhausted(flatten_association_entries_iterative(prefix, entries));
    }

    let child_association_depth = remaining_association_depth - 1;
    let mut flattened = Vec::new();
    let mut exhausted = false;

    for entry in entries {
        let mut path = prefix.clone();
        path.push(entry.key);

        match association_entry_forms_from_expr(&entry.value) {
            Some(children) if !children.is_empty() => {
                match flatten_association_entries_with_front_depth(
                    path,
                    children,
                    child_association_depth,
                ) {
                    BoundedTraversal::Complete(children) => flattened.extend(children),
                    BoundedTraversal::Exhausted(children) => {
                        exhausted = true;
                        flattened.extend(children);
                    }
                }
            }
            _ => flattened.push(AssociationEntryForm {
                key: Expr::call(system_expr(heads::structure::LIST), path),
                value: entry.value,
                delayed: entry.delayed,
            }),
        }
    }

    if exhausted {
        BoundedTraversal::Exhausted(flattened)
    } else {
        BoundedTraversal::Complete(flattened)
    }
}

struct FlattenAssociationWork {
    prefix: Vec<Expr>,
    entries: std::vec::IntoIter<AssociationEntryForm>,
}

#[cold]
#[inline(never)]
fn flatten_association_entries_iterative(
    prefix: Vec<Expr>,
    entries: Vec<AssociationEntryForm>,
) -> Vec<AssociationEntryForm> {
    let mut work = vec![FlattenAssociationWork {
        prefix,
        entries: entries.into_iter(),
    }];
    let mut flattened = Vec::new();

    while let Some(mut frame) = work.pop() {
        let Some(entry) = frame.entries.next() else {
            continue;
        };
        let mut path = frame.prefix.clone();
        path.push(entry.key);
        if !frame.entries.as_slice().is_empty() {
            work.push(frame);
        }

        match association_entry_forms_from_expr(&entry.value) {
            Some(children) if !children.is_empty() => {
                work.push(FlattenAssociationWork {
                    prefix: path,
                    entries: children.into_iter(),
                });
            }
            _ => flattened.push(AssociationEntryForm {
                key: Expr::call(system_expr(heads::structure::LIST), path),
                value: entry.value,
                delayed: entry.delayed,
            }),
        }
    }

    flattened
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(key: Expr, value: Expr, delayed: bool) -> Expr {
        Expr::call(
            system_expr(if delayed { "RuleDelayed" } else { "Rule" }),
            vec![key, value],
        )
    }

    fn association(entries: Vec<Expr>) -> Expr {
        Expr::call(system_expr("Association"), entries)
    }

    fn list(items: Vec<Expr>) -> Expr {
        Expr::call(system_expr("List"), items)
    }

    fn leaf(value: i64) -> NestedPathNode {
        NestedPathNode::Leaf {
            value: Expr::small_integer(value),
            delayed: false,
        }
    }

    fn insert_fixture(include_target: bool) -> NestedPathNodes {
        let mut deepest = NestedPathNodes::new();
        deepest.push(Expr::string("deep-first"), leaf(301));
        deepest.push(Expr::string("deep-second"), leaf(302));
        if include_target {
            deepest.push(
                Expr::string("target"),
                NestedPathNode::Leaf {
                    value: Expr::small_integer(999),
                    delayed: true,
                },
            );
        }

        let mut middle = NestedPathNodes::new();
        middle.push(Expr::string("child"), NestedPathNode::Branch(deepest));
        middle.push(Expr::string("middle-side"), leaf(201));

        let mut root = NestedPathNodes::new();
        root.push(Expr::string("root"), NestedPathNode::Branch(middle));
        root.push(Expr::string("root-side"), leaf(101));
        root
    }

    #[test]
    fn insert_path_forced_front_tail_matches_complete_front_on_branched_fixture() {
        let path = [
            Expr::string("root"),
            Expr::string("child"),
            Expr::string("target"),
        ];

        let mut through_tail = insert_fixture(false);
        let tail = insert_path_with_front_depth(
            &mut through_tail,
            &path,
            Expr::small_integer(999),
            true,
            0,
        );
        assert!(matches!(&tail, BoundedTraversal::Exhausted(Some(()))));
        assert_eq!(tail.into_value(), Some(()));

        let mut through_front = insert_fixture(false);
        let front = insert_path_with_front_depth(
            &mut through_front,
            &path,
            Expr::small_integer(999),
            true,
            16,
        );
        assert!(matches!(&front, BoundedTraversal::Complete(Some(()))));
        assert_eq!(front.into_value(), Some(()));

        let expected = insert_fixture(true);
        assert_eq!(through_tail, through_front);
        assert_eq!(through_tail, expected);
    }

    fn render_fixture() -> NestedPathNodes {
        let mut deepest = NestedPathNodes::new();
        deepest.push(
            Expr::string("deep-first"),
            NestedPathNode::Leaf {
                value: Expr::small_integer(301),
                delayed: true,
            },
        );
        deepest.push(Expr::string("deep-second"), leaf(302));

        let mut middle = NestedPathNodes::new();
        middle.push(Expr::string("child"), NestedPathNode::Branch(deepest));
        middle.push(Expr::string("middle-side"), leaf(201));

        let mut root = NestedPathNodes::new();
        root.push(Expr::string("root"), NestedPathNode::Branch(middle));
        root.push(Expr::string("root-side"), leaf(101));
        root
    }

    fn rendered_fixture() -> Expr {
        association(vec![
            rule(
                Expr::string("root"),
                association(vec![
                    rule(
                        Expr::string("child"),
                        association(vec![
                            rule(Expr::string("deep-first"), Expr::small_integer(301), true),
                            rule(Expr::string("deep-second"), Expr::small_integer(302), false),
                        ]),
                        false,
                    ),
                    rule(Expr::string("middle-side"), Expr::small_integer(201), false),
                ]),
                false,
            ),
            rule(Expr::string("root-side"), Expr::small_integer(101), false),
        ])
    }

    #[test]
    fn render_nested_path_forced_front_tail_matches_complete_front_on_branched_fixture() {
        let tail = render_nested_path_entries_with_front_depth(render_fixture(), 0);
        assert!(matches!(&tail, BoundedTraversal::Exhausted(_)));
        let tail = tail.into_value();

        let front = render_nested_path_entries_with_front_depth(render_fixture(), 16);
        assert!(matches!(&front, BoundedTraversal::Complete(_)));
        let front = front.into_value();

        let expected = rendered_fixture();
        assert_eq!(tail, front);
        assert_eq!(tail, expected);
    }

    fn flatten_fixture() -> Vec<AssociationEntryForm> {
        vec![
            AssociationEntryForm {
                key: Expr::string("root"),
                value: association(vec![
                    rule(
                        Expr::string("child"),
                        association(vec![
                            rule(Expr::string("deep-first"), Expr::small_integer(301), true),
                            rule(Expr::string("deep-second"), Expr::small_integer(302), false),
                        ]),
                        false,
                    ),
                    rule(Expr::string("middle-side"), Expr::small_integer(201), false),
                ]),
                delayed: false,
            },
            AssociationEntryForm {
                key: Expr::string("root-side"),
                value: Expr::small_integer(101),
                delayed: false,
            },
        ]
    }

    fn flattened_fixture() -> Vec<AssociationEntryForm> {
        vec![
            AssociationEntryForm {
                key: list(vec![
                    Expr::string("root"),
                    Expr::string("child"),
                    Expr::string("deep-first"),
                ]),
                value: Expr::small_integer(301),
                delayed: true,
            },
            AssociationEntryForm {
                key: list(vec![
                    Expr::string("root"),
                    Expr::string("child"),
                    Expr::string("deep-second"),
                ]),
                value: Expr::small_integer(302),
                delayed: false,
            },
            AssociationEntryForm {
                key: list(vec![Expr::string("root"), Expr::string("middle-side")]),
                value: Expr::small_integer(201),
                delayed: false,
            },
            AssociationEntryForm {
                key: list(vec![Expr::string("root-side")]),
                value: Expr::small_integer(101),
                delayed: false,
            },
        ]
    }

    #[test]
    fn flatten_association_forced_front_tail_matches_complete_front_on_branched_fixture() {
        let tail = flatten_association_entries_with_front_depth(Vec::new(), flatten_fixture(), 0);
        assert!(matches!(&tail, BoundedTraversal::Exhausted(_)));
        let tail = tail.into_value();

        let front = flatten_association_entries_with_front_depth(Vec::new(), flatten_fixture(), 16);
        assert!(matches!(&front, BoundedTraversal::Complete(_)));
        let front = front.into_value();

        let expected = flattened_fixture();
        assert_eq!(tail, front);
        assert_eq!(tail, expected);
    }
}
