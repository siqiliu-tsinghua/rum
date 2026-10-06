use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    let error = v == Variant::Error;
    let expected = match name {
        "Do" | "While" | "Until" | "For" => "Null",
        "Table" if role == "bounds" => "List[List[1], List[2], List[3], List[4], List[5]]",
        "Table" => "List[1, 2, 3, 4, 5]",
        "Sum" => "15",
        "Product" => "120",
        _ => unreachable!(),
    };
    if matches!(name, "Do" | "Table" | "Sum" | "Product") {
        if role == "bounds" {
            let bound = visits(&f, v, "Global`i", "1");
            f.source =
                format!("{name}[(Global`count++;Global`i),{{Global`i,5}},{{Global`j,{bound}}}]");
        } else {
            let body = visits(&f, v, "Global`i", "Global`i");
            f.source = format!("{name}[(Global`count++;{body}),{{Global`i,5}}]");
        }
        project(&mut f, "{Global`i,Global`j}", "List[91, 92]");
        project(
            &mut f,
            "Global`count",
            if error && role == "bounds" { "4" } else { "5" },
        );
    } else {
        let body = visits(&f, v, "Global`n", "Null");
        f.source = match (name, role) {
            ("While", "body") => format!("While[Global`n<5,(Global`n++;Global`count++;{body})]"),
            ("Until", "body") => format!("Until[Global`n==5,(Global`n++;Global`count++;{body})]"),
            ("While", "test") => {
                let test = visits(&f, v, "Global`n", "True");
                format!("While[(Global`n++;If[Global`n<=5,{test},False]),Global`count++]")
            }
            ("Until", "test") => {
                let test = visits(&f, v, "Global`n", "Global`n==5");
                format!("Until[{test},(Global`n++;Global`count++)]")
            }
            ("For", "body") => {
                format!("For[Global`n=1,Global`n<=5,Global`n++,(Global`count++;{body})]")
            }
            ("For", "increment") => {
                format!("For[Global`n=1,Global`n<=5,({body};Global`n++),Global`count++]")
            }
            ("For", "test") => {
                let test = visits(&f, v, "Global`n", "True");
                format!("For[Global`n=1,If[Global`n<=5,{test},False],Global`n++,Global`count++]")
            }
            ("For", "initialization") => {
                let init = sequence(&f, v, "Null");
                format!("For[({init};Global`n=1),Global`n<=5,Global`n++,Global`count++]")
            }
            _ => unreachable!(),
        };
        let count = if error && role == "initialization" {
            "0"
        } else if error && role == "test" && name != "Until" {
            "4"
        } else {
            "5"
        };
        let n = if error && role == "initialization" {
            "0"
        } else if error || name == "Until" || (name == "While" && role == "body") {
            "5"
        } else {
            "6"
        };
        project(&mut f, "Global`count", count);
        project(&mut f, "Global`n", n);
    }
    finish(f, v, expected)
}
