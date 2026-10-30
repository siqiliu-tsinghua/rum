use crate::runtime::profiling::logical::{Epoch, index::Index};
use std::collections::BTreeMap;

#[test]
fn logical_index_matches_ordered_map_through_reuse_and_bulk_removal() {
    let lease = crate::runtime::profiling::lease::lease();
    let epoch = Epoch::for_test(&lease, 4 * 1024 * 1024).unwrap();
    let mut index = Index::new(epoch.storage(0).unwrap());
    let mut expected = BTreeMap::new();
    let mut seed = 7u64;
    for step in 0..30_000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let key = (seed >> 32) % 4096;
        if step % 3 == 0 {
            assert_eq!(index.remove(key), expected.remove(&key));
        } else if let std::collections::btree_map::Entry::Vacant(entry) = expected.entry(key) {
            entry.insert(step);
            assert!(index.insert(key, step));
        }
        assert_eq!(index.get(key), expected.get(&key));
        assert_eq!(index.len(), expected.len());
        if step % 137 == 0 {
            assert_eq!(index.first_key(), expected.keys().next().copied());
            assert_eq!(
                index
                    .iter()
                    .map(|(k, v)| (k, *v))
                    .collect::<BTreeMap<_, _>>(),
                expected
            );
        }
    }
    let mut removed: Vec<_> = index.remove_matching(|v| v % 2 == 0).collect();
    let mut wanted: Vec<_> = expected.values().copied().filter(|v| v % 2 == 0).collect();
    removed.sort_unstable();
    wanted.sort_unstable();
    assert_eq!(removed, wanted);
    expected.retain(|_, v| *v % 2 != 0);
    assert_eq!(
        index
            .iter()
            .map(|(k, v)| (k, *v))
            .collect::<BTreeMap<_, _>>(),
        expected
    );
    while let Some(key) = index.first_key() {
        assert_eq!(index.remove(key), expected.remove(&key));
    }
    assert!(expected.is_empty());
    assert_eq!(index.len(), 0);
}
