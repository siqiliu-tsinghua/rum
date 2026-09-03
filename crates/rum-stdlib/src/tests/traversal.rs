use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn level_supports_standard_levelspecs_and_depth_first_ordering() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2, h[3]], 1]")
            .unwrap()
            .value
            .to_string(),
        "List[g[1], 2, h[3]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2, h[3]], 2]")
            .unwrap()
            .value
            .to_string(),
        "List[1, g[1], 2, 3, h[3]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2, h[3]], {2}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 3]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2, h[3]], -1]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[], {-1}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[]], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "List[g[]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[]], {-3}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[g[]]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[a], 0]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[a], {0}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a]]"
    );
    assert_eq!(
        session
            .eval_text("Level[{}, 0, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Level[{}, {-1}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[List]"
    );
    assert_eq!(
        session
            .eval_text("Level[{}, {0}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[1, g[2]], All]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, g[2], f[1, g[2]]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2], {2, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2], {-2, Infinity}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, g[1], 2]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2], {-3, 0}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[g[1], 2]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2], {-Infinity, -1}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, g[1], 2, f[g[1], 2]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2], {-Infinity, 0}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[g[1], 2]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2], {Infinity}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1]], {-2, Infinity}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f, g, 1, g[1]]"
    );
    assert_eq!(
        session
            .eval_text("Level[Association[], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[]]"
    );
    assert_eq!(
        session
            .eval_text("Level[Association[a -> 1], {1}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[Association, 1]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[1], {1}, {\"Heads\" -> True}]")
            .unwrap()
            .value
            .to_string(),
        "List[f, 1]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[1], {1}, {{\"Heads\" -> True}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f, 1]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[1], {1}, {}]")
            .unwrap()
            .value
            .to_string(),
        "List[1]"
    );
    let apply_head = session.eval_text("Level[f[a], {1}, h]").unwrap();
    assert_eq!(apply_head.value.to_string(), "h[a]");
    assert!(apply_head.messages.is_empty());
    let unknown_option = session.eval_text("Level[f[a], {1}, Bad -> 1]").unwrap();
    assert_eq!(
        unknown_option.value.to_string(),
        "Level[f[a], List[1], Rule[Bad, 1]]"
    );
    assert!(
        unknown_option
            .messages
            .iter()
            .any(|message| message.symbol == "Level" && message.tag == "optx")
    );
    let invalid_heads_value = session.eval_text("Level[f[a], {1}, Heads -> x]").unwrap();
    assert_eq!(
        invalid_heads_value.value.to_string(),
        "Level[f[a], List[1], Rule[Heads, x]]"
    );
    assert!(
        invalid_heads_value
            .messages
            .iter()
            .any(|message| message.symbol == "Level" && message.tag == "opttf")
    );
    let non_option = session.eval_text("Level[f[a], {1}, h, {1}]").unwrap();
    assert_eq!(
        non_option.value.to_string(),
        "Level[f[a], List[1], h, List[1]]"
    );
    assert!(
        non_option
            .messages
            .iter()
            .any(|message| message.symbol == "Level" && message.tag == "nonopt")
    );
}

#[test]
fn count_memberq_and_freeq_share_standard_traversal_semantics() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Count[f[g[1], 2, h[3]], _Integer]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Count[f[g[1], 2, h[3]], _Integer, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Count[f[g[1], 2], _, {2, 1}]")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
    assert_eq!(
        session
            .eval_text("Count[f[1, g[2]], x_Symbol, Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Count[f[1], _Symbol, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Count[f[1], _Symbol, Infinity, {\"Heads\" -> True}]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[f[1], _Symbol, Infinity, {\"Heads\" -> True}]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Cases[f[1], _Symbol, {0, Infinity}, {}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Count[f[g][a], _Symbol, {-1}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Count[f[g][a], _, {-2}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Count[f[g[]], g[], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Count[f[g[]], f[g[]], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
    assert_eq!(
        session
            .eval_text("MemberQ[f[g[1]], _Integer]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MemberQ[f[g[1]], _Integer, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MemberQ[f[g[1], 2], _, {2, 1}]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MemberQ[f[g[]], f[g[]], {-3}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MemberQ[f[1], f, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("spec = Association[\"kind\" -> \"infix\"]; MemberQ[{\"infix\", \"prefix\", \"binder\"}, spec[\"kind\"]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("x = String; MatchQ[7, x_]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[f[g[1]], _Integer]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[f[g[1]], _Integer, {1}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[f[g[]], g[], {-1}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[f[1], f]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[f[1], f, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Count[f[1], _, All]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("MemberQ[f[1], f[1], All]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[f[1], f[1], All]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    let count_unknown_option = session
        .eval_text("Count[f[a], _, Infinity, Bad -> 1]")
        .unwrap();
    assert_eq!(
        count_unknown_option.value.to_string(),
        "Count[f[a], Blank[], Infinity, Rule[Bad, 1]]"
    );
    assert!(
        count_unknown_option
            .messages
            .iter()
            .any(|message| message.symbol == "Count" && message.tag == "optx")
    );
    let memberq_invalid_heads_value = session
        .eval_text("MemberQ[f[a], _, Infinity, Heads -> x]")
        .unwrap();
    assert_eq!(
        memberq_invalid_heads_value.value.to_string(),
        "MemberQ[f[a], Blank[], Infinity, Rule[Heads, x]]"
    );
    assert!(
        memberq_invalid_heads_value
            .messages
            .iter()
            .any(|message| message.symbol == "MemberQ" && message.tag == "opttf")
    );
    let freeq_non_option = session
        .eval_text("FreeQ[f[a], _, Infinity, {Heads -> True, 1}]")
        .unwrap();
    assert_eq!(
        freeq_non_option.value.to_string(),
        "FreeQ[f[a], Blank[], Infinity, List[Rule[Heads, True], 1]]"
    );
    assert!(
        freeq_non_option
            .messages
            .iter()
            .any(|message| message.symbol == "FreeQ" && message.tag == "nonopt")
    );
}

#[test]
fn count_like_condition_guards_preserve_short_circuit_points() {
    let mut session = core_session();

    let member_q = session
        .eval_text(
            "seen = 0; {MemberQ[f[1, 2, 3], x_Integer /; \
             (seen = seen + 1; x == 2), Infinity], seen}",
        )
        .unwrap();
    assert_eq!(member_q.value.to_string(), "List[True, 2]");

    let free_q = session
        .eval_text(
            "seen = 0; {FreeQ[f[1, 2, 3], x_Integer /; \
             (seen = seen + 1; x == 2)], seen}",
        )
        .unwrap();
    assert_eq!(free_q.value.to_string(), "List[False, 2]");

    let count = session
        .eval_text(
            "seen = 0; {Count[f[1, 2, 3], x_Integer /; \
             (seen = seen + 1; x == 2), Infinity], seen}",
        )
        .unwrap();
    assert_eq!(count.value.to_string(), "List[1, 3]");
}

#[test]
fn level_supports_heads_and_apply_head_forms() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Level[x, All, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[x]"
    );
    assert_eq!(
        session
            .eval_text("Level[{}, All, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[List, List[]]"
    );
    assert_eq!(
        session
            .eval_text("Level[{}, {-1}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[List]"
    );
    assert_eq!(
        session
            .eval_text("Level[{}, {-2}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[List[]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[1, g[2]], 1, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f, 1, g[2]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1]], {2}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[g, 1]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1]], {-1}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f, g, 1]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g][a], {-1}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f, g, a]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g][a], {-2}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f[g], f[g][a]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g][a], {-3}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[h]][a], {-3}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f[g[h]]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[1, 2], {1}, Plus]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

#[test]
fn position_supports_standard_levelspecs_heads_and_limits() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Position[f[g[1], 2, h[3]], _Integer]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 1], List[2], List[3, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[g[1], 2], _, {2, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[1], f[x_Integer], 0, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "List[List[]]"
    );
    assert_eq!(
        session
            .eval_text("Position[x, _Symbol, All, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[List[]]"
    );
    assert_eq!(
        session
            .eval_text("Position[x, _Symbol, All, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "List[List[]]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[g][a], f[g][a], {-2}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[List[]]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[g][a], f[g][a], {-3}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[1, g[2]], x_Symbol]")
            .unwrap()
            .value
            .to_string(),
        "List[List[0], List[2, 0]]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[g[1], 2, h[3]], _Integer, Infinity, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 1], List[2]]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[1, g[2]], x_Symbol, Infinity, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[1], _Symbol, {1}, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[List[0]]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[1], _Symbol, Infinity, {}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[0]]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[1], f[1], All, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "List[List[]]"
    );
    let unknown_option = session
        .eval_text("Position[f[a], _, Infinity, Bad -> 1]")
        .unwrap();
    assert_eq!(
        unknown_option.value.to_string(),
        "Position[f[a], Blank[], Infinity, Rule[Bad, 1]]"
    );
    assert!(
        unknown_option
            .messages
            .iter()
            .any(|message| message.symbol == "Position" && message.tag == "optx")
    );
    let invalid_heads_value = session
        .eval_text("Position[f[a], _, Infinity, Heads -> x]")
        .unwrap();
    assert_eq!(
        invalid_heads_value.value.to_string(),
        "Position[f[a], Blank[], Infinity, Rule[Heads, x]]"
    );
    assert!(
        invalid_heads_value
            .messages
            .iter()
            .any(|message| message.symbol == "Position" && message.tag == "opttf")
    );
}

#[test]
fn traversal_uses_values_only_association_semantics_and_keyed_positions() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Level[Association[a -> 1, b -> f[2]], Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, f[2]]"
    );
    assert_eq!(
        session
            .eval_text("Level[Association[a -> 1, b -> f[2]], Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[Association, 1, f, 2, f[2]]"
    );
    assert_eq!(
        session
            .eval_text("Count[Association[a -> 1, b -> f[2]], _Symbol, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
    assert_eq!(
        session
            .eval_text("Count[Association[a -> 1, b -> f[2]], _Symbol, Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("MemberQ[Association[a -> 1, b -> f[2]], _Symbol, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[Association[a -> 1, b -> f[2]], _Symbol, Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Position[Association[a -> 1, b -> f[2]], _Integer, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[List[Key[a]], List[Key[b], 1]]"
    );
    assert_eq!(
        session
            .eval_text("Position[Association[a -> 1, b -> f[2]], _Symbol, All, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[List[0], List[Key[b], 0]]"
    );
}
