use super::*;
use crate::runtime::debug_tools::resources::occurrence::*;

#[path = "session_memory_arithmetic.rs"]
mod arithmetic;
#[path = "session_memory_debug_owners.rs"]
mod debug_owners;
#[path = "session_memory_maintained.rs"]
mod maintained;
#[path = "session_memory_oracle.rs"]
mod oracle;
#[path = "session_memory_request_owners.rs"]
mod request_owners;
#[path = "session_memory_structures.rs"]
mod structures;

#[test]
fn session_memory_field_decision_inventory_is_sorted_and_exhaustive() {
    let session = Session::empty();
    assert_session_memory_decisions_are_exhaustive(&session);

    assert_eq!(SESSION_MEMORY_FIELD_DECISIONS.len(), 94);
    assert_eq!(
        SESSION_MEMORY_FIELD_DECISIONS
            .iter()
            .filter(|(_, decision)| matches!(decision, SessionMemoryFieldDecision::Accounted(_)))
            .count(),
        53,
    );
    let breakdown = session_breakdown(&session);
    assert_eq!(breakdown.fields.len(), SESSION_MEMORY_FIELD_DECISIONS.len());
    assert!(
        SESSION_MEMORY_FIELD_DECISIONS
            .windows(2)
            .all(|entries| entries[0].0 < entries[1].0),
        "the Session memory-field decision inventory must stay sorted"
    );

    for (field, decision) in SESSION_MEMORY_FIELD_DECISIONS {
        let detail = match decision {
            SessionMemoryFieldDecision::Accounted(term) => term,
            SessionMemoryFieldDecision::Exempt(reason) => {
                assert_eq!(breakdown.field(field), Bytes::Exact(0));
                assert!(
                    reason.starts_with("fixed-size"),
                    "{field} uses an unsupported exemption reason: {reason}"
                );
                reason
            }
        };
        assert!(!detail.is_empty(), "{field} must record a term or reason");
    }
}

fn assert_model(session: &Session) -> MemoryBreakdown {
    let peak = session.peak_memory_proxy_bytes.get();
    let first = session_breakdown(session);
    assert_eq!(session.peak_memory_proxy_bytes.get(), peak);
    assert_eq!(
        session_breakdown(session),
        first,
        "oracle must be observational"
    );
    assert_eq!(
        first.total(),
        Bytes::Exact(session.current_memory_proxy_bytes_for_test())
    );
    first
}

fn reserved(text: &str, capacity: usize) -> String {
    let mut value = String::with_capacity(capacity);
    value.push_str(text);
    value
}

#[test]
fn stability_blooms_are_fixed_size_inline_values() {
    assert_eq!(
        std::mem::size_of::<StabilityBloom>(),
        2 * std::mem::size_of::<u128>()
    );
}

#[test]
fn session_memory_walk_counts_declared_exception_storage() {
    let mut session = Session::empty();
    let before = session.current_memory_proxy_bytes_for_test();

    let mut name = String::with_capacity(97);
    name.push_str("WalkChild");
    let name_capacity = name.capacity();
    let mut parent = String::with_capacity(131);
    parent.push_str("WalkParent");
    let parent_capacity = parent.capacity();
    session.exception_types.insert(name, Some(parent));

    let after = session.current_memory_proxy_bytes_for_test();
    let expected_delta = std::mem::size_of::<(String, Option<String>)>()
        .saturating_add(name_capacity)
        .saturating_add(parent_capacity);

    assert_eq!(after.saturating_sub(before), expected_delta);
}

#[test]
fn byte_array_proxy_tracks_encoded_string_capacity_and_preserves_invalid_fallback() {
    let mut session = Session::empty();
    let byte_array_head = system_expr("ByteArray");
    let Expr::Atom(Atom::Symbol(head_symbol)) = &byte_array_head else {
        unreachable!("ByteArray head should be a symbol")
    };
    let head_bytes = std::mem::size_of::<Expr>()
        .saturating_add(head_symbol.context().len())
        .saturating_add(head_symbol.name().len());

    for (payload, decoded_len) in [("AQID", Some(3usize)), ("not-base64", None)] {
        let mut encoded = String::with_capacity(64);
        encoded.push_str(payload);
        let encoded_capacity = encoded.capacity();
        let mut args = Vec::with_capacity(1);
        args.push(Expr::string(encoded));
        assert_eq!(args.capacity(), 1);
        let byte_array = Expr::call(byte_array_head.clone(), args);

        let actual = session
            .eval_byte_count(&system_expr("ByteCount"), std::slice::from_ref(&byte_array))
            .expect("ByteCount should accept one expression");
        let expected = std::mem::size_of::<Expr>()
            .saturating_add(head_bytes)
            .saturating_add(std::mem::size_of::<Expr>())
            .saturating_add(encoded_capacity);

        assert_eq!(actual, Expr::integer(BigInt::from(expected as u64)));
        if let Some(decoded_len) = decoded_len {
            let decoded_substitution = std::mem::size_of::<Expr>()
                .saturating_add(head_bytes)
                .saturating_add(decoded_len);
            assert_ne!(
                actual,
                Expr::integer(BigInt::from(decoded_substitution as u64)),
                "valid ByteArray text must not be replaced by its decoded length"
            );
        }
    }
}
