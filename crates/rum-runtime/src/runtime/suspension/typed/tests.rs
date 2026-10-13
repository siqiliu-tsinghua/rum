//! S1 exercises the dormant adapters directly; no test installs one in a queue.
use super::action::Action;
use super::channel::{CodecKey, ContinuationState, Slot, decode, encode};
use super::*;
use rum_core::Expr;
use std::sync::atomic::Ordering;

mod accounting;
mod composition;
mod faults;
mod registry;

mod support;
use support::{Finish, echo};
