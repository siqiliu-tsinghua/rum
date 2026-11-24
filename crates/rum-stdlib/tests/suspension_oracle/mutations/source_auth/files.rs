//! Canonical builder tree order, with no following of source symlinks.
use super::{AuthError, AuthResult, digest};
use std::{collections::BTreeMap, path::Path};

#[derive(Clone)]
pub(super) struct Snapshot {
    pub files: BTreeMap<String, Vec<u8>>,
    pub tree: String,
}

impl Snapshot {
    pub fn read(root: &Path) -> AuthResult<Self> {
        fn walk(
            root: &Path,
            dir: &Path,
            files: &mut BTreeMap<String, Vec<u8>>,
            stream: &mut Vec<u8>,
        ) -> AuthResult<()> {
            let mut entries = std::fs::read_dir(dir)
                .map_err(|_| AuthError::Io)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| AuthError::Io)?;
            entries.sort_by_key(|entry| entry.file_name());
            let mut directories = Vec::new();
            for entry in entries {
                let path = entry.path();
                let kind = entry.file_type().map_err(|_| AuthError::Io)?;
                if kind.is_symlink() {
                    return Err(AuthError::Symlink);
                }
                if kind.is_dir() {
                    if !matches!(
                        entry.file_name().to_str(),
                        Some(".git" | "target" | ".temp" | "__pycache__" | ".mypy_cache")
                    ) {
                        directories.push(path);
                    }
                } else if kind.is_file() {
                    let key = path
                        .strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .ok_or(AuthError::Io)?;
                    let bytes = std::fs::read(&path).map_err(|_| AuthError::Io)?;
                    stream.extend_from_slice(b"F\0");
                    stream.extend_from_slice(key.as_bytes());
                    stream.push(0);
                    stream.extend_from_slice(&bytes);
                    files.insert(key.into(), bytes);
                } else {
                    return Err(AuthError::Io);
                }
            }
            for directory in directories {
                walk(root, &directory, files, stream)?;
            }
            Ok(())
        }
        let mut files = BTreeMap::new();
        let mut stream = Vec::new();
        walk(root, root, &mut files, &mut stream)?;
        Ok(Self {
            files,
            tree: digest(&stream),
        })
    }
}
