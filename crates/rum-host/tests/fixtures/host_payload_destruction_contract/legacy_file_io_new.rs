use rum_runtime::kernel_api::{FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode};
use std::path::Path;

pub struct LegacyFileIo;

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
    FileIoProvider::new(LegacyFileIo)
}
