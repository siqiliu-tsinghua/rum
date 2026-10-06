use super::super::descriptors::Fixture;
use super::make;
pub fn fixtures() -> Vec<Fixture> {
    let mut rows = Vec::new();
    rows.push(make(r####"brief-548.Scan"####, r####"Scan[Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], InputString["p: "], x])],{1,2,3}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"Null"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.Scan-twice"####, r####"Scan[Function[x,(AppendTo[Global`log,x];If[Or[SameQ[x,2],SameQ[x,4]],InputString["p: "],x])],{1,2,3,4,5}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####), (r####"InputString"####, r####"p: "####, r####"four"####)], Some(r####"Null"####), r####"List[1, 2, 3, 4, 5]"####));
    rows.push(make(
        r####"brief-548.Differences"####,
        r####"a /: Times[-1,a] := (AppendTo[Global`log,2];Input["p: "]);Differences[{1,a,3,4}]"####,
        &[(r####"Input"####, r####"p: "####, r####"-2"####)],
        Some(r####"List[Plus[-1, a], 1, 1]"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-548.Dot-chain"####, r####"a /: Times[a,x_] := (AppendTo[Global`log,x];If[SameQ[Length[Global`log],2],InputString["p: "],Null];x);Dot[{{a,a},{a,a}},{{1,2},{3,4}},{{3,0},{0,3}}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[List[12, 18], List[12, 18]]"####), r####"List[1, 3, 2, 4, 1, 3, 2, 4]"####));
    rows.push(make(r####"brief-548.Do"####, r####"Do[Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], InputString["p: "], x])][x],{x,{1,2,3}}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"Null"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.Select"####, r####"Select[{1,2,3},Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, 2, 3]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.NestList"####, r####"NestList[Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], InputString["p: "], Null]; x+1)],1,3]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, 2, 3, 4]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.Table"####, r####"Table[Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], InputString["p: "], x])][i],{i,3}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, "two", 3]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.Map"####, r####"Map[Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], InputString["p: "], x])],{1,2,3}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, "two", 3]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.MapIndexed"####, r####"MapIndexed[Function[{x,i},(AppendTo[Global`log,x];If[SameQ[x,2],InputString["p: "],x])],{1,2,3}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, "two", 3]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.Fold"####, r####"Fold[Function[{a,x},(AppendTo[Global`log,x];If[SameQ[x,2],InputString["p: "],Null];a+x)],0,{1,2,3}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"6"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.FoldList"####, r####"FoldList[Function[{a,x},(AppendTo[Global`log,x];If[SameQ[x,2],InputString["p: "],Null];a+x)],0,{1,2,3}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[0, 1, 3, 6]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.AllTrue"####, r####"AllTrue[{1,2,3},Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.AnyTrue"####, r####"AnyTrue[{1,2,3},Function[x,(AppendTo[Global`log,x];If[SameQ[x,2],InputString["p: "],Null];False)]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"False"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.NoneTrue"####, r####"NoneTrue[{1,2,3},Function[x,(AppendTo[Global`log,x];If[SameQ[x,2],InputString["p: "],Null];False)]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.VectorQ"####, r####"VectorQ[{1,2,3},Function[x, (AppendTo[Global`log,x]; If[SameQ[x,2], SameQ[InputString["p: "],"two"], True])]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.SelectFirst"####, r####"SelectFirst[{1,2,3},Function[x,(AppendTo[Global`log,x];If[SameQ[x,2],SameQ[InputString["p: "],"two"],False])]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"2"####), r####"List[1, 2]"####));
    rows.push(make(r####"brief-548.SubsetMap"####, r####"SubsetMap[Function[x,(AppendTo[Global`log,x];InputString["p: "];Reverse[x])],{1,2,3},{{1},{3}}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[3, 2, 1]"####), r####"List[List[1, 3]]"####));
    rows.push(make(
        r####"brief-548.Input-nested"####,
        r####"AppendTo[Global`log,1];Input["p: "]"####,
        &[
            (
                r####"Input"####,
                r####"p: "####,
                r####"(AppendTo[log,2];InputString["q: "])"####,
            ),
            (r####"InputString"####, r####"q: "####, r####"two"####),
        ],
        Some(r####""two""####),
        r####"List[1, 2]"####,
    ));
    rows.push(make(r####"brief-548.Inner"####, r####"Inner[Function[{x,y},(AppendTo[Global`log,x];If[SameQ[x,2],InputString["p: "],Null];x*y)],{1,2,3},{1,1,1},Plus]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"6"####), r####"List[1, 2, 3]"####));
    rows.push(make(
        r####"brief-548.Mean-tail"####,
        r####"a /: Plus[a,b] := (AppendTo[Global`log,2];Input["p: "]);Mean[{a,b}]"####,
        &[(r####"Input"####, r####"p: "####, r####"6"####)],
        Some(r####"3"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.Median-tail"####,
        r####"a /: Plus[a,b] := (AppendTo[Global`log,2];Input["p: "]);Median[{a,b}]"####,
        &[(r####"Input"####, r####"p: "####, r####"6"####)],
        Some(r####"3"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-548.Assert-function"####, r####"$AssertFunction=Function[x,(AppendTo[Global`log,2];InputString["p: "];7)];Assert[False]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"Null"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Catch-handler"####, r####"Catch[Throw[7,t],t,Function[{x,y},(AppendTo[Global`log,2];InputString["p: "];x+1)]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"8"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Dialog"####, r####"Dialog[(AppendTo[Global`log,1];InputString["p: "];AppendTo[Global`log,3];Return[7])]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"7"####), r####"List[1, 3]"####));
    rows.push(make(r####"brief-548.ReleaseHold"####, r####"ReleaseHold[Hold[(AppendTo[Global`log,1];InputString["p: "];AppendTo[Global`log,3];77)]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"77"####), r####"List[1, 3]"####));
    rows.push(make(
        r####"brief-548.TimeConstrained-fallback"####,
        r####"TimeConstrained[While[True],0,(AppendTo[Global`log,2];InputString["p: "];77)]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"77"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.MemoryConstrained-fallback"####,
        r####"MemoryConstrained[Range[1000],0,(AppendTo[Global`log,2];InputString["p: "];77)]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"77"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-548.Differences-twice"####, r####"a /:Times[-1,a]:=(AppendTo[Global`log,2];Input["p: "]);b /:Times[-1,b]:=(AppendTo[Global`log,4];Input["p: "]);Differences[{1,a,3,b,5}]"####, &[(r####"Input"####, r####"p: "####, r####"-2"####), (r####"Input"####, r####"p: "####, r####"-4"####)], Some(r####"List[Plus[-1, a], 1, Plus[-3, b], 1]"####), r####"List[2, 4]"####));
    rows.push(make(r####"brief-548.Dot-twice"####, r####"a /: Times[a,x_] := (AppendTo[Global`log,x];If[Or[SameQ[Length[Global`log],2],SameQ[Length[Global`log],4]],InputString["p: "],Null];x);Dot[{{a,a},{a,a}},{{1,2},{3,4}},{{3,0},{0,3}}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####), (r####"InputString"####, r####"p: "####, r####"four"####)], Some(r####"List[List[12, 18], List[12, 18]]"####), r####"List[1, 3, 2, 4, 1, 3, 2, 4]"####));
    rows.push(make(
        r####"brief-548.Differences-order-two"####,
        r####"a /:Times[-1,a]:=(AppendTo[Global`log,2];Input["p: "]);Differences[{1,a,3,4},2]"####,
        &[(r####"Input"####, r####"p: "####, r####"-2"####)],
        Some(r####"List[Plus[1, Times[-1, Plus[-1, a]]], 0]"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-583.failure-properties"####, r####"Failure["tag",<|"MessageTemplate"->"value","MessageParameters":>(AppendTo[Global`log,1];InputString["p: "];{7})|>]["Message"]"####, &[(r####"InputString"####, r####"p: "####, r####"x"####)], Some(r####""value""####), r####"List[1]"####));
    rows.push(make(r####"brief-583.failure-styled"####, r####"Failure["tag",<|"MessageTemplate"->"value","MessageParameters":>(AppendTo[Global`log,1];InputString["p: "];{7})|>]["StyledMessage"]"####, &[(r####"InputString"####, r####"p: "####, r####"x"####)], Some(r####"Style["value", "Message"]"####), r####"List[1]"####));
    rows.push(make(r####"brief-583.replaceall-evaluate"####, r####"h[x_]:=(AppendTo[Global`log,x];x+10);ReplaceAll[{1,2,3},z_Integer/;(If[z==2,InputString["p: "]];True):>h[z]]"####, &[(r####"InputString"####, r####"p: "####, r####"x"####)], Some(r####"List[11, 12, 13]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-583.string-cases-twice"####, r####"StringCases["abc",x:LetterCharacter:>(AppendTo[Global`log,x];If[Or[x=="b",x=="c"],InputString["p: "]];x)]"####, &[(r####"InputString"####, r####"p: "####, r####"x"####), (r####"InputString"####, r####"p: "####, r####"y"####)], Some(r####"List["a", "b", "c"]"####), r####"List["a", "b", "c"]"####));
    rows.push(make(r####"brief-583.string-replace-twice"####, r####"StringReplace["abc",x:LetterCharacter:>(AppendTo[Global`log,x];If[Or[x=="b",x=="c"],InputString["p: "]];x)]"####, &[(r####"InputString"####, r####"p: "####, r####"x"####), (r####"InputString"####, r####"p: "####, r####"y"####)], Some(r####""abc""####), r####"List["a", "b", "c"]"####));
    rows.push(make(r####"brief-583.tally-twice"####, r####"Tally[{1,2,3},Function[{x,y},(AppendTo[Global`log,{x,y}];If[Length[Global`log]<=2,InputString["p: "]];False)]]"####, &[(r####"InputString"####, r####"p: "####, r####"x"####), (r####"InputString"####, r####"p: "####, r####"y"####)], Some(r####"List[List[1, 1], List[2, 1], List[3, 1]]"####), r####"List[List[2, 1], List[3, 1], List[3, 2]]"####));
    rows.push(make(r####"brief-583.union-twice"####, r####"Union[{1,2,3},SameTest->Function[{x,y},(AppendTo[Global`log,{x,y}];If[Length[Global`log]<=2,InputString["p: "]];False)]]"####, &[(r####"InputString"####, r####"p: "####, r####"x"####), (r####"InputString"####, r####"p: "####, r####"y"####)], Some(r####"List[1, 2, 3]"####), r####"List[List[2, 1], List[3, 1], List[3, 2]]"####));
    rows
}
