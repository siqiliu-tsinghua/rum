# brief-415 — the session memory walk must account for the whole session

Shape: **implementation**. Worker: **cdx (Codex)**.

Origin: my review of brief-414's plan. Codex's inventory covered *operations*
that grow bytes; this covers the *state* the walk forgets to look at. It comes
first because everything downstream compares against this number.

## Goal

`current_memory_proxy_bytes` is the baseline every memory-ceiling decision is
measured against — the strided check adds a projected expression to it
(`constraints.rs:145-159`), and every one of the 18 projected-byte prechecks
adds to it (`constraints.rs:174-186`). It walks 34 of `Session`'s 93 fields.
The walk begins at `std::mem::size_of::<Session>()` (`memory.rs:51`), so scalar
fields are already covered; the omissions that matter are container fields
whose contents grow with script activity.

When you are done: every field of `Session` either contributes a term to the
walk or carries a recorded exemption with a stated reason, and **adding a field
without making that decision fails a test**.

## Context pointers

- `AGENTS.md` first (operational notes, Testing Policy hard lines).
- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-262` — the
  walk. `:269-319` — `expr_memory_proxy_bytes`, what a value actually costs.
- `crates/rum-runtime/src/runtime/model/session.rs:29` — the `Session` struct.
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:145-186`
  — the two consumers of the baseline.
- **The precedent to copy** is `FrameRootTrace`: the derive makes an untraceable
  field a compile error, and `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` and its module
  keeps a sorted inventory of every handwritten impl so a new one cannot appear
  unnoticed. Read that test before designing this one.
- `crates/rum-runtime/src/runtime/model/cache.rs:332,458` and
  `model/definitions/tagged_value_store.rs:220` — the existing
  `memory_proxy_bytes` convention. Follow it; do not invent a second one.

## The finding, with evidence

Director-measured: `Session` declares 93 fields
(`crates/rum-runtime/src/runtime/model/session.rs:29`); the walk references 34
of them. Thirteen of the unreferenced ones are container-typed:

| Field | Type | Declared in |
|---|---|---|
| `installed_system_down_values` | `SymbolMap<Vec<(Expr, Expr)>>` | `model/session.rs` |
| `installed_system_own_values` | `SymbolSet` | `model/session.rs` |
| `sealed_units` | `Vec<SourceUnit>` | `scoping/packages.rs` |
| `exception_types` | `BTreeMap<String, Option<String>>` | `model/session.rs` |
| `gc_candidate_gensyms` | `HashSet<Symbol>` | `model/session.rs` |
| `dynamic_own_values` | `Vec<DynamicOwnValueFrame>` | `session/session_state.rs` |
| `module_activations` | `Vec<ModuleActivation>` | `model/session.rs` |
| `message_symbol_counts` | `SymbolCountMap` | `model/cache.rs` |
| `attribute_set_cache` | `AttributeSetCache` | `model/cache.rs` |
| `builtin_dispatch_spec_cache` | `BuiltinDispatchSpecCache` | `model/cache.rs` |
| `stable_symbol_cache` | `SymbolStabilityCache` | `model/cache.rs` |
| `symbol_call_effect_cache` | `SymbolCallEffectCache` | `model/cache.rs` |
| `call_stability_cache` | `CallStabilityCache` | `model/cache.rs` |

`stability_dirty_bloom` and `stability_rewritten_dirty_bloom` (`StabilityBloom`,
`model/cache.rs`) are probably fixed-size — establish that rather than assume
it, and if so they are exemptions with a reason, not omissions.

No test asserts the walk's completeness. I grepped
`crates/rum-runtime/src/runtime/tests/` and `crates/rum-runtime/tests/` for
`current_memory_proxy_bytes` / `memory_proxy`: the only hit is an unrelated
gensym-GC suspension test.

### The second omission: `ByteArray` is charged 25% under what it holds

`expr_memory_proxy_bytes` gives `ByteArray` a special case
(`memory.rs:289-305`): it charges `base64_decoded_len(&encoded)` — the length
of the payload *after* decoding. But a `ByteArray` physically holds the base64
**text**: `docs/guide/structured-data.md:14` states "`ByteArray` represents raw
byte payloads as base64 text", so an n-byte payload occupies a `String` of
`ceil(n/3)*4 ≈ 1.333n`. The model charges `n`.

Every other shape in that function charges what is actually held — a plain
string is charged `value.capacity()` (`memory.rs:283-285`). `ByteArray` is the one
shape charged less than it occupies, by construction, on every instance.

**User ruling 2026-09-02: charge the physical representation.** The ceiling
exists so the process does not exhaust memory; a 25% systematic under-charge on
the shape a script most easily makes large (`RandomBytes`, `StringToByteArray`,
`FromBase64`) is exactly the defect class this line is repairing. If the
representation ever changes to a native byte vector, the accounting follows the
representation — that is correct, because the ceiling is about process memory,
not about the logical size of the data.

## Deliverable

1. **A per-field decision for all 93 fields**, in one place, each row saying
   ACCOUNTED (with the term added) or EXEMPT (with the reason). Acceptable
   exemption reasons, and no others:
   - the field is a scalar or fixed-size value already covered by
     `size_of::<Session>()`;
   - the field's contents are already counted through another walked field
     (say which, and why double-counting would be wrong);
   - the field is provably bounded to a fixed maximum — state the bound and
     where it is enforced, with `path:line`.

   "It is only a cache" is **not** an acceptable reason on its own. A cache the
   script caused to grow is memory the script caused to grow. If you judge that
   charging a derived accelerator to the request ceiling is semantically wrong,
   say so as an open question in the report and account for it anyway; that
   decision is mine, not yours.

2. **The walk extended** with the terms from (1), following the existing
   `memory_proxy_bytes` convention on the owning types where one is natural
   (`model/cache.rs` already has two such methods).

3. **An invariant test** that fails when a `Session` field is added without a
   decision. Design it the way `frame_root_trace` does: a sorted, checked-in
   inventory that a new field breaks. Prefer any mechanism that makes the
   omission a *compile* error over one that makes it a test failure, but a
   sorted-inventory test is acceptable if no compile-time mechanism fits —
   say which you chose and why.

4. **A before/after number** for a session that exercises the newly accounted
   fields, showing the baseline moved. Choose the workload from the table above
   (`DefinePackage` for `sealed_units`, a declared exception hierarchy for
   `exception_types`, and so on) and paste the two numbers.

5. **`ByteArray` charged physically.** Replace the decoded-length substitution
   with the bytes the `Expr` actually holds — the base64 `String`'s capacity
   plus its overhead, on the same footing as every other string in that
   function. Keep the `unwrap_or_else` fallback behaviour for a malformed
   payload. Record the decision in a comment at the site so the next reader
   does not "fix" it back. Add a test asserting that a `ByteArray`'s proxy
   bytes track the encoded text, not the decoded length.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`
- `crates/rum-runtime/src/runtime/model/cache.rs`
- `crates/rum-runtime/src/runtime/model/session.rs`
- `crates/rum-runtime/src/runtime/model/mod.rs`
- `crates/rum-runtime/src/runtime/session/session_state.rs`
- `crates/rum-runtime/src/runtime/scoping/packages.rs`
- `crates/rum-runtime/src/runtime/tests/mod.rs`
- a new test module under `crates/rum-runtime/src/runtime/tests/`

If a type you must give a `memory_proxy_bytes` method to lives outside this
list, **STOP and report it** rather than reaching for it. That has been the
single most common defect in my briefs; the list above is my best enumeration
and it may still be short.

Files/areas you MUST NOT touch: `crates/rum-core/src/expr.rs`'s `Expr`/`Symbol`
representation, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, anything under
`docs/dev/codex/` except appending your report, `.git`, git config, `Cargo.lock`.

## Hard rules

- **The `ByteArray` change makes the number go UP, and that is the point.** Any
  test that asserted the old, lower figure is asserting the defect; update it
  and show the old and new numbers in the report. Do not compensate by lowering
  another term.
- **Do not change what the ceiling admits beyond the accounting itself.** This
  brief makes the baseline honest; it does not retune limits. If a default test
  or an existing memory-limit test now fails because the baseline legitimately
  grew, that is a result to REPORT with the numbers, not to paper over by
  raising a limit. Do not adjust `ACTIVE_MEMORY_LIMIT_CHECK_STRIDE` or any
  default in `ResourceLimits`.
- **No default-path cost.** The walk runs only under an armed ceiling or an
  explicit `MemoryInUse`/`MaxMemoryUsed` request; adding terms to it is fine,
  but do not add work to `Session` construction, to dispatch, or to any path
  that runs when no ceiling is armed. If a new `memory_proxy_bytes` method
  requires maintaining a running counter on insertion, that IS default-path
  cost — prefer computing on walk.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; I run the authoritative gate and commit. Do not touch
  `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No vendor engine, oracle harness, or compatibility matrix.
- Minimal diff: no drive-by reformatting or renames.
- **Cross-check any script-derived count against live evidence.** My "93 fields
  and 34 walked" came from an `awk`/`grep` pass; verify it yourself and say so
  if I am wrong. A regex over this repository has produced a confidently wrong
  count three separate times in this project's history.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline -p rum-runtime
cargo test --offline --workspace
```

Paste the verbatim `test result:` lines. `--workspace` is required, not
optional: the baseline is observable through `MemoryInUse`/`MaxMemoryUsed` and
through `TerminatedEvaluation["MemoryLimit"]`, so `rum-stdlib` and `rum-host`
tests can move.

No microbench A/B is required **if** you add no work outside the walk itself.
If you find you must touch an insertion path, STOP and report — that changes
this brief's shape and the decision is mine.

## Report format

1. What changed — per file, line ranges, one-line why.
2. The 93-field decision table.
3. Verification — every command, verbatim output.
4. The before/after baseline numbers from Deliverable 4.
5. Any test whose expectations moved, with the old and new numbers and why the
   move is correct.
6. Stop-loss / open questions.

---

# Report — 2026-09-02

## 1. What changed

- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-333` — extended the cold session walk from 34 referenced fields to all 52 fields classified ACCOUNTED; this includes the 13 listed omissions, the additionally missed `abandonment_worklist`, and four fixed heap owners (`file_io`, `entropy_source`, `print_sink`, and `request_interrupt_control`).
- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:346-405` — changed the valid `ByteArray` branch to charge its encoded `Expr::String` capacity and overhead while retaining validation and malformed-value fallback.
- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:407-577` — added walk-only renderers for cache tables, symbol sets, installed down-values, dynamic/module/package state, exception strings, shared call-cache arguments, and opaque heap owners. The cache fields are type aliases rather than owning newtypes, so central renderers are the natural convention-preserving choice; no insertion path or counter changed.
- `crates/rum-runtime/src/runtime/tests/mod.rs:175` — registered the new session-memory regression module.
- `crates/rum-runtime/src/runtime/tests/session_memory.rs:1-224` — added the sorted 93-field decision inventory, its compile-time exhaustive `Session` destructure, the fixed-size `StabilityBloom` proof, an exact exception-storage delta test, and valid/malformed `ByteArray` physical-storage tests.
- This report was appended to the assigned brief. No other file changed.

Cross-check: the compiler-checked inventory confirms 93 declared fields, including the `#[cfg(test)]` `gensym_gc_sweep_count`. A separate review of the pre-change walk confirmed 34 unique `self.<field>` references. The brief's count of 13 missed container fields was short by one: `abandonment_worklist: Vec<SuspendedEvaluation>` was also growable and unwalked. The final inventory is 52 ACCOUNTED and 41 EXEMPT fields.

## 2. The 93-field decision table

| Field | Decision | Term or allowed exemption reason |
|---|---|---|
| `abandonment_active` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `abandonment_worklist` | ACCOUNTED | Each suspended owner plus outer `Vec` spare capacity. |
| `abort_protect_depth` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `abort_protect_pending` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `active_accepted_request` | EXEMPT | Fixed-size wrapper is covered by `size_of::<Session>()`; its `Arc` points to the allocation counted through `request_interrupt_control`, so counting it again would double-count. |
| `asserts_enabled` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `association_lookup_cache` | ACCOUNTED | Existing `AssociationLookupCache::memory_proxy_bytes`. |
| `attribute_revision` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `attribute_set_cache` | ACCOUNTED | Hash-table capacity plus live `Symbol` text. |
| `attributes` | ACCOUNTED | Existing live attribute-key `Symbol` text term. |
| `builtin_dispatch_spec_cache` | ACCOUNTED | Hash-table capacity plus live `Symbol` text. |
| `builtins` | ACCOUNTED | Existing live `BuiltinSpecMap` entry-storage term. |
| `call_stability_cache` | ACCOUNTED | Hash-table capacity plus the retained `SharedExpr` and `SharedArgs` values. |
| `call_stability_generation` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `capability_policy` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `context_path` | ACCOUNTED | Existing `String` capacities and `Vec` capacity. |
| `context_stack` | ACCOUNTED | Existing `ContextFrame` contents and `Vec` spare capacity. |
| `current_context` | ACCOUNTED | Existing `String` capacity. |
| `current_memory_limit` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `current_message_list_checkpoint` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `current_message_list_dirty` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `current_time_deadline` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `declared_stubs` | ACCOUNTED | Existing live `Symbol` values and text. |
| `definitions` | ACCOUNTED | Existing definition keys, values, rules, guards, and dispatch indexes. |
| `disabled_message_selectors` | ACCOUNTED | Existing selector strings and `Vec` spare capacity. |
| `dynamic_own_values` | ACCOUNTED | `DynamicOwnValueFrame` symbols and expressions plus `Vec` spare capacity. |
| `entropy_source` | ACCOUNTED | `Arc` allocation header plus the concrete `EntropySource` object. |
| `eval_deadline` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `eval_timeout` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `evaluation_depth` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `evaluation_epoch` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `evaluation_stack` | ACCOUNTED | Existing `EvaluationStackEntry` expressions and `Vec` spare capacity. |
| `exception_types` | ACCOUNTED | `BTreeMap` entries and both owned `String` capacities. |
| `file_io` | ACCOUNTED | `Arc` allocation header plus the concrete `FileIo` object. |
| `gc_candidate_gensyms` | ACCOUNTED | Hash-table capacity plus live `Symbol` text. |
| `gensym_gc_sweep_count` | EXEMPT | Fixed-size test-only value already covered by `size_of::<Session>()`. |
| `gensym_gc_watermark` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `handled_non_loader_eval_errors` | ACCOUNTED | Existing `RuntimeMessage` strings and `Vec` spare capacity. |
| `has_up_value_definitions` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `in_gensym_sweep` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `installed_system_down_values` | ACCOUNTED | Hash-table storage, live `Symbol` text, rule expressions, and nested `Vec` spare capacity. |
| `installed_system_own_values` | ACCOUNTED | Hash-table capacity plus live `Symbol` text. |
| `interrupt_armed` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `interrupt_poll_counter` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `iteration_product_budget` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `last_message_list` | ACCOUNTED | Existing `RuntimeMessage` strings and `Vec` spare capacity. |
| `load_stack` | ACCOUNTED | Existing `PathBuf` payloads and `Vec` capacity. |
| `loaded_packages` | ACCOUNTED | Existing `String` capacities and `Vec` capacity. |
| `match_depth` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `memory_limit_check_counter` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `message_definitions` | ACCOUNTED | Existing message keys, tags, and immediate/delayed expressions. |
| `message_symbol_counts` | ACCOUNTED | Hash-table capacity plus live `Symbol` text. |
| `messages` | ACCOUNTED | Existing `RuntimeMessage` strings and `Vec` spare capacity. |
| `module_activations` | ACCOUNTED | `ModuleActivation` gensym vectors plus outer `Vec` spare capacity. |
| `monitor_callbacks_enabled` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `monitor_scopes` | ACCOUNTED | Existing `MonitorScope` expressions, display string, and `Vec` spare capacity. |
| `native_stack_base` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `native_stack_next_check_depth` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `next_generated_symbol_id` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `next_request_generation` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `next_sealed_symbol_id` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `next_source_unit_epoch` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `once_values` | ACCOUNTED | Existing owned `String` keys and `Expr` values. |
| `option_value_scopes` | ACCOUNTED | Existing `OptionValueScope` symbols/options and `Vec` spare capacity. |
| `options` | ACCOUNTED | Existing `Symbol` keys and `SymbolOptions` entries. |
| `package_search_paths` | ACCOUNTED | Existing `PathBuf` payloads and `Vec` capacity. |
| `peak_memory_proxy_bytes` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `pending_accepted_request` | EXEMPT | Fixed-size wrapper is covered by `size_of::<Session>()`; its `Arc` points to the allocation counted through `request_interrupt_control`, so counting it again would double-count. |
| `pending_eval_request` | ACCOUNTED | Existing conditional conservative `size_of::<PendingEvalRequest>()` term. |
| `pending_input_request` | ACCOUNTED | Existing `RuntimeInputRequest` prompt capacity. |
| `pending_suspension` | ACCOUNTED | Existing owned `SuspendedEvaluation` stack, frames, and roots. |
| `print_sink` | ACCOUNTED | Concrete boxed `FnMut` object's dynamic size. |
| `prints` | ACCOUNTED | Existing `String` capacities and `Vec` capacity. |
| `prng_state` | EXEMPT | Fixed-size `[u64; 4]` value already covered by `size_of::<Session>()`. |
| `queued_input_responses` | ACCOUNTED | Existing `String` capacities and `VecDeque` capacity. |
| `reap_scopes` | ACCOUNTED | Existing patterns, handlers, buckets, and `Vec` spare capacity. |
| `removed_symbols` | ACCOUNTED | Existing live `Symbol` values and text. |
| `request_interrupt_control` | ACCOUNTED | One `Arc` allocation header and its fixed `RequestInterruptState` object. |
| `resource_limits` | ACCOUNTED | Existing conservative `size_of::<ResourceLimits>()` term. |
| `restoring_snapshot` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `resuming_suspension` | ACCOUNTED | Existing owned `SuspendedEvaluation` stack, frames, and roots. |
| `sealed_units` | ACCOUNTED | `SourceUnit` strings, held expression, symbol vectors, private index, and outer `Vec` spare capacity. |
| `stability_dirty_bloom` | EXEMPT | Exactly two `u128` fields (`model/cache.rs:513-517`), hence fixed-size and already covered by `size_of::<Session>()`; the new size assertion proves this. |
| `stability_rewritten_dirty_bloom` | EXEMPT | Same exactly-two-`u128` bound as `stability_dirty_bloom`, already covered by `size_of::<Session>()`. |
| `stable_symbol_cache` | ACCOUNTED | Hash-table capacity plus live `Symbol` text. |
| `symbol_cache_generation` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `symbol_call_effect_cache` | ACCOUNTED | Hash-table capacity plus live key `Symbol` text. |
| `symbol_generations` | ACCOUNTED | Existing owned context/name `String` capacities and generation entries. |
| `syntax_information` | ACCOUNTED | Existing `Symbol` keys and `Expr` values. |
| `trace_event_depths` | ACCOUNTED | Existing `Vec` capacity. |
| `trace_events` | ACCOUNTED | Existing `TraceEvent` strings/expressions and `Vec` spare capacity. |
| `tracing_enabled` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |
| `value_q_effect_suppression_active` | EXEMPT | Fixed-size value already covered by `size_of::<Session>()`. |

The invariant uses a macro to generate both this classification's checked-in source inventory and an exhaustive `let Session { ... }` pattern. Rust emits a compile error for a newly added field until the macro invocation gains a row, and the test separately enforces lexical sort order and the live count of 93. This was chosen over repository source parsing because it asks the Rust compiler, not a regex, what fields exist.

## 3. Verification

Focused regression command:

```text
scripts/run-bounded.py --timeout 600 -- cargo test --offline -p rum-runtime session_memory -- --nocapture
```

Verbatim result:

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 932 filtered out; finished in 0.00s
```

Mandatory formatting command (spelled with Cargo's global offline flag to obey the no-network hard rule):

```text
scripts/run-bounded.py --timeout 120 -- cargo --offline fmt --check
```

Exit 0, no output.

Mandatory runtime command:

```text
scripts/run-bounded.py --timeout 1200 -- cargo test --offline -p rum-runtime
```

Verbatim result lines:

```text
test result: ok. 934 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.11s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Mandatory workspace command:

```text
scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace
```

This did not reach green. Cargo stopped at the first failing target:

```text
test result: FAILED. 10 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p rum-host --test suspension_abandonment_contract`
```

I then ran the workspace with `--no-fail-fast` and the explicit native-ZeroMQ sandbox opt-out. This separated the environmental loopback denial from the two accounting-affected targets:

```text
set -o pipefail
RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 scripts/run-bounded.py --timeout 2400 -- cargo test --offline --workspace --no-fail-fast 2>&1 | rg '^(test result:|error: [0-9]+ targets failed:|    `-p )'
```

Verbatim output:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: FAILED. 10 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 934 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.15s
test result: FAILED. 826 passed; 2 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.63s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.82s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 2 targets failed:
    `-p rum-host --test suspension_abandonment_contract`
    `-p rum-stdlib --lib`
```

Without `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, the sandbox additionally denied loopback to nine native tests (`97 passed; 9 failed`). With the explicit opt-out, the same target was green:

```text
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

## 4. Before/after baseline

The identical CLI workload installs a sealed package (`sealed_units` and installed definition state), declares an exception hierarchy (`exception_types`), and asks `ValueQ` to populate stability caches:

```text
DefinePackage["MemoryWalk`", Public[{f}], PackageScope[{limit}], limit = 10; helper[x_] := x + limit; f[x_] := helper[x]]; RegisterExceptionType["MemoryWalkError"]; RegisterExceptionType["MemoryWalkTimeout", "MemoryWalkError"]; ValueQ[f[1]]; MemoryInUse[]
```

Measured with the same offline `rum-cli` command:

```text
before: 69427
after:  78100
delta:   8673
```

## 5. Expectations and measured moves

- `ByteCount[ByteArray["AQID"]]`: **179 before, 212 after**. The old figure substituted decoded length `3`; the new figure includes the encoded `String`'s four-byte capacity plus its 32-byte `Expr` overhead, so the net increase is 33 bytes. No old exact-value assertion existed; the new regression asserts the physical formula and rejects the decoded substitution.
- `public_cancelled_suspension_restores_reap_monitor_trace_and_constraint_state`: existing bound is growth `< 16000`; the honest result is `before:69147, after:137459, growth:68312`.
- `public_monitor_matrix_restores_reap_monitor_trace_constraints_random_and_abort_state`: existing bound is growth `< 16000`; the honest result is `before:69167, after:329925, growth:260758`.
- `byte_count_and_memory_in_use_use_the_current_structural_proxy_subset`: the existing `after_on < after_off` relation changed from `True` to `False` because the formerly invisible evaluation-cache allocations retained across those separate requests now outweigh the removed message-selector bytes.
- `max_memory_used_tracks_session_memory_proxy_high_water_mark`: the in-expression `MaxMemoryUsed[] >= MemoryInUse[]` relation changed from `True` to `False`; evaluating the left operand and then the right grows newly visible cache state between the two samples.

No resource limit, stride, default, or existing test expectation was changed.

## 6. Stop-loss / open questions

- The required workspace gate is red on four accounting-sensitive assertions outside the brief's file allowlist. A temporary diagnostic (removed afterward) isolated the host deltas: the cancellation case went from zero call-cache entries to one entry charging 64,379 bytes; the monitor case went from zero to four entries charging 256,956 bytes. These are retained strong `SharedExpr`/`SharedArgs` trees, not leaked `Reap` buckets. Excluding them would recreate the omission this brief requires fixing.
- Restoring green requires a director decision outside this brief: either redesign the four tests to isolate the state they intend to test, or change call-stability retention/eviction semantics. The latter touches insertion paths and therefore requires a new brief plus the mandated hot-path A/B. Per the hard rule, I did not raise the 16,000-byte bounds, clear caches from the memory walk, retune a limit, or hide retained cache payloads.
- `FileIo`, `EntropySource`, and the print callback are opaque trait/closure objects. The walk can and now does charge each concrete object's immediate allocation (`size_of_val`) and allocation header where applicable, but Rust offers no generic way to discover heap allocations nested inside an arbitrary host implementation. No unsupported exemption was used.
- `memory.rs` grew from 22,366 to 32,019 bytes, still in the documented split-plan range. A future scope should extract the field-specific renderers into a sibling memory-walk module; this brief allowed no new production module, and splitting while changing the accounting would obscure the semantic review.
- No default-path work, running insertion counter, `unsafe`, limit change, or hot-path change was added. Therefore no microbench A/B was required. The worktree remains uncommitted.
