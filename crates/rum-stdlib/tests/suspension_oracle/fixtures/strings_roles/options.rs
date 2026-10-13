//! The heads are Ordinary; only RuleDelayed values reach the local option
//! resolver unevaluated. A computed option expression is not this role.
//! Values follow docs/reference/builtins.md and docs/guide/structured-data.md.
use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    let (key, value, arguments, expected) = contract(name);
    let value = sequence(&f, v, value);
    let options = if role == "defaultOption" {
        f.setup.push_str(&format!(
            ";Unprotect[{name}];SetOptions[{name},{key}:>{value}]"
        ));
        String::new()
    } else {
        assert_eq!(role, "explicitOption");
        format!(",{key}:>{value}")
    };
    f.source = format!("{name}[{arguments}{options}]");
    finish(f, v, expected)
}
fn contract(name: &str) -> (&str, &str, &str, &str) {
    match name {
        "AlignColumns" => (
            r#""Separator""#,
            r#""|""#,
            r#"{{"a","1"},{"bb","2"},{"c","3"},{"dd","4"},{"e","5"}}"#,
            r#""a |1\nbb|2\nc |3\ndd|4\ne |5""#,
        ),
        "HtmlUnescape" => (
            r#""Strict""#,
            "True",
            r#""&amp;&lt;&gt;&quot;&apos;""#,
            r#""&<>\"'""#,
        ),
        "ParseArgs" => (
            r#""ShortOptions""#,
            "True",
            r#"{"-ab","--color=red","--loud","file","tail"}"#,
            r#"Association[Rule["Positional", List["file", "tail"]], Rule["Options", Association[Rule["color", "red"]]], Rule["Flags", List["a", "b", "loud"]]]"#,
        ),
        "StringIndent" => (
            r#""BlankLines""#,
            "True",
            r#""a\n\nc\n \ne",">""#,
            r#"">a\n>\n>c\n> \n>e""#,
        ),
        "StringWrap" => (
            r#""PreserveNewlines""#,
            "True",
            r#""aa bb\n\ncc dd\n \nee ff",2"#,
            r#""aa\nbb\n\ncc\ndd\n \nee\nff""#,
        ),
        "UrlEncode" => (
            r#""FormEncoding""#,
            "True",
            r#""a b c d e""#,
            r#""a+b+c+d+e""#,
        ),
        "UrlDecode" => (
            r#""FormEncoding""#,
            "True",
            r#""a+b+c+d+e""#,
            r#""a b c d e""#,
        ),
        "UrlParse" => (
            r#""DecodeQuery""#,
            "True",
            r#""https://example.test:443/a?x=a+b#f""#,
            r#"Association[Rule["Scheme", "https"], Rule["Host", "example.test"], Rule["Port", 443], Rule["Path", "/a"], Rule["Query", "x=a b"], Rule["Fragment", "f"]]"#,
        ),
        "TemplateApply" => (
            r#""Strict""#,
            "True",
            r#""``-``-``-``-``",{"a","b","c","d","e"}"#,
            r#""a-b-c-d-e""#,
        ),
        "Uncompress" => (
            r#""Strict""#,
            "True",
            "Compress[{1,2,3,4,5}]",
            "List[1, 2, 3, 4, 5]",
        ),
        "StringContainsQ" | "StringStartsQ" | "StringEndsQ" | "StringMatchQ" => {
            ("IgnoreCase", "True", r#""ABCDE","abcde""#, "True")
        }
        "StringFreeQ" => ("IgnoreCase", "True", r#""ABCDE","abcde""#, "False"),
        "StringCases" => (
            "IgnoreCase",
            "True",
            r#""aBaBaBaBaB","b""#,
            r#"List["B", "B", "B", "B", "B"]"#,
        ),
        "StringCount" => ("IgnoreCase", "True", r#""aBaBaBaBaB","b""#, "5"),
        "StringDelete" => ("IgnoreCase", "True", r#""aBaBaBaBaB","b""#, r#""aaaaa""#),
        "StringPosition" => (
            "IgnoreCase",
            "True",
            r#""aBaBaBaBaB","b""#,
            "List[List[2, 2], List[4, 4], List[6, 6], List[8, 8], List[10, 10]]",
        ),
        "StringReplace" => (
            "IgnoreCase",
            "True",
            r#""aBaBaBaBaB","b"->"!""#,
            r#""a!a!a!a!a!""#,
        ),
        "StringSplit" => (
            "IgnoreCase",
            "True",
            r#""aBcBdBeBf","b""#,
            r#"List["a", "c", "d", "e", "f"]"#,
        ),
        _ => unreachable!("{name}"),
    }
}
