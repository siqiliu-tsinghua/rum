# brief-570 — design: rule and held-pattern continuations (group G4)

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

Group G4: rule left-hand sides and held patterns are normalized by evaluating
their ordinary subexpressions while keeping pattern constructs held, and a
subexpression that asks for input resumes **past** that work. brief-548
recorded sixteen sites (`.temp/brief-548/draft-G4.md` has the list):

- `definitions/definitions.rs:85, 108` — `eval_rule` / `eval_rule_delayed` →
  `evaluate_rule_lhs`; `:177, 187, 195, 204, 217, 222, 227, 229` — the
  recursive `evaluate_pattern_rule_lhs` and its `evaluate` calls;
- `kernel_data/rewrite_data/cases/match_q.rs:50, 68, 71, 83` —
  `resolve_matcher_pattern_expr_owned`, the recursive
  `resolve_match_pattern_subexprs`, `normalize_cases_spec`;
- `pattern_matching/core.rs:18` (`match_expr` → `match_expr_owned`) and
  `pattern_matching_interpreted.rs:320` (`match_expr_interpreted` →
  `…_owned`) — **both callers carry `#[allow(dead_code)]`**.

The director re-ran brief-548's probes on `f127e786`
(`.temp/director/brief548-probe-rerun-f127e786.txt`): 51 still diverge, among
them every G4 witness (`Rule-pattern`, `RuleDelayed-pattern`,
`MatchQ-pattern`, `Cases-pattern`, `Rule-twice`, `MatchQ-pattern-twice`, …).
Lost continuations today: 59; G4 accounts for sixteen.

## What the director's adjacent-boundary probes found

Release `rum-cli`, one codegen unit, source identical to HEAD
(`.temp/brief-569/candidate-09/rum-cli`), ruled 2 MiB stack, `v` a nested
list built incrementally by `Do[v = {v}, {n}]` (probe files in
`.temp/director/gate-g4/`):

| Program | n = 1 500 | n = 6 000 |
| --- | --- | --- |
| `Rule[v, 1]` | passes | passes |
| `MatchQ[1, v]` | `False` | **process abort (exit 134)** |
| `Cases[{1, 2}, v]` | `0` | **process abort (exit 134)** |
| `ReplaceAll[v, 1 -> 2]` | passes | fails closed (`StackLimitExceeded`) |

and, from the G3 probes, `MatchQ[v, w]` on two deep literals aborts between
800 and 1 200 levels. `v` here contains **no pattern syntax**, so
`resolve_match_pattern_subexprs` returns at its first `contains_pattern_syntax`
test; the abort is somewhere else in the MatchQ / Cases pipeline —
`contains_pattern_syntax`, `normalize_match_expr`, matcher compilation, or the
match itself are all candidates. **Do not run any probe that aborts a
process** — Codex's upstream content filter cut brief-567's first run at
exactly that step. Locate the recursion by reading the source; the director
runs any abort reproduction.

## What the design must settle

1. **The inventory.** Every function on the pipelines `Rule`/`RuleDelayed`
   left-hand side, `MatchQ`, `Cases` (and any other consumer of
   `resolve_matcher_pattern_expr_owned` / `evaluate_rule_lhs`) that recurses in
   proportion to the pattern's or value's depth, with `path:line`, whether it
   can run guest code, whether it is guarded, and which brief-548 site it owns.
   Name the most likely site of each abort above, with reasoning from source.
2. **The two dead-code sites.** Are `match_expr` and `match_expr_interpreted`
   reachable from any production path, any test, or any public or
   `#[doc(hidden)]` API? If not, the right repair is removal, not a
   continuation — say so with evidence, and say what removing them does to the
   census and the inventories.
3. **Ownership of depth.** Which recursive walkers G4 rewrites
   (`evaluate_pattern_rule_lhs`, `resolve_match_pattern_subexprs`, …) and
   therefore must make depth-safe — the G2/G3 precedent: a repair that rewrites
   a walker makes it resumable **and** depth-safe in one design, enforced by
   the recursion scanner as `lazy_depth_contract.rs` and
   `schema_depth_contract.rs` do — and which belong to the matcher proper
   (compile, normalize, match), whose depth defect the director has recorded
   for the hardening line. Draw the boundary from evidence, and say what a G4
   depth test can and cannot prove given the matcher's own limit.
4. **The design** for the G4-owned walkers: representation (an explicit work
   stack, or another shape — choose and justify against the guarded
   alternative), what one frame owns (the partly rebuilt expression, the
   cursor, held positions, pending subexpression), apply-once on resume, a
   second suspension, abandonment, and how `eval_rule`, `eval_rule_delayed`,
   `MatchQ`, `Cases` and any other consumer complete. Follow brief-563/568's
   protocol: immutable Session-rooted frames, a cold working copy on an actual
   resume, capture only on `InputRequired` on the error edge,
   `FrameRootTrace`/`PureFrameAbandon`, no `Session` borrow, observer or
   `NativeStackDepth` in a frame, rum-runtime's frame lists.
5. **The ordinary path.** This is the evaluator's pattern pipeline and is hot.
   Name the labels (existing `matchq-*`, `cases-*`, `replace*` and rule
   definition labels, plus new ones with pattern-bearing subexpressions to
   evaluate) and the plan; **both arms with one codegen unit**
   (`TEMPLATE.md` pitfall 8).
6. **Inventory, census and layout consequences**, listed so the director can
   authorize them by name.

## What I want back

1. The recursion inventory and the dead-code finding, with evidence.
2. The design, frame inventory, and the depth boundary.
3. Red tests, run in the scratch test and recorded: every brief-548 G4
   witness through the rum-vs-rum oracle (value, `log`, messages, prints,
   suspension asserted, exact prompts, no owner after, `1 + 1`), twice
   variants, an ordinary error after a resume, cancellation at a suspended
   subexpression, a nested pattern (`h[g[x_, (input)], y_]`), `Condition` and
   `PatternTest` positions that must stay held. Depth cases only at depths the
   current tree survives, as controls; write the post-fix deep cases unrun.
4. Drafted implementation brief(s) in `TEMPLATE.md` shape, with red tests,
   predicted lost-continuation count, the named inventory rows, the structural
   depth checks with falsifiers, the one-unit performance gate, the standing
   duty, and serial runs (`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`).
5. Open questions for the director.

## Context pointers

- brief-548's report, `.temp/brief-548/draft-G4.md`, `probes.json`,
  `lost.tsv`, `probe.rs`.
- briefs 563, 568 and their reports (the accepted flat-walk pattern and its
  structural depth checks); briefs 566 and 569 (ordinary-path tuning, one
  codegen unit).
- `crates/rum-runtime/src/runtime/definitions/definitions.rs`,
  `kernel_data/rewrite_data/cases/match_q.rs`, `pattern_matching/`,
  `pattern_matching_interpreted.rs`, `suspension/core.rs`,
  `native_stack_guard.rs`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-570/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief570_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-570/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-570/`.
- **Run no probe that aborts a process**, in-process or through the CLI.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief570_probe`
  (150 s); `.temp/brief-569/candidate-09/rum-cli` on passing programs (60 s);
  `valgrind --tool=callgrind` on it for attribution (150 s). File reads,
  `rg`/`grep`, `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The recursion inventory, the abort-site reasoning, the dead-code finding.
3. The design, frame inventory, and the depth boundary.
4. The red tests with their pre-fix values.
5. The drafted implementation brief(s).
6. Open questions for the director.

---
