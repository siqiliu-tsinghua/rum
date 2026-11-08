//! Exhaustive diagnostic text. No guest expression is constructed or evaluated.
use super::epoch;
use crate::runtime::{Session, profiling};
use std::sync::atomic::Ordering;

mod keys;
mod rows;

#[cfg(test)]
pub(in crate::runtime) mod test_hooks;

fn quoted(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

fn vector(out: &mut String, values: &[u64]) {
    out.push('{');
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push_str(&value.to_string());
    }
    out.push('}');
}

fn mode() -> String {
    let gates = [
        ("lexical", profiling::lexical_profile_enabled()),
        ("builtin-timing", profiling::builtin_timing_enabled()),
        ("rule-rhs-timing", profiling::rule_rhs_timing_enabled()),
        (
            "definition-rule-timing",
            profiling::definition_rule_timing_enabled(),
        ),
        ("cache", Session::snapshot_cache_profile_enabled()),
    ];
    let mut mode = String::new();
    for (name, enabled) in gates {
        if enabled {
            if !mode.is_empty() {
                mode.push('+');
            }
            mode.push_str(name);
        }
    }
    if mode.is_empty() {
        mode.push_str("none");
    }
    mode
}

pub(in crate::runtime) fn render(label: &str) -> String {
    let (epoch, failed) = epoch::publication_state();
    let rows = epoch
        .as_ref()
        .map(|epoch| rows::collect(epoch))
        .unwrap_or_default();
    #[cfg(test)]
    test_hooks::after_copy();
    // Read validity only after both copies and ownership validation. Reset
    // invalidates this captured Arc, never substitutes another epoch's rows.
    let valid = epoch.as_ref().is_some_and(|epoch| epoch.is_valid());
    let faults = epoch
        .as_ref()
        .map_or(0, |epoch| epoch.faults.load(Ordering::Relaxed));
    let id = epoch.as_ref().map(|epoch| epoch.id.0).or(failed);
    let mut out = String::from("{\"schema\",2,\"label\",");
    quoted(&mut out, label);
    out.push_str(",\"epoch\",");
    out.push_str(&id.map_or_else(|| "Null".into(), |id| id.to_string()));
    out.push_str(",\"validity\",");
    quoted(
        &mut out,
        if valid {
            "valid"
        } else if id.is_some() {
            "invalid"
        } else {
            "absent"
        },
    );
    if epoch.is_none() && failed.is_some() {
        out.push_str(",\"reason\",\"diagnostic-cap\"");
    }
    out.push_str(",\"mode\",");
    quoted(&mut out, &mode());
    out.push_str(",\"logical\",{");
    for (index, (key, row)) in rows.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push('{');
        quoted(&mut out, key);
        out.push_str(",\"vector\",");
        vector(&mut out, &row.vector);
        for (name, value) in ["open", "abandoned", "cancelled"]
            .into_iter()
            .zip(row.accounting)
        {
            out.push(',');
            quoted(&mut out, name);
            out.push(',');
            out.push_str(&value.to_string());
        }
        out.push('}');
    }
    out.push_str("},\"telemetry\",{\"faults\",");
    out.push_str(&faults.to_string());
    out.push_str(",\"rows\",{");
    for (index, (key, row)) in rows.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push('{');
        quoted(&mut out, key);
        out.push(',');
        vector(&mut out, &row.telemetry);
        out.push('}');
    }
    out.push_str(
        "}},\"durations\",{\"basis\",\"epoch-accrued+observing-thread-running\",\"rows\",{",
    );
    for (index, (key, row)) in rows.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push('{');
        quoted(&mut out, key);
        out.push(',');
        vector(&mut out, &row.durations);
        out.push('}');
    }
    out.push_str("}}}");
    out
}

pub(in crate::runtime) fn print(label: &str) {
    let line = render(label);
    eprintln!("rum-profile schema-v2 {line}");
}
