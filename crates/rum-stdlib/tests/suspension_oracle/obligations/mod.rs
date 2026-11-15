//! G15-CM identity and counted requirements. Neither stage settles obligations.
mod applicability;
mod crosswalk;
mod discovery;
mod identity;
mod imports;
mod owners;
mod partition;
mod policy;
mod requirements;
mod resolve;
mod routes;

pub(crate) use identity::digest as content_digest;

#[path = "../coverage/bindings.rs"]
pub(crate) mod bindings;

#[path = "../coverage/execution_tests.rs"]
mod execution_tests;
#[path = "../ledger/mod.rs"]
pub(crate) mod ledger;
#[path = "../coverage/receipts.rs"]
mod receipts;
#[path = "../coverage/schedule.rs"]
mod schedule;

#[cfg(test)]
mod requirement_tests;

#[cfg(test)]
mod tests;
