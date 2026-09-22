use std::iter::Iterator as Stream;

pub fn root(n: usize) {
    let _values: Vec<usize> = Stream::collect(0..n);
}

pub fn adaptor(n: usize) {
    let _values: Vec<usize> = (0..n).map(|v| v + 1).collect();
}
