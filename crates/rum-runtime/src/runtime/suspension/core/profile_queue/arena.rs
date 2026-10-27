//! Stable slots and an AVL sequence, shared only by queues in one epoch.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct CellId {
    index: usize,
    generation: u64,
}

struct Node {
    generation: u64,
    cell: Cell,
    parent: Option<CellId>,
    left: Option<CellId>,
    right: Option<CellId>,
    height: u8,
    deferred: usize,
    consumers: usize,
}
enum Slot {
    Live(Node),
    Free(Option<usize>),
}
pub(super) struct Arena {
    slots: Vec<Slot>,
    free: Option<usize>,
    storage: Storage,
}

impl Arena {
    #[cfg(test)]
    pub(super) fn ordered_and_balanced(&self, root: Option<CellId>) -> Vec<CellId> {
        fn visit(
            arena: &Arena,
            root: Option<CellId>,
            parent: Option<CellId>,
            depth: usize,
            output: &mut Vec<CellId>,
        ) -> (u8, usize, usize) {
            assert!(depth < 64, "sequence cycle or unbalanced tree");
            let Some(id) = root else {
                return (0, 0, 0);
            };
            let node = arena.node(id).unwrap();
            assert_eq!(node.parent, parent);
            let left = visit(arena, node.left, Some(id), depth + 1, output);
            output.push(id);
            let right = visit(arena, node.right, Some(id), depth + 1, output);
            assert!(left.0.abs_diff(right.0) <= 1);
            let expected = (
                1 + left.0.max(right.0),
                left.1 + right.1 + usize::from(node.cell.deferred),
                left.2 + right.2 + usize::from(node.cell.consumer()),
            );
            assert_eq!((node.height, node.deferred, node.consumers), expected);
            expected
        }
        let mut result = Vec::new();
        visit(self, root, None, 0, &mut result);
        result
    }
    pub(super) fn new(storage: Storage) -> Self {
        Self {
            slots: Vec::new(),
            free: None,
            storage,
        }
    }
    fn node(&self, id: CellId) -> Option<&Node> {
        match self.slots.get(id.index)? {
            Slot::Live(n) if n.generation == id.generation => Some(n),
            _ => None,
        }
    }
    fn node_mut(&mut self, id: CellId) -> Option<&mut Node> {
        match self.slots.get_mut(id.index)? {
            Slot::Live(n) if n.generation == id.generation => Some(n),
            _ => None,
        }
    }
    pub(super) fn cell(&self, id: CellId) -> Option<&Cell> {
        Some(&self.node(id)?.cell)
    }
    pub(super) fn clear_deferred(&mut self, id: CellId) -> Option<()> {
        self.node_mut(id)?.cell.deferred = false;
        self.update(id)
    }
    fn height(&self, id: Option<CellId>) -> u8 {
        id.and_then(|i| self.node(i)).map_or(0, |n| n.height)
    }
    fn deferred(&self, id: Option<CellId>) -> usize {
        id.and_then(|i| self.node(i)).map_or(0, |n| n.deferred)
    }
    fn consumers(&self, id: Option<CellId>) -> usize {
        id.and_then(|i| self.node(i)).map_or(0, |n| n.consumers)
    }
    fn update(&mut self, id: CellId) -> Option<()> {
        let n = self.node(id)?;
        let height = 1 + self.height(n.left).max(self.height(n.right));
        let deferred =
            usize::from(n.cell.deferred) + self.deferred(n.left) + self.deferred(n.right);
        let consumers =
            usize::from(n.cell.consumer()) + self.consumers(n.left) + self.consumers(n.right);
        let n = self.node_mut(id)?;
        n.height = height;
        n.deferred = deferred;
        n.consumers = consumers;
        Some(())
    }
    fn parent(&mut self, child: Option<CellId>, parent: Option<CellId>) -> Option<()> {
        if let Some(child) = child {
            self.node_mut(child)?.parent = parent;
        }
        Some(())
    }
    fn left(&mut self, id: CellId, child: Option<CellId>) -> Option<()> {
        self.node_mut(id)?.left = child;
        self.parent(child, Some(id))
    }
    fn right(&mut self, id: CellId, child: Option<CellId>) -> Option<()> {
        self.node_mut(id)?.right = child;
        self.parent(child, Some(id))
    }
    fn rotate_left(&mut self, id: CellId) -> Option<CellId> {
        let parent = self.node(id)?.parent;
        let next = self.node(id)?.right?;
        let middle = self.node(next)?.left;
        self.right(id, middle)?;
        self.left(next, Some(id))?;
        self.parent(Some(next), parent)?;
        self.update(id)?;
        self.update(next)?;
        Some(next)
    }
    fn rotate_right(&mut self, id: CellId) -> Option<CellId> {
        let parent = self.node(id)?.parent;
        let next = self.node(id)?.left?;
        let middle = self.node(next)?.right;
        self.left(id, middle)?;
        self.right(next, Some(id))?;
        self.parent(Some(next), parent)?;
        self.update(id)?;
        self.update(next)?;
        Some(next)
    }
    fn balance(&mut self, id: CellId) -> Option<CellId> {
        self.update(id)?;
        let n = self.node(id)?;
        let (left, right) = (n.left, n.right);
        let tilt = i16::from(self.height(left)) - i16::from(self.height(right));
        if tilt > 1 {
            let left = left?;
            let n = self.node(left)?;
            if self.height(n.right) > self.height(n.left) {
                let next = self.rotate_left(left)?;
                self.left(id, Some(next))?;
            }
            self.rotate_right(id)
        } else if tilt < -1 {
            let right = right?;
            let n = self.node(right)?;
            if self.height(n.left) > self.height(n.right) {
                let next = self.rotate_right(right)?;
                self.right(id, Some(next))?;
            }
            self.rotate_left(id)
        } else {
            Some(id)
        }
    }
    fn replace_child(&mut self, parent: CellId, old: CellId, new: Option<CellId>) -> Option<()> {
        if self.node(parent)?.left == Some(old) {
            self.left(parent, new)
        } else if self.node(parent)?.right == Some(old) {
            self.right(parent, new)
        } else {
            None
        }
    }
    fn balance_up(&mut self, mut id: CellId) -> Option<CellId> {
        loop {
            let parent = self.node(id)?.parent;
            let balanced = self.balance(id)?;
            if let Some(parent) = parent {
                self.replace_child(parent, id, Some(balanced))?;
                id = parent;
            } else {
                return Some(balanced);
            }
        }
    }

    pub(super) fn allocate(&mut self, cell: Cell) -> Option<CellId> {
        if self.free.is_none() && self.slots.len() == self.slots.capacity() {
            let old = self.slots.capacity();
            let capacity = old.checked_mul(2)?.max(4);
            let peak = old.checked_add(capacity)?.checked_mul(size_of::<Slot>())?;
            if !self.storage.resize(peak) {
                return None;
            }
            if self
                .slots
                .try_reserve_exact(capacity - self.slots.len())
                .is_err()
            {
                self.storage.resize(old * size_of::<Slot>());
                return None;
            }
            if !self
                .storage
                .resize(self.slots.capacity() * size_of::<Slot>())
            {
                return None;
            }
        }
        let generation = crate::runtime::profiling::logical::epoch::next_id();
        let n = Node {
            generation,
            deferred: usize::from(cell.deferred),
            consumers: usize::from(cell.consumer()),
            cell,
            parent: None,
            left: None,
            right: None,
            height: 1,
        };
        let index = if let Some(index) = self.free {
            let Some(Slot::Free(next)) = self.slots.get(index) else {
                return None;
            };
            self.free = *next;
            self.slots[index] = Slot::Live(n);
            index
        } else {
            self.slots.push(Slot::Live(n));
            self.slots.len() - 1
        };
        Some(CellId { index, generation })
    }
    pub(super) fn release(&mut self, id: CellId) -> Option<Cell> {
        self.node(id)?;
        let slot = self.slots.get_mut(id.index)?;
        match std::mem::replace(slot, Slot::Free(self.free)) {
            Slot::Live(n) => {
                self.free = Some(id.index);
                Some(n.cell)
            }
            other => {
                *slot = other;
                None
            }
        }
    }
    pub(super) fn belongs(&self, root: Option<CellId>, mut id: CellId) -> bool {
        while let Some(node) = self.node(id) {
            if let Some(parent) = node.parent {
                id = parent;
            } else {
                return root == Some(id);
            }
        }
        false
    }
    pub(super) fn first(&self, root: Option<CellId>) -> Option<CellId> {
        let mut id = root?;
        while let Some(left) = self.node(id)?.left {
            id = left;
        }
        Some(id)
    }
    pub(super) fn last(&self, root: Option<CellId>) -> Option<CellId> {
        let mut id = root?;
        while let Some(right) = self.node(id)?.right {
            id = right;
        }
        Some(id)
    }
    pub(super) fn last_consumer(&self, root: Option<CellId>) -> Option<CellId> {
        let mut id = root?;
        loop {
            let n = self.node(id)?;
            if self.consumers(n.right) != 0 {
                id = n.right?;
            } else if n.cell.consumer() {
                return Some(id);
            } else {
                id = n.left?;
            }
        }
    }
    pub(super) fn first_deferred(&self, root: Option<CellId>) -> Option<CellId> {
        let mut id = root?;
        loop {
            let n = self.node(id)?;
            if self.deferred(n.left) != 0 {
                id = n.left?;
            } else if n.cell.deferred {
                return Some(id);
            } else {
                id = n.right?;
            }
        }
    }
    pub(super) fn insert(
        &mut self,
        root: Option<CellId>,
        after: Option<CellId>,
        added: CellId,
    ) -> Option<CellId> {
        let Some(root) = root else {
            return after.is_none().then_some(added);
        };
        let parent = if let Some(after) = after {
            if !self.belongs(Some(root), after) {
                return None;
            }
            if let Some(right) = self.node(after)?.right {
                let next = self.first(Some(right))?;
                self.left(next, Some(added))?;
                next
            } else {
                self.right(after, Some(added))?;
                after
            }
        } else {
            let first = self.first(Some(root))?;
            self.left(first, Some(added))?;
            first
        };
        self.balance_up(parent)
    }
    fn join_pivot(
        &mut self,
        left: Option<CellId>,
        pivot: CellId,
        right: Option<CellId>,
    ) -> Option<CellId> {
        if self.height(left) > self.height(right) + 1 {
            let left = left?;
            let child = self.join_pivot(self.node(left)?.right, pivot, right)?;
            self.right(left, Some(child))?;
            self.balance(left)
        } else if self.height(right) > self.height(left) + 1 {
            let right = right?;
            let child = self.join_pivot(left, pivot, self.node(right)?.left)?;
            self.left(right, Some(child))?;
            self.balance(right)
        } else {
            self.left(pivot, left)?;
            self.right(pivot, right)?;
            self.update(pivot)?;
            Some(pivot)
        }
    }
    fn extract_min(&mut self, id: CellId) -> Option<(Option<CellId>, CellId)> {
        if let Some(left) = self.node(id)?.left {
            let (child, pivot) = self.extract_min(left)?;
            self.left(id, child)?;
            Some((Some(self.balance(id)?), pivot))
        } else {
            let right = self.node(id)?.right;
            self.parent(right, None)?;
            self.right(id, None)?;
            self.parent(Some(id), None)?;
            self.update(id)?;
            Some((right, id))
        }
    }
    pub(super) fn join(&mut self, left: Option<CellId>, right: Option<CellId>) -> Option<CellId> {
        let root = match (left, right) {
            (None, root) | (root, None) => root,
            (Some(left), Some(right)) => {
                let (right, pivot) = self.extract_min(right)?;
                self.parent(Some(left), None)?;
                self.parent(right, None)?;
                Some(self.join_pivot(Some(left), pivot, right)?)
            }
        };
        self.parent(root, None)?;
        root
    }
    pub(super) fn cut(&mut self, root: Option<CellId>, id: CellId) -> Option<Option<CellId>> {
        if !self.belongs(root, id) {
            return None;
        }
        let n = self.node(id)?;
        let (parent, left, right) = (n.parent, n.left, n.right);
        self.parent(left, None)?;
        self.parent(right, None)?;
        let joined = self.join(left, right);
        if (left.is_some() || right.is_some()) && joined.is_none() {
            return None;
        }
        self.left(id, None)?;
        self.right(id, None)?;
        self.parent(Some(id), None)?;
        self.update(id)?;
        if let Some(parent) = parent {
            self.replace_child(parent, id, joined)?;
            Some(Some(self.balance_up(parent)?))
        } else {
            Some(joined)
        }
    }
}
