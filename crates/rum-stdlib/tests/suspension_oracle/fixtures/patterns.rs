use super::super::descriptors::Fixture;
use super::make;
pub fn fixtures() -> Vec<Fixture> {
    let mut rows = Vec::new();
    rows.push(make(r####"brief-548.MatchQ-pattern-args"####, r####"MatchQ[h[1,2,3],h[_,(AppendTo[Global`log,2];Input["p: "]),(AppendTo[Global`log,3];3)]]"####, &[(r####"Input"####, r####"p: "####, r####"2"####)], Some(r####"True"####), r####"List[2, 3]"####));
    rows.push(make(
        r####"brief-548.MatchQ-pattern-head"####,
        r####"MatchQ[h[1],(AppendTo[Global`log,2];Input["p: "])[_]]"####,
        &[(r####"Input"####, r####"p: "####, r####"h"####)],
        Some(r####"True"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-548.Match-ordered"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);MatchQ[f[1,2,3],f[_?gate,_?gate,_?gate]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.Match-alternative"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);MatchQ[2,1|_?gate]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Match-except"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);MatchQ[2,Except[_?gate]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"False"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Match-except-inner"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);MatchQ[2,Except[1,_?gate]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Match-call-head"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);MatchQ[h[1,2],(_?Function[z,(AppendTo[Global`log,2];InputString["p: "];True)])[_,_]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Match-sequence"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);MatchQ[f[1,2,3],f[1,PatternTest[x__,Function[z,(AppendTo[Global`log,2];InputString["p: "];True)]],y_]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Match-repeated"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);MatchQ[f[1,2,3],f[Repeated[_?gate]]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[1, 1, 2, 1, 2, 3]"####));
    rows.push(make(r####"brief-548.Match-sequence-condition"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);MatchQ[f[1,2,3],f[x__ /; (AppendTo[Global`log,2];InputString["p: "];True), y__]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Match-flat"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);SetAttributes[f,Flat];MatchQ[f[1,2,3],f[1,_?gate,x__]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Match-orderless"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);SetAttributes[f,Orderless];MatchQ[f[1,2,3],f[_?gate,x__]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2, 3, 1, 3, 1, 2, 3]"####));
    rows.push(make(r####"brief-548.Match-orderless-flat"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);SetAttributes[f,{Orderless,Flat}];MatchQ[f[1,2,3],f[_?gate,x__]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2, f[2], 3, f[3], f[2, 3]]"####));
    rows.push(make(r####"brief-548.Match-orderless-sequence"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);SetAttributes[f,Orderless];MatchQ[f[1,2,3],f[PatternSequence[1,_?gate],3]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Match-optional-present"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);MatchQ[f[1,2,3],f[1,Optional[_?gate],3]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Match-key-value"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);MatchQ[<|"a"->1,"b"->2,"c"->3|>,KeyValuePattern[{"a"->_?gate,"b"->_?gate,"c"->_?gate}]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.Match-key-test"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);MatchQ[<|1->1,2->2,3->3|>,KeyValuePattern[{1->_,_?Function[z,(AppendTo[Global`log,z];If[SameQ[z,2],InputString["p: "];True,False])]->2}]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.ReplaceList-ordered"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);ReplaceList[f[1,2,3],f[1,x_?gate,3]:>{x,x}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[List[2, 2]]"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.ReplaceList-sequence"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);ReplaceList[f[1,2,3],f[x__ /; (AppendTo[Global`log,2];InputString["p: "];True),y__]:>{{x},{y}}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####), (r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[List[List[1], List[2, 3]], List[List[1, 2], List[3]]]"####), r####"List[2, 2]"####));
    rows.push(make(r####"brief-548.ReplaceList-flat"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);SetAttributes[f,Flat];ReplaceList[f[1,2,3],f[1,x_?gate,y__]:>{x,{y}}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[List[2, List[3]], List[f[2], List[3]]]"####), r####"List[2, f[2]]"####));
    rows.push(make(r####"brief-548.ReplaceList-orderless"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);SetAttributes[f,Orderless];ReplaceList[f[1,2,3],f[x_?gate,y__]:>{x,{y}}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[List[3, List[1, 2]], List[2, List[1, 3]], List[1, List[2, 3]]]"####), r####"List[2, 3, 1, 3, 1, 2, 3, 2, 1]"####));
    rows.push(make(r####"brief-548.ReplaceList-optional"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);ReplaceList[f[1,2,3],f[1,Optional[x_?gate],3]:>{x,x}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[List[2, 2]]"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.ReplaceList-guard"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);ReplaceList[f[1,2,3],f[1,x_,3]:>h[x]/;(AppendTo[Global`log,x];InputString["p: "];True)]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[h[2]]"####), r####"List[2]"####));
    rows.push(make(
        r####"brief-548.MatchQ-optional-default"####,
        r####"Default[f]:=(AppendTo[Global`log,2];Input["p: "]);MatchQ[f[3],f[x_.,3]]"####,
        &[(r####"Input"####, r####"p: "####, r####"2"####)],
        Some(r####"True"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-548.MatchQ-optional-default-orderless"####, r####"Default[f]:=(AppendTo[Global`log,2];Input["p: "]);SetAttributes[f,Orderless];MatchQ[f[3],f[x_.,3]]"####, &[(r####"Input"####, r####"p: "####, r####"2"####)], Some(r####"True"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.ReplaceList-optional-default"####, r####"Default[f]:=(AppendTo[Global`log,2];Input["p: "]);ReplaceList[f[3],f[x_.,3]:>{x,3}]"####, &[(r####"Input"####, r####"p: "####, r####"2"####)], Some(r####"List[List[2, 3]]"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.ReplaceList-optional-default-orderless"####, r####"Default[f]:=(AppendTo[Global`log,2];Input["p: "]);SetAttributes[f,Orderless];ReplaceList[f[3],f[x_.,3]:>{x,3}]"####, &[(r####"Input"####, r####"p: "####, r####"2"####)], Some(r####"List[List[2, 3]]"####), r####"List[2]"####));
    rows.push(make(
        r####"brief-548.ReplaceList-target"####,
        r####"ReplaceList[(AppendTo[Global`log,1];InputString["p: "];{1,2,3}),2->9]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[]"####),
        r####"List[1]"####,
    ));
    rows.push(make(
        r####"brief-548.ReplaceList-spec"####,
        r####"ReplaceList[{1,2,3},(AppendTo[Global`log,2];InputString["p: "];2->9)]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[]"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-548.Match-sequence-rest"####, r####"MatchQ[f[1,2,3],f[x__/;(AppendTo[Global`log,1];InputString["p: "];True),y__/;(AppendTo[Global`log,2];InputString["p: "];True)]]"####, &[(r####"InputString"####, r####"p: "####, r####"one"####), (r####"InputString"####, r####"p: "####, r####"two"####), (r####"InputString"####, r####"p: "####, r####"one"####)], Some(r####"True"####), r####"List[1, 2, 2]"####));
    rows.push(make(r####"brief-548.ReplaceList-sequence-rest"####, r####"ReplaceList[f[1,2,3],f[x__/;(AppendTo[Global`log,1];InputString["p: "];True),y__/;(AppendTo[Global`log,2];InputString["p: "];True)]:>{{x},{y}}]"####, &[(r####"InputString"####, r####"p: "####, r####"one"####), (r####"InputString"####, r####"p: "####, r####"two"####), (r####"InputString"####, r####"p: "####, r####"one"####), (r####"InputString"####, r####"p: "####, r####"two"####), (r####"InputString"####, r####"p: "####, r####"one"####)], Some(r####"List[List[List[1], List[2, 3]], List[List[1, 2], List[3]]]"####), r####"List[1, 2, 2, 1, 2]"####));
    rows.push(make(r####"brief-548.Match-flat-rest"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);SetAttributes[f,Flat];MatchQ[f[1,2,3],f[x_,y_?gate,z___]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.ReplaceList-flat-rest"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);SetAttributes[f,Flat];ReplaceList[f[1,2,3],f[x_,y_?gate,z___]:>{x,y,{z}}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[List[1, 2, List[3]], List[1, f[2], List[3]], List[1, f[2, 3], List[]], List[f[1], 2, List[3]], List[f[1], f[2], List[3]], List[f[1], f[2, 3], List[]], List[f[1, 2], 3, List[]], List[f[1, 2], f[3], List[]]]"####), r####"List[2, f[2], f[2, 3], 2, f[2], f[2, 3], 3, f[3]]"####));
    rows.push(make(r####"brief-548.MatchQ-optional-default-match"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);Default[f]=2;MatchQ[f[3],f[Optional[x_?gate],3]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"True"####), r####"List[3, 2]"####));
    rows.push(make(r####"brief-548.ReplaceList-optional-default-match"####, r####"seen=False;gate[v_]:=(AppendTo[Global`log,v];If[And[SameQ[v,2],Not[seen]],seen=True;InputString["p: "],Null];True);Default[f]=2;ReplaceList[f[3],f[Optional[x_?gate],3]:>{x,3}]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[List[2, 3]]"####), r####"List[3, 2]"####));
    rows.push(make(
        r####"brief-548.MatchQ-normalize-head"####,
        r####"MatchQ[f[1,2,3],(AppendTo[Global`log,1];Input["p: "])[_,_,_]]"####,
        &[(r####"Input"####, r####"p: "####, r####"f"####)],
        Some(r####"True"####),
        r####"List[1]"####,
    ));
    rows.push(make(r####"brief-548.MatchQ-normalize-twice"####, r####"MatchQ[f[1,2,3,4,5],f[1,(AppendTo[Global`log,2];Input["p: "]),3,(AppendTo[Global`log,4];Input["p: "]),5]]"####, &[(r####"Input"####, r####"p: "####, r####"2"####), (r####"Input"####, r####"p: "####, r####"4"####)], Some(r####"True"####), r####"List[2, 4]"####));
    rows.push(make(r####"brief-548.MatchQ-pattern-twice"####, r####"MatchQ[h[1,2,3,4,5],h[_,(AppendTo[Global`log,2];Input["p: "]),3,(AppendTo[Global`log,4];Input["p: "]),5]]"####, &[(r####"Input"####, r####"p: "####, r####"2"####), (r####"Input"####, r####"p: "####, r####"4"####)], Some(r####"True"####), r####"List[2, 4]"####));
    rows
}
