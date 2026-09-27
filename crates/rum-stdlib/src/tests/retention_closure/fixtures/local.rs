//! Compiler-checked bodies for the local lint's positive and negative controls.

pub fn unit_helper() {}
pub fn missing() {
    unit_helper();
}
pub fn scalar(value: u8) -> u8 {
    value
}
pub fn direct() -> Vec<u8> {
    Vec::<u8>::new()
}
type Bytes = Vec<u8>;
pub fn aliased() -> Bytes {
    Bytes::new()
}
pub fn allocator() -> Vec<u8> {
    Vec::<u8>::new()
}
pub fn through_helper() -> Vec<u8> {
    allocator()
}
pub fn vector(value: Vec<u8>) -> Vec<u8> {
    unit_helper();
    value
}
pub fn boxed(value: Box<u8>) -> Box<u8> {
    unit_helper();
    value
}
pub fn string(value: String) -> String {
    unit_helper();
    value
}
pub fn btree(value: std::collections::BTreeMap<u8, u8>) -> std::collections::BTreeMap<u8, u8> {
    unit_helper();
    value
}
pub fn hashmap(value: std::collections::HashMap<u8, u8>) -> std::collections::HashMap<u8, u8> {
    unit_helper();
    value
}
pub fn hashset(value: std::collections::HashSet<u8>) -> std::collections::HashSet<u8> {
    unit_helper();
    value
}
pub fn unknown(f: fn()) {
    f();
}
pub fn unknown_literal() -> f64 {
    0.0
}
