# CLAUDE.md — director's notes for `rum`

This file is **mine** (Claude). `AGENTS.md` is the operational guide that any
coding agent reads. This file is how *I* operate: direction, diagnosis,
delegation, and the authoritative gate.

**Keep it under 200 lines** (user ruling 2026-10-29). It is read on every call.
Anything longer belongs in a document read on demand:

- `docs/dev/director-method.md`: the full method, with the incident behind
  each rule (the long form of the sections below).
- `docs/dev/director-log.md`: the direction log, line by line, with each
  line's lessons. It is history, not the roadmap.
- Present state: `TODO.md` (authoritative),
  `docs/dev/security-hardening-progress.md` (the user's Chinese snapshot),
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
  `-j8` with `RUST_TEST_THREADS=1`, and test runs use `--no-fail-fast`.
  Timing, perf and callgrind runs are serial and run alone.

## Context economy (user ruling 2026-10-29)

My usage is calls times context, and my time-window quota is the bottleneck.
So:

- Worker reports are `rum-report/1` JSON (`scripts/report.py`, `TEMPLATE.md`).
  I read `summary`, pull a part with `show`, gate scope with `check
  --against-git`, and `render` the record into the brief. Diffs: in full.
- Procedures become scripts, inputs in and one table out: the gate is
  `director-gate.py run NNN [--perf resume,cli]`; long output in `.temp/`.
- Subagents are never switched to a smaller model or a lower effort to save
  tokens (user ruling): recovering from their errors costs more.
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
- **Codex's recurring failure modes.**
  - optimizing sub-second hotspots;
  - fixing at the wrong layer;
  - masking a per-op cost;
  - over-generalized scope.
- **Permission lists.** They get under-enumerated, so write a disclosure duty
  instead. An inventory rule must cover what new call edges make reachable.

## Current state (2026-10-31)

- **Nesting.** The security-hardening mainline (since 2026-09-02) contains
  the retention line (paused at E0), which contains the suspension
  continuation branch (active).
- **Branch queue, in order:**
  1. the workspace thread-count measurement (4/8/16/32) for the user's
     ruling — `rum-stdlib` at 8 threads was parked by the user on
     2026-10-29, so ask first. Test isolation is done (briefs 634–636);
  2. LP03–LP08;
  3. G15-C2's remaining 353 rows, then CM;
  4. `CHANGELOG.md` (user ruling: before S10);
  5. G13 S10–S54, F1–F23, R;
  6. E2.
- **After the branch:** the retention line from E0 with Q3 designed beside
  it, then Q1, Q2, Q4, Q5 (user ruling 2026-10-29), the binary snapshot
  format, the remaining audit items and the threat-model critiques. Only then
  `v0.3.0`.
- **Latest done:**
  - brief-636, the shared parse: one immutable `Source`, `Interfaces`,
    population and typed `Discovery` per process; test-isolation item 1 is
    done (briefs 634, 635, 636);
  - brief-635 amendment 2, the reset admits for its whole body and the
    evaluation-cache `ACTIVE` publishes;
  - brief-635 amendment 1, the lexical write gate: `5184797c`;
  - brief-635 stage 1, the epoch test lease: `b199d63c`;
  - brief-634, the test-artifact constructors: `467ff97a`;
  - brief-630, abandonment cleanup errors: `854cfa4a`, `1c115117`, `ba1e1e52`.
- **Workers.** kimi only (2026-10-30: cdx has no quota, grk no balance).
- **Earlier lines** are recorded in `director-log.md`; deferred items in
  `TODO.md` (full polynomial factorization stays deferred by user decision).

## Pointers

- `AGENTS.md`: the operational first stop (snapshot, commands, tests).
- `TODO.md`: the active roadmap. `PROGRESS.md`: the append-only log.
  `CHANGELOG.md`: release-facing changes.
- `docs/dev/codex/TEMPLATE.md`: the brief format, with the standing
  inventories every source stage replays.
- `~/.claude/projects/-home-remi-dev-rum/memory/`: my cross-session memory;
  `MEMORY.md` is its index.
