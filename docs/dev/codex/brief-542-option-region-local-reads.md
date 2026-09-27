# brief-542 — the option region's local reads: C02 from 34 to 15

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Goal

First of the four steps brief-541 planned
(`docs/dev/codex/brief-541-option-region-conversion-plan.md`, report from line
165; read its full row table). Make C02's **read-only** operations locally
checkable, without changing production:

- the borrowed `let Expr::Call { head, args } = expr else { … }` with a
  diverging `else`, preserving reference layers and cleanup — a borrowed
  pattern moves no field and mints no owner;
- the exact helper predicate `is_system_symbol_name`
  (`runtime/expr_runtime_helpers.rs:420-423`) as an installed contract;
- the `&SharedExpr -> &Expr` coercion as contract **I02**, which is a real
  call to a concrete `Deref` impl, not a free coercion;
- `bool && bool` with short-circuit flow, both branches' ownership states
  joined;
- confirming the existing `len` / index / `Clone` adapters resolve once field
  types are known — no new general method search.

**Predicted end state, written here before any work: C01 17, C02 15**, with
removed rows C02-02, 03, 05, 06, 07, 08, 09, 10, 11, 12, 13, 17, 18, 19, 20, 26,
27, 28, 29. Both raw constructions — `core.rs:101` and `:259` — still reject.
Production coverage stays 0. Any other movement gets a located
reconciliation, never an edited expectation.

## Director rulings this brief carries

- **U2 — keep brief-540's narrow records.** B04, B05, I01, I04 and the P
  records are installed and discharged *for the probe's exact instances*. They
  stay as they are. Their narrow strength must not be read as a quantitative
  proof anywhere else, and every still-`Pending` record remains a non-mark.
- **U4 — no quantitative claim without proof.** Where a contract touches
  `Expr` clone or construction, state only what its evidence establishes.
  Fieldwise `Clone` gives a type, not a capacity receipt.
- **U5 — option-specific workloads are authorized** for later briefs; this
  one is production-read-only and needs none.
- **U1 — the generic `R`** is settled for step 544 (a runtime
  certified-registration domain). It does not affect this brief; do not
  anticipate it.

## Scope

**An objective and a fence, not an enumeration.**

**The objective.** Inside the retention checker
(`crates/rum-stdlib/src/tests/retention_closure/`) and its inventories and
registration, implement exactly the support these C02 operations need,
including tests, oracle controls and contract records — including files the
guidance does not name.

**Guidance.** `local/mod.rs` and `flow.rs` own bindings and ownership;
`surface.rs` operators and indexing; `calls.rs` / `protocol.rs` exact
operation selection; `contracts.rs` and the inventory the nineteen-field
records. If you touch `flow.rs` (about 22.7k characters), split it by semantic
responsibility rather than by line count.

**The disclosure duty.** Every touched path appears in your report with line
ranges and why. An undisclosed edit is the violation; a disclosed necessary
one is not.

**The fence — STOP conditions:** all production Rust; all manifests and the
lockfile; `rum-core`; the interner, gensym, snapshots and `S`; `AGENTS.md`,
`PROGRESS.md`, `TODO.md`. No macro admission, no general `Deref` / `Index` /
trait / generic admission, no `Pending` record used as a mark, no promotion.

## Falsifiers — each watched red, then green

Wrong borrowed field or an extra reference layer; a non-diverging `else`; a
partial move out of the borrowed pattern; a changed concrete `Deref` impl; a
competing imported trait; the `&&` right-hand side's ownership on each branch;
a stale contract; a `Pending` record offered as a mark. And both original
`RET001` controls must remain red. Type and selection claims go through the
oracle's invariant carrier with the same-name negative control.

## Pitfalls

1. `usize == usize` is primitive; `Expr == Expr` is a trait call — a contract.
2. Unknown-versus-`Vec` is not evidence of coercion.
3. A compiler result is reusable only with a complete matching fingerprint.
4. Clone's known type is not a quantitative receipt.

## Verification (MANDATORY)

Director baseline on this tree: 37 checker tests green, C01 17, C02 34;
`cargo test --workspace` 2 635 passed, 0 failed.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Pass `< /dev/null` to anything that runs `rum-cli`. Only C02's count
expectation may change, and only to the predeclared 15. **Microbench A/B: not
applicable** — production is read-only.

Nine native ZeroMQ tests cannot bind loopback in your sandbox; report them as
skipped-by-environment. Do not set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

All scratch, raw output and `PROGRESS.md` go under `.temp/brief-542/`. After
each milestone record changed files, observed counts, falsifiers red/green,
compile state and the exact next action.

## Hard rules

No `git`. No production edits. No new `unsafe`, dependency or nightly. No
traversal — a caller's verdict never reads a callee's body. No general
admission, no `Pending` waiver, no weakened contract, no ignored failure.
Testing Policy is a hard line; using rustc as an oracle for Rust facts about
this repository is a director ruling and not its banned shape. Stop-loss is
keyed on progress: stop only after two consecutive attempts produce neither a
new falsifier nor a lower unresolved count.

## Report format

Changed paths and why; every command with verbatim rows; 17/15 predicted
versus observed, as located rows; each admission or contract and its
falsifier; remaining obligations and anything unsettled. Append to this file
and leave the tree uncommitted.

---

## brief-542 execution report — 2026-09-28

Implemented the read-only checker step. **Predicted C01/C02 17/15; observed 17/15.** C01 is byte-identical to its 17 baseline diagnostic rows. C02 loses exactly the nineteen predicted rows, with no added or altered surviving diagnostic. Both original raw constructions still reject; the four lease probes remain clean; production coverage remains **0**.

No git commands, production edits, new unsafe/dependency/nightly, macro admission, general Deref/Index/trait/generic admission, Pending waiver or promotion. The source checksum audit covers 1,643 original files and permits only the checker/inventory paths below plus this appended report. No microbench A/B applies because production is read-only.

### Changed paths and why

Ranges below are current lines; deleted paths give their original ranges. All paths are relative to `crates/rum-stdlib/src/tests/retention_closure/` unless stated otherwise. The full before/after hunk ranges are also in the saved fence-audit output.

| Path | Changed line ranges | Why |
| --- | --- | --- |
| `compiler_oracle/scratch.py` | 29–42, 56–56 | Refresh only the fenced scratch crate snapshot, so deleted modules cannot survive as untracked compiler inputs; retain its separate build cache. |
| `inventory/local-contracts.tsv` | 2–42 | Install I02 and B13 nineteen-field records; refresh all fingerprints without changing other record content. |
| `local/calls.rs` | 181–184 | Select only the exact &SharedExpr to &Expr contract adjustment for known parameter and argument types. |
| `local/contracts.rs` | 22–22, 43–43, 156–158, 184–185 | Register B13, discharge I02, require their exact effect and executable falsifier. |
| `local/expressions.rs` | 82–83, 85–86, 91–93 | Track divergence separately on each if branch, needed to reject a non-diverging let-else. |
| `local/flow.rs` | original 1–572 removed | Relocated by responsibility into flow/; no remaining implementation here. |
| `local/flow/mod.rs` | 1–111 | Temporal state, hazard checks and branch-state equality, moved from flow.rs. |
| `local/flow/places.rs` | 1–186 | Place ownership, bindings, block cleanup and joins; keep let-else failure ownership separate from success. |
| `local/flow/expressions.rs` | 1–322 | Temporal expression evaluation and captures; join skipped and evaluated && RHS paths. |
| `local/interface.rs` | 12–12, 151–160, 207–207 | Prepare direct I02/B13 declaration evidence at indexing, erase bodies, and fingerprint the new nested directories. |
| `local/marks.rs` | 96–104 | Require prepared direct declaration evidence when looking up the two new read contracts. |
| `local/mod.rs` | 17–17, 291–291, 303–305 | Register reads/ and route only let-else through its borrowed-pattern adapter. |
| `local/oracle.py` | original 1–241 removed | Relocated probe runner and extracted reusable invariant/MIR support. |
| `local/oracle.rs` | original 1–187 removed | Relocated unchanged logical test module into oracle/mod.rs. |
| `local/oracle/mod.rs` | 1–187 | Preserve probe registration, expose offset/JSON helpers, update relative paths. |
| `local/oracle/probe.py` | 1–135 | Preserve existing probe harness with shared support import; disable bytecode before import and use exact scratch refresh. |
| `local/oracle/support.py` | 1–115 | Shared invariant carriers, concrete MIR observation and same-name comparison; add exact I02 adjustment and helper selection. |
| `local/production.rs` | 76–81 | Assert the predeclared 17/15 alongside both original RET001 controls and clean probes. |
| `local/protocol.rs` | 92–92 | Only I02 and B13 are added to the closed borrowed/inert effect table; no new method search. |
| `local/reads/contracts.rs` | 1–58 | Independently validate the two concrete declarations before caller linting. |
| `local/reads/mod.rs` | 1–116 | Exact borrowed Expr::Call pattern, diverging else and I02 coercion; no owner creation. |
| `local/reads/oracle.py` | 1–219 | Stable compiler type/selection controls and private helper/pointer/short-circuit observations in scratch. |
| `local/reads/oracle.rs` | 1–135 | Generate invariant claims from actual C02 sites and require complete-fingerprint oracle evidence. |
| `local/reads/tests.rs` | 1–315 | Three tests with 23 named checker red/green controls for patterns, ownership and read contracts. |
| `local/surface.rs` | 37–37, 39–46, 55–57 | Admit only primitive bool &&, including a diverging RHS; existing usize/Expr equality distinction unchanged. |
| `local/tests.rs` | 218–221, 223–230 | Keep 31 historical discharged-record expectation, separately require I02/B13, and preserve all-record Pending/stale checks. |

This report is the sole documentation change, appended at **lines 129–456** in `docs/dev/codex/brief-542-option-region-local-reads.md`. Scratch infrastructure, checkpoints, source snapshots, compiler copies/mutants, overlays, raw logs, exact command records and the full [scratch path/extent manifest](/home/remi/dev/rum/.temp/brief-542/scratch-paths.tsv) are under `.temp/brief-542/`; none is a production edit. Existing checker writes addressed as `target/brief-*` are physically in `.temp/brief-542/build-upper/` through the read-only filesystem namespace. Cargo cache writes and compiler temporaries are also redirected there. Native networking remains the original sandbox's networking.

`flow.rs` was split before the new logic: temporal state/hazards (~3.5k chars), places/bindings/cleanup/joins (~8.1k), and temporal expression evaluation/captures (~13.1k). Existing bodies moved verbatim apart from required sibling visibility, then the new branches were added. The oracle split separates registration, transport and shared invariant/MIR mechanics. All 37 original test identities remain; four new identities were added. New responsibility directories also reduce pressure on the already crowded local directory and enter the fingerprint inputs.

Size assessment: `calls.rs` (~12.6k), `local/mod.rs` (~13.0k), `flow/expressions.rs` (~13.1k), `reads/tests.rs` (~10.7k), `reads/oracle.py` (~12.1k) and `local/tests.rs` (~14.2k) remain cohesive and below the test guidance's 15k preferred threshold. Future boundaries, if needed, are free versus method selection, driver versus bindings, and oracle transport versus concrete fixtures. No touched Rust implementation remains in the old 20k split-plan range. The nineteen-field TSV is structured data, not a Rust module. Root TODO is fenced and unchanged; these assessments are recorded here.

### Commands and verification

Every recorded shell command, exit status and verbatim output link is in [COMMANDS.md](/home/remi/dev/rum/.temp/brief-542/COMMANDS.md); nested compiler commands have their exact JSON argv and stdout/stderr/status artifacts. [STARTUP-COMMANDS.md](/home/remi/dev/rum/.temp/brief-542/STARTUP-COMMANDS.md) lists the initial read-only discovery commands, whose outputs remain in the conversation rather than separate raw files. Truncated initial displays were followed by narrower reads. All numbered substantive outputs, including failures, were persisted before inspection. Scratch edits used apply_patch; automated splits, refreshes, formatting and report assembly have numbered command records.

The mandatory commands execute inside the scratch-backed filesystem namespace; this changes write placement only. `scripts/run-bounded.py` bounds every long-running process group. Workspace commands inherit `< /dev/null`; `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` is absent and was never set.

**049-fmt-check, exit 0** ([raw](/home/remi/dev/rum/.temp/brief-542/049-fmt-check.raw)):

```bash
python3 scripts/run-bounded.py --timeout 600 -- python3 .temp/brief-542/sandbox.py cargo fmt --check
```

No formatter output; successful exit.

**066g-checker-default-final, exit 0** ([raw](/home/remi/dev/rum/.temp/brief-542/066g-checker-default-final.raw)):

```bash
python3 scripts/run-bounded.py --timeout 1800 -- python3 .temp/brief-542/sandbox.py cargo test -p rum-stdlib retention_closure -- --nocapture
```

```text
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 1027 filtered out; finished in 170.35s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**066f-checker-no-default-final, exit 0** ([raw](/home/remi/dev/rum/.temp/brief-542/066f-checker-no-default-final.raw)):

```bash
python3 scripts/run-bounded.py --timeout 1800 -- python3 .temp/brief-542/sandbox.py cargo test -p rum-stdlib --no-default-features retention_closure
```

```text
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 1021 filtered out; finished in 200.44s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**067-workspace, exit 101** ([raw](/home/remi/dev/rum/.temp/brief-542/067-workspace.raw)):

```bash
python3 scripts/run-bounded.py --timeout 2400 -- python3 .temp/brief-542/sandbox.py cargo test --workspace < /dev/null
```

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.72s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
```

**069-workspace-environment, exit 0** ([raw](/home/remi/dev/rum/.temp/brief-542/069-workspace-environment.raw)):

```bash
python3 scripts/run-bounded.py --timeout 2400 -- python3 .temp/brief-542/sandbox.py cargo test --workspace -- --skip zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket --skip zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets --skip zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip --skip zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages --skip zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata --skip zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown --skip zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply --skip zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end --skip zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages < /dev/null
```

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.10s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 97 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.23s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 1047 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.68s
test result: ok. 1016 passed; 0 failed; 52 ignored; 0 measured; 0 filtered out; finished in 43.18s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.33s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.90s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The unfiltered workspace command stops at the nine native ZeroMQ tests because loopback bind is denied. That command is **not** recorded as green. Per the brief they are classified as **skipped-by-environment**; the follow-up excludes precisely those nine fully qualified names and completes the normally enabled workspace suite: 2,630 passed, 0 failed. The 57 pre-existing ignored tests remain unchanged; no ignored annotation was added. No failure is silently ignored and no skip environment opt-in is used.

```text
zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket	skipped-by-environment: loopback bind denied
zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets	skipped-by-environment: loopback bind denied
zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip	skipped-by-environment: loopback bind denied
zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages	skipped-by-environment: loopback bind denied
zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata	skipped-by-environment: loopback bind denied
zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown	skipped-by-environment: loopback bind denied
zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply	skipped-by-environment: loopback bind denied
zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end	skipped-by-environment: loopback bind denied
zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages	skipped-by-environment: loopback bind denied
```

Workspace totals from the explicit-exclusion run:

```text
passed	2630
failed	0
existing-ignored	57
measured	0
skipped-by-environment	9
```

Implementation corrections, all retained in raw evidence: 018 failed compiling a test fixture's Vec instead of BTreeSet binding; 021 found a valid diverging RHS rejected as a Unit continuation, fixed in boolean typing; 034 proved all type/selection claims but caught an incorrectly constructed system-symbol fixture, corrected with explicit `Expr::system_symbol`/`contextual_symbol`. 042 restored all new tests green. 050/056 passed all 41 checker tests. 061's feature-disabled corpus run then exposed stale deleted flow.rs/oracle.rs files retained by dirs_exist_ok in an inherited scratch snapshot (40 passed, 1 failed; E0761, preserved in 066b). The fenced scratch refresh now removes obsolete compiler inputs; the read harness deliberately seeds an obsolete file and checks its removal. Both complete checker modes were rerun after that fix. Compiler errors from deliberate negative experiments are expected evidence, and the enclosing tests fail if they are missing or unlocated. No contract was weakened to obtain a pass. Stop-loss never reached two consecutive implementation attempts without new falsifier evidence or a reduced unresolved count.

### Predicted versus observed, with located rows

```text
body	predicted	measured
C01	reject	17 rejections
C02	reject	15 rejections
LeaseProbe538::probe	clean	0 rejections
LeaseProbe538::start	clean	0 rejections
LeaseProbe538::drive	clean	0 rejections
LeaseProbe538::resume	clean	0 rejections
production coverage	0	0
```

The following table uses the original brief-541 row IDs and source spans. `removed` means exactly that original diagnostic disappeared; `remains` means the entire original diagnostic row is unchanged. All sites below are in `crates/rum-runtime/src/runtime/options/core.rs`.

| Row | Kind | Original source span | Observed disposition |
| --- | --- | --- | --- |
| C01-01 | RET001 | 101:13-101:23 | remains |
| C01-02 | RET002 | 87:5-94:33 | remains |
| C01-03 | RET002 | 91:9-91:19 | remains |
| C01-04 | RET002 | 95:24-95:65 | remains |
| C01-05 | RET002 | 95:24-95:65 | remains |
| C01-06 | RET002 | 96:23-96:27 | remains |
| C01-07 | RET002 | 96:23-105:10 | remains |
| C01-08 | RET002 | 96:23-105:10 | remains |
| C01-09 | RET002 | 96:23-105:11 | remains |
| C01-10 | RET002 | 98:13-98:22 | remains |
| C01-11 | RET002 | 98:14-98:22 | remains |
| C01-12 | RET002 | 101:13-101:23 | remains |
| C01-13 | RET002 | 102:13-102:20 | remains |
| C01-14 | RET002 | 106:9-106:16 | remains |
| C01-15 | RET002 | 106:9-106:50 | remains |
| C01-16 | RET002 | 106:9-106:50 | remains |
| C01-17 | RET002 | 106:24-106:28 | remains |
| C02-01 | RET001 | 259:21-259:49 | remains |
| C02-02 | RET002 | 249:9-251:11 | removed |
| C02-03 | RET002 | 249:13-249:38 | removed |
| C02-04 | RET002 | 253:9-265:10 | remains |
| C02-05 | RET002 | 253:12-253:54 | removed |
| C02-06 | RET002 | 253:12-253:73 | removed |
| C02-07 | RET002 | 253:34-253:38 | removed |
| C02-08 | RET002 | 253:34-253:38 | removed |
| C02-09 | RET002 | 253:58-253:62 | removed |
| C02-10 | RET002 | 253:58-253:68 | removed |
| C02-11 | RET002 | 253:58-253:68 | removed |
| C02-12 | RET002 | 253:58-253:73 | removed |
| C02-13 | RET002 | 253:72-253:73 | removed |
| C02-14 | RET002 | 254:13-262:14 | remains |
| C02-15 | RET002 | 256:17-256:47 | remains |
| C02-16 | RET002 | 256:17-256:47 | remains |
| C02-17 | RET002 | 256:31-256:35 | removed |
| C02-18 | RET002 | 256:31-256:38 | removed |
| C02-19 | RET002 | 256:31-256:46 | removed |
| C02-20 | RET002 | 256:31-256:46 | removed |
| C02-21 | RET002 | 257:30-260:19 | remains |
| C02-22 | RET002 | 257:33-260:18 | remains |
| C02-23 | RET002 | 258:21-258:40 | remains |
| C02-24 | RET002 | 259:21-259:49 | remains |
| C02-25 | RET002 | 259:21-259:49 | remains |
| C02-26 | RET002 | 259:26-259:30 | removed |
| C02-27 | RET002 | 259:26-259:33 | removed |
| C02-28 | RET002 | 259:26-259:41 | removed |
| C02-29 | RET002 | 259:26-259:41 | removed |
| C02-30 | RET002 | 259:43-259:48 | remains |
| C02-31 | RET002 | 261:28-261:82 | remains |
| C02-32 | RET002 | 261:28-261:82 | remains |
| C02-33 | RET003 | 254:13-262:14 | remains |
| C02-34 | RET003 | 259:21-259:49 | remains |

[Full reconciliation and surviving C02 diagnostics](/home/remi/dev/rum/.temp/brief-542/033-row-reconciliation.raw). C01 is byte-identical, not merely equal in count. The RET001 sites remain `101:13–101:23` and `259:21–259:49`. No unexplained row movement requires an adjusted expectation.

### Admissions, contracts and falsifiers

| Admission / contract | Evidence and failing control |
| --- | --- |
| Borrowed `Expr::Call` let-else | Exact &Expr input and head/args schema; plain bindings produce &SharedExpr/&SharedArgs. Wrong fields/schema, swapped uses, explicit ref, mutable/owned pattern and extra input reference all reject. Invariant rustc claims independently prove the actual field references; eight swapped-field/extra-reference negatives fail at their carriers. |
| Diverging else and cleanup | Non-diverging else, one-returning-path and conditional RHS return reject. A failing branch's move/drop does not retire a success-path raw owner; that control remains RET001. Rustc's non-diverging else E0308 and borrowed partial move E0507 are located. |
| I02 | Installed Test-checked concrete SharedExpr Deref contract; exact Target/signature/body checked independently at indexing, then erased. Pointer identity and actual C02 coercion selected by stable MIR; changed impl, stale and Pending records reject. No general deref search. |
| B13 `is_system_symbol_name` | Separate installed Test-checked helper boundary, not B11. Exact body/signature and direct helper identity pinned; None, matching System name, different name/context and non-symbol cases exercised in copied lexical module. Changed helper, stale/Pending record and inverted predicate fail. No traversal from C02. |
| Primitive `bool && bool` | Skipped and evaluated RHS states joined; continuing ownership disagreement rejects, hazardous RHS retains raw owners, and terminating RHS preserves skipped-path ownership. Both runtime truth paths observed; conditional owner use fails rustc E0382. Existing primitive usize equality remains separate from contracted Expr equality. |
| Existing len/index/Clone instances | I01/B05/I04/B04 resolve once borrowed fields have exact types. The imported competing trait rejects; the invariant/MIR oracle distinguishes five same-name negative definitions and a same-owner competing Clone trait. A fresh Expr clone held across a call remains raw/unleased. |
| Existing rejection fence | All Pending records remain non-marks, both original RET001 controls remain red, both macros remain rejected, and the four existing lease probes stay clean. |

I02/B13 each have all nineteen fields in the inventory. Their effect is only `borrows;inert`: no owned output, no ownership transfer, and no capacity/clone accounting receipt. Body and field checks happen as independent flat index evidence; the caller receives signatures, schemas and marks, never a callee body. Fingerprints include every crate Rust/Python/TOML input and declared dependencies; oracle reuse additionally keys the claims, inventories, feature set, manifests, toolchain and harness inputs. The new nested modules are included. Fieldwise Clone establishes a type, never a quantitative receipt.

U2 is preserved: B04/B05/I01/I04 and every P record retain their first eighteen fields byte-for-byte; only fingerprints refresh. The original 31 installed-record cohort keeps its expectation, while the exact additional set {I02, B13} is independently asserted. This reconciles the necessary two-record addition without changing the old cohort or granting any other Pending record a mark: 33 Discharged, 8 Pending. This inventory movement was identified before implementation in the checkpoint, not fitted to a failed assertion.

Verbatim final compiler observations (both feature modes run the same controls):

`hash`:

```text
READ542_SCRATCH stale_input=removed
READ542_TYPES positive=pass invariant=true negative=E0308 located=26
READ542_FIELD_TYPES swapped_fields=E0308 extra_reference=E0308 located=8
READ542_RESOLUTION claims=12 positive=pass same_name_negatives=5 negative=pass competing_clone=distinct
READ542_RUST_CONTROL nondiverging-else negative=E0308 located=true
READ542_RUST_CONTROL borrowed-partial-move negative=E0507 located=true
READ542_RUST_CONTROL rhs-move negative=E0382 located=true
READ542_CONTRACTS helper_table=pass pointer_identity=pass bool_branches=pass pointer_negative=fail helper_negative=fail
READ542_RESTORED invariant_types=pass contracts=pass after_all_negatives=true
```

`no-default-features`:

```text
READ542_SCRATCH stale_input=removed
READ542_TYPES positive=pass invariant=true negative=E0308 located=26
READ542_FIELD_TYPES swapped_fields=E0308 extra_reference=E0308 located=8
READ542_RESOLUTION claims=12 positive=pass same_name_negatives=5 negative=pass competing_clone=distinct
READ542_RUST_CONTROL nondiverging-else negative=E0308 located=true
READ542_RUST_CONTROL borrowed-partial-move negative=E0507 located=true
READ542_RUST_CONTROL rhs-move negative=E0382 located=true
READ542_CONTRACTS helper_table=pass pointer_identity=pass bool_branches=pass pointer_negative=fail helper_negative=fail
READ542_RESTORED invariant_types=pass contracts=pass after_all_negatives=true
```

Verbatim checker control rows:

```text
READ542_CONTROL B13-pending red=reject green=clean
READ542_CONTROL B13-stale red=reject green=clean
READ542_CONTROL I02-pending red=reject green=clean
READ542_CONTROL I02-stale red=reject green=clean
READ542_CONTROL changed-deref red=reject green=clean
READ542_CONTROL changed-field-schema red=reject green=clean
READ542_CONTROL changed-helper red=reject green=clean
READ542_CONTROL conditional-rhs-return red=reject green=clean
READ542_CONTROL else-cleanup-keeps-success-owner red=reject green=clean
READ542_CONTROL explicit-ref red=reject green=clean
READ542_CONTROL extra-reference red=reject green=clean
READ542_CONTROL field-type-mismatch red=reject green=clean
READ542_CONTROL imported-trait red=reject green=clean
READ542_CONTROL nondiverging-else red=reject green=clean
READ542_CONTROL one-else-path-returns red=reject green=clean
READ542_CONTROL owned-clone-no-receipt red=reject green=clean
READ542_CONTROL owned-pattern red=reject green=clean
READ542_CONTROL partial-move red=reject green=clean
READ542_CONTROL rhs-hazard red=reject green=clean
READ542_CONTROL rhs-moves-owner red=reject green=clean
READ542_CONTROL rhs-new-owner red=reject green=clean
READ542_CONTROL rhs-return-keeps-skipped-owner red=reject green=clean
READ542_CONTROL wrong-field red=reject green=clean
```

### Remaining obligations and limits

All 17 C01 rows remain: the raw accumulator, open R/registration-domain work, receiver reborrows, defaults and their type/selection cascades. C02's 15 remaining rows are exactly its raw rule-child vector; macro typing/selection and cascading result/value types; continuation construction/schema; and Pending B10/B11. No later R-domain work or production conversion was anticipated. The consumer obligations and promoted inventory are unchanged.

These are local Rust read/selection facts and narrow installed boundary evidence. They grant no quantitative Expr construction/clone bound, no coverage for callers or escaping consumer copies, and no production retention certificate. Production coverage is still zero. No unresolved implementation discrepancy remains within this step; the nine native network cases remain unexecuted because of the environment. The tree is left uncommitted.
