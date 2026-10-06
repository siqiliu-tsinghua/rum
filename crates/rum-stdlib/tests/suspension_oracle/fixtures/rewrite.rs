use super::super::descriptors::Fixture;
use super::make;
pub fn fixtures() -> Vec<Fixture> {
    let mut rows = Vec::new();
    rows.push(make(r####"brief-548.Cases"####, r####"Cases[{1,2,3},_?Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, 2, 3]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.DeleteCases"####, r####"DeleteCases[{1,2,3},_?Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.Position"####, r####"Position[{1,2,3},_?Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[List[0], List[1], List[2], List[3], List[]]"####), r####"List[List, 1, 2, 3, List[1, 2, 3]]"####));
    rows.push(make(r####"brief-548.Count"####, r####"Count[{1,2,3},_?Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"3"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.ReplaceAll"####, r####"{1,2,3}/.z_Integer:>Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], InputString["p: "], x])][z]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, "two", 3]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.Replace"####, r####"Replace[{1,2,3},z_Integer:>Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], InputString["p: "], x])][z],{1}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, "two", 3]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.ReplaceAllUnheld"####, r####"ReplaceAllUnheld[{1,2,3},z_Integer:>(AppendTo[Global`log,z];If[SameQ[z,2],InputString["p: "],z])]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, "two", 3]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.ReplaceRepeated"####, r####"ReplaceRepeated[{1,2,3},z_Integer:>(AppendTo[Global`log,z];If[SameQ[z,2],InputString["p: "],Null];h[z]),MaxIterations->1]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"TerminatedEvaluation["IterationLimit"]"####), r####"List[1, 2, 3]"####));
    rows.push(make(
        r####"brief-548.Cases-target"####,
        r####"Cases[(AppendTo[Global`log,1];InputString["p: "];{1,2,3}),2]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[2]"####),
        r####"List[1]"####,
    ));
    rows.push(make(
        r####"brief-548.DeleteCases-target"####,
        r####"DeleteCases[(AppendTo[Global`log,1];InputString["p: "];{1,2,3}),2]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[1, 3]"####),
        r####"List[1]"####,
    ));
    rows.push(make(
        r####"brief-548.Position-target"####,
        r####"Position[(AppendTo[Global`log,1];InputString["p: "];{1,2,3}),2]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[List[2]]"####),
        r####"List[1]"####,
    ));
    rows.push(make(
        r####"brief-548.ReplaceRepeated-target"####,
        r####"ReplaceRepeated[(AppendTo[Global`log,1];InputString["p: "];{1,2,3}),2->9]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[1, 9, 3]"####),
        r####"List[1]"####,
    ));
    rows.push(make(
        r####"brief-548.ReplaceAt-target"####,
        r####"ReplaceAt[(AppendTo[Global`log,1];InputString["p: "];{1,2,3}),x_->9,2]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[1, 9, 3]"####),
        r####"List[1]"####,
    ));
    rows.push(make(
        r####"brief-548.Cases-spec"####,
        r####"Cases[{1,2,3},(AppendTo[Global`log,2];InputString["p: "];2)]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[2]"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.DeleteCases-spec"####,
        r####"DeleteCases[{1,2,3},(AppendTo[Global`log,2];InputString["p: "];2)]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[1, 3]"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.Position-spec"####,
        r####"Position[{1,2,3},(AppendTo[Global`log,2];InputString["p: "];2)]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[List[2]]"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.Replace-spec"####,
        r####"Replace[{1,2,3},(AppendTo[Global`log,2];InputString["p: "];2->9)]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[1, 2, 3]"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.ReplaceAt-spec"####,
        r####"ReplaceAt[{1,2,3},(AppendTo[Global`log,2];InputString["p: "];2->9),2]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[1, 9, 3]"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.ReplaceRepeated-chain"####,
        r####"ReplaceRepeated[1,{1:>(AppendTo[Global`log,2];Input["p: "]),2->3}]"####,
        &[(r####"Input"####, r####"p: "####, r####"2"####)],
        Some(r####"3"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-548.ReplaceRepeated-predicate"####, r####"ReplaceRepeated[{1,2,3},z_Integer /;(AppendTo[Global`log,z];If[SameQ[z,2],InputString["p: "],Null];True):>h[z],MaxIterations->1]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"TerminatedEvaluation["IterationLimit"]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.Cases-rule"####, r####"Cases[{1,2,3},z_Integer:>(AppendTo[Global`log,z];If[SameQ[z,2],InputString["p: "],z])]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, "two", 3]"####), r####"List[1, 2, 3]"####));
    rows.push(make(
        r####"brief-548.Replace-level"####,
        r####"Replace[{1,2,3},2->9,(AppendTo[Global`log,2];Input["p: "])]"####,
        &[(r####"Input"####, r####"p: "####, r####"{1}"####)],
        Some(r####"List[1, 9, 3]"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-548.ReplaceRepeated-twice"####, r####"ReplaceRepeated[{1,2,3,4,5},z_Integer /;(AppendTo[Global`log,z];If[Or[SameQ[z,2],SameQ[z,4]],InputString["p: "],Null];True):>h[z],MaxIterations->1]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####), (r####"InputString"####, r####"p: "####, r####"four"####)], Some(r####"TerminatedEvaluation["IterationLimit"]"####), r####"List[1, 2, 3, 4, 5]"####));
    rows
}
