use rum_runtime::{
    Session,
    kernel_api::{
        BoundedDestruction, FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode,
    },
};
use std::path::Path;

pub struct ReadOnlyFiles;
impl BoundedDestruction for ReadOnlyFiles {}
impl FileIo for ReadOnlyFiles {
    fn read_to_string(&self, _: &Path) -> Result<String, FileIoError> {
        Err(FileIoError::new("probe file absent"))
    }
    fn probe(&self, _: &Path) -> FileProbe {
        FileProbe::missing()
    }
    fn write_string(&self, _: &Path, _: &str, _: FileWriteMode) -> Result<(), FileIoError> {
        Err(FileIoError::new("probe provider is read-only"))
    }
}
pub fn configure(s: &mut Session) {
    s.set_file_io(Some(FileIoProvider::new(ReadOnlyFiles)));
}
