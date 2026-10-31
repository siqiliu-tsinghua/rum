//! Identities, located diagnostics and the finite ownership domain.

use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;

pub(super) type Id = String;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Site {
    pub file: String,
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl Site {
    pub fn new(file: &str, span: Span) -> Self {
        Self {
            file: file.into(),
            start: (span.start().line, span.start().column + 1),
            end: (span.end().line, span.end().column + 1),
        }
    }

    pub fn display(&self) -> String {
        format!(
            "{}:{}:{}-{}:{}",
            self.file, self.start.0, self.start.1, self.end.0, self.end.1
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum Kind {
    RET001,
    RET002,
    RET003,
    RET004,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Diagnostic {
    pub kind: Kind,
    pub site: Site,
    pub target: Option<Id>,
    pub witness: Vec<Id>,
    pub reason: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}\t{}\t{}\t{}\t{}",
            self.kind,
            self.site.display(),
            self.target.as_deref().unwrap_or("-"),
            self.witness.join(" -> "),
            self.reason
        )
    }
}

pub(super) type Checked<T> = Result<T, Diagnostic>;

pub(super) fn tokens(value: &impl ToTokens) -> String {
    value.to_token_stream().to_string()
}

pub(super) fn ast<T: syn::parse::Parse>(text: &str) -> T {
    syn::parse_str(text).unwrap()
}

pub(super) fn attr(text: &str) -> syn::Attribute {
    let attrs = syn::parse::Parser::parse_str(syn::Attribute::parse_outer, text).unwrap();
    assert_eq!(attrs.len(), 1);
    attrs.into_iter().next().unwrap()
}

pub(super) fn block(text: &str) -> String {
    tokens(&ast::<syn::Block>(text))
}
pub(super) fn signature(text: &str) -> String {
    tokens(&ast::<syn::Signature>(text))
}
pub(super) fn type_text(text: &str) -> String {
    tokens(&ast::<syn::Type>(text))
}
pub(super) fn attribute_text(text: &str) -> String {
    tokens(&attr(text))
}

fn source_offset(source: &str, point: proc_macro2::LineColumn) -> usize {
    source
        .match_indices('\n')
        .take(point.line - 1)
        .last()
        .map_or(0, |(index, _)| index + 1)
        + point.column
}

pub(super) fn rewrite_generics(text: &str, replacement: &str) -> String {
    let signature = ast::<syn::Signature>(text);
    let mut output = text.to_owned();
    if signature.generics.params.is_empty() && signature.generics.where_clause.is_none() {
        if replacement.is_empty() {
            return output;
        }
        output.insert_str(
            source_offset(text, signature.ident.span().end()),
            replacement,
        );
        return output;
    }
    let span = signature.generics.span();
    let range = source_offset(text, span.start())..source_offset(text, span.end());
    let replacement = if replacement.is_empty() {
        output[range.clone()]
            .chars()
            .map(|c| if c == '\n' { '\n' } else { ' ' })
            .collect::<String>()
    } else {
        replacement.to_owned()
    };
    output.replace_range(range, &replacement);
    output
}

pub(super) fn without_generics(text: &str) -> String {
    rewrite_generics(text, "")
}

pub(super) fn located<T: Spanned + ToTokens>(
    value: &T,
    file: &str,
    files: &BTreeMap<String, String>,
) -> String {
    if value.to_token_stream().to_string().is_empty() {
        return String::new();
    }
    let span = value.span();
    let source = &files[file];
    let text = &source[source_offset(source, span.start())..source_offset(source, span.end())];
    let start = span.start();
    format!(
        "{}{}{}",
        "\n".repeat(start.line - 1),
        " ".repeat(start.column),
        text
    )
}

#[derive(Clone)]
pub(super) struct Function {
    pub id: Id,
    pub module: Id,
    pub site: Site,
    pub sig: String,
    pub body: String,
    pub attrs: Vec<String>,
    pub owner: Option<Id>,
    pub boundary: Option<Id>,
    pub impl_self: Option<String>,
    pub impl_generics: String,
    pub substitutions: BTreeMap<String, ExactTy>,
    pub conditional: bool,
}

impl Function {
    pub fn sig(&self) -> syn::Signature {
        ast(&self.sig)
    }
    pub fn body(&self) -> syn::Block {
        ast(&self.body)
    }
    pub fn attrs(&self) -> Vec<syn::Attribute> {
        self.attrs.iter().map(|a| attr(a)).collect()
    }
    pub fn impl_self(&self) -> Option<syn::Type> {
        self.impl_self.as_ref().map(|ty| ast(ty))
    }
    pub fn impl_generics(&self) -> syn::Generics {
        ast(&self.impl_generics)
    }
}

#[derive(Clone)]
pub(super) struct Trait {
    pub item: String,
    pub conditional: bool,
}

impl Trait {
    pub fn item(&self) -> syn::ItemTrait {
        ast(&self.item)
    }
}

#[derive(Clone)]
pub(super) struct Nominal {
    pub module: Id,
    pub site: Site,
    pub generics: String,
    pub fields: Vec<Field>,
    pub variants: BTreeMap<String, Vec<Field>>,
    pub alias: Option<String>,
    pub attrs: Vec<String>,
    pub conditional: bool,
}

impl Nominal {
    pub fn generics(&self) -> syn::Generics {
        ast(&self.generics)
    }
    pub fn alias(&self) -> Option<syn::Type> {
        self.alias.as_ref().map(|ty| ast(ty))
    }
    pub fn attrs(&self) -> Vec<syn::Attribute> {
        self.attrs.iter().map(|a| attr(a)).collect()
    }
}

#[derive(Clone)]
pub(super) struct Field {
    pub name: String,
    pub ty: String,
    pub conditional: bool,
}

impl Field {
    pub fn ty(&self) -> syn::Type {
        ast(&self.ty)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Binding {
    pub target: Id,
    pub boundary: Option<Id>,
    pub glob: bool,
    pub conditional: bool,
}

#[derive(Clone)]
pub(super) struct Import {
    pub path: Vec<String>,
    pub name: Option<String>,
    pub boundary: Option<Id>,
    pub conditional: bool,
}

#[derive(Clone, Default)]
pub(super) struct Module {
    pub bindings: BTreeMap<String, BTreeSet<Binding>>,
    pub imports: Vec<Import>,
    pub hazards: Vec<Site>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Shared<T>(std::sync::Arc<T>);

impl<T> Shared<T> {
    fn new(value: T) -> Self {
        Self(std::sync::Arc::new(value))
    }
}

impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> std::ops::Deref for Shared<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Clone> std::ops::DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T {
        std::sync::Arc::make_mut(&mut self.0)
    }
}

impl<A, T: FromIterator<A>> FromIterator<A> for Shared<T> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        Self::new(T::from_iter(iter))
    }
}

impl<T: IntoIterator + Clone> IntoIterator for Shared<T> {
    type Item = T::Item;
    type IntoIter = T::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        match std::sync::Arc::try_unwrap(self.0) {
            Ok(value) => value.into_iter(),
            Err(shared) => shared.as_ref().clone().into_iter(),
        }
    }
}

impl<'a, T> IntoIterator for &'a Shared<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.as_ref().into_iter()
    }
}

impl<'a, T: Clone> IntoIterator for &'a mut Shared<T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        std::sync::Arc::make_mut(&mut self.0).into_iter()
    }
}

#[derive(Clone, Default)]
pub(super) struct Source {
    pub functions: Shared<BTreeMap<Id, Vec<Function>>>,
    pub types: Shared<BTreeMap<Id, Vec<Nominal>>>,
    pub modules: Shared<BTreeMap<Id, Module>>,
    pub impls: Shared<Vec<(Id, String, Site, bool)>>,
    pub traits: Shared<BTreeMap<Id, Vec<Trait>>>,
    pub trait_impls: Shared<Vec<(Id, String, Site, bool)>>,
    pub opaque_inherent_owners: Shared<BTreeSet<Id>>,
    pub constants: Shared<Vec<(Id, String, String)>>,
    pub value_names: Shared<BTreeSet<Id>>,
    pub drop_types: Shared<BTreeSet<Id>>,
    pub dependencies: Shared<BTreeMap<String, BTreeSet<String>>>,
    pub files: Shared<BTreeMap<String, String>>,
    pub reached: Shared<BTreeSet<String>>,
    pub production_files: Shared<BTreeSet<String>>,
    pub errors: Shared<Vec<String>>,
}

/// Rust identity, independent of whether cleanup or retention is proved.
/// ADT identity + arguments and lazy field substitution follow rustc's TyKind.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum ExactTy {
    Unit,
    Scalar(Id),
    Nominal(Id, Vec<ExactTy>),
    Tuple(Vec<ExactTy>),
    Reference(Box<ExactTy>, bool),
    Slice(Box<ExactTy>),
    Array(Box<ExactTy>, usize),
    Function(Id),
}

impl ExactTy {
    pub fn spelling(&self) -> String {
        match self {
            Self::Unit => "()".into(),
            Self::Scalar(id) | Self::Function(id) => id.clone(),
            Self::Nominal(id, args) if args.is_empty() => id.clone(),
            Self::Nominal(id, args) => format!(
                "{id}<{}>",
                args.iter()
                    .map(Self::spelling)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Tuple(args) => format!(
                "({},)",
                args.iter()
                    .map(Self::spelling)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Reference(inner, mutable) => format!(
                "&{}{}",
                if *mutable { "mut " } else { "" },
                inner.spelling()
            ),
            Self::Slice(inner) => format!("[{}]", inner.spelling()),
            Self::Array(inner, len) => format!("[{}; {len}]", inner.spelling()),
        }
    }
}

/// Historical name retained for the existing region contracts. This is ONLY
/// the retention domain; equality here never establishes Rust type equality.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Ty {
    Unit,
    Scalar(String),
    Tuple(Vec<Ty>),
    Fixed(Id),
    Raw(Id),
    Borrow(Box<Ty>, bool),
    Retained(Id),
    Callback(Id),
    Outgoing(Id),
    Fields(BTreeMap<String, Ty>),
    Variants(BTreeMap<String, BTreeMap<String, Ty>>),
    Unchecked(String),
    Moved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Provenance {
    pub source: Site,
    pub basis: String,
    pub projections: Vec<String>,
}

impl Ty {
    pub fn copyable(&self) -> bool {
        matches!(self, Self::Unit | Self::Scalar(_) | Self::Borrow(_, false))
            || matches!(self, Self::Tuple(ts) if ts.iter().all(Self::copyable))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Value {
    pub exact: ExactTy,
    pub ty: Ty,
    pub origin: Option<Id>,
    pub provenance: Option<Provenance>,
}

impl Value {
    pub fn new(ty: Ty) -> Self {
        let exact = match &ty {
            Ty::Unit => ExactTy::Unit,
            Ty::Scalar(id) => ExactTy::Scalar(id.clone()),
            _ => panic!("nonprimitive value requires an independent exact type"),
        };
        Self::typed(exact, ty)
    }

    pub fn typed(exact: ExactTy, ty: Ty) -> Self {
        Self {
            exact,
            ty,
            origin: None,
            provenance: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Local {
    pub value: Value,
    pub live: bool,
}

#[derive(Clone, Debug)]
pub(super) struct Fact {
    pub site: Site,
    pub ty: Ty,
    pub exact: Option<ExactTy>,
    pub provenance: Option<Provenance>,
    pub effect: &'static str,
}

#[derive(Default)]
pub(super) struct Proof {
    pub bodies: BTreeSet<Id>,
    pub instances: BTreeSet<LeafKey>,
    pub leaves: BTreeSet<LeafKey>,
    pub facts: Vec<Fact>,
    pub loop_invariants: usize,
    pub adoptions: usize,
    pub transfers: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct LeafKey {
    pub id: Id,
    pub instance: Vec<ExactTy>,
}

impl LeafKey {
    pub fn display(&self) -> String {
        if self.instance.is_empty() {
            self.id.clone()
        } else {
            format!(
                "{}::<{}>",
                self.id,
                self.instance
                    .iter()
                    .map(ExactTy::spelling)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}
