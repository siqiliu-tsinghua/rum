pub fn root(map: &mut std::collections::BTreeMap<usize, usize>) {
    map.entry(1).or_insert_with(|| 2);
}

pub fn raw() {
    let _map = std::collections::BTreeMap::<usize, usize>::new();
}
