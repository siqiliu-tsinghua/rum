# brief-576 — design: option readers that resume into their builtin (group G5)

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

Group G5 of the suspension-continuation branch: an option value can be
delayed (`Heads :> (…; InputString["p: "]; True)`) and is evaluated when the
builtin reads it. If that evaluation asks for input, the resumed request
completes with the option's value instead of the builtin's result —
`Monitor[(AppendTo[log, 3]; 77), 0, (…; InputString["p: "]; …; EvaluationMonitor :> Null)]`
resumes to `List[RuleDelayed[EvaluationMonitor, Null]]` where the queued path
returns `77`. brief-548 recorded 29 sites (`.temp/brief-548/draft-G5.md` has
the list), all reaching the shared readers — `system_option_bool`,
`runtime_option_expr_for_symbol`, `option_rule_expr_for_symbol` →
`resolve_option_entry_value` (`crates/rum-runtime/src/runtime/options/core.rs:76`)
— from `Monitor`, `TestReport`, `Trace`, `CheckAbort`, `Cases`,
`DeleteCases`, `Replace`, `Names`, `PersistPackageState`/`DumpSave`, `Count`,
`MemberQ`, `FreeQ`, `Level`, `Position`, `ArrayFilter` and `Depth`.

The director re-ran brief-548's probes on `5cec7b67`
(`.temp/director/brief548-probe-rerun-5cec7b67.txt`): 37 still diverge, and
every G5 witness is among them (`Monitor-options`, `Monitor-options-twice`,
`Cases-Heads-option`, `Cases-Heads-after-target`, `TestReport-option-*`,
`Names-*`, `CheckAbort-option`, `ArrayFilter-option`, `DumpSave-option`,
`Depth-Heads-option`, `Level-Heads-option`, …). Lost continuations today: 43;
G5 accounts for 29.

## What the design must settle

1. **The inventory.** Every reader and every caller, with `path:line`: which
   options are read, when (before or after the builtin's own guest work — note
   the `*-after-target` witnesses), which readers evaluate delayed values, and
   what each caller does after the read. Group the callers by what a
   continuation must carry.
2. **Per-site repair or one mechanism.** A frame at each of 29 call sites is
   one option. A shared mechanism is another — for example, a builtin's option
   reads resolved in a resumable phase whose completion hands the resolved
   values to the rest of the builtin, so a suspension inside an option value
   is a suspension of that phase and nothing else. `CLAUDE.md`'s standard: a
   fix the compiler or a type enforces beats one that relies on each site
   remembering, even if larger. Compare the options on correctness,
   enforceability and how a future builtin that reads an option gets the right
   behaviour by construction; recommend one, with the evidence.
3. **Evaluation order is observable.** Options read after the builtin has done
   guest work (`Cases-Heads-after-target`, `DeleteCases-Heads-after-target`)
   must stay after it; options read in a particular order must keep that
   order; an option never read on a path must stay unread. Say how the design
   preserves each.
4. **The retention line's certified option region.** `options/core.rs` holds
   the bodies the retention line certified (brief-546, certificates C01/C02;
   `docs/dev/retention-closure-design.md`). The standing duty refreshes their
   fingerprints, but a change to a certified **body** must still pass
   `local_production_acceptance` on its merits in both feature sets — not by a
   refreshed fingerprint alone. Say which certified bodies the design touches,
   and whether it can avoid touching them; if it must, say what the
   certificates will then check and why they still hold.
5. **Suspension protocol**: brief-563/568/572's settled rules — capture only
   on `InputRequired` on the error edge, immutable Session-rooted frames with a
   cold working copy on an actual resume, `FrameRootTrace`/`PureFrameAbandon`,
   no `Session` borrow, observer or `NativeStackDepth` in a frame, rum-runtime's
   frame lists, a second suspension, abandonment.
6. **The ordinary path.** Option reading is on many hot paths (`Cases`,
   `Count`, `MemberQ`, `FreeQ`, `Position`, `Replace` read `Heads` on every
   call). Name the labels and the plan; **both arms with one codegen unit**
   (`TEMPLATE.md` pitfall 8). Depth is not expected to matter (option lists
   are flat) — say so with evidence or show otherwise.
7. **Inventory, census and layout consequences**, listed so the director can
   authorize them by name, and the step split.

## What I want back

1. The inventory (point 1) with evidence.
2. The design (points 2-7), the frame inventory, and the comparison.
3. Red tests, run in the scratch test and recorded: every brief-548 G5 witness
   through the rum-vs-rum oracle (value, `log`, messages, prints, suspension
   asserted, exact prompts, no owner after, `1 + 1`), the twice variants, the
   after-target order cases, an ordinary error inside an option value after a
   resume, cancellation at a suspended option, an option that is present but
   never read on the taken path.
4. Drafted implementation brief(s) in `TEMPLATE.md` shape — each with its red
   tests, predicted lost-continuation count, named inventory rows and frame
   lists, the retention-certificate obligation, the one-unit performance gate,
   the standing duty, and serial runs (`env CARGO_BUILD_JOBS=1
   RUST_TEST_THREADS=1`).
5. Open questions for the director.

## Context pointers

- brief-548's report, `.temp/brief-548/draft-G5.md`, `probes.json`,
  `lost.tsv`, `probe.rs`.
- brief-530 (one lost continuation on the option path, fixed earlier) and
  brief-559's `OptionValue` loop frame
  (`crates/rum-runtime/src/runtime/options/builtins.rs`).
- briefs 563, 568, 572-575 (the accepted continuation and measurement
  patterns).
- `crates/rum-runtime/src/runtime/options/`, `suspension/core.rs`;
  `docs/dev/retention-closure-design.md`; the retention inventories under
  `crates/rum-stdlib/src/tests/retention_closure/`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-576/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief576_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-576/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-576/`.
- **Run no probe that aborts a process.**
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief576_probe`
  (150 s); `.temp/brief-575/candidate-08/rum-cli` (source identical to HEAD)
  on passing programs (60 s); `valgrind --tool=callgrind` on it (150 s). File
  reads, `rg`/`grep`, `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The inventory.
3. The design, frame inventory, comparison, retention obligation.
4. The red tests with their pre-fix values.
5. The drafted implementation brief(s).
6. Open questions for the director.

---

## Execution report — brief-576, design only

### 1. What changed

**Design only; no production change.** Appended this report to the dispatched brief. All other retained artifacts are under `.temp/brief-576/`: source/caller inventory, design and frame table, named inventory/layout plan, baseline instructions, 70-test scratch probe and raw evidence, three drafted implementation briefs, checkpoints and scope audit. The one permitted integration file, `crates/rum-stdlib/tests/zz_brief576_probe.rs`, was copied to `.temp/brief-576/probe.rs` and **deleted before this report was appended**.

No git command, network access, outside-repository scratch, new unsafe, production edit or process-aborting probe was used. Commands were serial and bounded. Cargo and CLI/profiler TMPDIR was `/home/remi/dev/rum/.temp/brief-576/tmp`; Cargo artifacts are in `.temp/brief-576/target`. A SHA256 audit of the pre-existing crates/docs/scripts files found no change before this append and no added source file after deleting the scratch integration test. The audit and final probe hash are saved in `final-scope-audit.json`; no source or inventory replay was left behind.

The recommended repair is a shared typed **read-and-complete** boundary, invoked at each existing read position. It preserves the two reader evaluation policies and observable order while making a caller supply the rest of its builtin. It can keep C01 and C02's certified bodies unchanged. It needs one new named suspension frame and the family state/inventory changes below; those are proposed, not implemented.

Artifact index (all local, reviewable): [inventory](../../../.temp/brief-576/inventory.md), [design](../../../.temp/brief-576/design.md), [inventory/layout plan](../../../.temp/brief-576/inventory-plan.md), [performance plan](../../../.temp/brief-576/performance-plan.md), [final probe](../../../.temp/brief-576/probe.rs), [cases](../../../.temp/brief-576/cases.json), [final test log](../../../.temp/brief-576/test-02.log), [all 66 oracle rows](../../../.temp/brief-576/oracle.tsv), [readable case results](../../../.temp/brief-576/all-case-results.md), [lifecycle](../../../.temp/brief-576/lifecycle.tsv), [draft A](../../../.temp/brief-576/draft-G5-A.md), [draft B](../../../.temp/brief-576/draft-G5-B.md), [draft C](../../../.temp/brief-576/draft-G5-C.md).

## 2. Inventory and evidence

All citations describe the incoming files, before the temporary integration test was removed. The full search is `reader-references.tsv`: 66 production definition/reference hits (tests excluded), with 12 boolean-reader calls, 2 rule-reader calls, 7 runtime-reader calls, 4 entry-resolver calls, 5 raw-reader calls, 1 value-reader call, 1 rule-vector-reader call, 2 C02 calls, and 23 merged-reader source call expressions. Those are distinct measures from the 29 historical lost edges and the 26 historical witness programs. The 23 merged call expressions dispatch the 33 variants of `crates/rum-stdlib/src/option_consumers.rs:9`; shared handler families explain the difference.

### Readers: the delayed bit has two different meanings

| Reader, definition | Semantics and direct callers |
| --- | --- |
| `crates/rum-runtime/src/runtime/options/core.rs:5` `system_option_bool` | Calls `option_value_for_symbol` at :12; missing value uses the supplied fallback; only System`True becomes true (:17). Its 12 source call sites are the A rows below. |
| `crates/rum-runtime/src/runtime/options/core.rs:44` `option_value_for_symbol` | Takes a per-read snapshot, exact-symbol lookup then name fallback (:49-53); calls `resolve_option_entry_value` at :56 and returns `Some`. Its sole direct caller is `system_option_bool` at :12. |
| `crates/rum-runtime/src/runtime/options/core.rs:62` `option_rule_expr_for_symbol` | Same lookup; absent returns `None`; :76 evaluates the entry and :77-80 wraps the value as `Rule[requestedOptionSymbol,value]`. Direct consumers: CheckAbort :101 and ArrayFilter :170 below. The wrapping itself is a lost continuation (P0979). |
| `crates/rum-runtime/src/runtime/options/core.rs:179` `resolve_option_entry_value` | **Evaluates delayed RHS** at :184; returns an ordinary value unchanged at :186. Called by value reader :56, rule reader :76, and `options/builtins.rs:379,388` in scalar OptionValue. Scalar OptionValue returns the value as its final result; the many-query loop already saves its prefix/cursor at `options/builtins.rs:252-352` (brief 559). |
| `crates/rum-runtime/src/runtime/options/core.rs:136` `raw_option_value_for_symbol` | No evaluation. Explicit option by name wins, otherwise current defaults; returns `(Expr,delayed)`. Calls: core :159; `builtins_input.rs:195` (FieldMasked), :293 (DialogProlog/DialogSymbols); `package_tools.rs:688,695` (Once ExpirationDate/PersistenceTime). |
| `crates/rum-runtime/src/runtime/options/core.rs:152` `runtime_option_expr_for_symbol` | **Holds delayed values** (:164-165); evaluates **ordinary values** (:167), including latent symbols with delayed ownvalues. It does NOT call `resolve_option_entry_value`. Seven callers: Monitor :72; TestReport evaluation :167,241; Names :73,90,270; package_tools :221. Preserve this inversion; making all delayed options eager changes callback semantics. |
| `crates/rum-runtime/src/runtime/options/core.rs:83` `option_rule_exprs_for_symbol` | No evaluation; returns rules. Used by `accounting_mode/options/buffer.rs:124`. |
| `crates/rum-runtime/src/runtime/options/core.rs:87,111,191` merged readers and C02 | A separate, already resumed/retained whole-option-vector protocol: snapshot then drive, requiring registered `OptionReentry`; C02 evaluates a RuleDelayed RHS and returns a Rule through the retention rule adapter. C02 calls are `accounting_mode/options/driver.rs:26,43`. The 23 external merged-reader expressions are all in `reader-references.tsv`. Preserve this protocol and its 33 consumer variants. It is not the G5 reader contract. |

The read-only `options_for_symbol` (:20), `SymbolOptions::{entry,entry_by_name}` (`options.rs:64,68`), and `compile_assigned_options_from_resolved` (core :171) are lookup/normalization support, not additional guest-evaluation edges. `parse_option_entries_from_resolved` (`options/builtins.rs:455`) flattens lists into a vector. Other raw consumers keep their existing separate behavior; no broad options-language redesign is proposed.

### The 29 frozen obligations, relocated to current source

Each row is one historical edge, not a new frame proposal. The source search above includes helper callers not assigned separate P rows (notably `names_ignore_case` :270 and the two value-reader edges). Step letters refer to the drafted briefs.

| ID | Current source | Enclosing function → callee | Step |
| --- | --- | --- | --- |
| P0118 | `crates/rum-runtime/src/runtime/debug_tools/monitor.rs:11` | `eval_monitor` → `evaluate` | B |
| P0119 | `crates/rum-runtime/src/runtime/debug_tools/monitor.rs:40` | `eval_monitor` → `monitor_option_expr` | B |
| P0120 | `crates/rum-runtime/src/runtime/debug_tools/monitor.rs:41` | `eval_monitor` → `monitor_option_expr` | B |
| P0122 | `crates/rum-runtime/src/runtime/debug_tools/monitor.rs:72` | `monitor_option_expr` → `runtime_option_expr_for_symbol` | B |
| P0157 | `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:167` | `parse_test_report_options` → `runtime_option_expr_for_symbol` | B |
| P0158 | `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:177` | `parse_test_report_options` → `test_report_constraint_expr` | B |
| P0159 | `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:179` | `parse_test_report_options` → `test_report_constraint_expr` | B |
| P0161 | `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:241` | `test_report_constraint_expr` → `runtime_option_expr_for_symbol` | B |
| P0167 | `crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:194` | `eval_test_report` → `parse_test_report_options` | B |
| P0168 | `crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:201` | `eval_test_report` → `parse_test_report_options` | B |
| P0193 | `crates/rum-runtime/src/runtime/debug_tools/trace.rs:52` | `eval_trace_named` → `system_option_bool` | A |
| P0452 | `crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:101` | `eval_check_abort` → `option_rule_expr_for_symbol` | C |
| P0711 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:108` | `resume_cases_after_resolved_spec_suspended` → `system_option_bool` | A |
| P0714 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:132` | `resume_cases_after_resolved_spec` → `system_option_bool` | A |
| P0764 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:102` | `resume_delete_cases_after_pattern_suspended` → `system_option_bool` | A |
| P0767 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:126` | `resume_delete_cases_after_pattern` → `system_option_bool` | A |
| P0848 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:138` | `resume_replace_after_rule_exprs` → `system_option_bool` | A |
| P0962 | `crates/rum-runtime/src/runtime/namespace_tools/names.rs:73` | `eval_names` → `runtime_option_expr_for_symbol` | B |
| P0963 | `crates/rum-runtime/src/runtime/namespace_tools/names.rs:90` | `eval_names` → `runtime_option_expr_for_symbol` | B |
| P0964 | `crates/rum-runtime/src/runtime/namespace_tools/names.rs:107` | `eval_names` → `names_ignore_case` | B |
| P0979 | `crates/rum-runtime/src/runtime/options/core.rs:76` | `option_rule_expr_for_symbol` → `resolve_option_entry_value` | C |
| P0997 | `crates/rum-runtime/src/runtime/package_tools.rs:221` | `eval_persist_package_state` → `runtime_option_expr_for_symbol` | B |
| P1385 | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:132` | `resume_count_after_pattern` → `system_option_bool` | A |
| P1389 | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:172` | `resume_member_q_after_pattern` → `system_option_bool` | A |
| P1393 | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:212` | `resume_free_q_after_pattern` → `system_option_bool` | A |
| P1401 | `crates/rum-runtime/src/runtime/tree_walkers/level.rs:13` | `eval_level` → `system_option_bool` | A |
| P1410 | `crates/rum-runtime/src/runtime/tree_walkers/position.rs:147` | `resume_position_after_pattern` → `system_option_bool` | A |
| P1708 | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:170` | `:eval_array_filter` → `option_rule_expr_for_symbol` | C |
| P2354 | `crates/rum-stdlib/src/structure_ops/surface.rs:41` | `:eval_depth` → `system_option_bool` | A |

Only P1708 and P2354 moved relative to draft-G5's cited lines: ArrayFilter 89 → 170; Depth 39 → 41. `eval_persist_package_state` is an internal shared implementation, not an installed builtin named PersistPackageState; Save and DumpSave use it, and **only DumpSave** reaches the compression read.

### Caller order, decoding, and remaining work

“Before” means before the builtin's own subsequent guest work; ordinary argument preparation may already have evaluated arguments. Keep the existing Hold/dispatch policy.

| Family / rows | Options, position and short circuits | Work still owed / state category |
| --- | --- | --- |
| Monitor P0118/119/120/122 | Arity first; evaluate the explicit option-list expression (:11), compile and reject unknown keys (:12-37), read EvaluationMonitor then StepMonitor (:40-41). Runtime policy above; `None` becomes absence (:74-77). All of this precedes installing a monitor scope. | Retain original body/display/args, parsed explicit options, stage, completed EvaluationMonitor. Then install scope, refresh display, evaluate body, and use existing `EvalMonitorSuspensionFrame`/finish/abandon at :43-84,160-181. Best-effort callbacks/display at :112-156 are intentionally separate. |
| TestReport P0157/158/159/161/167/168 | Pure option syntax/unknown-key check in evaluation.rs:153-165; SameTest, TimeConstraint, MemoryConstraint in that order (:167-179). SameTest absent → SameQ; constraints Infinity → absent (:250). Only a recognized option form in the two-argument branch or valid third argument enters these reads (report.rs:191-207). One-argument/property-only forms keep Rust defaults and read none. | Save head/args, projection properties, explicit options, stage, completed SameTest/time constraint. After all reads: merge report objects or parse tests, then existing test-case driver (report.rs:209-235). Do not start tests, timers or capture scopes before reads. |
| Trace / TraceDetail / TracePrint P0193 | Heads fallback false, read at trace.rs:52 **before even the arity/explicit-Heads checks**. Includes calls with an explicit override or invalid argument shape. | Save args/name/output kind. Decode call syntax, then start trace capture at :116-143; no trace scope exists at the option suspension. |
| CheckAbort P0452 | Arity check :86-99, default PropagateAborts rule :101, default parsing :105-111, explicit option parsing/evaluation :113-142, then primary. Read default even with an explicit override. | Save args; wrap the resumed default as a Rule before decoding propagation. Existing explicit-RHS frame and primary/fallback cleanup remain the next owners (:121-176). |
| Cases P0711/P0714 | Target evaluated (:31), spec/pattern resolved (:58/:85), spec compiled (:105/:130), default Heads=false (:108/:132), then explicit traversal expressions (:110/:133), then matching. Both the ordinary and already-suspended setup entries must be migrated. | Target, compiled spec (frozen only on suspension), original call arguments; then traversal config/limit and collect-and-wrap as List. Do not resolve target/spec again. |
| DeleteCases P0764/P0767 | Target (:30), pattern (:57/:81), default Heads=false (:102/:126), explicit traversal (:104/:127), then deletion. | Target, resolved pattern, call args; continue traversal and deletion/result wrapping. Preserve both setup entries and the existing ContinueDeleteCasesFrame protocol. |
| Replace P0848 | Partition trailing syntax before target (:72); target (:78); resolve/compile rule expressions (:106-136); only valid rules reach default Heads=false (:138); explicit Heads next (:140), then levels (:178), then rewriting. | Target, compiled RewriteRulePlan, partitioned Heads expressions, pending levels and original args. Preserve rule construction and its side effects exactly once. |
| Count / MemberQ / FreeQ P1385/P1389/P1393 | Arity, target, resolved pattern, then default Heads (false/false/true at count_like.rs:132/172/212), then trailing traversal expressions (:235-236), then matching. | Target, pattern, args, finish kind/default policy. Finish count, nonempty predicate or zero predicate respectively; one family continuation can serve all three. |
| Level P1401 | Arity first (:9); Heads=false (:13); levelspec parse (:14); explicit Heads/apply-head parse (:18-56); collect parts. | Evaluated call args; collection config and optional application head follow the read. Even an invalid levelspec still reaches the default read. |
| Position P1410 | Arity, target (:106), pattern (:125), Heads=true (:147), explicit traversal (:148), then collection. | Target/pattern/args; preserve list-of-paths completion and limit handling (:162-187). |
| Depth P2354 | Heads=false (surface.rs:41), then arity/explicit option classification (:42-59), then structural_depth (:61-64). | Evaluated args; no body scope to restore. Default is read even for an explicit override and malformed arity. |
| Names P0962/P0963/P0964 | Compile invocation first (:69), SpellingCorrection (:73, only False supported), validate/possibly return; ResolveContextAliases (:90, only True), validate/possibly return; IgnoreCase (:107/:270, only True maps true). **Declaration order in defaults.rs:92-96 is the reverse of this read order.** | Args, explicit snapshot, stage and validated pattern-source selection. Then enumerate/match current visible names (:109). A failing early option must keep later options unread. |
| DumpSave P0997 | Parse option syntax/path/context filters/unknown keys (:176-219); iff overwrite, read CompressionLevel (:221), require integer 1 (:227-240); then output path/provider, symbols, filtering, attributes, sealed-reference check and write (:244-296). | Args/operator, raw path spec, **already expanded** context filters, explicit options. Context filters depend on current_context (:736-759) and must not be reconstructed after guest option evaluation. Save does not read CompressionLevel. SymbolAttributes and context filters are raw reads, not new evaluating readers. |
| ArrayFilter P1708 | Arity, rectangular array/flattening (:154), radius/template spec (:161), then default Padding rule **only if fourth arg absent** (:168-174); padding parsing, operator application (:175-189). | Evaluated args plus parsed working shape/flat/spec on ordinary path. Cold resume may reparse these pure inputs, without reevaluating arguments/operator or rereading the option. |
| Shared rule reader P0979 | After a delayed RHS returns, Rule construction at core.rs:77-80 must still occur. | Requested option Symbol plus the caller continuation; yielding the scalar value directly is insufficient. |

Exact entry evidence: `RunCallDispatchStageSuspensionFrame::resume` (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:466-488`) merely validates/completes the incoming value. It cannot infer a builtin's missing Rust tail. A frame only around the innermost RHS would still leave the outer reader/consumer work lost.

## 3. Recommended design, frames, enforcement and retention

### A shared read-and-complete protocol, entered at each existing read position

Recommend **one generic option-read suspension frame plus typed caller continuations**, with small phase drivers for Monitor, TestReport and Names. Keep the successful path's existing borrowed/owned working values; convert them to immutable owned continuation data only on `InputRequired`. This is a shared mechanism for completing a read *into its consumer*, not a global preprocessing pass and not a cache of every option.

The API shape (design pseudocode, not a compiled patch) is:

```rust
trait OptionReadContinuation<V> {
    type Frozen: FrozenOptionReadContinuation<V>;
    fn finish(self, session: &mut Session, value: V) -> Result<Expr, EvalError>;
    fn freeze(self) -> Self::Frozen;
}
trait FrozenOptionReadContinuation<V>: FrameRootTrace + Send + Sync + 'static {
    fn finish(&self, session: &mut Session, value: V) -> Result<Expr, EvalError>;
}
// These require a continuation and return the completed builtin expression.
fn system_option_bool<C: OptionReadContinuation<bool>>(
    &mut self, symbol: &str, option: &str, fallback: bool, next: C,
) -> Result<Expr, EvalError>;
fn option_rule_expr_for_symbol<C: OptionReadContinuation<Option<Expr>>>(
    &mut self, symbol: &Symbol, option: &Symbol, next: C,
) -> Result<Expr, EvalError>;
// Crate-private runtime_option_expr_for_symbol has the same CPS shape,
// with Option<Expr>; eval_option_syntax uses Expr for Monitor's List evaluation.
```

One private `OptionReadSuspensionFrame<D,F>` holds a sealed decoder `D` and `F`, the frozen continuation. `D::Value` selects bool, Option<Expr> or Expr and is statically paired with the continuation. Four private decoders suffice: `BoolOptionDecode`, `RuntimeOptionDecode`, `RuleOptionDecode { option: Symbol }`, `OptionSyntaxDecode`. The rule decoder builds the missing `Rule[option,value]`; runtime decoding returns `Some(value)` without re-evaluation. Bool decoding tests System`True. Absence/fallback completes synchronously and never creates a frame. Identity syntax decoding resumes Monitor's explicit-list phase, including parsing and key validation, rather than handing the list to generic dispatch.

The central driver follows three edges: ordinary success decodes and calls `next.finish`; ordinary error returns the same error without freezing; `InputRequired` freezes `next`, inserts the frame, and propagates the request. Freeze is an error-edge operation in a cold, non-inlined helper. No success-path `Arc`, call-argument copy, option-prefix copy, dyn dispatch, pending-owner query, mutex or diagnostic counter is added. A working descriptor may borrow args and move values the caller already owns; it never holds a Session borrow. It is deliberately not constrained by existing `OptionReentry: Copy`, because resolved targets/specs/prefixes are not Copy.

On resume the frame remains rooted in `Session::resuming_suspension` while its method runs (`suspension/core.rs:764-802`). `Err` is forwarded without constructing working state. `Ok` is decoded once, and the frozen continuation makes only the working copy needed for its remaining work. No `Arc::try_unwrap`, `get_mut`, interior mutation or lease is used to consume the live frame. On a later option/body suspension, that code captures its fresh continuation; the runtime appends the still-rooted outer suffix once (:803-827). Never enqueue the old read again. For an RHS with two inputs, its own existing inner frames reach the second input before the option-reader frame receives the final RHS value.

Every new frame derives `FrameRootTrace` and `PureFrameAbandon`. Cancellation at an option drops pending work and does not run later options, Monitor display/body, TestReport tests, CheckAbort primary/fallback, ArrayFilter operator, or persistence. There is no G5-local scope installed yet that needs a stateful abandon action. Existing body frames still own monitor/trace/abort restoration after their scopes start. Do not replace those actions with pure abandonment. Observers, Session references and `NativeStackDepth` are absent from captured state; reacquire any observer inside resumed work if the existing driver needs it. No new cancellation polling schedule is part of this change.

### Why this mechanism rather than the alternatives

| Choice | Correctness/order | Enforcement and future builtin | Ordinary-path / maintenance consequence |
| --- | --- | --- | --- |
| One frame at each historical edge | Can be correct if *every* reader wrapper and outer consumer is captured, including both setup entries and Monitor normalization. A frame only on core :76 cannot complete the caller. | Every future reader call has to remember a separate frame. `Result<bool>`/`Result<Option<Expr>>` plus `?` still compiles while losing the continuation. | Smaller local edits, but up to 29 obligations repeated, codecs duplicated, and omissions remain easy. |
| Resolve all options at builtin dispatch / reuse the existing merged-options loop unchanged | Incorrect: hoists reads before target/spec work, reads unused options, changes Names order, evaluates runtime-held delayed callbacks, snapshots later defaults too early. | Centralized but enforces the wrong semantics. Existing OptionReentry carries only a Copy tag and frozen original call, not a partially completed builtin. | Would add work even to branches that currently read nothing and would disturb C01/C02's certified domain. Reject. |
| Replay the builtin using a resolved-option cache | Can replay target effects and diagnostics; recursive calls to the same builtin and guest mutation of Options make identity/cache lifetime ambiguous. | A cache does not encode the missing Rust tail. | Extra hot lookup/state, scope cleanup and invalidation. Reject. |
| **Typed CPS read boundary + caller phase data** | Preserves each existing read point and late lookup, with one decoder for immediate/resumed completion; no request replay. | Missing continuation is a compile error; raw evaluating getters are inaccessible outside options internals. New callers implement finish/freeze, not a suspension protocol. A repository gate checks tail use and the closed call inventory. | One generic frame, 14 family state descriptors, cold capture; measured hot cost remains a gate, not a promise. Recommended. |

Enforcement has precise limits. A trait cannot prove a consumer's language semantics, and `Result<Expr>` alone cannot prohibit another unframed `?` in a caller. Therefore the required source gate discovers all calls to the four resumable reader entry points and requires a tail expression or explicit return in the enclosing handler/phase method; it rejects unrecorded raw-reader evaluation bypasses in this reader region. Compile-fail controls show the old missing-continuation call failing, a borrowed/non-rootable frozen capture failing, and the now-private untyped readers inaccessible; corresponding compile-pass controls must run. A typed consumer that deliberately implements the wrong finish still requires semantic tests. Arbitrary host-written evaluators/hand-rolled reads are outside this local guarantee; do not describe an AST inventory as whole-program proof.

The old boolean and rule public methods change signature as their families migrate; the runtime-expression reader changes crate-private signature. `option_value_for_symbol` becomes options-private; `resolve_option_entry_value` stays options-private for scalar OptionValue, which already returns its final value. There is no public “old getter” compatibility shim that can silently retain the defect. `raw_option_value_for_symbol` remains the existing non-evaluating seam for Input/Dialog/Once and the new reader internals, with an exact existing-call allowlist. The separate certified merged API, OptionReentry trait/domain and OptionValue loop are preserved. The kernel helper signature change and new generic host-input inventory are explicit director decisions below.

### Order-preservation rules

1. Enter the reader exactly where the inventory puts it. Save completed target/pattern/spec/rule-plan work. Both Cases and DeleteCases entries converge on the *post-default-read* tail; no re-evaluation of their target or pattern. Use the existing frozen traversal finishers on cold paths, and keep their existing tagged Continue* frames only for the phases they already own.
2. Each multiread phase looks up **one** next entry only when it reaches that stage. Parsed explicit options remain the same snapshot the old caller compiled; absent explicit keys consult current Session defaults at that later read. The `Names-later-default-mutation` test requires this. Do not prefetch/snapshot all defaults at entry, unlike the deliberately snapshotting C01 loop.
3. Advance only after success and decoding/validation. Monitor: syntax → EvaluationMonitor → StepMonitor → scope/body. TestReport: valid option form → SameTest → TimeConstraint → MemoryConstraint → report/tests. Names: SpellingCorrection/check → ResolveContextAliases/check → IgnoreCase → enumeration. Names' invalid-value return must happen before the next read.
4. Preserve unread branches: ArrayFilter explicit padding; TestReport one-argument/property-only/malformed option form; Save's no-compression path; Names early invalid options; Replace invalid rules. Conversely do not optimize away defaults currently read despite an override (Heads readers and CheckAbort), and preserve Depth/Trace default reads before arity checks.
5. Rebuild only pure normalized working data when explicitly chosen below. Never reevaluate an Expr that has already run. No diagnostic or session-dependent normalization is replayed. The ordinary error path keeps messages/prints and returns immediately; later values stay unread.

### Frozen state and frame inventory (proposed exact names)

`args` below is `SharedArgs`; all Expr, Symbol, explicit-option, compiled-rule and prefix fields are traced. Working descriptors use existing ownership/borrows. These frozen descriptors are data *inside* the one generic frame, not 14 additional `SuspensionFrame` implementations.

| ID / step | New type and module, relative to crate src/runtime unless marked stdlib | Saved fields; resume destination |
| --- | --- | --- |
| F00 / A | `options/read_phase.rs:OptionReadSuspensionFrame<D,F>` | `decoder: D`, `next: F`; decode then finish. Derive both FrameRootTrace and PureFrameAbandon. |
| F01 / A | `options/read_phase.rs:BoolOptionDecode` | Scalar/zero-size decoder; fallback was already handled on absence. |
| F02 / B | `options/read_phase.rs:RuntimeOptionDecode` | Zero-size Some-value decoder; no re-evaluation. |
| F03 / C | `options/read_phase.rs:RuleOptionDecode` | Requested `option: Symbol`; construct Rule before the caller consumes it. |
| F04 / B | `options/read_phase.rs:OptionSyntaxDecode` | Identity Expr decoder for the explicit Monitor list. |
| S01 / A | `kernel_data/rewrite_data/cases/collect/options.rs:CasesOptionRead` | target Expr, `FrozenRewriteValue<CasesSpec>`, args; resolve remaining traversal args then collect/list. The two ordinary/previously-suspended working entries share this frozen type. |
| S02 / A | `kernel_data/rewrite_data/cases/delete/options.rs:DeleteCasesOptionRead` | target Expr, resolved pattern Expr, args; traversal then deletion/finish. |
| S03 / A | `kernel_data/rewrite_data/replace/once/options.rs:ReplaceOptionRead` | target Expr, RewriteRulePlan, levels Option<Expr>, partitioned Heads SharedArgs, args; explicit Heads then levels then existing rewrite driver. |
| S04 / A | `tree_walkers/count_like/options.rs:CountLikeOptionRead` | target Expr, pattern Expr, args, existing CountLikeFinish; derive name/default config from finish kind and enter finish_count_like_after_pattern's post-read tail. |
| S05 / A | `tree_walkers/position/options.rs:PositionOptionRead` | target Expr, pattern Expr, args; traversal then collect paths. |
| S06 / A | `tree_walkers/level/options.rs:LevelOptionRead` | args; parse levels/options, collect parts, apply head. |
| S07 / A | `debug_tools/trace/options.rs:TraceOptionRead` | args, static builtin name, existing TraceOutput; decode syntax then start trace capture. |
| S08 / A | stdlib `structure_ops/surface/options.rs:DepthOptionRead` | args; classify explicit Heads/arity then structural depth. |
| S09 / B | `debug_tools/monitor/options.rs:MonitorOptionRead` | args (body/display included), Option<Arc<SymbolOptions>>, stage, completed evaluation-monitor Option<Expr>. Syntax stage has no parsed snapshot. Start scope only after StepMonitor. |
| S10 / B | `debug_tools/test_report/options.rs:TestReportOptionRead` | head Expr, args, properties Option<Expr>, Arc<SymbolOptions>, stage, existing TestReportOptions used as the completed prefix. Tests remain in args until existing report driver starts. |
| S11 / B | `namespace_tools/names/options.rs:NamesOptionRead` | args, Option<Arc<SymbolOptions>>, stage, original pattern-argument index/absence, explicit_context. Completed validation is encoded by stage; it need not keep the discarded booleans. |
| S12 / B | `package_tools/persist_options.rs:PersistOptionRead` | args, operator/static mode, overwrite, raw path String, explicit Option<Arc<SymbolOptions>>, already-expanded PersistContextFilters; validate compression then existing persistence tail. |
| S13 / C | `kernel_data/control_data/abort_cleanup/options.rs:CheckAbortOptionRead` | args; default propagation decode then existing explicit-RHS/primary path. |
| S14 / C | stdlib `functional_ops/arrays/options.rs:ArrayFilterOptionRead` | evaluated args; pure cold reparse of shape/flat/radius, decode supplied rule, then apply_array_filter. |
| T01-T03 / B | MonitorOptionStage, TestReportOptionStage, NamesOptionStage, beside their S types | Finite scalar stage enums, each deriving FrameRootTrace. No nested continuation or self-link. |
| T04-T05 / B | existing `package_tools.rs:PersistContextSelection` and `PersistContextFilters` | Add FrameRootTrace (two existing types); includes BTreeSet<String> context selections. Names/frame symbol roots are not a substitute for this non-Expr owned storage traversal. |

For Names, its ordinary descriptor retains the already-compiled invocation. The frozen descriptor retains the **validated pattern source selection**, not a recursive RegexPattern. On actual resume rebuild only the pure pattern data through the existing free pattern compiler (`namespace_tools/names.rs:178-183,234-241`); keep the frozen explicit options and prior validation. No guest evaluation/diagnostic runs again, and visible names are still enumerated only after all options, using the then-current Session. This avoids adding recursive regex types to the frame capture graph. For ArrayFilter, both pre-read parsers take only Expr/rank inputs (`arrays/mod.rs:154-166`); reparse those already-evaluated args only on cold resume. If implementation cannot keep either reparse demonstrably pure, retain and derive the normalized data instead and bring the added inventory types to the director before proceeding.

Closed-list prediction for this exact layout: **one new named SuspensionFrame, 559 → 560** (`runtime/tests/frame_root_trace.rs:483`); **24 new FrameRootTrace derive entries** (10 in A, 11 in B, 3 in C, including the two existing Persist types), and **one new PureFrameAbandon entry** in `runtime/tests/frame_root_trace/derived_inventory.rs`. No new stateful abandonment entry or PureFrameFinalizer. The generic frame counts once, not once per monomorphization. Any changed type list must be disclosed, not hidden in a moved count.

### Retention obligation: avoid certified-body edits

C01 is **only** `Session::merged_option_exprs_for_symbol`, `options/core.rs:87-103`; C02 is **only** `Session::evaluate_runtime_option_expr`, :190-204. The exact roots are in `rum-stdlib/src/tests/retention_closure/local/certificates.rs:11-18` and `option_boundaries.rs:87-92`. The other G5 functions in the same file are not certified bodies. The recommended design can leave **both bodies byte-for-byte unchanged**, together with the accounting-mode option/rule drivers, OptionReentry domain, descriptor and installed-domain checks. New read machinery belongs in `options/read_phase.rs`, not inside the C01 loop. Changing G5 helper signatures/visibility in core.rs is a file/fingerprint change, not permission to reuse C01/C02 coverage for the new protocol.

Every implementation step still replays the standing duty: 68 contract fingerprints (current local-contract records), both default/hash and no-default/hash-false `local_production_acceptance`, then mechanically copies emitted certificate cells and reconciles ownership discovery. `contracts.rs:88-126` fingerprints all .rs/.py/.toml under crates, so even a test addition expires these marks. `local/production.rs:15-28` actually lints the bodies before promotion reconciliation; :51-57 requires **two C01 instances + one C02 mark, two production body rows, four separate probe marks**, with no universal all-R mark. Its outputs go to `.temp/brief-546/certificates` (:12,79-87), an explicit future-brief scratch exception.

If implementation touches either certified body despite this plan, **STOP for a revised scope/contract decision**. A revised step must make its local lint pass on the changed body against current discharged boundaries and finite M×R domain in both feature sets. Any new direct call needs an earned mark or a discharged/falsified boundary; a fresh fingerprint is not evidence. Never change LocalCovered to another status, suppress a rejection, broaden the domain or promote the new read phase to keep a test green. The certified local guarantees and exclusions (caller/whole consumer excluded, no inferred receipt) remain as documented in `docs/testing-strategy.md:130-182` and `docs/dev/retention-closure-design.md:144-179`.

### Depth boundary

This phase needs no new recursive walker. `SymbolOptions.entries` is a flat Vec (`options.rs:29-32`), lookup is linear iteration (:64-69), and a G5 multiread phase has at most three reads. Captured states have no ownership link to another phase state; repeated input uses the runtime's existing flat frame queue. The entry resolver makes one evaluate call, and arbitrary guest-expression depth is the evaluator's existing responsibility. Evidence does **not** imply every option input is flat: the earlier syntax normalizer recurses through nested List input (`options/builtins.rs:455-464`), and Expr/Regex/tensor depth remains outside this phase. No deep abort probe was run. Test ordinary/one/two-input and abandonment of these flat states; do not import G2/G3's unrelated thousand-level recursion redesign.

### Inventory, census and layout authorization list

These are proposed changes for the director's rewritten implementation briefs; none has been made here. `ownership-before.tsv` preserves every selected incoming row, with its source inventory filename; `layout-before.tsv` has byte/line counts. Selection covers the G5 caller files, core/options exports, and frame lists: 42 unknown-attribute, 123 unknown-capture, 174 unknown-macro, 2 unknown-storage, 3 self-reaching, 1 host-input and 1 ledger lines. These are rows to reconcile, not a prediction that all change, disappear or become proven.

- **I01, reader capabilities:** change the signatures of `Session::system_option_bool`, `Session::runtime_option_expr_for_symbol`, `Session::option_rule_expr_for_symbol`; restrict `option_value_for_symbol`; preserve the private scalar resolver, raw-input/dialog/Once readers and certified merged reader. Add read_phase module and kernel_api exports for the new typed continuation surface. Authorize named helper factoring/removal: Monitor `monitor_option_expr` becomes pure None decode; TestReport `parse_test_report_options` becomes pure syntax classification plus stage entry, and `test_report_constraint_expr` becomes pure Infinity decode; Names `names_ignore_case` becomes pure boolean decode. The old `?`-bearing helper identities cannot simply disappear from an exact inventory without this explicit authorization.
- **I02, continuation roots:** exactly F00-F04, S01-S14 and T01-T05 in the frame table; update both rum-runtime lists and the named-frame count. No new self-reaching data shape, manual root implementation, stateful abandon action or finalizer. Existing Cases/DeleteCases/Replace/Monitor/Trace/body frames retain identities and behavior; root/source locations can move only for the named helper extraction below.
- **I03, ownership discovery:** reconcile the named existing caller/helper rows as they are split into post-read tails, including moved/removed closures and macro sites in those bodies. Add new UnknownConstruct rows for new derives, stage arms, generic storage, macro calls and any closures at touched code, always unknown. The exact old row keys are in ownership-before.tsv, not guessed replacements. Position-only changes elsewhere in the touched files are replayed. Any other disappearing identity/status/disposition/exclusion, new row in untouched code or self-reaching cycle stops the step.
- **I04, public generic inputs — decision required:** adding required `next: C` to the two public readers is expected to add UnknownHostPayload/PublicInput rows, analogous to the existing `options/core.rs:113` OptionReentry input row. The public generic trait methods' `value: V` and frozen generic storage may also be discovered as opaque owned ports. Ask the director to authorize the exact semantic sites `Session::system_option_bool(next)`, `Session::option_rule_expr_for_symbol(next)`, `OptionReadContinuation::finish(value)` and `FrozenOptionReadContinuation::finish(value)` and any actual F00 owned-port discovery, with PublicInput/storage dispositions kept distinct. This is a new host trust surface, **not** part of the existing certified OptionReentry domain. Do not delete/change the old OptionReentry host row. If the director will not grant this explicit exception, implementation must STOP on discovery and return the rows; the standing duty alone does not authorize them. Do not make the classifier hide them or mark them covered. A closed enum alternative avoids an open trait but couples runtime to stdlib consumers; it is not the recommended layer boundary.
- **I05, retention records:** mechanically refresh all 68 `retention_closure/inventory/local-contracts.tsv` fingerprints and the two `promoted.tsv` rows' default/no-default certificate cells; retain all statuses, preconditions and exclusions. No changes to local checker rules, domain registration, `local-covered.tsv`, `leaves.tsv`, `projectable.tsv`, proposed boundaries or the four `consumer-obligations.tsv` D3 rows. The new protocol gets no retention certificate in this work.
- **I06, census anchors:** the current Unsplit/Pending rows named `Unsplit:Cases`, `Unsplit:Level`, `Unsplit:Position`, `Unsplit:Replace`, `Unsplit:Trace`, `Unsplit:TraceDetail`, `Unsplit:TracePrint`, `Unsplit:Names`, `Unsplit:TestReport`, `Unsplit:DumpSave`, `Unsplit:Save`, `Unsplit:ArrayFilter` may need moved evidence anchors in `retention_closure/inventory/obligations.tsv`. Keep their class/status/owner/exclusions. Catalog memory-growth declarations remain unchanged (including NonAmplifying Count/MemberQ/FreeQ/Depth and Delegated Monitor/CheckAbort); a new continuation is not a reclassification proof.
- **I07, closure of reader usage:** add `rum-stdlib/src/tests/options_suspension/read_inventory.rs` and its module line, and compile-fail/pass documentation/tests under `rum-runtime/src/runtime/options/tests/read_phase*`. Inventory every supported-reader call, require tail use, freeze/trace constraints and no public legacy getter; challenge omitted continuation, inaccessible raw getter, missing root, unexpected new caller and non-tail use. Keep the existing 33-member OptionReentry inventory separate and unchanged.
- **I08, regression/layout:** own semantic tests in `rum-stdlib/tests/option_reader_continuations.rs` with `support.rs`, `boolean.rs`, `phases.rs`, `rule.rs`, `lifecycle.rs` children. Runtime-only protocol/root/ordinary-error tests live under options/tests. Tests for earlier groups stay enabled. New production files are exactly the nested modules in F/S table, plus module declarations/necessary narrow visibility at their parents; no new root runtime file in the already 61-child directory. Working descriptors may live next to their frozen family descriptor and should not create a second forest of per-edge files.

Size assessments: `options/core.rs` is 6,867 bytes; keep C01/C02 there and the new ~4-8k protocol in read_phase.rs. `options.rs` is 10,063 bytes; add only module/export lines. `once.rs` is 34,371 bytes: put its post-default tail and descriptor in `once/options.rs`, do not push it past 35k. Trace is 29,365; TestReport evaluation/report 20,153/22,297; package_tools 28,498. Extract the option phase/tail into the named children; defer broader traversal/capture/report/persistence splits because this change is confined to entry continuation and their existing drivers are preserved. Assess and report final size; if a parent still exceeds 20k, identify its remaining capture/render/persistence boundary. Count-like 11,397, Position 10,890, Names 15,210, Cases collect 17,507, abort_cleanup 15,866 and surface 14,797 already warrant care: keep their new phase code in the listed children, with no unrelated formatting. Monitor (6,707), Level (6,577), ArrayFilter module (7,031), DeleteCases (8,131) have room but use coherent option children for the shared protocol. Frame inventories are deliberately declarative test data; do not split the 171,460-byte derived list as part of G5.

### Ordinary path and performance gate

No production change was made, so there is no candidate A/B and no no-regression claim. The authorized `.temp/brief-575/candidate-08/rum-cli` source manifest matches the incoming recorded source (zero changed files); its build record says codegen_units=1. `authorized-binary-provenance.json` preserves the SHA256, build record and source comparison. Three standalone result checks passed (Count 3, Depth 3, Names List[]), then six serial bounded callgrind runs measured exact `Do[statement,{N}]` bodies with N=100/200, stdin closed and RUM_PROFILE_* removed. These are whole-loop instruction baselines, including loop overhead, not isolated getter cost or hardware perf counts:

```tsv
label	N	Ir(N)	Ir(2N)	Ir/iteration
g5-count-small-default	100	4865035	6181644	13166.09
g5-depth-small-default	100	4428262	5312475	8842.13
g5-names-default	100	354379541	705308129	3509285.88

```

Raw counts, complete commands/exit statuses, result checks and callgrind files are under `perf/`; each CLI was bounded at 60 s and each callgrind child at 150 s. The Python orchestration first stopped after the successful Names check because its display expectation was `{}`; corrected it to the observed/documented full form `List[]` and ran **only** the two remaining Names profiles. Count/Depth were not repeated. No wall-clock difference was used for a timing decision.

For each implementation step:

- Preserve a source-hashed incoming binary (for A this provided one-unit binary is usable); build candidate with `env CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, a fresh step-local target directory and the same source path layout. Both A/B arms use **one** codegen unit, including any alloc-count arm. Do not compare to a default 16-unit build. Subsequent steps use the previous accepted step's one-unit binary as immediate base, and keep the pre-G5 base for cumulative comparison.
- Run the standard full `scripts/core-language-microbench.sh` with base and candidate, profile env off and no competing CPU-heavy work. Preserve and compare every `timings.tsv` row. Highlight `cases-wide-no-match`, `count-wide-no-match`, `position-wide-no-match`, `deletecases-wide-no-match`, `deletecases-wide-association-no-match`, `replace-wide-no-match`, `deletecases-leaf-delete-small`, `replace-leaf-rebuild-small`, `rewrite-guarded-hit`, `rewrite-guarded-decline-fallback`, `matchq-integer-success`, `matchq-integer-fail`, `replaceall-simple`. Their exact definitions stay in the script (e.g. :610-649); never wrap the target expression in another Set/Print/sink. Inherit the 23-label G4 panel `.temp/brief-573/perf/` as additional controls, especially MatchQ and rule construction.
- Use the same new scratch label definitions with both binaries. A: `g5-{cases,count,memberq,freeq,position,deletecases,replace,level,depth}-small-default`, with statements `Cases[{1,2,3},_Integer]`, `Count[{1,2,3},_Integer]`, `MemberQ[{1,2,3},2]`, `FreeQ[{1,2,3},2]`, `Position[{1,2,3},2]`, `DeleteCases[{1,2,3},2]`, `Replace[{1,2,3},2->x,{1}]`, `Level[{1,2,3},{1}]`, `Depth[{1,{2},3}]`. Add `g5-trace-default` = `Trace[1+2]` and TraceDetail/TracePrint variants, accepting their real output behavior. The small labels expose overhead wide traversal can hide.
- B: `g5-monitor-options-none` = `Monitor[77,0]`; `g5-monitor-explicit-held` = `Monitor[77,0,EvaluationMonitor:>Null]`; `g5-names-default` = `Names["Global`never576"]`; `g5-testreport-options-default` = `TestReport[{},"TestsSucceededCount",{}]`; `g5-testreport-unread-default` = `TestReport[{}]["TestsSucceededCount"]`. Names enumeration dominates its whole-call cost (baseline above), so it cannot alone establish a cheap reader. Cover ordinary latent values separately: setup SetOptions while v is unbound, bind v to the intended inert value, then repeat the identical call; do not include setup in the timed statement. Persistence may use an empty DumpSave to a step-local `.rum` path, but report filesystem/provider cost separately and use the semantic mock-provider witness to test the continuation.
- C: `g5-arrayfilter-padding-default` = `ArrayFilter[Total,{1,2,3},1]`; `g5-arrayfilter-padding-explicit` = `ArrayFilter[Total,{1,2,3},1,Padding->0]`; `g5-checkabort-default` = `CheckAbort[7,9]`; `g5-checkabort-explicit` = `CheckAbort[7,9,PropagateAborts->False]`. Keep A/B labels in the panel. For boolean and rule readers, add same-body non-suspending delayed-default setups (`Heads:>False` / appropriate true default, `Padding:>0`, `PropagateAborts:>True`) and explicit override controls to expose hot error-arm/code-layout effects.
- Result-check each workload outside its measured loop first. All generated language files use `.rum`. Determine N from passing bounded controls; use the same N and statement on both arms. Default-path allocations must not increase: if the director grants alloc-count builds, run them once per arm/label with one unit. The fine instruction gate uses two paired N/2N samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u` and `(count(2N)-count(N))/N`; callgrind may attribute a regression but must be calibrated against perf before being used as an instruction gate. No such hardware gate was run or authorized in this design brief.
- A regression on any hot label or same-tier neighbor is a defect to fix or report. Report the raw pair and attribution; do not silently remeasure until favorable, average away an affected label, or claim “cold” code is free. Keep the small entry/late error helper style established by 574/575; avoid hoisting a constructed frozen descriptor into a hot function merely because it is only stored on error. Two consecutive compiling candidates with no movement in repaired sites, passing fixtures, regressing labels or worst regression stop the step. Eight focused runs/two broad suites maximum absent a new explained failure; at most two paired performance samples per candidate.

## 4. Red tests and pre-fix evidence

The sole scratch integration file contained **70 tests**: all 26 historical G5 programs unchanged (including their mock DumpSave path), 23 same-option twice variants, 17 extra oracle/order/error/unread cases, and four first/second-prompt cancel/answer-free-abandon cases. `cases.json` records sources, answers, exact prompt sequences and grouping; `probe.rs` is its final source. Its fake FileIo provider returns the same read-only write error as brief-548 and performs no filesystem I/O, so the historical `.temp/brief-548/mock.rum` string is only a mock target. All actual outputs and TMPDIR stayed under brief-576.

The oracle uses two new rum sessions per case, one suspended/resumed and one with queued responses. It compares value **and** error/status/termination, ordered messages/prints, and Global`log. At every suspension it asserts InputRequired, the exact pending RuntimeInputRequest, and an overlapping-request rejection (live owner). Before logging a final row it asserts pending input is absent, `reject_overlapping_evaluation_request()` is Ok (no continuation generation), reads log and verifies `1+1 == 2` on both sessions. Exact prompt sequence is then asserted, including never-read controls with no prompts. This proves the owner/readiness checks ran even for cases whose final equality is red. Final response streams are cumulative; the harness also records per-suspension snapshots in suspensions.tsv and does not incorrectly sum duplicate prefixes.

Both test runs used this command, with log name changed:

```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-576/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-576/target cargo test -p rum-stdlib --test zz_brief576_probe > .temp/brief-576/test-02.log 2>&1
```

First run (test-01.log), exit 101, compiled successfully:

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 05s
test result: FAILED. 4 passed; 66 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
```

That run exposed three fixture issues which were corrected without changing production or weakening a language assertion: after-target expected prompts had been extracted in lexical source order instead of execution order; cancellation emits General::cancel as well as the guest message; and a new SetOptions-in-option mutation fixture returned an options list still containing its own live symbol, causing repeated prompts. The corrected mutation replaces SpellingCorrection with False as it updates the later default. Original raw evidence is preserved as run-01-*.

Final run (test-02.log), exit 101; the intended equality tests remain red:

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.27s
running 70 tests
test result: FAILED. 8 passed; 62 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
error: test failed, to rerun pass `-p rum-stdlib --test zz_brief576_probe`
```

### All 26 historical witness values

Values/logs below are the captured **pre-fix observations**, never proposed expected semantics. Exact messages, prints, errors and prompts are preserved per row in oracle.tsv; baseline-rows.tsv below is a verbatim subset for these 26. All historical exact prompt assertions passed. All messages are empty in both paths except the queued DumpSave I/O error. Only Monitor has additional print differences: resumed `[]`, queued `["0"]`. All historical final statuses are OK with no termination.

| Witness | Resumed value | Resumed log | Queued value | Queued log | Prompts |
| --- | --- | --- | --- | --- | --- |
| `Monitor-options` | `List[RuleDelayed[EvaluationMonitor, Null]]` | `List[1, 2]` | `77` | `List[1, 2, 3]` | `p: ` |
| `Cases-Heads-option` | `False` | `List[2]` | `List[1, 2, 3]` | `List[2]` | `p: ` |
| `DeleteCases-Heads-option` | `False` | `List[2]` | `List[1, 3]` | `List[2]` | `p: ` |
| `Count-Heads-option` | `False` | `List[2]` | `3` | `List[2]` | `p: ` |
| `MemberQ-Heads-option` | `False` | `List[2]` | `True` | `List[2]` | `p: ` |
| `FreeQ-Heads-option` | `True` | `List[2]` | `False` | `List[2]` | `p: ` |
| `Position-Heads-option` | `True` | `List[2]` | `List[List[2]]` | `List[2]` | `p: ` |
| `Level-Heads-option` | `False` | `List[2]` | `List[1, 2, 3]` | `List[2]` | `p: ` |
| `Depth-Heads-option` | `False` | `List[2]` | `3` | `List[2]` | `p: ` |
| `Replace-Heads-option` | `False` | `List[2]` | `List[1, x, 3]` | `List[2]` | `p: ` |
| `Trace-Heads-option` | `False` | `List[2]` | `List[Hold[Plus[1, 2]], Hold[3]]` | `List[2]` | `p: ` |
| `ArrayFilter-option` | `0` | `List[2]` | `List[3, 6, 5]` | `List[2]` | `p: ` |
| `CheckAbort-option` | `True` | `List[2]` | `7` | `List[2, 3]` | `p: ` |
| `Names-SpellingCorrection` | `False` | `List[2]` | `List[]` | `List[2]` | `p: ` |
| `Names-ResolveContextAliases` | `True` | `List[2]` | `List[]` | `List[2]` | `p: ` |
| `Names-IgnoreCase` | `False` | `List[2]` | `List[]` | `List[2]` | `p: ` |
| `Monitor-EvaluationMonitor` | `None` | `List[2]` | `77` | `List[2, 3]` | `p: ` |
| `Monitor-StepMonitor` | `None` | `List[2]` | `77` | `List[2, 3]` | `p: ` |
| `TestReport-option-SameTest` | `SameQ` | `List[2]` | `1` | `List[2]` | `p: ` |
| `TestReport-option-TimeConstraint` | `Infinity` | `List[2]` | `1` | `List[2]` | `p: ` |
| `TestReport-option-MemoryConstraint` | `Infinity` | `List[2]` | `1` | `List[2]` | `p: ` |
| `DumpSave-option` | `1` | `List[2]` | `$Failed` | `List[2]` | `p: ` |
| `Cases-Heads-after-target` | `False` | `List[1, 2]` | `List[2]` | `List[1, 2]` | `target:  → option: ` |
| `DeleteCases-Heads-after-target` | `False` | `List[1, 2]` | `List[1, 3]` | `List[1, 2]` | `target:  → option: ` |
| `TestReport-two-args` | `True` | `List[2]` | `1` | `List[2]` | `p: ` |
| `Monitor-options-twice` | `List[RuleDelayed[EvaluationMonitor, Null], RuleDelayed[StepMonitor, Null]]` | `List[2, 4]` | `77` | `List[2, 4, 5]` | `p:  → p: ` |

### Twice, order, error and abandonment outcomes

All 23 `-twice576` variants reach **both** `p: ` and `second: ` within the same option expression, then still return the option scalar/list instead of the builtin result. The two markers 20 and 21 occur once in both logs. The original Monitor-options-twice also reaches both option-list expression prompts but loses its body (List of the two delayed rules vs 77; log {2,4} vs {2,4,5}).

Separate-value phase tests fail earlier: Monitor-runtime-two-options only reaches `eval: ` (log {1,2}) vs queued log {1,2,3,4,5}; Names-two-options only reaches `spell: ` (log {1,2}) vs {1,2,3,4}; TestReport-three-options only reaches `same: ` (log {1}) vs {1,2,3,4}. Their required remaining exact prompts are recorded, not lowered to the one observed. `Names-later-default-mutation` returns False/log {1,2} after `spell: `, while queued returns List[]/log {1,2,3} and must visit the newly installed `new-alias: ` option. This rejects prefetching/snapshotting later defaults.

The historical Cases/DeleteCases after-target programs reach `target: ` then `option: `, log {1,2} on both arms, but resume to False; queued values are List[2] and List[1,3]. The five added Count/MemberQ/FreeQ/Position/Replace after-target variants also retain this exact order and return their option value instead of their builtin result. No reordering is needed to reproduce or repair the bug.

Ordinary non-InputRequired error **inside the same option after resume** already propagates correctly: Count-option-error-after-resume returns `Other { message: "uncaught throw: \"g5 boom\" with tag None" }`, log List[1,2], prints ["before","after"], and two guest messages followed by Throw::nocatch on both arms. The new frame must retain this behavior. The **later-option** error test is red: Names-later-option-error-after-resume yields False/log {1}/print before/one guest message; queued reaches the error/log {1,2}/prints before+after/two guest messages+Throw::nocatch. Monitor-option-streams is also red and checks omitted body output explicitly.

All three never-read controls pass unchanged: ArrayFilter with explicit padding returns List[3,6,5]; TestReport's one-argument form projects count 1; Names with unsupported SpellingCorrection returns its inert call and the same two Names::optval messages. Each log is List[], no prompt occurs, and later present options stay unread.

All four cancellation/abandonment controls pass: cancel by resume or retire without supplying an answer, each at the first and second input within the option. Each returns `TerminatedEvaluation["Cancelled"]`, termination `Some(Cancelled)`, prints ["before"], guest probe::tag then General::cancel, and log List[1] or List[1,2]. No option suffix/builtin body runs, no owner remains, and `1+1` works. Full rows are in lifecycle.tsv.

The 8 green controls are therefore 3 unread + 1 same-option error + 4 lifecycle. The 62 red cases are 26 original + 23 twice + 13 added cases. These are test counts, **not 62 separately inventoried lost continuations**. No broad suite, certificate replay, implementation, depth-abort probe or candidate performance gate was run in this design-only scope.

## 5. Drafted implementation briefs

The three complete TEMPLATE-shaped drafts follow, also saved separately under brief-576 scratch. The director rewrites and dispatches them serially after settling the questions below. Each specifies its red cases, frozen count delta, named roots/inventory rows, retention obligation, serial verification and one-unit gate. They deliberately leave the current implementation untouched.

### brief-NNN — G5 A: typed reader completion and boolean consumers (draft for director rewrite)

Implementation draft. No git in any form. Requires the director's explicit rewrite of the I04 decision; this is not authorization to implement during brief-576.

#### Goal

Install the shared read-and-complete protocol and migrate every system_option_bool consumer without moving a read or replaying guest work. Repair exactly 12 frozen G5 edges; predicted remaining Lost **43 → 31**, G5 **29 → 17**. This step does not claim the runtime-expression/rule-reader groups are repaired.

Named rows: **P0193, P0711, P0714, P0764, P0767, P0848, P1385, P1389, P1393, P1401, P1410, P2354**. Exact path:line sites are in brief-576 inventory and g5-current-rows.json.

#### Context pointers

Read AGENTS.md, docs/testing-strategy.md, docs/code-organization-guidelines.md and brief-576's appended report. Its `.temp/brief-576/{inventory.md,design.md,inventory-plan.md,performance-plan.md,reader-references.tsv,g5-current-rows.json,cases.json,probe.rs,oracle.tsv,lifecycle.tsv}` are the source inventory, design, exact incoming red evidence and proposed authorization list. Read suspension/core.rs:764-855, the owning existing body/resume drivers, options/core.rs and retention_closure/local/{certificates,production,contracts}.rs. Template pitfall 8 and briefs 563/568/572-575 govern capture, roots and measurement. The director must rewrite this draft and settle the explicit I04 inventory/API decision before dispatch; this draft grants no approval by itself.

#### Scope

**Production edit surface:** `crates/rum-runtime/src/runtime/options/{core.rs,read_phase.rs}`, `runtime/options.rs`, `runtime/kernel_api.rs`; `debug_tools/trace.rs` and new `trace/options.rs`; `kernel_data/rewrite_data/cases/{collect.rs,collect/options.rs,delete.rs,delete/options.rs}`; `kernel_data/rewrite_data/replace/{once.rs,once/options.rs}`; `tree_walkers/{count_like.rs,count_like/options.rs,position.rs,position/options.rs,level.rs,level/options.rs}`; stdlib `structure_ops/{surface.rs,surface/options.rs}`. Read-only existing resume_frames are used as continuation owners; no identity/behavior change there is planned. Add `options/tests/read_phase.rs`, `read_phase.md` and their compile-test wiring; add the semantic target/support/boolean/lifecycle test files under I08 and read_inventory under I07.

**Named authorization:** I01's boolean signature/private helper and exports; I02 F00/F01 and S01-S08; I03 only those affected source rows; I04's new generic trait and boolean public-input rows (must be explicitly approved by the director); I05; I06 Cases/Level/Position/Replace/Trace trio position anchors; I07; I08. Derive list **+10**, PureFrameAbandon **+1**, named frames **559 → 560**. No additions to the legacy OptionReentry domain.

**Standing duty, always in scope:** the existing retention-closure and ownership-graph inventory files, rum-runtime's two frame lists, and necessary tests of those exact invariants. Replay all 68 source contract fingerprints; run `local_production_acceptance` in default and `--no-default-features`, copy emitted cells from `.temp/brief-546/certificates`, then replay and reconcile ownership discovery. That legacy certificate-output directory is the only scratch exception outside this brief's own directory. Disclose all changed keys and lines. New UnknownConstructs in touched code remain unknown; only the director's named I01-I08 exceptions permit old-row identity changes/removals or I04's new host-input dispositions. STOP on any unapproved existing-row/status/disposition/exclusion change, new UnknownHostPayload, new row in untouched code, or new self-reaching state.

**Retention body obligation:** C01 core.rs:87-103 and C02 :190-204 stay verbatim, as do accounting drivers, OptionReentry/domain and the certified 33-member consumer protocol. Both certificate runs must earn the same two C01/one C02 marks on their merits. Fresh hashes alone do not pass. If a certified body must change, STOP for a rewritten local-proof obligation; do not relax the checker or marks.

**Fence:** no other suspension group; no Expr/SharedExpr/SharedArgs/serde representation change, broad evaluator/matcher redesign, leases, new unsafe, dependencies, Cargo.lock, catalog contract reclassification or overlapping-request policy change. No AGENTS.md, TODO.md, root PROGRESS.md, earlier briefs or other docs edits. The new internal helper rustdoc/compile tests document the changed kernel API; the rum-host API stays unchanged. Append the report only to the dispatched brief. No unconditional diagnostics/counters/locks/extra allocations on the ordinary path.

#### Deliverable

- Keep all ordinary values borrowed/moved until InputRequired; freeze the named states only in the central cold branch. Both Cases/DeleteCases setup entries share a frozen post-default continuation, retaining completed target/spec work. Count/MemberQ/FreeQ share a finish discriminator. Preserve Trace/Depth reads before arity and explicit overrides.
- Implement the decoder/finish pairing and derive-based root requirements; test missing continuation, raw-getter visibility, missing root, normal success/error, actual input, second suspension, abandonment. Start the source tail-use gate with only explicit remaining legacy runtime/rule sites recorded for B/C; new callers cannot join that legacy list silently.
- Red first: every cases.json historical Heads witness, both Cases/DeleteCases after-target witnesses, all their twice576 variants, Count/MemberQ/FreeQ/Position/Replace after-target576, TraceDetail/TracePrint, Count option error, first/second cancellation and answer-free abandon. Original pre-fix examples: Count False vs 3; Depth False vs 3; Cases after target False vs List[2] with both logs List[1,2] and prompts target then option. Run all applicable tests; retain passing controls.
- Perf gate: full standard suite and G4 controls plus performance-plan A labels; smallest boolean reads and explicit overrides/delayed-but-nonsuspending defaults are mandatory, not only wide traversals.

#### rum / Rust pitfalls (read twice)

1. Red tests assert equality, actual suspension, exact prompts/order, value/error/termination, messages, prints, log, no pending owner and `1+1`. Never make the pre-fix value the expected answer.
2. Every read API returns its consumer's Expr. Use a tail call; a later unframed `?` discards that guarantee. Compile-fail/pass and source inventory tests enforce the interface and tail convention.
3. Capture only InputRequired, on the error edge. Ordinary errors do not freeze; immutable frames remain Session-rooted during resume; make working state only on an actual Ok resume. No Session borrow, observer, NativeStackDepth, Arc::try_unwrap or interior mutable frame. A second suspension installs the new cursor once. Pure abandonment runs no pending code.
4. Preserve exact read order and conditional reachability, delayed-vs-ordinary policy, current late default lookup and completed guest prefixes. Existing body scopes/frames keep their cleanup duties.
5. Hot error-arm code can change codegen. Both base and candidate, including allocation builds, use CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1. No owned-tree cloning or extra frame allocation on success. Runtime timing uses AbsoluteTiming/Instant, never wall-clock subtraction.
6. Two consecutive **compiling** candidates moving none of repaired rows, passing fixtures, regressing labels or worst regression trigger STOP. Falling compiler-error counts are progress. Never weaken assertions or drop labels to move counters.

#### Verification (MANDATORY — run to green or evidence)

All commands serial and bounded, with `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<repo>/.temp/brief-NNN/tmp CARGO_TARGET_DIR=<repo>/.temp/brief-NNN/target`. Bounds inherited from accepted brief-575: focused 150 s (brief-576 scratch compiled in 65 s and reran in 7.27 s), runtime/retention 300 s, one-unit release build 900 s, each instruction process 150 s, workspace 2400 s. No overlapping CPU-heavy work. The director should preserve/calibrate each script bound from same-shape logs when rewriting.

Run red tests before production edits, then the step's focused target until green, then:

```text
python3 scripts/run-bounded.py --timeout 60 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp cargo fmt --check
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --test option_reader_continuations
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --lib options_suspension
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test --workspace --no-fail-fast
```

Use the existing replay to refresh fingerprints before expecting acceptance; its first stale-cell failure may emit the new cells, which must then be copied/reconciled mechanically and checked again. The broad workspace gate includes doctest compile controls; also run the targeted read_phase compile-pass/fail tests and the source-gate falsifiers as soon as the interface exists. Paste real output, errors, skip counts and exits. The design-only scratch target is not a substitute for these gates.

**One-unit performance gate:** run performance-plan.md's full core-language-microbench base/candidate A/B with identical labels, preserve timings.tsv and compare every hot/neighbor row; use the step's new labels and the previous ones. Run two paired N/2N hardware-instruction samples (taskset/perf command specified there); candidate builds and any allocation arms use `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, with independent fresh target directories and source/binary hashes. All profile flags off for timing, stdin closed. Attribute regressions with bounded callgrind; never claim a 16-unit/1-unit comparison passes. A target or neighbor regression is a defect to fix or report for the director. No favorable-only reruns.

#### Checkpointing — protect the scene

All raw logs, source snapshots, red rows, workload .rum files, inventory deltas, command ledger and PROGRESS.md go under `.temp/brief-NNN/`. Save evidence after red capture, each compiling candidate and each gate. Record counters, compilation state, exact next action and every unresolved risk. Use no git; leave the tree uncommitted for the director's authoritative gate.

#### Hard rules

- No git in any form, no network, no outside-repository activity. Only scoped edits/file reads/rg/Python and the listed bounded Cargo, CLI, microbench, alloc/instruction/callgrind commands; the director must retain those command grants in the dispatched brief.
- No production edit outside the named scope or certified-body fence. STOP on an unapproved inventory decision rather than hiding it in a fingerprint refresh. Preserve user changes.
- Testing Policy is permanent: rum-vs-rum only; no other-vendor engines, artifacts or compatibility work. No new unsafe. Run no probe that aborts a process.
- No ordinary-path or neighbor regression waived by a “cold” intention; no leased-memory or recursion redesign inserted into this continuation repair.
- At most eight focused red/green runs and two completed broad suites absent a newly explained failure; two paired performance samples per candidate. Stop on two consecutive non-progress compiling attempts as above.

#### Report format (append to the dispatched brief)

1. Changed files/functions and each frame/root field, final size/split assessment.
2. Verbatim red/green/lifecycle/compiler/ownership/certificate/workspace results and commands/bounds/exits.
3. Exact named obligation delta, passing fixture counts, every changed inventory row, one-unit per-label timing/instruction/allocation rows and neighbor verdict.
4. Counter/stop-loss state, unresolved/new defects and decisions for the director. Do not claim the frozen predicted count proves a new whole-tree census.

### brief-NNN — G5 B: runtime-valued options and ordered builtin phases (draft for director rewrite)

Implementation draft. No git in any form. Requires the director's explicit rewrite of the I04 decision; this is not authorization to implement during brief-576.

#### Goal

Use A’s shared protocol for Monitor syntax/runtime options, TestReport’s optional options path, Names’ ordered conditional readers, and DumpSave compression. Repair exactly 14 frozen edges; predicted remaining Lost **31 → 17**, G5 **17 → 3**. Preserve late lookup, held delayed callbacks and all completed-prefix effects.

Named rows: **P0118, P0119, P0120, P0122, P0157, P0158, P0159, P0161, P0167, P0168, P0962, P0963, P0964, P0997**. Exact path:line sites are in brief-576 inventory and g5-current-rows.json.

#### Context pointers

Read AGENTS.md, docs/testing-strategy.md, docs/code-organization-guidelines.md and brief-576's appended report. Its `.temp/brief-576/{inventory.md,design.md,inventory-plan.md,performance-plan.md,reader-references.tsv,g5-current-rows.json,cases.json,probe.rs,oracle.tsv,lifecycle.tsv}` are the source inventory, design, exact incoming red evidence and proposed authorization list. Read suspension/core.rs:764-855, the owning existing body/resume drivers, options/core.rs and retention_closure/local/{certificates,production,contracts}.rs. Template pitfall 8 and briefs 563/568/572-575 govern capture, roots and measurement. The director must rewrite this draft and settle the explicit I04 inventory/API decision before dispatch; this draft grants no approval by itself.

#### Scope

**Production edit surface:** `runtime/options/{core.rs,read_phase.rs}`; `debug_tools/{monitor.rs,monitor/options.rs}`; `debug_tools/test_report/{evaluation.rs,report.rs,options.rs}`; `namespace_tools/{names.rs,names/options.rs}`; `package_tools.rs` and `package_tools/persist_options.rs` (including the two existing context-selection/filter derives); necessary module declarations. Add phases/lifecycle cases to `rum-stdlib/tests/option_reader_continuations/` and extend read_inventory/compile tests. Existing body frames, option domain, regex implementation and all other evaluation drivers remain unchanged.

**Named authorization:** I01's runtime reader/helper splits; I02 F02/F04, S09-S12, T01-T05; I03 touched phase rows; I05; I06 Monitor-independent Names/TestReport/Save/DumpSave evidence positions; I07; I08. Derive list **+11**, no new SuspensionFrame/PureFrameAbandon beyond A; named frames remain **560**. No new public runtime-valued helper is needed; if a new host row beyond A's approved generic surface is discovered, STOP.

**Standing duty, always in scope:** the existing retention-closure and ownership-graph inventory files, rum-runtime's two frame lists, and necessary tests of those exact invariants. Replay all 68 source contract fingerprints; run `local_production_acceptance` in default and `--no-default-features`, copy emitted cells from `.temp/brief-546/certificates`, then replay and reconcile ownership discovery. That legacy certificate-output directory is the only scratch exception outside this brief's own directory. Disclose all changed keys and lines. New UnknownConstructs in touched code remain unknown; only the director's named I01-I08 exceptions permit old-row identity changes/removals or I04's new host-input dispositions. STOP on any unapproved existing-row/status/disposition/exclusion change, new UnknownHostPayload, new row in untouched code, or new self-reaching state.

**Retention body obligation:** C01 core.rs:87-103 and C02 :190-204 stay verbatim, as do accounting drivers, OptionReentry/domain and the certified 33-member consumer protocol. Both certificate runs must earn the same two C01/one C02 marks on their merits. Fresh hashes alone do not pass. If a certified body must change, STOP for a rewritten local-proof obligation; do not relax the checker or marks.

**Fence:** no other suspension group; no Expr/SharedExpr/SharedArgs/serde representation change, broad evaluator/matcher redesign, leases, new unsafe, dependencies, Cargo.lock, catalog contract reclassification or overlapping-request policy change. No AGENTS.md, TODO.md, root PROGRESS.md, earlier briefs or other docs edits. The new internal helper rustdoc/compile tests document the changed kernel API; the rum-host API stays unchanged. Append the report only to the dispatched brief. No unconditional diagnostics/counters/locks/extra allocations on the ordinary path.

#### Deliverable

- Implement Monitor stages Syntax/EvaluationMonitor/StepMonitor, TestReport SameTest/TimeConstraint/MemoryConstraint, Names SpellingCorrection/ResolveContextAliases/IgnoreCase. Read one current default at a time; validate Names before advancing; freeze parsed explicit options and completed prefix only on suspension. Runtime RuleDelayed values stay held.
- Monitor starts its scope/display/body after all reads and preserves best-effort callback policy. TestReport preserves the two-argument property fallback, third-argument invalid residual, one-argument defaults and existing report merge/test-case drivers. DumpSave preserves pre-read expanded context filters and post-read path/provider/write order; Save stays on its no-compression path. Use only the named pure cold Names pattern reparse, never re-run guest expressions or context-filter expansion.
- Red first: Monitor-options, Monitor-options-twice, Monitor-EvaluationMonitor, Monitor-StepMonitor and all twice576 variants; TestReport-option-* and TestReport-two-args/twice; Names-* single/twice; DumpSave-option/twice; Monitor runtime-two-options/streams, Names two-options/later-default-mutation/later-option-error, TestReport three-options; unread TestReport/Names controls. Incoming Monitor returns List[RuleDelayed[EvaluationMonitor,Null]] with log {1,2} instead of 77/{1,2,3} and print 0; TestReport SameTest returns SameQ vs 1; Names missing later error returns False vs uncaught Throw. Preserve exact messages and prints from oracle.tsv, including currently repeated Names optval messages.
- Keep all A tests green. Perf gate: full suite and A labels, plus performance-plan B labels and current small runtime-read forms; no pre-evaluation of every option or unconditional normalized-state freeze.

#### rum / Rust pitfalls (read twice)

1. Red tests assert equality, actual suspension, exact prompts/order, value/error/termination, messages, prints, log, no pending owner and `1+1`. Never make the pre-fix value the expected answer.
2. Every read API returns its consumer's Expr. Use a tail call; a later unframed `?` discards that guarantee. Compile-fail/pass and source inventory tests enforce the interface and tail convention.
3. Capture only InputRequired, on the error edge. Ordinary errors do not freeze; immutable frames remain Session-rooted during resume; make working state only on an actual Ok resume. No Session borrow, observer, NativeStackDepth, Arc::try_unwrap or interior mutable frame. A second suspension installs the new cursor once. Pure abandonment runs no pending code.
4. Preserve exact read order and conditional reachability, delayed-vs-ordinary policy, current late default lookup and completed guest prefixes. Existing body scopes/frames keep their cleanup duties.
5. Hot error-arm code can change codegen. Both base and candidate, including allocation builds, use CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1. No owned-tree cloning or extra frame allocation on success. Runtime timing uses AbsoluteTiming/Instant, never wall-clock subtraction.
6. Two consecutive **compiling** candidates moving none of repaired rows, passing fixtures, regressing labels or worst regression trigger STOP. Falling compiler-error counts are progress. Never weaken assertions or drop labels to move counters.

#### Verification (MANDATORY — run to green or evidence)

All commands serial and bounded, with `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<repo>/.temp/brief-NNN/tmp CARGO_TARGET_DIR=<repo>/.temp/brief-NNN/target`. Bounds inherited from accepted brief-575: focused 150 s (brief-576 scratch compiled in 65 s and reran in 7.27 s), runtime/retention 300 s, one-unit release build 900 s, each instruction process 150 s, workspace 2400 s. No overlapping CPU-heavy work. The director should preserve/calibrate each script bound from same-shape logs when rewriting.

Run red tests before production edits, then the step's focused target until green, then:

```text
python3 scripts/run-bounded.py --timeout 60 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp cargo fmt --check
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --test option_reader_continuations
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --lib options_suspension
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test --workspace --no-fail-fast
```

Use the existing replay to refresh fingerprints before expecting acceptance; its first stale-cell failure may emit the new cells, which must then be copied/reconciled mechanically and checked again. The broad workspace gate includes doctest compile controls; also run the targeted read_phase compile-pass/fail tests and the source-gate falsifiers as soon as the interface exists. Paste real output, errors, skip counts and exits. The design-only scratch target is not a substitute for these gates.

**One-unit performance gate:** run performance-plan.md's full core-language-microbench base/candidate A/B with identical labels, preserve timings.tsv and compare every hot/neighbor row; use the step's new labels and the previous ones. Run two paired N/2N hardware-instruction samples (taskset/perf command specified there); candidate builds and any allocation arms use `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, with independent fresh target directories and source/binary hashes. All profile flags off for timing, stdin closed. Attribute regressions with bounded callgrind; never claim a 16-unit/1-unit comparison passes. A target or neighbor regression is a defect to fix or report for the director. No favorable-only reruns.

#### Checkpointing — protect the scene

All raw logs, source snapshots, red rows, workload .rum files, inventory deltas, command ledger and PROGRESS.md go under `.temp/brief-NNN/`. Save evidence after red capture, each compiling candidate and each gate. Record counters, compilation state, exact next action and every unresolved risk. Use no git; leave the tree uncommitted for the director's authoritative gate.

#### Hard rules

- No git in any form, no network, no outside-repository activity. Only scoped edits/file reads/rg/Python and the listed bounded Cargo, CLI, microbench, alloc/instruction/callgrind commands; the director must retain those command grants in the dispatched brief.
- No production edit outside the named scope or certified-body fence. STOP on an unapproved inventory decision rather than hiding it in a fingerprint refresh. Preserve user changes.
- Testing Policy is permanent: rum-vs-rum only; no other-vendor engines, artifacts or compatibility work. No new unsafe. Run no probe that aborts a process.
- No ordinary-path or neighbor regression waived by a “cold” intention; no leased-memory or recursion redesign inserted into this continuation repair.
- At most eight focused red/green runs and two completed broad suites absent a newly explained failure; two paired performance samples per candidate. Stop on two consecutive non-progress compiling attempts as above.

#### Report format (append to the dispatched brief)

1. Changed files/functions and each frame/root field, final size/split assessment.
2. Verbatim red/green/lifecycle/compiler/ownership/certificate/workspace results and commands/bounds/exits.
3. Exact named obligation delta, passing fixture counts, every changed inventory row, one-unit per-label timing/instruction/allocation rows and neighbor verdict.
4. Counter/stop-loss state, unresolved/new defects and decisions for the director. Do not claim the frozen predicted count proves a new whole-tree census.

### brief-NNN — G5 C: rule-valued readers and close the legacy escape hatches (draft for director rewrite)

Implementation draft. No git in any form. Requires the director's explicit rewrite of the I04 decision; this is not authorization to implement during brief-576.

#### Goal

Complete default option Rule wrapping into CheckAbort and ArrayFilter, then close the remaining legacy reader surface and tail-use inventory. Repair exactly P0452/P0979/P1708; predicted remaining Lost **17 → 14**, G5 **3 → 0**. The overall prediction is the director’s incoming frozen 43 minus G5’s 29, not a new whole-tree census.

Named rows: **P0452, P0979, P1708**. Exact path:line sites are in brief-576 inventory and g5-current-rows.json.

#### Context pointers

Read AGENTS.md, docs/testing-strategy.md, docs/code-organization-guidelines.md and brief-576's appended report. Its `.temp/brief-576/{inventory.md,design.md,inventory-plan.md,performance-plan.md,reader-references.tsv,g5-current-rows.json,cases.json,probe.rs,oracle.tsv,lifecycle.tsv}` are the source inventory, design, exact incoming red evidence and proposed authorization list. Read suspension/core.rs:764-855, the owning existing body/resume drivers, options/core.rs and retention_closure/local/{certificates,production,contracts}.rs. Template pitfall 8 and briefs 563/568/572-575 govern capture, roots and measurement. The director must rewrite this draft and settle the explicit I04 inventory/API decision before dispatch; this draft grants no approval by itself.

#### Scope

**Production edit surface:** `runtime/options/{core.rs,read_phase.rs}`, narrowly necessary `options.rs`/`kernel_api.rs` exports or visibility; `kernel_data/control_data/abort_cleanup.rs` and `abort_cleanup/options.rs`; stdlib `functional_ops/arrays/{mod.rs,options.rs}`. No tensor/operator-driver change. Complete all semantic test modules, the runtime read_phase compile controls and stdlib read_inventory; remove only the final named legacy reader allowances.

**Named authorization:** I01's rule-reader signature and final private legacy capability closure; I02 F03/S13/S14; I03 affected rows; I04 Session::option_rule_expr_for_symbol(next) input (director must authorize/dispose exact discovery); I05; I06 ArrayFilter anchors if moved; I07/I08. Derive list **+3**, no further named frame/abandon changes: final total remains **560** (+1 from incoming), derives **+24** cumulative. Existing 33-member merged reentry domain and four unresolved D3 consumer rows stay unchanged.

**Standing duty, always in scope:** the existing retention-closure and ownership-graph inventory files, rum-runtime's two frame lists, and necessary tests of those exact invariants. Replay all 68 source contract fingerprints; run `local_production_acceptance` in default and `--no-default-features`, copy emitted cells from `.temp/brief-546/certificates`, then replay and reconcile ownership discovery. That legacy certificate-output directory is the only scratch exception outside this brief's own directory. Disclose all changed keys and lines. New UnknownConstructs in touched code remain unknown; only the director's named I01-I08 exceptions permit old-row identity changes/removals or I04's new host-input dispositions. STOP on any unapproved existing-row/status/disposition/exclusion change, new UnknownHostPayload, new row in untouched code, or new self-reaching state.

**Retention body obligation:** C01 core.rs:87-103 and C02 :190-204 stay verbatim, as do accounting drivers, OptionReentry/domain and the certified 33-member consumer protocol. Both certificate runs must earn the same two C01/one C02 marks on their merits. Fresh hashes alone do not pass. If a certified body must change, STOP for a rewritten local-proof obligation; do not relax the checker or marks.

**Fence:** no other suspension group; no Expr/SharedExpr/SharedArgs/serde representation change, broad evaluator/matcher redesign, leases, new unsafe, dependencies, Cargo.lock, catalog contract reclassification or overlapping-request policy change. No AGENTS.md, TODO.md, root PROGRESS.md, earlier briefs or other docs edits. The new internal helper rustdoc/compile tests document the changed kernel API; the rum-host API stays unchanged. Append the report only to the dispatched brief. No unconditional diagnostics/counters/locks/extra allocations on the ordinary path.

#### Deliverable

- The shared RuleOptionDecode retains the requested Symbol and produces Some(Rule[option,value]) on both paths; absence yields None directly. Only then run CheckAbort’s default decoding/explicit read/primary flow or ArrayFilter padding decoding/operator. Preserve CheckAbort’s unconditional default read despite an explicit override and ArrayFilter’s skipped default with explicit padding. Cold ArrayFilter reparse is of already-evaluated args and pure shape/radius only.
- Red first: CheckAbort-option and twice576 (True vs 7; log {2} vs {2,3}); ArrayFilter-option and twice576 (0 vs List[3,6,5]); unread explicit-padding control. Add targeted second suspension in explicit CheckAbort RHS/body and ArrayFilter operator after the option phase, and cancellation at a suspended rule option; preserve scope cleanup. These additional tests test the shared handoff into existing body owners rather than expand their implementations.
- Run **all 70** archived scratch fixtures as owning permanent tests (66 queued/resumed cases and four cancel/abandon cases), plus reader capability/root/tail controls. No InputRequired may complete with the decoded option as the whole builtin. Final discovery shows zero legacy evaluating reader calls outside the private scalar OptionValue exception and existing certified merged machinery. Raw non-evaluating Input/Dialog/Once calls remain explicitly recorded.
- Keep A/B tests and performance labels green; add performance-plan C labels. Run the final full default workspace and both retention feature gates, publish cumulative pre-G5 vs final one-unit per-label results and every inventory change; leave the remaining 14 frozen obligations out of scope.

#### rum / Rust pitfalls (read twice)

1. Red tests assert equality, actual suspension, exact prompts/order, value/error/termination, messages, prints, log, no pending owner and `1+1`. Never make the pre-fix value the expected answer.
2. Every read API returns its consumer's Expr. Use a tail call; a later unframed `?` discards that guarantee. Compile-fail/pass and source inventory tests enforce the interface and tail convention.
3. Capture only InputRequired, on the error edge. Ordinary errors do not freeze; immutable frames remain Session-rooted during resume; make working state only on an actual Ok resume. No Session borrow, observer, NativeStackDepth, Arc::try_unwrap or interior mutable frame. A second suspension installs the new cursor once. Pure abandonment runs no pending code.
4. Preserve exact read order and conditional reachability, delayed-vs-ordinary policy, current late default lookup and completed guest prefixes. Existing body scopes/frames keep their cleanup duties.
5. Hot error-arm code can change codegen. Both base and candidate, including allocation builds, use CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1. No owned-tree cloning or extra frame allocation on success. Runtime timing uses AbsoluteTiming/Instant, never wall-clock subtraction.
6. Two consecutive **compiling** candidates moving none of repaired rows, passing fixtures, regressing labels or worst regression trigger STOP. Falling compiler-error counts are progress. Never weaken assertions or drop labels to move counters.

#### Verification (MANDATORY — run to green or evidence)

All commands serial and bounded, with `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<repo>/.temp/brief-NNN/tmp CARGO_TARGET_DIR=<repo>/.temp/brief-NNN/target`. Bounds inherited from accepted brief-575: focused 150 s (brief-576 scratch compiled in 65 s and reran in 7.27 s), runtime/retention 300 s, one-unit release build 900 s, each instruction process 150 s, workspace 2400 s. No overlapping CPU-heavy work. The director should preserve/calibrate each script bound from same-shape logs when rewriting.

Run red tests before production edits, then the step's focused target until green, then:

```text
python3 scripts/run-bounded.py --timeout 60 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp cargo fmt --check
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --test option_reader_continuations
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --lib options_suspension
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<scratch>/tmp CARGO_TARGET_DIR=<scratch>/target cargo test --workspace --no-fail-fast
```

Use the existing replay to refresh fingerprints before expecting acceptance; its first stale-cell failure may emit the new cells, which must then be copied/reconciled mechanically and checked again. The broad workspace gate includes doctest compile controls; also run the targeted read_phase compile-pass/fail tests and the source-gate falsifiers as soon as the interface exists. Paste real output, errors, skip counts and exits. The design-only scratch target is not a substitute for these gates.

**One-unit performance gate:** run performance-plan.md's full core-language-microbench base/candidate A/B with identical labels, preserve timings.tsv and compare every hot/neighbor row; use the step's new labels and the previous ones. Run two paired N/2N hardware-instruction samples (taskset/perf command specified there); candidate builds and any allocation arms use `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, with independent fresh target directories and source/binary hashes. All profile flags off for timing, stdin closed. Attribute regressions with bounded callgrind; never claim a 16-unit/1-unit comparison passes. A target or neighbor regression is a defect to fix or report for the director. No favorable-only reruns.

#### Checkpointing — protect the scene

All raw logs, source snapshots, red rows, workload .rum files, inventory deltas, command ledger and PROGRESS.md go under `.temp/brief-NNN/`. Save evidence after red capture, each compiling candidate and each gate. Record counters, compilation state, exact next action and every unresolved risk. Use no git; leave the tree uncommitted for the director's authoritative gate.

#### Hard rules

- No git in any form, no network, no outside-repository activity. Only scoped edits/file reads/rg/Python and the listed bounded Cargo, CLI, microbench, alloc/instruction/callgrind commands; the director must retain those command grants in the dispatched brief.
- No production edit outside the named scope or certified-body fence. STOP on an unapproved inventory decision rather than hiding it in a fingerprint refresh. Preserve user changes.
- Testing Policy is permanent: rum-vs-rum only; no other-vendor engines, artifacts or compatibility work. No new unsafe. Run no probe that aborts a process.
- No ordinary-path or neighbor regression waived by a “cold” intention; no leased-memory or recursion redesign inserted into this continuation repair.
- At most eight focused red/green runs and two completed broad suites absent a newly explained failure; two paired performance samples per candidate. Stop on two consecutive non-progress compiling attempts as above.

#### Report format (append to the dispatched brief)

1. Changed files/functions and each frame/root field, final size/split assessment.
2. Verbatim red/green/lifecycle/compiler/ownership/certificate/workspace results and commands/bounds/exits.
3. Exact named obligation delta, passing fixture counts, every changed inventory row, one-unit per-label timing/instruction/allocation rows and neighbor verdict.
4. Counter/stop-loss state, unresolved/new defects and decisions for the director. Do not claim the frozen predicted count proves a new whole-tree census.

## 6. Open questions for the director

1. **Approve the typed CPS surface and kernel helper signature changes?** Recommended: yes, with ordinary borrowed working descriptors, one private generic frame and required typed completion. The two public Session getters will no longer return a bare bool/optional rule; the stable rum-host request surface stays unchanged. Retaining an unrestricted old getter would preserve the defect's compile-time escape hatch.
2. **Authorize I01-I08, particularly I04's new generic input/owned-port rows and the named helper splits?** Recommended: explicitly authorize the semantic sites and required PublicInput/storage dispositions in the rewritten A/C scopes, then stop on any additional discovered host payload. Keep unknowns unknown. Without this decision the standing duty requires the executor to stop on those new rows; refreshing a fingerprint cannot authorize them.
3. **Accept the cold-only pure Names-pattern/ArrayFilter shape reparse, and the three-step split?** Recommended: yes; retain actual pre-read guest results and DumpSave's session-dependent expanded filters, rebuild only the two pure datasets specified. This keeps recursive regex/tensor types out of new frame roots and isolates the hot boolean gate (A), ordered phases (B), and rule wrapping/final capability closure (C). If exact normalized-state retention is preferred, add the extra root types/rows and size assessment before dispatch.
4. **Confirm the implementation performance command grant and baseline handoff.** Recommended: the full rum-vs-rum timing suite plus the named small/default/latent/override instruction panel, both arms one codegen unit; director-provided incoming binary for A and preserved accepted binaries for B/C. Hardware perf/alloc builds and broad suites belong in the implementation briefs, not this design brief's restricted commands. Report any unavailable metric or residual regression for a director decision; do not change the gate silently.

The predicted frozen-count sequence is 43 → 31 → 17 → 14; G5 29 → 17 → 3 → 0. This report reproduced G5, not the director's entire 37-program remaining-divergence rerun, and does not certify the other 14 obligations or the separate Unsettled list. No implementation is authorized or attempted here.

### Final handoff

1. **Changed:** design only. The [report and drafts](/home/remi/dev/rum/docs/dev/codex/brief-576-option-reader-continuation-design.md:139) are appended; the scratch test is archived and deleted. No production edits or git commands.
2. **Inventory:** all 29 G5 obligations and their current callers are documented.
3. **Design:** typed reader continuations preserve evaluation order; one shared frame; C01/C02 bodies stay unchanged.
4. **Tests:** 70 run — **62 red, 8 passing controls**. All 26 original witnesses reproduced.
5. **Drafts:** three TEMPLATE-shaped briefs; predicted remaining Lost: **43 → 31 → 17 → 14**.
6. **Director decisions:** kernel helper signatures, inventory exceptions, cold reparse choices, and performance gates.
