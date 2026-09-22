use super::*;

#[test]
fn overflow_saturates_and_invalid_releases_stay_closed() {
    use Bytes::{Exact, Invalid};
    let max = usize::MAX;
    assert_eq!(Exact(max - 1).add(Exact(1)), Exact(max));
    assert_eq!(Exact(max).add(Exact(1)), Invalid);
    assert!(!Exact(max).exceeds(max));
    assert!(Exact(max).exceeds(max - 1));
    assert!(Invalid.exceeds(max));
    assert!(Invalid.exceeds(0));
    assert_eq!(Exact(max / 2 + 1).multiply(2), Invalid);
    assert_eq!(Exact(max).multiply(0), Exact(0));
    assert_eq!(Invalid.multiply(0), Invalid);
    assert_eq!(Exact(max).release(Exact(1)), Exact(max - 1));
    assert_eq!(Exact(3).release(Exact(4)), Invalid);
    assert_eq!(Exact(3).release(Invalid), Invalid);
    let overflowed = Exact(max).add(Exact(10));
    for release in [0, 1, 20, max] {
        assert_eq!(overflowed.release(Exact(release)), Invalid);
        assert_eq!(overflowed.release(Exact(release)).ceiling_bytes(), max);
    }
    assert_eq!(Invalid.release(Invalid), Invalid);
    assert_eq!(Invalid.add(Exact(0)), Invalid);
    assert_eq!(Exact(0).add(Invalid), Invalid);

    // The rejected saturated-subtraction rule provably understates this release.
    let true_remaining = (max as u128 + 10) - 20;
    assert!((max.saturating_sub(20) as u128) < true_remaining);
    assert!((overflowed.release(Exact(20)).ceiling_bytes() as u128) >= true_remaining);
}

#[test]
fn field_and_session_sums_preserve_overflow_information() {
    let breakdown = MemoryBreakdown {
        session_inline: Bytes::Exact(10),
        fields: vec![
            FieldContribution {
                field: "synthetic_a",
                bytes: Bytes::Exact(usize::MAX - 5),
            },
            FieldContribution {
                field: "synthetic_b",
                bytes: Bytes::Exact(8),
            },
        ],
    };
    assert_eq!(breakdown.total(), Bytes::Invalid);
    assert_eq!(breakdown.total().ceiling_bytes(), usize::MAX);
    assert_eq!(breakdown.total().release(Bytes::Exact(100)), Bytes::Invalid);
}

#[test]
fn synthetic_charge_release_sequences_never_understate_representable_remainders() {
    let values = [0, 1, 7, usize::MAX / 2, usize::MAX - 1, usize::MAX];
    for a in values {
        for b in values {
            let exact_charge = a as u128 + b as u128;
            let charge = Bytes::Exact(a).add(Bytes::Exact(b));
            for release in values {
                let remaining = charge.release(Bytes::Exact(release));
                match exact_charge.checked_sub(release as u128) {
                    Some(exact) if exact <= usize::MAX as u128 => {
                        assert!(remaining.ceiling_bytes() as u128 >= exact);
                    }
                    _ => assert_eq!(remaining, Bytes::Invalid),
                }
                if exact_charge <= usize::MAX as u128 && release as u128 <= exact_charge {
                    assert_eq!(
                        remaining,
                        Bytes::Exact((exact_charge - release as u128) as usize)
                    );
                }
            }
        }
    }
}
