# brief-600 — design: G13, typed continuation state crosses frames in-band

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

The suspension-continuation branch closed G1-G12 and E1 (briefs 548-599): every
declared guest-effect entry of the installed catalog has an executed descriptor
in the rum-vs-rum oracle (resumed path == queued-input path, suspending in the
middle and twice, lifecycle at both pauses). brief-597's sweep found **G13**,
which is not a lost continuation but a flaw in the channel continuations use.
This brief designs its repair. The repair must be structural — the compiler
rejects a wrongly typed completion, and guest code cannot construct anything a
decoder accepts as typed state — and its relation to E2 (a may-suspend result
type with no `Try`) is part of the design.

## The problem (verified by the director at the 597 gate)

- Frames complete with `Result<Expr, EvalError>` (`FrameAction::Complete`,
  `crates/rum-runtime/src/runtime/suspension/core.rs:100-135`). A function
  returning `Result<T, EvalError>` that suspends inserts a frame whose
  completed `Expr` the caller's frame decodes with `T::from_continuation_expr`
  (`ContinuationValue`, `core.rs:137-141`; 25 impls) or with an ad-hoc decoder
  such as `prepared_call_arguments_from_continuation_expr`
  (`eval_call_prepare.rs:145-185`). Production code uses 206 distinct
  ``System`Internal*`` heads.
- The heads are ordinary ``System`` symbols matched by text
  (`is_system_symbol_name`, `expr_runtime_helpers.rs:420`): **guest source can
  write them.**
- The decoders are lenient — `Option<Expr>` reads any unmarked value as `Some`
  (`suspension/primitive.rs:36-66`), `Vec<Expr>` as a one-element vector
  (`:68-83`), `bool` anything but `True` as false (`:13-21`), `String` via
  `to_string` (`:23-34`) — or they `panic!`/`expect`/`assert!` on an
  unexpected shape (`primitive.rs:85-153`; `eval_call_prepare.rs:155-185`;
  `eval_core/call.rs:322-323`; many sites in `error.rs`,
  `pattern_matching/continuation.rs`,
  `scoping/lexical/block/continuation.rs`).
- Nothing checks that a slot is filled by its own type's encoder. brief-597
  found three frames completing an `Option<Expr>` slot with a raw value:
  `FinishCasesRuleMatchSuspensionFrame` (`cases/collect/resume_frames.rs:295`),
  `TryApplyFunctionSuspensionFrame` (`scoping/callable.rs:141`),
  `FinishPositionalPartSelectorEvaluationSuspensionFrame`
  (`part_assignment/resume_frames.rs:5`). The lenient decoder is why ordinary
  values never showed it.
- Director-reproduced witnesses (resumed against queued;
  `.temp/brief-597/sweep.rs`, `.temp/director/gate-597/g13-witness.txt`):
  a guest value ``System`InternalContinuationNone[]`` is decoded as `None` by
  `resume_suspended_call_dispatch_frame` (`eval_core/call.rs:291-338`), which
  moves to the next dispatch stage, and the resumed call asks for input a
  second time; ``Function[(InputString["sweep"];System`InternalContinuationSome[5])][0]``
  resumes to `5` instead of `InternalContinuationSome[5]`; with
  `InternalContinuationSome[1,2]` a debug build panics at
  `suspension/primitive.rs:51` (a `debug_assert`; release would return `1` —
  inferred, not run).

## Resume — a previous run's checkpoint exists

The first run (Codex) was stopped by an upstream content filter at ~298k
tokens, mid-way through section 1; its log is
`.temp/logs/cdx_brief_600.filter-1.log`. This run continues it. What it left:

- `.temp/brief-600/PROGRESS.md` — counters and the exact next action.
- Inventory drafts under `.temp/brief-600/` (`counts.json`, `decoders.tsv`,
  `decode-sites.tsv`, `encoders.tsv`, `completion-sites.tsv`,
  `slot-producers.tsv`, `verified-slot-edges.tsv`, `reachability.tsv`,
  `internal-heads.tsv`, `marker-corpus.tsv`, `falsifier-descriptors.tsv`,
  `queue-producer-edges.tsv`, …) and the scripts that built them
  (`inventory_base.py`, `inventory_tables.py`, `producer_tables.py`,
  `queue_edges.py`, `behavior.py`, and raw `ast-*.tsv` exports, some of them
  megabytes — query them with scripts, do not read them whole).
- `witnesses.tsv`: 15 executed observations, 6 equal controls and 9
  divergences, none panicking. The scratch test
  `crates/rum-stdlib/tests/zz_brief600_probe.rs` is still in place (a copy is
  at `.temp/brief-600/probe-run1-snapshot.rs`); keep using it and delete it
  before appending the report, as Scope says.

These are drafts from an interrupted run: **verify before relying on them**
(spot-check counts and verdicts against source), then finish sections 1-7.
Its census counted 564 `SuspensionFrame` and 59 `PureFrameFinalizer`
implementors, 25 `ContinuationValue` impls, 134 decoder bodies, 643 decode
sites and 206 internal heads (203 continuation markers).

## What the design must settle

1. **Inventory.** Every encoder/decoder pair and every decode site, with its
   behaviour on (a) a raw ordinary value, (b) another type's marker, (c) a
   malformed marker of its own type: lenient / error / panic. For every
   decoded slot, every frame or finalizer that fills it, with a verdict:
   encodes with the slot's encoder / completes raw / completes with another
   type's encoding. Classify all ``System`Internal*`` heads: continuation-state
   markers versus other internal uses (which are out of scope unless they are
   decoded as typed state). Recount the `SuspensionFrame` and
   `PureFrameFinalizer` implementors, the `ContinuationValue` impls and the
   ad-hoc decoders; do not rely on the numbers above.
2. **Reachability, by source reading.** For each lenient or panicking decoder:
   can guest code deliver a forged shape to it, and by which path? Also every
   path by which an encoded value reaches guest-visible state (a result,
   `Sow`/`Reap`, a trace, a message, a definition), where it could be captured
   and replayed. **Executable witnesses only for value divergence** (resumed
   against queued, no panic), in the scratch test. For a decoder that would
   panic, give the guest program and the path; **do not run it** — the
   director reproduces those.
3. **The repair design**, meeting three requirements:
   - (i) a frame cannot complete a slot with the wrong type — **a compile
     error, not a convention**;
   - (ii) guest code cannot construct anything a decoder accepts as typed
     state — **out of band**, not a renamed or "sealed" symbol;
   - (iii) decoding typed state cannot panic or misread on guest input; any
     residual runtime check is an internal invariant that only a kernel bug
     can violate, stated with why guest input cannot reach it.

   Compare, with evidence, at least: a typed output bound at insertion (the
   frame names the type it completes; the insertion point in a function
   returning `Result<T, _>` accepts only frames completing `T`; the driver
   adapts at the type-erased queue); a typed out-of-band payload carried by
   `FrameAction::Complete` / the resume outcome; and sealed tag identities —
   stating what those do not guarantee (a raw value of the wrong type; replay
   of a leaked encoding). Say where guest values legitimately travel as `Expr`
   and must stay `Expr`.
4. **Relation to E2** (`.temp/brief-583/e2-outline.md`: a may-suspend result
   type with no `Try`, typed continue descriptors, explicit output codecs):
   is G13's repair a foundation of E2, independent of it, or subsumed by it?
   Can it be built on today's `Result<T, EvalError>`, or does it need E2's
   type first? Recommend the sequence with evidence. If one migration should
   carry both, say so and lay out its steps; E2's representation choice still
   requires its measured prototype, so do not decide that here.
5. **A class-wide falsifier**, rum against rum, so G13's class cannot return:
   for example an oracle variant in which a callback or body returns each
   continuation-state marker shape and the resumed path must equal the queued
   path. Say which descriptors it applies to, that it is red today on the
   three frames, and what it checks once markers no longer exist in guest
   space.
6. **Ripple and cost.** The full ripple — `SuspensionFrame` implementors,
   `FrameAction`, `PureFrameFinalizer`, the resume driver, `kernel_api`
   exports, `rum-derive`, the frame-root trace inventory, the ownership and
   retention inventories and fingerprints, the oracle, the existing
   continuation suites. Which ordinary-path functions change shape (the frame
   is built only on suspension; anything that touches a hot return path is
   named), and the one-codegen-unit instruction gate plan: labels, `N`/`2N`,
   build both arms at the same path.
7. **Drafted implementation briefs** in `TEMPLATE.md` shape, one per gated
   step, small enough that each step lands green; the three witnesses and any
   new value-divergence witnesses become reds; each step keeps the oracle
   green in both builds.

## What I want back

1. The inventory tables (`.temp/brief-600/*.tsv`) and their counts.
2. The reachability table, the executed value-divergence witnesses with
   output, and the not-run panic paths for the director.
3. The repair design with the comparison, the E2 relation and sequence, the
   falsifier, the ripple and the gate plan.
4. The drafted implementation briefs.
5. Open questions for the director.

## Context pointers

- brief-597's report and sweep (`docs/dev/codex/brief-597-*.md`,
  `.temp/brief-597/`), `.temp/director/gate-597/g13-witness.txt`.
- `crates/rum-runtime/src/runtime/suspension/`, the frames named above, the
  resume driver, `rum-derive`'s frame derives, `kernel_api`.
- `.temp/brief-583/e2-outline.md`, `.temp/brief-548/enforcement.md`.
- The oracle: `crates/rum-stdlib/tests/suspension_oracle/`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-600/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief600_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-600/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-600/`.
- **Run no probe that panics or aborts.** A probe that would reach a panic is
  reported, not run.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief600_probe`
  (300 s); `python3` scripts (300 s each). File reads, `rg`/`grep`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No production code.** No new `unsafe`.
- Checkpoint to `.temp/brief-600/PROGRESS.md` after each section with the
  exact next action.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The inventory and the reachability.
3. The repair design, the E2 relation and sequence.
4. The falsifier, the ripple and the gate plan.
5. The drafted implementation briefs.
6. Open questions for the director.

---
# brief-600 report — G13 design (run 2, resumed from the filtered run 1)

## 1. What changed

Design only. No production source change. Scratch under `.temp/brief-600/`
(inventory tables and scripts from run 1, re-verified and extended; new:
`find_raw_completers.py`, `write_raw_classification.py`,
`raw-completers-classification.tsv`, `heads-verify.txt`, extended
`witnesses.tsv`, `probe.rs`). The one scratch integration test
`crates/rum-stdlib/tests/zz_brief600_probe.rs` was kept in place during the
run, extended with three witness families, executed green twice, copied to
`.temp/brief-600/probe.rs`, and **deleted** before this report was appended.
No `git` command was run. No probe that panics was run.

## 2. Inventory (brief §1)

### 2.1 Counts — run-1 census re-verified independently

| item | count | verification |
|---|---|---|
| `SuspensionFrame` impls (production) | **564** (364 rum-runtime + 200 rum-stdlib) | fresh `rg` block count: 368 runtime − 4 in `src/runtime/tests/*`, 201 stdlib − 1 oracle `mutations/driver.rs` |
| `PureFrameFinalizer` impls | **59** (58 runtime + 1 stdlib `resumable_merge_sort.rs`) | fresh `rg` block count |
| `ContinuationValue` impls | **25** | enumerated by type name (bool, String, Expr, Option<Expr>, Vec<Expr>, Vec<SymbolExprMap>, Option<Vec<Expr>>, Option<Vec<RewriteRuleInfo>>, Option<CasesLevelSpec>, Vec<ReplacePartTarget>, CasesTraversalConfig, CasesTraversalResolution, CasesTraversalNormalizationError, TraversalWithoutLimitResolution, TraversalWithoutLimitNormalizationError, CasesCollectionOutcome, PositionCollectionOutcome, DeleteCasesOutcome, TraversalCountOutcome, ReplaceOutcome, ReplaceAllTraversalOutcome, CompiledMatchOutcome, HeadsOptionClassification, EvalCapturedOutcome, EvalError) |
| decoder bodies | 134 (25 ContinuationValue, 38 ad-hoc, 38 helper, 33 inline) | run-1 AST tables, spot-checked |
| decode sites | 643 | run-1 table, row count re-checked |
| encoder bodies | 143 | run-1 table, spot-checked |
| `System`Internal*` heads | **206** | fresh `rg '"Internal[A-Za-z0-9]+"'` over runtime+stdlib src: exact set match with `internal-heads.tsv` (zero diff both directions) |
| continuation-state heads | **203** (incl. 5 `InternalInterpreted*` without the `Continuation` prefix) | classification verified at `pattern_matching_interpreted.rs:594,717` (encoded+decoded as typed state) |
| other internal heads | 3: `InternalConfirmTag` (default tag value), `InternalEncloseTag` (gensym seed), `InternalConfirmAutomaticInfo` (lenient Option-like sentinel: `confirmation.rs:103` treats it as "no info") | out of scope per brief; the third is noted because it is the same *lenient sentinel* shape |

### 2.2 Decoder behaviour on the three shapes

`decoders.tsv` (134 rows) carries per-decoder verdicts on (a) raw ordinary
value, (b) another type's marker, (c) malformed own marker. Summary by class:

- **Lenient (misread)**: `Option<Expr>` (`primitive.rs:44-65`: raw → `Some(raw)`;
  `Some[]` → `None`), `Option<Vec<Expr>>` (`:166-199`: bare List accepted,
  `Some[]` → empty vec), `Vec<Expr>` (`:73-83`: raw → singleton),
  `bool` (`:18-21`: anything non-`True` → false), `String` (`:28-33`: Display),
  `failure_type_from_string` (unknown label → Error),
  `rewrite_rule_info_from_expr` (bad arity → whole expr becomes lhs),
  `EvalCapturedOutcome` (wrong arity → Error capture), `CompiledMatchOutcome`
  (`outcome.rs:31-49`: ordinary `List[True, List[]]` IS its encoding — a
  guest-writable shape decodes as matched-with-bindings).
- **Panicking**: `Option<Expr>` 2-arg `Some` (`debug_assert`, `primitive.rs:51`),
  `Option<Vec<Expr>>` multi-slot (`:175-190`), `Vec<SymbolExprMap>`
  (`:108-153` `panic!`/`expect`), `CompiledMatchOutcome` (`outcome.rs:47`
  release `panic!` on non-List; `:52-80` `panic!`/`assert!` on bad bindings),
  `EvaluatedPartSelectors` (`evaluated_part_selectors.rs:110` release `panic!`),
  `prepared_call_arguments_from_continuation_expr`
  (`eval_call_prepare.rs:145-185`), `decode_prepared_interpreted_*`
  (`pattern_matching_interpreted.rs:606-625` `assert!`/`expect`),
  `call.rs:322-323` `.expect` on `None` completing a prepare stage,
  `binding_map_from_continuation_expr`, symbol/string/usize decoders on
  non-atoms. Full per-site detail in `decoders.tsv` columns `raw_ordinary`,
  `other_type_marker`, `malformed_own`.
- **Erroring (not panicking)**: `prepared_call_arguments_from_continuation_expr`
  returns `Err(original)` on a non-wrapper (falls into `call.rs:322`);
  `runtime_messages_from_continuation_expr` / `strings_from_continuation_expr`
  return `None` on bad lists.

### 2.3 Slot producers — every decoded slot vs its fillers

`slot-producers.tsv` (643 rows, source-family joins),
`producer-families.tsv` (607 candidate edges),
`verified-slot-edges.tsv` (6 hand-verified edges), and — new this run —
**`raw-completers-classification.tsv`**: I re-derived the raw-completer set
from scratch (run 1's table hard-coded the three known names, which was
circular). My scan found **30 frames that complete raw while their insertion
owner returns a codec ok-type**, plus 14 raw completers without a macro
insertion row. Source-reading each one:

- **REAL — completes raw into a typed slot, guest value divergence: 5 frames**
  1. `TryApplyFunctionSuspensionFrame` (`scoping/callable.rs:141-153`) — slot
     `Option<Expr>` of `try_apply_function`; completes
     `finish_function_application(outcome)` raw; missing `map(Some)`+encoder.
  2. `FinishCasesRuleMatchSuspensionFrame`
     (`kernel_data/rewrite_data/cases/collect/resume_frames.rs:295-309`) —
     slot `Option<Expr>` of `finish_cases_rule_match`; `Ok(value)` raw.
  3. `FinishPositionalPartSelectorEvaluationSuspensionFrame`
     (`definitions/part_assignment/resume_frames.rs:5-27`) — slot
     `Option<Expr>` of `try_set_positional_part_lvalue`; completes
     `finish_positional_part_assignment` raw. (Its *input* decode
     `EvaluatedPartSelectors::from_continuation_expr` is a separate, protected
     edge — the selector chain encodes.)
  4. `FinishAssociationSetPartSelectorEvaluationSuspensionFrame`
     (`kernel_data/association_data/resume_frames.rs:11-33`) — slot
     `Option<Expr>` of `try_set_association_lvalue`; same shape as (3).
     **New this run**, witnessed (§3.2).
  5. `RunCallDispatchStageSuspensionFrame` (`eval_dispatch/mod.rs:466-490`) —
     slot `Option<Expr>` of `try_dispatch_builtin_call` /
     `suspend_threaded_call_dispatch`; completes `Ok(value)` raw for any
     builtin whose suspended execution resumes to a value. **New this run**,
     witnessed via `Fold` and `Nest` (§3.2).
- **Benign — helper/finalizer encodes, or the completion always wraps**: 21
  frames (`complete_interpreted_match_action` `pattern_matching_interpreted.rs:470-477`
  encodes; `complete_guard_frame` `rhs_guards/resume_frames.rs:103-115`
  finalizes; `prepare_orderless_optional_action`
  `orderless/optional/resume_frames.rs:27-42` finalizes;
  `finish_suspended_replace_action` `replace/once/resume_frames.rs:586-597`
  finalizes; `finish_suspended_replace_part_targets_action`;
  `RenderDelayedMessageTextFrame` completes `Expr::string(...)` which IS the
  String codec; `FailureVirtualStyledMessageValueSuspensionFrame` always wraps
  in `styled_message_expr`, so a marker-valued message cannot pass bare).
- **Consumers over protected producers**: 3 stdlib lazy-completion frames
  (`First`/`Normal`/`Take` decode `Option<Expr>` at their
  `lazy_completion.rs:33`); their producer `LazyRealizeSuspensionFrame`
  encodes `Some(List[...]).into_continuation_expr()` at
  `stdlib/src/lazy/continuation.rs` `resume_realization` — guest element
  values are nested inside the encoder's wrapper.
- **Witness-pending: 1** — `FailureMessageFromTemplateSuspensionFrame`
  (`control_data/status/resume_frames.rs:26-52`) completes
  `finish_failure_message_from_template` (`status.rs:250`) raw over an
  `Option<Expr>` slot; whether a guest Failure template can make the completed
  value a *bare* marker needs the falsifier corpus (§5).

### 2.4 Where the channel is typed today only by convention

`resume_or_suspend!` pairs a frame with its slot only by the author writing
matching encoder/decoder calls; nothing checks it. The ok-arm of the cold
re-entry form (e.g. `strings/patterns/search/rules.rs:239-255`) is the
documented twin of the frame body (`rules/resume_frames.rs:17-51`), which
repeats it with decode prepended and re-encode appended — the same manual
convention G9/G10 already showed can drop a cursor or a `map(Some)`.

## 3. Reachability (brief §2)

### 3.1 The two guest-value entry routes (source reading)

- **Route E — `FrameAction::Evaluate`**: the driver runs
  `self.evaluate(expr)` (`suspension/core.rs:749-758`) and hands the raw
  guest-authored result to `next_frame`. Production has exactly **2** such
  sites: `pattern_matching/core/resume_frames.rs:785`
  (`ContinueSuspendedSequenceCheckFrame` — checks `True` on the raw value: a
  semantic predicate check, not a state decode) and
  `stdlib/src/structure_ops/resumable_merge_sort.rs:340` (callback values into
  the merge). Neither decodes markers. Route E is small and clean.
- **Route R — `resume_or_suspend!` frames**: a frame inserted at a callee's
  `InputRequired` receives, on resume, whatever the callee's own frame chain
  finally completes. When the callee (transitively) evaluates guest code —
  `Session::evaluate`, function bodies, rule RHSs, selectors, callback
  arguments — the completing value is guest-authored. `queue-producer-edges.tsv`
  maps this: 629 source-resolved edges, 351 OPEN (helper-result parameters),
  of which 89 resolve through `resolution.rs:318 Session::evaluate`. The five
  REAL frames are exactly the Route-R sites whose bodies complete that guest
  value raw into a typed slot. Everything else on Route R either passes the
  value through as an identity slot (`T = Expr`) or re-encodes.
- Guest input cannot forge markers through the *seed* outcome
  (`Ok(Expr::string(input))`, `core.rs:607`) — it is always a String atom.

### 3.2 Executed value-divergence witnesses (resumed vs queued, no panic)

`witnesses.tsv` now holds **30 observations** (6 families × 5 shapes): 12
equal controls (`ordinary`, `other_type`), **18 divergences** (`none`,
`some`, `empty_some` × 6), zero panics. Verbatim rows for the new families
(`assoc`, `fold`, `nest` — added this run; the `function`, `cases`, `part`
rows are run 1's, unchanged):

```
assoc.none   queued: value=InternalContinuationNone[];...  resumed: UnexpectedPause(1)   false
assoc.some   queued: value=InternalContinuationSome[5];... resumed: value=5;...          false
fold.some    queued: value=InternalContinuationSome[5];... resumed: value=5;...          false
nest.none    queued: value=InternalContinuationNone[];...  resumed: UnexpectedPause(1)   false
```

Sources (full table has all 30): `Global`swA=<|"k"->{0}|>;Global`swA[[(InputString["sweep"];1)]]=<marker>`
and `Fold[Function[(InputString["sweep"];<marker>)],0,{1}]` /
`Nest[...]`. `Fold` and `Nest` both route through
`RunCallDispatchStageSuspensionFrame` — one frame, several constructs,
confirming the class is per-frame. Probe run: `2 passed` (`ast_inventory`,
`non_panicking_value_witnesses`), log `probe-test.log`, copy at `probe.rs`.

### 3.3 Panic paths — reported, NOT run (for the director)

- **Debug `debug_assert` at `suspension/primitive.rs:51`** (2-arg
  `InternalContinuationSome`): guest program
  `Function[(InputString["sweep"];System`InternalContinuationSome[1,2])][0]`,
  resumed arm. Release build would return `1` (first arg) — inferred, not
  run. The same shape through `Cases[{1},x_:>(...)]`, both Part forms, and
  `Fold`/`Nest` reaches the same decoder.
- **Release panics with no known guest route** (each is an internal
  invariant; the stated reason guest input cannot reach it):
  - `eval_core/call.rs:322-323` `.expect` on `None` completing a prepare
    stage — requires a PrepareArguments suspension to complete as bare
    `InternalContinuationNone[]`; its only producer
    `FinishPreparedCallArgumentsFinalizer`
    (`eval_call_prepare/resume_frames.rs:79`) always wraps (verified edge).
  - `CompiledMatchOutcome::from_continuation_expr` `panic!` on non-List
    (`outcome.rs:47`) — every producer in its chains re-encodes
    (`complete_interpreted_match_action`, `GuardOutcomeFinalizer`, the
    matcher frames); guest guard values are consumed by the `True` check
    (`rhs_guards.rs:206`, `resume_frames.rs:71`) before any encoding.
  - `EvaluatedPartSelectors::from_continuation_expr` `panic!`
    (`evaluated_part_selectors.rs:110`) — producers encode
    (`FinishEvaluatedPartSelectorsSuspensionFrame` via the
    `Vec<Expr>` marker; `FinishArgumentResolutionFinalizer`
    `arguments.rs:230-234` encodes).
  - `prepared_call_arguments_from_continuation_expr` panics on malformed
    wrappers — wrappers are kernel-constructed; guest values only ever
    appear as nested argument fields.
  - `Vec<SymbolExprMap>` binding-map panics, `Option<Vec<Expr>>` 2-arg
    debug-asserts, `arguments.rs:47` accumulator-uniqueness panic —
    kernel-internal encodings / uniqueness invariants only.

### 3.4 Capture and replay of encodings (why sealing by name fails)

Guest does not need to *leak* a kernel encoding to forge state: the marker
heads are ordinary `System` symbols matched **by text**
(`is_system_symbol_name`, `expr_runtime_helpers.rs:420`), so guest source
writes them directly (all witnesses do), can store them
(`x = System`InternalContinuationSome[5]` then replay through any suspending
body), and can construct them dynamically (`Symbol`/`ToExpression`).
Queued-path *results* legitimately contain them whenever guest wrote them.
Additionally `CompiledMatchOutcome`'s own encoding (`List[True, List[Rule[...]]]`)
is built from guest-writable heads. Consequence: any repair based on renamed,
fresh, or "sealed" symbol identities fails requirement (ii) — secrecy is not
a type. Only payloads guest cannot represent (Rust values) satisfy it.

### 3.5 Where guest values legitimately travel as `Expr` and must stay `Expr`

Route E sub-evaluations; identity slots (`T = Expr`: loader forms, list
rebuilds, sort merges, message renders); guest-visible *fields* nested inside
typed state (selector values, argument values, lazy element values, binding
RHS values); the input-string seed; guard/predicate results before the
`True` check. The design keeps all of these as an explicit `Value` arm (§4).

## 4. The repair design (brief §3)

### 4.1 Requirements restated

(i) wrong-type completion is a **compile error**; (ii) guest cannot construct
anything a decoder accepts as typed state — **out of band**; (iii) decoding
cannot panic or misread on guest input; residual checks are internal
invariants with a stated reason guest cannot reach them.

### 4.2 The three options compared

**Option A — typed output bound at insertion** (the frame names the type it
completes; the insertion point in a function returning `Result<T, _>` accepts
only frames completing `T`; the driver adapts at the type-erased queue).
Satisfies (i) if and only if the frame's action type is parameterised by the
slot (see 4.3): the macro infers `T` from the suspended result and requires
`F: SlotFrame<T>`; completing `Ok(raw_expr)` into an `Option<Expr>` slot
stops compiling. Alone it does NOT satisfy (ii): the channel still carries
`Expr`, so guest-forged markers still decode (the three witnesses would still
diverge — the encoder would just be compiler-forced, and the lenient decoder
would still misread guest markers written *inside* legitimately-typed slots'
value fields when they arrive bare). Evidence: every divergence in
`witnesses.tsv` is a *decoder* misreading a guest value; forcing the encoder
changes none of them.

**Option B — typed out-of-band payload carried by `FrameAction::Complete` /
the resume outcome**: the channel carries
`FrameOutcome::Value(Expr) | FrameOutcome::State(Box<dyn ErasedSlotValue>)`.
Satisfies (ii) structurally (guest source cannot name a Rust enum arm; the
only `Expr` that enters is the explicit `Value` arm) and (iii) (decoding is
an exact-type downcast; a mismatch is a classified internal error, never a
panic, never a silent misread — the entire lenient-decoder class of §2.2
becomes unrepresentable). Alone it does NOT satisfy (i): a frame can still
put `Value(e)` where its consumer expects `State` — nothing binds the arm to
the slot type without A's insertion-point typing.

**Option C — sealed tag identities** (fresh/gensym'd or renamed heads).
Fails (ii) as shown in §3.4: guest can write or dynamically construct any
symbol; a leaked or guessed name replays; `List`-built encodings
(`CompiledMatchOutcome`) are not even name-sealable. Also fails (i) outright
(no type binding). What it would add over today: nothing enforceable.
**Rejected.**

**Chosen: A + B composed.** A supplies the per-edge compile-time binding at
insertion; B supplies the out-of-band channel. Each covers exactly the
requirement the other cannot. The existing `EvaluatedPartSelectorsContinuation`
(`Arc<OnceLock>`, `evaluated_part_selectors.rs:20-54`) is a one-off ancestor
of B's idea — a slot filled out-of-band by a private frame — and is subsumed.

### 4.3 The design in detail

Layer 1 — channel (`suspension/core.rs`, `#[doc(hidden)]`, exported via
`kernel_api`):

```rust
pub enum FrameOutcome {
    /// An ordinary guest-visible value (identity slots, Route E results).
    Value(Expr),
    /// Out-of-band typed continuation state; constructed only by the adapter.
    State(Box<dyn ErasedSlotValue>),
}
pub trait ErasedSlotValue: Send + Sync + 'static {
    fn as_any(&self) -> &dyn std::any::Any;
}
```

`SuspensionFrame::resume(self: Arc<Self>, &mut Session, Result<FrameOutcome, EvalError>)`
`-> FrameAction`; `FrameAction::Complete(Result<FrameOutcome, EvalError>)`;
`PureFrameFinalizer::finish -> Result<FrameOutcome, EvalError>`. The driver
seeds with `Ok(FrameOutcome::Value(Expr::string(input)))`; the `Evaluate` arm
wraps its evaluation result into `Value` at the driver. Errors stay
`EvalError` (the Err side is untouched — that is E2's half).

Layer 2 — slot typing:

```rust
pub trait SlotValue: Sized + Send + Sync + 'static {
    fn into_outcome(self) -> FrameOutcome;
    fn from_outcome(outcome: FrameOutcome) -> Result<Self, SlotTypeError>;
}
pub trait SlotFrame<T: SlotValue>: SuspensionFrame {
    fn resume_slot(self: Arc<Self>, session: &mut Session,
                   outcome: Result<FrameOutcome, EvalError>) -> SlotAction<T>;
}
pub enum SlotAction<T: SlotValue> {
    Complete(Result<T, EvalError>),                              // type-checked
    Evaluate { expr: Expr, next: Arc<dyn SuspensionFrame> },     // next receives Value
    Finalize(Box<dyn SlotFinalizer<T>>),                         // typed finalizer
}
```

`impl SlotValue for Expr` maps to/from `Value` — guest values stay `Expr`.
Every other slot type maps to/from `State` by exact-type downcast.
**Closed impl set, no blanket impl** (a blanket would overlap `Expr`); the
closed set is pinned by a unit test, and a new slot type requires writing its
impl — which is exactly the review point. `SlotTypeError` is a classified
internal error (kernel-bug-only; see (iii) below).

The adapter (one generic type, enters the derived inventories once):

```rust
pub(crate) struct SlotFrameAdapter<F, T> { frame: F, marker: PhantomData<fn() -> T> }
```

implements `SuspensionFrame` by calling `resume_slot` and erasing
(`Complete(Ok(t))` → `Complete(Ok(t.into_outcome()))`, plus finalizer
adaptation), and delegates `FrameRootTrace` and `FrameAbandon` to the inner
frame — the compile-error traversal guarantee extends through it.

Layer 3 — the macro. The ordinary arm infers `T` from `$result`'s Ok type and
requires the frame expression to implement `SlotFrame<T>`:

```rust
resume_or_suspend!(session, result, ok(v) => continue, suspend => frame)
// result: Result<T, EvalError>;  frame: F where F: SlotFrame<T>
```

The insert arm names the slot explicitly (no result infers it):
`resume_or_suspend!(insert session, slot T, frame)`.

Heterogeneous chains (a frame whose input and output slots differ, e.g. the
call-dispatch frame decoding `PreparedCallArguments` at one stage and
`Option<Expr>` at others) get a **per-chain slot enum**
(e.g. `CallStageOutcome { Prepared(PreparedCallArguments), Stage(Option<Expr>) }`)
— typed constructors, one downcast, variant mismatch = classified error.
This is the same device `FrameOutcome` itself uses; it keeps one slot type
per frame without splitting the dispatch frame per stage.

What each requirement gets:

- (i) `FinishCasesRuleMatchSuspensionFrame` implements
  `SlotFrame<Option<Expr>>`; its body must complete
  `SlotAction::Complete(Ok(Some(value)))` — today's `Ok(value)` no longer
  compiles. The 5 REAL frames become unrepresentable as written.
- (ii) Guest source can produce `FrameOutcome::Value` payloads only, and only
  through the `Expr` it already owns; `State` is constructed solely inside
  `SlotFrameAdapter` (and the driver's finalizer adaptation) from a `T` a
  frame produced in Rust. There is no symbol to write, no shape to forge.
- (iii) `SlotValue::from_outcome` is an exact-type downcast: it either
  returns `T` or `SlotTypeError` (classified cold error; `debug_assert!`
  in debug). Guest input *reaches* only `Value` arms (Route E, identity
  slots, the seed), and state slots reject `Value` by type, not by shape —
  no `panic!`, no misread, and the residual check fires only on a mis-paired
  frame chain, which is a kernel bug: the pairings are fixed at compile time
  by Layer 3.
- The 203 continuation heads, 25 `ContinuationValue` impls, 134 decoder
  bodies and 143 encoder bodies are deleted across the family conversions;
  ad-hoc payload decoders (`decode_prepared_interpreted_*`,
  `prepared_call_arguments_from_continuation_expr`, binding-map decoders)
  become plain Rust struct construction — their nested `Expr` fields stay
  `Expr` (§3.5).

### 4.4 Relation to E2 (brief §4) and sequence

`.temp/brief-583/e2-outline.md` defines E2 as the *propagation* side: a
may-suspend result type with no `Try`, so an ordinary `?` cannot carry
`InputRequired` unnoticed; its disposition 1 ("typed continue descriptor …
output codecs are explicit") presumes exactly the codec layer this design
builds. G13's repair governs the *completion payload* side (the Ok value
crossing frames); E2 governs the Err side (how suspension propagates). They
compose: when E2 later changes function signatures to a no-`Try` type,
`SlotValue`/`SlotFrame` are unchanged — `T` is still `T`; only the macro's
`InputRequired` matching moves to the new type.

- **G13's repair is a foundation of E2, not subsumed by it and not dependent
  on it.** It builds fully on today's `Result<T, EvalError>`: the slot type
  is the Ok-type of existing signatures; nothing in §4.3 touches `?`.
- **Recommended sequence: G13 repair first, E2's measured prototype and
  migration after.** Merging both into one migration would require E2's
  representation to be chosen already — the outline explicitly leaves it open
  until measured; and E1's falsifiers must keep running throughout, which the
  representation-only steps here preserve. Doing G13 first also means E2's
  disposition-1 descriptors inherit compile-checked codecs instead of
  re-inventing them.
- E2's G9/G10 obligations (cursor/prefix ownership, checked tails) are
  unaffected by this design: they live in the ok-arm bodies, which stay
  hand-written per frame.

## 5. The class-wide falsifier (brief §5)

**Oracle variant `marker-corpus`** (in
`crates/rum-stdlib/tests/suspension_oracle/`, alongside the E1 catalog):

- For every E1 descriptor entry whose callback/body result flows into a slot
  decode (selection predicate: the reachability tables' guest-reachable
  families — the 5 REAL + witness-pending + every identity-slot entry as
  control), generate a variant fixture per corpus shape: the callback/body
  returns that shape instead of its ordinary value. Corpus (from
  `witnesses.tsv` + `marker-corpus.tsv`): `System`InternalContinuationNone[]`,
  `InternalContinuationSome[5]`, `InternalContinuationSome[]`,
  `InternalContinuationVec[1,2]`, and the guest-writable compound encodings
  `List[True, List[]]` (CompiledMatchOutcome), `List[]`
  (Option<Vec<Expr>>/Vec). Each variant asserts the E1 checks:
  resumed == queued in value, log, messages, prints, trace; suspension
  asserted; suspending mid-loop and twice (the branch's standing
  class-falsifier shape).
- **Red today on the five REAL frames**: the 18 divergence rows of
  `witnesses.tsv` are exactly this falsifier's first 18 reds (the 3 run-1
  families + assoc + fold/nest).
- **What it checks once markers no longer exist in guest space**: the corpus
  shapes become *ordinary values* — after the repair, returning them from any
  callback must change nothing (resumed == queued still holds, and the value
  passes through untouched). The falsifier therefore stays green permanently
  and catches any future decoder that again treats a guest value as typed
  state. The compile-side complement: the lenient `from_continuation_expr`
  API is deleted in the final step, so reintroducing marker decoding cannot
  compile silently.
- Panic shapes (2-arg `Some`) stay OUT of the falsifier (a red oracle must
  not abort the binary); they remain director-side probes.

## 6. Ripple and cost (brief §6)

- **`suspension/core.rs`**: `FrameOutcome`, `ErasedSlotValue`, `SlotValue`,
  `SlotFrame`, `SlotAction`, `SlotFinalizer`, `SlotFrameAdapter`, driver and
  `FrameAction` signatures, both macro arms.
- **`kernel_api.rs`**: new doc(hidden) exports; `ContinuationValue` removed
  at the final step. In-tree consumers of the codec/frame surface: 49 stdlib
  files via `kernel_api`, 56 stdlib files using `rum_runtime::resume_or_suspend!`.
  No in-tree host (cli/repl/jupyter) touches frames (rum-host only) — verify
  at brief time.
- **All 564 `SuspensionFrame` + 59 `PureFrameFinalizer` impls**: the channel
  flip rewrites every `resume`/`finish` body mechanically (step 1 codemod,
  driven by the AST tables; compile errors count as progress per TEMPLATE
  stop-loss). Family conversions then delete that family's codecs.
- **`rum-derive`**: unchanged (FrameRootTrace/PureFrameAbandon). The adapter
  carries manual delegating impls.
- **Closed lists**: the adapter (and any per-chain slot enum frames) enter
  `runtime/tests/frame_root_trace/derived_inventory.rs` and the named-frame
  count in `runtime/tests/frame_root_trace.rs`; converted frames whose fields
  change (encoded-Expr fields → typed fields) update their inventory rows.
- **Retention + ownership inventories**: every step expires
  `inventory/local-contracts.tsv` fingerprints — replay per TEMPLATE standing
  duty (both default and `--no-default-features`
  `local_production_acceptance`, cells to `.temp/brief-546/certificates/`);
  reconcile ownership rows; new `UnknownConstruct` rows expected for new
  derives/macros, disclosed per row.
- **Oracle**: the `marker-corpus` variant (§5); all existing E1 descriptors
  and suites stay green in both builds at every step.
- **Ordinary-path functions that change shape: none.** Frames, outcomes and
  adapters exist only after a suspension; every changed signature
  (`drive_resuming_frames`, `resume`, `finish`,
  `resume_suspended_call_dispatch_frame`, `suspend_owned_guards`,
  `suspend_realization`, …) is on the cold path, most already `#[cold]`. The
  suspend branches inside hot-adjacent functions (`try_apply_function`,
  `try_dispatch_builtin_call`, `try_set_positional_part_lvalue`,
  `try_set_association_lvalue`, `finish_cases_rule_match`) change only their
  frame construction.
- **Instruction gate plan** (regression guard; sub-1% threshold): build base
  and candidate from same-path source exports into separate target dirs,
  both `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`;
  `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, N and 2N, stdin
  `/dev/null`, raw output persisted under `.temp/brief-NNN/perf/`. Labels:
  `downvalue-exact-hit`, `downvalue-pattern-hit`, `downvalue-guarded-hit`,
  `downvalue-guarded-decline-fallback`, `matchq-integer-success`,
  `matchq-integer-fail`, `matcher-single-tiny`, `cases-wide-no-match`,
  `replaceall-simple`, `rule-rhs-deep-small`, `rewrite-guarded-hit`,
  `rewrite-guarded-decline-fallback`, `replacepart-guarded-hit`,
  `set-part-fixed100-len100`, `set-part-fixed100-len800`, `part-list`,
  `block-init-read-repeat`, `sort-callback-small` (+ same-tier neighbours
  `subvalue-unguarded-hit`, `default-guarded-hit`, `table-iterator-plus`,
  `deletecases-wide-no-match`). Plus the standing microbench A/B
  (`scripts/core-language-microbench.sh`) per the rewritten gate brief.

## 7. Drafted implementation briefs (brief §7)

Five briefs, each gated, each landing green; briefs 602-605 each end with
the full oracle green in both builds.

### brief-601 — G13 step 1: the two-arm channel, representation only

**Goal.** Introduce `FrameOutcome { Value, State }` and flip
`SuspensionFrame::resume`, `FrameAction::Complete`,
`PureFrameFinalizer::finish` and the driver to carry it, with **no behaviour
change**: every existing frame mechanically wraps/ unwraps the `Value` arm;
no `State` is produced yet; markers still exist and still round-trip.

**Scope (MAY modify).** `crates/rum-runtime/src/runtime/suspension/core.rs`;
every file with a `SuspensionFrame`/`PureFrameFinalizer` impl or
`resume_or_suspend!` use (the codemod: rum-runtime + rum-stdlib, enumerated
from `ast-frames-raw.tsv` / `ast-macros-raw.tsv`);
`crates/rum-runtime/src/runtime/kernel_api.rs`; the two `FrameAction::evaluate`
driver wrap sites; standing-duty inventories per TEMPLATE. **No decoder
semantics change in this brief.**

**Key steps.** (1) Add `FrameOutcome` + `ErasedSlotValue` + conversion
helpers (`into_value_result` — strips `State` via debug-assert + classified
error, unreachable this brief since nothing emits `State`). (2) Flip the four
signatures; codemod every body: consuming `Ok(value)` →
`Ok(outcome)::into_value_result()`-shaped access; completing
`FrameAction::complete(res)` → `complete(res.map(FrameOutcome::Value))`; the
macro arms adapt. (3) Fix compile errors to zero (they are the progress
counter). (4) Replay retention/ownership inventories; update the closed
frame lists if any frame type is introduced (none should be).

**Verification.** `cargo fmt --check`; `cargo test --workspace` green;
`suspension` suites and the E1 oracle green in default and
`--no-default-features` builds. No perf gate (cold paths only; state it).

**Hard rules.** TEMPLATE standard + no `git`, no new `unsafe`, no
reformatting outside the codemod, stop-loss on compile-error progress.

### brief-602 — G13 step 2: slot-typed frames + the five REAL frames

**Goal.** Add `SlotValue`/`SlotFrame`/`SlotAction`/`SlotFinalizer`/
`SlotFrameAdapter` + the typed macro arms; convert the five REAL frames and
their consumer edges (`try_apply_function`→call dispatch `Option<Expr>`;
`finish_cases_rule_match`→cases traversal `Option<Expr>`;
`try_set_positional_part_lvalue`, `try_set_association_lvalue`;
`try_dispatch_builtin_call`/`suspend_threaded_call_dispatch`); build the
oracle `marker-corpus` falsifier. The 18 witness divergences turn equal.

**Scope (MAY modify).** `suspension/core.rs`; `kernel_api.rs`;
`scoping/callable.rs`; `kernel_data/rewrite_data/cases/collect/`
(`mod`+resume_frames); `definitions/part_assignment.rs` (+resume_frames);
`kernel_data/association_data.rs` (+resume_frames); `eval_dispatch/mod.rs`;
`eval_core/call.rs` (consumer decodes for these edges —
`CallStageOutcome` per-chain enum if the dispatch frame needs both slots);
`crates/rum-stdlib/tests/suspension_oracle/` (falsifier + catalog wiring);
closed frame lists + inventories (the adapter enters once).

**Key steps.** (1) Land the machinery (§4.3) with the closed-`SlotValue`-set
unit test and adapter delegating trace/abandon. (2) Convert the five frames
to `SlotFrame<...>` — each `Ok(raw)` completion becomes the typed
constructor; their consumers decode via `SlotValue::from_outcome`. (3) Add
the `marker-corpus` oracle variant with the §5 corpus; the six witness
families' descriptors go in; assert resumed==queued. (4) Full oracle green
both builds; inventories replayed.

**Verification.** `cargo fmt --check`; `cargo test --workspace`; oracle
`marker-corpus` green (it was red on exactly these frames — paste the
before-red from the director's witness run and the after-green); a one-
codegen-unit instruction spot-check on `downvalue-exact-hit`,
`downvalue-guarded-hit`, `set-part-fixed100-len100`, `sort-callback-small`
(cold-path claim sanity; ±0.1% expected, sub-1% threshold).

**Hard rules.** TEMPLATE standard; markers not otherwise deleted in this
brief (other families still use them); the falsifier must not include
panic shapes.

### brief-603 — G13 step 3: runtime codec families

**Goal.** Convert the remaining rum-runtime codec families to slot-typed
state and delete their markers: matcher (`CompiledMatchOutcome`, binding
maps, sequence/orderless-optional incl. the `Vec<SymbolExprMap>` slots),
prepared-call (`PreparedCallArguments` + `eval_call_prepare.rs` decoders,
removing the `call.rs:322` expect), interpreted payloads
(`InternalInterpreted*`), guards (`rhs_guards`), cases/replace/delete/
position/count traversal outcomes, `EvaluatedPartSelectors` +
`EvaluatedPartSelectorsContinuation` (subsume the `Arc<OnceLock>` into
`SlotValue`), `EvalCapturedOutcome`, message/render (`String` slot),
failure/status (`FailureMessageFromTemplateSuspensionFrame` settles
witness-pending), block/scoping, loader/guest-loading frames.

**Scope.** The corresponding runtime module trees (enumerated per family
from `decoders.tsv`/`slot-producers.tsv`); no stdlib source beyond what the
kernel_api signatures force. Steps: one family per commit-sized unit inside
the brief, each leaving the workspace green; family order by
`queue-producer-edges` SCC so producer/consumer edges flip together.

**Verification.** `cargo fmt --check`; `cargo test --workspace`; oracle
green both builds per family; extend `marker-corpus` descriptors to each
converted family; one-codegen-unit slopes on the §6 label set after the last
family.

### brief-604 — G13 step 4: stdlib codec families

**Goal.** Convert rum-stdlib's frames (lazy sequences incl.
`LazyRealizeSuspensionFrame`, string search/replace, associations/structure
`lazy_completion` consumers, `resumable_merge_sort`, delete-missing, any
remaining `kernel_api::ContinuationValue` users — 49 files) to the typed
macro arms.

**Scope.** `crates/rum-stdlib/src/**` per the same family enumeration;
oracle + closed lists as usual. **Verification.** as brief-603 plus
`local_production_acceptance` both feature builds replayed.

### brief-605 — G13 step 5: delete the in-band marker system

**Goal.** Remove `ContinuationValue`, `from_continuation_expr`/
`into_continuation_expr`, the lenient primitive decoders, and all
`System`InternalContinuation*` / `InternalInterpreted*` head references;
assert zero continuation heads remain in production source (a drift test
against a fresh scan, like the builtin-reference drift test); final
instruction gate + full microbench A/B.

**Scope.** `suspension/primitive.rs` (deleted), the codec modules listed in
`decoders.tsv`, the drift test, `docs/` reference regeneration if any head
was documented (check; expectation: none — they are undocumented internal
symbols). **Verification.** `cargo fmt --check`; `cargo test --workspace`;
both feature builds; `scripts/core-language-microbench.sh` A/B with the §6
labels + neighbours, one-codegen-unit slopes, raw rows persisted.

## 8. Open questions for the director

1. **`SlotValue` closed set vs blanket**: explicit impls (recommended — the
   closed set is a review point and `Expr` stays unboxed) vs a blanket with
   `Expr` boxed through `State` (fewer impls, muddier guest-value boundary).
2. **Per-chain slot enums** (e.g. `CallStageOutcome`): acceptable as the
   device for heterogeneous dispatch frames, or should the dispatch frame be
   split per stage so every frame has a primitive slot type?
3. **Falsifier corpus breadth**: guest-reachable-slot entries + identity-slot
   controls only (recommended), or all 438 E1 entries × corpus (6× oracle
   runtime)?
4. **Witness permanence**: the 18 divergences live in the oracle
   `marker-corpus` variant from brief-602 (recommended single home); no
   standalone failing tests can land pre-repair.
5. **Marker symbol cleanup**: delete all `system_expr("InternalContinuation*")`
   references in brief-605 (recommended) or keep inert constants for
   diagnostics?
6. **Insert-arm spelling**: `resume_or_suspend!(insert session, slot T, frame)`
   vs a separate `insert_slot!` macro — 60 sites, cosmetic.
7. **Brief count/size**: five briefs estimated; brief-601's codemod is the
   mechanical risk (compile-error volume across ~623 impl bodies). If the
   director prefers smaller steps, brief-601 can split runtime-first /
   stdlib-second, at the cost of a temporarily mixed channel (safe: the
   channel type is identical, only the bodies differ).
8. **`FailureMessageFromTemplateSuspensionFrame`**: witness-pending — the
   falsifier settles it in brief-602; if real, it joins the five; the
   conversion is identical either way.
