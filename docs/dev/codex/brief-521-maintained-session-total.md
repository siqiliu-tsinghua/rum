# brief-521 — maintain the session total unconditionally

## Goal

Replace the O(session) walk inside the armed checkpoint with a **maintained
session total**, updated as session state changes, and measure what it costs
on both paths.

brief-520 established the prize: a stubbed O(1) session term removes
**99.79%-99.99%** of the armed overhead, taking the worst shape from
801,714,469 instructions to 5,374,916, with a residual of ~50 instructions
per checkpoint that the disassembly fully accounts for. The budget it
derived is generous — a real implementation may spend about **2,495 / 3,017
/ 7,697** instructions per steady checkpoint for the three session sizes and
still retain 99% removal.

## Director decision: no accounting mode (2026-09-22)

brief-518 built a mode route so that **unarmed** sessions could skip
accounting. The user's corrected ruling removes its reason for existing: an
unarmed cost of up to about **0.7%** is now acceptable, because the memory
ceiling is a documented MUST for untrusted input rather than off-by-default
diagnostics. **If we are willing to pay unarmed, the mode buys nothing.**

So maintain the total **unconditionally**, with no mode, no specialization,
and no branch in the hot evaluator. This also avoids what brief-518 measured
as a side effect: `evaluate_once_unstable_after_input_check_with_cycle_tracking`
growing from 1,854 to 4,150 bytes because both specializations were inlined
into it.

brief-518's prototype stays available at
`target/accounting-route-518/prototype.patch` if this route fails. **Do not
restore it here.**

## Why this is not brief-519's refuted route

brief-519 failed on **fail-open**: it *inferred* freshness from existing
counters, and 42 of the 52 walked fields had no covering counter, so it
reused a stale low total on the very reproduction this line exists to fix.

This brief does not infer anything. It **maintains** the total at the sites
that change it. The failure mode is therefore different and must be guarded
differently: a missed charge or release is **drift**, and downward drift is
fail-open. That is what the falsifier below is for.

## The work

1. **Maintain the total.** Use brief-516's charge-and-release analysis and
   brief-517's per-field occurrence specification as the definition of what
   is being maintained. The maintained value must equal what the walker
   computes — the **occurrence** model, per the user's ruling, not a
   unique-allocation count.
2. **Enumerate charge and release separately**, as brief-516 required.
   Release paths include error and panic exits; a release that is skipped on
   an error path is a permanent upward drift, and a charge skipped anywhere
   is a permanent downward one.
3. **Use the maintained total in the armed checkpoint**, keeping the
   expression term exactly as it is — brief-520 showed the expression walk is
   the residual that must remain.
4. **Keep the walker.** brief-517 built it as an independent oracle
   specifically so this step could be checked against it. It stays in the
   tree and stays authoritative for `MemoryInUse[]`-style observation.

## The falsifier, which must be watched failing

Assert **maintained total == walker total** at every armed checkpoint under a
debug or test configuration, across a workload set that exercises
definitions, caches, contexts, the evaluation stack, prints, traces, reaps,
suspension, abandonment, and error exits.

Then **watch it fail**: delete one charge site, show the assertion red;
delete one release site, show it red the other way; restore both and show
green. Paste all of it. A drift detector nobody has seen fire is not known
to work — and this line has already been saved twice by insisting on that.

## The measurement, which decides acceptance

**Unarmed** — the maintenance cost, since it is now unconditional. Use
`target/brief-501/smoke50000.rum` (calibrated: base 1,376,080,468,
within-arm spread 0.00005%) plus at least two evaluation-heavy labels lifted
verbatim from `scripts/core-language-microbench.sh`. Report **scaled slopes**.
**The bar is ~0.7%**; above it, stop and report rather than tuning.

**Armed** — the nine shapes at `target/brief-514/armed-*.rum`, unchanged, so
the numbers compose with brief-514's table and brief-520's ceiling. Report
the fraction of overhead removed against both.

Recipe: `taskset -c 2 perf stat -e instructions:u -x, --`,
`cpu_core/instructions/u`, `100.00` multiplex, three runs per arm, release,
profiling env OFF, both arms built at the same path, stdin closed. Verify
counts scale with the workload. **Explain any anomaly before accepting it** —
brief-519's favourable numbers were an inlining artifact and were caught only
because explanation was required. **Do not run the timing panel.**

## Scope

MAY modify: the accounting modules under
`crates/rum-runtime/src/runtime/debug_tools/resources/`, the `Session`
mutation sites the inventory names, their module declarations, the
corresponding runtime tests, and `docs/guide/resource-limits-and-sandbox.md`.

Anything else the compiler forces is a **disclosure duty**: list it with the
reason. Not compiler-forced and not listed is a stop.

Do NOT touch: `Expr`/`SharedExpr`/`SharedArgs`, the interner, the builtin
catalog's growth declarations, `scripts/**`, `AGENTS.md`, `TODO.md`,
`PROGRESS.md`, `Cargo.toml`, `Cargo.lock`, `.git`, the ownership-graph
inventory fixtures. **Do not restore brief-518's prototype.** **Do not touch
the 165 per-builtin `Incremental` declarations** — that is a separate debt.

## Verification (MANDATORY — you run it, paste it verbatim)

```
cargo fmt --check
cargo test --workspace --no-fail-fast
```

**Expected on `main`:** 2 509 passed, 0 failed, 57 ignored on a host that can
bind loopback. Run **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; report the
nine native ZeroMQ failures as environmental if your sandbox denies loopback
and say plainly they did not execute. Any other failure is yours.

Bound the suite with `scripts/run-bounded.py --timeout 900 --`.

**Append your report to this file**, and keep raw evidence under
`target/brief-521/`.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **No accounting mode, no specialization, no hot-path branch on armed
  state.** If the total cannot be maintained without one, that is a
  refutation to report.
- **Do not weaken the occurrence model** to make maintenance easier. Shared
  subtrees are charged per occurrence; a cheaper different number is not an
  answer.
- **Do not delete or weaken the walker.** It is the oracle.
- **Testing Policy is a hard line** (`AGENTS.md`).
- No network. Touch nothing outside this repo; read `target/brief-5*/` and
  `target/accounting-route-518/` and `target/measure-achievable-520/` freely,
  overwrite nothing there.
- **Stop-loss, keyed on progress.** Same step failing twice with no new
  information means stop and report.
- Paste real verbatim output, including failures.

## Report format

1. The charge and release inventory, the two listed separately.
2. The falsifier, with all three runs: green, red on a deleted charge, red on
   a deleted release.
3. Unarmed numbers with scaled slopes, against the ~0.7% bar.
4. Armed numbers against brief-514's table and brief-520's ceiling.
5. Verification, verbatim, with the complete failure set.
6. Anything you refused, and open questions.


---

## Execution report — 2026-09-22 — stopped at the unarmed cost gate

**The maintained-session-total implementation is incomplete.** The first
unconditional owner, the evaluation stack, already exceeded the approximately
0.7% unarmed bar: **+2.6193% smoke, +3.0584% symbol-ownvalue-read, and +2.3845%
downvalue-pattern-hit**, measured as scaled instruction slopes. I stopped
implementation at that result, without tuning, as the brief directs.

I tested this hot owner before extending maintenance to the other 51 classified
fields. This is an early rejection of this implementation, not a completed
52-field candidate. No checkpoint was switched to an incomplete total. The
production walker remained authoritative throughout. The whole-session equality
falsifier and candidate armed comparison were **not reached**. The narrower
stack-contribution falsifier below is identified as such.

The rejected probe and its tests are preserved in
`target/brief-521/stack-probe.patch` and `target/brief-521/stack-probe/`.
All 12 probe files were then restored or removed using saved original bytes and
hash guards. All 1,512 initially tracked files matched their initial hashes
before this report was appended. The release executable was rebuilt and is
byte-identical to the measured base. **Only this appended report remains as a
tracked change.** All raw evidence and scratch tools are under
`target/brief-521/`.

### 1. Charge inventory and release inventory

The definition of the quantity remains brief-517's occurrence model. The
walker and independent oracle were not simplified. In the temporary probe,
`evaluation_stack_bytes: Option<usize>` added 16 inline bytes, covered by
`size_of::<Session>()`; its schema row added an inline exemption without
changing any of the existing 52 accounted fields. No per-entry or heap weight
cache was introduced. `None` meant arithmetic invalidity, never an accounting
mode. Maintenance executed irrespective of whether a memory limit was armed.

Prefixes below expand as follows: `R/` is
`crates/rum-runtime/src/runtime/`; `P/` is the archived probe's corresponding
runtime directory under `target/brief-521/stack-probe/`. The following charge
and release lists keep the broader obligations visible. Only the evaluation
stack row was implemented and measured; the remaining fields are outstanding.
The owner boundaries are from brief-516's charge/release inventory and the
field formulas from brief-517; they are not claims of newly implemented hooks.

**Charges, listed separately**

| Fields (52 total) | Charge boundary and model obligation |
| --- | --- |
| `evaluation_stack` (probe implemented) | `P/debug_tools/resources/maintained.rs::push_accounted_evaluation_entry` weighs the cloned retained expression once, pushes it, then charges that occurrence and the actual capacity delta. `replace_accounted_evaluation_stack` weighs the incoming owner on cold transfer. Called from evaluator push, resume/abandon stack transfers and rollback restoration. |
| `builtins`, `request_interrupt_control`, `resource_limits` | `R/session/session_state.rs`, `session_api.rs`: initialize inline/session terms, live builtin entries, the fixed interrupt Arc object and the conservative additional limits term. |
| `attributes`, `attribute_set_cache`, `builtin_dispatch_spec_cache` | `R/resolution.rs`, `session/session_api.rs`: live attribute key text; cache key text and actual hash capacity. |
| `definitions`, `installed_system_own_values`, `installed_system_down_values` | `R/definitions/`, `model/definitions/`, `session/session_api.rs`: incoming own/rule/guard occurrences, keys, markers, retained rule pairs, mutable and read-side lazy dispatch indexes. Part/Append/association lvalues must charge their structural deltas too. |
| `stable_symbol_cache`, `symbol_call_effect_cache`, `call_stability_cache`, `association_lookup_cache` | `R/eval_core/stability.rs`, `evaluation_cache/`, `model/cache.rs`: cache key/capacity/index growth; call cache heads and args are additional occurrences. Weak association roots do not become charged Expr roots. |
| `options`, `syntax_information` | `R/options/core.rs`, `namespace_tools/syntax.rs`: key text, option entry capacity, option values and syntax expressions. |
| `declared_stubs`, `removed_symbols`, `symbol_generations`, `once_values` | `R/package_tools.rs`, `definitions/definition_clear.rs`, `scoping/unique.rs`: newly retained symbols/text, generation entry strings, Once key capacity and expression. |
| `current_context`, `context_path`, `context_stack`, `package_search_paths`, `loaded_packages`, `load_stack` | `R/session/session_api.rs`, `session_eval.rs`, `definitions/definitions.rs`, `builtins_context_loader.rs`: current/saved String capacities, actual outer Vec capacity and length-based PathBuf payloads. Saved frames and current roots count independently. |
| `file_io`, `entropy_source`, `print_sink` | `R/session/session_api.rs`, `model/session.rs`, `messages/builtins_messages_inspect.rs`: concrete provider object plus Arc header or concrete boxed sink size, including reinstall after a temporary take. |
| `exception_types` | `R/kernel_data/control_data/exceptions.rs`: live registry entry shell and owned name/parent String capacities. |
| `dynamic_own_values`, `module_activations`, `gc_candidate_gensyms`, `option_value_scopes` | `R/eval_core/symbol.rs`, `scoping/lexical/module.rs`, `scoping/gensym_gc.rs`, `options/patterns.rs`: pushed frame payloads, symbol text, option entries, nested and outer capacities. |
| `sealed_units` | `R/scoping/packages.rs`: unit wrapper, text, public/package symbol vectors and private index contents on registration/replacement. |
| `reap_scopes`, `monitor_scopes` | `R/debug_tools/reap.rs`, `monitor.rs`: pushed patterns/handlers/displays/callbacks; each new Sow tag/value occurrence; replacement display String capacity. No repeated-prefix recomputation is an acceptable hot append implementation. |
| `disabled_message_selectors`, `message_definitions`, `message_symbol_counts`, `messages`, `handled_non_loader_eval_errors`, `last_message_list` | `R/debug_tools/message_switch.rs`, `messages/message_catalog.rs`, `resolution.rs`, `session/session_helpers.rs`, `session_eval.rs`, `session_requests.rs`: selector/message strings, definition roots, count-map capacity and independent cloned message-list occurrences. |
| `prints`, `trace_events`, `trace_event_depths` | `R/session/session_helpers.rs`, `definitions/application/mod.rs`: buffered String capacities, trace expression/string occurrences, actual event/depth Vec growth. A string delivered to a sink is not also a buffered occurrence. |
| `queued_input_responses`, `pending_input_request`, `pending_eval_request`, `pending_suspension`, `resuming_suspension`, `abandonment_worklist` | `R/session/session_api.rs`, `session_requests.rs`, `suspension/core.rs`: queued strings/capacity, prompt and pending-request conservative terms, every suspension shell/frame/traced root, active-stack terms and destination queue/worklist capacity on transfers. |

**Releases, listed separately**

| Owner/exit | Release obligation |
| --- | --- |
| Evaluation stack: normal and ordinary error return (probe implemented) | `P/eval_core/mod.rs` pops after `evaluate_inner_unstable` on both `Ok` and `Err`; `pop_accounted_evaluation_entry` subtracts the saved incoming payload weight. Reserved Vec capacity remains charged. No second expression traversal is paid on pop. |
| Evaluation stack: resume, abandonment and rollback (probe implemented) | `P/suspension/core.rs` routes replacement/take/clear and caller-stack restoration through the accounting helpers. `P/session/session_helpers.rs` reconciles rollback's actual cloned vector. A clear releases all live payload but retains its capacity term. |
| Evaluation stack: panic (probe tested) | The existing evaluator retains stack roots when a host callback panics before pop. Their charge must also remain; subtracting merely because execution unwound would undercount the actual retained state. The test catches a deliberate panic, checks the retained owner against the oracle, then checks rollback's real release. Session destruction discards its local ledger with its owners. |
| Definitions and installed markers/rules | Overwrite, unset, matching-rule replacement/removal, Clear/ClearAll/Remove, empty pruning and gensym GC release old roots and dispatch/index storage. Restored Block/iterator definitions add their actual new contributions; deletion may separately add generation/removed-symbol state. |
| Attribute and evaluation caches; association indexes | Invalidation, cap eviction, stale weak-entry eviction, remove/reinsert and clear release roots/key text/index storage; retained table/vector capacity remains charged. Read-side eviction is a write boundary too. |
| Options, syntax metadata, stubs, removed/generation state, Once | Replacement/removal and scope restoration remove the actual old entry terms. Clear/Remove is not necessarily a net release across all owners. |
| Context/package/path/load owners | End/EndPackage, loader failure, loaded-package truncation, load-stack pop and restoration release saved/current payloads according to actual capacities. A wrong-kind frame pushed back is still live. |
| Providers, sink, request control and limits | Replacement/take/drop removes the previous concrete-object term where it leaves the modeled owner; reinstall adds it back. Accepted-request wrappers keep their existing exemption. |
| Exception registry and sealed units | Replace/drop old registry strings or units; failed registration/rollback releases minted definitions and replaced unit roots. |
| Dynamic/module/option/GC scopes | Normal, error, resume and abandonment alternatives pair each push with one actual pop/truncate/reclaim; retain outer capacity and legitimately escaped roots. |
| Reap and Monitor | Pop on completion/error/abandonment; release every removed bucket/pattern/handler/display occurrence. Values moved to local output builders leave session S; builder accounting remains a separate debt. |
| Message definitions/selectors/buffers/counts | On/remove/replace, quiet/check suppression, consume, truncate, split, response transfer and last-list replacement release the actual old strings/roots. Surviving copies in traces/last-message lists remain separate charges. |
| Prints and traces | Take/split transfers source storage out of Session; truncate releases payload but not retained capacity. Clear depth buffers according to their capacity-only model. |
| Input queue/prompt/pending request | Pop-front, take, clear, completion, cancellation and abandonment remove the leaving payload/wrapper while retaining queue capacity if still owned. |
| Suspension owners/worklist | Pending-to-resuming and suffix transfers, popped frames, normal/error completion, nested suspension and cleanup that suspends again must transfer each actual occurrence once. Saved-stack spare capacity has a different formula from the live Session stack. |
| Multi-owner rollback, panic/poison/reset/drop | ValueQ/loader/Block/iterator restoration must reconcile actual post-clone capacities and warmed caches. Panic cannot be treated as speculative destruction of retained roots. A complete ledger must remain valid or fail closed on every partial mutation; reset discards the old Session and its ledger together. |

The prototype's only remaining direct stack writes were the four accounting
helpers (push, pop, replace and clear). Other field maintenance, lazy-index
accounting and complete session arithmetic were not implemented after the
performance stop. The temporary implementation also reused the unchanged
production expression weigher; it did not close brief-517's separately recorded
internal-overflow gaps. Those gaps remain open for a complete implementation.

### 2. Falsifier — the implemented stack component only

At every armed expression checkpoint and projected-byte precheck in a **test
build**, the prototype asserted its maintained stack contribution against the
independent oracle's `evaluation_stack` field. No release-build diagnostic branch
was added. This does **not** constitute the required whole-session equality
assertion: the other 51 fields had not been migrated.

The green workload covers definitions and warm caches, contexts, Block/Module,
prints, Trace, Reap/Sow, messages, ValueQ rollback, Throw, input suspension,
resume and abandonment. Separate tests cover repeated shared occurrences,
retained Vec capacity and panic followed by rollback. These tests show stack
consistency while those behaviors execute; they do not certify accounting of
those other owners.

The exact removed charge was `.and_then(|total| total.checked_add(payload))`;
capacity charging remained. The exact removed release was the assignment using
`total.checked_sub(payload)`. The patches are `deleted-charge.patch` and
`deleted-release.patch`. The first deletion produced a low maintained total,
160 versus 480; the second produced a high one, 480 versus 160. Both faults
were restored, and all three focused tests passed again. Verbatim runs follow.

````console
$ scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime maintained_stack -- --nocapture
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.45s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 3 tests
test runtime::tests::session_memory::maintained::maintained_stack_counts_shared_occurrences_and_retained_capacity ... ok

thread 'runtime::tests::session_memory::maintained::maintained_stack_survives_panic_and_rollback' panicked at crates/rum-runtime/src/runtime/tests/session_memory_maintained.rs:74:5:
intentional retained-stack panic
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test runtime::tests::session_memory::maintained::maintained_stack_survives_panic_and_rollback ... ok
test runtime::tests::session_memory::maintained::maintained_stack_checkpoint_workloads ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1015 filtered out; finished in 0.01s

EXIT_STATUS 0
````

````console
$ scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime maintained_stack_checkpoint_workloads -- --nocapture
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.00s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 1 test

thread 'runtime::tests::session_memory::maintained::maintained_stack_checkpoint_workloads' panicked at crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:68:9:
assertion `left == right` failed: maintained evaluation-stack contribution drifted
  left: Exact(160)
 right: Exact(480)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test runtime::tests::session_memory::maintained::maintained_stack_checkpoint_workloads ... FAILED

failures:

failures:
    runtime::tests::session_memory::maintained::maintained_stack_checkpoint_workloads

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1017 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p rum-runtime --lib`
EXIT_STATUS 101
````

````console
$ scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime maintained_stack_checkpoint_workloads -- --nocapture
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: unused variable: `payload`
  --> crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:29:73
   |
29 |     pub(in crate::runtime) fn pop_accounted_evaluation_entry(&mut self, payload: usize) {
   |                                                                         ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_payload`
   |
   = note: `#[warn(unused_variables)]` on by default

warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 4 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.29s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 1 test

thread 'runtime::tests::session_memory::maintained::maintained_stack_checkpoint_workloads' panicked at crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs:66:9:
assertion `left == right` failed: maintained evaluation-stack contribution drifted
  left: Exact(480)
 right: Exact(160)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test runtime::tests::session_memory::maintained::maintained_stack_checkpoint_workloads ... FAILED

failures:

failures:
    runtime::tests::session_memory::maintained::maintained_stack_checkpoint_workloads

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1017 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p rum-runtime --lib`
EXIT_STATUS 101
````

````console
$ scripts/run-bounded.py --timeout 900 -- cargo test -p rum-runtime maintained_stack -- --nocapture
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.43s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 3 tests
test runtime::tests::session_memory::maintained::maintained_stack_counts_shared_occurrences_and_retained_capacity ... ok

thread 'runtime::tests::session_memory::maintained::maintained_stack_survives_panic_and_rollback' panicked at crates/rum-runtime/src/runtime/tests/session_memory_maintained.rs:74:5:
intentional retained-stack panic
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test runtime::tests::session_memory::maintained::maintained_stack_survives_panic_and_rollback ... ok
test runtime::tests::session_memory::maintained::maintained_stack_checkpoint_workloads ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1015 filtered out; finished in 0.03s

EXIT_STATUS 0
````

The panic message in each green run is the deliberately caught panic test;
it is not a failed test. The earlier full runtime suite with stack maintenance
also passed 1,012 unit tests plus 2 doc tests, with 3 ignored; its complete
output is `target/brief-521/stack-runtime.log`.

### 3. Unarmed instruction measurements and scaled slopes

Both arms were built offline with the workspace Rust 1.85.0 release toolchain
at `/home/remi/dev/rum/target/release/rum-cli`, and executed from that exact same
path. All `RUM_*` variables and `RUST_LOG` were absent. Stdin was closed.
Measurements were serial, without concurrent task-local build/test work. Every
accepted row used `cpu_core/instructions/u` at `100.00` coverage. No timing panel
was run. The matrix contains 54 base runs (27 unarmed plus 27 armed) and 27
unarmed stack-probe runs; a separate preliminary smoke run verified perf access.

The per-run command was:

```text
scripts/run-bounded.py --timeout 60 -- taskset -c 2 perf stat -e instructions:u -x, -o <capture>.perf.csv -- /home/remi/dev/rum/target/release/rum-cli <source>.rum < /dev/null
```

`smoke50000` is the original `target/brief-501/smoke50000.rum`, unchanged.
The shorter smoke sources differ only in N and their checked expected sum.
The other two workload calls were copied verbatim from the canonical script:

```rum
rumMicroBench["symbol-ownvalue-read", rumMicroN[2000], xOwn];
rumMicroBench["downvalue-pattern-hit", rumMicroN[1000], fPattern[1]];
```

Their deterministic driver is the preserved brief-514 driver with scales 5,
10 and 20, seven repetitions, checked result and a completion marker. It does
not run AbsoluteTiming. N below is the total repeated body count, excluding
fixed warmup/setup. Medians are of three runs:

| Workload | N | Base instructions | Stack probe instructions |
| --- | ---: | ---: | ---: |

| smoke | 12,500 | 346,679,318 | 355,667,889 |
| smoke | 25,000 | 689,829,354 | 707,804,525 |
| smoke | 50,000 | 1,376,122,702 | 1,412,075,422 |
| symbol-ownvalue-read | 70,000 | 153,195,123 | 157,763,732 |
| symbol-ownvalue-read | 140,000 | 301,949,054 | 311,066,761 |
| symbol-ownvalue-read | 280,000 | 599,433,656 | 617,649,986 |
| downvalue-pattern-hit | 35,000 | 514,148,575 | 526,305,411 |
| downvalue-pattern-hit | 70,000 | 1,023,767,129 | 1,048,049,415 |
| downvalue-pattern-hit | 140,000 | 2,043,043,539 | 2,091,656,503 |

Slopes are `(median_large - median_small) / (N_large - N_small)`; the
regression is `100 * (probe_slope / base_slope - 1)`. All three scales grow
with workload, and both adjacent intervals show the same rejection:

| Workload | N interval | Base instructions / iteration | Probe instructions / iteration | Change |
| --- | ---: | ---: | ---: | ---: |

| smoke | 12,500 → 25,000 | 27,452.002880 | 28,170.930880 | +2.618854% |
| smoke | 25,000 → 50,000 | 27,451.733920 | 28,170.835880 | +2.619514% |
| smoke | 12,500 → 50,000 | 27,451.823573 | 28,170.867547 | +2.619294% |
| symbol-ownvalue-read | 70,000 → 140,000 | 2,125.056157 | 2,190.043271 | +3.058136% |
| symbol-ownvalue-read | 140,000 → 280,000 | 2,124.890014 | 2,189.880179 | +3.058519% |
| symbol-ownvalue-read | 70,000 → 280,000 | 2,124.945395 | 2,189.934543 | +3.058391% |
| downvalue-pattern-hit | 35,000 → 70,000 | 14,560.530114 | 14,906.971543 | +2.379319% |
| downvalue-pattern-hit | 70,000 → 140,000 | 14,561.091571 | 14,908.672686 | +2.387054% |
| downvalue-pattern-hit | 35,000 → 140,000 | 14,560.904419 | 14,908.105638 | +2.384476% |

````console
$ scripts/run-bounded.py --timeout 60 -- python3 target/brief-521/analyze.py
RAW_RUNS_VERIFIED 81
ALL_EVENTS cpu_core/instructions/u; ALL_COVERAGE 100.00; ALL_EXITS 0
family	start	end	base_instructions_per_iteration	stack_probe_instructions_per_iteration	regression_pct
smoke	12500	25000	27452.002880	28170.930880	2.618854
smoke	25000	50000	27451.733920	28170.835880	2.619514
smoke	12500	50000	27451.823573	28170.867547	2.619294
symbol-ownvalue-read	70000	140000	2125.056157	2190.043271	3.058136
symbol-ownvalue-read	140000	280000	2124.890014	2189.880179	3.058519
symbol-ownvalue-read	70000	280000	2124.945395	2189.934543	3.058391
downvalue-pattern-hit	35000	70000	14560.530114	14906.971543	2.379319
downvalue-pattern-hit	70000	140000	14561.091571	14908.672686	2.387054
downvalue-pattern-hit	35000	140000	14560.904419	14908.105638	2.384476
STOP_LOSS True
EXIT_STATUS 0
````

The fresh smoke50000 median, 1,376,122,702, is 42,234 instructions
(0.003069%) above the brief's historical calibration. I used fresh paired
slopes, not that historical count as the denominator. This small historical
whole-process offset was not isolated to a cause. It does not produce the
measured approximately 719 additional instructions per smoke iteration:
both fresh adjacent slope comparisons independently show +2.62%. The measured
base binary is also byte-identical to brief-520's recorded restored binary.
No favorable unexplained offset is being accepted as a speedup.

The cost is visible in disassembly. The probe calls the unchanged recursive
expression weigher on each entering tracked stack occurrence, saves the weight,
reconciles actual Vec capacity with checked arithmetic, stores the contribution,
and subtracts the saved weight on pop. The helper bodies were inlined into the
existing evaluator, with no mode specialization. The evaluator grew from
1,854 to **2,051 bytes**, while `evaluate_inner_unstable` remains a separate
9,971-byte function and `expr_memory_proxy_bytes` remains 638 bytes. Calls to
`evaluate_inner_unstable` remain present. The source and disassembly therefore
identify real added bookkeeping and weighing, rather than a disappeared inner
evaluator call of the kind that invalidated brief-519's apparent improvement.

Whole-process deltas also include layout/code-generation effects; this is not
a claim that every added instruction is a literal counter update. The result
applies to this particular implementation. For example, the ownvalue label's
approximately 65 extra instructions per repeated body exceed its approximately
14.87-instruction 0.7% allowance. A cheaper unconditional implementation remains
an open design question; I did not attempt one after the gate failed.

Exact function disassemblies and sizes are in `*-evaluate_*.asm`,
`*-expr_memory_proxy_bytes.asm` and `assembly-index.json`. The initial assembly
extractor matched similarly named projection helpers too; that broad-match
index is preserved separately, and `assembly-exact-symbols.log` records the
corrected exact-symbol extraction. This changed evidence extraction only,
not the measured candidate.

### 4. Armed reference measurements and unreached candidate gate

The nine original `target/brief-514/armed-*.rum` files were executed unchanged,
three times each, on the fresh base before implementation. The following table
places those medians beside the historical references. B514 was strided, F514
was unstrided, M520 was main, and S520 was the deliberately incorrect constant
session-term stub. Historical raw perf rows and completion markers were
revalidated in this task (108 records); `historical-reference.json` records
the resulting table.

| Shape / N | B514 | F514 | M520 | S520 ceiling experiment | Fresh base 521 |
| --- | ---: | ---: | ---: | ---: | ---: |

| empty / 16 | 3,827,120 | 8,661,387 | 8,660,408 | 3,574,179 | 8,655,932 |
| empty / 128 | 4,392,850 | 37,222,136 | 37,220,519 | 3,633,301 | 37,219,142 |
| empty / 1024 | 8,406,144 | 265,700,367 | 265,700,870 | 4,125,501 | 265,696,833 |
| list1k / 16 | 4,021,776 | 9,848,047 | 9,846,447 | 3,713,715 | 9,841,373 |
| list1k / 128 | 4,691,417 | 44,254,099 | 44,255,362 | 3,776,340 | 44,249,355 |
| list1k / 1024 | 9,435,123 | 319,502,628 | 319,503,212 | 4,267,124 | 319,498,524 |
| list10k / 16 | 5,597,630 | 20,315,567 | 20,314,703 | 4,823,146 | 20,311,336 |
| list10k / 128 | 7,203,868 | 107,139,604 | 107,138,340 | 4,885,906 | 107,134,629 |
| list10k / 1024 | 18,499,514 | 801,717,142 | 801,714,469 | 5,374,916 | 801,711,251 |

````console
$ scripts/run-bounded.py --timeout 60 -- python3 target/brief-521/reference_evidence.py
HISTORICAL_RAW_RECORDS_VERIFIED 108
ALL_HISTORICAL_CORE_EVENTS cpu_core/instructions/u; COVERAGE 100.00; COMPLETION_MARKERS verified
No candidate armed result: stop condition reached before session-total completion or checkpoint cutover.
EXIT_STATUS 0
````

**Candidate armed counts and fraction of overhead removed: not measured.**
The stop condition was reached before completing session maintenance or changing
the checkpoint. Measuring this incomplete stack-only probe as if it were a
maintained-session implementation would not answer the brief. No fraction
against either brief-514 or brief-520 is claimed, and the historical stub's win
is not credited to this attempt. The unchanged walker remains on the armed path.

### 5. Mandatory verification — complete verbatim output

After restoring the original source, `cargo fmt --check` exited 0 without
output. `cargo test --workspace --no-fail-fast`, bounded by 900 seconds,
reported **2,500 passed, 9 failed, 57 ignored**, exit 101. The complete failure
set is the nine native ZeroMQ tests below, all blocked by loopback bind with
`Operation not permitted (os error 1)`. Their native transport scenarios
**did not execute**. `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` was absent; it was not set
to hide these failures. There were no other workspace failures.

The complete captured stdout/stderr is pasted here without elision.

````console
$ scripts/run-bounded.py --timeout 60 -- cargo fmt --check
EXIT_STATUS 0
````

````console
$ scripts/run-bounded.py --timeout 900 -- cargo test --workspace --no-fail-fast
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-jupyter v0.2.0 (/home/remi/dev/rum/crates/rum-jupyter)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.88s
     Running unittests src/main.rs (target/debug/deps/rum_cli-071a386fb6152ac7)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/cli.rs (target/debug/deps/cli-527e9f2d0cca7f28)

running 28 tests
test parse_flag_prints_debug_dump ... ok
test format_flag_respects_custom_line_width ... ok
test format_flag_accepts_prefix_equals_line_width ... ok
test format_flag_rejects_invalid_line_width ... ok
test format_flag_prints_canonical_surface_form ... ok
test unknown_long_flag_is_rejected ... ok
test eval_flag_keeps_non_hold_form_canonical_output_unchanged ... ok
test no_arg_non_tty_stdin_is_filter_program_without_repl_chatter ... ok
test eval_exit_uses_requested_status_code_without_error_rendering ... ok
test script_input_preserves_quotes_and_backslashes_verbatim ... ok
test script_args_separator_without_args_binds_empty_list ... ok
test eval_flag_keeps_to_string_form_arms_unchanged ... ok
test eval_flag_renders_prints_before_final_value ... ok
test closed_stdout_pipe_exits_successfully_without_stderr ... ok
test file_exit_uses_requested_status_code_without_error_rendering ... ok
test file_exit_renders_accumulated_prints_without_error_rendering ... ok
test positional_dash_reads_stdin_as_program ... ok
test eval_flag_renders_source_aware_syntax_diagnostics ... ok
test eval_exit_renders_accumulated_prints_without_error_rendering ... ok
test filter_mode_uses_script_semantics_and_prints_final_result_only ... ok
test runtime_messages_go_to_stderr_and_values_go_to_stdout ... ok
test eval_flag_binds_piped_stdin_as_script_input ... ok
test file_flag_renders_source_aware_syntax_diagnostics ... ok
test script_args_after_separator_are_bound_as_strings ... ok
test eval_flag_renders_hold_form_transparently_in_canonical_style ... ok
test file_and_stdin_scripts_render_later_error_at_the_written_character ... ok
test random_default_seed_is_reproducible_across_cli_invocations ... ok
test eval_flag_treats_replace_repeated_limit_as_recoverable_value ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)

running 7 tests
test expr::tests::displays_nested_call_expression ... ok
test expr::tests::displays_global_symbol_without_context_prefix ... ok
test expr::tests::displays_string_atoms_with_escaped_special_characters ... ok
test expr::tests::normalizes_int_values_and_orders_by_numeric_value ... ok
test expr::tests::normalizes_rational_atoms ... ok
test expr::tests::displays_message_name_shorthand ... ok
test expr::tests::displays_system_symbol_without_context_prefix ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)

running 5 tests
test tests::unions_are_rejected_with_a_clear_error ... ok
test tests::an_existing_frame_root_trace_bound_is_not_duplicated ... ok
test tests::pure_frame_abandon_preserves_generics_and_where_clauses ... ok
test tests::generic_bounds_are_added_only_for_parameters_used_by_fields ... ok
test tests::expansion_covers_all_struct_and_enum_field_shapes ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)

running 3 tests
test accessor_detaches_the_original_link_and_leaves_payloads_movable ... ok
test next_link_supports_non_clone_payloads_lifetimes_and_const_generics ... ok
test unsupported_arrangements_report_specific_diagnostics ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)

running 3 tests
test derives_all_enum_variant_shapes ... ok
test derives_named_tuple_unit_and_generic_structs ... ok
test pure_frame_abandon_discards_structs_and_enums_without_inspecting_fields ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)

running 89 tests
test file_io::tests::normalize_path_falls_back_to_current_dir_for_missing_relative_paths ... ok
test file_io::tests::std_file_io_converts_read_and_write_errors ... ok
test file_io::tests::std_file_io_probe_distinguishes_files_directories_and_missing_paths ... ok
test file_io::tests::std_file_io_lists_only_immediate_directory_entries ... ok
test tests::capability_profiles::every_host_profile_inherits_the_derived_native_stack_budget ... ok
test model::outcome::tests::failure_code_maps_runtime_request_errors_to_host_codes ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_clock_builtins_at_dispatch ... ok
test session::tests::panic_is_complete_response_marks_session_poisoned_and_invalid ... ok
test tests::capability_profiles::default_host_config_keeps_diagnostic_catalog_unrestricted ... ok
test tests::capability_profiles::capability_policy_can_allow_clock_without_filesystem ... ok
test tests::capability_profiles::trusted_host_file_names_uses_the_std_file_io_provider ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_runtime_constructed_filesystem_calls ... ok
test tests::capability_profiles::permissive_policy_without_entropy_provider_returns_noentropy_failed ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_diagnostic_builtins_at_dispatch ... ok
test tests::capability_profiles::default_host_config_installs_entropy_provider_for_random_bytes ... ok
test tests::capability_profiles::permissive_policy_without_file_provider_returns_noio_failed ... ok
test tests::capability_profiles::sandboxed_host_config_still_allows_pure_work_and_path_string_helpers ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_interactive_builtins_at_dispatch ... ok
test tests::capability_profiles::clock_policy_allows_composed_datetime_current_readers ... ok
test tests::capability_profiles::default_host_config_keeps_filesystem_catalog_unrestricted ... ok
test tests::capability_profiles::capability_policy_can_explicitly_allow_diagnostics ... ok
test tests::capability_profiles::capability_policy_can_allow_reads_without_writes ... ok
test tests::capability_profiles::sandboxed_host_config_allows_pure_datetime_forms ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_filesystem_builtins_at_dispatch ... ok
test tests::capability_profiles::random_policy_allows_entropy_reseed ... ok
test tests::capability_profiles::default_host_config_installs_entropy_provider_for_random_password ... ok
test tests::capability_profiles::sandboxed_host_config_with_file_provider_blocks_internal_loader_calls ... ok
test tests::eval_contract::host_failure_codes_report_session_reset_requirement ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_entropy_builtins_but_allows_prng_builtins ... ok
test tests::editor_surface::host_eval_renders_top_level_hold_form_transparently_in_canonical_style ... ok
test tests::editor_surface::host_lower_dump_returns_canonical_text_and_rendered_syntax_failures ... ok
test lifecycle_contract_tests::reset_between_requests_isolates_implicit_session_state ... ok
test tests::session_lifecycle::host_session_and_cancel_handle_are_send ... ok
test tests::editor_surface::host_parse_inspect_complete_and_is_complete_cover_editor_flows ... ok
test tests::editor_surface::host_evaluates_multiline_string_literals_as_ordinary_strings ... ok
test tests::editor_surface::host_eval_reports_value_messages_prints_and_structured_outputs ... ok
test tests::eval_contract::host_eval_script_uses_last_form_and_classifies_syntax_failures ... ok
test tests::eval_contract::host_eval_ordinary_value_has_no_termination_reason ... ok
test tests::eval_contract::host_cancel_handle_interrupts_running_eval_from_another_thread ... ok
test lifecycle_contract_tests::panic_poison_blocks_public_entries_until_reset ... ok
test tests::eval_contract::host_eval_script_suspends_but_resume_fails_without_running_remaining_forms ... ok
test tests::editor_surface::host_print_sink_streams_exclusively_and_clear_restores_buffering ... ok
test tests::editor_surface::host_suspend_resume_flow_wraps_input_requests ... ok
test tests::editor_surface::host_queued_input_completes_without_suspension ... ok
test tests::eval_contract::host_iteration_accumulator_completes_after_multiple_inputs_without_poisoning ... ok
test tests::eval_contract::host_eval_literal_terminated_evaluation_has_no_termination_reason ... ok
test tests::editor_surface::host_processes_inline_multiline_csv_data ... ok
test tests::eval_contract::host_eval_with_unused_handle_and_generous_timeout_still_succeeds ... ok
test tests::eval_contract::host_eval_memory_ceiling_reports_structured_termination_and_session_recovers ... ok
test tests::session_lifecycle::host_boundary_catches_panic_and_poisons_session ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_exit_at_dispatch_but_trusted_allows_it ... ok
test tests::eval_contract::host_eval_recursion_limit_reports_structured_termination_reason ... ok
test tests::session_lifecycle::host_boundary_normal_eval_does_not_poison_session ... ok
test tests::eval_contract::host_failures_use_stable_failure_codes ... ok
test tests::eval_contract::host_eval_file_uses_provider_and_missing_provider_is_other ... ok
test tests::editor_surface::host_message_catalog_supports_global_symbol_and_invalid_queries ... ok
test tests::session_lifecycle::host_session_introspection_enumerates_user_definitions_and_contexts ... ok
test tests::typed_bridge::host_call_enforces_typed_input_depth_cap ... ok
test tests::session_lifecycle::host_session_introspection_survives_poisoned_session ... ok
test tests::typed_bridge::host_call_rejects_empty_head ... ok
test tests::typed_bridge::host_call_normalizes_rational_input ... ok
test tests::typed_bridge::host_call_evaluates_integer_arguments ... ok
test tests::typed_bridge::host_call_resolves_short_and_fully_qualified_symbols ... ok
test tests::typed_bridge::host_call_rejects_output_only_other_argument ... ok
test lifecycle_contract_tests::abandoned_poisoned_session_does_not_contaminate_a_fresh_session ... ok
test tests::typed_bridge::host_call_round_trips_large_integer_and_immediate_rule ... ok
test lifecycle_contract_tests::reset_rebuilds_all_runtime_accumulation_dimensions ... ok
test tests::typed_bridge::host_call_resolves_short_system_head_for_string_join ... ok
test tests::capability_profiles::hermetic_profile_replays_rendered_results_across_sessions_and_reset ... ok
test tests::typed_bridge::host_call_rejects_zero_denominator_rational ... ok
test tests::typed_bridge::host_call_rejects_malformed_big_integer ... ok
test tests::typed_bridge::host_call_treats_string_arguments_as_injection_safe_data ... ok
test tests::typed_bridge::host_call_round_trips_typed_association ... ok
test tests::typed_bridge::host_eval_exposes_immediate_rule_and_rejects_delayed_rule_data_view ... ok
test tests::typed_bridge::host_eval_exposes_ordered_typed_association_entries ... ok
test tests::session_lifecycle::host_reset_removes_print_sink_and_restores_buffering ... ok
test tests::session_lifecycle::host_session_reset_restores_original_config_not_mutated_state ... ok
test tests::typed_bridge::host_eval_exposes_typed_value_variants ... ok
test tests::typed_bridge::host_eval_keeps_delayed_association_out_of_typed_data_view ... ok
test tests::session_lifecycle::host_session_reset_clears_user_state ... ok
test tests::session_lifecycle::host_session_reset_recovers_poisoned_session ... ok
test tests::session_lifecycle::host_session_snapshot_roundtrip_rederives_lazy_list_identically ... ok
test tests::session_lifecycle::host_session_snapshot_roundtrip_relinks_sealed_values ... ok
test tests::session_lifecycle::host_session_snapshot_bytes_roundtrip_and_restore_replaces_user_state ... ok
test tests::capability_profiles::scripting_profile_pins_all_capabilities_and_providers ... ok
test tests::capability_profiles::hermetic_profile_pins_all_capabilities_and_providers ... ok
test tests::capability_profiles::sandboxed_profile_pins_all_capabilities_and_providers ... ok
test tests::session_lifecycle::parallel_host_sessions_move_to_threads_and_isolate_definitions ... ok
test tests::eval_contract::host_eval_with_timeout_interrupts_running_eval ... ok

test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)

running 7 tests
test public_contract_rejects_overlapping_eval_and_preserves_block_suspension ... ok
test public_contract_covers_eval_outputs_without_runtime_types ... ok
test every_public_evaluation_entrypoint_rejects_a_pending_continuation ... ok
test public_contract_preserves_input_suspension_and_resume_boundaries ... ok
test public_contract_covers_editor_requests_and_failures ... ok
test public_config_facade_controls_core_builtin_installation ... ok
test script_suspension_requires_reset_or_restore_while_inspection_remains_available ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)

running 1 test
test host_payloads_declare_bounded_destruction_at_the_crate_edge ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.45s

     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)

running 4 tests
test installer_order_and_no_core_path_are_preserved ... ok
test installer_config_clones_reapply_downstream_builtins ... ok
test installer_builtins_obey_capabilities_and_survive_reset ... ok
test installer_snapshot_restore_round_trips_definitions_not_builtin_handlers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)

running 7 tests
test file_and_entropy_providers_from_arc_share_identity_and_state ... ok
test host_session_builder_policy_and_core_builtin_controls_are_applied ... ok
test host_config_uses_injected_file_io_for_get_and_needs ... ok
test host_request_string_adapters_and_defaults_cover_public_facade ... ok
test host_config_tracing_and_assert_flags_flow_into_built_sessions ... ok
test host_config_uses_fixed_entropy_for_reseed_random ... ok
test restore_from_snapshot_source_that_requests_input_returns_synthesized_failure ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)

running 16 tests
test stale_cancel_cannot_erase_a_current_request_cancel ... ok
test restore_from_invalid_bytes_returns_failure_without_panic_or_state_loss ... ok
test pending_input_request_tracks_suspension_and_clears_after_resume ... ok
test eval_with_timeout_returns_normal_value_before_deadline ... ok
test cancel_accepted_before_eval_request_survives_request_prologue ... ok
test cancel_handle_targets_one_request_and_is_inert_after_completion ... ok
test resource_limit_returns_terminated_evaluation_and_session_remains_usable ... ok
test definitions_and_contexts_start_clean_and_track_user_state ... ok
test host_session_and_cancel_handle_are_send_across_worker_thread ... ok
test structured_host_value_depth_cap_preserves_text_and_uses_other_for_deep_tail ... ok
test cancel_handle_interrupts_running_eval_and_session_remains_usable ... ok
test reset_is_idempotent_and_old_cancel_handles_do_not_affect_rebuilt_session ... ok
test snapshot_restore_preserves_package_contexts_and_replaces_prior_state ... ok
test replace_repeated_limit_returns_terminated_success ... ok
test terminated_snapshot_restore_fails_while_unlimited_restore_succeeds ... ok
test eval_with_timeout_terminates_long_eval_and_session_remains_usable ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)

running 2 tests
test sandboxed_host_config_blocks_random_password_policy_options ... ok
test missing_entropy_provider_fails_random_password_policy_options ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)

running 6 tests
test battered_session_recovers_after_repeated_recoverable_limit_trips ... ignored
test cancel_rewrite_loop_stability_10_cycles ... ignored
test cancel_interrupts_heavy_rewrite_loop_and_session_recovers ... ok
test snapshot_restore_preserves_20_generated_definitions_and_attributes ... ok
test eval_with_timeout_interrupts_heavy_rewrite_loop_and_session_recovers ... ok
test bug_snapshot_restore_500_generated_definitions_overflows_native_stack ... ok

test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.20s

     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)

running 13 tests
test immediate_resume_finishes_with_a_generous_host_timeout ... ok
test rejected_timeout_cannot_arm_or_shorten_a_pending_request ... ok
test reset_releases_suspended_timeout_and_invalidates_its_cancel_handle ... ok
test accepted_cancellation_survives_timeout_arming_and_the_request_prologue ... ok
test cancellation_during_resumed_work_is_observed_with_timeout_still_armed ... ok
test rejected_entrypoints_and_inspection_preserve_a_suspended_deadline ... ok
test two_suspensions_keep_the_same_host_deadline ... ok
test panics_during_timed_eval_or_resume_poison_until_reset ... ok
test timeout_can_expire_during_a_resumed_segment ... ok
test cancellation_of_expired_suspensions_wins_before_input_conversion ... ok
test expired_input_suspensions_resume_under_the_original_host_deadline ... ok
test completion_and_syntax_failure_release_the_owned_timeout ... ok
test cancellation_via_each_evaluation_entrypoint_releases_the_suspended_timeout ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s

     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)

running 12 tests
test public_cancel_handle_acquired_while_suspended_abandons_before_input_conversion ... ok
test public_cancelled_with_suspension_restores_scope_without_running_continuation ... ok
test public_cancelled_block_suspension_restores_scope_without_running_continuation ... ok
test public_cancelled_script_suspension_is_a_reset_free_escape_route ... ok
test public_cancelled_nested_with_cleanup_runs_cleanup_and_abandons_nested_input ... ok
test public_cancelled_suspension_restores_attribute_iterator_and_option_state ... ok
test public_cancelled_module_suspension_reclaims_local_without_running_continuation ... ok
test public_rollback_matrix_restores_lexical_iterator_attribute_and_option_state ... ok
test public_cancelled_suspension_restores_prng_and_abort_protect_state ... ok
test public_monitor_matrix_restores_capture_package_and_cleanup_owners ... ok
test public_cancelled_suspension_restores_reap_monitor_trace_and_constraint_state ... ok
test public_monitor_matrix_restores_reap_monitor_trace_constraints_random_and_abort_state ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)

running 106 tests
test adapter::render::tests::error_output_covers_resource_limit_failures ... ok
test adapter::render::tests::error_output_covers_memory_constraint_failures ... ok
test json_support::tests::parser_handles_nested_json_scalars_and_arrays ... ok
test adapter::render::tests::json_from_typed_rejects_ambiguous_or_partial_trees ... ok
test adapter::render::tests::json_from_typed_maps_supported_tree_and_escapes_strings ... ok
test json_support::tests::parser_reports_malformed_json_without_panicking ... ok
test json_support::tests::parser_round_trips_utf8_and_surrogate_pairs_in_strings ... ok
test adapter::render::tests::error_output_covers_time_constraint_failures ... ok
test signature::tests::hmac_sha256_matches_known_test_vector ... ok
test adapter::render::tests::render_rule_application_source_covers_replace ... ok
test tests::bootstrap::connection_file_parser_loads_jupyter_connection_shapes ... ok
test tests::bootstrap::connection_file_parser_reports_missing_required_fields ... ok
test tests::bootstrap::connection_file_parser_round_trips_unicode_string_fields ... ok
test tests::bootstrap::local_socket_transport_backend_exposes_frontend_boundary_for_frontend_style_flows ... ok
test tests::bootstrap::socket_session_config_derives_zero_mq_facing_socket_shapes ... ok
test tests::bootstrap::local_socket_transport_backend_echoes_heartbeat_payloads ... ok
test tests::bootstrap::connection_file_signature_policy_respects_declared_signature_scheme ... ok
test tests::bootstrap::bootstrap_config_derives_endpoints_and_wire_policy_from_connection_file ... ok
test tests::crate_manifest_keeps_jupyter_on_host_facade ... ok
test tests::adapter::dispatch::dispatch_request_emits_busy_and_idle_status_for_kernel_info ... ok
test tests::adapter::dispatch::shell_request_envelope_dispatches_kernel_info_and_shutdown ... ok
test tests::adapter::dispatch::shell_request_envelope_dispatches_is_complete_requests ... ok
test tests::adapter::execute_json_representable_value_adds_json_without_changing_expr_mime ... ok
test tests::adapter::complete_request_replaces_the_full_symbol_when_cursor_is_inside_it ... ok
test tests::bootstrap::mock_transport_backend_exposes_bootstrap_boundary_for_future_real_transports ... ok
test tests::adapter::dispatch::shell_request_envelope_dispatches_execute_requests ... ok
test tests::adapter::execute_depth_capped_typed_value_omits_json_mime ... ok
test tests::bootstrap::socket_driver_observes_shutdown_promptly_during_running_execute ... ok
test tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately ... ok
test tests::transport_coverage::codec_preserves_reply_routing_ids_for_replies_and_stdin_requests ... ok
test tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle ... ok
test tests::transport_coverage::codec_rejects_wrong_channels_for_response_iopub_and_stdin_messages ... ok
test tests::transport_coverage::codec_rejects_missing_signature_bad_delimiter_and_session_mismatch ... ok
test tests::transport_coverage::local_socket_transport_pair_routes_messages_and_heartbeats_in_memory ... ok
test tests::adapter::dispatch::dispatch_request_handles_comm_info_without_touching_runtime_session ... ok
test tests::adapter::execute_request_cycle_termination_emits_stderr_trace_and_result ... ok
test tests::wire::execute_request_round_trips_allow_stdin_flag ... ok
test tests::adapter::execute_literal_cancelled_termination_value_remains_successful ... ok
test tests::adapter::inspect_request_maps_symbol_at_cursor_to_session_inspect_response ... ok
test tests::adapter::execute_request_maps_to_session_eval_and_tracks_execution_count ... ok
test tests::adapter::execute_non_representable_value_keeps_only_existing_expr_mime ... ok
test tests::wire::request_headers_round_trip_date_field_through_wire_codec ... ok
test tests::wire::optional_signature_policy_accepts_unsigned_messages_but_still_checks_signed_ones ... ok
test tests::wire::signed_transport_codec_rejects_signature_mismatch ... ok
test tests::wire::signed_transport_codec_round_trips_request_messages ... ok
test tests::bootstrap::socket_driver_pumps_heartbeat_and_execute_over_local_session ... ok
test tests::wire::transport_codec_accepts_structured_frontend_metadata ... ok
test tests::wire::transport_codec_decodes_escaped_and_utf8_unicode_request_payloads ... ok
test tests::wire::transport_codec_rejects_invalid_channel_for_request_type ... ok
test tests::wire::transport_codec_round_trips_stdin_input_messages ... ok
test transport::json::encode::tests::encodes_iopub_and_helper_json_values_without_losing_escaping ... ok
test transport::json::encode::tests::encodes_less_common_request_and_response_shapes_as_structured_json ... ok
test transport::json::parse::tests::aggregate_field_helpers_parse_payloads_history_language_and_help_links ... ok
test transport::json::parse::tests::aggregate_field_helpers_reject_malformed_nested_shapes ... ok
test transport::json::parse::tests::field_helpers_accept_supported_shapes_and_report_typed_errors ... ok
test transport::json::parse::tests::iopub_content_parsing_covers_payload_variants_and_errors ... ok
test transport::json::parse::tests::message_header_parsing_defaults_date_and_reports_field_errors ... ok
test transport::json::parse::tests::request_content_parsing_covers_defaults_known_kinds_and_ignored_messages ... ok
test tests::adapter::dispatch::control_request_interrupt_updates_lifecycle_state ... ok
test transport::json::parse::tests::response_content_parsing_covers_reply_variants_and_malformed_statuses ... ok
test transport::json::parse::tests::status_parsers_cover_all_known_values_and_error_fields ... ok
test zeromq::tests::fake::zeromq_backend_binds_expected_socket_endpoints ... ok
test zeromq::tests::fake::zeromq_backend_propagates_independently_resolved_bind_endpoints ... ok
test tests::pump::pump_next_prioritizes_control_messages_before_shell_messages ... ok
test tests::adapter::dispatch::dispatch_request_replies_to_debug_request_as_unsupported_without_exiting ... ok
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... FAILED
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... FAILED
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... FAILED
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... FAILED
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... FAILED
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... FAILED
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... FAILED
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... FAILED
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... FAILED
test zeromq::tests::support::harness_tests::parent_scope_excludes_sentinels_including_late_duplicates_and_stops_at_target_idle ... ok
test tests::adapter::dispatch::dispatch_request_ignores_comm_messages_without_affecting_session_state ... ok
test tests::adapter::dispatch::dispatch_request_emits_iopub_status_execute_input_and_execute_result ... ok
test tests::pump::pump_until_idle_stops_after_shutdown_and_leaves_later_shell_work_queued ... ok
test tests::adapter::execute_request_with_tracing_emits_display_data_separately_from_execute_result ... ok
test tests::bootstrap::frontend_style_execute_round_trip_uses_local_socket_backend ... ok
test tests::pump::pump_next_prioritizes_stdin_replies_while_kernel_is_waiting_for_input ... ok
test tests::adapter::complete_request_maps_cursor_prefix_to_session_completion_response ... ok
test tests::adapter::is_complete_request_maps_runtime_completeness_to_kernel_shape ... ok
test tests::adapter::dispatch::control_request_shutdown_transitions_kernel_to_shutdown ... ok
test tests::bootstrap::frontend_style_stdin_round_trip_uses_local_socket_backend ... ok
test tests::bootstrap::socket_driver_resumes_stdin_round_trip_over_local_session ... ok
test tests::stdin::execute_request_with_inputstring_stages_stdin_and_defers_execute_reply ... ok
test tests::stdin::execute_request_with_disallowed_stdin_returns_input_error_reply ... ok
test tests::stdin::adapter_can_stage_stdin_requests_for_future_execute_integration ... ok
test tests::stdin::input_reply_without_parent_header_resumes_pending_execute ... ok
test tests::stdin::input_reply_resumes_execute_without_rerunning_prefix_side_effects ... ok
test tests::wire::comm_info_request_round_trips_with_empty_comm_registry ... ok
test tests::stdin::adapter_rejects_stdin_requests_when_execute_disallows_them ... ok
test zeromq::tests::fake::zeromq_backend_reports_malformed_shell_frames_as_structured_wire_errors ... ok
test tests::wire::event_pump_ignores_comm_messages_and_keeps_processing_later_execute ... ok
test tests::wire::history_request_round_trips_with_empty_reply ... ok
test tests::wire::debug_request_round_trips_as_explicit_unsupported_reply ... ok
test tests::wire::transport_encoding_serializes_reply_and_iopub_messages ... ok
test tests::wire::mock_socket_transport_routes_reply_and_iopub_messages_by_channel ... ok
test tests::wire::kernel_info_round_trips_on_control_channel ... ok
test tests::stdin::input_reply_resumes_delayed_message_templates_through_internal_string_driver ... ok
test tests::stdin::input_reply_resumes_execute_and_preserves_shell_routing_ids ... ok
test tests::wire::transport_codec_decodes_execute_reply_and_iopub_messages ... ok
test zeromq::tests::fake::zeromq_backend_round_trips_execute_over_fake_router_socket ... ok
test tests::stdin::input_reply_resumes_multi_sequence_matcher_backtracking_across_multiple_prompts ... ok
test zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets ... ok

failures:

---- zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket stdout ----

thread 'zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets stdout ----

thread 'zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip stdout ----

thread 'zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages stdout ----

thread 'zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata stdout ----

thread 'zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown stdout ----

thread 'zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply stdout ----

thread 'zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end stdout ----

thread 'zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

---- zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages stdout ----

thread 'zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages' panicked at crates/rum-jupyter/src/zeromq/tests/../../zeromq_native_test_support.rs:346:17:
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip


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

error: test failed, to rerun pass `-p rum-jupyter --lib`
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/rum_repl-78a8fab085fc98d8)

running 45 tests
test commands::tests::parse_history_reports_invalid_counts ... ok
test commands::tests::parse_format_returns_a_structured_command ... ok
test commands::tests::trace_step_renderer_degrades_for_missing_and_unknown_fields ... ok
test tests::repl_format_command_rejects_invalid_line_width ... ok
test tests::repl_bare_trace_command_shows_usage_instead_of_evaluating ... ok
test tests::repl_bare_inspect_command_shows_usage_instead_of_evaluating ... ok
test tests::repl_messages_lists_catalog_entries_for_a_symbol ... ok
test tests::repl_complete_command_lists_builtin_and_user_candidates ... ok
test tests::repl_trace_command_filters_with_a_pattern_form ... ok
test tests::repl_renders_hold_form_transparently_in_canonical_style ... ok
test tests::repl_format_command_accepts_suffix_equals_line_width ... ok
test tests::repl_inspect_command_does_not_fall_through_to_evaluation ... ok
test tests::repl_parse_command_renders_source_diagnostic_for_bad_input ... ok
test tests::repl_inspect_surfaces_message_definition_flags ... ok
test tests::repl_renders_parser_diagnostics_for_regular_input_errors ... ok
test tests::repl_inspect_surfaces_attribute_flags ... ok
test tests::repl_messages_reports_when_a_symbol_has_no_messages ... ok
test tests::repl_reports_incomplete_multiline_input_when_input_ends ... ok
test tests::repl_format_command_respects_custom_line_width ... ok
test commands::tests::execute_history_renders_numbered_entries ... ok
test tests::repl_trace_command_passes_a_numeric_depth_levelspec ... ok
test tests::repl_inspect_shows_definition_summaries ... ok
test tests::repl_parse_command_prints_syntax_debug_output ... ok
test tests::repl_handles_help_and_quit ... ok
test tests::repl_format_command_prints_canonical_surface_form ... ok
test tests::repl_trace_command_renders_a_small_tree ... ok
test tests::repl_renders_runtime_messages_before_results ... ok
test tests::repl_lower_command_prints_canonical_core_expression ... ok
test tests::repl_reads_multiline_input_until_a_form_is_complete ... ok
test tests::repl_history_persists_when_a_history_path_is_configured ... ok
test tests::repl_trace_without_rules_is_byte_for_byte_unchanged ... ok
test tests::repl_trace_rules_composes_with_form_rewrite_and_depth ... ok
test completeness::tests::multiline_completeness_work_is_linear_in_appended_source ... ok
test tests::repl_trace_rules_renders_a_definition_stored_step ... ok
test tests::repl_trace_command_truncates_by_default_and_full_is_the_escape_hatch ... ok
test tests::repl_trace_command_uses_the_host_failure_renderer_for_bad_input ... ok
test tests::repl_trace_command_rewrites_a_bare_symbol_filter ... ok
test tests::repl_trace_command_renders_messages_before_the_tree ... ok
test tests::repl_trace_rules_command_renders_a_small_step ... ok
test tests::repl_trace_rules_names_the_overlapping_definition_that_fired ... ok
test tests::repl_history_persists_multiline_inputs_as_single_logical_entries ... ok
test tests::repl_trace_rules_truncates_and_full_is_the_escape_hatch ... ok
test completeness::tests::incremental_completeness_matches_the_host_matrix ... ok
test completeness::tests::fast_incomplete_hint_never_hides_lexer_or_parser_errors ... ok
test completeness::tests::certified_incomplete_prefix_corpus_agrees_with_the_host_parser ... ok

test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s

     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)

running 1015 tests
test runtime::accumulator_chain::tests::live_weak_does_not_prevent_drain ... ok
test runtime::accumulator_chain::tests::exclusive_depth_64 ... ok
test runtime::accumulator_chain::tests::exclusive_depth_16 ... ok
test runtime::accumulator_chain::tests::shared_tail_stops_then_drains ... ok
test runtime::capabilities::tests::capability_policy_allowlist_is_default_deny ... ok
test runtime::accumulator_chain::tests::wrapper_is_transparent_and_extraction_preserves_ownership ... ok
test runtime::capabilities::tests::capability_policy_can_explicitly_allow_diagnostics ... ok
test runtime::capabilities::tests::capability_policy_narrowing_preserves_existing_restrictions ... ok
test runtime::capabilities::tests::capability_policy_builder_can_allow_and_deny_individual_capabilities ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_clock_builtins ... ok
test runtime::call_preparation::tests::planner_allows_raw_sequence_splicing_for_held_arguments ... ok
test runtime::call_preparation::tests::planner_respects_evaluate_wrapper_for_held_arguments ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_diagnostic_builtins ... ok
test runtime::capabilities::tests::capability_policy_defaults_to_unrestricted ... ok
test runtime::call_preparation::tests::planner_keeps_held_arguments_verbatim_under_holdallcomplete ... ok
test runtime::call_preparation::tests::planner_respects_unevaluated_wrapper_for_nonheld_arguments ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_io_builtins ... ok
test runtime::definitions::application::replace_all::plan::frame_root_trace_tests::rewrite_rule_plan_visits_the_exact_rule_roots ... ok
test runtime::definitions::definition_lhs_preparation::syntax_policy::tests::every_registered_matcher_head_has_an_explicit_lhs_policy ... ok
test runtime::definitions::unique_symbols::tests::keeps_small_symbol_sets_on_the_linear_path ... ok
test runtime::evaluation_cache::tests::call_stability_profile_reports_per_source_head_outcomes ... ignored, release-only profiling gate; run with RUM_PROFILE_EVALUATION_CACHE=1
test runtime::definitions::unique_symbols::tests::preserves_first_seen_order_across_the_adaptive_index_threshold ... ok
test runtime::builtin_catalog::tests::moved_stdlib_system_attributes_remain_kernel_known_without_registration ... ok
test runtime::builtin_catalog::tests::core_catalog_drives_both_dispatch_and_default_attributes ... ok
test runtime::evaluation_cache::tests::shared_args_make_mut_profiles_only_cow_and_preserves_alias_values ... ok
test runtime::evaluation_cache::tests::stability_native_stack_poll_is_strided_and_fails_closed ... ok
test runtime::evaluation_cache::tests::association_stability_index_preserves_distinct_and_duplicate_verdicts ... ok
test runtime::evaluation_cache::tests::path_eviction_removes_only_the_named_exact_call_nodes ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::appended_chain_abandonment_uses_drain ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::into_vec_preserves_prefix_and_tail_order_when_unique_or_shared ... ok
test runtime::evaluation_cache::tests::evaluation_cache_is_invalidated_by_new_down_value ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::shared_tail_survives_abandoned_clone ... ok
test runtime::kernel_data::rewrite_data::suspended_state::tests::abandoning_shared_rewrite_queue_preserves_remaining_binding_order ... ok
test runtime::kernel_data::structure_data::exact_paths::tests::collect_exact_paths_for_position_spec_forced_front_preserves_branch_order ... ok
test runtime::kernel_data::structure_data::exact_paths::tests::exact_expr_at_path_forced_front_matches_each_branched_witness ... ok
test runtime::kernel_data::structure_data::replace::plan::tests::replace_plan_resolves_duplicates_and_prefixes_during_insertion ... ok
test runtime::kernel_data::structure_data::replace::plan::tests::root_target_discards_existing_and_later_descendants ... ok
test runtime::evaluation_cache::tests::append_to_preserves_an_aliased_list_value ... ok
test runtime::kernel_data::structure_data::replace::suspended_state::tests::abandoning_guarded_target_branch_preserves_surviving_chunks_and_updates ... ok
test runtime::definitions::definition_lhs_preparation::tests::already_canonical_lhs_reuses_its_shared_arguments ... ok
test runtime::kernel_data::structure_data::exact_paths::tests::replace_exact_path_forced_front_matches_head_and_child_edges ... ok
test runtime::evaluation_cache::tests::evaluation_cache_is_invalidated_by_attribute_change ... ok
test runtime::evaluation_cache::tests::generation_stale_pointer_matched_entry_is_evicted_without_proof ... ok
test runtime::evaluation_cache::tests::repeated_writes_to_one_symbol_keep_invalidation_sound ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_update_preserves_an_aliased_value ... ok
test runtime::evaluation_cache::tests::pointer_mismatched_entry_is_left_untouched ... ok
test runtime::eval_call_prepare::abandonment_tests::abandoning_prepared_arguments_and_restores_skips_remaining_arguments ... ok
test runtime::session::session_requests::tests::eval_error_mapper_preserves_non_request_surface_limit_codes ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_alpha_rename_does_not_rewrite_inserted_bound_values ... ok
test runtime::evaluation_cache::tests::dirty_pointer_matched_entry_is_evicted_without_proof ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_substitution_preserves_unchanged_sibling_call_shared_nodes ... ok
test runtime::evaluation_cache::tests::proof_reinsertion_uses_current_generation_dirty_snapshot_and_bloom ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_sequence_splice_preserves_spliced_item_shared_nodes ... ok
test runtime::evaluation_cache::tests::stability_cache_survives_unrelated_symbol_writes ... ok
test runtime::substitution::lexical_substitution::tests::rule_rhs_substitution_respects_pattern_locals ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_substitution_reuses_bound_call_shared_nodes ... ok
test runtime::substitution::lexical_substitution::tests::with_body_treats_binding_names_as_shadowed ... ok
test runtime::substitution::lexical_substitution::tests::explicit_function_parameters_shadow_outer_bindings ... ok
test runtime::suspension::primitive::tests::binding_vec_continuation_round_trips_multiple_symbol_maps ... ok
test runtime::suspension::primitive::tests::option_vec_expr_continuation_round_trips_some_none_and_fallback_payloads ... ok
test runtime::suspension::traversal::tests::option_cases_level_spec_continuation_round_trips_some_none_and_bare_spec ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_direct_update_rolls_back_append_and_replace ... ok
test runtime::suspension::rewrite_rules::tests::option_rewrite_rule_info_continuation_round_trips_some_none_and_fallbacks ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_direct_update_polls_interrupt_before_mutation ... ok
test runtime::evaluation_cache::tests::valid_call_stability_proof_preflight_is_non_consuming_and_reuses_validity ... ok
test runtime::definitions::value_table_assignment::tests::abandoning_prepared_value_table_keeps_old_definitions_and_skips_pending_lhs ... ok
test runtime::pattern_syntax::tests::matcher_pattern_registry_round_trips_system_heads_only ... ok
test runtime::evaluation_cache::tests::removing_valid_call_stability_entry_preserves_evaluation_outcome ... ok
test runtime::lowering::tests::lowering_error_restores_front_budget ... ok
test runtime::evaluation_cache::tests::repeated_unrelated_writes_do_not_bump_call_stability_generation ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_recovery_falls_back_when_data_became_unstable ... ok
test runtime::evaluation_cache::tests::association_stability_tail::unstable_early_child_cache_trace_matches_recursive_front ... ok
test runtime::tests::arithmetic::power_memory_projection_refuses_before_large_allocation ... ok
test runtime::evaluation_cache::tests::association_stability_tail::duplicate_key_cache_trace_matches_recursive_front ... ok
test runtime::kernel_data::structure_data::replace::plan_application::tests::iterative_replace_plan_rebuild_preserves_unchanged_sibling_storage ... ok
test runtime::tests::arithmetic::power_integer_work_limit_guards_large_denominators ... ok
test runtime::lowering::tests::head_chains_below_bound_preserve_every_element ... ok
test runtime::tests::association_lookup_cache::association_lookup_index_weak_identity_survives_cow_updates ... ok
test runtime::tests::arithmetic::exact_division_and_negative_integer_powers_produce_rationals ... ok
test runtime::tests::association_part_cursor::below_threshold_part_reads_stay_on_the_linear_scan_path ... ok
test runtime::tests::association_part_cursor::duplicate_association_keys_keep_first_position_and_last_value ... ok
test runtime::evaluation_cache::tests::association_stability_tail::rule_delayed_cache_trace_matches_recursive_front ... ok
test runtime::tests::association_part_cursor::part_reads_reuse_the_lookup_cache_and_observe_cow_writes ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_direct_update_keeps_args_identity_and_reinserts_proof ... ok
test runtime::tests::arithmetic::power_integer_work_limit_allows_normal_and_trivial_results ... ok
test runtime::tests::arithmetic::power_reads_the_live_integer_work_limit ... ok
test runtime::tests::arithmetic::round_handles_exact_numbers_decimal_literals_and_half_even_ties ... ok
test runtime::tests::arithmetic::powermod_negative_exponents_use_modular_inverse ... ok
test runtime::evaluation_cache::tests::append_to_args_become_exclusive_after_definition_take_and_cache_entry_removal ... ok
test runtime::tests::association_lookup_cache::repeated_wide_association_reads_build_one_linear_index ... ok
test runtime::evaluation_cache::tests::association_stability_tail::recursive_front_exhaustion_owns_intervening_ordinary_calls ... ok
test runtime::tests::arithmetic::factorial_respects_integer_work_limit ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_nested_multi_and_bare_part_targets_use_fallback ... ok
test runtime::tests::arithmetic::rational_atoms_expose_a_rational_head_for_patterns ... ok
test runtime::tests::arithmetic::power_respects_integer_work_limit_for_large_positive_exponents ... ok
test runtime::tests::arithmetic::plus_and_times_fold_numeric_parts_without_forcing_symbolic_algebra ... ok
test runtime::tests::comparison_lowering::comparison_lowering_preserves_resolver_call_count_and_order ... ok
test runtime::tests::arithmetic::exact_integer_arithmetic_evaluates_core_heads ... ok
test runtime::tests::arithmetic::exact_integer_mod_quotient_gcd_lcm_and_powermod_follow_floor_style_semantics ... ok
test runtime::tests::arithmetic::power_realistic_memory_ceiling_admits_one_million_bit_result ... ok
test runtime::tests::arithmetic::scalar_math_builtins_thread_over_lists_and_leave_skipped_forms_symbolic ... ok
test runtime::tests::arithmetic::scalar_math_builtins_report_declared_attributes ... ok
test runtime::tests::arithmetic::scalar_math_builtins_exact_cases ... ok
test runtime::lowering::tests::branched_lowering_certifies_front_and_tail ... ok
test runtime::tests::arithmetic::total_evaluates_documented_level_forms_and_leaves_invalid_specs_inert ... ok
test runtime::tests::association_write_index::small_association_writes_bypass_the_lookup_cache ... ok
test runtime::tests::association_write_index::rewriting_an_appended_key_replaces_it_through_the_index ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::accumulator_is_abandoned_mid_iteration_without_resuming_body ... ok
test runtime::tests::context_and_loader::bare_symbols_search_context_path_before_current_context ... ok
test runtime::tests::context_and_loader::begin_package_and_end_package_restore_outer_context_and_export_package_context ... ok
test runtime::tests::context_and_loader::begin_and_end_balance_context_and_context_path ... ok
test runtime::lowering::tests::branched_lowering_matches_original_exact_result ... ok
test runtime::tests::context_and_loader::begin_package_expands_relative_context_specs_against_the_current_context ... ok
test runtime::lowering::tests::tail_preserves_sugar_and_symbol_resolution_order ... ok
test runtime::tests::context_and_loader::begin_package_loads_present_dependency_and_enters_package ... ok
test runtime::evaluation_cache::tests::association_stability_tail::dirty_and_generation_cache_traces_match_recursive_front ... ok
test runtime::tests::context_and_loader::begin_package_usage_messages_make_public_symbols_visible_from_private_context ... ok
test runtime::tests::context_and_loader::begin_package_second_dependency_failure_restores_state_and_package_list ... ok
test runtime::tests::association_write_index::repeated_replacement_writes_stay_indexed ... ok
test runtime::tests::context_and_loader::begin_package_dependencies_stay_on_private_context_path_when_already_loaded ... ok
test runtime::tests::context_and_loader::context_variables_and_builtin_inspection_reflect_session_contexts ... ok
test runtime::tests::context_and_loader::context_builtin_reports_session_and_symbol_contexts_without_evaluating_symbols ... ok
test runtime::tests::context_and_loader::begin_package_missing_dependency_fails_closed_and_restores_state ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_absent_stale_and_dirty_proofs_recover_by_recomputation ... ok
test runtime::tests::context_and_loader::explicitly_protected_session_configuration_variables_reject_assignments ... ok
test runtime::tests::context_and_loader::context_path_and_packages_assignments_update_session_fields ... ok
test runtime::tests::association_write_index::appended_keys_after_a_wide_read_stay_readable ... ok
test runtime::tests::context_and_loader::script_forms::split_script_into_forms_skips_initial_shebang_trivia ... ok
test runtime::tests::context_and_loader::script_forms::split_script_into_forms_stops_hard_syntax_errors_from_poisoning_later_forms ... ok
test runtime::tests::context_and_loader::script_forms::split_script_into_forms_treats_multiline_comments_as_incomplete_trivia ... ok
test runtime::tests::context_and_loader::file_loading::get_loads_a_script_file_and_returns_the_last_value ... ok
test runtime::tests::context_and_loader::contexts_lists_known_session_contexts ... ok
test runtime::tests::context_and_loader::file_loading::get_rolls_back_loader_side_effects_and_collapses_failed_file_messages ... ok
test runtime::tests::arithmetic::wide_plus_and_times_group_late_symbolic_duplicates ... ok
test runtime::tests::context_and_loader::file_loading::findfile_resolves_package_names_and_file_wrappers ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_observable_or_constrained_environments_use_fallback ... ok
test runtime::tests::context_and_loader::script_forms::eval_script_pins_the_second_form_syntax_error_message ... ok
test runtime::tests::context_and_loader::file_loading::get_cycle_termination_does_not_abort_remaining_file_forms ... ok
test runtime::tests::context_and_loader::file_loading::needs_loads_package_once_and_tracks_loaded_packages ... ok
test runtime::tests::context_and_loader::file_loading::loader_builtins_accept_resolved_load_target_specs ... ok
test runtime::tests::context_and_loader::file_loading::get_treats_resource_termination_as_failed_load_and_rolls_back ... ok
test runtime::tests::context_and_loader::file_loading::missing_findfile_emits_runtime_message ... ok
test runtime::tests::context_and_loader::script_forms::eval_script_preserves_messages_emitted_before_a_later_error ... ok
test runtime::tests::context_and_loader::file_loading::get_propagates_exit_without_collapsing_it_to_failed_load ... ok
test runtime::tests::context_and_loader::file_loading::nested_get_prefers_the_current_loader_directory_over_package_search_paths ... ok
test runtime::tests::context_and_loader::file_loading::missing_needs_returns_failed_and_emits_messages ... ok
test runtime::tests::context_and_loader::file_loading::needs_accepts_resolved_context_and_file_specs ... ok
test runtime::tests::context_and_loader::file_loading::needs_supports_explicit_file_targets ... ok
test runtime::tests::context_and_loader::path_assignment_remains_writable_without_explicit_protect ... ok
test runtime::tests::context_and_loader::script_forms::lex_fallback_preserves_normalized_crlf_values_and_file_error_spans ... ok
test runtime::tests::debug_tools::rum_profile_hooks_are_inert_without_profile_env ... ok
test runtime::tests::debug_tools::on_and_off_gate_matching_messages_at_session_scope ... ok
test runtime::tests::debug_tools::echo_family_prints_and_returns_values_in_the_current_first_pass_subset ... ok
test runtime::tests::context_and_loader::file_loading::get_and_needs_syntax_messages_report_the_loaded_file_location ... ok
test runtime::tests::debug_tools::short_uses_the_current_canonical_text_truncation_subset ... ok
test runtime::tests::debug_tools::trace_detail_returns_the_full_evaluation_pass_association_without_leaking_trace_state ... ok
test runtime::tests::debug_tools::trace_detail_identifies_the_downvalue_that_won_among_overlapping_definitions ... ok
test runtime::tests::debug_tools::trace_detail_form_filter_matches_the_underlying_expression_not_the_association ... ok
test runtime::tests::debug_tools::trace_detail_preserves_definition_symbol_kind_and_value ... ok
test runtime::tests::debug_tools::trace_print_uses_the_same_nested_structure_for_deeper_trace_branches ... ok
test runtime::tests::debug_tools::trace_detail_supports_levelspec_and_heads_options ... ok
test runtime::tests::debug_tools::trace_detail_has_the_same_list_nesting_shape_as_trace ... ok
test runtime::tests::debug_tools::trace_uses_recorded_depths_to_keep_nested_branches_stable ... ok
test runtime::tests::debug_tools::trace_print_emits_trace_lines_and_returns_the_underlying_value ... ok
test runtime::tests::definition_lhs_preparation::active_definition_lhs_positions_complete_own_value_evaluation_for_both_set_forms ... ok
test runtime::tests::definition_lhs_preparation::clear_then_reinstall_has_no_stale_prepared_identity_or_matcher ... ok
test runtime::tests::definition_lhs_preparation::adding_hold_attributes_after_an_existing_definition_keeps_old_and_new_identities_live ... ok
test runtime::tests::definition_lhs_preparation::active_lhs_and_pattern_test_pattern_operands_still_prepare_once ... ok
test runtime::tests::debug_tools::trace_returns_a_first_pass_structured_expression_list_without_leaking_session_trace_state ... ok
test runtime::tests::definition_lhs_preparation::active_nested_calls_dispatch_held_nested_calls_do_not_and_callable_heads_prepare_first ... ok
test runtime::tests::definitions_and_rules::delayed_self_alias_preserves_downvalues ... ok
test runtime::tests::definition_lhs_preparation::reassigned_then_cleared_lhs_own_value_changes_only_future_definition_identities ... ok
test runtime::tests::definition_lhs_preparation::one_metadata_snapshot_controls_every_argument_even_after_an_earlier_mutation ... ok
test runtime::tests::definitions_and_rules::defaultvalues_and_definition_include_explicit_syntax_information ... ok
test runtime::tests::definition_lhs_preparation::definition_lhs_pattern_regions_prepare_constraints_but_defer_binders_and_predicates ... ok
test runtime::tests::definitions_and_rules::defaultvalues_lists_optional_defaults_and_clearall_removes_them ... ok
test runtime::tests::comparison_lowering::comparison_parser_rejects_a_long_uniform_chain_gracefully ... ok
test runtime::tests::definitions_and_rules::definition_returns_empty_hold_for_symbols_without_definitions ... ok
test runtime::tests::definition_lhs_preparation::held_definition_lhs_positions_keep_raw_symbols_and_source_lookup_reaches_them ... ok
test runtime::tests::definition_lhs_preparation::held_ground_flat_and_orderless_identities_are_reachable_from_both_forms ... ok
test runtime::tests::definitions_and_rules::definition_returns_held_first_pass_definition_statements ... ok
test runtime::tests::definitions_and_rules::definition_inspection_builtins_accept_symbol_name_strings ... ok
test runtime::tests::definitions_and_rules::immediate_self_alias_preserves_downvalues ... ok
test runtime::tests::definition_lhs_preparation::definition_lhs_preparation_obeys_evaluate_unevaluated_and_sequence_controls ... ok
test runtime::tests::definitions_and_rules::information_uses_catalog_and_symbol_name_usage_fallbacks_and_stored_message_tags ... ok
test runtime::tests::definition_lhs_preparation::matcher_wrappers_keep_literal_pattern_regions_raw_and_prepare_nested_constraints ... ok
test runtime::tests::definitions_and_rules::information_returns_first_pass_symbol_summary ... ok
test runtime::tests::definitions_and_rules::self_referential_delayed_ownvalue_returns_symbol_without_message ... ok
test runtime::tests::definitions_and_rules_cases::cases_can_match_the_root_expression ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules::ownvalues_lists_immediate_and_delayed_symbol_definitions ... ok
test runtime::tests::definitions_and_rules::set_assigns_simple_symbol_and_lookup_works ... ok
test runtime::tests::definitions_and_rules_cases::cases_collects_nested_matches ... ok
test runtime::tests::definitions_and_rules_cases::cases_does_not_treat_heads_as_separate_candidates ... ok
test runtime::tests::definitions_and_rules::setdelayed_stores_rhs_without_immediate_evaluation ... ok
test runtime::tests::definitions_and_rules_cases::cases_applies_rule_transformations ... ok
test runtime::tests::definitions_and_rules::self_referential_ownvalue_returns_symbol_without_message ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_except_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules::valueq_uses_rollback_trial_evaluation_without_side_effects ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_optional_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_patterntest_patterns ... ok
test runtime::tests::association_write_recovery::association_incremental_writes_stay_direct_past_the_cache_size_cap ... ok
test runtime::tests::definitions_and_rules_cases::cases_rule_rhs_splices_named_blanksequence_matches ... ok
test runtime::tests::definitions_and_rules_part2::lowers_slots_and_pure_functions ... ok
test runtime::tests::definitions_and_rules_part2::lowers_named_pattern_assignment ... ok
test runtime::tests::definitions_and_rules_cases::cases_preserves_immediate_and_delayed_rule_behavior ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_deletes_top_level_matches_by_default ... ok
test runtime::tests::definitions_and_rules::dynamic_iterator_lhs_values_are_normalized_with_the_existing_cycle_guard ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_can_substitute_multiple_named_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_standard_integer_level_specs ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_matches_after_argument_evaluation ... ok
test runtime::tests::definitions_and_rules_cases::cases_traverses_depth_first_with_leaves_before_roots_and_respects_match_limits ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_splices_named_blanksequence_into_rhs_calls ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_splices_named_blanknullsequence_into_empty_lists ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_supports_standard_levelspecs_and_limits ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_can_delete_heads_and_association_values ... ok
test runtime::tests::definitions_and_rules::real_iterator_definitions_precede_dynamic_values_and_clear_reveals_them_again ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_exact_range_negative_and_infinity_level_specs ... ok
test runtime::tests::definition_store_sealing::prepared_store_and_removal_call_site_inventory_is_closed ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_explicit_head_traversal_control ... ok
test runtime::tests::definitions_and_rules_part2::matchq_operator_form_is_inert_and_applies_to_data ... ok
test runtime::tests::definitions_and_rules_part2::compiled_matcher_matches_interpreted_matcher_on_supported_subset ... ok
test runtime::tests::definitions_and_rules_part2::matchq_reports_false_for_non_matching_pattern ... ok
test runtime::tests::definitions_and_rules_part2::inspect_definition_catalog_reports_kinds_and_delayedness ... ok
test runtime::tests::definitions_and_rules_part2::matchq_reports_true_for_matching_pattern ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_blanknullsequence_in_argument_lists ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_part2::interpreted_matcher_owned_can_resume_sequence_conditions_across_input ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_except_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_blanksequence_in_argument_lists ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_conditions_on_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_optional_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_patterntest_on_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_plain_blank_arguments_under_fixed_heads ... ok
test runtime::tests::definitions_and_rules_part2::named_slots_fill_from_rules_and_association_arguments ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_part2::setdelayed_stores_and_applies_simple_downvalue ... ok
test runtime::tests::definitions_and_rules_part2::unmatched_downvalue_leaves_call_in_place ... ok
test runtime::tests::definitions_and_rules_part2::set_supports_immediate_pattern_based_downvalues_and_subvalues ... ok
test runtime::tests::definitions_and_rules::iterator_lhs_normalization_covers_nested_and_shadowed_iterators ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_traverses_nested_expressions ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rule_uses_current_rhs_value_at_application_time ... ok
test runtime::tests::definitions_and_rules_part3::immediate_rule_captures_current_rhs_value ... ok
test runtime::tests::definitions_and_rules_part3::replace_defaults_to_the_root_level ... ok
test runtime::tests::definitions_and_rules_part3::replace_heads_option_controls_head_traversal ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_applies_single_rule ... ok
test runtime::tests::definitions_and_rules_part3::replace_rewrites_association_values_without_touching_keys ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replace_supports_levelspecs_and_rule_lists ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_applies_rule_list_in_order ... ok
test runtime::tests::context_and_loader::script_forms::parse_script_forms_matches_the_line_driven_splitter_over_the_corpus ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_splices_named_blanksequence_into_rhs_calls ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_except_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_shape_prefilter_stays_conservative_for_head_attributes ... ok
test runtime::tests::association_part_cursor::wide_association_single_key_and_position_reads_return_exact_values ... ok
test runtime::tests::definitions_and_rules_part3::flat_and_orderless_flat_heads_regroup_for_match_replace_and_replacelist ... ok
test runtime::tests::association_write_recovery::association_incremental_writes_preserve_ordinary_symbol_semantics_past_the_cache_size_cap ... ok
test runtime::tests::definitions_and_rules_part3::replace_and_replaceat_exact_rule_index_preserve_first_rule_priority ... ok
test runtime::tests::definitions_and_rules_part3::replacelist_collects_multiple_backtracking_matches_for_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::alternatives_downvalue_matches_each_branch ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_preserves_same_name_local_binding_behavior ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_limit_returns_terminated_value_for_non_terminating_rewrite_cycle ... ok
test runtime::tests::definitions_and_rules_part3::rewrite_options_report_new_defaults ... ok
test runtime::tests::definitions_and_rules_part4::function_accepts_explicit_attributes_for_application ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_binds_multiple_named_args ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_reaches_a_fixed_point ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_handles_named_typed_blank_hit ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_delayed_rule_limit_returns_terminated_value ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_falls_back_for_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replacelist_supports_operator_form_rule_order_and_max_results ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_keeps_typed_blank_miss_unevaluated ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_falls_back_for_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_preserves_exact_vs_pattern_precedence ... ok
test runtime::tests::definitions_and_rules_patterns::conditioned_downvalue_matches_only_when_test_passes ... ok
test runtime::tests::definitions_and_rules_part3::rewrite_family_operator_forms_are_inert_and_apply_to_data ... ok
test runtime::tests::definitions_and_rules_patterns::except_downvalue_prefilter_stays_conservative ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_max_iterations_option_controls_effective_limit ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_keeps_inserted_compound_values_outside_new_locals ... ok
test runtime::tests::definitions_and_rules_patterns::except_optional_and_repeated_downvalues_match_expected_shapes ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_exact_rule_index_preserves_first_rule_priority ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_except_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_rejects_malformed_repetition_bounds_with_head_specific_messages ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::outer_conditioned_downvalue_is_stored_and_applied ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_optional_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::patternsequence_flattens_argument_patterns_in_matching_rules_and_definitions ... ok
test runtime::tests::definitions_and_rules_patterns::patterntest_downvalue_matches_only_when_test_passes ... ok
test runtime::tests::definitions_and_rules_patterns::orderlesspatternsequence_uses_orderless_group_matching_inside_orderless_heads ... ok
test runtime::tests::definitions_and_rules_patterns::orderless_heads_backtrack_across_exact_and_patternsequence_groups ... ok
test runtime::tests::definitions_and_rules_patterns::verbatim_matches_literal_pattern_language_forms ... ok
test runtime::tests::definitions_and_rules_patterns::orderless_sequence_matching_still_accepts_small_normal_cases ... ok
test runtime::tests::definitions_and_rules_patterns::longest_and_shortest_control_sequence_pattern_greediness ... ok
test runtime::tests::definitions_and_rules_patterns::rule_lhs_evaluates_values_without_evaluating_pattern_names ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::inspect_rewrite_rules_reports_normalized_rules_and_delayed_flags ... ok
test runtime::tests::definitions_and_rules_tags::downvalues_lists_stored_pattern_definitions_in_order ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::normalize_and_inspect_rewrite_rules_rebuild_rhs_guards ... ok
test runtime::tests::definitions_and_rules_patterns::wide_downvalue_store_index_preserves_specificity_and_alpha_replacement ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::normalize_rules_lists_single_rules_rule_lists_and_preserves_delayedness ... ok
test runtime::tests::definitions_and_rules_patterns::wide_ground_downvalue_scan_falls_back_to_live_order_after_matcher_mutation ... ok
test runtime::tests::definitions_and_rules_patterns::wide_ground_downvalue_index_preserves_pattern_priority_fallback_and_invalidation ... ok
test runtime::tests::definitions_and_rules_tags::explicit_tagunset_removes_tagged_subvalue_definitions ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_upvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_downvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::subvalues_list_and_apply_nested_head_definitions ... ok
test runtime::tests::definitions_and_rules_tags::literal_downvalue_is_prioritized_over_existing_typed_downvalue ... ok
test runtime::tests::definitions_and_rules_tags::clear_name_patterns_expand_before_mutation_and_preserve_exact_targets ... ok
test runtime::tests::definitions_and_rules_tags::tag_definition_messages_distinguish_missing_tags_from_invalid_positions ... ok
test runtime::tests::definitions_and_rules_tags::tagged_subvalues_attach_to_the_head_symbol_not_to_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_subvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::clearall_and_remove_expand_name_patterns ... ok
test runtime::tests::definitions_and_rules_tags::tagset_preserves_tag_ownership_for_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::tagset_immediate_captures_rhs_before_later_changes ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::inspect_rewrite_rules_can_suspend_and_resume_with_host_input ... ok
test runtime::tests::definitions_and_rules_tags::string_targets_flow_through_clear_tag_and_unset_normalization ... ok
test runtime::tests::definitions_and_rules_tags::upset_emits_nosym_when_no_valid_argument_tag_symbol_exists ... ok
test runtime::tests::definitions_and_rules_tags::typed_subvalues_are_prioritized_over_earlier_generic_subvalues ... ok
test runtime::tests::definitions_and_rules_tags::typed_upvalues_are_prioritized_over_earlier_generic_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::upset_stores_immediate_upvalues_on_inferred_argument_heads ... ok
test runtime::tests::definitions_and_rules_tags::unset_can_remove_symbol_downvalue_and_upvalue_definitions ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_does_not_capture_inserted_values_in_lexical_scopes ... ok
test runtime::tests::definitions_and_rules_tags::upvalue_candidate_filter_preserves_matching_order_for_valid_tags ... ok
test runtime::tests::definitions_and_rules_tags::upsetdelayed_can_store_on_multiple_inferred_tag_symbols ... ok
test runtime::tests::definitions_and_rules_tags::upvalue_dispatch_ignores_tags_that_appear_only_below_direct_arguments ... ok
test runtime::tests::definitions_and_rules_tags::upvalues_lists_and_applies_tagged_definitions ... ok
test runtime::tests::definitions_rhs_guards::delayed_downvalue_rhs_guard_accepts_or_declines_without_messages ... ok
test runtime::tests::definitions_rhs_guards::guard_errors_propagate_like_pattern_position_condition_errors ... ok
test runtime::tests::definitions_rhs_guards::downvalues_guard_reconstruction_round_trips_behavior ... ok
test runtime::tests::definitions_rhs_guards::every_non_true_guard_result_declines_to_the_next_candidate ... ok
test runtime::tests::definitions_rhs_guards::guarded_and_unguarded_downvalues_coexist_in_installation_order ... ok
test runtime::tests::definitions_rhs_guards::explicit_true_guard_is_distinct_from_an_unguarded_downvalue ... ok
test runtime::tests::definitions_rhs_guards::rhs_guard_side_effects_run_before_the_body_is_selected ... ok
test runtime::tests::definitions_and_rules::non_iterator_lhs_binding_controls_keep_their_exact_results ... ok
test runtime::tests::definitions_rhs_guards::immediate_downvalue_rhs_evaluates_completely_before_guard_splitting ... ok
test runtime::tests::definitions_rhs_guards::rhs_guards_and_bodies_share_the_definition_option_scope ... ok
test runtime::tests::definitions_rhs_guards::unset_lhs_removes_every_rhs_guard_variant ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::listable_guarded_downvalue_recompiles_and_enforces_its_guard ... ok
test runtime::tests::definitions_rhs_guards::wide_exact_dispatch_preserves_guard_variant_order ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::matcher_ownership_guard_is_enforced_after_attribute_revision_invalidation ... ok
test runtime::tests::definitions_rhs_guards::same_alpha_canonical_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::interleaved_guarded_downvalue_definitions_survive_later_bumps ... ok
test runtime::tests::definitions_rhs_guards::nested_rhs_guards_flatten_in_order_for_both_spellings ... ok
test runtime::tests::definitions_and_rules_tags::exact_downvalue_dispatch_preserves_ground_and_pattern_semantics ... ok
test runtime::tests::definitions_rhs_guards_default_values::declining_default_guard_leaves_optional_matching_without_a_default ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::eval_file_rollback_preserves_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::repeated_attribute_bumps_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_default_values::definition_inspection_rebuilds_all_remaining_definition_guard_forms ... ok
test runtime::tests::definitions_rhs_guards_default_values::every_non_true_defaultvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_default_values::defaultvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::attributes_on_the_guarded_symbol_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_own_values::a_later_guarded_ownvalue_replaces_the_single_existing_slot ... ok
test runtime::tests::definitions_rhs_guards_default_values::immediate_and_delayed_defaultvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_own_values::ownvalue_guards_run_left_to_right_and_decline_before_the_body ... ok
test runtime::tests::definitions_rhs_guards_default_values::nested_defaultvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_own_values::ownvalue_rhs_guard_resumes_without_replaying_the_guard ... ok
test runtime::tests::definitions_rhs_guards_default_values::same_defaultvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_own_values::immediate_and_delayed_ownvalue_rhs_guards_accept_or_decline ... ok
test runtime::tests::definitions_rhs_guards_own_values::declining_ownvalue_guards_fall_through_inside_block_and_module ... ok
test runtime::tests::definitions_rhs_guards_own_values::non_true_ownvalue_guards_decline_and_valueq_reports_false ... ok
test runtime::tests::definitions_rhs_guards_default_values::unset_clears_every_defaultvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::valueq_rollback_preserves_every_guarded_lane_and_post_trigger_value ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::unrelated_set_and_clear_attributes_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::declining_guarded_rules_handoff_in_rule_list_order ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::guarded_exact_rule_preserves_wide_plan_order_and_fallback_index ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::every_non_true_rewrite_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::nested_rewrite_guards_run_left_to_right_and_second_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::replace_repeated_stops_when_the_only_guard_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::immediate_and_delayed_rewrite_rules_split_bound_guards ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rule_shaped_data_keeps_its_existing_condition_forms ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rewrite_guard_side_effect_runs_once_per_candidate_tested ... ok
test runtime::tests::definitions_rhs_guards_sub_values::nested_subvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::replace_list_guard_decline_keeps_later_binding_enumeration ... ok
test runtime::tests::definitions_rhs_guards_sub_values::same_alpha_canonical_subvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_sub_values::subvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_rhs_guards_sub_values::immediate_and_delayed_subvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::suspending_rewrite_guard_resumes_same_rule_and_position_without_replay ... ok
test runtime::tests::definitions_rhs_guards_sub_values::every_non_true_subvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_sub_values::unset_clears_every_subvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_rhs_guards_sub_values::subvalue_rhs_guards_share_the_guarded_body_option_scope ... ok
test runtime::tests::definitions_and_rules_patterns::wide_pattern_downvalue_store_index_replaces_renamed_condition_pattern ... ok
test runtime::tests::definitions_rhs_guards_up_values::every_non_true_upvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rewrite_rhs_guards_accept_or_decline_for_every_generic_consumer ... ok
test runtime::tests::definitions_rhs_guards_up_values::nested_upvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_up_values::immediate_and_delayed_upvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_up_values::same_alpha_canonical_upvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_up_values::upvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_rhs_guards_up_values::tagunset_clears_every_upvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_rhs_guards_up_values::upvalue_rhs_guards_share_the_guarded_body_option_scope ... ok
test runtime::tests::definitions_tagged_preparation::upset_infers_tags_only_after_preparation_and_keeps_nosym ... ok
test runtime::tests::definitions_unset_preparation::unset_after_attribute_invalidation_addresses_only_the_new_identity ... ok
test runtime::tests::definitions_tagged_preparation::upset_multiple_owner_protection_is_preserved_after_canonicalization ... ok
test runtime::tests::definitions_unset_preparation::clear_removes_all_identities_and_reinstall_has_no_stale_unset_target ... ok
test runtime::tests::definitions_unset_preparation::unset_and_postfix_unset_prepare_active_definition_identities ... ok
test runtime::tests::definitions_value_table_assignment::cross_symbol_downvalues_copy_fails_closed_on_head_mismatch ... ok
test runtime::tests::definitions_value_table_assignment::cross_symbol_downvalues_copy_works_after_target_substitution ... ok
test runtime::tests::definitions_unset_preparation::unset_after_own_value_rebinding_and_clear_preserves_the_stored_snapshot ... ok
test runtime::tests::definitions_tagged_preparation::all_tagged_install_entry_points_store_the_prepared_identity_by_owner ... ok
test runtime::tests::definitions_unset_preparation::unset_prepares_held_and_pattern_lhs_by_construction ... ok
test runtime::tests::definitions_value_table_assignment::downvalues_identity_round_trip_restores_definition_behavior ... ok
test runtime::tests::definitions_value_table_assignment::later_mismatched_rule_cannot_partially_replace_a_value_table ... ok
test runtime::tests::definitions_unset_preparation::unset_snapshot_identity_removes_without_rebinding_but_not_after_rebinding ... ok
test runtime::tests::definitions_value_table_assignment::downvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::frame_root_trace::root_visitation::association_entry_form_visits_key_and_value_roots_in_field_order ... ok
test runtime::tests::frame_root_trace::root_visitation::block_symbol_state_map_visits_the_bare_symbol_key ... ok
test runtime::tests::definitions_value_table_assignment::ownvalues_identity_round_trip_restores_the_owned_value ... ok
test runtime::tests::frame_root_trace::root_visitation::container_implementations_visit_keys_values_and_elements ... ok
test runtime::tests::frame_root_trace::root_visitation::expr_is_one_whole_root_and_bare_symbol_is_an_independent_root ... ok
test runtime::tests::frame_root_trace::root_visitation::file_io_error_is_the_zero_root_negative_calibration_target ... ok
test runtime::tests::definitions_value_table_assignment::malformed_value_table_rhs_leaves_the_existing_table_unchanged ... ok
test runtime::tests::frame_root_trace::root_visitation::enum_shapes_and_generic_arrays_trace_without_special_cases ... ok
test runtime::tests::frame_root_trace::root_visitation::matcher_binding_table_visits_the_exact_symbol_and_expression_roots ... ok
test runtime::tests::frame_root_trace::root_visitation::nested_derived_fields_are_all_traversed ... ok
test runtime::tests::frame_root_trace::root_visitation::recursive_matcher_arg_visits_every_nested_root_exactly_once ... ok
test runtime::tests::frame_root_trace::root_visitation::shared_expression_types_borrow_their_expression_roots ... ok
test runtime::tests::definitions_tagged_preparation::tagged_preparation_preserves_patterns_all_holds_and_callable_heads ... ok
test runtime::tests::definitions_tagged_preparation::tagged_snapshot_identity_tracks_attribute_rebind_clear_and_owner_invalidation ... ok
test runtime::tests::definitions_value_table_assignment::protection_is_rechecked_after_value_table_lhs_preparation ... ok
test runtime::tests::definitions_value_table_assignment::prepared_value_table_identity_is_a_snapshot_across_rebind_clear_and_reinstall ... ok
test runtime::tests::definitions_value_table_assignment::protected_value_table_target_rejects_replacement ... ok
test runtime::tests::definitions_value_table_assignment::successful_value_table_assignment_replaces_instead_of_merging ... ok
test runtime::tests::file_names::file_names_depth_controls_nested_provider_enumeration ... ok
test runtime::tests::file_names::file_names_malformed_specs_remain_inert ... ok
test runtime::tests::definitions_value_table_assignment::subvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::file_names::file_names_guard_precedes_the_provider_check ... ok
test runtime::tests::definitions_value_table_assignment::value_table_assignment_accepts_immediate_delayed_and_unwrapped_rules ... ok
test runtime::tests::file_names::file_names_matches_immediate_entries_and_sorts_and_deduplicates_paths ... ok
test runtime::tests::definitions_value_table_assignment::upvalues_and_subvalues_identity_round_trips_restore_behavior ... ok
test runtime::tests::definitions_value_table_assignment::upvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::holding_and_control_io::pause_rejects_unrepresentable_integer_and_rational_durations ... ok
test runtime::tests::definitions_and_rules_tags::wide_subvalue_store_index_preserves_specificity_replacement_and_invalidation ... ok
test runtime::tests::holding_and_control_part2::binary_function_syntax_applies_explicit_parameter ... ok
test runtime::tests::holding_and_control_part2::dispatch_plan_uses_builtin_metadata_and_head_traits ... ok
test runtime::tests::holding_and_control_part2::cycle_detection_returns_iteration_limit_for_mutual_ownvalues_and_recovers ... ok
test runtime::tests::holding_and_control_part2::cycle_detection_returns_iteration_limit_for_mutual_downvalues ... ok
test runtime::tests::holding_and_control_part2::fixed_point_evaluation_revisits_expressions_after_side_effects_change_head_meaning ... ok
test runtime::tests::holding_and_control_part2::evaluation_pass_limit_bounds_fixed_point_revisits ... ok
test runtime::tests::holding_and_control_io::input_print_and_pause_cover_current_first_pass_io_subset ... ok
test runtime::tests::holding_and_control_part2::function_applies_slots_positionally ... ok
test runtime::tests::definitions_and_rules_tags::wide_upvalue_store_index_preserves_specificity_replacement_and_invalidation ... ok
test runtime::tests::holding_and_control_part2::explicit_parameter_functions_respect_shadowing_and_pattern_locals ... ok
test runtime::tests::holding_and_control_part2::native_stack_limit_stops_unbounded_definition_recursion_and_session_stays_usable ... ok
test runtime::tests::holding_and_control_part2::explicit_parameter_functions_ignore_extra_arguments ... ok
test runtime::tests::holding_and_control_part2::pure_function_syntax_uses_slots_and_evaluated_arguments ... ok
test runtime::tests::holding_and_control_part2::head_evaluation_can_activate_special_forms_and_held_builtins ... ok
test runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds ... ok
test runtime::tests::comparison_lowering::comparison_lowering_keeps_parenthesized_subtrees_independent ... ok
test runtime::tests::holding_and_control_part2::recursion_limit_stops_expanding_recursive_downvalues ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_attribute_round_trips ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_applies_at_every_application_level ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_flattens_sequence_like_hold_all ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_holds_outer_arguments_and_evaluate_pierces ... ok
test runtime::tests::holding_and_control_part2::tracing_records_definition_rule_and_evaluation_events ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_strips_and_restores_unevaluated ... ok
test runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions ... ok
test runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions ... ok
test runtime::tests::definitions_rhs_guards_sub_values::wide_subvalue_identity_index_keeps_guard_variants_distinct ... ok
test runtime::tests::iterative_loop_drivers::for_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions ... ok
test runtime::tests::iterative_loop_drivers::unbounded_while_reaches_the_existing_eval_deadline ... ok
test runtime::tests::iterative_loop_drivers::module_resumes_mid_initializer_without_replaying_bindings ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_preserves_message_name_double_resolution_order ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_preserves_cross_family_shape_and_resolver_order ... ok
test runtime::tests::holding_and_control_part2::sum_and_product_valid_iterators_remain_message_free ... ok
test runtime::tests::left_chain_lowering::left_chain_two_mib_full_round_trip_accepts_and_rejects_without_abort ... ignored, release-only deployment-stack probe; run explicitly with --release
test runtime::tests::left_chain_lowering::left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary ... ignored, release-only deployment-stack probe; run explicitly with --release
test runtime::tests::left_chain_lowering::left_chain_lowering_keeps_direct_canonical_calls_opaque ... ok
test runtime::tests::left_chain_lowering::flat_chain_evaluated_results_remain_unchanged ... ok
test runtime::tests::iterative_loop_drivers::until_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::iterative_loop_drivers::while_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::definitions_rhs_guards_up_values::wide_upvalue_identity_index_keeps_guard_variants_distinct ... ok
test runtime::tests::lowering_and_context::long_statement_chain_lowers_to_flat_nary_compound_expression ... ok
test runtime::tests::lowering_and_context::lowering_rejects_invalid_integer_literals_before_runtime ... ok
test runtime::tests::lowering_and_context::lowers_big_integer_into_core_expr ... ok
test runtime::tests::lowering_and_context::lowers_message_name_syntax_into_core_expr ... ok
test runtime::tests::comparison_lowering::comparison_lowering_bounds_direct_chain_nodes ... ok
test runtime::tests::lowering_and_context::lowers_multiline_string_literals_like_ordinary_strings ... ok
test runtime::tests::lowering_and_context::lowers_simple_decimal_literals_to_exact_rationals ... ok
test runtime::tests::lowering_and_context::lowers_tagset_and_unset_syntax_into_core_expr ... ok
test runtime::tests::lowering_and_context::over_cap_statement_chain_line_does_not_swallow_later_lines ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_rejects_a_20_000_link_hand_built_ast_gracefully ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_messages_and_context_heads ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_tagged_definition_sugar ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_flat_arithmetic_family_transitions ... ok
test runtime::tests::left_chain_lowering::held_sameq_explicitly_observes_variadic_non_flat_chain_shapes ... ok
test runtime::tests::left_chain_lowering::held_sameq_explicitly_observes_flat_chain_shapes ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::remaining_attribute_revision_sources_preserve_every_guarded_lane ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_three_link_left_chains ... ok
test runtime::tests::lowering_and_context::attributes_reports_builtin_and_user_defined_attribute_sets ... ok
test runtime::tests::lowering_and_context::compound_expression_sugar_flows_through_lowering_and_evaluation ... ok
test runtime::tests::lowering_and_context::evaluates_identity_call ... ok
test runtime::tests::lowering_and_context::lowering_preserves_tagset_shape_for_runtime_semantic_validation ... ok
test runtime::tests::memory_growth_contract::core_builtin_memory_growth_inventory_is_sorted_and_exhaustive ... ok
test runtime::tests::memory_growth_contract::known_unprojected_preflight_builtin_allowlist_is_sorted_and_unique ... ok
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_unsuspended_values ... ok
test runtime::tests::definitions_and_rules::iterator_bindings_normalize_set_lhs_for_every_eager_iterator_and_assignment ... ok
test runtime::tests::left_chain_lowering::variadic_non_flat_chain_matching_shapes_are_explicit ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_argument_finisher_collapses_equal_changed_prefix ... ok
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_frozen_prefix_and_unique_tail_values ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_rules_patterns_slots_and_scoping ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_changed_arg_boundary_keeps_unchanged_child_arcs ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_nested_flat_is_bottom_up_and_fully_flattened ... ok
test runtime::tests::comparison_lowering::comparison_lowering_accepts_a_uniform_chain_at_the_boundary ... ok
test runtime::tests::holding_and_control_part2::sum_and_product_malformed_iterators_emit_argform_messages ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_uses_normalized_function_head_attributes ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_bounded_front_handles_exhaustion_locally ... ok
test runtime::tests::memory_growth_contract::preflight_contract_accepts_recorded_projection ... ok
test runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_after_input_resume - should panic ... ok
test runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_through_dispatch - should panic ... ok
test runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_after_input_resume - should panic ... ok
test runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_through_dispatch - should panic ... ok
test runtime::tests::memory_growth_contract::preflight_contract_remembers_projection_across_input_resume ... ok
test runtime::tests::frame_root_trace::manual_frame_root_trace_implementations_match_sorted_allowlist ... ok
test runtime::tests::messages::inspect_reports_message_definition_presence ... ok
test runtime::tests::messages::builtin_message_registry_populates_listings_and_information ... ok
test runtime::tests::messages::inspect_message_catalog_exposes_templates_and_delayed_flags ... ok
test runtime::tests::messages::delayed_message_templates_are_evaluated_when_emitted ... ok
test runtime::tests::messages::direct_set_and_setdelayed_calls_share_message_assignment_normalization ... ok
test runtime::tests::messages::inspect_message_catalog_merges_builtin_templates_and_user_overrides ... ok
test runtime::tests::messages::invalid_builtin_argument_shapes_emit_runtime_messages ... ok
test runtime::tests::messages::invalid_message_name_form_emits_runtime_message ... ok
test runtime::tests::messages::inert_message_carriers_list_seeded_tags_and_reject_assignment ... ok
test runtime::tests::frame_root_trace::derived_frame_root_trace_inventory_matches_the_closed_capture_graph ... ok
test runtime::tests::messages::message_list_reports_messages_emitted_so_far_in_current_input ... ok
test runtime::tests::messages::message_name_and_message_emit_runtime_messages ... ok
test runtime::tests::messages::message_definitions_builtin_lists_immediate_and_delayed_templates ... ok
test runtime::tests::messages::message_family_accepts_symbol_name_strings_and_messagename_targets ... ok
test runtime::tests::messages::message_templates_can_be_defined_and_formatted ... ok
test runtime::tests::messages::push_message_allows_open_user_symbol_families ... ok
test runtime::tests::messages::push_message_rejects_unregistered_builtin_message_tags - should panic ... ok
test runtime::tests::messages::push_message_allows_test_local_builtins_without_declared_messages ... ok
test runtime::tests::messages::message_templates_leave_unsupported_format_specs_unchanged ... ok
test runtime::tests::messages::message_templates_leave_unknown_placeholders_unchanged ... ok
test runtime::tests::messages::message_templates_support_richer_format_specifiers ... ok
test runtime::tests::messages::message_templates_support_message_name_and_all_args_placeholders ... ok
test runtime::tests::messages::quiet_keeps_suppressed_messages_out_of_message_list ... ok
test runtime::tests::messages::request_message_list_reports_messages_emitted_so_far_in_current_input ... ok
test runtime::tests::messages::push_message_allows_user_defined_builtin_message_names ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference ... ok
test runtime::tests::messages::upto_stays_inert_and_selector_diagnostic_stays_exact ... ok
test runtime::tests::messages_part2::check_and_quiet_accept_message_name_selector_forms ... ok
test runtime::tests::messages_part2::check_can_catch_non_loader_evaluation_errors_via_general_messages ... ok
test runtime::tests::messages::usage_message_assignment_holds_symbol_with_existing_own_value ... ok
test runtime::tests::messages::message_tags_builtin_lists_global_symbol_and_exact_message_tags ... ok
test runtime::tests::messages_part2::check_can_match_by_message_symbol_only ... ok
test runtime::tests::messages_part2::cycle_termination_emits_pass_limit_message_at_top_level ... ok
test runtime::tests::messages_part2::check_can_match_specific_general_error_selectors ... ok
test runtime::tests::messages_part2::check_returns_fallback_when_messages_are_emitted ... ok
test runtime::tests::messages_part2::check_leaves_value_unchanged_when_no_messages_are_emitted ... ok
test runtime::tests::messages_part2::explicit_parameter_function_arity_mismatch_emits_runtime_message ... ok
test runtime::tests::messages_part2::check_can_target_specific_message_selectors ... ok
test runtime::tests::messages_part2::invalid_message_selector_specs_emit_runtime_messages ... ok
test runtime::tests::messages_part2::invalid_builtin_argument_counts_emit_runtime_messages ... ok
test runtime::tests::messages_part2::quiet_accepts_selector_lists ... ok
test runtime::tests::messages_part2::quiet_prevents_check_from_observing_suppressed_messages ... ok
test runtime::tests::messages_part2::quiet_can_selectively_suppress_message_streams ... ok
test runtime::tests::hygiene::kernel_crates_do_not_use_std_fs_outside_tests ... ok
test runtime::tests::messages_part2::quiet_suppresses_cycle_termination_messages_without_masking_value ... ok
test runtime::tests::messages_part3::check_can_catch_iteration_limit_messages_from_replace_repeated ... ok
test runtime::tests::messages_part2::unbalanced_context_frames_emit_runtime_messages ... ok
test runtime::tests::messages_part3::beginpackage_can_load_dependencies_via_needs ... ok
test runtime::tests::messages_part3::beginpackage_accepts_resolved_dependency_lists ... ok
test runtime::tests::messages_part2::quiet_suppresses_runtime_messages_from_nested_evaluation ... ok
test runtime::tests::messages_part3::hold_family_respects_builtin_hold_attributes ... ok
test runtime::tests::messages_part3::explicit_non_system_heads_do_not_trigger_builtin_dispatch ... ok
test runtime::tests::messages_part3::path_builtin_reflects_package_search_roots ... ok
test runtime::tests::mutation_operator_forms::append_to_association_key_part_stays_on_generic_path ... ok
test runtime::tests::mutation_operator_forms::append_to_capture_defers_its_value_until_application ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_copies_list_shared_at_two_positions ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_non_list_leaf_still_runs_definitions ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_with_monitor_preserves_spine_observations ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_with_tracing_preserves_existing_trace ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_preserves_inner_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_preserves_outer_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_deep_exact_part_preserves_middle_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_rolls_back_multilevel_spine ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_protected_owner_still_refuses ... ok
test runtime::tests::mutation_operator_forms::append_to_modified_list_attributes_stays_on_evaluated_path ... ok
test runtime::tests::mutation_operator_forms::append_to_list_downvalue_still_observes_the_old_value_and_fires ... ok
test runtime::tests::mutation_operator_forms::append_to_item_that_requires_target_evaluation_stays_on_evaluated_path ... ok
test runtime::tests::mutation_operator_forms::append_to_list_upvalue_still_observes_the_old_value_and_fires ... ok
test runtime::tests::mutation_operator_forms::append_to_non_list_head_still_runs_matching_user_code ... ok
test runtime::tests::mutation_operator_forms::append_to_and_add_to_direct_calls_match_their_operator_forms ... ok
test runtime::tests::mutation_operator_forms::append_to_nothing_keeps_list_normalization_semantics ... ok
test runtime::tests::mutation_operator_forms::append_to_with_monitor_preserves_old_and_new_display_observations ... ok
test runtime::tests::mutation_operator_forms::append_to_rolls_back_if_fast_update_fails_after_definition_take ... ok
test runtime::tests::mutation_operator_forms::append_to_with_tracing_preserves_the_existing_trace ... ok
test runtime::tests::mutation_operator_forms::append_to_resumes_item_input_without_losing_or_replaying_the_update ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_keeps_nothing_and_upvalue_behavior ... ok
test runtime::tests::mutation_operator_forms::append_to_plain_list_preserves_alias_and_returned_value ... ok
test runtime::tests::mutation_operator_forms::append_to_protected_list_target_preserves_value_and_message ... ok
test runtime::tests::mutation_operator_forms::kernel_mutation_heads_advertise_subvalues_hold_all ... ok
test runtime::tests::mutation_operator_forms::kernel_mutation_operator_forms_apply_to_raw_held_targets ... ok
test runtime::tests::mutation_operator_forms::mutation_captures_are_quiet_but_invalid_applications_report_argx ... ok
test runtime::tests::native_stack_guard::native_stack_guard_polls_at_the_stride_and_fails_sticky ... ok
test runtime::tests::native_stack_guard::logical_depth_never_refuses_when_the_byte_budget_is_available ... ok
test runtime::tests::namespace_names::symbol_name_and_name_q_core_anchors ... ok
test runtime::tests::nothing::nothing_removal_happens_after_sequence_splicing_for_lists_only ... ok
test runtime::tests::namespace_names::names_rejects_unsupported_resolve_context_alias_values ... ok
test runtime::tests::nothing::nothing_resolves_as_system_symbol ... ok
test runtime::tests::namespace_names::names_rejects_unsupported_spelling_correction_values ... ok
test runtime::tests::options_and_protection::block_localizes_options_alongside_other_symbol_state ... ok
test runtime::tests::namespace_names::names_without_explicit_context_only_sees_visible_contexts ... ok
test runtime::tests::options_and_protection::codec_and_schema_options_are_discoverable ... ok
test runtime::tests::mutation_operator_forms::mutation_operator_form_preserves_direct_protection_failure ... ok
test runtime::tests::namespace_names::names_accepts_current_options_subset ... ok
test runtime::tests::options_and_protection::delayed_option_rules_are_preserved_and_evaluated_on_lookup ... ok
test runtime::tests::options_and_protection::argumentsoptions_and_checkarguments_cover_current_first_pass_subset ... ok
test runtime::tests::options_and_protection::options_assignment_unset_and_clearall_interact_with_symbol_state ... ok
test runtime::tests::options_and_protection::options_assignment_and_queries_cover_current_first_pass_subset ... ok
test runtime::tests::options_and_protection::optionspattern_can_use_an_explicit_target_symbol ... ok
test runtime::tests::options_and_protection::optionspattern_and_one_arg_optionvalue_cover_current_first_pass_subset ... ok
test runtime::tests::options_and_protection::optionvalue_emits_current_optnf_and_nodef_messages ... ok
test runtime::tests::options_and_protection::optionvalue_accepts_lists_of_option_names ... ok
test runtime::tests::options_and_protection::setoptions_blocks_protected_system_head_and_unprotect_allows_update ... ok
test runtime::tests::options_and_protection::setoptions_accepts_target_lists_and_returns_per_target_results ... ok
test runtime::tests::options_and_protection::protect_blocks_direct_options_assignment_and_unset_until_unprotected ... ok
test runtime::tests::options_and_protection::protected_targets_reject_attribute_updates_and_preserve_attributes ... ok
test runtime::tests::options_and_protection::setoptions_and_optionvalue_match_known_system_options_by_name ... ok
test runtime::tests::package_tools::declared_package_stub_autoloads_when_symbol_is_used ... ok
test runtime::tests::options_and_protection::setoptions_updates_known_options_but_rejects_unknown_ones ... ok
test runtime::tests::messages::messages_builtin_lists_defined_message_names ... ok
test runtime::tests::options_and_protection::test_create_options_are_discoverable_and_match_consumed_defaults ... ok
test runtime::tests::options_and_protection::protected_attribute_removal_and_unprotect_remain_escape_hatches ... ok
test runtime::tests::package_tools::declare_package_declares_names_and_updates_context_path ... ok
test runtime::tests::options_and_protection::protect_and_unprotect_expand_name_patterns_without_literal_wildcards ... ok
test runtime::tests::namespace_names::names_supports_current_first_pass_string_patterns ... ok
test runtime::tests::mutation_operator_forms::append_to_complex_part_selectors_keep_generic_behavior ... ok
test runtime::tests::package_tools::dump_save_returns_qualified_symbol_names_without_reevaluating_values ... ok
test runtime::tests::package_tools::dump_save_rejects_unsupported_compression_levels_and_preserves_default_bytes ... ok
test runtime::tests::package_tools::dump_save_context_strings_skip_module_gensyms_but_keep_dollar_globals ... ok
test runtime::tests::package_tools::once_caches_per_session_and_per_location_key ... ok
test runtime::tests::package_tools::dump_save_overwrites_file_and_returns_saved_symbols ... ok
test runtime::tests::package_tools::save_rejects_unknown_options_and_once_reports_unknown_option_names ... ok
test runtime::tests::package_tools::once_rejects_unimplemented_expiration_policies_without_evaluating_body ... ok
test runtime::tests::package_tools::save_and_dumpsave_honor_current_context_filter_and_attribute_options_subset ... ok
test runtime::tests::package_tools::once_accepts_option_rules_in_second_position_and_respects_zero_persistence_subset ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_root_alias ... ok
test runtime::tests::package_tools::definition_snapshot_rebuilds_all_rhs_guards_for_roundtrip ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_shared_child_at_a_second_position ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_keeps_spine_identity_and_leaves_proofs_evicted ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_the_selected_child_alias ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_polls_interrupt_before_cache_eviction_or_mutation ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_three_level_middle_alias ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_an_intermediate_list_alias ... ok
test runtime::tests::package_tools::save_and_dumpsave_accept_names_pattern_objects ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_rolls_back_slot_and_definition_after_replacement ... ok
test runtime::tests::package_tools::save_rebuilds_downvalue_rhs_guards_for_roundtrip ... ok
test runtime::tests::part_assignment::set_positional_part_refuses_a_target_cleared_by_a_selector ... ok
test runtime::tests::package_tools::save_writes_textual_state_that_get_can_roundtrip ... ok
test runtime::tests::part_assignment::set_positional_part_rechecks_protection_after_rhs_and_selector_callbacks ... ok
test runtime::tests::pattern_matching_orderless_optional::installed_system_down_values_surface_as_a_whole_after_user_mutation ... ok
test runtime::tests::part_assignment::set_positional_part_supports_flat_nested_and_value_semantics ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_optional_omission_collects_every_match_for_builtin_heads ... ok
test runtime::tests::part_assignment::set_positional_part_rederives_after_rhs_and_orders_selectors_left_to_right ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_orderless_optional_with_sequence_neighbors ... ok
test runtime::tests::part_assignment::set_positional_part_route_preserves_other_lvalues_and_definition_lhs_forms ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_dot_uses_head_default ... ok
test runtime::tests::pattern_matching_orderless_optional::one_identity_promotes_singletons_when_remaining_arguments_are_omissible ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_orderless_optional_matches_collects_and_omits ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_omission_uses_explicit_defaults ... ok
test runtime::tests::reap::nested_reap_scopes_keep_their_own_buckets ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_with_other_args_matches_present_and_omitted ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_present_matches_and_binds_in_documented_order ... ok
test runtime::tests::reap::reap_patterns_select_which_sows_are_captured ... ok
test runtime::tests::registered_builtin_resolution::registered_builtin_absent_from_static_catalog_resolves_as_system_symbol ... ok
test runtime::tests::reap::reap_buckets_stay_in_first_appearance_order ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_nonmatch_edges_stay_false ... ok
test runtime::tests::part_assignment::set_positional_part_selector_resume_is_single_pass_and_rederives_the_owner ... ok
test runtime::tests::reap::reap_index_preserves_wide_order_and_appends_to_existing_buckets ... ok
test runtime::tests::pattern_matching_orderless_optional::snapshot_after_unset_resurrects_an_installed_system_down_value ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_with_sequence_neighbors_uses_documented_backtracking_order ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_operator_form_applies_to_multiple_inputs ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_masks_only_held_argument_positions ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_rule_shape_messages_match_replace_all ... ok
test runtime::tests::rule_rhs_materialization::literal_sequence_wrapping_a_sequence_binding_keeps_its_literal_head ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_uses_original_and_compound_head_attributes ... ok
test runtime::tests::scoping_gensym_gc::gensym_gc_watermark_arithmetic_saturates ... ok
test runtime::tests::definition_store_sealing::definition_store_and_targeted_removal_are_sealed_behind_prepared_tokens ... ok
test runtime::tests::rule_rhs_materialization::held_delayed_rhs_firing_is_consumer_independent ... ok
test runtime::tests::rule_rhs_materialization::literal_sequence_splicing_is_deferred_until_replacement_result_evaluation ... ok
test runtime::tests::rule_rhs_materialization::replacepart_inserts_delayed_rhs_raw_before_result_evaluation ... ok
test runtime::tests::rule_rhs_materialization::replaceat_immediate_rhs_still_evaluates_at_rule_construction ... ok
test runtime::tests::rule_rhs_materialization::other_rewrite_traversals_share_replacement_sequence_splicing ... ok
test runtime::tests::scoping_gensym_gc::all_live_modules_run_one_sweep_before_the_next_watermark ... ok
test runtime::tests::rule_rhs_materialization::replacepart_immediate_rhs_still_evaluates_at_rule_construction ... ok
test runtime::tests::scoping_gensym_gc::immediate_module_reclaim_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::clear_all_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::partial_sweep_rearms_from_the_post_sweep_candidate_count ... ok
test runtime::tests::rule_rhs_materialization::replacement_sequence_splice_respects_parent_attributes ... ok
test runtime::tests::rule_rhs_materialization::replaceat_inserts_delayed_rhs_raw_before_result_evaluation ... ok
test runtime::tests::scoping_gensym_gc::loader_snapshot_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::rule_rhs_materialization::sequence_pattern_bindings_keep_structural_splicing ... ok
test runtime::tests::scoping_gensym_gc::gensym_gc_threshold_parses_env_in_fresh_processes ... ok
test runtime::tests::rule_rhs_materialization::top_level_sequence_replacement_has_no_parent_to_splice_into ... ok
test runtime::tests::rule_rhs_materialization::replacement_produced_sequence_splices_into_held_parent_rebuild ... ok
test runtime::tests::rule_rhs_materialization::unchanged_literal_sequence_does_not_splice_during_rebuild ... ok
test runtime::tests::part_assignment::set_positional_part_observable_constrained_and_dynamic_scopes_use_generic_path ... ok
test runtime::tests::scoping_gensym_gc::reachability::attribute_state_on_dead_candidate_is_cleared_during_reclaim ... ok
test runtime::tests::scoping_gensym_gc::reachability::delayed_own_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::downvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::empty_candidate_set_sweep_is_a_noop ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_error_path_definition_escape_stays_deferred ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_throw_payload_root_retains_its_gensym_definition ... ok
test runtime::tests::scoping_gensym_gc::reachability::live_candidate_definition_marks_transitive_candidate_dependencies ... ok
test runtime::tests::scoping_gensym_gc::reachability::down_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::global_root_keeps_escaped_candidate_until_root_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_result_root_retains_only_its_transitive_local_chain ... ok
test runtime::tests::scoping_gensym_gc::reachability::nested_module_activations_keep_their_candidates_until_each_activation_is_removed ... ok
test runtime::tests::scoping_gensym_gc::reachability::outcome_extra_roots_are_empty_for_non_value_errors_and_candidate_is_reclaimed ... ok
test runtime::tests::scoping_gensym_gc::reachability::message_root_escape_keeps_candidate_until_message_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::option_root_escape_keeps_candidate_until_options_are_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::own_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::remove_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::block_frame_bare_symbol_is_reclaimed_after_resume_and_next_sweep ... ok
test runtime::tests::scoping_gensym_gc::reachability::returned_expr_extra_root_keeps_candidate_definition_live ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::executing_frame_and_queued_suffix_remain_rooted_during_resume_sweep ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::path_a_second_request_keeps_frame_only_candidate_live ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::matcher_guard_binding_survives_a_sweep_while_suspended ... ok
test runtime::tests::scoping_gensym_gc::reachability::sub_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::path_b_resume_sweep_keeps_remaining_frame_candidate_live ... ok
test runtime::tests::scoping_gensym_gc::unreachable_candidates_are_reclaimed_at_the_next_watermark_boundary ... ok
test runtime::tests::scoping_gensym_gc::reachability::suspended_computation_keeps_candidate_live_until_resume_finishes ... ok
test runtime::tests::scoping_gensym_gc::reachability::syntax_information_root_escape_keeps_candidate_until_syntax_info_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::up_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::unique_generated_symbols_are_not_module_gc_candidates ... ok
test runtime::tests::scoping_gensym_gc::reachability::subvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::upvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::nested_module_error_reclamation_mirrors_success_conservatism ... ok
test runtime::tests::scoping_gensym_gc::reachability::transitive_definition_kind_chain_keeps_all_reachable_candidates_live ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::large_frame_state_trips_memory_limit_and_session_recovers ... ok
test runtime::tests::part_assignment::set_positional_part_invalid_targets_report_without_protected_part_fallback ... ok
test runtime::tests::scoping_unique::unique_generates_fresh_symbols_for_current_prefix_subset ... ok
test runtime::tests::scoping_unique::unique_reports_current_invalid_boundaries ... ok
test runtime::tests::scoping_unique::unique_skips_existing_generated_names ... ok
test runtime::tests::scoping_unique::unique_registers_generated_symbols_as_known_names ... ok
test runtime::tests::sealed_packages::context_validation_returns_the_call_inert ... ok
test runtime::tests::sealed_packages::decoy_assignment_is_distinct ... ok
test runtime::tests::sealed_packages::declaration_validation ... ok
test runtime::tests::sealed_packages::package_install_resumes_at_the_next_form ... ok
test runtime::tests::sealed_packages::internal_snapshot_round_trips_registry ... ok
test runtime::tests::sealed_packages::private_state_inside_package ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_unreferenced_error_paths_reclaim_gensyms_immediately ... ok
test runtime::tests::sealed_packages::package_scope_is_fqn_reachable ... ok
test runtime::tests::sealed_packages::registration_failure_cleans_sealed_state ... ok
test runtime::tests::sealed_packages::public_reaches_sealed_private ... ok
test runtime::tests::sealed_packages::re_registration_replaces_in_place ... ok
test runtime::tests::sealed_packages::remove_sealed_writes_no_generation ... ok
test runtime::tests::sealed_snapshot::marker_refused_in_ordinary_evaluation ... ok
test runtime::tests::sealed_packages::spelled_private_name_is_ordinary ... ok
test runtime::tests::sealed_snapshot::sealed_free_snapshot_bytes_unchanged ... ok
test runtime::tests::sealed_snapshot::information_sealed_unit_row ... ok
test runtime::tests::sealed_snapshot::save_refuses_sealed_references ... ok
test runtime::tests::sealed_visibility::completion_hides_sealed ... ok
test runtime::tests::sealed_visibility::completion_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::contexts_hides_private_context ... ok
test runtime::tests::sealed_visibility::contexts_reveals_with_diagnostic ... ok
test runtime::tests::sealed_snapshot::snapshot_refuses_orphans ... ok
test runtime::tests::session_config::default_native_stack_budget_is_derived_from_minimum_supported_stack ... ok
test runtime::tests::sealed_visibility::definition_hides_sealed_body ... ok
test runtime::tests::sealed_visibility::definition_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::information_hides_sealed ... ok
test runtime::tests::session_config::session_with_config_normalizes_contexts_paths_and_limits ... ok
test runtime::tests::scoping_gensym_gc::reachability::outcome_extra_roots_keep_success_return_throw_and_cleanup_candidates_live ... ok
test runtime::tests::session_memory::arithmetic::field_and_session_sums_preserve_overflow_information ... ok
test runtime::tests::session_memory::arithmetic::overflow_saturates_and_invalid_releases_stay_closed ... ok
test runtime::tests::session_memory::arithmetic::synthetic_charge_release_sequences_never_understate_representable_remainders ... ok
test runtime::tests::session_memory::byte_array_proxy_tracks_encoded_string_capacity_and_preserves_invalid_fallback ... ok
test runtime::tests::sealed_visibility::information_reveals_with_diagnostic ... ok
test runtime::tests::session_memory::oracle::byte_array_charges_encoded_capacity_for_valid_invalid_and_nonstring_payloads ... ok
test runtime::tests::session_memory::oracle::container_capacity_and_legacy_length_terms_are_distinct ... ok
test runtime::tests::session_memory::oracle::exception_registry_and_exception_expression_payloads_are_exact ... ok
test runtime::tests::session_memory::oracle::expressions_charge_each_shared_occurrence_and_spare_slot ... ok
test runtime::tests::session_memory::oracle::numeric_payloads_follow_signed_bytes_not_limb_capacity ... ok
test runtime::tests::sealed_visibility::decoy_visibility_is_callers_own ... ok
test runtime::tests::session_memory::session_memory_field_decision_inventory_is_sorted_and_exhaustive ... ok
test runtime::tests::session_memory::session_memory_walk_counts_declared_exception_storage ... ok
test runtime::tests::session_memory::stability_blooms_are_fixed_size_inline_values ... ok
test runtime::tests::session_memory::structures::call_cache_roots_are_additional_occurrences_of_retained_values ... ok
test runtime::tests::session_memory::structures::association_indexes_keep_stale_storage_until_the_owner_prunes_it ... ok
test runtime::tests::sealed_visibility::nameq_stays_false ... ok
test runtime::tests::session_memory::structures::live_frames_messages_and_traces_preserve_conservative_inline_terms ... ok
test runtime::tests::session_memory::structures::session_memory_layout_baseline ... ok
test runtime::tests::session_memory::structures::dispatch_metadata_and_lazy_index_have_exact_separate_charges ... ok
test runtime::tests::session_memory::structures::suspended_frames_charge_storage_repeated_roots_and_exception_payloads ... ok
test runtime::tests::session_memory::structures::tagged_dispatch_indexes_are_charged_for_both_value_tables ... ok
test runtime::tests::session_config::default_session_configuration_matches_default_session_state ... ok
test runtime::tests::session_config::apply_config_updates_host_settings_without_clearing_existing_definitions ... ok
test runtime::tests::session_config::session_setters_update_limits_capabilities_asserts_and_context_lookup ... ok
test runtime::tests::session_config::trace_message_and_print_accessors_expose_and_take_buffers ... ok
test runtime::tests::session_config::random_u64_sequences_are_pinned_for_byte_and_integer_seeds ... ok
test runtime::tests::sealed_visibility::nested_registration_keeps_registry_consistent ... ok
test runtime::tests::sealed_snapshot::snapshot_roundtrip_restores_seal ... ok
test runtime::tests::sealed_visibility::save_excludes_sealed ... ok
test runtime::tests::session_memory::oracle::empty_and_installed_sessions_match_without_sampling_the_peak ... ok
test runtime::tests::sealed_snapshot::snapshot_persists_public_mutation ... ok
test runtime::tests::sealed_visibility::names_hides_sealed ... ok
test runtime::tests::sealed_visibility::names_reveals_with_diagnostic ... ok
test runtime::tests::sealed_snapshot::snapshot_roundtrip_relinks_leaked_values ... ok
test runtime::tests::session_requests::api::accepted_cancel_before_request_entry_is_observed ... ok
test runtime::tests::sealed_visibility::wildcard_ops_skip_sealed ... ok
test runtime::tests::session_requests::api::accepted_cancel_abandons_input_before_conversion_or_continuation ... ok
test runtime::tests::session_requests::api::complete_request_returns_transport_friendly_matches ... ok
test runtime::tests::session_requests::api::completed_request_handle_is_inert_for_later_requests ... ok
test runtime::tests::session_requests::api::dropped_accepted_ticket_and_generation_rotation_retire_handles ... ok
test runtime::tests::hygiene::kernel_crates_do_not_write_stdout_outside_tests_and_bins ... ok
test runtime::tests::session_requests::api::eval_request_returns_transport_friendly_success_data ... ok
test runtime::tests::session_requests::api::eval_request_memory_ceiling_returns_classified_termination_and_session_recovers ... ok
test runtime::tests::session_requests::api::eval_request_nested_memory_constraint_catches_before_higher_request_ceiling ... ok
test runtime::tests::session_requests::api::eval_request_returns_transport_friendly_cycle_termination_data ... ok
test runtime::tests::session_requests::api::eval_expr_request_aggregates_messages_and_classifies_budget_termination ... ok
test runtime::tests::session_requests::api::eval_request_surfaces_structured_input_requests ... ok
test runtime::tests::session_requests::api::is_complete_request_distinguishes_complete_incomplete_and_invalid_input ... ok
test runtime::tests::session_requests::api::eval_request_take_pending_input_request_moves_and_clears_the_request ... ok
test runtime::tests::session_requests::api::inspect_request_aggregates_symbol_definitions_and_messages ... ok
test runtime::tests::session_requests::api::accepted_script_and_file_stop_before_later_forms_after_cancellation ... ok
test runtime::tests::session_requests::api::ordinary_request_and_lazy_observer_leave_interrupt_poll_unarmed ... ok
test runtime::tests::session_requests::api::parse_request_returns_structured_syntax_diagnostics ... ok
test runtime::tests::session_requests::api::stale_cancel_cannot_overwrite_current_request_cancel ... ok
test runtime::tests::session_requests::api::cancellation_observer_held_across_session_rebuild_is_inert ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_reap_handler_rendering ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_trace_capture ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_repeated_timing_samples ... ok
test runtime::tests::session_requests::input_resume::queued_input_responses_feed_inputstring_without_prompting_again ... ok
test runtime::tests::session_requests::input_resume::nested_input_suspension_precedes_the_old_suffix_without_reordering_effects ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_can_finish_delayed_message_templates ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_continues_replace_all_without_restarting_target ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_continues_symbol_evaluation_after_fixed_point_suspend ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_can_suspend_multiple_times_without_restarting ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_finishes_dialog_subsets ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_formats_delayed_message_templates_after_resume ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_finishes_with_bindings_without_reprompting ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_keeps_monitor_display_and_finishes_the_body ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_accepts_a_boundary_hand_built_alternating_ast ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_preserves_already_evaluated_prefix_arguments ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_reports_input_parse_errors_as_syntax_diagnostics ... ok
test runtime::tests::session_requests::api::file_request_maps_completed_syntax_io_and_missing_provider_errors ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_preserves_block_scope_until_completion ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_skips_uninitialized_module_bindings ... ok
test runtime::tests::session_requests::matcher_resume::defaultvalue_rhs_guard_resumes_during_matching_without_replaying_the_candidate ... ok
test runtime::tests::session_requests::matcher_resume::downvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fifth_arg ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fourth_arg ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_third_arg ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_level_spec ... ok
test runtime::tests::file_names::file_names_infinity_stops_at_the_fixed_entry_cap_without_recursing_the_stack ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_match_limit ... ok
test runtime::tests::session_requests::matcher_resume::cases_collection_survives_multiple_matcher_suspensions ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_matchq ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_symbol_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_cases ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_downvalue_matching ... ok
test runtime::tests::session_requests::matcher_resume::matcher_binding_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::guarded_replacement_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::position_collection_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::matcher_resume::ordered_single_arg_cursor_resumes_without_replaying_the_matched_prefix ... ok
test runtime::tests::session_requests::matcher_resume::nested_rule_application_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::position_collection_survives_multiple_matcher_suspensions ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_freeq_with_suspending_heads_option ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_nested_matcher_inside_sequence_conditioned_matchq ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_count_with_suspending_symbol_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_position_with_suspending_symbol_spec ... ok
test runtime::tests::sealed_visibility::message_catalog_hides_sealed ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_sequence_conditioned_matchq ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_position_with_suspending_match_limit ... ok
test runtime::tests::session_requests::matcher_resume::subvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_multi_sequence_backtracking_conditions ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_orderless_patternsequence_group_matching ... ok
test runtime::tests::session_requests::matcher_resume::upvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_replace_all ... ok
test runtime::tests::sealed_visibility::message_catalog_reveals_with_diagnostic ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_target ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_normalize_rules_after_suspending_rule_source ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_pattern_symbol ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_match_limit ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replace_after_suspending_levelspec ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceall_after_suspending_rule_source ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replace_after_suspending_target ... ok
test runtime::tests::session_requests::matcher_resume::traversal_count_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume::compiled_rule_vec_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume::normalized_rule_expr_vec_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::matcher_resume::sequence_binding_vec_continuation_roundtrip_preserves_multiple_bindings ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceat_after_suspending_position_spec ... ok
test runtime::tests::session_requests::transforms_resume::deletecases_iterative_tail_match_resume_matches_uninterrupted_association_walk ... ok
test runtime::tests::session_requests::transforms_resume::replace_levelspec_option_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_patterntest_matchq ... ok
test runtime::tests::session_requests::transforms_resume::replacepart_resumes_mid_walk_with_the_same_rule_and_partial_result ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacepart_with_suspending_delayed_rhs ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacelist_after_multiple_backtracking_matches ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacelist_after_suspending_delayed_rhs ... ok
test runtime::tests::session_requests::transforms_resume::deletecases_iterative_tail_match_resume_matches_uninterrupted_call_walk ... ok
test runtime::tests::session_requests::transforms_resume_patterns::eval_request_reports_prints_on_failure_and_across_input_resume ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacepart_after_suspending_pattern_position_spec ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_except_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_setattributes_with_suspending_attribute_spec ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_repeated_patterntest_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_sequence_patterntest_matchq ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceat_after_suspending_delayed_rhs ... ok
test runtime::tests::session_requests::transforms_resume::replace_iterative_tail_resumes_at_the_same_deep_rule_attempt ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_thread_with_suspending_arg_spec ... ok
test runtime::tests::structure_data::replace::replacepart_deep_path_shallow_companion_pins_three_level_rebuild ... ok
test runtime::tests::structure_data::replace::replaceat_supports_exact_positions_operator_form_and_rule_lists ... ok
test runtime::tests::session_requests::api::script_requests_report_file_coordinates_for_every_form_shape ... ok
test runtime::tests::structure_data::replace::replacepart_rule_lists_preserve_sequential_conflicts_and_missing_positions ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_declining_later_guard_hands_position_to_earlier_rule ... ok
test runtime::tests::structure_data::replace::replacepart_handles_duplicate_and_overlapping_targets_more_stably ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_unguarded_rules_keep_explicit_later_rule_wins_behavior ... ok
test runtime::tests::structure_data::replace::replacepart_span_preserves_lenient_missing_positions_and_raw_delayed_rhs ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_guard_acceptance_preserves_raw_delayed_bodies ... ok
test runtime::tests::structure_data::replace::replacepart_supports_pattern_position_specs_and_rhs_bindings ... ok
test runtime::tests::structure_data::replace::replacepart_spans_reuse_part_selection_and_fail_closed_on_malformed_shapes ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_guard_suspension_resumes_same_rule_position_and_plan ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_evaluates_each_rule_position_guard_at_most_once ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_rhs_guards_accept_and_decline_exact_position_forms ... ok
test runtime::tests::structure_data::selectors_part2::invalid_span_messages_render_infix_specs ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_emit_single_take_for_explicit_invalid_spans ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_contested_guards_run_last_to_first_and_stop_after_acceptance ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_pattern_guards_receive_bindings_and_option_scope ... ok
test runtime::tests::structure_data::selectors_part2::invalid_upto_span_diagnostics_are_not_duplicated_after_normalization ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_take_messages_render_span_ranges ... ok
test runtime::tests::structured_exceptions::exception_type_registry_round_trips_through_loader_snapshot ... ok
test runtime::tests::structure_data::selectors_part2::direct_empty_span_selects_full_range ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_report_branch_selector_failures ... ok
test runtime::tests::structure_data::replace::replacepart_supports_immediate_delayed_and_head_replacements ... ok
test runtime::tests::structure_data::selectors_part2::direct_single_endpoint_span_selects_prefix_or_suffix_ranges ... ok
test runtime::tests::structured_exceptions::declining_exception_handler_guard_continues_ancestor_and_handler_scans ... ok
test runtime::tests::structured_exceptions::suspending_exception_handler_guard_resumes_its_scan_cursor_without_replay ... ok
test runtime::tests::suspension_abandonment::abandonment_generations_are_monotonic_and_pure_frames_run_no_continuation ... ok
test runtime::tests::suspension_abandonment::abandonment_finishes_module_activation_without_initializers_or_body ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_monitor_without_a_final_refresh_or_callback ... ok
test runtime::tests::suspension_abandonment::abandonment_evaluates_only_withcleanup_cleanup_and_abandons_nested_input_lifo ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_an_existing_time_deadline_exactly ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_reap_without_rendering_or_running_body_suffix ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_option_value_scope_without_running_definition_suffix ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_block_random_prng_state ... ok
test runtime::tests::session_requests::api::file_request_maps_resource_limit_exit_and_cancel_errors ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_abort_protect_and_check_abort_checkpoints ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_block_initializers_and_bodies_inner_to_outer ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_constraint_nesting_without_fallbacks ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_both_loop_iterator_scope_forms_and_iteration_budget ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_test_report_stream_capture_without_a_report ... ok
test runtime::tests::suspension_abandonment::abandonment_rolls_back_partial_package_install_without_reporting_failure ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_trace_and_evaluation_data_captures_without_rendering ... ok
test runtime::tests::suspension_abandonment::checkpoint_rollback_preserves_its_captured_generation ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_check_and_quiet_captures_without_check_fallback ... ok
test runtime::tests::suspension_abandonment::checkpoint_rollback_abandons_a_trial_generation_before_restoring_state ... ok
test runtime::tests::syntax_information_lifecycle::unsetting_system_syntax_information_reverts_to_default_rules ... ok
test runtime::tests::syntax_information_debug_catalog::syntax_information_covers_current_cl09_debugging_surface_subset ... ok
test runtime::tests::suspension_abandonment::queued_input_abandons_the_older_generation_before_consuming_the_value ... ok
test runtime::tests::syntax_information_lifecycle::protect_blocks_direct_syntax_information_assignment_and_unset ... ok
test runtime::tests::syntax_information_lifecycle::syntax_information_defaults_assignment_and_unset_cover_current_first_pass_subset ... ok
test runtime::tests::syntax_information_lifecycle::syntax_information_participates_in_block_and_clearall ... ok
test runtime::tests::thread_and_dispatch::evaluated_head_function_attributes_control_argument_evaluation ... ok
test runtime::tests::thread_and_dispatch::downvalue_miss_stability_shortcut_invalidates_on_later_definition ... ok
test runtime::tests::thread_and_dispatch::block_localizes_definitions_attributes_and_messages ... ok
test runtime::tests::session_requests::transforms_resume::replace_and_replaceat_indexed_rule_plans_survive_mid_traversal_suspension ... ok
test runtime::tests::thread_and_dispatch::argument_preparation_cow_preserves_dispatch_semantics ... ok
test runtime::tests::thread_and_dispatch::listable_interacts_with_builtin_and_function_attributes ... ok
test runtime::tests::thread_and_dispatch::held_downvalue_matching_normalizes_nested_attribute_heads ... ok
test runtime::tests::thread_and_dispatch::one_identity_allows_singleton_head_patterns_to_match_atoms ... ok
test runtime::tests::thread_and_dispatch::module_skips_existing_generated_symbol_names ... ok
test runtime::tests::thread_and_dispatch::listable_threads_user_defined_calls_and_broadcasts_scalars ... ok
test runtime::tests::thread_and_dispatch::flat_attribute_flattens_calls_and_matches_flat_patterns ... ok
test runtime::tests::thread_and_dispatch::listable_mismatched_lengths_emit_thread_message_and_leave_call_unevaluated ... ok
test runtime::tests::thread_and_dispatch::stable_call_shortcut_preserves_symbol_and_upvalue_evaluation ... ok
test runtime::tests::thread_and_dispatch::orderless_attribute_canonicalizes_calls_and_rules ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_accepts_resolved_head_and_spec_values ... ok
test runtime::tests::thread_and_dispatch::thread_evaluates_its_first_argument_before_threading ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_default_and_custom_thread_heads ... ok
test runtime::tests::traversal_part2::cases_and_position_report_invalid_match_limit_messages ... ok
test runtime::tests::traversal_part2::count_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::thread_and_dispatch_part2::thread_builtin_mismatch_emits_message_and_stays_unevaluated ... ok
test runtime::tests::traversal_part2::level_reports_invalid_levelspec_messages ... ok
test runtime::tests::traversal_part2::freeq_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::position_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::memberq_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_standard_sequence_spec_subset ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_wide_dense_argument_selection ... ok
test runtime::tests::valueq_stable_shortcut::value_q_shortcut_populates_only_evaluation_caches ... ok
test runtime::tests::valueq_stable_shortcut::value_q_shortcut_skips_snapshot_and_suppression_even_after_attribute_revision ... ok
test runtime::tests::traversal_part2::traversal_family_reports_invalid_levelspec_messages ... ok
test runtime::tests::wide_child_traversals::cases_resumes_mid_walk_between_children ... ok
test runtime::tests::valueq_stable_shortcut::value_q_stability_shortcut_matches_the_trial_path_over_one_shared_corpus ... ok
test runtime::tests::wide_definition_dispatch::sub_value_dispatch_resumes_mid_walk_with_pending_outcome ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_resumes_mid_walk_with_pending_outcome ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_resumes_to_next_candidate_when_condition_fails ... ok
test runtime::tests::left_chain_lowering::left_chain_parser_rejects_20_000_links_gracefully_for_all_operators ... ok
test runtime::tests::scoping_gensym_gc::reachability::recursive_module_sweep_reclaims_dead_retained_gensyms ... ok
test runtime::tests::wide_child_traversals::replace_all_walks_thirty_thousand_children ... ok
test runtime::tests::session_requests::api::eval_request_print_flood_trips_memory_ceiling_with_classified_termination ... ok
test runtime::tests::wide_child_traversals::position_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::delete_cases_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::count_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::replace_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::cases_walks_thirty_thousand_children ... ok
test runtime::lowering::tests::public_lowering_paths_reject_manual_trees_above_bound ... ok
test runtime::tests::structure_data::replace::replacepart_handles_twenty_thousand_structural_rules_in_one_walk ... ok
test runtime::tests::wide_definition_dispatch::replace_list_walks_thirty_thousand_rules ... ok
test runtime::tests::wide_definition_dispatch::reap_walks_thirty_thousand_buckets ... ok
test runtime::evaluation_cache::tests::association_stability_tail::cache_cap_postorder_insertion_trace_matches_recursive_front ... ok
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... ok
test runtime::tests::thread_and_dispatch::module_handles_many_uninitialized_bindings_without_exhausting_the_native_stack ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_accepts_the_boundary_for_all_operators ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_walks_thirty_thousand_definitions ... ok
test runtime::tests::wide_definition_dispatch::sub_value_dispatch_walks_thirty_thousand_definitions ... ok
test runtime::tests::iterative_loop_drivers::until_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::module_drives_thirty_thousand_initialized_bindings_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::while_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::for_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok

test result: ok. 1012 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.86s

     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)

running 989 tests
test associations::basic::lookup::tests::lookup_outcome_drop_is_bounded_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack drop certificate; run explicitly with --release
test associations::basic::lookup::tests::lookup_query_count_forced_front_tail_matches_complete_front_exactly ... ok
test associations::key_reshape::traversal::tests::insert_path_forced_front_tail_matches_complete_front_on_branched_fixture ... ok
test associations::basic::lookup::tests::lookup_planning_forced_front_tail_matches_complete_front_exactly ... ok
test associations::basic::keys::tests::keys_forced_tail_preserves_first_error_in_source_order ... ok
test associations::basic::values::tests::values_forced_tail_preserves_outer_expression_error_payload ... ok
test associations::basic::keys::tests::keys_forced_front_tail_matches_complete_front_exactly ... ok
test associations::key_reshape::traversal::tests::flatten_association_forced_front_tail_matches_complete_front_on_branched_fixture ... ok
test associations::basic::values::tests::values_forced_front_tail_matches_complete_front_exactly ... ok
test associations::basic::key_take_drop::tests::key_take_drop_forced_tail_preserves_invalid_payload_distinction ... ok
test associations::basic::lookup::tests::lookup_rendering_forced_fronts_agree_owned_borrowed_and_exactly ... ok
test associations::missing::traversal::counting::tests::zero_front_uses_heap_tails_for_delayed_wrapper_counts ... ok
test associations::missing::traversal::projection::tests::zero_front_heap_projection_preserves_the_exact_preflight_count ... ok
test polynomials::core::factors::tests::ordered_factor_merge_preserves_canonical_ties ... ok
test polynomials::core::intersection::tests::intersection_matches_structural_bases_across_canonical_ties ... ok
test json::encode::depth_tests::to_json_accepts_nesting_at_the_depth_limit ... ok
test polynomials::core::factors::tests::ordered_factor_merge_matches_concat_and_normalize ... ok
test associations::key_reshape::traversal::tests::render_nested_path_forced_front_tail_matches_complete_front_on_branched_fixture ... ok
test json::encode::depth_tests::to_json_refuses_nesting_one_past_the_depth_limit ... ok
test polynomials::core::factors::tests::batch_term_product_matches_repeated_binary_product ... ok
test strings::patterns::matcher::engine::tests::except_filters_with_one_excluded_evaluation ... ok
test strings::patterns::matcher::literal::tests::literal_matches_preserve_overlap_and_unicode_character_offsets ... ok
test strings::patterns::matcher::reverse::tests::reversed_number_atom_preserves_full_match_language ... ok
test strings::patterns::matcher::api::tests::reversible_suffix_pattern_uses_two_match_attempts ... ok
test strings::patterns::matcher::api::tests::reversed_suffix_matching_preserves_repeated_capture_equality ... ok
test strings::patterns::matcher::split_trim::tests::mixed_list_stays_on_the_general_pattern_path ... ok
test strings::similarity::selection::tests::rational_comparison_handles_equal_and_distinct_denominators ... ok
test strings::patterns::matcher::reverse::tests::reversal_swaps_sequence_direction_and_boundaries ... ok
test predicates::collections::matching_tests::duplicate_free_state_visits_the_exact_expression_roots ... ok
test strings::similarity::selection::tests::partial_nearest_selection_preserves_distance_then_source_order ... ok
test functional_ops::apply_map::traversal::map_indexed::tests::map_indexed_forced_front_tail_matches_complete_front_on_branched_fixture ... ok
test structure_ops::sequence::delete::tests::delete_planner_starts_at_the_measured_crossover ... ok
test structure_ops::sequence::edit_plan::tests::many_deletions_use_one_linear_parent_pass ... ok
test strings::scripting::html::tests::unterminated_entity_scan_work_probe ... ok
test strings::patterns::regex::tests::reversed_regex_accepts_exactly_reversed_full_matches ... ok
test structure_ops::sequence::edit_plan::tests::many_insertions_use_one_linear_parent_pass ... ok
test strings::patterns::matcher::state_set::tests::wide_distinct_ends_build_one_index_after_the_small_scan_tier ... ok
test strings::patterns::matcher::literal::tests::linear_literal_search_bounds_long_prefix_comparison_work ... ok
test strings::similarity::selection::tests::best_ties_selection_uses_one_comparison_pass ... ok
test structure_ops::sequence::insert::tests::insert_planner_starts_at_the_measured_crossover ... ok
test tests::association_key_reshape_depth::association_key_deflatten_abandons_a_deep_partial_tree_without_growing_the_stack ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_deflatten_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test strings::patterns::matcher::state_set::tests::indexed_collision_chain_compares_bindings_and_preserves_order ... ok
test tests::association_key_reshape_depth::association_key_deflatten_flatten_round_trip_is_exact_at_physical_height_4097 ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_flatten_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_take_drop_depth::key_drop_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_take_drop_depth::key_take_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_keys_depth::keys_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_lookup_depth::lookup_hit_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_lookup_depth::lookup_miss_and_resume_drop_complete_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_values_depth::values_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_depth_contract::recursion_inventory::synthetic_unregistered_recursion_is_reported_exactly ... ok
test structure_ops::sequence::edit_plan::tests::batched_insert_matches_descending_sequential_reference_with_overlaps ... ok
test associations::basic::key_take_drop::tests::key_take_drop_forced_front_tail_matches_complete_front_for_both_flags ... ok
test structure_ops::sequence::take_drop::tests::iterative_take_tail_preserves_fanout_order ... ok
test structure_ops::sequence::take_drop::tests::iterative_drop_tail_preserves_fanout_order ... ok
test structure_ops::sequence::edit_plan::tests::batched_delete_matches_descending_sequential_reference_with_overlaps_and_head ... ok
test associations::missing::traversal::deletion::tests::zero_front_heap_tail_matches_front_for_branched_associations ... ok
test polynomials::factor_collections::tests::indexed_factor_collections_preserve_order_and_multiplicity ... ok
test strings::patterns::matcher::split_trim::tests::literal_list_fast_path_is_selected_once_and_bounds_boundary_work ... ok
test polynomials::core::term_order::tests::sparse_order_matches_dense_basis_order_exhaustively ... ok
test polynomials::core::construction::tests::indexed_batch_merge_cancels_duplicates_and_sorts_once ... ok
test polynomials::core::variables::tests::wide_variable_collection_indexes_overlaps_and_sorts_once ... ok
test structure_ops::sequence::take_drop::tests::linear_drop_filter_matches_set_membership_for_small_ranges ... ok
test polynomials::core::term_order::tests::sparse_order_avoids_wide_zero_coordinate_probes ... ok
test strings::similarity::selection::tests::partial_nearest_selection_matches_full_sort_with_linear_comparison_work ... ok
test catalog::memory_growth_contract::installed_builtin_memory_growth_inventory_is_sorted_and_exhaustive ... ok
test tests::associations::associations_key_reshape::key_group_by_groups_association_entries_by_transformed_keys ... ok
test tests::association_key_reshape_depth::association_key_flatten_three_level_companion_pins_exact_paths_and_rule_heads ... ok
test tests::associations::associations::association_canonicalization_preserves_order_and_last_rule_above_index_threshold ... ok
test tests::associations::associations_key_reshape::merge_nested_suspended_combiner_resumes_without_repeating_calls ... ok
test tests::associations::associations_key_reshape::association_key_flatten_and_deflatten_transform_path_keys ... ok
test tests::associations::association_part_cursor::association_key_selector_suspension_resumes_the_original_part_call ... ok
test tests::association_key_reshape_depth::association_key_deflatten_three_level_companion_pins_order_overwrite_and_conflicts ... ok
test tests::association_lookup_depth::lookup_three_level_companion_pins_shape_hits_misses_default_and_wrapper ... ok
test tests::associations::associations_key_reshape::association_key_deflatten_preserves_wide_branch_order_after_indexing ... ok
test associations::missing::target::tests::resume_target_work_given_a_carried_prefix_does_not_replay_it ... ok
test tests::associations::associations_key_reshape::association_key_flatten_and_deflatten_round_trip_well_formed_inputs ... ok
test tests::associations::associations::association_and_associationq_cover_first_pass_rule_subset ... ok
test tests::associations::associations_basic_diagnostics::association_lookup_keys_values_diagnostics_cover_invalid_forms ... ok
test tests::associations::associations_key_reshape::merge_nested_recursively_merges_associations_and_combines_leaf_values ... ok
test tests::associations::associations_aggregate_diagnostics::association_aggregate_diagnostics_cover_invalid_forms ... ok
test tests::associations::association_part_cursor::association_part_and_extract_value_message_matrix_is_exact ... ok
test tests::associations::associations_key_reshape::merge_nested_three_level_companion_pins_branch_order_and_combiner_schedule ... ok
test tests::associations::associations_key_diagnostics::key_transform_diagnostics_cover_invalid_forms ... ok
test predicates::collections::matching_tests::augmenting_path_matching_agrees_with_exhaustive_small_graphs ... ok
test structure_ops::sequence::edit_plan::tests::batched_delete_matches_reference_for_every_nested_path_subset ... ok
test tests::associations::associations::lookup_keys_values_and_key_cover_first_pass_association_queries ... ok
test tests::associations::associations_key_reshape::merge_nested_suspension_freezes_the_iterative_tail_without_replay ... ok
test tests::associations::associations_missing_diagnostics::missing_and_delete_missing_diagnostics_cover_invalid_forms ... ok
test tests::associations::associations_key_reshape::nested_association_additions_leave_malformed_inputs_inert ... ok
test tests::associations::associations_key_sort::custom_key_sort_by_decorates_once_in_input_order ... ok
test tests::associations::associations_key_reshape::nested_association_additions_support_operator_forms ... ok
test tests::associations::associations_key_arity_diagnostics::key_function_arity_and_currying_diagnostics_cover_invalid_forms ... ok
test tests::association_values_depth::values_three_level_companion_pins_wrappers_sibling_value_order_and_outer_error ... ok
test tests::association_key_take_drop_depth::key_take_drop_three_level_companion_pins_wrappers_sibling_key_order_and_rule_heads ... ok
test tests::associations::associations_mutation::associateto_mutates_deep_key_path_targets ... ok
test tests::associations::associations_mutation::associateto_mutates_bulk_nested_part_targets ... ok
test tests::associations::associations_mutation::keydropfrom_rejects_bulk_nested_part_targets_without_mutation ... ok
test tests::associations::associations_key_value_pattern::key_value_pattern_matches_association_and_rule_inputs ... ok
test tests::association_keys_depth::keys_three_level_companion_pins_wrappers_sibling_key_order_and_first_error ... ok
test tests::associations::associations_key_sort::custom_key_sorts_preserve_the_full_comparator_result_policy ... ok
test tests::associations::associations_mutation::associateto_mutates_mixed_integer_and_key_part_targets ... ok
test tests::associations::associations_mutation::keydropfrom_mutates_mixed_integer_and_key_leaf_targets ... ok
test tests::associations::associations_key_sort::custom_key_sorts_preserve_entry_forms_ordering_and_stable_ties ... ok
test tests::associations::associations_mutation::keydropfrom_mutates_deep_key_path_targets ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_after_attribute_invalidation_addresses_the_new_held_identity ... ok
test tests::associations::associations_key_sets::association_thread_and_key_set_ops_cover_first_pass_subset ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_after_owner_clear_uses_only_the_reinstalled_identity ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_curried_operator_forms_store_the_prepared_identity ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_after_index_rebind_and_clear_preserves_older_snapshots ... ok
test structure_ops::sequence::edit_plan::tests::batched_insert_matches_reference_for_every_nested_path_subset ... ok
test tests::associations::associations_mutation_operator_forms::association_mutation_operator_forms_report_application_arity ... ok
test tests::associations::associations_mutation_definition_fallback::symbol_and_part_mutation_fast_paths_remain_separate_from_definition_preparation ... ok
test tests::associations::associations_mutation_operator_forms::association_mutation_heads_advertise_subvalues_hold_all ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_reads_index_and_outer_definition_exactly_once ... ok
test tests::associations::associations_mutation_diagnostics::association_mutation_diagnostics_cover_invalid_targets_and_arity ... ok
test tests::associations::associations_mutation_operator_forms::association_mutation_operator_forms_apply_to_raw_held_targets ... ok
test tests::associations::associations_part2::group_by_nested_specs_rekey_delayed_association_values_per_level ... ok
test tests::associations::associations_mutation_surface::keydropfrom_preserves_association_order_at_indexed_width ... ok
test tests::associations::associations_part2::group_by_rule_specs_consume_delayed_association_values_for_key_and_value ... ok
test tests::associations::associations_part2::group_by_wide_association_preserves_group_and_entry_order ... ok
test tests::associations::associations_set_lvalue::set_association_lvalue_falls_back_or_reports_failed_nested_keys ... ok
test tests::associations::associations_set_lvalue::set_single_association_key_still_refuses_a_protected_target ... ok
test tests::bytes::byte_catalog_entries_are_pure ... ok
test tests::associations::associations_part2::counts_explicit_key_specs_cover_nested_duplicate_and_empty_edges ... ok
test tests::associations::associations_key_sort::every_custom_key_sort_by_key_resumes_with_the_uninterrupted_trace ... ok
test tests::associations::associations_set_lvalue::set_single_association_key_finishes_rhs_and_selector_evaluation_first ... ok
test tests::associations::associations_set_lvalue::set_single_association_key_keeps_canonicalization_and_invalid_fallbacks ... ok
test tests::associations::join_across::join_across_accepts_empty_sides_for_every_join_type ... ok
test tests::associations::join_across::join_across_duplicate_values_form_an_ordered_cross_product ... ok
test tests::associations::associations_set_lvalue::set_single_association_keys_preserves_aliases_order_and_rule_heads ... ok
test tests::associations::join_across::join_across_left_right_and_outer_preserve_walk_order_and_bare_rows ... ok
test tests::associations::associations_set_lvalue::set_writes_association_function_and_part_keys ... ok
test tests::associations::associations_mutation_surface::associateto_keydropfrom_and_keyvaluepattern_cover_first_pass_mutation_surface ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_resumes_preparation_and_outer_read_without_replay_for_both_heads ... ok
test tests::associations::join_across::join_across_failures_emit_one_declared_message_and_stay_inert ... ok
test tests::associations::join_across::join_across_inner_join_is_left_major ... ok
test tests::associations::join_across::join_across_rule_renamed_keys_keep_both_columns ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_uses_canonical_owner_for_head_specific_protection_checks ... ok
test tests::associations::join_across::join_across_supports_multi_key_and_expression_key_specs ... ok
test tests::associations::join_across::join_across_missing_keys_are_unmatched_without_padding ... ok
test tests::association_depth_contract::recursion_inventory::association_recursion_inventory_is_sorted_exhaustive_and_declared ... ok
test tests::association_depth_contract::registry::real_catalog_misdeclaration_names_association_key_flatten_recursive_edge - should panic ... ok
test tests::catalog_attributes_match_kernel_vocabulary ... ok
test tests::catalog_capabilities::date_and_time_object_predicates_remain_pure ... ok
test tests::catalog_capabilities::datetime_catalog_capabilities_match_clock_separation_surface ... ok
test tests::catalog_capabilities::random_catalog_capabilities_match_dispatch_surface ... ok
test tests::association_depth_contract::registry::association_depth_registry_is_sorted_and_exhaustive ... ok
test tests::bytes::crc32_known_vectors_are_unsigned_integers ... ok
test tests::bytes::digest_json_end_to_end_is_pinned ... ok
test tests::associations::join_across::join_across_uses_right_row_values_on_column_collisions ... ok
test associations::missing::target::tests::target_front_handoff_carries_live_cursors_and_completed_prefixes ... ok
test tests::builtin_reference_docgen::builtin_reference_is_in_sync ... ok
test tests::associations_missing_surface::missing_and_delete_missing_cover_first_pass_subset ... ok
test tests::bytes::hex_and_base64_codecs_round_trip_boundaries ... ok
test tests::bytes::unit6_codec_byte_datetime_and_schema_heads_are_protected ... ok
test tests::associations::join_across::join_across_wide_key_index_preserves_duplicate_and_missing_key_semantics ... ok
test tests::calculus::d_preserves_rational_and_abstract_power_rules ... ok
test tests::bytes::unsupported_argument_types_remain_inert ... ok
test tests::calculus::d_preserves_nested_chain_rule_factor_order ... ok
test tests::calculus::d_supports_indexed_variables_and_higher_order_specs ... ok
test tests::calculus::d_keeps_current_variable_boundary_for_pattern_or_non_symbol_heads ... ok
test tests::calculus::d_preserves_plus_zero_elision_and_combination ... ok
test tests::calculus::d_preserves_abstract_call_fallback_and_variable_free_prepass ... ok
test tests::codecs_matrix::grand_tour_json_toml_yaml_json_is_identity ... ok
test tests::calculus::d_symbolic_order_and_valid_derivatives_stay_message_free ... ok
test tests::codecs_matrix::csv_headers_flow_into_json_and_back_to_csv ... ok
test tests::calculus::d_reports_concrete_invalid_variable_and_negative_order_specs ... ok
test tests::calculus::d_preserves_times_term_and_factor_order ... ok
test tests::calculus::d_handles_polynomial_and_rational_function_rules ... ok
test tests::calculus::d_uses_abstract_derivative_forms_for_symbol_headed_calls ... ok
test tests::codecs_matrix::json_toml_json_round_trip_preserves_the_shared_value ... ok
test tests::codecs_matrix::ini_flows_into_json_and_back_as_strings ... ok
test tests::associations::associations_mutation_surface::association_part_mutations_evaluate_symbol_selectors_before_reading_the_owner ... ok
test tests::codecs_text::codecs_compose_with_json_and_editing_flows ... ok
test tests::codecs_matrix::json_yaml_json_round_trip_preserves_the_shared_value ... ok
test tests::codecs_matrix::xml_values_extract_into_the_shared_model ... ok
test tests::codecs_text::from_csv_parses_core_rows_quoting_newlines_and_unicode ... ok
test tests::bytes::unknown_digest_output_format_fails_closed ... ok
test tests::associations::associations_key_sort::every_custom_key_sort_comparison_resumes_with_the_uninterrupted_trace ... ok
test tests::codecs_text::csv_codecs_fail_closed_with_messages ... ok
test tests::codecs_text::to_csv_encodes_lists_and_association_rows ... ok
test tests::codecs_text::from_ini_parses_documented_dialect ... ok
test tests::codecs_text::csv_headers_tsv_ragged_rows_and_round_trips_are_pinned ... ok
test tests::codecs_toml::from_toml_maps_datetime_forms ... ok
test tests::bytes::byte_array_to_string_and_predicate_cover_boundaries ... ok
test tests::codecs_text::ini_codecs_fail_closed_with_messages ... ok
test tests::codecs_toml::from_toml_golden_error_corpus_is_byte_stable ... ok
test tests::codecs_text::ini_to_ini_output_and_round_trips_are_pinned ... ok
test tests::codecs_toml::from_toml_depth_limit_rejects_each_descent_one_past ... ok
test tests::codecs_toml::from_toml_multiline_strings_trim_and_continue_lines ... ok
test tests::codecs_toml::from_toml_rejects_invalid_input_fail_closed ... ok
test tests::codecs_text::to_csv_wide_association_rows_follow_header_order ... ok
test tests::codecs_toml::from_toml_depth_limit_rejects_the_far_past_stack_exhaustion_shape ... ok
test tests::codecs_toml::toml_codecs_compose_with_json ... ok
test tests::codecs_text::ini_wide_key_and_section_indexes_preserve_round_trips ... ok
test tests::codecs_toml::from_toml_numbers_are_exact ... ok
test tests::codecs_toml::from_toml_golden_corpus_is_byte_stable ... ok
test tests::codecs_toml::to_toml_golden_error_corpus_is_byte_stable ... ok
test tests::codecs_toml::to_toml_selects_tables_quotes_keys_and_preserves_assignment_order ... ok
test tests::codecs_xml::from_xml_decodes_unicode_and_numeric_entities ... ok
test tests::codecs_toml::from_toml_parses_realistic_config ... ok
test tests::codecs_toml::to_toml_golden_corpus_is_byte_stable ... ok
test tests::codecs_xml::from_xml_depth_limit_rejects_the_far_past_stack_exhaustion_shape ... ok
test tests::codecs_xml::from_xml_depth_limit_rejects_element_nesting_one_past ... ok
test tests::codecs_xml::xml_round_trips_are_pinned ... ok
test tests::codecs_xml::to_xml_rejects_non_xml_trees_fail_closed ... ok
test tests::codecs_xml::from_xml_rejects_invalid_xml_fail_closed ... ok
test tests::codecs_xml::to_xml_encodes_compact_escaped_and_pretty_forms ... ok
test tests::codecs_xml::from_xml_parses_realistic_document_to_symbolic_tree ... ok
test tests::codecs_toml::from_toml_depth_limit_accepts_each_descent_at_the_limit ... ok
test tests::codecs_xml::xml_values_compose_with_json ... ok
test tests::codecs_yaml::from_yaml_literal_blocks_pin_chomping ... ok
test tests::codecs_yaml::from_yaml_golden_error_corpus_is_byte_stable ... ok
test tests::codecs_yaml::from_yaml_golden_corpus_is_byte_stable ... ok
test tests::codecs_yaml::from_yaml_uses_norway_safe_typing ... ok
test tests::codecs_yaml::yaml_codecs_compose_with_json ... ok
test tests::codecs_yaml::from_yaml_rejects_unsupported_yaml_fail_closed ... ok
test tests::codecs_yaml::from_yaml_parses_k8s_flavored_config ... ok
test tests::codecs_xml::xml_wide_attribute_index_preserves_round_trip ... ok
test tests::codecs_yaml::from_yaml_depth_limit_accepts_every_collection_descent_at_the_limit ... ok
test tests::codecs_yaml::from_yaml_numbers_and_nulls_are_exact ... ok
test tests::codecs_yaml::from_yaml_depth_limit_rejects_every_collection_descent_one_past ... ok
test tests::combinatorics::binomial_respects_integer_work_limit ... ok
test tests::codecs_yaml::to_yaml_round_trips_adversarial_strings_and_empties ... ok
test tests::combinatorics::cheap_combinatorics_builtins_report_declared_attributes ... ok
test tests::codecs_yaml::yaml_wide_mapping_index_preserves_round_trip ... ok
test tests::combinatorics::fibonacci_respects_integer_work_limit_boundaries ... ok
test tests::combinatorics::cheap_combinatorics_builtins_integer_anchors ... ok
test tests::datetime::absolute_time_reads_exact_timestamp_and_composes_with_date_list ... ok
test tests::codecs_xml::from_xml_depth_limit_accepts_deep_element_nesting ... ok
test tests::combinatorics::fibonacci_wrong_arity_reports_argx_and_stays_inert ... ok
test tests::combinatorics::recurrence_combinatorics_respect_integer_work_limit ... ok
test tests::datetime::date_plus_and_date_difference_use_exact_fixed_duration_arithmetic ... ok
test tests::datetime::date_list_date_string_absolute_time_and_from_date_list_report_arity_errors ... ok
test tests::bytes::digest_known_answer_vectors_cover_string_and_byte_array_inputs ... ok
test tests::datetime::date_plus_operator_form_rewrites_to_data_first_call ... ok
test tests::delete_missing_depth::delete_missing_recursive_armed_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::datetime::date_list_round_trips_exact_rational_timestamps ... ok
test tests::delete_missing_depth::delete_missing_recursive_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::datetime::from_date_string_accepts_documented_iso_subset_and_offsets ... ok
test tests::delete_missing_depth::delete_missing_scan_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::datetime::leap_year_and_month_length_edges_are_validated ... ok
test tests::delete_missing_target_certificates::delete_missing_target_association_clears_64000_siblings_on_two_mib ... ignored, release-only width probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_list_clears_64000_siblings_on_two_mib ... ignored, release-only width probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_scan_all_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_scan_zero_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::datetime::from_date_string_fails_closed_on_invalid_shapes ... ok
test tests::datetime::day_of_week_returns_iso_weekday_symbols ... ok
test tests::debug_tools_resources::sow_and_reap_collect_by_tag_and_optional_handler ... ok
test tests::datetime::from_date_list_and_date_list_epoch_pins_are_utc ... ok
test tests::datetime::date_string_renders_utc_and_rejects_non_terminating_fractional_seconds ... ok
test tests::debug_tools_resources::memory_constraint_checks_each_repeated_string_growth_step ... ok
test tests::debug_tools_resources::memory_constraint_keeps_projected_string_repeat_precheck ... ok
test tests::debug_tools_resources::timing_and_absolute_timing_return_elapsed_and_value ... ok
test tests::combinatorics::cheap_combinatorics_builtins_thread_and_passthrough_current_scope ... ok
test tests::debug_tools_evaluation::evaluation_data_captures_result_messages_prints_and_failure_type_without_leaking_streams ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_prepares_every_discovered_shape_for_both_mutation_heads ... ok
test tests::debug_tools_resources::monitor_runs_current_display_and_callback_subset ... ok
test tests::delete_missing_depth::delete_missing_recursive_shallow_companion_pins_level_interpretation ... ok
test tests::debug_tools_reports::testreport_merges_existing_reports_and_accepts_report_level_options ... ok
test tests::debug_tools_resources::time_and_memory_constraints_cover_current_first_pass_resource_subset ... ok
test tests::definition_lhs_assignment_resume::unset_lhs_preparation_resumes_exactly_once_without_replay ... ok
test tests::delete_missing_target_semantic_locks::cancelling_target_mode_suspension_discards_frames_without_callbacks ... ok
test tests::delete_missing_depth::delete_missing_scan_shallow_companion_pins_scan_depth ... ok
test tests::definition_lhs_assignment_resume::tagunset_resumes_preparation_once_and_removes_only_the_canonical_identity ... ok
test tests::delete_missing_depth::delete_missing_recursive_armed_shallow_companion_pins_level_and_delayed_head ... ok
test tests::definition_lhs_assignment_resume::value_table_lhs_preparation_resumes_multiple_entries_without_replay ... ok
test tests::delete_missing_target_semantic_locks::delayed_rhs_above_target_gets_one_level_minus_one_cleanup_wrapper ... ok
test tests::delete_missing_target_certificates::delete_missing_target_shallow_companion_pins_level_and_scan ... ok
test tests::definition_lhs_assignment_resume::upset_resumes_rhs_inferred_tag_and_lhs_once_without_replay ... ok
test tests::definition_lhs_assignment_resume::tagset_resumes_rhs_tag_position_and_lhs_once_in_settled_order ... ok
test tests::exact_reduction_shapes::mean_reports_empty_and_invalid_arity_boundaries ... ok
test tests::definition_lhs_assignment_resume::value_table_failure_after_suspension_keeps_the_old_table_intact ... ok
test tests::associations::associations_part2::group_by_wide_association_nested_and_reducer_phases_preserve_order_and_call_counts ... ok
test tests::delete_missing_target_semantic_locks::target_association_evaluates_only_reached_delayed_rhs_once ... ok
test tests::debug_tools_resources::time_constrained_immediate_input_resume_finishes_before_deadline ... ok
test tests::exact_reduction_shapes::exact_statistics_cover_median_commonest_and_quantile_contracts ... ok
test tests::debug_tools_reports::testcreate_testreport_and_stack_cover_current_first_pass_subset ... ok
test tests::exact_reduction_shapes::exact_denominator_and_apply_integer_reducers_keep_certificate_shape ... ok
test tests::exact_reduction_shapes::commonest_accepts_count_limit_form ... ok
test tests::delete_missing_target_semantic_locks::target_mode_propagates_non_input_errors_unchanged ... ok
test tests::exact_reduction_shapes::exact_statistics_report_empty_and_invalid_boundaries ... ok
test tests::delete_missing_target_semantic_locks::target_list_parent_resumes_first_delayed_sibling_without_replay ... ok
test tests::functional_ops::callable::callable_functional_builtins_report_current_argument_forms ... ok
test tests::exact_reduction_shapes::quantile_accepts_parameter_lists_and_fails_closed ... ok
test tests::functional_ops::arrays::array_and_outer_cover_current_dense_generation_subset ... ok
test tests::exact_reduction_shapes::commonest_preserves_wide_first_occurrence_order ... ok
test tests::held_association_exact_path_depth::associate_to_part_target_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::functional_ops::apply_map::apply_map_family_reports_current_argument_forms ... ok
test tests::held_association_exact_path_depth::key_drop_from_part_target_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::exact_reduction_shapes::unicode_letter_identifiers_load_as_ordinary_source_symbols ... ok
test tests::functional_ops::arrays::array_functional_builtins_report_current_argument_forms ... ok
test tests::exact_reduction_shapes::exact_aggregation_shapes_reduce_arithmetic_terms ... ok
test tests::delete_missing_target_semantic_locks::target_list_parent_resumes_middle_delayed_sibling_without_prefix_replay ... ok
test tests::functional_ops::arrays::inner_and_dot_dimension_mismatches_report_only_rectangular_same_head_cases ... ok
test tests::functional_ops::callable::operate_supports_head_chain_updates_and_operator_form ... ok
test tests::delete_missing_target_semantic_locks::target_mode_mixed_list_parent_pins_exact_scan_zero_and_all_shapes ... ok
test tests::delete_missing_target_semantic_locks::target_mode_mixed_association_parent_pins_exact_scan_zero_and_all_shapes ... ok
test tests::functional_ops::apply_map::apply_and_mapapply_cover_operator_forms_levelspecs_and_association_values ... ok
test tests::delete_missing_target_semantic_locks::target_list_parent_resumes_last_delayed_sibling_without_prefix_replay ... ok
test tests::bytes::malformed_codecs_fail_closed ... ok
test tests::functional_ops::incidental_callable::through_threads_arguments_through_current_head_subset ... ok
test tests::functional_ops::incidental_apply_map::map_scan_and_comap_family_cover_current_first_pass_subset ... ok
test tests::functional_ops::iteration::iteration_builtins_report_current_argument_forms ... ok
test tests::functional_ops::iteration::fixed_point_cycle_tracking_covers_value_and_list_results ... ok
test tests::delete_missing_target_semantic_locks::target_association_parent_resumes_two_different_list_branches_in_order ... ok
test tests::functional_ops::iteration::nest_and_fixed_point_families_cover_current_iteration_subset ... ok
test tests::functional_ops::arrays::inner_arrayfilter_and_arrayreduce_cover_current_subset ... ok
test tests::functional_ops::targeted_map::mapat_mapthread_and_subsetmap_cover_current_first_pass_subset ... ok
test tests::functional_ops::targeted_map::targeted_mapping_builtins_report_current_argument_forms ... ok
test tests::functional_ops::targeted_map::targeted_mapping_reports_only_explicit_missing_or_incompatible_paths ... ok
test tests::holding_and_control::evaluate_forces_evaluation_inside_held_arguments ... ok
test tests::held_association_exact_path_depth::shared_exact_path_consumers_keep_exact_shallow_behavior ... ok
test tests::held_association_exact_path_depth::held_association_part_three_level_companion_pins_selectors_order_and_results ... ok
test tests::holding_and_control::holdallcomplete_blocks_upvalue_application ... ok
test tests::holding_and_control::holdpattern_preserves_patterns_without_prior_evaluation ... ok
test tests::holding_and_control::holdallcomplete_prevents_evaluate_override_and_sequence_splicing ... ok
test tests::holding_and_control::if_evaluates_only_the_selected_branch ... ok
test tests::holding_and_control::protect_and_unprotect_round_trip_and_leave_attribute_updates_available ... ok
test tests::holding_and_control::sequencehold_prevents_sequence_splicing ... ok
test tests::holding_and_control::sequence_splices_in_non_held_and_ordinary_held_calls ... ok
test tests::holding_and_control::setattributes_and_clearattributes_control_argument_evaluation ... ok
test tests::holding_and_control::unevaluated_consumers_still_receive_the_raw_unwrapped_argument ... ok
test tests::holding_and_control::setattributes_accepts_symbol_lists_and_combined_hold_attributes ... ok
test tests::holding_and_control::setattributes_accepts_resolved_attribute_specs ... ok
test tests::holding_and_control::releasehold_releases_supported_hold_wrappers ... ok
test tests::holding_and_control::unevaluated_is_restored_only_when_no_call_stage_consumes_it ... ok
test tests::functional_ops::targeted_map::subset_map_validates_wide_disjoint_paths_at_scale ... ok
test tests::holding_and_control::unevaluated_sequence_splicing_drops_single_position_provenance ... ok
test tests::holding_and_control::unevaluated_temporarily_preserves_direct_function_arguments ... ok
test tests::holding_and_control::which_evaluates_conditions_in_order ... ok
test tests::holding_and_control::user_defined_holdallcomplete_attribute_holds_all_arguments_completely ... ok
test tests::functional_ops::sequence::sequence_functional_builtins_report_current_argument_forms ... ok
test tests::codecs_yaml::from_yaml_depth_limit_rejects_the_far_past_stack_exhaustion_shape ... ok
test tests::holding_and_control_abort::abortprotect_and_failsafe_resume_through_host_input ... ok
test tests::holding_and_control_part3::block_supports_sequential_initializers_and_nested_shadowing ... ok
test tests::holding_and_control_part3::block_applies_dynamic_scope_across_called_definitions ... ok
test tests::functional_ops::arrays::array_and_outer_respect_resource_limits ... ok
test tests::holding_and_control_cleanup::withcleanup_defers_cleanup_until_suspended_body_finishes ... ok
test tests::holding_and_control_part3::catch_throw_and_return_support_first_pass_nonlocal_control_flow ... ok
test tests::holding_and_control_part3::block_temporarily_rebinds_and_restores_symbol_values ... ok
test tests::holding_and_control_failure_properties::failure_property_accessors_cover_current_subset ... ok
test tests::holding_and_control_part3::compound_expression_evaluates_sequentially_and_returns_the_last_value ... ok
test tests::holding_and_control_part3::configurable_replace_repeated_limit_is_respected ... ok
test tests::holding_and_control_part3::module_generates_fresh_lexical_symbols_and_initializes_sequentially ... ok
test tests::holding_and_control_part3::module_respects_nested_shadowing_and_pattern_locals ... ok
test tests::holding_and_control_part3::module_global_definition_escape_retains_gensym_definition ... ok
test tests::holding_and_control_part3::module_local_assignment_invalidates_after_prior_read ... ok
test tests::holding_and_control_part3::scoping_builtins_require_raw_binding_lists_instead_of_resolved_variables ... ok
test tests::holding_and_control_part3::scoping_builtins_accept_canonical_raw_binding_lists ... ok
test tests::holding_and_control_part3::staged_with_fails_closed_on_a_malformed_middle_declaration_list ... ok
test tests::holding_and_control_part3::switch_matches_against_held_forms_and_residualizes_when_nothing_matches ... ok
test tests::holding_and_control_part3::throw_through_block_restores_dynamic_scope_before_catch_handles_it ... ok
test tests::holding_and_control_part3::with_evaluates_bindings_and_substitutes_lexically ... ok
test tests::holding_and_control_part3::with_respects_pattern_local_names_but_substitutes_pattern_metadata ... ok
test tests::holding_and_control_part3::return_is_consumed_at_definition_application_boundaries ... ok
test tests::holding_and_control_part3::with_supports_rule_style_bindings_and_nested_shadowing ... ok
test tests::holding_and_control_part3::with_zero_and_one_argument_forms_keep_exact_arity_diagnostics ... ok
test tests::holding_and_control_part3::with_supports_staged_declaration_lists ... ok
test tests::debug_tools_resources::time_constrained_charges_input_suspension_to_original_deadline ... ok
test tests::holding_and_control_part4::return_propagates_through_loop_bodies_to_function_boundaries ... ok
test tests::holding_and_control_part5::confirm_info_is_lazy_and_explicit_tags_are_dynamically_caught ... ok
test tests::holding_and_control_part5::iterator_variables_expose_dynamic_ownvalues_without_leaking_assignments ... ok
test tests::holding_and_control_part5::iterator_dynamic_values_interact_with_nested_block_shadowing ... ok
test tests::holding_and_control_part5::iteration_product_scope_resets_between_statements_and_after_errors ... ok
test tests::holding_and_control_part4::break_and_continue_are_handled_by_current_loop_forms ... ok
test tests::holding_and_control_part4::do_supports_count_iterators_and_nested_iterators ... ok
test tests::holding_and_control_abort::abortprotect_assert_and_blockrandom_cover_current_first_pass_subset ... ok
test tests::holding_and_control_part4::while_until_and_for_support_first_pass_loop_control ... ok
test tests::holding_and_control_part5::large_do_count_loop_does_not_recurse_through_the_rust_stack ... ok
test tests::debug_tools_resources::time_constrained_preserves_deadline_through_two_input_suspensions ... ok
test tests::holding_and_control_part5::nested_iteration_constructs_share_the_running_product ... ok
test tests::holding_and_control_part5::multi_iterator_table_charges_the_generated_product ... ok
test tests::holding_and_control_part5::sum_and_product_report_declared_attributes ... ok
test tests::holding_and_control_part5::sum_and_product_leave_unsupported_specs_unevaluated_with_argform ... ok
test tests::holding_and_control_part5::sum_and_product_reuse_concrete_iterator_specs ... ok
test tests::holding_and_control_part5::suspended_nested_loop_retains_and_then_resets_the_running_product ... ok
test tests::holding_and_control_part5::table_memory_constraint_uses_projected_list_size_before_materialization ... ok
test tests::holding_and_control_part6::cleanup_throw::catch_tag_form_keeps_visible_side_effects ... ok
test tests::holding_and_control_part6::exit_reports_a_request_level_code_and_stops_evaluation ... ok
test tests::holding_and_control_part6::cleanup_throw::uncaught_throw_request_messages_follow_nocatch_shape ... ok
test tests::holding_and_control_part8::assert_function_none_restores_normal_assert_control ... ok
test tests::holding_and_control_part8::enclose_handler_can_suspend_and_resume_through_host_input ... ok
test tests::holding_and_control_part6::cleanup_throw::catch_evaluates_symbol_valued_tag_form_before_matching ... ok
test tests::holding_and_control_part6::top_level_break_and_continue_report_nofwd_messages ... ok
test tests::holding_and_control_part6::checkabort_validates_entire_option_lists_before_evaluating_body ... ok
test tests::holding_and_control_part8::confirm_information_can_suspend_and_resume_inside_enclose ... ok
test tests::holding_and_control_part8::confirm_predicate_and_test_can_suspend_and_resume_inside_enclose ... ok
test tests::incidental_arithmetic::arithmetic_builtins_report_declared_attributes ... ok
test tests::holding_and_control_part6::abort_and_checkabort_cover_current_first_pass_subset ... ok
test tests::holding_and_control_part5::multi_iterator_table_precharges_exact_arithmetic_counts ... ok
test tests::holding_and_control_part6::cleanup_throw::withcleanup_runs_cleanup_on_values_throws_and_aborts ... ok
test tests::holding_and_control_success::missingq_and_success_cover_current_subset ... ok
test tests::incidental_debug_tools_dialogs::dialog_catches_return_and_respects_current_option_subset ... ok
test tests::incidental_arithmetic::max_min_handle_exact_extrema_lists_and_registration ... ok
test tests::incidental_debug_tools_dialogs::dialog_uses_current_default_options_from_setoptions ... ok
test tests::incidental_arithmetic::numerator_mirrors_denominator_for_exact_numbers_and_registration ... ok
test tests::incidental_context_and_loader::path_variables_and_file_name_helpers_cover_current_script_environment_subset ... ok
test tests::incidental_debug_tools_dialogs::dialog_without_body_uses_a_current_single_input_line_subset ... ok
test tests::incidental_associations_part2::group_by_rule_specs_transform_group_values ... ok
test tests::incidental_definitions_and_rules_lifecycle::clear_and_clearall_manage_symbol_lifecycle_state ... ok
test tests::incidental_associations_part2::group_by_nested_specs_recurse_through_lists_and_associations ... ok
test tests::incidental_debug_tools_resources::memory_in_use_accounts_for_removed_symbol_identity_state ... ok
test tests::incidental_definitions_and_rules_lifecycle::definition_lhs_evaluates_outer_call_head_before_selecting_definition_owner ... ok
test tests::holding_and_control_part5::iterator_lhs_normalization_preserves_lazy_and_container_lvalue_controls ... ok
test tests::holding_and_control_part5::multi_iterator_control_siblings_charge_iteration_product ... ok
test tests::incidental_definitions_and_rules_lifecycle::definition_lhs_evaluates_call_heads_before_storing_rules ... ok
test tests::incidental_definitions_and_rules_cases::cases_and_deletecases_use_value_only_association_traversal ... ok
test tests::incidental_arithmetic::power_over_times_distributes_numeric_exponents ... ok
test tests::incidental_debug_tools_resources::max_memory_used_tracks_session_memory_proxy_high_water_mark ... ok
test tests::incidental_definitions_and_rules_lifecycle::remove_marks_missing_same_evaluation_symbols_stale ... ok
test tests::incidental_definitions_and_rules_lifecycle::protected_symbols_block_definition_and_clear_writes ... ok
test tests::incidental_definitions_and_rules_lifecycle::definition_lhs_evaluates_non_pattern_constants_before_storing_rules ... ok
test tests::holding_and_control_part5::do_and_table_respect_resource_limits ... ok
test tests::incidental_arithmetic::power_times_combine_like_bases_and_terms ... ok
test tests::functional_ops::iteration::iteration_lists_preserve_wide_lengths_and_callback_counts ... ok
test tests::incidental_definitions_and_rules_lifecycle::remove_reports_protected_symbols_without_clearing_state ... ok
test tests::incidental_definitions_and_rules_lifecycle::set_list_destructuring_assigns_leaf_targets ... ok
test tests::incidental_definitions_and_rules_lifecycle::remove_clears_current_symbol_state_and_declared_stubs ... ok
test tests::incidental_definitions_and_rules_lifecycle::remove_marks_same_evaluation_symbols_stale_and_allows_later_reuse ... ok
test tests::incidental_definitions_and_rules_part2::single_dot_default_pattern_syntax_lowers_to_optional ... ok
test tests::incidental_definitions_and_rules_part2::default_pattern_syntax_lowers_to_optional_patterns ... ok
test tests::incidental_definitions_and_rules_patterns::arithmetic_one_identity_optional_idioms_use_installed_defaults ... ok
test tests::incidental_definitions_and_rules_patterns::nested_compound_optional_idiom_remains_unmatched_at_the_power_boundary ... ok
test tests::incidental_definitions_and_rules_patterns::install_seeds_mutable_arithmetic_head_defaults ... ok
test tests::incidental_definitions_and_rules_part2::matcher_pattern_arguments_resolve_ordinary_subexpressions ... ok
test tests::incidental_definitions_and_rules_patterns::compiled_downvalue_matcher_falls_back_for_condition_patterns ... ok
test tests::incidental_definitions_updates::held_update_builtins_mutate_immediate_symbol_values ... ok
test tests::incidental_definitions_updates::apply_to_rechecks_protection_at_store_time ... ok
test tests::incidental_definitions_updates::held_update_invalid_selector_diagnostics_use_the_evaluated_offender ... ok
test tests::incidental_definitions_updates::held_updates_allow_empty_bulk_part_targets_without_mutation ... ok
test tests::incidental_definitions_updates::held_update_part_selectors_rederive_owner_state_after_callbacks ... ok
test tests::incidental_definitions_updates::held_update_syntax_mutates_module_locals ... ok
test tests::incidental_definitions_and_rules_patterns::orderless_sequence_backtracking_depth_limit_terminates_gracefully ... ok
test tests::incidental_definitions_updates::held_updates_cover_duplicate_bulk_part_targets ... ok
test tests::incidental_debug_tools_resources::byte_count_and_memory_in_use_use_the_current_structural_proxy_subset ... ok
test tests::incidental_definitions_updates::held_updates_respect_protected_symbols ... ok
test tests::incidental_definitions_and_rules_patterns::arithmetic_defaults_and_promoted_matching_survive_snapshot_roundtrip ... ok
test tests::incidental_definitions_updates::held_updates_reject_nested_part_selector_lists_without_mutation ... ok
test tests::incidental_definitions_updates::held_updates_mutate_bulk_exact_part_targets ... ok
test tests::incidental_definitions_updates::held_updates_report_missing_or_invalid_targets_without_writing ... ok
test tests::incidental_holding_and_control_part5::confirm_quiet_catches_and_quiets_matching_messages ... ok
test tests::incidental_holding_and_control_part3::module_options_and_sow_escapes_retain_gensym_definitions ... ok
test tests::incidental_holding_and_control_part3::module_cross_gensym_definition_retains_transitive_dependency ... ok
test tests::incidental_holding_and_control_part2::cycle_detection_allows_same_call_after_state_progress ... ok
test tests::incidental_definitions_updates::held_updates_mutate_single_exact_part_targets ... ok
test tests::incidental_holding_and_control_part5::table_propagates_return_to_function_boundaries ... ok
test tests::incidental_holding_and_control_part3::module_nested_and_error_paths_remain_conservative ... ok
test tests::incidental_holding_and_control_part3::module_result_escape_retains_gensym_definition ... ok
test tests::holding_and_control_failure_edges::failure_symbolic_property_and_predicate_edges_cover_current_subset ... ok
test tests::holding_and_control_part5::large_iteration_products_still_refuse ... ok
test tests::incidental_holding_and_control_part6::failsafe_wrapper_preserves_failure_like_inputs_and_supports_test_forms ... ok
test tests::incidental_holding_and_control_part5::table_supports_count_iterators_and_nested_iterators ... ok
test tests::holding_and_control_part3::module_leaf_definition_gc_reclaims_loop_local_gensyms ... ok
test tests::incidental_holding_and_control_part8::confirm_family_info_can_suspend_and_resume_inside_enclose ... ok
test tests::incidental_lowering_and_context::applies_at_sugar_through_lowering_and_evaluation ... ok
test tests::incidental_nothing::nothing_is_removed_from_direct_list_elements_only ... ok
test tests::incidental_nothing::association_values_keep_nothing_but_list_inputs_drop_it ... ok
test tests::incidental_holding_and_control_part8::generated_confirmation_failures_expose_current_oracle_metadata_slice ... ok
test tests::incidental_options_and_protection::setoptions_updates_system_heads_defaults_for_cases_position_and_depth ... ok
test tests::incidental_predicates::unsame_q_wide_index_detects_late_duplicates ... ok
test tests::incidental_lowering_and_context::functional_operator_sugars_flow_through_lowering_and_first_pass_runtime ... ok
test tests::holding_and_control_part5::ordinary_nested_iteration_results_are_unchanged ... ok
test tests::incidental_print_sink::prints_accumulate_without_sink ... ok
test tests::incidental_print_sink::print_sink_streams_prints_and_leaves_outcome_prints_empty ... ok
test tests::incidental_session_requests_input_resume::definition_lhs_preparation_resumes_a_callable_head_exactly_once ... ok
test tests::incidental_session_requests_debug_tooling_resume::resume_eval_request_with_input_can_finish_constraint_limit_and_fallback ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_can_finish_select_predicate ... ok
test tests::incidental_scoping_unique::unique_supports_list_and_attribute_forms ... ok
test tests::incidental_session_requests_debug_tooling_resume::resume_eval_request_with_input_can_finish_test_report_input_and_sametest ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_preserves_do_iterator_scope_until_completion ... ok
test tests::incidental_session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_while_sequence_helpers ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_finishes_evaluation_data_capture ... ok
test tests::incidental_predicates::predicate_q_family_covers_current_exact_and_structural_subsets ... ok
test tests::incidental_session_requests_input_resume::definition_lhs_preparation_resumes_multiple_arguments_left_to_right_without_replay ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_preserves_prepared_lookup_across_default_and_wrapper ... ok
test tests::incidental_session_requests_transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_deleting_an_association_head ... ok
test tests::incidental_session_requests_matcher_resume::resume_eval_request_with_input_can_finish_conditioned_cases_over_association_values_only ... ok
test tests::incidental_session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_associationmap_over_association_entries ... ok
test tests::incidental_structure_ops_replace::replacepart_bare_association_keys_remain_nonselecting_position_patterns ... ok
test tests::incidental_session_requests_input_resume::resumed_set_rhs_normalizes_iterator_lhs_once_and_restores_clean_scope ... ok
test tests::incidental_session_requests_transforms_resume_associations::resume_eval_request_with_input_preserves_associationmap_list_progress ... ok
test tests::incidental_structure_ops_replace::replacepart_rhs_guards_cover_keyed_association_position_forms ... ok
test tests::incidental_package_tools::dump_save_restore_preserves_registered_core_builtins ... ok
test tests::iterative_traversal_depth::apply_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::incidental_session_requests_transforms_resume_associations::resume_eval_request_with_input_preserves_keyselect_progress ... ok
test tests::iterative_traversal_depth::association_assignment_read_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::incidental_structure_ops_replace::replaceat_supports_sequential_paths_all_and_keyed_positions ... ok
test tests::iterative_traversal_depth::association_map_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::cases_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::construction_reaches_physical_height_20001_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::count_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::d_nested_sin_completes_at_physical_height_4097_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::iterative_traversal_depth::deletecases_active_spine_returns_in_band_resource_error_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::incidental_structure_ops_ordering_sets::sort_with_ordering_function_evaluates_comparator ... ok
test tests::iterative_traversal_depth::deletecases_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::incidental_structure_ops_selectors_part2::part_and_extract_head_access_preserve_symbol_diagnostics ... ok
test tests::iterative_traversal_depth::freeq_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::level_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::incidental_structure_ops_selectors_part2::extract_preserves_nested_key_and_empty_selector_shapes ... ok
test tests::incidental_structure_ops_replace::replacepart_supports_keyed_association_paths ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_preserves_predicate_traversal_state ... ok
test tests::incidental_structure_ops_surface::flatten_reports_current_invalid_boundaries ... ok
test tests::iterative_traversal_depth::map_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::matchq_completes_at_physical_height_20001_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::memberq_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::merge_nested_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::position_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::replace_active_spine_returns_in_band_resource_error_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::incidental_structure_ops_surface::arithmetic_builtins_thread_over_lists ... ok
test tests::iterative_traversal_depth::replace_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::incidental_structure_ops_surface::fullform_evaluates_its_argument_but_preserves_inner_hold ... ok
test tests::iterative_traversal_depth::replaceall_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::replacepart_deep_path_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_preserves_ordering_comparator_state ... ok
test tests::iterative_traversal_depth::rule_rhs_substitution_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::take_multispec_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::tostring_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::with_substitution_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::incidental_structure_ops_surface::hold_form_holds_argument_and_fullform_shows_wrapper ... ok
test tests::incidental_structure_ops_surface::first_last_rest_and_most_support_defaults_and_empty_calls ... ok
test tests::incidental_thread_and_dispatch::dispatch_stage_guards_preserve_value_and_builtin_semantics ... ok
test tests::installs_identity_builtin ... ok
test tests::integer_digits::integer_digit_builtins_report_invalid_base_and_negative_length ... ok
test tests::integer_digits::integer_digit_builtins_report_declared_attributes ... ok
test tests::incidental_structure_ops_surface::head_length_and_depth_cover_atoms_calls_and_compound_heads ... ok
test tests::integer_digits::integer_digit_builtins_thread_and_passthrough_current_scope ... ok
test tests::integer_digits::integer_digit_valid_bases_and_defaults_stay_message_free ... ok
test tests::incidental_associations_part2::group_by_wide_list_nested_and_reducer_phases_preserve_order_and_call_counts ... ok
test tests::iterative_traversal_depth::deletecases_limit_pins_post_order_visit_order ... ok
test tests::iterative_traversal_depth::association_map_certificate_companion_pins_descending_selector ... ok
test tests::integer_digits::integer_digit_builtins_integer_anchors ... ok
test tests::iterative_traversal_depth::lexical_substitution_keeps_nested_with_shadow_local_to_its_subtree ... ok
test tests::iterative_traversal_depth::map_certificate_companion_pins_descending_selector ... ok
test tests::iterative_traversal_depth::lexical_substitution_certificate_companion_pins_three_level_body ... ok
test tests::iterative_traversal_depth::deletecases_certificate_companion_pins_descending_selector ... ok
test tests::incidental_session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_keysort_after_suspending_comparer ... ok
test tests::iterative_traversal_depth::replaceall_certificate_companion_pins_matching_rule ... ok
test tests::iterative_traversal_depth::apply_certificate_companion_pins_descending_selector ... ok
test tests::iterative_traversal_depth::replace_certificate_companion_pins_descending_selector ... ok
test tests::iterative_traversal_depth::lexical_substitution_iterative_tail_keeps_shadow_local_to_nested_with ... ok
test tests::iterative_traversal_depth::rule_rhs_substitution_certificate_companion_pins_three_level_body ... ok
test tests::json::from_json_depth_cap_fails_before_native_stack_depth ... ok
test tests::json::from_json_maps_nested_values_to_exact_rum_expressions ... ok
test tests::json::from_json_rejects_invalid_json_fail_closed ... ok
test tests::json::json_codec_supports_transform_round_trip ... ok
test tests::json::from_json_wide_object_key_index_preserves_all_members ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_preserves_logical_sequence_state ... ok
test tests::json::to_json_rejects_unsupported_values_fail_closed ... ok
test tests::lazy::bounded_realizers_pull_lazy_ranges ... ok
test tests::json::to_json_encodes_supported_subset_compactly_and_pretty ... ok
test tests::holding_and_control_part5::non_retaining_consumers_do_not_propagate_their_product_into_table ... ok
test tests::iterative_traversal_depth::association_assignment_certificate_companion_pins_both_key_selector_spellings ... ok
test tests::lazy::finite_lazy_transformer_compositions_realize_to_lists ... ok
test tests::lazy::huge_lazy_combinatorial_spaces_fail_closed_at_iteration_ceiling ... ok
test tests::lazy::lazy_combinatorial_builders_reject_invalid_inputs_without_partial_values ... ok
test tests::lazy::lazy_nest_and_fold_nested_errors_propagate ... ok
test tests::lazy::lazy_fold_list_emits_init_then_running_accumulators ... ok
test tests::lazy::lazy_drop_and_rest_keep_the_tail_lazy ... ok
test tests::lazy::lazy_pull_cancellation_does_not_stale_the_same_value_for_a_fresh_request ... ok
test tests::lazy::lazy_permutations_large_take_is_lazy_cheap_and_opaque ... ok
test tests::lazy::lazy_range_rejects_invalid_arguments_without_partial_values ... ok
test tests::lazy::lazy_list_is_opaque_structural_and_inert ... ok
test tests::lazy::lazy_select_filters_under_bounded_realization ... ok
test tests::lazy::lazy_range_builds_canonical_infinite_descriptors ... ok
test tests::lazy::lazy_nest_list_builds_an_opaque_doubling_stream ... ok
test tests::lazy::lazy_subsets_large_take_is_lazy_and_opaque ... ok
test tests::lazy::lazy_permutations_matches_eager_canonical_order_and_edges ... ok
test tests::lazy::lazy_table_holds_its_body_and_binds_the_iterator_lexically ... ok
test tests::lazy::lazy_table_result_is_opaque_and_bounded_take_is_lazy ... ok
test tests::lazy::lazy_subsets_matches_eager_order_and_edges ... ok
test tests::lazy::normal_on_infinite_lazy_nest_list_fails_closed ... ok
test tests::lazy::lazy_table_realizes_arithmetic_iterator_forms ... ok
test tests::lazy::lazy_table_rejects_invalid_specs_without_partial_values ... ok
test tests::lazy::new_lazy_builders_reject_invalid_arity_and_fold_input ... ok
test tests::lazy::never_matching_lazy_select_fails_closed_at_source_pull_ceiling ... ok
test tests::lazy::lazy_tuples_matches_eager_order_and_edges ... ok
test tests::lazy::lazy_transformers_reject_invalid_inputs_without_partial_values ... ok
test tests::lazy::lazy_transformers_build_canonical_opaque_descriptors ... ok
test tests::lazy::nested_lazy_map_and_select_eval_errors_propagate ... ok
test tests::lazy::normal_on_infinite_lazy_range_fails_closed_at_iteration_ceiling ... ok
test tests::lazy::positive_integer_part_redispatches_through_lazy_realizers ... ok
test tests::lazy::other_lazy_part_specs_follow_the_existing_part_path ... ok
test tests::map_indexed_association_depth::map_indexed_association_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::map_indexed_association_depth::map_indexed_association_heads_refuses_in_band_rather_than_terminating ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::logic::comparison::comparison_operator_forms_apply_single_arguments_on_the_left ... ok
test tests::merge_native_stack_guard::merge_zero_native_stack_boundary_fails_in_band_and_session_recovers ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::logic::inequality::mixed_comparison_chain_does_not_short_circuit_operand_evaluation ... ok
test tests::logic::comparison::equal_and_unequal_leave_mixed_symbolic_chains_intact ... ok
test tests::logic::inequality::mixed_comparison_chain_evaluates_middle_operand_once ... ok
test tests::logic::inequality::comparison_parentheses_bound_chain_full_forms ... ok
test tests::logic::comparison::unequal_wide_value_index_detects_late_duplicates ... ok
test tests::logic::inequality::inequality_binary_symbolic_and_attribute_contracts_are_explicit ... ok
test tests::logic::inequality::uniform_less_chain_keeps_its_nary_head ... ok
test tests::logic::inequality::parenthesized_and_unparenthesized_equality_results_remain_distinct ... ok
test tests::logic::inequality::uniform_less_chain_evaluates_every_operand_once ... ok
test tests::ownership_graph::accumulator_coverage::fresh_names_and_additional_self_edges_are_checked ... ok
test tests::logic::session_requests::truth_aggregates_short_circuit_and_resume_input_requests ... ok
test tests::ownership_graph::fixtures::coverage_is_not_inferred_from_test_like_filenames ... ok
test tests::logic::inequality::mixed_comparison_chains_have_alternating_inequality_full_forms ... ok
test tests::ownership_graph::fixtures::fresh_names_cross_module_pair_and_recursion_after_comma ... ok
test tests::ownership_graph::fixtures::inline_module_path_attribute_and_super_imports_resolve ... ok
test tests::ownership_graph::fixtures::named_generic_public_inputs_are_opaque_but_borrowed_inputs_are_not ... ok
test tests::ownership_graph::fixtures::opaque_constructs_are_located_and_inventory_removal_fails ... ok
test tests::ownership_graph::fixtures::owning_ports_and_nonowning_storage_do_not_mix_instantiations ... ok
test tests::logic::part2::order_comparison_operator_forms_report_application_arity ... ok
test tests::ownership_graph::fixtures::resolution_uses_declarations_not_container_spellings ... ok
test tests::logic::part2::equal_and_unequal_handle_boolean_pairs_without_overreaching ... ok
test tests::ownership_graph::fixtures::unknown_configuration_cannot_invent_an_unconditional_cycle ... ok
test tests::ownership_graph::fixtures::unmodeled_lexical_modules_and_async_captures_are_explicit_unknowns ... ok
test tests::logic::part2::equal_reduces_adjacent_mixed_comparable_pairs_only ... ok
test tests::ownership_graph::host_payload_ledger::ownership::attribution_resolves_alias_scopes_and_excludes_nonowning_paths ... ok
test tests::logic::comparison::and_or_short_circuit_side_effects_and_compare_exact_values ... ok
test tests::logic::part2::equality_operator_forms_accept_multi_argument_applications ... ok
test tests::lazy::snapshot_roundtrip_rederives_lazy_combinatorial_descriptors ... ok
test tests::lazy::snapshot_roundtrip_rederives_lazy_table_descriptor ... ok
test tests::logic::part2::unequal_detects_repeated_known_prefix_values_before_unknown_tail ... ok
test tests::logic::session_requests::logic_short_circuit_can_avoid_or_resume_input_requests ... ok
test tests::map_indexed_association_depth::map_indexed_association_three_level_companion_pins_positions_and_levelspec ... ok
test tests::logic::comparison::wide_numeric_order_chains_reduce_in_one_pass ... ok
test tests::logic::truth::wide_xor_and_equivalent_indexes_preserve_parity_and_uniqueness ... ok
test tests::lazy::snapshot_roundtrip_rederives_composed_lazy_descriptor ... ok
test tests::number_theory::next_prime_follows_signed_prime_order ... ok
test tests::number_theory::prime_reports_positive_integer_message ... ok
test tests::number_theory::factorization_builtins_report_declared_attributes ... ok
test tests::number_theory::factorization_respects_integer_work_limit ... ok
test tests::number_theory::prime_and_prime_pi_respect_integer_work_limit ... ok
test tests::lazy::snapshot_roundtrip_rederives_lazy_nest_and_fold_descriptors ... ok
test tests::number_theory::factorization_builtins_thread_and_passthrough_like_current_scope ... ok
test tests::incidental_definitions_updates::held_update_part_selectors_run_left_to_right_after_binary_rhs_preparation ... ok
test tests::number_theory::factorization_builtins_integer_anchors ... ok
test tests::logic::truth::and_or_not_respect_current_short_circuit_and_identity_semantics ... ok
test tests::number_theory::large_semiprime_factorization_uses_pollard_rho_path ... ok
test tests::polynomials::coefficient_extracts_exact_monomial_powers_without_requiring_manual_expand ... ok
test tests::polynomials::coefficient_family_fails_closed_on_leaked_laurent_target_powers ... ok
test tests::polynomials::coefficient_family_keeps_current_pattern_bearing_boundary_for_now ... ok
test tests::polynomials::expand_distributes_products_and_positive_integer_powers ... ok
test tests::polynomials::coefficient_list_builds_rectangular_grids_in_the_requested_variable_order ... ok
test tests::polynomials::coefficient_family_keeps_non_laurent_residues_and_polynomials ... ok
test tests::polynomials::expand_preserves_non_polynomial_or_pattern_bearing_regions ... ok
test tests::polynomials::expand_supports_rational_coefficients_and_indexed_indeterminates ... ok
test tests::polynomials::factor_keeps_current_pattern_bearing_boundary_for_now ... ok
test tests::polynomials::factor_extracts_content_common_monomials_and_linear_factors_over_q ... ok
test tests::polynomials::factor_handles_symmetric_multivariate_cubic_example ... ok
test tests::polynomials::together_keeps_current_pattern_bearing_boundary_for_now ... ok
test tests::polynomials::polynomial_predicates_cover_current_expression_subsets ... ok
test tests::polynomials::together_combines_rational_function_terms_over_common_polynomial_denominators ... ok
test tests::polynomials::together_now_cancels_common_polynomial_factors_after_combining ... ok
test tests::predicates::collection_predicates_accept_same_test_options ... ok
test tests::polynomials::together_preserves_reduced_canonical_rational_function_forms ... ok
test tests::predicates_part2::negative_power_order_compares_bases_before_exponents ... ok
test tests::predicates::predicate_operator_forms_are_inert_and_apply_to_data ... ok
test tests::predicates::truth_aggregates_and_coprime_q_cover_current_subsets ... ok
test tests::random::block_random_does_not_restore_random_password_entropy ... ok
test tests::predicates_part2::order_places_negative_powers_before_their_bases ... ok
test tests::polynomials::wide_monomial_products_normalize_factors_once ... ok
test tests::predicates::ordering_predicates_accept_custom_comparators ... ok
test tests::random::character_class_constants_are_protected ... ok
test tests::predicates::list_relation_and_shape_predicates_cover_current_exact_subsets ... ok
test tests::random::character_class_constants_have_pinned_values_and_compose ... ok
test tests::random::random_choice_and_sample_cover_current_list_contracts ... ok
test tests::polynomials::wide_factor_collections_preserve_polynomial_shapes ... ok
test tests::random::random_family_heads_report_declared_attributes ... ok
test tests::random::block_random_restores_the_prng_stream ... ok
test tests::random::block_random_restores_the_random_string_stream ... ok
test tests::random::random_integer_respects_large_exact_ranges ... ok
test tests::random::random_password_combined_policy_uses_whole_password_rejection ... ok
test tests::random::random_password_does_not_advance_the_prng_stream ... ok
test tests::random::random_password_exclude_policy_rejects_empty_substrings ... ok
test tests::random::random_password_fixed_entropy_uses_rejection_sampling ... ok
test tests::random::random_password_no_adjacent_repeats_policy_resamples ... ok
test tests::random::random_password_policy_cap_exhaustion_fails_closed ... ok
test tests::random::random_integer_default_seed_sequence_is_pinned ... ok
test tests::random::random_password_policy_rejects_non_positive_length ... ok
test tests::random::random_integer_and_choice_accept_nested_output_dimensions ... ok
test tests::random::random_password_respects_custom_alphabet_membership ... ok
test tests::random::random_string_alphabet_duplicates_weight_draws ... ok
test tests::random::random_string_draws_from_unicode_code_points ... ok
test tests::random::random_string_default_seed_sequence_is_pinned ... ok
test tests::random::random_password_exclude_policy_resamples_and_can_ignore_case ... ok
test tests::random::random_string_invalid_count_or_empty_alphabet_stays_inert ... ok
test tests::polynomials::wide_order_preserving_polynomial_transforms_keep_canonical_shapes ... ok
test tests::random::random_password_policy_options_are_discoverable_without_changing_draws ... ok
test tests::random::random_uuid_and_uuid_q_cover_format_boundaries ... ok
test tests::random::random_password_require_policy_resamples_until_groups_are_present ... ok
test tests::random::seed_random_integer_sequence_is_pinned ... ok
test tests::random::seed_random_is_deterministic_only_and_reseed_random_has_exact_arity ... ok
test tests::scan_and_keypath_set_native_stack_guard::keypath_set_zero_native_stack_boundary_preserves_both_lvalue_roots ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::scan_and_keypath_set_native_stack_guard::scan_zero_native_stack_boundary_covers_each_edge_without_callbacks ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::random::whitespace_character_constant_has_pinned_value_and_composes ... ok
test tests::random::random_choice_weighted_forms_use_exact_weights_and_fail_closed ... ok
test tests::random::random_string_seed_random_reproducibility_is_pinned ... ok
test tests::random::random_text_composes_with_character_class_constants ... ok
test tests::random::random_password_require_policy_rejects_malformed_groups ... ok
test tests::resumable_traversal_regression::large_inner_callback_loop_does_not_recurse_through_the_rust_stack ... ok
test tests::resumable_traversal_regression::large_catenate_list_walk_does_not_recurse_through_the_rust_stack ... ok
test tests::schema::conform_schema_emits_schema_messages_and_fails_closed ... ok
test tests::schema::conform_schema_validates_nested_config_and_fills_defaults ... ok
test tests::schema::list_schema_covers_empty_and_nested_associations ... ok
test tests::schema::conform_schema_wide_association_preserves_value_order ... ok
test tests::schema::optional_defaults_and_extra_keys_follow_schema_contract ... ok
test tests::schema::pattern_leaf_breadth_and_held_schema_semantics ... ok
test tests::polynomials::wide_factor_common_monomial_and_exact_division_preserve_expansion ... ok
test tests::random::reseed_random_uses_entropy_source_in_unrestricted_session ... ok
test tests::resumable_traversal_regression::delayed_association_aggregates_evaluate_each_value_once ... ok
test tests::polynomials::wide_expand_batches_sum_and_product_term_normalization ... ok
test tests::schema::schema_errors_returns_exact_nested_invalid_config_errors ... ok
test tests::resumable_traversal_regression::resumable_array_traversals_visit_each_generated_item_once ... ok
test tests::schema::schema_validation_composes_with_codecs_and_error_data_serializes ... ok
test tests::schema_operator_forms::direct_schema_value_evaluation_resumes_after_input ... ok
test tests::schema_operator_forms::direct_schema_calls_keep_attributes_and_allow_extra_keys_option ... ok
test tests::schema::schema_errors_are_capped_with_summary ... ok
test tests::schema_operator_forms::schema_captures_are_inert_and_wrong_application_arity_reports_argx ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_fixedpoint_callback_loop ... ok
test tests::resumable_traversal_regression::resumable_traversals_visit_each_completed_item_once ... ok
test tests::schema_operator_forms::schema_operator_value_side_effect_fires_once ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_fixedpointlist_callback_loop ... ok
test tests::resumable_traversal_regression::resumable_fold_traversals_visit_each_item_once ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_nest_callback_loop ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_nestlist_callback_loop ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_nestwhile_test_and_step ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_nestwhilelist_test_and_step ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_accumulate_and_blockmap_callbacks ... ok
test tests::schema_operator_forms::schema_capture_is_quiet_and_defers_schema_side_effects_until_validation ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_delete_duplicates_by_key_loop ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_fold_callback_loop ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_foldlist_callback_loop ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_gatherby_function_list_key_loop ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_foldpair_operator_and_projector ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_foldwhile_step_and_test ... ok
test tests::resumable_traversal_regression::large_resumable_fold_loops_do_not_recurse_through_the_rust_stack ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_sequencefoldlist_callback_loop ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_sortby_key_loop ... ok
test tests::session_requests_functional_targeted_resume::resume_eval_request_with_input_can_finish_mapat_callback_loop ... ok
test tests::session_requests_functional_targeted_resume::resume_eval_request_with_input_can_finish_subsetmap_callback_and_final_eval ... ok
test tests::schema_operator_forms::schema_operator_forms_cover_success_failure_and_error_lists ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_custom_equivalence_loops ... ok
test tests::session_requests_functional_tensor_string_resume::resume_eval_request_with_input_can_finish_array_callback_helpers ... ok
test tests::session_requests_functional_tensor_string_resume::resume_eval_request_with_input_can_finish_inner_term_and_reducer_callbacks ... ok
test tests::session_requests_functional_tensor_string_resume::resume_eval_request_with_input_preserves_array_and_outer_progress ... ok
test tests::session_requests_functional_tensor_string_resume::resume_eval_request_with_input_preserves_completed_inner_terms ... ok
test tests::session_requests_predicate_collections_resume::resume_eval_request_with_input_preserves_duplicate_free_pair_progress ... ok
test tests::session_requests_predicate_collections_resume::resume_eval_request_with_input_preserves_same_test_matrix_progress ... ok
test tests::session_requests_transforms_resume_associations::association_part_mutation_resumes_one_selector_without_replay_and_refetches_owner ... ok
test tests::preflight_memory::projected_preflight_builtins_refuse_before_allocation_and_admit_realistic_inputs ... ok
test tests::session_requests_transforms_resume_associations::associateto_requires_an_immediate_symbol_value_instead_of_suspending_delayed_values ... ok
test tests::session_requests_transforms_resume_associations::association_part_mutation_keeps_opaque_completion_across_two_suspensions ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_catenate_after_suspending_delayed_association_value ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_counts_after_suspending_delayed_association_value ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_deletemissing_eager_delayed_predicate ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_countsby_after_multiple_key_suspensions ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_groupby_after_suspending_delayed_association_value ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_groupby_after_suspending_value_transform ... ok
test tests::stress_corpus::peano_arithmetic_heavy_multiplication ... ignored
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_groupby_list_key_and_value_loops ... ok
test tests::functional_ops::targeted_map::map_at_evaluates_each_target_once_at_scale ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_keygroupby_key_and_reducer_loops ... ok
test tests::stress_corpus::rule_based_differentiator_handles_heavy_generated_polynomial ... ignored
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_merge_after_suspending_delayed_association_value ... ok
test tests::stress_corpus::tokenize_with_ini_grammar_handles_heavy_generated_config ... ignored
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_keyunion_after_suspending_fill_function ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_nested_deletemissing_predicate ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_nested_groupby_after_multiple_key_suspensions ... ok
test tests::session_requests_transforms_resume_associations::held_part_update_resumes_two_selectors_without_replay_and_refetches_owner ... ok
test tests::session_requests_functional_sequence_resume::numeric_suspend_resume_liveness_probes_return_exact_values ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_reports_catenate_invalid_value_after_suspension ... ok
test tests::stress_corpus::guard_adjacent_user_recursion_below_limit_succeeds ... ok
test tests::stress_corpus::guard_adjacent_user_recursion_above_limit_terminates_and_session_recovers ... ok
test tests::stress_corpus::reduced_native_stack_limit_returns_stack_limit_and_session_recovers ... ok
test tests::session_requests_set_same_test_resume::custom_same_test_set_operations_preserve_progress_across_input ... ok
test tests::strings::advanced_patterns::bounded_repetition_and_except_extend_string_patterns ... ok
test tests::strings::advanced_patterns::character_class_pattern_atoms_extend_the_current_string_pattern_subset ... ok
test tests::strings::advanced_patterns::regular_expression_patterns_extend_the_current_string_pattern_subset ... ok
test tests::incidental_definitions_updates::held_updates_evaluate_rebound_part_selectors_for_all_eleven_heads ... ok
test tests::stress_corpus::peano_arithmetic_multiplies_through_pure_rewriting ... ok
test tests::strings::advanced_patterns::string_pattern_consumers_reject_malformed_repetition_bounds ... ok
test tests::strings::advanced_patterns::repeated_anchors_and_shortest_longest_extend_string_patterns ... ok
test tests::strings::advanced_patterns::symbolic_blank_patterns_extend_the_current_string_pattern_subset ... ok
test tests::strings::encoding::hash_byte_identity_and_formats_extend_cl07 ... ok
test tests::strings::incidental_runtime::failure_messages::confirm_failure_wrapping_resumes_delayed_source_failure_message_template ... ok
test tests::strings::incidental_runtime::failure_messages::failure_and_enclose_confirm_family_cover_first_pass_error_flow ... ok
test tests::strings::incidental_runtime::association_key_ops::key_take_and_drop_preserve_order_rules_at_scan_and_indexed_widths ... ok
test tests::holding_and_control_part5::iterator_spec_matrix_covers_counts_lists_ranges_steps_and_rejections ... ok
test tests::strings::encoding::byte_arrays_transport_and_nearest_extend_cl07 ... ok
test tests::strings::incidental_runtime::failure_messages::failure_message_template_properties_cover_current_subset ... ok
test tests::stress_corpus::rule_based_differentiator_handles_generated_polynomial ... ok
test tests::strings::incidental_runtime::functional_ops::operator_wrappers_cover_current_curried_subset ... ok
test tests::strings::incidental_runtime::functional_ops::accumulate_blockmap_takewhile_and_fold_family_cover_current_subset ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_groupby_nested_and_reducer_group_loops ... ok
test tests::strings::incidental_runtime::logical_syntax::logical_and_comparison_syntax_lowers_to_existing_builtins ... ok
test tests::strings::incidental_runtime::association_aggregation::merge_catenate_counts_groupby_and_positionindex_cover_first_pass_subset ... ok
test tests::stress_corpus::list_pipeline_processes_large_range_with_exact_result ... ok
test tests::strings::incidental_runtime::options::delayed_system_default_options_feed_current_builtin_consumers ... ok
test tests::strings::incidental_runtime::failure_messages::message_list_reports_last_top_level_messages ... ok
test tests::strings::incidental_runtime::association_key_ops::key_predicates_take_drop_map_and_sort_cover_first_pass_subset ... ok
test tests::strings::incidental_runtime::options::setoptions_updates_current_system_default_subset_behavior ... ok
test tests::strings::incidental_runtime::options::system_default_options_cover_current_first_pass_subset ... ok
test tests::strings::incidental_runtime::session_requests::resume_eval_request_with_input_can_finish_delayed_string_rules ... ok
test tests::strings::incidental_runtime::session_requests::resume_eval_request_with_input_can_finish_keysortby_after_each_sort_key_suspends ... ok
test tests::strings::incidental_runtime::session_requests::guarded_string_rules_resume_without_replaying_candidates_or_output ... ok
test tests::strings::incidental_runtime::session_requests::resume_eval_request_with_input_can_finish_keysortby_after_suspending_sort_key ... ok
test tests::strings::incidental_runtime::session_requests::resume_eval_request_with_input_can_finish_replacepart_pattern_bindings_in_delayed_rhs ... ok
test tests::strings::incidental_runtime::session_requests::resume_eval_request_with_input_can_finish_string_apply_list_callbacks ... ok
test tests::strings::incidental_runtime::session_requests::string_replace_and_cases_resume_mid_walk_with_accumulated_output ... ok
test tests::strings::incidental_runtime::session_requests::string_apply_resumes_mid_walk_with_accumulated_output ... ok
test tests::strings::incidental_runtime::structure_ops::delete_duplicates_by_keeps_first_item_for_each_evaluated_key ... ok
test tests::strings::parts::string_apply_delete_and_count_options_cover_the_next_wave ... ok
test tests::strings::incidental_runtime::syntax_information::syntax_information_includes_current_system_default_subset ... ok
test tests::predicates::contains_exactly_custom_same_test_uses_polynomial_matching ... ok
test tests::strings::parts::string_take_drop_and_part_support_basic_integer_and_range_specs ... ok
test tests::strings::parts::string_builtins_preserve_current_boundary_for_non_string_or_invalid_specs ... ok
test tests::strings::scripting::scripting_pipeline_combines_args_url_and_aligned_output ... ok
test tests::strings::incidental_runtime::package_contexts::dump_save_context_strings_roundtrip_public_and_private_contexts ... ok
test tests::strings::scripting::html_shell_path_url_and_args_utilities_cover_boundaries ... ok
test tests::strings::scripting::string_form_is_eager_and_reports_missing_slots ... ok
test tests::strings::scripting::text_shaping_utilities_cover_wrap_indent_dedent_tabs_and_tables ... ok
test tests::strings::search_replace::delayed_string_rules_evaluate_at_application_time ... ok
test tests::strings::search_replace::guarded_string_cases_pin_overlapping_adjacent_and_empty_matches ... ok
test tests::strings::search_replace::guarded_string_rule_lists_try_later_rules_at_the_same_position ... ok
test tests::strings::search_replace::guarded_string_rules_preserve_character_positions_and_candidate_counts ... ok
test tests::strings::search_replace::string_cases_and_position_honor_boolean_overlaps_options ... ok
test tests::strings::search_replace::non_boolean_string_guards_decline_and_delayed_bodies_wait_for_acceptance ... ok
test tests::strings::search_replace::string_count_matchq_and_freeq_extend_current_pattern_subset ... ok
test tests::strings::search_replace::string_position_reports_overlapping_one_based_ranges ... ok
test tests::strings::search_replace::rhs_guard_whole_line_regression_covers_every_rule_lane ... ok
test tests::strings::search_replace::string_expression_and_alternatives_extend_the_current_string_subset ... ok
test tests::strings::search_replace::rhs_condition_guards_control_string_replace_for_both_rule_heads ... ok
test tests::strings::search_replace::string_repeat_and_exact_substring_predicates_cover_first_tier_two_wave ... ok
test tests::strings::surface::input_like_renderer_uses_part_and_map_family_syntax ... ok
test tests::strings::search_replace::string_rule_guards_and_bodies_share_the_enclosing_option_scope ... ok
test tests::strings::search_replace::string_split_does_not_accept_rule_delimiter_specs ... ok
test tests::stress_corpus::tokenize_with_ini_grammar_handles_generated_100kb_config ... ok
test tests::strings::search_replace::string_search_and_rewrite_builtins_support_wl_operator_forms ... ok
test tests::strings::search_replace::string_split_trim_replace_and_cases_cover_common_text_processing_flows ... ok
test tests::strings::search_replace::string_trim_literal_list_handles_a_wide_input ... ok
test tests::strings::search_replace::string_trim_literal_lists_preserve_exact_boundary_semantics ... ok
test tests::strings::search_replace::string_trim_literal_lists_preserve_unicode_threading_and_general_fallback ... ok
test tests::strings::search_replace::symbolic_string_patterns_bind_named_captures_in_rules ... ok
test tests::strings::surface::case_conversion_threads_nested_and_mixed_lists_like_listable ... ok
test tests::strings::surface::protected_string_pattern_symbols_reject_definitions_without_changing_matching ... ok
test tests::strings::similarity::alphabet_distance_alignment_and_diff_extend_cl07 ... ok
test tests::strings::search_replace::wide_literal_search_preserves_predicate_overlap_and_collection_semantics ... ok
test tests::strings::surface::characters_character_range_and_tostring_cover_basic_text_flows ... ok
test tests::strings::surface::string_character_classes_and_case_mapping_extend_the_string_subset ... ok
test tests::strings::surface::string_construction_and_measurement_use_code_point_semantics ... ok
test tests::strings::surface::string_formatting_padding_and_riffle_cover_the_next_surface_wave ... ok
test tests::strings::surface::string_helper_builtins_cover_surface_edges ... ok
test tests::strings::scripting::numeric_text_padding_respects_generated_element_limit_boundaries ... ok
test tests::strings::surface::string_pattern_symbols_are_protected ... ok
test tests::strings::surface::syntaxq_textstring_charactercounts_and_removediacritics_extend_cl07 ... ok
test tests::strings::surface::templates_text_words_codes_and_to_expression_extend_cl07 ... ok
test tests::strings::text_odds::decimal_string_invalid_forms_remain_inert ... ok
test tests::strings::text_odds::decimal_string_formats_exact_numbers_with_fixed_fraction_digits ... ok
test tests::session_requests_functional_sequence_resume::every_custom_sort_comparison_resumes_with_the_uninterrupted_trace ... ok
test tests::strings::text_odds::decimal_string_respects_generated_element_limit_boundaries ... ok
test tests::strings::text_odds::string_lines_splits_lf_and_crlf_with_trailing_newline_rule ... ok
test tests::strings::text_odds::string_pad_right_mirrors_left_padding_subset ... ok
test tests::strings::text_odds::string_text_odds_report_invalid_arities ... ok
test tests::strings::tokenize::rum_tokens_preserves_trivia_and_operator_coverage ... ok
test tests::strings::tokenize::rum_tokens_matches_pre_change_offset_goldens ... ok
test tests::strings::tokenize::rum_tokens_reports_total_ascending_spans_across_inputs ... ok
test tests::strings::tokenize::rum_tokens_recovers_malformed_input_with_error_spans ... ok
test tests::strings::tokenize::rum_tokens_reports_multibyte_code_point_positions ... ok
test tests::strings::tokenize::tokenize_with_fails_closed_for_invalid_inputs ... ok
test tests::strings::tokenize::tokenize_with_ini_grammar_reports_exact_spans ... ok
test tests::strings::tokenize::tokenize_with_skips_zero_width_rules_and_coalesces_text ... ok
test tests::strings::tokenize::tokenize_with_uses_longest_match_then_earliest_rule ... ok
test tests::strings::tokenize::tokenize_with_reports_total_ascending_spans_across_inputs ... ok
test tests::strings::tokenize::tokenize_with_spans_round_trip_through_string_take ... ok
test tests::structure_ops::collection_stats::collection_operator_forms_rewrite_to_data_first_calls ... ok
test tests::structure_ops::collection_stats::by_extrema_take_and_reverse_sort_cover_ties_and_boundaries ... ok
test tests::strings::surface::string_generators_respect_resource_limits ... ok
test tests::structure_ops::collection_stats::collection_stats_support_normal_expression_heads_where_structure_ops_do ... ok
test tests::structure_ops::collection_stats::collection_stats_report_invalid_arity_and_empty_selection ... ok
test tests::structure_ops::collection_stats::gather_by_groups_in_first_key_order ... ok
test tests::structure_ops::collection_stats::max_by_and_min_by_accept_top_counts ... ok
test tests::structure_ops::collection_stats::custom_reverse_sort_covers_boundaries_and_reverses_stable_tagged_ties ... ok
test tests::structure_ops::collection_stats::maximal_by_and_minimal_by_return_all_extrema_in_input_order ... ok
test tests::structure_ops::collection_stats::maximal_by_key_evaluation_supports_user_downvalues ... ok
test tests::structure_ops::collection_stats::tally_and_riffle_cover_first_occurrence_and_boundaries ... ok
test tests::structure_ops::collection_stats::tally_and_take_largest_by_support_top_counts_idiom ... ok
test tests::strings::text_odds::string_padding_respects_generated_element_limit_boundaries ... ok
test tests::structure_ops::dedup::delete_duplicates_accepts_custom_same_test ... ok
test tests::structure_ops::dedup::delete_duplicates_keeps_first_structural_occurrence ... ok
test tests::structure_ops::dedup::delete_duplicates_reports_current_invalid_boundaries ... ok
test tests::structure_ops::join::join_concatenates_associations_with_later_keys_winning ... ok
test tests::structure_ops::join::join_concatenates_top_level_same_head_expressions ... ok
test tests::structure_ops::list_lookup::binary_search_uses_canonical_order_and_not_found_missing ... ok
test tests::structure_ops::join::join_reports_head_mismatches_and_leaves_atomic_inputs_inert ... ok
test tests::structure_ops::join::join_reports_current_level_argument_boundaries ... ok
test tests::structure_ops::list_lookup::duplicates_returns_repeated_values_once_in_first_appearance_order ... ok
test tests::structure_ops::list_plumbing::differences_covers_successive_and_ordered_plain_list_subset ... ok
test tests::structure_ops::list_lookup::select_positions_returns_position_paths_matching_position ... ok
test tests::structure_ops::join::join_supports_current_level_argument_subset ... ok
test tests::structure_ops::list_plumbing::combinatorial_list_ops_symbolically_passthrough_invalid_boundaries ... ok
test tests::structure_ops::list_plumbing::constant_array_covers_scalar_and_multidimensional_integer_shapes ... ok
test tests::structure_ops::list_plumbing::list_plumbing_attributes_match_wk_anchors ... ok
test tests::structure_ops::list_plumbing::pad_left_covers_padding_truncation_cyclic_and_normal_heads ... ok
test tests::structure_ops::ordering_sets::qualified_symbol_name_order_matches_the_allocating_text_order_exactly ... ok
test tests::structure_ops::list_plumbing::pad_right_covers_padding_truncation_cyclic_and_normal_heads ... ok
test tests::structure_ops::list_plumbing::partition_covers_wk_anchored_block_and_offset_forms ... ok
test tests::structure_ops::list_plumbing::partition_operator_form_rewrites_to_data_first_call ... ok
test tests::structure_ops::list_plumbing::permutations_are_distinct_and_canonically_lexicographic ... ok
test tests::structure_ops::list_plumbing::permutations_fail_before_materializing_a_huge_result ... ok
test tests::structure_ops::list_plumbing::tuples_covers_wk_anchored_repetition_and_cartesian_forms ... ok
test tests::structure_ops::list_plumbing::subsets_covers_wk_anchored_size_specs_and_ordering ... ok
test tests::structure_ops::ordering_sets::intersection_covers_wk_anchored_same_head_subset ... ok
test tests::structure_ops::ordering_sets::custom_sort_and_sort_by_cover_boundaries_and_stable_tagged_ties ... ok
test tests::structure_ops::ordering_sets::every_default_structural_order_consumer_keeps_qualified_symbol_order ... ok
test tests::structure_ops::ordering_sets::ordering_and_set_helpers_report_current_invalid_boundaries ... ok
test tests::structure_ops::ordering_sets::qualified_symbol_name_ties_ignore_context_elision_and_generated_identity_stably ... ok
test tests::structure_ops::ordering_sets::separate_canonical_and_numeric_order_consumers_keep_their_base_outputs ... ok
test tests::structure_ops::ordering_sets::set_operation_attributes_match_wk_anchors ... ok
test tests::structure_ops::ordering_sets::set_operations_accept_same_test_option ... ok
test tests::structure_ops::ordering_sets::sort_by_multi_key_evaluation_supports_user_downvalues ... ok
test tests::structure_ops::ordering_sets::sort_and_sortby_cover_current_top_level_subset ... ok
test tests::structure_ops::ordering_sets::symbol_sort_order_and_mixed_atom_fallback_match_the_pinned_lists ... ok
test tests::structure_ops::ordering_sets::sort_by_supports_stable_multi_key_specs_and_tuple_ordering ... ok
test tests::structure_ops::ordering_sets::union_and_complement_cover_current_same_head_subset ... ok
test tests::structure_ops::reordering::reverse_covers_current_positive_level_subset ... ok
test tests::structure_ops::reordering::reverse_reports_current_invalid_boundaries ... ok
test tests::structure_ops::selectors::part_paths::part_supports_exact_indices_nested_paths_and_head_access ... ok
test tests::structure_ops::reordering::transpose_covers_current_plain_list_matrix_subset ... ok
test tests::structure_ops::reordering::transpose_reports_current_invalid_boundaries ... ok
test tests::structure_ops::selectors::wide_batched_key_part_preserves_order_duplicates_and_missing_values ... ok
test tests::structure_ops::selectors::extract_consumes_exact_paths_and_position_output ... ok
test tests::structure_ops::selectors::invalid_part_and_extract_specs_report_selector_messages ... ok
test tests::structure_ops::sequence::range_constructs_plain_integer_lists_for_current_subset ... ok
test tests::structure_ops::selectors::keyed_part_and_extract_cover_association_paths ... ok
test tests::structure_ops::sequence::range_respects_generated_element_limit_before_allocating ... ok
test tests::structure_ops::sequence::sequence_operator_forms_rewrite_to_data_first_calls ... ok
test tests::structure_ops::sequence::take_drop_accept_upto_sequence_specs ... ok
test tests::structure_ops::sequence::take_drop_append_and_prepend_report_current_invalid_boundaries ... ok
test tests::structure_ops::sequence::take_existing_sequence_specs_are_unchanged ... ok
test tests::structure_ops::sequence::take_multispec_failure_below_first_spec_preserves_message_channel ... ok
test tests::structure_ops::sequence::take_drop_insert_delete_and_take_list_cover_current_structural_subset ... ok
test tests::structure_ops::sequence::take_multispec_shallow_companion_pins_three_level_fanout ... ok
test tests::structure_ops::sequence::take_upto_reports_atom_and_malformed_spec_messages ... ok
test tests::structure_ops::surface::append_prepend_symbol_and_construct_cover_current_surface_subset ... ok
test tests::structure_ops::surface::first_last_rest_and_most_emit_expected_messages_for_invalid_inputs ... ok
test tests::structure_ops::surface::power_threads_over_lists_without_disturbing_canonicalization ... ok
test tests::structured_exceptions::catch_exceptions_dispatch_survives_body_suspension ... ok
test tests::structured_exceptions::classic_and_exception_channels_do_not_cross_catch ... ok
test tests::structure_ops::surface::flatten_covers_current_same_head_level_subset ... ok
test tests::structured_exceptions::cleanup_runs_and_enclose_does_not_intercept_exceptions ... ok
test tests::structured_exceptions::exception_registry_round_trips_and_reregistration_fails_closed ... ok
test tests::structured_exceptions::exception_dispatch_honors_exact_ancestry_handler_order_and_outer_reraise ... ok
test tests::structured_exceptions::unregistered_and_uncaught_exceptions_use_fail_closed_in_band_values ... ok
test tests::traversal::count_like_condition_guards_preserve_short_circuit_points ... ok
test tests::traversal::position_supports_standard_levelspecs_heads_and_limits ... ok
test tests::strings::advanced_patterns::regular_expression_group_nesting_is_capped_and_far_past_shapes_stay_inert ... ok
test tests::traversal::level_supports_heads_and_apply_head_forms ... ok
test tests::traversal::count_memberq_and_freeq_share_standard_traversal_semantics ... ok
test toml::encode::depth_tests::to_toml_accepts_nesting_at_the_depth_limit ... ok
test tests::valueq_effect_suppression::valueq_restores_the_exact_prior_policy_on_every_outcome ... ok
test tests::traversal::level_supports_standard_levelspecs_and_depth_first_ordering ... ok
test tests::valueq_effect_suppression::valueq_refuses_file_write_and_process_then_restores_the_policy ... ok
test tests::valueq_effect_suppression::valueq_allows_read_clock_diagnostic_and_random_capabilities ... ok
test tests::traversal::traversal_uses_values_only_association_semantics_and_keyed_positions ... ok
test tests::valueq_effect_suppression::valueq_restores_prng_on_success_error_suspension_and_nesting ... ok
test toml::encode::depth_tests::to_toml_refuses_nesting_one_past_the_depth_limit ... ok
test yaml::encode::depth_tests::to_yaml_accepts_nesting_at_the_depth_limit ... ok
test tests::valueq_effect_suppression::valueq_suppresses_an_installed_print_sink_and_restores_it ... ok
test yaml::encode::depth_tests::to_yaml_refuses_nesting_one_past_the_depth_limit ... ok
test xml::encode::depth_tests::to_xml_accepts_nesting_at_the_depth_limit ... ok
test xml::encode::depth_tests::to_xml_refuses_nesting_one_past_the_depth_limit ... ok
test tests::strings::incidental_runtime::association_key_ops::key_sort_by_evaluates_each_key_once_at_scale ... ok
test tests::predicates::collection_same_test_pairs_run_once_each_without_native_stack_recursion ... ok
test tests::associations::associations_key_sort::custom_key_sorts_obey_the_merge_bound_without_growing_the_stack ... ok
test tests::structure_ops::dedup::custom_delete_duplicates_test_preserves_wide_pair_count_without_growing_the_stack ... ok
test tests::structure_ops::collection_stats::custom_tally_test_preserves_wide_pair_count_without_growing_the_stack ... ok
test tests::structure_ops::ordering_sets::wide_symbol_sort_matches_the_full_list_and_pinned_base_rendering ... ok
test tests::ownership_graph::fixtures::manually_checked_definition_counts ... ok
test tests::stress_corpus::bug_list_pipeline_range_50000_overflows_native_stack ... ok
test tests::structure_ops::ordering_sets::wide_custom_same_test_set_operations_preserve_comparison_counts ... ok
test tests::structure_ops::collection_stats::custom_reverse_sort_comparator_obeys_the_merge_bound_without_growing_the_stack ... ok
test tests::strings::parts::string_apply_walks_fifty_thousand_strings ... ok
test tests::ownership_graph::fixtures::rediscovery_and_each_missing_mutual_member ... ok
test tests::ownership_graph::discovery_report_without_inventory_input ... ok
test tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates ... ok
test tests::ownership_graph::host_payload_ledger::tests::metadata_duplicates_and_witnesses_are_enforced ... ok
test tests::structure_ops::ordering_sets::custom_sort_comparators_obey_the_merge_bound_without_growing_the_stack ... ok
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... ok
test tests::strings::search_replace::string_replace_and_cases_walk_fifty_thousand_strings ... ok
test tests::ownership_graph::production_inventory_is_exact ... ok
test tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected ... ok
test tests::ownership_graph::source::root_set::syntax_root_discovers_added_recursive_type_and_restores_source ... ok

test result: ok. 937 passed; 0 failed; 52 ignored; 0 measured; 0 filtered out; finished in 20.13s

     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)

running 193 tests
test ast::tests::display_formats_less_common_nodes_and_precedence_parentheses ... ok
test ast::tests::operator_tokens_cover_all_ast_operator_variants ... ok
test ast::tests::syntax_string_display_escapes_round_trippable_characters ... ok
test ast::tests::constructors_preserve_kinds_and_spans_while_equality_ignores_spans ... ok
test document::tests::source_diagnostic_exposes_structured_span_data ... ok
test document::tests::parsed_source_debug_dump_includes_syntax_tokens_and_trivia ... ok
test document::tests::source_diagnostic_renders_multiline_spans ... ok
test document::tests::source_diagnostic_renders_line_column_and_snippet ... ok
test document::tests::parsed_source_preserves_source_slices_locations_and_trivia ... ok
test document::tests::source_diagnostic_renders_related_opening_span ... ok
test format_request::tests::parse_format_command_accepts_prefix_and_suffix_width_flags ... ok
test format_request::tests::parse_format_cli_args_accepts_prefix_and_suffix_width_flags ... ok
test format_request::tests::parse_format_command_rejects_invalid_width_with_shared_message ... ok
test format_request::tests::parse_format_command_rejects_unknown_leading_flags ... ok
test formatter::tests::formatter_accounts_for_current_column_when_fitting_nested_groups ... ok
test formatter::tests::formatter_actively_expands_binary_expressions_with_long_inline_comments ... ok
test formatter::tests::formatter_actively_expands_calls_with_escaped_string_literals ... ok
test formatter::tests::formatter_actively_expands_calls_with_long_inline_comments ... ok
test formatter::tests::formatter_actively_expands_long_list_literals ... ok
test formatter::tests::formatter_actively_expands_wide_binary_rhs_inside_calls ... ok
test formatter::tests::formatter_actively_expands_wide_binary_chains_inside_calls ... ok
test formatter::tests::formatter_actively_expands_long_call_argument_lists ... ok
test formatter::tests::formatter_actively_expands_wide_top_level_binary_expressions ... ok
test formatter::tests::formatter_actively_expands_wide_top_level_lists ... ok
test formatter::tests::formatter_actively_expands_wide_two_argument_calls ... ok
test formatter::tests::formatter_dedents_closing_delimiters_after_multiline_comments ... ok
test formatter::tests::formatter_canonicalizes_core_spacing ... ok
test formatter::tests::formatter_expands_wide_parent_calls_around_nested_lists ... ok
test formatter::tests::formatter_expands_long_nested_calls_without_expanding_the_parent ... ok
test formatter::tests::formatter_expands_parent_calls_when_nested_binary_chains_make_the_parent_wide ... ok
test formatter::tests::formatter_dedents_nested_closing_delimiters_after_multiline_comments ... ok
test formatter::tests::formatter_handles_multiline_comments_at_multiple_nesting_levels ... ok
test formatter::tests::formatter_handles_association_literals ... ok
test formatter::tests::formatter_handles_multiline_comments_in_parenthesized_binary_expressions_with_inline_comments ... ok
test formatter::tests::formatter_handles_deeply_nested_multiline_comments_with_following_inline_comments ... ok
test formatter::tests::formatter_handles_stacked_comments_before_nested_closing_delimiters ... ok
test formatter::tests::formatter_handles_stacked_rhs_comments_inside_parenthesized_calls ... ok
test formatter::tests::formatter_actively_expands_wide_top_level_binary_chains ... ok
test formatter::tests::formatter_indents_binary_chains_with_multiline_comments_inside_calls ... ok
test formatter::tests::formatter_indents_multiline_comments_before_nested_outer_arguments ... ok
test formatter::tests::formatter_keeps_lhs_inline_comments_while_expanding_binary_chains ... ok
test formatter::tests::formatter_keeps_max_width_binary_expressions_inline ... ok
test formatter::tests::formatter_keeps_comparison_chains_and_parenthesis_boundaries ... ok
test formatter::tests::formatter_handles_mixed_inline_and_multiline_comments_inside_nested_lists ... ok
test formatter::tests::formatter_handles_nested_lists_followed_by_outer_comment_blocks ... ok
test formatter::tests::formatter_keeps_multiline_comments_attached_in_place ... ok
test formatter::tests::formatter_indents_multiline_comments_inside_parenthesized_binary_rhs ... ok
test formatter::tests::formatter_indents_multiline_comments_inside_nested_lists ... ok
test formatter::tests::formatter_indexed_break_points_preserve_every_wide_list_item ... ok
test formatter::tests::formatter_normalizes_binary_chains_with_middle_multiline_comments ... ok
test formatter::tests::formatter_normalizes_binary_chains_with_mixed_inline_and_multiline_comments ... ok
test formatter::tests::formatter_normalizes_binary_chains_with_trailing_multiline_comments ... ok
test formatter::tests::formatter_indents_multiline_comments_inside_nested_calls ... ok
test formatter::tests::formatter_preserves_outer_comment_trivia ... ok
test formatter::tests::formatter_keeps_max_width_calls_inline ... ok
test formatter::tests::formatter_handles_mixed_inline_and_multiline_comments_inside_nested_calls ... ok
test formatter::tests::formatter_preserves_plain_multiline_call_arguments ... ok
test formatter::tests::formatter_is_idempotent_on_regression_corpus ... ok
test formatter::tests::formatter_respects_custom_line_width_for_calls ... ok
test formatter::tests::formatter_keeps_parent_calls_inline_when_nested_binary_chains_fit ... ok
test formatter::tests::formatter_keeps_short_binary_chains_inline ... ok
test formatter::tests::formatter_keeps_stacked_multiline_comments_grouped_with_the_following_arg ... ok
test lexer::tests::comments_do_not_shift_token_spans_for_following_tokens ... ok
test formatter::tests::formatter_spaces_apply_sugar ... ok
test lexer::tests::decodes_named_character_escapes_in_code_symbols ... ok
test formatter::tests::formatter_wraps_right_associative_assignment_chains ... ok
test formatter::tests::formatter_spaces_compound_expression_sugar ... ok
test lexer::tests::decodes_named_character_escapes_in_strings ... ok
test lexer::tests::decodes_named_character_operator_with_source_span ... ok
test formatter::tests::formatter_spaces_functional_operator_sugars ... ok
test formatter::tests::formatter_normalizes_partially_multiline_binary_chains ... ok
test formatter::tests::formatter_indents_multiline_comments_before_binary_rhs_inside_calls ... ok
test formatter::tests::formatter_preserves_inline_comments_inside_calls_and_binary_ops ... ok
test formatter::tests::formatter_indents_binary_chains_with_multiline_comments_inside_parenthesized_calls ... ok
test lexer::tests::lexes_bare_dot_without_regressing_decimal_or_repeated_tokens ... ok
test formatter::tests::formatter_indents_binary_chains_with_mixed_inline_and_multiline_comments_inside_calls ... ok
test formatter::tests::formatter_preserves_multiline_string_literals_verbatim ... ok
test lexer::tests::lexes_string_literals_with_basic_and_unicode_escapes ... ok
test lexer::tests::lexes_escaped_multiline_string_literals ... ok
test lexer::tests::preserves_comments_and_whitespace_as_leading_trivia ... ok
test lexer::tests::preserves_trailing_trivia_on_end_token ... ok
test lexer::tests::lexes_unicode_letter_symbols_used_by_package_source ... ok
test lexer::tests::pins_multiline_string_delimiter_greedy_edges ... ok
test lexer::tests::reports_unknown_named_character_escapes ... ok
test lexer::tests::reports_unterminated_nested_comment ... ok
test lexer::tests::reports_unexpected_character_byte_span ... ok
test lexer::tests::skips_comments_and_nested_comments_as_trivia ... ok
test lexer::tests::reports_unterminated_multiline_string_literals_with_spans ... ok
test formatter::tests::formatter_respects_custom_line_width_for_binary_chains ... ok
test formatter::tests::formatter_preserves_plain_multiline_binary_rhs_inside_calls ... ok
test formatter::tests::formatter_preserves_plain_multiline_nested_call_arguments ... ok
test formatter::tests::formatter_preserves_plain_multiline_list_elements ... ok
test lexer::tests::lexes_context_qualified_symbol ... ok
test lexer::tests::tokens_carry_byte_spans ... ok
test lexer::tests::treats_initial_shebang_as_script_comment_trivia ... ok
test parser::script::tests::crlf_sources_lose_their_carriage_returns ... ok
test lexer::tests::lexes_new_infix_operators_before_shared_prefixes ... ok
test lexer::tests::lexes_multi_char_operators ... ok
test parser::script::tests::hard_errors_poison_through_the_offending_line ... ok
test parser::script::tests::incomplete_tail_reports_the_legacy_shaped_error ... ok
test lexer::tests::lexes_postfix_operator_tokens_without_splitting_decimal_literals ... ok
test lexer::tests::lexes_raw_multiline_string_literals_only_in_operand_position ... ok
test parser::script::tests::lex_failures_are_reported_for_the_caller_to_fall_back ... ok
test lexer::tests::lexes_replace_repeated_before_replace_all_prefix ... ok
test parser::tests::apply_compound::parses_apply_and_composition_operator_precedence ... ok
test parser::script::tests::stops_before_new_line_operators_and_postfixes ... ok
test parser::tests::apply_compound::parses_apply_level_operators_as_right_associative ... ok
test lexer::tests::lexes_specific_slash_operators_before_postfix_apply ... ok
test formatter::tests::formatter_leaves_top_level_multiline_binary_rhs_flush_left ... ok
test parser::tests::apply_compound::parses_function_postfix_on_set_rhs_before_map ... ok
test parser::tests::apply_compound::parses_postfix_apply_as_left_associative ... ok
test parser::tests::apply_compound::parses_postfix_apply_between_set_and_replace_all_precedence ... ok
test parser::tests::apply_compound::parses_right_associative_apply_sugar ... ok
test parser::script::tests::required_continuations_cross_line_boundaries ... ok
test parser::script::tests::parsed_forms_carry_syntax_and_poisoned_forms_do_not ... ok
test parser::tests::basics::parses_large_integer_literal ... ok
test parser::tests::apply_compound::parses_compound_expression_sugar ... ok
test parser::script::tests::shebang_blank_and_comment_lines_never_start_a_form ... ok
test parser::tests::apply_compound::parses_empty_mid_chain_statement_as_locally_left_leaning ... ok
test parser::tests::apply_compound::parses_function_postfix_as_map_lhs ... ok
test parser::tests::basics::parses_association_literals ... ok
test parser::tests::basics::parses_lists_and_nested_calls ... ok
test parser::script::tests::splits_on_new_line_tokens_at_depth_zero ... ok
test parser::tests::basics::parses_multiline_string_literals_as_string_atoms ... ok
test parser::tests::basics::parses_simple_decimal_literal_as_numeric_atom ... ok
test parser::tests::diagnostics::reports_bare_pure_function_marker_as_hard_error ... ok
test parser::tests::diagnostics::reports_incomplete_association_input ... ok
test parser::tests::diagnostics::distinguishes_empty_input_from_incomplete_input ... ok
test parser::tests::diagnostics::reports_malformed_call_arguments_without_marking_input_incomplete ... ok
test parser::tests::diagnostics::reports_incomplete_call_input ... ok
test parser::tests::diagnostics::reports_incomplete_double_bracket_input ... ok
test parser::tests::apply_compound::independent_statement_spines_do_not_accumulate ... ok
test parser::tests::diagnostics::reports_malformed_list_items_without_marking_input_incomplete ... ok
test parser::tests::diagnostics::reports_incomplete_group_input ... ok
test parser::tests::diagnostics::reports_missing_closing_tilde_in_infix_application ... ok
test parser::tests::apply_compound::parses_long_single_form_statement_chain ... ok
test parser::tests::diagnostics::reports_incomplete_rhs_after_binary_operator ... ok
test parser::tests::diagnostics::reports_trailing_pattern_tokens_as_hard_errors ... ok
test parser::tests::diagnostics::reports_unexpected_closing_delimiters_with_token_labels ... ok
test parser::tests::diagnostics::reports_unterminated_multiline_string_as_incomplete_input ... ok
test parser::tests::diagnostics::source_offset_applies_to_syntax_parser_and_lexer_error_spans ... ok
test parser::tests::infix_operators::comparison_chains_do_not_cross_parenthesis_boundaries ... ok
test parser::tests::infix_operators::parses_implicit_multiplication_juxtaposition ... ok
test parser::tests::basics::pins_power_chain_operator_depth_boundary ... ok
test parser::tests::infix_operators::parses_logical_and_comparison_operator_precedence ... ok
test parser::tests::infix_operators::parses_prefix_minus_with_power_precedence ... ok
test parser::tests::infix_operators::parses_new_infix_binary_operators ... ok
test parser::tests::diagnostics::script_form_source_map_preserves_normalized_text_and_raw_file_offsets ... ok
test parser::tests::infix_operators::parses_string_expression_infix_sugar ... ok
test parser::tests::infix_operators::parses_string_join_infix_sugar ... ok
test parser::tests::basics::rejects_deep_postfix_blank_chain_without_panicking ... ok
test parser::tests::infix_operators::parses_tilde_infix_application_as_call ... ok
test parser::tests::basics::rejects_too_deep_nested_call_without_panicking ... ok
test document::tests::debug_dump_matches_alternate_debug_across_front_and_tail ... ok
test parser::tests::basics::pins_nested_call_parse_depth_boundary ... ok
test parser::tests::left_chain_limits::left_chain_limit_error_is_terminal ... ok
test parser::tests::parts_spans_comments::keeps_group_and_postfix_delimiters_inside_node_spans ... ok
test parser::tests::patterns::parses_named_patterns ... ok
test parser::tests::parts_spans_comments::parses_expressions_with_comments_between_tokens ... ok
test parser::tests::patterns::parses_colon_named_pattern_sugar ... ok
test parser::tests::parts_spans_comments::attaches_spans_to_root_and_nested_nodes ... ok
test parser::tests::patterns::parses_optional_default_patterns ... ok
test parser::tests::patterns::parses_pattern_alternatives_sugar ... ok
test parser::tests::parts_spans_comments::parses_part_and_span_sugar_into_canonical_forms ... ok
test parser::tests::patterns::parses_pattern_test_sugar ... ok
test parser::tests::rules_slots_assignments::does_not_merge_slot_specs_across_trivia ... ok
test parser::tests::rules_slots_assignments::does_not_merge_slots_across_trivia_into_a_sequence_slot ... ok
test parser::tests::rules_slots_assignments::parses_factorial_derivative_and_repeated_postfix_forms ... ok
test parser::tests::rules_slots_assignments::parses_message_name_before_assignment ... ok
test parser::tests::rules_slots_assignments::parses_message_name_inside_calls ... ok
test parser::tests::rules_slots_assignments::parses_replace_repeated_precedence ... ok
test parser::tests::rules_slots_assignments::parses_rule_and_replace_all_precedence ... ok
test parser::tests::apply_compound::pins_statement_chain_boundary_with_dedicated_error ... ok
test parser::tests::rules_slots_assignments::parses_rule_delayed_rhs_before_assignment_rhs ... ok
test parser::tests::rules_slots_assignments::parses_rule_expression_as_pure_function_body ... ok
test parser::tests::rules_slots_assignments::parses_rule_rhs_before_assignment_rhs ... ok
test parser::tests::rules_slots_assignments::parses_slots_and_pure_functions ... ok
test parser::tests::rules_slots_assignments::parses_tagset_before_assignment_rhs ... ok
test parser::tests::rules_slots_assignments::parses_unset_postfix ... ok
test trivia::tests::parsed_comments_attach_to_following_tokens_and_end_token_trivia ... ok
test trivia::tests::syntax_token_kind_labels_cover_atoms_punctuation_and_operators ... ok
test trivia::tests::trivia_constructors_labels_and_lexed_source_accessors_preserve_ranges ... ok
test parser::tests::basics::rejects_deep_prefix_chain_without_panicking ... ok
test parser::tests::apply_compound::rejects_empty_statement_spine_without_overflowing ... ok
test parser::tests::left_chain_limits::left_chain_limit_does_not_swallow_later_script_forms ... ok
test parser::tests::left_chain_limits::left_chain_limit_diagnostic_marks_the_offending_operator ... ok
test document::tests::debug_dump_preserves_every_syntax_variant_and_leaf_layout ... ok
test parser::tests::left_chain_limits::left_chain_mixed_boundary_is_exact ... ok
test parser::tests::left_chain_limits::left_chain_implicit_times_boundary_is_exact ... ok
test parser::tests::basics::rejects_deep_right_associative_chain_without_panicking ... ok
test parser::tests::left_chain_limits::left_chain_homogeneous_boundary_is_exact ... ok
test parser::tests::left_chain_limits::left_chain_independent_spines_have_independent_counts ... ok
test document::tests::debug_dump_deep_head_chain_completes_on_small_stack ... ok

test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.41s

     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)

running 5 tests
test tests::host_eval_response_uses_hold_transparent_canonical_contract_text ... ok
test tests::host_eval_response_keeps_canonical_contract_bytes ... ok
test tests::host_inspect_renders_summary_and_definition_lines ... ok
test tests::host_failure_renders_messages_before_syntax_diagnostic ... ok
test tests::host_message_catalog_renders_immediate_and_delayed_templates ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rum_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rum_derive

running 7 tests
test crates/rum-derive/src/lib.rs - derive_accumulator_chain_node (line 45) - compile fail ... ok
test crates/rum-derive/src/lib.rs - derive_frame_root_trace (line 102) - compile fail ... ok
test crates/rum-derive/src/lib.rs - derive_frame_root_trace (line 86) - compile fail ... ok
test crates/rum-derive/src/lib.rs - derive_accumulator_chain_node (line 56) - compile fail ... ok
test crates/rum-derive/src/lib.rs - derive_accumulator_chain_node (line 63) - compile fail ... ok
test crates/rum-derive/src/lib.rs - derive_accumulator_chain_node (line 24) - compile fail ... ok
test crates/rum-derive/src/lib.rs - derive_accumulator_chain_node (line 32) - compile fail ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

   Doc-tests rum_host

running 1 test
test crates/rum-host/src/lib.rs - (line 12) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.65s

   Doc-tests rum_jupyter

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rum_repl

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rum_runtime

running 2 tests
test crates/rum-runtime/src/runtime/native_stack_guard.rs - runtime::native_stack_guard::NativeStackDepth (line 19) - compile fail ... ok
test crates/rum-runtime/src/runtime/native_stack_guard.rs - runtime::native_stack_guard::run_with_native_stack_guard (line 79) - compile fail ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

   Doc-tests rum_stdlib

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rum_syntax

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rum_terminal

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p rum-jupyter --lib`
EXIT_STATUS 101
````

### 6. Refusals, scope, restoration and open questions

I stopped implementation after the first owner's measured cost exceeded the
brief's bar. I did not tune that implementation, try an alternative mode,
restore brief-518's prototype, weaken occurrence accounting, or replace the
walker with a partial value. The outstanding work is the complete maintained
session sum, all remaining owner mutation/release paths, the full equality
falsifier at every armed checkpoint, and the candidate's nine armed comparison
rows. These are not reported as completed.

The rejection establishes a cost failure for the attempted owner implementation.
It does not determine whether a different design can meet 0.7%; that remains
open, along with brief-517's production overflow gaps and interior-mutating
retained owners. The 165 per-builtin Incremental declarations remain untouched
and outside this task.

Temporary source files, all within the named accounting/mutation/test/module
scope, were:

- `crates/rum-runtime/src/runtime/debug_tools/resources.rs` (113 bytes in the probe).
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs` (16,909 bytes in the probe).
- `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/schema.rs` (14,705 bytes in the probe).
- `crates/rum-runtime/src/runtime/eval_core/mod.rs` (8,496 bytes in the probe).
- `crates/rum-runtime/src/runtime/model/session.rs` (9,052 bytes in the probe).
- `crates/rum-runtime/src/runtime/session/session_helpers.rs` (24,613 bytes in the probe).
- `crates/rum-runtime/src/runtime/session/session_state.rs` (24,128 bytes in the probe).
- `crates/rum-runtime/src/runtime/suspension/core.rs` (21,992 bytes in the probe).
- `crates/rum-runtime/src/runtime/tests/session_memory.rs` (5,016 bytes in the probe).
- `crates/rum-runtime/src/runtime/tests/session_memory_structures.rs` (14,605 bytes in the probe).
- `crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs` (3,018 bytes in the probe).
- `crates/rum-runtime/src/runtime/tests/session_memory_maintained.rs` (3,953 bytes in the probe).

No compiler-forced change outside the allowed scope was needed. The schema
and inventory-count test temporarily classified the added inline field; the
existing direct stack fixture used the new cold replacement method. No ownership
inventory fixture, Expr/SharedExpr/SharedArgs, interner, builtin growth
classification, script under `scripts/`, Cargo file, AGENTS/TODO/PROGRESS file,
or Git branch/index/config was modified. No network was used and all task
writes, including TMPDIR, stayed in this repository. Earlier evidence directories
were read, not overwritten.

Organization assessment for temporary touches: `suspension/core.rs` (21,992
bytes) remained the existing suspension owner/driver module; its future split
boundary is resume versus abandonment drivers. `session/session_helpers.rs`
(24,613 bytes) can split rollback/snapshot handling from output/request helpers.
`session/session_state.rs` (24,128 bytes) can split Session construction from
frame/state type definitions. This probe changed only owner handoffs or field
initialization in those files; no organizational split was undertaken before
the feasibility gate. Both newly added files were small. All source changes
have now been removed, so these assessments introduce no permanent module debt.

Restoration used saved bytes guarded by the archived candidate hashes, without
any Git checkout/restore/index operation. Verbatim restoration and release-build
output:

````console
$ scripts/run-bounded.py --timeout 60 -- python3 target/brief-521/restore_probe.py
RESTORED_PROBE_FILES 12
ALL_INITIAL_TRACKED_HASHES_MATCH 1512
EXIT_STATUS 0
````

````console
$ scripts/run-bounded.py --timeout 900 -- cargo build --release -p rum-cli
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 22.72s
EXIT_STATUS 0
````

The base and restored release executable both have SHA-256:

```text
beda3aa055222aa5ca4db7320684e05848ad2bbad5f247431e84e6d02d103376
```

`binary-hashes.json` also records the rejected probe binary. The original brief
prefix is preserved byte for byte; only this execution report is appended.
No production implementation is offered for acceptance.

Final report integrity checks:

The first whitespace check found an extra blank line at the report EOF;
this final evidence block removes that trailing blank-line condition without
changing any captured test output. The initial diagnostic is preserved:

```console
$ git diff --check
docs/dev/codex/brief-521-maintained-session-total.md:3577: new blank line at EOF.
EXIT_STATUS 2
```

```text
UNCHANGED_TRACKED_FILES 1511
ONLY_TRACKED_CHANGE docs/dev/codex/brief-521-maintained-session-total.md
ORIGINAL_BRIEF_PREFIX_UNCHANGED
FALSIFIER_AND_MANDATORY_LOGS_PASTED_VERBATIM
BASE_WORKLOAD_HASHES_UNCHANGED
PROBE_SOURCE_REMOVED
```

```console
$ git status --porcelain
 M docs/dev/codex/brief-521-maintained-session-total.md
EXIT_STATUS 0
```

```console
$ git diff --check
EXIT_STATUS 0
```
