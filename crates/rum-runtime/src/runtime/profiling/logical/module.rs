//! Explicit Module continuation aliases. No key-equality reopening.
use super::*;

pub(in crate::runtime) fn open_resume_alias() {
    let Some(epoch) = executing_epoch() else {
        return;
    };
    let Some(path) = segments::ancestry(&epoch) else {
        epoch.fault();
        return;
    };
    // This is the context captured for Frame2, whose exact innermost record
    // is the loop. Validate that identity; do not search by phase equality.
    if !matches!(
        path.key,
        AggregateKey::Runtime {
            phase: "module.resume_initializers",
            ..
        }
    ) || !path.is_open()
    {
        epoch.fault();
        return;
    }
    let Some(storage) = epoch.storage(std::mem::size_of::<PhaseEntry>() + 16) else {
        return;
    };
    #[cfg(test)]
    eprintln!(
        "MODULE_ALIAS record={} key=module.resume_initializers",
        path.record.0
    );
    PHASE_LIFO.with_borrow_mut(|stack| {
        stack.push(PhaseEntry {
            key: PhaseKey::ModuleInitializers,
            call: None,
            _storage: Some(storage),
        })
    });
}

pub(super) fn initial_state(key: AggregateKey) -> super::totals::State {
    match key {
        AggregateKey::Runtime {
            phase: "module.eval.total",
            source,
        } => super::totals::State {
            module_scalar_alias: true,
            module_source_alias: source,
            ..Default::default()
        },
        _ => Default::default(),
    }
}
