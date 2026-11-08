# Brief 709 — LP08 schema-v2 publication: the epoch-wide logical families and the snapshot renderer

**Design for the live tree: `Scope: NONE — no live source, workload,
manifest, inventory or version-control change.` Disposable build probes
only, under this brief's root.**

LP08's counter promotion landed at `b9b39763` (install 705, repair 707,
final campaign 708) and the tree is green (workspace 4,156 passed / 0
failed / 61 skipped; five-mode LP01 parents byte-identical). The remaining
LP08 deliverable is the **public schema-v2 snapshot**: `RumProfileSnapshot`
still returns `Null` after printing the legacy diagnostics
(`crates/rum-runtime/src/runtime/debug_tools/basic.rs:124-139`), and
`docs/dev/profiling.md:130-139` specifies the target. This round designs
that publication and measures the collection it needs, on the current tree.

The design history is binding context, not a plan. Brief 676 measured three
publication forms and excluded all three: the legacy ranked summary
truncates 96 real definition keys to 64 rows; the epoch-only renderer misses
21 logical keys in a thread-handoff witness (missing families `cache`,
`cache-source`, `cache-symbol`, `counter`, `runtime`); the hybrid of epoch
rows plus the test-only exhaustive observer prints those families but has 23
logical rows that depend on the snapshot thread. Its Q2 (an epoch-wide
synchronous-event collector) was authorized; the collector that later landed
(`crates/rum-runtime/src/runtime/profiling/logical/collector.rs`) is a
pause-boundary stash, not the synchronous-family collector, so the Q2 design
was never done. Brief 676's evidence trees were pruned by `clean-temp`;
`.temp/brief-676/DESIGN.md` remains, and every count and `path:line` in it
is a claim to re-derive, never evidence.

## Goal

Answer one question with measured evidence: **what exact form publishes the
additive schema-v2 snapshot on the current tree, what epoch-wide collection
does it need for the families the epoch does not own today, at what disabled
cost, and what implementation briefs land it?**

1. **The complete family table, re-derived.** For every logical family in
   `docs/dev/profiling.md:20-27` — runtime path, builtin, definition rule,
   rule RHS, definition dispatch, evaluation-cache entry events — and for
   the `counter|…` rows the LP01 observer publishes
   (`profiling/tests/logical_contract.rs:91,100`): the exact key shape, the
   vector fields, every current recording site with `path:line`, the
   storage (epoch `Index` / TLS map / process atomic), the gate flag, the
   reset and retirement behaviour, the thread behaviour, and whether the
   epoch collects it today. State the union of keys the schema must publish
   and prove it is exactly the contract's "every logical key exactly once".
   Name every one of the 21 historically missing keys on the current tree;
   a family that the current source cannot produce is excluded with its
   measured witness, not assumed away.

2. **The collection form for the missing families.** Decide, per family,
   whether it joins the epoch's own storage (same budget and index) or a
   separate epoch-scoped, bounded side table, and why. The form must be:
   - **process-wide per epoch** — a row touched on thread A and continued
     on thread B aggregates once, and the snapshot does not read any
     thread-local map;
   - **exactly once** — no double counting with an epoch row of the same
     key, no duplicate publication;
   - **bounded and retired** — the existing diagnostic cap governs it;
     reset, Session drop, profiling reset, cap exhaustion and
     instrumentation faults retire it exactly once with the existing
     reasons, and an invalid epoch publishes visibly;
   - **zero disabled cost** — every collection site sits inside an
     existing enabled branch; the disabled path is instruction- and
     stack-identical. Measure at one source path, CGU=1, two independent
     builds, with the queued path's own standing gate as control.

3. **The renderer form.** The exact guest expression, stated as code or as
   an exact printed example: the labelled schema-v2 header with validity,
   then every logical key exactly once in a **stable order defined over
   symbol text, never interned identity, address or thread**, then the
   family's final vector plus explicit open / abandoned / cancelled
   accounting, then separate labelled telemetry and duration sections. The
   order must be reproducible across processes and across a thread
   handoff. State the label handling, the invalid/absent epoch case, and
   the lock discipline (no interner, allocation or guest evaluation under
   the epoch lock; copy under the lock, build after). No truncation: no
   top-N, no cap below the epoch's own budget, no dedup of distinct keys.
   Equal keys are one row; the >64-key case must publish every key.

4. **The per-mode behaviour matrix.** The oracle collects in five modes
   (disabled, lexical-only, timing-family, evaluation-cache,
   all-detail/source) and compares per mode. State for each mode which
   families exist, what `RumProfileSnapshot` returns, and the exact row of
   the snapshot's **own** entered invocation (open while it runs). If the
   contract sentence "a snapshot's own entered invocation is open" cannot
   hold in a mode (e.g. the builtin family needs the timing flag), say so
   with the measured row and name the ruling it needs. With the logical
   profile off, the call must keep today's behaviour exactly: return
   `Null`, print the same diagnostics, add no work.

5. **The 21-key witness, re-derived and re-run.** Construct a cross-thread
   resume workload from the current fixtures, publish it through the
   proposed form, and show every key the union defines appears exactly
   once, with the rows matching the native epoch collection
   (`Session::logical_profile_test_rows()` plus the new families' native
   accessor). A guest-vs-native key-for-key and field-for-field comparison
   is the parity bar; it is rum against rum.

6. **The lifecycle cases** the contract names: a snapshot taken
   mid-evaluation, more than 64 distinct keys, repeated equal keys, nested
   reset, old-epoch markers, a Session resumed on another thread,
   Drop-before-summary, diagnostic-cap exhaustion, and the disabled mode.
   Each case gets its expected observable and its gate test.

7. **The plan.** A step-by-step design with `path:line` evidence per step,
   the invariants each step preserves, the gate tests and mutants, the
   call-site/ripple inventory (every caller of a function whose duty
   moves), risks and open questions — plus the drafted implementation
   briefs in `TEMPLATE.md` shape. Stage them; the expected staging is
   (A) the family collector and its model tests, (B) the renderer and its
   lifecycle tests, (C) the acceptance campaign (parity, disabled cost,
   LP01/LP02 replays, host contract, workspace). A different staging is
   acceptable if measured.

If a requested form has no zero-disabled-cost shape under these
constraints, the record is a measured exhaustion proof and the
minimum-cost form is named; "impossible" without a measured exclusion is
not a result.

## Hard bars

- `docs/dev/profiling.md:130-139` governs the publication; its lifecycle
  section governs the accounting. The contract's words are the bar.
- Definition-dispatch and evaluation-cache entry events are **logical
  rows**, not telemetry. `input_required`, pauses, segments, abandonment
  reasons and nanos/self_nanos stay out of the logical vector
  (`docs/dev/profiling.md:113-128`). No reclassification to make a row fit.
- The epoch, record, token and queue machinery; the LP03-LP07 adopted keys;
  the completion order; the carriers; the landed counter promotion and the
  loss inventories are **not reopened**. No oracle weakening, no
  re-baseline, no widened or narrowed promotion set.
- No default-path cost; see 2 above. A claim without the two-build
  instruction and stack measurement is not closed.
- No new `unsafe` in `rum-core`/`rum-runtime`; no new dependency; no guest
  evaluation or arbitrary user-code call from the renderer.
- `RumProfileSnapshot`'s current disabled behaviour is a protected
  behaviour: reproduce it on the pre-change tree before any replacement
  bar is trusted.
- Every count is per mode and per key-table, never a top-N or a sample.
- Persist raw output before parsing; calibrate every instrument against a
  known current row before believing it; a red is reproduced before it is
  called a defect.

## Context pointers

- `docs/dev/profiling.md` — the approved target contract: schema v2,
  telemetry/duration separation, lifecycle equations, family keys.
- `.temp/brief-676/DESIGN.md` — the historical design record (its evidence
  trees are pruned; treat its numbers as claims).
- `docs/dev/codex/brief-676-lp08-schema-v2-and-manifest-promotion-design.md`
  — the design brief that produced it.
- `.temp/director/brief-{682..696}-gate.md` — the pause-collector and
  exact-stack campaign; the +64 frame lesson and the stack bars.
- `.temp/director/brief-708-gate.md` — the installed form and the current
  measurement discipline.
- Live sites: `crates/rum-runtime/src/runtime/profiling/` (`mod.rs`,
  `config.rs`, `state.rs`, `records.rs`, `timing.rs`, `summary.rs`,
  `report_timing.rs`, `report_rebuild.rs`, `reset.rs`, `logical/`),
  `crates/rum-runtime/src/runtime/debug_tools/basic.rs`,
  `crates/rum-runtime/src/runtime/builtin_catalog/debug.rs`,
  `crates/rum-runtime/src/runtime/evaluation_cache/profile/`,
  `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs`.
- Read-only oracles: `crates/rum-runtime/src/runtime/profiling/tests/logical_contract.rs`
  (the test-only exhaustive observer and LP01 rows),
  `crates/rum-stdlib/tests/suspension_oracle/`,
  `crates/rum-host/tests/logical_profile_lifecycle.rs`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-709/` — probe trees,
builds, binaries, receipts, drafts, the report.

MUST NOT touch: the live repository tree (no source, test, workload,
manifest, inventory, docs or `.git` change); any retained `.temp` root
except to read and copy; the public API, features, serde, manifests,
dependencies, `unsafe`. You run no version-control command — the fence
refuses it and a refusal is a STOP, not a retry.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs and
probes; it repairs nothing.

## Verification

One cargo command at a time; jobs 8; offline/locked; the worker's cargo
runs plain inside the dispatcher's 16G scope (never nested). Build both
arms at one path with one release codegen unit for every comparison;
Callgrind is serial and CPU-pinned; clear every `RUM_PROFILE_*` variable
except the declared mode. A timeout is not data; derive any bound from
data. Keep every probe tree compiling at each checkpoint.

## Checkpointing

All scratch under `.temp/brief-709/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved.

## Deliverable

- `.temp/brief-709/report-design.json` in `rum-report/1`, with the family
  table, the form families and their measurements, the per-mode matrix,
  the witness, the lifecycle table, the plan and every open question.
- `.temp/brief-709/drafts/` — the drafted implementation briefs in
  `TEMPLATE.md` shape, staged, each with its gate tests, falsifiers,
  call-site inventory and a "must not change" list beside every cost bar.
- Run `python3 -I scripts/report.py check 709` and return its summary and
  all compliance verdict lines.

## Report format

Every run and verdict line quoted; the summary states the one measured
answer and the draft's bar list.

## Director gate — 2026-11-08: ACCEPTED as a design STOP; rulings issued

The round did the right thing: it measured the selected form, reproduced the
contract's duration limit instead of quietly widening the bar, and stopped.
Verdict ACCEPTED as a design; the six questions are ruled below and stage A is
authorized.

### What I verified myself

- Tree unchanged: `git status` clean; the worker's scope verdict says 2,670
  live files, 0 changed/added/removed, writes confined to `.temp/brief-709`.
  `report.py check` reproduces 830 citations.
- The disabled cost rows in `evidence/costs-v5.json`: base and candidate
  `try_apply_function` own/incoming vectors are byte-identical in both
  independent builds (24,549/49,049 at N/2N, slope 49, call slope 1), and the
  stack rows are identical in all four build×arm pairs (slope 18,304,
  intercept 12,392, residual 0, the same eight absolute samples).
- The full-table row (`extra-checks-final.log`): 96 native and 96 guest
  definition keys, legacy ranks 64, 715 total keys, 0 duplicates; stable order
  under reversed interner insertion; the two semantic identities survive.
- The duration limit (`duration-limit-{1,2}.log`): an open native row reports
  24,913 ns and 4,100 ns while the accrued-only schema row reports 0; after
  closure both observers agree (123,743 ns / 37,968 ns). The gap is real and
  the failure is honest.
- The lifecycle rows (`candidate-final1-lifecycle.log`): absent, old epoch,
  nested reset, drop-before-summary, repeated equal keys, distinct semantic
  identities, cap (123 insertions), and fault all pass with the stated
  observables. The 23-key witness is enumerated in `WITNESS-KEYS.md`.

### Rulings

- **Q1 — RULED: publish the native observation; do not build a mirror.** The
  duration section's basis is epoch-accrued totals **plus the observing
  thread's currently running intervals**, which is exactly what the existing
  `Epoch::rows()` fold computes (`logical/epoch/snapshot.rs:30` via
  `logical/segments.rs:361`). Stage B must source native rows from that fold,
  make `rows()` and `segments::running` production-visible (their bodies are
  unchanged; both are called only from the publication path), and name the
  basis in the header. Cross-thread in-flight intervals stay outside the
  basis, exactly as they are for the native observer; no process-wide active
  mirror is authorized. The B gate must show guest == native mid-interval and
  after closure.
- **Q2 — RULED: qualify the sentence in `docs/dev/profiling.md`.** "A
  snapshot's own entered invocation is open while the snapshot runs" holds
  where its family's gate is enabled. In lexical/cache/disabled modes the
  Builtin family does not exist; the header states the effective mode. Do not
  widen flags and do not manufacture a row.
- **Q3 — RULED: accept the current 23-key witness.** It is exhaustive,
  re-derivable, and covers the same five families. The historical 21
  identities were pruned and are unrecoverable; they are not a bar and no
  count is forced. The substitution is recorded here.
- **Q4 — RULED: no second charge.** The retained-ledger cap governs stored
  rows only. Transient publication buffers are bounded by the captured
  epoch's own budget, are not charged again, and a read cannot invalidate the
  epoch. Reusing `Epoch::rows()` makes this concrete: the copy is bounded by
  the 16 MiB diagnostic ceiling, and there is no new fallible-copy failure
  path.
- **Q5 — RULED: accept conservative invalidation.** A timed event whose start
  equals the epoch birth cannot establish membership; a visible
  `InstrumentationFault` is correct. It must never change the guest value or
  the guest resource outcome.
- **Q6 — RULED: adopt the injective key encoding.** Symbols render as
  `s(ESC(context),ESC(name),identity-or-n)` with `% | , ( ) #` percent-escaped;
  None source is `-`; Some always begins `s(`; rows sort by bytewise encoded
  text. Interner ids and `Symbol::Ord` never appear. B carries the adversarial
  host-created context/name tests and the identity-less/identity-bearing
  duplicate tests.

### Also confirmed

- The publication channel stays as the 676 form: `RumProfileSnapshot` returns
  `Null` and emits a `rum-profile schema-v2 ` diagnostic line; legacy
  diagnostics are unchanged. `docs/reference/builtins.md` is updated in B.
- The union `N ∪ S ∪ A ∪ Z` and the epoch-owned side index within the existing
  budget are approved, as are the seven recorder seams and their
  must-not-change list.
- Stage A (`710`) is authorized from draft A with the rulings above; B and C
  follow after A's gate.
