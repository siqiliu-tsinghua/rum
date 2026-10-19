# CLAUDE.md — director's notes for `rum`

This file is **mine** (Claude). `AGENTS.md` is the operational first-stop guide
that any coding agent — including Codex — reads when executing work. This file
is about how *I* operate on this project: direction, diagnosis, delegation, and
the authoritative gate. Keep it short and high-signal; link to canonical docs
rather than duplicating them.

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

## Current direction (read these for state)

- **ACTIVE BRANCH: SUSPENSION CONTINUATIONS (inserted 2026-09-28 by user
  ruling, briefs 548-550; runs to completion before the retention line resumes
  at E0).** Handoff is `TODO.md`'s top section. A builtin that suspends for
  input mid-work must resume *into* that work; brief-548 found 86 sites in
  eight groups that resume past it, plus a loader that swallows the suspension
  while leaking its owner. User ruling: guest loading becomes resumable; the
  host batch APIs stay non-resumable as documented. Done (2026-09-28 →
  10-06): guest loading (550), batch boundary (551), all front ends (553-558),
  and **all eight groups G1-G8 (559-582; lost 86 → 0 by the frozen census;
  all 195 brief-548 probes agree on `033711f9`)**. G2 (lazy machine), G3
  (schema validator) and G4 (rule left-hand sides, held patterns) were also
  guest-reachable native-stack aborts, fixed together as flat work stacks
  with structural depth checks; G5 is a typed read-and-complete option
  protocol (a read without its continuation does not compile); each group's
  ordinary-path cost was then tuned with one-codegen-unit instruction gates
  and the residual recorded. Since then (583-597): the re-census found G9 and
  G10; E1 built the catalog-wide oracle (586) and drained it family by family,
  finding G11 (output a suspended capture owns, 593-595) and G12 (597); 597's
  sweep found **G13 — typed continuation state crosses frames in-band as
  `Expr` under `System`Internal*` heads guest source can write**, decoded
  leniently or by panic. 598-599 closed E1: all 438 declared entries have an
  executed descriptor in both builds and the owed ledger is gone. G13's design
  took four rounds (600-605: two by GLM, each rejected by a Codex review, then
  Codex designing against its own review in a forked session). Round 4 is
  accepted except its atomic 528-file conversion, replaced by type-then-flip
  (a per-type transport mode, so typing steps stay `Value`-compatible and
  each lands green). Round 5 (606) partitioned it into 78 gated steps — S1
  dormant API, S2 driver, S3-S54 family typing, F1-F23 one flip per legacy
  type, R removal. S1 (607, the dormant API), S2 (608, the compatible
  driver, `cb09bfde`), S3 (609, evaluation and dispatch, nine dispatches),
  S4 (610, argument/rewrite/traversal resolution, the first Kimi K3 brief)
  and S5 (611, definition application, four dispatches) are done. G14 (612 design, 613
  implementation, four dispatches) is done: resume never unwound the
  evaluation stack it restored, a defect as old as resumable input
  (`b51dd794`, day 4). Every queue cell now carries its insertion context,
  and an inline return event runs the skipped completion checks. S6 (614,
  guards and binding completion, one dispatch), S7 (615, Part selectors
  and held assignment, the first typed stdlib frames, one dispatch) and S8
  (616, ReplaceAll and the traversal collectors; the worker's run was cut by
  Codex's usage limit at its timing stage and I finished the gate) are
  done, and S9 (617, control and cleanup) is done. agy and grk were both cut
  by their quotas, and I finished the gate by user ruling. **Now: G15,
  inserted before S10 by my S10 pre-probe** (`TODO.md`'s G15 entry): a
  `Failure` property call that pauses resumes past the Function stage's
  tail, so a `Return` escapes the body; its design (618) found a resumed
  LHS losing its cycle recovery too. Both were rows brief-548 left
  `Unsettled`. brief-583's re-census then "resolved" all 597 of its
  `Unsettled` rows by reading, with 14 probes, and called these two
  TailCorrect ("the fixed point evaluates the final Expr") and FrameCarries
  ("including own-symbol cycle handling"). Both claims were false, and the
  oracle compared top-level values, which the evaluator's fixed point
  repairs. G15-A (619, `afdcd02f`) and G15-B (620, `43a26c72`) are done. The
  G15-C design (621) is accepted: 2 186 rows, a 25-policy tail enum, and an
  80k-probe corpus. A row is settled only when its tuples are SAME *and* its
  own tail-skip mutant is caught. 198 rows are settled; the other 1 988 are
  `OwnedObligation`s, 1 024 of them for G15-C2 and 964 for later G13 steps.
  It found G15-C1, a ReplaceAll finish lost on resume and visible only in
  the ordered trace, now done (622, `b4f81ebe`). Next: G15-C2 (623, settle
  its 1 024), then the manifest, then CHANGELOG, then S10, one step per
  brief. *Unsettled is not a
  verdict, and neither is a verdict by reading*: a row settled only by an
  argument is an open obligation, and a closed line that carries one is not
  closed. G15-B's lesson on stack bars: bound the chain of frames live at
  once, including the helpers the entry calls, never one function's
  prologue (I set the wrong bar twice);
  then E2 (a
  may-suspend result type with no `Try`). From S5 the per-step stack bar is
  the named closure environment attributed per slot: S4 showed that an
  every-pair at-or-below-base bar at 16-B granularity is not reachable by
  any macro expansion. G14's timing lesson: an identical-instruction cycle
  residual is settled causally, by rebuilding both arms with every function
  64-byte aligned (`-C llvm-args=-align-all-functions=6`). A timing probe
  runs the benchmark program itself, whole, with `rumMicroReps` raised (S8):
  my S5 key-read probe hit a fast path 250x cheaper than the label and
  cleared it by luck, and a cut can drop a label's prelude. S5's lesson: *a shape that needs an `unreachable!`
  arm for a state the types allow is pointing at the shape nobody tried* —
  its cold helpers took the producer's whole `Result`; the design's own
  shape (the ZST witness and the request cross the call) cut the stack
  residual from 2 320 B to near-base and deleted all fourteen impossible
  arms. S9's lessons: *calibrate a worker's instrument in the worker's environment* — grk's copy count matched libc names in a sandbox where libc had no symbols, read a constant, and was reported as a pass; and *a copy fix is not done until the stack shapes are rerun* — the arms that removed the extra memcpy grew `cleanup-body` by ~300 B per level in the inlined dispatcher. S3's last defect lived in S1:
  dormant code is unmeasured code. The typed finalizer reboxed itself on
  every finish from day one, and only the first step that executed one on a
  measured path could see it.
  Runs are serial by user ruling. Memory `suspension-continuation-branch.md`.

  **Four things from it are method, not history.**

  - *A repair that closes the instances seen recurs as the class.* brief-530
    fixed one lost continuation on the option path; the sweep found 86 more of
    the same mechanism. The class-wide falsifier is rum against rum: the
    resumed path must equal the queued-input path in value and side effects,
    with the suspension asserted, suspending mid-loop and twice.
  - *A swallowed suspension must retire its owner.* Converting
    `InputRequired` into a value without abandoning the pending owner leaves a
    session that looks idle and refuses every request. A cleared prompt is not
    a retired owner.
  - *A limit probe's control must fire before its treatment is read.* Three of
    the director's budget probes measured another limit first — the element
    limit, the iteration limit, a loop body that never polls — before the
    fourth showed `Get` renewing the request deadline per form.
  - *Probe the structure a design keeps, not only the defect it fixes.*
    brief-561 designed G2 on the recursive `PullStepper`; one depth probe
    showed that structure already aborts the process at ~2.2k stages on the
    ruled 2 MiB stack, and the design would have added four more walks of that
    shape. The fix was a representation in which deep recursion cannot be
    written (a flat stage vector under a cursor), not a guard each walk must
    remember — `Clone`, tracing and `Drop` cannot even return the guard's error.

- **THE RETENTION LINE (2026-09-22 → 28, briefs 524-547) — PAUSED at E0 for
  the branch above.** The design and its theoretical basis are
  `docs/dev/retention-closure-design.md`; the 2026-09-28 `PROGRESS.md`
  entries have the narrative; memory `retention-closure-line.md`. The option
  region's two bodies are the first certified production coverage
  (`ee81758a`). brief-547 settled the map: of 643 declarations, N 195, P 291,
  OutOfClass 56, Unsettled 101, in 13 regions by conversion mechanism needing
  seven new protocol surfaces. **Resume order: E0 (the four escaping consumer
  copies) → H (a lint exporter for named production bodies) → settle the 101
  with H → K/A → each surface before its users → regions by leverage**, each
  with the option-region pattern. Grouping by named helper hid this — after
  the option path the leverage looked flat. The snapshot format is queued
  behind the line.

  **Four things from this line are method, not history.**

  - *Traversal was the wrong shape, and the theory says why.* A region that
    runs guest code has the evaluator in its call closure. Prove each body
    locally against contracts at its boundaries; the second condition of
    coverage is a mark lookup, never a walk. Do not revive A3-A6.
  - *Define the contract before building what relies on it*, and classify every
    rejection before drafting — a single misclassification cost brief-540 two
    dispatches.
  - *Stop-loss counts compile errors and failing tests as progress.* It is in
    `TEMPLATE.md` now; brief-544 lost two dispatches when it was missing.
  - *Instruction slopes decide, timing does not* — on this host timing has
    reported +145% where instructions showed +0.04%. Pin with `taskset -c 2`,
    take the `cpu_core` row, feed `rum-cli` `< /dev/null`, and keep all scratch
    in the git-ignored `.temp/`.

- **THE PREVIOUS LINE IS CLOSED (2026-09-08/21, briefs 459-498, through
  `c07cb414`): association depth, the ownership graph it exposed, and that
  branch's own leftovers.** `TODO.md` is authoritative; the 2026-09-21
  `PROGRESS.md` entry has the narrative; memories
  `stability-walk-and-rebuild-class.md` and `ownership-graph-line.md`.
  **NEXT: the security-hardening mainline resumes** — the 2026-09-02 audit's
  open findings, headed by the process-global symbol interner that the memory
  ceiling cannot see, plus the two items that work opened and never briefed.

  The class-wide **fail-open** direction is closed for associations: a guard
  substrate (`runtime/native_stack_guard.rs`, two `#[doc(hidden)]` `kernel_api`
  items, two `compile_fail` doctests), two enforcement layers (a 44-row per-site
  registry and a recursion inventory where an `UnguardedNative` row owing
  nothing fails the test), and conversions to a depth-16 recursive front plus a
  cold iterative heap tail. `basic.rs` is fully closed.

  **Three rules this produced.** A conversion may *redo* its traversal only when
  the walker takes no `Session` and therefore cannot run user code. Its
  certificate must be an in-module forced-front falsifier on a **branched**
  fixture — private `*_with_front_depth`, front `0` → `Exhausted`, front `16` →
  `Complete`, `tail == front == exact expected` — because nothing weaker proves
  the tail ran. And **probe a certificate's premise before writing it into a
  brief**: two briefs demanded certificates that fail identically on the
  pre-conversion build.

  **The ownership branch is closed.** The byte guard constrains the call stack;
  the host *destructor* walks an ownership graph nothing modelled, which is why
  both real drop hazards were found by accident. It ends with: all **29**
  accumulator-chain members draining iteratively behind one `#[repr(transparent)]`
  wrapper, kept closed by a discovery-derived participation gate; a disposition
  ledger over all **50** `UnknownHostPayload` rows (3 `Declared`, 3 `Unbounded`
  owed to the `Expr` debt with external implementations stated unresolved, 44
  `PublicInput`), whose attribution is *verified* rather than written; and the
  scanner's own root set corrected — `rum-syntax` was unscanned while the kernel
  owned its recursive AST.

  **Step D was cancelled on evidence, and that is the line's best judgement.**
  Declaring `SuspensionFrame`/`PureFrameFinalizer` bounded would have asked ~598
  implementors to assert what the project records as false: `Expr`'s destruction
  is unbounded standing debt and two thirds of the frames own one. **A
  declaration that is not true is not made acceptable by being cheap.**

  **Four defects it found on the way**, none of them what it was looking for: a
  resumed `Table` panicking the kernel (`Arc::try_unwrap` demanding uniqueness a
  suspension frame legitimately breaks); a **source-reachable** fail-open where a
  call-head chain evades the parser's 128 counters and overflowed lowering at
  1.2-1.5x the contract height; a **cubic** parse dump (`{:#?}` wraps the writer
  per nesting level); and the standing-debt entry's claim that source-created
  trees are depth-guarded, which was false for that shape and is now corrected.

  **E2 was examined and declined.** Of 7 068 `UnknownConstruct` rows only 70 can
  hide an owning edge, and those are 29 `std`, 6 `num_bigint`, 6 `rum_syntax`
  and flat leaves. The maintenance tax is real; the safety yield is not. **An
  axis declaring an unknown "cannot introduce an owning edge" silences it —
  maximal conservatism is loud and useless but never lies.**

  **Dispatcher**: never pipe `scripts/dispatch-brief.sh` — `pipefail` plus a
  closed pipe kills it before it dispatches and the result mimics a silent
  worker termination. `cgl` is now proven for ordinary conversions too.

- **The traversal-depth line's second half is CLOSED (2026-09-04, briefs
  436-446, `0f679699` through `cc8ba769`).** `TODO.md` is authoritative; the two
  2026-09-04 `PROGRESS.md` entries have the narrative. Three defects, each found
  by re-probing the one before it: the native stack guard's default was 6 MiB
  against the ruled 2 MiB embedder stack, so on the ruled configuration it could
  never fire and `SECURITY.md` published the opposite promise; a traversal that
  rebuilds a deep result was killed not by its own walker but by the shared
  evaluation-stability walk downstream of it; and that walk carried
  off-by-default profiling state in its recursive frame. It now costs 192 bytes
  per level instead of 811, reaches physical height 4,097 with 30% margin, and
  fails closed above it.

  **Four things from this are method, not history.**

  - *A certificate must force the transformation it names.* Twelve release
    certificates were green and **none exercised a rebuild** — two used a level
    spec selecting only the root, one replaced a symbol absent from the value,
    and every one asserted a summary (`Depth`, `Length`) that is correct when
    the transform never ran. The rule now: a unique deepest target whose change
    must propagate, an assertion on the exact result, and a selector pinned
    separately on a shallow fixture. That shallow companion is also the
    falsifier when the deep case aborts the test binary.
  - *On the hottest function, move locals, not control flow.* Two functionally
    complete iterative conversions — differential tests, cache-insertion-order
    traces, all invariants — cost 2-7% on the evaluator's hottest label
    families, because a depth parameter and an `Option` return on the
    most-consulted function is not free. Outlining the cold branches instead
    cost nothing and cut the frame 3.17x. Both rejections are kept as stashes.
  - *Stack bytes per level are a first-class cost, and off-by-default state
    living in a hot recursive frame is a hard-line violation even at zero CPU
    cost.* The profiling key returned `None` immediately when disabled; its
    64 bytes of frame were what the depth contract was actually made of.
  - *Put the check where the growth is.* The same strided poll cost
    +7.99%/+7.98% on an atom-heavy label at the walk's entry and nothing at all
    on the call-cache miss path, which is the only place that recurses.

  Measurement notes worth reusing: `/proc` VmStk high-water at four heights
  gives a stack slope with no abort at all, and is what to use when gdb ptrace
  is denied. Two of my own instruments lied again — a parser reading the wrong
  filename that printed a table of zeroes, and a bisection whose binaries this
  host had wiped from `/tmp` mid-session, so every run failed and it converged
  on the low bound.

- **Two lines are PART BUILT and neither is closed (2026-09-03, briefs
  412-427).** `TODO.md` is authoritative for both; the 2026-09-03 `PROGRESS.md`
  entry has the narrative. The memory-accounting line made the session walk
  complete, made every builtin declare how it can grow, and made two of those
  contracts falsifiable. The traversal-depth line bounded four recursive
  descents that ended the process and converted ten traversals, but the
  construction-side bound is **not installed**: caching the physical height
  that makes admission O(1) costs 30% on list rebuilds, the stop-loss fired,
  and the spike is in `stash@{0}`.

  **Four things from these are method, not history.**

  - *An instrument that has never been seen to fail is not known to work.* Both
    growth falsifiers ship with checked-in `#[should_panic]` proofs and I
    verified each by misdeclaring a real catalog builtin. This is now a
    required section in any brief that builds a check.
  - *Seven of my own probes measured something other than what they claimed*,
    every one the same shape — a fast path, a level spec that barely descends,
    a `grep -rl | head -1` that returned a reference rather than a definition,
    two inventories summed without checking their overlap. Calibrate against a
    known answer before believing a result, including a negative one.
  - *Codex's content filter is not a wording problem.* It killed brief-422
    twice, the second time with the security framing already removed, and
    brief-424 twice at 120k tokens on the worker's own generated probe code.
    **The judgement is whether the brief needs an executable reproduction**, not
    which words it uses; if it does, either run it myself or send it to GLM.
  - *A reclassification into an unenforced contract is a deferral wearing a
    repair's clothes.* Five of the eleven declarations brief-417 moved landed in
    `Preflight` while it had no falsifier, and brief-419 caught all five the day
    it gained one. Briefs since forbid reclassification outright.

- **The suspension and cancellation line is COMPLETE (2026-09-02, briefs
  397-411, `8df8f144` through `4c4dbfab`).** It opened with the first
  release-profile full-test run on the Linux host, which produced two
  `rum-jupyter` failures the debug gate never showed, and closed three kernel
  defects: cancellation erased by its own request prologue, abandoning a
  suspension leaking every scope its frames owned, and overlapping public
  requests silently replacing a pending continuation (now rejected, by user
  ruling; supersede stays unimplemented rather than being an accident of
  consuming queued input). Detail in `TODO.md` and the 2026-09-02
  `PROGRESS.md` entry.

  **Three things from it are method, not history.**

  - *An instrument reported nine skips as nine passes.* A native ZeroMQ test
    that cannot bind loopback printed to stderr, early-returned, and libtest
    said `ok`. That had already cost brief-384 a false 0-of-30. It is now a red
    test unless `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` opts in. **Before trusting any
    green from a worker, ask what its environment can actually execute.**
  - *A worker's gate is worth exactly what it can run.* brief-409 was green in
    its sandbox and broke all nine native tests on this host, masked by that
    same skip path. The one-line fix mattered less than brief-410's socket-free
    harness coverage, which gave the worker a gate it could run — and paid off
    immediately when family B needed one.
  - *Refuting a repair beats performing one.* brief-401 killed brief-400's
    mechanism before it was built: driving saved frames with `Err(Cancelled)`
    is not an unwind, because ordinary frames consume errors and run work.
    brief-400 is superseded, not resumed; `stash@{0}` is kept as reference.

  My own recurring failure this line was **trusting a predicate without
  calibrating it against a known answer** — three bad probes in one session,
  and a MAY list short twice on module roots. A module root belongs in the list
  whenever a type it exports changes.

- **Post-`v0.1.0` safe-Rust algorithmic audit is the active maintenance
  line.** Batches 1-7 are closed (through `29c5e174`). `TODO.md` is
  authoritative for what remains; `PROGRESS.md` has the per-batch narrative.
  Do not reconstruct status from the historical bullets below.
- **Batch 7 (briefs 342-363, 2026-08-26/27) was six performance items and
  produced two correctness lines instead**, both found by probes written to
  measure something else — which is the argument for the re-probe rule, not
  an accident. `Do[f[i] = i, {i,1,3}]` had never installed more than one raw
  definition, and right-hand-side `Condition` guards (`f[x_] := rhs /; test`)
  were not implemented in any of the eight candidate-selecting lanes. Both are
  fixed; the guard line took eight briefs and one repair of my own gate.
- **The suspension-frame roots line is COMPLETE (2026-08-28, briefs 364-375,
  `d13aa27f` through `4e4c229f`).** The gensym defect batch 7 opened is closed,
  and the re-probe found four defects rather than the two recorded — including
  a `Block` frame's bare `Symbol` that leaked in the opposite direction, which
  is why the visitor treats `Symbol` as a root. The repair is enforced, not
  maintained: `SuspensionFrame: FrameRootTrace` makes a missing traversal a
  compile error, and an invariant test records every handwritten traversal.
  **Two claims here were overstated and the brief-389 review corrected them.**
  That list is an inventory of handwritten impls, now 59 entries, most of which
  deliberately walk their contents (`Expr`, `SharedArgs`, `Option<T>`, `Vec<T>`,
  maps, tuples); only the no-op leaves need to be structurally root-free. And
  the queue is the single insertion route for ordinary code, not literally —
  `insert_suspension_frame` is public, the resume driver pushes directly, and
  the invariant exempts `suspension/core.rs` and test directories. No current
  path bypasses `SuspensionFrame`, so the traversal guarantee holds; the guard's
  reach is narrower than "one route" suggests. It also removed a 63.8 KiB per-call
  ordinary-path capture. Detail in `TODO.md`, the 2026-08-28 `PROGRESS.md`
  entry, and memory `suspension-frame-roots-line.md`.
- **The assignment-target normalization line is COMPLETE (2026-08-28/31,
  briefs 376-388).** Eight defects across two repair families, all from one
  bidirectional invariant: the identity a definition is stored under must come
  from the same attribute-aware preparation as the identity a lookup
  constructs. Four failed to prepare, two prepared a position that had to stay
  raw, one needed `Flat`/`Orderless` match normalization. Storage and removal
  now travel as tokens whose constructors are private, so a raw lhs from
  source-language evaluation does not compile. Then three long-standing items
  closed in one day: the `rum-jupyter` ZeroMQ flake (a test asserting a timing
  property where the poller promises a priority one — reproduces at ~70% in
  release, near-invisible in the debug gate), syntax error locations (spans were
  form-local and rendered against the whole file), and comparison-chain
  associativity (parentheses were transparent; `Inequality` introduced by user
  ruling). The call-stability-proof flake is also closed: all three sightings
  were one Bloom collision driven by a process-global interner id, and the
  repair makes the collision structurally impossible rather than rare.
- **A four-part consistency review of briefs 364-388 is COMPLETE (2026-08-31,
  briefs 389-396).** Eight findings, three high, all repaired. Every repair
  replaced a convention with a structure — a list of protected pattern wrappers
  became a registry shared with the matcher, assertions resting on interner
  order moved to a lane-disjoint construction, a public installer that trusted
  its caller now demands a typed lhs, and a wrapper claiming two constructors
  left the continuation protocol that offered a third. **Three lessons outlast
  the findings**: two of them were corrections to my own briefs; one was a gate
  miss where brief-388 closed the two instances that had been seen rather than
  the class, because diagnosis and repair both looked only at recorded
  sightings; and **a boundary can hold inside the repository and evaporate at
  the crate edge** — the test policing it scanned in-repository callers, and a
  source scan cannot establish a property about code that does not exist yet.
  The gate that settled those two compiled an external crate and required the
  compiler errors. Reviews were dispatched in four independent parts so an
  interrupted run would lose at most one, with findings written before any
  verification command, and none tripped the upstream content filter.
- **`TODO.md` currently has no known-and-worth-fixing item.** What remains is
  deliberate conservatism (`ReplacePart` sequencing, product propagation),
  something with no practical impact (trailing-operator re-parsing), and
  association keyed reads, which are sublinear by a standing cache-admission
  decision that needs its memory tradeoff stated before it can be reopened.
- **The gate lesson from batch 7, now in memory as
  `gate-assert-after-invalidation`:** six green gates missed a guard that
  stopped being enforced after any `SetAttributes`, because every gate — and
  the whole-line regression test — asserted the guard *before* the trigger and
  never after one. For anything that caches or compiles state derived from a
  definition, the test must act after the invalidation. Also: two runs do not
  estimate a noise floor; that produced a false stop-loss whose real
  same-binary spread was 12.6-26.7%.
- Full polynomial factorization is separately deferred by user decision. The
  current linear-factor/monomial subset stays documented as such until a
  coherent finite-field polynomial layer is designed.
- The July statement that wide symbolic `Plus`/`Times` grouping remained an
  accepted O(n^2) tradeoff is superseded by `48838b3`'s adaptive group index.
  Current docs and TODO have been corrected.

## Historical direction through 2026-07-30

The remainder of this section is a point-in-time director log. It explains
decisions and measurement lessons but is not the current roadmap; use
`TODO.md`, `CHANGELOG.md`, and the latest `PROGRESS.md` entry for present state.

- **INCREMENTAL CONTAINER UPDATE + PART ASSIGNMENT — BOTH COMPLETE
  (2026-07-29/30, briefs 291-300). Maintenance-era work, triggered by a
  real question, not by a hunt for things to optimize.** Origin: a rum-hol
  feasibility question ("is log replay expensive?") led to measuring
  `assoc[k] = v`, which was O(n^3). Two design-only briefs (291, 297) then
  drove two implementation arcs, plan-first throughout.
  - **plan-291 (briefs 292-296)**: microbench labels → COW + cache-proof
    primitives → flat `AppendTo` fast path → Part spine → Association direct
    canonical-Rule mutation. `AppendTo` O(n^2)→O(n) (6.4x), Part spine 13.2x,
    Association 25.1→0.43 ms (**57.9x**), `shared_expr_alloc` 20322→222.
    Step 6 (bounded key index) deliberately NOT done: the linear scan it
    targets is not the dominant term at these sizes, and it would reintroduce
    exactly the extra-strong-holder shape this arc spent three steps removing.
  - **plan-297 (briefs 298-300)**: the diagnosis brief found something far
    bigger than the perf question that prompted it — **`lst[[i]] = v` never
    worked at all**, silently leaving the list unchanged and emitting a false
    `Set::wrsym: Symbol Part is Protected`. A year of work missed it because
    the gap is oddly shaped: compound Part updates worked, Association Part
    assignment worked, Part reads worked, `ReplacePart` worked; only plain
    `Set` into a List position was absent, and no test covered it. Fixed as a
    feature first (`88625df`), then measured (`885ec0f`), then made
    width-independent (`60c4ab4`, length quotient 2.87→0.995).
  - **The measurement lessons are the durable part** — this suite lies in
    four distinct ways, each caught the hard way:
    1. **Counters beat timings.** Allocation counts are load-immune; when a
       video call invalidated a whole timing pass, every "did the mechanism
       fire" conclusion still stood. Brief rule since 294: report counters
       first, and if they don't move, do not quote a time.
    2. **Per-run whole-run offset.** Every run carries a scale factor moving
       ALL labels together by a few percent. Two runs of the IDENTICAL script
       read −4.07% with 7/38 up. **A lopsided direction split is not
       evidence.** Alternating order does not cancel it.
    3. **Never normalize against the suite median in an optimization lane.**
       It drifts cumulatively: the small-Association sentinel read +21.9%
       across steps 1/3/4/5 purely because the optimized labels dragged the
       median down. Normalize against *named untouched labels*, or use
       within-run quotients (immune by construction).
    4. **Cross-session comparison is unreliable even with paired ratios** —
       the same comparison read −4.96% across sessions and −0.16% within one.
       Run base and candidate in one window.
  - **My own worst error of the arc: a probe that measured a failing path.**
    The `lst[[i]] = v` "performance" numbers were the cost of
    rebuild-then-reject. The probe *contained* a calibration point
    (`Print[qL[[200]]]`, printing 0 instead of 200) that I never read — I
    grepped only the counter lines. And the scaling check this lane relies on
    did NOT catch it: the failing path is genuinely O(m), so the number scaled
    correctly while measuring the wrong thing. **Every perf probe needs an
    assertion that the work actually happened**; brief-299 makes it a required
    section.
  - Remaining known candidate, NOT scheduled: `PrependTo` at 2.81x (head
    insertion moves all elements even when exclusive, so it has a real floor).
  - Workflow: **plan-first proved itself immediately.** Codex's design brief
    refuted three of my five findings on 291 (the call-stability cache as the
    hidden third owner; "only ApplyTo runs user code" false for all eleven
    compound assignments; Association being O(n^3) not O(n^2)), all confirmed
    at the gate — and on 297 it found the feature gap I had mis-diagnosed.
- **PUBLICATION-PREP LINE — COMPLETE (2026-07-21). rum is PUBLISHED at
  https://github.com/siqiliu-tsinghua/rum** (main `b854d3e` with full
  unrewritten history + annotated tag `v0.1.0`; remote HEAD verified equal to
  local). All four buckets closed. GitHub topics set (12, wolfram/mathematica
  deliberately avoided — user); `keywords` aligned. **PROJECT NOW PAUSED
  post-publication (user 2026-07-21): this repo enters maintenance; downstream
  capabilities (`rum-fs`/os/net/db, the Python binding) open in SEPARATE
  workspaces/repos — NOT this folder — and the next active work is the `dun`
  editor, its own project.** **User ruled divergence is now SUFFICIENT
  and reversed the 2026-07-06 publication-deferral (see the updated bullet
  below).** Two rulings
  set the shape: (1) release form = **source-available GitHub only**
  (no crates.io ⇒ no per-crate `publish`/metadata/semver burden, just a
  `repository` field); (2) the public tree is **fully transparent** —
  `CLAUDE.md`, `docs/dev/` briefs+design docs, `PROGRESS.md`, the Codex
  dispatch tooling all stay public (cleanup = accuracy pass, not
  removal). Four buckets, security first: ① security audit
  (capability-classification consistency + gate/limit coverage + the
  three hard lines) ② doc completion (SECURITY.md, CONTRIBUTING,
  CODE_OF_CONDUCT, CHANGELOG, rustdoc) ③ light cleanup/accuracy
  (dead links, local absolute paths in a few docs) ④ mechanism
  handoff (repository field, v0.1.0 tag, bundle; the actual public repo
  create+push is the user's account action). **Secrets scan: DONE
  by me 2026-07-21, CLEAN** — current tree (git grep ~15 pattern
  classes) AND all 1495 commits of preserved history (`git log --all -G`
  / `--diff-filter=A`): no keys/tokens/PII, zero tracked `.json`, no
  sensitive files ever added. Two non-secret notes → cleanup bucket
  (local absolute paths in a few docs; a benign `token="rum-browser-smoke"` test
  literal). First brief = **brief-289** = capability-classification &
  limit-coverage consistency audit (diagnostic; written in an
  engineering-hygiene register to clear the Codex cybersecurity filter —
  see `codex-cybersecurity-filter.md`; the secrets scan is deliberately
  kept a director step, NOT in the brief). Memory:
  `publication-prep-line.md`.
- **MATCHER + DEBT-CLEARING + REPL/TRACE LINES — ALL COMPLETE
  (2026-07-20 → 07-21, briefs 275-288). NEXT DIRECTION = fresh user
  decision; no active plan.** Memory: `matcher-oneidentity-flat-line.md`,
  `repl-cli-polish-line.md`. Detail in `TODO.md`'s top two COMPLETE
  sections. Endstate: workspace 1675/0, reference 642, **zero files over
  the 35k architecture-debt line**.
  Durable lessons from these lines, in order of how much they cost me:
  - **A brief's MAY list failed FIVE times on this arc.** Every stop-loss
    was Codex's correct call on my omission. The fix is not "list files
    more carefully" — it is (a) trace an existing sibling's wiring to
    enumerate a registration chain rather than recalling it, and
    (b) write a GENERAL accept-vs-stop criterion into the brief so a
    missed file still cannot admit a real regression (brief-279's
    "collect-all row ADDITIONS on Flat-without-OneIdentity heads are
    legitimate; any first-match change is a stop" is the model).
  - **A line's motivation can be a fossil.** The matcher work traced back
    to WK-derived tests that brief-276 had just retired as policy
    fossils — i.e. it was chasing what it had just repudiated. Re-ground
    it on an independent, measured defect before continuing
    (`MatchQ[g[1,2], g[a_g, b_]]` was False though `g[g[1], 2]` flattens
    to `g[1,2]`).
  - **One change containing two mechanisms needs per-mechanism exposure
    analysis.** "No built-in is affected" held for the wrapped-candidate
    half but NOT for the `optional.rs` under-collection fix, which is not
    `OneIdentity`-gated and so changed built-in `Plus`/`Times`. The tell
    was an asymmetric baseline (`Orderless` collected `{a,{b},0}` but not
    its mirror).
  - **Capability classification must be read from the assertion, not
    inferred from file location.** I told the user `Trace` was
    `Diagnostic`-gated because it lives in `debug.rs`; it is
    `Capability::Pure` (`capabilities.rs:231`).
  - **Instruments lied repeatedly** — a timing probe that measured an
    already-evaluated OwnValue (tell: time DECREASED as n grew), and
    extraction regexes that undercounted twice (missed multi-line
    entries; missed `pub(super) fn`). Cross-check every script-derived
    count before believing it.
  - Frozen snapshots passed their first real outing: adding `:trace` to
    help changed exactly the `:help` snapshot and nothing else.
- **LAZY SEQUENCES + SEALED SCOPING LINE — COMPLETE (2026-07-18 →
  07-20; user ruling; two WL15-survey outcomes, design-first, full
  ambition; design docs `docs/dev/lazy-sequences-design.md` +
  `sealed-scoping-design.md`; memory
  `lazy-and-sealed-scoping-line.md`). NEXT DIRECTION = fresh user
  decision; no active plan.** Sequencing: (c) first (low
  risk), then (d) (touches symbol resolution). **(c) LAZY PURE SEQUENCES
  — COMPLETE (briefs 262-267):** 12 lazy heads (`LazyList`/`LazyRange`/
  `LazyMap`/`LazySelect`/`LazyTake`/`LazyDrop`/`LazyTable`/`LazyNestList`/
  `LazyFoldList`/`LazySubsets`/`LazyTuples`/`LazyPermutations`) + the
  kernel `Part` symbolic re-dispatch seam (`lz[[i]]` →
  `self.evaluate(First[LazyDrop[lz, i-1]])`, no reverse crate dep) + a
  folk eager `Permutations`. Opaque `LazyList` (`{HoldAllComplete,
  Protected}`) over a per-element pull `PullStepper` (source/transformer
  arms; ceiling+cancel charged at the source → infinite/huge spaces fail
  closed, never hang); all `Pure`, zero new capability, snapshot
  re-derives (no memo in v1); reference 621→634; guide `evaluation.md`
  §Lazy Sequences. The STATEFUL half (`IncrementalObject`/`NextValue`
  cursors) is CLOSED WON'T-DO in-kernel (downstream pkg only if pulled).
  Codex stop-lossed 3× on my brief scope/ripple omissions (Part
  cross-crate; `structure_ops/mod.rs` re-export) — clean each time,
  revise+redispatch. **(d) SEALED SCOPING — COMPLETE (2026-07-19 → 07-20, briefs
  268-271):** grounding 268 (REFUTED the original Module-gensym
  substrate; the seal = fresh bit-62 identities kept OUT of
  `symbol_generations`, resolver byte-untouched, miss branch = the
  boundary; `mark_symbol_removed` exclusion) → spine 269
  (`DefinePackage` three-tier, suspension-safe install, reference 637)
  → introspection 270 (one bit-62 predicate + one Diagnostic-gated
  reveal across the full Q5 surface; host APIs = trusted) → snapshot
  271 (snapshot-by-re-registration; `RestoreSealedReferences`
  restore-mode re-link marker; stale-epoch rule enforced as
  snapshot-time orphan refusal; fallible `snapshot_to_bytes`;
  Save/DumpSave sealedref refusal; reference 638) → 3b close-out
  (guide §Sealed Packages, README divergence section per the
  2026-07-19 ruling). Gate catches this line: brief-271's undeclared
  process-global identity allocator reverted to session-scoped
  (identities are session-scoped; numeric reuse across sessions is
  sound — my own over-pinned cross-restore assert had provoked it).
  A/B method closure: the microsecond list-tier labels are
  layout-lottery-sensitive (±60% bidirectional; 269's movements
  reversed wholesale by 271) — judge by direction-mix + adjacent-tier
  flatness + structural argument, never magnitude alone.
- **CLEANUP + MESSAGE-REGISTRY LINE — COMPLETE (2026-07-17 → 07-18;
  user ruling 2026-07-17; queue in TODO.md §ACTIVE PLAN). All five
  items closed:** (1) message-tag registry + docgen — briefs 243,
  245-249, 252-253 (965 registered pairs, four-surface merge,
  debug-assertion enforcement with the declared-or-exempt rule;
  carriers General+UpTo per user ruling); (2) `vm_rhs_template/`
  DELETED (brief-244: ON path crashed the guard-adjacent recursion
  test; A/B clean); (3) jupyter `signature.rs` → RustCrypto
  (brief-250; flake did NOT recur); (4) rum-cli feature forwarding
  (brief-251, four-crate chain, NDF digest-free); (5) **the wl15/wfr
  design pass — COMPLETE (briefs 254-259, 2026-07-18)**: batch 1
  Take/Drop `UpTo`; batch 2 the hold half (`SubValuesHoldAll` 255 +
  mutation/schema operator forms 256-257 + `ReplaceAllUnheld` 258);
  batch 3 structured exceptions (259 — new `EvalError::ExceptionThrown`
  channel separate from Throw/Catch by construction, string type
  hierarchy, ancestry dispatch, in-band `Failure` on uncaught).
  Memory `wl15-wfr-pass-line.md`; control-family registration lesson
  `control-family-builtin-architecture.md`. **Catalog-split follow-up
  from item 1 — DONE (briefs 260-261):** kernel `namespace.rs` and the
  stdlib strings/structure/associations catalogs split by family, all
  now under threshold, reference byte-identical.
  `stdlib_attributes.rs` (28KB) accepted as debt (user 2026-07-18;
  binary_search over one sorted const, no family boundary).
  Reference at that close 621; workspace 1554/0. Dispatch sandbox defaults to
  workspace-write (67cbc88; loopback caveat + escalation in
  TEMPLATE.md). **Same 2026-07-17 ruling: the parked pools are CLOSED
  WON'T-DO** — stdlib parked candidates (ByteArray audit, compression,
  unicode normalization, ipaddress folk, fuzzing, codec follow-ons)
  and the embedding parked tail (background-eval helper, per-request
  rollback API, lighter checkpoint). Standing Engineering Debt stays a
  record of known limitations, not a queue. fs/os/net stay deferred
  until after publication. **NEXT DIRECTION decided 2026-07-18: the lazy
  sequences + sealed scoping line (top of this section) — both halves
  COMPLETE 2026-07-20.** Memory: `message-registry-cleanup-line.md`,
  `wl15-wfr-pass-line.md`.
- **RELEASE TRANSFORMATION LINE — COMPLETE (2026-07-04 → 07-06).** `main`
  itself was transformed into release shape with full history kept (no
  remote; endpoint = clean tree + bundle cold backup; publication mechanics
  are a later user decision). Landed: briefs 124-131 + M0/M1/M1.5/M2 —
  difftest and external-engine tooling removed, docs archived, narrative and
  policy rewritten, wording scan, generated builtin reference + drift test,
  nine-chapter guide, rustdoc + metadata, native stack guard +
  ReplaceRepeated limit-as-value. Removed material lives in the private
  archive `../Archive/rum-history/` (MANIFEST.md maps
  each item to its removal commit; recycling candidates flagged). Memory:
  `release-transformation-line.md`.
- **PUBLICATION DEFERRAL — REVERSED 2026-07-21 (superseded; see the
  PUBLICATION-PREP LINE bullet at the top of this section).** The
  2026-07-06 directive was "publication deferred indefinitely, do not
  steer toward release; rum publishes only after sufficient divergence."
  On 2026-07-21 the user ruled divergence IS now sufficient and opened
  the publication-prep line. Kept here as the historical record, not a
  live directive. The stated direction that earned the divergence —
  embedded scripting + sandbox safety + rapid script development
  (elisp/guile/lua/python family) — still holds. The FS-purity line
  (briefs 132-134)
  landed the three mechanical kernel hard lines: zero unsafe, zero std::fs
  (non-test), Pure = deterministic. The stdlib-expansion line (briefs
  135-156) is COMPLETE.
- **USABILITY PASS — COMPLETE (2026-07-07 → 2026-07-14, briefs 161-230).**
  The "works well" layer over every installed builtin (three axes:
  flexibility/convenience/safety), audit-then-fix per sweep unit. Tracker
  `docs/dev/usability-pass.md` (per-unit ledgers + close-outs); memory
  `usability-pass-line.md`. **All 10 units closed — 605 builtins swept**
  (units 1-7 = 446 stdlib, briefs 161-207; unit 8 = 55 kernel A, 208-213;
  unit 9 = 47 kernel B, 214-222; unit 10 = 57 kernel C, 223-230), reference
  613 after the FileNames restore. The kernel units carried the highest-value
  finds:
  - **Unit 8 (kernel A)**: `Protected` enforcement surface lifted 129/612 →
    608/612; a REAL `ApplyTo` protection-bypass bug caught at the gate
    (`store_held_update` skipped the recheck). Standing boundary: **`Protected`
    is a mistake-guard + WL conformance, NOT a sandbox boundary** — real
    isolation = per-request rollback = the dun host line; do not conflate.
  - **Unit 9 (kernel B)**: evaluator hold correctness across three user
    rulings — `Unevaluated` no-consumer restore, `HoldAll` Sequence splice,
    Module error-path gensym reclamation (WL semantics; mma-hol Strict gate),
    staged `With`, replacement-produced Sequence splices into held rebuilds.
  - **Unit 10 (kernel C)**: a REAL loader capability bypass caught at the gate
    (`BeginPackage`/`DeclarePackage` → `eval_needs` skipped the `FileRead`
    dispatch check; fixed by gating at the `eval_get`/`eval_needs` seam) and a
    REAL `Pause[10^100]` host-process panic (Duration overflow). Three
    capability-decision candidates ruled: `Input`/`InputString`/`Dialog` →
    new `Capability::Interactive` (sandbox denies, trusted allows); `TestReport`
    → `Clock` KEPT (a test report is intrinsically multi-dimensional — timing
    is its job, not an impure contaminant; **not every impure label is a
    mislabel to purify**); `FileNames` → restored as a `FileRead`-gated folk
    glob through a new `list_dir` provider method.
  - DEFERRED (stated, not dropped): the **builtin message-tag registry**
    (`Messages[Set]` empty even after `Set::argx`; ~108 ad-hoc `push_message`
    sites, no static catalog). Its own project; pairs with docgen.
  Standing user directives (still in force for any future builtin work): high
  coverage incl. the WL-inherited surface; compat enhancement only (default
  behavior stays WL-conformant, new powers via options / rum-native siblings).
  Standing design rulings: arithmetic singularities stay INERT (no
  special-value arithmetic — user); Orderless/Flat/OneIdentity on folk numeric
  reducers + Equivalent + the Implies short-circuit all REJECTED;
  ConformSchema/SchemaErrors operator forms DEFERRED (HoldRest hold-semantics).
- **STATEFUL↔PURE SEPARATION LINE — COMPLETE (2026-07-08, briefs 200-202).**
  User rulings, spun out of unit-6 datetime. Killed the pure/impure
  DUAL-FORM anti-pattern (one symbol mixing a pure + a stateful form).
  Final: `AbsoluteTime[]` = the sole numeric wall-clock reader (`Clock`,
  spec-level, reader-only, kernel); `FromDateList` = pure converter (`Pure`,
  stdlib, inverse of DateList); `DateList`/`DateString` genuinely pure;
  `Now` REMOVED (reserved for downstream rum-os); `SeedRandom` split →
  `ReseedRandom[]` (`Random`) + `SeedRandom[seed]` (`Pure`). **Standing
  rule: stateful reads are single-purpose symbols gated by a SPEC-level
  capability — never crammed into a pure symbol, never gated by a runtime
  check (stdlib now has ZERO runtime capability checks).** Whole-surface
  audit done; the lone mild kernel candidate `TestReport` was reviewed and
  CLOSED with NO CHANGE (unit 10, user ruling 2026-07-14): a test report is
  intrinsically multi-dimensional (correctness + timing + memory), so `Clock`
  is the honest label, not a mislabel to purify. Detail:
  `usability-pass-line.md` + tracker.
- **HOST-EMBEDDING LINE — COMPLETE. Concern (b) closed at b4 (2026-07-15);
  concern (a) = the EMBEDDING API LINE, COMPLETE (2026-07-16, briefs
  236-242, opened and closed in one day; canonical doc
  `docs/dev/embedding-api-line.md`, memory `embedding-line-a-scenarios`).**
  (a) endstate: scenario design (three user-supplied abandoned downstream
  concepts: Python/AI-infra adapter, Rust/wasm tooling, tauri typst IDE =
  dun's ancestor) + in-tree consumer survey (findings F1-F8) drove five
  gated increments — a1 contract honesty (exit_status, requires_reset, b1
  termination-field adoption), a2 dogfood arc (contract surface + rum-cli +
  rum-repl ports; **in-tree hosts are pure rum-host consumers — cli/repl
  have ZERO rum-runtime/rum-stdlib dependencies; user ruling: dogfood
  stance A**), a3 typed bridge out (Bool/Association/Rule + jupyter
  application/json), a4 typed bridge in (expr_from_typed + injection-safe
  `HostSession::call`; kernel seam `eval_expr_request`), a5 named profiles
  (hermetic/scripting; all-8-capability pins, a ninth Capability variant
  fails compilation until every profile takes a stance). Parked pending
  scenario pull: background-eval helper, per-request rollback API, lighter
  checkpoint. Cold backup `rum-2026-07-16-3d31e09.bundle`. Watch: a
  rum-jupyter native ZeroMQ ordering flake has surfaced twice
  (brief-165, brief-242; green in isolation both times) — brief it on a
  third occurrence. **NEXT DIRECTION decided 2026-07-17: the cleanup +
  message-registry line (top of this section); the parked tail above is
  closed WON'T-DO by the same ruling.**
  Original sequencing (user 2026-07-14): concern (b) dun pure-sandbox +
  protocol conformance FIRST (explicit requirements, clear boundary),
  concern (a) general embedding API only AFTER (b) completes, and (a) MUST
  open with scenario design + boundary drawing — no feature bricklaying on
  instinct. That is how it ran.
  **b1–b4 DONE and reviewed boundary-clean** (b1 structured termination
  discriminator, b2 request-level memory ceiling, b3 stdout-ownership hygiene
  hard line, b4 crash-only + per-request isolation contract — all generic
  sandbox-hardening primitives, zero dun/editor/protocol naming in the
  kernel). **b5 (dun protocol host prototype) CANCELLED (user boundary ruling
  2026-07-15): building a dun plugin is downstream work, not rum's job — rum
  is a language project, dun an editor project; rum's obligation was to harden
  the sandbox to downstream-usable strength, achieved at b4, and hosting one
  editor's reference plugin crosses the "don't make rum an editor framework"
  line. Part 1 (brief-235, the `rum-dun-host` codec crate) had landed and was
  reverted (`6062c03`+`ed01e33`, revert commit `93a93d2`).** **fs/os/net
  downstream capabilities are DEFERRED until after publication (user
  2026-07-15) — explicitly NOT the next target.** (Historical diagnosis
  detail below; the concern-(a) gaps it lists were all closed by briefs
  236-242.) Diagnosis headline: gates 1-3 done (sandboxed
  profile, Interactive capability, panic isolation), stdout/crash-only/
  cold-start (~4.8 ms full cycle) already satisfied but unpinned; the two
  real seams = structured termination discriminator (graceful cancel/timeout
  is in-band `TerminatedEvaluation[...]` Success) + request-level memory
  ceiling (mechanism exists via `MemoryConstrained`/`current_memory_limit`,
  missing the `set_eval_timeout`-analog knob; active-path walk needs
  amortization). The downstream `dun` editor is the first real pull, but the
  framing was RESET twice, both recorded in `dun-plugin-requirements-line.md`:
  1. **dun went protocol-first (2026-07-13, third doc
     `docs/dev/dun-protocol-host-requirements.md`).** dun shipped a
     protocol-first plugin system: plugins run in EXTERNAL host processes
     speaking a framed-stdio JSON protocol; the editor links no runtime. As
     dun-facing surface, the in-process embedding asks (`session.call`,
     `HostTyped` as dun's result surface) are DEAD — they demote to internal
     needs of a small `rum-host` shim (protocol on one side, kernel on the
     other). What SURVIVES and is now load-bearing for dun's `pure-sandbox`
     trust class (rum's exclusive slot — Python/Lua hosts can't clear it):
     pure-only + non-interactive + per-request isolation + crash-only +
     resource-outcome classification, PLUS new protocol-born asks (stdout
     fully owned by the embedder so plugin `Print` can't corrupt framing;
     kernel-side memory ceiling with a classified outcome — dun has no RSS
     watchdog; cold-start latency; JSON boundary).
  2. **User strategic reframe (2026-07-14): do NOT narrow the line to dun's
     protocol shim.** dun began as a showcase of rum's EMBEDDABILITY; its
     pivot (rum kernel too big for a lightweight editor) means dun now shows
     off only the sandbox, not the kernel's session-management / in-kernel-
     communication strengths. Those strengths are still wanted by FUTURE
     tighter-embedding applications, so evaluate the general embedding API
     (rich `HostTyped`, typed `call`, session management) on **general
     embeddable-language merits, not dun's current needs**. → the line has
     **two parallel concerns**: (a) the general embeddable-language API,
     judged on general value; (b) dun's `pure-sandbox` + protocol conformance.
     Overlapping but not conflated.
  How it actually ran: the diagnosis landed (2026-07-14), then concern (b)
  executed as briefs b1–b4 (231–234) and CLOSED at b4 with b5 cut (above).
  The `syntax-highlight` `rum-host` prototype that the diagnosis proposed was
  the cancelled b5 — that idea is retired (downstream work). Hard line held:
  don't make rum an editor framework (the protocol boundary made the
  separation physical). Memory: `dun-plugin-requirements-line.md`.
- Earlier arcs (correctness mainline, memory line, host-hardening Tier-1/2,
  pattern-matching line, crate rebalance route 2a, file slimming) are all
  CLOSED; their plans/audits are in the archive and in git history. Cross-
  session state lives in my memory directory, not in `PROGRESS.md` (user ban
  on reading it for context; appending entries is fine).

## Pointers

- `AGENTS.md` — operational first-stop (snapshot, commands, testing policy,
  reading path).
- `TODO.md` — active roadmap. `PROGRESS.md` — append-only log.
- `~/.claude/.../memory/` — my cross-session memory of this project's arc and
  the recurring lessons (`rum-project-context.md`,
  `release-transformation-line.md`, …).
