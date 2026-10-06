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
