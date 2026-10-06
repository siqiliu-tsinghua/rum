use super::super::descriptors::Fixture;
use super::make;
pub fn fixtures() -> Vec<Fixture> {
    let mut rows = Vec::new();
    rows.push(make(
        r####"brief-548.Rule-plain"####,
        r####"Rule[(AppendTo[Global`log,1];InputString["p: "]),(AppendTo[Global`log,3];9)]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"Rule["two", 9]"####),
        r####"List[1, 3]"####,
    ));
    rows.push(make(r####"brief-548.Rule-pattern"####, r####"Rule[h[x_,(AppendTo[Global`log,2];InputString["p: "]),(AppendTo[Global`log,3];3)],9]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"Rule[h[Pattern[x, Blank[]], "two", 3], 9]"####), r####"List[2, 3]"####));
    rows.push(make(r####"brief-548.RuleDelayed-plain"####, r####"RuleDelayed[(AppendTo[Global`log,1];InputString["p: "]),(AppendTo[Global`log,3];9)]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"RuleDelayed["two", CompoundExpression[AppendTo[log, 3], 9]]"####), r####"List[1]"####));
    rows.push(make(r####"brief-548.RuleDelayed-pattern"####, r####"RuleDelayed[h[x_,(AppendTo[Global`log,2];InputString["p: "]),(AppendTo[Global`log,3];3)],9]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"RuleDelayed[h[Pattern[x, Blank[]], "two", 3], 9]"####), r####"List[2, 3]"####));
    rows.push(make(
        r####"brief-548.Rule-Blank"####,
        r####"Rule[Pattern[x,Blank[(AppendTo[Global`log,2];Input["p: "])]],9]"####,
        &[(r####"Input"####, r####"p: "####, r####"Integer"####)],
        Some(r####"Rule[Pattern[x, Blank[Integer]], 9]"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-548.Rule-PatternTest"####, r####"Rule[PatternTest[Pattern[x,Blank[(AppendTo[Global`log,2];Input["p: "])]],IntegerQ],9]"####, &[(r####"Input"####, r####"p: "####, r####"Integer"####)], Some(r####"Rule[PatternTest[Pattern[x, Blank[Integer]], IntegerQ], 9]"####), r####"List[2]"####));
    rows.push(make(
        r####"brief-548.Rule-Condition"####,
        r####"Rule[Condition[Pattern[x,Blank[(AppendTo[Global`log,2];Input["p: "])]],True],9]"####,
        &[(r####"Input"####, r####"p: "####, r####"Integer"####)],
        Some(r####"Rule[Condition[Pattern[x, Blank[Integer]], True], 9]"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.Rule-wrapper"####,
        r####"Rule[Alternatives[x_,Blank[(AppendTo[Global`log,2];Input["p: "])]],9]"####,
        &[(r####"Input"####, r####"p: "####, r####"Integer"####)],
        Some(r####"Rule[Alternatives[Pattern[x, Blank[]], Blank[Integer]], 9]"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.Rule-head"####,
        r####"Rule[(AppendTo[Global`log,2];Input["p: "])[x_],9]"####,
        &[(r####"Input"####, r####"p: "####, r####"h"####)],
        Some(r####"Rule[h[Pattern[x, Blank[]]], 9]"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-548.Rule-twice"####, r####"Rule[h[x_,(AppendTo[Global`log,2];InputString["p: "]),(AppendTo[Global`log,4];InputString["p: "])],9]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####), (r####"InputString"####, r####"p: "####, r####"four"####)], Some(r####"Rule[h[Pattern[x, Blank[]], "two", "four"], 9]"####), r####"List[2, 4]"####));
    rows
}
