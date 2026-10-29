# Director method — the full notes

Moved verbatim from `CLAUDE.md` on 2026-10-29, when `CLAUDE.md` was cut to
its 200-line short form (user ruling). `CLAUDE.md` keeps the rules; this file
keeps them with the incidents that produced them. Read it on demand.

## Role split (the working mode)

- **I own**: overall direction, the diagnosis, picking the next target,
  plan review, the authoritative gate, and the git commit. I do the
  *judgment*; the legwork I delegate.
- **Codex owns**: producing the plan (design-only briefs), then executing
  exactly one implementation brief at a time, self-verifying to green, and
  reporting with verbatim evidence. It does not commit, branch, or push — it
  leaves changes in the working tree for me to gate.
- **PLAN-FIRST IS THE DEFAULT (user ruling 2026-07-29; adapted from dun's
  2026-07-22 rule, with one extra layer).** For any non-trivial or
  cross-cutting task, do NOT hand-write a from-scratch implementation brief.
  The sequence is:
  1. **Design-only brief** — `Scope: NONE — no source change`. States the
     problem and current state; asks Codex for a concrete step-by-step plan
     (files/functions per step, how invariants stay intact, gate tests, a
     call-site inventory, risks and open questions), everything backed by
     `path:line` evidence. **Codex must also draft the follow-up
     implementation briefs themselves** — not just step descriptions, but
     usable brief drafts in `TEMPLATE.md` shape. Codex plans and writes no
     code.
  2. **I review the plan** — decide the open questions, correct what is
     wrong, reject what is over-scoped. This is the judgment step and it
     stays mine.
  3. **I write my own brief from Codex's draft** — the draft is raw
     material, not something I dispatch as-is. My version is what gets
     committed and dispatched, one step at a time, each gated.

  Rationale: it moves codebase spelunking and reach inventory onto Codex
  (cutting my token burn), and a Codex-authored plan enumerates the ripple
  up front, so a cross-cutting change is not silently under-scoped — the
  exact failure that cost this project five MAY-list misses on the matcher
  arc. It also puts two independent LLMs on the plan (Codex drafts, I
  review and rewrite), so a single model's blind spot is less likely to
  reach the code. Trivial mechanical jobs may still go straight to an
  implementation brief.
- The brief method lives in `docs/dev/codex/`: `TEMPLATE.md` is the format,
  `brief-NNN-<slug>.md` are the packages. Dispatch via the helper (commit the
  brief FIRST, as a separate step):

  ```
  scripts/dispatch-brief.sh NNN     # e.g. 124, or brief-124, or a slug fragment
  ```

  It resolves `docs/dev/codex/brief-NNN-*.md`, runs `cdx exec -s
  workspace-write` with stdin from `/dev/null`, and logs the full report to
  `.temp/logs/cdx_brief_NNN.log` (`DRY_RUN=1` prints the resolution without
  dispatching). **Launch it via the Bash tool's `run_in_background:true`** so
  the run is harness-tracked — do NOT hand-type `cdx exec … &`; the `&`-detach
  is not tracked. Sandbox default = `workspace-write` (user ruling
  2026-07-17, least privilege; writes confined to repo + tmp). Escalate ONLY
  for a brief that truly needs process attach (`sample` profiling — the
  original 2026-06-21 reason for full access), via
  `CDX_SANDBOX=danger-full-access`, telling the user. The brief's
  **Scope + Hard rules** remain the inner fence.
- Do not mutate the repo while a brief is in flight (my commits queue until
  the gate).

## The gate I run before crediting any brief

1. **Scope check**: `git status` / `git diff` touched only the brief's
   MAY-modify list.
2. **Tests**: `cargo fmt --check` and `cargo test` green. Default to
   `--workspace`; use a narrower `-p` gate ONLY when the change is provably
   crate-local (a green narrow gate is not a green tree).
3. **Claims**: I reproduce the evidence myself — test totals, sweeps, and for
   perf work the microbench A/B (`scripts/core-language-microbench.sh`,
   rum-vs-rum, per-label `timings.tsv`, same-tier neighbors checked). I trust
   the numbers I reproduce, not the report's interpretation.
4. Only then do I commit and dispatch the next brief.

## The standard for choosing between options (user ruling 2026-08-27)

**Pick the best and most correct option. Cost arguments do not count.** This
is a research/experimental project built with coding agents: there are no
downstream users, so no schedule pressure, and no human labour to budget. The
closest analogy is a portfolio piece — the point is to show what is possible.

Consequences for how I present and pick work:

- **Never score an option down for its size.** "398 call sites" is a
  description of the work, not an argument against it.
- **Do not propose stopgap-now / real-fix-later** unless the user asks to stop
  the bleeding. A temporary patch is itself debt. Go straight at the right fix.
- The arguments that still count: correctness, semantic loss, whether the fix
  is structurally unavoidable, whether it leaves an open-ended obligation,
  enforceability, measurability.
- Corollary: **a fix the compiler or a derive enforces always beats one that
  relies on people remembering**, even an order of magnitude more expensive.
- **The boundary**: this retires cost arguments, not design restraint. Scope
  is still set by the problem; over-generalized scope is still a stop. Every
  hard line below is untouched.

## Hard lines I enforce (do not let these drift)

- **Reply to the user in Chinese — always.** This project's working language
  is Chinese; the user cannot read any other language. Every user-facing
  message is in Chinese; only technical tokens (code identifiers,
  command/file names, WL/Rust terms) stay verbatim in English. Briefs,
  commits, and repo docs are drafted in English (the codebase is English),
  but the conversation is Chinese. Watch for ANY non-Chinese drift in long
  reports.
- **The testing policy is permanent** (see `AGENTS.md` §Testing Policy and
  `DISCLAIMER.md`): tests verify rum's own documented behavior plus
  rum-vs-rum perf A/B only. No black-box differential testing against other
  vendors' engines, no oracle harnesses, no compatibility matrices or
  reports, no committed third-party engine outputs — and I do not let a brief
  or a contributor reintroduce them.
- **No default-path cost for diagnostics or off-by-default features.**
  Counters and instrumentation go behind the env-gated cold-load `AtomicU8`
  pattern; a `Mutex`/allocation/branch paid on every call when the flag is
  off is a regression.
- **No new `unsafe` in `rum-core`/`rum-runtime`** (zero today).
- **A line, once opened, is finished (user ruling 2026-09-21).** Stopping a
  line half-built is worse than either finishing it or never starting: the
  ownership branch at 2 of 29 members converted left 27 types with a live
  hazard and step D blocked. When a branch is inserted into another line, it
  still runs to completion before the interrupted line resumes. Present the
  cost of finishing, not a menu that includes abandonment.
- **Do not tag (user ruling 2026-09-19).** `v0.2.0` onward is the
  **security-hardening line**; the ownership-graph work is a branch inserted
  from `serval`'s third-party review, not a replacement for it. Hardening
  resumes when this branch closes, and `v0.3.0` is considered only once
  hardening is *finished*. No tag lands mid-line, and no single step of a
  line is a release. An API break inside the line is fine — rum has no
  downstream consumers and the API is deliberately unlocked — but it is not a
  reason to cut a version.
- **Measure honestly**: profiling env OFF for any timing number; one variable
  per A/B; persist raw output to files before parsing; derive timeouts from
  data, not guesses. Elapsed probes use in-kernel `AbsoluteTiming` backed by
  Rust's monotonic `Instant`, never wall-clock differencing. A competing
  CPU-heavy workload invalidates timing judgments, and the benchmark runs in
  the ordinary workspace sandbox.

## Diagnosis method (durable lessons)

- Pick targets from the bottom-up microbench ratio, not from a sampler's
  loudest leaf; decompose a composite probe into factors before attributing
  its cost; estimate the expected win from the profile before keeping a
  change.
- **Re-probe an audit finding before briefing a fix for it — an earlier batch
  may have already changed it.** Unit 8's audit recorded a "silent false
  round-trip" for `DownValues[g] = DownValues[f]`; by the time I got to that
  batch, the `Protected` sweep (two batches earlier) had already turned it into
  a loud failure. The premise had shifted from "fix a safety hole" to "build a
  WL feature" — a different decision, which belonged to the user, not to me
  silently. Findings age; verify, then brief.
- Codex's recurring failure modes to watch in review: optimizing sub-second
  hotspots while the parent workload is elsewhere; fixing at the wrong layer;
  masking a symptom (reducing an operation count to hide a per-op slowness)
  instead of fixing the per-op cost; **over-generalized scope** — a correct
  root-cause observation attributed too widely (brief-208 correctly saw that
  `store_held_update` skips a protection recheck, then attributed the resulting
  bypass to all 11 compound assignments when only `ApplyTo` — whose update runs
  user code in the store window — can actually trigger it).
- **A dangling evidence citation may be a truncated write, not a fabrication —
  and my own falsifying probe may be too weak to trust.** brief-208 cited
  "Evidence E15/E16" that were absent from the file; the run was truncated at
  the end (an upstream `cybersecurity risk` content filter fired in the dispatch
  log's tail, so the protection-bypass section was likely cut off mid-write). I
  gated it "false" from ONE counter-probe (`AddTo` with a protecting downvalue)
  that returned `wrsym` — but that probe could not reach the code window (the
  second arg is not held, so protection fires before the guard). Reading the
  code showed the window is real and `ApplyTo` triggers it (`x = 101` on a
  `{Protected}` symbol, no message). Lesson: **for a diagnostic-level safety
  claim, a single green counter-probe does NOT falsify it — read the code path
  and confirm the mechanism.** A weak probe's NOT-REPRODUCED is a false
  negative. When a report cites evidence that isn't in the file, suspect
  truncation (check the dispatch log tail for a filter/quota cutoff) before
  suspecting invention, and settle the claim from the source, not from one
  probe.
- **Calibrate the instrument before believing it — three ways it lied to me in
  one session (unit 8):**
  - *A timing harness that measured nothing.* An A/B over `Do[…, {300000}]`
    showed both arms identical — because the big `N` silently hit the iteration
    ceiling and bailed out, so I was timing process startup. **The tell: `N=1e6`
    came back FASTER than `N=1e5`.** Always check that time SCALES with `N`
    before trusting a single timing number.
  - *A parser that under-reported.* `docs/reference/builtins.md` backticks each
    attribute separately (`` `Listable`, `Protected` ``); a regex capturing only
    the first backticked token said "527 unprotected" when the truth was 483.
    Cross-check any script-derived count against a few live probes.
  - *A hybrid test session.* `session_with_test_control_builtins()` resolves SOME
    symbols to test-registered builtins and others to the real catalog (a test
    registration with EMPTY attributes falls through to the catalog). My
    "obvious" uniform fix to four attribute assertions was wrong for two of them.
    Probe ground truth per symbol; never assume a uniform rule across a test
    harness.
- **Noise discipline for an A/B**: if the UNAFFECTED control labels move as much
  as (or more than) the affected one, that IS the noise floor — report no
  regression, and do not "fix" noise. Say plainly when you did NOT re-run a
  gate and why (e.g. no eval-hot-path code changed and the panel never calls the
  changed symbol) rather than performing the ceremony.
- Profiling toolkit: `scripts/sample-self-time.py`,
  `scripts/sample-attribute-callers.py`, and the env-gated runtime counters;
  self-time + caller attribution + counters beat call trees for the recursive
  evaluator.
