use rum_runtime::kernel_api::{
    BoundedDestruction, FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode,
};
use std::path::Path;
use std::sync::Arc;

pub struct LegacyFileIo;

/// This fieldless unit struct has no state to destroy, including at the last Arc.
impl BoundedDestruction for LegacyFileIo {}

impl FileIo for LegacyFileIo {
    fn read_to_string(&self, _path: &Path) -> Result<String, FileIoError> {
        Ok(String::new())
    }

    fn probe(&self, _path: &Path) -> FileProbe {
        FileProbe::missing()
    }

    fn write_string(
        &self,
        _path: &Path,
        _contents: &str,
        _mode: FileWriteMode,
    ) -> Result<(), FileIoError> {
        Ok(())
    }
}

pub fn provider() -> FileIoProvider {
    let erased: Arc<dyn FileIo> = Arc::new(LegacyFileIo);
    FileIoProvider::from_arc(erased)
}
