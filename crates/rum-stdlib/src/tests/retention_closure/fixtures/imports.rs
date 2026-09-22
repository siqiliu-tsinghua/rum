//! Compiler-checked namespace collision and cyclic glob regression.
#![allow(unused_imports, dead_code)]

// The imported name is a macro, not a module shadowing the external crate.
use formatting::Write;
use library::fmt as formatting;
use std as library;
use std::{format, format as std};

mod a {
    pub use super::b::*;
    pub fn helper() {}
}

mod b {
    pub use super::a::*;
}

pub fn root() {
    b::helper();
}
