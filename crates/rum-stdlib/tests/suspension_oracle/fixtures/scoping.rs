use super::super::descriptors::Fixture;
use super::make;
pub fn fixtures() -> Vec<Fixture> {
    let mut rows = Vec::new();
    rows.push(make(
        r####"brief-548.WithCleanup"####,
        r####"WithCleanup[(AppendTo[Global`log,1];InputString["p: "]),AppendTo[Global`log,3]]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####""two""####),
        r####"List[1, 3]"####,
    ));
    rows.push(make(r####"brief-548.Block"####, r####"x=9;{Block[{x=1},(AppendTo[Global`log,x];InputString["p: "];AppendTo[Global`log,x];x)],x}"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, 9]"####), r####"List[1, 1]"####));
    rows.push(make(r####"brief-548.Module"####, r####"x=9;{Module[{x=1},(AppendTo[Global`log,x];InputString["p: "];AppendTo[Global`log,x];x)],x}"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, 9]"####), r####"List[1, 1]"####));
    rows.push(make(r####"brief-548.With"####, r####"x=9;{With[{x=1},(AppendTo[Global`log,x];InputString["p: "];AppendTo[Global`log,x];x)],x}"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[1, 9]"####), r####"List[1, 1]"####));
    rows.push(make(r####"brief-548.Block-init"####, r####"x=9;y=8;{Block[{x=(AppendTo[Global`log,1];InputString["p: "]),y=(AppendTo[Global`log,2];3)},AppendTo[Global`log,3];{x,y}],x,y}"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List[List["two", 3], 9, 8]"####), r####"List[1, 2, 3]"####));
    rows.push(make(r####"brief-548.Function-return"####, r####"Function[x,(AppendTo[Global`log,x];InputString["p: "];Return[7];AppendTo[Global`log,3])][2]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"7"####), r####"List[2]"####));
    rows.push(make(
        r####"brief-548.Block-single-init"####,
        r####"x=9;{Block[{x=(AppendTo[Global`log,2];InputString["p: "])},x],x}"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List["two", 9]"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.Block-multi-body"####,
        r####"x=9;y=8;{Block[{x=1,y=2},AppendTo[Global`log,x];InputString["p: "];{x,y}],x,y}"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List[List[1, 2], 9, 8]"####),
        r####"List[1]"####,
    ));
    rows
}
