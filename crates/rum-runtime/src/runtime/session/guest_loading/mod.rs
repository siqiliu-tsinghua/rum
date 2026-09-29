//! Guest-only loading. Public script/file batches retain their separate policy.
//! Source/syntax and rollback payloads are named for the later R11/R12 retention
//! work; tracing these roots is not a capacity receipt or a lease.
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};
use crate::runtime::suspension::{
    AbandonmentContext, FrameAbandon, FrameAbandonAction, FrameAction, SuspensionFrame,
};
use crate::runtime::*;
use std::sync::Arc;

mod file;
mod scope;
mod script;

pub(in crate::runtime) use file::GuestFileTransaction;
use file::*;
pub(in crate::runtime) use scope::{GuestFormEnvironment, GuestFormIdentity, GuestFormScope};

#[cfg(test)]
pub(in crate::runtime) mod tests;
