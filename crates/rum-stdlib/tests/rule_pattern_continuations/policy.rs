use super::support::{compare, session};

#[test]
fn structural_plain_children_are_rebuilt_without_applying_the_call() {
    let mut s = session(None);
    s.eval_request("g[z_] := (AppendTo[log,99];77)").unwrap();
    for (lhs, expected) in [
        ("Pattern[x,g[2]]", "Pattern[x, g[2]]"),
        ("Condition[g[2],True]", "Condition[g[2], True]"),
        ("PatternTest[g[2],IntegerQ]", "PatternTest[g[2], IntegerQ]"),
        ("Alternatives[g[2],3]", "Alternatives[g[2], 3]"),
    ] {
        let value = s.eval_request(&format!("Rule[{lhs},9]")).unwrap().value;
        assert_eq!(value.to_string(), format!("Rule[{expected}, 9]"));
    }
    assert_eq!(s.eval_request("log").unwrap().value.to_string(), "List[]");
}

#[test]
fn structural_plain_child_resumes_without_applying_the_call() {
    compare(
        "structural-plain-child",
        r#"Rule[Pattern[x,g[Input["first: "]]],9]"#,
        &["2"],
        &["first: "],
        "g[z_] := (AppendTo[log,99];77)",
    );
}

#[test]
fn binder_is_held() {
    compare(
        "binder-held",
        r#"Rule[Pattern[(AppendTo[log,99];InputString["held: "]),Blank[Input["first: "]]],9]"#,
        &["Integer"],
        &["first: "],
        "",
    );
}

#[test]
fn construction_and_teardown_control_2000_on_2mib() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            use rum_core::Expr;
            let head = Expr::symbol("h");
            let mut root = Expr::small_integer(0);
            for _ in 0..2_000 {
                root = Expr::call(head.clone(), vec![root]);
            }
            let mut cursor = &root;
            for _ in 0..2_000 {
                let Expr::Call { head: actual, args } = cursor else {
                    panic!("short spine");
                };
                assert!(actual.as_ref() == &head);
                assert_eq!(args.len(), 1);
                cursor = &args[0];
            }
            assert!(cursor == &Expr::small_integer(0));
            let mut pending = vec![root];
            while let Some(node) = pending.pop() {
                if let Expr::Call { head, args } = node {
                    pending.push(head.as_ref().clone());
                    pending.extend(args.iter().cloned());
                }
            }
            println!(
                "CONSTRUCTION_TEARDOWN n=2000 stack=2MiB iterative inspection and teardown passed"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
