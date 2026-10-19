//! G15-B: definition LHS preparation recovers only a cycle of its source symbol.
//! Held definitions and their stored value tables agree with queued input after
//! every pause (language guide: definitions, evaluation and control flow).
use super::super::descriptors::{Checks, accept};

fn equality(
    name: &str,
    setup: &str,
    source: &str,
    prompts: &[&str],
    value: &str,
    projection: &str,
    stored: &str,
) {
    let pauses: Vec<_> = prompts.iter().map(|p| ("InputString", *p, "1")).collect();
    let mut fixture = super::make(name, source, &pauses, Some(value), "List[]");
    fixture.setup = format!("log = {{}};{setup}");
    fixture.projections.push(projection.into());
    fixture.expected_projections.push(stored.into());
    fixture.pause_prints = Some(vec![vec![]; prompts.len()]);
    fixture.pause_messages = Some(vec![vec![]; prompts.len()]);
    fixture.pause_trace = Some(vec![vec![]; prompts.len()]);
    // ALL includes exact phase prompts, pending ownership, overlap rejection,
    // exhausted queues and a fresh request. Fault absence is unconditional.
    let result = accept(&fixture, Checks::ALL);
    if result.is_err() {
        println!(
            "{name} queued: {:?}",
            super::super::descriptors::run(&fixture, true, Checks::ALL)
        );
        println!(
            "{name} resumed: {:?}",
            super::super::descriptors::run(&fixture, false, Checks::ALL)
        );
    }
    super::super::support::faults::assert_all();
    let accepted = result.unwrap_or_else(|error| panic!("{name}: {error:?}"));
    println!(
        "G15-B {name} pauses={} PASS: {:?}; stored={:?}",
        prompts.len(),
        accepted.observation,
        accepted.projections
    );
}

macro_rules! case {
    ($name:ident, $setup:expr, $source:expr, $prompts:expr, $value:expr, $projection:expr, $stored:expr) => {
        #[test]
        fn $name() {
            equality(
                concat!("G15B.", stringify!($name)),
                $setup,
                $source,
                $prompts,
                $value,
                $projection,
                $stored,
            );
        }
    };
}

case!(
    arg_set,
    "x := (InputString[\"w\"]; x)",
    "{f[x] = 3, DownValues[f]}",
    &["w"],
    "List[3, List[RuleDelayed[HoldPattern[f[x]], 3]]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[x]], 3]]"
);

case!(
    head_set,
    "x := (InputString[\"w\"]; x)",
    "{x[1] = 3, SubValues[x]}",
    &["w"],
    "List[3, List[]]",
    "{DownValues[x], SubValues[x]}",
    "List[List[RuleDelayed[HoldPattern[x[1]], 3]], List[]]"
);

case!(
    pattern_blank,
    "x := (InputString[\"w\"]; x)",
    "{f[Blank[x]] = 3, DownValues[f]}",
    &["w"],
    "List[3, List[RuleDelayed[HoldPattern[f[Blank[x]]], 3]]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[Blank[x]]], 3]]"
);

case!(
    pattern_named,
    "x := (InputString[\"w\"]; x)",
    "{f[x_x] = 3, DownValues[f]}",
    &["w"],
    "List[3, List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[x]]]], 3]]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[Pattern[x, Blank[x]]]], 3]]"
);

case!(
    arg_setdelayed,
    "x := (InputString[\"w\"]; x)",
    "{f[x] := 3, DownValues[f]}",
    &["w"],
    "List[Null, List[RuleDelayed[HoldPattern[f[x]], 3]]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[x]], 3]]"
);

case!(
    arg_upset,
    "x := (InputString[\"w\"]; x)",
    "{g[x] ^= 3, UpValues[x]}",
    &["w"],
    "List[3, List[RuleDelayed[HoldPattern[g[x]], 3]]]",
    "UpValues[x]",
    "List[RuleDelayed[HoldPattern[g[x]], 3]]"
);

case!(
    arg_upsetdelayed,
    "x := (InputString[\"w\"]; x)",
    "{g[x] ^:= 3, UpValues[x]}",
    &["w"],
    "List[Null, List[RuleDelayed[HoldPattern[g[x]], 3]]]",
    "UpValues[x]",
    "List[RuleDelayed[HoldPattern[g[x]], 3]]"
);

case!(
    arg_tagset,
    "x := (InputString[\"w\"]; x)",
    "{x /: g[x] = 3, UpValues[x]}",
    &["w"],
    "List[3, List[RuleDelayed[HoldPattern[g[x]], 3]]]",
    "UpValues[x]",
    "List[RuleDelayed[HoldPattern[g[x]], 3]]"
);

case!(
    arg_tagsetdelayed,
    "x := (InputString[\"w\"]; x)",
    "{x /: g[x] := 3, UpValues[x]}",
    &["w"],
    "List[Null, List[RuleDelayed[HoldPattern[g[x]], 3]]]",
    "UpValues[x]",
    "List[RuleDelayed[HoldPattern[g[x]], 3]]"
);

case!(
    arg_tagunset,
    "x /: g[x] = 3; x := (InputString[\"w\"]; x)",
    "{x /: g[x] =., UpValues[x]}",
    &["w"],
    "List[Null, List[]]",
    "UpValues[x]",
    "List[]"
);

case!(
    arg_unset,
    "f[x] = 3; x := (InputString[\"w\"]; x)",
    "{f[x] =., DownValues[f]}",
    &["w"],
    "List[Null, List[]]",
    "DownValues[f]",
    "List[]"
);

case!(
    arg_two_cycles,
    "x := (InputString[\"w\"]; x); y := (InputString[\"v\"]; y)",
    "{f[x, y] = 3, DownValues[f]}",
    &["w", "v"],
    "List[3, List[RuleDelayed[HoldPattern[f[x, y]], 3]]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[x, y]], 3]]"
);

case!(
    arg_same_cycle_twice,
    "x := (InputString[\"w\"]; x)",
    "{f[x, x] = 3, DownValues[f]}",
    &["w", "w"],
    "List[3, List[RuleDelayed[HoldPattern[f[x, x]], 3]]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[x, x]], 3]]"
);

case!(
    arg_after_prior,
    "z := (InputString[\"u\"]; 2); x := (InputString[\"w\"]; x)",
    "{f[z, x] = 3, DownValues[f]}",
    &["u", "w"],
    "List[3, List[RuleDelayed[HoldPattern[f[2, x]], 3]]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[2, x]], 3]]"
);

case!(
    arg_do,
    "x := (InputString[\"w\"]; x)",
    "Do[f[x] = i, {i, 2}]; DownValues[f]",
    &["w", "w"],
    "List[RuleDelayed[HoldPattern[f[x]], 2]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[x]], 2]]"
);

case!(
    arg_table,
    "x := (InputString[\"w\"]; x)",
    "Table[f[x] = i, {i, 2}]",
    &["w", "w"],
    "List[1, 2]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[x]], 2]]"
);

case!(
    owned_cycle,
    "x := (InputString[\"w\"]; x)",
    "f[x]=3",
    &["w"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[x]], 3]]"
);

case!(
    owned_cycle_head,
    "x := (InputString[\"w\"]; x)",
    "x[1]=3",
    &["w"],
    "3",
    "{DownValues[x], SubValues[x]}",
    "List[List[RuleDelayed[HoldPattern[x[1]], 3]], List[]]"
);

case!(
    owned_cycle_pattern,
    "x := (InputString[\"w\"]; x)",
    "f[Blank[x]]=3",
    &["w"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[Blank[x]]], 3]]"
);

case!(
    owned_cycle_twice,
    "x := (InputString[\"w\"]; x);y:=(InputString[\"w\"];y)",
    "f[x,y]=3",
    &["w", "w"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[x, y]], 3]]"
);

case!(
    owned_after_prior_pause,
    "y:=(InputString[\"w\"];2);x := (InputString[\"w\"]; x)",
    "f[y,x]=3",
    &["w", "w"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[2, x]], 3]]"
);

case!(
    control_other_symbol,
    "x:=y;y := (InputString[\"v\"]; y)",
    "{f[x]=3,DownValues[f]}",
    &["v"],
    "TerminatedEvaluation[\"IterationLimit\"]",
    "DownValues[f]",
    "List[]"
);

case!(
    control_non_symbol,
    "x := (InputString[\"w\"]; x)",
    "{f[g[x]]=3,DownValues[f]}",
    &["w"],
    "TerminatedEvaluation[\"IterationLimit\"]",
    "DownValues[f]",
    "List[]"
);

case!(
    control_throw,
    "x:=(InputString[\"w\"];Throw[3])",
    "Catch[f[x]=4]",
    &["w"],
    "3",
    "DownValues[f]",
    "List[]"
);

case!(
    control_plain_pause,
    "x:=(InputString[\"w\"];5)",
    "{f[x]=3,DownValues[f]}",
    &["w"],
    "List[3, List[RuleDelayed[HoldPattern[f[5]], 3]]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[5]], 3]]"
);

case!(
    control_owned_noncycle,
    "x:=(InputString[\"w\"];Throw[3])",
    "Catch[f[x]=4]",
    &["w"],
    "3",
    "DownValues[f]",
    "List[]"
);

case!(
    control_owned_non_symbol_cycle,
    "x := (InputString[\"w\"]; x)",
    "f[g[x]]=3",
    &["w"],
    "TerminatedEvaluation[\"IterationLimit\"]",
    "DownValues[f]",
    "List[]"
);

case!(
    control_owned_other_symbol_cycle,
    "x:=y;y:=(InputString[\"w\"];y)",
    "f[x]=3",
    &["w"],
    "TerminatedEvaluation[\"IterationLimit\"]",
    "DownValues[f]",
    "List[]"
);

case!(
    control_ordinary_cycle_after_pause,
    "z:=(InputString[\"u\"];2);cL:=cL",
    "{f[z,cL]=3,DownValues[f]}",
    &["u"],
    "List[3, List[RuleDelayed[HoldPattern[f[2, cL]], 3]]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[2, cL]], 3]]"
);

case!(
    control_head_throw,
    "x:=(InputString[\"w\"];Throw[3])",
    "Catch[x[1]=4]",
    &["w"],
    "3",
    "DownValues[x]",
    "List[]"
);

case!(
    control_pattern_throw,
    "x:=(InputString[\"w\"];Throw[3])",
    "Catch[f[Blank[x]]=4]",
    &["w"],
    "3",
    "DownValues[f]",
    "List[]"
);

case!(
    control_head_other_symbol,
    "x:=y;y := (InputString[\"v\"]; y)",
    "x[1]=3",
    &["v"],
    "TerminatedEvaluation[\"IterationLimit\"]",
    "DownValues[x]",
    "List[]"
);

case!(
    control_pattern_other_symbol,
    "x:=y;y := (InputString[\"v\"]; y)",
    "f[Blank[x]]=3",
    &["v"],
    "TerminatedEvaluation[\"IterationLimit\"]",
    "DownValues[f]",
    "List[]"
);

case!(
    argument_three_inner_pauses,
    "x:=(InputString[\"w1\"];InputString[\"w2\"];InputString[\"w3\"];x)",
    "f[x]=3",
    &["w1", "w2", "w3"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[x]], 3]]"
);

case!(
    head_three_inner_pauses,
    "x:=(InputString[\"w1\"];InputString[\"w2\"];InputString[\"w3\"];x)",
    "x[1]=3",
    &["w1", "w2", "w3"],
    "3",
    "{DownValues[x], SubValues[x]}",
    "List[List[RuleDelayed[HoldPattern[x[1]], 3]], List[]]"
);

case!(
    pattern_three_inner_pauses,
    "x:=(InputString[\"w1\"];InputString[\"w2\"];InputString[\"w3\"];x)",
    "f[Blank[x]]=3",
    &["w1", "w2", "w3"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[Blank[x]]], 3]]"
);

case!(
    argument_after_three_prior,
    "z1:=(InputString[\"u1\"];2);z2:=(InputString[\"u2\"];3);z3:=(InputString[\"u3\"];4);x := (InputString[\"w\"]; x)",
    "f[z1,z2,z3,x]=3",
    &["u1", "u2", "u3", "w"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[2, 3, 4, x]], 3]]"
);

case!(
    pattern_after_three_prior,
    "z1:=(InputString[\"u1\"];2);z2:=(InputString[\"u2\"];3);z3:=(InputString[\"u3\"];4);x := (InputString[\"w\"]; x)",
    "f[Blank[z1],Blank[z2],Blank[z3],Blank[x]]=3",
    &["u1", "u2", "u3", "w"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[Blank[2], Blank[3], Blank[4], Blank[x]]], 3]]"
);

case!(
    head_after_three_prior,
    "z1:=(InputString[\"u1\"];2);z2:=(InputString[\"u2\"];3);z3:=(InputString[\"u3\"];4);x := (InputString[\"w\"]; x)",
    "f[Alternatives[h[z1],h[z2],h[z3],x[1]]]=3",
    &["u1", "u2", "u3", "w"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[Alternatives[h[2], h[3], h[4], x[1]]]], 3]]"
);

case!(
    argument_recovery_then_pause,
    "x := (InputString[\"w\"]; x);z:=(InputString[\"u\"];2)",
    "f[x,z]=3",
    &["w", "u"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[x, 2]], 3]]"
);

case!(
    pattern_recovery_then_pause,
    "x := (InputString[\"w\"]; x);z:=(InputString[\"u\"];2)",
    "f[Blank[x],Blank[z]]=3",
    &["w", "u"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[Blank[x], Blank[2]]], 3]]"
);

case!(
    head_recovery_then_pause,
    "x := (InputString[\"w\"]; x);z:=(InputString[\"u\"];2)",
    "f[Alternatives[x[1],h[z]]]=3",
    &["w", "u"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[Alternatives[x[1], h[2]]]], 3]]"
);

case!(
    pattern_two_cycles,
    "x := (InputString[\"w\"]; x);y := (InputString[\"v\"]; y)",
    "f[Blank[x],Blank[y]]=3",
    &["w", "v"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[Blank[x], Blank[y]]], 3]]"
);

case!(
    head_two_cycles,
    "x := (InputString[\"w\"]; x);y := (InputString[\"v\"]; y)",
    "f[Alternatives[x[1],y[2]]]=3",
    &["w", "v"],
    "3",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[Alternatives[x[1], y[2]]]], 3]]"
);

case!(
    table_down,
    "x := (InputString[\"w\"]; x)",
    "DownValues[f]={HoldPattern[f[x]]:>3}",
    &["w"],
    "List[RuleDelayed[HoldPattern[f[x]], 3]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[x]], 3]]"
);

case!(
    table_up,
    "x := (InputString[\"w\"]; x)",
    "UpValues[x]={HoldPattern[g[x]]:>3}",
    &["w"],
    "List[RuleDelayed[HoldPattern[g[x]], 3]]",
    "UpValues[x]",
    "List[RuleDelayed[HoldPattern[g[x]], 3]]"
);

case!(
    table_sub,
    "x := (InputString[\"w\"]; x)",
    "SubValues[f]={HoldPattern[f[1][x]]:>3}",
    &["w"],
    "List[RuleDelayed[HoldPattern[f[1][x]], 3]]",
    "SubValues[f]",
    "List[RuleDelayed[HoldPattern[f[1][x]], 3]]"
);

case!(
    table_head,
    "x := (InputString[\"w\"]; x)",
    "DownValues[x]={HoldPattern[x[1]]:>3}",
    &["w"],
    "List[RuleDelayed[HoldPattern[x[1]], 3]]",
    "DownValues[x]",
    "List[RuleDelayed[HoldPattern[x[1]], 3]]"
);

case!(
    table_next_pattern,
    "z:=(InputString[\"u\"];2);x := (InputString[\"w\"]; x)",
    "DownValues[f]={HoldPattern[f[z]]:>2,HoldPattern[f[Blank[x]]]:>3}",
    &["u", "w"],
    "List[RuleDelayed[HoldPattern[f[z]], 2], RuleDelayed[HoldPattern[f[Blank[x]]], 3]]",
    "DownValues[f]",
    "List[RuleDelayed[HoldPattern[f[2]], 2], RuleDelayed[HoldPattern[f[Blank[x]]], 3]]"
);

#[test]
fn lifecycle_at_each_awaiting_state() {
    for source in ["f[x] = 3", "x[1] = 3", "f[Blank[x]] = 3"] {
        let mut fixture = super::make(
            "G15B.lifecycle",
            source,
            &[("InputString", "first", "1"), ("InputString", "later", "1")],
            None,
            "List[]",
        );
        fixture.setup = "log={};x:=(InputString[\"first\"];InputString[\"later\"];x)".into();
        fixture.pause_prints = Some(vec![vec![]; 2]);
        fixture.pause_messages = Some(vec![vec![]; 2]);
        fixture.pause_trace = Some(vec![vec![]; 2]);
        assert_eq!(
            super::super::descriptors::exercise_fixture(&fixture, |_, _, _| {}),
            12
        );
    }
    super::super::support::faults::assert_all();
}
