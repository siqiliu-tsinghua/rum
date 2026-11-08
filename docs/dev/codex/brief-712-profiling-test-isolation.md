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
