//! Shared AST facts and lexical scanner facade.
use proc_macro2::Span;
use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet};

mod discovery;
mod projection;
mod sites;
pub use discovery::{files, normalize, scan};
pub use projection::resolution_files;

pub fn text(value: &impl ToTokens) -> String {
    value
        .to_token_stream()
        .to_string()
        .replace(['\t', '\n', '\r'], " ")
}

#[derive(Clone, Debug)]
pub struct Site {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}
impl Site {
    fn new(file: &str, span: Span) -> Self {
        Self {
            file: file.into(),
            line: span.start().line,
            column: span.start().column + 1,
            end_line: span.end().line,
            end_column: span.end().column + 1,
        }
    }
    pub fn key(&self) -> (String, usize, usize) {
        (self.file.clone(), self.line, self.column)
    }
    pub fn fields(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}",
            self.file, self.line, self.column, self.end_line, self.end_column
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Population {
    Production,
    TestOnly,
    TestSupport,
}
impl Population {
    pub fn label(self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::TestOnly => "test-only",
            Self::TestSupport => "test-support",
        }
    }
}

#[derive(Clone)]
pub struct Function {
    pub identity: String,
    pub scope: String,
    pub owner: String,
    pub trait_name: String,
    pub name: String,
    pub signature: String,
    pub signature_ast: syn::Signature,
    pub body: syn::Block,
    pub cold: bool,
    pub site: Site,
    pub test: bool,
    pub population: Population,
    pub boundary: Option<String>,
}
#[derive(Clone)]
pub struct Use {
    pub caller: String,
    pub scope: String,
    pub kind: String,
    pub name: String,
    pub target: String,
    pub receiver: String,
    pub site: Site,
    pub test: bool,
    pub population: Population,
    pub ordinal: usize,
    pub shadowed: bool,
}
#[derive(Clone)]
pub struct Implementation {
    pub site: Site,
    pub input: String,
    pub output: String,
    pub mode: String,
    pub population: Population,
    pub scope: String,
    pub trait_path: String,
}
#[derive(Default)]
pub struct Facts {
    pub functions: Vec<Function>,
    pub uses: Vec<Use>,
    pub implementations: BTreeMap<(String, usize, usize), Implementation>,
    pub unresolved: Vec<String>,
    pub visited: BTreeSet<(String, String, bool)>,
    pub guards: Vec<Site>,
    pub guard_modules: BTreeMap<(String, usize, usize), String>,
    pub aliases: BTreeMap<String, String>,
}
