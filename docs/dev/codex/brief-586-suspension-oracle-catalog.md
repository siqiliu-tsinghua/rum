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

## Implementation report — brief 586

Step A is implemented. E1 remains open: the exact ledger records **500 owed
entries**, and the final follow-up must delete the ledger mechanism. These are
not claimed as fully exercised by the 209 historical fixtures.

### 1. Files, scope and mechanism

New integration root: `crates/rum-stdlib/tests/suspension_oracle.rs`. Its test
subtree contains these files, grouped by responsibility:

```
catalog.rs
coverage/mod.rs
coverage/source.rs
descriptors.rs
fixtures/boundaries.rs
fixtures/debug.rs
fixtures/definitions.rs
fixtures/lazy.rs
fixtures/loops.rs
fixtures/mod.rs
fixtures/options.rs
fixtures/patterns.rs
fixtures/preparation.rs
fixtures/primary.rs
fixtures/rewrite.rs
fixtures/rule_construction.rs
fixtures/schema.rs
fixtures/scoping.rs
fixtures/strings.rs
inventory/catalog.tsv
inventory/effects.tsv
inventory/historical-labels.tsv
inventory/owed.tsv
inventory/shared-effects.tsv
manifest.rs
mod.rs
mutations/driver.rs
mutations/mod.rs
support/drive.rs
support/files.rs
support/lifecycle.rs
support/mod.rs
support/observe.rs
support/session.rs
```

`catalog.rs` owns exact set/debt rejection; `manifest.rs` binds reviewed data to
live registration and executable descriptor factories; `coverage/source.rs`
checks only named function existence. `descriptors.rs` and `support/` drive
sessions, typed observations, transcript validation, output prefixes, queue
exhaustion, fresh requests and lifecycle. `mutations/` supplies the deliberately
bad tail/fold implementations. `fixtures/` contains six complete role factories
and 209 supplemental historical fixtures. `inventory/` contains explicit data,
including every owed ID and every historical label-to-descriptor mapping.

The nine suites and five shared helpers changed only for shared driving and
observations; their scenario/lifecycle/depth/resource assertions remain:

```
crates/rum-stdlib/tests/default_fallback_continuations.rs
crates/rum-stdlib/tests/default_fallback_continuations/support.rs
crates/rum-stdlib/tests/destructuring_continuations.rs
crates/rum-stdlib/tests/destructuring_continuations/support.rs
crates/rum-stdlib/tests/guest_loading_resume.rs
crates/rum-stdlib/tests/lazy_pull_continuations.rs
crates/rum-stdlib/tests/loop_continuations.rs
crates/rum-stdlib/tests/option_reader_continuations.rs
crates/rum-stdlib/tests/option_reader_continuations/support.rs
crates/rum-stdlib/tests/rule_pattern_continuations.rs
crates/rum-stdlib/tests/rule_pattern_continuations/support.rs
crates/rum-stdlib/tests/schema_continuations.rs
crates/rum-stdlib/tests/sow_trace_continuations.rs
crates/rum-stdlib/tests/sow_trace_continuations/support.rs
```

Other edits:

- `scripts/check-suspension-oracle.sh`: required serial, bounded two-feature gate.
- `docs/testing-strategy.md:74`: the gate and its exact limits.
- `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:743`: separate
  fixture source/name population and count **1**.
- `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs:1575`:
  one fixture derive row in each of the separate `FrameRootTrace` and
  `PureFrameAbandon` lists, both for
  `crates/rum-stdlib/tests/suspension_oracle/mutations/driver.rs:OracleFoldFrame`.
- Retention `inventory/local-contracts.tsv` and `inventory/promoted.tsv`: replayed
  fingerprints/certificate references only, disclosed below.
- This brief: report appended.

SHA256 comparison against the initial 1,832-file scene found no production Rust,
manifest, lockfile, dependency, benchmark, root-guide, TODO or root-PROGRESS
changes. The original runtime inventory files are byte-exact prefixes of the
new files (30,342 and 175,130 bytes respectively): every production population,
count and exclusion is unchanged. No new unsafe. No git command was run. No
external network access was used. All source changes remain uncommitted.

The complete file audit and initial hashes are in
[the audit](../../../.temp/brief-586/audit.txt) and
[baseline hashes](../../../.temp/brief-586/baseline-hashes.json). The final file
list includes this report. Support modules are at most 6,425 characters;
scenario modules at most 13,981 characters. No performance gate was run, as
this brief explicitly exempts the test-only change.

### 2. Catalog, observations, coverage and debt

| Build | Installed | Manifest | GuestEntries rows | NoLocalGuestEffect rows | Boundary entries | All entries | Complete descriptors | Historical fixtures | Owed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| default | 643 | 643 | 240 | 403 | 4 | 506 | 6 | 209 | 500 |
| no-default | 637 | 637 | 240 | 397 | 4 | 506 | 6 | 209 | 500 |

Boundary is a subset of GuestEntries, not a third catalog-row disposition.
There are 504 builtin entries plus two shared evaluator entries. Origins are
167 retained core, 18 stdlib overrides of core and 458 additional stdlib specs
(default), or 452 additional stdlib specs without default features.

The six complete descriptors cover `Scan.callback`,
`Evaluator.callPreparation`, `ValueQ.trial`, `Monitor.display`,
`Monitor.evaluationMonitor`, and `Monitor.stepMonitor`. Their exact required
variant sets execute centrally: **19 completion/retirement variants**. Scan and
argument preparation each perform five operations with distinct second/fourth
positions. Their two continuation owners additionally pass **12 lifecycle
obligations** (cancel, reset and rejection of an overlapping request at the
first and second pauses); completion is included in the variant runs. Reset is
Session replacement using the same configuration, matching the kernel lifetime
operation; existing rum-host suites remain responsible for the host reset API.

The runner validates prompt and kind before delivering an answer and checks
queue exhaustion immediately after the terminal result, before guest state
projections. Typed values, errors, termination and complete RuntimeMessage
payloads compare without normalization. Cumulative prints are checked as
prefixes and compared once. Global-qualified logs, projections and explicit
expected values protect against equal-but-wrong results. Every terminal run
admits a fresh request. No duration equality or nondeterministic normalization
was added.

Retirement witnesses observe the child's actual typed InputRequired through a
small test-only tail adapter before the enclosing production sink retires it.
This matters for ValueQ because its guest-state rollback alone would erase the
attempt. Exact attempted kind/prompt and no surviving pending owner are checked;
Monitor also proves that the prefix ran and suspended suffix did not. These
boundary cases use empty response plans because no input may surface. The
adapter is installed only in those fixture sessions; the mutation builtins are
installed only in their own sessions.

Historical descriptors preserve the historical source workloads (with explicit
Global log qualification, exact response counts replacing padded pools, and a
`.rum` mock path). They are executable equality fixtures, not blanket coverage
marks for complete role/lifecycle obligations. Destructuring, Default fallback
and option cases additionally project persistent state. The old loader sink
controls retain their intentional response pools and specialized expectations;
the new boundary descriptors independently enforce exact retirement and empty
queues.

Non-obvious dispositions and trust limits:

- The live no-default count is **637**, not the inferred 638. `Hash` is also
  feature-gated (`crates/rum-stdlib/src/catalog/strings_text.rs:105`), along with
  Crc32, Md5, Sha1, Sha256 and Sha512. All six predicates are explicit data.
- Held definition targets are local guest work (`Unset.target` included).
  Metadata-only inspection/normalization (Attributes, Information, Definition,
  OwnValues, UpValues, DownValues, SubValues, DefaultValues, SyntaxInformation,
  MessageName, Messages, MessageDefinitions, MessageTags, Protect, Unprotect,
  Options, ArgumentsOptions, CheckArguments) borrows/normalizes syntax without
  locally evaluating it. Shared evaluator preparation/result evaluation still
  applies; these are not language-wide no-effects claims.
- `Message.template` identifies delayed template evaluation; SetAttributes and
  ClearAttributes identify attribute-list evaluation, not their structural
  symbol target. On/Off parse message selectors. Clear/ClearAll/Remove normalize
  symbol/string/list targets. Thread constructs threaded syntax. Their exact
  per-row rationales are in the checked-in manifest.
- Containment predicates explicitly declare SameTest callbacks and merged
  option phases. Comap, ComapApply, Operate and Through explicitly tail-call
  evaluation. Lookup separately declares held data, key, selected default,
  selected wrapper and delayed-value/result work. Constants, held heads,
  callable constructors, native codecs/numeric operations and symbolic
  rebuilding distinguish local work from shared evaluator behavior in their
  recorded rationales.
- Lazy construction/pull roles, generated arithmetic/upvalues, delayed option
  reads, pattern/guard paths and loader forms remain explicit declared debt
  where a complete descriptor is not attached. An ordinary-argument witness
  never discharges one of those roles.
- External installed handlers and providers (files, entropy, output/capability
  services) remain host trust seams. The live domain is Session::new plus the
  repository install_core catalog. A provider's private implementation or an
  arbitrary host-installed handler is not certified.
- Entry ownership is reviewed data. Acceptance proves only that its named
  function exists in its named file. Many owning landing functions delegate to
  helpers; no body fingerprint or heuristic source-edge discovery was introduced.
  Compiler-enforced binding of every may-suspend edge remains E2.

The exhaustive dispositions are in
[inventory/catalog.tsv](../../../crates/rum-stdlib/tests/suspension_oracle/inventory/catalog.tsv),
with a review copy in [disposition-review.tsv](../../../.temp/brief-586/disposition-review.tsv)
and named source locations in [owner-locations.tsv](../../../.temp/brief-586/owner-locations.tsv).
Every owed ID is explicitly listed in
[inventory/owed.tsv](../../../crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv).
Exact set equality rejects both stale debt and entries missing from debt and
coverage; a surviving empty ledger is rejected.

| Owed family | Entries | Drafted implementation follow-up |
| --- | ---: | --- |
| arithmetic | 3 | [draft-E1-01-arithmetic.md](../../../.temp/brief-586/followups/draft-E1-01-arithmetic.md) |
| associations | 13 | [draft-E1-02-associations.md](../../../.temp/brief-586/followups/draft-E1-02-associations.md) |
| associations_transform | 19 | [draft-E1-03-associations_transform.md](../../../.temp/brief-586/followups/draft-E1-03-associations_transform.md) |
| contexts | 12 | [draft-E1-04-contexts.md](../../../.temp/brief-586/followups/draft-E1-04-contexts.md) |
| control | 50 | [draft-E1-05-control.md](../../../.temp/brief-586/followups/draft-E1-05-control.md) |
| debug | 41 | [draft-E1-06-debug.md](../../../.temp/brief-586/followups/draft-E1-06-debug.md) |
| functional | 43 | [draft-E1-07-functional.md](../../../.temp/brief-586/followups/draft-E1-07-functional.md) |
| lazy | 12 | [draft-E1-08-lazy.md](../../../.temp/brief-586/followups/draft-E1-08-lazy.md) |
| logic | 5 | [draft-E1-09-logic.md](../../../.temp/brief-586/followups/draft-E1-09-logic.md) |
| predicates | 45 | [draft-E1-10-predicates.md](../../../.temp/brief-586/followups/draft-E1-10-predicates.md) |
| random | 1 | [draft-E1-11-random.md](../../../.temp/brief-586/followups/draft-E1-11-random.md) |
| rewrite | 55 | [draft-E1-12-rewrite.md](../../../.temp/brief-586/followups/draft-E1-12-rewrite.md) |
| rules | 73 | [draft-E1-13-rules.md](../../../.temp/brief-586/followups/draft-E1-13-rules.md) |
| schema | 4 | [draft-E1-14-schema.md](../../../.temp/brief-586/followups/draft-E1-14-schema.md) |
| scoping | 15 | [draft-E1-15-scoping.md](../../../.temp/brief-586/followups/draft-E1-15-scoping.md) |
| strings | 24 | [draft-E1-16-strings.md](../../../.temp/brief-586/followups/draft-E1-16-strings.md) |
| strings_match | 38 | [draft-E1-17-strings_match.md](../../../.temp/brief-586/followups/draft-E1-17-strings_match.md) |
| strings_text | 10 | [draft-E1-18-strings_text.md](../../../.temp/brief-586/followups/draft-E1-18-strings_text.md) |
| structure | 6 | [draft-E1-19-structure.md](../../../.temp/brief-586/followups/draft-E1-19-structure.md) |
| structure_ordering | 21 | [draft-E1-20-structure_ordering.md](../../../.temp/brief-586/followups/draft-E1-20-structure_ordering.md) |
| symbols | 9 | [draft-E1-21-symbols.md](../../../.temp/brief-586/followups/draft-E1-21-symbols.md) |
| evaluator | 1 | [draft-E1-22-evaluator.md](../../../.temp/brief-586/followups/draft-E1-22-evaluator.md) |
| **Total** | **500** | **22 drafts** |

### 3. Verification and every inventory change

Commands used `scripts/run-bounded.py`, one build worker, one test worker,
CARGO_NET_OFFLINE=true and this brief's TMPDIR. Bounds were 900 seconds per
oracle build, 300 seconds for runtime/ownership/retention and 3,000 seconds for
the workspace. The gate-script wrapper permits both sequential 900-second
children. Formatting was finally rerun to exit 0 serially. One earlier formatting
invocation returned an asynchronous handle before the test-name listing was
started; its exit 0 was then confirmed. No test executions overlapped.

All six falsifiers first pass the deliberately weakened checker, then the same
acceptance function rejects the mutation precisely; the good control passes.
These are actual final default-build log lines (the other build also passes):

```text
WEAK missing_owed: PASS
STRICT missing_owed: Err(MissingOwed("Scan.callback"))
CONTROL missing_owed: PASS
WEAK false_tail: PASS
STRICT false_tail: Value
CONTROL false_tail: PASS (framed and no-pause)
WEAK missing_prefix: PASS (both prompts reached)
STRICT missing_prefix: Value
CONTROL missing_prefix: PASS
WEAK non_suspending_fixture: PASS (equal final values)
STRICT non_suspending_fixture: PauseCount { expected: 1, actual: 0 }
CONTROL non_suspending_fixture: PASS
WEAK omitted_registration: PASS
STRICT omitted_registration: Err(MissingInstalled("System`Scan"))
WEAK omitted_registration: PASS
STRICT omitted_registration: Err(UndeclaredInstalled("System`Scan"))
WEAK omitted_registration: PASS
STRICT omitted_registration: Err(MissingInstalled("System`OracleUndeclared"))
CONTROL omitted_registration: PASS
WEAK stale_owed: PASS
STRICT stale_owed: Err(StaleOwed("Scan.callback"))
CONTROL stale_owed: PASS
```

The omitted-registration falsifier covers Scan removed from live enumeration,
a missing manifest row and an undeclared/stale manifest row. The false-tail
control includes framed and no-pause controls; the missing-prefix mutant really
resumes through a frame and reaches both prompts with its accumulator erased.
The two owed-ledger mutants are temporary until ledger deletion.

All nine normalized before/after name sets are identical. No existing test was
renamed, removed or added:

```text
suite	before	after	missing	added
default_fallback_continuations	21	21	0	0
destructuring_continuations	16	16	0	0
guest_loading_resume	47	47	0	0
lazy_pull_continuations	69	69	0	0
loop_continuations	21	21	0	0
option_reader_continuations	100	100	0	0
rule_pattern_continuations	71	71	0	0
schema_continuations	40	40	0	0
sow_trace_continuations	51	51	0	0
TOTAL	436	436	0	0
```

Evidence: [before listing](../../../.temp/brief-586/list-before.txt),
[after listing](../../../.temp/brief-586/list-after.txt), both normalized JSONs,
and [fold-in diff](../../../.temp/brief-586/suite-fold-in.diff). Removed local
input-driving assertions are enforced by the shared driver (including the
original pending-owner/overlap guard); InputString-only assertions remain in
the two helpers that previously required them. Specialized child-module
assertions were not edited.

The checked-in historical label ledger has exactly 195 brief-548 and 14
brief-583 labels, each mapping to an executed fixture. All 209 are equality
checks. In particular, the five formerly divergent cases are now asserted equal:
`destructure-result`, `destructure-suffix`, `destructure-twice`,
`destructure-nested`, and `default-decline-twice`. No diagnostic inequality
expectation was copied forward.

`cargo fmt --check` exited 0 with no output. The final two-feature script exited 0.

`cargo test -p rum-stdlib --test suspension_oracle` (also executed by the gate script), verbatim:

```text
LIFECYCLE obligations=12 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=240 NoLocalGuestEffect=403 Boundary=4 entries=506 descriptors=6 historical=209 owed=500
OWED families={"arithmetic": 3, "associations": 13, "associations_transform": 19, "contexts": 12, "control": 50, "debug": 41, "evaluator": 1, "functional": 43, "lazy": 12, "logic": 5, "predicates": 45, "random": 1, "rewrite": 55, "rules": 73, "schema": 4, "scoping": 15, "strings": 24, "strings_match": 38, "strings_text": 10, "structure": 6, "structure_ordering": 21, "symbols": 9}
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.19s
```

`cargo test -p rum-stdlib --no-default-features --test suspension_oracle` (also executed by the gate script), verbatim:

```text
LIFECYCLE obligations=12 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=240 NoLocalGuestEffect=397 Boundary=4 entries=506 descriptors=6 historical=209 owed=500
OWED families={"arithmetic": 3, "associations": 13, "associations_transform": 19, "contexts": 12, "control": 50, "debug": 41, "evaluator": 1, "functional": 43, "lazy": 12, "logic": 5, "predicates": 45, "random": 1, "rewrite": 55, "rules": 73, "schema": 4, "scoping": 15, "strings": 24, "strings_match": 38, "strings_text": 10, "structure": 6, "structure_ordering": 21, "symbols": 9}
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
```

`cargo test -p rum-runtime`, verbatim:

```text
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.32s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.65s
```

`cargo test -p rum-stdlib --lib ownership_graph`, verbatim:

```text
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7683 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 60.11s
```

`cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture`, verbatim:

```text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.22s
```

`cargo test -p rum-stdlib --lib local_production_acceptance`, verbatim:

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.19s
```

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance`, verbatim:

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.18s
```

`cargo test --workspace --no-fail-fast -- --nocapture` completed within its bound, with **exit 101**. Summing the 49 target result lines gives **3,352 passed, 9 failed, 58 ignored**. All nine failures were the native ZeroMQ loopback capability check; all other targets passed. Full results: [workspace.txt](../../../.temp/brief-586/workspace.txt) and [workspace-results.tsv](../../../.temp/brief-586/workspace-results.tsv). The standard-library unit result and all nine folded suites, verbatim:

```text

Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1097.03s
Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.50s
Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.21s
Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-1756073e73a214f6)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.24s
```

Native environment failure from the original full run, verbatim:

```text
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
test result: FAILED. 142 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
error: 1 target failed:
    `-p rum-jupyter --lib`
```

The failed target was rerun serially under the same offline worker/TMPDIR settings, with the test helper's explicit `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, using `cargo test -p rum-jupyter --lib -- --nocapture` (300-second bound). It exited 0. The harness counts early skip returns as passed; the honest interpretation is **142 non-native checks passed and 9 native checks SKIPPED**, not 151 native-capable passes:

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
```

The successful targets from the full workspace run were not repeated. The original full command remains an explicitly reported environment failure, followed by the successful scoped retry. No mandatory command was omitted; native socket behavior did not execute. The 58 ignored tests retain their existing status (the ignored fingerprint writer was also run explicitly as maintenance).


Ownership reconciliation made **zero row changes**: no identity, status,
disposition, exclusion or position moved; no new unknown/self-reaching/host row
appeared. The frame-fixture rows above are in a separate population only.

Fingerprint replay generated 68 rows using the existing maintenance test. Its
legacy writer emits `target/brief-544/local-contracts.tsv`; the exact replay was
copied into the inventory and preserved under this brief's scratch directory.
Every non-fingerprint field is identical to the initial inventory. All changed
rows, verbatim:

```text
id	before	after
B01	fnv1a64:31cf155b2a4545a4	fnv1a64:9cc55546787aed0e
B02	fnv1a64:880497fdf91349c1	fnv1a64:87653a1853c15bb7
B03	fnv1a64:fce180323fe16c26	fnv1a64:543f9347d8e3dae4
B04	fnv1a64:39ffda1845b77eec	fnv1a64:68c7bed338e9b386
B05	fnv1a64:eed91d8c2fa0a48d	fnv1a64:aaacfade47b8268b
B06	fnv1a64:65f8ffcaafdd6f28	fnv1a64:eb3a6f2aafdf9dea
B07	fnv1a64:99adb8f2eb0fb6bd	fnv1a64:91a81a4c9a257edb
B08	fnv1a64:a197aab55e923102	fnv1a64:87ae900c59fbec08
B09	fnv1a64:c8598950fc927090	fnv1a64:baeed2680d4d1d42
B10	fnv1a64:5a4d8c06484daf43	fnv1a64:dbe2e4dca7c78015
B11	fnv1a64:fb578b755faba4ec	fnv1a64:2a1f703052ddd986
B12	fnv1a64:8766094cd89b0590	fnv1a64:79fb5263e955b242
I01	fnv1a64:0fb1005896fd3c24	fnv1a64:7aa74043e532e38e
I02	fnv1a64:461a9b2027d72c8d	fnv1a64:01ee78723feeae8b
I03	fnv1a64:3e6f228ab31b6dd0	fnv1a64:76e6ac401bd1c102
I04	fnv1a64:faa81e0379d42b61	fnv1a64:64b4db5dc32855d7
P01	fnv1a64:1fe64dc7ad7eae48	fnv1a64:b4222e76367c7f0a
P02	fnv1a64:928b5a988e664513	fnv1a64:35c63e7603b2e445
P03	fnv1a64:4b1773143b7c9e3e	fnv1a64:795197df27a15a2c
P04	fnv1a64:7cdfb6137dbc65c1	fnv1a64:7c40582dd86a77b7
P05	fnv1a64:da206b676bf31234	fnv1a64:5b78b9f288e52dfe
P06	fnv1a64:d7e52e1d64ca9f10	fnv1a64:ca7a773475854bc2
P070	fnv1a64:1232ca5f43401910	fnv1a64:04c8137653fac5c2
P080	fnv1a64:31a421b1e8c08e83	fnv1a64:34a0f197e21e87d5
P090	fnv1a64:7c30557c5a888b01	fnv1a64:bbe90c1c58b85c77
P100	fnv1a64:a5717dac1f280a5c	fnv1a64:2ec97dcbb4565296
P110	fnv1a64:e09e90050200641b	fnv1a64:9a9ad29c2e141e1d
P120	fnv1a64:1f8cf385ae269da2	fnv1a64:b98e0701308817a8
P130	fnv1a64:3d7b125019c390bd	fnv1a64:357573a9c8d958db
P140	fnv1a64:792a64d7abcff3f9	fnv1a64:2d4dd9256747761f
P150	fnv1a64:ae8e650c2fcecca1	fnv1a64:9a2914528af6f397
P071	fnv1a64:2ebafab09584455d	fnv1a64:dd9a3565550d527b
P081	fnv1a64:d422c6e4fbe6afb0	fnv1a64:ef29ce2aac0987e2
P091	fnv1a64:23fff8e176d4ba88	fnv1a64:f4ef80adaee07dca
P101	fnv1a64:d8a180d23764011b	fnv1a64:929dc3696377bb1d
P111	fnv1a64:f8e486a5626f1a00	fnv1a64:0c7f257946488ed2
P121	fnv1a64:5b0fe059abda9477	fnv1a64:1ed585c9e8faf791
P131	fnv1a64:d7a9fc7e7c0ba06c	fnv1a64:0671e1396f3dd506
P141	fnv1a64:ba1d95f3f7bcf768	fnv1a64:0364eeb8d56d7faa
P151	fnv1a64:92a4e77d0476d762	fnv1a64:05d20700c914ece8
B13	fnv1a64:f4099353e07a2b28	fnv1a64:794b02b3e07c59ea
R01	fnv1a64:64770958f48af9f1	fnv1a64:cdab66b698497287
R02	fnv1a64:413d16016889297e	fnv1a64:ab975470ad6171ec
R03	fnv1a64:c760c3ece428ba25	fnv1a64:ff8d961bac621153
R040	fnv1a64:9cd0743024c99e11	fnv1a64:2b7452027c9b88a7
R050	fnv1a64:67d3ae14e1a57794	fnv1a64:f7ae3995e02a55de
R060	fnv1a64:c336706352209769	fnv1a64:ead2b5bd86755aef
R070	fnv1a64:3c908efedd886eb0	fnv1a64:579796448dab46e2
R080	fnv1a64:c1ac86f64958f461	fnv1a64:2bb9445092ad1ed7
R090	fnv1a64:6f05466aae46be9c	fnv1a64:bb6b3204a2d67056
R100	fnv1a64:c2072484e9b716a6	fnv1a64:1965379a82b98564
R110	fnv1a64:5f23f7f611f7528b	fnv1a64:0875c80c8de71fed
R041	fnv1a64:da42f67df7a1ea28	fnv1a64:5f8465ddf7a418ea
R051	fnv1a64:0a528391e7b69781	fnv1a64:4a0b3a31e5e668f7
R061	fnv1a64:949d644b89b8d052	fnv1a64:e12983b695ffff78
R071	fnv1a64:0349c296472d0f5f	fnv1a64:4f28020b3253e869
R081	fnv1a64:615be700bad3df5c	fnv1a64:eab3e72050022796
R091	fnv1a64:787932512f2585b9	fnv1a64:58cfc016c856ff5f
R101	fnv1a64:58f971b653036aa9	fnv1a64:ef4747e4ab30a5af
R111	fnv1a64:2907bc0e8c6a5bea	fnv1a64:9faacffe41d23640
R12	fnv1a64:92b1a914e054b8aa	fnv1a64:e785687dca663180
R13	fnv1a64:c03c0944a8b66e76	fnv1a64:3ac040d81ac8a894
R14	fnv1a64:bfc98c21fbd8ca5c	fnv1a64:49218c4191071296
R15	fnv1a64:6e20d6edbdec1e38	fnv1a64:a25738e923bf4b1a
O010	fnv1a64:5fc6a0d66abc9eb2	fnv1a64:6fa6637790c4ffd8
O0200	fnv1a64:73f206abfa8719f9	fnv1a64:28157af9b5fe9c1f
O011	fnv1a64:05bf2ceb5fb25be9	fnv1a64:2d5b724594071f6f
O0210	fnv1a64:41730de68b33f60a	fnv1a64:ae12e7c4fd6d76e0
```

Both certificate replays first failed with the expected stale C01 instance
certificate, after emitting complete current cells. C01 and C02 cells for both
hash configurations were copied from `.temp/brief-546/certificates/`, then both
acceptances passed. The four changed cells contain only direct-contract
fingerprint changes:

```text
body	feature	changed direct-contract IDs
C01	hash=true	O010,O011,O0200,O0210
C01	hash=false	O010,O011,O0200,O0210
C02	hash=true	B05,B13,I01,I02,I04,R01,R02
C02	hash=false	B05,B13,I01,I02,I04,R01,R02
```

Body identity, source positions, LocalCovered status, preconditions, strength,
exclusions, build inputs, body/domain fingerprints and instance marks are
unchanged. Full old/new cells are preserved in
[certificate-changes.tsv](../../../.temp/brief-586/certificate-changes.tsv);
replay outputs and both red/green logs are in the same scratch directory. No
fingerprint or certificate hash was invented manually.

### 4. Follow-ups, findings and stopping state

All 22 drafts follow TEMPLATE.md and enumerate their exact owed IDs, source
owners, permitted edits, variants, lifecycle, maintenance and serial gates.
Their union is exactly the 500-entry ledger, without duplicates. The last,
`draft-E1-22-evaluator.md`, requires deletion of owed.tsv, reconciliation/error
paths, the two ledger falsifiers and the nonempty-ledger escape guard, replacing
them with exact declared-entry/executable-descriptor-entry equality. These are
raw director-review drafts, not dispatch authority.

No production semantic defect was found or repaired. Review corrected draft
manifest classifications and the inherited Hash feature inference. The nine
native ZeroMQ checks require the explicit supported skip flag in this sandbox;
their execution/skip status is stated above, not counted as native validation.

Final progress counters: **643 specs reviewed; 6 complete entries; 19 completion/
retirement variants; 12 lifecycle obligations; 209 labels mapped and executed;
6/6 falsifiers observed rejecting; 68 fingerprints and four certificate cells
replayed; ownership inventories exact and unchanged; 436 legacy names preserved.**
The compiling repair attempts advanced coverage/labels/falsifiers/lifecycle or
reconciliation; intermediate compile failures decreased before the next green
milestone. Mandatory successful verification repeats were not tuning attempts.
The checkpoint records milestones and exact next actions.

E1 remains open for the 500 listed role obligations; E2 and external handler
coverage remain outside this step. No claim of exhaustive guest-program safety,
host-provider internals, native ZeroMQ behavior in this sandbox or production
performance certification is made. No requested in-scope code work is left
half-edited. Step A is ready for the director's gate; the sandbox-native exception and remaining E1 debt are explicit.

