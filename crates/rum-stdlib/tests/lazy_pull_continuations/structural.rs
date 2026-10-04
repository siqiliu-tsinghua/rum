use super::*;

probe!(
    last_take_child,
    "last permitted Take child",
    r#"Normal[LazyTake[LazyMap[Function[x,(AppendTo[log,x];InputString["p: "];x)],LazyRange[1]],1]]"#,
    &["one"],
    None
);
probe!(
    alternating_take_drop,
    "alternating Take/Drop skip and yield",
    r#"Normal[LazyTake[LazyDrop[LazyTake[LazyDrop[LazyMap[Function[x,(AppendTo[log,x];If[Or[x==2,x==4],InputString["p: "]];x)],LazyRange[1]],1],5],2],2]]"#,
    &["skip", "yield"],
    None
);
probe!(
    fold_init_above_suspending_adapter,
    "Fold init above suspending Map",
    r#"Take[LazyMap[Function[x,(AppendTo[log,{outer,x}];InputString["p: "];x)],LazyFoldList[Plus,0,LazyMap[Function[x,(AppendTo[log,{inner,x}];InputString["p: "];x)],LazyRange[1]]]],2]"#,
    &["init", "child", "fold"],
    Some(1)
);
probe!(
    finite_positive_drop,
    "positive Drop finite exhaustion",
    r#"Normal[LazyDrop[LazyMap[Function[x,(AppendTo[log,x];If[x==2,InputString["p: "]];x)],LazyTake[LazyRange[1],2]],3]]"#,
    &["two"],
    None
);
probe!(
    finite_rejected_select,
    "resumed Select rejects last finite item",
    r#"First[LazySelect[Function[x,(AppendTo[log,x];Input["p: "])],LazyPermutations[{}]],99]"#,
    &["False"],
    None
);
probe!(
    nested_realization,
    "callback realizes a suspending lazy list",
    r#"Take[LazyMap[Function[x,(AppendTo[log,{outer,x}];First[LazyMap[Function[y,(AppendTo[log,{inner,y}];InputString["p: "];y+x)],LazyRange[10]]])],LazyRange[1]],2]"#,
    &["first", "second"],
    None
);

#[test]
fn one_callback_two_inputs() {
    let source = r#"Take[LazyMap[Function[x,(AppendTo[log,x];InputString["first: "];InputString["second: "];x+1)],LazyRange[1]],1]"#;
    let resumed = run(source, &["a", "b"], false, None);
    let queued = run(source, &["a", "b"], true, None);
    record("one callback two inputs", &resumed, &queued, 2);
    assert_eq!(resumed.2, ["first: ", "second: "]);
    assert!(queued.2.is_empty());
    assert_eq!(resumed.0, queued.0);
    assert_eq!(resumed.1, queued.1);
    assert_eq!(queued.0.value.as_ref().unwrap().to_string(), "List[2]");
    assert_eq!(queued.1, "List[1]");
}
