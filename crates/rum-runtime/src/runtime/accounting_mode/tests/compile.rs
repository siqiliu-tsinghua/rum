use crate::runtime::tests::compiled_library::CompiledLibrary;
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn retention_compile_controls_precede_pinned_failures() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let scratch = workspace.join("target/brief-538/compile-proof");
    fs::create_dir_all(&scratch).unwrap();
    let library = CompiledLibrary::get();
    let deps = library.deps();
    fs::write(scratch.join("library-build.stdout"), library.build_stdout()).unwrap();
    fs::write(scratch.join("library-build.stderr"), library.build_stderr()).unwrap();
    for section in include_str!("../protocol.md").split("### ").skip(1) {
        let name = section.lines().next().unwrap();
        for (case, expected) in [
            ("control", None),
            (
                "rejected",
                Some(
                    section
                        .split_once("```compile_fail,")
                        .unwrap()
                        .1
                        .lines()
                        .next()
                        .unwrap(),
                ),
            ),
        ] {
            let fence = if expected.is_none() {
                "```no_run\n".to_owned()
            } else {
                format!("```compile_fail,{}\n", expected.unwrap())
            };
            let source = section
                .split_once(&fence)
                .unwrap()
                .1
                .split_once("```")
                .unwrap()
                .0;
            let fixture = scratch.join(format!("{name}-{case}.rs"));
            fs::write(&fixture, source).unwrap();
            let output = Command::new("python3")
                .arg(workspace.join("scripts/run-bounded.py"))
                .args([
                    "--timeout",
                    "60",
                    "--",
                    "rustc",
                    "--edition=2024",
                    "--crate-name",
                    "retention_proof",
                    "--emit=metadata",
                    "--cap-lints=allow",
                ])
                .arg("--out-dir")
                .arg(&scratch)
                .arg("-L")
                .arg(format!("dependency={}", deps.display()))
                .arg("--extern")
                .arg(format!("rum_runtime={}", library.runtime().display()))
                .arg(&fixture)
                .output()
                .unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();
            fs::write(scratch.join(format!("{name}-{case}.stderr")), &stderr).unwrap();
            fs::write(scratch.join(format!("{name}-{case}.stdout")), output.stdout).unwrap();
            assert_eq!(
                output.status.success(),
                expected.is_none(),
                "{name}/{case}: {stderr}"
            );
            if let Some(code) = expected {
                assert!(
                    stderr.contains(&format!("error[{code}]")),
                    "{name}: {stderr}"
                );
                assert_eq!(
                    stderr.matches("error[E").count(),
                    1,
                    "no unrelated failures: {stderr}"
                );
            }
            println!(
                "{name}/{case}: success={}\n{stderr}",
                output.status.success()
            );
        }
    }
}
