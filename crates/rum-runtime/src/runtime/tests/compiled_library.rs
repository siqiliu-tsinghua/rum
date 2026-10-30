use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

static COMPILED: OnceLock<CompiledLibrary> = OnceLock::new();

pub(in crate::runtime) struct CompiledLibrary {
    deps: PathBuf,
    runtime: PathBuf,
    core: PathBuf,
    build_stdout: Vec<u8>,
    build_stderr: Vec<u8>,
}

impl CompiledLibrary {
    /// One fresh build per test process. No caller supplies a path.
    pub(in crate::runtime) fn get() -> &'static Self {
        COMPILED.get_or_init(Self::build)
    }

    pub(in crate::runtime) fn deps(&self) -> &Path {
        &self.deps
    }

    pub(in crate::runtime) fn runtime(&self) -> &Path {
        &self.runtime
    }

    pub(in crate::runtime) fn core(&self) -> &Path {
        &self.core
    }

    pub(in crate::runtime) fn build_stdout(&self) -> &[u8] {
        &self.build_stdout
    }

    pub(in crate::runtime) fn build_stderr(&self) -> &[u8] {
        &self.build_stderr
    }

    fn build() -> Self {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let target = workspace.join("target/test-compile-library");
        let build = Command::new("python3")
            .arg(workspace.join("scripts/run-bounded.py"))
            .args([
                "--timeout",
                "180",
                "--",
                "cargo",
                "build",
                "--offline",
                "-p",
                "rum-core",
                "-p",
                "rum-runtime",
                "--lib",
                "--target-dir",
            ])
            .arg(&target)
            .current_dir(workspace)
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "fresh protocol library: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        let debug = target.join("debug");
        let deps = debug.join("deps");
        let runtime = debug.join("librum_runtime.rlib");
        let core = debug.join("librum_core.rlib");
        assert!(
            runtime.is_file(),
            "missing constructed runtime library: {}",
            runtime.display()
        );
        assert!(
            core.is_file(),
            "missing constructed core library: {}",
            core.display()
        );
        Self {
            deps,
            runtime,
            core,
            build_stdout: build.stdout,
            build_stderr: build.stderr,
        }
    }
}
