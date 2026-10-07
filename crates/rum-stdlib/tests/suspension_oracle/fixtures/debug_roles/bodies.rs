use super::super::super::descriptors::{Fixture, Variant};
use super::support::*;

pub fn fixture(entry: &str, v: Variant) -> Fixture {
    let mut f = base(entry, v);
    let (name, role) = entry.split_once('.').unwrap();
    let mut expected = "73".to_string();
    f.source = match (name, role) {
        ("Timing" | "AbsoluteTiming", _) => {
            f.duration_paths = vec![vec![0]];
            expected = "List[0, 73]".into();
            format!("{name}[{}]", sequence(&f, v, "73"))
        }
        ("RepeatedTiming", _) => {
            // rum's contract runs exactly three samples. Only the first sample
            // requests input; all 15 operations and their outputs are retained.
            f.duration_paths = vec![vec![0]];
            expected = "List[0, 73]".into();
            let first = sequence(&f, v, "73");
            let rest =
                sequence(&f, Variant::Middle, "73").replace(&format!("Input[\"{entry}:2\"];"), "");
            if v != Variant::Error {
                f.expected_log = Some("List[1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 1, 2, 3, 4, 5]".into());
                f.expected_prints = Some(
                    (0..3)
                        .flat_map(|_| (1..=5).map(|i| i.to_string()))
                        .collect(),
                );
            }
            project(
                &mut f,
                "Global`count",
                if v == Variant::Error { "1" } else { "3" },
            );
            format!("RepeatedTiming[Global`count++;If[Global`count==1,{first},{rest}]]")
        }
        ("MemoryConstrained" | "TimeConstrained", "body") => format!(
            "{name}[{}, {},99]",
            sequence(&f, v, "73"),
            if name == "TimeConstrained" {
                "3600"
            } else {
                "1000000000"
            }
        ),
        ("MemoryConstrained" | "TimeConstrained", "fallback") => {
            format!("{name}[73,0,{}]", sequence(&f, v, "73"))
        }
        ("Monitor", _) => {
            f.expected_prints
                .as_mut()
                .unwrap()
                .insert(0, "display".into());
            format!("Monitor[{},\"display\"]", sequence(&f, v, "73"))
        }
        ("EchoFunction", _) => {
            if v != Variant::Error {
                f.expected_prints.as_mut().unwrap().push("91".into());
            }
            format!("EchoFunction[Function[x,{}]][73]", sequence(&f, v, "91"))
        }
        ("Reap", "body") => {
            expected = "List[73, List[List[1, 2, 3, 4, 5]]]".into();
            format!(
                "Reap[{};73]",
                (1..=5)
                    .map(|i| step(&f, v, i, &format!("Sow[{i}]")))
                    .collect::<Vec<_>>()
                    .join(";")
            )
        }
        ("Reap", "combiner") => {
            let arms = (1..=5)
                .map(|i| format!("{i},{}", step(&f, v, i, "{tag,values}")))
                .collect::<Vec<_>>()
                .join(",");
            expected="List[73, List[List[1, List[10]], List[2, List[20]], List[3, List[30]], List[4, List[40]], List[5, List[50]]]]".into();
            format!(
                "Reap[(Sow[10,1];Sow[20,2];Sow[30,3];Sow[40,4];Sow[50,5];73),_,Function[{{tag,values}},Switch[tag,{arms}]]]"
            )
        }
        ("Sow", _) => {
            let patterns = (1..=5)
                .map(|i| format!("(_?Function[tag,{}])", step(&f, v, i, "True")))
                .collect::<Vec<_>>()
                .join(",");
            expected="List[73, List[List[List[73]], List[List[73]], List[List[73]], List[List[73]], List[List[73]]]]".into();
            format!("Reap[Sow[73,tag],{{{patterns}}}]")
        }
        _ => unreachable!("{entry}"),
    };
    finish(f, v, &expected)
}
