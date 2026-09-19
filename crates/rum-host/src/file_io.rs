use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use rum_runtime::kernel_api::{
    BoundedDestruction, FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode,
};

/// Host filesystem provider backed by `std::fs`.
///
/// Trusted host sessions install this provider by default. Sandboxed sessions
/// omit it so file builtins fail at construction level if capability policy
/// has explicitly allowed them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StdFileIo;

impl StdFileIo {
    /// Creates a provider value.
    pub fn new() -> Self {
        Self
    }

    /// Returns a shared cloneable provider handle for host configuration.
    pub fn provider() -> FileIoProvider {
        static PROVIDER: OnceLock<FileIoProvider> = OnceLock::new();
        PROVIDER
            .get_or_init(|| FileIoProvider::new(Self::new()))
            .clone()
    }
}

/// This fieldless unit struct has no state to destroy, including at the last shared holder.
impl BoundedDestruction for StdFileIo {}

impl FileIo for StdFileIo {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        fs::read_to_string(path).map_err(file_io_error)
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileIoError> {
        fs::read_dir(path)
            .map_err(file_io_error)?
            .map(|entry| entry.map(|entry| entry.path()).map_err(file_io_error))
            .collect()
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

        result.map_err(file_io_error)
    }
}

fn file_io_error(error: std::io::Error) -> FileIoError {
    FileIoError::new(error.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("rum-host-file-io-{name}-{unique}"))
    }

    #[test]
    fn std_file_io_probe_distinguishes_files_directories_and_missing_paths() {
        let root = unique_temp_path("probe");
        fs::create_dir_all(&root).unwrap();
        let file = root.join("source.rum");
        fs::write(&file, "1").unwrap();

        let file_probe = StdFileIo.probe(&file);
        assert!(file_probe.exists);
        assert!(file_probe.is_file);
        assert_eq!(
            file_probe.resolved_path,
            Some(fs::canonicalize(&file).unwrap())
        );

        let dir_probe = StdFileIo.probe(&root);
        assert!(dir_probe.exists);
        assert!(!dir_probe.is_file);
        assert!(dir_probe.resolved_path.is_none());

        let missing_probe = StdFileIo.probe(&root.join("missing.rum"));
        assert!(!missing_probe.exists);
        assert!(!missing_probe.is_file);
        assert!(missing_probe.resolved_path.is_none());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn std_file_io_lists_only_immediate_directory_entries() {
        let root = unique_temp_path("list-dir");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("a.wl"), "1").unwrap();
        fs::write(nested.join("deep.wl"), "2").unwrap();

        let mut entries = StdFileIo.list_dir(&root).unwrap();
        entries.sort();

        assert_eq!(entries, vec![root.join("a.wl"), nested]);
        assert!(
            !StdFileIo
                .list_dir(&root.join("missing"))
                .expect_err("missing directory enumeration should fail")
                .message()
                .is_empty()
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn normalize_path_falls_back_to_current_dir_for_missing_relative_paths() {
        let relative = Path::new("rum-host-missing-relative-file.rum");
        let normalized = normalize_path(relative);

        assert!(normalized.is_absolute());
        assert!(normalized.ends_with(relative));
    }

    #[test]
    fn std_file_io_converts_read_and_write_errors() {
        let root = unique_temp_path("errors");
        fs::create_dir_all(&root).unwrap();

        let read_error = StdFileIo
            .read_to_string(&root.join("missing.rum"))
            .expect_err("missing file read should fail");
        assert!(!read_error.message().is_empty());

        let write_error = StdFileIo
            .write_string(&root, "contents", FileWriteMode::Replace)
            .expect_err("writing contents to a directory should fail");
        assert!(!write_error.message().is_empty());

        fs::remove_dir_all(root).unwrap();
    }
}
