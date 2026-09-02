use super::*;

fn eval_zzz_probe(
    _session: &mut Session,
    _head: &Expr,
    _args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let _args = _args.as_ref();
    Ok(Expr::integer(BigInt::from(9001_u16)))
}

#[test]
fn registered_builtin_absent_from_static_catalog_resolves_as_system_symbol() {
    let mut session = Session::new();
    session.register_builtin(BuiltinSpec::ordinary(
        "ZzzProbe",
        "Runtime registration resolution probe.",
        eval_zzz_probe,
        MemoryGrowthContract::NonAmplifying,
    ));

    let result = session.eval_text("ZzzProbe[1]").unwrap();

    assert_eq!(result.value.to_string(), "9001");
}
