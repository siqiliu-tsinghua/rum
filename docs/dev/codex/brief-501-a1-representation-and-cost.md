# brief-501 — swap the representation, and measure what A1 costs

## Goal

This is the measurement the whole spike exists for. Replace `Symbol`'s
`&'static SymbolText` with an `Arc`-backed owned representation and a content
fingerprint — A1's representation, **not** its ownership — and return the two
numbers that decide the line: the candidate type sizes, and the instruction
cost against today's kernel.

**Nothing here lands.** Leave the tree uncommitted on this branch. Report
numbers, not a decision; the director chooses A1 or the A2 fallback.

## Where the tree is

Branch `spike/brief-499-symbol-non-copy` at `a9dbc0c0`. `Symbol` is already
non-`Copy` and the workspace compiles, `cargo fmt --check` is green, and the
whole ownership migration is done (briefs 499-500: 92 files, a 35-type
`Copy`-loss closure). **`main` is clean at `3024c33c` and is the A/B base arm
— do not switch, merge, rebase, or commit.**

What is still exactly as it was on `main`: the process-global leaking
interner, the dense `u32` id, `PartialEq`/`Hash`/`Ord`, `interned_id`, serde,
and every memory-accounting term. Those are this brief's subject.

## Still not in scope — the ownership half

Keep the interner **process-global and still leaking**. Do not build a
session-owned interner, a reclamation path on `reset()`, a canonical-pool
redesign, escaped-value lifetime handling, or any memory-ledger accounting.

The reason is the measurement, not the effort: A1's cost is the
representation, and A1's benefit is the ownership. If the representation alone
is too expensive, A1 falls on this number and the ownership plumbing never has
to exist. Building both at once makes the number unattributable.

## Stage A — the representation

```rust
pub struct SymbolText { fingerprint: u64, context: String, name: String }
pub struct Symbol { fingerprint: u64, identity: Option<NonZeroU64>, text: Arc<SymbolText> }
```

- The interner becomes a map to **strong** `Arc<SymbolText>`. It still never
  reclaims. That is deliberate.
- **The fingerprint is a fixed, seedless, process-stable hash** of
  `(context, name)` — FNV-1a or equivalent written inline. No `RandomState`,
  no `DefaultHasher`, nothing whose value varies between processes. The
  2026-09-02 ruling redefines `interned_id` as a stable content fingerprint,
  and that promise is *stronger* than today's: same text ⇒ same value across
  processes, not merely within one.
- `eq`: `identity` equal AND (`Arc::ptr_eq` fast accept OR fingerprint equal
  AND context and name equal). **A fingerprint match is never on its own an
  accept.**
- `hash`: fingerprint and identity, mixed exactly as today
  (`crates/rum-core/src/expr.rs:258-263`), so the distribution the stability
  bloom depends on is unchanged.
- **`Ord` is the correctness trap of this brief.** Today's fast path returns
  on `id` equality (`:271-285`). Under a fingerprint that is wrong: a
  collision would order two *different* symbols as `Equal` and silently
  corrupt every sorted structure, every `BTreeMap` keyed by `Symbol`, and
  canonical ordering. The fast path must become `Arc::ptr_eq`; the fallback
  stays exactly `context.cmp` → `name.cmp` → `identity.cmp`. Ordering must not
  move by one pair.
- `interned_id()` keeps its signature, returns the low 32 bits of the
  fingerprint, and its rustdoc is rewritten so it no longer claims a dense id.
  Its only production consumer (`runtime/evaluation_cache/stability.rs:70`)
  multiplies by the golden-ratio constant, which is Fibonacci hashing and is
  unharmed by receiving a hash.
- Delete the `u32::try_from(pairs.len()).expect(...)` panic path at `:88`; a
  fingerprint has no successor to it.
- Serde stays text-based and byte-identical (`:287-320`).
- `canonical_system_symbols` stays a process-static map, now of `Arc`-backed
  symbols, so `Symbol::system` costs one `Arc` clone per lookup. **That cost
  is part of what you are measuring — do not special-case it away**, and do
  not add a `&'static`-versus-`Arc` enum to dodge it. If the number comes back
  bad, that lever is the director's to pull, not yours.
- **No `unsafe`, for any reason, including to preserve a layout.**

Then repair the workspace to compiling. The ownership migration is already
done, so this fallout should be small and confined to `rum-core`; if it is
not, that is itself worth reporting.

**One more duty, specific to this change.** Briefs 499-500 inserted about a
hundred `Symbol` clones that were free on a `Copy` layout and become atomic
increments now. Walk the ones that sit on a path that runs **when profiling is
OFF** and convert any to a borrow where a borrow fits. The diagnostics hard
line (`AGENTS.md`) is that off-by-default machinery costs nothing on the
default path; an atomic increment is a cost. List every one you changed, and
every one you judged unavoidable.

## Stage B — the sizes

Report, for **all three arms**: `size_of::<Symbol>()`, `size_of::<Atom>()`,
`size_of::<Expr>()`, `size_of::<SharedExpr>()`. Measure them; do not
transcribe the 24/32 bytes recorded elsewhere in this directory.

**If `Expr` grows, stop and report before measuring anything else.**
`size_of::<Expr>()` is read by the memory model at a dozen stdlib accounting
sites, so a growth moves the ceiling's own arithmetic. That is a director
decision, not a cost to absorb.

## Stage C — the cost

### Three arms, and why three

The director has built the first two release binaries, both **at this same
path**, and left them at:

- `target/brief-501/rum-base` — `main`, `3024c33c`. Today's kernel.
- `target/brief-501/rum-stage1` — `a9dbc0c0`. Non-`Copy`, old representation.

You build the third from the working tree, release profile, in place.

`base → stage1` isolates what the ownership migration alone costs, which
theory says is zero because a `Symbol` clone is a 24-byte `memcpy` on that
layout. `stage1 → candidate` isolates the representation. `base → candidate`
is the total, and **the total is what decides A1** — the migration is not
separable from A1 in practice. Report all three deltas. If `base → stage1` is
not ~0, say so loudly; it would mean the migration itself moved something and
every other number needs re-reading.

### The instrument

```
taskset -c 2 perf stat -e instructions:u -x, -- <binary> <workload>.rum < /dev/null
```

Read `cpu_core/instructions/u`, require `100.00` multiplex coverage, three
runs per arm per workload, profiling env OFF, stdin closed or the probes hang.
Judge every delta against the observed within-arm spread; this recipe repeats
to about 0.003% on this host, while the timing panel's own untouched controls
have moved +3.70% to +12.45%. **The panel is not the instrument for this
question** — do not run it.

### Calibration the director already did — read before writing a workload

**The iteration ceiling will silently eat your probe.** A `Do` over 200 000
iterations dies with `evaluation exceeded iteration work limit (100000)` and
the process exits 1 after ~3.4M instructions, which is startup. Measured that
way, base and candidate differ by nothing and the arm difference is noise.
This project's records show this same instrument lying three times before.

So: **verify the instruction count scales with `N` before trusting any
number.** On `target/brief-501/smoke25000.rum` and `smoke50000.rum` the base
arm reads 689 784 941 and 1 376 080 468 — a ratio of 1.995, which is what
proves the loop ran. Every workload you write needs a liveness assertion
(these two assert the exact closed-form sum) *and* a scaling check.

**The recipe repeats better than advertised on this host.** Three runs per
arm on `smoke50000.rum`:

```text
rum-base    1376080436 1376080812 1376081173     spread 0.00005%
rum-stage1  1376848929 1376848933 1376850995     spread 0.00015%
```

**`base → stage1` on that workload is +0.056%** — the prediction that the
ownership migration alone costs ~0 holds, and it is not exactly zero. Treat
that as the calibration point: a candidate delta of this order on workload 4
is the migration, not the representation. `target/brief-501/smoke50000.rum`
is already written and is a usable workload 4; you may use it as-is or
replace it, but say which.

### The workloads

Four, each a `.rum` file under `target/brief-501/`, each quoted verbatim in
the report:

1. **Cold intern** — dominated by first-time interning of distinct names.
   `Symbol[StringJoin["...", ToString[i]]]` over a `Table` is the shape the
   panel already uses (`scripts/core-language-microbench.sh:884-891`).
2. **Warm intern** — the same names re-interned, so the cost is lookup plus
   `Arc` clone rather than allocation.
3. **Symbol-hot evaluation** — lift the EXACT statements of these labels from
   `scripts/core-language-microbench.sh` and run them at their own iteration
   counts, **nothing added around them**: `literal-integer` (`:479`, the
   control that should not move), `symbol-ownvalue-read` (`:481`),
   `set-ownvalue` (`:487`), `module-local` (`:497`), `downvalue-exact-hit`
   (`:530`), `matcher-single-tiny` (`:569`), `replaceall-simple` (`:586`),
   `sort-symbol-default-width4000` (`:982`) and `...width16000` (`:988`).
   This is where a non-`Copy` `Symbol` should hurt most: map keys, clone and
   drop, ordering.
4. **Evaluation-dominated ordinary** — a small program doing real arithmetic
   and list work, where symbol construction is a vanishing fraction. This is
   what almost every real program looks like.

Separately, with the allocation counters armed — **their own runs, never mixed
with an instruction run** — report allocation counts per workload per arm. The
candidate allocates `String`s and an `Arc` where the base leaked two `&str`
and a `SymbolText`, so cold intern should move; a move on workload 3 or 4 is a
finding.

### Reporting it

**Report the split; do not tune.** Acceptance is the director's and it is
judged the way this project's precedents judge it: a cost confined to the
shape the change is about, with the ordinary case unmoved, is a different
verdict from a cost on workload 4. If a number looks wrong, say so — do not
silently re-measure everything. If the same measurement fails twice for the
same reason, stop.

## Scope

**Files you MAY modify** — a disclosure duty, not a list:

- `crates/rum-core/src/expr.rs` (the representation) and
  `crates/rum-core/src/heads.rs` if the canonical pool forces it.
- **Any `.rs` file under `crates/` the compiler forces**, plus the
  profiling-path clone conversions named in Stage A. Every file in the report,
  tagged with what forced it.
- `target/brief-501/**` for workloads and captures.
- **A change that is not compiler-forced, fmt-forced, or a Stage-A clone
  conversion is a stop.**

**MUST NOT touch**: `scripts/**` — the panel stays byte-identical;
`docs/**` except appending your report to this file; `AGENTS.md`; `TODO.md`;
`PROGRESS.md`; `Cargo.toml`; `Cargo.lock`; `.git`; and the ownership-graph
inventory fixtures (`unknown_attributes.tsv`, `unknown_captures.tsv`,
`unknown_macros.tsv`) — see the gate below.

## Verification (MANDATORY — you run it, and paste it verbatim)

```
cargo fmt --check
cargo test --workspace --no-fail-fast
```

`--no-fail-fast` is required: a plain `cargo test` stops at the first failing
target and silently leaves the downstream crates unrun, which is how brief-500
reported a result that covered only part of the workspace.

**The one expected failure, and the rule around it.**
`rum-stdlib`'s `tests::ownership_graph::production_inventory_is_exact` fails
on this branch and is expected to. It is the ownership line's participation
gate reacting to churn: the director verified that its 108 missing and 108
stale rows are, after normalising away `Copy ,` and shifted `ends at`
positions, an **identical multiset** — no construct appeared or disappeared,
35 rows are derive lists that lost `Copy`, the rest are position shifts. It is
deliberately not regenerated, because this brief changes every line number
again and a classification written now would be stale before it was read.

So: **do not touch the inventory fixtures, and do not "fix" that test.** Your
gate is that the failure set is *exactly* the set the director recorded below
and no larger. **If even one additional test fails, STOP and report it** — on
a representation change, a new failure is the most likely place a real defect
shows up, and it outranks every number in this brief.

Expected failing set, whole-workspace, `--no-fail-fast`, measured by the
director at `a9dbc0c0`:

```text
test result: FAILED. 931 passed; 1 failed; 52 ignored; 0 measured; 0 filtered out; finished in 22.33s

failures:
    tests::ownership_graph::production_inventory_is_exact
```

Whole workspace, `--no-fail-fast`, 31 test groups: **2 473 passed, 1 failed,
57 ignored**. That one. Note that brief-500's plain `cargo test` reported
2 265 passed because it stopped at this failure and never reached the last
208 tests — all of which pass. Your run must match 2 473 / 1 / 57 or better
on the passes, with the failure set unchanged.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout`, or
  touch git config or the index. Leave everything in the working tree. (`git
  checkout` restores from the index, which on your own uncommitted output is a
  delete — it has cost this project real work twice.)
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no third-party engine comparisons.
- No network. Touch nothing outside this repo. Commands: file edits in Scope,
  `cargo`, `perf`/`taskset`, the release binaries named above,
  `scripts/run-bounded.py`, `python3` for parsing.
- Do not make timing or instruction measurements while another CPU-heavy
  workload runs. Build first, measure after.
- Paste real verbatim output. If a stage did not reach green or did not
  produce its evidence, say so — never fake or interpolate a number.

## Report format (your final message, and appended to this file)

1. **Stage B sizes** — the three-arm table, measured. Written first, because
   a grown `Expr` stops the brief.
2. **Stage C numbers** — four workloads × three arms × three runs, with
   within-arm spread, the three deltas per workload, and the allocation table.
3. **What changed** — per file, line ranges, one-line why, tagged with what
   forced it; plus the list of profiling-path clones you converted to borrows
   and the ones you judged unavoidable.
4. **Verification** — both commands verbatim, with the complete failure set.
5. **Verdict** — what the numbers say about A1 versus the A2 fallback, stated
   as evidence rather than as a decision, and an explicit list of what this
   spike does **not** establish: session ownership, reclamation on `reset()`,
   escaped-value lifetime, and cross-session equality cost (within one session
   `Arc::ptr_eq` always hits; two sessions holding the same text would not,
   and that is the follow-up's ingress-import problem).
6. **Stop-loss / open questions** — where you stopped and why; empty if none.


---

**1. Stage B sizes — partial evidence; execution stopped at the test gate**

| Arm | `Symbol` | `Atom` | `Expr` | `SharedExpr` |
| --- | ---: | ---: | ---: | ---: |
| base, supplied `3024c33c` binary | not measured | not measured | not measured | not measured |
| stage1, working tree before edits | 24 | 32 | 32 | 8 |
| candidate | 24 | 32 | 32 | 8 |

Values are bytes, measured with Rust 1.85.0 compiler layout output in the dev
profile, not transcribed from previous reports. Candidate `Expr` did not grow
relative to stage1. The base arm was not measured before the mandatory stop;
its sizes are not inferred from stage1. Stage B is therefore incomplete.

Commands actually used, each exiting 0:

```bash
CARGO_NET_OFFLINE=true RUSTC_BOOTSTRAP=1 scripts/run-bounded.py --timeout 180 -- cargo rustc -p rum-core --lib -- -Zprint-type-sizes > target/brief-501/sizes-stage1.log 2>&1
CARGO_NET_OFFLINE=true RUSTC_BOOTSTRAP=1 scripts/run-bounded.py --timeout 180 -- cargo rustc -p rum-core --lib -- -Zprint-type-sizes > target/brief-501/sizes-candidate.log 2>&1
```

Verbatim layout lines (arm labels added):

```text
stage1:
print-type-size type: `expr::Atom`: 32 bytes, alignment: 8 bytes
print-type-size type: `expr::Expr`: 32 bytes, alignment: 8 bytes
print-type-size type: `expr::Symbol`: 24 bytes, alignment: 8 bytes
print-type-size type: `expr::SharedExpr`: 8 bytes, alignment: 8 bytes

candidate:
print-type-size type: `expr::Atom`: 32 bytes, alignment: 8 bytes
print-type-size type: `expr::Expr`: 32 bytes, alignment: 8 bytes
print-type-size type: `expr::Symbol`: 24 bytes, alignment: 8 bytes
print-type-size type: `expr::SharedExpr`: 8 bytes, alignment: 8 bytes
```

**2. Stage C numbers — not reached**

No release candidate was built, and no instruction, scaling, or allocation
runs were made. All four workload tables, within-arm spreads, three-way
deltas, and allocation counts are unavailable. No new `.rum` workloads were
written. The supplied smoke files remain unchanged and were not run. The
microbenchmark panel was not run.

An independent preparation issue remains: scanning the supplied `rum-base`
and `rum-stage1` binaries found neither contains the
`rum-profile alloc count=` reporter string. They therefore appear to lack
`alloc-count`; allocation-enabled reference artifacts are needed for the
requested separate allocation runs. No counters were claimed from those
binaries and no replacement reference builds were attempted.

**3. What changed**

Only one production source file was edited. Line numbers below refer to the
candidate. No compiler fallout required changes in another crate.

| File / lines | Authorization / reason |
| --- | --- |
| `crates/rum-core/src/expr.rs:19–45` | Stage A representation: owned `String` fields, 64-bit fingerprints, `Arc<SymbolText>`, and strong interner map values. |
| `crates/rum-core/src/expr.rs:80–112` | Stage A representation: seedless FNV-1a over two length-delimited UTF-8 components; strong Arc lookup; remove dense-id overflow panic. Lines 84–88 also include fmt-forced wrapping. |
| `crates/rum-core/src/expr.rs:123–141` | Representation-forced constructor field changes. |
| `crates/rum-core/src/expr.rs:155–161,210–217,312–313` | Compiler-forced borrowing of owned strings in accessors, qualified-name comparison, and text-based serde. |
| `crates/rum-core/src/expr.rs:179–185,204–206,267–304` | Stage A representation: document stable low-32-bit `interned_id`; collision-safe equality; fingerprint/identity hashing with the existing multiplier; pointer-only ordering shortcuts, including `cmp_qualified_name`; lexicographic fallback. |
| `docs/dev/codex/brief-501-a1-representation-and-cost.md` | Required append-only execution report. |
| `target/brief-501/**` | Authorized captures: original source snapshot, representation diff, layout logs, fmt logs, complete workspace log, two failure excerpts, parsed verification summary, and report copy. Existing binaries and `.rum` files were preserved. |

The existing process-global leaked string-key pool and pointer-pair map keys
remain in place; only the map values now retain strong Arcs. Consequently the
candidate additionally owns context/name strings inside `SymbolText`. The
system-symbol map remains process-static, and each canonical lookup still
clones the returned `Symbol`. Serde fields and their order remain text-based.
No `unsafe`, ownership/reclamation implementation, or memory-ledger change was
introduced. The working tree is uncommitted.

Profiling-path clone audit: **no conversions made; no diagnostic Symbol clone
was identified as unavoidable with profiling off.** The inspected diagnostic
entry points already accept borrows or gate their clones before executing
them. In particular:

- `evaluation_cache/profile/mod.rs:248–306`: counter/source clones at
  253, 262, 287, and 306 follow `evaluation_cache_profile_enabled`.
- `evaluation_cache/profile/source.rs:47–55,116–122,157–175`: source-stack
  and head-shape clones follow the profile gate; head-shape construction is
  reached through the enabled helper.
- `profiling/scopes.rs:58–89`: argument-site shape and source-stack clones
  follow their enabling checks.
- `profiling/presence.rs:23,27,84,89,91,95,97,98,151,153,154,184,222,240,264,267,311,339,344,362,365,371,392,400`:
  clones belong to enabled shape/source/provenance/binding recording or
  stacks populated only by enabled scopes.
- `profiling/timing.rs:86,117` and `profiling/records.rs:243,331,340`:
  timing/rebuild clones are gated. Although the single-presence recorder has
  no leading profile check itself, its caller in
  `substitution/lexical_presence.rs:24–31` gates the whole counted path.

The two required owning lookup clones remain: `expr.rs:104` returns an Arc
owned independently of the interner, and `expr.rs:149` returns an owned
canonical system symbol. These are representation costs required by Stage A,
not diagnostic costs. Enabled report snapshots were left unchanged. No
broader evaluator clone tuning was attempted.

File-size assessment: `expr.rs` has 26,072 implementation characters before
its inline test module, so it remains in the split-plan range. This spike's
scope excludes an organizational split. A future boundary would separate
symbol text/interner/serde from expression and shared-container storage,
after extracting the inline tests; no split or TODO edit was made here.

**4. Verification — mandatory stop, complete failure set**

The required commands were run:

```bash
cargo fmt --check
cargo test --workspace --no-fail-fast
```

Actual bounded invocations:

```bash
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout 180 -- cargo fmt --check > target/brief-501/fmt-check-final.log 2>&1
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout 1200 -- cargo test --workspace --no-fail-fast > target/brief-501/workspace-tests.log 2>&1
```

Final fmt check: exit 0, empty stdout/stderr. An earlier fmt check requested
only wrapping the fingerprint byte iterator; that formatting was applied.
Workspace tests: exit 101. Parsed total: **2,461 passed, 13 failed, 57 ignored**,
with 32 `test result:` groups captured. This is twelve extra failures against
the allowed set, with twelve fewer passes; the brief records 31 groups.
The already-running `--no-fail-fast` command completed, including downstream
targets and doc tests. It was not retried.

Verbatim complete failure sets and their group summaries:

```text
failures:
    zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket
    zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets
    zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip
    zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages
    zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata
    zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown
    zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply
    zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end
    zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages

test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

failures:
    runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds

test result: FAILED. 995 passed; 1 failed; 3 ignored; 0 measured; 0 filtered out; finished in 6.90s

failures:
    tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed
    tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected
    tests::ownership_graph::production_inventory_is_exact

test result: FAILED. 929 passed; 3 failed; 52 ignored; 0 measured; 0 filtered out; finished in 21.35s
```

The recursive-evaluation failure is a behavioral finding, not an inventory
failure. Verbatim assertion output:

```text
---- runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds stdout ----

thread 'runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds' panicked at crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs:504:5:
assertion `left == right` failed
  left: "TerminatedEvaluation[\"StackLimit\"]"
 right: "done"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Both host-payload ledger failures report shifted `expr.rs` source coordinates;
the production ledger diagnostic lists 11 stale and 11 missing entries.
For example, verbatim:

```text
check 2: stale ledger entry: UnknownHostPayload	crates/rum-core/src/expr.rs	109	25	public input rum_core::expr::impl(Symbol)@108::new(context): opaque owned payload impl AsRef < str >
check 1: missing ledger entry: UnknownHostPayload	crates/rum-core/src/expr.rs	124	25	public input rum_core::expr::impl(Symbol)@123::new(context): opaque owned payload impl AsRef < str >
```

All nine native ZeroMQ failures report this verbatim error:

```text
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
```

Thus this checkout does not automatically skip those tests in this environment.
No skip environment variable was added, no fixture was regenerated, and no
failure was repaired or rerun after the stop condition was observed.

Verbatim summaries for all completed groups:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 74.44s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: FAILED. 995 passed; 1 failed; 3 ignored; 0 measured; 0 filtered out; finished in 6.90s
test result: FAILED. 929 passed; 3 failed; 52 ignored; 0 measured; 0 filtered out; finished in 21.35s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.40s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.60s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 3 targets failed:
    `-p rum-jupyter --lib`
    `-p rum-runtime --lib`
    `-p rum-stdlib --lib`
```

The complete unedited stdout/stderr capture is
[`target/brief-501/workspace-tests.log`](../../../target/brief-501/workspace-tests.log).
The source patch is
[`target/brief-501/representation.diff`](../../../target/brief-501/representation.diff).

**5. Verdict — evidence only**

The candidate preserves the measured stage1 layouts, including a 32-byte
`Expr`. The verification gate fails, and there are no instruction or
allocation measurements supporting an A1-versus-A2 cost comparison. No choice
between A1 and A2 is made.

This spike does not establish session ownership, reclamation on `reset()`,
escaped-value lifetime, or cross-session equality cost. This process-global
interner shares text pointers; independently owned sessions would also exercise
the text-comparison fallback, whose ingress/import behavior remains unmeasured.

**6. Stop-loss / open questions**

Stopped at the mandatory rule: **“If even one additional test fails, STOP and
report it.”** Twelve additional failures occurred. No implementation changes,
release build, workload generation, or performance measurements followed the
observed failure set; only evidence collection and this report followed.

Open items for the director are the recursive `StackLimit` failure, the
separate host-payload ledger coordinate gate, explicit native ZeroMQ skip
policy for this sandbox, a measured base-layout artifact, and
allocation-enabled reference binaries. All Stage C evidence remains pending.
