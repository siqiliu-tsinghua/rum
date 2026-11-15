//! Profile switches are configured in isolated children, never mutated globally.
pub fn switches(off: bool) -> Vec<(String, String)> {
    let mut switches = vec![(
        "RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL".into(),
        "0".into(),
    )];
    for name in [
        "RUM_PROFILE_LEXICAL_SUBSTITUTION",
        "RUM_PROFILE_BUILTIN_TIMING",
        "RUM_PROFILE_DEFINITION_RULE_TIMING",
        "RUM_PROFILE_RULE_RHS_TIMING",
        "RUM_PROFILE_EVALUATION_CACHE",
    ] {
        switches.push((name.into(), if off { "0" } else { "1" }.into()));
    }
    switches.sort();
    switches
}

pub fn off_worker(nonce: &str) -> std::io::Result<std::process::Output> {
    let mut command = std::process::Command::new(std::env::current_exe()?);
    command.args([
        "--exact",
        "oracle::obligations::execution_tests::cm_live_off_worker",
        "--nocapture",
        "--test-threads=8",
    ]);
    for (key, _) in std::env::vars().filter(|(key, _)| key.starts_with("RUM_PROFILE_")) {
        command.env_remove(key);
    }
    command.env_remove("RUM_CM739_CHILD");
    command.env("RUM_CM739_OFF_WORKER", nonce);
    for (key, value) in switches(true) {
        command.env(key, value);
    }
    command.output()
}

pub fn p0543_worker(nonce: &str) -> std::io::Result<std::process::Output> {
    let binary = std::env::var_os("RUM_CM739_LANE_BINARY")
        .ok_or_else(|| std::io::Error::other("P0543 build lane is not configured"))?;
    let mut command = std::process::Command::new(binary);
    command.args([
        "--exact",
        "oracle::obligations::execution_tests::cm_live_p0543_worker",
        "--nocapture",
        "--test-threads=8",
    ]);
    for (key, _) in std::env::vars().filter(|(key, _)| key.starts_with("RUM_PROFILE_")) {
        command.env_remove(key);
    }
    command
        .env_remove("RUM_CM739_CHILD")
        .env_remove("RUM_CM739_OFF_WORKER")
        .env_remove("RUM_CM739_GROWTH_CONTROL_CHILD")
        .env("RUM_CM739_P0543_WORKER", nonce)
        .env("RUM_CM739_FEATURE_LANE", "p0543-unchecked")
        .env("RUM_TEST_BUILTIN_MEMORY_GROWTH", "0");
    for (key, value) in switches(false) {
        command.env(key, value);
    }
    let started = std::time::Instant::now();
    let output = command.output()?;
    println!(
        "CM P0543 LANE COST elapsed_seconds={:.6} success={}",
        started.elapsed().as_secs_f64(),
        output.status.success()
    );
    Ok(output)
}

pub fn profile_child(test: &str) -> bool {
    if std::env::var("RUM_CM739_CHILD").as_deref() == Ok(test) {
        return true;
    }
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", test, "--nocapture", "--test-threads=8"]);
    for (key, _) in std::env::vars().filter(|(key, _)| key.starts_with("RUM_PROFILE_")) {
        command.env_remove(key);
    }
    command
        .env("RUM_CM739_CHILD", test)
        .env("RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL", "0");
    for key in [
        "RUM_PROFILE_LEXICAL_SUBSTITUTION",
        "RUM_PROFILE_BUILTIN_TIMING",
        "RUM_PROFILE_DEFINITION_RULE_TIMING",
        "RUM_PROFILE_RULE_RHS_TIMING",
        "RUM_PROFILE_EVALUATION_CACHE",
    ] {
        command.env(key, "1");
    }
    let output = command.output().unwrap();
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(
        output.status.success(),
        "CM profile child {test}: {}",
        output.status
    );
    false
}
