//! Source identities and discovery facts. There are deliberately no dispositions.

use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;
use quote::ToTokens;

pub(super) type Id = String;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Location {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn new(file: &str, span: Span) -> Self {
        let start = span.start();
        Self {
            file: file.into(),
            line: start.line,
            column: start.column + 1,
        }
    }

    pub fn display(&self) -> String {
        format!("{}:{}:{}", self.file, self.line, self.column)
    }
}

pub(super) fn spelling(value: &impl ToTokens) -> String {
    value.to_token_stream().to_string()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum UnknownKind {
    UnknownHostPayload,
    UnknownConstruct,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Unknown {
    pub kind: UnknownKind,
    pub location: Location,
    pub construct: String,
}

impl Unknown {
    pub fn row(&self) -> String {
        format!(
            "{:?}\t{}\t{}\t{}\t{}",
            self.kind, self.location.file, self.location.line, self.location.column, self.construct
        )
    }
}

#[derive(Clone)]
pub(super) struct Field {
    pub path: String,
    pub ty: syn::Type,
    pub location: Location,
    pub conditional: bool,
}

#[derive(Clone)]
pub(super) struct Definition {
    pub scope: Id,
    pub location: Location,
    pub generics: syn::Generics,
    pub fields: Vec<Field>,
    pub alias: Option<syn::Type>,
}

// A visibility boundary is the module whose descendants can see the binding;
// None means public. Lexical block scopes also carry the enclosing module.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Binding {
    pub target: Id,
    pub boundary: Option<Id>,
    pub priority: u8,
}

#[derive(Clone)]
pub(super) struct Import {
    pub path: Vec<String>,
    pub name: Option<String>,
    pub boundary: Option<Id>,
}

#[derive(Default)]
pub(super) struct Scope {
    pub module: Id,
    pub lexical_parent: Option<Id>,
    pub bindings: BTreeMap<String, BTreeSet<Binding>>,
    pub imports: Vec<Import>,
}

pub(super) struct DropImpl {
    pub scope: Id,
    pub ty: syn::Type,
    pub trait_path: syn::Path,
    pub generics: syn::Generics,
    pub positive: bool,
    pub conditional: bool,
    pub location: Location,
}

pub(super) struct Boundary {
    pub scope: Id,
    pub location: Location,
    pub name: String,
    pub ty: syn::Type,
    pub parameters: BTreeMap<String, String>,
}

#[derive(Default)]
pub(super) struct Source {
    pub definitions: BTreeMap<Id, Definition>,
    pub scopes: BTreeMap<Id, Scope>,
    pub modules: BTreeSet<Id>,
    pub unknowns: BTreeSet<Unknown>,
    pub drops: Vec<DropImpl>,
    pub boundaries: Vec<Boundary>,
    pub files: BTreeSet<String>,
    pub test_files: BTreeSet<String>,
}

impl Source {
    pub fn unknown(&mut self, location: Location, construct: String) {
        self.unknowns.insert(Unknown {
            kind: UnknownKind::UnknownConstruct,
            location,
            construct,
        });
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Edge {
    pub target: Id,
    pub witness: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Cycle {
    pub id: Id,
    pub location: Location,
    pub witness: String,
}

impl Cycle {
    pub fn row(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.id, self.location.file, self.location.line, self.witness
        )
    }
}

pub(super) struct Discovery {
    pub source: Source,
    pub edges: BTreeMap<Id, BTreeSet<Edge>>,
    pub cycles: Vec<Cycle>,
    pub drops: BTreeMap<Id, Vec<Location>>,
    pub unknowns: BTreeSet<Unknown>,
}
