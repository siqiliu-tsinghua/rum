use std::cmp::Ordering;
use std::collections::hash_map::RandomState;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use rum_core::{BigInt, Expr, Rational, SharedArgs, SharedExpr, Symbol, WeakSharedArgs};
use rum_syntax::SyntaxError;

use crate::runtime::metadata::BuiltinFn;

/// A borrowed visitor over the two kinds of roots held by suspension state.
pub trait FrameRootVisitor {
    fn visit_expr(&mut self, expr: &Expr);
    fn visit_symbol(&mut self, symbol: &Symbol);
}

/// Recursively visits expression and bare-symbol roots without taking ownership.
pub trait FrameRootTrace {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor);
}

impl FrameRootTrace for Expr {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        visitor.visit_expr(self);
    }
}

impl FrameRootTrace for Symbol {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        visitor.visit_symbol(self);
    }
}

impl FrameRootTrace for SharedExpr {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        visitor.visit_expr(self.as_ref());
    }
}

impl FrameRootTrace for SharedArgs {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        for expr in self {
            visitor.visit_expr(expr);
        }
    }
}

impl FrameRootTrace for WeakSharedArgs {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl<T: FrameRootTrace> FrameRootTrace for Option<T> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        if let Some(value) = self {
            value.trace_roots(visitor);
        }
    }
}

impl<T: FrameRootTrace, E: FrameRootTrace> FrameRootTrace for Result<T, E> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        match self {
            Ok(value) => value.trace_roots(visitor),
            Err(error) => error.trace_roots(visitor),
        }
    }
}

impl<T: FrameRootTrace> FrameRootTrace for Vec<T> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        for value in self {
            value.trace_roots(visitor);
        }
    }
}

impl<T: FrameRootTrace> FrameRootTrace for VecDeque<T> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        for value in self {
            value.trace_roots(visitor);
        }
    }
}

impl<T: FrameRootTrace> FrameRootTrace for [T] {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        for value in self {
            value.trace_roots(visitor);
        }
    }
}

impl<T: FrameRootTrace, const N: usize> FrameRootTrace for [T; N] {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.as_slice().trace_roots(visitor);
    }
}

impl<T: FrameRootTrace + ?Sized> FrameRootTrace for Box<T> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.as_ref().trace_roots(visitor);
    }
}

impl<T: FrameRootTrace> FrameRootTrace for Arc<T> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.as_ref().trace_roots(visitor);
    }
}

impl<T: FrameRootTrace> FrameRootTrace for Arc<[T]> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        for value in self.iter() {
            value.trace_roots(visitor);
        }
    }
}

impl<K: FrameRootTrace, V: FrameRootTrace, S> FrameRootTrace for HashMap<K, V, S> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        for (key, value) in self {
            key.trace_roots(visitor);
            value.trace_roots(visitor);
        }
    }
}

impl<T: FrameRootTrace, S> FrameRootTrace for HashSet<T, S> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        for value in self {
            value.trace_roots(visitor);
        }
    }
}

impl<K: FrameRootTrace, V: FrameRootTrace> FrameRootTrace for BTreeMap<K, V> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        for (key, value) in self {
            key.trace_roots(visitor);
            value.trace_roots(visitor);
        }
    }
}

impl<T: FrameRootTrace> FrameRootTrace for BTreeSet<T> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        for value in self {
            value.trace_roots(visitor);
        }
    }
}

impl<T: FrameRootTrace> FrameRootTrace for std::vec::IntoIter<T> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.as_slice().trace_roots(visitor);
    }
}

impl<T: FrameRootTrace> FrameRootTrace for OnceLock<T> {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        if let Some(value) = self.get() {
            value.trace_roots(visitor);
        }
    }
}

impl<T: FrameRootTrace + ?Sized> FrameRootTrace for &T {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        (*self).trace_roots(visitor);
    }
}

impl<T: FrameRootTrace + ?Sized> FrameRootTrace for &mut T {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        (**self).trace_roots(visitor);
    }
}

impl<A: FrameRootTrace> FrameRootTrace for (A,) {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.0.trace_roots(visitor);
    }
}

impl<A: FrameRootTrace, B: FrameRootTrace> FrameRootTrace for (A, B) {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.0.trace_roots(visitor);
        self.1.trace_roots(visitor);
    }
}

impl<A: FrameRootTrace, B: FrameRootTrace, C: FrameRootTrace> FrameRootTrace for (A, B, C) {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.0.trace_roots(visitor);
        self.1.trace_roots(visitor);
        self.2.trace_roots(visitor);
    }
}

impl<A: FrameRootTrace, B: FrameRootTrace, C: FrameRootTrace, D: FrameRootTrace> FrameRootTrace
    for (A, B, C, D)
{
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.0.trace_roots(visitor);
        self.1.trace_roots(visitor);
        self.2.trace_roots(visitor);
        self.3.trace_roots(visitor);
    }
}

impl<A: FrameRootTrace, B: FrameRootTrace, C: FrameRootTrace, D: FrameRootTrace, E: FrameRootTrace>
    FrameRootTrace for (A, B, C, D, E)
{
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.0.trace_roots(visitor);
        self.1.trace_roots(visitor);
        self.2.trace_roots(visitor);
        self.3.trace_roots(visitor);
        self.4.trace_roots(visitor);
    }
}

impl<
    A: FrameRootTrace,
    B: FrameRootTrace,
    C: FrameRootTrace,
    D: FrameRootTrace,
    E: FrameRootTrace,
    F: FrameRootTrace,
> FrameRootTrace for (A, B, C, D, E, F)
{
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.0.trace_roots(visitor);
        self.1.trace_roots(visitor);
        self.2.trace_roots(visitor);
        self.3.trace_roots(visitor);
        self.4.trace_roots(visitor);
        self.5.trace_roots(visitor);
    }
}

impl<
    A: FrameRootTrace,
    B: FrameRootTrace,
    C: FrameRootTrace,
    D: FrameRootTrace,
    E: FrameRootTrace,
    F: FrameRootTrace,
    G: FrameRootTrace,
> FrameRootTrace for (A, B, C, D, E, F, G)
{
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.0.trace_roots(visitor);
        self.1.trace_roots(visitor);
        self.2.trace_roots(visitor);
        self.3.trace_roots(visitor);
        self.4.trace_roots(visitor);
        self.5.trace_roots(visitor);
        self.6.trace_roots(visitor);
    }
}

impl<
    A: FrameRootTrace,
    B: FrameRootTrace,
    C: FrameRootTrace,
    D: FrameRootTrace,
    E: FrameRootTrace,
    F: FrameRootTrace,
    G: FrameRootTrace,
    H: FrameRootTrace,
> FrameRootTrace for (A, B, C, D, E, F, G, H)
{
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        self.0.trace_roots(visitor);
        self.1.trace_roots(visitor);
        self.2.trace_roots(visitor);
        self.3.trace_roots(visitor);
        self.4.trace_roots(visitor);
        self.5.trace_roots(visitor);
        self.6.trace_roots(visitor);
        self.7.trace_roots(visitor);
    }
}

impl FrameRootTrace for () {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for bool {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for char {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for i8 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for i16 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for i32 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for i64 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for i128 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for isize {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for u8 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for u16 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for u32 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for u64 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for u128 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for usize {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for f32 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for f64 {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for String {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for str {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for BigInt {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for Rational {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for RandomState {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for SyntaxError {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for Ordering {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for Instant {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for Duration {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for BuiltinFn {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

impl FrameRootTrace for fn(&Expr) -> bool {
    fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
}

#[cfg(test)]
pub(in crate::runtime) const MANUAL_FRAME_ROOT_TRACE_IMPLS: &[&str] = &[
    "&T",
    "&mutT",
    "()",
    "(A,)",
    "(A,B)",
    "(A,B,C)",
    "(A,B,C,D)",
    "(A,B,C,D,E)",
    "(A,B,C,D,E,F)",
    "(A,B,C,D,E,F,G)",
    "(A,B,C,D,E,F,G,H)",
    "Arc<T>",
    "Arc<[T]>",
    "AtomicUsize",
    "BTreeMap<K,V>",
    "BTreeSet<T>",
    "BigInt",
    "Box<T>",
    "BuiltinFn",
    "Duration",
    "Expr",
    "HashMap<K,V,S>",
    "HashSet<T,S>",
    "Instant",
    "OnceLock<T>",
    "Option<T>",
    "Ordering",
    "PreparedDefinitionLhsCallback",
    "RandomState",
    "Rational",
    "Result<T,E>",
    "SharedArgs",
    "SharedExpr",
    "String",
    "Symbol",
    "SyntaxError",
    "Vec<T>",
    "VecDeque<T>",
    "WeakSharedArgs",
    "[T;N]",
    "[T]",
    "bool",
    "char",
    "f32",
    "f64",
    "fn(&Expr)->bool",
    "i128",
    "i16",
    "i32",
    "i64",
    "i8",
    "isize",
    "std::vec::IntoIter<T>",
    "str",
    "u128",
    "u16",
    "u32",
    "u64",
    "u8",
    "usize",
];
