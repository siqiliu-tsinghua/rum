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
