use super::assert_queued_and_resumed;

pub(super) const REMAINING_CONSUMERS: &[&str] = &[
    "Uncompress",
    "HtmlUnescape",
    "TemplateApply",
    "Union",
    "Intersection",
    "Complement",
    "ParseArgs",
    "UrlEncode",
    "UrlDecode",
    "UrlParse",
    "StringWrap",
    "StringIndent",
    "AlignColumns",
    "DuplicateFreeQ",
    "IntersectingQ",
    "DisjointQ",
    "SubsetQ",
    "ContainsAll",
    "ContainsAny",
    "ContainsExactly",
    "ContainsNone",
    "ContainsOnly",
];

#[test]
fn uncompress_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"Strict :> (InputString["option: "]; True)"#;
        let call = if default {
            format!(
                "Unprotect[Uncompress]; SetOptions[Uncompress, {option}]; Uncompress[{}]",
                r#"Compress[{a, b}]"#
            )
        } else {
            format!("Uncompress[{}, {option}]", r#"Compress[{a, b}]"#)
        };
        let source = call;
        assert_queued_and_resumed(&source, &[("option: ", "yes")], r#"List[a, b]"#);
    }
    println!(r#"consumer=Uncompress; queued=resumed=List[a, b]; default+explicit suspended"#);
}

#[test]
fn htmlunescape_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"Strict :> (InputString["option: "]; True)"#;
        let call = if default {
            format!(
                "Unprotect[HtmlUnescape]; SetOptions[HtmlUnescape, {option}]; HtmlUnescape[{}]",
                r#""&amp;""#
            )
        } else {
            format!("HtmlUnescape[{}, {option}]", r#""&amp;""#)
        };
        let source = call;
        assert_queued_and_resumed(&source, &[("option: ", "yes")], r#""&""#);
    }
    println!(r#"consumer=HtmlUnescape; queued=resumed="&"; default+explicit suspended"#);
}

#[test]
fn templateapply_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"Strict :> (InputString["option: "]; True)"#;
        let call = if default {
            format!(
                "Unprotect[TemplateApply]; SetOptions[TemplateApply, {option}]; TemplateApply[{}]",
                r#""`name`", <|"name" -> "Ada"|>"#
            )
        } else {
            format!(
                "TemplateApply[{}, {option}]",
                r#""`name`", <|"name" -> "Ada"|>"#
            )
        };
        let source = call;
        assert_queued_and_resumed(&source, &[("option: ", "yes")], r#""Ada""#);
    }
    println!(r#"consumer=TemplateApply; queued=resumed="Ada"; default+explicit suspended"#);
}

#[test]
fn union_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (True &))"#;
        let call = if default {
            format!(
                "Unprotect[Union]; SetOptions[Union, {option}]; Union[{}]",
                r#"{1, 3}"#
            )
        } else {
            format!("Union[{}, {option}]", r#"{1, 3}"#)
        };
        let source = call;
        assert_queued_and_resumed(&source, &[("option: ", "yes")], r#"List[1]"#);
    }
    println!(r#"consumer=Union; queued=resumed=List[1]; default+explicit suspended"#);
}

#[test]
fn intersection_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (True &))"#;
        let call = if default {
            format!(
                "Unprotect[Intersection]; SetOptions[Intersection, {option}]; Intersection[{}]",
                r#"{1}, {3}"#
            )
        } else {
            format!("Intersection[{}, {option}]", r#"{1}, {3}"#)
        };
        let source = call;
        assert_queued_and_resumed(&source, &[("option: ", "yes")], r#"List[1]"#);
    }
    println!(r#"consumer=Intersection; queued=resumed=List[1]; default+explicit suspended"#);
}

#[test]
fn complement_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (True &))"#;
        let call = if default {
            format!(
                "Unprotect[Complement]; SetOptions[Complement, {option}]; Complement[{}]",
                r#"{1}, {3}"#
            )
        } else {
            format!("Complement[{}, {option}]", r#"{1}, {3}"#)
        };
        let source = call;
        assert_queued_and_resumed(&source, &[("option: ", "yes")], r#"List[]"#);
    }
    println!(r#"consumer=Complement; queued=resumed=List[]; default+explicit suspended"#);
}

#[test]
fn parseargs_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"ShortOptions :> (InputString["option: "]; True)"#;
        let call = if default {
            format!(
                "Unprotect[ParseArgs]; SetOptions[ParseArgs, {option}]; ParseArgs[{}]",
                r#"{"-ab", "tail"}"#
            )
        } else {
            format!("ParseArgs[{}, {option}]", r#"{"-ab", "tail"}"#)
        };
        let source = call;
        assert_queued_and_resumed(
            &source,
            &[("option: ", "yes")],
            r#"Association[Rule["Positional", List["tail"]], Rule["Options", Association[]], Rule["Flags", List["a", "b"]]]"#,
        );
    }
    println!(
        r#"consumer=ParseArgs; queued=resumed=Association[Rule["Positional", List["tail"]], Rule["Options", Association[]], Rule["Flags", List["a", "b"]]]; default+explicit suspended"#
    );
}

#[test]
fn urlencode_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"FormEncoding :> (InputString["option: "]; True)"#;
        let call = if default {
            format!(
                "Unprotect[UrlEncode]; SetOptions[UrlEncode, {option}]; UrlEncode[{}]",
                r#""a b""#
            )
        } else {
            format!("UrlEncode[{}, {option}]", r#""a b""#)
        };
        let source = call;
        assert_queued_and_resumed(&source, &[("option: ", "yes")], r#""a+b""#);
    }
    println!(r#"consumer=UrlEncode; queued=resumed="a+b"; default+explicit suspended"#);
}

#[test]
fn urldecode_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"FormEncoding :> (InputString["option: "]; True)"#;
        let call = if default {
            format!(
                "Unprotect[UrlDecode]; SetOptions[UrlDecode, {option}]; UrlDecode[{}]",
                r#""a+b""#
            )
        } else {
            format!("UrlDecode[{}, {option}]", r#""a+b""#)
        };
        let source = call;
        assert_queued_and_resumed(&source, &[("option: ", "yes")], r#""a b""#);
    }
    println!(r#"consumer=UrlDecode; queued=resumed="a b"; default+explicit suspended"#);
}

#[test]
fn urlparse_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"DefaultScheme :> (InputString["option: "]; "https")"#;
        let call = if default {
            format!(
                "Unprotect[UrlParse]; SetOptions[UrlParse, {option}]; UrlParse[{}]",
                r#""example.com/a""#
            )
        } else {
            format!("UrlParse[{}, {option}]", r#""example.com/a""#)
        };
        let source = call;
        assert_queued_and_resumed(
            &source,
            &[("option: ", "yes")],
            r#"Association[Rule["Scheme", "https"], Rule["Host", "example.com"], Rule["Port", Null], Rule["Path", "/a"], Rule["Query", Null], Rule["Fragment", Null]]"#,
        );
    }
    println!(
        r#"consumer=UrlParse; queued=resumed=Association[Rule["Scheme", "https"], Rule["Host", "example.com"], Rule["Port", Null], Rule["Path", "/a"], Rule["Query", Null], Rule["Fragment", Null]]; default+explicit suspended"#
    );
}

#[test]
fn stringwrap_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"PreserveNewlines :> (InputString["option: "]; True)"#;
        let call = if default {
            format!(
                "Unprotect[StringWrap]; SetOptions[StringWrap, {option}]; StringWrap[{}]",
                r#""a\nb", 10"#
            )
        } else {
            format!("StringWrap[{}, {option}]", r#""a\nb", 10"#)
        };
        let source = call;
        assert_queued_and_resumed(&source, &[("option: ", "yes")], r#""a\nb""#);
    }
    println!(r#"consumer=StringWrap; queued=resumed="a\nb"; default+explicit suspended"#);
}

#[test]
fn stringindent_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"BlankLines :> (InputString["option: "]; True)"#;
        let call = if default {
            format!(
                "Unprotect[StringIndent]; SetOptions[StringIndent, {option}]; StringIndent[{}]",
                r#""a\n\nb", ">""#
            )
        } else {
            format!("StringIndent[{}, {option}]", r#""a\n\nb", ">""#)
        };
        let source = call;
        assert_queued_and_resumed(&source, &[("option: ", "yes")], r#"">a\n>\n>b""#);
    }
    println!(r#"consumer=StringIndent; queued=resumed=">a\n>\n>b"; default+explicit suspended"#);
}

#[test]
fn aligncolumns_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"Separator :> (InputString["option: "]; "|")"#;
        let call = if default {
            format!(
                "Unprotect[AlignColumns]; SetOptions[AlignColumns, {option}]; AlignColumns[{}]",
                r#"{{"a", "b"}, {"long", "c"}}"#
            )
        } else {
            format!(
                "AlignColumns[{}, {option}]",
                r#"{{"a", "b"}, {"long", "c"}}"#
            )
        };
        let source = call;
        assert_queued_and_resumed(&source, &[("option: ", "yes")], r#""a   |b\nlong|c""#);
    }
    println!(
        r#"consumer=AlignColumns; queued=resumed="a   |b\nlong|c"; default+explicit suspended"#
    );
}

#[test]
fn duplicatefreeq_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (AppendTo[seen, compared]; True) &)"#;
        let call = if default {
            format!(
                "Unprotect[DuplicateFreeQ]; SetOptions[DuplicateFreeQ, {option}]; DuplicateFreeQ[{}]",
                r#"{1, 3}"#
            )
        } else {
            format!("DuplicateFreeQ[{}, {option}]", r#"{1, 3}"#)
        };
        let source = format!("seen = {{}}; result = ({}); {{result, seen}}", call);
        assert_queued_and_resumed(
            &source,
            &[("option: ", "yes")],
            r#"List[False, List[compared]]"#,
        );
    }
    println!(
        r#"consumer=DuplicateFreeQ; queued=resumed=List[False, List[compared]]; default+explicit suspended"#
    );
}

#[test]
fn intersectingq_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (AppendTo[seen, compared]; True) &)"#;
        let call = if default {
            format!(
                "Unprotect[IntersectingQ]; SetOptions[IntersectingQ, {option}]; IntersectingQ[{}]",
                r#"{1}, {3}"#
            )
        } else {
            format!("IntersectingQ[{}, {option}]", r#"{1}, {3}"#)
        };
        let source = format!("seen = {{}}; result = ({}); {{result, seen}}", call);
        assert_queued_and_resumed(
            &source,
            &[("option: ", "yes")],
            r#"List[True, List[compared]]"#,
        );
    }
    println!(
        r#"consumer=IntersectingQ; queued=resumed=List[True, List[compared]]; default+explicit suspended"#
    );
}

#[test]
fn disjointq_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (AppendTo[seen, compared]; True) &)"#;
        let call = if default {
            format!(
                "Unprotect[DisjointQ]; SetOptions[DisjointQ, {option}]; DisjointQ[{}]",
                r#"{1}, {3}"#
            )
        } else {
            format!("DisjointQ[{}, {option}]", r#"{1}, {3}"#)
        };
        let source = format!("seen = {{}}; result = ({}); {{result, seen}}", call);
        assert_queued_and_resumed(
            &source,
            &[("option: ", "yes")],
            r#"List[False, List[compared]]"#,
        );
    }
    println!(
        r#"consumer=DisjointQ; queued=resumed=List[False, List[compared]]; default+explicit suspended"#
    );
}

#[test]
fn subsetq_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (AppendTo[seen, compared]; True) &)"#;
        let call = if default {
            format!(
                "Unprotect[SubsetQ]; SetOptions[SubsetQ, {option}]; SubsetQ[{}]",
                r#"{1}, {3}"#
            )
        } else {
            format!("SubsetQ[{}, {option}]", r#"{1}, {3}"#)
        };
        let source = format!("seen = {{}}; result = ({}); {{result, seen}}", call);
        assert_queued_and_resumed(
            &source,
            &[("option: ", "yes")],
            r#"List[True, List[compared]]"#,
        );
    }
    println!(
        r#"consumer=SubsetQ; queued=resumed=List[True, List[compared]]; default+explicit suspended"#
    );
}

#[test]
fn containsall_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (AppendTo[seen, compared]; True) &)"#;
        let call = if default {
            format!(
                "Unprotect[ContainsAll]; SetOptions[ContainsAll, {option}]; ContainsAll[{}]",
                r#"{1}, {3}"#
            )
        } else {
            format!("ContainsAll[{}, {option}]", r#"{1}, {3}"#)
        };
        let source = format!("seen = {{}}; result = ({}); {{result, seen}}", call);
        assert_queued_and_resumed(
            &source,
            &[("option: ", "yes")],
            r#"List[True, List[compared]]"#,
        );
    }
    println!(
        r#"consumer=ContainsAll; queued=resumed=List[True, List[compared]]; default+explicit suspended"#
    );
}

#[test]
fn containsany_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (AppendTo[seen, compared]; True) &)"#;
        let call = if default {
            format!(
                "Unprotect[ContainsAny]; SetOptions[ContainsAny, {option}]; ContainsAny[{}]",
                r#"{1}, {3}"#
            )
        } else {
            format!("ContainsAny[{}, {option}]", r#"{1}, {3}"#)
        };
        let source = format!("seen = {{}}; result = ({}); {{result, seen}}", call);
        assert_queued_and_resumed(
            &source,
            &[("option: ", "yes")],
            r#"List[True, List[compared]]"#,
        );
    }
    println!(
        r#"consumer=ContainsAny; queued=resumed=List[True, List[compared]]; default+explicit suspended"#
    );
}

#[test]
fn containsexactly_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (AppendTo[seen, compared]; True) &)"#;
        let call = if default {
            format!(
                "Unprotect[ContainsExactly]; SetOptions[ContainsExactly, {option}]; ContainsExactly[{}]",
                r#"{1}, {3}"#
            )
        } else {
            format!("ContainsExactly[{}, {option}]", r#"{1}, {3}"#)
        };
        let source = format!("seen = {{}}; result = ({}); {{result, seen}}", call);
        assert_queued_and_resumed(
            &source,
            &[("option: ", "yes")],
            r#"List[True, List[compared]]"#,
        );
    }
    println!(
        r#"consumer=ContainsExactly; queued=resumed=List[True, List[compared]]; default+explicit suspended"#
    );
}

#[test]
fn containsnone_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (AppendTo[seen, compared]; True) &)"#;
        let call = if default {
            format!(
                "Unprotect[ContainsNone]; SetOptions[ContainsNone, {option}]; ContainsNone[{}]",
                r#"{1}, {3}"#
            )
        } else {
            format!("ContainsNone[{}, {option}]", r#"{1}, {3}"#)
        };
        let source = format!("seen = {{}}; result = ({}); {{result, seen}}", call);
        assert_queued_and_resumed(
            &source,
            &[("option: ", "yes")],
            r#"List[False, List[compared]]"#,
        );
    }
    println!(
        r#"consumer=ContainsNone; queued=resumed=List[False, List[compared]]; default+explicit suspended"#
    );
}

#[test]
fn containsonly_resumes_into_consumer() {
    for default in [false, true] {
        let option = r#"SameTest :> (InputString["option: "]; (AppendTo[seen, compared]; True) &)"#;
        let call = if default {
            format!(
                "Unprotect[ContainsOnly]; SetOptions[ContainsOnly, {option}]; ContainsOnly[{}]",
                r#"{1}, {3}"#
            )
        } else {
            format!("ContainsOnly[{}, {option}]", r#"{1}, {3}"#)
        };
        let source = format!("seen = {{}}; result = ({}); {{result, seen}}", call);
        assert_queued_and_resumed(
            &source,
            &[("option: ", "yes")],
            r#"List[True, List[compared]]"#,
        );
    }
    println!(
        r#"consumer=ContainsOnly; queued=resumed=List[True, List[compared]]; default+explicit suspended"#
    );
}
