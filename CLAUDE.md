# CLAUDE.md — director's notes for `rum`

This file is **mine** (Claude): direction, diagnosis, delegation and the
authoritative gate. `AGENTS.md` is what coding agents read. **Keep it under
200 lines** (user ruling 2026-10-29) — it is read on every call. The long
form of each rule, with its incident, is in `docs/dev/director-method.md`;
the direction log is `docs/dev/director-log.md` (history, not the roadmap);
state lives in `TODO.md`, `docs/dev/security-hardening-progress.md` and
memory `suspension-continuation-branch.md`.

## Role split

- **I own** direction, diagnosis, picking the next target, plan review, the
  authoritative gate, the git commit and its push: every commit goes to the
  public `origin` at once, never batched (user ruling 2026-10-30).
- **Workers own** design-only plans, then exactly one implementation brief at
  a time, self-verified to green and reported with verbatim evidence. They do
  not commit, branch or push; they leave the tree for my gate.
- **Plan-first is the default** (user ruling 2026-07-29) for any non-trivial
  or cross-cutting task:
  1. A design-only brief (`Scope: NONE`). The worker plans step by step with
     `path:line` evidence (call-site inventory, invariants, gate tests, risks,
     open questions) and drafts the implementation briefs in `TEMPLATE.md`
     shape.
  2. I review the plan, decide its open questions, and correct or cut what is
     wrong or over-scoped.
  3. I write my own brief from the draft. Mine is what gets committed and
     dispatched, one gated step at a time.

  Trivial mechanical jobs may go straight to an implementation brief.
- Briefs live in `docs/dev/codex/`, in the shape of `TEMPLATE.md`. Commit the
  brief first, then dispatch it with `scripts/dispatch-brief.sh NNN` through
  the Bash tool's `run_in_background: true`. Never pipe it and never
  `&`-detach it.
- The sandbox is `workspace-write`. Escalate to `CDX_SANDBOX=danger-full-access`
  only for process attach, and tell the user. Worker routing, sessions and
  quotas are in memory `brief-dispatch-worker-resolution.md`.
- No repo mutation while a brief is in flight; my commits queue until the
  gate. Archive the dispatch log and its `.git-attempts` before every resume.

## The gate before I credit a brief

1. **Scope.** The diff touches only what the brief allows, and every
   disclosed extra is justified.
2. **Tests.** `cargo fmt --check` and `cargo test --workspace` are green. A
   narrow `-p` gate is allowed only when the change is provably crate-local.
3. **Claims.** I reproduce the evidence myself: totals, sweeps, the standing
   inventories, and for performance work the rum-vs-rum instruction gates. I
   trust the numbers I reproduce, not the report's interpretation, and I read
   the code diff itself.
4. Only then do I commit and dispatch the next brief.

## The standard for choosing (user ruling 2026-08-27)

Pick the best and most correct option; cost arguments do not count. This is a
research project built with coding agents, with no downstream users and no
human labour to budget.

- Never score an option down for its size.
- Do not propose a stopgap now and the real fix later unless the user asks.
- What counts: correctness, semantic loss, structural necessity, open-ended
  obligations, enforceability, measurability.
- A fix that the compiler or a derive enforces beats one that relies on
  memory.
- This retires cost arguments, not design restraint: over-generalized scope
  is still a stop.

## Hard lines

- **Reply to the user in Chinese, always.** Only technical tokens stay in
  English. Briefs, commits and repo docs are English. Watch for drift after
  long English runs and after compaction.
- **The testing policy is permanent** (`AGENTS.md`, `DISCLAIMER.md`): rum's
  own documented behaviour plus rum-vs-rum A/B, nothing else. No differential
  testing against other engines, no compatibility matrices, no committed
  third-party outputs.
- **No default-path cost for diagnostics or off-by-default features.** Use the
  env-gated cold-load `AtomicU8` pattern. Off-by-default state in a hot frame
  is a violation even at zero CPU cost.
- **No new `unsafe` in `rum-core`/`rum-runtime`** (zero today).
- **A line, once opened, is finished** (user ruling 2026-09-21). An inserted
  branch runs to completion before the interrupted line resumes. Present the
  cost of finishing, never a menu that includes abandonment.
- **Do not tag** (user ruling 2026-09-19). `v0.2.0` onward is the
  security-hardening line; `v0.3.0` is considered only once hardening is
  finished. An API break inside the line is fine.
- **Measure honestly.** Profiling is off for any timing number. One variable
  per A/B. Persist raw output before parsing. Derive timeouts from data.
  Elapsed probes use in-kernel `AbsoluteTiming`. No competing load.
  Instruction slopes decide, not timing; build both arms at one path with one
  codegen unit.
- **Host discipline.** All scratch goes in the git-ignored `.temp/`, never
  `/tmp`. Every cargo command runs in `scripts/memory-scope.sh <unit> 14G` at
  `-j8`, tests at `RUST_TEST_THREADS=8` (user ruling 2026-11-01: 876 s serial
  → 315 s at a 6.33 GB peak), with `--no-fail-fast`; a red is reproduced at
  `--test-threads=1` before it is called a defect. Timing, perf and callgrind
  runs are serial and run alone. Probe builds go through
  `scripts/probe-build.py` (stable `.temp/probe/active`, content-addressed
  cache, one build per arm, one `--force` sha256 spot-check per round).
  **After every commit and push, run `python3 -I scripts/clean-temp.py
  --apply` (dry-run first; user ruling 2026-11-07): it prunes `.temp/` —
  never `./target`, which is bounded and now kept — and refuses unless the
  tree is clean and HEAD is pushed.**

## Context economy (user ruling 2026-10-29)

My usage is calls times context. The time-window quota bottleneck ended with
the DeepSeek backend (user, 2026-11-08); context growth still compounds. So:

- Worker reports are `rum-report/1` JSON (`scripts/report.py`, `TEMPLATE.md`).
  I read `summary`, pull a part with `show`, gate scope with `check
  --against-git`, and `render` the record into the brief. Diffs: in full.
- Procedures become scripts, inputs in and one table out: the gate is
  `director-gate.py run NNN [--perf resume,cli]`; long output in `.temp/`.
- Never switch a subagent to a smaller model or lower effort to save tokens:
  recovering from its errors costs more.
- The `rum-director` mod (`tools/claude-mod/rum-director/`) enforces the
  dispatch, commit and push discipline, and compacts after every commit at
  200k tokens or more. Before every commit, rewrite `.temp/director/state.md`
  (the handoff it re-injects). Stage, commit and push in separate calls.

## Diagnosis method, short form

The incidents behind each rule are in `docs/dev/director-method.md` and
`docs/dev/director-log.md`.

- **Picking targets.** Pick targets from the bottom-up microbench ratio, not
  from a sampler's loudest leaf. Decompose a composite before attributing it,
  and estimate the win before keeping a change.
- **Old findings.** Re-probe an audit finding before briefing it; findings
  age.
- **Safety claims.** One green counter-probe does not falsify a safety claim;
  read the code path. A dangling citation may be a truncated write; check the
  log tail.
- **Instruments.** Calibrate every instrument against a known answer before
  believing it, a negative result most of all.
  - Time must scale with N.
  - Compare distributions, not single draws.
  - Trace a mutant's predicted red through to the observer.
- **Checks.** A check nobody has watched fail is not known to work; ship its
  falsifier. A certificate must force the transformation it names.
- **Repairs.**
  - A repair that closes the instances seen recurs as the class. The
    class-wide falsifier is rum against rum: the resumed path must equal the
    queued path.
  - A reading is not a verdict: a row settled only by argument is an open
    obligation.
- **Shape.** Green measurements do not cover shape. Grep the diff for
  `unreachable!`/`expect` on states the types allow, duplicated bodies, new
  warnings, and file sizes. List every caller of a function whose duty moved.
- **Worker evidence.**
  - A worker's gate is worth only what its sandbox can run.
  - Worker reports can be invented. Gate from code and reruns, and require
    copied verdict lines.
- **Noise.** If untouched controls move as much as the target, that is the
  noise floor. Say plainly when a gate was not rerun, and why.
- **Hot recursive frames.** Move locals, not control flow. Stack bytes per
  level are a first-class cost; put the check where the growth is.
- **Codex's recurring failure modes.** Optimizing sub-second hotspots;
  fixing at the wrong layer; masking a per-op cost; over-generalized scope.
- **Permission lists.** They get under-enumerated, so write a disclosure duty
  instead. An inventory rule must cover what new call edges make reachable.

## Current state (2026-11-10)

- **Nesting.** The security-hardening mainline (since 2026-09-02) holds the
  retention line (paused at E0) and the active suspension branch.
- **Branch queue:** G15-C2's rows (721 installed the definition-resume
  repair; continuation 8 found P1470/P1471 Lost; brief 722 accepted the
  fixed-point publication design; brief 723 reproduced it; its four
  STOPs are ruled and continuation 5 repairs two nested-admission call
  sites, then finishes stage B; C2 at 334 residuals + 2 Lost + 35
  `NoObservableTail`; prepare-arguments → continuation 9); then CM,
  `CHANGELOG.md` (before S10), G13 S10–S54, F1–F23, R; E2.
- **After the branch:** retention from E0 + Q3, then Q1/Q2/Q4/Q5, the binary snapshot format, audit items; `v0.3.0`.
- **Latest done:** LP08 closed (`b9b39763`). **Briefs 719-721:** P0248's
  fallback repaired (8/8 recipes, 36/36 bindings SAME); publication
  rides the enabled-only profiled queue join (caller 81,588/30 every
  arm; 147 → 0). **Continuations 5-8:** 25 rows Equal, ten
  `NoObservableTail` groups, three Lost rows; the repaired rows and
  P2604 settle Equal (470/470), and P1470/P1471 are new Lost rows.
  **Brief 722 accepted (design):** the copy repairs all 18 fixed-point
  differences with 84 controls exact and both omissions caught (20/14).
  **Brief 723 stage A:** 90/12/470 reproduced (74/74), lifecycle and
  foreign probes pass, the duplicate fault reads one visible fault,
  zero transfer and guest 73; run 4 re-anchored the formatted receipts,
  installed the five files, replayed the standings, and STOPed on a
  real nested admission at LP01 — continuation 5 repairs both sites.
  `BRIEF_WORKER=mst` (GLM-5.3) is the fallback.
- **Earlier lines** are in `director-log.md`; deferred items in `TODO.md`.

## Pointers

- `AGENTS.md`: the operational first stop.
- `TODO.md` (roadmap), `PROGRESS.md` (log), `CHANGELOG.md` (release
  notes), `docs/dev/codex/TEMPLATE.md` (brief format and inventories).
- `~/.claude/projects/-home-remi-dev-rum/memory/`: my cross-session
  memory; `MEMORY.md` is the index.
