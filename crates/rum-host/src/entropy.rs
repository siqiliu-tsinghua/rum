use std::fs::File;
use std::io::Read;
use std::sync::OnceLock;

use rum_runtime::kernel_api::{BoundedDestruction, EntropyError, EntropyProvider, EntropySource};

/// Host entropy provider backed by `/dev/urandom`.
///
/// Trusted host sessions install this provider by default on Unix-like hosts.
/// Sandboxed sessions omit it so entropy builtins fail at construction level if
/// capability policy has explicitly allowed them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StdEntropySource;

impl StdEntropySource {
    /// Creates a provider value.
    pub fn new() -> Self {
        Self
    }

    /// Returns a shared cloneable provider handle for host configuration.
    pub fn provider() -> EntropyProvider {
        static PROVIDER: OnceLock<EntropyProvider> = OnceLock::new();
        PROVIDER
            .get_or_init(|| EntropyProvider::new(Self::new()))
            .clone()
    }
}

/// This fieldless unit struct has no state to destroy, including at the last shared holder.
impl BoundedDestruction for StdEntropySource {}

impl EntropySource for StdEntropySource {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        if buf.is_empty() {
            return Ok(());
        }

        let mut file = File::open("/dev/urandom").map_err(entropy_error)?;
        file.read_exact(buf).map_err(entropy_error)
    }
}

fn entropy_error(error: std::io::Error) -> EntropyError {
    EntropyError::new(error.to_string())
}
