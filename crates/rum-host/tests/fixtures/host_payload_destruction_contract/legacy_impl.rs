use rum_runtime::kernel_api::{EntropyError, EntropySource};

pub struct LegacyEntropy;

impl EntropySource for LegacyEntropy {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        buf.fill(0x5a);
        Ok(())
    }
}
