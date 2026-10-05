//! Flat work records. No record owns another record through a parent/child link.
use rum_core::{Expr, SharedArgs};
use rum_runtime::kernel_api::{EvalError, FrameRootTrace, FrameRootVisitor};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FrameRootTrace)]
pub(crate) struct SchemaOptions {
    pub(crate) allow_extra_keys: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(crate) struct SchemaError {
    pub(crate) path: String,
    pub(crate) message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(crate) struct SchemaEntry {
    pub(crate) key: String,
    pub(crate) spec: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EntrySpec<'a> {
    Required(&'a Expr),
    Optional {
        spec: &'a Expr,
        default: Option<&'a Expr>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(crate) struct SchemaPath {
    pub(crate) rendered: String,
}

#[derive(Debug)]
pub(crate) enum SchemaAbort {
    Malformed { path: String, message: String },
    Eval(EvalError),
}

#[derive(Clone, Copy, FrameRootTrace)]
pub(crate) enum SchemaMode {
    Errors,
    Conform,
}

impl SchemaMode {
    pub(super) fn symbol(self) -> &'static str {
        match self {
            Self::Errors => "SchemaErrors",
            Self::Conform => "ConformSchema",
        }
    }
}

#[derive(Clone, FrameRootTrace)]
pub(crate) struct SchemaState<Index = HashMap<String, usize>> {
    pub(crate) mode: SchemaMode,
    pub(crate) options: SchemaOptions,
    pub(crate) stack: Vec<SchemaContainer<Index>>,
    pub(crate) path: SchemaPath,
    pub(crate) errors: Vec<SchemaError>,
}

#[derive(Clone, FrameRootTrace)]
pub(crate) enum SchemaContainer<Index = HashMap<String, usize>> {
    List(ListWork),
    Association(AssociationWork<Index>),
}

#[derive(Clone, FrameRootTrace)]
pub(crate) struct ListWork {
    pub(crate) items: SharedArgs,
    pub(crate) item_schema: Expr,
    pub(crate) next: usize,
    pub(crate) output: Vec<Expr>,
    pub(crate) path_len: usize,
    pub(crate) waiting: Option<SchemaDestination>,
}

#[derive(Clone, FrameRootTrace)]
pub(crate) struct AssociationWork<Index = HashMap<String, usize>> {
    pub(crate) schema_entries: Vec<SchemaEntry>,
    pub(crate) value_entries: Vec<(Expr, Expr)>,
    pub(crate) next_schema: usize,
    pub(crate) matched: Vec<bool>,
    pub(crate) conformed_by_index: Vec<Option<Expr>>,
    pub(crate) default_entries: Vec<(Expr, Expr)>,
    pub(crate) positions: Option<Index>,
    pub(crate) path_len: usize,
    pub(crate) waiting: Option<SchemaDestination>,
}

#[derive(Clone, FrameRootTrace)]
pub(crate) enum SchemaDestination {
    ListItem,
    ExistingEntry { index: usize },
    DefaultEntry { key: String },
}

#[derive(Clone, FrameRootTrace)]
pub(crate) enum SchemaAwait {
    Leaf { value: Expr, schema: Expr },
    Default { spec: Expr },
}

// Live cursor only. Default separates child selection from evaluation, so no
// container reference survives a callback. Only SchemaAwait is ever frozen.
pub(super) enum SchemaTask {
    Visit { value: Expr, schema: Expr },
    Default { value: Expr, schema: Expr },
    Next,
    ChildDone { value: Expr },
}

pub(super) struct SchemaDriveError {
    pub(super) abort: SchemaAbort,
    pub(super) pending: Option<SchemaAwait>,
}

impl From<SchemaAbort> for SchemaDriveError {
    fn from(abort: SchemaAbort) -> Self {
        Self {
            abort,
            pending: None,
        }
    }
}

impl From<EvalError> for SchemaAbort {
    fn from(error: EvalError) -> Self {
        Self::Eval(error)
    }
}

// The running and suspended validator both use indices into value_entries. The
// default index parameter preserves the existing root-contract fixture shape;
// both index representations contain only root-neutral keys/indices.
pub(super) type LiveSchemaState = SchemaState<Vec<usize>>;
pub(super) type LiveSchemaContainer = SchemaContainer<Vec<usize>>;
pub(super) type LiveAssociationWork = AssociationWork<Vec<usize>>;
