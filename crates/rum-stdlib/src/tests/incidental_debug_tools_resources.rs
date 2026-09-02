use crate::install_core;
use rum_core::Expr;
use rum_runtime::Session;
use rum_runtime::kernel_api::system_expr;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn byte_count_and_memory_in_use_use_the_current_structural_proxy_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("List[ByteCount[1], ByteCount[123456789], ByteCount[\"abc\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[32, 32, 35]"
    );

    let string_small = session.eval_text("ByteCount[\"abc\"]").unwrap().value;
    let string_large = session.eval_text("ByteCount[\"abcdefgh\"]").unwrap().value;
    let atom = session.eval_text("ByteCount[1]").unwrap().value;
    let call = session.eval_text("ByteCount[f[1, 2, 3]]").unwrap().value;
    let byte_array_small = session
        .eval_text("ByteCount[StringToByteArray[\"abc\"]]")
        .unwrap()
        .value;
    let byte_array_large = session
        .eval_text("ByteCount[StringToByteArray[\"abcdefgh\"]]")
        .unwrap()
        .value;

    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Less"),
                vec![string_small.clone(), string_large]
            ))
            .unwrap()
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(system_expr("Less"), vec![atom, call]))
            .unwrap()
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Less"),
                vec![byte_array_small, byte_array_large]
            ))
            .unwrap()
            .to_string(),
        "True"
    );

    let small_int = session.eval_text("ByteCount[1]").unwrap().value;
    let large_int = session.eval_text("ByteCount[10^80]").unwrap().value;
    assert_eq!(
        session
            .evaluate(Expr::call(system_expr("Less"), vec![small_int, large_int]))
            .unwrap()
            .to_string(),
        "True"
    );

    let before = session.eval_text("MemoryInUse[]").unwrap().value;
    session.eval_text("byteProbe = Table[i, {i, 200}]").unwrap();
    let after = session.eval_text("MemoryInUse[]").unwrap().value;
    assert_eq!(
        session
            .evaluate(Expr::call(system_expr("Less"), vec![before, after]))
            .unwrap()
            .to_string(),
        "True"
    );

    let before_off = session.eval_text("MemoryInUse[]").unwrap().value;
    session.eval_text("Off[FindFile::notfound]").unwrap();
    let after_off = session.eval_text("MemoryInUse[]").unwrap().value;
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Less"),
                vec![before_off.clone(), after_off.clone()]
            ))
            .unwrap()
            .to_string(),
        "True"
    );

    session.eval_text("On[FindFile::notfound]").unwrap();

    // MemoryInUse[] populates the call-stability cache, so a later same-session
    // sample cannot isolate the bytes released by On. These fresh sessions run
    // the same On/Off requests in opposite order, leaving the same cache keys
    // but differing in whether the message selector remains disabled.
    let mut selector_retained = core_session();
    selector_retained
        .eval_text("On[FindFile::notfound]")
        .unwrap();
    selector_retained
        .eval_text("Off[FindFile::notfound]")
        .unwrap();
    let retained_memory = selector_retained.eval_text("MemoryInUse[]").unwrap().value;

    let mut selector_removed = core_session();
    selector_removed
        .eval_text("Off[FindFile::notfound]")
        .unwrap();
    selector_removed
        .eval_text("On[FindFile::notfound]")
        .unwrap();
    let removed_memory = selector_removed.eval_text("MemoryInUse[]").unwrap().value;
    assert_eq!(
        selector_removed
            .evaluate(Expr::call(
                system_expr("Less"),
                vec![removed_memory, retained_memory]
            ))
            .unwrap()
            .to_string(),
        "True"
    );

    let before_once = session.eval_text("MemoryInUse[]").unwrap().value;
    session
        .eval_text("Once[Table[i, {i, 50}], \"memory-probe\"]")
        .unwrap();
    let after_once = session.eval_text("MemoryInUse[]").unwrap().value;
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Less"),
                vec![before_once, after_once]
            ))
            .unwrap()
            .to_string(),
        "True"
    );

    session
        .eval_text(
            "Options[memoryScoped] = {alpha -> 1, beta -> 2}; memoryScoped[OptionsPattern[]] := {MemoryInUse[], OptionValue[{alpha, beta}]}; memoryBaseline[] := MemoryInUse[]",
        )
        .unwrap();
    let scoped_baseline = session.eval_text("memoryBaseline[]").unwrap().value;
    let scoped_memory = session
        .eval_text("Part[memoryScoped[alpha -> 10, beta -> 20], 1]")
        .unwrap()
        .value;
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Less"),
                vec![scoped_baseline, scoped_memory]
            ))
            .unwrap()
            .to_string(),
        "True"
    );
}

#[test]
fn memory_in_use_accounts_for_removed_symbol_identity_state() {
    let mut session = core_session();

    let before = session.eval_text("MemoryInUse[]").unwrap().value;
    for index in 0..12 {
        session
            .eval_text(&format!("Remove[cl09RemovedMemory{index}]"))
            .unwrap();
    }
    let after = session.eval_text("MemoryInUse[]").unwrap().value;

    assert_eq!(
        session
            .evaluate(Expr::call(system_expr("Less"), vec![before, after]))
            .unwrap()
            .to_string(),
        "True"
    );
}

#[test]
fn max_memory_used_tracks_session_memory_proxy_high_water_mark() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("IntegerQ[MaxMemoryUsed[]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    let initial = session.eval_text("MaxMemoryUsed[]").unwrap().value;
    assert!(initial.to_string().parse::<usize>().unwrap() > 0);

    // MemoryInUse[] itself grows the call-stability cache. A peak sampled first
    // therefore cannot cover state retained before the later current-use walk;
    // sample current use first, then observe the high-water mark.
    assert_eq!(
        session
            .eval_text("MemoryInUse[] <= MaxMemoryUsed[]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );

    let before = session.eval_text("MaxMemoryUsed[]").unwrap().value;
    session.eval_text("Do[gmem[i] = i, {i, 1, 300}]").unwrap();
    let after = session.eval_text("MaxMemoryUsed[]").unwrap().value;
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("GreaterEqual"),
                vec![after.clone(), before.clone()]
            ))
            .unwrap()
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(system_expr("Greater"), vec![after, before]))
            .unwrap()
            .to_string(),
        "True"
    );

    assert_eq!(
        session
            .eval_text("MaxMemoryUsed[x]")
            .unwrap()
            .value
            .to_string(),
        "MaxMemoryUsed[x]"
    );
}
