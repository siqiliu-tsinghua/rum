//! Return propagation during projection and absorption during value evaluation.
case!(
    director_01,
    r#"f[] := (Failure["t", <|"MessageTemplate" :> Return[3], "MessageParameters" :> (InputString["w"]; {})|>]["Message"]; 7)"#,
    r#"f[]"#,
    &["w"],
    r#"7"#,
    r#"List[]"#
);
case!(
    director_02,
    r#""#,
    r#"Do[r = Failure["t", <|"MessageTemplate" :> Return[3], "MessageParameters" :> (InputString["w"]; {})|>]["Message"]; s = 5, {1}]; {r, s}"#,
    &["w"],
    r#"List[3, 5]"#,
    r#"List[]"#
);
case!(
    director_03,
    r#"g[] := (Failure["t", <|"MessageTemplate" :> Return[3], "MessageParameters" :> (InputString["w"]; {})|>]["StyledMessage"]; 8)"#,
    r#"g[]"#,
    &["w"],
    r#"8"#,
    r#"List[]"#
);
case!(
    status_projection_return,
    r#"f[]:=(Failure["t",<|"MessageTemplate"->"x","MessageParameters":>(InputString["w"];Return[3])|>]["Message"];7)"#,
    r#"f[]"#,
    &["w"],
    r#"3"#,
    r#"List[]"#
);
case!(
    failure_value_return,
    r#""#,
    r#"f[] := (Failure["t",<|"x":>(InputString["value"];Return[3])|>]["x"];7);f[]"#,
    &["value"],
    r#"7"#,
    r#"List[]"#
);
case!(
    success_value_return,
    r#""#,
    r#"f[] := (Success["t",<|"x":>(InputString["value"];Return[3])|>]["x"];7);f[]"#,
    &["value"],
    r#"7"#,
    r#"List[]"#
);
case!(
    styled_loop_store,
    r#""#,
    r#"Do[r=Failure["t",<|"MessageTemplate":>Return[3],"MessageParameters":>(InputString["projection"];{})|>]["StyledMessage"];s=5,{1}];{r,s}"#,
    &["projection"],
    r#"List[3, 5]"#,
    r#"List[]"#
);
case!(
    projection_value_return,
    r#""#,
    r#"f[]:=(Failure["t",<|"MessageTemplate":>(InputString["value"];Return[3]),"MessageParameters":>(InputString["projection"];{})|>]["Message"];7);f[]"#,
    &["projection", "value"],
    r#"7"#,
    r#"List[]"#
);
