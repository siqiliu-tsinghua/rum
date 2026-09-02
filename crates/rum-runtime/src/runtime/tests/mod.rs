use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::test_builtins::{
    session_with_test_control_builtins, session_with_test_rewrite_builtins,
};
use super::*;

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("rum-{prefix}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative: &str, source: &str) -> PathBuf {
        let path = self.path.join(relative);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(&path, source).unwrap();
        path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[derive(Clone, Copy, Debug)]
struct TestFileIo;

impl FileIo for TestFileIo {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        fs::read_to_string(path).map_err(|error| FileIoError::new(error.to_string()))
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>, FileIoError> {
        fs::read_dir(path)
            .map_err(|error| FileIoError::new(error.to_string()))?
            .map(|entry| {
                entry
                    .map(|entry| entry.path())
                    .map_err(|error| FileIoError::new(error.to_string()))
            })
            .collect()
    }

    fn probe(&self, path: &Path) -> FileProbe {
        match fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => FileProbe::file(normalize_test_path(path)),
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

pub(in crate::runtime) fn install_test_file_io(session: &mut Session) {
    session.set_file_io(Some(FileIoProvider::new(TestFileIo)));
}

fn normalize_test_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    })
}

fn rum_path(path: &Path) -> String {
    normalize_test_path(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

mod arithmetic;
mod association_lookup_cache;
mod association_part_cursor;
mod association_write_index;
mod association_write_recovery;
mod comparison_lowering;
mod context_and_loader;
mod debug_tools;
mod definition_lhs_preparation;
mod definition_store_sealing;
mod definitions_and_rules;
mod definitions_and_rules_cases;
mod definitions_and_rules_part2;
mod definitions_and_rules_part3;
mod definitions_and_rules_part4;
mod definitions_and_rules_patterns;
mod definitions_and_rules_rewrite_inspect;
mod definitions_and_rules_tags;
mod definitions_rhs_guards;
mod definitions_rhs_guards_attribute_revision;
mod definitions_rhs_guards_default_values;
mod definitions_rhs_guards_own_values;
mod definitions_rhs_guards_rewrite_rules;
mod definitions_rhs_guards_sub_values;
mod definitions_rhs_guards_up_values;
mod definitions_tagged_preparation;
mod definitions_unset_preparation;
mod definitions_value_table_assignment;
mod file_names;
mod frame_root_trace;
mod holding_and_control_io;
mod holding_and_control_part2;
mod hygiene;
mod iterative_loop_drivers;
mod left_chain_lowering;
mod lowering_and_context;
mod lowering_golden;
mod messages;
mod messages_part2;
mod messages_part3;
mod mutation_operator_forms;
mod namespace_names;
mod nothing;
mod options_and_protection;
mod package_tools;
mod part_assignment;
mod pattern_matching_orderless_optional;
mod reap;
mod registered_builtin_resolution;
mod replace_all_unheld;
mod rule_rhs_materialization;
mod scoping_gensym_gc;
mod scoping_unique;
mod sealed_packages;
mod sealed_snapshot;
mod sealed_visibility;
mod session_config;
mod session_requests;
mod structure_data;
mod structured_exceptions;
mod suspension_abandonment;
mod syntax_information_debug_catalog;
mod syntax_information_lifecycle;
mod thread_and_dispatch;
mod thread_and_dispatch_part2;
mod traversal_part2;
mod valueq_stable_shortcut;
mod wide_child_traversals;
mod wide_definition_dispatch;
