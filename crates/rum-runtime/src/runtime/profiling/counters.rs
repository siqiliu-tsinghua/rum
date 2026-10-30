use super::*;

pub(super) fn record_timing(
    count: &AtomicU64,
    total_nanos: &AtomicU64,
    name: &str,
    start: Option<Instant>,
) {
    let Some(start) = start else {
        return;
    };

    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    let elapsed_nanos = start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
    let count_value = count.fetch_add(1, Ordering::Relaxed) + 1;
    let total_value = total_nanos
        .fetch_add(elapsed_nanos, Ordering::Relaxed)
        .saturating_add(elapsed_nanos);
    if lexical_profile_incremental_enabled() && count_value.is_power_of_two() {
        let avg = total_value / count_value;
        eprintln!(
            "rum-profile lexical {name}_count={count_value} {name}_total_ns={total_value} {name}_avg_ns={avg}"
        );
    }
}

pub(super) fn record_profile_lexical_timing(
    total_count: &AtomicU64,
    timing_count: &AtomicU64,
    total_nanos: &AtomicU64,
    name: &str,
    start: Option<Instant>,
) {
    let Some(start) = start else {
        return;
    };

    let elapsed_nanos = elapsed_nanos_since(start);
    record_profile_lexical_timing_elapsed(
        total_count,
        timing_count,
        total_nanos,
        name,
        Some(elapsed_nanos),
    );
}

pub(super) fn record_profile_lexical_timing_elapsed(
    total_count: &AtomicU64,
    timing_count: &AtomicU64,
    total_nanos: &AtomicU64,
    name: &str,
    elapsed_nanos: Option<u64>,
) {
    let Some(elapsed_nanos) = elapsed_nanos else {
        return;
    };

    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    let timing_value = timing_count.fetch_add(1, Ordering::Relaxed) + 1;
    let total_value = total_nanos
        .fetch_add(elapsed_nanos, Ordering::Relaxed)
        .saturating_add(elapsed_nanos);
    if lexical_profile_incremental_enabled() && timing_value.is_power_of_two() {
        let total_calls = total_count.load(Ordering::Relaxed);
        let avg = total_value / timing_value;
        eprintln!(
            "rum-profile lexical {name}_timed={timing_value} {name}_calls={total_calls} {name}_total_ns={total_value} {name}_avg_ns={avg}"
        );
    }
}

pub(super) fn record_counter(counter: &AtomicU64, name: &str) {
    if !lexical_profile_enabled() {
        return;
    }

    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    let value = counter.fetch_add(1, Ordering::Relaxed) + 1;
    if lexical_profile_incremental_enabled() && value.is_power_of_two() {
        eprintln!("rum-profile lexical {name}={value}");
    }
}

pub(super) fn record_add(counter: &AtomicU64, name: &str, amount: u64) {
    if !lexical_profile_enabled() {
        return;
    }

    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    let previous = counter.fetch_add(amount, Ordering::Relaxed);
    let value = previous.saturating_add(amount);
    if lexical_profile_incremental_enabled() && crossed_power_of_two(previous, value) {
        eprintln!("rum-profile lexical {name}={value}");
    }
}

pub(super) fn record_max(counter: &AtomicU64, name: &str, value: u64) {
    #[cfg(any(
        test,
        feature = "test-builtin-memory-growth",
        feature = "test-continuation-slots"
    ))]
    let Some(_admitted) = crate::runtime::profiling::lease::admit() else {
        return;
    };
    let mut previous = counter.load(Ordering::Relaxed);
    while value > previous {
        match counter.compare_exchange_weak(previous, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => {
                if lexical_profile_incremental_enabled() {
                    eprintln!("rum-profile lexical {name}={value}");
                }
                return;
            }
            Err(actual) => previous = actual,
        }
    }
}

pub(super) fn load(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Relaxed)
}

pub(super) fn average(total: &AtomicU64, count: &AtomicU64) -> u64 {
    let count = load(count);
    if count == 0 { 0 } else { load(total) / count }
}

fn crossed_power_of_two(previous: u64, current: u64) -> bool {
    if current == 0 {
        return false;
    }

    let previous_next = if previous == 0 {
        0
    } else {
        previous.next_power_of_two()
    };
    current.is_power_of_two() || current.next_power_of_two() != previous_next
}

pub(super) fn elapsed_nanos_since(start: Instant) -> u64 {
    start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}
