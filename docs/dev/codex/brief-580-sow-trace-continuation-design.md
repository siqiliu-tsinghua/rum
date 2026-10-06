# brief-580 — design: Sow tag matching and trace filters that resume (group G7)

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

Group G7, the last group of the suspension-continuation branch: two places
run guest pattern tests in the middle of their own work and resume **past**
that work.

- **`Sow` against `Reap`'s tag pattern** (`debug_tools/reap.rs:38`,
  `eval_sow` → `match_expr_owned`, P0125): `Reap[Do[Sow[x, x], {x, {1, 2, 3}}],
  _?(…InputString…)]` resumes to `{Null, {{1}, {3}}}` where the queued path
  returns `{Null, {{1}, {2}, {3}}}` — the sown value whose tag test suspended
  is lost.
- **The structured trace filters** (`debug_tools/trace.rs`): `Trace`,
  `TraceDetail` and `TracePrint` with a pattern or a level/head filter walk
  the captured trace tree and match each item — `finish_structured_trace_capture_expr`
  → `filter_structured_trace_expr` → the recursive `filter_structured_trace_items`
  and `filter_structured_trace_leaf`, and the detail family
  `filter_structured_trace_detail_items` → `trace_detail_item_matches` →
  `trace_detail_underlying_expr_matches` → `trace_detail_candidate_matches` →
  `match_expr_owned`. Thirteen sites, P0199-P0213 (the list is in
  `.temp/brief-548/draft-G7.md`).

The director re-ran brief-548's probes on `be3ad44f`
(`.temp/director/brief548-probe-rerun-be3ad44f.txt`): eleven diverge, all
G7 (`Sow-match`, `Sow-twice`, `Trace-filter-heads`, `Trace-filter-levels`,
`Trace-pattern`, `Trace-twice`, `TraceDetail-filter-heads`,
`TraceDetail-filter-levels`, `TraceDetail-nested-pattern`, `TraceDetail-twice`,
`TracePrint-nested-pattern`). Lost continuations today: 14; G7 accounts for
fourteen.

**Depth:** the trace tree's depth follows the evaluation's nesting, and the
director measured the evaluator's own native-stack guard failing closed for
guest recursion between 60 and 80 levels (`f[k_] := f[k - 1] + 1`, release,
one codegen unit, 2 MiB and 8 MiB alike; identical on the pre-G2 binary —
`.temp/director/gate-g7/`). So the trace filters walk shallow trees, and the
matcher's own depth belongs to the hardening line (brief-570's boundary). Say
whether anything here can be deep regardless — e.g. a captured expression
nested deeper than the evaluation that produced it — with evidence.

## What the design must settle

1. **The inventory**, with `path:line`: the Sow path (how the tag is matched
   against each enclosing `Reap` pattern, what is collected, in what order);
   the trace filter walks (what is captured, the order items are matched and
   emitted, `TracePrint`'s printing during or after the walk, the level and
   head filters, the detail family's underlying-expression and candidate
   matching).
2. **The design**: what a frame owns at each suspension (for Sow: the value,
   the tag, which `Reap` scopes remain to test, what is already recorded; for
   the filters: the walk position, the emitted prefix, pending item), how the
   answer is applied once, a second suspension, abandonment (no skipped match
   runs; `TracePrint` output emitted exactly once). Choose between a flat
   work stack and per-level frames for the filters, with reasons — the
   accepted precedent is a flat walk (briefs 563, 568, 572-573), but the trees
   here are shallow; correctness and enforceability decide, not size.
3. **The protocol**: brief-563/568/572-579's settled rules — capture only on
   `InputRequired` on the error edge, immutable Session-rooted frames with a
   cold working copy on an actual resume, `FrameRootTrace` / `PureFrameAbandon`,
   no `Session` borrow, observer or `NativeStackDepth` in a frame, rum-runtime's
   frame lists.
4. **The ordinary path.** `Sow` without a suspending tag test and an unfiltered
   or non-suspending filtered `Trace` must not pay; name the labels and the
   plan — one codegen unit on both arms, a per-label base-against-base noise
   floor first (`TEMPLATE.md` pitfall 8; brief-578).
5. **Inventory, census and layout consequences**, listed so the director can
   authorize them by name, and the step split if any.

## What I want back

1. The inventory with evidence, and the depth assessment.
2. The design, the frame inventory, and the choice.
3. Red tests, run in the scratch test and recorded: all eleven brief-548 G7
   witnesses through the rum-vs-rum oracle (value, `log`, messages, prints —
   `TracePrint`'s printed lines especially — suspension asserted, exact
   prompts, no owner after, `1 + 1`), the twice variants, nested `Reap`
   scopes with different tag patterns, an ordinary error in a tag test after a
   resume, cancellation at a suspended tag test and at a suspended trace
   filter.
4. Drafted implementation brief(s) in `TEMPLATE.md` shape — red tests,
   predicted lost-continuation count, named inventory rows and frame lists,
   the one-unit performance gate with noise floors, the standing duty, serial
   runs (`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`).
5. Open questions for the director.

## Context pointers

- brief-548's report, `.temp/brief-548/draft-G7.md`, `probes.json`,
  `lost.tsv`, `probe.rs`.
- briefs 563, 568, 572-579 and their reports (the accepted continuation,
  structural and measurement patterns); brief-570 (the matcher depth boundary).
- `crates/rum-runtime/src/runtime/debug_tools/{reap.rs,trace.rs}`,
  `suspension/core.rs`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-580/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief580_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-580/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-580/`.
- **Run no probe that aborts a process.**
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief580_probe`
  (150 s); `.temp/brief-579/candidate-04/rum-cli` (production source identical
  to HEAD) on passing programs (60 s); `valgrind --tool=callgrind` on it
  (150 s). File reads, `rg`/`grep`, `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The inventory and the depth assessment.
3. The design, frame inventory, and the choice.
4. The red tests with their pre-fix values.
5. The drafted implementation brief(s).
6. Open questions for the director.

---

## Report — brief-580 (2026-10-06)

### 1. What changed

**Design only; no production change.** Scratch evidence and the two drafted
implementation briefs are under `.temp/brief-580/`. The sole permitted scratch
integration target, `crates/rum-stdlib/tests/zz_brief580_probe.rs`, was run twice,
its exact final content copied to [probe.rs](../../../.temp/brief-580/probe.rs),
and **deleted before this report was appended**. No git or network command was
run, no unsafe was added, and no process-aborting probe was run.

The final target compiled and produced 51 cases: 26 passing and 25 expected
pre-fix equality/prompt failures. This is red design evidence, not a green
implementation gate. All eleven original G7 witnesses reproduce. All 16
cancellation/answer-free abandonment cases pass. The evidence also includes
actual nested trace Lists, differing nested Reap patterns, decline/fallthrough,
same-tag accumulation, errors after resume and exact TracePrint streams.

[commands.json](../../../.temp/brief-580/commands.json) records the four bounded
executions and their exits. TMPDIR and all newly created scratch stay under
`.temp/brief-580/`. One fixed 128-call held-expression CLI source was unexpectedly
rejected by the parser's 128 nesting limit (exit 1, before evaluation); this was
an unsuccessful intended passing witness, and is disclosed rather than counted
as trace evidence. The smaller 96-call source was validated in the scratch test
before its successful release CLI run. No abort threshold search was attempted.
No performance/callgrind, broad production suite, formatting or standing replay
was run: those belong to the implementation drafts.

Before appending, a hash audit of 1,783 existing source/document/inventory files
found no changes, additions or deletions, with the temporary target absent:
[scope-audit-before-append.json](../../../.temp/brief-580/scope-audit-before-append.json).
The only persistent non-scratch edit is this appended report.

### 2. Inventory and depth assessment

The complete ordering and caller evidence is in
[inventory.md](../../../.temp/brief-580/inventory.md), with machine-readable
[g7-sites.tsv](../../../.temp/brief-580/g7-sites.tsv). All paths in this table
start `crates/rum-runtime/src/runtime/debug_tools/`; frozen IDs refer to 548,
while the citations below are current:

| ID | Current path:line | Caller → callee |
| --- | --- | --- |
| P0125 | `reap.rs:38` | `eval_sow` → `match_expr_owned` |
| P0199 | `trace.rs:119` | `finish_structured_trace_capture_expr` → `filter_structured_trace_expr` |
| P0200 | `trace.rs:298` | `filter_structured_trace_expr` → `filter_structured_trace_items` |
| P0201 | `trace.rs:321` | `filter_structured_trace_detail_expr` → `filter_structured_trace_detail_items` |
| P0202 | `trace.rs:342` | `filter_structured_trace_detail_items` → `filter_structured_trace_detail_items` |
| P0203 | `trace.rs:364` | `filter_structured_trace_detail_items` → `trace_detail_item_matches` |
| P0204 | `trace.rs:386` | `trace_detail_item_matches` → `trace_detail_underlying_expr_matches` |
| P0205 | `trace.rs:410` | `trace_detail_underlying_expr_matches` → `trace_detail_candidate_matches` |
| P0206 | `trace.rs:425` | `trace_detail_underlying_expr_matches` → `trace_detail_candidate_matches` |
| P0208 | `trace.rs:453` | `trace_detail_candidate_matches` → `match_expr_owned` |
| P0209 | `trace.rs:477` | `filter_structured_trace_items` → `filter_structured_trace_items` |
| P0210 | `trace.rs:500` | `filter_structured_trace_items` → `filter_structured_trace_leaf` |
| P0211 | `trace.rs:535` | `filter_structured_trace_leaf` → `match_expr_owned` |
| P0213 | `trace.rs:568` | `filter_structured_trace_leaf` → `match_expr_owned` |

`Sow` tests scopes **outermost to innermost**, patterns in each scope's order,
and saves accepted pairs (`reap.rs:32-43`). It commits this Sow to buckets only
**after all tests finish** (`:45-57`), keeping first-seen tag order and value
order (`:341-368`). A List tag is one tag expression. The Session owns all Reap
scopes; existing Reap frames own their pop/render/abandon responsibilities
(`:90-114,404-411`). No current-Sow bucket has been committed at a suspended
match; prior completed Sows and predicate side effects may already exist.

Trace capture restores the old tracing flag before filtering
(`trace.rs:106-151`). The builder uses recorded/active-expression depths
(`:178-282`, `trace/builder.rs:16-72`). Lists are filtered depth-first with
left-to-right siblings; empty children disappear, and the existing single-List
child collapse is preserved (`trace.rs:330-371,465-509`). Structured leaves
match the underlying Hold expression or its selected parts and preserve
adjacent-only deduplication. Detail matches Input→Result, Before→After or Value,
then candidates and pattern alternatives; its first true retains the full
association once (`:374-463,652-668`). RuleLeft/metadata are not candidates.
Parts use the existing postorder collector with Heads/level policies
(`tree_walkers/level.rs:23-39,62-144`).

`TracePrint` prints **after the whole filter**, never during it; indentation is
computed from the completed trace and it returns the saved body result
(`trace.rs:127-134,726-761`). Predicate prints/messages precede those lines.
Errors/cancellation before completion emit no pending trace lines. Matcher
continuations return a CompiledMatchOutcome encoding, which must be decoded
(`pattern_matching/outcome.rs:9-48`), never exposed as a trace or Sow value.

P0198 (`trace.rs:111`) was already TailCorrect on the Detail branch. P0207
(`:433`) and P0212 (`:558`) are dependent local-tail U5 obligations, not two
additional Lost sites. The new filter absorbs their routing and discharges the
caller obligations without inflating the frozen count. The other Unsettled
population is not certified by this work.

**Depth:** the director's 60–80-level guard observation supports shallow ordinary
trace List topology, not a universal hard maximum. A captured expression can
be deeper: [held-depth-96.rum](../../../.temp/brief-580/held-depth-96.rum) holds 96
nested f calls under `$RecursionLimit=32`; the trace has one item, yet selecting
level 97 reaches x. Both the scratch test and supplied release binary returned:

```text
List[1, List[Hold[x]]]
```

The level collector already has a 16-call bounded front and an explicit traversal
fallback. Matcher/compiler/normalizer and shared Expr operations retain
brief-570's separate hardening boundary. No extra depth cap, deep abort suite,
or universal stack-safe-destruction claim is proposed.

### 3. Design, frame inventory and choice

[design.md](../../../.temp/brief-580/design.md) settles the transitions and
field/root ownership. Choose **one flat filter work stack and one named filter
frame**, plus one Sow match frame. Per-level frames could preserve the existing
recursive ordinary walk, but would leave multiple boolean/Vec/Expr caller
completion obligations and a Print finisher to enforce separately. Flat state
keeps the pending match, all ancestor prefixes and presentation mode together;
its owned control graph and edges can be checked without relying on an empirical
trace-depth bound. This is a correctness/enforcement choice, not a measured
claim of zero ordinary-path cost.

Sow's frozen cursor retains value, tag, initial scope count, active scope/pattern
indices and count, and accepted pairs. A resumed matched flag adds the pending
pair once, then advances once. False advances without recording; a later error
commits none of this Sow. Completion runs the existing bucket and occurrence-
charging loop once, returns the original value, and leaves scope cleanup to
existing Reap frames. Pure abandonment runs no missing match or bucket commit.

The flat trace driver owns current-level remaining siblings/output prefix,
parent levels, pending original item, detail-field/candidate/pattern indices,
selected candidates and the leaf's accepted prefix. It freezes only on matcher
InputRequired. On success it decodes/applies once; false advances pattern, then
candidate, then detail field; detail true emits its original association once
and skips the remaining search. Structured completion wraps original candidates
in Hold and performs the existing deduplication/shape rules. Root completion
alone may render/push TracePrint lines and return the saved body result.

| New type | Frozen fields / responsibility |
| --- | --- |
| SowMatchState | value/tag roots; scope limit/index, pattern index/count; ordered accepted pairs |
| SowMatchSuspensionFrame | SowMatchState; matcher answer and pure abandon |
| TraceFilterState | existing output mode; saved body value; pattern/levels/Heads; current level, flat parents and pending leaf |
| TraceFilterLevel | remaining Expr siblings and emitted Expr prefix |
| TraceFilterLeaf | original item, underlying expressions/index, patterns/index, candidate source/index and selected leaf prefix |
| TraceFilterCandidates | inline Single(Expr) or Parts(Vec<Expr>) |
| TraceFilterSuspensionFrame | TraceFilterState; matcher answer and pure abandon |

Seven new FrameRootTrace derives; only the two named frames additionally derive
PureFrameAbandon. Update both closed derive lists in runtime's
`tests/frame_root_trace/derived_inventory.rs` and the named assertion in
`tests/frame_root_trace.rs:483`: **560 → 561 → 562**. Existing capture/Reap
frames and custom abandon behavior remain intact. No new erased host payload,
manual root implementation or finalizer is proposed.

Follow the settled protocol (`suspension/core.rs:545-563,764-855`): error-edge
capture only for InputRequired; immutable Session-rooted Arc frames; cold
working copy only on actual successful resume; direct error forwarding; updated
state on second suspension with the generic driver preserving the suffix once.
No Session borrow, observer, NativeStackDepth, mutable frame interior or
Arc::try_unwrap. The filter never restores capture a second time. New structural
and root checks must have observed falsifiers for lost/replayed answers, skipped
false fallthrough, missing prefixes/roots, early Print, recursive control edges
and self-reaching state.

Ordinary Sow keeps its two-phase loop. Normal Trace must not allocate frames or
snapshot state; keep the root/current candidate inline and freeze only remaining
iterator tails at suspension. Do not return raw unfiltered builder output,
because shape normalization still applies. The flat parent vector's ordinary
nested-trace cost is a **specific acceptance risk**. It must meet the allocation,
instruction and timing gates; it is not waived as an inevitable refactor cost.

[performance-plan.md](../../../.temp/brief-580/performance-plan.md) names 42 exact
new no-input statements in [performance-labels.json](../../../.temp/brief-580/performance-labels.json),
covering Sow's scope/tag/predicate variants and all three trace names' plain,
filtered, decline, levels, Heads and actual nesting paths. It also retains
matchq/matcher-single, guarded rewrite, cases/count/position/deletecases,
replaceall, take, module-retained-gensym/Sow and small Dot/Differences controls.
Both arms use one codegen unit. First measure per-label base-against-base noise
floors with two pairs; then allocation count/bytes and two instruction samples.
A label regresses when both samples exceed `max(+0.05%, floor)`; extra ordinary
allocation is separately reported. Run the full rum-vs-rum microbench timing A/B
with identical label definitions and inspect every hot/neighbor row. No
performance evidence or pass is claimed today.

### 4. Red tests and pre-fix values

Full exact values, logs, errors, messages, prints, prompts and lifecycle rows:
[red-results.md](../../../.temp/brief-580/red-results.md),
[red-results.json](../../../.temp/brief-580/red-results.json),
[oracle.tsv](../../../.temp/brief-580/oracle.tsv),
[suspensions.tsv](../../../.temp/brief-580/suspensions.tsv),
[lifecycle.tsv](../../../.temp/brief-580/lifecycle.tsv).

The final bounded command was:

```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-580/tmp cargo test -p rum-stdlib --test zz_brief580_probe
```

Verbatim final output excerpts (exit 101):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.28s
running 51 tests
test result: FAILED. 26 passed; 25 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
error: test failed, to rerun pass `-p rum-stdlib --test zz_brief580_probe`
```

The first run was 25 pass / 22 fail across 47 tests. Final run added three actual
nested-tree witnesses and one depth control; one additional TraceDetail effect
fixture was corrected to produce enough callbacks for its second requested
prompt. The eleven original548 sources are unchanged. Every final comparison
checks no pending input/owner and successful `1+1` before the equality assertion.
The actual/required prompts remain asserted even when the bug skips prompt2.

Original eleven, exact pre-fix values (their complete log/effect columns are in
the linked red results):

| Witness | Resumed value | Queued value | Prompts reached / required |
| --- | --- | --- | --- |
| Sow-match | `List[Null, List[List[1], List[3]]]` | `List[Null, List[List[1], List[2], List[3]]]` | 1 / 1 (`p: ` each) |
| Sow-twice | `List[Null, List[List[1], List[3], List[5]]]` | `List[Null, List[List[1], List[2], List[3], List[4], List[5]]]` | 2 / 2 (`p: ` each) |
| Trace-pattern | `List[True, List[]]` | `List[Hold[Plus[1, 2]], Hold[3]]` | 1 / 1 (`p: ` each) |
| Trace-filter-heads | `List[True, List[]]` | `List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[7], Hold[f], Hold[3], Hold[7], Hold[f[3, 7]]]` | 1 / 1 (`p: ` each) |
| Trace-filter-levels | `List[True, List[]]` | `List[Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[1], Hold[2], Hold[3], Hold[4], Hold[3], Hold[7]]` | 1 / 1 (`p: ` each) |
| Trace-twice | `List[True, List[]]` | `List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]` | 1 / 2 (`p: ` each) |
| TraceDetail-nested-pattern | `List[True, List[]]` | `List[Association[Rule["Step", "EvaluationPass"], Rule["Before", Hold[Plus[1, 2]]], Rule["After", Hold[3]]], Association[Rule["Step", "EvaluationPass"], Rule["Before", Hold[Plus[3, 4]]], Rule["After", Hold[7]]], Association[Rule["Step", "EvaluationPass"], Rule["Before", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule["After", Hold[f[3, 7]]]]]` | 1 / 1 (`p: ` each) |
| TraceDetail-filter-heads | `List[True, List[]]` | `List[Association[Rule["Step", "EvaluationPass"], Rule["Before", Hold[Plus[1, 2]]], Rule["After", Hold[3]]], Association[Rule["Step", "EvaluationPass"], Rule["Before", Hold[Plus[3, 4]]], Rule["After", Hold[7]]], Association[Rule["Step", "EvaluationPass"], Rule["Before", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule["After", Hold[f[3, 7]]]]]` | 1 / 1 (`p: ` each) |
| TraceDetail-filter-levels | `List[True, List[]]` | `List[Association[Rule["Step", "EvaluationPass"], Rule["Before", Hold[Plus[1, 2]]], Rule["After", Hold[3]]], Association[Rule["Step", "EvaluationPass"], Rule["Before", Hold[Plus[3, 4]]], Rule["After", Hold[7]]], Association[Rule["Step", "EvaluationPass"], Rule["Before", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule["After", Hold[f[3, 7]]]]]` | 1 / 1 (`p: ` each) |
| TraceDetail-twice | `List[False, List[]]` | `List[]` | 1 / 2 (`p: ` each) |
| TracePrint-nested-pattern | `List[True, List[]]` | `f[3, 7]` | 1 / 1 (`p: ` each) |

The nested different-pattern Sow case asks only `outer: ` on resume instead of
`outer: ` then `inner: `, loses tag2 in both enclosing scopes, and skips the two
inner tests for that tag. Direct suspended Sow returns the matcher payload as
the Reap body instead of77. Repeated-tag Sow produces `{1,3}` instead of
`{1,2,3}`. A later-pattern Throw is skipped on the broken resumed path; a Throw
inside the already-running predicate after its input agrees queued/resumed and
is retained as an ordinary-error control.

For `TracePrint-nested-pattern`, resumed prints are `[]`; queued lines are,
verbatim:

```text
f[Plus[1, 2], Plus[3, 4]]
Plus[1, 2]
3
Plus[3, 4]
7
f[3, 7]
```

The additional actual definition-nesting case has resumed prints `[]` and
queued lines `"f[3]", "  Plus[g[3], 1]", "  g[3]", "    Plus[3, 1]",
"    4", "  4", "  5", "5"`, returning5. The effectful twice case records only
`before` and one message after the broken resume, while its queued path records
`before`, `after`, two messages, then all six trace lines. This checks callback
streams and final trace presentation independently.

All 16 cancellation cases (Sow/Trace/TraceDetail/TracePrint × first/second prompt
inside the pending predicate × resume-after-cancel/answer-free abandonment)
return `TerminatedEvaluation["Cancelled"]`, termination `Some(Cancelled)`,
`log={1}` or `{1,2}`, prints `["before"]`, and exactly probe::tag then
General::cancel. None executes the suffix append3 or trace-line output. Each
also checks fresh Reap/Trace behavior after cancellation. These second prompts
are within the predicate; distinct later-candidate/scope prompts remain the
red twice witnesses and must not be confused with passing cancellation controls.

### 5. Drafted implementation briefs and inventory/layout consequences

Both drafts contain TEMPLATE's Goal, Context, Scope, Deliverable, pitfalls,
Verification, Checkpointing, Hard rules and Report sections. They are raw
material for the director to rewrite, not implementation authorization.

| Draft | Gate and ownership | Predicted frozen Lost / named frames |
| --- | --- | --- |
| [A — Sow](../../../.temp/brief-580/draft-A-sow.md) | P0125;13 Sow cases (7 red, 6 controls); Sow state/frame and exact deferred commit | 14 → 13;560 → 561 |
| [B — trace](../../../.temp/brief-580/draft-B-trace.md) | All 13 trace IDs;38 trace/depth cases (18 red, 20 controls), retain A; flat state/filter frame and exactly-once presentation | 13 → 0;561 → 562 |

[inventory-plan.md](../../../.temp/brief-580/inventory-plan.md) names I01–I07 for
explicit authorization: frozen-ID mapping; seven derives/two named frames;
ownership rows and exact replaced-construct retirements; retention census and
both certificate replays; module/test layout; performance harness; documentation
boundary. [existing-inventory-rows.json](../../../.temp/brief-580/existing-inventory-rows.json)
preserves 79 relevant existing rows: 18 unknown attributes, 21 captures, 33 macros,
one host-payload ledger witness, one self-reaching row and five obligations.

The notable requested retirements are the old trace wrapping-map closures
`:552:26,:593:26` and six replaced vec! sites
`:519:23,:552:70,:559:57,:575:25,:593:70,:596:32`. All other identities/statuses/
exclusions remain. Unknown constructs in new/changed code stay unknown; new
host payloads, unexpected disappearances or rows in untouched code require a
stop. Sow/Reap/Trace/TraceDetail/TracePrint remain **Unsplit/Pending** in the
retention census. Existing ReapRenderNode debt and finalizer witness disposition
remain; source positions may move.

Move eval_sow into `debug_tools/reap/sow.rs`, shrinking20,002-character reap.rs;
extract trace filtering into `trace/filter.rs` and `filter/{leaf,state}.rs` from
26,912-character trace.rs. Keep capture/options/builder/render responsibilities
and existing frame identities stable. New modules aim below 10k, facades below 20k;
debug_tools direct children rise 17→18. Tests split by Sow, trace and lifecycle,
with shared oracle support and private root/source checks.

Both drafts require serial bounded commands with
`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true`, scoped TMPDIR,
focused red-to-green tests, runtime/workspace gates and source/root falsifiers.
They carry the standing duty: refresh global contract fingerprints by replay,
earn/copy both feature certificate cells, reconcile ownership discovery and
disclose every row. C01/C02 and proof strengths remain unchanged. The frozen
Lost=0 prediction is not a new whole-tree continuation census or retention proof.

### 6. Open questions for the director

1. **Approve the two-step scope and flat filter choice?** Recommended as drafted;
   its ordinary nested-trace allocation/codegen risk is subject to the gate.
   If it cannot meet the gate within the bounded attempts, require an attributed
   report and a rewritten next step, rather than silently choosing per-level
   frames or accepting a regression.
2. **Authorize I01–I07, especially I03's exact old-construct retirements and I02's
   seven derives/two named frames?** Without named authorization, the standing
   rule requires an implementation to stop on those disappearing old rows.
   No new host payload or census reclassification is requested.
3. **Supply/confirm each stage's immutable one-unit base, allocation variant,
   hardware instruction command and full-microbench parent bound.** Prior
   578/579 bounds cover focused/runtime/metric/build/workspace commands; the
   current design command grant does not authorize executing those future
   performance gates. Keep per-label floors and cumulative pre-G7 evidence.
4. **Keep the demonstrated deep-expression boundary separate?** Recommended:
   retain the safe 96/32 regression control, reuse the already bounded level
   collector, and leave matcher/shared-Expr hardening and generic evaluator
   recursion behavior to their existing line. Do not infer a universal trace
   depth maximum from the60–80-level observation.
