//! Schema-owned recursion and ownership depth contract.
use super::recursion_inventory::recursive_edges_from_source;

#[test]
fn structural_no_schema_recursion() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut pending = vec![root.join("schema.rs"), root.join("schema")];
    let mut edges = Vec::new();
    let mut combined = String::new();
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            for e in std::fs::read_dir(&path).unwrap() {
                pending.push(e.unwrap().path());
            }
        } else if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
            let source = std::fs::read_to_string(&path).unwrap();
            edges.extend(recursive_edges_from_source(path.to_str().unwrap(), &source));
            combined.push_str(
                &source
                    .lines()
                    .filter(|l| {
                        !l.trim_start().starts_with("//!") && !l.trim_start().starts_with("#![")
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            combined.push('\n');
        }
    }
    edges.extend(recursive_edges_from_source(
        "schema (all modules)",
        &combined,
    ));
    println!("SCHEMA_RECURSION {edges:?}");
    assert!(
        edges.is_empty(),
        "schema recursion must be eliminated: {edges:?}"
    );
}
#[test]
fn structural_recursion_falsifiers() {
    let one = "fn schema_drive() { schema_drive(); }";
    let edges = recursive_edges_from_source("schema/machine.rs", one);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].caller, "schema_drive");
    let a = "fn schema_drive() { fold_child(); }";
    let b = "fn fold_child() { schema_drive(); }";
    assert!(recursive_edges_from_source("a", a).is_empty());
    assert!(recursive_edges_from_source("b", b).is_empty());
    let edges = recursive_edges_from_source("all", &format!("{a}\n{b}"));
    assert_eq!(edges.len(), 2);
    println!("CROSS_MODULE_FALSIFIER {edges:?}");
}
fn assert_no_self_reaching(text: &str) {
    assert!(
        !text.lines().any(|l| l.contains("rum_stdlib::schema::")),
        "self-reaching schema ownership"
    );
}
#[test]
fn structural_no_schema_self_reaching() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/tests/ownership_graph/inventory/self_reaching.tsv"
    ))
    .unwrap();
    assert_no_self_reaching(&text);
}
#[test]
#[should_panic(expected = "self-reaching schema ownership")]
fn structural_self_reaching_falsifier() {
    assert_no_self_reaching(
        "rum_stdlib::schema::state::SchemaWork\tschema/state.rs\t1\towned child",
    );
}

use crate::schema::{continuation::SchemaValidationSuspensionFrame, state::*};
use rum_core::{Expr, Symbol};
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

struct SchemaRoots(Vec<Expr>);
impl FrameRootVisitor for SchemaRoots {
    fn visit_expr(&mut self, expr: &Expr) {
        // Every fixture root is scalar: the visitor adds no recursive Expr walk.
        self.0.push(expr.clone());
    }
    fn visit_symbol(&mut self, _: &Symbol) {
        panic!("the schema state has no bare Symbol fields");
    }
}

fn schema_roots(value: &impl FrameRootTrace) -> Vec<Expr> {
    let mut roots = SchemaRoots(Vec::new());
    value.trace_roots(&mut roots);
    roots.0
}

fn root_fixture() -> (SchemaState, Vec<Expr>) {
    let roots: Vec<_> = (1..=16).map(Expr::small_integer).collect();
    let list = ListWork {
        items: roots[0..2].to_vec().into(),
        item_schema: roots[2].clone(),
        next: 1,
        output: roots[3..5].to_vec(),
        path_len: 0,
        waiting: Some(SchemaDestination::ListItem),
    };
    let association = AssociationWork {
        schema_entries: vec![SchemaEntry {
            key: "spec".into(),
            spec: roots[5].clone(),
        }],
        value_entries: vec![
            (roots[6].clone(), roots[7].clone()),
            (roots[8].clone(), roots[9].clone()),
        ],
        next_schema: 1,
        matched: vec![true, true],
        conformed_by_index: vec![Some(roots[10].clone()), Some(roots[11].clone())],
        default_entries: vec![
            (roots[12].clone(), roots[13].clone()),
            (roots[14].clone(), roots[15].clone()),
        ],
        positions: Some(std::collections::HashMap::from([("key".into(), 0)])),
        path_len: 0,
        waiting: Some(SchemaDestination::ExistingEntry { index: 1 }),
    };
    let state = SchemaState {
        mode: SchemaMode::Conform,
        options: SchemaOptions {
            allow_extra_keys: true,
        },
        stack: vec![
            SchemaContainer::List(list),
            SchemaContainer::Association(association),
        ],
        path: SchemaPath {
            rendered: "prefix.λ[2]".into(),
        },
        errors: vec![SchemaError {
            path: "earlier".into(),
            message: "saved error".into(),
        }],
    };
    (state, roots)
}

fn assert_schema_roots(value: &impl FrameRootTrace, expected: &[Expr]) {
    assert_eq!(
        schema_roots(value),
        expected,
        "every schema Expr-bearing field must be traced"
    );
}

#[test]
fn schema_frame_roots_are_complete_and_flat_clone_drop_is_bounded() {
    std::thread::Builder::new().name("schema-roots-2mib".into())
        .stack_size(2 * 1024 * 1024).spawn(|| {
            let (state, roots) = root_fixture();
            assert_schema_roots(&state, &roots);
            let leaf = SchemaValidationSuspensionFrame {
                state: state.clone(),
                pending: SchemaAwait::Leaf { value: Expr::small_integer(17), schema: Expr::small_integer(18) },
            };
            let mut expected = roots.clone();
            expected.extend([Expr::small_integer(17), Expr::small_integer(18)]);
            assert_schema_roots(&leaf, &expected);
            let default = SchemaValidationSuspensionFrame {
                state: state.clone(), pending: SchemaAwait::Default { spec: Expr::small_integer(19) },
            };
            let mut expected = roots.clone();
            expected.push(Expr::small_integer(19));
            assert_schema_roots(&default, &expected);
            let mut deep = state.clone();
            deep.stack.clear();
            let mut expected = Vec::new();
            for _ in 0..1000 {
                deep.stack.push(state.stack[0].clone());
                expected.extend_from_slice(&roots[..5]);
            }
            for _ in 0..500 {
                let mut container = state.stack[1].clone();
                if let SchemaContainer::Association(work) = &mut container {
                    work.waiting = Some(SchemaDestination::DefaultEntry { key: "saved".into() });
                }
                deep.stack.push(container);
                expected.extend_from_slice(&roots[5..]);
            }
            deep.mode = SchemaMode::Errors;
            let copy = deep.clone();
            assert_schema_roots(&deep, &expected);
            assert_schema_roots(&copy, &expected);
            drop(copy); drop(deep); drop(default); drop(leaf); drop(state);
            println!("SCHEMA_ROOTS every field and both pending phases; clone/trace/drop list=1000 association=500 on 2 MiB");
        }).unwrap().join().unwrap();
}

#[test]
#[should_panic(expected = "every schema Expr-bearing field must be traced")]
fn schema_root_completeness_falsifier() {
    let (mut state, expected) = root_fixture();
    let SchemaContainer::Association(work) = &mut state.stack[1] else {
        unreachable!()
    };
    work.default_entries.clear();
    assert_schema_roots(&state, &expected);
}

fn schema_direct_depth(kind: &'static str, mode: &'static str, n: usize) {
    std::thread::Builder::new()
        .name("schema-direct-2mib".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let mut session = rum_runtime::Session::new();
            crate::install_core(&mut session);
            let container = if kind == "list" {
                "{v}"
            } else {
                "<|\"k\"->v|>"
            };
            let setup = session
                .eval_request(&format!("v=1;Do[v={container},{{{n}}}];Null"))
                .unwrap();
            assert!(setup.termination.is_none());
            let mut schema = Expr::small_integer(1);
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
            let args = vec![Expr::contextual_symbol("Global`", "v"), schema].into();
            let result = if mode == "conform" {
                crate::schema::eval_conform_schema(
                    &mut session,
                    &Expr::system_symbol("ConformSchema"),
                    &args,
                )
            } else {
                crate::schema::eval_schema_errors(
                    &mut session,
                    &Expr::system_symbol("SchemaErrors"),
                    &args,
                )
            }
            .expect("direct validator must complete");
            if mode == "errors" {
                assert_eq!(result, Expr::call(Expr::system_symbol("List"), vec![]));
            } else {
                let mut cursor = &result;
                for _ in 0..n {
                    let Expr::Call { head, args } = cursor else {
                        panic!("missing container level");
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
                    cursor = &args[0];
                    if kind == "association" {
                        let Expr::Call { head, args } = cursor else {
                            panic!("missing rule");
                        };
                        assert_eq!(head.to_string(), "Rule");
                        assert_eq!(args.len(), 2);
                        assert_eq!(args[0], Expr::string("k"));
                        cursor = &args[1];
                    }
                }
                assert!(cursor == &Expr::small_integer(1), "exact terminal scalar");
            }
            drop(result);
            drop(args);
            drop(session);
            println!("DIRECT {mode} {kind} n={n} exact; 2 MiB including Session teardown");
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn schema_direct_list_depth_isolates_the_validator_from_result_evaluation() {
    schema_direct_depth("list", "conform", 1000);
}
#[test]
fn schema_direct_association_depth_isolates_the_validator_from_result_evaluation() {
    schema_direct_depth("association", "conform", 500);
}
#[test]
fn schema_direct_errors_list_depth() {
    schema_direct_depth("list", "errors", 1000);
}
#[test]
fn schema_direct_errors_association_depth() {
    schema_direct_depth("association", "errors", 500);
}
