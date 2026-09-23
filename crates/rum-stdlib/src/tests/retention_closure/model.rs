//! Identities, located diagnostics and the finite ownership domain.

use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;
use quote::ToTokens;

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

#[derive(Clone)]
pub(super) struct Function {
    pub id: Id,
    pub module: Id,
    pub site: Site,
    pub sig: syn::Signature,
    pub body: syn::Block,
    pub attrs: Vec<syn::Attribute>,
    pub owner: Option<Id>,
    pub conditional: bool,
}

#[derive(Clone)]
pub(super) struct Nominal {
    pub module: Id,
    pub site: Site,
    pub generics: syn::Generics,
    pub fields: Vec<Field>,
    pub variants: BTreeMap<String, Vec<Field>>,
    pub alias: Option<syn::Type>,
    pub attrs: Vec<syn::Attribute>,
    pub conditional: bool,
}

#[derive(Clone)]
pub(super) struct Field {
    pub name: String,
    pub ty: syn::Type,
    pub conditional: bool,
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

#[derive(Clone, Default)]
pub(super) struct Source {
    pub functions: BTreeMap<Id, Vec<Function>>,
    pub types: BTreeMap<Id, Vec<Nominal>>,
    pub modules: BTreeMap<Id, Module>,
    pub impls: Vec<(Id, syn::ItemImpl, Site, bool)>,
    pub constants: Vec<(Id, String, syn::Expr)>,
    pub drop_types: BTreeSet<Id>,
    pub dependencies: BTreeMap<String, BTreeSet<String>>,
    pub files: BTreeMap<String, String>,
    pub reached: BTreeSet<String>,
    pub production_files: BTreeSet<String>,
    pub errors: Vec<String>,
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
