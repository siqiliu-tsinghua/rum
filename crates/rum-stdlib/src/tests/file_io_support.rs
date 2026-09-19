use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use rum_runtime::Session;
use rum_runtime::kernel_api::{
    BoundedDestruction, FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode,
};

#[derive(Clone, Copy, Debug)]
struct TestFileIo;

/// This fieldless unit struct has no state to destroy.
impl BoundedDestruction for TestFileIo {}

impl FileIo for TestFileIo {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        fs::read_to_string(path).map_err(|error| FileIoError::new(error.to_string()))
    }

    fn probe(&self, path: &Path) -> FileProbe {
        match fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => FileProbe::file(normalize_path(path)),
            Ok(_) => FileProbe::non_file(),
            Err(_) => FileProbe::missing(),
        }
    }

    fn write_string(
        &self,
        path: &Path,
        contents: &str,
        mode: FileWriteMode,
    ) -> Result<(), FileIoError> {
        let result = match mode {
            FileWriteMode::Replace => fs::write(path, contents),
            FileWriteMode::Append => fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .and_then(|mut file| file.write_all(contents.as_bytes())),
        };

        result.map_err(|error| FileIoError::new(error.to_string()))
    }
}

pub(super) fn install_test_file_io(session: &mut Session) {
    session.set_file_io(Some(FileIoProvider::new(TestFileIo)));
}

fn normalize_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    })
}
