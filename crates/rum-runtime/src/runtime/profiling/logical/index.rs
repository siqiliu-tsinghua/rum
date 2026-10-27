//! A budgeted AVL index. Slots are recycled; capacity grows geometrically.
use super::Storage;
use std::cmp::Ordering;
use std::mem::size_of;

struct Node<K, V> {
    key: K,
    value: V,
    left: Option<usize>,
    right: Option<usize>,
    height: u8,
}
enum Slot<K, V> {
    Live(Node<K, V>),
    Free(Option<usize>),
}
pub(in crate::runtime) struct Index<K, V> {
    slots: Vec<Slot<K, V>>,
    root: Option<usize>,
    free: Option<usize>,
    len: usize,
    storage: Storage,
}

pub(in crate::runtime) struct Matching<'a, K, V, F> {
    index: &'a mut Index<K, V>,
    cursor: usize,
    predicate: F,
}
impl<K: Ord + Copy, V, F: FnMut(&V) -> bool> Iterator for Matching<'_, K, V, F> {
    type Item = V;
    fn next(&mut self) -> Option<V> {
        while self.cursor < self.index.slots.len() {
            if let Some(node) = self.index.node(self.cursor) {
                if (self.predicate)(&node.value) {
                    // Deletion may move a successor into this slot. Revisit it.
                    let key = node.key;
                    return self.index.remove(key);
                }
            }
            self.cursor += 1;
        }
        None
    }
}

impl<K: Ord + Copy, V> Index<K, V> {
    pub(in crate::runtime) fn new(storage: Storage) -> Self {
        Self {
            slots: Vec::new(),
            root: None,
            free: None,
            len: 0,
            storage,
        }
    }
    pub(in crate::runtime) fn len(&self) -> usize {
        self.len
    }

    fn node(&self, i: usize) -> Option<&Node<K, V>> {
        match self.slots.get(i)? {
            Slot::Live(node) => Some(node),
            Slot::Free(_) => None,
        }
    }
    fn node_mut(&mut self, i: usize) -> Option<&mut Node<K, V>> {
        match self.slots.get_mut(i)? {
            Slot::Live(node) => Some(node),
            Slot::Free(_) => None,
        }
    }
    fn find(&self, key: K) -> Option<usize> {
        let mut at = self.root;
        while let Some(i) = at {
            let node = self.node(i)?;
            at = match key.cmp(&node.key) {
                Ordering::Less => node.left,
                Ordering::Greater => node.right,
                Ordering::Equal => return Some(i),
            };
        }
        None
    }
    pub(in crate::runtime) fn get(&self, key: K) -> Option<&V> {
        Some(&self.node(self.find(key)?)?.value)
    }
    pub(in crate::runtime) fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let i = self.find(key)?;
        Some(&mut self.node_mut(i)?.value)
    }

    fn height(&self, i: Option<usize>) -> u8 {
        i.and_then(|i| self.node(i)).map_or(0, |n| n.height)
    }
    fn update(&mut self, i: usize) -> Option<()> {
        let n = self.node(i)?;
        let height = 1 + self.height(n.left).max(self.height(n.right));
        self.node_mut(i)?.height = height;
        Some(())
    }
    fn rotate_left(&mut self, i: usize) -> Option<usize> {
        let next = self.node(i)?.right?;
        let middle = self.node(next)?.left;
        self.node_mut(i)?.right = middle;
        self.node_mut(next)?.left = Some(i);
        self.update(i)?;
        self.update(next)?;
        Some(next)
    }
    fn rotate_right(&mut self, i: usize) -> Option<usize> {
        let next = self.node(i)?.left?;
        let middle = self.node(next)?.right;
        self.node_mut(i)?.left = middle;
        self.node_mut(next)?.right = Some(i);
        self.update(i)?;
        self.update(next)?;
        Some(next)
    }
    fn balance(&mut self, i: usize) -> Option<usize> {
        self.update(i)?;
        let n = self.node(i)?;
        let (left, right) = (n.left, n.right);
        let tilt = i16::from(self.height(left)) - i16::from(self.height(right));
        if tilt > 1 {
            let left = left?;
            let n = self.node(left)?;
            if self.height(n.right) > self.height(n.left) {
                let next = self.rotate_left(left)?;
                self.node_mut(i)?.left = Some(next);
            }
            self.rotate_right(i)
        } else if tilt < -1 {
            let right = right?;
            let n = self.node(right)?;
            if self.height(n.left) > self.height(n.right) {
                let next = self.rotate_right(right)?;
                self.node_mut(i)?.right = Some(next);
            }
            self.rotate_left(i)
        } else {
            Some(i)
        }
    }

    fn grow(&mut self) -> bool {
        if self.free.is_some() || self.slots.len() < self.slots.capacity() {
            return true;
        }
        let old = self.slots.capacity();
        let capacity = old.saturating_mul(2).max(4);
        let Some(peak) = old
            .checked_add(capacity)
            .and_then(|n| n.checked_mul(size_of::<Slot<K, V>>()))
        else {
            return false;
        };
        // Charge both buffers while the allocator may be moving the old one.
        if !self.storage.resize(peak) {
            return false;
        }
        if self
            .slots
            .try_reserve_exact(capacity - self.slots.len())
            .is_err()
        {
            self.storage.resize(old * size_of::<Slot<K, V>>());
            return false;
        }
        if !self
            .storage
            .resize(self.slots.capacity() * size_of::<Slot<K, V>>())
        {
            return false;
        }
        true
    }
    fn attach(&mut self, root: Option<usize>, added: usize, key: K) -> Option<usize> {
        let Some(i) = root else {
            return Some(added);
        };
        if key < self.node(i)?.key {
            let child = self.attach(self.node(i)?.left, added, key)?;
            self.node_mut(i)?.left = Some(child);
        } else {
            let child = self.attach(self.node(i)?.right, added, key)?;
            self.node_mut(i)?.right = Some(child);
        }
        self.balance(i)
    }
    pub(in crate::runtime) fn insert(&mut self, key: K, value: V) -> bool {
        if self.find(key).is_some() {
            return false;
        }
        if !self.grow() {
            return false;
        }
        let node = Node {
            key,
            value,
            left: None,
            right: None,
            height: 1,
        };
        let index = if let Some(i) = self.free {
            if let Some(Slot::Free(next)) = self.slots.get(i) {
                self.free = *next;
                self.slots[i] = Slot::Live(node);
                i
            } else {
                return false;
            }
        } else {
            self.slots.push(Slot::Live(node));
            self.slots.len() - 1
        };
        let Some(root) = self.attach(self.root, index, key) else {
            return false;
        };
        self.root = Some(root);
        self.len += 1;
        true
    }

    fn release(&mut self, i: usize) -> Option<Node<K, V>> {
        let slot = self.slots.get_mut(i)?;
        match std::mem::replace(slot, Slot::Free(self.free)) {
            Slot::Live(node) => {
                self.free = Some(i);
                self.len -= 1;
                Some(node)
            }
            free => {
                *slot = free;
                None
            }
        }
    }
    fn take_min(&mut self, i: usize) -> Option<(Option<usize>, Node<K, V>)> {
        if let Some(left) = self.node(i)?.left {
            let (next, node) = self.take_min(left)?;
            self.node_mut(i)?.left = next;
            Some((Some(self.balance(i)?), node))
        } else {
            let right = self.node(i)?.right;
            Some((right, self.release(i)?))
        }
    }
    fn erase(&mut self, i: usize, key: K) -> Option<(Option<usize>, V)> {
        match key.cmp(&self.node(i)?.key) {
            Ordering::Less => {
                let (child, value) = self.erase(self.node(i)?.left?, key)?;
                self.node_mut(i)?.left = child;
                Some((Some(self.balance(i)?), value))
            }
            Ordering::Greater => {
                let (child, value) = self.erase(self.node(i)?.right?, key)?;
                self.node_mut(i)?.right = child;
                Some((Some(self.balance(i)?), value))
            }
            Ordering::Equal => {
                let n = self.node(i)?;
                let (left, right) = (n.left, n.right);
                if let (Some(_), Some(right)) = (left, right) {
                    let (child, successor) = self.take_min(right)?;
                    let n = self.node_mut(i)?;
                    n.key = successor.key;
                    n.right = child;
                    let value = std::mem::replace(&mut n.value, successor.value);
                    Some((Some(self.balance(i)?), value))
                } else {
                    Some((left.or(right), self.release(i)?.value))
                }
            }
        }
    }
    pub(in crate::runtime) fn remove(&mut self, key: K) -> Option<V> {
        let (root, value) = self.erase(self.root?, key)?;
        self.root = root;
        Some(value)
    }
    #[cfg(test)]
    pub(in crate::runtime) fn first_key(&self) -> Option<K> {
        let mut i = self.root?;
        while let Some(left) = self.node(i)?.left {
            i = left;
        }
        Some(self.node(i)?.key)
    }
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    pub(in crate::runtime) fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.slots.iter().filter_map(|s| match s {
            Slot::Live(n) => Some((n.key, &n.value)),
            Slot::Free(_) => None,
        })
    }
    pub(in crate::runtime) fn remove_matching<F: FnMut(&V) -> bool>(
        &mut self,
        predicate: F,
    ) -> Matching<'_, K, V, F> {
        Matching {
            index: self,
            cursor: 0,
            predicate,
        }
    }
}
