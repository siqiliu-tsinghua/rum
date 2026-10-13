# brief-606 — design round 5: G13's sequence as type-then-flip

`Scope: NONE — no production change.` Design-only, with one optional scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

Round 4 (brief-605, your design) is accepted except for its green sequence and
one addition. This round replaces the single atomic conversion with a sequence
in which **every step lands green and is measured on its own**, and produces
the implementation drafts for that sequence. Nothing else in round 4 is
reopened.

## The director's decisions on round 4

**Accepted as designed** (`docs/dev/codex/brief-605-g13-design-round-4.md`,
`.temp/brief-605/design.md`): the closed channel and its visibility
arrangement with the codec key; the witness, `FrameChain<I, O>`, the
fresh-state `ContinueToken`, `ResumeStop`/`finish_work` and every deferred
family in `deferred-obligations.tsv`; the default-fallback, retention and
public-file-rollback routes; the private `DriverFault`, its structural unwind
and the request latch; two-Arc accounting in the maintained and independent
models; the test-only `FaultProbe` behind the runtime feature
`test-continuation-slots`; the 26 compile-control pairs; the inventory
transitions; the E2 boundary; the full per-label A/B on every step that
changes a hot function. The 28 ordinary witnesses and all 16 raw tables were
reproduced byte-identically by the director, and 623 implementations and 600
production macro sites match an independent scan.

**D1 — the atomic 608 is replaced by type-then-flip.** Round 4 argued that a
smaller step needs bridges and an edge proof. That holds only if a step
changes the transport. Separate the two:

- **Per-type transport mode, on the sealed codec.** Every registered slot type
  has exactly one transport at any time, a compile-time property of its sealed
  implementation: `Expr` is always `Value`; a type that has an existing
  in-band codec starts as *legacy* (it travels as `Value(Expr)` through that
  existing encoder and decoder, exactly as today); a new nominal carrier is
  `State` from birth. A decoder accepts only its type's one arm. This is not
  the dual-arm codec round 4 rightly forbids, and it is deleted with the legacy
  codecs in the last step.
- **Typing steps (phase 1).** Convert frames, finalizers and macro sites to the
  typed API family by family. A typed frame whose types are all legacy or
  `Expr` exchanges only `Value` with its neighbours, so it is compatible with
  legacy frames on both sides whatever the call graph says: no edge proof is
  needed for these steps. Each step is green in both builds, runs the whole
  oracle and the full A/B, and fixes the transformations, chains and deferred
  obligations of the frames it owns. The six raw completers are corrected by
  their return binding in the step that types them.
- **New carriers.** A carrier that is `State` from birth replaces an existing
  encoded payload between specific frames. The step that introduces it owns
  every frame that produces or consumes that payload today: the frames that
  name the carrier (the compiler finds them) and every old decoder use of the
  payload it replaces (`whole-slot-consumer-sites.tsv`,
  `nested-decoder-use-sites.tsv`). A missed consumer is still a legacy frame
  receiving `State`, which the legacy entry rejects as a fault: loud, never a
  silent decode.
- **Flip steps (phase 2).** Once every frame that consumes or produces a
  legacy type is typed, switch that type's transport to `State`. Any subset of
  types may flip in any order, because each type's mode is global. Each flip
  runs that type's internal-encoding corpus (each descriptor, position and
  shape separately) and unconditional `FaultProbe` absence in both builds; a
  frameless transformation that the tables missed shows up there as a fault,
  which is exactly when the atomic plan would have found it too.
- **Removal.** Then the legacy exports, codecs, internal heads, raw insertion,
  the transport mode itself and the temporary macro name go, as in draft 609.

Why: in both plans a missed frameless edge is caught only at run time, and
loudly, so the atomic step buys no detection. What it costs is the gate per
step, a per-label A/B attributable to one family (one variable per A/B), and
a diff a reviewer can read. If you find a population for which this argument
fails, say so with source evidence; do not paper over it.

**D2 — no `TransitRoots` in G13.** A consumer's decoded input is a Rust local
while the consumer evaluates guest code, exactly as today's `Expr` input is
(`suspension/core.rs:736`); typed state was already encoded inside that
`Expr`, so G13 neither creates nor widens this gap. It is one instance of a
wider pre-existing class the director has just demonstrated on the ordinary
path, with no suspension involved: the gensym sweep's roots
(`scoping/gensym_gc.rs:9`, `:136`) are Session fields, so a value held only in
a builtin's Rust accumulator is not a root.
`Table[Module[{x}, x = i; Hold[x]], {i, 4}]` released element by element gives
`{1, 2, 3, 4}`, and `{x$1, x$2, x$3, 4}` with `RUM_GENSYM_GC_THRESHOLD=1`.
That class is recorded for its own repair. G13 states the synchronous transit
contract in one paragraph, adds no transit record, and leaves the maintained
and independent accounting of transit unchanged.

## What this round must produce

1. **The transport mode** in Rust: how the sealed codec declares it, how the
   adapter encodes and decodes per mode, why no decoder can accept both arms,
   how the compile controls cover it, and its deletion. Revise
   `api_prototype.rs` into a compiled model that includes the mode, and add
   compile-control pairs for it to `compile-controls.tsv`.
2. **The slot table by mode**: for each of the 67 whole slots, its initial
   mode (legacy because an existing codec carries it, or `State` from birth),
   and for each `State`-from-birth carrier the frames, sites and old decoder
   uses its introducing step must own.
3. **The partition.** Every row of `all-frame-migrations.tsv` (623), every
   macro site in `macro-dataflow.tsv` (600), every chained insert, deferred
   obligation, insertion route and transformation-disposition row assigned to
   exactly one typing step, with a reconciliation script that proves the
   assignment is a partition of each table. Order the steps; for each give its
   exact file scope, the rows it owns, why it is green beside legacy
   neighbours, which of the 21 recorded divergences and 28 witnesses it
   changes (so the gate knows which comparison must move from divergent to
   equal), and the microbench labels its hot functions touch. **Size every
   step so one dispatch can finish it**: say what you used as the size
   measure and why it is safe; no step may be the whole conversion again.
4. **The flip steps**: which types flip together and why (a cycle among
   legacy types that must flip at once is a reason; convenience is not), the
   precondition each checks (no legacy frame or decoder still names the type,
   by a census that runs in the gate), and the corpus positions per type.
5. **Drafts in `TEMPLATE.md` shape**: revise 606 (the dormant API, now with
   the mode) and 607 (the compatible driver, now mode-aware and without
   transit), and write a full draft for every typing step, every flip step and
   the removal step. Shared sections may be identical across drafts; each
   draft must still stand alone when dispatched. Name drafts by step label
   (`S1`, `S2`, … `F1`, … `R`), not by brief number.
6. Open questions for the director.

## Evidence rules

- Witnesses, if you add any, are rum-vs-rum equality controls with ordinary
  values only, run as the oracle runs them
  (`crates/rum-stdlib/tests/suspension_oracle/descriptors.rs:191`), with the
  pause attributed to a source line. This round runs no internal-encoding
  shapes.
- Every count reconciles against a raw source scan or against your round-4
  tables; say which.
- Reuse the round-4 tables; copy them, do not edit them.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-606/`; **one** optional scratch integration test,
  `crates/rum-stdlib/tests/zz_brief606_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-606/probe.rs`.
- Everything else is read-only, including `.temp/brief-60*/` other than your
  own directory and `.temp/director/`.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-606/`.
- Run no probe that panics or aborts.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief606_probe`
  (900 s); `python3` scripts (300 s each). File reads, `rg`/`grep`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No production code.** No new `unsafe`.
- Checkpoint to `.temp/brief-606/PROGRESS.md` after each section with the
  exact next action.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the optional test.
2. The transport mode and the slot table by mode.
3. The partition and its reconciliation; the flip steps.
4. The drafts, with the scope table.
5. Open questions for the director.

---
