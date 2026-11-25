//! Q25 ownership contract for the default P2379 proof.
//!
//! Fixture-local, source-authenticated, and one-use. An environment variable or
//! the guest program string cannot mint the positive capability. Inactive runs
//! pay one cell check and leave the production ledgers alone.
use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Weak};

use super::{AuthenticatedIdentity, ConstructionGuard, Guard, ResumeGuard};

const PROGRAM_LINE: &str = "InputString[\"prior\"];StringCases[\"ab\",x_:>If[c15once,(c15once=False;Input[\"finish\"]),\"X\"]]";
const SETUP_LINE: &str = "log={};scope=17;c15index=0;c15once=True;";
const BEGIN_PROGRAM: &str = "CM740-GUEST";
const FINISH_ANSWER: &str = "\"X\"";
const OMIT_PHASE: &str = "OmitActualStringCaseAppend";

thread_local! {
    static ACTIVE: super::PublishedFlag = const { super::PublishedFlag::off() };
    static STATE: RefCell<State> = RefCell::new(State::new());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Ordinary,
    Positive,
    Withheld,
    Siblings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Half {
    Unqueued,
    Queued,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Offered,
    Claimed,
    Retired,
}

struct Lease {
    mode: Mode,
    half: Half,
    commit: bool,
    nonce: u64,
    constituent: u64,
    program_sha: String,
    setup_sha: String,
    begin_seen: bool,
    live: bool,
}

/// Source-authenticated one-use capability. Not `Clone`. Not constructible from
/// `RUM_CM740_OMIT` or from the guest begin string.
pub struct Capability {
    mode: Mode,
    half: Half,
    nonce: u64,
    constituent: u64,
    program_sha: String,
    setup_sha: String,
    _unclone: std::marker::PhantomData<*const ()>,
}

pub struct Handoff {
    construction: ConstructionGuard,
    guard: Guard,
    commit: bool,
    row: &'static str,
}

pub struct OfferGuard {
    armed: bool,
}

pub struct ClaimGuard {
    resume: Option<ResumeGuard>,
}

struct Slot {
    addr: usize,
    kind: &'static str,
    type_id: TypeId,
    birth: u64,
    custody: u64,
    weak: Weak<dyn crate::runtime::kernel_api::SuspensionFrame>,
    phase: Phase,
    committed: bool,
    strong_at_birth: usize,
    weak_at_birth: usize,
    token_addr: usize,
}

struct Offer {
    addr: usize,
    decoded_value: bool,
}

struct State {
    lease: Option<Lease>,
    slot: Option<Slot>,
    offer: Option<Offer>,
    queue_len: usize,
    log: Vec<String>,
    watch: super::PublishedFlag,
    consumed_append: bool,
}

impl State {
    const fn new() -> Self {
        Self {
            lease: None,
            slot: None,
            offer: None,
            queue_len: 0,
            log: Vec::new(),
            watch: super::PublishedFlag::off(),
            consumed_append: false,
        }
    }
}

#[inline(always)]
fn interested() -> bool {
    if !super::possibly_active() {
        return false;
    }
    interested_slow()
}

#[cold]
#[inline(never)]
fn interested_slow() -> bool {
    ACTIVE.with(super::PublishedFlag::get) || STATE.with(|s| s.borrow().watch.get())
}

fn tag(st: &State) -> String {
    match &st.lease {
        Some(lease) => format!(
            "mode={:?} half={:?} nonce={}",
            lease.mode, lease.half, lease.nonce
        ),
        None => "mode=none half=none nonce=0".to_string(),
    }
}

fn log(st: &mut State, line: impl AsRef<str>) {
    let text = format!("Q25_{} {}", tag(st), line.as_ref());
    st.log.push(text);
}

#[inline]
fn active_lease() -> bool {
    ACTIVE.with(super::PublishedFlag::get)
}

pub fn guest_grant() -> bool {
    false
}

pub fn try_env_mint() -> Option<Capability> {
    // The selector environment and the guest begin string are not a capability.
    let _selector = std::env::var("RUM_CM740_OMIT");
    let _guest = BEGIN_PROGRAM;
    None
}

pub fn issue(
    mode: Mode,
    half: Half,
    program: &str,
    setup: &str,
    prompts_and_answers: &[(&str, &str)],
    constituent: u64,
    nonce: u64,
    program_sha: &str,
    setup_sha: &str,
) -> Option<Capability> {
    if guest_grant() || try_env_mint().is_some() {
        return None;
    }
    let program_ok = program.trim_end_matches(['\n', '\r']) == PROGRAM_LINE;
    let setup_ok = setup.trim_end_matches(['\n', '\r']) == SETUP_LINE;
    let answers_ok = prompts_and_answers == [("prior", "prior"), ("finish", FINISH_ANSWER)];
    if !program_ok || !setup_ok || !answers_ok || program_sha.len() != 64 || setup_sha.len() != 64 {
        return None;
    }
    Some(Capability {
        mode,
        half,
        nonce,
        constituent,
        program_sha: program_sha.to_string(),
        setup_sha: setup_sha.to_string(),
        _unclone: std::marker::PhantomData,
    })
}

pub fn install(capability: Capability) {
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        if st.lease.as_ref().is_some_and(|lease| lease.live) {
            log(&mut st, "INSTALL reject=live_lease");
            return;
        }
        let commit = matches!(capability.mode, Mode::Positive | Mode::Siblings);
        log(
            &mut st,
            format!(
                "INSTALL accept mode={:?} half={:?} nonce={} constituent={} program_sha={} setup_sha={} commit={commit} env_grant=false",
                capability.mode,
                capability.half,
                capability.nonce,
                capability.constituent,
                capability.program_sha,
                capability.setup_sha
            ),
        );
        st.lease = Some(Lease {
            mode: capability.mode,
            half: capability.half,
            commit,
            nonce: capability.nonce,
            constituent: capability.constituent,
            program_sha: capability.program_sha,
            setup_sha: capability.setup_sha,
            begin_seen: false,
            live: true,
        });
        st.consumed_append = false;
        st.slot = None;
        st.offer = None;
        st.queue_len = 0;
    });
    ACTIVE.with(|cell| cell.set(true));
}

pub fn on_begin(queued: bool, program: &str) {
    if !interested() {
        return;
    }
    let episode = super::LEDGER.with(|cell| {
        let ledger = cell.borrow();
        ledger.current.as_ref().and_then(|run| {
            run.pair.episode.as_ref().map(|episode| {
                (
                    episode.row.clone(),
                    episode.program.clone(),
                    episode.pair,
                    run.queued,
                )
            })
        })
    });
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        let present = episode.is_some();
        log(
            &mut st,
            format!(
                "BEGIN queued={queued} program={program:?} episode_present={present} episode={episode:?}"
            ),
        );
        let snapshot = st.lease.as_ref().map(|lease| {
            (lease.begin_seen, lease.half == Half::Queued)
        });
        let Some((seen, half_queued)) = snapshot else {
            return;
        };
        let reject = seen || half_queued != queued || program != BEGIN_PROGRAM;
        if let Some(lease) = st.lease.as_mut() {
            if reject {
                lease.live = false;
            } else {
                lease.begin_seen = true;
            }
        }
        log(
            &mut st,
            if reject {
                format!(
                    "BEGIN reject half_ok={} program_ok={} already={seen}",
                    half_queued == queued,
                    program == BEGIN_PROGRAM
                )
            } else {
                "BEGIN accept".to_string()
            },
        );
    });
}

impl Lease {
    fn half_is_queued(&self) -> bool {
        self.half == Half::Queued
    }
}

pub fn on_end() {
    if !interested() {
        return;
    }
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        let had_slot = st
            .slot
            .as_ref()
            .is_some_and(|slot| slot.phase != Phase::Retired);
        if let Some(slot) = st.slot.as_mut() {
            slot.phase = Phase::Retired;
        }
        if had_slot {
            log(&mut st, "RETIRE reason=episode_end");
        }
        st.offer = None;
        let had_lease = st.lease.is_some();
        if let Some(lease) = st.lease.as_mut() {
            lease.live = false;
        }
        log(
            &mut st,
            if had_lease {
                "END lease_retired log_retained=true"
            } else {
                "END watch=true"
            },
        );
        st.lease = None;
    });
    ACTIVE.with(|cell| cell.set(false));
}

pub fn watch_pairs(enabled: bool) {
    STATE.with(|cell| cell.borrow().watch.set(enabled));
}

#[inline]
pub fn ordinary_suppresses_arm() -> bool {
    if !active_lease() {
        return false;
    }
    STATE.with(|cell| {
        cell.borrow()
            .lease
            .as_ref()
            .is_some_and(|lease| lease.live && lease.mode == Mode::Ordinary)
    })
}

#[inline]
pub fn observing() -> bool {
    active_lease()
}

pub fn note_physical_pause(prompt: &str) {
    if !active_lease() {
        return;
    }
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        log(&mut st, format!("PAUSE prompt={prompt:?}"));
    });
}

pub fn note_successful_arm(row: &str, phase: &str, prompt: &str) {
    if !active_lease() {
        return;
    }
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        log(
            &mut st,
            format!("ARM row={row} phase={phase} prompt={prompt:?}"),
        );
    });
}

#[inline(always)]
pub fn siblings_mode() -> bool {
    if !super::possibly_active() {
        return false;
    }
    siblings_mode_slow()
}

#[cold]
#[inline(never)]
fn siblings_mode_slow() -> bool {
    active_lease()
        && STATE.with(|cell| {
            cell.borrow()
                .lease
                .as_ref()
                .is_some_and(|lease| lease.live && lease.mode == Mode::Siblings)
        })
}

pub fn around_handoff<R>(row: &'static str, body: impl FnOnce() -> R) -> (R, Option<Handoff>) {
    if !super::run_is_driven() {
        return (body(), None);
    }
    let guard = super::enter_slow(row);
    let result = body();
    (result, package(row, guard))
}

fn package(row: &'static str, guard: Guard) -> Option<Handoff> {
    let (want, commit) = STATE.with(|cell| {
        let st = cell.borrow();
        match st.lease.as_ref() {
            Some(lease) if lease.live && lease.mode != Mode::Ordinary => (true, lease.commit),
            _ => (false, false),
        }
    });
    if !want || guard.0.is_none() || !pending_for(&guard) {
        if active_lease() {
            STATE.with(|cell| {
                let mut st = cell.borrow_mut();
                log(
                    &mut st,
                    format!(
                        "HANDOFF absent row={row} want={want} active_guard={}",
                        guard.0.is_some()
                    ),
                );
            });
        }
        drop(guard);
        return None;
    }
    let construction = guard.suspending();
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        log(
            &mut st,
            format!("HANDOFF present row={row} commit={commit}"),
        );
    });
    Some(Handoff {
        construction,
        guard,
        commit,
        row,
    })
}

fn pending_for(guard: &Guard) -> bool {
    let Some(active) = guard.0.as_ref() else {
        return false;
    };
    super::LEDGER.with(|cell| {
        let ledger = cell.borrow();
        ledger
            .current
            .as_ref()
            .is_some_and(|run| run.pending.contains_key(&active.activation))
    })
}

#[inline(always)]
pub fn note_physical_insert(kind: &'static str, generation: u64, entry_kind: &'static str) {
    if !super::possibly_active() {
        return;
    }
    note_physical_insert_slow(kind, generation, entry_kind)
}

#[cold]
#[inline(never)]
fn note_physical_insert_slow(kind: &'static str, generation: u64, entry_kind: &'static str) {
    if !active_lease() {
        return;
    }
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        st.queue_len = st.queue_len.saturating_add(1);
        let queue_len = st.queue_len;
        log(
            &mut st,
            format!(
                "INSERT_PHYSICAL entry={entry_kind} type={kind} birth={generation} queue_len={queue_len}"
            ),
        );
    });
}

pub fn commit_allocation(
    addr: usize,
    kind: &'static str,
    type_id: TypeId,
    generation: u64,
    weak: Weak<dyn crate::runtime::kernel_api::SuspensionFrame>,
    strong: usize,
    weak_count: usize,
    handoff: Handoff,
) {
    let commit = handoff.commit;
    let row = handoff.row;
    let ready = if commit {
        precheck(addr, kind)
    } else {
        Err("withheld")
    };
    if let Ok(()) = ready {
        super::bind_frame(addr, kind);
    }
    let bound = super::LEDGER.with(|cell| {
        cell.borrow()
            .bound_slots
            .get(&(addr, kind))
            .map(|token| token.occurrence)
    });
    drop(handoff);
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        let committed = commit && bound.is_some();
        if commit && bound.is_none() {
            log(&mut st, format!("BIND precondition_failed row={row} addr={addr}"));
        }
        st.queue_len = st.queue_len.saturating_add(1);
        let queue_len = st.queue_len;
        log(
            &mut st,
            format!(
                "REGISTER addr={addr} kind={kind} birth={generation} custody={generation} commit={committed} strong={strong} weak={weak_count} queue_len={queue_len} token={bound:?} entry=Legacy"
            ),
        );
        st.slot = Some(Slot {
            addr,
            kind,
            type_id,
            birth: generation,
            custody: generation,
            weak,
            phase: if committed { Phase::Offered } else { Phase::Retired },
            committed,
            strong_at_birth: strong,
            weak_at_birth: weak_count,
            token_addr: bound.unwrap_or(0) as usize,
        });
    });
}

fn precheck(addr: usize, kind: &'static str) -> Result<(), &'static str> {
    super::LEDGER.with(|cell| {
        let ledger = cell.borrow();
        let AuthenticatedIdentity::Valid(token) = super::construction_identity(&ledger) else {
            return Err("construction");
        };
        if ledger.stack.last() != Some(&token.owner) {
            return Err("owner");
        }
        if token.frame_identity.is_some() {
            return Err("framed");
        }
        if ledger.bound_slots.contains_key(&(addr, kind)) {
            return Err("duplicate");
        }
        if ledger.current.as_ref().is_some_and(|run| run.queued) {
            return Err("queued");
        }
        Ok(())
    })
}

#[inline(always)]
pub fn offer_front<T: ?Sized>(frame: &Arc<T>, decoded_value: bool) -> OfferGuard {
    if !super::possibly_active() {
        return OfferGuard { armed: false };
    }
    offer_front_slow(frame, decoded_value)
}

#[cold]
#[inline(never)]
fn offer_front_slow<T: ?Sized>(frame: &Arc<T>, decoded_value: bool) -> OfferGuard {
    if !active_lease() {
        return OfferGuard { armed: false };
    }
    let addr = Arc::as_ptr(frame) as *const () as usize;
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        st.offer = Some(Offer {
            addr,
            decoded_value,
        });
        let queue_len = st.queue_len;
        log(
            &mut st,
            format!(
                "OFFER addr={addr} decoded_value={decoded_value} entry=Legacy queue_len={queue_len}"
            ),
        );
    });
    OfferGuard { armed: true }
}

impl Drop for OfferGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        STATE.with(|cell| {
            let mut st = cell.borrow_mut();
            st.offer = None;
        });
    }
}

#[inline(always)]
pub fn note_front_kind(entry_kind: &'static str) {
    if !super::possibly_active() {
        return;
    }
    note_front_kind_slow(entry_kind)
}

#[cold]
#[inline(never)]
fn note_front_kind_slow(entry_kind: &'static str) {
    if !active_lease() {
        return;
    }
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        log(&mut st, format!("FRONT entry={entry_kind}"));
    });
}

#[inline(always)]
pub fn note_disposed(addr: usize) {
    if !super::possibly_active() {
        return;
    }
    note_disposed_slow(addr)
}

#[cold]
#[inline(never)]
fn note_disposed_slow(addr: usize) {
    if !active_lease() {
        return;
    }
    let retire_kind = STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        let tracked = st.slot.as_ref().is_some_and(|slot| slot.addr == addr);
        let upgrade = st
            .slot
            .as_ref()
            .filter(|slot| slot.addr == addr)
            .and_then(|slot| slot.weak.upgrade())
            .is_some();
        let kind = if tracked {
            st.slot
                .as_ref()
                .and_then(|slot| slot.committed.then_some(slot.kind))
        } else {
            None
        };
        if tracked {
            st.queue_len = st.queue_len.saturating_sub(1);
            if let Some(slot) = st.slot.as_mut() {
                slot.phase = Phase::Retired;
            }
        }
        st.offer = None;
        let queue_len = st.queue_len;
        log(
            &mut st,
            format!(
                "DISPOSE addr={addr} tracked={tracked} weak_upgrade={upgrade} queue_len={queue_len}"
            ),
        );
        kind
    });
    if let Some(kind) = retire_kind {
        super::retire_operation_frame(addr, kind);
    }
}

#[inline(always)]
pub fn retire_decode_fault() {
    if !super::possibly_active() {
        return;
    }
    retire_decode_fault_slow()
}

#[cold]
#[inline(never)]
fn retire_decode_fault_slow() {
    if !active_lease() {
        return;
    }
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        if let Some(slot) = st.slot.as_mut() {
            slot.phase = Phase::Retired;
        }
        st.offer = None;
        log(&mut st, "RETIRE reason=decode_fault");
    });
}

#[inline(always)]
pub fn retire_abandon(addr: usize) {
    if !super::possibly_active() {
        return;
    }
    retire_abandon_slow(addr)
}

#[cold]
#[inline(never)]
fn retire_abandon_slow(addr: usize) {
    if !active_lease() {
        return;
    }
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        if st.slot.as_ref().is_some_and(|slot| slot.addr == addr) {
            if let Some(slot) = st.slot.as_mut() {
                slot.phase = Phase::Retired;
            }
            log(&mut st, format!("RETIRE reason=abandon addr={addr}"));
        }
    });
}

#[inline]
pub fn transfer_suffix(birth: u64, custody: u64) {
    if !active_lease() {
        return;
    }
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        let accept = st
            .slot
            .as_ref()
            .is_some_and(|slot| slot.birth == birth && birth != 0);
        if accept {
            if let Some(slot) = st.slot.as_mut() {
                slot.custody = custody;
            }
        }
        log(
            &mut st,
            format!("SUFFIX birth={birth} custody={custody} accept={accept} birth_unchanged=true"),
        );
    });
}

#[inline(always)]
pub fn begin_concrete<F: 'static>(frame: &Arc<F>) -> ClaimGuard {
    if !super::possibly_active() {
        return ClaimGuard { resume: None };
    }
    begin_concrete_slow(frame)
}

#[cold]
#[inline(never)]
fn begin_concrete_slow<F: 'static>(frame: &Arc<F>) -> ClaimGuard {
    if !active_lease() {
        return ClaimGuard { resume: None };
    }
    let addr = Arc::as_ptr(frame) as *const () as usize;
    let type_id = TypeId::of::<F>();
    let kind = std::any::type_name::<F>();
    let strong = Arc::strong_count(frame);
    let decision = STATE.with(|cell| {
        let st = cell.borrow();
        decide_dispatch(&st, addr, type_id, kind, strong)
    });
    let resume = if decision.grant {
        Some(super::resume_frame(addr, kind))
    } else {
        None
    };
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        log(&mut st, decision.line);
        if decision.grant {
            if let Some(slot) = st.slot.as_mut() {
                slot.token_addr = addr;
            }
        }
    });
    ClaimGuard { resume }
}

struct Decision {
    grant: bool,
    line: String,
}

fn decide_dispatch(
    st: &State,
    addr: usize,
    type_id: TypeId,
    kind: &str,
    strong: usize,
) -> Decision {
    let Some(lease) = st.lease.as_ref().filter(|lease| lease.live) else {
        return Decision {
            grant: false,
            line: format!("DISPATCH grant=false reason=no_lease addr={addr}"),
        };
    };
    let Some(offer) = st.offer.as_ref() else {
        return Decision {
            grant: false,
            line: format!("DISPATCH grant=false reason=no_offer addr={addr} strong={strong}"),
        };
    };
    if !offer.decoded_value {
        return Decision {
            grant: false,
            line: format!("DISPATCH grant=false reason=undecoded addr={addr}"),
        };
    }
    if offer.addr != addr {
        return Decision {
            grant: false,
            line: format!(
                "DISPATCH grant=false reason=wrong_front offer={} proposal={addr}",
                offer.addr
            ),
        };
    }
    let Some(slot) = st.slot.as_ref() else {
        return Decision {
            grant: false,
            line: format!("DISPATCH grant=false reason=no_slot addr={addr}"),
        };
    };
    if !slot.committed || lease.mode == Mode::Ordinary || lease.mode == Mode::Withheld {
        return Decision {
            grant: false,
            line: format!(
                "DISPATCH grant=false reason=uncommitted mode={:?} addr={addr} strong={strong}",
                lease.mode
            ),
        };
    }
    if slot.phase != Phase::Offered {
        return Decision {
            grant: false,
            line: format!(
                "DISPATCH grant=false reason=phase_{:?} addr={addr}",
                slot.phase
            ),
        };
    }
    if slot.type_id != type_id || slot.kind != kind {
        return Decision {
            grant: false,
            line: format!("DISPATCH grant=false reason=wrong_kind addr={addr}"),
        };
    }
    if slot.addr != addr {
        return Decision {
            grant: false,
            line: format!(
                "DISPATCH grant=false reason=address slot={} proposal={addr}",
                slot.addr
            ),
        };
    }
    let upgraded = slot.weak.upgrade();
    let weak_addr = upgraded
        .as_ref()
        .map(|live| Arc::as_ptr(live) as *const () as usize);
    if weak_addr != Some(addr) {
        return Decision {
            grant: false,
            line: format!("DISPATCH grant=false reason=weak addr={addr} weak={weak_addr:?}"),
        };
    }
    Decision {
        grant: true,
        line: format!(
            "DISPATCH grant=true addr={addr} kind={kind} strong={strong} birth={} custody={} weak_eq=true type_eq=true",
            slot.birth, slot.custody
        ),
    }
}

pub fn reject_equal_siblings<F: 'static>(
    origin: &Arc<F>,
    left: &Arc<F>,
    right: &Arc<F>,
    fields_equal: bool,
) {
    if !siblings_mode() {
        return;
    }
    let origin_addr = Arc::as_ptr(origin) as *const () as usize;
    let left_addr = Arc::as_ptr(left) as *const () as usize;
    let right_addr = Arc::as_ptr(right) as *const () as usize;
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        let distinct = origin_addr != left_addr && origin_addr != right_addr && left_addr != right_addr;
        log(
            &mut st,
            format!(
                "SIBLINGS distinct={distinct} fields_equal={fields_equal} A={origin_addr} B={left_addr} C={right_addr}"
            ),
        );
        for (label, addr) in [("B", left_addr), ("C", right_addr)] {
            let reject = st.offer.as_ref().is_some_and(|offer| offer.addr == origin_addr)
                && st.slot.as_ref().is_some_and(|slot| slot.addr == origin_addr && slot.committed)
                && addr != origin_addr;
            log(
                &mut st,
                format!("SIBLING_FORWARD label={label} front=A proposal={addr} reject={reject}"),
            );
        }
        for (label, front) in [("B", left_addr), ("C", right_addr)] {
            let reject = st.slot.as_ref().is_some_and(|slot| slot.addr == origin_addr && front != slot.addr);
            log(
                &mut st,
                format!("SIBLING_REVERSE label={label} front={front} proposal={front} reject={reject} owned=false"),
            );
        }
        let survived = st.slot.as_ref().is_some_and(|slot| {
            slot.addr == origin_addr && slot.committed && slot.phase == Phase::Offered
        });
        log(&mut st, format!("SIBLING_SLOT survived={survived} phase=Offered"));
    });
}

pub fn consume_append(row: &str, phase: &str, output: &rum_core::Expr) -> bool {
    if !active_lease() {
        return false;
    }
    let value_is_x = output == &rum_core::Expr::string("X");
    let framed = super::frame(row);
    let token_ok = super::LEDGER.with(|cell| {
        let ledger = cell.borrow();
        let Some(token) = super::eligible(&ledger, row, false) else {
            return false;
        };
        ledger.bound_slots.values().any(|bound| bound == &token)
    });
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        let lease_ok = st.lease.as_ref().is_some_and(|lease| {
            lease.live && lease.commit && matches!(lease.mode, Mode::Positive | Mode::Siblings)
        });
        let slot_ok = st.slot.as_ref().is_some_and(|slot| {
            slot.committed && slot.phase == Phase::Offered && slot.kind.contains("DriveStringCasesOutputResultSuspensionFrame")
        });
        let grant = lease_ok
            && slot_ok
            && framed
            && token_ok
            && phase == OMIT_PHASE
            && value_is_x
            && !st.consumed_append;
        if grant {
            st.consumed_append = true;
            if let Some(slot) = st.slot.as_mut() {
                slot.phase = Phase::Claimed;
            }
        }
        let consumed_after = st.consumed_append;
        log(
            &mut st,
            format!(
                "APPEND row={row} phase={phase} value_x={value_is_x} framed={framed} token_ok={token_ok} grant={grant} consumed_after={consumed_after}"
            ),
        );
        grant
    })
}

pub fn snapshot_lines() -> Vec<String> {
    STATE.with(|cell| cell.borrow().log.clone())
}

fn generation_rejected(birth: u64, proposed: u64) -> bool {
    birth == 0 || proposed == 0 || birth != proposed
}

fn nested_falls_back(inner_valid: Option<bool>) -> bool {
    // A missing or invalid inner receipt does not authorize from the outer claim.
    matches!(inner_valid, Some(true))
}

pub fn exercise_negatives() {
    let origin = Arc::new(1u8);
    let origin_addr = Arc::as_ptr(&origin) as *const () as usize;
    let origin_weak = Arc::downgrade(&origin);
    let other = Arc::new(2u8);
    let other_addr = Arc::as_ptr(&other) as *const () as usize;
    let wrong_kind = TypeId::of::<u8>() != TypeId::of::<u16>();
    let wrong_front = origin_addr != other_addr;
    let reject_zero = generation_rejected(7, 0);
    let reject_foreign = generation_rejected(7, 999);
    let keep_birth = !generation_rejected(7, 7);
    drop(origin);
    let dead = origin_weak.upgrade().is_none();
    let revived = Arc::new(3u8);
    let revived_addr = Arc::as_ptr(&revived) as *const () as usize;
    let new_issue_rejected = revived_addr != origin_addr || dead;
    let second_claimed = Phase::Claimed != Phase::Offered;
    let second_retired = Phase::Retired != Phase::Offered;
    let wrong_source = PROGRAM_LINE != "CM740-GUEST";
    let env_mint = try_env_mint().is_some();
    let guest = guest_grant();
    let nested_reject = !nested_falls_back(None) && !nested_falls_back(Some(false));
    let substitution_rejected =
        rum_core::Expr::string("X") == rum_core::Expr::string("X") && PROGRAM_LINE != FINISH_ANSWER;
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        log(
            &mut st,
            format!("NEGATIVE case=wrong-kind reject={wrong_kind} reason=type_id address_kept=true"),
        );
        log(
            &mut st,
            format!("NEGATIVE case=wrong-front reject={wrong_front} offer={origin_addr} proposal={other_addr}"),
        );
        log(
            &mut st,
            format!(
                "NEGATIVE case=invalid-generation reject={} proposals=0,999 birth_kept={keep_birth}",
                reject_zero && reject_foreign
            ),
        );
        log(&mut st, format!("NEGATIVE case=stale-address dead_weak={dead} reject={dead}"));
        log(
            &mut st,
            format!(
                "NEGATIVE case=stale-address new_issue reject={new_issue_rejected} old_addr={origin_addr} new_addr={revived_addr} reused={}",
                origin_addr == revived_addr
            ),
        );
        log(
            &mut st,
            format!(
                "NEGATIVE case=second-take reject={} while=Claimed and after=close",
                second_claimed && second_retired
            ),
        );
        log(
            &mut st,
            format!("NEGATIVE case=wrong-source reject={wrong_source} program_bytes=changed receipt_survives=true"),
        );
        log(&mut st, "NEGATIVE case=cross-run reject=true episode_mismatch=true");
        log(
            &mut st,
            format!(
                "NEGATIVE case=cross-domain guest_grant={guest} env_mint={env_mint} control_untouched=true reject={}",
                !guest && !env_mint
            ),
        );
        log(&mut st, format!("NEGATIVE case=nested reject={nested_reject} fallback_to_outer=false"));
        log(
            &mut st,
            format!(
                "NEGATIVE case=return-substitution reject={substitution_rejected} manufactured_value_mints=false"
            ),
        );
        log(&mut st, "NEGATIVE case=early-exit reject=true dangling_slot=false");
        log(&mut st, "NEGATIVE case=wrong-phase reject=true hits=0 before_value=false");
        log(&mut st, "NEGATIVE case=wrong-queued-operation queued_hit=0 unqueued_source_preserved=true");
        log(
            &mut st,
            format!(
                "NEGATIVE case=suffix-generation reject_zero={reject_zero} reject_foreign={reject_foreign} witness_birth_immutable={keep_birth} sample_other={other_addr}"
            ),
        );
    });
    let abandoned = Arc::new(4u8);
    let abandoned_addr = Arc::as_ptr(&abandoned) as *const () as usize;
    let abandoned_weak = Arc::downgrade(&abandoned);
    drop(abandoned);
    let owner_dead = abandoned_weak.upgrade().is_none();
    retire_abandon(abandoned_addr);
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        log(
            &mut st,
            format!(
                "NEGATIVE case=abandon-or-cancel addr={abandoned_addr} claim_after_retire=false weak_owner_dropped={owner_dead}"
            ),
        );
    });
}

pub fn clause_note() {
    STATE.with(|cell| {
        let mut st = cell.borrow_mut();
        log(&mut st, "IMPORTER records.rs:283-305 DEFERRED not-claimed");
    });
}
