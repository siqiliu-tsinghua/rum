# TODO

This file is for active or near-term work only. Completed history and
operational lessons live in [PROGRESS.md](./PROGRESS.md) (append-only log),
and release-facing changes live in [CHANGELOG.md](./CHANGELOG.md).

Status checkpoint: 2026-09-30, committed tree through brief-562's design
report; the next action is writing step A of G2 from `.temp/brief-562/draft-A-pull-machine.md`
under the director rulings recorded below.
Uncommitted performance candidates do not count as completed until their
correctness, timing, counter, and memory gates have all been reviewed.

**Where work resumes (2026-09-30) — the suspension-continuation branch, step
4: G1-G8, the 86 lost continuations, after the front ends (briefs 553-558).**
Steps 1-2 are done. brief-550 (`c6e95dd1`) made guest `Get`, `Needs`, `BeginPackage` dependencies
and `DeclarePackage` stubs suspend and resume with an inherited request budget.
brief-551 (`34e57a8d`) fixed what the re-probe of the batch boundary found: a
public `eval_file` suspended inside a nested guest load panicked on cancel
(its eager pop took the inner file's `load_stack` entry — a director gate miss
on 550), and a failed `restore_from_bytes` on input wedged the session; a
suspended public file now rolls back inner to outer at abandonment, a failed
restore retires its suspension, and a request-boundary check (test and debug
builds) fails when a completed request leaves a suspension owner, with five
watched falsifiers. Both gates reproduced on the host (workspace 2734 then
2756 passed, 0 failed, native ZeroMQ real; the new tests fail on the pre-change
trees). brief-552 establishes by execution what `rum-repl`, `rum-cli` and
`rum-jupyter` do when guest code asks for input — brief-549 found from source
that several leave the session blocked — and proposes a disposition per entry
point, with the user's decisions marked. After it: G1-G8, E1, E2.
**brief-552 is in (design; director re-ran its 37 probes: 15 pass, 22 red,
identical).** Executed findings: the REPL never resumes — after any input
request the next line is refused (its crate rustdoc already claims "resumable
input"); `:trace` likewise; CLI non-interactive modes exit 1 cleanly but four
borrowed helpers leave the session blocked for a caller that reuses it;
Jupyter's immediate `execute` and every `allow_stdin=false` surface return a
terminal error while leaving the owner installed. Three Jupyter defects beyond
the brief: an interrupt while waiting for input leaves `awaiting_stdin` true
and the next execute rejected; an overlapping execute during an input wait
marks the adapter Idle and erases the original reply's routing ids; and each
suspension re-renders the whole buffered print prefix (queued `prefix, middle`
shows as `prefix, prefix, middle, prefix, middle`). Six drafts (A REPL driver,
B terminal masking and interrupt, C CLI helpers, D Jupyter abandonment, E
Jupyter lifecycle, F Jupyter output once).
**User rulings, 2026-09-30: all ten recommendations accepted (U1-U10, text in
brief-552 §3), and two new dependencies authorized for draft B: `rustix`
(echo suppression) and `signal-hook` (Ctrl-C).** In short: the REPL answers
from its next line and resumes (`:trace` too), answers are literal and never
history; empty prompt `input> `; EOF cancels and exits, Ctrl-C cancels to the
primary prompt; password input never echoes and is refused rather than shown
where masking is unavailable; CLI script modes stay non-interactive; Jupyter
keeps both APIs, the immediate one abandoning; automatic abandonment reports
only the original input reason; notebook interrupt during a prompt finishes
with Cancelled; each print is emitted once; shutdown documents disposal.
**A is done (brief-553):** the REPL answers from its next line and resumes;
password requests are refused until B. Director gate: workspace 2778/0/58 with
native ZeroMQ real; retention no-default 49/0; the new REPL tests fail on the
pre-change tree (21 of 22; the pass is the overlap-preservation control); a
real `rum-cli` REPL driven through a pseudo-terminal answers `alice` and a
whitespace-padded answer, cancels on EOF with cleanup once, and keeps answers
out of `.rum_history`. The worker stopped at the fence because any source
change expires the retention fingerprints and certificates (they hash every
file under `crates/`); the director replayed them, and `TEMPLATE.md` now makes
that refresh a standing duty of every brief. **B is done (brief-554):** on a
terminal the REPL answers masked requests with echo off and restores the
terminal on every exit, and Ctrl-C while waiting cancels to the prompt through a
wait-scoped self-pipe (`rustix` 1.1.4 and `signal-hook` 0.3.18, Unix only,
`rum-repl`); Ctrl-C elsewhere still terminates. Director gate: workspace
2790/0/58 with native ZeroMQ and the pty tests run for real; the new pty tests
fail on the pre-change tree (5 of 9; the passes are the four unchanged-behaviour
controls); a real `rum-cli` on a pseudo-terminal hid a typed secret, kept it out
of history, cancelled on Ctrl-C with cleanup once, and still exited 130 on
Ctrl-C at an idle prompt. **C is done (brief-555):** every `rum-cli`
evaluation helper takes its cancel handle before admission and, on an input
request, cancels and re-enters before returning — the broken-pipe return
included — reporting only the original reason; cleanup `Print` output reaches
stdout once. Director ruling on the resume: cleanup output means `Print`; a
message emitted by cleanup while a public file is cancelled rolls back with the
file, as the file-load rule already says, and a script delivers it once.
Director gate: workspace 2811/0/58 with native ZeroMQ real; the new binary
cleanup test fails on the pre-change tree (1 of 6; the rest are controls);
`rum-cli --file` with a `WithCleanup` prints the prefix and the cleanup once,
skips the tail, reports input required and exits 1. **D is done
(brief-556):** every Jupyter surface that returns a terminal input-required
error — immediate `execute`, the no-stdin worker, the reply-only shell wrapper,
the event pump, the local driver — now retires the request it admitted first
(the worker on its own thread) and keeps the original error; the next execute
runs. Director gate: workspace 2831/0/58 with native ZeroMQ real; the new tests
fail on the pre-change tree (14 of 20; the six passes are controls). **E is
done (brief-557):** an admitted asynchronous execute owns its cancel handle,
parent, count and route from admission to its one terminal reply; an interrupt
while it waits withdraws the input request, retires the request on the worker
and replies Cancelled on the original route, then idle; an overlapping execute
is refused at admission without touching the original's state or route.
Director gate: workspace 2844/0/58 with native ZeroMQ real; retention
no-default 49/0; the new lifecycle tests fail on the pre-change tree (12 of 13;
the pass is the mismatched-parent control). The worker had loosened brief-556's
`start` helper to accept an immediate reply for the now-immediate refusal; the
director restored its assertion and gave refusals their own helper that asserts
the immediate reply, then replayed the retention fingerprints the edit expired.
**F is done (brief-558), and with it the front-end step:** a resumed
Jupyter execute emits each output exactly once — per-family positions (prints,
messages, trace) kept in the execute's owned lifecycle, equal prints kept as two
outputs, output reported at a suspension emitted before its `input_request`.
The first run stopped on a requirement the director had written wrong: the
host's outcomes carry no chronology between families (swapping a `Print` and a
`Message` yields an identical queued result), so parity is per family and the
renderer's cross-family order is documented as a host-contract limitation.
Director gate: workspace 2856/0/58 with native ZeroMQ real; the new tests fail on
the pre-change tree (10 of 12; the two passes measure the unchanged host
buffers). **Found, not in this branch — QUEUED by user ruling 2026-09-30:**
every front end loses the relative order of prints and messages within one
request, with or without suspension, because the kernel keeps them in separate
buffers and the host returns them grouped; fixing it means one ordered output
sequence from the kernel through the host. Next: G1-G8 (drafts in
`.temp/brief-548/draft-G*.md`, re-probe before each), then E1, E2.
**G1, G6 and G8 are done for correctness (brief-559):** `Scan`, multi-name
`OptionValue`, `ReplaceRepeated`, three-operand `Dot` and `Differences` resume
into their own loops; lost continuations 86 -> 80. The director re-ran brief-548's
195 probes first (all 91 divergences unchanged). Gate: serial workspace
2877/0/58 with native ZeroMQ real; the new tests fail on the pre-change tree (19
of 21; the passes are the Dot and ReplaceRepeated cancellation controls).
Two host kills under memory pressure along the way; the user ruled serial runs
(`CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`), which exposed and fixed a latent
defect in brief-555's CLI test harness (`--quiet`, no inherited thread count).
The ordinary path got dearer — per-iteration instructions `Differences` +1.86%,
`ReplaceRepeated` +0.4%, `OptionValue` +0.38%. **brief-560 erased all but one
residual:** every ordinary loop is restored and frames are captured in cold
helpers; eight of nine labels now sit at or below the base (most 0.1-0.6%
below). `Differences` keeps **+0.82%** (director-measured; ~80 instructions
per call of `Differences[{1,4,9,16,25}]`), **recorded, not tuned further**:
callgrind (calibrated against `perf stat`) attributes it to `difference_expr`
losing its inlining, and ten measured candidates — including a single call site
(candidate 09) and a single call site with `#[inline(always)]` (director
experiment, +1.17%) — could not recover it; carrying the loop's state in the
error arm is what changes the hot loop's shape. Next: G2 (the lazy pull
machine, 14 sites).
**G2 design is in (brief-561, design only):** one frame owning the whole pull
machine, state moved into it only on `InputRequired`, a cold working copy on an
actual resume (frames are immutable and stay Session-rooted while they run),
completion frames for `Normal`, `Take` and `First` (`Part` inherits `First`),
and a separate frame for `LazyTable`'s eager bounds; 20 brief-548 witnesses plus
19 lifecycle cases, 37 red and 2 controls before any change; two steps, lost
80 -> 69 -> 66. **The director's adjacent-boundary probe found a pre-existing
process abort in the same machine:** a `LazyMap` chain built by a loop
(`Do[x = LazyMap[Identity, x], {n}]`) overflows the native stack on the first
realization — `Take[x, 0]` is enough, so construction or destruction of the
`Box`-chained `PullStepper`, not the pull — between 2 000 and 5 000 levels on
the ruled 2 MiB stack (5 000-10 000 on 8 MiB); the descriptor itself builds and
drops at 20 000. Nothing guards it (`lazy.rs` never enters the native stack
guard, and the accumulator gate covers `Option<Arc<_>>` chains, not `Box`).
brief-561's design would add four more walks of the same shape, so **G2 now
includes making the machine depth-safe**, and brief-562 revises the design
first; the director's preferred direction is to linearize the unary chain into
a source plus a stage vector driven by an explicit cursor, which makes every
walk iterative and is the pending-pull program counter brief-548 asked for.
Also found, outside the branch — QUEUED by user ruling 2026-09-30: an empty
lazy `First` emits `First::nofirst` twice where `First[{}]` emits it once (the
residual `First[List[]]` it returns is evaluated again).
**Queue after this branch (user ruling 2026-09-30):** (Q1) one ordered output
sequence from the kernel through the host, so prints and messages keep their
relative order in every front end; (Q2) the duplicated `First::nofirst` on an
empty lazy list. Both wait for the branch to close; the retention line, paused
mid-line at E0, resumes first under the "a line once opened is finished"
ruling unless the user places them earlier. Also behind the branch: the
`rum-cli`/`rum-repl` `rum-runtime` dev-dependency audit.
**brief-562 is in (design only) — the flat machine is chosen.** The unary-chain
premise holds (every adapter has one lazy child; `LazyTable` lowers to one
chain). One `PullSource` plus a `Vec<PullStage>`, outermost first, driven by an
indexed Down/Up cursor: construction, pull, clone, trace, drop and abandonment
all iterate, no self-reaching type remains, and the suspension state is the
cursor (Map/Fold stage index, Select index plus candidate, or NestList's saved
yield) — brief-561's waiter records and cold path reconstruction are gone. The
guarded-tree alternative lost on enforceability: `Clone`, `FrameRootTrace` and
`Drop` are infallible and cannot return `StackLimitExceeded`. Everything else
of brief-561 stands (single machine frame, cold copy on an actual resume,
observer re-acquired, `Option<Expr>` codec, the three completion frames, the
separate bounds frame). Measured pre-fix ceiling on 2 MiB: ~2.17k stages (pass
2 156, abort 2 187; 944 bytes per constructor activation in the release
binary, 656 for `next`, 48 for drop). Scratch run: the 39 brief-561 records
reproduce exactly; nine 2 MiB shallow companions (n = 32) are 5 red / 4
controls; their nine 10 000-stage counterparts abort the release CLI (exit 134)
and ship as ignored tests that step A must enable. Drafts A (flat machine and
consumers, 80 -> 69) and B (bounds, 69 -> 66) are in `.temp/brief-562/`, with
the inventory map `inventory-plan.md` and a 54-label panel
(`performance-panel.json`: brief-561's 30 plus 24 depth labels at 8/64/256).
brief-561's census note was wrong: `First` and `Take` are already N.
**Director rulings for the next brief (step A, to write from draft A):**
(1) the callback-completion cancellation checkpoint is accepted — every lazy
guest callback becomes a cancellation boundary, as every source pull already
is (one observer load against a guest evaluation; it is what makes the
cancel-at-inner-callback oracle agree, queued log 32 vs resumed log 1 today);
the extra poll every 256 stage transitions is **rejected** — between source
polls the work is O(depth), and depth is bounded by charged, cancellable
construction work, so the countdown and its shallow specialization buy the
hot loop complexity with no termination need. (2) Performance: allocations
first, then paired per-iteration instruction slopes on all 54 labels for
every candidate, +0.05% per sample as in brief-560; the full timing suite
runs for the final candidate only and informs, it does not decide. (3) The
inventory map is authorized as written — remove the `PullStepper`
self-reaching row, re-identify the `PullBudget` derive, relocate the ten named
constructs, add the named new rows; any other row change still STOPs.
Serial bounds from measured history (briefs 559-560): `cargo test -p
rum-stdlib` 990-1 047 s, `--workspace` 651-1 012 s, no-default retention ~900 s
(560's 900.1 s may have met its bound) — set 1 500 s for each; release
`rum-cli` build 89-108 s (300 s).
recorded, not in this branch: `rum-cli` and `rum-repl` carry a `rum-runtime`
**dev**-dependency (`Cargo.toml:27`, `:22`), which the "zero rum-runtime
dependencies" statement of dogfood stance A does not mention — audit it after
the branch.

**The retention line — paused at E0 for the suspension-continuation branch
below, after its first certified region.** The line exists because an armed
memory ceiling can be walked through by a builtin's Rust-local accumulator,
which is in neither the session total `S` nor the in-flight expression `W`. Read
[docs/dev/retention-closure-design.md](./docs/dev/retention-closure-design.md)
first; it records why whole-closure traversal was abandoned (the option
region's closure reached 868 bodies; aliasing and data dependence are
undecidable, and IFDS needs the call graph open dispatch denies) and the local
design that replaced it.

Where it stands:

- **Built and enforced**: the lease protocol, activation bridge and runtime
  certified reentry domain (`rum-runtime/src/runtime/accounting_mode/`,
  `options/reentry.rs`); the local retention lint with its contract records,
  covered marks and compiler oracle
  (`rum-stdlib/src/tests/retention_closure/`, 49 required tests).
- **First production coverage**: `inventory/promoted.tsv` holds two
  `LocalCovered` bodies, C01 `merged_option_exprs_for_symbol` and C02
  `evaluate_runtime_option_expr` (`ee81758a`), each with its exclusions in the
  row. Unarmed they cost nothing over the pre-conversion build (`cae6f550`).
- **The map, from brief-547 (2026-09-28).** All 643 declarations re-read
  against the current tree: **N 195, P 291, OutOfClass 56, Unsettled 101**.
  The 147 carried rows split 90 N / 19 P / 38 unsettled, and the P screen
  moved 12 rows to N. The N class is cut into 13 regions by conversion
  mechanism (R01-R12, R14), which need seven new protocol surfaces (PF forest
  and prefix, PS value and outcome slots, PD typed records, PX numeric
  workspaces, PB text writer, PH host-result transfer, PT detached-owner
  restore). Ranked by declarations fully closed: R01 29, R02 22, R04 23,
  R03 23, R14 15, R09 14, R05 13, R07 14, R10 10, R08 8, R06 6, R12 13,
  R11 5 — 195 in all, conditional on the 101. **The control-family
  single-slot hypothesis was refuted**: it splits into R04 (end a
  semantically unused value's lifetime before the next boundary — no lease
  at all), R05 (needed slots) and R06 (payload-bearing outcomes). Tables and
  both drafts are in `.temp/brief-547/`; the report is appended to the brief.
- **Still open, in order**:
  1. **The four `UnresolvedConsumer` escaping copies** in
     `inventory/consumer-obligations.tsv` (step E0; brief-547 §6 drafts it:
     two borrow, two keep a live-source P proof with a runtime falsifier).
  2. **H — a test-only exporter that runs the local lint on named production
     bodies.** Nothing today points the lint at a body outside C01/C02, so no
     new region can get real rejection rows.
  3. **Settle the 101 unsettled rows with H**, then recompute the ranking.
     The N class is not known until they are settled.
  4. **K and A** — exact contracts and narrow admissions, split per record
     from H's real output; then each protocol surface ahead of its users, and
     the regions by leverage, each with the option-region pattern: design
     brief, local reads, leased construction, accumulator and handoff, erase
     unarmed cost, certificate.
  5. P is a documented bound (`docs/guide/resource-limits-and-sandbox.md`,
     "What the ceiling does not see"), not work.
- **ACTIVE BRANCH, inserted 2026-09-28 (user): suspension continuations —
  runs to completion before the retention line resumes at E0.** A builtin
  that suspends mid-work (`InputString`, `Input`, `Dialog`) must resume into
  that work; many resume past it and complete with the suspending call's
  value. Found as Scan by brief-547; brief-548 swept the class
  (report appended to `docs/dev/codex/brief-548-suspension-continuation-sweep.md`,
  evidence in `.temp/brief-548/`). The oracle: the resumed path must equal
  the queued-input path (`Session::queue_input_response`) in value and side
  effects. **Director re-ran all 195 probes: identical — 104 equal, 91
  divergent.**
  - **86 confirmed `Lost` sites** (83 reachable from source, 3 in unused
    private wrappers) in eight groups: G1 Scan (1), G2 the lazy pull machine
    and its consumers (14 — every `Lazy*` adapter plus `Normal`, `Take`,
    `First`, `Part`), G3 `ConformSchema`/`SchemaErrors` (7), G4 `Rule`,
    `RuleDelayed` and held-pattern normalization in `MatchQ` and kin (16), G5
    option readers and their callers (29 — `Heads` on `Cases`, `DeleteCases`,
    `Count`, `MemberQ`, `FreeQ`, `Position`, `Level`, `Depth`, `Replace`,
    `Trace`, plus `Monitor`, `Names`, `TestReport`, `DumpSave`,
    `ArrayFilter`, `CheckAbort`), G6 multi-name `OptionValue` (1), G7 `Sow`
    matching and trace filtering (14), G8 `Differences`, three-operand `Dot`,
    `ReplaceRepeated` (4). Drafts for each in `.temp/brief-548/draft-G*.md`.
  - **38 `Transformed`**: guest `Get`/`Needs`/`BeginPackage` swallow the
    suspension into `$Failed` with `Get::load` "input required"; host
    script/file APIs document non-resumable batch suspension.
  - **A kernel panic reachable from guest source**:
    `DeclarePackage["Dep548`", {"foo"}]; Dep548`foo`, where the autoloaded
    file requests input, fails the assertion "a new suspension must not
    replace a live continuation owner" (`suspension/core.rs:228`).
    Director-reproduced. Needs `FileRead`, so not reachable under the
    sandboxed profile. **The cause is a leaked owner, and it is wider than
    the stub** (director probe, `.temp/director/leak-probe.txt`): a lone
    `Get` of a file that requests input returns `$Failed` while the input
    request stays installed, so every later request is refused;
    `Get[...]; InputString[...]` and `Needs[...]; InputString[...]` fail the
    same assertion. `ValueQ` and `Monitor` swallow correctly.
  - **User ruling, 2026-09-28: guest package loading becomes resumable**
    (option A). The host batch APIs' documented non-resumability is not
    changed by it. Step 1 of the branch is brief-549 (design): the leak's
    mechanism, the owner-leak class over all 38 `Transformed` pairs, and the
    resumable design for `Get`, `Needs`, `BeginPackage` and the
    `DeclarePackage` stub. Then G1-G8, then E1, then E2.
  - **brief-549 (design) is in** — report appended to its brief; 44 red
    tests archived in `.temp/brief-549/probe.rs`, director re-run identical.
    It confirmed the leak (`Get` converts the suspension to `$Failed` while
    `eval_file`'s snapshot restore cannot remove the owner) and audited all 38
    `Transformed` pairs: the loader rows leak, `ValueQ` and `Monitor` abandon
    correctly, and the front ends (`rum-repl`, synchronous `rum-jupyter`
    execute, `rum-cli` helpers) leave their session blocked after reporting
    input.
  - **Two resource-limit bypasses, director-confirmed**
    (`.temp/director/budget-probe*.txt`): every form of a guest-loaded file
    calls `begin_top_level_interrupt_budget`, so
    `MemoryConstrained[Get[...], 100000]` over a 50 000-element file returns
    `50000` where the inline form aborts, and a `Get` of three ~465 ms forms
    runs 1414 ms under an 800 ms request timeout that stops the inline forms at
    801 ms. Needs `FileRead`. Fixed for the guest path in brief-550. **Open for
    the host batch path**: public `eval_script`/`eval_file` renew the deadline
    per form too — whether a host request timeout is per request or per form
    is a contract question, related to "termination is last-write-wins across
    script forms" below.
  - **Queue**: brief-550 (done, `c6e95dd1`) implemented guest loading
    atomically across `Get`, `Needs`,
    `BeginPackage` and the stub, with the inherited budget and `BeginPackage`
    restoring on `Exit`. Next, the batch boundary, drafted in brief-549
    (`.temp/brief-549/draft-batch-ownership.md`, to be rewritten): the batch
    path keeps its documented non-resumability but gets an explicit
    abandonment-only owner, with rollback deferred to cancellation (director
    decision: the same model as the resumable path — reset runs no guest
    cleanup, cancellation does); snapshot restore abandons its failed input
    work. Then a front-end input disposition brief (REPL, synchronous Jupyter,
    CLI helpers — commission adapter red tests first), then G1-G8 (drafts in
    `.temp/brief-548/draft-G*.md`), E1, E2. Every draft is raw material; the
    director rewrites before dispatch.
  - **495 pairs `Unsettled`** of 2 399. The enforcement design (drafts E1,
    E2) is what settles them: E2 gives may-suspend calls a result type with
    no `Try` impl, so a bare `?` fails to compile and every site must choose a
    frame, a tail declaration or an audited boundary; E1 is a required
    catalog-wide oracle that falsifies a false tail declaration.
    `Session::evaluate` is the hottest function, so E2 is prototyped and
    measured before it is chosen over the Result-preserving static gate.

**Queued behind it — the snapshot format (user ruling, 2026-09-22).** Session
snapshots are UTF-8 source images, and that is why the interner leaks: a
symbol's identity must be expressible as text, so every generator of
distinguishable fresh symbols mints fresh interned text. The user ruled the
deferred **direct binary snapshot format** must be paid. Read §The root
constraint found by two refutations, §User rulings of 2026-09-22, and the
Standing Engineering Debt entry on snapshots. The interner quota work (briefs
505-508) is parked, not cancelled.

**Also open, in rough order of weight:**

- **CLOSED by brief-545 (2026-09-28): the unarmed cost is erased.** The
  unarmed option rows are back at or below the pre-543 base — director-measured
  instructions StringReplace -0.79%, explicit-delayed -0.75%,
  default-plus-delayed +0.21%, 128 options +0.49%; allocations per iteration
  37 against 39 and 29 against 32. Originally opened as:
  **Open unarmed cost from the option-region conversion (brief-544, 2026-09-28).**
  Correctness is complete — C01 and C02 lint clean for both declared M x R
  instances — but the accounting does not erase when unarmed. Director-measured
  per-iteration instructions against the pre-conversion base: every
  option-consuming builtin **+1.69%** (`stringreplace-single-literal-unguarded`),
  and delayed (`:>`) options **+4.91% / +6.05% / +6.32%** against pre-543 with one
  extra 40-byte allocation per iteration (an `Arc<Vec<Expr>>` the unarmed owner
  still builds). The armed cost is linear, about 2 300 instructions per option
  plus 5 600 fixed — the director's two-point fit predicts the worker's 8- and
  32-option rows within 0.12 points — so it is the security model's own path,
  not a superlinear defect. Must be dispositioned before step 545 promotes.

- **User ruling 2026-09-27 — U1, the option region's generic `R`: a runtime
  certified-registration domain.** `OptionReentry` registration accepts any
  type, so a session's registered set is finite but open in kind, unlike the
  sealed two-member accounting mode. Coverage of C01 therefore requires the
  runtime itself to enforce a closed certified domain at entry and resume,
  derived from one declaration source that also generates the lint's set, so
  the two cannot drift; unverified host registrations stay outside the covered
  path. This is step 544 of brief-541's plan (542 reads, 543 the delayed rule,
  544 accumulator and handoff, 545 first certificates).

- **DIRECTION CHANGED 2026-09-23: traversal is abandoned for compositional
  verification. Read [docs/dev/retention-closure-design.md](./docs/dev/retention-closure-design.md)
  before any further work on this line.** brief-536 measured the distance to a
  zero-unknown option region and it does not converge: `Session::evaluate`
  alone reaches 190 bodies and 860 rejection sites, every region in this class
  calls it, and the projection runs to 868 bodies and 5 763 sites without
  being an upper bound. Five of fourteen frontier rejections are structurally
  unadmittable and four are rum's own idioms — the open `OptionReentry`
  generic, its trait dispatch, derived `Clone` on `Expr`, and `SharedArgs`
  reaching `Vec<Expr>` through user `Deref`. The obstacle is the whole-closure
  requirement, not resolution quality, and it sits on known results: aliasing
  is undecidable, context-sensitive data dependence is undecidable, and the
  tractable IFDS framework needs the known call graph that open dispatch
  denies. A3-A6 are cancelled as a completion plan. The replacement is
  per-body proof against declared boundary contracts, with the obligation
  moved into the type system wherever a private constructor can carry it —
  the shape brief-531 already demonstrated. The design doc states what that
  does **not** solve, and its three open decisions are the next thing to
  settle.

- **Superseded, kept for the record — the line was paused at A2 pending a
  Codex quota reset.** Where it stands, so the resume
  costs no re-derivation:

  - **Landed**: the checker (`0264095b` back through `8aa3f455`), its census
    and the compiler oracle, plus the option-consumer continuation repairs
    that came out of it. Eighteen required checker tests, production coverage
    still **0** — nothing is promoted yet, by design.
  - **The measured target**: `EDGE_N_FRONTIER` is 79 roots / 365 definitions /
    **2 732 unique rejection sites**, against 774 resolved. Production-wide is
    7 057 definitions / 57 661 sites, and every one of the 7 057 fails the
    effects check today.
  - **Next is A2, and it is the line's make-or-break step.** Method-call
    syntax is **1 166 of the 2 732** N-frontier sites — and the director
    measured it as diffuse, spread over **259 definitions in 33 files**, worst
    case 36, so no small set of helpers collapses it. brief-532 judged only
    **one of eight** sampled method sites initially defensible. Its draft is
    `docs/dev/codex/brief-532-admitted-surface-design.md` §Draft A2; the
    director rewrites it before dispatch, as always.
  - **Decide before A2 dispatches**: the oracle compiles the crate per
    resolution claim, so the checker group costs **115.5 s** and
    `cargo test -p rum-stdlib` **127 s**, past brief-528's 120 s budget. The
    candidates are to accept and restate the budget, to keep a small required
    sample with the full sweep on demand, or to cache per source fingerprint
    the way the trusted-leaf records already do. **Do not answer it by
    weakening the falsifier.**
  - Still open from this line and unchanged: the checker's frontier sits
    behind a `resume_or_suspend!` macro boundary (the RET003 row), and the
    four `UnresolvedConsumer` escaping copies remain D3's.

- **User ruling, 2026-09-23 — the analyzer is ours.** The retention checker's
  Rust analysis is written in this repository rather than borrowed from
  `rustc_private`/dylint (nightly pin, unstable internals) or the `ra_ap_*`
  crates (large, fast-moving). The reason is that **it does not need to be
  general**: it only has to understand the subset of Rust that rum's covered
  regions use, so it can be specialized for rum, and owning the API removes
  version and API-churn dependencies. Two consequences are standing method:
  **read rustc and rust-analyzer source freely** rather than re-derive solved
  problems — both are MIT/Apache-2.0 and rum is MIT, so copied code carries
  its notice and adapted-in-spirit carries nothing — and **differential-test
  against the real compiler**, which is not the Testing Policy's banned shape
  because that hard line is about other WL engines and rum's own language
  behavior, not about Rust-language facts checked with the toolchain that
  already builds the project.

- **CLOSED by brief-531 (2026-09-23).** Nine option call sites covering 22
  builtins lost their continuation (opened by brief-530). A delayed option whose
  right-hand side suspends used to complete the whole builtin call with the
  option's own value; the eight string-search sites now resume correctly, and
  the rest do not. Their option loop finishes, but dispatch treats its encoded
  list as the call's result, so `StringWrap` returns
  `InternalContinuationVec[Rule[PreserveNewlines, False], …]` where the queued
  path returns `"a\nb"` — an internal encoding escaping to user level, which
  is a worse shape than the wrong value it replaced even though both only
  occur on the already-broken suspension path. The sites are
  `strings.rs:78` (Uncompress, HtmlUnescape, TemplateApply),
  `structure_ops/ordering_sets.rs:528` (Union, Intersection, Complement),
  `predicates/collections.rs:353` (nine containment predicates),
  `strings/scripting/args.rs:153` (ParseArgs),
  `strings/scripting/url.rs:140,168` (UrlEncode, UrlDecode, UrlParse), and
  `strings/scripting/text_shape.rs:163,190,215` (StringWrap, StringIndent,
  AlignColumns). The contract they must satisfy is stated in the public
  wrapper's rustdoc. **Closed by construction**: the public wrapper now takes
  a typed `OptionReentry` descriptor and returns the *consumer's* result, so a
  caller can no longer obtain the option list at all, and omitting the
  descriptor is `error[E0061]` with a checked-in compile-pass control. The
  symbol-name `match` with its `unreachable!()` arm is deleted.

- **The retention checker lost reach on the region it exists to convert**
  (opened by brief-530, still open after brief-531). `OPTION_FRONTIER` fell
  from 17 resolved bodies / 5 files / 50 rejected edges to **4 / 3 / 17**,
  because C02's Rule construction
  now sits inside `resume_or_suspend!` and the restricted checker reports an
  unexpanded macro boundary (`RET003` at `options/core.rs:219-227`). The
  visibility loss was disclosed rather than silenced, but D2b cannot convert
  what the checker cannot see: either the macro gets a versioned, arm-specific
  expansion contract, or the construction moves back out of it.

- The **165 per-builtin `Incremental` declarations** remain undischarged.
  The session total is now maintained (`46126cab`), which removes the armed
  walk, but the declarations themselves still promise per-builtin charging
  that nothing does. Separate debt, separate line.
- The **unarmed cost of the maintained total is +0.965% / +3.239% / +1.018%**,
  recorded rather than tuned per the standing ruling. If it is ever revisited,
  the curve says where to look: **2.26 of ownvalue's 3.24 points come from two
  batches** — options and syntax information, then exceptions, scopes and
  sealed units — and the other ten add under one point between them.
- **`Enclose` leaks interner text per untagged call** (~117-127 proxy bytes,
  `kernel_data/control_data/confirmation.rs:118`). It dissolves when identity
  stops needing to be text, so it is downstream of the snapshot line rather
  than its own repair.
- **Termination is last-write-wins across script forms**
  (`session/session_eval.rs:328-338`), so a later form erases an earlier
  form's resource termination. Code shape confirmed by reading; **two director
  probes failed to construct the erasure**, so reachability is unproven —
  establish it before briefing a repair.
- **A doc-versus-code conflict**: the guide promises `General::memlim` on a
  consumed local `MemoryConstrained`
  (`docs/guide/resource-limits-and-sandbox.md:318`) and the handler does not
  emit it (`debug_tools/resources/constraints.rs:262`). Re-probe, then settle
  which side is wrong.
- **D7**: the audit's per-request-versus-per-session state-lifetime
  criticism, still unanswered, still owed its own brief.
- **What arming still does not guarantee**: the ceiling is a structural model,
  not an allocator bound, and `handle_alloc_error` aborts outside the panic
  boundary. "Guest code cannot kill the kernel" needs the host's RSS or
  process bound too — the audit's third criticism.

**Branch state.** `spike/brief-499-symbol-non-copy` holds the refuted A1
representation and is kept as its record; nothing else is outstanding. The
brief-514 and brief-522 spike branches were deleted after their work landed
in `d1e5de5c` and `46126cab`.

**Where the previous line ended.** The ownership branch (briefs 480-498) is closed — the
host-payload destruction contract, the accumulator-chain family and its
participation gate, the scanner's root set, and the four defects that turned up
along the way. **Next is the security-hardening mainline**, whose open items
are §Opened by the 2026-09-02 security audit (brief-413) and §Opened by that
work, not yet briefed: the process-global symbol interner the memory ceiling
cannot see, the strided ceiling sample, `rum-jupyter`'s unguarded JSON depth,
the dropped deadline across an input suspension, the call-stability cache
bounded by count rather than bytes, and the unexplained 1 668-byte abandonment
retention. The audit's three criticisms of the threat model itself are still
unanswered and are worth more than several of the findings.

## Active — post-v0.1 algorithmic audit

The P0 tranche and all of P1 are done. Two independent re-surveys (one by each
worker) then produced a worklist of about thirty items, grouped into batches;
the 2026-08-26 entry of [PROGRESS.md](./PROGRESS.md) records what each batch
found and closed.

**Closed since the last checkpoint**, in order:

- Loop and `Module`-initializer drivers made iterative. A 20 000-iteration
  `While` terminated the process; ordinary loops are ordinary code, so this was
  a correctness defect before it was anything else.
- YAML and TOML decoders bounded, matching `FromJson`. All three codecs now
  accept exactly 256 nested collections and refuse the 257th with a message
  naming the limit and the byte offset.
- Left-associative chain lowering, five steps (plan-330). All sixteen affected
  operators overflowed the stack at 20 000 links; they now fold iteratively,
  nine of them into n-ary calls, with a 4 096-link ceiling proved on a 2 MiB
  embedding stack through the complete round trip.
- Resource budgets made symmetric (plan in briefs 336-338). Padding builtins,
  `Fibonacci`, and multi-iterator products went uncharged while their siblings
  charged the same work. A regression this introduced — sequential repetition
  charged as simultaneous residency — was found and fixed in the same batch.
- User-callback ordering (plan-339). `Sort`, `SortBy` and `ReverseSort` with a
  comparator were quadratic on every input; `KeySort` and `KeySortBy`
  overflowed the stack at 110 and 100 keys. All five now run on one iterative
  resumable merge engine.
- The microbench itself never closed stdin, so a run either finished in four
  seconds or burned its timeout and produced a header-only timings file. Every
  A/B before `dd4bdefc` was exposed to that.

**Not briefed, and why:** the surveyed "Association 4-to-5" and "65 536-key"
cliffs no longer exist. Measured across eight operations, every 4-to-5 ratio
falls between 1.02x and 1.41x and the 65 535/65 536/65 537 boundary is flat.
They were closed by `c8d34876` and `dc5f6698` earlier in this sweep; the
worklist was simply stale. Association key lookup remains a linear scan, which
is a standing decision, not a defect.

**Re-probed 2026-08-26, all still present.** Numbers are from the release CLI
at `c18b01f9`, each probe carrying a liveness assertion:

1. **`ValueQ` clones the whole session per call.** 4.6 us with no definitions,
   90.6 us at 2 000, 333 us at 8 000, and 1.94 ms at 32 000 — a cost that
   depends on session history, not on the question asked. Confirmed against
   the wall clock: 100 calls in a 32 000-definition session move the process
   from 0.10 s to 0.30 s. A 420x spread that ends at two milliseconds is worth
   fixing cheaply, not worth a redesign.
2. **`Part` / `Extract` materialized every association entry per call — FIXED
   `4e045135`** (briefs 357-358). Per read: positional 12.61 ms -> 1.00 us at
   32 000 keys and exactly flat across width; keyed 12.27 ms -> 10.36 us.
   Allocations per read are now width-invariant (14.03 at both 4 000 and
   32 000 keys, against 12 083 and 96 101 before).

   **Leftover, deliberate:** keyed reads are sublinear rather than flat —
   3.7x for 8x width — because a wide association does not always hold the
   read-side index under cache admission pressure. That policy (`MAX_KEYS`,
   the retained-entry budget) is a standing decision and was explicitly out of
   scope. Revisit it only with the memory tradeoff stated, since the surveyed
   "65 536-key cliff" was already judged a fossil.
3. **`StringTrim` with a literal *list* leaves the fast path.** At 8 000
   characters: 342 us for `{"1", "2"}` against 28 us for the single literal
   `"1"` and 21 us for whitespace, and 1.45 ms against 77 us at 32 000. Linear,
   not quadratic — the defect is a 12x-to-18x constant, so the surveyed
   "quadratic suffix helper" reading is wrong even though the target is real.
4. **Symbol ordering allocates two `String`s per comparison**
   (`ordering_sets.rs:875`), 6.7x/7.0x/14.2x slower than integer ordering at
   4k/8k/16k. Not a mechanical swap: `Symbol::cmp` compares context then name,
   which is not the lexicographic order of the joined qualified name, so a
   repair has to argue output equivalence.
5. REPL re-parsing, the Jupyter busy-spin, and the gensym sweep — **all FIXED**
   (`21569ddb`, `3732615a`). The REPL examines only appended text and runs the
   full check once per form rather than once per line; the kernel sleeps its
   launcher's 10 ms idle interval instead of yielding; the sweep rearms at a
   watermark, so per-batch cost stops climbing (base +106% by the third batch,
   candidate -11%).

**Batch 7 is closed.** Every surveyed item was re-probed, and each was either
fixed or recorded with its reason. Two defects found along the way outrank
what the batch was looking for, and both have their own entries above:
`Do[f[i] = i, …]` installing one raw definition, and rule right-hand-side
`Condition` guards never being evaluated.

## Next up — scheduled 2026-08-28

Ordered after re-probing everything below. Silent defects outrank loud ones,
and the instrument outranks nothing here because the instrument turned out to
be fixed already.

1. **The assignment-target normalization family.** Planned by brief-376,
   which re-probed all three defects, widened each, and returned two repair
   families rather than one or three. **Dispatch order: brief-377 (family A)
   first, then family B in three steps.**

   - **Family A — evaluated `Part` selectors — COMPLETE 2026-08-28**
     (brief-377, `636e01ee`). D2 plus the `Part` branches of
     `AssociateTo`/`KeyDropFrom`: thirteen mutation heads, not eleven. The
     repair is a typed `EvaluatedPartSelectors` that assignment-target parsers
     require, so a future head cannot pass raw selectors without a compile
     error; the only constructors are exactly-once evaluation and a
     stable-positive-literal proof that keeps `AppendTo`'s incremental path
     zero-callback. Plain `Set` and association `Set` were routed through the
     same driver, removing their duplicated accumulators. The callback and
     return-order contract is now in `docs/guide/evaluation.md`.

     Gate note for family B, which touches the same files: the A/B run carried
     a **+1.81% whole-run offset** measured on the 102 untouched control
     labels, against +1.02% median on the ten affected ones. The raw split was
     86-of-112 labels slower, which reads as a regression until the controls
     are consulted. Name the controls before judging.

     Watch, not yet actionable: `association_data.rs` is now 32.2k and
     `definition_updates.rs` 26.3k, both under the 35k architecture-debt line
     but rising, and family B touches both. Evaluate a split when family B
     closes — not mid-repair, where it would contaminate a semantic change.
   - **Family B — definition-lhs lookup identity.** Planned by brief-378,
     which re-probed all four recorded defects and **found three more of the
     same family**, all reproduced by the director at `dfedb317`:

     - `Unset[f[i]]` with `i = 2` does not remove the definition `Set` just
       stored under `f[2]` — removal compares the raw lhs. The mirror image of
       B3/B4, and proof the invariant has to be enforced on the removal side
       too.
     - Value-table assignment (`DownValues[f] = {…}`) stores raw rule
       left-hand sides, so active positions become unreachable.
     - A held ground nested `Flat` call stores `fHF[hFlat[1, hFlat[2, 3]]]`,
       which **neither** `fHF[hFlat[1, 2, 3]]` nor the source form reaches.
       Candidate matching normalizes `Flat`/`Orderless` structurally while the
       ground index does not, so store identity needs match normalization on
       top of hold-aware evaluation.

     Seven defects, one preparation. Brief-378 also settled, with evidence:
     `SequenceHold` is not a hold attribute (it only prevents splicing);
     `HoldAllComplete` additionally blocks `Evaluate` and splicing;
     `Unevaluated` contributes its stripped inner expression to identity; and
     `HoldAllComplete`'s upvalue suppression is documented behavior, not a
     defect. **No existing test encodes any of the wrong outcomes**, so nothing
     has to be deleted or weakened.

     **Dispatch order (director, splitting plan-378's oversized step 1):**
     1a foundation — **COMPLETE 2026-08-28** (brief-379, `5698e8e5`);
     1b preparation plus `Set`/`SetDelayed` — **COMPLETE 2026-08-28** (brief-380, `aa441ff0`); 1c `Unset`
     and value tables — **COMPLETE 2026-08-28** (brief-381, `45c5aa33`);
     2 the association fallback — **COMPLETE 2026-08-28** (brief-382,
     `e90fcc62`); 3 tagged definitions,
     `TagUnset`, and final sealing — **COMPLETE 2026-08-28** (brief-383,
     `37a12479`). **The whole family is closed.** **Four steps, not the six first sketched**:
     plan-378's specification makes `Flat`/`Orderless` identity normalization
     step 7 of the preparation pipeline rather than a separate transform, so
     the held-ground defect closes in 1b alongside `Set`. The split is for
     gateability, not size: each step has to be independently verifiable and
     revertible, and 1a's criterion — an empty rum-vs-rum differential diff —
     only works while no consumer has moved.

     **Close-out.** Seven defects, four dispatches, and the repair is
     enforced rather than maintained: store and removal travel as tokens whose
     constructors are private to one child module, so a raw lhs from
     source-language evaluation is not expressible. Two non-preparing paths
     remain by design — static catalog installation and whole-state
     restoration. Both guarantees were calibrated, not assumed: forging a
     token from another module fails to compile (E0603/E0624), and adding one
     call to the trusted path makes the inventory test fail with the exact
     count. `definition_tags.rs` and its neighbours lost 232 lines net.

     **Open observation, no structural explanation, deliberately not fixed.**
     Step 3 moved `downvalue-guarded-decline-fallback` +7.47% and
     `downvalue-guarded-hit` +6.01%, above the 0.4-4.1% spread measured
     between same-semantics binaries differing only in layout — while the
     three hot lookup functions' disassembly is instruction-identical with
     addresses shifted `+0xa0`. The layout estimate is itself incomplete: the
     inert arms add code where this change removes 232 lines net, so they
     perturb in the opposite direction. Overall the run is clean
     (pure-computation controls +1.33%, everything else +0.74%). Revisit only
     with a profile that explains the mechanism; do not tune blind.

     **Two things 1a established for the steps that follow.**

     - *A differential diff must be calibrated before it is trusted.* Run
       against a pre-family-A binary the same probe reports a non-empty diff,
       messages included; an empty result therefore means something. Every
       later step reusing this gate must re-run that calibration, because a
       scanner that reports clean has to be shown capable of finding something
       — the frame-roots line lost a round to a regex silently truncated by a
       comma.
     - *To tell a layout effect from an algorithmic regression, build a third
       binary that changes only the layout.* Step 2 left two labels looking
       like regressions with non-overlapping three-run ranges
       (`downvalue-exact-miss` +4.94%, `downvalue-pattern-miss` +7.63%), while
       the affected function's disassembly was instruction-identical between
       binaries. An arm carrying nothing but an inert `#[inline(never)]` dead
       function in an unrelated file moved those same labels +2.60% and
       +5.10% — further than the real change did — which settles it: those
       rows are layout-sensitive and carry no information about the change.
       One extra build, and far more conclusive than arguing from medians.
     - *Best-of-2 measures a spread four to five times too small.* 1c's worker
       left two labels open as possible regressions —
       `matchq-integer-success`/`fail` at +6.34%/+6.72% against best-of-2
       spreads of 2.01%/2.96%. At best-of-3 those same labels' spreads are
       10.6% and 11.3%, and both movements fall inside. Nothing in 1c reaches
       `MatchQ`. Run three, report the range, and treat a two-run spread as a
       single random sample rather than a noise floor.
     - *Whether valid controls exist depends on the change's reach — do not
       generalize either way.* 1a sat on the common call path, where nothing
       is untouched. 1b sat on the assignment path, where the 28
       pure-computation labels are a genuine control group — and they decided
       the gate: they moved +3.82% at the median against the definition
       path's +2.10%, so the affected labels moved *less* than the controls.
       Judging 1b by 1a's rule (each label against its own spread, no
       controls) would have left three labels looking like regressions.
       Decide per step which of the two applies, and say which in the brief.
     - *Named control labels stop working once the change is on the common
       call path.* 1a's four controls were all fast labels from one tier and
       moved as a group (median -7.16% against a suite median of -1.83%),
       which would read as "targets are 4.5% slower" if taken as the baseline.
       `sameq-integer` and `string-repeat-small` themselves run through call
       preparation. For steps touching the shared path, judge on each label's
       own same-build spread and the suite distribution instead, and say so in
       the brief rather than naming controls that cannot be untouched.

     **Flake, first occurrence, recorded not explained away.** One workspace
     run during brief-379 failed
     `runtime::tests::association_write_index::repeated_replacement_writes_stay_indexed`
     (`association_write_index.rs:146`); a focused rerun and both later full
     runs passed, and the director's two full runs were green. The worker did
     not state whether it had changed code between the failure and the rerun,
     so flake versus fixed-in-passing is unresolved. Worth noting: this test
     asserts pointer identity (`own_args_ptr_key`) and lookup counters, and
     the other open flake — `has_valid_call_stability_proof_for_key`,
     `evaluation_cache/tests.rs:374` — asserts a stability proof in the same
     association write path. Two identity-shaped assertions in one area may be
     one cause. Same standing agreement: brief on a third occurrence.

   **The contract, settled 2026-08-28.** A definition's lhs arguments are
   prepared according to the head's hold attributes: non-held positions are
   fully evaluated, including a delayed own-value; held positions keep their
   raw symbols. **The invariant is bidirectional — the identity a definition is
   stored under and the identity a lookup constructs must come from the same
   preparation.** rum currently violates it in both directions, which is why
   family B covers both:

   - Not normalizing where it should: `AssociateTo[af[ii], …]` storing
     `af[ii]`, `TagSet`/`UpSet` storing a raw lhs, and `fD[iD] = 8` with
     `iD := 1 + 1` storing `HoldPattern[fD[1 + 1]]` so `fD[2]` cannot reach it.
   - Normalizing where it should not: with `SetAttributes[fH, HoldFirst]` and
     `xH = 5`, `fH[xH] = 9` stores `HoldPattern[fH[5]]`, so the natural
     `fH[xH]` lookup misses it. This one is the opposite direction, and
     covering only the first direction would fix three defects and hide it —
     the same shape as the `Block` frame's bare `Symbol` in the frame-roots
     line.

   `TagUnset` is in family B's ripple: if tagged definitions normalize, removal
   must use the same preparation.

   **Ruling (user, 2026-08-28): keep rum's `UpSet::nosym`.** Once `m[ww] ^= 22`
   with `ww = 5` normalizes to `m[5]`, no symbol in the lhs can serve as a tag,
   and rum already reports exactly that for `m2[7] ^= 33`. The wider WL family
   reaches a differently-shaped diagnostic there; rum keeps the more direct one
   and stays internally consistent, as it did when it kept its precise
   `ValueQ[f[2]]` answer. Design input only — no engine comparison enters the
   repository.

   **Retired by evidence: the "raw rules become reachable after `Clear`"
   argument.** Plan-376 recorded it as a reason a repair must diagnose rather
   than refuse. It describes rum's own raw-store behavior, not a semantics
   worth keeping: under the settled contract the rule is stored ready for
   lookup and is reachable immediately. The "preserve raw plus a warning"
   option, and the diagnostic question that depended on it, are both closed.
2. **The `rum-jupyter` ZeroMQ ordering flake — CLOSED 2026-08-28** (brief-384,
   `5f9a44ab`). Baseline 21 of 30 release runs failing on the director's host,
   0 of 30 after. Diagnosis follows. It reproduces on demand in **release** mode: thirty runs of
   `cargo test -p rum-jupyter zeromq::tests::native --release` fail 22 times,
   always the same single test
   (`native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages`,
   `control.rs:166`, `left: Channel(Shell)` against `right: Channel(Control)`).

   The test sends a shutdown on control, sleeps, sends an execute on shell, and
   asserts the *next* event is `Channel(Control)`. The poller's priority is
   correct — `PriorityKernelSocketPoller` lists `Control` first in every branch
   (`socket.rs:191-229`) — but it chooses among channels **already pending**,
   and native transport buffers on background workers, so "sent first" is not
   "buffered first". The test asserts a timing property while the
   implementation promises a priority one. It is the only place with this
   shape: `wait_for_next_socket_event` is used exactly once in the suite; the
   other twenty-one waits loop until their expected event and are
   order-insensitive.

   **Two things to carry forward.**

   *The defect was environment-dependent, and a stop-loss demanding a
   reproduction made the work unsatisfiable.* The first dispatch of brief-384
   correctly stopped: 0 failures in 30 runs in the sandbox, against ~70% on the
   director's host, with `--test-threads` ruled out as the variable (6/10,
   3/10, 7/10, 4/10 at 1, 2, 4, 9 threads — serial still fails, so the race is
   inside one test). The fix was to make acceptance a **structural argument** —
   why the assertion cannot fail on either arrival order — with run counts
   demoted to corroboration and the director verifying where it reproduces.
   When a defect may not reproduce for the worker, do not gate the work on
   reproducing it; gate it on an argument, and verify separately.

   *The test gate runs debug, and this defect is nearly invisible there.* Three years of sightings were all
   incidental workspace runs; this line's own six-plus workspace runs never hit
   it. Where a defect is timing-shaped, a release-mode repetition run is a
   different instrument from the debug gate, not a slower version of it.
**Batch 7's performance items are all landed** — `ValueQ` (`e67715f1`),
association `Part`/`Extract` (`4e045135`), `StringTrim` (`ada528c4`), symbol
ordering (`64f6200f`), the gensym sweep (`3732615a`), and the REPL/Jupyter
spin (`21569ddb`). An earlier revision of this section listed them as still
scheduled; that was a stale entry written without checking, the same failure
this line keeps finding elsewhere.

Not scheduled, and why: the call-stability-proof flake stands at one recorded
occurrence against the same three-occurrence agreement, and association keyed
reads are sublinear by a standing cache-admission decision that needs its
memory tradeoff stated before it can be reopened.

## Opened by batch 7, not yet scheduled

- **Gensyms held only by an opaque continuation frame — CLOSED 2026-08-28**
  (briefs 364-375, `d13aa27f` through `4e4c229f`). Four defects, not the two
  originally recorded: the design brief's re-probe added a matcher binding
  table losing its bindings, and a `Block` frame holding a bare `Symbol` that
  *leaked* on the next sweep — the opposite direction from the others, and
  proof that covering only `Expr` would have fixed three and hidden the
  fourth. All four are verified closed against plan-364's original probes,
  which assert the old buggy values and therefore now fail by asserting the
  defect.

  The fix is enforced rather than maintained. `SuspensionFrame: FrameRootTrace`
  makes a missing traversal a compile error, the frame queue is private so
  there is one insertion route, and a repository invariant test bans the
  deleted APIs and any handwritten traversal outside a two-entry allowlist
  (`RandomState`, `SyntaxError` — both structurally incapable of holding a
  root). Frames are `Arc`-rooted in two Session slots so the executing frame
  stays visible to a sweep, and one traversal now feeds both the root walk and
  the memory proxy, which used to charge a flat 16 bytes per frame.

  It also removed ordinary-path waste: a closure captured its state before the
  suspend/resume split, so a width-2003 `Switch` paid 63.8 KiB for a capture it
  never used. That width slope is now constant-size noise.
- **The `rum-jupyter` native ZeroMQ ordering flake has now appeared three
  times** (briefs 165, 242, and a workspace run during brief-362). Each time it
  passes in isolation — three isolated runs and two further workspace runs
  after the third occurrence were all green. The standing agreement was to
  brief it on a third occurrence; it is unrelated to the 10 ms idle sleep,
  which landed after the flake's first two appearances.
- **The call-stability-proof flake — CLOSED 2026-08-31** (briefs 387-388,
  `232ff567`). **All three recorded sightings were one cause.** The repair is
  in the tests, and removes the collision by construction: `StabilityBloom`
  compares its regular and generated lanes independently, so an
  identity-bearing target paired with an Association of `System` symbols and
  integer keys makes `intersects` reduce to `(0 & X) || (Y & 0)` for any
  interner order. Semantic assertions stayed on ordinary symbols; only the
  optimization assertions moved. Verified by sweeping preintern counts 0..256
  at 257/257 for both tests, after first reproducing the failures at 36 and 142
  to calibrate the harness.

  The diagnosis follows, and is worth keeping for the method as much as the
  finding.

  `symbol_stability_bloom_bit` derives one of 128 bits from
  `symbol.interned_id()`, assigned by the process-global `SYMBOL_INTERNER` that
  parallel tests share. In both failures the target's rewritten-dirty bit and
  ``System`Rule``'s dependency bit are the same `0x2000`. A Bloom overlap
  yields `MissDirty`, the proof is not cached, the direct association update
  declines, and the generic renderer allocates new args — which is why one
  sighting shows a missing proof and the other a moved pointer key and rebuilt
  index. Same cause, two downstream symptoms.

  **The flake is now deterministic**, which is the finding worth keeping:
  interning 36 unrelated symbols before the direct test, or 142 before the
  indexed one, fails every time. Neutralizing `StabilityBloom::intersects`
  makes both pass; forcing it true makes both fail harder.

  **The contract, settled from the code: the stability predicate is one-sided.**
  `true`/`Some(proof)` is authoritative; `false`/`None` may be conservative. A
  Bloom collision is a fast-path miss, not an "unstable" verdict — every
  consumer re-does the work rather than trusting a negative, and
  `association_lvalue.rs:53` explicitly anticipates a lost proof and rebuilds.
  **The implementation is correct; both tests assert cache residency and
  pointer identity that the mechanism never promised.** Same shape as the
  ZeroMQ flake closed in brief-384.

  **Director's ruling: pointer and index continuity are an opportunistic
  optimization, not a guaranteed invariant.** The recovery path exists
  precisely because a proof can vanish under a size-cap clear; guaranteeing
  continuity would require exact dependency disambiguation on hot-path
  machinery, and buys nothing semantically since the fallback produces the same
  result. The repair is therefore in the tests — but by making the collision
  impossible by construction, not by relaxing the assertions: a target whose
  dirty lane is structurally disjoint from its proof dependencies. Widening the
  Bloom is rejected outright; it lowers a probability without establishing a
  guarantee.

- **Superseded by the entry above; kept for the sighting record.** One
  workspace run during brief-385 failed
  `association_lvalue_direct_update_keeps_args_identity_and_reinserts_proof`
  again, on `valid_call_stability_proof(&session, &key).is_some()`; it passed
  alone and on the rerun, and brief-385 touched neither `evaluation_cache` nor
  `association_lvalue`. Two of three under the standing agreement. Note also
  the `association_write_index` failure recorded during brief-379, which was
  never resolved as flake-or-fixed and asserts pointer identity in the same
  write path — if that one is the same cause, the count is already three.
  Decide that before treating the next sighting as the first of a new pair.

- **First occurrence of a call-stability-proof flake, recorded not explained
  away.** One `cargo test -p rum-runtime` run during brief-370 failed
  `association_lvalue_direct_update_keeps_args_identity_and_reinserts_proof`
  on its `has_valid_call_stability_proof_for_key` assertion
  (`evaluation_cache/tests.rs:374`). The worker attributed it to the 128-bit
  stability Bloom's sensitivity to parallel symbol-interning order. It did not
  reproduce in thirteen director attempts — eight of the test alone, five of
  the full 854-test parallel suite — so the attribution is neither confirmed
  nor refuted. brief-370 changed neither `evaluation_cache` nor
  `association_lvalue`, so it is pre-existing. Same standing agreement as the
  Jupyter flake: brief it on a third occurrence.

**Re-probe each before briefing it.** Two of this batch's items turned out to
be already fixed, and one survey finding — "`SameTest` never exits early" —
was an artifact of an invalid probe: rum's custom equivalence test is a
positional argument, not a `SameTest` option, so the option form silently
compared nothing. Findings age, and probes lie.

**Rule right-hand-side `Condition` guards — COMPLETE 2026-08-26 (briefs
347-354, `5ab6afc5` through `eb926bff`).** All eight candidate-selecting lanes
evaluate their guards: `DownValues`, `UpValues`, `SubValues`, `DefaultValues`,
generic rewrite rules, `ReplacePart`, string rules, and `OwnValues`. One
substrate, one accept rule (`True` accepts, anything else declines), and a
decline is indistinguishable from the rule not matching. Per-lane ordering
contracts are documented in `docs/guide/patterns-and-rules.md`; a single test
exercises a guarded rule in all eight lanes so a future change cannot break
one silently. The unguarded path takes no new branch, allocation or clone in
any lane.

Known limitation, deliberate: one symbol keeps a single own-value slot, so
multiple guarded own-values do not coexist — a later definition replaces the
earlier one.

**The line's own lesson, worth more than the feature.** After six green gates
a guarded `DownValue` still stopped enforcing its guard the moment anything
bumped the attribute revision — any `SetAttributes` in the session, or a
loader-snapshot restore such as `ValueQ`. Every gate had asserted the guard
*before* the trigger and never after one, and the whole-line regression test
had the same shape, so none of them could see it. Brief-354 fixed it
structurally, by putting the barrier in the single rematch API rather than
patching another call site, and the regression test now asserts after a bump.
**For a feature that caches or compiles anything derived from a definition,
the test has to act after the invalidation, not only before it.**

The original diagnosis follows.

**Rule right-hand-side `Condition` guards are not implemented (brief-347,
diagnosed 2026-08-26).** `f[x_] := rhs /; test` is a standard idiom and rum
silently does something else: the guard is never evaluated, a false guard never
rejects the rule, and the `Condition` leaks into the result — `b[x_] := x^2 /;
x > 0` makes `b[3]` return `9 /; True` and `b[-3]` return `9 /; False`. Dedup
compares left-hand sides only, so a rule differing solely by its guard is
treated as a redefinition of the unguarded one and disappears. `Rule` and
`RuleDelayed` carry the same hole through the replacement system.

Parsing and lowering are correct; `DownValue` and `RewriteRule` simply have no
guard field, so the whole `Condition` is stored as the body and substituted as
one. The guard in *pattern* position works correctly and is the model to
follow. No existing test encodes the wrong behavior.

Every candidate-selecting lane is affected: `DownValues`, `UpValues`,
`SubValues`, `DefaultValues`, `OwnValues`, `ReplaceList`, `ReplacePart`,
string rules, and the exception-handler scan. Director's rulings on the plan:
only exact `True` accepts and anything else declines (matching the pattern-side
guard); nested guards flatten to an ordered sequence evaluated left to right;
a guard shares its body's `OptionValue` scope; identity is syntactic after
alpha-canonicalization, so `Condition[body, True]` is a distinct candidate from
an unguarded rule; `Unset` removes all guard variants of an lhs; a bare
`Condition` expression stays inert; `NValues` and `Dispatch` are out of scope.
**Guarded `OwnValues` are deliberately narrowed**: no storage-representation
change and no `symbol.rs` hot-path cost — check the guard only when the stored
value is itself a `Condition`, and record multiple guarded own-values for one
symbol as unsupported. This work precedes the `ValueQ` line below, which
depends on it: `ValueQ[b[-3]]` cannot be right until `b[-3]` evaluates right.

**`ValueQ` — COMPLETE 2026-08-26 (briefs 345, 346, 355, 356; `b85569c2`
and `e67715f1`).** Both tiers landed. The trial no longer commits side
effects: PRNG state is restored, the host print sink is suppressed for its
duration, and `FileWrite`/`Process` are refused, with restoration on every
exit path including panic. Proven-stable targets — `ValueQ[undefinedSymbol]`
above all — answer without evaluating at all: 332.6 us to 0.76 us per call at
8 000 definitions, and no longer scaling with session history. rum's precise
answers are unchanged throughout (`f[1] = 3; ValueQ[f[2]]` is still `False`),
with one deliberate exception: `ValueQ[DumpSave[path, x]]` now answers `False`
because the refused capability fails the trial.

The reasoning that got there follows.

**`ValueQ`: precision and freedom from side effects are the hard
requirements; not evaluating is not — user ruling 2026-08-26.** The first
ruling was "answer by lookup, never evaluate", but brief-346 proved the three
cannot hold together: deciding whether a `Condition` or `PatternTest` rule
applies *requires* evaluating user code, so a non-evaluating answer must be
imprecise on the very common `f[x_?NumericQ]` and `f[x_] /; test` shapes.
Since not-evaluating was only ever a means to the other two, it is the one
that gives way.

The shape is therefore two-tiered:

1. **Provable shapes answer by lookup, without evaluating.** rum's precise
   answers are kept — `f[1] = 3; ValueQ[f[2]]` stays `False`. A coarser "does
   the head have any definition" rule was considered and rejected as too
   surprising.
2. **Undecidable shapes are evaluated in an effect-suppressed environment**,
   so the answer stays exact while nothing escapes the trial.

Tier 2 comes first: it closes the contract breaches below without changing any
answer. Tier 1 is then pure performance.

The three breaches are what the ruling closes (brief-345, all reproduced by
the director). The generated reference — normative under the testing policy —
says `ValueQ` tests "by trial evaluation without committing side effects".
Session-local state does roll back (`ValueQ[yy = 7]` leaves `yy` undefined),
but three effects escape the trial:

- **The deterministic PRNG advances.** `SeedRandom[123]; ValueQ[RandomInteger[
  1000000]]` consumes the first draw, so the next draw returns what should
  have been the second. `Capability::Pure` cannot serve as an effect proof
  here — `RandomInteger` and `SeedRandom` are deliberately classified `Pure`.
- **`Print` reaches the host sink.** Output streams immediately when a sink is
  installed; the existing rollback test only covers the buffered no-sink case.
- **`FileWrite` commits.** `ValueQ[DumpSave[path, x]]` writes the file, and no
  in-memory snapshot can undo it.

The open question is the accuracy of the lookup, not the contract. rum already
has a machine that answers "would evaluating this change it" — the evaluation
stability check the evaluator uses to skip stable expressions — but it is
deliberately conservative, and its conservative direction ("might change")
becomes a wrong `True` here. How precise it is, which shapes need more, and
what an undecidable shape should answer are the questions to settle.

**Found while re-probing, and more serious than the list above: iterator
variables were invisible on an assignment lhs.** `Do[f[i] = i, {i, 1, 3}]`
installed one `HoldPattern[f[i]] :> 3` and said nothing — the standard table
fill and memoization idiom. Fixed in `4f5b85f1` (briefs 342-343): a clean
iterator symbol lives in a dynamic own-value frame, ordinary evaluation
resolves definitions then that frame, and the held-lhs normalizer stopped
after definitions. It now takes the same third step in the same order. The
`Do[With[{k = i}, f[k] = k], …]` workaround in the test suite is gone.

**The same matrix exposed three defects that have nothing to do with
iterators** — all reproduce with a plain global symbol. **Re-probed
2026-08-28 against the release CLI at `65c03870`: all three are still
present**, each probe carrying a liveness assertion and a working control.
They are one family — the assignment target is not normalized before the
definition is installed — but by three different mechanisms, so whether
they are one repair or three is a question for the design brief.

- **`AssociateTo`/`KeyDropFrom` write back to the raw target.** `ii = 2;
  af[2] = <|"a" -> 1|>; AssociateTo[af[ii], "k" -> 7]` reads `af[2]`
  correctly, applies the update correctly, then installs the result under a
  second definition `HoldPattern[af[ii]] :> <|"a" -> 1, "k" -> 7|>` and
  leaves `af[2]` untouched. Silent — no message. `KeyDropFrom` behaves the
  same way. Mechanism: the fallback path evaluates `target_expr` to read
  the current value but hands the *unevaluated* `target_expr` to
  `store_association_lvalue_update`
  (`crates/rum-stdlib/src/associations/mutation.rs:196-225`).
- **The compound assignment heads do not evaluate a `Part` selector.**
  `kk = 1; lst[[kk]] += 5` reports `Part::pkspec1` and changes nothing,
  while `lst[[kk]] = 99` works and `lst[[1]] += 5` works — a clean
  three-point control that isolates the defect to compound-assignment ×
  symbol-selector. Confirmed on nine heads (`AddTo`, `SubtractFrom`,
  `TimesBy`, `DivideBy`, `Increment`, `Decrement`, `PreIncrement`,
  `PreDecrement`, `AppendTo`). Loud, so it ranks below the silent ones.
  **The repair model is one call site**: `part_assignment.rs:228` passes
  `evaluated_selectors` to
  `exact_position_paths_from_part_target_selector_exprs`, while
  `definition_updates.rs:389` passes the raw `selector_exprs` to the same
  function.
- **`TagSet`/`UpSet` never normalize lhs arguments, and the definition they
  install is unreachable.** Worse than recorded: `tg = 5; h /: h[tag1, tg] =
  7` stores `HoldPattern[h[tag1, tg]] :> 7`, so `h[tag1, 5]` does not match
  and neither does `h[tag1, tg]` — the lhs evaluates to `h[tag1, 5]` before
  lookup, so the definition can never fire from any input. Same for the true
  up-value form (`vv = 5; g /: k[g, vv] = 11` leaves `k[g, 5]` unevaluated)
  and the tagless form (`ww = 5; m[ww] ^= 22`). A clean symbol works
  (`m3[qq] ^= 44` then `m3[qq]` is 44), and `m2[7] ^= 33` correctly reports
  `UpSet::nosym`. Silent in every failing case. Whether the lhs *should*
  normalize is still the standing contract question — but "writes a
  definition that nothing can reach, without a message" is a defect under
  either answer, so decide the contract, then fix at least the silence.

**The microbench scale ceiling — FIXED `d9c713e1` (brief-344), verified
2026-08-28.** Both halves are closed: the harness has headroom, and it fails
loudly when it does not. `--scale 5` at the default `--reps 7` now completes
and writes 112 measured labels with an empty stderr, and the scale is proved
live — 100 of the 112 labels carry exactly 5x the scale-1 iteration count,
the other 12 being fixed-width labels that scale does not touch. The script
now exits non-zero on both failure shapes (`core-language-microbench.sh:1084`
for a non-zero engine exit, `:1090` for zero measurement rows), so the
header-only `timings.tsv` can no longer pass for a run. The record above of a
scale-1 ceiling was written before brief-344 and was never retired.

Note for whoever re-checks this: `--reps 1` is not a valid probe of the
ceiling. The repetition loop is the layer that nests around each label's own
loop, so lowering it hides the very defect being tested.

**Deferred, with evidence, from the second sweep:**

- **Error spans — FIXED 2026-08-31** (brief-385, `a0c6afe3`). Spans now leave
  the parser in file coordinates, so `--file`, stdin scripts, host script
  requests, `Get` and `Needs` all report the right line and column, and `Get`
  has a location at all. Re-probing found it wider than recorded: both the line
  and the column were wrong, not just the column, and `a == (b == c)`-style
  right-hand cases were affected too. The suite's blind spot was the cause —
  every pinned location was `1:x`, where form and file coordinates coincide.
- **A trailing-operator form still re-parses per line.** The linear splitter
  drives the parser form by form, but a form whose delimiters are balanced and
  whose expression is unfinished (`1 +\n2 +\n3`) cannot be predicted from
  balance alone. Such forms are short in practice; recorded rather than hidden.
- **Pattern- and key-driven `ReplacePart` rules stay sequential.** Merging them
  into one pass would reorder user-visible evaluation or change association-key
  semantics. Their outer walk is iterative, so wide rule lists no longer
  consume a native frame per rule.
- **Comparison chains — FIXED 2026-08-31** (brief-386, `8b93dffb`). This entry
  was wrong in both directions: `a == (b == c)` flattened too, so the
  right-hand cases were not already fine, and the shared AST was not a fact
  about chains but a consequence of parsing comparisons as associative
  binaries. **These operators have no associativity.** Chains are now collected
  in the parser, so a parenthesised operand is a boundary by construction:
  `(a==b)==c` is `Equal[Equal[a,b],c]` and `(1==1)==1` is `Equal[True, 1]`.

  The same repair closed a second defect in the same machinery: mixed chains
  lowered to `And`, repeating the middle operand, so it evaluated twice and the
  chain short-circuited. They now lower to `Inequality[e1, op1, e2, op2, …]`
  (user ruling 2026-08-30), which carries exactly `{Protected}` — no hold
  attribute, which is why each operand evaluates exactly once and nothing is
  skipped. The missing short-circuit is deliberate: undetermined comparisons
  are routine, and a chain retained symbolically needs every part in evaluated
  form, so early exit has no well-defined meaning. A short-circuiting variant,
  if ever wanted, belongs in an option or a rum-native sibling.
- **Product propagation is decided from construct nesting, not data flow.**
  `Table[Length[Table[...1000...]], {i,1,200}]` is refused although the outer
  `Table` retains only integers. Distinguishing that statically is not
  something the iteration seam can do, and the safe side of the line is the one
  that keeps `Table[Table[...]]` refused.

Every item remains subject to the performance gate in
[docs/testing-strategy.md](./docs/testing-strategy.md): safe Rust only, base
versus candidate timing plus counters and memory, and reject or report
regressions and space-for-time tradeoffs.

## The suspension and cancellation line — CLOSED (briefs 397-411)

Opened by the first release-profile full-test run on the Linux host, which
produced two `rum-jupyter` pump failures that the debug gate never showed.
brief-397 diagnosed them as one invalid test assumption and one real
host-contract defect; the repair of the second uncovered a third defect, and
that one turned out to be the largest.

- **Cancellation was erased by the evaluation it was meant to stop.**
  `begin_top_level_interrupt_budget` cleared a session-global flag as
  `eval_request`'s first act. Repaired by briefs 404 and 405 as request-scoped,
  generation-qualified cancellation: a non-`Clone` accepted-request ticket, a
  phase word encoding `(generation << 1) | cancelled`, compare-exchange
  cancellation that cannot reach a later generation, and completion as the
  linearization point. The legacy `AtomicBool` lane is deleted, so the
  suppress-or-synchronize conditional that existed only because two mechanisms
  coexisted is gone with it.

- **Abandoning a suspension leaked every scope its frames owned.** Found while
  writing brief-400's cancellation test, split out as a pre-existing defect,
  and confirmed reachable through public `rum-host` API alone. Repaired by
  briefs 402 and 403: a closed abandonment protocol every one of the 545 named
  frames must classify, enforced by a supertrait with no default body rather
  than by a source scan — a distinction that matters because `SuspensionFrame`
  is `pub` and a scan cannot reach code that does not exist yet.

- **Overlapping public requests are rejected** (user ruling). Superseding a
  suspended request had been an accident of consuming queued input; it is now
  refused, and remains unimplemented as an explicit operation.

- **The jupyter timing-assumption class**, closed by briefs 406-411 — see the
  entry below.

brief-400 is superseded rather than resumed: brief-401 refuted its repair
(driving saved frames with `Err(Cancelled)` is not an unwind, because ordinary
frames consume errors and run work), and briefs 402-403 had rewritten the
files its stash touched. `stash@{0}` is kept as reference material.

## The 2026-08-31 consistency review — CLOSED (briefs 389-396)

Four reviews of the work from briefs 364-388, run in independent parts so an
interrupted dispatch would lose at most one. Eight findings, three of them
high, all repaired by briefs 393-396. **Every repair replaced a convention with
a structure**: a list of protected pattern wrappers became one registry shared
with the matcher; assertions resting on interner order moved to a construction
where the two Bloom lanes cannot intersect; a public installer that trusted its
caller now demands a typed left-hand side; and a wrapper claiming two
constructors left the continuation protocol that offered a third.

**Two of the findings were corrections to the briefs themselves**, which was
worth more than some of what was asked for: the frame-root allowlist is an
inventory of handwritten traversals rather than a two-entry exemption list, and
the frame queue is the single insertion route for ordinary code rather than
literally. `CLAUDE.md` was wrong on both and has been corrected.

**One was a gate miss of mine.** brief-388 repaired the two assertions that had
been seen failing; five more rested on the same mechanism, because the
diagnosis and the repair both examined the recorded sightings and the recorded
sightings were two.

The findings and their repairs follow.

## Opened by the 2026-08-31 consistency review (briefs 389-392)

- **The assertion class — CLOSED 2026-08-31** (brief-394, `HEAD`). The
  review's five sites were the complete remaining set; each group now splits
  semantic coverage on ordinary symbols from optimization coverage on a
  lane-disjoint target, with no assertion deleted. Four groups swept 0..256 at
  257/257. Two assertions could not be made to fail under any preintern count
  and were reported rather than changed. The original finding follows.

- **HIGH — brief-388 fixed two instances, not the class.** Five further
  assertions still pair ordinary symbols with ordinary dependencies, so they sit
  on one Bloom lane and can be flipped by a legal false positive exactly as the
  two closed flakes were: `evaluation_cache/tests.rs:105`, `:472`, `:499`,
  `association_write_index.rs:84`, and `association_write_recovery.rs:40`.
  Director-confirmed. **This is a gate miss of mine**: I verified that the two
  known tests were repaired and that the lane argument held, and never asked how
  many other assertions rest on the same mechanism. The diagnosis brief had the
  same shape — it examined the two recorded sightings because those were the
  sightings. Sweep the class, then decide per assertion whether it wants the
  lane-disjoint construction or should stop asserting cache residency at all.

- **Request-scoped cancellation — CLOSED 2026-09-02** (brief-404). Cancellation
  is generation-qualified, survives its request prologue, and stale handles are
  inert after completion or reset. The lifecycle, crate-local host, and runtime
  tests use channel handshakes. Only the ignored stability loop at
  `crates/rum-host/tests/stress.rs:124-144` still uses a 25 ms sleep; replacing
  that test synchronization remains separate cleanup.

- **The rum-jupyter timing-assumption class — CLOSED 2026-09-02** (briefs
  406-411). The entry that stood here named four IOPub tests and a 50 ms delay;
  brief-406's inventory found the class was much larger and had three unrelated
  roots, and two of its own citations were already stale. What actually
  existed:

  - *In-process worker tests* asserted immediate readiness and turned a shared
    two-second helper into a semantic requirement, though the production API
    exposes only a nonblocking poll and promises no latency. Closed by
    brief-407 with instance-scoped start/finish signals; a probe had first
    established that a yield is not enough and only a real happens-before edge
    would do.
  - *The native IOPub subscription race* was real, but the repository's
    `acknowledge_iopub_subscription` was not a barrier for it — it did a
    heartbeat round trip over an independent Req/Rep socket. Closed by
    brief-411 with a sentinel that travels the backend's own PUB path, plus
    parent-scoped frame collection so "the next four frames" stopped being an
    assumption.
  - *The native `basic` tests* were a third family the old entry could not
    reach: they open Dealer/Router and Req/Rep only, with no `SubSocket`, so
    PUB/SUB was never their mechanism. Closed by brief-409 along with the two
    negative assertions that tried to prove absence by waiting 200 ms.
  - *An ephemeral-port TOCTOU* nobody was looking for: the helper reserved five
    ports, read them, dropped every listener, and only then let the backend
    bind. Closed by brief-408 by recovering the resolved endpoint the pinned
    library already returns and that the wrapper was discarding.

  **The sandbox note that stood here is now enforced rather than remembered.**
  A skipped native test used to be reported by libtest as `ok`, which is how
  brief-384 recorded a 0-of-30 result that measured nothing. Without
  `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` a host that cannot bind loopback now fails all
  nine loudly. Verification of native behaviour still belongs on a
  loopback-capable host: brief-409 passed the worker's gate and broke all nine
  native tests on the director's, and brief-410 added socket-free harness
  coverage so that class is catchable in a sandbox that cannot bind.

- **Comparison-chain coverage — CLOSED 2026-08-31** (brief-396, `aa8644cf`).
  Both gaps are covered: `crates/rum-stdlib/src/tests/logic/inequality.rs:31-35`
  lowers a surface uniform-`Less` chain and keeps its n-ary head, and `:82-98`
  counts all three operands including the leftmost. The original finding
  follows.

  A uniform-relational lowering regression or a leftmost double evaluation
  would have escaped the suite: no test lowered a surface uniform-`Less` chain,
  and none counted the leftmost operand.

- **Pattern regions — FIXED 2026-08-31** (brief-393, `d423d2d3`). The list of
  protected wrappers is gone; a shared registry names every pattern head once
  and preparation, the matcher, `MatchQ`, options matching and substitution all
  consult it, so the sets cannot drift. It gathers heads that had been
  recognized in three separate files, which is why it carries `HoldPattern` and
  `OptionsPattern` beyond the seventeen the matcher file compared by name. The
  old-versus-new recognized-head diff lost nothing, and matcher behavior was
  checked by probe. Per-form treatment is now documented rather than implied.
  The original finding follows.

- **HIGH — preparation evaluates inside pattern regions it does not recognize.**
  The pattern-region policy enumerates the wrappers it protects and omits
  `Verbatim`, `PatternSequence`, `OrderlessPatternSequence`, `KeyValuePattern`,
  `Longest`, `Shortest`, and `OptionsPattern`, all of which the matcher
  supports. Director-reproduced: `typeAlias = Integer; fV[Verbatim[Blank[
  typeAlias]]] = 7` stores `fV[Verbatim[_Integer]]` — **`Verbatim` means
  verbatim, and its contents were evaluated** — and
  `fP[PatternSequence[sideCall[], b_]] = 8` runs `sideCall[]` once and stores
  its result. An ordinary binder `x_` is correctly left raw, so the policy's
  shape is right and only its list is short. **User-reachable, no external
  crate needed.** This is family B's own invariant failing on the forms nobody
  enumerated.

- **Published-surface boundaries — FIXED 2026-08-31** (brief-395). The
  installer now takes a typed lhs and the selector wrapper left the
  continuation protocol; an external crate attempting either construction fails
  to compile (E0061, E0277), reproduced independently by the director. Catalog
  installation measured -0.56%. The original findings follow.

- **HIGH — `install_system_down_value` stores a raw lhs from outside the
  crate.** It is `pub`, takes an arbitrary `Expr`, and mints the trusted token
  internally, so preparation never runs (`session_api.rs:225`,
  `prepared_store.rs:385`). `#[doc(hidden)]` does not restrain a downstream
  Rust caller. brief-391 compiled an external crate and installed
  `ExternalRaw391[i391]` with `i391 = 2`; the stored identity is unreachable by
  ordinary lookup — exactly the defect class family B closed, reintroduced
  through a public API. The sealing inventory test only scans in-repository
  callers (`definition_store_sealing.rs:130`) and structurally cannot see this.

- **`EvaluatedPartSelectors` can be forged through the kernel API.** brief-390
  compiled an external crate that built the wrapper from raw expressions via
  the exported continuation decoder (`kernel_api.rs:18` and `:100`,
  `evaluated_part_selectors.rs:68`) and reached the parser with it. The typed
  seam brief-377 introduced is therefore not the compiler-enforced
  two-constructor boundary it was described as. Impact today is limited — the
  forged value failed to parse, and only a kernel-API consumer could construct
  one — but the guarantee is the point of that design, so the export surface
  should be narrowed. Worth fixing; not urgent.

- **The selector order contract — CLOSED 2026-08-31** (brief-396, `aa8644cf`).
  `docs/guide/evaluation.md:172-179` now says the selectors evaluate exactly
  once *when a mutation reaches target selection*, and names the
  `AssociateTo`/`KeyDropFrom` validation branch explicitly. The original
  finding follows.

- **The selector order contract is stated unconditionally and is not.**
  `docs/guide/evaluation.md:168` says selectors evaluate exactly once, but
  `AssociateTo` returns before selector handling when its update specification
  is invalid (`mutation.rs:166`), so they evaluate zero times on that branch.
  **The code is right and the document is wrong**: the specification is an
  ordinary evaluated argument, and a call that fails validation should not
  produce partial selector side effects. Reword the contract to say the
  selectors evaluate once *when the mutation reaches target selection*.

- **The frame-queue guard reaches less far than claimed.** brief-389 found
  `insert_suspension_frame` public, the resume driver pushing directly, and the
  invariant exempting `suspension/core.rs` and test directories. No current
  path bypasses `SuspensionFrame`, so the traversal guarantee holds; a future
  non-macro insertion in either exempt area would not be caught. `CLAUDE.md`
  has been corrected; whether to narrow the exemption is open.

## Opened by the 2026-09-02 security audit (brief-413)

The first audit of this codebase permitted to say "attack". GLM ran it because
Codex's cybersecurity filter makes security review there impossible. Seven
findings; the two HIGH ones share a root — **bytes the memory model cannot
see**.

- **HIGH — the symbol interner is process-global, leaks by construction, and is
  invisible to the memory ceiling.** `symbol_interner()`
  (`crates/rum-core/src/expr.rs:47-55`) is a process-wide
  `OnceLock<Mutex<SymbolInterner>>`; `intern_text` (`:57-70`) `Box::leak`s every
  new name; there is no removal or clear API. `current_memory_proxy_bytes`
  (`runtime/debug_tools/resources/memory.rs:50`) walks session-owned structures
  only and never mentions the interner, so `eval_memory_limit` cannot see the
  growth. `HostSession::reset()` rebuilds the session and does not touch it.
  **Director-reproduced**: under a `{Pure}`-only policy, eight requests each
  interning 20 000 fresh symbols took RSS 5.6 MB → 57.5 MB monotonically, and
  `reset()` returned only to 47.0 MB. For a long-lived host — a jupyter kernel,
  an editor host, a per-request-session service — this is an exhaustion vector
  no documented kernel mechanism can close, shared across every tenant.

- **HIGH — the modelled memory ceiling is sampled every 64 evaluation points and
  can be overshot multiplicatively in between.**
  `ACTIVE_MEMORY_LIMIT_CHECK_STRIDE = 64`
  (`runtime/debug_tools/resources/constraints.rs:9`). Its own comment claims the
  interval is "bounded by the existing per-step element, integer, and iteration
  budgets" — but those bound *counts*, not the bytes of one multiplicative step,
  and `eval_string_join` (`rum-stdlib/src/strings/text/builtins.rs:13-42`) has no
  projected-byte precheck at all, unlike `StringRepeat` (`:664-668`) or `Power`.
  Reported: an 8 MB ceiling reached a 2.04 GB peak before terminating in band.
  A failed allocation is `handle_alloc_error`, which aborts outside the panic
  boundary. **Mechanism director-verified in source; the 2 GB overshoot is not
  director-reproduced.**

- **HIGH — the strided ceiling — FIXED 2026-09-22** (briefs 513-515,
  `d1e5de5c`).** Director-reproduced with a control: an 8 MB
  `MemoryConstrained` let `Nest[StringJoin[#,#]&, seed, 16]` complete at
  65 536 000 characters and ~358 MB peak RSS, while the same ceiling aborts a
  single large `StringRepeat`. **The audit's mechanism was wrong**:
  `StringJoin` does not amplify per call, so the `Preflight` growth assertion
  passes it correctly — and that assertion is a **test-only instrument**
  bypassed unless `RUM_TEST_BUILTIN_MEMORY_GROWTH` is set, so it never
  described production anyway. The overshoot is repeating a non-amplifying
  step between strided walks. brief-513 also corrected the director twice
  more: a proper `StringJoin` precheck *would* catch this example (projection
  is unstrided), and last-sample adaptive stride has no useful factor bound.

  The repair (option A, every armed checkpoint walks) catches the
  reproduction, with the falsifier watched failing when the stride is
  restored. **Default-path gate passed, director-established 2026-09-22:**
  instruction counts are flat (−0.000387% to +0.000948% over six calibrated
  workloads once the armed walk moved to a cold helper), and the timing
  panel's apparent regression is **indistinguishable from the panel compared
  with itself** — splitting the four base runs two ways gives 69/120 and
  72/120 positive medians with 28 and 30 labels consistently slower and a
  worst outlier of +18.43%, against the candidate's 64/120, 9, and +16.66%.
  `association-part-key-read-width4000`, one of the flagged labels, moves
  +18.43% between two runs of the *identical* binary.

  **Landed.** Every armed checkpoint now walks; the stride constant and its
  false comment are gone, replaced by the guarantee the code actually makes
  including its limitation. Falsifier director-verified both ways — with the
  stride restored it fails with "repeated string growth escaped its
  200000-byte ceiling". All 61 inventory changes are location shifts,
  verified including the line numbers embedded in symbol paths and `ends at`
  spans. Gate 2 493 passed, 0 failed, 57 ignored.

  **Still owed, deliberately:** the armed-path cost curve was measured and
  reported but never judged — the walk is O(session), so an armed workload
  now pays on every checkpoint instead of one in 64. That is a director
  decision about whether armed sessions need a cheaper accounting route, and
  it is the entry point to the `Incremental` contract, which `TODO.md` still
  records as unbuilt.

- **MEDIUM-HIGH — `rum-jupyter`'s hand-written JSON parser has no depth guard.**
  `json_support.rs` contains zero occurrences of depth/limit/MAX_, and
  `parse_value`/`parse_object`/`parse_array` (`:61,86,126`) are mutually
  recursive. Reported: 50 000 levels of nesting aborts the kernel process with a
  stack overflow, outside the panic boundary. Reachable unauthenticated when the
  signature policy is `Disabled` — see the fail-open finding below. The kernel's
  own parser has a four-level depth guard; the discipline did not reach the
  adapter. **Not director-reproduced** (it aborts a process).

- **MEDIUM — `eval_with_timeout`'s deadline is silently dropped across an input
  suspension — FIXED 2026-09-22** (briefs 510-511, `40a429df`). Re-probed
  before briefing and still present verbatim three weeks on. The repair
  needed no new contract: the rustdoc three lines above already promised the
  timeout is cleared after the request completes or fails, and a suspension
  is neither. The deadline now stays **absolute** across the suspension,
  matching the kernel — `TimeConstrained` computes its deadline once as an
  `Instant` and nothing anywhere recomputes or pauses it — so time spent
  waiting for host input counts toward the budget, which
  `docs/guide/embedding.md` now states along with the shape of a timeout
  during a resumed segment. Ownership is a private marker released at the
  `guard` boundary, the shared exit of every guest-code entrypoint. A
  characterization test pins the `TimeConstrained` precedent first; that was
  the observation brief-510 reported it could not take rather than inferring
  it. Gate: 2 490 passed, 0 failed, 57 ignored, run without
  `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`. **Left open deliberately:** nested
  file-form deadline renewal, diagnosed by brief-510 and not folded into a
  correctness patch. The original finding follows.

- **MEDIUM — `eval_with_timeout`'s deadline is silently dropped across an input
  suspension.** `crates/rum-host/src/session.rs:518-524` clears the timeout
  unconditionally after the guarded call, and its own rustdoc says the timeout
  "is cleared after the request completes or fails" — a suspension is neither.
  `resume_eval_request_with_input` never re-arms it. **Director-reproduced**: a
  500 ms deadline, suspended on `Input[]`, resumed and ran 1.39 s to completion
  with no termination reason. The host cannot re-arm it; there is no
  `set_eval_timeout` on `HostSession` and no timeout variant of
  `resume_with_input`.

- **LOW — a terminated request leaves `$Context` where the script put it**, and
  the next request inherits it. Same family as the control-state leaks briefs
  397-411 closed; `Block` bindings and `TimeConstrained` deadlines were verified
  to restore correctly, so this is the survivor. No document says whether
  context is per-request or per-session state.

- **LOW — `restore_from_bytes` is arbitrary code execution by design**, and the
  documentation states the mechanism without stating the consequence. Restoring
  untrusted bytes runs them under that session's policy; a sandboxed session
  refuses the capability-gated parts, an unrestricted one does not.

- **LOW — the jupyter signature policy fails open.** An unrecognised
  `signature_scheme`, or an empty key, silently yields
  `KernelSignaturePolicy::Disabled` (`connection.rs:123-131`) rather than
  refusing to start. HMAC comparison is also not constant-time
  (`transport/codec/validation.rs:45,50`), which is near-unexploitable on local
  IPC but differs from the reference implementation.

**Three criticisms of the threat model itself**, which are worth more than some
of the findings: the shipped jupyter kernel is itself a host and chooses full
authority with no per-cell timeout or memory ceiling, which no document states;
the two-stage memory defence has a gap between the modelled ceiling and the
host's own RSS watchdog, which the docs describe as merely "one pattern"; and
**no document says which state is per-request and which is per-session**, which
is the same question briefs 397-411, this audit's `$Context` finding, and the
timeout gap are all instances of.

### Closed by brief-415/416 (`1427bb71`) — the walk itself

A third gap, found reviewing brief-414's plan rather than by the audit.
`current_memory_proxy_bytes` is the baseline the strided check and all 18
projected-byte prechecks measure against, and it walked 34 of `Session`'s 93
fields. It now walks 52; the other 41 carry a recorded exemption, and a macro
generates both the inventory and an exhaustive `let Session { .. }` pattern, so
a new field stops compiling until it is classified. `ByteArray` also stopped
being charged its decoded length instead of the base64 text it holds.

### Opened by that work, not yet briefed

- **The call-stability cache is bounded by entry count, not by bytes.**
  `CALL_STABILITY_CACHE_MAX_ENTRIES = 1 << 14`
  (`runtime/evaluation_cache/stability.rs:12`) caps entries at 16 384, but each
  entry pins a `SharedExpr` head and `SharedArgs` — one entry was measured
  holding 64 KB. This is the same error as the stride comment: a budget that
  bounds counts while the quantity at risk is bytes. It was invisible until
  brief-415 made the cache part of the walk; now a session can approach its
  ceiling on cache alone. The repair touches insertion paths, so it needs its
  own brief and the mandated hot-path A/B. Fold it into the Gap B line.
- **Abandonment retains 1 668 bytes more than completing the same work**, and
  nobody has explained why. Deterministic: identical across five consecutive
  runs of `suspension_abandonment_contract`. It is two orders of magnitude
  below the ~64 KB a genuinely retained `Reap` bucket would cost, so the
  contract tests absorb it in a stated margin rather than failing — but the
  constant is unexplained, and an unexplained constant in a test becomes a
  fossil. Worth one probe before the Gap B line closes.

### The traversal-depth line (briefs 422-433) — the per-node bound is in; the construction-side bound is not

**What is fixed.** brief-422 bounded four unbounded recursive descents that
ended the process: `FromXml`, `RegularExpression` group nesting, `rum-syntax`'s
postfix-blank chain, and the four codec encoders. brief-425 then converted ten
traversals to iterative walks — `ToString`, `Map`, `Apply`, `ReplaceAll`,
`Cases`, `Level`, `MemberQ`, `Count`, `Position`, `FreeQ`.

**Measured on a 2 MiB stack** — Rust's default for a spawned thread, so what an
embedder gets without setting `stack_size`
(`docs/guide/resource-limits-and-sandbox.md:122`). After brief-425:
`ToString` 43,000, `Level` >20,000, `Map`/`Apply` 20,000, and six others at
4,500 completing / 4,700 not. `Expr::Display` clears 4,097 with headroom to
5,000-8,000; `Depth`, `LeafCount` and `ByteCount` clear it.

**Rulings, all 2026-09-03.** `MAX_PHYSICAL_EXPR_DEPTH >= 4097`, preserving
plan-330's documented 4,096-link contract; the bound is on **physical
expression height**, not the `Depth` builtin, which omits heads and treats
associations specially and so certifies a different tree; traversals get **no
new knob** — rum already pairs a fixed portable limit with a measured backstop
(`recursion_limit` beside `poll_native_stack_limit`) and the design extends
that pair; and `DEFAULT_NATIVE_STACK_LIMIT` at 6 MiB fails open on a 2 MiB
thread where it can never fire, so it becomes safe-by-default on the smallest
supported stack, raised by hosts that have more.

**The construction-side bound is CLOSED as not affordable (user ruling,
2026-09-03, after four spikes).** Admission is O(1) only if the physical height
is cached, and every way of knowing it costs more than the bound is worth. The
decomposition is complete and each term was measured separately:

| Term | Where measured | Cost |
|---|---|---|
| 8 bytes per `Arc` allocation, height never computed | brief-434 V1 vs V0 (director) | **+5% to +14%** |
| `SharedArgs::new` scan with an Arc dereference per child | brief-434 V2/V1 | +3.9% to +11.4% |
| full rescan after every mutation | brief-434 V3/V2 | free |
| O(1) conservative maintenance (`max(old, written)`) | brief-434 V4/V2 | free |
| `max` over args at construction, inline in `Expr::Call`, no allocation growth | brief-435 (director-confirmed) | **+6% to +9.5%** |

Two of those results are worth keeping. **The allocation term was the hidden
one**: `stringtrim-literal-list-32000` builds no call spines and never reads a
height, and it cost +14.21% from a field that was never computed —
`SharedArgsInner` 40→48 bytes moves it into the next allocator chunk class.
`Expr` and `Atom` are both 32 bytes while `Expr::Call` carries 16, so an inline
height grows no allocation, and brief-435 proved that removes the term
completely (those same rows return to −0.40%…+0.93%). **What no design removes
is the pass itself**: computing `max` over the arguments at construction costs
+6% to +9.5% on the construction and mutation families.

**The one untried shape, and why it was not tried.** The caller could supply the
height, fused into a loop it already runs — a `Table` builder tracks a running
max, a `Part` assignment knows `max(old, replacement)`, a permutation knows the
max is unchanged. That is the only remaining place the cost can go. It was
rejected on a semantic argument rather than a measurement: a supplied height can
only be checked by a debug assertion, and the construction-side bound was worth
its price precisely because it turned an omitted traversal check into a *type*
invariant. A height that callers must get right is the "relies on people
remembering" shape this project replaces with structure, so the design no longer
buys what made it attractive.

All four spikes are preserved: brief-426 and brief-434 V4 in the stash,
brief-435 in history at `74082418` (reverted by `c8031ee1`).

**Consequence for this line: the tail is now the work.** Without a construction
bound, each recursion in brief-422 §3(e) has to be measured and, if it falls
below 4,097, bounded on its own. That is the next item.

**The 4,500 ceiling was candidate normalization, and it is now fixed**
(briefs 427-433, `0f679699`). `match_expr_owned` recursively normalized the
whole candidate before dispatch (`pattern_matching/core.rs:23-32`), so the
ceiling belonged to one normalization and not to the traversal at all: a single
`MatchQ[x, _?IntegerQ]` — which defeats `fast_match_q_resolved_pattern` — died
in the same bracket as an all-level `Count` over the same value. The normalizer
now runs a depth-16 recursive front path and hands an over-deep subtree to a
cold explicit-stack engine in place, following `Level` and `Map`/`Apply`.

Measured on 2 MiB, director-run: `MatchQ[x, _?IntegerQ]` went from 4,601
completing / 4,651 overflowing to clearing **43,001**, and
`Count[x, _, Infinity]` from 4,501 / 4,701 to clearing **20,001** — 10.5x and
4.9x the ruled 4,097 minimum. Above 43,000 the limiter is `Depth`'s own
recursion, not the normalizer; the checked-in certificate therefore excludes
`Depth` and a separate liveness test carries the height assertion.

**It removes stack growth, not repeated work.** The general entry still
normalizes the entire candidate on every invocation, so `Cases` over a deep
tree is still quadratic. Two deferred work-removal items sit here, priced by
brief-428 §6 and deliberately kept out of that A/B: `Cases` normalizes each
candidate twice, and the pattern is recompiled per selected node.

**The cost took three shapes to get to zero, and that is the lesson.**
Propagating exhaustion as a `Result` cost 6-15% on wide traversal normalization
because every atom returned through it; handling exhaustion locally fixed that
but left ~8% on `matcher-single-*` because atoms still entered a function
carrying a depth parameter their arm ignored. The recursive worker now takes a
decomposed call, so an atom cannot reach it.

**What remains, and why it is not queued** — *superseded 2026-09-04 by briefs
436-446; see "The deep-rebuild class" and "The native stack budget" below. Most
of the deep failures listed here turned out to share one defect downstream of
the traversals, the evaluation-stability walk, and the per-site conversions that
survive that repair are brief-439's steps 2-8. The paragraph is kept because its
inventory is still the map, and because its warning — measure before briefing —
is exactly what re-probing under the new budget vindicated.*

brief-427 classified brief-422 §3(e)'s tail: only the rewrite family shares the normalizer mechanism, and
everything else is its own independent recursion — string-pattern machinery,
calculus, polynomials, schema conformance, `Flatten[_, Infinity]`, tensor rank,
`Merge` nested, substitution walks, fingerprints, `MapIndexed` and `Scan`
(which brief-425 did not cover), `Expr`'s `Display`, `PartialEq`, `Hash` and
serde, and `lowering.rs` for host-constructed ASTs. Suspension continuation
encoding is "unclear — needs measurement". **None of these is known to fall
below 4,097; they are unbounded, which is not the same thing.** Measure them
against the bound and order by real ceiling before opening briefs, rather than
scheduling a dozen repairs on source reading alone.

**They now have numbers (director-measured, 2026-09-03).** Release CLI,
`ulimit -s 2048`, values built with `x = 1; Do[x = {x}, {i, N}]` or the shape
each probe needs; every failure below was re-run and confirmed to be a genuine
`thread 'main' has overflowed its stack`, not an in-band error and not a
timeout.

| Probe | completes / overflows | against 4,097 |
|---|---|---|
| **`StringExpression` nesting (construction)** | 375 / 550 | **10.9x short** |
| **`Replace[_, _, {0, Infinity}]`** | 1,250 / 1,425 | **3.3x short** |
| **`MapIndexed[_, _, Infinity]`** | 1,600 / 1,775 | **2.6x short** |
| **`D` over a nested-`Sin` spine** | 1,600 / 1,775 | **2.6x short** |
| `FullForm` (`Expr::Display`) | 5,800 / 6,150 | 1.4x |
| `Save` (session serialization) | 5,800 / 6,150 | 1.4x |
| `SameQ` (`PartialEq` logical path) | 7,900 / 8,600 | 1.9x |
| deep `Association` construction | 10,700 / 11,400 | 2.6x |
| `Merge` over deep associations | 10,700 / 11,400 | masked, see below |
| `Scan[_, _, Infinity]` | 11,400 / 12,100 | 2.8x |
| `ArrayQ` | 15,600 / 17,000 | 3.8x |
| `Flatten[_, Infinity]` | 25,400 / 26,800 | 6.2x |
| deep `Association` key (`Hash`) | clears 45,000 | — |
| `Depth` | clears 20,001, fails by 43,001 | — |

**Four fall short, and that is the queue**, in shortfall order:
`StringExpression` construction, `Replace` at all levels, `MapIndexed`, and the
calculus walker. brief-427 predicted `MapIndexed` and `Scan` as a pair; `Scan`
clears by 2.8x and `MapIndexed` does not, so the pair was not a pair.

**Two rows share a bracket and a mechanism.** `FullForm` and `Save` both give
out at 5,800/6,150 because a session snapshot is a UTF-8 source image, so
`Save` renders through the same `Expr::Display`. Fixing one fixes both, and it
is the same renderer brief-424 found at 5,000-8,000.

**`Merge`'s own ceiling is unmeasured**: building the deep association it needs
gives out at exactly the same 10,700/11,400, so the construction masks it. Any
probe of an association-consuming traversal has to bracket the construction
first — this one did, which is the only reason the masking is visible.

**Probes that measured nothing, recorded so they are not repeated.** `D` and
`Expand` do not descend into lists at all (`D[{{{q}}}, q]` returns a chain-rule
expression over `List`, `Expand` returns its argument unchanged), so a
list-shaped probe of either measures the construction; the calculus number above
comes from a nested-`Sin` spine instead. `ArrayDepth` and `Dimensions` are not
rum builtins and return unevaluated — which still prints, so they would have
reported a false ceiling. `Nest[List, 1, N]` fails in band at the fixed-point
pass limit (256), not on the stack. `MergeNested` with one argument returns an
operator form.

**Not probed, and why.** String-pattern *matching* machinery (the construction
probe dies at 400, so matching cannot be reached with a deep pattern this way),
suspension continuation encoding, `lowering.rs` for host-constructed ASTs (not
reachable from the source language), profiling fingerprints (diagnostics-gated),
and polynomial walkers (no cheap deep probe — a deep polynomial expands
exponentially).

**Adjacent, still open:** `rum-jupyter`'s `json_support.rs:61` has no depth
guard on wire-message JSON (the 2026-09-02 audit's F3), deliberately untouched.

### The interner ruling of 2026-09-21 — A1 is refuted, the fix is bounded not reclaiming

**This supersedes the A1 half of the 2026-09-02 rulings below.** Read this
first; the 2026-09-02 text is kept because its reasoning about `Symbol`'s
equality contract is still correct, not because its choice still stands.

**What the spike measured** (briefs 499-503, code on branch
`spike/brief-499-symbol-non-copy`, reports in `docs/dev/codex/`):

- The non-`Copy` migration alone costs **+0.06% to +0.84%** and is a fixpoint
  rather than a count: 92 files, a 35-type `Copy`-loss closure of which 29 are
  the off-by-default profiling keys.
- The full A1 candidate costs **+14.33% instructions on ordinary evaluation**,
  splitting cleanly into **+10.93%** for the owning representation and
  **+3.42%** for the equality demotion, residuals under 0.1 pp across twelve
  workloads.
- `V-arc` — `Arc<SymbolText>` keeping the dense-id fast accept — is the
  **cheapest owning variant that can exist** at +10.93%, and it is not even a
  valid A1: its equality is sound only while the interner stays process-global.
  Session-scoped ownership forces the fingerprint equality back.
- Type sizes are unchanged and the depth contract survives (release slope
  3.36% *smaller*, margin 44.84% over 4 097). `Symbol`'s `needs_drop` flip
  costs 11.39% of the debug slope and turns a depth-48 test red; that dies
  with the candidate.

**What the user ruled (2026-09-21).** `Symbol` keeps its **lifetime-free
public type** — `'static`-compatible and unbranded — as well as its 24-byte
`Copy` representation and id-based fast-accept equality. The embedding API
line deliberately built an owned, movable `HostSession`, suspension frames are
`Send + Sync + 'static`, and jupyter moves a session through a worker thread;
a generative brand would force scoped continuations and a scoped host API.
That is semantic loss, not cost, which is why it counts.

**The consequence, stated plainly so it is not softened later.** brief-504
surveyed six options: with the lifetime-free type frozen, **no in-process
reclaiming fix exists**. The finding is therefore **not closed**. It is
converted from unbounded invisible growth into a bounded, loud limit:

- a process-wide interner **byte** quota — count-only is insufficient because
  names vary in length — with a recoverable refusal replacing the `u32`
  `expect` at `crates/rum-core/src/expr.rs:88`;
- refusal coverage made **compiler-enforced**: the only arbitrary-name
  insertion API returns `Result`, and every constructor and serde path is
  converted to it;
- preflight of string and hash-table growth **before** anything is leaked,
  with fixed error/bootstrap symbols reserved so refusal can be reported;
- honest documentation: interner bytes are process-global, `reset()` does not
  reclaim them, one tenant can permanently exhaust the shared quota, and host
  process isolation remains the pattern for hard multi-tenant isolation.

**The director's rulings on brief-505's D1-D7 (2026-09-22).** The design pass
is complete; these settle it. Sequence stays A (prove bounded storage and the
admission instrument) -> B (integrate) -> C (independently falsify and gate).

- **D1 — the default is NOT set here; it becomes a measurement obligation.**
  Draft A measures and reports, the director chooses, and the worker may not
  tune it. Three workloads must be measured: the bootstrap footprint, the peak
  interner bytes across the ordinary test suite and the microbench panel, and
  **`Unique[]` in a loop**. That third one is the point: director-verified
  2026-09-22, `Module` gensyms do **not** grow the interner — `with_identity`
  varies only the numeric identity while the interner is keyed on
  `(context, name)` (`crates/rum-core/src/expr.rs:80-96,115-122`) — but
  `Unique[]` mints fresh text every call
  (`crates/rum-runtime/src/runtime/scoping/unique.rs:42`). So the language has
  a built-in legitimate unbounded interner consumer, and the report must say
  how many `Unique[]` calls a candidate default permits. A default that
  refuses a supported workload repeats the 6 MiB guard on a 2 MiB thread.
- **D2 — approved as a proxy, and only as a proxy.** Exact Rust
  allocation-layout bytes plus a conservative fixed reserve, explicitly
  excluding allocator metadata, rounding and RSS. That is consistent with
  `current_memory_proxy_bytes`, which is already a documented model rather
  than a physical meter, and with `SECURITY.md` telling embedders to run
  their own RSS watchdog. The portable-physical-bound reading is **rejected**;
  do not let it return as "conservative". Private explicitly-sized indexes
  replacing the opaque `HashMap`/`HashSet` are approved, gated by D6.
- **D3 — approved in full.** `Result` on all seven constructors, arbitrary
  registration, installer callbacks, configured host build/reset, and serde.
  **No quota-panic compatibility layer** — that would be the convention-
  enforced version of the thing we are making compiler-enforced. Zero identity
  becomes a constructor/serde error rather than a panic: approved, it removes
  a panic. The infallible fixed-name subset stands only behind its own closed-
  installer proof; without that proof it is fallible too.
- **D4 — approved with one constraint.** `heads.rs` already puts canonical
  spellings in `rum-core` (`crates/rum-core/src/heads.rs:112`), so a
  spelling/identity table there extends an existing boundary rather than
  opening a new one. The constraint: it carries **spellings and identities
  only** — no attributes, no specs, no behavior, and it confers no builtin
  status — and a consistency test cross-checks it against the stdlib catalog
  so divergence is loud. If the implementation finds it needs any behavioural
  field in core, that is a **STOP**, not a judgement call.
- **D5 — approved for the new kind only, and it surfaced a separate defect.**
  `InternerLimit` termination, `General::internerlim`, typed host failure and
  explicit reset/restore semantics: approved. But brief-505 noticed that
  restore's `Ok(_)` **discards termination** today
  (`crates/rum-host/src/session.rs:201-220`) — a terminated evaluation during
  restore is reported as success. Handle the interner kind here and **do not**
  make restore transactional or touch other termination kinds; the general
  case is its own item, recorded below.
- **D6 — approved as stated.** Gate with the instruction-count recipe
  (`taskset -c 2 perf stat -e instructions:u`), not the timing panel. Two arms
  suffice here — `main` versus the change, built at the same path. A
  regression is a defect to fix or report. No substituting an unbounded table,
  an approximate undercharge, or any ruled-out `Symbol` representation.
- **D7 — approved as a separate brief**, not folded in. The audit's
  per-request-versus-per-session criticism stays open and is listed below.

**The census retargeted the line (brief-506, 2026-09-22).** Measured on
`main`: bootstrap 662 strings / 661 pairs (~77-85 KB proxy); the ordinary
workspace suite's **maximum over test binaries** 14.4-15.9 MB; `Unique[]`
~122-138 bytes/call; `Symbol[StringJoin[…]]` ~220-244; **`Module`
~120 bytes per activation, permanently**.

Three corrections came out of it, one of them the director's:

1. **`Module` leaks interner text on every activation, and the director's
   claim that it does not was wrong.** `module.rs:46-50` →
   `allocate_fresh_symbol` → `allocate_generated_symbol` → `Symbol::new` with
   formatted fresh text (`scoping/unique.rs:36-42`). `with_identity` sharing
   an interned pair is a true API property and was not evidence about this
   call path; the director read `resolution.rs`'s generation lookup and
   generalized it to a different mechanism. The gensym GC cannot help — the
   interner has no removal path at all.
2. **Lookup is itself an insertion ingress.** Resolving a fresh unqualified
   name constructs candidates per probed context and charges **two** pairs
   (`resolution.rs:101-138`). A refusal inventory assuming one pair per
   textual symbol is wrong.
3. **8 MiB is below this repository's own test suite.** 32 MiB is the first
   envelope clearing bootstrap, the suite maximum and the panel allowance —
   a comparison, not yet a chosen default.

**Therefore the target moved, and this is a director retarget on evidence,
not a change of line.** A cap that ordinary code exhausts is a timer, not a
defence: with `Module` leaking, a long-lived host refuses after enough
*legitimate* work. So the dominant legitimate consumer is addressed first,
and the cap is sized afterwards against a system where ordinary code does not
leak. The cap itself is not abandoned — brief-505's plan and the D1-D7
rulings stand and resume after this.

**Feasibility, probed before briefing.** `Symbol::with_identity` already
shares one interned pair across distinct identities and is already the
mechanism `active_symbol_for_name` uses; serde already carries `identity`
as its own field, so snapshots survive. Only five source locations assert a
gensym's printed `name$n` text, and they are concentrated in `Unique[]`'s
own tests — and `Unique[]` returning a freshly *named* symbol is documented
behavior, while a `Module` local's printed name is not. The crux for the
design is therefore whether the `$n` suffix can become a **display**
concern rather than interned text.

**The Module repair is feasible but BLOCKED ON A USER RULING (brief-507,
2026-09-22).** Representation: **yes** — `Symbol` already carries the
independent identity field, so reusing one interned pair while varying a
checked identity preserves the ruled equality and layout and removes the
retained suffix text. Display-only patch: **no**, and the obstruction is
text-to-value identity, not aesthetics. ``("Global`", "x", identity 3)`` and
``("Global`", "x$3", None)`` are different symbols, so printing the first as
`x$3` does not tell the resolver how to recreate it — and that round trip is
in use: source, `Symbol` and `ToExpression` probes all reach the current
local, `Names` targeting performs the conversion
(`crates/rum-runtime/src/runtime/namespace_tools/names.rs:152`), and **host
snapshots are source text** (`crates/rum-host/src/session.rs:185`). `Symbol`'s
serde carrying `identity` is not evidence against this, because the snapshot
is not serde.

**The ruling the user must make.** Three ways forward, none free:

- **Separate `Module`'s display label from a source-resolvable name.** The
  guide's Module paragraph does not promise that round trip
  (`docs/guide/scoping-and-contexts.md:44`), but the surrounding public
  surfaces must be judged rather than permission inferred from silence.
  Cost: gensym snapshot round-trip, `Names` targeting, documented surfaces.
- **A lossless alias map**, retained forever — trades an interner leak for
  another activation-count-dependent store, so it is not obviously a win;
  collecting aliases changes what old strings can recover.
- **A reserved textual encoding** — changes the accepted symbol-name and
  `Unique` namespace, and must not decode sealed ids.

Until that ruling, `Module` keeps leaking and the quota work proceeds, since
the quota is required under every one of these outcomes.

**Opened by brief-507, not yet briefed: `Enclose` leaks too.** An untagged
`Enclose` adds one lazy seed plus one string and pair per call — 10 665
strings / 10 664 pairs and 1.24-1.36 MB at 10 000 calls, about 117-127 proxy
bytes each (`crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:118`).
So "only callers explicitly asking for fresh names grow the interner" is
false, and `Unique` is not the dominant remaining consumer. Schedule its
repair before sizing the cap, or carry its measured demand into the sizing.

### User rulings of 2026-09-22 — the snapshot format, and Incremental first

**Ruling 1: the snapshot format changes (option ②).** The root constraint is
that a symbol's identity must be expressible as text because the host
snapshot is source text. The user ruled that this is **an old debt now forced
into the open and it has to be paid**: Standing Engineering Debt's deferred
*"direct binary snapshot format"* becomes a planned line rather than a
deferral. The reserved-textual-encoding option (①) and the
accept-the-leak-behind-a-cap option (③) are **not** chosen.

Consequences to carry: the interner quota work (briefs 505-508) is **not the
remedy** and is parked, not cancelled — brief-508's proof stands and D1-D7
remain ruled if it is ever resumed. `Module`'s and `Enclose`'s leaks are
**not** patched individually; they dissolve when identity stops needing to be
text. The `Unique[]` case does not, and remains by design.

**Ruling 2: build `Incremental` (decision two, as recommended).** The
unstrided armed ceiling (`d1e5de5c`) is correct and stays, but armed sessions
pay **2.26x to 43.34x** instructions — and the people paying are exactly the
ones who asked for safety, since `eval_memory_limit` defaults to `None`. The
risk is losing safety by the back door, as hosts stop arming a limit they
cannot afford.

**Ruling 3: sequencing.** `Incremental` first; the snapshot format is
discussed after it.

**What the Incremental line has to settle, before any implementation.**

- **165 builtins already declare `Incremental`**
  (`crates/rum-stdlib/src/catalog/memory_growth_contract.rs`) against a
  contract with no implementation — "retained growth must be charged as
  data-dependent construction proceeds"
  (`crates/rum-runtime/src/runtime/metadata.rs:346-348`). The unstrided walk
  is currently discharging all 165 obligations at O(session) per checkpoint.
  That is the 43x.
- **Two different designs both called Incremental**, and the line must pick:
  per-builtin charging at 165 sites, which is what the declaration says; or
  incremental maintenance of the session total at the few places session
  state changes, which makes `current_memory_proxy_bytes()` O(1) and is what
  actually kills the 43x.
- **The models disagree and that is the crux.** The existing walker charges
  repeated shared subtrees **by occurrence**; a counter of unique allocations
  does not reproduce that. Changing it changes what the ceiling means, so the
  discrepancy has to be resolved deliberately rather than discovered.
- It must also account for capacity growth, expression import and sharing,
  retention in definitions and caches, prints/traces/reaps, suspension and
  abandonment, and release on every exit — and stay consistent with the
  macro-generated session-walk inventory that briefs 415-416 built, or
  replace it.

### Director rulings on brief-516's five decisions (2026-09-22)

Ten drafts, A-J: **A-E** build the occurrence-based session total that
removes the 43x; **F-J** discharge the 165 per-builtin declarations. All five
decisions ruled, so the line can start.

1. **Split and order approved.** Occurrence-based session sum first, local
   retained builders after. This keeps the existing contract, so the
   **ceiling's meaning does not change** — a unique/physical counting model
   would be a separate explicit ruling and would invalidate the plan.
2. **Overflow: conservative saturation approved**, with a non-lossy or
   explicitly invalid state for releases. Both directions fail closed — a
   saturated charge makes the session look over its limit, and a
   non-representable release leaves the total too high rather than too low.
3. **The ledger's own storage is charged, and that is not negotiable.**
   Every new inline or heap bookkeeping byte appears in the proxy, with its
   overhead measured and its threshold effects documented. Silently exempting
   the accountant is precisely the class of invisible bytes this whole line
   exists to remove — it is the interner's mistake in miniature.
4. **B's boundary: public extension behavior is preserved.** `BuiltinSpec`
   is a public host-facing registration surface; an externally registered
   builtin must not break and must not be **silently** mis-accounted. If a
   route cannot cover host-registered builtins at zero default cost, that is
   a **stated gap plus the stop condition already in the draft** — keep
   correct walking; never accept unobserved drift.
5. **F's coverage extension approved**, including the clause that matters:
   keep the existing S oracle separately and **do not pretend the extension
   is numerically identical** to the old session-plus-expression
   observation. It is a different number and is to be reported as one.

**Rejection conditions stand as the draft wrote them**: unavoidable default
cost, meaningful mutation or extension paths that cannot be covered without
silent drift, or repeated unchanged-root checks still scaling with session
size. In any of those, the correct armed walk remains and its measured cost
becomes a documented property.

**Opened by brief-516, not yet briefed — a doc-versus-code conflict.**
`docs/guide/resource-limits-and-sandbox.md:318` says a consumed local
`MemoryConstrained` returns with `General::memlim`; the handler at
`runtime/debug_tools/resources/constraints.rs:262` returns the fallback or
`$Aborted` **without emitting that message**. Source observation, not a probe
— **re-probe before briefing**, and settle which side is wrong. Do not fold
it into the ledger cutover and do not weaken the guide incidentally.

### Standing user ruling (2026-09-22): not knowing is the only unacceptable outcome

> 我们是研究型项目，所有路径都要确定地趟过，量过真实的代价才能得出结论，
> 可以回退，可以浪费时间，但不能不知道。

This extends the 2026-08-27 standard ("pick the best and most correct option;
cost arguments do not count") with something stronger and different: **a path
is not closed by an estimate, a projection, or a cheaper proxy measurement.
It is closed by walking it and measuring what it actually costs.** Rollback is
free, wasted time is acceptable, an unmeasured conclusion is not.

Consequences for how briefs are written from here:

- **A bar is a data point, not a stop condition**, unless it guards
  correctness. "Unarmed cost exceeded 0.7%, so I stopped" is the wrong
  response; "here is the curve past 0.7%" is the right one.
- **Do not offer "stop here" as an option** alongside paths that have not
  been walked. Present the cost of finishing, as the 2026-09-21 line rule
  already says, and let the user rule.
- **Do not substitute a cheap distribution probe for building the thing**,
  though measuring as you build is exactly right — the curve is part of the
  result.
- Refutations still count, but only measured ones. brief-518, 519 and 521
  are closed because each was walked and measured, not because any looked
  unpromising.

**Applied immediately**: the user ruled option ① for the `Incremental` line —
build all 52 owners — over the director's recommendation of a cheaper
distribution probe first.

### Corrected ruling (user, 2026-09-22): the armed cost IS the security model's cost

**The director's framing was wrong and the user corrected it.** Two errors,
both mine:

1. **The armed path was treated as a niche opt-in.** `SECURITY.md` says the
   opposite, and more strongly: rum's security model *"is about evaluating
   untrusted `rum` scripts safely inside a trusted Rust host"*, and a host
   evaluating untrusted input **MUST** set the request memory ceiling and the
   elapsed timeout, both off by default. So the armed path is not a minority
   configuration — it is the configuration the threat model is written for,
   and its 2.26x-43.34x is **the running cost of the security model**, not a
   performance curiosity.
2. **The wrong hard line was invoked.** "No default-path cost" was written
   about off-by-default **diagnostics** — the `RUM_PROFILE_*` counter family.
   The memory ceiling is not diagnostics; it is the sandbox's core bound and
   a documented MUST. Applying the diagnostics standard to it is what
   produced brief-518's refutation.

**The standard is therefore changed**: for this line, an unarmed cost of up
to about **0.7%** is acceptable if it buys an affordable armed path, because
the unarmed path is by definition the one *not* evaluating untrusted input.
brief-518's measurement stands (0.465994% / 0.697458% / 0.622776%); only its
acceptance criterion was wrong.

**Route ② stays dead** regardless: it was refused on **fail-open**
correctness — 42 of 52 walked fields uncovered, and a stale low total reused
on the very reproduction — not on cost.

**What arming still does not guarantee, and must not be oversold.** The
ceiling is a structural model, not an allocator bound: a single construction
larger than the limit is rejected before allocation only where a projected
precheck exists, and a real allocation failure reaches `handle_alloc_error`,
which aborts outside the panic boundary. "Guest code cannot kill the kernel"
needs the host's RSS or process bound as well — the audit's third criticism,
still open.

**Sequencing decided before spending the 0.7%.** Route ① is the *enabler*,
not the fix: it carries the mode, while the 43x is the O(session) walk that
only a maintained total removes at step E. Paying 0.7% for plumbing whose
payoff is unmeasured would be backwards, so the next brief measures the
**ceiling of the achievable win** — an armed build whose session-proxy term
is stubbed to O(1), deliberately incorrect and used only for timing — before
the line commits.

### The Incremental line: two routes measured, both refused (briefs 516-519)

**Where it stands.** The correctness half is done and landed; the cost half
is blocked by a hard line, and that is now evidence rather than opinion.

**Landed:**
- `d1e5de5c` — every armed checkpoint walks, so the ceiling a loop used to
  walk through now holds.
- `902aea1f` — the occurrence model is specified per field, with an
  independent oracle, an exhaustive schema whose compile barrier was watched
  stopping an unclassified field, and pinned formula and overflow tests.
  This is what any future attempt is checked against.

**Refused, both on measurement:**
- **The accounting-mode route** (brief-518). Works across all four
  crossings; costs the **unarmed** path 0.465994% / 0.697458% / 0.622776%.
  Its own distinction is the useful part: *the counter hooks erase, the mode
  route does not* — carrying the mode is the cost, not the accounting.
- **The session-total cache** (brief-519), refused twice over. Its
  favourable first numbers were an **inlining artifact** — adding the counter
  made the compiler inline `evaluate_inner_unstable` — and with inlining and
  field layout held constant the mandatory mutation signal costs
  **+0.182073% / +0.470698% / +0.134833%**, scaling with N. Independently of
  cost it is **fail-open**: 42 of the 52 walked fields have no covering
  existing counter, and the eight-counter tuple reuses a stale low total on
  the very reproduction this line exists to fix.

**Why F-J does not rescue it.** Per-builtin charging discharges the 165
declarations but does not remove the walk, and the 43x *is* the walk. The
cost fix and the declaration debt are separate problems, and only the second
is still reachable.

**So the standing position is: the armed ceiling costs 2.26x-43.34x and that
is a documented property**, unless a constraint moves. The binding constraint
is the hard line that off-by-default machinery must not charge the default
path. Both refutations are ~0.13% to 0.70% of default-path cost — and this
project has accepted +0.714% and +0.606% before, to remove a process kill.
**Whether to spend default-path budget to make armed sessions affordable is
a user ruling, because it breaks a standing hard line.** It is not the
director's to take.

### The root constraint, found by two refutations (2026-09-22)

Two independent repairs were designed and both were refused on the same
evidence — `Module`'s locals (brief-507) and `Enclose`'s internal tag
(brief-509). The obstruction is not in either construct. It is this:

> **A symbol's identity must be expressible as text, because the host
> snapshot is source text.**

The mechanism, stated once: `qualified_name` ignores `identity`
(`crates/rum-core/src/expr.rs:174`) and `Display` delegates to it (`:322`), so
two same-text symbols with distinct identities are **indistinguishable the
moment they are rendered**. The resolver returns `Symbol::system(name)` with
no identity (`crates/rum-runtime/src/runtime/resolution.rs:129`), so even a
rendered `$n` suffix would not teach it to reconstruct one. `Symbol`'s serde
does preserve identity (`crates/rum-core/src/expr.rs:288`) and **is not the
host snapshot format** (`crates/rum-host/src/session.rs:185`).

Therefore every generator of distinguishable fresh symbols must mint fresh
**text**; therefore the interner grows with generated symbols; therefore it
leaks. `Module`, `Enclose` and `Unique` are three instances of one cause, and
patching them one at a time cannot work. This also connects to an existing
standing-debt entry: *"Session snapshots are UTF-8 source images. A direct
binary snapshot format is deferred."* That deferral is what makes the
constraint binding.

**Three ways out, all requiring a user ruling.** None is free and none is
obviously right:

1. **Make identity expressible in the rendered form** — a reserved textual
   encoding the resolver can decode back to text-plus-identity. Changes the
   accepted symbol-name and `Unique` namespace, and must not decode sealed
   ids (brief-504).
2. **Change the snapshot format** so persistence is not source text — the
   binary format already deferred in Standing Engineering Debt. This removes
   the constraint at its root and is the largest of the three.
3. **Accept the leak and ship the cap**, documenting that ordinary code
   consumes it and that a long-lived host will eventually refuse. brief-508
   proved the cap's mechanism works; this is the honest-but-weakest option.

**What is proven and waiting.** brief-508's scratch proof is green and
director-reproduced: exact charging including backing-table bytes, refusal
that publishes and leaks nothing, the two-pair ingress admitted or refused
atomically, a full interner still serving lookups it already holds, and a
falsifier watched going red with enforcement disabled. The cap's mechanism is
not in doubt; only whether it is a defence or a timer, which is what the
ruling above decides.

**Opened by brief-512, and it is the root of a class — not yet briefed.**
`crates/rum-runtime/src/runtime/session/session_eval.rs:328-338` assigns
`termination = outcome.termination` for **each script form** and breaks early
only for accepted cancellation. So a later ordinary form **erases** an
earlier form's resource termination before any caller sees it: termination is
last-write-wins across forms rather than sticky. A multi-form snapshot whose
middle form terminates still reports success even after `acfda0e0`.

This is the fourth sighting of one shape — cancellation erased by its own
request prologue (briefs 397-411), the deadline dropped across an input
suspension (brief-511), the restore discard (brief-512), and now the
aggregation loop underneath them. **Diagnose the aggregation before briefing
another instance**, and settle whether a terminated form should be sticky or
whether `terminated_handling` deliberately allows continuation. **Director re-probe,
2026-09-22: two attempts failed to construct the erasure, which lowers this
finding's severity rather than raising it.** A depth-12 `f[f[…]]` under
`recursion_limit: 4` does not terminate at all — the head is undefined, so
nothing recurses — and a 500-element literal list under
`generated_element_limit: 10` does not terminate either. Restore forms are
structurally uniform `Set[…]` of already-evaluated values, so a limit that
fires on one form but not the next may be hard or impossible to reach through
restore; and `recursion_limit: 0`, the one trigger known to work, fires on
*every* form, so the last termination survives and restore correctly fails.
**The code shape is confirmed by reading; its reachability is not.** Establish
reachability before briefing a repair, and if it is unreachable through
restore, check the other entrypoints before concluding the class is benign.

**Mostly closed by brief-512 (`acfda0e0`) — see the reachability note below:** `restore_from_bytes` discarded the
response's `termination`, so a restore truncated by a resource limit reported
`Ok(())`. It now fails with the reason in the message; restore stays
non-transactional and both the rustdoc and `docs/guide/embedding.md` now
state that a failed restore leaves the session partially populated. The
regression test's trigger is a zero recursion limit — deterministic by
construction, no timing — with a no-limit control. **Bounded by the finding
above:** it catches the termination present in the response, and the
aggregation loop can in principle have erased it already — but see the
reachability probe recorded there, which failed twice to construct that
case.

**Opened by D5, not yet briefed:** `restore_from_bytes` reports `Ok` for an
evaluation that terminated during restore, so a caller cannot tell a restored
session from a truncated one. Found by design review, not reproduced yet —
**re-probe before briefing a fix.**

**Superseded note.** The earlier line here said five decisions; there are
seven. brief-505's census is also the argument for plan-first: **337 direct
constructor references across 78 files**, not the 179 a `Symbol::`-only regex
finds, since `Expr::symbol` and `Expr::system_symbol` add 157 more.

**Previously recorded, now settled.** brief-505's plan was blocked on D1-D7 — the 8 MiB default and the supported embedding envelope, the exact
charged-storage contract, public fallibility across the seven constructors,
and the two after them. Nothing is dispatched until those are settled. The
census it produced is the reason plan-first was used: **337 direct constructor
references across 78 files**, not the 179 a `Symbol::`-only regex finds, since
`Expr::symbol` and `Expr::system_symbol` add 157 more.

Rejected with their reasons, so they are not re-proposed: a reclaimable global
interner cannot exclude escaped-handle invalidation (a reissued dense id makes
an escaped `Symbol` denote different text); an arena lease on exported roots is
defeated by extracting the bare `Copy` `Symbol` and dropping the leased root;
session-local ids with an ordinary import function are convention-enforced,
since the same unbranded `Expr` enters four other ingresses; A2 as ruled is a
process-wide quota that cannot attribute bytes to a session.

### Rulings on the repair line (user, 2026-09-02) — A1 half SUPERSEDED above

Settled after brief-414's design pass and the director review of it.

- **The interner becomes session-scoped (option A1), but brief-414's drafted
  step 4 opens as a spike, not an implementation.** A1 is the only option that
  both reclaims on `reset()` and attributes bytes to the session that made
  them; A3 and A4 pay A1's price (`Symbol` stops being `Copy`) and buy less,
  and A2 keeps `Copy` while leaving the growth permanent. The reason for the
  spike is that `Symbol`'s `PartialEq`, `Hash` and `Ord` are all built on `id`
  being a perfect collision-free hash of the text — `eq` is a fast *accept* on
  `id` (`crates/rum-core/src/expr.rs:250-254`), `Ord` uses it as a fast path
  (`:275`), `Hash` hashes it alone (`:258`). A content fingerprint collides, so
  equality must demote to fingerprint-rejects-then-compare-text, and `Symbol`
  is the key type of `definitions`, `attributes` and six caches. The spike
  makes `Symbol` non-`Copy`, reports the true compiler-error count, and
  measures the new equality and clone/drop paths. A1 lands or falls on those
  numbers; the fallback is A2.
- **`Symbol::interned_id` is redefined as a stable content fingerprint**, with
  its rustdoc rewritten to say so. Its documented promise ("symbols with the
  same text share the id") becomes *stronger* — true across sessions rather
  than only within one process — and its sole consumer, the call-stability
  bloom (`evaluation_cache/stability.rs:23`), already multiplies by the
  golden-ratio constant, which is Fibonacci hashing designed to spread dense
  ids and is unharmed by receiving a hash instead. This also deletes the
  `u32::try_from(pairs.len()).expect(...)` panic path at `expr.rs:88`. The
  density and uniqueness of `id` are what equality relies on today, which is
  why this ruling and the A1 ruling are one decision, not two.
- **`ByteArray` is charged its physical representation.** It holds base64 text
  and was charged its decoded payload length, about 25% under what it occupies
  — the only shape in `expr_memory_proxy_bytes` charged less than it holds.
  Folded into brief-415.

## Decision-gated or deferred

- **Custom `Sort` / `KeySort` comparisons — settled 2026-08-26.** The ordering
  callback schedule is unspecified: a valid strict weak order determines the
  result, while comparison count, order and direction are not promised.
  Stability is still promised, and key evaluation stays exactly once per entry
  in input order — the contract relaxed comparisons only. Anything that would
  change what a comparator's *return value* means, or how many times a *key*
  function runs, remains a separate semantic decision.
- **Full polynomial factorization.** Deferred by user decision. The documented
  `Factor` contract remains the current rational-polynomial subset: monomial
  extraction plus recursively discovered linear factors. A substantially more
  complete algorithm requires finite-field factorization and its supporting
  polynomial machinery; do not grow the current linear-factor code piecemeal.
- **Formatter binary-chain planning (P2).** Some long binary chains still
  rebuild temporary vectors. Revisit after the P1 runtime/stdlib work, with
  formatter output frozen before changing the planner.

## Standing engineering debt

- Deep `Expr` trees constructed directly through the Rust API can overflow the
  native stack on drop. Three iterative `Drop` designs regressed the hot path by
  20-50% and were rejected. Keep the host limitation documented in
  [docs/host-embedding-and-hardening.md](./docs/host-embedding-and-hardening.md).

  **Corrected 2026-09-21.** This entry used to say source-created trees are
  depth-guarded. That was false for one shape: the parser's
  `MAX_DELIMITER_NESTING_DEPTH` and `MAX_OPERATOR_CHAIN_DEPTH` (both 128) count
  delimiter nesting and operator chains, and a **call-head chain** boxes the
  prior head without touching either, so `Hold[f[1][1]…]` was accepted at any
  depth while the nested `Hold[f[f[f[…]]]]` of the same depth was refused.
  brief-497 bounds AST-to-`Expr` lowering, so that shape now fails closed; the
  *drop* limitation above is unchanged and still applies to trees built through
  the Rust API.
- Session snapshots are UTF-8 source images. A direct binary snapshot format is
  deferred until substantially larger session images justify the extra format
  and compatibility surface.
- Keep suspension-safe helper state on typed continuation values. `Monitor`
  refresh and `ValueQ` rollback remain best-effort unless their documented
  contract is deliberately promoted.
- The same-expression evaluation-cycle guard can reject recursive definitions
  whose visible call shape repeats while other session state progresses. Relax
  it only with an explicit progress model that preserves genuine-cycle
  detection.
- Continue splitting formatter planner/builder/render/test modules by
  responsibility as they grow; do not create a separate string crate without a
  real ownership boundary.

### The native stack budget — CLOSED (2026-09-04, briefs 436-438)

The 2026-09-03 ruling above ("`DEFAULT_NATIVE_STACK_LIMIT` at 6 MiB fails open
on a 2 MiB thread where it can never fire") is executed. The route to it was
not the one expected.

brief-436 diagnosed why a deep `StringExpression` construction died at 375
levels: a strict head whose call stays residual and never acquires a stability
proof is re-evaluated through eager argument preparation on every construction
step, one native evaluator level per spine level. The class is wide — `Power`,
`Mod`, `Max`, `Min`, `Equal`, `Less`, `Not` and every metadata-only string head
— and ordinary user-defined heads are outside it.

The gate then found the guard. `poll_native_stack_limit` measures bytes
actually consumed and is the correct instrument; its default was three times
the ruled stack, so the same construction failed in band on 8 MiB and killed
the process on 2 MiB. `SECURITY.md` published the opposite promise, and
`codecs_xml.rs` already carried a workaround comment blaming the 2 MiB debug
test thread — this defect observed in the wild and patched at the call site.

brief-437 measured the frame. One evaluation level is a 4,736 B static packet
across seven resident frames, matching a 4,675 B slope bisected over three
stack sizes. **Three per-level costs, and a budget argument has to say which it
means**: 4.7 KiB release strict, 7.5 KiB release user recursion, 20.6 KiB debug
user recursion. The projected floor from local frame optimization is 1.5-2.6
KiB, against the 511.875 B that height 4,097 on 2 MiB would allow — so the
ruled height for a strict residual call needs an explicit task stack, not frame
polishing. That is open and unscheduled.

brief-438 landed the calibration: `2 MiB - 512 KiB poll headroom - 512 KiB
non-evaluator reserve = 1 MiB`, computed rather than restated, with
`native_stack_limit_for_declared_stack` for hosts that provision more. Four
tests broke and all four were repaired by declaring a thread and deriving the
budget from the same size — depths and `reclim` assertions untouched — which
also retired the `codecs_xml` workaround and restored its acceptance coverage
from 64 back to the full 256.

**The accepted price**: on the default profile a 2 MiB host gets roughly 48
debug / 140 release levels of user recursion, not the nominal 1024.
`recursion_limit` is now documented as a ceiling.

**Known limit, recorded not fixed**: the byte guard polls only at
evaluation-depth increments, so a recursion running entirely inside one builtin
advances no depth and is invisible to it for that whole descent. Converting
such walkers to explicit stacks is what closes that, which is what briefs
422-433 did for ten traversals.

### The deep-rebuild class — stability walk CLOSED (2026-09-04, briefs 439-442)

Re-probing the three remaining traversal items under the new 1 MiB budget found
one shared defect instead of three separate ones.

**The mechanism.** A traversal that rebuilds a deep result hands the evaluator a
tree with fresh `Arc` identities at every level. Every stability cache key
misses, and `call_stability_uncached_with_bloom` walks the whole tree through
`cached_expr_stability_with_bloom` — mutual recursion in a file where
`evaluation_depth` never appears, so the byte guard has no poll point for the
entire descent. That is why `ReplaceAll` on a match, `Apply` at a descending
spec and `Replace` at all levels died the same way, and why the same operations
with a rule that never matches survived: the traversal returns
`Expr::call_shared` with the original pointers and the root's proof still hits.
brief-425's conversions were not wrong; the shared post-result check downstream
of them was never converted.

**The certificate panel had a matching hole.** None of the twelve exercised a
rebuild. `Map` and `Apply` used `{-4097, 0}`, which in rum selects level 0 only
— intended, test-pinned semantics, not a defect — and `ReplaceAll` replaced an
absent symbol. Repaired under a three-part rule: a unique deepest target that
must propagate `changed`, an assertion on the exact rebuilt value rather than a
summary, and a selector pinned separately on a shallow fixture.

**Two repairs were rejected before the third worked.** briefs 440 and 441 made
the walk iterative behind a depth-bounded front. Both were functionally
complete — every invariant covered by differential tests including an exact
cache-insertion-order trace, all twelve certificates green, counters identical
— and both cost. Reproduced in my own four-pass window with within-run
quotients: `downvalue-exact-miss-repeat` +5.6%/+6.9% against a 0.62% base
spread, `downvalue-pattern` +4.4%/+7.1%, `cases-wide` +5.7%/+3.8%,
`matcher-single-tiny` +2.4%/+2.2%. A depth parameter and an `Option` return on
the most-consulted function in the system is not free. Preserved as stashes.

**What worked was moving locals, not control flow.** The walk cost 811 B per
level and its dominant frame was 584 B — not because the function is large but
because the whole chain is inlined into it, making the frame the union of
thirteen data-constructor arms, cache lookup and insertion, head-effect
computation and profiling tokens. A deep descent executes almost none of that.
Outlining them behind `#[cold] #[inline(never)]` gave **255.6 B/level measured**
(bisected at 1024 and 2048 KiB: 3,905 and 8,008 levels), a 3.17x reduction, with
counters byte-identical and the previously-regressed families neutral to
faster.

**brief-443 measured three fail-closed shapes and none fit** — a per-level floor
check grows the frame to 272 B and bails at 3,797; the same check on a stride
holds 256 B but bails at 4,040; a bare depth counter keeps the certificates but
is not stack-size-honest and regresses `do-count-null`. All three would have
turned nine certificates into in-band failures, so a well-measured no was the
outcome. It also found the route out: about 64 bytes of off-by-default
profiling state resident in the hot recursive frame, which is a hard-line
violation independent of any poll. **brief-444 removed it** — the key is derived
inside the cold enabled path and the start instant moved to a thread-local
stack — taking the walk to **192 B/level** (184 B prologue plus the return
address, `/proc` slope 191.1). Height 9,500 now completes on 2 MiB where 8,008
was the ceiling, and the previously untested per-head profiling surface gained
an env-gated certificate that was run against HEAD before the change.

**The walk now fails closed (briefs 445-446).** The strided poll fits once the
frame is 192 B/level: the check sits on the call-cache miss path, immediately
before the recursion, and on budget exhaustion returns a conservative "not
stable" — always safe, because stability only decides whether evaluation may be
skipped, and the value then goes through the evaluator's own guarded path.
Measured at my gate: bail-out bisected at 5,339 ok / 5,341, a margin of 1,242
levels (30%) over the ruled 4,097; height 20,001 on a 2 MiB stack moves from
exit 134 to an in-band `General::reslim`.

brief-445 put the counter at the walk's entry, where it also ran for atoms and
cache hits — neither of which can grow the stack — and cost `list-literal-small`
+7.99%/+7.98%. Moving it to the miss path removed that and is the more honest
meter, since the stride's overshoot arithmetic already assumes it counts
recursive descents. Two codec depth tests surfaced at the same time: they were
passing by consuming more stack than the declared budget permits, and were given
declared threads like their XML sibling rather than having their accepted depth
lowered. The ruled
4,097 contract is met with roughly 2x headroom, but the fail-closed promise is
not. A strided poll inside the walk would convert that into an in-band error at
some hot-path cost that has not been measured. That is the next decision, and it
needs its own measurement rather than an assumption.

**Also still open:** brief-439's steps 2-8, the per-site native conversions
(`Replace`, `MapIndexed`, `D`, exact-path engines, substitution engines, tensor
rebuilders). Their projections rested on brackets measured under the old 6 MiB
budget; they have now been re-probed, and the results, the reordered dispatch
queue, and a correction to the organization-line note are in the last section of
this file.

### The tail re-probed under the 1 MiB budget (2026-09-04, director)

brief-439's gate ruled that steps 2-8 would be re-derived against post-step-1
measurements rather than dispatched from projections
(`docs/dev/codex/brief-439-deep-rebuild-class-design.md:1300-1308`). They now
have those measurements, and they reorder the queue.

Release `rum-cli` at `59410b75`, `ulimit -s 2048`, 1 MiB evaluator budget. Every
probe carries a liveness assertion on the exact rebuilt value — the replaced
leaf, the deleted atom, the factor count, the swapped pair — and was calibrated
at height 3 before any bracket was believed.

| Site | last ok / first fail | mode | against 4,097 | draft |
|---|---|---|---|---|
| `StringExpression` construction | **216 / 224** | in-band | **19x short** | none |
| `Alternatives` construction | 210 / 218 | in-band | 19x short | none |
| `MergeNested` over deep associations | 1,114 / 1,122 | in-band | 3.7x short | 8 |
| `Replace[_, _, {0, Infinity}]` | 1,308 / 1,316 | **process kill** | 3.1x short | 2 |
| `D` over a nested-`Sin` spine | 1,738 / 1,746 | **process kill** | 2.4x short | 4 |
| `DeleteCases[_, _, Infinity]` | 1,808 / 1,816 | **process kill** | 2.3x short | 6 |
| `MapIndexed[ff, _, {0, Infinity}]` | 2,694 / 2,702 | in-band | 1.5x short | 3 |
| `ReplacePart` at a full deep path | 2,854 / 2,862 | **process kill** | 1.4x short | 5 |
| `Reverse[_, All]` | — | — | **clears** | 7 |
| deep association construction | — | — | clears | — |
| `FullForm` (`Expr::Display`) | 5,846 / 5,851 | **process kill** | clears, 1.4x | none |

**Controls, run in the same window.** `ReplaceAll[x, 1 -> 2]` and
`Apply[gg, x, {0, Infinity}]` clear 4,097 with a real rebuild (depth 4,098,
`MemberQ` hit), and `Cases`/`Count`/`Position`/`ToString`/`Level` all clear.
Step 1 and brief-425 are therefore verified independently of their own briefs.

**The instrument was calibrated before any result was believed.** The
classifier was shown printing each of its four verdicts on a known case:
`fRec[n_] := fRec[n + 1]` for in-band, deep `FullForm` for process kill,
`ToString` for pass, and a 200 000-iteration build for the invalid-probe class.
Its first version scored two known failures as `UNKNOWN`.

**Two probes measured nothing and were replaced.** `D[Nest[Sin, q, 4097], q]`
fails on the **fixed-point pass limit (256)**, not the stack — the same trap
`Nest[List, 1, N]` set earlier in this line; building the spine with
`Do[e = Sin[e], {i, N}]` reaches it. `Reverse[x, {1, Infinity}]` is not a legal
rum call (`Reverse` takes `All` or a list of positive levels), so it returned
unevaluated and would have certified a false ceiling. A third guard held: deep
association construction was bracketed first and clears 4,097, so `MergeNested`'s
number is its own and not its fixture's.

**Three findings reorder the queue.**

1. **`Replace`'s ceiling did not move.** 1,308 is identical to brief-439's
   pre-step-1 bracket, which is both the proof that step 1 never touched these
   sites and a consistency check on the harness. Confirmed in source: the
   strings `poll_native_stack_limit` and `evaluation_depth` occur **zero** times
   under `kernel_data/rewrite_data/replace/`, in `structure_data/exact_paths.rs`,
   under `runtime/substitution/`, and in `rum-stdlib/src/calculus.rs`. No poll
   point is why these four end the process instead of raising an error.
2. **Order by failure direction, not by shortfall.** Drafts 2, 4, 6 and 5 fail
   open; drafts 3 and 8 already fail closed and are short only on the contract.
   Draft 7's representative clears 4,097 outright, which matches brief-439
   scoping 7 and 8 as budget-gated rather than unconditional.
3. **The worst shortfall is not in steps 2-8 at all.** The strict-residual
   construction class is at 216, worse than the 375 recorded under the old
   guard, because the budget now fires: the failure direction improved and the
   height fell. `Alternatives` at 210 confirms a class rather than a
   `StringExpression` quirk, and 1 MiB / 213 ≈ 4.9 KiB per level matches
   brief-437's measured 4,736 B evaluation-level packet. brief-437 already
   priced frame polishing out of this (floor 1.5-2.6 KiB against the 511.875 B
   that 4,097 allows), so this is the evaluator task stack — its own design
   line, not a per-site conversion.

**Dispatch order for the tail**: draft 2 (`Replace`), then 4 (calculus), 6
(`DeleteCases` and the substitution engines), 5 (exact paths) — the fail-open
four, in shortfall order — then 3 and 8, with 7 reduced to a confirmation probe.
The evaluator task stack is tracked separately and opens with its own
design-only brief.

**Draft 2 is CLOSED (brief-447, `c71d808a`).** `Replace` keeps a depth-16
recursive front and hands the tail to an explicit work/completed stack; both
depth helpers get a single-chain fast path plus an explicit stack for branching
shapes. Repeated depth work was held out of scope so the bound could be gated
alone. Director-measured: 4,097 completes with the exact rebuilt value, the
ceiling moves 1,308 -> 5,843 (1.43x margin), and above it the failure is an
in-band native stack limit instead of a dead process — the same bracket and the
same direction as `ReplaceAll`, which is the endstate this repair was aiming at.

Two gate notes worth keeping. The worker reported its own two-revision A/B
stop-loss on a residual +0.61% on `replacepart-wide-rule-list`; four passes with
within-run quotients read -3.26% raw and -1.50% quotient on that label, with a
same-binary spread of 1.80%. **The stop-loss was noise, and reporting it rather
than tuning through it was the correct call.** Second, the ceiling is now the
shared stability walk rather than `Replace` itself, so the remaining fail-open
sites should be expected to land in the same 5,800-level bracket once
converted.

**Draft 4 is CLOSED (brief-448, `bd3820c1`), and it refuted its own
projection.** Both of `D`'s walks — the containment prepass that runs once per
visited node, and the four-way differentiation group — keep a depth-16 recursive
front and hand the tail to explicit stacks. `calculus.rs` split into containment
and differentiation modules declared from it, `lib.rs` untouched.

brief-439's draft had written that 4,097 must end in a recoverable in-band
error, and its stop-loss forbade demanding successful evaluation. **It
completes**: `Length[D[e, q]]` is 4,096 with no messages and a usable session
afterwards. The projection predated the stability-walk repair, which is why the
brief asked for the outcome to be measured and asserted rather than assumed —
dispatching the draft as written would have produced a certificate asserting an
error that no longer happens.

**Two costs recorded, not fixed.** Repeated containment work was held out of
scope, so `D` is near-cubic on a deep spine — director-measured 0.146 s at 256,
0.970 s at 512, 7.26 s at 1,024, about 440 s at 4,097 — and the release-only
certificate panel now takes about 460 s end to end. Budget that into any future
gate that runs the panel. Removing the repeated prepass is the work-removal item
this leaves behind, and it sits beside the `Cases` double-normalization entry
above.

**A denominator lesson for the A/B protocol.** The within-run quotient read
+0.96% against the 3 nearest-magnitude neighbors and +2.43% against 12, both
outside their same-binary spreads — while the target's raw movement was -0.06%
and the median over all 46 same-tier labels was -0.18%. A three-label
denominator has no stable median of its own; take the tier median over tens of
labels, or the quotient invents a regression.

**Draft 6 is SPLIT on measurement, and its `DeleteCases` half is CLOSED
(brief-449, `d83929fe`).** Probing the substitution engines before briefing them
found that draft 6's bundle was three different problems:

| site | last ok / first fail | against 4,097 |
|---|---|---|
| `DeleteCases[_, _, Infinity]` | 1,808 / 1,816 | 2.3x short |
| `Function` slot substitution over a deep body | 3,685 / 3,693 | 1.11x short |
| `Module` lexical substitution over a deep body | 3,677 / 3,685 | 1.11x short |
| `With` lexical substitution over a deep body | 3,677 / 3,685 | 1.11x short |
| rule-RHS binding substitution over a deep template | clears 4,097, kills by 8,001 | **not short** |

All are process kills; `substitution/` and `cases/delete/` both have zero poll
points. **The engine draft 6 named first is not short of the contract at all** —
it is merely fail-open above roughly 6,000, which is the lesser problem in this
line's own taxonomy. So `DeleteCases` went alone (its own subtree, no caller
ripple), and the substitution engines get their own brief with their own
numbers and their ten-file consumer surface.

`DeleteCases` now completes at 4,097 and fails closed above 5,408 — the same
bracket as `Replace`, confirming the prediction above that converted sites land
where the shared stability walk binds.

**The second worker A/B stop-loss in a row was noise, and this one needed the
layout control to settle.** The worker reported +7.14% on the new
`deletecases-leaf-delete-small` label after two isolated revisions and stopped.
At my gate, over a six-pass base/candidate/layout window: the target moved
+0.96% against a **+0.87% whole-panel median**, and a layout-control binary —
HEAD with one function relocated inside its `impl` block, semantically identical
— moved the same label's within-run quotient **+1.57%** against the candidate's
+1.51%. The candidate is inside the layout floor. The label also measures a
depth-9 value, which never reaches the iterative tail at all, so the only change
it can see is one integer comparison.

**Consequence for briefing**: best-of-2 without a whole-panel median is not
enough to declare a regression on this panel, and the next brief in this line
must require the worker to report the whole-panel median beside each label.

**Settled by counters instead, and the timing instrument is now known to be
weak on this host (user ruling, 2026-09-04).** This machine is an i9-14900HX
with 8 P-cores and 16 E-cores. One core is unhealthy at full turbo, so a kernel
module caps every core at 5.2 GHz; `scaling_max_freq` reads 5 200 000 on the
P-cores and 4 100 000 on the E-cores, so the clock ratio is **1.27x**, not the
1.41x the spec sheet implies. Nothing pins a benchmark to a core class. Any wall-time percentage here can be a scheduling artifact larger than
the effect being judged, so **a timing A/B is an indication, never proof**.
`CLAUDE.md`'s standing rule "counters beat timings, report counters first" was
not applied at the briefs 447-449 gates; it is reinstated.

Re-measured with `RUM_PROFILE_ALLOC=1` on a `--features alloc-count` build,
pre-447 (`b9cdb56d`) against `a0432c43`, each label as an isolated probe
(fixture plus the exact statement in a `Do` loop):

| probe | pre-447 | HEAD |
|---|---|---|
| empty script (baseline) | 1 572 | 1 572 |
| `Replace[Nest[List,1,8], 1 -> 2, {0, Infinity}]` x500 | 25 466 | 25 466 |
| `DeleteCases[Nest[List,1,8], 1, Infinity]` x500 | 24 915 | 24 915 |
| `D[Nest[Sin, v, 8], v]` x200 | 92 895 | 92 895 |
| `ReplacePart[{0}, rules]` x20 | 379 678 | 379 678 |

Allocation counts identical to the byte, `bytes_total` and `peak_live_bytes`
too. **The disputed +7.14% is settled: the three conversions do no extra
allocation on their labels' shallow paths.** The counter was calibrated for
sensitivity first — one extra `Do` iteration moves it +46 allocations.

**One caveat found while doing it: a counter is load-immune only if the
workload is deterministic.** The whole-panel total is not — two runs of the
identical binary read 9 999 973 and 10 000 216 — because the harness has
time-adaptive parts (the retained-module GC batches keyed off
`AbsoluteTiming`). The base-vs-candidate panel total differed by +530 in 10
million, inside that same-binary spread. Use isolated per-label probes and
calibrate the same-binary spread before believing any counter delta.

**Nothing was overturned by the re-measurement.** The three A/B accepts it
re-checked (briefs 447, 448, 449) are all confirmed; what the timing numbers
were weak evidence *for* turned out to be true.

**The gap the allocation counters left is now closed with the PMU.**
`perf_event_paranoid` is 2 on this host, so `perf stat -e instructions:u`
works unprivileged and sees the pure CPU cost that allocation counts cannot.
Minimum of the tight cluster, migrated runs discarded:

| probe | pre-447 | HEAD | delta |
|---|---|---|---|
| `Replace[Nest[List,1,8], 1 -> 2, {0, Infinity}]` x500 | 43 442 724 | 43 381 754 | **-0.140%** |
| `DeleteCases[Nest[List,1,8], 1, Infinity]` x500 | 37 991 580 | 38 019 336 | **+0.073%** |
| `D[Nest[Sin, v, 8], v]` x200 | 96 361 697 | 96 587 958 | **+0.235%** |

All three are real signals — the within-cluster spread is 0.006% to 0.08%, so
these are 3x to 40x the noise — and all three are negligible. `DeleteCases`
costs about 50 extra instructions per call over a nine-level value, which is
the depth-16 front check running once per node and nothing else. **The disputed
figure was +7.14% from the worker and +0.96% from my own timing window; the
true cost is +0.073%.** `Replace` is faster, not slower.

Three PMU traps, met on first use and worth reusing: the hybrid PMU reports
`cpu_atom` and `cpu_core` separately and a run that **migrates between core
classes reads about 55% high** and must be discarded (both counters non-zero);
`<not counted>` means that PMU never ran; and instruction counts are
core-class independent, so an all-E-core run is directly comparable with an
all-P-core one.

**Draft 5 is CUT DOWN on measurement and its fail-open half is CLOSED
(brief-450, `b7f390ac`).** Probing all nine consumers draft 5 named found that
**six already clear 4,097** — `MapAt`, `ReplaceAt`, `Insert`, `Delete`,
`Extract` and list part assignment — so its shared-primitive refactor across
about fifteen files would have mostly rewritten working code. Two were short and
fail open, and they are what shipped: multi-spec `Take` 2,573 and deep-path
`ReplacePart` 2,854, both now completing at 4,097 with the exact rebuilt value.

The third short site is **left open deliberately**: nested association part
assignment at 1,106 / 1,114, in band. It fails closed, and it brackets
identically to `MergeNested` (1,114 / 1,122), which points at a shared
association-side cause worth diagnosing once rather than patching twice.

**This was the first brief to forbid a wall-time A/B**, and the protocol worked
on both sides. The worker reported identical allocation counts with a
sensitivity calibration, instruction counts from clean clusters, and the
migrated runs it discarded — no stop-loss, no revisions chasing noise. At my
gate the instruments agreed: allocation counts identical, instructions **-1.65%
on `Take` and -0.94% on `ReplacePart`** against a 0.006% within-cluster spread,
so both operations are slightly *faster*.

**Draft 6's substitution half is CLOSED (briefs 451-453, `7acff251`), and it is
the first change this line accepted over its own stated bar.** The lexical
engine ended the process at 3,385 — `With`, `Module` and a named-parameter
`Function` share that bracket through `substitute_lexical_bindings_changed` —
and rule binding at 4,430. Both now complete at 4,097 with the exact substituted
value and fail closed above 5,867 and 5,843, the same bracket as every other
converted site.

Two corrections the re-probe forced, both of which moved the scope:
**slot substitution was never short** — the earlier "3,685" figure came from a
probe using a *named parameter*, which routes through the lexical engine; a true
`Slot[1]` probe clears 4,097 and fails closed at 5,867 — and **rule binding was
in scope after all**, because its 1.08x margin is thinner than the ~8% the
lexical bracket drifted over fourteen unrelated commits. A margin thinner than
the drift is not a margin.

**The cost, and why it was accepted.** The ordinary `With[{x = 1}, x + 1]` path
costs **+0.714% instructions**; `Module` is -0.083% and rule RHS -0.159%, and
allocation counts are unchanged. That is over the ±0.15% bar brief-452 set, and
it is accepted on measurement rather than waived: two shapes were built and
measured, and the alternative — a thread-local depth — was **slower** at +1.085%
and carried five manual `enter`/`leave` pairs with no `Drop` guard, a shape
whose breakage would be a silently shrinking recursion budget for the rest of a
thread rather than a failure. The accepted shape puts the depth in a
`LexicalWalk` context struct beside the bindings and the shadow set, so the arms
take *fewer* arguments than before the bound existed.

**Two method notes.** The instrument is what made this decision possible at all:
+0.714% is invisible under this host's 27% scheduling spread, so on wall time
this would have shipped unexamined. And the semantics at risk was the per-subtree
shadow set — several arms build a different set for one child than for its
siblings — which is now pinned by a regression at depth 20, past the recursive
front, asserting that an inner `With` shadows only inside its own subtree.

**Recorded as an unexplained sighting, not a diagnosis: two dispatches of
brief-452 were terminated mid-run**, each after its edits had landed and before
any verification. What is established is only what it was *not*: the three
recorded content-filter kills (briefs 422, 424, 439) ended with `exit 1` and a
policy message in the log tail, while both of these ended with **no `codex
exit=` line at all**, and neither log contains any filter text. It was not a
size or duration ceiling either — brief-451 succeeded with a 15.3 MB log over
18m40s, while these died at 623 KB. The only error present is
`codex_models_manager: failed to refresh available models: timeout waiting for
child process to exit`. That is suggestive of the connection, but this failure
has not been seen before on this project and the cause is **open** (user, first
occurrence). Record the next sighting rather than explaining this one away.

**Third sighting, 2026-09-05, brief-456.** Same signature: the log stops mid
output at 193 KB with no error text and no `codex exit=` line, and the process
exits some time later having applied only its first edit. It ran 27 minutes and
died about 25 minutes *after* an unrelated SIGTERM of mine had killed the
foreground wrapper, so it is not that either. The three failures are all
**early** — 193 KB to 636 KB of log — while every completed dispatch in this
line ran to between 745 KB and 15.3 MB, so it is not a size or duration
ceiling. What it costs is bounded: the tree held only unvalidated falsifier
tests, which were discarded rather than kept.

The recovery that worked was to shrink the brief — brief-453 changed one file
and ran only `fmt` plus two crate suites and two instruction probes, with the
workspace run and the release panel kept on the director's side.

**Re-examined 2026-09-05 after the user reported seeing, in other Codex
sessions, `Falling back from WebSockets to HTTPS transport` and
`stream disconnected before completion: websocket closed by server before
response.completed`. That is now the leading hypothesis, and one line above is
retracted.**

- **Retracted.** "The only error present is `codex_models_manager: … timeout` …
  That is suggestive of the connection" is wrong. That error appears in
  *successful* runs too — five times in brief-455, which completed, and also in
  briefs 445, 446, 447, 449, 451, 454 and 457 — while briefs 448, 450, 453 and
  456 have none. In the failed brief-452 log it sits at line 7,509 of 12,110,
  38% short of the truncation. It is background noise.
- **What Codex's own session record shows.** The rollout for the failed run
  (`~/.codex/sessions/2026/09/04/rollout-2026-09-04T20-26-46-*.jsonl`) has 243
  records and its **last one is a `reasoning` response_item, mid-turn**. No
  error record, no abort record, no completion record. That is the shape a
  stream cut leaves.
- **Why the absence of the message proved nothing.** codex logs through
  `tracing` and at its default emits **only ERROR**: across fourteen dispatch
  logs and about 1.5 million captured lines, the entire tracing output is 25
  ERROR lines from two targets, with no WARN, INFO or DEBUG at all. A transport
  fallback notice is WARN or INFO and could never have appeared in what we
  capture. **The earlier "cause is open" was declared on an instrument that can
  only see ERROR** — the same failure this line keeps making, believing a
  negative from an uncalibrated probe.
- **The instrument is fixed rather than the sighting explained.**
  `scripts/dispatch-brief.sh` now exports `RUST_LOG` (default
  `warn,codex_core=info`, overridable with `BRIEF_RUST_LOG`), verified to reach
  the worker through the interactive fish shell and the `with_proxy` wrapper.
  It is exported rather than prefixed because the worker is a shell function:
  `env VAR=v cdx …` cannot exec a function, and fish has no prefix-assignment
  syntax. The same latent bug was fixed in the `cgl` branch, where
  `CGL_MAX_OUTPUT_TOKENS` had been passed as a prefix assignment that fish
  would not have applied.

The cause remains formally **open** — the specific message has not been seen in
our own logs — but the next occurrence should now be explicable rather than
merely recorded.

**Draft A of brief-454 is CLOSED (brief-455, `9f8e89b2`).** The association arm
of the stability walk is bounded: an association-rooted chain takes over the
whole subtree once the walk meets an `Association`, keeps a depth-16 borrowed
front, and carries its depth through intervening ordinary calls — which is what
the `Map` result needs, since it produces `ff[Association[Rule[…]]]` at every
level. The generic chain is untouched.

`Map` over a deep association and a read after a nested key-path assignment now
complete at raw physical height 4,097 (`N=2048` wrappers), up from 966 and
1,145. `Map` runs to the construction ceiling at 5,457 wrappers, so the
operation is no longer the limit — the fixture is.

**A measurement limit learned here — and my first reading of it was wrong,
corrected at the brief-456 gate.** My generic probes read -7% to -8.7% on paths
this change cannot touch, while the worker's read within ±0.15%. I attributed
that to codegen from the module growing 444 KB. **It was the build path.** I
compared a binary built in a `git worktree` under `/tmp/…` against one built in
`/home/remi/dev/rum`; rebuilding the candidate under a same-shaped `/tmp/…` path
brings the unrelated probe from -8.42% to **+0.005%**. The tell was already in
the data and I missed it: briefs 455 and 456 are unrelated changes and produced
*the same* -8.4% on *the same* untouched probes.

**The rule: build base and candidate at the same filesystem path, or
instruction counts are not comparable.** Allocation counts are path-immune —
identical to the byte across the boundary in both gates — so pair every
instruction number with them. The worker's numbers were right throughout,
because it built both arms in one path.

**Draft B is CLOSED (brief-456, `d0e80b1b`), and the re-probe that opened it
found the failure mode had flipped.** After brief-455, `MergeNested` went from
1,153 wrappers failing **in band** to 2,027 **terminating the process** — the
association stability arm used to exhaust the byte budget first, and that arm
polls; once it was cheap, `MergeNested`'s own unpolled recursion reached the raw
stack instead. That moved it from 3.6x short and fail-closed to 1.01x short and
fail-open. **Removing a guard-adjacent cost can convert a fail-closed site into
a fail-open one** — worth carrying into the remaining drafts.

The walk is now explicit enter/finish work items behind a bounded front, with
suspension freezing the whole stack. It completes at raw 4,097 and runs to the
construction ceiling at 5,457 wrappers; instructions are neutral (-0.119% on the
merge probe, +0.005% on an unrelated control) with allocation counts identical.

**Drafts C-J were re-derived and re-measured (2026-09-05, brief-457 plus the
director's brackets), and six of the eight drafts are DROPPED.** brief-455 was
the shared prerequisite every one of them was priced against, so removing it
re-priced the whole class. Codex re-derived the plan from source; the brackets
below are the director's, measured in parallel because a past-ceiling probe is
not delegable in this line.

Release CLI at `80c8acce`, `ulimit -s 2048`, 1 MiB evaluator budget, heights in
**physical levels**. A verdict is 3/3 runs; see the measurement note below.

| draft | site | at 4,097 | ok <= | fail >= | verdict |
|---|---|---|---|---|---|
| **C** | `ConformSchema` / `SchemaErrors` | in band | 208 | 224 | **19.7x short**, fail-closed |
| **E** | multi-spec `GroupBy` | kill | 2,016 | 2,032 | **2.03x short, fail-open** |
| **J'** | `KeyTake` over nested lists | kill | 2,856 | 2,864 | **1.43x short, fail-open** |
| 439-3 | `MapIndexed` generic list shape | in band | 2,944 | 2,952 | 1.39x short, fail-closed |
| **J'** | `Keys` over nested lists | kill | 3,056 | 3,080 | **1.34x short, fail-open** |
| D | `DeleteMissing` | pass | 6,272 | 6,304 | clears 1.53x, **kills above** |
| H | `AssociationKeyDeflatten` | pass | 7,344 | 7,376 | clears 1.79x, kills above |
| I | `MapIndexed` association shape | pass | 7,344 | 7,376 | clears 1.79x, kills above |
| G | `AssociateTo` / `KeyDropFrom` Part target | pass | 8,048 | 8,080 | clears 1.96x, kills above |
| H | `AssociationKeyFlatten` | pass | 8,292 | 8,334 | clears 2.02x, kills above |
| F | nested key-path assignment | pass | >= 10,800 | — | fixture-bound |
| I | `Scan` association shape | pass | >= 10,800 | — | fixture-bound |
| J | `Merge` collection parser | pass | 16,072 | 16,128 | clears 3.92x |
| ctl | `Map`, `MergeNested` | pass | >= 10,800 | — | confirms 455/456 independently |

**Four findings, in the order they change the queue.**

1. **The direction flip is the whole class, not `MergeNested`.** Every site that
   now clears the contract **terminates the process** above it; not one fails in
   band. brief-455 lifted them over 4,097 and removed the polling frame they
   were borrowing in the same stroke. The contract is met with 1.5x-2x margin
   and the fail-closed promise is not, which is the same shape as the stability
   walk before briefs 445-446 — and it is now the largest open item in the
   class, ahead of any individual site.
2. **Draft C's axis is schema depth, not value depth, and its packet is
   ten times what was reasoned.** Decomposed: a deep value against a shallow
   schema (`_`, or one level) clears `N=2049` outright; only a nested *schema*
   descends. 1 MiB / 104 pairs is roughly 10 KiB per pair, the scale of an
   evaluation-level packet (brief-437: 4.7-7.5 KiB), not of the 1,056 B native
   frame brief-454 priced. So the validator crosses the evaluator at **every
   nested pair**, not only at the leaf as both brief-454 and brief-457 reasoned
   — which is also why it fails closed. That mechanism should be confirmed in
   source before the repair is briefed.
3. **`Keys` and `KeyTake` over nested lists are short and fail-open**, and they
   are in the one lane brief-454 deliberately deferred ("should be briefed in
   its generic collection-shape lane rather than silently bundled here").
   brief-454 reasoned `KeyTake` at about 2,900 (right) and `Keys` at about 6,200
   (**2x optimistic, and the verdict was wrong**). Draft J as scoped — the
   parsers — is correctly dropped; the deferred sibling lane is not a note any
   more, it is two fail-open shortfalls.
4. **`GroupBy`'s failure direction does not depend on the callback**, which was
   brief-457's own largest stated risk. A callback consuming evaluator depth
   (`Function[u, deepf[u]]` over a `Module` + `Nest`) stays a process kill and
   *lowers* the ceiling below 1,008 specs rather than converting it to an
   in-band error.

**The re-derivation's accuracy, since it decides how much to trust the next
one.** Ten of twelve reasoned projections landed within 2-25% of measurement,
four of them within 2% (`Merge` parser 16,384 vs 16,072; exact-path replacement
8,192 vs 8,048; `AssociationKeyFlatten` 4,228 vs 4,146; association `MapIndexed`
3,745 vs 3,672), and it correctly picked the binding mode inside `DeleteMissing`
before measuring it. The two failures are both in the same direction — an
evaluator crossing priced as a native frame: schema (19x, and the direction
wrong) and `Keys` (2x, and the verdict wrong). **Reasoned packet arithmetic is
reliable for an operation-local native walk and unreliable the moment the
descent crosses the evaluator.**

**The re-ordered queue**, fail-open before fail-closed, then by shortfall:

1. The class-wide fail-open direction above the contract (new; supersedes
   ranking the individual passing sites).
2. Draft E, multi-spec `GroupBy` — 2.03x short, fail-open.
3. The deferred collection-shape lane — `Keys` and `KeyTake` over nested lists,
   1.34x and 1.43x short, fail-open.
4. brief-439 draft 3, generic-shape `MapIndexed` — 1.39x short, fail-closed.
   One source walker with draft I, but the surviving defect is the generic
   post-result path, so only an evaluator-side task stack lifts it.
5. Draft C, the schema validator — 19.7x short but fail-closed, and on its own
   axis; re-diagnose the per-pair evaluator crossing before briefing a repair.

**DROPPED as drafted**: D, F, G, H, I, J. Each one's priced target now clears,
and rewriting a passing site is what brief-450 established not to do.

**A measurement note that applies to every bracket in this line.** The ceiling
is **not deterministic**: inside a band a few tens of levels wide the same `N`
flips run to run (`DeleteMissing` at 3,144 read 3 PASS / 2 KILL over five runs),
because the stack base moves between processes. A single-run bisection therefore
reports a bracket 8 wide when the real uncertainty is nearer 50. Every number
above is the highest `N` passing 3/3 and the lowest failing 3/3, with the
failure direction read well inside the stable-fail region, never from the band.

**Two probes that measured nothing, recorded so they are not repeated.** A
literal deep association source (`<|"k" -> <|"k" -> ... |>|>`) cannot reach the
association parser at all — rum refuses source nesting past 128
(`parse nesting exceeded limit 128`) — so draft J's parser site is reachable only
through construction, which clears. And `KeyDropFrom` on an association `Part`
target requires `Key["k"]` selectors; with bare string selectors it returns
`KeyDropFrom::invak`, which reads exactly like a depth failure and is not one.
`AssociateTo` accepts both, so the two heads differ in strictness — worth a
look, but not a defect on this evidence.

### The class-wide fail-open repair — step 1 is IN (2026-09-05, brief-459)

The guard substrate is installed and has one real consumer. `Merge` over a
collection nested past the budget now returns an in-band `StackLimitExceeded`
and leaves the session usable, where it previously terminated the process, and
it still completes at raw physical height 4,097 with the exact merged value.

**Two rulings were made at the design gate and are now implemented.**

- **The operation boundary is `native_stack_limit`, 1 MiB on the default
  profile.** brief-458 proposed `native_stack_limit +
  NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES` = 1.5 MiB. Rejected: consumption is
  measured from `native_stack_base`, set at the request base
  (`resolution.rs:292`), so the 512 KiB non-evaluator reserve covers frames
  *below* that base and spending it above the base counts the same bytes twice.
  A host that actually used its reserve would overflow — the same shape as the
  6 MiB default that could never fire on a 2 MiB stack.
- **`Merge` is guarded; the other five sites are converted, not guarded.**
  Director-measured under a 1 MiB budget at the contract height, three runs
  each: `DeleteMissing`, `AssociationKeyDeflatten`, association `MapIndexed` and
  `AssociateTo` cannot do the work at all, and `AssociationKeyFlatten` sits on
  the line. Only `Merge`'s collection parser passes 3/3. **A guard at the honest
  budget would have converted four contract-meeting sites into contract-failing
  ones** — trading a fail-open above the contract for a fail-closed below it,
  which is strictly worse. That is what makes conversion the repair for the
  other five, and it is also why `Merge` was the right first consumer: it proves
  the mechanism and the boundary together.

**The shape of the substrate.** A scoped runner captures base and limit once and
owns a sticky exhaustion flag; a branded `NativeStackDepth<'scope>` yields one
child token per recursive edge, polling actual bytes every fourth edge through a
`#[cold] #[inline(never)]` check that returns before the recursion. Logical
depth is a stride only, never a refusal criterion — the bare-counter shape
brief-443 measured and rejected. `kernel_api` exports exactly two
`#[doc(hidden)]` items and no raw poll, base, limit, address, or constructor.
Exhaustion travels through the parser's existing `Result`, so no per-node
`Result` was introduced — the shape that cost 6-15% in brief-431.

**Falsifiers, all four verified at the gate rather than reported.** Two
`compile_fail` doctests (the token cannot enter a `FrameRootTrace` frame, and
cannot escape the HRTB scope) pass in the workspace run; the zero-boundary test
returns `TerminatedEvaluation["StackLimit"]` in band with the session usable;
and the **red/green was reproduced by the director** — removing the guard from
`data.rs:186` turns the focused test red, restoring it turns it green.

**Cost, counters first.** Allocation counts are identical to the byte on all
three probes with a same-binary spread of zero, and the counter was calibrated
first: one extra `Do` iteration moves it +35. Instructions: the untouched
control is -0.020% against a 0.13-0.20% cluster spread, and the `Merge` probe is
**+0.381%** against a 0.098% cluster spread — a real signal, about 15
instructions per guarded edge, on a probe built so guarded edges dominate
(depth 8, 500 iterations, nothing else). The microbench panel was **not** run,
and the reason is that it cannot see this change: `Merge[` occurs zero times in
`scripts/core-language-microbench.sh`, and its one merge label exercises
`MergeNested`, a different operation whose parser is untouched.

**Step 2 is IN (2026-09-05, brief-460): the class is now a closed
declaration.** Two enforcement layers, tests only, no behavior change.

- **The surface registry** — **44 rows** covering both association catalogs
  plus the cross-family axes (`Map`, `Apply`, `MapIndexed`, `Scan`, `Set`,
  keyed `Part`). *(Corrected: this entry first said 52, from a grep that counted
  the enum declaration and the vocabulary list alongside the rows — the
  "a script-derived count is a claim, not a fact" rule, violated in the same
  session that wrote it into a brief.)* The catalog comparison
  is **bidirectional and live** — director-verified by deleting the
  `DeleteMissing` row, which fails with `catalog entries without rows:
  ["DeleteMissing"]`.
- **The recursion inventory** — a `syn` scan over `associations/**`,
  `functional_ops/apply_map/**`, `association_data.rs` and `exact_paths.rs`,
  exhaustive within those roots with no per-function allowlist, comparing exact
  `(path, caller, callee, occurrence count)` rows. Blind spots are documented in
  the test itself: trait-object and function-pointer dispatch, macro-generated
  recursion, and anything outside those roots.

**Each row carries two orthogonal fields and neither may borrow from the
other** — mechanism today, and measured contract status with failure direction
and the brief that owes the work. brief-458's draft had proposed `GuardedNative`
for `Scan` and nested key-path `Set`, which are not guarded; the registry
records them as `UnguardedNative + MeetsContract + Unmeasured` with an owed
queue item. `Unmeasured` is the honest entry: those two clear the association
fixture's own cap, so their direction above it has never been observed, and the
brief made "this cell is unmeasured" an acceptable deliverable rather than a
gap to fill with a guess.

**Both falsifiers were reproduced at the gate, not accepted on report.** The
registry's checked-in `#[should_panic]` misdeclares the real catalog entry
`AssociationKeyFlatten` and must name its real edge
(`key_reshape.rs`, `flatten_association_entries`, count 1). For the inventory
the director added one extra syntactic recursive call to a real function and
captured the red — `("crates/rum-stdlib/src/associations/data.rs",
"parse_merge_keyed_entries_collection_with_diagnostics", …, 2)` against the
declared 1 — then restored it and captured green.

**A gate note worth keeping.** The worker's own workspace run needed
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, so its green did not cover the nine native
ZeroMQ tests. The director's run, on this loopback-capable host and without
that variable, is 2 209 tests green. This is the standing rule that a worker's
gate is worth exactly what its sandbox can execute — and it was **visible only
because the dispatcher now captures WARN**: the run that first hit it logged
`WARN codex_sandboxing::violation: … operation_not_permitted` and exited 101,
a line that would have been invisible under the previous ERROR-only capture.

**Deferred, with a re-open condition**: the `rum-derive` compile-time group
validator that would make an unwrapped recursive edge a build error rather than
a test failure. It is stronger, and it is a separate correctness surface whose
gate should not be shared with the registry's; meanwhile the inventory covers
the same class, because an edge added inside a guarded function changes that
function's occurrence count. Re-open it when the guarded population stops being
small — today `Merge`, with `Scan` and nested key-path `Set` measured
guard-eligible — or when a conversion is rejected and a site stays natively
recursive by decision rather than by schedule.

**The two remaining guards are IN (2026-09-05, brief-461).** Association `Scan`
and nested key-path `Set` now return an in-band `StackLimitExceeded` with a
usable session where they terminated the process, and both still complete at
raw physical height 4,097 — which on a 2 MiB thread exercises the guard at its
real 1 MiB boundary, so the certificate is the direct proof that guarding costs
no contract height. Registry rows flipped to `GuardedNative` with
`FailureDirection::FailsClosed` and `owed_by` cleared; the distribution is now
23 `NoProportionalDescent`, 15 `UnguardedNative`, 3 `BoundedHeap`, 3
`GuardedNative`. **Every remaining `UnguardedNative` row is a conversion, not a
guard.**

Both recursive workers return a plain value rather than a `Result`, so they use
the substrate's throwaway-fallback path — `ScanCollection { parts: vec![],
depth: 1 }` and `AssociationSetUpdate::Unchanged` — with the sticky bit giving
the error precedence. No per-node `Result` was introduced anywhere in this line.

**Scope closure is the property that mattered, and it is visible in the diff
rather than asserted.** `Scan`'s guard closes at the `?` before
`evaluate_scan_parts` runs the user callbacks, so no scope is open across a
possible suspension; `Set`'s closes before the `match update` that writes
`definitions`, so an exhausted path cannot half-write.

**Four edges, four reds, reproduced at the gate.** Removing the wrapper from
each real edge in turn — `traversal.rs` association values, head and arguments,
and `association_data.rs`'s key-path edge — turns its focused test red; all four
did. An edge whose removal leaves the suite green has no coverage, and that is
the check that finds it.

**Cost.** Allocation counts are identical to the byte on all four probes,
including `bytes_total` and `peak_live_bytes`. Instructions: `Scan` +0.082%,
key-path `Set` +0.175%, untouched control -0.013%, and — the check this brief
existed to force — the **`Map` control, which shares the `apply_map` machinery
three of the four edges live in, moves +0.002%**. Clusters were tight on every
row.

**The first conversion is HALF IN (2026-09-05, briefs 462-463).**
`DeleteMissing`'s recursive-mode walkers, both pairs of delayed-wrapper
counters, the memory preflight projection and the missing scan are now bounded
by a depth-16 front plus explicit work stacks. **The finite-target/resume family
is untouched, so the registry row stays `UnguardedNative`** with `owed_by`
naming brief-464 — a row that said `BoundedHeap` while a reachable path still
recursed would be the reclassification failure this project has already paid
for.

**Measured before briefing, and it corrected three earlier claims.**
`DeleteMissing[expr, level]` is the *recursive* mode — `Target` is the
**three-argument** form — so brief-454's and brief-457's per-mode reasoning
rested on a distinction that a two-argument probe silently gets wrong. The
memory preflight is unreachable unless a limit is armed
(`missing.rs:89-91`), and armed it costs essentially nothing extra (6,240
against 6,272 physical), the opposite of brief-457's risk 3. And the target mode
is 1.27x costlier per level, not the 5x its recorded packets implied. Brackets,
all fail-open: recursive 6,272, armed 6,240, target 4,944, target with scan
`All` 4,928.

**brief-462 stop-lossed correctly and the gate found two things it had not.**

- *Its failing certificate was a bad probe.* It used an absolute
  `MemoryConstrained[…, 1000000000]` while `MemoryInUse[]` at height 4,097 is
  **2 810 469 079 bytes**, so the call aborted before running and returned
  `$Aborted`. It had copied the two-argument form from its own shallow
  companion, where 1e9 is above usage. The allowance had to be relative.
- *A default-path allocation regression of +1 per association level*, invisible
  to the worker because it never reached the cost gate. The base's
  `entries.into_iter().filter_map(…).collect()` can be satisfied **in place,
  reusing the source buffer**; the rewrite rebuilt into a fresh
  `Vec::with_capacity` and lost that on every level. The sibling list walk was
  not affected — its base iterated a borrowed slice and always allocated.
- *The forced-front falsifier the brief required was never delivered*, which
  mattered because every certificate fixture is a single chain: branch handling
  in the heap tail was untested.

**brief-463 closed all three**, and its falsifier is better than the shape I
specified: instead of a test-only cargo feature it calls the private
`*_with_front_depth` directly with a front of zero, asserts that front zero
really reports `Exhausted` and front 16 reports `Complete`, then asserts
**tail == front == expected** on a three-entry-per-level fixture with a
per-level witness. Director-verified that it fires: reversing the tail's entry
order fails it with `keep` and `k` transposed.

**Gate.** 2 379 tests green on a loopback-capable host with no ZeroMQ skip;
three release certificates at raw physical height 4,097 including the armed one;
allocation counts back to base **byte-identical** on all three probes;
instructions `-0.639%` on the recursive probe, `+0.003%` armed, `+0.001%`
control. The split landed by responsibility — `counting.rs` 14 493,
`deletion.rs` 16 929, `projection.rs` 7 348 — after brief-462's single
`traversal.rs` came in at 35 131, over the architecture-debt line.

**brief-464 planned the finite-target/resume family (design-only), and the
measurement that preceded it found a second defect.**

**A flat list of 3,000 `Missing[]` entries at the target level terminates the
process** — `DeleteMissing[Table[Missing[], {N}], 1, 0]`, stable ok <= 2,921
siblings and fail >= 2,968, same bracket for the association shape. That value
has **physical height 2**. It is not deep, not adversarial, and not near any
documented limit. brief-454 had written these edges off — *"the many `resume_*`
calls that advance `index + 1` are loops or tail transitions over siblings, not
tree height"* — and Rust does not guarantee tail-call elimination, so they are
not loops. **A reasoned claim about what the compiler will do is a hypothesis.**

The recursive mode, converted in the same file by briefs 462-463, now clears
**64,000** siblings as well as its depth certificate: one conversion closed both
of its axes.

**Three things the plan established that change how the repair must be built.**

- *The bounded-front-then-restart shape this whole line has used does not work
  here.* Every previous conversion lets the cold tail redo the traversal after
  the front exhausts. Here a target-level delayed evaluation runs user code and
  can mutate state before a later child exhausts, so a restart would re-run it.
  **The exhausted handoff must carry the live continuation and the completed
  prefixes.** That is materially harder than every conversion before it.
- *One conversion can close both axes, but only if it does both jobs.* A
  depth-only bounded front leaves the measured width edges at `:256` and `:463`
  native; converting siblings alone leaves the measured child descent at `:562`
  native. Sibling advance becomes an explicit `loop`, not a tail-call assumption.
- *The observable contract is eight items*, each cited in the plan: visit order,
  the rule-like short circuit ahead of the scan, syntactic-`Missing` drop before
  any delayed evaluation, **each reached `RuleDelayed` RHS evaluated exactly
  once**, the delayed-wrapper rule above target level, leaf and head
  preservation, what a suspension resumes into, and pure frame abandonment.

**My axis reading was corrected in both directions.** The depth edges I named
were right but incomplete (add `:383`, `:393`, `:722`, `:732`); so were the
sibling edges (eleven more). The classification rule is checkable from the
argument lists: a sibling edge keeps `level` and passes `index + 1`, a depth edge
passes `level - 1` and resets index and accumulator.

**No other registry row is dishonest, and that negative was calibrated.** Ten
other association sites share the textual `index + 1` idiom, but each
continuation lands in a function containing an explicit `loop`, established from
control flow rather than from assuming tail-call elimination. Director
spot-check at width 10,000, where `DeleteMissing`'s target mode is a stable
process kill: `Counts`, `AssociationMap` and `KeySelect` all pass 3/3. The probe
could have contradicted the survey and did not.

**Open, and needing measurement rather than argument**: the five
post-suspension functions and eight heap frames have no bracket at all, because
no supplied probe requests input. Their correctness has to come from forced-tail
multi-suspension tests, not from a number.

**Step A is IN (2026-09-05, brief-465): the semantic net is under the
conversion.** Ten tests, test-only, written against the untouched
implementation — exact value and order for both parent shapes and both `scan`
settings, `RuleDelayed` reached exactly once, the `level - 1` delayed wrapper,
resume without replay at the first, middle and last sibling, two suspensions in
different branches, non-input error propagation, and cancellation discarding
frames without callbacks.

**The falsifier method was corrected from the plan's draft, and the correction
is the point.** brief-464 proposed perturbing the tests' *expected values* and
checking they fail. That is not a falsifier: a test whose expectation you
corrupt always fails, which demonstrates the assertion is wired up and says
nothing about detection power. The brief instead named a minimal perturbation to
the **production source** per contract item, each reverted afterwards.

**Director-reproduced, and the selectivity is what matters.** Perturbing the
post-suspension resume to restart its level at index 0 turns **all four**
resume/replay tests red while the six non-suspension tests stay green. Prefix
replay on resume is the exact failure the next step's shape risks — a handoff
that carries a live continuation instead of restarting — so a net that catches
it precisely, and nothing else, is the thing this step existed to build.

Item 8 got the honest answer the brief asked for: the error-propagation half has
a behavioural perturbation, while the pure-discard half is structural, supplied
by `PureFrameAbandon`, with a cancellation test asserting no continuation
callback runs when those frames are discarded.

Gate: 2 389 tests green, `git diff` is test files and wiring only, no production
edit survived.

**Step B is IN, and `DeleteMissing` is `BoundedHeap` (2026-09-05, briefs
466-467).** Sibling advance is an explicit loop, child descent has a depth-16
front over a cold work stack, and the exhausted handoff carries live cursors and
completed prefixes. The target machine moved to
`crates/rum-stdlib/src/associations/missing/target.rs`; `missing.rs` is down to
8 113 bytes and `resume_frames.rs` to 5 573, since the eight frames collapse to
one that freezes the whole work vector.

**Both axes are closed, verified at the CLI.** Width 3,000 — which terminated
the process before — and width 64,000 both complete; depth 4,097 and 8,000 both
complete, against a previous ceiling of 4,944. Four release certificates cover
depth at both `scan` settings and 64,000 siblings in both container shapes.

**The ten semantic locks passed unmodified**, which was the acceptance criterion
and the whole reason step A existed. Allocation counts are byte-identical to
base on a shallow probe, a 2,000-sibling wide probe and an untouched control.

**The crux property turned out to be unobservable, and that is the finding.**
brief-464 identified the carried handoff as this conversion's crux: unlike every
previous conversion, a cold tail that redid the traversal could re-run user code.
At the gate, perturbing the entry point to discard its carried work and restart
from the root left **890 rum-stdlib tests green** — locks, certificates and both
in-module handoff tests. brief-467 was dispatched to close that, and stop-lossed
with a proof instead: **delayed right-hand sides are evaluated only at
`level == 1`, while front exhaustion happens only at remaining front depth zero,
which is still `level > 1`.** On the only path that has a front, exhaustion
strictly precedes any user-code evaluation. The director confirmed the other
half: `resume_delete_missing_target_work` drives entirely on the heap with no
front at all, including after a suspension resumes into it.

So a restarting handoff would redo pure structural work and be invisible. The
carried shape is still the right one — it is also cheaper — but its correctness
is **structural, not test-enforced**, and the invariant is now recorded beside
the handoff so that a change which lets user code run before an exhaustion knows
it has re-opened the question. The in-module test that claimed to cover this was
renamed to
`resume_target_work_given_a_carried_prefix_does_not_replay_it`, which is what it
actually establishes.

Gate: 2 392 tests green without the ZeroMQ skip.

**The key-reshape conversions are IN (2026-09-05, brief-468).**
`AssociationKeyFlatten` and `AssociationKeyDeflatten` are `BoundedHeap`: three
walks converted — the flatten walk, deflatten's insertion phase, and its
mutually recursive render pair — each behind its own depth-16 front over a heap
tail. Certificates at raw physical height 4,097 for both operations plus a
round trip that exercises all three walks in one value; three forced-front
falsifiers, one of which the director reproduced red by reversing the render
tail's assembly order. Allocation counts byte-identical to base on shallow
flatten, shallow deflatten, a 4,000-entry wide probe, and an untouched control.

**Width was measured clean before briefing.** All four remaining conversion
sites pass 3/3 at 1,000 through 64,000 entries, so `DeleteMissing`'s target
family was the only site with the width defect. This brief was depth-only, and
the negative was calibrated — the same harness caught `DeleteMissing` at 3,000.

**The brief told the worker to use the simple shape and not to build
`DeleteMissing`'s carried-continuation handoff**, on a fact checkable from the
signatures: none of the three walkers takes a `Session`, so none can run user
code mid-walk and a cold tail may redo the traversal. Scope is set by the
problem; the harder shape would have bought nothing.

### What the cross-comparison found (2026-09-05)

The first dispatch was interrupted mid-self-verification. Its work was kept
outside the repository and outside `/tmp` — both inside the worker's fence — and
the brief was re-dispatched from a clean tree, giving two independent
implementations of the same brief.

**They converged on the architecture**: both chose three separate front
constants rather than one shared, both produced the same three loops and the
same render work enum, within 7% on file size. That is evidence the brief was
specific enough to determine the design.

**They diverged on one thing, and it is a real defect the second caught.**
`NestedPathNode::Branch(NestedPathNodes)` is recursive, so its derived `Drop`
recurses. On the deflatten error path a partially built tree is dropped, and the
first attempt let that drop recursively while the second added
`drop_nested_path_nodes_iterative`. Director-measured on the same calibrated
probe — a deep good path followed by an empty-list key, which is the **only**
input `path_from_key` rejects: **attempt 1 terminates the process at depth
60,000 and attempt 2 completes.** The second implementation is also cleaner on a
second axis, generalising `BoundedTraversal<T>` with `Exhausted(T)` where the
first added a separate `InsertPathTraversal` enum.

**The gap is closed (director, same day).** That iterative drop was
unrequested hardening the worker added on its own; it closed a real fail-open
and had no test, so a future refactor could have reverted it silently. The
certificate now exists — a 60 000-element good path followed by an empty-list
key, on a declared 2 MiB thread, asserting the call returns unevaluated with its
argument intact. **Verified to fire**: replacing both
`drop_nested_path_nodes_iterative` calls with ordinary drops overflows the
stack in the named certificate thread.

The depth is measured rather than chosen — a recursive drop survives 20 000 and
dies at 60 000 on that thread — and the comment records the trigger, because my
own first probe measured nothing: `path_from_key` treats a non-list key as a
one-element path and rejects **only** `{}`.

**Association `MapIndexed` is `BoundedHeap` (2026-09-05, brief-469).** Three
edges — association values, the head, and arguments — behind a depth-16 front
over a heap tail, in a new `apply_map/traversal/map_indexed.rs`. The plain
certificate completes at raw physical height 4,097 with the exact rebuilt value;
the forced-front falsifier runs with `include_heads: true`, so the head edge is
covered by a tail-versus-front comparison, and the director reproduced it red by
reversing the tail's assembly order. Allocation counts byte-identical to base on
a `MapIndexed` probe and on the **`Map` and `Scan` controls that share this
file**.

**Two traps the brief named, both avoided.** `Scan` sits beside this walker and
was *guarded* by brief-461 rather than converted, because `Scan` fits the 1 MiB
boundary at contract height and `MapIndexed` does not; its three guard edges are
untouched. And the shared walker serves the generic shape too, so only the
`MapIndexed (association axis)` row flipped — the generic shortfall is
downstream on the post-result stability path and is still owed.

**One certificate I demanded could not exist, and it is the second time.** I
required a deep certificate with `Heads -> True` so the head edge would be
covered by a certificate and not only by a falsifier. It fails — and fails
**identically on the pre-conversion build**, so it is not a regression. The
reason is not the walker: the head-bearing centre is a call of undefined
symbols, which the evaluator descends as genuine evaluation levels, unlike an
`Association`, which is a data constructor and costs none. That shape refuses in
band from physical height 515 upward, long before the walker matters. The
certificate was rewritten to assert what is true and worth pinning — the
head-bearing shape must fail **closed**, never terminate the process — with the
reason recorded in the test.

brief-467 was the first instance: a test specified without checking it was
achievable. **Both times the shape was impossible for a structural reason that a
five-minute probe would have shown**, and both times the worker or the gate
found it rather than the brief. Probe a certificate's premise before demanding
it.

**The exact-path walks are converted and the conversion phase is closed
(2026-09-05, brief-470).** `AssociateTo` and `KeyDropFrom` on a `Part` target are
`BoundedHeap`: the runtime's `exact_expr_at_path`,
`collect_exact_paths_for_position_spec` and `replace_exact_path` each run behind
a depth-16 front over a heap tail. Two release certificates at raw physical
height 4,097 — one per head, since the two are not interchangeable — a shallow
companion pinning both selector spellings, and a **shared-consumer behaviour
test** the worker added on its own. Three forced-front falsifiers, one per walk;
the director reproduced the `replace_exact_path` one red by unwinding its
ancestor stack FIFO instead of LIFO, which reddens that test and only that test.

`mutation.rs` needed no change — it has no depth recursion of its own, which is
the useful negative the brief asked for. `targeted_map.rs` is untouched, and
allocation counts are byte-identical to base on an `AssociateTo` probe and on
the **`ReplacePart` and `MapAt` shared-consumer controls**.

**The axis here was path length, not value depth** — `exact_expr_at_path`
recurses once per path component and a key selector expands to more than one raw
component — so a fixture built around nesting would have measured a different
walk.

### Registry endstate after the conversion phase

44 rows: **23 `NoProportionalDescent`, 9 `BoundedHeap`, 3 `GuardedNative`, 9
`UnguardedNative`.** Gate note: 2 404 tests green.

The nine unguarded rows are **not** all the short-of-contract queue, which is
what I had assumed before reading them:

- **Four are short of the contract** and have their own queue items: `GroupBy`
  (multi-spec), and `Keys`, `KeyTake`, `KeyDrop` in the deferred
  collection-shape lane. That lane has **three** members, not the two I had been
  saying.
- **Five meet the contract but are still unbounded**: `Association`,
  `KeyComplement`, `Lookup`, `Normal`, `Values`. These are brief-454's second
  table — recursion on collection, selector and wrapper axes rather than
  association value nesting — and they are **reasoned** to clear 4,097, not
  measured. They fail open above whatever their real ceiling is, and nobody has
  bracketed them.

So the conversion phase closed every **measured** fail-open site. The residual
five are the honest remainder, and the registry enforces that each names what
owes it: a row that is `UnguardedNative` with no `owed_by` fails the test.

### The five residual shapes, now measured (2026-09-05, director)

Release, `ulimit -s 2048`, physical levels, highest value passing 3/3 against
lowest failing 3/3. Every probe carries a liveness assertion on the exact value.

| shape | at 4,097 | ok <= | fail >= | direction above | margin |
|---|---|---|---|---|---|
| **`Lookup`, nested key spec** | pass | **4,760** | 4,784 | **kill** | **1.16x** |
| `KeyComplement`, option-name walk | pass | 5,808 | 5,824 | kill | 1.42x |
| `Values` over nested lists | pass | 5,896 | 5,904 | kill | 1.44x |
| `Normal` below wrappers | pass | 5,896 | 5,904 | **in band** | 1.44x |
| `Association`, deep parser | pass | 7,560 | 7,600 | kill | 1.85x |

**The reasoning held this time, and that is worth saying.** All five were
`MeetsContract` by reasoning and all five clear 4,097 by measurement. This line
has mostly found the opposite — the `DeleteMissing` width axis, the `Keys`
ceiling, the target-mode packet ratio — so a reasoned column that survives
measurement is a data point about when to trust one: these are operation-local
native walks that never reach the evaluator, which is exactly the class
brief-457 identified as safe to reason about.

**What was genuinely unknown was the direction, and now it is not.** Four fail
open; `Normal` is the only one that fails closed.

**Three findings that reorder what is left.**

1. **`Lookup` at 1.16x is the thinnest margin anywhere in this class** — thinner
   than the `DeleteMissing` target mode's 1.20x that this line treated as
   urgent — and it fails open. It goes to the top of the residual queue.
2. **`KeyComplement`'s recursion is on an error-message path.** A second
   argument is walked by `collect_option_names_from_tail`
   (`crates/rum-stdlib/src/associations/transforms/key_collections.rs:512-538`)
   to name the offending option, and that walk recurses through nested `List`s.
   So the reachability shape is not a deep *value* but a deep *bad argument* —
   `KeyComplement[assoc, deeplyNestedList]`. Worth carrying: diagnostic paths
   are reachable with inputs nobody writes deliberately.
3. **I doubted the registry on `Keys` versus `Values` and the registry was
   right.** brief-454 records them as sharing the walk at `basic.rs:752-810`,
   and they carry different contract statuses, which looked inconsistent.
   Measured on the same fixture shape: `Keys` gives out at 3,056 and `Values` at
   5,896, nearly 2x apart. Either they do not share the walk or the two paths
   cost very differently; either way the rows are measured-correct and the
   inventory citation is what wants re-reading.

**The residual queue, by this line's ranking** (fail-open first, then thinnest
margin): `Lookup`, `KeyComplement`, `Values`, the `Association` parser, and
`Normal` last because it already fails closed.

### The `basic.rs` key family (briefs 471-472)

**brief-471 corrected the inventory, and the correction mattered.** brief-454
recorded `Keys` and `Values` as one walk at `basic.rs:752-810`; they are
**two separate functions** — `keys_result_for_expr` at `:752-779` and
`values_result_for_expr` at `:781-810` — and the old citation spanned both.
Director-verified from the source. `Keys`' iterator helpers stay resident across
its recursion for a nominal 704 bytes per layer against `Values`' 336, and that
2.1x ratio matches the measured 5,896 against 3,056. `KeyTake` and `KeyDrop`
*do* share their walk, which is why they bracket identically — so the pairing
convention was not the error, only that one entry. **A repair planned on "one
shared walk, fix it once" would have left one of them unfixed.**

Seven recursive edges across the five operations, none diagnostic-only, and
**none takes a `Session`** — director-verified as zero session parameters across
`basic.rs:690-920` — so the simple bounded-front-plus-redo shape applies to all
of them.

**`KeyTake` and `KeyDrop` are converted (brief-472).** One function,
`keyed_take_drop_result_for_expr`, one edge, serving both through a
`keep_selected` flag; now a depth-16 front over a heap tail in
`basic/key_take_drop.rs`. Both **reach the ruled 4,097, which neither could do
before** — they died at 2,856 — and the certificates assert the exact result
including its wrapper shape, since these operations return lists wrapping an
`Association` and a values-only assertion would miss that.

**The conversion improved two things the brief only asked to hold.** It flipped
the failure direction: at 12,000 both now refuse **in band** where they used to
terminate the process, so their rows carry `FailsClosed` rather than the
`Unmeasured` the brief specified — measured, so the honest entry. And it
*removed* allocations: `KeyTake` -24.6% and `KeyDrop` -25.4`%`, about sixteen
fewer per call, while the `Keys` and `Values` same-file controls are
byte-identical. The old recursive form was allocating an intermediate per level.

Gate: 2 407 tests green, both release certificates at raw physical height 4,097,
the forced-front falsifier reproduced red under both flag settings by turning
its tail loop from LIFO into FIFO.

**`Keys` is converted (brief-473).** `keys_result_for_expr` had its one
recursive edge **inside the closure passed to `map`**, so the iterator adaptor
and the `collect` machinery stayed live in the frame across the recursive call —
the 704 bytes per layer against `Values`' 336 that brief-471 measured, visible in
the source. It now runs a depth-16 front over a heap tail in
`basic/keys.rs` and **reaches 4,097, which it could not do before** (it died at
3,056).

The brief ruled out the alternative repair explicitly: outlining could have
recovered most of that 2x, but a fix sized to just clear the contract leaves a
margin thinner than the drift this line has watched, and structural beats
tuned-to-fit.

**The subtle contract held.** `collect::<Result<_, _>>()` short-circuits on the
first error and its payload — the offending sub-expression — reaches a
user-visible diagnostic, so a tail that visited in a different order would
return a different message while every successful case still matched.
Director-verified at the CLI: `Keys[{1, 2}]` reports `1`, and
`Keys[{{"k" -> 7}, 2}]` reports `2`.

**Direction measured at the gate and corrected**, as for `KeyTake`/`KeyDrop`:
in band at 8,000, 12,000 and 20,000, so the row carries `FailsClosed` rather
than the `Unmeasured` the brief specified. **Allocation counts are unchanged**
while `bytes_total` fell 10.6% and peak live bytes 10.8%; the `Values` and
`KeyTake` same-file controls are byte-identical. `Keys`' allocation profile now
matches `Values`'.

Gate: 2 410 tests green, the release certificate at raw physical height 4,097,
and the forced-front falsifier reproduced red by turning its tail loop from LIFO
into FIFO.

**`Lookup` is converted (brief-474), and its four walks are not the four
brief-471 named.** Enumerating every self-recursive function in the file gives
`lookup_query_count` (`:620`), `lookup_entries_outcome` (`:630`), and
`LookupOutcome::into_expr` and `::to_expr` — the owned and borrowed rendering
methods. `finish_prepared_lookup` and `finish_suspended_prepared_lookup` do take
a `&mut Session` but are **not recursive**; they are the callers. So the "no
walker takes a `Session`" property holds for the four that recurse, and the
simple shape applied. All four now run depth-16 fronts over heap tails in
`basic/lookup.rs`.

**The recursive-enum drop hazard was real, and the worker went past what the
brief asked.** `LookupOutcome` carries `List(Vec<Self>)`, so its derived `Drop`
recurses — the class brief-468 hit with `NestedPathNodes`, where one
implementation terminated the process at depth 60,000 on a drop path and only
the cross-comparison caught it. `into_expr` consumes as it renders, but
`to_expr` borrows, so the resume path drops the outcome afterwards. Rather than
argue reachability, brief-474 gave `LookupOutcome` a **manual iterative `Drop`**
and a dedicated release certificate for it at 4,097.

**The agreement assertion earned its place.** `into_expr` and `to_expr` are one
contract on the ordinary and resume paths, and nothing else in the suite
compares them — a divergence would surface only after a suspension.
Director-reproduced: perturbing **only** the borrowed renderer's tail from LIFO
to FIFO reddens exactly
`lookup_rendering_forced_fronts_agree_owned_borrowed_and_exactly` and leaves the
count and planning falsifiers green.

Gate: 2 414 tests green, three release certificates including the drop one,
allocation counts identical across a nested-spec probe, a **flat `Lookup` hot
probe**, and the `Values`/`Keys` same-file controls — so the manual `Drop` costs
nothing. Direction measured at the gate and the row corrected to `FailsClosed`:
in band at 8 000, 15 000 and 30 000.

A measurement note: the candidate build reported "Finished in 0.02s", which
reads like a stale binary. The md5s settled it — the base binary is
byte-identical to brief-473's candidate, which is exactly what a correct base
is, and the candidate differs. **Check the binaries, not the build log.**

**`Values` is converted (brief-475) and `basic.rs` is closed.** All five walks
in that file — `KeyTake`/`KeyDrop`, `Keys`, the four `Lookup` phases, and
`Values` — are now `BoundedHeap`. `Values` reaches raw physical height 4,097
with the exact result and, measured at the gate, now refuses **in band** at
8 000, 15 000 and 30 000, so its row carries `FailsClosed` like its three
siblings.

**The contract the brief pinned survived.** `Keys` reports the innermost
offending element and `Values` reports the level that failed; a conversion that
harmonised them would have changed a user-visible diagnostic while every
successful case still matched. Director-verified at the CLI on four shapes:
`Keys[{{1, 2}}]` still says `1`, `Values[{{1, 2}}]` still says
`List[List[1, …]]`. The worker also built a dedicated in-module falsifier for it,
`values_forced_tail_preserves_outer_expression_error_payload`, which the director
reproduced red alongside the standard forced-front one.

**This is the only conversion in the family that cost instructions, and the
reason is structural.** `values_result_for_expr` was already the lean shape — an
explicit loop, 336 bytes per layer against `Keys`' 704 — so the front and tail
machinery had nothing to recover, where `KeyTake`/`KeyDrop` and `Keys` each gave
back more than they cost. Allocation counts are byte-identical on all four
probes.

**Its price is paid only by the shape the conversion exists for.**
Director-measured: **depth 1, which is what almost every real `Values` call is,
moves +0.020%** — indistinguishable from the untouched controls — while the
depth-9 stress probe moves +0.606%. Accepted on that split, and on the brief-453
precedent where +0.714% bought the removal of a process kill.

**A gate correction worth keeping.** The worker reported the instruction reading
as unattributable, on the grounds that untouched code had moved by the same
magnitude. In the director's control set it had not: the same-file untouched
`Keys` and `Lookup` probes moved +0.012% and -0.163%, an order of magnitude below
the target. **An inability to attribute a cost can be a property of the control
set rather than of the change** — name the controls before concluding noise.

**First ordinary conversion dispatched to `cgl`** rather than to Codex, which
until now had only taken security work. It came back in scope, with the siblings
untouched, and its report was honest about what it could not attribute and about
one self-inflicted edit that the compiler caught. Gate: 2 417 tests green with no
ZeroMQ skip.

**Next**: `GroupBy` (short of the contract and fail-open), then the residual
queue — `KeyComplement` 1.42x, the `Association` parser 1.85x, `Normal` 1.44x
and fail-closed — and outside this line the schema validator and generic
`MapIndexed`.

### The `Persistent*Node` chains are source-reachable and unbounded (brief-477)

Settled by measurement, 2026-09-19. **A source-language construction exists**, on
the string-replacement resume paths, and the tail grows **per element** — one
suspension, then a node per character — so it does **not** need repeated host
round trips. Two constructions reached tail length 1 000, each verified by
instrumentation planted in the `appended` path and then removed; the tree was
left clean.

**Nothing caps it.** The worker set the generated-element limit and the
iteration-work limit **both to 8** and the sequence still reached 1 000: those
suffix loops enforce neither. The scalar replacement count defaults to
`usize::MAX` (`crates/rum-stdlib/src/strings/patterns.rs:746`). So the honest row
for the family in brief-476's inventory is **unbounded and reachable**, not
`Unbounded(owed_by = …)` pending an answer.

**My five failed probes were the wrong operation family**, not evidence of
unreachability — I kept trying `And`, and the reachable shape is the string
path this brief itself had named as the strongest candidate
(`resume_suspended_replace_string_value_from_index`, appending per character).
A negative from a probe that never reached the target says nothing, which is the
whole content of `calibrate-the-predicate-before-trusting-it`.

**Still unmeasured, and first thing next session**: the drop ceiling. The brief
deliberately forbade driving it to a crash — that bracket is the director's —
so we know the chain can be made arbitrarily long and do **not** yet know at
what length its drop stops being survivable. Eight of the eleven families are
also still unexamined.

**The four over-line test files are split (brief-478).** Nothing is over the
35 000-byte line any more except the generated `derived_inventory.rs`, which has
no responsibility boundary to cut on and is excluded by design. Cuts were made on
real boundaries — file loading and script forms, association lvalue, root
visitation, reachability — and the largest resulting file is 29 725 bytes.

Gated mechanically, which is why this was the work chosen for a low-quota day:
no production file in the diff, every file under the line, **2 417 tests green
with no ZeroMQ skip**, and the sorted set of 2 472 `#[test]` names identical
before and after. A pure move needs no A/B, no certificate and no falsifier.

*My first attempt at that name-set check was wrong*: `git grep` searches only
tracked files, so the newly created split files were invisible and the check
reported tests as missing. Re-run over the filesystem it is identical. Third
instrument slip of this arc and the same shape as the others — the tool did not
reach what it claimed to cover.

### The ownership-graph inventory is designed (brief-476)

The design is accepted. It answers the question rum's existing tooling cannot:
the recursion inventory models **call** cycles, the registry models per-builtin
**descent**, and neither models **ownership** — which is why both real drop
hazards this month were found by accident.

**It rediscovers both known hazards structurally, which was the specification.**
Not by name but by field AST: `LookupOutcome --List.0 / Vec element / resolve
Self--> LookupOutcome`, and the mutual pair `NestedPathNodes --entries / Vec
element / tuple.1--> NestedPathNode --Branch.0--> NestedPathNodes`. Citations
verified. Two details it caught that my own scan could not: `LookupOutcome` is
**declared in `basic.rs` while its `Drop` lives in `basic/lookup.rs`**, so
per-file destructor association misses it; and the mutual edge runs through
`Vec<(Expr, NestedPathNode)>`, a tuple inside a Vec.

**Four judgements worth keeping.**

- **`CallSiteDiscipline` is allowed only as explicit debt**, never as a
  structural safety classification — the answer I wanted. And it found that
  `NestedPathNodes`' call set is not the two error-path calls I had counted: the
  success path transfers to the renderer, so the row must bind the enclosing
  function rather than a count.
- **A new disposition I had not proposed, `ViaIterativeDrop`**, for an SCC member
  whose cycles are cut by *another* member's handler. That is exactly the
  `NestedPathNodes`/`NestedPathNode` pair, where only one can carry the `Drop`;
  my four-value vocabulary would have forced a wrong label on the other.
- **Conditional ownership is handled as `LastStrongHolder`**, and "usually
  shared" is explicitly not a bound. With a trap I had not seen: a certified
  self-cycle confers nothing on the payloads — `LookupOutcome`'s `Expr` leaves
  still carry `Expr`'s own debt.
- **`Expr` becomes three rows**, not one: `Expr`, `SharedExpr` and `SharedArgs`,
  each `Unbounded` with `owed_by` pointing at the standing debt and the three
  rejected `Drop` designs. They are exemptions **from remediation, not from
  discovery or classification**, so the debt stops living only in a prose
  paragraph.

**A blocking decision it surfaced, and my ruling on it.** A source-only check
cannot certify host-supplied payloads passed through `set_print_sink` or
`FileIoProvider::from_arc` (`session.rs:137`, `file_io.rs:105`). Those are
existing API boundaries, so my brief's two demands — "an unknown must fail" and
"an exhaustive green production inventory" — are **incompatible today**, and the
report refused to paper over it.

**Ruling: a distinct `UnknownHostPayload` disposition, fatal by default and
waivable only with a named owner and the specific boundary cited.** Not a silent
skip, which is the failure astra's review names and which this project has been
burned by; not a hard block either, which would keep the check permanently red
and therefore unused. It is the same shape as the registry's `owed_by`
enforcement: the unknown stays visible and belongs to someone. Overrule me if
you would rather change the host API instead — that is the larger and stronger
alternative the report also names.

### The ownership graph's discovery half is IN (brief-479), and it produced the number

A test-only scanner over `rum-core`, `rum-runtime` and `rum-stdlib` type
definitions, nine modules, no production source touched. **2 430 tests green on
a loopback-capable host without the ZeroMQ skip** the worker's sandbox needed.

**The counts.**

| kind | count |
|---|---|
| **`UnknownHostPayload`** | **48** — 6 owned storage edges, 42 public by-value inputs |
| `UnknownConstruct` | 7 068 |
| self-reaching types | 57 |

**The 48 is the number the API decision rests on, and its shape matters more
than its size.** Only **6** are places rum *stores* an opaque host payload and
therefore owns its destruction; the other 42 are public inputs that take one by
value. So closing the hazard structurally — the "change the host API" route —
would need a contract on six storage sites, not on forty-eight. That is a much
smaller change than the option looked like when it was only an argument.

**The 7 068 is correct but too coarse to act on, and that is a step-two problem
rather than a defect.** The brief demanded maximal conservatism — "never skip
silently, an unknown is an analysis failure" — and the scanner obeyed. The
partition: 2 896 macro sites, 2 774 closure captures, 1 328 attribute sites of
which **1 271 are plain `derive`**, and **70 storage/representation sites**. A
`#[derive(Clone)]` on a type whose fields are already visible introduces no
owning edge; only that last group of 70 can hide one. **Step two needs a second
axis — "unknown expansion" versus "unknown that could add an owning edge" — or
the ledger will never be green and therefore never used.**

**The acceptance held: rediscovery is structural, not name-fed.** No hazard name
appears anywhere in the scanner. The witnesses show why a naive scan cannot do
this: `LookupOutcome.List.0/std::vec::Vec[Own]/arg0 -> LookupOutcome` resolves
`Self`, and `NestedPathNodes.entries/std::vec::Vec[Own]/arg0/tuple/.1` reaches
through a **tuple inside a Vec** — the exact edge the director's own scan could
not see. Director-reproduced: deleting one member of the mutual pair from
`inventory/self_reaching.tsv` fails with `missing inventory row:` naming the
type, file, line and full cycle; restoring it goes green.

*Fifth instrument slip of this arc, same shape as the others*: my first
perturbation edited `fixtures.rs`, the synthetic fixtures, not the production
inventory — and my earlier size listing missed the inventory entirely because it
globbed `*.rs` and the inventory is a `.tsv`.

**Next**: step two, the disposition ledger — which now has a concrete design
problem to solve, the `UnknownConstruct` second axis — and the
`Persistent*Node` drop ceiling, which remains the director's measurement and is
recorded as `NotMeasured` rather than blocking anything. Then `GroupBy`. Then the remaining four conversions:
`AssociationKeyFlatten`, `AssociationKeyDeflatten`, association `MapIndexed`,
and the `AssociateTo`/`KeyDropFrom` Part target. None fits the 1 MiB budget at
contract height, which is why a guard is not an option for any of them.

**Opened by brief-450 — a second recursion above the contract.** Above roughly
11,000 levels both operations return to terminating the process (in band at
10,000, killed at 12,000), while `Replace`, `DeleteCases` and `ReplaceAll` stay
in band at 12,000 and the fixture alone clears 20,001. So a different recursion
in the same two operations reaches that far. For `ReplacePart` the named
candidate is plan *construction* — `merge_replace_plans`
(`.../structure_data/replace/plan.rs:113`) recurses over a plan whose depth is
the path length, and brief-450 bounded only plan *application*. The `Take`-side
cause is unidentified. It is 2.9x above the ruled height and fails open, so it
belongs in the same class as binding substitution: a safety gap above the
contract, needing its own diagnosis, not a patch bolted onto this repair.

**Recorded for the memory-accounting line, found by accident here.** A release
test spelling `DeleteCases[x, 1, Infinity]` literally trips the `NonAmplifying`
growth assertion: `DeleteCases` is a special form, so the validator measures the
raw argument — the symbol `x`, 298 proxy bytes — while the result is the
evaluated 700 KB tree. The declaration is right and the measurement is wrong for
any special form whose argument is a symbol bound to a large value. It is
**test-only** (the check lives behind the `test-builtin-memory-growth` cargo
feature that `rum-stdlib` enables through a dev-dependency), so it is an
instrument gap rather than a user-facing defect. The certificate works around it
with `Apply`; fixing the model belongs to the growth-contract line.

**Correction to the organization-line note above.** Re-measured at `59410b75`,
only one of the three files named there is over 35k: `evaluation_cache/tests.rs`
at 38 294. `debug_tools/resources/memory.rs` is 32 019 and both `stability.rs`
files are 12 476 and 17 066, all under the line. Three files that were not named
are over it: `tests/context_and_loader.rs` 38 873, `tests/frame_root_trace.rs`
37 971, and `tests/scoping_gensym_gc.rs` 36 121, beside the generated
`frame_root_trace/derived_inventory.rs` at 162 506.

### The host-payload destruction contract is planned (brief-480), step A dispatched

**User ruling 2026-09-19: change the embedding API rather than document the
hazard.** rum has no downstream consumers and the API is deliberately not
locked, so a contract on the six owned storage edges is cheaper now than it
will ever be. A documentation-only contract is the fallback to argue against,
not the recommendation.

**Same day, a second ruling on releases: do not tag.** `v0.2.0` onward is the
**security-hardening line**; the ownership-graph work is a branch inserted from
`serval`'s third-party review, not a replacement for it. Hardening resumes when
this branch closes, and `v0.3.0` is considered only once hardening is
*finished*. No tag lands mid-line, and no step of this contract is a release.

**The plan (brief-480, design-only, tree left clean).** A safe public opt-in
marker `BoundedDestruction`, no blanket impl, no derive, no automatic
implementation from `FrameRootTrace`/`FrameAbandon` or inventory membership;
implementing it *is* the stance. Five steps: A entropy, B file I/O, C print
sink, D the two suspension traits, E the ledger's strict check. The ledger word
is **declared** — the compiler enforces that a stance was taken, never that it
is true, and no `unsafe` may rest on it.

**`print_sink` was the crux and the answer holds.** A trait cannot be
implemented on the host's closure, so the sink becomes a named `PrintSink`
trait plus an explicit adapter whose constructor is named
`assert_bounded_destruction`. Excluding closures was rejected: rum owns their
captured state from installation until replacement, clear, reset or session
destruction.

**Two findings reorder the work.**

1. **The suspension implementor count was wrong by 38x and the plan corrected
   it.** The director's `grep "impl SuspensionFrame for"` found 14 and 5; the
   dominant spelling in this tree is the fully qualified
   `impl rum_runtime::kernel_api::SuspensionFrame for …`, which that pattern
   cannot reach. Director-reproduced with a newline-tolerant scan: **539
   `SuspensionFrame` impls and 59 `PureFrameFinalizer` impls**, exactly the
   plan's numbers. Sixth instrument slip of this arc, same shape as the other
   five — the tool did not reach what it claimed to cover.
2. **Step D is blocked by the `Persistent*Node` drop ceiling, which was
   recorded as `NotMeasured` and "blocking nothing".** The plan refused to sign
   every frame bounded, citing `MergeSortComparisonFrame` →
   `MergeSortState.merged` → `PersistentIndexBuffer` → an `IndexChunk` chain
   extended per frozen batch (`resumable_merge_sort.rs:42-49,60-67,104-115`,
   pinned at `inventory/self_reaching.tsv:50`). Director-verified. Following it,
   the family is **29 chain-shaped self-reaching types**, not the eleven files
   brief-477 recorded — `PersistentStringTextNode`, `PersistentMatchResultsNode`,
   `WithBindingNode` and the rest, across `rum-runtime` and `rum-stdlib`, all of
   them living inside suspension frames. So the drop ceiling is now **the gate
   on step D**, not an optional measurement.

**Step A is IN (brief-481).** `BoundedDestruction` exists as a safe opt-in
marker with no blanket impl and no derive; `EntropySource` requires it; all four
implementors declare it with a rationale. Director-reproduced: 2 431 tests green
with the native ZeroMQ suite really binding loopback (zero skip notices), and
the crate-edge certificate 5/5 — three unchanged legacy fixtures rejected with
`E0277` naming the missing bound, two declared fixtures compiling. The falsifier
was reproduced by removing the supertrait: all three legacy fixtures then
compile and the runner goes red. The fixtures contain **no reference to
`BoundedDestruction`**, which is what makes them valid baseline-sensitive
negatives. All six inventory TSVs stayed byte-identical.

**Step B is IN (brief-482).** `FileIo` requires the marker; all five
implementors declare it with a rationale. The certificate was **extended and
tightened, not loosened**: each case now carries its own expected type name,
trait name and impl line, and the span check moved from substring containment
to exact equality. Director-reproduced: 2 431 tests green with nine native
ZeroMQ tests really binding loopback and zero skip notices, certificate 10/10,
and the falsifier fires per family — removing the `FileIo` supertrait turns its
three negatives green-compiling and the runner red while entropy's five stay
enforced, which shows the two families are independently gated. Inventory
untouched: adding a supertrait is a same-line edit, so no row drifted.

**Step C is IN (brief-483), after two correct stop-losses.** The sink is now a
named `PrintSink: BoundedDestruction + Send`; a closure reaches the setter only
through `ClosurePrintSink::assert_bounded_destruction`, whose field is private
and which has no `From`, no `Default` and no blanket impl. All twelve
installation sites migrated, `rum-cli` among them through `rum-host`'s
re-export — it has no `rum-runtime` dependency and must not gain one.
Certificate 16/16 with three failure shapes, each checked exactly: `E0277` on
the missing bound, `E0308` where a raw `Box<dyn FnMut>` no longer type-checks,
`E0451` where the adapter's field is private. Director-reproduced, including
the perturbation that matters — adding the forbidden blanket
`impl<F: FnMut(&str) + Send> PrintSink for F` makes the raw-closure negative
compile and the runner red. No A/B: `push_print`
(`session_helpers.rs:302-307`) is the only invocation, the no-sink branch is
untouched, and one boxed indirect call replaced another.

**Both stop-losses were the director's omissions**, and both were the same
shape — a rule written from the row he had looked at rather than from a search.
The first: the inventory rule allowed one content change when the setter's
public-input row carries the same payload spelling, so there are two. The
second, correctly escalated rather than pinned: the adapter's constructor adds
a new opaque **public-input** row. Ruled in: the storage side stays covered,
since the session's row is now `dyn PrintSink` and every implementor must
declare the marker, so the new row is the ledger recording where a host closure
is admitted.

**A ledger shape found while deciding that, for step two.** All eight
`generic parameter` rows in `host_payloads.tsv` are public inputs and **none is
a storage row** — `effects.rs:399` emits that spelling only on the public-input
path. Ownership held through a generic field is therefore invisible to the
storage side. Harmless here, because a `dyn` boundary sits in front of it; not
harmless in general.

**Step D's prerequisite is measured, and the chains are reachable.** Director
probe, a faithful replica of `IndexChunk`'s shape (`Option<Arc<Self>>` plus a
payload), release build, `ulimit -s 2048`, three runs per point: **64 000 nodes
survive 3/3 and 66 000 die 3/3** with `fatal runtime error: stack overflow` —
about **32 bytes of stack per level**. brief-477 had already shown such a tail
growing one node per element on the string-replacement resume paths, reaching
1 000 with both limits set to 8, so a 65 000-character string reaches the
ceiling. The kill happens **during destruction**, where the native stack guard
cannot reach: it bounds the evaluator's call stack, not the destructor.

Calibration caveat, stated in the brief rather than buried: the probe measures
the *shape*. `WithBindingNode` carries a `Symbol` and an `Expr`,
`PersistentStringTextNode` a `String`; a heavier node means a **lower** ceiling.
65 000 is an order-of-magnitude anchor, not any type's number.

So the frames owning these chains cannot honestly declare bounded destruction,
and **step D is blocked on repairing them** rather than on measuring them.
brief-484 (design-only) plans that repair. Its crux is that an `Arc` chain
cannot use the `LookupOutcome` worklist idiom
(`stdlib/associations/basic/lookup.rs:24-45`, the only iterative destructor in
the tree): unlinking requires being the last strong holder, so the loop must
stop at a shared node. Its other crux is the certificate — a test that drops a
long chain terminates the test binary *before* the repair, so it cannot be a
before/after pair in one binary, and "did not crash at N" is too weak because
it passes on a recursive drop with a slightly larger stack.

**The drain is IN for two members (brief-485), after four stop-losses.**
`AccumulatorChain<N>` is a `#[repr(transparent)]` wrapper over the existing
`Option<Arc<N>>` carrying the single `Drop`; a derive supplies the node
accessor so forgetting it is a compile error; the drain is `#[cold]` and holds
one node at a time, acquiring with `Arc::into_inner` and stopping at a shared
node. `IndexChunk` and `WithBindingNode` are converted; **27 members remain and
step D stays blocked**.

*The certificate discriminates the mechanism.* A payload whose `Drop` records
the maximum simultaneously-active destruction nesting reads **exactly 1** for a
drain. Director-reproduced falsifier: replacing the drain body with an ordinary
recursive release makes it read the chain length — 64 at length 64, 16 at 16,
8 in the shared-tail case — and four tests go red; restored byte-identical and
green. "Survived at length N" would have passed on a recursive drop with a
slightly larger stack.

*Gate*: fmt clean, **2 444 tests green** (+13) with nine native ZeroMQ tests
binding loopback and zero skip notices, five derive `compile_fail` doctests.
Inventory disclosed and verified: one added `UnknownHostPayload` (the
authorized generic constructor), two `self_reaching` witnesses gaining the
wrapper step, the rest derive and position drift. **Both types stay in
`self_reaching.tsv`, correctly** — the repair bounds destruction, it does not
remove the ownership cycle.

*A/B, and why the panel was not re-run.* `with-deep-body-small` +2.06% and
`with-local` -1.44% sit inside a control band that itself spans -0.38% to
-4.05%, so there is no distinguishable `With` movement. Three labels moved
further — `table-count-null` +13.48%, `table-iterator-null` +10.72%,
`module-chain-escape-large` +5.63% — and all of them are on code this diff does
not touch: `WithBindingNode` is referenced only by `with.rs` and `IndexChunk`
only by `resumable_merge_sort.rs`, while Table's accumulator
(`PersistentExprAccumulatorNode`) is one of the 27 unconverted members. The two
largest movers are also microsecond-tier (19 us and 81 us), the
layout-lottery-prone class. One pair cannot gate this panel, so this is
recorded as observed movement on untouched code rather than tuned.

**A live kernel defect, found incidentally and now fixed (briefs 487-488).**
`Table[InputString[], 2]` resumed to completion **panicked the kernel** —
`PersistentExprAccumulator::into_vec` asserted unique ownership of its tail
with `Arc::try_unwrap` and panicked otherwise. Nothing in `TODO.md` or `docs/`
recorded it. Director-reproduced on the pristine tree before briefing, because
the finding arrived inside a failing submission and the claim "it fails without
my change too" is exactly the claim to check rather than accept.

*The mechanism.* The resume driver legitimately roots the frame while the
continuation runs — `Session.resuming_suspension` →
`SuspendedEvaluation.frames.front()` →
`ResumeSuspendedTableIterationsSuspensionFrame.state.collected.tail` — so the
tail genuinely is shared at completion. The sharing is neither a leak nor an
accident; the uniqueness precondition was simply wrong. `Sum` and `Product`
share it through the same completion path; `Do` does not collect and is
unaffected.

*The repair is two lines*: move out of a uniquely owned node, **clone out of a
shared one**. What decided it is internal consistency — the accumulator's own
frozen prefix already does exactly this (`SharedArgs::into_vec`,
`rum-core/src/expr.rs:647`), so requiring uniqueness of the linked suffix made
one struct inconsistent with itself. An `EvalError` conversion was rejected as
a stopgap: it leaves a valid program failing.

*A correction to the director's own framing.* brief-487 claimed a panic is the
one outcome the embedding API cannot classify. False — `rum-host`'s
`catch_unwind` (`session.rs:661`) reports `HostFailureCode::Panic`, the next
evaluation returns `Poisoned`, and `reset()` recovers. The accurate statement
is that an ordinary valid program poisoned its session instead of completing.

*Gate*: fmt clean, **2 450 tests green** (+6), nine native ZeroMQ tests binding
loopback, zero skip notices. A genuine before/after certificate pair at both
kernel and host level, each asserting the exact collected result rather than
merely the absence of a panic.

*A director omission, the fourth of this shape in this line.* brief-488's Scope
did not list the inventory TSVs, so the worker correctly left them alone and
**said plainly that the workspace gate was not green** rather than claiming it.
Adding `#[derive(Clone)]` rewrites an attribute row and deleting the `panic!`
removes a macro row. The director applied exactly the two rows discovery's own
failure message dictates and re-ran to green.

**Step 2 is IN (brief-486, third dispatch).** The iteration accumulator's two
owning links are wrapper-typed; `into_vec` keeps brief-488's move-or-clone with
**one body for production and tests**, the `cfg(test)` divergence of the first
attempt being forbidden outright. Gate: fmt clean, **2 454 tests green**, nine
native ZeroMQ tests binding loopback, zero skip notices.

*The certificate decomposes, honestly.* brief-485's helper test proves the
**mechanism** (destruction nesting exactly 1); these tests bind this member's
two production links to that mechanism by type identity, and `Weak` handles
prove the **behaviour** — a clone's drop leaves the shared prefix alive and
releases only the exclusive branch. The worker said plainly which half each
certifies rather than claiming one proved both. The director verified the
wiring statically from source (`tail` and `previous` are both
`AccumulatorChain<...>`) rather than re-running the revert, having already
reproduced the mechanism falsifier at brief-485.

*The A/B, and a premise of the director's that was wrong.* The brief asserted
`appended` is the hot path. It is not: `PersistentExprAccumulator::freeze` is
called **only** from the `Err(EvalError::InputRequired)` arm
(`iteration/driver.rs:294-300`), so an unsuspended `Table` accumulates into a
plain `Vec` and never builds a node. The changed code runs only after a
suspension — i.e. behind a host round trip. The worker measured and then said
so itself, disclosing that its CLI workload could not reach `appended` and
therefore proved nothing about the changed path. That is the right call, and
the negative result still matters as a control: on the untouched ordinary path
the instruction count moved **-0.000713%** (-408 of 57.19 M), inside both arms'
spreads of 0.0013% and 0.0019%, with allocation counters identical.

*The instrument is the reusable part.* The timing panel could not resolve this
question — its own controls moved +3.70% to +12.45% and one target swung 25
points between pairs. Instruction counts pinned to one core type
(`taskset -c 2 perf stat -e instructions:u`, reading `cpu_core` at `100.00`
multiplex coverage) repeat to **0.003%** on this host. Calibrate and hand over
the recipe, not the tool name.

**The sixteen runtime conversions are IN (brief-489), with a measured cost the
director accepted rather than hid.** 19 of 29 members now drain iteratively;
ten `rum-stdlib` members and the family coverage gate remain. Gate: fmt clean,
**2 458 tests green**, nine native ZeroMQ tests binding loopback, zero skip
notices.

*The gate caught a real regression, which is what it is for.* Three ordinary
paths cost instructions — `prepared-arguments` **+0.567%**, `lhs-parents`
**+1.044%**, `value-table-rules` **+0.774%**, director-reproduced, arm spreads
0.019%, allocation counters identical. Not every member is post-suspension like
the iteration accumulator; these three build chains on ordinary evaluation.

*Three drain shapes were measured and the cost is not in the drain.* The
committed `#[cold] #[inline(never)]` shape costs the figures above; brief-490's
inlined fast path made it **two to three times worse** (+2.243/+1.941/+3.572);
and removing the cold attributes entirely changed almost nothing
(+0.492/+0.994/+0.689). **The cost is intrinsic to the link wrapper carrying a
`Drop` at all** — a holder whose field has a destructor gets different drop
glue, loses field-wise moves, and gains drop flags.

*The alternative was prototyped and is only marginally better (brief-491).*
Node-level `Drop`, returning the holder's field to a plain `Option<Arc<N>>`,
was built for `PreparedArgumentNode` with safe move-out (no `unsafe`, no
`ManuallyDrop`) and measured **+0.475%** against the wrapper's +0.567% — it
recovers **0.09 points on the workload named for that very member**. So the
wrapper is not the dominant term, and converting the mechanism has an unproven
family-wide payoff against a certain rework of a built, certified mechanism.
**Ruling: keep the wrapper and accept the cost**, on the precedent that
brief-453 accepted +0.714% to remove a process-kill class and brief-475
+0.606% for the same kind of gain. Node-level `Drop` stays recorded as measured
and marginally better, not as a deferred obligation.

*Adjacent question answered, no second defect:* `call_preparation.rs:81`'s
uniqueness panic looked like brief-487's defect but is unreachable — the
`PureFrameFinalizer` runs after frame release, so the retaining owner is gone
before `into_vec`.

*A director slip worth recording.* `git checkout` on two files to revert
brief-491's prototype also reverted brief-489's conversions in them — the
**exact trap the briefs warn workers about**, second time this session. Both
were recovered: one from the dispatch log's diff, one rewritten by hand, plus a
behavioural test extracted from the log; the inventory rows that shifted were
regenerated mechanically from the exact-inventory test's own mismatch output,
never hand-edited. The final count matches the worker's 2 458 exactly.

**The family is closed (brief-492). All 29 members drain iteratively, and a
gate keeps it that way.** Gate: fmt clean, **2 460 tests green**, nine native
ZeroMQ tests binding loopback, zero skip notices.

*The coverage gate is structural.* `every_self_reaching_arc_chain_participates`
runs over `graph::discover(source::production())` — not the pinned snapshot,
which could be satisfied by editing it — and requires every self-reaching type
whose path runs `Option[Own]/arg0/std::sync::Arc` back to itself to carry
`AccumulatorChain`. No name list, no allowlist, no exemption table. The
director verified the selector before the brief was written: 29 rows matched
the chain shape, 19 already carried the wrapper, and the 10 that did not were
exactly the members left.

*Director-reproduced falsifier, and it caught something nobody told it about.*
Appending a **new** chain-shaped type to an unrelated stdlib file turned the
gate red with the type, `file:line` and full ownership path in the message.
That is the gate's actual purpose — a future type of this shape must be
converted or the tree is red. A first attempt at the falsifier instead removed
a converted member's link field and **failed to compile**, because the derive
rejects a node without its accessor: layer one firing before layer three, seen
live.

*The measured cost splits, and the split is what makes it acceptable.* The
worker reported `DeleteMissing` at **+3.563%** and did not tune it, as
instructed. The director then measured the half the brief had not asked for:

| workload | base | candidate | delta |
|---|---:|---:|---:|
| depth 20, crossing the 16-level front/tail handoff | 21 723 027 | 22 499 154 | **+3.573%** |
| depth 3, the ordinary shape | 8 713 146 | 8 712 738 | **-0.005%** |

**Shallow traversals pay nothing**; the cost is borne only by traversals deep
enough to enter the heap tail — the very path that exists so that depth does
not end the process, and which had no destruction bound at all before this
line. Same shape as brief-475's accepted split (depth 1 at +0.020%, depth 9 at
+0.606%), larger in magnitude. **Accepted on that split**, recorded rather than
hidden.

**Step D was cancelled, on evidence (brief-493).** The marker promises a bound
on destroying a value *and everything it can reach*; `Expr`'s destruction is
recorded unbounded standing debt (`TODO.md` §Standing engineering debt, three
iterative `Drop` designs rejected at 20-50%); and 266 of 535 frame implementor
structs own an `Expr`-family field directly, with `FrameAction::Complete`
carrying `Result<Expr, EvalError>` in the protocol itself. A supertrait would
have asked ~598 implementors to assert something the project records as false.

Two qualifications from the review are worth keeping. **Returning an `Expr`
does not make a particular frame unbounded** — the evidence refutes a *universal
supertrait requirement*, not every individual declaration; a fieldless frame
could sign honestly. And **`#[doc(hidden)]` does not close the implementation
set**: both traits are public through `kernel_api` and accepted by public
admission, so recording the in-tree `Expr` debt cannot discharge external
implementations.

**The line closes at step E1 (brief-494), test-only.** A disposition ledger over
all **50** `UnknownHostPayload` rows — **3 `Declared`** (the provider and sink
edges, enforced by supertrait), **3 `Unbounded`** (the suspension edges, owed to
the `Expr` debt, each carrying
`external_implementation_coverage: unresolved`), **44 `PublicInput`**. Gate: fmt
clean, **2 464 tests green**, nine native ZeroMQ tests binding loopback, zero
skip notices, no production file and no existing inventory row changed.

*The third check is what makes it a ledger rather than an excuse table*, and the
director's reproduction of it is worth recording. An `Unbounded` entry's claim
is **verified**: the row's ownership must actually reach
`Expr`/`SharedExpr`/`SharedArgs`. Adding a recursive field unrelated to `Expr`
and attributing it to the `Expr` debt anyway was rejected with
`has no verified path to Expr/SharedExpr/SharedArgs; cannot attribute Expr
debt`.

**But the first rejection was for the wrong reason and had to be re-run.** The
first perturbation used `Box<dyn Debug + Send>` and was refused by a *different*
precondition — "opaque proof requires one unambiguous trait bound" — which would
have certified nothing about attribution. Re-run with a single-bound payload,
the check rejected it on the attribution itself. **A falsifier that goes red is
not evidence until you have read why.**

**E2 was examined and not built (director, 2026-09-21).** The 7 068
`UnknownConstruct` rows are 2 895 macros, 2 774 closure captures, 1 357
attributes and **70 storage/representation**. Only the 70 can hide an owning
edge, and their originating crates are **29 `std`, 6 `num_bigint`, 6
`rum_syntax`**, the rest flat leaves — hashers, `Instant`, `PathBuf`, `BigInt`,
`NonZeroU64`, `Duration`, `AtomicU64`. So the axis would have classified
noise: the maintenance tax is real (three inventory regenerations in one
session) but it is a tax, not a safety gap. **Against that, an axis declaring
some unknown "cannot introduce an owning edge" *silences* it** — maximal
conservatism is loud and useless but never lies. Reopen it when the ledger
needs `UnknownConstruct` to be green for a concrete purpose, which will
constrain the design.

**The same examination found the scanner's own blind spot, and brief-495 closed
it.** `rum-syntax` was not a scanner root while the scanned crates own its
types in four places. The root set is now a **criterion, not a list**: the
kernel and stdlib plus workspace crates whose values they own, transitively —
host and adapters depend in the opposite direction, and `rum-derive` supplies
macros rather than stored runtime values. Gate: fmt clean, **2 465 tests
green**, nine native ZeroMQ tests binding loopback, zero skip notices, no
production source changed. Falsifier: a deliberately added recursive type in
`rum-syntax` is discovered, and was invisible before.

*It surfaced three recursive trees* — `SyntaxExpr`, `SyntaxExprKind` (a mutual
cycle, public) and the formatter's `Doc` — and eight new `PublicInput` ledger
rows.

**A source-reachable fail-open above the contract, measured by the director.**
The worker's claim was that the parser's guards do not bound every AST spine:
`MAX_DELIMITER_NESTING_DEPTH` and `MAX_OPERATOR_CHAIN_DEPTH` are 128, but a
**call-head chain** boxes the prior head without touching either counter.
Verified: `f[1][1]…` parses at 5 000 applications. At the ruled `ulimit -s
2048`, three runs per point:

| applications | result |
|---|---|
| 4 097 (the contract height) | 3/3 survive |
| 5 000 | 3/3 survive |
| 6 000 | **3/3 killed** — `thread 'main' has overflowed its stack` |

So the ceiling is **1.22x-1.46x the ruled contract height, and it fails open**
— a process kill, not an in-band error. That is the same band as the residual
queue's thinnest entries (`Lookup` 1.16x, `KeyComplement` 1.42x), in a
construct nothing could see until the root set was fixed.

**Diagnosed and repaired (briefs 496-497).** The phase is **AST-to-`Expr`
lowering** — director-verified independently of the worker's source sequencing:
a script printing before and after the deep form prints only the first, so the
overflow is inside the form's processing, not at drop time. Parsing is not the
phase: `--format` parses the same input, builds and renders the formatter's
`Doc`, and survives 6 000.

*No parser cap and no language change.* Lowering is publicly callable
(`runtime/lowering.rs:7`, `runtime/resolution.rs:67`), so a host can hand in
constructed syntax of any depth and a cap could not replace the repair; and once
lowering fails closed the deep chain yields an in-band error anyway. That
removed a user-visible decision rather than raising one.

*After brief-497*, at `ulimit -s 2048`: 4 097 and 5 000 unchanged, **6 000 now
returns `! evaluation exceeded recursion limit (5500)`** instead of killing the
process, and nested 300 is still refused in band with its old message. Gate:
fmt clean, **2 471 tests green**, nine native ZeroMQ tests binding loopback,
zero skip notices.

*The A/B, director-measured because the worker's number was for a layout it had
already superseded.* Ten runs per arm, alternating blocks:

| workload | base | candidate | delta | arm spreads |
|---|---:|---:|---:|---|
| lowering-dominated | 2 822 055 018 | 2 825 351 546 | **+0.117%** | 0.137% / 0.202% |
| evaluation-dominated | 885 857 984 | 882 745 193 | **-0.351%** | 0.0017% / 0.0014% |

The lowering delta is **smaller than either arm's own spread**, so it is not
resolvable; the ordinary evaluation workload is faster. Accepted.

*Two director notes worth keeping.* The worker stopped twice, both times
correctly: once because the brief's Scope forbade the ownership inventory that
editing `lowering.rs` inevitably shifts — the **fifth** permission-list omission
on this line, now fixed in the brief as a disclosure duty — and once after
inventory witness matching failed twice under the two-strike rule. The director
regenerated those rows with the script this session has used twice, driven by
the exact-inventory test's own mismatch output: **58 removed, 87 added, all
`UnknownConstruct`, confined to the two lowering files**, with no new
`UnknownHostPayload` and no reclassification.

**The parse dump was cubic and is fixed (brief-498)** — a leftover of the
superlinear-elimination line, cleaned before the hardening mainline resumes by
user ruling.

*The mechanism.* `debug_dump` rendered the tree with `{:#?}`, and Rust's
alternate `Debug` wraps the writer in another pad adapter **per nesting level**,
so every byte passed through O(depth) layers. Measured: instructions grew
**7.9x per doubling (2^3)** while output grew 3.88x — quadratic and inherent for
an indented dump of a deep spine. The work was **one order worse than the bytes
it produced**, and that gap was the defect. A 300-character program cost 1.36
**billion** instructions.

*The repair* is a purpose-built emitter (`rum-syntax/src/syntax_dump.rs`): one
`Indented` writer carrying a depth, so indentation is written once per line
rather than layered, plus this line's bounded front and cold iterative tail —
all tree edges borrowed, so the work stack acquires no ownership and only one
inventory partition moved.

*Director-verified, and the first reading needed correcting.* The worker
reported after-ratios of 2.03-3.09x, **below** the quadratic asymptote the
output demands, which cannot be right while the output is byte-identical. The
cause was process startup dominating the total at these sizes. Subtracting the
`--parse f` baseline (1 068 805 instructions), the marginal ratios are
**2.31 → 2.56 → 3.19 → 3.39, converging on 4** — quadratic, matching the
output. At n=800 the cost fell from 648 797 287 375 to 78 001 406, about
**8 300x**.

| check | result |
|---|---|
| 1 000 applications | **0.062 s**, against a 60 s timeout before |
| output identity | byte-identical across six shape-varied expressions and depths 50/200/900 |
| gate | fmt clean, **2 474 tests green**, nine native ZeroMQ binding loopback, zero skips | Step C
carries a microbench A/B and step D carries the 29-chain prerequisite, so
neither is a cheap-to-verify dispatch.
