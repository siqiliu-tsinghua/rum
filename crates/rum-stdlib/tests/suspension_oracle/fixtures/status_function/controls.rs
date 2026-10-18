//! Passing status projection, consumer, held-value, and splice controls.
case!(
    director_07,
    r#""#,
    r#"Failure["t", <|"MessageTemplate" :> (1 + 1), "MessageParameters" :> (InputString["w"]; {})|>]["Message"]"#,
    &["w"],
    r#"2"#,
    r#"List[]"#
);
case!(
    director_08,
    r#"x = 5"#,
    r#"Failure["t", <|"MessageTemplate" :> x, "MessageParameters" :> (InputString["w"]; {})|>]["Message"]"#,
    &["w"],
    r#"5"#,
    r#"List[]"#
);
case!(
    director_09,
    r#""#,
    r#"Failure["t", <|"MessageTemplate" :> Return[3], "MessageParameters" :> (InputString["w"]; {})|>]["Message"]"#,
    &["w"],
    r#"3"#,
    r#"List[]"#
);
case!(
    director_10,
    r#""#,
    r#"Catch[Failure["t", <|"MessageTemplate" :> Throw[4], "MessageParameters" :> (InputString["w"]; {})|>]["Message"]; 8]"#,
    &["w"],
    r#"4"#,
    r#"List[]"#
);
case!(
    director_11,
    r#""#,
    r#"{Failure["t", <|"MessageTemplate" :> Sequence[1, 2], "MessageParameters" :> (InputString["w"]; {})|>]["Message"]}"#,
    &["w"],
    r#"List[1, 2]"#,
    r#"List[]"#
);
case!(
    director_12,
    r#""#,
    r#"Failure["t", <|"MessageTemplate" -> System`InternalContinuationNone[], "MessageParameters" :> (InputString["w"]; {})|>]["StyledMessage"]"#,
    &["w"],
    r#"Style[InternalContinuationNone[], "Message"]"#,
    r#"List[]"#
);
case!(
    director_13,
    r#""#,
    r#"Enclose[Confirm[Failure["t", <|"MessageTemplate" :> Return[3], "MessageParameters" :> (InputString["w"]; {})|>]], "Message"]"#,
    &["w"],
    r#"3"#,
    r#"List[]"#
);
case!(
    director_14,
    r#"h[] := (Enclose[Confirm[$Failed], (InputString["h"]; #["ConfirmationType"]) &]; 9)"#,
    r#"h[]"#,
    &["h"],
    r#"9"#,
    r#"List[]"#
);
case!(
    status_template,
    r#"m::x := (InputString["w"]; "value=`1`")"#,
    r#"Failure["t",<|"MessageTemplate" :> m::x,"MessageParameters"->{3}|>]["Message"]"#,
    &["w"],
    r#""value=3""#,
    r#"List[]"#
);
case!(
    status_twice,
    r#"m::x := (InputString["w"]; "value=`1`")"#,
    r#"Failure["t",<|"MessageTemplate" :> m::x,"MessageParameters":>(InputString["w"];{3})|>]["StyledMessage"]"#,
    &["w", "w"],
    r#"Style["value=3", "Message"]"#,
    r#"List[]"#
);
case!(
    status_unevaluated,
    r#""#,
    r#"f[Failure["t",<|"MessageTemplate":>Unevaluated[x],"MessageParameters":>(InputString["w"];{})|>]["Message"]]"#,
    &["w"],
    r#"f[x]"#,
    r#"List[]"#
);
case!(
    status_held,
    r#""#,
    r#"f[Failure["t",<|"MessageTemplate":>Hold[Return[3]],"MessageParameters":>(InputString["w"];{})|>]["Message"]]"#,
    &["w"],
    r#"f[Hold[Return[3]]]"#,
    r#"List[]"#
);
case!(
    status_sideeffects,
    r#""#,
    r#"Failure["t",<|"MessageTemplate":>(AppendTo[log,2];3),"MessageParameters":>(AppendTo[log,1];InputString["w"];{})|>]["Message"];AppendTo[log,4];log"#,
    &["w"],
    r#"List[1, 2, 4]"#,
    r#"List[1, 2, 4]"#
);
case!(
    status_list_property,
    r#""#,
    r#"Failure["t",<|"x":>(InputString["w"];3)|>][{"x","y"}]"#,
    &["w"],
    r#"List[3, Missing["NotAvailable", List["x", "y"]]]"#,
    r#"List[]"#
);
case!(
    status_enclose_none,
    r#""#,
    r#"Enclose[Confirm[Failure["t",<|"MessageTemplate"->System`InternalContinuationNone[],"MessageParameters":>(InputString["w"];{})|>]],"Message"]"#,
    &["w"],
    r#"InternalContinuationNone[]"#,
    r#"List[]"#
);
case!(
    status_confirm_none,
    r#""#,
    r#"Enclose[Confirm[Failure["t",<|"MessageTemplate"->System`InternalContinuationNone[],"MessageParameters":>(InputString["w"];{})|>]],"MessageTemplate"]"#,
    &["w"],
    r#"InternalContinuationNone[]"#,
    r#"List[]"#
);
case!(
    status_nothing,
    r#""#,
    r#"{Failure["t",<|"MessageTemplate":>Nothing,"MessageParameters":>(InputString["w"];{})|>]["Message"],8}"#,
    &["w"],
    r#"List[8]"#,
    r#"List[]"#
);
case!(
    sentinels_held,
    r#""#,
    r#"{11,Failure["t",<|"MessageTemplate":>Hold[Return[3]],"MessageParameters":>(InputString["projection"];{})|>]["Message"],22}"#,
    &["projection"],
    r#"List[11, Hold[Return[3]], 22]"#,
    r#"List[]"#
);
case!(
    sentinels_unevaluated,
    r#""#,
    r#"{11,Failure["t",<|"MessageTemplate":>Unevaluated[x],"MessageParameters":>(InputString["projection"];{})|>]["Message"],22}"#,
    &["projection"],
    r#"List[11, x, 22]"#,
    r#"List[]"#
);
case!(
    sentinels_sequence,
    r#""#,
    r#"{11,Failure["t",<|"MessageTemplate":>Sequence[1,2],"MessageParameters":>(InputString["projection"];{})|>]["Message"],22}"#,
    &["projection"],
    r#"List[11, 1, 2, 22]"#,
    r#"List[]"#
);
case!(
    sentinels_nothing,
    r#""#,
    r#"{11,Failure["t",<|"MessageTemplate":>Nothing,"MessageParameters":>(InputString["projection"];{})|>]["Message"],22}"#,
    &["projection"],
    r#"List[11, 22]"#,
    r#"List[]"#
);
case!(
    harness_parameters,
    r#""#,
    r#"Failure["t", <|"MessageTemplate" -> "m `1`", "MessageParameters" :> (InputString["projection"]; {1})|>]["Message"]"#,
    &["projection"],
    r#""m 1""#,
    r#"List[]"#
);
case!(
    harness_template,
    r#"g15msg::m := (InputString["template"]; "m `1`");"#,
    r#"Failure["t", <|"MessageTemplate" :> g15msg::m, "MessageParameters" -> {1}|>]["Message"]"#,
    &["template"],
    r#""m 1""#,
    r#"List[]"#
);
case!(
    harness_styled,
    r#""#,
    r#"Failure["t", <|"MessageTemplate" -> "m `1`", "MessageParameters" :> (InputString["projection"]; {1})|>]["StyledMessage"]"#,
    &["projection"],
    r#"Style["m 1", "Message"]"#,
    r#"List[]"#
);
case!(
    harness_raw_failure,
    r#""#,
    r#"Failure["t", <|"x" :> (InputString["value"]; 17)|>]["x"]"#,
    &["value"],
    r#"17"#,
    r#"List[]"#
);
case!(
    harness_raw_success,
    r#""#,
    r#"Success["t", <|"x" :> (InputString["value"]; 17)|>]["x"]"#,
    &["value"],
    r#"17"#,
    r#"List[]"#
);
case!(
    harness_projection_value,
    r#""#,
    r#"Failure["t", <|"MessageTemplate" :> (InputString["value"]; 19), "MessageParameters" :> (InputString["projection"]; {})|>]["Message"]"#,
    &["projection", "value"],
    r#"19"#,
    r#"List[]"#
);
case!(
    harness_enclose,
    r#""#,
    r#"Enclose[Throw[Failure["t", <|"MessageTemplate" -> "m `1`", "MessageParameters" :> (InputString["projection"]; {1})|>], t], "Message", t]"#,
    &["projection"],
    r#""m 1""#,
    r#"List[]"#
);
case!(
    harness_confirm,
    r#""#,
    r#"Enclose[Confirm[Failure["t", <|"MessageTemplate" -> "m `1`", "MessageParameters" :> (InputString["projection"]; {1})|>]], "Message"]"#,
    &["projection"],
    r#""m 1""#,
    r#"List[]"#
);
