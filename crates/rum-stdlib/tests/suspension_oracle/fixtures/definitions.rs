use super::super::descriptors::Fixture;
use super::make;
pub fn fixtures() -> Vec<Fixture> {
    let mut rows = Vec::new();
    rows.push(make(
        r####"brief-548.HeldUpdate-selector"####,
        r####"a={1,2,3};{a[[(AppendTo[Global`log,2];Input["p: "])]]++,a}"####,
        &[(r####"Input"####, r####"p: "####, r####"2"####)],
        Some(r####"List[2, List[1, 3, 3]]"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.Set-destructure"####,
        r####"a={1,2,3};{x,a[[(AppendTo[Global`log,2];Input["p: "])]]}={7,8};{x,a}"####,
        &[(r####"Input"####, r####"p: "####, r####"2"####)],
        Some(r####"List[7, List[1, 8, 3]]"####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-583.destructure-result"####,
        r####"a={0};b=0; {a[[(AppendTo[Global`log,1];InputString["p: "];1)]],b}={7,9}"####,
        &[(r####"InputString"####, r####"p: "####, r####"x"####)],
        Some(r####"List[7, 9]"####),
        r####"List[1]"####,
    ));
    rows.push(make(r####"brief-583.destructure-suffix"####, r####"a={0};b=0; r=({a[[(AppendTo[Global`log,1];InputString["p: "];1)]],b}={7,9});{r,a,b}"####, &[(r####"InputString"####, r####"p: "####, r####"x"####)], Some(r####"List[List[7, 9], List[7], 9]"####), r####"List[1]"####));
    rows.push(make(r####"brief-583.destructure-twice"####, r####"a={0,0};b=0;r=({a[[(AppendTo[Global`log,1];InputString["p: "];1)]],a[[(AppendTo[Global`log,2];InputString["q: "];2)]],b}={7,8,9});{r,a,b}"####, &[(r####"InputString"####, r####"p: "####, r####"x"####), (r####"InputString"####, r####"q: "####, r####"y"####)], Some(r####"List[List[7, 8, 9], List[7, 8], 9]"####), r####"List[1, 2]"####));
    rows.push(make(r####"brief-583.destructure-nested"####, r####"a={0};b=0;r=({{a[[(AppendTo[Global`log,1];InputString["p: "];1)]]},b}={{7},9});{r,a,b}"####, &[(r####"InputString"####, r####"p: "####, r####"x"####)], Some(r####"List[List[List[7], 9], List[7], 9]"####), r####"List[1]"####));
    rows.push(make(r####"brief-583.default-decline"####, r####"n=0;Default[d]:=7/;(n=n+1;AppendTo[Global`log,n];If[n==1,InputString["p: "]];n>=2);Default[d]"####, &[(r####"InputString"####, r####"p: "####, r####"x"####)], Some(r####"7"####), r####"List[1, 2]"####));
    rows.push(make(r####"brief-583.default-decline-twice"####, r####"n=0;Default[d]:=7/;(n=n+1;AppendTo[Global`log,n];If[n<=2,InputString["p: "]];n>=3);Default[d]"####, &[(r####"InputString"####, r####"p: "####, r####"x"####), (r####"InputString"####, r####"p: "####, r####"y"####)], Some(r####"7"####), r####"List[1, 2, 3]"####));
    rows
}
