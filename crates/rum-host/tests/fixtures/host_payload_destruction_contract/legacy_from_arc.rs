use rum_runtime::kernel_api::{EntropyError, EntropyProvider, EntropySource};
use std::sync::Arc;

pub struct LegacyEntropy;

impl EntropySource for LegacyEntropy {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        buf.fill(0x5a);
        Ok(())
    }
}

pub fn provider() -> EntropyProvider {
    let erased: Arc<dyn EntropySource> = Arc::new(LegacyEntropy);
    EntropyProvider::from_arc(erased)
}
