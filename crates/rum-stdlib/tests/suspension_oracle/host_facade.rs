//! Compile the unchanged public host facade into this integration-test crate.
//! rum-stdlib cannot import its downstream rum-host without changing the Cargo
//! graph. Source reuse lets the boundary invoke the real restore_from_bytes
//! implementation, with no copied restore logic or manifest changes.
#![allow(dead_code, unused_imports)]
#[path = "../../../rum-host/src/config.rs"]
pub mod config;
#[path = "../../../rum-host/src/cursor.rs"]
pub mod cursor;
#[path = "../../../rum-host/src/entropy.rs"]
mod entropy;
#[path = "../../../rum-host/src/file_io.rs"]
mod file_io;
#[path = "../../../rum-host/src/model/mod.rs"]
pub mod model;
#[path = "../../../rum-host/src/session.rs"]
mod session;
pub use config::*;
pub use entropy::StdEntropySource;
pub use file_io::StdFileIo;
pub use model::*;
pub use rum_runtime::kernel_api::{BoundedDestruction, ClosurePrintSink, PrintSink};
pub use rum_runtime::native_stack_limit_for_declared_stack;
pub use session::*;
