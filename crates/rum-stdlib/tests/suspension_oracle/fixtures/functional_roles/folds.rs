use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, 5);
    let (name, role) = entry.split_once('.').unwrap();
    let result = if name == "FoldPair" {
        if role == "combiner" {
            "Total[p]"
        } else {
            "{10 (x+y),x+y}"
        }
    } else if role == "predicate" {
        if v == Variant::Decline {
            "If[Global`state==5,Global`unknown,True]"
        } else {
            "x<15"
        }
    } else {
        "x+y"
    };
    let body = visits(&f, v, 5, result);
    let parameters = if role == "combiner" {
        "p"
    } else if role == "predicate" {
        "x"
    } else {
        "{x,y}"
    };
    f.setup
        .push_str(&format!(";Global`f=Function[{parameters},{body}]"));
    let (source, expected) = match (name, role) {
        ("Fold", _) => ("Fold[Global`f,0,{1,2,3,4,5}]", "15"),
        ("FoldList", _) => (
            "FoldList[Global`f,0,{1,2,3,4,5}]",
            "List[0, 1, 3, 6, 10, 15]",
        ),
        ("FoldPair", "callback") => ("FoldPair[Global`f,0,{1,2,3,4,5}]", "150"),
        ("FoldPair", "combiner") => (
            "FoldPair[Function[{x,y},{10 (x+y),x+y}],0,{1,2,3,4,5},Global`f]",
            "165",
        ),
        ("FoldWhile", "callback") => ("FoldWhile[Global`f,0,{1,2,3,4,5},Function[x,True]]", "15"),
        ("FoldWhile", "predicate") => ("FoldWhile[Plus,0,{1,2,3,4,5,6},Global`f]", "15"),
        ("SequenceFold", _) => ("SequenceFold[Global`f,{0},{1,2,3,4,5}]", "15"),
        ("SequenceFoldList", _) => (
            "SequenceFoldList[Global`f,{0},{1,2,3,4,5}]",
            "List[0, 1, 3, 6, 10, 15]",
        ),
        _ => unreachable!("{entry}"),
    };
    f.source = source.into();
    finish(f, v, expected)
}

pub fn iteration(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, 5);
    let (name, role) = entry.split_once('.').unwrap();
    let result = if role == "predicate" {
        if v == Variant::Decline {
            "If[x<4,True,Global`unknown]"
        } else {
            "x<4"
        }
    } else if name.starts_with("FixedPoint") {
        "Min[x+1,4]"
    } else {
        "x+1"
    };
    let body = visits(&f, v, 5, result);
    f.setup.push_str(&format!(";Global`f=Function[x,{body}]"));
    let tail = if name.starts_with("NestWhile") {
        if role == "predicate" {
            "Function[x,x+1],0,Global`f"
        } else {
            "Global`f,0,Function[x,x<5]"
        }
    } else {
        "Global`f,0,5"
    };
    f.source = format!("{name}[{tail}]");
    let expected = if name.starts_with("FixedPoint") {
        if name.ends_with("List") {
            "List[0, 1, 2, 3, 4, 4]"
        } else {
            "4"
        }
    } else if role == "predicate" {
        if name.ends_with("List") {
            "List[0, 1, 2, 3, 4]"
        } else {
            "4"
        }
    } else if name.ends_with("List") {
        "List[0, 1, 2, 3, 4, 5]"
    } else {
        "5"
    };
    finish(f, v, expected)
}

pub fn predicate(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v, 5);
    let name = entry.split_once('.').unwrap().0;
    let result = if name == "SelectFirst" {
        if v == Variant::Decline {
            "If[x<5,False,Global`unknown]"
        } else {
            "x==5"
        }
    } else if v == Variant::Decline {
        "If[x<5,True,Global`unknown]"
    } else {
        "True"
    };
    let body = visits(&f, v, 5, result);
    f.setup.push_str(&format!(";Global`f=Function[x,{body}]"));
    f.source = format!(
        "{name}[{{1,2,3,4,5}},Global`f{}]",
        if name == "SelectFirst" { ",-1" } else { "" }
    );
    let expected = match (name, v == Variant::Decline) {
        ("SelectFirst", true) => "-1",
        ("SelectFirst", false) | ("LengthWhile", false) => "5",
        ("LengthWhile", true) => "4",
        (_, true) => "List[1, 2, 3, 4]",
        _ => "List[1, 2, 3, 4, 5]",
    };
    finish(f, v, expected)
}
