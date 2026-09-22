use std::fmt::Write;
use std::format as renamed;

pub fn root(n: usize) {
    let _text = format!("{n}");
}

pub fn alias(n: usize) {
    let _text = renamed!("{n}");
}

pub fn writer(text: &mut String, n: usize) {
    let _result = write!(text, "{n}");
}
