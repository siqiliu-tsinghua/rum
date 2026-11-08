# Brief 712 — the profiling test lease's exclusion is incomplete (design)

**Shape: design-only. `Scope: NONE` — no source change.** The director's
gate for brief 711 measured a pre-existing cross-test interference class in
`rum-runtime`'s lib test binary. This round inventories the state, proves
the exact leaking path on the pre-fix tree, and drafts the implementation
brief (or briefs) that close the class. You write no code outside
`.temp/brief-712/`.

## Goal

Make this true, with evidence: while a test holds the profiling lease and
has the process-global profiling flags enabled, **no test that does not hold
and register for that lease can read or write any process-global profiling
state** — not the epoch, not its rows, records, segments, counters, queue
owners, hooks or flags — and no such test can observe a modified flag. The
full `rum-runtime` lib binary then passes repeatedly at 8 threads.

The repair must not weaken an assertion, add a skip, change the default test
thread count, or add any cost to the production path when profiling is off.
A class-level repair, not a fix for the named instances.

## The measured class (director's evidence, 2026-11-08)

Host, both test features, full lib binary (1,393 tests), the two standing
root-context falsifiers skipped:

```text
cargo test -p rum-runtime --lib \
  --features test-builtin-memory-growth,test-continuation-slots \
  --offline --locked -j 8 --no-run
target/debug/deps/rum_runtime-<hash> --test-threads=8 \
  --skip lp655_root_context_falsifier --skip lp656_public_root_context_falsifier
```

Twelve runs on the stage-A base `e9ba6dd7`, which contains none of brief
711's changes: **5 of 12 red**.

- `runtime::profiling::logical::assignments::lp05_resumed_finish`
- `runtime::profiling::logical::assignments::lp05_a2_association_resumed_error_is_not_some`
  (an `evaluate.once_unstable` row with `open = 1` at `assignments.rs:18`)
- `runtime::profiling::logical::control_tests::lp06_a2_catch_ok`
- `runtime::profiling::logical::control_tests::lp06_a2_catch_bypass_match`
- `runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_deep_two_suspensions`
  (a non-lease test: `profile_queue.rs:625`'s assertion on the attach
  boundary policy fires inside its `held-pattern-2mib` thread)

Twelve runs on the brief-711 candidate: **3 of 12 red**, with
`runtime::profiling::tests::module::lp07_lifecycle_reasons_scopes_and_storage`,
`matcher_pattern_deep_two_suspensions`, and
`runtime::session::guest_loading::tests::message_scope_and_print_sink_are_not_replayed_across_inputs`.
Both arms are green at `--test-threads=1`. The full workspace at 8 threads
drew green once on the host (4,185/0/61), so this is a scheduling flake, not
a deterministic red; one green draw does not close it.

The class was already latent before brief 711; that brief's longer-running
lease tests widened the window. The worker's own filtered two-test base run
was green and could not see it — do not repeat that instrument.

## Context pointers

- `crates/rum-runtime/src/runtime/profiling/lease.rs`: `lease()` (:28),
  `TestLease` (`:23`), `allow_current_thread()` (:82), `Drop` (:88),
  `admit()` (:63), `blocks_unregistered_thread()` (:99), the write gate and
  the generation fence.
- `crates/rum-runtime/src/runtime/profiling/tests/engine_support.rs:16`:
  `enabled()` swaps `LEXICAL_PROFILE_ENABLED`/`LEXICAL_PROFILE_ACTIVE` for
  the test duration and holds the lease mutex.
- `crates/rum-runtime/src/runtime/profiling/state.rs:24-36`: the
  process-global `AtomicU8` gates; `config.rs` reads them on every
  evaluation.
- `crates/rum-runtime/src/runtime/profiling/logical/epoch.rs`: the existing
  lease checks at `:110`, `:134`, `:161`, `:180`, `:206`, `:409`, `:426`,
  `:442` — establish what each guards and what has no guard.
- `crates/rum-runtime/src/runtime/profiling/logical/collector.rs`:
  `set_hooks`, `drain_attached`, `drain_attached_for`.
- `crates/rum-runtime/src/runtime/suspension/core/profile_queue.rs:591-625`:
  `register_owner` installs the hooks; `attach_boundary`'s policy assertion
  runs before the owner lookup.
- `crates/rum-runtime/src/runtime/suspension/core/storage.rs:57`:
  `register_owner` call site.
- `crates/rum-runtime/src/runtime/profiling/tests/publication/support.rs`
  (the brief-711 `Flags` helper, which swaps twelve flags) and
  `crates/rum-runtime/src/runtime/profiling/tests/synchronous_families.rs`
  (stage A's lease tests) — the population the repair must cover.
- Brief 635's write gate and brief 643's thread-registration release are the
  prior art; their records are in `PROGRESS.md` and
  `docs/dev/codex/brief-635-*.md`.

## Scope

- Files you MAY write: anything under `.temp/brief-712/` — probe trees,
  builds, binaries, instrumented copies, logs, drafts, the report.
- MUST NOT modify: the live repository tree (no source, test, workload,
  manifest, inventory, docs or `.git` change). Instrument only copies under
  `.temp/brief-712/`. Any live-tree edit is a STOP.
- MUST NOT: run the workspace acceptance, the census or any standing
  inventory replay (the source does not change), or invent a fix without the
  measured path.

## The inventory the design must produce

One table, every row with `path:line`:

1. Every process-global profiling state item: the thirteen `AtomicU8` gates,
   the `AtomicU64` counters, the epoch slot (`CURRENT`), the side index, the
   thread-local segment stacks, the queue `owners()` list, the installed
   hooks, the lease generation/registration, and anything else a lease test
   can change. For each: its readers and writers.
2. For every writer: is it inside `admit()`/`blocks_unregistered_thread()`,
   is it unreachable with the flags off, or is it ungated? Prove by reading
   the code, not by assumption.
3. The subset that an unregistered thread reaches **only because a lease
   test turned the flags on** — that is the class boundary.
4. Every test that takes the lease (`grep` `enabled()` and `lease()` across
   `profiling/tests/`, `logical/control_tests`, `logical/assignments`, the
   publication tests and any host child) — the repair must cover all of
   them, not only the five named reds.

## The mechanism proof the design must produce

On an instrumented copy under `.temp/brief-712/` (never the live tree):

- Instrument the ungated candidate sites with a thread id, the current
  generation and the call site, and reproduce a red run. Show, from the
  captured trace, the **exact** write that produces the stray
  `open = 1` row on a lease test and the exact call path that trips
  `attach_boundary` in the non-lease test. A named path with a captured
  line, not an argument from code reading.
- Establish why it needs the 8-thread schedule (which interleaving lets it
  in) and why 1 thread is green.
- If a candidate site cannot be made to fire, say so and list it as
  excluded from the causal set; a negative result must name its instrument.

## Design constraints

- Production binaries must pay nothing: any new check must be
  `cfg(any(test, feature = "test-builtin-memory-growth", feature =
  "test-continuation-slots"))`, and with no lease active it must be a single
  relaxed/acquire load that returns early — the existing
  `blocks_unregistered_thread()` shape.
- No public API, feature, serde, manifest, dependency or `unsafe` change.
- No assertion weakened, no test skipped, no default thread count changed,
  no serialization of the whole suite, no test moved behind an env var that
  the acceptance run does not set.
- The lease's lock discipline must survive: no path may take `TEST_LEASE`
  or `WRITE_GATE` while holding another lock that an admitted write also
  takes; reuse the existing fence rather than inventing a second one.
- The repair must be **class-level**: every site in inventory item 3 is
  either blocked for unregistered threads or proven side-effect-free while
  a lease is current. An instance fix (e.g. only `attach_boundary`) is
  rejected unless the inventory proves the other sites unreachable.

## Options the design must evaluate

For each: target shape (code), invariants, cost, risks, and what it does
not cover.

1. **Total lease exclusion at every profiling entry point** — extend the
   `blocks_unregistered_thread()` checks to cover the queue/hook/segment
   writes found in the inventory, with the assertion moved behind the
   owner lookup where it belongs.
2. **Process isolation of lease tests** — move the lease-taking tests into
   their own test binary (child re-exec or a separate integration target)
   the way the host lifecycle tests work. State what private API exposure
   this needs and why it is or is not acceptable.
3. **Per-thread gate state** — replace the process-global flags with
   thread-scoped values for tests. Say plainly what this costs in the hot
   predicate and why it does or does not violate the no-default-path-cost
   rule.
4. Anything the measurements suggest; the list above is not exhaustive.

Recommend one, and draft the implementation brief in this template's shape:
the exact target shape, the gate tests (each with the observer that must go
red on a mutant of the fix), the falsifier, the disabled-cost bar, the
must-not-change list, and the staged stops.

## The falsifier and the acceptance bar

- Derive N from the measured pre-fix flake rate (reproduce it first on your
  machine; if the class does not reproduce there, STOP with your base-rate
  table). With the director's 5/12, N = 24 consecutive green full-lib runs
  at 8 threads gives a null probability ≈ 4e-7; state the bound you derive.
- The falsifier must be watched **red before the fix and green after**:
  the deterministic probe for the proven ungated path (two threads, a
  barrier, one lease-registered and one not, asserting the unregistered
  thread's attempted write leaves no row/fault), plus the repeated-run bar.
- A fix that only makes the named reds pass but leaves any inventory item-3
  site reachable is incomplete; say which sites the falsifier covers and
  which it cannot.

## Deliverable

- `.temp/brief-712/report.json` in `rum-report/1`: the inventory table, the
  instrumented trace proving the two paths, the base-rate table on your
  machine, the option comparison, the recommendation and its risks.
- `.temp/brief-712/drafts/` — the drafted implementation brief(s) in
  `TEMPLATE.md` shape.
- `python3 -I scripts/report.py check 712`, its summary and every compliance
  verdict line.

## Verification

One cargo command at a time, jobs 8, offline/locked, plain in the
dispatcher's 16G scope; `RUST_TEST_THREADS=8` for tests. The reproduction
may use the binary directly (after `--no-run`). Scratch only under
`.temp/brief-712/`. Host-level runs are the director's; if the class does
not reproduce in your sandbox, say so and keep the evidence you have.

## Checkpointing

Keep `.temp/brief-712/PROGRESS.md` with the exact next action and the
receipts already saved; keep every instrumented tree compiling at each
checkpoint; preserve failed logs.

## Hard rules

AGENTS and TEMPLATE apply. rum-only and rum-vs-rum; no third-party
differential engine. No git command. No live-tree edit: instrument copies
only. A live edit, a weakened assertion, a skip, a changed default or a
production-cost regression is a STOP. The director owns the gate, the
rulings and every commit.

## Director gate — 2026-11-08: design ACCEPTED; Q1-Q4 settled

The round is accepted as the design it is. I reproduced the state it depends
on: the live tree is byte-identical to `dd7c19e5` (`git status` clean; every
authored file sits under the git-ignored `.temp/brief-712/`); `report.py
check 712` is OK with 260 citations verified; the two named paths are real in
the captured traces (trace10 seq155 `<held-pattern-2mib>` gen3/reg0 removes
`[(1,129),(1,119),(1,7),(1,3)]` through `collector::take_stash`, seq168 the
holder snapshots `open=1 records=[5,67,99,129]`, seq170 the same foreign
thread restores them; trace-parent seq11 maps thread 157 to
`matcher_pattern_deep_two_suspensions`, seq45 captures
`resume -> adopted timing -> drain_matching -> take_stash`, seq856 reads
`epoch=1888 record=1936 policy=None`); the deterministic probes are red2
`(1,1,0)` / `stale_attach_no_fault=false` and green2 `(0,0,0)` / `true`; the
base rate on the worker's machine is 4/12 at 8 threads with serial green. The
389-row inventory (177 state rows, 212 consumers) is accepted as the working
map. The six FLAGGED rows are instrument defects and disclosures, not
evidence: `instrumentation`, `probe_control`, `concurrency`, `trace_budget`
and `attribution_population` were corrected or excluded from every rate, and
the extra parent-attribution draw (1,395 tests) is an attribution instrument
only.

**Erratum (mine).** My N = 24 example was arithmetically wrong:
`(7/12)^24 = 2.41e-6`, not `4e-7`. The worker's correction stands, and the
bar is derived per machine from that machine's measured `p_hat`: on its 4/12
it is 37 consecutive full-lib greens at 8 threads, `(2/3)^37 = 3.05e-7`. My
host measured 5/12 on the stage-A base and 3/12 on the stage-B candidate, and
the worker measured 4/12 on the current tree; these are consistent samples of
a schedule-dependent rate. No consumed draw may be rerun or replaced to
obtain a friendlier N.

### Rulings

**Q1 - strict reads versus the no-lease cost shape: allowed, and the writer
side is required.** The "one load, no lease" sentence in my brief described
the early-out shape of `blocks_unregistered_thread()`; it is not a budget
that forbids a transaction where the strict property needs one. Test-cfg
payload transactions and flag getters may take the existing `lease::admit()`
(the `WRITE_GATE` read) around the check and the access; production bodies
stay exactly today's and every addition is cfg-erased. Two conditions the
design must add: (a) every test helper that swaps a process-global flag - the
twelve in `engine_support` / `publication::support::Flags`, the three cache
flags through `Session::test_cache_profile_flags`, and any raw store - does
so while holding the **write** side of the same fence, so a reader holding
the read side cannot observe a modified flag across publication; no second
lock is introduced. (b) No admission may span guest evaluation or a
callback; admitted bodies take the permit and never re-acquire. A nested
acquisition on a thread already holding the read side while a writer is
queued is a deadlock and a STOP.

**Q2 - prior-owned cleanup: preserved, never refused.** Cleanup of state a
thread already owns is not a foreign access to the lease. A live owned
object dropped or refunded after a different lease is enabled must complete:
`tests/lease_release.rs:50`'s pre-enable drain stays as it is, and a refund
is never early-returned (that leaks a charge). What is refused is a foreign
thread's access to the **current** lease's state. The design must classify
each retained/destructor path as *caller-owned* (complete, with provenance
and lock proofs) or *lease state* (refuse before the first payload lock), and
prove the caller-owned class cannot reach `CURRENT`, the current stash, the
hook cells or the current epoch's rows. A bookkeeping-only teardown that
cannot be run safely (for example a foreign `drop_session` taking the global
stash) may be deferred to the next `reset_for_lease`, provided no charge is
lost, no guest result changes, and the next lease's reset releases it before
`drain_for_lease` waits on the process byte counter. Bottleneck warning: the
drain waits on the *process* byte counter, so any refusal that defers a
charge across it is a stall; the proof must cover that ordering.

**Q3 - the old flag-observation fixture: authorized.** The setup-only change
at `tests/unregistered_writes.rs:105-132` is ruled in: the first lease's
**registered** holder captures the start token, the token is handed to a
thread that is unregistered to the second lease, and the delayed write must
leave the counter map unchanged. The full observer and the exact map equality
stay; do not delete the cross-generation test. The old premise assertion (an
unregistered worker's start is taken under the flag) must be inverted into
its own deterministic observer - an unregistered start must be refused - and
that observer must be red on the pre-fix tree.

**Q4 - scope: the item-3 boundary.** The contract covers all state a lease
can enable or modify: the flags it swaps, the epoch and side index, records,
segments, stems, counters, stash, hook cells, queue owners, the budget, the
legacy summary/scope/presence/timing state and the evaluation-cache profile
state. Independent env-only profilers (`RUM_PROFILE_REPR`,
substitution-sharing, association) are explicitly outside this repair because
no lease helper modifies their gates; the inventory must state per profiler
why it is lease-independent, the repair must not touch them, and any
interference observed there is a separate design. This narrows the literal
"any process-global profiling state" of my goal to the state the lease
exposes; it does not weaken the measured class.

### Option choice

Option 1 (total exclusion at the state boundary, on the existing fence) is
the repair. Option 2 (process isolation) leaves an unregistered worker inside
a lease test able to observe or steal the holder's state, and so fails the
deterministic class contract by itself; Option 3 (per-thread gates) does not
shield the ungated state paths (hook cells, owners, retained handles,
cleanup) and would need Option 1 as well; Option 4 (a take-only patch) is an
instance fix the inventory refutes. The implementation brief is
`brief-713-profiling-lease-exclusion.md`, written from the two drafts with
these rulings built in; the 37-run campaign is a separate source-unchanged
validation brief after the implementation gate.
