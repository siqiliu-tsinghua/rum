use super::support::compare;

#[test]
fn first_leaf_returns_complete_rhs() {
    compare(
        "full-rhs",
        r#"a={0};b=0;{a[[(AppendTo[log,1];InputString["p: "];1)]],b}={7,9}"#,
        &["p: "],
        "{a,b}",
        Some("List[7, 9]"),
        "List[1]",
        "List[List[7], 9]",
    );
}

#[test]
fn enclosing_assignment_keeps_complete_rhs() {
    compare(
        "enclosing",
        r#"a={0};b=0;r=({a[[(AppendTo[log,1];InputString["p: "];1)]],b}={7,9});{r,a,b}"#,
        &["p: "],
        "{r,a,b}",
        Some("List[List[7, 9], List[7], 9]"),
        "List[1]",
        "List[List[7, 9], List[7], 9]",
    );
}

#[test]
fn two_targets_suspend_in_store_order() {
    compare(
        "twice",
        r#"a={0,0};b=0;r=({a[[(AppendTo[log,1];InputString["p: "];1)]],a[[(AppendTo[log,2];InputString["q: "];2)]],b}={7,8,9});{r,a,b}"#,
        &["p: ", "q: "],
        "{r,a,b}",
        Some("List[List[7, 8, 9], List[7, 8], 9]"),
        "List[1, 2]",
        "List[List[7, 8, 9], List[7, 8], 9]",
    );
}

#[test]
fn nested_witness_finishes_parent_sibling() {
    compare(
        "nested",
        r#"a={0};b=0;r=({{a[[(AppendTo[log,1];InputString["p: "];1)]]},b}={{7},9});{r,a,b}"#,
        &["p: "],
        "{r,a,b}",
        Some("List[List[List[7], 9], List[7], 9]"),
        "List[1]",
        "List[List[List[7], 9], List[7], 9]",
    );
}

#[test]
fn completed_prefix_and_middle_pause_keep_ordered_output() {
    let out = compare(
        "middle",
        r#"x=0;a={0};b=0;{x,a[[(AppendTo[log,x];Print["before"];Message[probe::tag];InputString["middle: "];AppendTo[log,2];Print["after"];Message[probe::tag];1)]],b}={5,7,9}"#,
        &["middle: "],
        "{x,a,b}",
        Some("List[5, 7, 9]"),
        "List[5, 2]",
        "List[5, List[7], 9]",
    );
    assert_eq!(out.view.prints, ["before", "after"]);
    assert_eq!(
        out.view
            .messages
            .iter()
            .map(|m| (m.symbol.as_str(), m.tag.as_str()))
            .collect::<Vec<_>>(),
        [("probe", "tag"), ("probe", "tag")]
    );
}

#[test]
fn last_leaf_still_returns_whole_rhs() {
    compare(
        "last",
        r#"x=0;a={0};{x,a[[(AppendTo[log,x];InputString["last: "];1)]]}={5,7}"#,
        &["last: "],
        "{x,a}",
        Some("List[5, 7]"),
        "List[5]",
        "List[5, List[7]]",
    );
}

#[test]
fn ordinary_later_error_after_resume_preserves_only_committed_prefix() {
    let out = compare(
        "later-error",
        r#"x=0;a={0,0};b=0;{x,a[[(AppendTo[log,1];Print["before"];Message[probe::tag];InputString["p: "];1)]],a[[(AppendTo[log,2];Print["error"];Message[probe::tag];Throw["leaf boom"])]],b}={5,7,8,9}"#,
        &["p: "],
        "{x,a,b}",
        None,
        "List[1, 2]",
        "List[5, List[7, 0], 0]",
    );
    assert_ne!(out.view.status, "OK");
    assert!(out.view.status.contains("leaf boom"));
    assert_eq!(out.view.prints, ["before", "error"]);
    assert_eq!(
        out.view
            .messages
            .iter()
            .take(2)
            .map(|m| (m.symbol.as_str(), m.tag.as_str()))
            .collect::<Vec<_>>(),
        [("probe", "tag"), ("probe", "tag")]
    );
}

#[test]
fn nested_shape_mismatch_validates_before_any_store_or_prompt() {
    let out = compare(
        "shape",
        r#"x=0;a={0};b=0;{x,{a[[(AppendTo[log,1];InputString["unread: "];1)]],b}}={5,{7}}"#,
        &[],
        "{x,a,b}",
        Some("List[5, List[7]]"),
        "List[]",
        "List[0, List[0], 0]",
    );
    assert_eq!(
        out.view
            .messages
            .iter()
            .map(|m| (m.symbol.as_str(), m.tag.as_str()))
            .collect::<Vec<_>>(),
        [("Set", "shape")]
    );
    assert!(out.view.prints.is_empty());
}

#[test]
fn nested_lists_keep_work_between_pauses_and_after_last_pause() {
    compare(
        "nested-twice",
        r#"x=0;y=0;b=0;a={0,0};{x,{a[[(AppendTo[log,x];InputString["p: "];1)]],y},a[[(AppendTo[log,y];InputString["q: "];2)]],b}={5,{7,6},8,9}"#,
        &["p: ", "q: "],
        "{x,y,a,b}",
        Some("List[5, List[7, 6], 8, 9]"),
        "List[5, 6]",
        "List[5, 6, List[7, 8], 9]",
    );
}

#[test]
fn empty_lists_and_list_valued_leaf_keep_original_rhs() {
    compare(
        "empty-and-list-leaf",
        r#"a={0};b=0;{{},a[[(InputString["p: "];1)]],b}={{},7,{8,9}}"#,
        &["p: "],
        "{a,b}",
        Some("List[List[], 7, List[8, 9]]"),
        "List[]",
        "List[List[7], List[8, 9]]",
    );
}

#[test]
fn association_and_definition_lvalues_continue_to_next_store() {
    compare(
        "mixed-lvalues",
        r#"a=<|"k"->0|>;b=0;{a["k"],f[(AppendTo[log,1];InputString["definition: "];2)],b}={7,8,9}"#,
        &["definition: "],
        r#"{a["k"],f[2],b}"#,
        Some("List[7, 8, 9]"),
        "List[1]",
        "List[7, 8, 9]",
    );
}

#[test]
fn deeper_parent_stack_resumes_through_each_level() {
    compare(
        "parent-spill",
        r#"a={0};x=0;y=0;b=0;{{{a[[(AppendTo[log,1];InputString["deep: "];1)]],x},y},b}={{{7,5},6},9}"#,
        &["deep: "],
        "{a,x,y,b}",
        Some("List[List[List[7, 5], 6], 9]"),
        "List[1]",
        "List[List[7], 5, 6, 9]",
    );
}
