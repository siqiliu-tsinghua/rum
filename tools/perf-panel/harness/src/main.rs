mod g15;
mod g15c1;
mod lhs;
mod s5;
mod s6;
mod s7;
mod s8;
mod s9;
use rum_host::{HostEvalOutcome, HostSession, HostTyped};

#[cfg(feature = "alloc-count")]
#[path = "../../../../crates/rum-cli/src/alloc_count.rs"]
mod alloc_count;
#[cfg(not(feature = "alloc-count"))]
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    #[cfg(feature = "alloc-count")]
    alloc_count::init();
    let args: Vec<String> = std::env::args().collect();
    let label = args.get(1).expect("workload");
    let n: usize = args.get(2).expect("iteration count").parse().unwrap();
    assert!(n > 0);
    if lhs::run(label, n, args.get(3).map(String::as_str)) {
        #[cfg(feature = "alloc-count")]
        if alloc_count::enabled() {
            let p = alloc_count::snapshot();
            eprintln!(
                "rum-profile alloc count={} bytes_total={} peak_live_bytes={}",
                p.count, p.bytes_total, p.peak_live_bytes
            );
        }
        return;
    }
    let owned = g15c1::workload(label)
        .or_else(|| g15::workload(label))
        .or_else(|| s9::workload(label))
        .or_else(|| s8::workload(label))
        .or_else(|| s7::workload(label))
        .or_else(|| s6::workload(label))
        .or_else(|| s5::workload(label));
    let source = match label.as_str() {
        "table" => format!("Table[InputString[\"resume\"],{{{n}}}]"),
        "do-chain" => format!("Do[InputString[\"resume\"],{{i,{n}}}]"),
        "call-repeat" => format!("f[{}]", vec!["InputString[\"resume\"]"; n].join(",")),
        "cleanup" => {
            format!("marker=0; WithCleanup[Do[InputString[\"resume\"],{{{n}}}],marker=17];marker")
        }
        "fold" => format!("Fold[Function[{{a,x}},InputString[\"resume\"]],0,Range[{n}]]"),
        "argument-twice" => format!(
            "hv={{{{1,2}},{{3,4}}}};Do[hv[[(InputString[\"first\"];1),(InputString[\"later\"];2)]]+=10,{{{n}}}];hv"
        ),
        "rewrite-exprs" => format!("Table[Replace[1,(InputString[\"rules\"];1->2)],{{{n}}}]"),
        "rewrite-compiled" => format!("Table[ReplaceAll[1,(InputString[\"rules\"];1->2)],{{{n}}}]"),
        "cases-spec" => format!("Table[Cases[{{2}},Rule[(InputString[\"spec\"];2),9]],{{{n}}}]"),
        "cases-traversal" => {
            format!("Table[Cases[{{1}},1,(InputString[\"level\"];{{1}})],{{{n}}}]")
        }
        "position-traversal" => {
            format!("Table[Position[{{1}},1,(InputString[\"level\"];{{1}})],{{{n}}}]")
        }
        "count-traversal" => {
            format!("Table[Count[{{1}},1,(InputString[\"level\"];{{1}})],{{{n}}}]")
        }
        "cases-shared" => format!(
            "Table[Cases[{{2}},Rule[(InputString[\"spec\"];2),9],(InputString[\"level\"];{{1}})],{{{n}}}]"
        ),
        _ => {
            let w = owned.as_ref().expect("unknown workload");
            format!(
                "{}Table[(s5Tick=s5Iteration;{}),{{s5Iteration,1,{n}}}]",
                w.setup, w.body
            )
        }
    };
    let expected_resumes = if let Some(w) = &owned {
        w.prompts.len() * n
    } else if matches!(label.as_str(), "argument-twice" | "cases-shared") {
        2 * n
    } else {
        n
    };
    let expected_prompts: &[&str] = match label.as_str() {
        "argument-twice" => &["first", "later"],
        "rewrite-exprs" | "rewrite-compiled" => &["rules"],
        "cases-spec" => &["spec"],
        "cases-traversal" | "position-traversal" | "count-traversal" => &["level"],
        "cases-shared" => &["spec", "level"],
        _ => owned.as_ref().map_or(&["resume"], |w| w.prompts),
    };
    let mut session = HostSession::new();
    let mut outcome = session.eval(source.as_str());
    let mut resumes = 0;
    loop {
        match outcome {
            HostEvalOutcome::InputRequired(suspension) => {
                assert_eq!(
                    suspension.request.prompt,
                    expected_prompts[resumes % expected_prompts.len()]
                );
                assert!(suspension.messages.is_empty());
                assert!(suspension.prints.is_empty());
                resumes += 1;
                assert!(resumes <= expected_resumes);
                outcome = session.resume_with_input("1");
            }
            HostEvalOutcome::Success(success) => {
                assert_eq!(resumes, expected_resumes);
                assert!(success.termination.is_none());
                assert!(success.messages.is_empty());
                assert!(success.prints.is_empty());
                match label.as_str() {
                    "table" => assert_eq!(
                        success.value.typed,
                        HostTyped::List(vec![HostTyped::Str("1".into()); n])
                    ),
                    "do-chain" => assert_eq!(success.value.text, "Null"),
                    "call-repeat" => assert_eq!(
                        success.value.text,
                        format!("f[{}]", vec!["\"1\""; n].join(", "))
                    ),
                    "cleanup" => assert_eq!(success.value.typed, HostTyped::Integer(17)),
                    "fold" => assert_eq!(success.value.typed, HostTyped::Str("1".into())),
                    "argument-twice" => assert_eq!(
                        success.value.text,
                        format!("List[List[1, {}], List[3, 4]]", 2 + 10 * n)
                    ),
                    "rewrite-exprs" | "rewrite-compiled" => assert_eq!(
                        success.value.text,
                        format!("List[{}]", vec!["2"; n].join(", "))
                    ),
                    "cases-spec" | "cases-shared" => assert_eq!(
                        success.value.text,
                        format!("List[{}]", vec!["List[9]"; n].join(", "))
                    ),
                    "cases-traversal" => assert_eq!(
                        success.value.text,
                        format!("List[{}]", vec!["List[1]"; n].join(", "))
                    ),
                    "position-traversal" => assert_eq!(
                        success.value.text,
                        format!("List[{}]", vec!["List[List[1]]"; n].join(", "))
                    ),
                    "count-traversal" => assert_eq!(
                        success.value.text,
                        format!("List[{}]", vec!["1"; n].join(", "))
                    ),
                    _ => {
                        let w = owned.as_ref().unwrap();
                        assert_eq!(
                            success.value.text,
                            format!("List[{}]", vec![w.item; n].join(", "))
                        );
                    }
                }
                assert!(session.pending_input_request().is_none());
                println!("PASS {label} resumes={resumes}");
                break;
            }
            HostEvalOutcome::Failure(failure) => panic!("{failure:?}"),
        }
    }
    #[cfg(feature = "alloc-count")]
    if alloc_count::enabled() {
        let p = alloc_count::snapshot();
        eprintln!(
            "rum-profile alloc count={} bytes_total={} peak_live_bytes={}",
            p.count, p.bytes_total, p.peak_live_bytes
        );
    }
}
