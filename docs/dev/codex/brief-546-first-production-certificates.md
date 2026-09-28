# brief-546 — record the first production certificates

Implementation brief. **Do not run `git` in any form.** Everything you need
about the tree's state is in this brief.

## Goal

The option region is converted, lints clean, and no longer costs anything
unarmed (`7d7680de`, `cae6f550`). Record what it has earned: **the first
production coverage this line has produced** — two production body rows, C01
(`Session::merged_option_exprs_for_symbol`) and C02
(`Session::evaluate_runtime_option_expr`), each with its finite instance
certificates attached.

This is the last of brief-541's steps. Its draft (§7 "Draft brief-545 — record
the first local production certificates") and its §6 "What promotion
certifies — and does not" are in
`docs/dev/codex/brief-541-option-region-conversion-plan.md` from line 165.
**Read both in full; they apply**, with the director's additions below.

**Predicted end state, written before any work:** C01 0 and C02 0 for the
declared instances; **exactly two production body rows** in `promoted.tsv`;
the four `UnresolvedConsumer` rows unchanged. If a helper genuinely earns its
own production mark, disclose and count it separately — never hide it inside
a region mark to keep the number at two, and never suppress it.

## What a certificate records

Mint a mark only after that body and instance pass their own lint in this run.
Each production row records: the source/body fingerprint; the exact build,
accounting mode and `R` instance; its direct covered-mark and contract
dependencies; preconditions; contract strengths; and **exclusions**, stated in
the row, not only in prose elsewhere. The exclusions include at least:

- its callers and the whole option consumer — coverage is local;
- arbitrary external `R` — and specifically **the limit brief-544 found**: the
  certified-domain installer is public, so a host can install its own
  declaration and assert its own trust; that is outside this repository's
  certificates;
- guest- and host-private allocation, process RSS and allocator overhead,
  interner storage, stack-safe destruction of deep `Expr`s, and any
  universal allocation bound derived from tests;
- the four escaping consumer copies.

## The four consumer obligations stay open

The four `UnresolvedConsumer` rows in
`inventory/consumer-obligations.tsv` are preserved exactly as they stand in
the current tree — their anchors moved in brief-544, so use the file, not the
older copy in brief-541's §6. A required falsifier must reject changing any of
them to covered merely because C01 and C02 are promoted: a clean callee does
not launder its caller.

## Scope

**An objective and a fence, with a disclosure duty.**

**The objective.** Enable earned production marks for the exact declared
instances, implement durable inventory validation and its falsifiers, and
state the exact guarantee in `docs/testing-strategy.md`. **Guidance:**
`local/marks.rs` (it refuses production today), `local/production.rs` and
`retention_closure/mod.rs` (they assert an empty promotion), `interface.rs`
(fingerprints), and the `inventory/` records. Replace the unconditional
empty-inventory and zero-production assertions with exact expected-row checks
that still refuse shrinkage, staleness, redirection and absence.

**The disclosure duty.** Every touched path appears in your report with line
ranges and why.

**The fence — STOP conditions:** production Rust and its performance are
read-only; no new admission or contract, no broader domain, no caller
promotion, no traversal, no consumer-obligation discharge; no `rum-core`,
interner, gensym, snapshot or `S` change; no manifests or lockfile; no
`AGENTS.md`, `PROGRESS.md`, `TODO.md`.

## Falsifiers — watched red, then green

A raw local held across a guest call; a missing, stale or `Pending` mark; an
expired body or domain fingerprint; a removed or redirected promotion; a
runtime certified set that no longer matches the declaration; an omitted
declared instance; an escaping-copy row falsely marked covered. Replay the
fresh C01/C02 certificates and every `LeaseProbe538` check.

## Verification (MANDATORY)

Director baseline: 45 checker tests green (one ignored entry is a fingerprint
regeneration tool), C01 0 for both instances, C02 0; workspace 2 671 passed,
0 failed.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

**No new A/B**: production is read-only. Cite the measurement artifacts of
brief-543, brief-544 and brief-545 by path rather than producing new numbers.
Nine native ZeroMQ tests cannot bind loopback in your sandbox; report them as
skipped-by-environment and do not set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

All scratch under `.temp/brief-546/`. Update `.temp/brief-546/PROGRESS.md`
after the certificate plumbing, the inventory reconciliation, the
documentation, and the gate.

## Hard rules

No `git`, no production edits, no new `unsafe`, dependency or nightly, no
`Pending` waiver, no generalised admission, no traversal, no weakened test.
Refuse stale or absent evidence. Do not mark the goal achieved if C01 passes
only by omitting unproved executions from a universal claim. Stop-loss keyed
on progress, as the template states: failing required tests, rejection counts
and compiler errors falling are all progress.

## Report format

Changed paths and ranges; every command with verbatim output; the
certificate table — exactly what each row certifies and excludes; the four
`UnresolvedConsumer` rows unchanged; falsifier outcomes; the cited
measurement artifacts; remaining limits. Append to this file and leave the
tree uncommitted.

---

## brief-546 execution report — 2026-09-28

The first local production coverage is recorded: exactly two body rows, C01 and C02. C01 has two concrete mode/R certificates in each feature build; C02 has one nongeneric certificate in each build. All six recorded instances lint clean in their own current runs. Four LeaseProbe538 bodies remain independently earned and clean. No helper body is promoted; existing helper contracts remain contracts. The four escaping consumer rows are byte-identical.

**Execution deviation:** the initial parallel inspection, before the brief had been read, included `git status --short` in `pwd && rg --files -g AGENTS.md -g '!target' -g '!vendor' && git status --short`. It returned no status rows. This violated the explicit no-git rule. No further git command was run, and the tree is left uncommitted. This run must not be described as exact compliance with every Hard rule. The initial read output was persisted retrospectively in `.temp/brief-546/000-brief.raw` through `003-discovery.raw`; the full initial guide/plan output is reconstructed from unchanged input files after tool-display truncation. The bootstrap is disclosed in `startup-audit.md`.

### Changed paths and ranges

All paths below are relative to the repository root. The full baseline-relative patch and precise ranges are in [source-changes.diff](../../../.temp/brief-546/source-changes.diff) and [changed-paths.json](../../../.temp/brief-546/changed-paths.json).

| Path | Current line ranges | Why |
| --- | --- | --- |
| `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs` | 210, 212 | Replace the historical empty-promotion assertion with fresh exact two-body replay; preserve all frontier rejection counts. |
| `crates/rum-stdlib/src/tests/retention_closure/consumers.rs` | 49, 103–113 | Expose the existing consumer reconciliation to promotion checks and challenge Covered status independently for all four rows. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 2–69 | Refresh only the 68 fingerprint fields after checker changes; all contract identities, statuses, guarantees and evidence requirements are unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv` | 27–28 | Change only C01/C02 from Pending to LocalCovered; all 315 caller/catalog obligations remain Pending. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 1–3 | Record exactly two body rows with six independently earned build/instance certificates and explicit preconditions, strengths and exclusions. |
| `crates/rum-stdlib/src/tests/retention_closure/local/certificates.rs` | 1–167 | Define finite production instance keys and exact body/build/domain stamps; reconcile the repository declaration with actual runtime installation. |
| `crates/rum-stdlib/src/tests/retention_closure/local/interface.rs` | 15–16, 172–177, 216–217 | Attach production domain/build facts at indexing and use checkout-relative checker paths in persistent fingerprints. |
| `crates/rum-stdlib/src/tests/retention_closure/local/marks.rs` | 12, 59–64, 68–72, 102–126, 132, 160–192, 201, 205–216 | Mint only successful exact production instances; validate current bodies and flat direct mark/contract dependencies, including revocation to Pending. |
| `crates/rum-stdlib/src/tests/retention_closure/local/mod.rs` | 7, 19–20, 33 | Register certificate, inventory and falsifier modules and expose exact promotion replay to existing gates. |
| `crates/rum-stdlib/src/tests/retention_closure/local/option_domain.rs` | 242 | Expose the existing reconciliation function to the certificate validator; no new domain or admission. |
| `crates/rum-stdlib/src/tests/retention_closure/local/production.rs` | 1–2, 4, 8, 11–12, 14–32, 34–37, 42, 45–48, 51–52, 54–55, 57–98 | Replace report-only/zero-coverage acceptance with current production certificates, separate probe replay and exact inventory validation. |
| `crates/rum-stdlib/src/tests/retention_closure/local/promotion.rs` | 1–183 | Render complete finite certificates and reject absent, stale, redirected, incomplete or broader inventory claims. |
| `crates/rum-stdlib/src/tests/retention_closure/local/promotion_tests.rs` | 1–326 | Four durable falsifier tests exercise issuance/revocation, raw retention, fingerprints, inventory, runtime drift and all four escaping consumers. |
| `crates/rum-stdlib/src/tests/retention_closure/mod.rs` | 1, 123–131, 155–158 | Reconcile exactly two local production obligations/promotions while preserving pending catalog obligations and historical measurements. |
| `docs/testing-strategy.md` | 129–141, 143–149, 151–162, 164–209, 281–282, 318–320 | State the exact local guarantee, finite build/instance domain, contract strengths, public-installer trust seam and exclusions; replace superseded zero-coverage claims. |
| `docs/dev/codex/brief-546-first-production-certificates.md` | 124–382 | Required appended execution report. |

The new certificate and inventory modules are each under 10k characters; the new falsifier file is under the 15k test-file guideline. Existing `local/mod.rs` and `local/option_domain.rs` remain in the 10k–20k range: these edits only register modules/expose reconciliation, so their coherent lint/domain responsibilities are retained. The root test module remains under the 25k test warning ceiling; its change updates the obligation gate. Future splits would separate lint execution from grammar support, domain parsing from its drift tests, and historical census from fixture tests. TSV inventories are records, not Rust module-size debt. No root TODO or administrative document was changed.

All authored scratch, source baselines, checkpoints, raw output, review copies, falsifier evidence and generated compiler/build/cache artifacts are physically under `.temp/brief-546/`. The [scratch manifest](../../../.temp/brief-546/scratch-manifest.tsv) enumerates their paths and text ranges or binary sizes. Cargo and existing compiler probes run in a read-only filesystem namespace with writable overlays beneath that directory; printed `target/brief-*` and older `.temp/brief-*` paths are namespace views, not writes to the original locations. [PROGRESS.md](../../../.temp/brief-546/PROGRESS.md) records each required milestone. Production Rust, performance code, manifests, lockfile, core/interner/gensym/snapshot/S and protected administrative files remain unchanged.

### Commands and verification

[COMMANDS.md](../../../.temp/brief-546/COMMANDS.md) lists every recorded command with its exit status and linked unabridged verbatim output. Every bounded command has a `.command`, `.raw` and `.exit` record. The normal gate payloads run through `python3 .temp/brief-546/isolate.py`; `run.py` delegates to `scripts/run-bounded.py` with the required 600/1800/2400-second per-command limits. No network, new dependency, nightly, unsafe or elevated execution was used. No new A/B was run.

**Formatting**, exit 0. [Full verbatim output](../../../.temp/brief-546/094-final-fmt.raw).

```bash
/usr/bin/python3 scripts/run-bounded.py --timeout 600 -- /usr/bin/python3 .temp/brief-546/isolate.py cargo fmt --check
```

Verbatim stdout/stderr: empty.

**Default checker**, exit 0. [Full verbatim output](../../../.temp/brief-546/095-final-checker-default.raw).

```bash
/usr/bin/python3 scripts/run-bounded.py --timeout 1800 -- /usr/bin/python3 .temp/brief-546/isolate.py cargo test -p rum-stdlib retention_closure -- --nocapture
```

```text
test result: ok. 49 passed; 0 failed; 1 ignored; 0 measured; 1028 filtered out; finished in 232.96s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**No-default-features checker**, exit 0. [Full verbatim output](../../../.temp/brief-546/096-final-checker-no-default.raw).

```bash
/usr/bin/python3 scripts/run-bounded.py --timeout 1800 -- /usr/bin/python3 .temp/brief-546/isolate.py cargo test -p rum-stdlib --no-default-features retention_closure
```

```text
test result: ok. 49 passed; 0 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 257.94s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Mandatory workspace command**, exit 101. [Full verbatim output](../../../.temp/brief-546/097-final-workspace.raw).

```bash
/usr/bin/python3 scripts/run-bounded.py --timeout 2400 -- /usr/bin/python3 .temp/brief-546/isolate.py cargo test --workspace
```

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.90s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
error: test failed, to rerun pass `-p rum-jupyter --lib`
```

**Complete workspace continuation**, exit 101. [Full verbatim output](../../../.temp/brief-546/098-final-workspace-all.raw).

```bash
/usr/bin/python3 scripts/run-bounded.py --timeout 2400 -- /usr/bin/python3 .temp/brief-546/isolate.py cargo test --workspace --no-fail-fast
```

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.15s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.79s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
error: test failed, to rerun pass `-p rum-jupyter --lib`
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 1069 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.99s
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 233.92s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.49s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.76s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.92s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 1 target failed:
```

Complete workspace totals: **2666 passed, 9 raw failed, 58 existing ignored**. The nine native ZeroMQ failures below are **skipped-by-environment**: the sandbox refuses loopback socket binding. Their raw failures and exit 101 are retained, not relabeled as successful executions. `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` was not set. The no-fail-fast continuation reaches every remaining package and doctest.

```text
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
    zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket
    zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets
    zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip
    zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages
    zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata
    zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown
    zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply
    zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end
    zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages
```

Intermediate outcomes are preserved: baseline `015` was 45 passed/0 failed/1 ignored; `037` and `040` proved all local instances/probes clean then rejected absent inventory. `047` was a namespace setup collision before Cargo, corrected by waiting for fmt to exit. `048` was 48 passed/1 failed/1 ignored at the last historical empty-promotion assertion; its replacement keeps all census counts exact. `069` was the deliberately observed earned-Pending revocation failure. The validator extraction initially had three missing-identity compiler errors (`073`, repeated by the prematurely started `075/077` pipeline); `085` fixed them and `087` compiled successfully. Final certificate refreshes `091/092` rejected stale cells before `093` attached the new independently earned cells. Searches `009`/`039` include a missing guessed path; no missing artifact was used as evidence. Scratch manifest generation first encountered unreadable Rust incremental lock files (`118`); the manifest records their paths and sizes without trying to read protected cache contents. Compiler-oracle negative Rust diagnostics are expected falsifier output, not failed checker tests.

### Certificate table and exact limits

| Production body | Certified instances in each of the two builds | Direct covered dependencies | Direct contract dependencies |
| --- | --- | --- | --- |
| C01 `rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol` | `Unarmed × OptionConsumer`, `AccountingShadow × OptionConsumer`; `hash=true` and `hash=false` | None | O010, O011, O0200, O0210 |
| C02 `rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr` | Nongeneric body; no R parameter; existing rule contracts dispatch Unarmed/AccountingShadow; `hash=true` and `hash=false` | None | B05, B13, I01, I02, I04, R01, R02 |

Exact build: the recorded Linux x86_64, 64-bit, debug Rust 1.85.0 builds. Each attached instance contains full compiler/build identity, body/context and domain fingerprints, its own mark stamp, and direct dependency stamps plus their preconditions, guarantees and strengths. These are three instance marks per active build, six persisted certificates under two body rows; no generic/all-R C01 mark exists.

| Body | Feature | Body/context fingerprint | Domain fingerprint | Instance mark fingerprints |
| --- | --- | --- | --- | --- |
| C01 | hash=true | fnv1a64:21e2d1806dc149ba | fnv1a64:f1b1b5ec1f9ab7c9 | fnv1a64:c77d4bcb761d2a31, fnv1a64:6b4796131c512fcb |
| C01 | hash=false | fnv1a64:fdbc1ffa40ebc9d7 | fnv1a64:ed48860e2b2c615f | fnv1a64:edab0e6abdee2ee7, fnv1a64:a94f94777b07873d |
| C02 | hash=true | fnv1a64:a800135762aa3ad4 | fnv1a64:f1b1b5ec1f9ab7c9 | fnv1a64:d84982c77aa9794c |
| C02 | hash=false | fnv1a64:085afca365ca9f1d | fnv1a64:ed48860e2b2c615f | fnv1a64:de0fb3fac6454bff |

Both production rows explicitly contain the following preconditions, strength and exclusions (verbatim fields):

**Preconditions:** exact recorded build and instance; current own-body lint and direct marks; repository declaration equals compiled and installed runtime domain and registrations; sealed accounting modes; checked concrete Copy/FrameRootTrace descriptor; private typed owners and borrowed inputs; each direct contract caller precondition remains required.

**Strength:** local retention across may-run-guest/may-grow calls and branch/error cleanup within admitted syntax; exact resolution and current direct marks; typed lease/frame transfer only at discharged boundaries; Runtime-enforced or Test-checked as recorded per contract; no inferred receipt from a return type.

**Exclusions:** local only; callers and whole option consumer; arbitrary external R and host-installed declarations asserting host trust through the public certified-domain installer; guest-private and host-private allocation; process RSS and allocator overhead; interner storage; stack-safe destruction of deep Expr; universal allocation bounds derived from tests; between-checkpoint peaks; four escaping consumer copies; remaining N class; P bound unchanged.

In particular, the public certified-domain installer lets a host install its own declaration and assert its own trust. Those host declarations and arbitrary external R are outside these certificates. Snapshot/driver/rule/read contracts remain separately scoped; this change introduces no new admission or discharged record. All 68 contract records retain their original first eighteen fields: 62 Discharged and six Pending (B01/B02/B06/B07/B08/I03).

The fresh acceptance tables are verbatim:

```text
body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```

```text
body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```

### Four unchanged consumer obligations

SHA256 before/after: `9db426c2da12e58b3eff82e86a7e24f3b0159c00fce80f14be429af711559c42`.

```text
# id	status	function	source-anchor	copy-expression	retained-quantity	boundary
D3-predicates-same-test	UnresolvedConsumer	rum_stdlib::predicates::collections::effective_same_test	crates/rum-stdlib/src/predicates/collections.rs:471	value . clone ()	option-derived Expr occurrence escapes the option collection	D3; not discharged by C01/C02 or PUBLIC-outgoing
D3-sets-same-test	UnresolvedConsumer	rum_stdlib::structure_ops::ordering_sets::effective_same_test	crates/rum-stdlib/src/structure_ops/ordering_sets.rs:593	value . clone ()	option-derived Expr occurrence escapes the option collection	D3; not discharged by C01/C02 or PUBLIC-outgoing
D3-align-columns-separator	UnresolvedConsumer	rum_stdlib::strings::scripting::text_shape::parse_align_columns_options	crates/rum-stdlib/src/strings/scripting/text_shape.rs:284	value . to_string ()	option-derived separator String escapes the option collection	D3; not discharged by C01/C02 or PUBLIC-outgoing
D3-url-parse-scheme	UnresolvedConsumer	rum_stdlib::strings::scripting::url::parse_url_parse_options	crates/rum-stdlib/src/strings/scripting/url.rs:245	default_scheme . map (str :: to_owned)	option-derived scheme String escapes the option collection	D3; not discharged by C01/C02 or PUBLIC-outgoing
```

### Falsifier outcomes

All four new required tests pass in both feature gates, with 61 rejecting/restored-control records per build. The controls mutate in-memory source/index/mark/inventory/runtime inputs, require a located rejection, restore the original input and require success; original production files are untouched. Per-case RED/GREEN verdicts are in [falsifiers/](../../../.temp/brief-546/falsifiers/) and the default gate raw output.

| Falsifier | Observed result |
| --- | --- |
| Raw local across a guest/growth call in C01 and C02 | RET001 at the live-across-call obligation; restoration earns each body. |
| Missing/stale/Pending direct contracts | Both issuance and already-earned mark lookup reject; restoration succeeds. `069` first failed specifically for Pending O0210; the final gate passes the same assertion after the flat validation fix. |
| Expired body, declaration/domain source or build fingerprint | Old marks reject; original indexed evidence validates. |
| Absent/removed/redirected/stale promotion; removed other-build cell; changed exclusions | Inventory replay rejects; restoring exactly two rows succeeds. |
| Omitted declared instance | Every active instance is independently removed and rejected in both builds. |
| Runtime certified set differs from declaration | Empty runtime installed set rejects; reinstalling the repository declaration succeeds. Existing declaration/impl/derive/installer/key controls also replay. |
| Caller or escaping-copy row falsely covered | Caller obligation and each of the four UnresolvedConsumer rows reject despite clean callee certificates; restoring Pending/UnresolvedConsumer succeeds. |
| Universal/all-R claim | Unspecialized C01 lint rejects and no generic C01 lookup exists; finite declared instances pass. |
| LeaseProbe538 | All four bodies remain clean and independently marked; existing protocol, compiler, rule, read and ownership controls replay. |

### Reused measurements; no new A/B

The existing measurement artifacts were checked by path and content identity, not rerun. No new allocation, instruction or timing claim is made.

- brief-543: [performance-summary.json](../../../.temp/brief-543/performance-summary.json), [performance-table.md](../../../.temp/brief-543/performance-table.md), [base timings.tsv](../../../.temp/brief-543/base/timing/timings.tsv), [candidate timings.tsv](../../../.temp/brief-543/candidate/timing/timings.tsv), [fresh binary identities](../../../.temp/brief-543/fresh-binary-sha256.json), `base-source-sha256.json` and `candidate-source-sha256.json`. All eight recorded binaries match.
- brief-544: [measurement identity](../../../.temp/brief-544/measurement-identity.json), [final measurement audit](../../../.temp/brief-544/final-measurement-audit.json), [timing-results.tsv](../../../.temp/brief-544/timing-results.tsv), [base timings.tsv](../../../.temp/brief-544/base/timing/timings.tsv), [candidate timings.tsv](../../../.temp/brief-544/candidate/timing/timings.tsv). Of 187 pinned inputs, 186 still match; the sole parser change matches the existing [documented identity recovery](../../../.temp/brief-544/measurement-identity-recovery.json), with no changed workload or binary.
- brief-545: [candidate22 summary](../../../.temp/brief-545/candidate22-summary.json), [candidate22 timing comparison](../../../.temp/brief-545/candidate22-timing-comparison.tsv), [timings.tsv](../../../.temp/brief-545/candidate22/timing/timings.tsv), [pin identity](../../../.temp/brief-545/pin-identity.json), [attempt22 binary identities](../../../.temp/brief-545/attempt22-binary-sha256.json), [candidate22 runtime source identities](../../../.temp/brief-545/candidate22-runtime-source-sha256.json). All 183 pinned inputs and four binaries match; all 461 runtime source files match the measured candidate22.

The checks are saved in [reused-measurement-inputs.json](../../../.temp/brief-546/reused-measurement-inputs.json) and [production-performance-identity.json](../../../.temp/brief-546/production-performance-identity.json). Historical measurement limits and residuals are unchanged.

### Remaining limits and stop-loss

Coverage is local, finite, build-specific and contract-scoped. It establishes neither coverage of arbitrary host executions nor an all-input memory/whole-program theorem. Caller/catalog obligations and all four escaping consumer copies remain open; no traversal or consumer discharge was introduced. The source and certificate gates progressed to the final results above; no two compiled no-progress attempts occurred. The earlier startup no-git violation and the nine sandbox-native omissions remain explicit execution limits. The final patch is uncommitted.
