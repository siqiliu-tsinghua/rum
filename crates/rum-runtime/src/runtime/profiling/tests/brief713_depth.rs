use super::*;

#[test]
fn brief713_nested_admission_is_caught() {
    let _enabled = crate::runtime::profiling::tests::engine_support::enabled();
    // An ordinary composite getter must thread its permit, not reacquire it.
    assert!(crate::runtime::profiling::lexical_profile_enabled());
    let access = admit().unwrap();
    let caught = std::panic::catch_unwind(admit).is_err();
    drop(access);
    eprintln!("B713 DEPTH nested_admission_caught={caught}");
    assert!(
        caught,
        "nested admission must fail before acquiring the gate"
    );
    assert!(admit().is_some(), "unwinding must leave the depth balanced");
}
