use super::support::compare;

#[test]
fn rule_plain() {
    compare(
        r###"Rule-plain"###,
        r###"Rule[(AppendTo[log,1];InputString["p: "]),(AppendTo[log,3];9)]"###,
        &[r###"two"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn rule_pattern() {
    compare(
        r###"Rule-pattern"###,
        r###"Rule[h[x_,(AppendTo[log,2];InputString["p: "]),(AppendTo[log,3];3)],9]"###,
        &[r###"two"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn ruledelayed_plain() {
    compare(
        r###"RuleDelayed-plain"###,
        r###"RuleDelayed[(AppendTo[log,1];InputString["p: "]),(AppendTo[log,3];9)]"###,
        &[r###"two"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn ruledelayed_pattern() {
    compare(
        r###"RuleDelayed-pattern"###,
        r###"RuleDelayed[h[x_,(AppendTo[log,2];InputString["p: "]),(AppendTo[log,3];3)],9]"###,
        &[r###"two"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn rule_blank() {
    compare(
        r###"Rule-Blank"###,
        r###"Rule[Pattern[x,Blank[(AppendTo[log,2];Input["p: "])]],9]"###,
        &[r###"Integer"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn rule_patterntest() {
    compare(
        r###"Rule-PatternTest"###,
        r###"Rule[PatternTest[Pattern[x,Blank[(AppendTo[log,2];Input["p: "])]],IntegerQ],9]"###,
        &[r###"Integer"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn rule_condition() {
    compare(
        r###"Rule-Condition"###,
        r###"Rule[Condition[Pattern[x,Blank[(AppendTo[log,2];Input["p: "])]],True],9]"###,
        &[r###"Integer"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn rule_wrapper() {
    compare(
        r###"Rule-wrapper"###,
        r###"Rule[Alternatives[x_,Blank[(AppendTo[log,2];Input["p: "])]],9]"###,
        &[r###"Integer"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn rule_head() {
    compare(
        r###"Rule-head"###,
        r###"Rule[(AppendTo[log,2];Input["p: "])[x_],9]"###,
        &[r###"h"###],
        &[r###"p: "###],
        r###""###,
    );
}

#[test]
fn rule_twice() {
    compare(
        r###"Rule-twice"###,
        r###"Rule[h[x_,(AppendTo[log,2];InputString["p: "]),(AppendTo[log,4];InputString["p: "])],9]"###,
        &[r###"two"###, r###"four"###],
        &[r###"p: "###, r###"p: "###],
        r###""###,
    );
}

#[test]
fn replacepart_pattern() {
    compare(
        r###"ReplacePart-pattern"###,
        r###"ReplacePart[{{1,2},{3,4}}, {_,(AppendTo[log,2];Input["first: "])}:>9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn normalizerules_rule_pattern() {
    compare(
        r###"NormalizeRules-rule-pattern"###,
        r###"NormalizeRules[h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]->9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn replaceall_rule_pattern() {
    compare(
        r###"ReplaceAll-rule-pattern"###,
        r###"ReplaceAll[h[1,2,3],h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]->9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn replacerepeated_rule_pattern() {
    compare(
        r###"ReplaceRepeated-rule-pattern"###,
        r###"ReplaceRepeated[h[1,2,3],h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]->9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn replace_rule_pattern() {
    compare(
        r###"Replace-rule-pattern"###,
        r###"Replace[h[1,2,3],h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]->9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn replacelist_rule_pattern() {
    compare(
        r###"ReplaceList-rule-pattern"###,
        r###"ReplaceList[h[1,2,3],h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]->9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn replaceat_rule_pattern() {
    compare(
        r###"ReplaceAt-rule-pattern"###,
        r###"ReplaceAt[{h[1,2,3]},h[_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,3];3)]->9,{1}]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn rule_distinct_twice() {
    compare(
        r###"Rule-distinct-twice"###,
        r###"Rule[h[x_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,4];Input["second: "]),(AppendTo[log,5];5)],9]"###,
        &[r###"2"###, r###"4"###],
        &[r###"first: "###, r###"second: "###],
        r###""###,
    );
}

#[test]
fn ruledelayed_distinct_twice() {
    compare(
        r###"RuleDelayed-distinct-twice"###,
        r###"RuleDelayed[h[x_,(AppendTo[log,2];Input["first: "]),(AppendTo[log,4];Input["second: "]),(AppendTo[log,5];5)],(AppendTo[log,99];9)]"###,
        &[r###"2"###, r###"4"###],
        &[r###"first: "###, r###"second: "###],
        r###""###,
    );
}

#[test]
fn rule_nested_pattern() {
    compare(
        r###"Rule-nested-pattern"###,
        r###"Rule[h[g[x_,(AppendTo[log,2];Input["first: "])],y_,(AppendTo[log,3];3)],9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn rule_lhs_then_rhs() {
    compare(
        r###"Rule-lhs-then-rhs"###,
        r###"Rule[h[x_,(AppendTo[log,2];Input["first: "])],(AppendTo[log,4];Input["second: "])]"###,
        &[r###"2"###, r###"4"###],
        &[r###"first: "###, r###"second: "###],
        r###""###,
    );
}

#[test]
fn rule_one_child_two_inputs() {
    compare(
        r###"Rule-one-child-two-inputs"###,
        r###"Rule[h[x_,(AppendTo[log,1];Input["first: "];AppendTo[log,2];Input["second: "];AppendTo[log,3];3),(AppendTo[log,4];4)],9]"###,
        &[r###"1"###, r###"2"###],
        &[r###"first: "###, r###"second: "###],
        r###""###,
    );
}

#[test]
fn rule_error_after_resume() {
    compare(
        r###"Rule-error-after-resume"###,
        r###"Rule[h[x_,(AppendTo[log,1];Print["prefix"];Message[probe::tag];Input["first: "]),(AppendTo[log,2];Print["suffix"];Message[probe::tag];Throw["g4 boom"]),(AppendTo[log,3];3)],9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn rule_condition_held() {
    compare(
        r###"Rule-Condition-held"###,
        r###"Rule[Condition[h[x_,(AppendTo[log,2];Input["first: "])],(AppendTo[log,99];InputString["held: "];True)],9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn rule_patterntest_held() {
    compare(
        r###"Rule-PatternTest-held"###,
        r###"Rule[PatternTest[h[x_,(AppendTo[log,2];Input["first: "])],(AppendTo[log,99];InputString["held: "];True)],9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn rule_holdpattern_held() {
    compare(
        r###"Rule-HoldPattern-held"###,
        r###"Rule[h[HoldPattern[g[InputString["held: "]]],x_,(AppendTo[log,2];Input["first: "])],9]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}

#[test]
fn ruledelayed_rhs_held() {
    compare(
        r###"RuleDelayed-rhs-held"###,
        r###"RuleDelayed[h[x_,(AppendTo[log,2];Input["first: "])],(AppendTo[log,99];InputString["rhs-held: "])]"###,
        &[r###"2"###],
        &[r###"first: "###],
        r###""###,
    );
}
