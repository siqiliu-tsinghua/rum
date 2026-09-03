# brief-435 — spike: the height inside the value, not behind the Arc

**Shape: spike. The deliverable is a cost measurement.** No enforcement, no
limit constant, no refusal path, no public contract. If the numbers are good I
will design the bound afterwards; if they are not, this line closes.

## Why this shape and no other

Three attempts have now put a cached physical height behind the `Arc`
allocations. brief-426 measured −30.04% and blamed its constructor scan;
brief-434 decomposed it and found the constructor scan real (+3.9% to +11.4%)
and the mutation rescan free. Then the comparison neither of us had run settled
it: a build with **the height field present but never computed** already costs
+14.21% on `stringtrim-literal-list-32000` — a label that builds no call spines
and never reads a height — and +9.58% of `set-part-fixed100-len800`'s +12.21%
total.

**The floor is the 8 bytes per Arc allocation**, not any computation.
`SharedExprInner` goes 48→56 and `SharedArgsInner` 40→48, which under a
16-byte-granular allocator moves the args inner into the next chunk class and
costs cache density everywhere that allocates. No maintenance strategy recovers
that, so both the constructor redesign and the O(1) conservative update are
retired as answers.

What is left is measured, not guessed. Verified by compile-time assertion on
this toolchain:

```rust
assert!(std::mem::size_of::<Expr>() == 32);
assert!(std::mem::size_of::<Atom>() == 32);
assert!(std::mem::size_of::<SharedExpr>() == 8);
assert!(std::mem::size_of::<SharedArgs>() == 8);
```

`Expr::Call { head, args }` carries 16 bytes of payload inside a 32-byte enum,
because `Atom` is what makes `Expr` 32. **There are about 16 spare bytes inside
the value that already exists.** A height stored there grows no allocation, and
reading a child's height becomes a field read at a fixed offset in contiguous
memory instead of an Arc dereference.

## Goal

Put a saturating physical height in `Expr::Call` itself and measure it. The
question this brief answers is exactly one: **does the height cost anything
when it does not grow an allocation?**

Build it the simplest way that is honest:

- A new field on the `Expr::Call` variant, `u16` or `u32` — whichever keeps
  `size_of::<Expr>()` at 32.
- The `Expr::call*` constructors compute it: `max(head_height, max over args) +
  1`, saturating. Atoms are height 1, matching the existing convention.
- `SharedExprInner` and `SharedArgsInner` **must not gain a field.** Their
  allocation sizes must not change. That is the entire point.
- A public `Expr::physical_height()` that reads the field, so nothing is
  dead-code eliminated.
- Every other construction site and pattern updated mechanically; rustc will
  name them. Roughly 73 struct-literal constructions and ~900 `Expr::Call`
  mentions, most of them patterns needing `..`.

**Do not add** a limit, a refusal, a `Result` constructor, an enforcement test,
a `#[non_exhaustive]` attribute, or any API narrowing. Those belong to the
design that follows if this is affordable.

## The two hard invariants, and how to prove them

1. **`size_of::<Expr>()` stays 32.** Put a `const` assertion in the source so a
   regression is a compile error, and paste it. If it becomes 40, **STOP and
   report** — the spike has failed and nothing after that is worth measuring.
2. **No Arc allocation grows.** brief-426 established the technique: read the
   allocation request size from the release disassembly of `SharedExpr::new`
   and `SharedArgs::new` (it found `0x30`→`0x38` and `0x28`→`0x30` there).
   Show the same two numbers unchanged against base. Allocation *counts* must
   also be identical — the microbench reports them and they are load-immune, so
   check them before you look at a single timing.

## Correctness, kept cheap

This is a spike, but a wrong height would make the timings meaningless. Add a
focused test comparing `Expr::physical_height()` against a straightforward
recursive reference over a corpus: atoms, unary and wide calls, call-valued
heads, nested heads, and a spine of depth 65. Include one case built through
each public construction route you touched. That is enough; do not build the
equivalence apparatus the normalizer line needed.

`cargo test --workspace` must be green. If a test fails because it constructs
`Expr::Call` literally, updating it mechanically is in scope; changing what it
asserts is not.

## The A/B — the protocol, and a panel chosen from where the cost landed

```
cargo build --release -p rum-cli          # BASE, before any edit
cp target/release/rum-cli /tmp/rum-cli-base435
# ... make the change ...
cargo build --release -p rum-cli
cp target/release/rum-cli /tmp/rum-cli-cand435
# four passes in one window: base, candidate, base, candidate
```

Report **the whole panel**, not a selection, plus these three columns: the
same-binary delta (base pass 1 → base pass 2), and both cross deltas normalised
by the **median ratio over all 112 labels** rather than by a named control
group. The last brief's panel was chosen from where the cost was expected and
missed `sort-default-wide`, `reversesort-default-wide`, `set-part-fill-*` and
`keysort-default-wide`, which were among the worst rows.

Call out these rows explicitly, because they are where the rejected designs
hurt: `stringtrim-literal-list-32000`, `stringtrim-literal-list-4000`,
`stringtrim-single-literal-32000`, `set-part-fixed100-len800`,
`set-part-fixed100-len100`, `set-part-fill-large`, `set-part-fill-small`,
`sort-default-wide`, `reversesort-default-wide`, `keysort-default-wide`,
`table-iterator-pair-list`, `appendto-part-outer1000-small`,
`appendto-part-outer1000-large`, `appendto-list-incremental-small`,
`appendto-list-incremental-large`.

**A movement inside a label's own same-binary spread is not a finding.** Say so
per row rather than reporting every delta.

## Stop-loss

- `size_of::<Expr>()` is not 32 → stop, report, change nothing further.
- Either Arc allocation request size changes → stop and report.
- Allocation counts differ from base → stop and report; do not proceed to
  timings.
- A coherent family is slower in both replicates and outside its same-binary
  spread by more than 3% → stop and report. **Do not tune.** Three designs have
  now been rejected on measurement and a fourth rejection is a perfectly good
  outcome; what is not acceptable is a tuned number.

## Scope

- You may modify whatever the change requires across the workspace. Nothing is
  committed; I decide what survives.
- `docs/**` is read-only except appending your report to
  `docs/dev/codex/brief-435-inline-height-spike.md`.
- Do not change what any existing test asserts. Mechanical updates for the new
  field are in scope; semantic edits are not.

## Hard rules

- Do NOT `git commit`, branch, push, stash, checkout, or restore. Leave your
  changes in the working tree.
- No network. Touch nothing outside this repository.
- **No new `unsafe`**, including `unsafe` attributes. If you find yourself
  reaching for a union, a transmute, or bit-stealing to fit the field, STOP and
  report — that is a design decision, not a spike detail.
- **Write no probe that builds a deep value to see it fail.** This brief is
  timings, counters and one correctness test; there is no stack-overflow
  reproduction in it and none is wanted.
- Profiling env OFF for every timing number. Bound long commands with
  `scripts/run-bounded.py --timeout SECONDS`, sized from a measured run.
- Paste real verbatim output. If a build failed or a run did not complete, say
  so.
- **Append your report to this file** before you finish.

## Report format

1. What changed — per file, with line ranges, and the total site count.
2. The two invariants — the `size_of` assertion, and the two allocation request
   sizes against base, verbatim.
3. Allocation counters, base against candidate.
4. The full-panel timing table, same-binary column first.
5. The verdict in one sentence, before any explanation: is the height free when
   it does not grow an allocation?

---

# Report — 2026-09-03 — Codex

**No: physical height is not free when it stays inside the 32-byte value; the unchanged Arc sizes remove the allocation-density penalty, but constructor scanning still causes coherent +3.6% to +9.3% regressions and trips the stop-loss.**

## 1. What changed

No limit, refusal path, `Result` constructor, enforcement test, API narrowing, `#[non_exhaustive]`, or `unsafe` was added.

- `crates/rum-core/src/expr.rs:749-957,1084-1132` adds a `u16` `physical_height` field to `Expr::Call`, a compile-time 32-byte layout assertion, constant-time `Expr::physical_height()`, and the saturating `1 + max(head, args)` computation in all three public `call*` constructors. Hashing and serialization deliberately retain the old logical expression shape, so the cache does not add hash work or become accepted serialized metadata. The focused recursive-reference test covers atoms, unary and 128-wide calls, call-valued and nested heads, all three public constructors, and the required depth-65 spine.
- The 19 non-core `Expr::Call` expression literals were mechanically changed to `Expr::call_shared` so they retain the old Arc-reuse behavior and compute the field: `definition_updates.rs:454`; `incremental_append.rs:79`; `eval_call_prepare.rs:116`; `exact_paths.rs:190,196`; `ordered.rs:195,269`; `match_normalization.rs:76`; `truth.rs:644`; `delete.rs:227,236`; `edit_plan.rs:103,165,199,385,420`; and `insert.rs:198,203,205`.
- Rustc named 723 non-core destructuring patterns that needed `..`; the four existing core patterns were updated the same way. Together with the three core constructor literals and 19 non-core constructor literals, the total is **749 pre-existing sites: 727 patterns and 22 constructions** across 211 Rust files. There were 911 `Expr::Call` mentions before the edit; already-open `..` patterns required no change.
- `SharedExpr` remains `Arc<Expr>` and `SharedArgs` remains `Arc<Vec<Expr>>`; neither Arc allocation payload gained a field.

The candidate-side line ranges for every changed Rust file follow. Files beyond the substantive and constructor sites described above contain only mechanical `..` pattern updates.

<details>
<summary>Complete 211-file line inventory</summary>

```text
crates/rum-core/src/expr.rs:749, 751-773, 786, 791, 805-837, 839-847, 881, 885, 890-895, 899, 903, 917, 930, 951-957, 1084-1132
crates/rum-host/src/model/outcome.rs:575
crates/rum-runtime/src/runtime/arithmetic/canonical.rs:56, 175, 283-285, 418, 499, 560, 583, 599, 682, 706, 724, 732-734
crates/rum-runtime/src/runtime/arithmetic/extremum.rs:22, 110
crates/rum-runtime/src/runtime/arithmetic/parts.rs:155
crates/rum-runtime/src/runtime/builtins_input.rs:328, 336, 348, 361, 395
crates/rum-runtime/src/runtime/call_preparation.rs:191
crates/rum-runtime/src/runtime/debug_tools/reap.rs:327
crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:366, 391
crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:40, 325, 334, 351
crates/rum-runtime/src/runtime/debug_tools/test_report/parsing.rs:81, 119
crates/rum-runtime/src/runtime/debug_tools/trace.rs:342, 365, 392, 401, 527, 536, 770, 789, 792, 816
crates/rum-runtime/src/runtime/debug_tools/trace/builder.rs:78
crates/rum-runtime/src/runtime/definitions/application/fast_match.rs:63, 170, 257, 262
crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:56, 281, 340
crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:61, 77, 119, 157, 172, 202, 247, 316, 347, 410-412, 422, 444, 455, 467, 489, 505
crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:285
crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal_continuation.rs:23, 35, 86, 145
crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs:270
crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/prepared_store.rs:235, 629-631, 634-635, 637, 645, 652
crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs:114, 180
crates/rum-runtime/src/runtime/definitions/definition_updates.rs:436, 454, 637, 693, 729, 733
crates/rum-runtime/src/runtime/definitions/definitions.rs:174, 183, 191, 201, 210, 213, 221, 563
crates/rum-runtime/src/runtime/definitions/incremental_append.rs:79, 101, 125, 137, 187, 205, 214
crates/rum-runtime/src/runtime/definitions/part_assignment.rs:16, 286, 314, 337, 389
crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:26
crates/rum-runtime/src/runtime/definitions/value_table_assignment.rs:110, 292, 314
crates/rum-runtime/src/runtime/eval_call_prepare.rs:112, 116, 130, 139, 149
crates/rum-runtime/src/runtime/eval_core/mod.rs:110
crates/rum-runtime/src/runtime/eval_core/stability.rs:357, 373, 383
crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:210
crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:8, 159
crates/rum-runtime/src/runtime/expr_helpers.rs:44, 56, 86-88
crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:5, 14-16, 25, 107, 127, 132, 226-228
crates/rum-runtime/src/runtime/hold_thread.rs:65
crates/rum-runtime/src/runtime/kernel_data/association_data.rs:58, 72, 114, 166, 185, 232, 246, 264, 294, 416, 461, 550, 743, 956, 969
crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs:20, 73, 93, 159, 263
crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:401
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/continuation.rs:25
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/spec.rs:8, 42, 64, 142, 162, 216, 226-228
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/table_fast_path.rs:126, 189
crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:301, 304
crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:463, 474, 494
crates/rum-runtime/src/runtime/kernel_data/control_data/failsafe.rs:61
crates/rum-runtime/src/runtime/kernel_data/control_data/status.rs:94, 283, 287, 313, 330, 500
crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs:105
crates/rum-runtime/src/runtime/kernel_data/functional_data.rs:17, 23, 250, 312, 344, 384, 508, 557, 582, 642, 702, 751, 817, 883, 946, 979-984
crates/rum-runtime/src/runtime/kernel_data/rewrite_data.rs:67, 80, 89, 97, 108-110, 148
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/resume_frames.rs:110
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs:136
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs:72, 84, 296
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:63, 137, 157, 168, 183
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:369, 502
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching.rs:85
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching/resume_frames.rs:17
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:335, 383, 538, 566, 645, 744, 771, 836, 874
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs:157
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:186, 275
crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs:15, 188, 190, 196
crates/rum-runtime/src/runtime/kernel_data/structure_data/mod.rs:89
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded_plan.rs:88, 139
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/parsing.rs:11, 33, 64, 92, 144
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/plan.rs:24, 29, 168, 217, 222, 227, 232, 237-239, 264, 286, 314-321
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:334
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:500
crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:347
crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/diagnostics.rs:187
crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/extract.rs:21, 53, 83
crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors/parse.rs:28, 56, 84, 108, 188
crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:39, 82, 386, 416, 424, 434
crates/rum-runtime/src/runtime/lowering/binary.rs:158, 188, 196, 204
crates/rum-runtime/src/runtime/matcher_and_symbols.rs:75, 164, 176, 187, 198, 203, 214, 219, 229, 241, 259-261, 268, 277-279, 288, 301, 314, 322, 329, 336, 344, 360, 377, 380, 395, 484, 574
crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:485, 504, 514
crates/rum-runtime/src/runtime/messages/message_catalog.rs:62, 191
crates/rum-runtime/src/runtime/messages/message_format.rs:168
crates/rum-runtime/src/runtime/messages/message_selector_parsing.rs:34, 84
crates/rum-runtime/src/runtime/messages/parse_and_message_helpers.rs:23, 176, 216, 279, 288, 297, 310, 324, 372, 419, 454, 461
crates/rum-runtime/src/runtime/metadata.rs:269
crates/rum-runtime/src/runtime/model/definitions.rs:148, 248
crates/rum-runtime/src/runtime/namespace_tools/names.rs:170, 338
crates/rum-runtime/src/runtime/namespace_tools/syntax.rs:48, 60
crates/rum-runtime/src/runtime/options.rs:150, 160, 214
crates/rum-runtime/src/runtime/options/arguments.rs:90, 107, 362
crates/rum-runtime/src/runtime/options/builtins.rs:332, 356, 382, 389
crates/rum-runtime/src/runtime/options/core.rs:164
crates/rum-runtime/src/runtime/options/patterns.rs:88, 93, 98, 142, 147, 152
crates/rum-runtime/src/runtime/package_tools.rs:337
crates/rum-runtime/src/runtime/pattern_matching/continuation.rs:8, 124, 264, 351, 415
crates/rum-runtime/src/runtime/pattern_matching/core.rs:280
crates/rum-runtime/src/runtime/pattern_matching/core/key_value.rs:208, 234, 246, 275, 287
crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:151, 355, 478, 613
crates/rum-runtime/src/runtime/pattern_matching/outcome.rs:33, 53, 63
crates/rum-runtime/src/runtime/pattern_matching/sequence.rs:244
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:68, 342, 421
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/collect.rs:39, 73, 382
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:61, 147
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional/resume_frames.rs:318
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/pattern.rs:191, 286
crates/rum-runtime/src/runtime/pattern_matching/sequence/ordered.rs:191, 195, 265, 269
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/candidates.rs:22
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs:38, 64, 374
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:45, 67, 79, 378, 457
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:30
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional/resume_frames.rs:229
crates/rum-runtime/src/runtime/pattern_matching/sequence/resume_frames.rs:448, 472
crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:164, 434, 623, 756, 875
crates/rum-runtime/src/runtime/profiling/fingerprints.rs:39, 96
crates/rum-runtime/src/runtime/profiling/presence.rs:304
crates/rum-runtime/src/runtime/resolution.rs:457
crates/rum-runtime/src/runtime/resolution/match_normalization.rs:33, 61, 76, 85, 113-118, 135
crates/rum-runtime/src/runtime/scoping/callable.rs:97
crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:273
crates/rum-runtime/src/runtime/scoping/lexical/block/continuation.rs:8
crates/rum-runtime/src/runtime/scoping/packages.rs:22, 87, 125, 137, 195, 363, 519, 533, 578, 604, 645, 665, 721
crates/rum-runtime/src/runtime/scoping/unique.rs:98, 182
crates/rum-runtime/src/runtime/session/session_eval.rs:400
crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:55, 115, 144-146, 149, 152, 155, 340, 409, 739, 744
crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs:5-7, 16, 27, 37, 96-98, 104-106, 120-122, 138, 153, 164, 178, 195
crates/rum-runtime/src/runtime/substitution/lexical_presence.rs:73, 97, 132, 156
crates/rum-runtime/src/runtime/substitution/lexical_substitution.rs:396-398, 401, 409, 414, 417, 423, 426, 438, 486, 515-517
crates/rum-runtime/src/runtime/substitution/slot_substitution.rs:13, 41, 82, 105
crates/rum-runtime/src/runtime/suspension/collections.rs:17, 29, 63, 79, 113
crates/rum-runtime/src/runtime/suspension/error.rs:8, 147, 396, 627
crates/rum-runtime/src/runtime/suspension/primitive.rs:46, 57, 75, 110, 115, 120, 168, 181, 187, 193
crates/rum-runtime/src/runtime/suspension/rewrite_rules.rs:12, 50, 63, 69, 75
crates/rum-runtime/src/runtime/suspension/traversal.rs:17, 54, 64, 91, 138, 146
crates/rum-runtime/src/runtime/tests/arithmetic.rs:447, 468
crates/rum-runtime/src/runtime/tests/association_write_index.rs:54-56
crates/rum-runtime/src/runtime/tests/association_write_recovery.rs:9
crates/rum-runtime/src/runtime/tests/comparison_lowering.rs:41, 72, 102, 111
crates/rum-runtime/src/runtime/tests/debug_tools.rs:365
crates/rum-runtime/src/runtime/tests/holding_and_control_io.rs:253
crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs:421, 457, 468, 479, 489, 499
crates/rum-runtime/src/runtime/tests/match_normalization_contract.rs:270, 345, 375, 380, 403, 418, 448
crates/rum-runtime/src/runtime/tests/part_assignment.rs:35
crates/rum-runtime/src/runtime/tests/reap.rs:57, 66
crates/rum-runtime/src/runtime/tests/structure_data/replace.rs:360
crates/rum-runtime/src/runtime/tests/wide_child_traversals.rs:72, 88, 127, 143
crates/rum-runtime/src/runtime/tests/wide_definition_dispatch.rs:32, 41
crates/rum-runtime/src/runtime/tree_walkers/level.rs:161
crates/rum-runtime/src/runtime/tree_walkers/post_order.rs:111, 286
crates/rum-stdlib/src/arithmetic/stats.rs:73
crates/rum-stdlib/src/associations/basic.rs:24, 556, 746
crates/rum-stdlib/src/associations/data.rs:22, 97, 110, 128, 146, 205, 219, 249
crates/rum-stdlib/src/associations/grouping.rs:118
crates/rum-stdlib/src/associations/join_across.rs:180, 193
crates/rum-stdlib/src/associations/missing.rs:130, 145, 156, 178, 190, 201, 226
crates/rum-stdlib/src/associations/mutation.rs:56, 63, 510
crates/rum-stdlib/src/associations/transforms/key_collections.rs:502, 508
crates/rum-stdlib/src/bytes/codec.rs:28, 34
crates/rum-stdlib/src/calculus.rs:66, 100, 103, 106-108, 111, 233, 262, 303, 308
crates/rum-stdlib/src/control_flow.rs:176, 190, 234
crates/rum-stdlib/src/csv.rs:217
crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:277, 376, 429, 459
crates/rum-stdlib/src/functional_ops/apply_map/traversal/recursive.rs:78, 191
crates/rum-stdlib/src/functional_ops/arrays/generators.rs:418, 438, 578
crates/rum-stdlib/src/functional_ops/arrays/spec.rs:50, 118-120
crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:595
crates/rum-stdlib/src/functional_ops/callable/curried.rs:84
crates/rum-stdlib/src/functional_ops/callable/head_ops.rs:46, 62, 102
crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:187
crates/rum-stdlib/src/functional_ops/sequence/select.rs:395, 467, 491
crates/rum-stdlib/src/functional_ops/sequence/window.rs:551, 584
crates/rum-stdlib/src/functional_ops/targeted_map.rs:490, 504, 706, 721, 743
crates/rum-stdlib/src/integer_digits.rs:361, 406
crates/rum-stdlib/src/json.rs:97
crates/rum-stdlib/src/json/encode.rs:109
crates/rum-stdlib/src/lazy.rs:418, 610, 687
crates/rum-stdlib/src/logic/truth.rs:539, 551, 633, 644
crates/rum-stdlib/src/polynomials/core.rs:66, 75, 100
crates/rum-stdlib/src/polynomials/core/factors.rs:153, 162, 308
crates/rum-stdlib/src/polynomials/mod.rs:146, 197
crates/rum-stdlib/src/polynomials/rational_function.rs:25, 28, 37
crates/rum-stdlib/src/predicates.rs:73, 266, 286, 289, 292-294, 319, 322, 325-327, 341, 344, 347-349, 370, 408
crates/rum-stdlib/src/predicates/collections.rs:310, 326
crates/rum-stdlib/src/random.rs:99
crates/rum-stdlib/src/random/password_policy.rs:185
crates/rum-stdlib/src/schema.rs:417, 475
crates/rum-stdlib/src/strings.rs:106, 364
crates/rum-stdlib/src/strings/encoding.rs:295
crates/rum-stdlib/src/strings/patterns.rs:185, 283, 365, 538, 573, 622, 667, 703, 725, 758, 787, 820, 844
crates/rum-stdlib/src/strings/patterns/parts.rs:147, 166
crates/rum-stdlib/src/strings/patterns/search.rs:156
crates/rum-stdlib/src/strings/scripting/args.rs:16, 204
crates/rum-stdlib/src/strings/scripting/text_shape.rs:239
crates/rum-stdlib/src/strings/scripting/url.rs:405
crates/rum-stdlib/src/strings/similarity/mod.rs:484
crates/rum-stdlib/src/strings/similarity/nearest.rs:21, 89, 99, 106
crates/rum-stdlib/src/strings/template.rs:119, 278
crates/rum-stdlib/src/strings/text/builtins.rs:192
crates/rum-stdlib/src/strings/tokenize.rs:108, 121
crates/rum-stdlib/src/structure_ops/collection_stats.rs:927
crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:60, 161, 216, 252
crates/rum-stdlib/src/structure_ops/dedup.rs:74, 100, 140
crates/rum-stdlib/src/structure_ops/flatten.rs:225, 251, 264, 275, 356
crates/rum-stdlib/src/structure_ops/join.rs:186, 218, 248
crates/rum-stdlib/src/structure_ops/list_plumbing.rs:114-117, 295-298
crates/rum-stdlib/src/structure_ops/mod.rs:118, 147
crates/rum-stdlib/src/structure_ops/ordering_sets.rs:146, 178, 197, 408, 489, 505, 1012, 1032, 1037
crates/rum-stdlib/src/structure_ops/reordering.rs:160, 237, 323
crates/rum-stdlib/src/structure_ops/sequence/delete.rs:37, 91, 108, 132, 216, 224, 227, 236
crates/rum-stdlib/src/structure_ops/sequence/edit_plan.rs:38, 103, 115, 165, 199, 376, 385, 400, 420
crates/rum-stdlib/src/structure_ops/sequence/insert.rs:101, 121, 193, 198, 203, 205
crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:240, 333, 380
crates/rum-stdlib/src/structure_ops/sequence/take_list.rs:51, 68
crates/rum-stdlib/src/structure_ops/surface.rs:181, 215, 234, 281, 302
crates/rum-stdlib/src/tests/stress_corpus.rs:361
crates/rum-stdlib/src/tests/strings/tokenize.rs:221
crates/rum-stdlib/src/toml/encode.rs:145, 213, 220
crates/rum-stdlib/src/xml.rs:94
crates/rum-stdlib/src/xml/encode.rs:90
```

</details>

Code-organization assessment: `expr.rs` is 31,355 bytes total and 27,461 bytes before its inline tests, in the split-plan range. It was not split because this spike must isolate one hot representation change; a module split would itself require a separate A/B. The future boundary is Symbol/interner and numeric atoms in siblings, with `SharedExpr`/`SharedArgs`/`Expr` kept together. Other touched warning/split-plan files received only compiler-required one-token pattern openness, so splitting them here would add unrelated churn to a rejected measurement spike.

## 2. The two invariants

The 32-byte invariant is compiled into the source verbatim:

```rust
const _: () = assert!(std::mem::size_of::<Expr>() == 32);
```

Its first focused compilation completed:

```text
    Checking rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
```

The preserved binaries are:

```text
e28bc552b8c0b7168560f3646e09c2acc2c9399bd445c0b9a90e58ec3e8cbf97  /tmp/rum-cli-base435
48894a1c4a35b3360c8103c42970c91a7766d2cb2d9e08152578ce736a2fe05f  /tmp/rum-cli-cand435
```

Release disassembly shows the complete Arc requests unchanged against base. Verbatim deciding instructions:

```text
BASE SharedExpr::new:
  7a66d4: bf 30 00 00 00        mov    $0x30,%edi
  7a66d9: be 08 00 00 00        mov    $0x8,%esi

CANDIDATE SharedExpr::new:
  7a85f4: bf 30 00 00 00        mov    $0x30,%edi
  7a85f9: be 08 00 00 00        mov    $0x8,%esi

BASE SharedArgs::new:
  7a6d13: bf 28 00 00 00        mov    $0x28,%edi
  7a6d18: be 08 00 00 00        mov    $0x8,%esi

CANDIDATE SharedArgs::new:
  7a8c43: bf 28 00 00 00        mov    $0x28,%edi
  7a8c48: be 08 00 00 00        mov    $0x8,%esi
```

Thus `SharedExpr::new` stays at `0x30` (48 bytes) and `SharedArgs::new` stays at `0x28` (40 bytes).

## 3. Allocation counters before timings

One full 112-label scale-1, 7-repetition counter pass was run per binary. Both completed, and every reported ownership/allocation counter is identical:

| Binary | `shared_expr_alloc` | `shared_args_alloc` | `into_vec_total/clone` | `into_owned_total/clone` |
|---|---:|---:|---:|---:|
| Base | 726850 | 855636 | 116273 / 115274 | 84054 / 84054 |
| Candidate | 726850 | 855636 | 116273 / 115274 | 84054 / 84054 |

Verbatim stderr:

```text
rum-profile repr into_vec_total=116273 into_vec_clone=115274 into_owned_total=84054 into_owned_clone=84054 shared_args_alloc=855636 shared_expr_alloc=726850
rum-profile repr into_vec_total=116273 into_vec_clone=115274 into_owned_total=84054 into_owned_clone=84054 shared_args_alloc=855636 shared_expr_alloc=726850
```

Verbatim run records:

```text
rum ok 0 0.849176 target/brief435/counters/base/raw/rum.stdout target/brief435/counters/base/raw/rum.stderr
rum ok 0 0.904606 target/brief435/counters/candidate/raw/rum.stdout target/brief435/counters/candidate/raw/rum.stderr
```

## 4. Full-panel timing table

The four passes ran consecutively in the required base, candidate, base, candidate order at scale 10 and 21 repetitions. `RUM_PROFILE_REPR=0` was explicit, every other profiling switch known to the harness was set to `0` by the harness, and no CPU-heavy work ran concurrently. The 90-second outer and 75-second inner bounds were sized from the prior measured 37-second runs.

Verbatim completion and process records:

```text
core microbench output: target/brief435/timings/base1
core microbench output: target/brief435/timings/candidate1
core microbench output: target/brief435/timings/base2
core microbench output: target/brief435/timings/candidate2

rum ok 0 36.518361 target/brief435/timings/base1/raw/rum.stdout target/brief435/timings/base1/raw/rum.stderr
rum ok 0 36.701075 target/brief435/timings/candidate1/raw/rum.stdout target/brief435/timings/candidate1/raw/rum.stderr
rum ok 0 36.375967 target/brief435/timings/base2/raw/rum.stdout target/brief435/timings/base2/raw/rum.stderr
rum ok 0 36.733680 target/brief435/timings/candidate2/raw/rum.stdout target/brief435/timings/candidate2/raw/rum.stderr
```

The whole-run median candidate/base ratios over all 112 labels are `0.996736454607` for pass pair 1 and `0.996872556301` for pass pair 2. Each normalized cross delta is `100 * ((candidate/base) / whole-run-median-ratio - 1)`. The first numeric column is the signed base pass 1 to base pass 2 delta. Assessment uses the corresponding unsigned same-binary spread, `100 * (max(base1, base2) / min(base1, base2) - 1)`: a row says `inside own spread` rather than treating such motion as a finding, and `not replicated outside spread` whenever both cross passes do not clear that row-specific envelope in the same direction.

| Label | Base 1 -> Base 2 | Normalized C1/B1 | Normalized C2/B2 | Assessment |
|---|---:|---:|---:|---|
| `association-part-key-read-width4000` | +4.74% | +1.71% | -3.70% | inside own spread |
| `association-part-key-read-width32000` | +4.02% | -1.87% | -5.62% | not replicated outside spread |
| `association-part-positional-read-width4000` | -3.90% | -5.22% | -0.89% | not replicated outside spread |
| `association-part-positional-read-width32000` | -5.69% | -5.19% | +0.64% | inside own spread |
| `loop-empty` | -0.15% | -0.33% | -0.57% | faster twice, outside spread |
| `do-count-null` | -0.06% | +3.48% | -0.01% | not replicated outside spread |
| `do-iterator-null` | +0.34% | +1.93% | -0.53% | not replicated outside spread |
| `table-count-null` | -0.54% | +8.06% | +5.06% | slower twice, outside spread; both >3% |
| `table-iterator-null` | +1.40% | +8.94% | +7.78% | slower twice, outside spread; both >3% |
| `table-iterator-symbol` | -1.96% | +4.28% | +6.18% | slower twice, outside spread; both >3% |
| `literal-integer` | -2.95% | +0.19% | +3.56% | not replicated outside spread |
| `literal-string` | -2.85% | -4.29% | -0.47% | not replicated outside spread |
| `symbol-ownvalue-read` | -0.23% | -1.12% | +0.38% | not replicated outside spread |
| `symbol-ownvalue-guarded-read` | +1.13% | -1.33% | -2.47% | faster twice, outside spread |
| `set-ownvalue` | +0.77% | -0.32% | -1.36% | not replicated outside spread |
| `list-literal-small` | +1.25% | -1.58% | -2.75% | faster twice, outside spread |
| `head-list` | -0.11% | +0.00% | +0.04% | inside own spread |
| `length-list` | +0.29% | +1.07% | +0.25% | not replicated outside spread |
| `part-list` | -0.07% | -0.00% | -1.20% | not replicated outside spread |
| `sameq-integer` | +2.18% | -0.57% | -2.95% | not replicated outside spread |
| `sameq-list` | -0.21% | -1.05% | -0.04% | not replicated outside spread |
| `if-true` | +0.24% | -1.77% | -1.73% | faster twice, outside spread |
| `module-local` | -4.02% | -2.95% | +0.13% | inside own spread |
| `module-chain-escape-small` | +0.81% | +3.32% | +2.07% | slower twice, outside spread |
| `module-chain-escape-large` | -0.23% | +3.84% | +2.16% | slower twice, outside spread |
| `block-local` | -6.03% | -4.13% | +0.13% | inside own spread |
| `block-symbol-only-repeat` | +7.39% | +0.61% | -5.58% | inside own spread |
| `block-init-null-repeat` | -1.89% | -0.35% | +2.29% | not replicated outside spread |
| `block-init-read-repeat` | -0.86% | -1.63% | -0.02% | not replicated outside spread |
| `with-local` | -3.62% | -3.71% | -1.47% | inside own spread |
| `table-small` | -1.72% | -1.72% | -0.10% | inside own spread |
| `table-iterator-plus` | +0.38% | +2.96% | +3.13% | slower twice, outside spread |
| **`table-iterator-pair-list`** | -0.21% | +3.71% | +1.41% | slower twice, outside spread |
| `do-small` | -1.19% | -2.23% | -1.58% | faster twice, outside spread |
| `downvalue-exact-hit` | -0.20% | -0.58% | -1.20% | faster twice, outside spread |
| `downvalue-exact-miss` | +2.63% | +1.32% | -1.65% | inside own spread |
| `downvalue-pattern-hit` | -0.99% | -1.30% | +1.57% | not replicated outside spread |
| `downvalue-pattern-miss` | +1.08% | +0.16% | -1.11% | not replicated outside spread |
| `downvalue-exact-miss-repeat` | +4.69% | +1.69% | -2.98% | inside own spread |
| `downvalue-pattern-miss-repeat` | +3.97% | +0.89% | -2.75% | inside own spread |
| `downvalue-guarded-hit` | -0.84% | -0.28% | +2.61% | not replicated outside spread |
| `downvalue-guarded-decline-fallback` | +0.64% | +1.46% | +0.49% | not replicated outside spread |
| `upvalue-guarded-hit` | -4.07% | -3.58% | +0.71% | inside own spread |
| `subvalue-unguarded-hit` | +0.79% | +2.26% | +2.43% | slower twice, outside spread |
| `subvalue-unguarded-miss` | +0.42% | +0.88% | +0.79% | slower twice, outside spread |
| `subvalue-guarded-hit` | +0.74% | +0.28% | +0.61% | inside own spread |
| `default-unguarded-hit` | -1.24% | -1.53% | -0.35% | not replicated outside spread |
| `default-unguarded-miss` | -1.95% | -1.80% | -0.85% | inside own spread |
| `default-guarded-hit` | -0.12% | -0.78% | -0.07% | not replicated outside spread |
| `matchq-integer-success` | +0.64% | -1.23% | -1.89% | faster twice, outside spread |
| `matchq-integer-fail` | +0.43% | -0.98% | +0.05% | not replicated outside spread |
| `matcher-single-tiny` | -1.50% | -2.03% | -0.42% | not replicated outside spread |
| `matcher-single-wide-small` | -4.83% | -4.05% | -1.02% | inside own spread |
| `matcher-single-wide-large` | -6.16% | -6.40% | -0.11% | inside own spread |
| `replaceall-simple` | +0.18% | -1.21% | -1.39% | faster twice, outside spread |
| `rewrite-guarded-hit` | -0.27% | -1.21% | -0.32% | faster twice, outside spread |
| `rewrite-guarded-decline-fallback` | +0.04% | +1.43% | +1.46% | slower twice, outside spread |
| `cases-wide-no-match` | +0.19% | +2.80% | +0.75% | slower twice, outside spread |
| `count-wide-no-match` | +0.13% | +1.60% | +2.50% | slower twice, outside spread |
| `position-wide-no-match` | +0.21% | +0.90% | +2.11% | slower twice, outside spread |
| `deletecases-wide-no-match` | +1.38% | +1.26% | -0.75% | inside own spread |
| `deletecases-wide-association-no-match` | +1.74% | +3.20% | -2.00% | not replicated outside spread |
| `replace-wide-no-match` | +0.12% | +2.98% | +2.54% | slower twice, outside spread |
| `replacepart-wide-rule-list` | +0.34% | -0.23% | +1.45% | not replicated outside spread |
| `replacepart-guarded-hit` | +0.95% | +6.15% | +0.72% | not replicated outside spread |
| `replacepart-guarded-decline-fallback` | +0.28% | +5.28% | +1.83% | slower twice, outside spread |
| `string-repeat-small` | +0.75% | -0.54% | -1.48% | not replicated outside spread |
| `character-range-small` | +0.68% | +4.55% | +7.29% | slower twice, outside spread; both >3% |
| **`stringtrim-literal-list-4000`** | +0.29% | +0.95% | -0.01% | not replicated outside spread |
| **`stringtrim-literal-list-32000`** | -0.23% | -0.31% | -0.38% | faster twice, outside spread |
| `stringtrim-single-literal-4000` | +0.03% | -0.03% | -0.03% | faster twice, outside spread |
| **`stringtrim-single-literal-32000`** | -0.14% | -0.25% | -0.42% | faster twice, outside spread |
| `stringtrim-whitespace-4000` | -0.24% | -0.51% | -0.12% | not replicated outside spread |
| `stringtrim-whitespace-32000` | -0.09% | -0.15% | -0.40% | faster twice, outside spread |
| `stringreplace-single-literal-unguarded` | -0.64% | -0.33% | +0.28% | inside own spread |
| `stringreplace-pattern-unguarded` | -0.26% | +0.30% | -0.34% | not replicated outside spread |
| `stringreplace-guarded-hit` | +0.10% | +1.56% | +0.15% | slower twice, outside spread |
| `stringreplace-guarded-decline-fallback` | -0.68% | +0.68% | +0.50% | inside own spread |
| **`appendto-list-incremental-small`** | -0.34% | +5.46% | +5.88% | slower twice, outside spread; both >3% |
| **`appendto-list-incremental-large`** | -0.52% | +5.43% | +6.12% | slower twice, outside spread; both >3% |
| **`appendto-part-outer1000-small`** | -0.40% | +3.78% | +3.64% | slower twice, outside spread; both >3% |
| **`appendto-part-outer1000-large`** | -0.21% | +4.40% | +4.61% | slower twice, outside spread; both >3% |
| `association-set-key-incremental-small` | -0.73% | +1.00% | +1.53% | slower twice, outside spread |
| `association-set-key-incremental-large` | +0.03% | +1.50% | +1.03% | slower twice, outside spread |
| `association-small-literal-repeat` | -1.45% | +0.42% | +1.46% | inside own spread |
| **`set-part-fixed100-len100`** | -0.30% | +7.17% | +6.71% | slower twice, outside spread; both >3% |
| **`set-part-fixed100-len800`** | +0.27% | +9.34% | +8.64% | slower twice, outside spread; both >3% |
| **`set-part-fill-small`** | +0.13% | +8.53% | +7.72% | slower twice, outside spread; both >3% |
| **`set-part-fill-large`** | -0.78% | +8.96% | +8.86% | slower twice, outside spread; both >3% |
| `upvalue-candidates-tiny` | -0.87% | -0.10% | +0.18% | inside own spread |
| `upvalue-candidates-wide-small` | -0.57% | -0.67% | +1.51% | not replicated outside spread |
| `upvalue-candidates-wide-large` | +0.87% | +0.27% | +1.32% | not replicated outside spread |
| `sort-callback-small` | +0.18% | -2.53% | -1.85% | faster twice, outside spread |
| `sort-callback-large` | +1.28% | -1.05% | -2.19% | not replicated outside spread |
| `sortby-callback-small` | +5.18% | -0.05% | -4.94% | inside own spread |
| `sortby-callback-large` | +6.62% | -1.51% | -6.56% | inside own spread |
| `reversesort-callback-small` | +0.98% | +1.04% | +0.01% | not replicated outside spread |
| `reversesort-callback-large` | -0.66% | -0.58% | +1.40% | not replicated outside spread |
| `keysort-callback-small` | -2.99% | -3.26% | -0.59% | not replicated outside spread |
| `keysort-callback-large` | -3.88% | -3.46% | -0.53% | inside own spread |
| `keysortby-callback-small` | +3.99% | +0.42% | -2.92% | inside own spread |
| `keysortby-callback-large` | +5.69% | +0.56% | -4.17% | inside own spread |
| **`sort-default-wide`** | +0.53% | +4.10% | +2.96% | slower twice, outside spread |
| `sort-symbol-default-width4000` | +0.14% | +3.34% | +2.64% | slower twice, outside spread |
| `sort-symbol-default-width16000` | -1.30% | +0.78% | -0.26% | inside own spread |
| `sortby-default-wide` | -1.10% | -2.03% | -2.50% | faster twice, outside spread |
| **`reversesort-default-wide`** | +0.28% | +5.99% | +3.97% | slower twice, outside spread; both >3% |
| **`keysort-default-wide`** | -0.22% | +1.02% | +1.55% | slower twice, outside spread |
| `keysortby-default-wide` | +2.44% | -0.04% | -2.31% | inside own spread |
| `valueq-undefined-symbol-definitions-2000` | -0.74% | -0.63% | -0.57% | inside own spread |
| `valueq-undefined-symbol-definitions-8000` | -0.49% | -0.42% | -0.52% | not replicated outside spread |
| `module-retained-gensym-gc-batch` | -5.94% | -9.44% | +3.60% | not replicated outside spread |

### Required row callouts

- `stringtrim-literal-list-32000`: -0.31% / -0.38%, faster twice outside its 0.23% base spread; no regression finding.
- `stringtrim-literal-list-4000`: +0.95% / -0.01%; the direction does not replicate.
- `stringtrim-single-literal-32000`: -0.25% / -0.42%, faster twice outside its 0.14% base spread; no regression finding.
- `set-part-fixed100-len800`: +9.34% / +8.64%, slower twice outside its 0.27% base spread and above 3% twice.
- `set-part-fixed100-len100`: +7.17% / +6.71%, slower twice outside its 0.31% base spread and above 3% twice.
- `set-part-fill-large`: +8.96% / +8.86%, slower twice outside its 0.79% base spread and above 3% twice.
- `set-part-fill-small`: +8.53% / +7.72%, slower twice outside its 0.13% base spread and above 3% twice.
- `sort-default-wide`: +4.10% / +2.96%, slower twice outside its 0.53% base spread; only the first replicate exceeds 3%.
- `reversesort-default-wide`: +5.99% / +3.97%, slower twice outside its 0.28% base spread and above 3% twice.
- `keysort-default-wide`: +1.02% / +1.55%, slower twice outside its 0.22% base spread but below 3%.
- `table-iterator-pair-list`: +3.71% / +1.41%, slower twice outside its 0.21% base spread; only the first replicate exceeds 3%.
- `appendto-part-outer1000-small`: +3.78% / +3.64%, slower twice outside its 0.40% base spread and above 3% twice.
- `appendto-part-outer1000-large`: +4.40% / +4.61%, slower twice outside its 0.21% base spread and above 3% twice.
- `appendto-list-incremental-small`: +5.46% / +5.88%, slower twice outside its 0.34% base spread and above 3% twice.
- `appendto-list-incremental-large`: +5.43% / +6.12%, slower twice outside its 0.52% base spread and above 3% twice.

## 5. Stop-loss, correctness, and verdict support

The stop-loss is met without tuning: all four Set/Part rows, all four required incremental AppendTo rows, and the `table-count-null` / `table-iterator-null` / `table-iterator-symbol` construction family are coherently slower in both normalized replicates, outside their own same-binary spreads, by more than 3%. `reversesort-default-wide` and `character-range-small` independently meet the row-level condition too. The candidate is therefore rejected as measured.

The focused reference test passed verbatim:

```text
running 1 test
test expr::tests::physical_height_matches_recursive_reference_across_construction_routes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
```

The first unqualified workspace run compiled and passed all non-socket suites reached, then failed only because this sandbox rejects native loopback binds:

```text
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
error: test failed, to rerun pass `-p rum-jupyter --lib`
```

The documented sandbox rerun, `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace`, exited 0. Representative verbatim terminal results include:

```text
test result: ok. 843 passed; 0 failed; 15 ignored; 0 measured; 0 filtered out; finished in 4.18s

   Doc-tests rum_host

running 1 test
test crates/rum-host/src/lib.rs - (line 12) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.13s

   Doc-tests rum_terminal

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo fmt --all --check` also exited 0 with no output. No source was changed after the candidate binary was built, apart from appending this report.

## Director's gate (2026-09-03)

**Rejection confirmed, and the mechanism is now isolated to one term.** I
re-ran the four passes on the worker's own binaries (base `e28bc552`, which is
HEAD, and candidate `48894a1c`). Whole-panel median offset +0.01% — a clean
window — with these results:

| Row family | My measurement | Same-binary |
|---|---:|---:|
| `stringtrim-*` (six rows) | **−0.40% to +0.93%** | ≤0.60% |
| `table-count-null` | +9.50% | +2.66% |
| `module-retained-gensym-gc-batch` | +8.85% | −1.12% |
| `table-iterator-null` | +8.54% | −2.24% |
| `appendto-list-incremental-small` | +6.69% | +0.85% |
| `table-iterator-pair-list` | +6.55% | −2.75% |
| `set-part-fill-small` | +6.28% | +6.85% |
| `set-part-fixed100-len100` | +6.15% | +4.97% |

**The `stringtrim` rows are the finding.** They cost +7.4% to +14.2% when the
height lived behind the `Arc` and are at parity now. The inline representation
did exactly what it was built to do: the allocation-growth term is gone.

What is left is the other term, and it is unambiguous: **computing
`max` over the arguments at every call construction costs +6% to +9.5% on the
construction and mutation families.** It is not the allocation, not the pointer
chase, and not the mutation strategy — those have each been separately measured
and separately removed. It is the pass itself.

### Where this leaves the line

Four spikes, four rejections, and between them a complete decomposition:

| Term | Where measured | Cost |
|---|---|---|
| 8 bytes per `Arc` allocation | brief-434 V1 vs V0 (director) | +5% to +14% |
| `SharedArgs::new` scan with Arc dereference | brief-434 V2/V1 | +3.9% to +11.4% |
| full rescan after every mutation | brief-434 V3/V2 | free |
| O(1) conservative maintenance | brief-434 V4/V2 | free |
| `max` over args at construction, inline, no allocation growth | brief-435 (this) | +6% to +9.5% |

Every design that computes the height **at construction** pays for the pass.
The one shape never tried is to not compute it there: have the caller supply
it, fused into work it is already doing — a Table builder tracks a running max
one compare per element it already holds, a Part assignment knows
`max(old, replacement)`, a sort of a permuted vector knows the max is
unchanged. That is the only place the remaining cost can go, and it trades the
compiler-enforced correctness of a computed height for a supplied one that only
a debug assertion can check.

That trade is a decision for the user, not for me.
