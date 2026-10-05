use super::session;
use rum_core::Expr;

fn on_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .name("schema-2mib".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}
fn exact_chain(mut value: &Expr, kind: &str, n: usize) {
    for _ in 0..n {
        let Expr::Call { head, args } = value else {
            panic!("missing container");
        };
        assert_eq!(
            head.to_string(),
            if kind == "list" {
                "List"
            } else {
                "Association"
            }
        );
        assert_eq!(args.len(), 1);
        value = &args[0];
        if kind == "association" {
            let Expr::Call { head, args } = value else {
                panic!("missing rule");
            };
            assert_eq!(head.to_string(), "Rule");
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], Expr::string("k"));
            value = &args[1];
        }
    }
    assert!(
        value == &Expr::small_integer(1),
        "every level and final scalar leaf"
    );
}
fn make_schema(kind: &str, n: usize, leaf: i64) -> Expr {
    let mut schema = Expr::small_integer(leaf);
    for _ in 0..n {
        schema = if kind == "list" {
            Expr::call(Expr::system_symbol("List"), vec![schema])
        } else {
            Expr::call(
                Expr::system_symbol("Association"),
                vec![Expr::call(
                    Expr::system_symbol("Rule"),
                    vec![Expr::string("k"), schema],
                )],
            )
        };
    }
    schema
}
// ConstantArray constructs a fresh nested result without native recursion.
// No equivalent fresh association constructor exists; its list result is the
// director-authorized control for associations as well.
fn fresh_result_control(
    s: &mut rum_runtime::Session,
    kind: &str,
    n: usize,
    twice: bool,
) -> Option<rum_runtime::EvalTerminationReason> {
    let dimensions = if twice {
        format!("Prepend[Table[1, {{{}}}], 2]", n - 1)
    } else {
        format!("Table[1, {{{n}}}]")
    };
    let control = s
        .eval_request(&format!("ConstantArray[1, {dimensions}]"))
        .unwrap();
    println!(
        "FRESH CONTROL kind={kind} n={n} twice={twice} constructor=ConstantArray termination={:?}",
        control.termination
    );
    if let Some(reason) = control.termination {
        assert_eq!(reason, rum_runtime::EvalTerminationReason::StackLimit);
    } else if twice {
        let Expr::Call { head, args } = &control.value else {
            panic!("missing control list");
        };
        assert_eq!(head.to_string(), "List");
        assert_eq!(args.len(), 2);
        for child in args.iter() {
            exact_chain(child, "list", n - 1);
        }
    } else {
        exact_chain(&control.value, "list", n);
    }
    control.termination
}

fn depth_case(kind: &'static str, mode: &'static str, fixed_n: usize) {
    let n = fixed_n; // Fixed acceptance depth; never silently lower it.
    on_stack(move || {
        let mut s = session(None);
        let source = if kind == "list" {
            format!("v=1;Do[v={{v}},{{{n}}}];Null")
        } else {
            format!("v=1;Do[v=<|\"k\"->v|>,{{{n}}}];Null")
        };
        let setup = s.eval_request(&source).unwrap();
        assert_eq!(setup.termination, None, "descriptor construction guard");
        let value = s.eval_request("v").unwrap();
        assert_eq!(value.termination, None);
        exact_chain(&value.value, kind, n);
        let schema = make_schema(kind, n, 1);
        exact_chain(&schema, kind, n);
        println!("BUILT kind={kind} mode={mode} n={n}");
        if mode != "control" {
            let expected_termination = if mode == "literal" {
                fresh_result_control(&mut s, kind, n, false)
            } else {
                None
            };
            let schema = if mode == "mismatch" {
                make_schema(kind, n, 2)
            } else {
                schema
            };
            let builtin = if mode == "errors" || mode == "mismatch" {
                "SchemaErrors"
            } else {
                "ConformSchema"
            };
            let call = Expr::call(
                Expr::system_symbol(builtin),
                vec![Expr::contextual_symbol("Global`", "v"), schema],
            );
            let result = s.eval_expr_request(call).unwrap();
            assert_eq!(
                result.termination, expected_termination,
                "same fresh-result height contract"
            );
            println!(
                "VALIDATED kind={kind} mode={mode} n={n} termination={:?}",
                result.termination
            );
            if result.termination.is_none() {
                assert!(
                    result.messages.is_empty(),
                    "messages: {:?}",
                    result.messages
                );
            }
            if mode == "errors" {
                assert_eq!(
                    result.value,
                    Expr::call(Expr::system_symbol("List"), vec![])
                );
            } else if mode == "mismatch" {
                let path = if kind == "list" {
                    "[1]".repeat(n)
                } else {
                    vec!["k"; n].join(".")
                };
                let expected = Expr::call(
                    Expr::system_symbol("List"),
                    vec![Expr::call(
                        Expr::system_symbol("Association"),
                        vec![
                            Expr::call(
                                Expr::system_symbol("Rule"),
                                vec![Expr::string("Path"), Expr::string(path)],
                            ),
                            Expr::call(
                                Expr::system_symbol("Rule"),
                                vec![
                                    Expr::string("Message"),
                                    Expr::string("expected to match 2, got 1"),
                                ],
                            ),
                        ],
                    )],
                );
                assert_eq!(result.value, expected);
            } else if result.termination.is_none() {
                exact_chain(&result.value, kind, n);
            } else {
                assert_eq!(
                    result.value.to_string(),
                    "TerminatedEvaluation[\"StackLimit\"]"
                );
            }
            assert!(s.pending_input_request().is_none());
            assert_eq!(s.eval_request("1+1").unwrap().value, Expr::small_integer(2));
        }
        drop(value);
        drop(s);
        println!("PASSED kind={kind} mode={mode} n={n} including teardown");
    });
}
#[test]
fn depth_control_list() {
    depth_case("list", "control", 1000);
}
#[test]
fn depth_control_association() {
    depth_case("association", "control", 500);
}
#[test]
fn depth_list_literal() {
    depth_case("list", "literal", 1000);
}
#[test]
fn depth_association_literal() {
    depth_case("association", "literal", 500);
}
#[test]
fn depth_schemaerrors_list() {
    depth_case("list", "errors", 1000);
}
#[test]
fn depth_schemaerrors_association() {
    depth_case("association", "errors", 500);
}
#[test]
fn depth_mismatch_list() {
    depth_case("list", "mismatch", 1000);
}
#[test]
fn depth_mismatch_association() {
    depth_case("association", "mismatch", 500);
}
#[test]
fn shallow_list_literal() {
    depth_case("list", "literal", 16);
}
#[test]
fn shallow_association_literal() {
    depth_case("association", "literal", 16);
}

fn wrap_chain(kind: &str, n: usize, mut leaf: Expr) -> Expr {
    for _ in 0..n {
        leaf = if kind == "list" {
            Expr::call(Expr::system_symbol("List"), vec![leaf])
        } else {
            Expr::call(
                Expr::system_symbol("Association"),
                vec![Expr::call(
                    Expr::system_symbol("Rule"),
                    vec![Expr::string("k"), leaf],
                )],
            )
        };
    }
    leaf
}

fn deep_lifecycle(kind: &'static str, n: usize, mode: &'static str) {
    on_stack(move || {
        let mut s = session(None);
        let twice = mode != "single";
        let levels = if twice { n - 1 } else { n };
        let expected_termination = fresh_result_control(&mut s, kind, n, twice);
        let setup = if kind == "list" {
            format!(
                "v=1;Do[v={{v}},{{{levels}}}];{}Null",
                if twice { "v={v,v};" } else { "" }
            )
        } else {
            format!(
                "v=1;Do[v=<|\"k\"->v|>,{{{levels}}}];{}Null",
                if twice {
                    "v=<|\"k\"->v,\"j\"->v|>;"
                } else {
                    ""
                }
            )
        };
        assert!(s.eval_request(&setup).unwrap().termination.is_none());
        let leaf = s
            .lower_text(
                r#"_?Function[x,(AppendTo[log,1];InputString["deep: "];AppendTo[log,2];True)]"#,
            )
            .unwrap();
        let inner = wrap_chain(kind, levels, leaf);
        let schema = if twice && kind == "list" {
            Expr::call(Expr::system_symbol("List"), vec![inner])
        } else if twice {
            Expr::call(
                Expr::system_symbol("Association"),
                vec![
                    Expr::call(
                        Expr::system_symbol("Rule"),
                        vec![Expr::string("k"), inner.clone()],
                    ),
                    Expr::call(Expr::system_symbol("Rule"), vec![Expr::string("j"), inner]),
                ],
            )
        } else {
            inner
        };
        let call = Expr::call(
            Expr::system_symbol("ConformSchema"),
            vec![Expr::contextual_symbol("Global`", "v"), schema],
        );
        println!("LIFECYCLE BUILT kind={kind} mode={mode} n={n}");
        let cancel = s.request_cancel_handle();
        let mut outcome = s.eval_expr_request(call);
        let count = if twice { 2 } else { 1 };
        for index in 0..count {
            let Err(failure) = &outcome else {
                panic!("must suspend at deep leaf");
            };
            let rum_runtime::SessionRequestError::InputRequired { request } = &failure.error else {
                panic!("must request deep input");
            };
            assert_eq!(request.prompt, "deep: ");
            assert_eq!(s.pending_input_request(), Some(request));
            assert!(failure.messages.is_empty());
            assert!(failure.prints.is_empty());
            println!(
                "LIFECYCLE SUSPEND kind={kind} mode={mode} n={n} ordinal={}",
                index + 1
            );
            if mode == "abandon" && index == 1 {
                cancel.cancel();
                // The Session request API consumes cancellation when input is
                // resumed, abandoning queued frames before evaluating the reply.
                outcome = s.resume_eval_request_with_input("unused");
            } else {
                outcome = s.resume_eval_request_with_input("go");
            }
        }
        let result = outcome.unwrap();
        if mode == "abandon" {
            assert_eq!(format!("{:?}", result.termination), "Some(Cancelled)");
            assert_eq!(
                s.eval_request("log").unwrap().value.to_string(),
                "List[1, 2, 1]"
            );
        } else {
            assert_eq!(
                result.termination, expected_termination,
                "same fresh-result height contract"
            );
            println!(
                "LIFECYCLE RESULT kind={kind} mode={mode} n={n} termination={:?}",
                result.termination
            );
            if result.termination.is_none() {
                assert!(result.messages.is_empty());
            }
            if result.termination.is_some() {
                assert_eq!(
                    result.value.to_string(),
                    "TerminatedEvaluation[\"StackLimit\"]"
                );
            } else if twice {
                let Expr::Call { head, args } = &result.value else {
                    panic!("missing outer container");
                };
                assert_eq!(
                    head.to_string(),
                    if kind == "list" {
                        "List"
                    } else {
                        "Association"
                    }
                );
                assert_eq!(args.len(), 2);
                for (index, child) in args.iter().enumerate() {
                    let child = if kind == "association" {
                        let Expr::Call { head, args } = child else {
                            panic!("missing rule");
                        };
                        assert_eq!(head.to_string(), "Rule");
                        assert_eq!(args.len(), 2);
                        assert_eq!(args[0], Expr::string(if index == 0 { "k" } else { "j" }));
                        &args[1]
                    } else {
                        child
                    };
                    exact_chain(child, kind, levels);
                }
            } else {
                exact_chain(&result.value, kind, n);
            }
            assert_eq!(
                s.eval_request("log").unwrap().value.to_string(),
                if twice {
                    "List[1, 2, 1, 2]"
                } else {
                    "List[1, 2]"
                }
            );
        }
        assert!(s.pending_input_request().is_none());
        assert_eq!(s.eval_request("1+1").unwrap().value, Expr::small_integer(2));
        drop(result);
        drop(s);
        println!("LIFECYCLE PASSED kind={kind} mode={mode} n={n} including Session teardown");
    });
}

#[test]
fn lifecycle_list_suspension() {
    deep_lifecycle("list", 1000, "single");
}
#[test]
fn lifecycle_list_twice() {
    deep_lifecycle("list", 1000, "twice");
}
#[test]
fn lifecycle_list_abandonment() {
    deep_lifecycle("list", 1000, "abandon");
}
#[test]
fn lifecycle_association_suspension() {
    deep_lifecycle("association", 500, "single");
}
#[test]
fn lifecycle_association_twice() {
    deep_lifecycle("association", 500, "twice");
}
#[test]
fn lifecycle_association_abandonment() {
    deep_lifecycle("association", 500, "abandon");
}
