# brief-586 — E1 step A: the catalog-wide suspension oracle, its manifest and its falsifiers

Implementation brief, **test-only**. **Do not run `git` in any form**,
including before you have read this brief. Leave every change uncommitted for
the director's gate.

## Goal

Every group of lost continuations brief-548 and brief-583 found is repaired
(G1-G10, briefs 559-585; lost 0 by the re-census). E1 makes that hold for the
whole installed catalog: **a builtin cannot be installed without declaring how
it can run guest code, and each declared guest-effect entry carries an
executable rum-vs-rum witness** — resumed equals queued, with the suspension
asserted, at a middle and a second position, under the lifecycle.

E1 runs in steps. **This step (A)** builds the whole mechanism and settles the
whole catalog:

1. the manifest — one reviewed row per installed spec (default and
   no-default builds, exact set equality both ways), each with an explicit
   disposition and its guest-effect entry IDs;
2. the two-session runner, observation and lifecycle support;
3. the four falsifiers, each watched to fail;
4. the existing nine continuation suites folded onto the shared runner without
   losing a test name or an assertion; the 195 historical brief-548 labels
   and brief-583's 14 probes as descriptors;
5. an **owed ledger**: every declared entry without a descriptor yet, checked
   for exact equality — an owed entry that gains a descriptor, or an entry
   missing from both, fails. Later steps drain it; E1 closes only when the
   ledger is empty **and the ledger mechanism is deleted**, so an empty ledger
   cannot survive as an escape hatch;
6. the two-feature gate script.

brief-583's `.temp/brief-583/oracle-design.md` and `draft-E1.md` are the raw
material; **this brief governs** where they differ.

## Director rulings on brief-583's open questions

- The layout of `oracle-design.md` §7 is approved, as is
  `scripts/check-suspension-oracle.sh` as the required two-feature gate and its
  entry in `docs/testing-strategy.md`.
- The test-only mutation frame enters a **separate, explicit test-fixture
  population** with its own named list and count in
  `runtime/tests/frame_root_trace.rs` and `frame_root_trace/derived_inventory.rs`;
  every production list, count and exclusion stays unchanged.
- **No source-drift effect discovery in E1** (draft item 6 is withdrawn). A
  heuristic index of effect sites is the census again — the raw tool reported
  0 Lost where review found 4. Binding source edges to entries belongs to E2,
  where the compiler forces every may-suspend edge to an explicit disposition;
  E2 will key those dispositions to **this step's entry IDs**, so make them
  stable, readable identifiers (`Scan.callback`, `Sort.comparator`, …). E1
  checks only that each entry's named owning function exists in its named file.
- No performance gate: nothing here is compiled into a production path.

## Context pointers

- brief-583's report and `.temp/brief-583/{oracle-design.md,draft-E1.md,e2-outline.md,reviewed-census.tsv,unsettled-resolution.tsv,supplemental-edges.tsv,option-read-implementations.tsv,new-probes.json,probe.rs,installed-default-catalog.tsv}`;
  `.temp/brief-548/{probes.json,probe.rs,make_probes.py,enforcement.md}`.
- The enumerators `Session::registered_builtin_specs`
  (`runtime/session/session_api.rs:388`) and `rum-stdlib/src/docgen.rs:309`.
- The nine suites under `crates/rum-stdlib/tests/`: `loop_`, `lazy_pull_`,
  `schema_`, `rule_pattern_`, `option_reader_`, `sow_trace_`,
  `destructuring_`, `default_fallback_continuations.rs` and their children.
- `docs/guide/{evaluation,patterns-and-rules,structured-data,scoping-and-contexts,embedding}.md`,
  the generated reference, `docs/testing-strategy.md`, `TEMPLATE.md`.

## Scope

**Objective:** the oracle, its manifest, the owed ledger, the falsifiers, the
fold-in and the gate script.

**Edit surface:** new `crates/rum-stdlib/tests/suspension_oracle.rs` and its
subtree (`oracle-design.md` §7); the nine existing suites and their children,
only to share the runner and descriptors; new
`scripts/check-suspension-oracle.sh`; `docs/testing-strategy.md`, only the
gate's entry and its exact guarantee; the two runtime frame-test files, only
the separate fixture population.

**Standing duty — always in scope** (`TEMPLATE.md`; test edits under `crates/`
expire the fingerprints too): fingerprints, both certificates, ownership
reconciliation, every row disclosed. **STOP** on any existing row change or
disappearance, any new `UnknownHostPayload` or `self_reaching` row, any new
row in untouched code.

**Fence:** no production `.rs`, `lib.rs` registration, manifest, lockfile or
dependency change; no host behaviour change; no leases; no retention-census
expansion; benchmark scripts unchanged. No edits to `AGENTS.md`, `TODO.md`,
root `PROGRESS.md`, earlier briefs or other docs. A defect found in production
code is **reported with a witness, not repaired** — it becomes a group of its
own.

**Disclosure duty:** every manifest disposition that is not obvious from the
source, every owed entry, every trust seam (external handlers, feature-gated
rows), every changed test name, every inventory row, every adjacent defect.

## Deliverable

**Manifest.** One row per installed spec, keyed by qualified name and filtered
by an explicit feature predicate; live enumeration in both builds (643 default
observed; 638 no-default is an inference — enumerate it). Exactly one
disposition: `GuestEntries{ids}` (non-empty) or `NoLocalGuestEffect{rationale}`
with its shared preparation policy. Documented sinks (best-effort display,
trial evaluation) are `GuestEntries` with a `Boundary` policy and a witness
that the suspension is retired, not propagated. The manifest is checked-in
reviewed data, never regenerated from the observed set during acceptance;
missing, duplicate and stale rows fail. Core specs preinstalled by
`Session::new` are distinguished from `install_core`'s.

**Entries and descriptors.** An entry names its builtin(s), role, owning
function and file, disposition (`Frame`, `Tail`, `Boundary`) and its
descriptor IDs. A descriptor builds guest source for each required variant
(`Middle`, `Twice`, and where the role has them `Decline`, `Error`,
`AlreadyResumed`, `Nested`); loops pause at least twice inside five or more
operations. Ordinary argument evaluation never stands in for a callback; one
fixture covers several roles only if it demonstrably exercises each.

**Runner.** Two fresh identically configured sessions per run. Resumed side:
empty queue, assert `InputRequired` from the intended role, exact kind and
prompt, deliver the next planned answer; reject too few, too many, wrong order
or kind; `pending_input_request() == None` after. Queued side: the same answers
pre-queued, no surfaced suspension, the queue proven exhausted. Compare typed
values, error and termination, message identities and payloads, the final
cumulative prints (segments verified as prefixes, never double-counted), a
fully qualified callback log and declared state projections — plus a few
explicit expected values, so two identically wrong runs cannot pass. Normalize
only declared nondeterministic leaves. A fresh request after every terminal
run. **Lifecycle** for every distinct continuation owner, at the first and the
second pause: completion, cancellation (documented cleanup, structured
cancelled outcome, fresh request), reset (no guest cleanup, empty pending and
queued state), and a pending request rejecting a new one.

**Falsifiers** (`oracle-design.md` §5), each through the same acceptance
function as the real descriptors, asserting the precise failure, then the good
control: an omitted registration (`Scan` dropped from an enumerated copy; a
missing and an undeclared manifest row); a non-suspending fixture (the
`InputString` replaced by a literal, equal final value); a false tail (a
test-only builtin forwarding the child's result past its loop); a missing
prefix (a test-only fold continuing through a real frame with an empty
accumulator). Plus the owed ledger's two falsifiers. Test-only builtins
register only in their own sessions.

**Fold-in.** Capture each suite's `--list` before and after; the normalized
name sets must match. Specialized depth, policy and resource assertions stay.
A ledger maps every historical label and every brief-583 probe to its
descriptor; the five formerly divergent ones are equality tests.

## rum / Rust pitfalls (read twice)

1. A first-position or single-pause fixture misses the repaired mechanisms;
   an eager-only fixture proves nothing.
2. A pending request rejects a new one; it is not superseded. Cancel runs the
   documented cleanup, reset does not.
3. Generated guest sources use `.rum`. No aborting fixture, no wall-time
   equality, no historical loader-abort probe.
4. Support modules under 10k characters, scenario modules under 15k, split by
   responsibility.
5. Stop-loss counters: specs reviewed, entries with descriptors, lifecycle
   obligations passing, labels mapped, falsifiers watched to fail, inventories
   reconciled. Stop after two consecutive compiling attempts that move none;
   fewer compiler errors is progress. Never reduce an obligation to move a
   counter.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-586/tmp …`. Bounds: the oracle target 900 s per
build; runtime, ownership and retention 300 s; `cargo test --workspace
--no-fail-fast` 3 000 s.

Red first, recorded: each falsifier against a deliberately weakened checker
passes, then fails against the real one for the stated reason.

```
cargo fmt --check
cargo test -p rum-stdlib --test suspension_oracle
cargo test -p rum-stdlib --no-default-features --test suspension_oracle
scripts/check-suspension-oracle.sh
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Each of the nine suites runs green after its fold-in. Report native ZeroMQ
skips as skips.

## Checkpointing — protect the scene

Everything under `.temp/brief-586/`; `PROGRESS.md` after each milestone with
the counters, compile state and the exact next action. This is a long run:
checkpoint so a resumed run loses at most one milestone.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the new gate script. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only. No
  new `unsafe`.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Files changed; the no-production-edit audit.
2. Per build: installed, manifest, `GuestEntries` / `NoLocalGuestEffect` /
   `Boundary` counts; entries, descriptors, lifecycle obligations; the owed
   ledger's size by family.
3. The six falsifiers, red then green, verbatim; the fold-in name sets and the
   label ledger; the broad suites; every inventory row change.
4. **Drafted follow-up briefs** (`TEMPLATE.md` shape) that drain the owed
   ledger family by family, the last one deleting the ledger; defects found;
   stop-loss counters; open questions.

---
