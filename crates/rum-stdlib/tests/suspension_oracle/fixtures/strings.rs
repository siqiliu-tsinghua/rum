use super::super::descriptors::Fixture;
use super::make;
pub fn fixtures() -> Vec<Fixture> {
    let mut rows = Vec::new();
    rows.push(make(
        r####"brief-548.StringReplace"####,
        r####"StringReplace["aba","b":>(AppendTo[Global`log,2];InputString["p: "])]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####""atwoa""####),
        r####"List[2]"####,
    ));
    rows.push(make(
        r####"brief-548.StringCases"####,
        r####"StringCases["aba","b":>(AppendTo[Global`log,2];InputString["p: "])]"####,
        &[(r####"InputString"####, r####"p: "####, r####"two"####)],
        Some(r####"List["two"]"####),
        r####"List[2]"####,
    ));
    rows.push(make(r####"brief-548.ToExpression"####, r####"ToExpression["AppendTo[Global`log,1];InputString[\"p: \"];AppendTo[Global`log,3];7"]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"7"####), r####"List[1, 3]"####));
    rows.push(make(r####"brief-548.Failure-Message"####, r####"Failure["tag",<|"MessageTemplate"->"value ``", "MessageParameters":>(AppendTo[Global`log,2];{InputString["p: "]})|>]["Message"]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####""value two""####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Failure-StyledMessage"####, r####"Failure["tag",<|"MessageTemplate"->"value ``", "MessageParameters":>(AppendTo[Global`log,2];{InputString["p: "]})|>]["StyledMessage"]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"Style["value two", "Message"]"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Uncompress"####, r####"c=Compress[f[]];f[]:=(AppendTo[Global`log,2];InputString["p: "];77);Uncompress[c]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"77"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.Failure-message-template"####, r####"f::msg:=(AppendTo[Global`log,2];InputString["p: "]);Failure["tag",<|"MessageTemplate":>f::msg|>]["Message"]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####""two""####), r####"List[2]"####));
    rows.push(make(r####"brief-548.ToExpression-head"####, r####"ToExpression["1",InputForm,Function[x,(AppendTo[Global`log,2];InputString["p: "];77)]]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"77"####), r####"List[2]"####));
    rows.push(make(r####"brief-548.StringCases-fourth"####, r####"StringCases["aba","b":>(AppendTo[Global`log,2];InputString["p: "]),1,IgnoreCase->False]"####, &[(r####"InputString"####, r####"p: "####, r####"two"####)], Some(r####"List["two"]"####), r####"List[2]"####));
    rows
}
