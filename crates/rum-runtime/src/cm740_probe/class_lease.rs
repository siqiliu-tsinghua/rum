//! Guest class lease. An absent lease returns before any row table lookup.
//! This registry is not the CM740-SEMANTIC structural domain.
use std::cell::{Cell, RefCell};
use std::sync::Arc;

mod class_rows;

thread_local! {
    static FLAG: Cell<u8> = const { Cell::new(0) };
    static LEASE: RefCell<Option<Lease>> = const { RefCell::new(None) };
    static KINDS: RefCell<Vec<(usize, &'static str)>> = const { RefCell::new(Vec::new()) };
}

struct Receipt {
    addr: usize,
    kind: &'static str,
    generation: u64,
    issuer: &'static str,
    owner: &'static str,
    occurrence: u64,
    slot: u64,
    committed: bool,
}

struct Lease {
    row: &'static str,
    commit: bool,
    group: &'static str,
    frame: &'static str,
    cursor: &'static str,
    source_sha256: &'static str,
    authorized_hits: u64,
    occurrence: u64,
    next_slot: u64,
    cursor_seen: Option<&'static str>,
    receipts: Vec<Receipt>,
    lines: Vec<String>,
}

#[inline(always)]
fn engaged() -> bool {
    super::ENABLED && FLAG.with(|flag| flag.get()) != 0
}

pub fn digest(bytes: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state = [
        0x6a09e667_u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut padded = bytes.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&((bytes.len() as u64) * 8).to_be_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut w = [0_u32; 64];
        for (i, b) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(b.try_into().unwrap());
        }
        for i in 16..64 {
            let a = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let b = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(a)
                .wrapping_add(w[i - 7])
                .wrapping_add(b);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let s = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let t = h
                .wrapping_add(s)
                .wrapping_add((e & f) ^ (!e & g))
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let u = s.wrapping_add((a & b) ^ (a & c) ^ (b & c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t);
            d = c;
            c = b;
            b = a;
            a = t.wrapping_add(u);
        }
        for (s, v) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *s = s.wrapping_add(v);
        }
    }
    state.iter().map(|word| format!("{word:08x}")).collect()
}

fn recipe(row: &str) -> Option<&'static class_rows::Recipe> {
    class_rows::RECIPES.iter().find(|item| item.row == row)
}

fn kind_matches(kind: &str, frame: &str) -> bool {
    !frame.is_empty() && (kind == frame || kind.ends_with(&format!("::{frame}")))
}

pub fn install(row: &'static str, commit: bool, program_bytes: &[u8]) -> bool {
    if !super::ENABLED || program_bytes.is_empty() || program_bytes == b"CM740-GUEST" {
        return false;
    }
    if std::env::var_os("RUM_CM740_OMIT").is_some() {
        return false;
    }
    let Some(item) = recipe(row) else {
        return false;
    };
    if item.source_sha256.len() != 64 || digest(program_bytes) != item.source_sha256 {
        return false;
    }
    LEASE.with(|slot| {
        *slot.borrow_mut() = Some(Lease {
            row,
            commit,
            group: item.group,
            frame: item.frame,
            cursor: item.cursor,
            source_sha256: item.source_sha256,
            authorized_hits: 0,
            occurrence: 0,
            next_slot: 1,
            cursor_seen: None,
            receipts: Vec::new(),
            lines: vec![format!(
                "CLASS_INSTALL row={row} commit={commit} group={} frame={} program={} source={}",
                item.group, item.frame, item.program, item.source_sha256
            )],
        });
    });
    KINDS.with(|slot| slot.borrow_mut().clear());
    FLAG.with(|flag| flag.set(1));
    true
}

pub fn clear() {
    FLAG.with(|flag| flag.set(0));
    LEASE.with(|slot| *slot.borrow_mut() = None);
    KINDS.with(|slot| slot.borrow_mut().clear());
}

pub fn snapshot_lines() -> Vec<String> {
    LEASE.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|lease| lease.lines.clone())
            .unwrap_or_default()
    })
}

pub fn authorized_hits() -> u64 {
    LEASE.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|lease| lease.authorized_hits)
            .unwrap_or(0)
    })
}

#[inline(always)]
pub fn authorize_existing_hit(row: &str) -> bool {
    if !engaged() {
        return true;
    }
    LEASE.with(|slot| {
        let mut lease = slot.borrow_mut();
        let Some(lease) = lease.as_mut() else {
            return true;
        };
        if lease.row != row {
            return true;
        }
        if !lease.commit {
            lease.lines.push(format!("CLASS_HIT_SUPPRESSED row={row}"));
            return false;
        }
        let proved = lease.receipts.iter().any(|item| item.committed);
        if proved {
            lease.authorized_hits = lease.authorized_hits.saturating_add(1);
            lease.lines.push(format!(
                "CLASS_HIT row={row} authorized={}",
                lease.authorized_hits
            ));
        }
        proved
    })
}

fn with_lease(mut body: impl FnMut(&mut Lease)) {
    if !engaged() {
        return;
    }
    LEASE.with(|slot| {
        if let Some(lease) = slot.borrow_mut().as_mut() {
            body(lease);
        }
    });
}

pub fn remember_concrete_kind(addr: usize, kind: &'static str) {
    if !engaged() || kind == "dyn ErasedFrame" {
        return;
    }
    KINDS.with(|slot| {
        let mut kinds = slot.borrow_mut();
        if let Some(found) = kinds.iter_mut().find(|(have, _)| *have == addr) {
            found.1 = kind;
        } else {
            kinds.push((addr, kind));
        }
    });
}

pub fn concrete_kind_for(addr: usize) -> &'static str {
    if !engaged() {
        return "dyn ErasedFrame";
    }
    KINDS.with(|slot| {
        slot.borrow()
            .iter()
            .find(|(have, _)| *have == addr)
            .map(|(_, kind)| *kind)
            .unwrap_or("dyn ErasedFrame")
    })
}

fn veto_erased(lease: &mut Lease, kind: &str) -> bool {
    if kind == "dyn ErasedFrame" {
        lease
            .lines
            .push(format!("CLASS_VETO row={} kind=dyn ErasedFrame", lease.row));
        true
    } else {
        false
    }
}

pub fn note_allocation(addr: usize, kind: &'static str, generation: u64, issuer: &'static str) {
    with_lease(|lease| {
        if veto_erased(lease, kind) {
            return;
        }
        let matches = kind_matches(kind, lease.frame);
        let cursor_ok = lease.group != "cursor" || lease.cursor_seen == Some(lease.cursor);
        if !matches || !cursor_ok || generation == 0 {
            lease.lines.push(format!(
                "CLASS_UNRELATED row={} kind={kind} generation={generation} issuer={issuer} cursor={:?}",
                lease.row, lease.cursor_seen
            ));
            return;
        }
        lease.occurrence = lease.occurrence.saturating_add(1);
        let slot = lease.next_slot;
        lease.next_slot = lease.next_slot.saturating_add(1);
        let committed = lease.commit;
        lease.lines.push(format!(
            "CLASS_RECEIPT row={} commit={committed} kind={kind} generation={generation} addr={addr} issuer={issuer} owner={} occurrence={} slot={slot}",
            lease.row, lease.row, lease.occurrence
        ));
        lease.receipts.push(Receipt {
            addr,
            kind,
            generation,
            issuer,
            owner: lease.row,
            occurrence: lease.occurrence,
            slot,
            committed,
        });
        if !committed {
            lease.lines.push(format!(
                "CLASS_WITHHELD row={} producer=observed receipt=suppressed kind={kind} generation={generation}",
                lease.row
            ));
        }
    });
}

fn note_return_kind(row: &'static str, generation: u64, kind: &'static str) {
    with_lease(|lease| {
        if lease.row != row || lease.group != "synchronous" {
            return;
        }
        lease.occurrence = lease.occurrence.saturating_add(1);
        let slot = lease.next_slot;
        lease.next_slot = lease.next_slot.saturating_add(1);
        let committed = lease.commit;
        lease.lines.push(format!(
            "CLASS_RECEIPT row={row} commit={committed} kind={kind} generation={generation} addr=0 issuer=Return owner={row} occurrence={} slot={slot}",
            lease.occurrence
        ));
        lease.receipts.push(Receipt {
            addr: 0,
            kind,
            generation,
            issuer: "Return",
            owner: lease.row,
            occurrence: lease.occurrence,
            slot,
            committed,
        });
        if committed {
            lease.lines.push(format!(
                "CLASS_CONSUME row={row} addr=0 issuer=Return owner={row} occurrence={} slot={slot}",
                lease.occurrence
            ));
        } else {
            lease.lines.push(format!(
                "CLASS_WITHHELD row={row} producer=observed receipt=suppressed kind={kind} generation={generation}"
            ));
        }
    });
}

pub fn note_input_return(row: &'static str, generation: u64) {
    note_return_kind(row, generation, "EvalError::InputRequired");
}

pub fn observe_producer_return(row: &'static str) {
    if wants_input_return(row) {
        note_return_kind(row, 0, "guarded-producer-return");
    }
}

#[inline(always)]
pub fn wants_input_return(row: &str) -> bool {
    if !engaged() {
        return false;
    }
    LEASE.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|lease| lease.row == row && lease.group == "synchronous")
    })
}

#[inline(always)]
pub fn wants_cursor() -> bool {
    if !engaged() {
        return false;
    }
    LEASE.with(|slot| {
        slot.borrow()
            .as_ref()
            .is_some_and(|lease| lease.group == "cursor")
    })
}

pub fn note_cursor_discriminant(discriminant: &'static str) {
    with_lease(|lease| {
        if lease.group != "cursor" {
            return;
        }
        lease.cursor_seen = Some(discriminant);
        let accept = discriminant == lease.cursor;
        lease.lines.push(format!(
            "CLASS_CURSOR row={} discriminant={discriminant} expect={} accept={accept}",
            lease.row, lease.cursor
        ));
    });
}

pub fn note_typed_constructor(kind: &'static str) {
    with_lease(|lease| {
        if veto_erased(lease, kind) {
            return;
        }
        if !kind_matches(kind, lease.frame) {
            return;
        }
        lease.lines.push(format!(
            "CLASS_CONSTRUCTOR row={} kind={kind} group={}",
            lease.row, lease.group
        ));
        if lease.group == "queued-success" {
            if lease.commit {
                lease.lines.push(format!(
                    "CLASS_QUEUED_PERMIT row={} kind={kind} paired=true",
                    lease.row
                ));
            } else {
                lease.lines.push(format!(
                    "CLASS_QUEUED_PERMIT row={} kind={kind} paired=false withheld=true",
                    lease.row
                ));
            }
        }
    });
}

pub fn note_queued_span(row: &'static str) {
    with_lease(|lease| {
        if lease.row == row && lease.group == "queued-success" {
            lease
                .lines
                .push(format!("CLASS_QUEUED_SPAN row={row} guard=enter"));
        }
    });
}

fn note_consumer(addr: usize, issuer: &'static str, kind: &'static str) {
    with_lease(|lease| {
        if veto_erased(lease, kind) {
            return;
        }
        let matched = lease
            .receipts
            .iter()
            .any(|item| item.committed && item.addr == addr && kind_matches(kind, lease.frame));
        if matched && lease.commit {
            lease.lines.push(format!(
                "CLASS_CONSUME row={} addr={addr} issuer={issuer} kind={kind} owner={}",
                lease.row, lease.row
            ));
        } else if !lease.commit && kind_matches(kind, lease.frame) {
            lease.lines.push(format!(
                "CLASS_WITHHELD_CONSUME row={} addr={addr} issuer={issuer} kind={kind}",
                lease.row
            ));
        }
    });
}

pub fn begin_row<F: 'static>(frame: &Arc<F>) {
    if !engaged() {
        return;
    }
    let addr = Arc::as_ptr(frame) as *const () as usize;
    note_consumer(addr, "ArcResume", std::any::type_name::<F>());
}

pub fn note_legacy_front<T: ?Sized>(frame: &Arc<T>) {
    if !engaged() {
        return;
    }
    let addr = Arc::as_ptr(frame) as *const () as usize;
    with_lease(|lease| {
        let matched = lease.receipts.iter().any(|item| item.addr == addr);
        if !matched {
            return;
        }
        if lease.commit {
            lease.lines.push(format!(
                "CLASS_CONSUME row={} addr={addr} issuer=LegacyDriver owner={}",
                lease.row, lease.row
            ));
        } else {
            lease.lines.push(format!(
                "CLASS_WITHHELD_CONSUME row={} addr={addr} issuer=LegacyDriver",
                lease.row
            ));
        }
    });
}

pub fn note_typed_consumer(addr: usize, kind: &'static str) {
    note_consumer(addr, "TypedAdapter", kind);
}
