# Durable suspension — persisting a computation across process restarts

**Status: DEFERRED, by user ruling 2026-09-02.** This document records a design
discussion, not an active plan. Nothing here is scheduled.

**Sequencing ruled by the user:** finish the current correctness line
(briefs 401-405: the abandonment protocol, its caller migration, the
request-scoped cancellation repair, and the `rum-jupyter` pump test), then the
optimization line drawing on the Mojo reading, and only then reopen this.

The point of writing it now is that the current line is, incidentally, building
this line's foundation: brief-401 inventoried what state each suspension frame
owns, and brief-402 is making that classification compiler-enforced. That
inventory is the same inventory a persistence layer needs. Recording the
argument while it is fresh costs nothing; re-deriving it later costs a lot.

## 1. The scenario

Run a long computation. Pause it on demand — send the kernel a message, it
saves its state under an identifier. Power the machine off. Days later, power
on, send the kernel the identifier, and the computation continues from where it
stopped. The model is session management in a CLI agent, or a laptop's
suspend-to-disk.

## 2. What the kernel can do today

Three layers. Two exist; the third does not.

### Layer A — persisting definition state across processes: yes, as a text image

`HostSession::snapshot_to_bytes` / `restore_from_bytes`
(`crates/rum-host/src/session.rs:187,203`) over
`Session::snapshot_definitions_source`
(`crates/rum-runtime/src/runtime/session/session_api.rs:299`). The image is
regenerated *source text*; restore resets the session and evaluates it.

Its own documentation lists what it does not capture: attribute-only symbols,
`$` variables, options, messages, installer-provided builtin handlers, the
gensym counter, PRNG state, and contexts beyond what the generated source
re-establishes.

Two existing refusals are worth noting, because they are the right instinct and
this line should extend them rather than invent a new policy:

- restore refuses an image whose source itself suspends
  (`crates/rum-host/tests/lifecycle_buildout.rs:131`);
- snapshot refuses orphaned sealed references
  (`crates/rum-runtime/src/runtime/tests/sealed_snapshot.rs:161`).

`Save` / `DumpSave` also exist, gated by `Capability::FileWrite`
(`crates/rum-runtime/src/runtime/capabilities.rs:254-255`).

### Layer B — pausing mid-computation: yes, in-process, with one trigger

This is the suspension machinery the current line is repairing. Its boundaries
are narrower than they look:

- **The only trigger is `EvalError::InputRequired`.**
  `begin_input_suspension` (`runtime/suspension/core.rs:125`) has exactly one
  production call site: `runtime/builtins_input.rs:237`. Only `InputString`,
  `Input`, and `Dialog` can stop a computation in flight.
- Suspension is gated by `Capability::Interactive` — "May request interactive
  input from the host **or suspend for host input**"
  (`runtime/capabilities.rs:11-12`). A hermetic profile cannot suspend at all.
- Resumption is `resume_with_input`, entirely in memory.

### Layer C — persisting mid-computation state across processes: none

There is no session identity in the kernel — no UUID, no session id. The only
`session_id` in the tree is `rum-jupyter`'s ZeroMQ wire-protocol routing field
(`crates/rum-jupyter/src/model.rs:483`), which is not kernel state.

## 3. Why layer C is impossible on today's implementation

The continuation is not data. It is a queue of trait objects:

```rust
// runtime/suspension/core.rs:10-13
pub(in crate::runtime) struct SuspendedEvaluation {
    active_stack: Vec<EvaluationStackEntry>,
    frames: VecDeque<Arc<dyn SuspensionFrame>>,
}
```

**545 named suspension frames** (the count the repository invariant enforces;
see the appendix, including how a first grep undercounted it by 30%). Each
frame holds arbitrary Rust state — matcher
binding tables, iterator cursors, tree-walker worklists,
`Arc<BTreeMap<Symbol, BlockSymbolState>>`. Trait objects carry vtable pointers,
have no external representation, and cannot be enumerated as data.

So the gap is not a missing API. The continuation is not a portable thing.

## 4. Where rum actually stands

This is the part that changes the assessment, and it is the reason to write the
document rather than just say "deferred".

### The industry outcome: transparent checkpointing lost

Two approaches have competed for three decades in large-scale computing, and
application-level checkpointing won for production work:

| Approach | Representatives | Standing |
|---|---|---|
| Transparent / system-level: snapshot the process image | BLCR (kernel module, defunct), DMTCP, **CRIU** (the basis of container checkpoint) | Marginal outside container migration; largely abandoned for scientific computing |
| Application-level: the program writes its own restart file | GROMACS `.cpt`, LAMMPS restart, climate models; SCR and VeloC make it fast and multi-level rather than transparent | Dominant |
| Log plus checkpoint | Every database (ARIES): recovery loads a checkpoint and replays the log tail | Dominant |
| Declared operator state | Flink's asynchronous barrier snapshotting; Spark's lineage recomputation (deterministic replay) | Dominant |

Transparent checkpointing lost to three specific killers:

1. **OS resources in the image** — file descriptors, sockets, GPU contexts, MPI
   connections. They do not restore, or restore wrong.
2. **Code pointers** — closure environments, function pointers, vtables. A
   saved address is meaningless in a new process. Persistent-memory frameworks
   hit the same wall: Intel's PMDK forbids storing raw pointers or vtables in
   persistent objects.
3. **Size and version coupling** — the image is RAM-sized and bound to a kernel
   build and machine.

### rum is immune to two of the three, by hard line rather than by luck

Measured on the committed tree (`4abdd19f`; commands in the appendix):

- **OS resources: zero.** `std::fs` and `std::net` appear **0 times** in
  `rum-runtime` and `rum-core` outside tests. This is the FS-purity hard line
  from briefs 132-134. The kernel holds no handles to lose. The single raw
  pointer in the suspension module is `native_stack_base`
  (`runtime/suspension/core.rs:442`), a stack address for the native-stack
  guard, which is per-process by definition and simply recomputed.
- **Code pointers: zero in frames.** There are exactly **two** `Box<dyn FnMut>`
  in the entire runtime, both the host-supplied `print_sink` on `Session`
  (`runtime/model/session.rs:92,132`). No suspension frame captures a closure.

That second measurement is the important one, and it deserves a name.

### rum's frames are already defunctionalized continuations

A frame is a **named struct with explicit fields**, paired with a closed
transition protocol (`FrameAction::{Complete, Evaluate, Finalize}`,
`runtime/suspension/core.rs:33`):

```rust
#[derive(FrameRootTrace)]
pub(super) struct EvalSingleSymbolBlockSuspensionFrame {
    pub(super) state: SuspendedSingleBlockScope,
}
```

Representing "the rest of the computation" as named data instead of a closure
is *defunctionalization*, and it is precisely the transformation that makes
continuations serializable. Racket's stateless web servlets get it from a
compiler pass (Jay McCarthy's serializable continuations); Java's Javaflow and
RIFE bought it with bytecode instrumentation; Seaside and Smalltalk get it free
because execution contexts are first-class objects in the image, which is why
a Pharo or Squeak image can be saved mid-computation and resumed after a
reboot — the scenario in §1, as ordinary practice.

**rum did the transformation by hand, for an unrelated reason** — to avoid
holding `&mut Session` across a suspension point. Serializability's
precondition came free.

The applicable precedent is therefore Smalltalk images and Racket serializable
continuations, not CRIU. rum sits much closer to that end than to the C/MPI
end. This was not obvious and is worth not forgetting.

## 5. What OS hibernation teaches

Suspend-to-disk (S4) works like this: the freezer stops every task **at
kernel-defined safe points** (`try_to_freeze()` checks scattered through the
kernel); drivers tear device state down rather than saving it (`dpm_suspend`);
physical memory is written to swap with a header; on resume a fresh kernel
restores the pages and jumps back; **a signature mismatch refuses the resume**.

Four transferable lessons:

1. **Safe points, not arbitrary points.** You never need to snapshot at an
   arbitrary instruction — you need a dense set of *declared* safe points.
   rum's already exist and already cover the tree: the `resume_or_suspend!`
   seam (`runtime/suspension/core.rs`), with 545 named frames as evidence of
   its density. This demotes "pause on demand" from a hard problem to
   mechanical work: add a suspension reason beside `InputRequired` and let
   it propagate through seams that are already there.
2. **Split "copy verbatim" from "tear down and re-derive".** Hibernation does
   not save device state; it saves how to rebuild it. A persistence layer needs
   the same split across frame state. **rum has already used this move once**:
   lazy descriptors are re-derived on snapshot rather than memoized.
3. **Refuse, do not guess.** Hibernation embeds a version signature and refuses
   a mismatched resume. Restoring a continuation into a different binary must
   fail closed. Both existing refusals in §2 are the same instinct.
4. **Two phases: quiesce, then capture.** Freezing before snapshotting means
   the capture code never has to be reentrant against running evaluation. The
   rum analogue is: reach a seam, then serialize. This removes a whole class of
   concurrency problems from the design.

**The one thing that does not transfer, and it is the core of hibernation's
magic:** hibernation restores into an *identical world* — same kernel build,
same machine, same address-space layout. It never has to *interpret* the state;
it copies bytes back. rum wants to restore into a possibly different binary.
**rum therefore needs a schema; hibernation needs none.** That is killer 3 from
§4, and it is where this line's real cost lives.

## 6. Two phases (user framing, 2026-09-02)

The line splits into two phases, and the split is the user's:

- **Phase 1 — graceful hibernation.** The user asks the kernel to pause. It
  stops at a seam, saves state under an identifier, and the process may exit.
  Later, restore from that identifier and continue.
- **Phase 2 — surviving an unannounced death.** Power loss, or a UPS window too
  short to finish gracefully. Periodic checkpoints; on restart, resume from the
  most recent good one.

**Phase 2 is phase 1 wrapped in a loop.** Capture and restore are the same
mechanism; only the trigger and the cadence differ. The retention policy —
interval, location, how many generations, how they rotate — is reliability
engineering that lives in the host, not the kernel, and can be designed
independently. The restore path in particular is shared: phase 1 exercises and
hardens it, phase 2 simply calls it.

### Why phase 2 must exist at all

Not every death is announced. Kernel panics and OOM kills are not. This machine
froze on 2026-09-01 at 01:24:15 with no shutdown sequence, and that outage is
why brief-399's draft was lost. "Save from an arbitrary point when a signal
arrives" is also the hardest capability to build and the one that can neither
be triggered reliably nor tested. Periodic checkpointing subsumes it: once
state is captured every N steps, reacting to a UPS signal is just shortening
the interval to zero — a bonus, never the design goal.

The question to design against is therefore *the process dies at an arbitrary
instant with no warning; how much is lost?* The answer is set by the checkpoint
interval and is independent of whether a warning arrives. Databases settled
this decades ago.

### The constraint the split imposes on phase 1

Phase 1 is **capture and stop**. Phase 2 is **capture and continue**. If phase
1's capture is destructive — serializing the frames while unwinding them —
phase 2 cannot wrap it, and every checkpoint would have to be followed by an
immediate self-restore.

The destructive design is the tempting one, because the consuming unwind is
exactly what `FrameAbandon` (brief-402) already provides. It must not be the
model. **Capture has to be a read-only traversal from the first line of code**,
shaped after `FrameRootTrace`'s visitor, not after `FrameAbandon`'s unwind.
Both protocols exist; picking the wrong one silently produces a phase 1 that
phase 2 cannot reuse.

### The two requirements phase 2 adds that phase 1 does not have

1. **Liveness — a bounded time to reach a capture point.** Phase 1 can wait:
   the pause is user-initiated and its latency is negotiable. Phase 2 cannot —
   if a stretch of computation can run for minutes without reaching a seam, the
   effective interval is not "every N steps" but "whenever a seam happens",
   which is unbounded.

   This quantity is already being measured by the current line: the time from
   an arbitrary point to a point where a request is noticed is exactly
   cancellation latency, which is what briefs 400-405 are repairing. **The
   checkpoint interval's floor and the cancellation latency are the same
   measurement**, and it will be in hand when that line closes.

2. **Repeated cost.** Phase 1 pays for capture once and can afford seconds.
   Phase 2 pays throughout the computation. A full capture costing O(heap) and
   several seconds cannot run every ten seconds; that pressure is what drives
   real systems toward incremental checkpointing (SCR's multi-level scheme,
   a database's WAL versus its full checkpoint).

   **rum has a shortcut here.** `Pure` means deterministic is a mechanical hard
   line, so sparse full checkpoints plus forward replay cover the gap without
   any incremental-capture machinery. That is the database recovery model, and
   it fits this codebase's hard lines unusually well. Route 3 below is
   therefore not a fallback — it is what makes "phase 2 is just a loop" true.

## 7. The three routes

### Route 1 — serializable continuations (the real capability)

Give the frames a third mandatory protocol, `FramePersist`: serialization, a
type-tag registry, and a deserializing constructor.

The shape has two precedents in this codebase already: `FrameRootTrace`
(GC rooting, compiler-enforced per frame) and `FrameAbandon` (brief-402, in
flight). A third protocol of the same shape is enumerable and enforceable the
same way.

The cost is not the frame count. It is four things:

1. **Non-`Expr` Rust state** needs an external representation: matcher
   bindings, iterator cursors, worklists.
2. **`Symbol` is a process-local interner id** and must be re-interned by name
   on restore. This project has already been bitten once by a process-global
   interner id, in the call-stability-proof Bloom collision.
3. **`Arc` sharing is lost across a round trip.** One large `Expr` shared by ten
   frames comes back as ten copies. That is a real space and semantics problem,
   not a detail.
4. **Values that already do not round-trip**, with an existing precedent for
   each: gensyms (`Module` locals), sealed references (`Save`/`DumpSave`
   already refuse them), lazy descriptors (re-derived).

Plus versioning: restoring a continuation into a different kernel build.

### Route 2 — application-level checkpointing (available today, zero kernel change)

Write the long computation as a resumable loop over explicit state; every N
steps write the state to a file (`Capability::FileWrite`); on restart read it
back and continue from the saved iteration. The limits are that the state must
be round-trippable `Expr` data and that the program, not the operator, chooses
the pause points.

### Route 3 — deterministic replay

`Pure` means deterministic is a mechanical hard line (briefs 132-134), so a
pure computation is replayable by construction. Replay alone is useless for
pausing a ten-hour job — it redoes the work — but combined with route 2 it is
exactly the database recovery model: load the last checkpoint, replay forward.

Route 3 is load-bearing for phase 2, not a fallback: it is what lets a sparse,
affordable checkpoint interval still bound the loss window, and therefore what
makes "phase 2 is phase 1 in a loop" true in practice.

**Recommended short-term posture: routes 2 and 3 together.** They need no
kernel change and give a bounded, tunable loss window.

## 8. What must be answered before opening this line

The scale of route 1 is currently a guess. It does not have to be. Three
questions are mechanically countable once brief-403 lands and the per-frame
protocol is locked by the compiler. The fourth is a design question the same
inventory answers:

1. Of the 545 named frames, how many have fields that are **purely**
   `Expr` / `Symbol` / primitive data — i.e. serializable with no new design?
2. For the remainder, **how many distinct shapes** of non-data state are there?
   (Ten shapes across sixty frames is a very different project from sixty
   shapes.)
3. `FrameRootTrace` is a read-only visitor. Can its traversal be **inverted**
   into a constructor, or does rooting discard structure that reconstruction
   needs?
4. Can capture be **non-destructive** — a read-only traversal of the frame
   queue that leaves it resumable — for every frame, or do some frames hold
   state that can only be extracted by consuming them? Phase 2 depends on the
   answer being yes (§6); if some family answers no, that family needs a
   different representation before either phase is designed.

A design-only brief asking exactly these four questions is the correct opening
move — after the current line closes, not before. Counting them now would
measure a tree that briefs 402 and 403 are actively reshaping.

## 9. Preconditions

- brief-403 landed: every frame's state ownership is compiler-enforced, so the
  inventory is a fact rather than a document.
- The Mojo-derived optimization line closed, per the user's sequencing.
- A decision on whether pausing on demand needs a suspension reason distinct
  from `InputRequired`, and how `Capability::Interactive` should relate to it —
  a hermetic profile that cannot suspend for input may still want to checkpoint.

## Appendix — evidence

Frame counts are the ones the repository invariant enforces, printed by
`suspension_protocol_repository_invariants_hold` after brief-402:

```text
named_suspension_frames=545
named_pure_finalizers=60
pure_frame_abandon_derives=516
handwritten_stateful_frame_abandons=29
```

**A methodological note worth keeping.** The first pass at this count used
`git grep -c "SuspensionFrame for"` and reported **377**. That was a 30%
undercount: many impl headers wrap as

```rust
impl crate::runtime::suspension::SuspensionFrame
    for EvalFooSuspensionFrame
```

so the substring never appears on one line. A cross-line regex over the same
committed tree returns 546. The wrong number reached brief-402's prose before
the worker caught it against the enforced inventory. This is the standing
project rule in action: cross-check any script-derived count against a live
probe before relying on it — and prefer a count some test enforces over one a
grep produces.

The remaining measurements were taken on the committed tree at `4abdd19f`,
before brief-402's changes.

```
$ grep -rn "Box<dyn Fn\|Box<dyn FnMut\|Arc<dyn Fn" \
      crates/rum-runtime/src/runtime/ crates/rum-stdlib/src/ | grep -v "/tests/"
crates/rum-runtime/src/runtime/model/session.rs:92:    pub(in crate::runtime) print_sink: Option<Box<dyn FnMut(&str) + Send>>,
crates/rum-runtime/src/runtime/model/session.rs:132:    pub fn set_print_sink(&mut self, sink: Box<dyn FnMut(&str) + Send>) {
```

```
$ grep -rn "std::fs\|std::net" crates/rum-runtime/src/ crates/rum-core/src/ \
      | grep -v "/tests/\|test" | wc -l
0
```

```
$ grep -rn "RawWaker\|\*const \|\*mut " crates/rum-runtime/src/runtime/suspension/
crates/rum-runtime/src/runtime/suspension/core.rs:442:  self.native_stack_base = (&stack_marker as *const u8) as usize;
```
