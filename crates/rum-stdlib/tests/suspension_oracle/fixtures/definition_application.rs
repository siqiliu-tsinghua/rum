//! Definition RHS values must survive every application continuation unchanged.
//! Contract: guide definitions/rules; queued input equals resumed input in all channels.
use super::super::descriptors::{Checks, accept};

fn equality(name: &str, template: &str, guard: bool, value: &str, expected: &str) {
    let source = template.replace(
        "$RHS",
        &format!("(AppendTo[Global`log,1];InputString[\"rhs\"];{value})"),
    );
    let mut pauses = Vec::new();
    if guard {
        pauses.push(("InputString", "guard", "1"));
    }
    pauses.push(("InputString", "rhs", "1"));
    let id = format!("s5.{name}");
    let fixture = super::make(
        &id,
        &source,
        &pauses,
        Some(expected),
        if guard { "List[0, 1]" } else { "List[1]" },
    );
    let result = accept(&fixture, Checks::ALL);
    super::super::support::faults::assert_all();
    let accepted = result.unwrap_or_else(|error| panic!("{id}: {error:?}"));
    println!(
        "S5 {id} strict equality PASS: {:?}; log={}",
        accepted.observation, accepted.log
    );
}

macro_rules! rhs_site {
    ($none:ident, $some:ident, $ordinary:ident, $source:literal, $guard:literal) => {
        #[test]
        fn $none() {
            equality(
                stringify!($none),
                $source,
                $guard,
                "System`InternalContinuationNone[]",
                "InternalContinuationNone[]",
            );
        }
        #[test]
        fn $some() {
            equality(
                stringify!($some),
                $source,
                $guard,
                "System`InternalContinuationSome[5]",
                "InternalContinuationSome[5]",
            );
        }
        #[test]
        fn $ordinary() {
            equality(stringify!($ordinary), $source, $guard, "7", "7");
        }
    };
}

// S4 producer: application/down_values.rs:221.
rhs_site!(
    down_exact_none,
    down_exact_some,
    down_exact_ordinary,
    r#"f[1]:=$RHS;f[1]"#,
    false
);

// S4 producer: application/down_values.rs:322.
rhs_site!(
    down_pattern_none,
    down_pattern_some,
    down_pattern_ordinary,
    r#"f[x_]:=$RHS;f[1]"#,
    false
);

// S4 producer: application/down_values.rs:438.
rhs_site!(
    down_resumed_none,
    down_resumed_some,
    down_resumed_ordinary,
    r#"f[x_ /; (AppendTo[Global`log,0];InputString["guard"];True)]:=$RHS;f[1]"#,
    true
);

// S4 producer: application/sub_values.rs:230.
rhs_site!(
    sub_exact_none,
    sub_exact_some,
    sub_exact_ordinary,
    r#"k[1][2]:=$RHS;k[1][2]"#,
    false
);

// S4 producer: application/sub_values.rs:230.
rhs_site!(
    sub_pattern_none,
    sub_pattern_some,
    sub_pattern_ordinary,
    r#"k[1][x_]:=$RHS;k[1][2]"#,
    false
);

// S4 producer: application/sub_values.rs:349.
rhs_site!(
    sub_resumed_none,
    sub_resumed_some,
    sub_resumed_ordinary,
    r#"k[1][x_ /; (AppendTo[Global`log,0];InputString["guard"];True)]:=$RHS;k[1][2]"#,
    true
);

// S4 producer: application/up_values.rs:183.
rhs_site!(
    up_exact_none,
    up_exact_some,
    up_exact_ordinary,
    r#"g /: h[g,1]:=$RHS;h[g,1]"#,
    false
);

// S4 producer: application/up_values.rs:183.
rhs_site!(
    up_pattern_none,
    up_pattern_some,
    up_pattern_ordinary,
    r#"g /: h[g,x_]:=$RHS;h[g,1]"#,
    false
);

// S4 producer: application/up_values.rs:103.
rhs_site!(
    up_sync_guard_none,
    up_sync_guard_some,
    up_sync_guard_ordinary,
    r#"g /: h[g,x_]:=$RHS/;True;h[g,1]"#,
    false
);

// S4 producer: application/up_values.rs:299.
rhs_site!(
    up_resumed_none,
    up_resumed_some,
    up_resumed_ordinary,
    r#"g /: h[x_ /; (AppendTo[Global`log,0];InputString["guard"];True),g]:=$RHS;h[1,g]"#,
    true
);

// S4 producer: application/default_values.rs:83.
rhs_site!(
    default_first_plain_none,
    default_first_plain_some,
    default_first_plain_ordinary,
    r#"Default[d]:=$RHS;Default[d]"#,
    false
);

// S4 producer: application/default_values.rs:94.
rhs_site!(
    default_first_guarded_table_none,
    default_first_guarded_table_some,
    default_first_guarded_table_ordinary,
    r#"Default[other]:=0/;True;Default[d]:=$RHS;Default[d]"#,
    false
);

// S4 producer: application/default_values.rs:122.
rhs_site!(
    default_cursor_match_none,
    default_cursor_match_some,
    default_cursor_match_ordinary,
    r#"Default[d]:=$RHS/;True;Default[d]"#,
    false
);

// S4 producer: application/default_values.rs:141.
rhs_site!(
    default_cursor_next_none,
    default_cursor_next_some,
    default_cursor_next_ordinary,
    r#"Default[d]:=0/;False;Default[d,1]:=$RHS;Default[d]"#,
    false
);

// S4 producer: application/default_values.rs:210.
rhs_site!(
    default_resumed_match_none,
    default_resumed_match_some,
    default_resumed_match_ordinary,
    r#"Default[d]:=$RHS/;(AppendTo[Global`log,0];InputString["guard"];True);Default[d]"#,
    true
);

// S4 producer: application/default_values.rs:229.
rhs_site!(
    default_resumed_next_none,
    default_resumed_next_some,
    default_resumed_next_ordinary,
    r#"Default[d]:=0/;(AppendTo[Global`log,0];InputString["guard"];False);Default[d,1]:=$RHS;Default[d]"#,
    true
);
