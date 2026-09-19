use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

fn expected_path_list(paths: &[&Path]) -> String {
    let mut paths = paths.iter().map(|path| rum_path(path)).collect::<Vec<_>>();
    paths.sort();
    Expr::call(
        system_expr("List"),
        paths.into_iter().map(Expr::string).collect(),
    )
    .to_string()
}

fn eval_value(session: &mut Session, source: &str) -> String {
    let outcome = session.eval_text(source).unwrap();
    assert!(
        outcome.messages.is_empty(),
        "{source}: {:?}",
        outcome.messages
    );
    outcome.value.to_string()
}

#[test]
fn file_names_matches_immediate_entries_and_sorts_and_deduplicates_paths() {
    let dir = TempDirGuard::new("file-names-immediate");
    let b = dir.write("b.wl", "b");
    let a = dir.write("a.wl", "a");
    let c = dir.write("c.txt", "c");
    let dir_expr = Expr::string(rum_path(dir.path())).to_string();
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        eval_value(
            &mut session,
            &format!("FileNames[\"*.wl\", {{{dir_expr}, {dir_expr}}}]")
        ),
        expected_path_list(&[&a, &b])
    );
    assert_eq!(
        eval_value(&mut session, &format!("FileNames[All, {dir_expr}]")),
        expected_path_list(&[&a, &b, &c])
    );

    let default = session.eval_text("FileNames[]").unwrap();
    assert!(default.messages.is_empty(), "{:?}", default.messages);
    assert!(list_items(&default.value).is_some());
    assert_eq!(
        eval_value(
            &mut session,
            "FileNames[\"brief-230-definitely-missing-*\"]"
        ),
        "List[]"
    );
}

#[test]
fn file_names_depth_controls_nested_provider_enumeration() {
    let dir = TempDirGuard::new("file-names-depth");
    let top = dir.write("top.wl", "top");
    let deep = dir.write("nested/deep.wl", "deep");
    let deepest = dir.write("nested/deeper/deepest.wl", "deepest");
    let dir_expr = Expr::string(rum_path(dir.path())).to_string();
    let missing_expr = Expr::string(rum_path(&dir.path().join("missing"))).to_string();
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        eval_value(&mut session, &format!("FileNames[\"*.wl\", {dir_expr}, 1]")),
        expected_path_list(&[&top])
    );
    assert_eq!(
        eval_value(&mut session, &format!("FileNames[\"*.wl\", {dir_expr}, 2]")),
        expected_path_list(&[&top, &deep])
    );
    assert_eq!(
        eval_value(
            &mut session,
            &format!("FileNames[\"*.wl\", {dir_expr}, Infinity]")
        ),
        expected_path_list(&[&top, &deep, &deepest])
    );
    assert_eq!(
        eval_value(&mut session, &format!("FileNames[\"*.wl\", {dir_expr}, 0]")),
        "List[]"
    );
    assert_eq!(
        eval_value(&mut session, &format!("FileNames[\"*\", {missing_expr}]")),
        "List[]"
    );
}

#[test]
fn file_names_malformed_specs_remain_inert() {
    let mut session = session_with_test_control_builtins();

    for (source, expected) in [
        ("FileNames[1]", "FileNames[1]"),
        ("FileNames[\"*\", 1]", "FileNames[\"*\", 1]"),
        (
            "FileNames[\"*\", {\".\"}, -1]",
            "FileNames[\"*\", List[\".\"], -1]",
        ),
    ] {
        assert_eq!(eval_value(&mut session, source), expected);
    }
}

#[test]
fn file_names_guard_precedes_the_provider_check() {
    let mut session = session_with_test_control_builtins();
    session.set_file_io(None);
    session.set_capability_policy(CapabilityPolicy::allow([Capability::Pure]));

    let value = session.eval_file_names(&[]).unwrap();

    assert_eq!(value.to_string(), "$Failed");
    assert!(session.messages().iter().any(|message| {
        message.symbol == "General"
            && message.tag == "sandbox"
            && message.text == "FileNames is not permitted in this sandbox."
    }));
    assert!(
        !session
            .messages()
            .iter()
            .any(|message| message.tag == "noio")
    );
}

#[derive(Debug)]
struct LoopingFileIo {
    list_calls: AtomicUsize,
}

/// Its sole atomic counter has no owned state to recurse into, even at the last Arc.
impl BoundedDestruction for LoopingFileIo {}

impl FileIo for LoopingFileIo {
    fn read_to_string(&self, _path: &Path) -> Result<String, FileIoError> {
        Err(FileIoError::new("read unsupported"))
    }

    fn list_dir(&self, _path: &Path) -> Result<Vec<PathBuf>, FileIoError> {
        self.list_calls.fetch_add(1, Ordering::Relaxed);
        Ok(vec![PathBuf::from("/loop")])
    }

    fn probe(&self, _path: &Path) -> FileProbe {
        FileProbe::non_file()
    }

    fn write_string(
        &self,
        _path: &Path,
        _contents: &str,
        _mode: FileWriteMode,
    ) -> Result<(), FileIoError> {
        Err(FileIoError::new("write unsupported"))
    }
}

#[test]
fn file_names_infinity_stops_at_the_fixed_entry_cap_without_recursing_the_stack() {
    let file_io = Arc::new(LoopingFileIo {
        list_calls: AtomicUsize::new(0),
    });
    let mut session = session_with_test_control_builtins();
    session.set_file_io(Some(FileIoProvider::from_arc(file_io.clone())));

    let outcome = session
        .eval_text("FileNames[\"*\", {\"/loop\"}, Infinity]")
        .unwrap();

    assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
    assert_eq!(list_items(&outcome.value).unwrap().len(), 1);
    assert_eq!(file_io.list_calls.load(Ordering::Relaxed), 100_000);
}
