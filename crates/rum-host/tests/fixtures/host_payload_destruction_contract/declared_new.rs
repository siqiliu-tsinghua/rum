use rum_runtime::kernel_api::{BoundedDestruction, EntropyError, EntropyProvider, EntropySource};

pub struct LegacyEntropy;

/// This fieldless unit struct has no state to destroy.
impl BoundedDestruction for LegacyEntropy {}

impl EntropySource for LegacyEntropy {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        buf.fill(0x5a);
        Ok(())
    }
}

pub fn provider() -> EntropyProvider {
    EntropyProvider::new(LegacyEntropy)
}
