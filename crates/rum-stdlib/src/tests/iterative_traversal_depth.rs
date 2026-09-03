use super::*;

const TARGET_PHYSICAL_HEIGHT: usize = 4_097;

fn run_on_two_mib_thread(name: &str, action: impl FnOnce() -> String + Send + 'static) -> String {
    std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(2 * 1024 * 1024)
        .spawn(action)
        .expect("2 MiB probe thread should spawn")
        .join()
        .unwrap_or_else(|_| panic!("2 MiB probe thread {name:?} panicked"))
}

fn evaluate_certificate(body: &str) -> String {
    evaluate_certificate_at_height(TARGET_PHYSICAL_HEIGHT, body)
}

fn evaluate_certificate_at_height(physical_height: usize, body: &str) -> String {
    let mut session = Session::new();
    install_core(&mut session);
    let mut source = format!("x = 1; Do[x = {{x}}, {{i, {}}}]; ", physical_height - 1);
    source.push_str(body);
    session
        .eval_text(&source)
        .expect("depth certificate should evaluate")
        .value
        .to_string()
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn matchq_completes_at_physical_height_20001_on_two_mib() {
    let certificate = run_on_two_mib_thread("matchq-depth-20001", || {
        evaluate_certificate_at_height(20_001, "MatchQ[x, _?IntegerQ]")
    });
    assert_eq!(certificate, "False");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn construction_reaches_physical_height_20001_on_two_mib() {
    let certificate = run_on_two_mib_thread("construction-depth-20001", || {
        evaluate_certificate_at_height(20_001, "Depth[x]")
    });
    assert_eq!(certificate, "20001");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn tostring_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("tostring-depth-4097", || {
        evaluate_certificate("StringLength[ToString[x]]")
    });
    assert_eq!(certificate, "8193");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn map_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("map-depth-4097", || {
        evaluate_certificate("Depth[Map[Identity, x, {-4097, 0}]]")
    });
    assert_eq!(certificate, "4097");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn apply_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("apply-depth-4097", || {
        evaluate_certificate("Depth[Apply[f, x, {-4097, 0}]]")
    });
    assert_eq!(certificate, "4097");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn replaceall_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("replaceall-depth-4097", || {
        evaluate_certificate("Depth[ReplaceAll[x, z -> q]]")
    });
    assert_eq!(certificate, "4097");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn cases_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("cases-depth-4097", || {
        evaluate_certificate("Length[Cases[x, z, Infinity]]")
    });
    assert_eq!(certificate, "0");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn level_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("level-depth-4097", || {
        evaluate_certificate("Length[Level[x, Infinity]]")
    });
    assert_eq!(certificate, "4096");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn memberq_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("memberq-depth-4097", || {
        evaluate_certificate("MemberQ[x, z, Infinity]")
    });
    assert_eq!(certificate, "False");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn count_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("count-depth-4097", || {
        evaluate_certificate("Count[x, z, Infinity]")
    });
    assert_eq!(certificate, "0");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn position_completes_at_physical_height_4097_on_two_mib() {
    let certificate = run_on_two_mib_thread("position-depth-4097", || {
        evaluate_certificate("Length[Position[x, z, Infinity]]")
    });
    assert_eq!(certificate, "0");
}

#[test]
#[ignore = "release-only deployment-stack probe; run explicitly with --release"]
fn freeq_completes_at_physical_height_4097_on_two_mib() {
    let certificate =
        run_on_two_mib_thread("freeq-depth-4097", || evaluate_certificate("FreeQ[x, z]"));
    assert_eq!(certificate, "True");
}
