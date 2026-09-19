use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub struct FileIoError {
    message: String,
}

impl FileIoError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for FileIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for FileIoError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileProbe {
    pub exists: bool,
    pub is_file: bool,
    pub resolved_path: Option<PathBuf>,
}

impl FileProbe {
    pub fn missing() -> Self {
        Self {
            exists: false,
            is_file: false,
            resolved_path: None,
        }
    }

    pub fn non_file() -> Self {
        Self {
            exists: true,
            is_file: false,
            resolved_path: None,
        }
    }

    pub fn file(resolved_path: impl Into<PathBuf>) -> Self {
        Self {
            exists: true,
            is_file: true,
            resolved_path: Some(resolved_path.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileWriteMode {
    Replace,
    Append,
}

pub trait FileIo: super::host_payload::BoundedDestruction + Send + Sync {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError>;

    /// Lists the immediate entries of one directory.
    ///
    /// Providers that do not support directory enumeration retain their
    /// existing behavior and report an error when the operation is requested.
    fn list_dir(&self, _path: &Path) -> Result<Vec<PathBuf>, FileIoError> {
        Err(FileIoError::new(
            "directory enumeration is not supported by this file provider",
        ))
    }

    fn probe(&self, path: &Path) -> FileProbe;
    fn write_string(
        &self,
        path: &Path,
        contents: &str,
        mode: FileWriteMode,
    ) -> Result<(), FileIoError>;
}

#[derive(Clone)]
pub struct FileIoProvider {
    inner: Arc<dyn FileIo>,
}

impl FileIoProvider {
    pub fn new(provider: impl FileIo + 'static) -> Self {
        Self {
            inner: Arc::new(provider),
        }
    }

    pub fn from_arc(provider: Arc<dyn FileIo>) -> Self {
        Self { inner: provider }
    }

    pub fn as_file_io(&self) -> &dyn FileIo {
        self.inner.as_ref()
    }
}

impl fmt::Debug for FileIoProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FileIoProvider")
    }
}

impl PartialEq for FileIoProvider {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for FileIoProvider {}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FileIoError>();
};
