use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::NonZeroU64;
use std::ops::Deref;
use std::sync::{Arc, Mutex, OnceLock, Weak};

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

use crate::profiling::{
    record_into_owned, record_into_vec, record_shared_args_alloc, record_shared_expr_alloc,
};

pub const GLOBAL_CONTEXT: &str = "Global`";
pub const SYSTEM_CONTEXT: &str = "System`";

/// Interned `(context, name)` pair shared by every symbol with that text.
/// One instance is leaked per distinct pair, so pointer identity implies
/// text identity.
#[derive(Debug)]
pub struct SymbolText {
    id: u32,
    context: &'static str,
    name: &'static str,
}

/// A symbol referencing interned context/name text.
///
/// Equality and hashing operate on the interned text pointer/id plus the
/// optional generated identity, so they are integer operations independent
/// of name length. Ordering remains lexicographic over
/// `(context, name, identity)` to preserve canonical ordering semantics.
#[derive(Clone, Copy, Debug)]
pub struct Symbol {
    id: u32,
    identity: Option<NonZeroU64>,
    text: &'static SymbolText,
}

struct SymbolInterner {
    strings: HashSet<&'static str>,
    pairs: HashMap<(usize, usize), &'static SymbolText>,
}

fn symbol_interner() -> &'static Mutex<SymbolInterner> {
    static SYMBOL_INTERNER: OnceLock<Mutex<SymbolInterner>> = OnceLock::new();
    SYMBOL_INTERNER.get_or_init(|| {
        Mutex::new(SymbolInterner {
            strings: HashSet::new(),
            pairs: HashMap::new(),
        })
    })
}

fn intern_text(strings: &mut HashSet<&'static str>, value: &str) -> &'static str {
    if let Some(canonical) = canonical_static_str(value) {
        strings.insert(canonical);
        return canonical;
    }

    if let Some(existing) = strings.get(value) {
        existing
    } else {
        let leaked: &'static str = Box::leak(value.to_owned().into_boxed_str());
        strings.insert(leaked);
        leaked
    }
}

fn canonical_static_str(value: &str) -> Option<&'static str> {
    match value {
        GLOBAL_CONTEXT => Some(GLOBAL_CONTEXT),
        SYSTEM_CONTEXT => Some(SYSTEM_CONTEXT),
        _ => None,
    }
}

fn intern_symbol_text(context: &str, name: &str) -> &'static SymbolText {
    let mut interner = symbol_interner()
        .lock()
        .expect("symbol interner lock should not be poisoned");
    let SymbolInterner { strings, pairs } = &mut *interner;
    let context = intern_text(strings, context);
    let name = intern_text(strings, name);
    let key = (context.as_ptr() as usize, name.as_ptr() as usize);
    let next_id = u32::try_from(pairs.len()).expect("interned symbol count should fit in u32");
    pairs.entry(key).or_insert_with(|| {
        Box::leak(Box::new(SymbolText {
            id: next_id,
            context,
            name,
        }))
    })
}

fn canonical_system_symbols() -> &'static HashMap<&'static str, Symbol> {
    static CANONICAL_SYSTEM_SYMBOLS: OnceLock<HashMap<&'static str, Symbol>> = OnceLock::new();
    CANONICAL_SYSTEM_SYMBOLS.get_or_init(|| {
        crate::heads::CANONICAL_CORE_HEADS
            .iter()
            .map(|name| (*name, Symbol::new(SYSTEM_CONTEXT, *name)))
            .collect()
    })
}

impl Symbol {
    pub fn new(context: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        let text = intern_symbol_text(context.as_ref(), name.as_ref());
        Self {
            id: text.id,
            identity: None,
            text,
        }
    }

    pub fn with_identity(context: impl AsRef<str>, name: impl AsRef<str>, identity: u64) -> Self {
        let text = intern_symbol_text(context.as_ref(), name.as_ref());
        let identity = NonZeroU64::new(identity).expect("symbol identity must be non-zero");
        Self {
            id: text.id,
            identity: Some(identity),
            text,
        }
    }

    pub fn from_name(name: impl AsRef<str>) -> Self {
        Self::new(GLOBAL_CONTEXT, name)
    }

    pub fn system(name: impl AsRef<str>) -> Self {
        if let Some(symbol) = canonical_system_symbols().get(name.as_ref()) {
            return *symbol;
        }

        Self::new(SYSTEM_CONTEXT, name)
    }

    pub fn context(&self) -> &str {
        self.text.context
    }

    pub fn name(&self) -> &str {
        self.text.name
    }

    pub fn is_global(&self) -> bool {
        self.text.context == GLOBAL_CONTEXT
    }

    pub fn is_system(&self) -> bool {
        self.text.context == SYSTEM_CONTEXT
    }

    pub fn is_system_name(&self, name: &str) -> bool {
        self.is_system() && self.text.name == name
    }

    pub fn identity(&self) -> Option<u64> {
        self.identity.map(NonZeroU64::get)
    }

    /// Dense id of the interned `(context, name)` pair. Symbols with the
    /// same text share the id even when their `identity` differs.
    pub fn interned_id(&self) -> u32 {
        self.id
    }

    pub fn fully_qualified_name(&self) -> String {
        format!("{}{}", self.text.context, self.text.name)
    }

    pub fn qualified_name(&self) -> String {
        if self.is_global() || self.is_system() {
            self.text.name.to_string()
        } else {
            self.fully_qualified_name()
        }
    }

    /// Compares the bytes rendered by [`Self::qualified_name`] without
    /// allocating either qualified name.
    ///
    /// Generated identities are deliberately ignored because they are not
    /// part of the rendered name.
    pub fn cmp_qualified_name(&self, other: &Self) -> Ordering {
        if self.id == other.id {
            return Ordering::Equal;
        }

        let lhs_context = if self.is_global() || self.is_system() {
            ""
        } else {
            self.text.context
        };
        let rhs_context = if other.is_global() || other.is_system() {
            ""
        } else {
            other.text.context
        };

        if lhs_context == rhs_context {
            return self.text.name.as_bytes().cmp(other.text.name.as_bytes());
        }

        compare_concatenated_bytes(
            [lhs_context.as_bytes(), self.text.name.as_bytes()],
            [rhs_context.as_bytes(), other.text.name.as_bytes()],
        )
    }
}

#[inline]
fn compare_concatenated_bytes(lhs: [&[u8]; 2], rhs: [&[u8]; 2]) -> Ordering {
    let mut lhs_part = 0;
    let mut rhs_part = 0;
    let mut lhs_offset = 0;
    let mut rhs_offset = 0;

    loop {
        while lhs_part < lhs.len() && lhs_offset == lhs[lhs_part].len() {
            lhs_part += 1;
            lhs_offset = 0;
        }
        while rhs_part < rhs.len() && rhs_offset == rhs[rhs_part].len() {
            rhs_part += 1;
            rhs_offset = 0;
        }

        match (lhs.get(lhs_part), rhs.get(rhs_part)) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(lhs_bytes), Some(rhs_bytes)) => {
                let lhs_remaining = &lhs_bytes[lhs_offset..];
                let rhs_remaining = &rhs_bytes[rhs_offset..];
                let compared_len = lhs_remaining.len().min(rhs_remaining.len());
                let ordering = lhs_remaining[..compared_len].cmp(&rhs_remaining[..compared_len]);
                if ordering != Ordering::Equal {
                    return ordering;
                }
                lhs_offset += compared_len;
                rhs_offset += compared_len;
            }
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.identity == other.identity
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let identity = self.identity.map(NonZeroU64::get).unwrap_or(0);
        state.write_u64((u64::from(self.id)).wrapping_mul(0x9e3779b97f4a7c15) ^ identity);
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.id == other.id {
            return self.identity.cmp(&other.identity);
        }

        let context_order = if std::ptr::eq(self.text.context, other.text.context) {
            Ordering::Equal
        } else {
            self.text.context.cmp(other.text.context)
        };
        context_order
            .then_with(|| self.text.name.cmp(other.text.name))
            .then_with(|| self.identity.cmp(&other.identity))
    }
}

impl Serialize for Symbol {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let fields = 2 + usize::from(self.identity.is_some());
        let mut state = serializer.serialize_struct("Symbol", fields)?;
        state.serialize_field("context", self.text.context)?;
        state.serialize_field("name", self.text.name)?;
        if let Some(identity) = self.identity {
            state.serialize_field("identity", &identity.get())?;
        }
        state.end()
    }
}

#[derive(Deserialize)]
#[serde(rename = "Symbol")]
struct SymbolDeserializeParts {
    context: String,
    name: String,
    #[serde(default)]
    identity: Option<u64>,
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let parts = SymbolDeserializeParts::deserialize(deserializer)?;
        Ok(match parts.identity {
            Some(identity) => Symbol::with_identity(parts.context, parts.name, identity),
            None => Symbol::new(parts.context, parts.name),
        })
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.qualified_name())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IntValue {
    Small(i64),
    Big(Box<BigInt>),
}

impl IntValue {
    pub fn from_bigint(value: BigInt) -> Self {
        match i64::try_from(&value) {
            Ok(value) => Self::Small(value),
            Err(_) => Self::Big(Box::new(value)),
        }
    }

    pub fn from_i64(value: i64) -> Self {
        Self::Small(value)
    }

    pub fn to_bigint(&self) -> BigInt {
        match self {
            Self::Small(value) => BigInt::from(*value),
            Self::Big(value) => (**value).clone(),
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Small(value) => Some(*value),
            Self::Big(_) => None,
        }
    }

    pub fn is_zero(&self) -> bool {
        matches!(self, Self::Small(0))
    }
}

impl PartialOrd for IntValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IntValue {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Small(lhs), Self::Small(rhs)) => lhs.cmp(rhs),
            _ => self.to_bigint().cmp(&other.to_bigint()),
        }
    }
}

impl Serialize for IntValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Small(value) => BigInt::from(*value).serialize(serializer),
            Self::Big(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for IntValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        BigInt::deserialize(deserializer).map(Self::from_bigint)
    }
}

impl fmt::Display for IntValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Small(value) => write!(f, "{value}"),
            Self::Big(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Rational {
    numerator: BigInt,
    denominator: BigInt,
}

impl Rational {
    pub fn new(numerator: BigInt, denominator: BigInt) -> Option<Self> {
        if denominator == BigInt::from(0_u8) {
            return None;
        }

        if numerator == BigInt::from(0_u8) {
            return Some(Self {
                numerator,
                denominator: BigInt::from(1_u8),
            });
        }

        let mut numerator = numerator;
        let mut denominator = denominator;

        if denominator < BigInt::from(0_u8) {
            numerator = -numerator;
            denominator = -denominator;
        }

        let divisor = bigint_gcd(bigint_abs(&numerator), denominator.clone());

        Some(Self {
            numerator: numerator / divisor.clone(),
            denominator: denominator / divisor,
        })
    }

    pub fn from_integer(value: BigInt) -> Self {
        Self {
            numerator: value,
            denominator: BigInt::from(1_u8),
        }
    }

    pub fn zero() -> Self {
        Self::from_integer(BigInt::from(0_u8))
    }

    pub fn one() -> Self {
        Self::from_integer(BigInt::from(1_u8))
    }

    pub fn numerator(&self) -> &BigInt {
        &self.numerator
    }

    pub fn denominator(&self) -> &BigInt {
        &self.denominator
    }

    pub fn is_zero(&self) -> bool {
        self.numerator == BigInt::from(0_u8)
    }

    pub fn is_one(&self) -> bool {
        self.numerator == BigInt::from(1_u8) && self.denominator == BigInt::from(1_u8)
    }

    pub fn is_integer(&self) -> bool {
        self.denominator == BigInt::from(1_u8)
    }

    pub fn as_integer(&self) -> Option<&BigInt> {
        self.is_integer().then_some(&self.numerator)
    }

    pub fn negated(&self) -> Self {
        Self {
            numerator: -self.numerator.clone(),
            denominator: self.denominator.clone(),
        }
    }

    pub fn reciprocal(&self) -> Option<Self> {
        Self::new(self.denominator.clone(), self.numerator.clone())
    }

    pub fn add(&self, rhs: &Self) -> Self {
        Self::new(
            &self.numerator * &rhs.denominator + &rhs.numerator * &self.denominator,
            &self.denominator * &rhs.denominator,
        )
        .expect("adding normalized rationals should keep a non-zero denominator")
    }

    pub fn sub(&self, rhs: &Self) -> Self {
        self.add(&rhs.negated())
    }

    pub fn mul(&self, rhs: &Self) -> Self {
        Self::new(
            &self.numerator * &rhs.numerator,
            &self.denominator * &rhs.denominator,
        )
        .expect("multiplying normalized rationals should keep a non-zero denominator")
    }

    pub fn div(&self, rhs: &Self) -> Option<Self> {
        Some(self.mul(&rhs.reciprocal()?))
    }

    pub fn pow(&self, exponent: &BigInt) -> Option<Self> {
        if exponent == &BigInt::from(0_u8) {
            return (!self.is_zero()).then(Self::one);
        }

        if exponent > &BigInt::from(0_u8) {
            return Some(Self::new(
                pow_bigint(&self.numerator, exponent),
                pow_bigint(&self.denominator, exponent),
            )?);
        }

        let positive = -exponent.clone();
        Some(self.reciprocal()?.pow(&positive)?)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Atom {
    Integer(IntValue),
    Rational(Box<Rational>),
    String(String),
    Symbol(Symbol),
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::Rational(value) => write!(f, "{value}"),
            Self::String(value) => write_escaped_string(f, value),
            Self::Symbol(symbol) => write!(f, "{symbol}"),
        }
    }
}

fn write_escaped_string(f: &mut fmt::Formatter<'_>, value: &str) -> fmt::Result {
    f.write_str("\"")?;

    for ch in value.chars() {
        match ch {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            other if other.is_control() => write!(f, "\\u{{{:X}}}", other as u32)?,
            other => write!(f, "{other}")?,
        }
    }

    f.write_str("\"")
}

#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct SharedExpr(Arc<Expr>);

impl SharedExpr {
    pub fn new(expr: Expr) -> Self {
        let shared = Self(Arc::new(expr));
        record_shared_expr_alloc();
        shared
    }

    pub fn into_owned(self) -> Expr {
        match Arc::try_unwrap(self.0) {
            Ok(expr) => {
                record_into_owned(false);
                expr
            }
            Err(expr) => {
                record_into_owned(true);
                expr.as_ref().clone()
            }
        }
    }

    pub fn ptr_eq(lhs: &Self, rhs: &Self) -> bool {
        Arc::ptr_eq(&lhs.0, &rhs.0)
    }

    pub fn ptr_key(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }
}

impl PartialEq for SharedExpr {
    fn eq(&self, other: &Self) -> bool {
        Self::ptr_eq(self, other) || self.as_ref() == other.as_ref()
    }
}

impl From<Expr> for SharedExpr {
    fn from(expr: Expr) -> Self {
        Self::new(expr)
    }
}

impl AsRef<Expr> for SharedExpr {
    fn as_ref(&self) -> &Expr {
        &self.0
    }
}

impl Deref for SharedExpr {
    type Target = Expr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for SharedExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_ref(), f)
    }
}

#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct SharedArgs(Arc<Vec<Expr>>);

impl SharedArgs {
    pub fn new(args: Vec<Expr>) -> Self {
        let shared = Self(Arc::new(args));
        record_shared_args_alloc();
        shared
    }

    pub fn into_vec(self) -> Vec<Expr> {
        match Arc::try_unwrap(self.0) {
            Ok(args) => {
                record_into_vec(false);
                args
            }
            Err(args) => {
                record_into_vec(true);
                args.as_ref().clone()
            }
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut Vec<Expr>> {
        Arc::get_mut(&mut self.0)
    }

    pub fn make_mut(&mut self) -> &mut Vec<Expr> {
        let original = Arc::as_ptr(&self.0);
        let args = Arc::make_mut(&mut self.0);
        if !std::ptr::eq(original, args) {
            record_shared_args_alloc();
        }
        args
    }

    pub fn ptr_eq(lhs: &Self, rhs: &Self) -> bool {
        Arc::ptr_eq(&lhs.0, &rhs.0)
    }

    pub fn ptr_key(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }

    pub fn downgrade(&self) -> WeakSharedArgs {
        WeakSharedArgs(Arc::downgrade(&self.0))
    }
}

#[derive(Clone, Debug)]
pub struct WeakSharedArgs(Weak<Vec<Expr>>);

impl WeakSharedArgs {
    pub fn ptr_eq(&self, args: &SharedArgs) -> bool {
        std::ptr::eq(self.0.as_ptr(), Arc::as_ptr(&args.0))
    }

    pub fn is_alive(&self) -> bool {
        self.0.strong_count() != 0
    }
}

impl Default for SharedArgs {
    fn default() -> Self {
        Self(Arc::new(Vec::new()))
    }
}

impl PartialEq for SharedArgs {
    fn eq(&self, other: &Self) -> bool {
        Self::ptr_eq(self, other) || self.as_ref() == other.as_ref()
    }
}

impl From<Vec<Expr>> for SharedArgs {
    fn from(args: Vec<Expr>) -> Self {
        Self::new(args)
    }
}

impl AsRef<[Expr]> for SharedArgs {
    fn as_ref(&self) -> &[Expr] {
        self.0.as_slice()
    }
}

impl Deref for SharedArgs {
    type Target = Vec<Expr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for SharedArgs {
    type Item = Expr;
    type IntoIter = std::vec::IntoIter<Expr>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}

impl<'a> IntoIterator for &'a SharedArgs {
    type Item = &'a Expr;
    type IntoIter = std::slice::Iter<'a, Expr>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub enum Expr {
    Atom(Atom),
    Call { head: SharedExpr, args: SharedArgs },
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Atom(lhs), Self::Atom(rhs)) => lhs == rhs,
            (
                Self::Call {
                    head: lhs_head,
                    args: lhs_args,
                },
                Self::Call {
                    head: rhs_head,
                    args: rhs_args,
                },
            ) => {
                (SharedExpr::ptr_eq(lhs_head, rhs_head) || lhs_head.as_ref() == rhs_head.as_ref())
                    && (SharedArgs::ptr_eq(lhs_args, rhs_args)
                        || lhs_args.as_ref() == rhs_args.as_ref())
            }
            _ => false,
        }
    }
}

impl Eq for Expr {}

impl Expr {
    pub fn integer(value: BigInt) -> Self {
        Self::Atom(Atom::Integer(IntValue::from_bigint(value)))
    }

    pub fn small_integer(value: i64) -> Self {
        Self::Atom(Atom::Integer(IntValue::from_i64(value)))
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::Atom(Atom::String(value.into()))
    }

    pub fn rational(value: Rational) -> Self {
        if let Some(integer) = value.as_integer() {
            Self::integer(integer.clone())
        } else {
            Self::Atom(Atom::Rational(Box::new(value)))
        }
    }

    pub fn symbol(name: impl AsRef<str>) -> Self {
        Self::Atom(Atom::Symbol(Symbol::from_name(name)))
    }

    pub fn contextual_symbol(context: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        Self::Atom(Atom::Symbol(Symbol::new(context, name)))
    }

    pub fn system_symbol(name: impl AsRef<str>) -> Self {
        Self::Atom(Atom::Symbol(Symbol::system(name)))
    }

    pub fn call(head: Expr, args: Vec<Expr>) -> Self {
        Self::Call {
            head: SharedExpr::new(head),
            args: SharedArgs::new(args),
        }
    }

    pub fn call_shared(head: SharedExpr, args: SharedArgs) -> Self {
        Self::Call { head, args }
    }

    pub fn call_with_shared_args(head: Expr, args: SharedArgs) -> Self {
        Self::Call {
            head: SharedExpr::new(head),
            args,
        }
    }

    pub fn unwrap_shared_expr(expr: SharedExpr) -> Self {
        expr.into_owned()
    }

    pub fn unwrap_shared_args(args: SharedArgs) -> Vec<Expr> {
        args.into_vec()
    }

    pub fn into_call_parts(self) -> Option<(Self, Vec<Self>)> {
        match self {
            Self::Call { head, args } => Some((
                Self::unwrap_shared_expr(head),
                Self::unwrap_shared_args(args),
            )),
            Self::Atom(_) => None,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atom(atom) => write!(f, "{atom}"),
            Self::Call { head, args } => {
                if let Some(rendered) = format_message_name_call(head, args) {
                    return f.write_str(&rendered);
                }

                write!(f, "{head}[")?;

                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }

                    write!(f, "{arg}")?;
                }

                f.write_str("]")
            }
        }
    }
}

fn format_message_name_call(head: &Expr, args: &[Expr]) -> Option<String> {
    let Expr::Atom(Atom::Symbol(symbol)) = head else {
        return None;
    };

    if symbol.context() != SYSTEM_CONTEXT || symbol.name() != "MessageName" || args.len() != 2 {
        return None;
    }

    let tag = match &args[1] {
        Expr::Atom(Atom::String(tag)) => tag.clone(),
        Expr::Atom(Atom::Symbol(tag)) => tag.name().to_string(),
        _ => return None,
    };

    Some(format!("{}::{}", args[0], tag))
}

fn bigint_abs(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0_u8) {
        -value.clone()
    } else {
        value.clone()
    }
}

fn bigint_gcd(mut lhs: BigInt, mut rhs: BigInt) -> BigInt {
    let zero = BigInt::from(0_u8);

    while rhs != zero {
        let remainder = &lhs % &rhs;
        lhs = rhs;
        rhs = remainder;
    }

    bigint_abs(&lhs)
}

fn pow_bigint(base: &BigInt, exponent: &BigInt) -> BigInt {
    let zero = BigInt::from(0_u8);
    let one = BigInt::from(1_u8);
    let two = BigInt::from(2_u8);
    let mut result = BigInt::from(1_u8);
    let mut factor = base.clone();
    let mut remaining = exponent.clone();

    while remaining > zero {
        if &remaining % &two == one {
            result *= &factor;
        }

        remaining /= &two;

        if remaining > zero {
            factor = &factor * &factor;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_global_symbol_without_context_prefix() {
        let symbol = Symbol::from_name("x");
        assert_eq!(symbol.to_string(), "x");
    }

    #[test]
    fn displays_system_symbol_without_context_prefix() {
        let symbol = Symbol::system("If");
        assert_eq!(symbol.to_string(), "If");
        assert_eq!(symbol.fully_qualified_name(), "System`If");
    }

    #[test]
    fn displays_nested_call_expression() {
        let expr = Expr::call(
            Expr::symbol("f"),
            vec![Expr::integer(BigInt::from(42_u32)), Expr::symbol("x")],
        );

        assert_eq!(expr.to_string(), "f[42, x]");
    }

    #[test]
    fn normalizes_rational_atoms() {
        let rational = Rational::new(BigInt::from(6_i32), BigInt::from(-8_i32)).unwrap();

        assert_eq!(rational.numerator().to_string(), "-3");
        assert_eq!(rational.denominator().to_string(), "4");
        assert_eq!(Expr::rational(rational).to_string(), "-3/4");
    }

    #[test]
    fn normalizes_int_values_and_orders_by_numeric_value() {
        let small = IntValue::from_bigint(BigInt::from(i64::MAX));
        assert!(matches!(small, IntValue::Small(value) if value == i64::MAX));

        let big_positive = IntValue::from_bigint(BigInt::from(i64::MAX) + BigInt::from(1_u8));
        assert!(matches!(big_positive, IntValue::Big(_)));

        let big_negative = IntValue::from_bigint(BigInt::from(i64::MIN) - BigInt::from(1_u8));
        assert!(matches!(big_negative, IntValue::Big(_)));
        assert!(big_negative < IntValue::from_i64(0));
        assert!(big_positive > IntValue::from_i64(0));
    }

    #[test]
    fn displays_message_name_shorthand() {
        let expr = Expr::call(
            Expr::system_symbol("MessageName"),
            vec![Expr::symbol("f"), Expr::string("argx")],
        );

        assert_eq!(expr.to_string(), "f::argx");
    }

    #[test]
    fn displays_string_atoms_with_escaped_special_characters() {
        let expr = Expr::string("he said \"hi\"\n\\");
        assert_eq!(expr.to_string(), "\"he said \\\"hi\\\"\\n\\\\\"");
    }
}
