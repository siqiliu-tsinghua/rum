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

use std::{collections::BTreeMap, sync::Mutex};

struct MemoryFiles(Mutex<BTreeMap<String, String>>);
impl BoundedDestruction for MemoryFiles {}
impl FileIo for MemoryFiles {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        self.0
            .lock()
            .unwrap()
            .get(&path.to_string_lossy().into_owned())
            .cloned()
            .ok_or_else(|| FileIoError::new("oracle file absent"))
    }
    fn probe(&self, path: &Path) -> FileProbe {
        if self
            .0
            .lock()
            .unwrap()
            .contains_key(&path.to_string_lossy().into_owned())
        {
            FileProbe::file(path.to_owned())
        } else {
            FileProbe::missing()
        }
    }
    fn write_string(
        &self,
        path: &Path,
        source: &str,
        mode: FileWriteMode,
    ) -> Result<(), FileIoError> {
        let mut files = self.0.lock().unwrap();
        let key = path.to_string_lossy().into_owned();
        match mode {
            FileWriteMode::Replace => {
                files.insert(key, source.into());
            }
            FileWriteMode::Append => {
                files.entry(key).or_default().push_str(source);
            }
        }
        Ok(())
    }
}

pub fn configure_memory(s: &mut Session, sources: &[(String, String)]) {
    use rum_runtime::{Capability, CapabilityPolicy};
    s.set_capability_policy(CapabilityPolicy::allow([
        Capability::Pure,
        Capability::Interactive,
        Capability::Clock,
        Capability::Diagnostic,
        Capability::FileRead,
        Capability::FileWrite,
    ]));
    s.set_file_io(Some(FileIoProvider::new(MemoryFiles(Mutex::new(
        sources.iter().cloned().collect(),
    )))));
    s.add_package_search_path(std::path::PathBuf::from("/oracle"));
}
