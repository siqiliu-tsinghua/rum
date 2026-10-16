# brief-614 — G13 step S6: type guards and binding completion

## Goal

Type the fourth production family through the typed API. S6 owns the frames
that suspend while rum evaluates a **guard**: a right-hand-side `Condition`
(`f[x_] := rhs /; test`, nested conditions flatten to several guards), a
definition whose left-side match pauses before its guards run, the read-only
guard sequence that `ReplacePart` selection uses, and the guard check that
guarded `StringReplace` rules use.

When you are done:

- the five owned rows (four frames and one finalizer) implement the typed
  API, and the five owned macro sites use the witnessed typed macro, the
  token, or the S5 route trait, as their call path requires;
- every other family still runs through the transport it uses today;
- the ordinary path of every program executes no more work than at G14, and
  the cost of S6's resume path is measured and attributed.

This is step S6 of G13 (design round 5, brief-606, ratified). S1-S5 (briefs
607-611) built the typed API and driver and typed evaluation, argument and
rewrite resolution, and definition application. G14 (briefs 612-613,
`762563cd`) then gave every queue cell its evaluation context. S7-S54 type
the remaining families, F1-F23 flip transport per type, and R removes the
legacy machinery.

**Guards sit on the guarded paths of the hottest code**: every guarded
DownValue, SubValue, UpValue and default value, every pattern `Condition`,
guarded rewrite rules, guarded `ReplacePart` and guarded `StringReplace`.
Treat every ordinary-path instruction and every stack byte as owed.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- Authority, in order: this brief; brief-606 (its D1/D2 and report) with
  `.temp/brief-606/design.md` and `census-spec.md`; `.temp/brief-605/design.md`
  §§2-5 for the API, token, fault, unwind and accounting design, **except
  §5's transit record, which D2 rejected**.
- **S3, S4 and S5 as built are your template.** Read brief-609, brief-610 and
  brief-611 in full: their bodies, every director amendment and the gate
  records. Their rulings are now the rules:
  - **Two arms on the typed macro.** `resume_typed_or_suspend!` is defined in
    `suspension/typed/witness.rs`. Use the caller-place arm (`$result:ident`)
    wherever the producer's result is already bound to a local. The
    expression arm is for a producer written inline.
  - **The macro expansion is fixed.** Do not change `witness.rs`. Its third
    arm, `evaluation_return =>`, belongs to the two fixed-point sites in
    `resolution.rs` (G14) and is not for families.
  - **The witness and the request cross a cold call, never the producer's
    `Result`** (S5, redispatch 3). Take the ZST witness at the producer.
    Then pass `(witness, request)` to a cold helper. A shape that needs an
    `unreachable!` arm for a state the types allow is the wrong shape.
  - **The token.** `ContinueToken::suspend(session, witness, request,
    fresh)` re-inserts fresh state of the resuming frame's own type, and it
    carries no `Arc`.
  - **The route trait** (S5): `DefinitionResumeRoute<F>` in
    `definitions/application/mod.rs:173`, implemented for `()` (the ordinary
    route, `witness.suspend`) and for `ContinueToken<'_, F>` (the resumed
    route). Monomorphization keeps the ordinary instantiation unchanged.
    It is visible throughout `definitions/`, so S6 can use it.
  - **The finalizer erasure.** A typed finalizer is erased as its own box,
    through the blanket `impl<F: SlotFinalizer> ErasedFinalizer for F`.
    Deferred work returns `Result<T, ResumeStop<O>>`, and
    `TypedAction::finish_work(work, finish)` selects the finalizer.
  - **The retention analyzer reads the typed macro** through one checked
    adapter, `retention_closure/typed_resume_macro.rs`.
  - **G14:** typed frames get their evaluation context at the common
    insertion endpoints. A family has nothing to do for it.
- **Rows, as input; never edit them.** All of these are in `.temp/brief-606/`:
  - `all-frame-migrations.tsv`: F0266-F0269, F0569;
  - `macro-dataflow.tsv` and `macro-frame-ownership.tsv`: M0273-M0277;
  - `transformation-dispositions.tsv`: T0022;
  - `deferred-obligations.tsv`: D0003;
  - `witness-step-gates.tsv`: `guards.twice`, `default.decline.twice`;
  - `microbench-step-labels.tsv`: the 26 S6 labels;
  - `draft-S6.md`: Codex's plan for this step.

  S6 has no chained inserts, insertion routes, State births, historical
  divergences or new compile-control pairs. The draft is raw material, and
  where it and this brief differ, this brief governs. In particular, its
  Scope, its new-file list (those files were built in S1-S5), its scratch
  path and its measurement section (timing does not decide) are superseded.
- **Every slot type S6 exchanges is already registered:** `Expr` is `Value`,
  and `bool` and `CompiledMatchOutcome` are Legacy (`suspension/typed/legacy.rs`).
  Each keeps its single global mode.
- **The director re-probed the family before writing this brief.** 22
  shapes were run through the resumed path against the queued-input path,
  at `762563cd` (`.temp/director/s6-probe/run1.txt`). All 22 are SAME, and
  every one actually paused. The shapes were:
  - a single guard; a False guard; a non-Boolean guard;
  - two nested guards, and a nested False;
  - multi-argument bindings; a `Table` of guarded calls;
  - a pausing left side followed by a pausing guard, True and False;
  - `MatchQ` true and false, and `Cases`;
  - a guarded `ReplaceAll`, `ReplacePart` and `StringReplace`;
  - OwnValue guards, True and False;
  - a `Default` guard;
  - `Map` over a guarded definition with a fallback;
  - SubValue and UpValue guards;
  - nested guarded calls.

  Unlike S5, this step has no known defect to repair. The typing must keep
  all 22 equal.

### The owned rows, at `762563cd`

Frames, in `crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs`:

| Row | Type | Line | Input → Output |
| --- | --- | --- | --- |
| F0266 | `MatchCompiledDefinitionConditionOwnedSuspensionFrame` | 5 | `CompiledMatchOutcome` → `CompiledMatchOutcome` |
| F0267 | `EvaluateOwnedGuardsSuspensionFrame` | 42 | `Expr` (a guard's value) → `CompiledMatchOutcome` |
| F0268 | `EvaluateReadonlySuspendedGuardsSuspensionFrame` | 57 | `Expr` → `bool` |
| F0569 | `GuardOutcomeFinalizer` | 117 | finalizer, `Output = CompiledMatchOutcome` |
| F0269 | `EvaluateDefinitionRhsGuardsForMatchSuspensionFrame` | 129 | `CompiledMatchOutcome` → `bool` |

Derive each `Input` from the actual producer `Result` before any
transformation, and bind each `Output` at the innermost function or closure
return. The table is the director's reading; the producer decides.

Macro sites, in `crates/rum-runtime/src/runtime/definitions/rhs_guards.rs`:

| Site | Line | Function | Who reaches it |
| --- | --- | --- | --- |
| M0273 | 126 | cold `suspend_definition_condition_match` | The ordinary match (`match_compiled_definition_condition_owned`, reached from `pattern_matching/core.rs:514`) |
| M0274 | 248 | cold `suspend_owned_guards` | The ordinary guard sequence (`evaluate_owned_guards`), from every caller of `evaluate_guard_owned` / `evaluate_guard_sequence_owned`. That includes resumed frames of other families, such as S5's DownValue scan |
| M0275 | 284 | `evaluate_suspended_guards_deferred` | **Two routes:** F0267's own resume (through `resume_suspended_guard_deferred`), which re-inserts F0267; and F0266's resume (through `continue_guard_frame`), which inserts F0267, a *different* frame type |
| M0276 | 308 | `evaluate_readonly_suspended_guards` | **Two routes:** the ordinary `ReplacePart` selection (`kernel_data/structure_data/replace/guarded.rs:480`, via `evaluate_guard_sequence_readonly_suspended`), and F0268's own resume |
| M0277 | 394 | `evaluate_definition_rhs_guards_for_match` (`#[doc(hidden)] pub`, re-exported by `kernel_api.rs:81`) | Guarded `StringReplace` in `rum-stdlib` (`strings/patterns/search/guarded.rs:23`); expression form, `ok(outcome) => Ok(outcome.matched)` |

**Rulings on the routes.**

- **A site reached both from the ordinary path and from its own frame's
  resume (M0276, and M0275 from F0267) uses the S5 route trait.** The
  ordinary instantiation is `()`, and its ordinary-path code must not
  change. The resumed instantiation consumes the token with fresh state.
- **F0266's resume reaching M0275 inserts F0267, which is not F0266's own
  type.** This is an ordinary nested insertion from inside a resume, the
  same as S5's resumed DownValue scan inserting a guard frame today. It uses
  the witnessed macro route with the witness from the actual guard
  `Result`. F0266 then stops with `ResumeStop::<CompiledMatchOutcome>::failure`,
  carrying the request. The inserted F0267 owns the rest of the sequence,
  and its `CompiledMatchOutcome` reaches whatever follows, as today. Do not
  invent an insertion route, and do not chain the two frames.
- **D0003** is the deferred guard sequence. The helper carries the frozen
  binding accumulator and returns `Result<SuspendedGuardCompletion,
  ResumeStop<CompiledMatchOutcome>>`. The owning frame's token preserves
  the `CompiledMatchOutcome` output. `finish_work` selects
  `GuardOutcomeFinalizer`, which becomes a typed `SlotFinalizer`.
- **T0022** (`resume_frames.rs:75:22`) is F0268's
  `.map(|matched| matched.into_continuation_expr())`. It is identity
  transport and disappears; the typed `Output = bool` completes the exact
  helper result.
- **F0269's `outcome.matched` is a semantic map.** It runs identically on
  both paths.
- Report a table of every call path into each site and the route each one
  takes.

If you find that one of these routes is wrong, STOP and report the shape
with evidence. Do not substitute a different route.

## Scope

A listed file or class is permission to do this step's owned work there.
It is not permission to:
- convert another step's rows;
- change evaluation beyond the owned rows;
- tidy.

Disclose every file you edit.

**Classes.**

- **The family's source files:** `definitions/rhs_guards.rs` and
  `definitions/rhs_guards/resume_frames.rs`, under
  `crates/rum-runtime/src/runtime/`.
- **The route trait**, `definitions/application/mod.rs`, only if S6's use
  needs a visibility or bound change. Disclose it.
- **Callers of a changed signature.** Edit only the construct that changed.
  This applies to `kernel_data/structure_data/replace/guarded.rs`,
  `pattern_matching/core.rs`, `kernel_api.rs` and
  `rum-stdlib/src/strings/patterns/search/guarded.rs`, or any file where an
  owned frame, helper or signature is named. The public
  `evaluate_definition_rhs_guards_for_match` keeps its signature unless the
  typed API forces a change; if it does, disclose it.
- **Module roots on an owned type's path.** Make only the visibility or
  re-export edit that the typed API needs, and disclose it.
- **`crates/rum-runtime/src/runtime/suspension/`**, only where S6's use
  exposes a gap in the API or driver. Disclose each edit and why it is
  needed. `typed/witness.rs` is excluded. Sealing, opacity and the
  no-dual-arm rule stay intact.
- **Sibling unit tests.** A test that needs a family module's private items
  goes in that module's sibling `tests.rs`, declared as `#[cfg(test)] mod
  tests;`. Never use `#[path = "../…"]`: the ownership loader does not
  follow `..` and fails closed.
- **Census, oracle and controls.**
  - **Census:** everything under `crates/rum-stdlib/tests/typed_slot_census/`
    and `typed_slot_census.rs`.
    - **Owed from S5:** make the census's scratch path step-independent. It
      pins `.temp/brief-611/r3/census/` today. Use `.temp/typed-slot-census/`,
      so later steps need no edit, as S5 did for the compile controls.
  - **Oracle:** `crates/rum-stdlib/tests/suspension_oracle/fixtures/` for the
    new fixtures and the `typed_slots.rs` receipt, and
    `suspension_oracle/inventory/typed-slot-stages.tsv` and
    `typed-slot-shapes.tsv`.
  - **Compile controls:** `crates/rum-stdlib/tests/typed_slot_compile_controls.rs`,
    only if a pin follows code. There is no new pair.
- **Derived facts follow the diff** (brief-613's class rule). For every
  derived inventory, frontier, census receipt, fingerprint, certificate or
  test pin that names a touched construct, each row falls into exactly one
  case:

  | Its operation in this diff | Ruling |
  | --- | --- |
  | Deleted by an owned conversion | Retired with its source. Disclose it. |
  | Added by this diff | A new row. It stays rejected or unknown; claim no mark or coverage. Disclose it. |
  | Moved, or its text follows an owned rename | Paired. Disclose it old → new. |

  The class covers:
  - `runtime/tests/frame_root_trace{.rs,/derived_inventory.rs}`;
  - the ownership inventories, by discovery replay;
  - the retention inventories (`obligations`, `local-contracts`, `promoted`),
    by replay;
  - `retention_closure/census/tests.rs`, if the frontier moves, with a
    comment in its per-brief style;
  - the census receipts;
  - count and layout pins in tests.

  A **STOP** is a change of *meaning* where the source did not change:
  - a row whose operation's source text is unchanged disappears, or changes
    kind, reason, status, disposition or exclusion;
  - a retention mark, coverage or promotion is gained or lost;
  - a new `UnknownHostPayload`;
  - a change to a production census receipt that is not one of S6's owned
    rows.

  Never hand-write a fingerprint, certificate cell or ledger chain.
- **Canonical tool outputs.** Any unchanged canonical gate tool, meaning a
  repository test or script this brief requires you to run, may write to
  its own fixed output paths under `target/` or `.temp/`. You may not
  modify, redirect or wrap it to move its output. Disclose each such path.
  Your own scratch stays in `.temp/brief-614/`.

`Cargo.toml` files, `Cargo.lock`, the toolchain, docs, `AGENTS.md`,
`TODO.md` and the core `Expr`/`Symbol` representation stay unchanged.

**Disclosure duty.** If a needed change falls outside every list and class,
make no edit there. STOP and report the file, the line and why.

## Steps

1. **Harness, probe and bases first, before any source edit.**
   - **Supplied.** `.temp/brief-614/` holds:
     - `base/rum-cli` and `base/rum-cli-alloc`: the G14 one-codegen-unit
       binaries, with their build commands. The director proved them fresh
       against the committed tree.
     - `base-source/`: the `762563cd` export.
     - `base-hashes.txt`.

     Verify the hashes first.
   - **Resume harness.** Copy `.temp/brief-613/resume-harness/` to
     `.temp/brief-614/resume-harness/`. Add workloads that pause in each S6
     frame. Each repeats its pause N times and checks its final value:
     - F0267, a single right-side guard, and `guards.twice`'s two pausing
       guards (F0267 re-inserting itself through the token);
     - F0266, a pausing left side whose guard then pauses, which is F0266's
       resume inserting F0267;
     - F0268, a guarded `ReplacePart` rule with two nested guards, so the
       second pauses inside F0268's own resume;
     - F0269, a guarded `StringReplace`.

     Then build `resume` and `resume-alloc` **from the unedited tree**,
     with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, each in its own target
     directory under `.temp/brief-614/`. Record their source hashes; they
     must match `base-source/`. Each workload's callgrind frame-call census
     must show the intended S6 frame resuming once per resume.
   - **Stack probe.** The S5 probe is `.temp/brief-613/stack-base/probe.rs`,
     with five shapes: `peel`, `plus`, `sub`, `up` and `guarded`. Add one
     shape, `rhs-guard`: `f[n_] := n /; (n == 0 || f[n - 1] >= 0)`, whose
     recursion runs inside guard evaluation on every level. The director
     checked that `f[30]` returns `30`.
     - Build the base probe binaries in debug and in release (one codegen
       unit) from a scratch copy of the **unedited** tree, with the probe
       appended to `runtime/tests/holding_and_control_part2.rs` in the copy,
       never in the repository.
     - **Calibration:** the five old shapes must reproduce the G14 bytes
       per level exactly:

       | Profile | peel | plus | sub | up | guarded |
       | --- | --- | --- | --- | --- | --- |
       | debug | 19 024 | 34 912 | 29 376 | 28 064 | 35 840 |
       | release | 7 344 | 13 792 | 13 168 | 11 456 | 14 240 |

       If they do not, the base is wrong; STOP.
2. **Labels.** Copy `.temp/brief-613/perf/labels.json` and its programs into
   `.temp/brief-614/perf/`.
   - Add the ten S6 labels that are missing, in the same N/2N format:
     - `matcher-single-tiny`, `matcher-single-wide-large`;
     - `replacepart-deep-path-small`, `replacepart-wide-rule-list`,
       `replacepart-guarded-hit`, `replacepart-guarded-decline-fallback`;
     - `stringreplace-single-literal-unguarded`,
       `stringreplace-pattern-unguarded`, `stringreplace-guarded-hit`,
       `stringreplace-guarded-decline-fallback`.
   - Each program checks its own result, so a label cannot time a failing
     path.
   - Confirm that each guarded label reaches an S6 function, by its
     appearing in a base callgrind profile.
   - Add the new resume workloads as resume labels.
3. **Fixtures.** Before converting anything, add S6's oracle fixtures and
   run them on the unedited tree. Each compares the queued-input path with
   the resumed path in value, log, messages, prints and trace, under
   `Checks::ALL`, with unconditional `FaultProbe` absence.
   - Cover the director's 22 shapes. Add F0266's cross-type route, F0268's
     token route and F0269's `StringReplace` route, each pausing twice.
   - Each fixture passes on the unedited tree, as the director's probe did.
     Record that, so a later failure is attributable to this step.
4. **Type the owned rows**, following the rulings above.
   - Keep exactly:
     - the guard index advance before the next callback;
     - the frozen bindings and the separate false bindings;
     - the option-value scope;
     - held and evaluation order;
     - owner drop order;
     - profiling balance.
   - Each M site's ordinary path matches the producer `Result` once and
     constructs no frame, `Arc` or `Vec`. Frame state is built in the cold
     path. Move locals, not control flow.
   - The transport encodings in T0022 and the frames' other
     `into_continuation_expr` / `from_continuation_expr` calls disappear.
     Only identity transport goes; a semantic map runs on both paths.
5. **Ordinary controls.** Rerun the 28 round-4 ordinary controls through
   brief-611's harness:
   - copy `.temp/brief-611/ordinary-controls/` to `.temp/brief-614/`;
   - re-sync its copied `suspension_oracle/` support from the committed tree
     first;
   - all 28 must pass, including `guards.twice` and `default.decline.twice`.
6. **Census.** Advance the receipt to S6 in `typed_slot_census` and in
   `suspension_oracle/fixtures/typed_slots.rs`, and record every owned row's
   transition.
   - Production at G14 is 607 implementations and 577 macro sites, with 37
     implementations typed.
   - Expect 42 typed implementations if no row splits or merges. Report the
     new totals.
7. **Inventories.** Replay fingerprints, both certificate sets and ownership
   discovery, and disclose every changed row under the derived-facts class.

## Verification (MANDATORY — run to green or evidence)

**How to run.**
- Run everything serially, each command through `python3
  scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
  RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
  TMPDIR=<repo>/.temp/brief-614/tmp …`.
- Bounds: workspace 3 600 s; oracle 1 500 s per build; everything else
  900 s.
- **Your shell tool's own timeout must exceed the command's bound.** A
  workspace run needs about 3 700 s.

Run these in the repository tree:

```
cargo fmt --check
cargo test -p rum-runtime --lib suspension::typed::tests
cargo test -p rum-runtime --lib frame_root_trace
cargo test -p rum-runtime --lib resume_evaluation_context
cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact
cargo test -p rum-stdlib --test typed_slot_compile_controls
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls
cargo test -p rum-stdlib --test typed_slot_census
cargo test -p rum-stdlib --no-default-features --test typed_slot_census
scripts/check-suspension-oracle.sh 1500
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test -p rum-stdlib --lib production_rejection_census_is_complete
cargo test --workspace --no-fail-fast
```

Also run the ordinary-controls harness in both builds, and rerun the
director's 22-shape probe (`.temp/director/s6-probe/pairs.py`, through a
copy of `.temp/director/depth-leak-probe/` built under `.temp/brief-614/`).

Report:
- every `test result:` total;
- the oracle's fault-occurrence delta for each build;
- native ZeroMQ skips, as skips, not passes;
- any compiler warning the diff introduces. Neither oracle build may warn.

**Performance.** `TEMPLATE.md` pitfall 8 governs, with the bars brief-611
and brief-613 set. Build the candidates with the base commands and the same
cwd, into separate target directories. Preflight every N/2N run, both the
result and the actual resume count.

1. **Allocations and bytes first**, on every label, against the bases.
   - Ordinary labels must be exactly equal, in slope and in the
     whole-program counts at N and 2N. A constant that cancels from a slope
     is still work.
   - On the resume workloads, a typed entry adds exactly one allocation, the
     outer adapter `Arc` (24 B), per typed frame per resume. Any further
     difference is attributed at its allocation sites, to a named type's
     mode wire. Anything else is a defect.
2. **Instruction slopes.** Run two interleaved `perf stat` pairs per arm on
   all labels: `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`,
   stdin `/dev/null`, slope `(count(2N) - count(N)) / N`. Save raw output
   before parsing.
   - A label is flagged when its delta exceeds `max(0.05%, base range,
     candidate range)`.
   - A flagged ordinary label is cleared only by five runs per arm, or by
     six or more callgrind processes per arm with every deterministic delta
     named at its addresses.
3. **Copies and leaves.** On every ordinary label, the `memcpy` count is at
   most the base's. `literal-integer`, `do-count-null` and
   `do-iterator-null` show no deterministic instruction growth, by address.
4. **Stack.** Measure all six shapes, in debug and release, at four heights
   with three samples each, base against candidate. Report the residuals.
   - Tabulate the prologue frame size of every function S6 changes, in both
     profiles.
   - **Bar:** no growth beyond the typed macro's named closure environment,
     attributed per slot.
   - **Any growth on `peel`, `plus` or `rhs-guard`, in either profile, is
     reported with its slot table for the director to rule on.** Do not
     accept it yourself.
   - The 48-level test must stay green.
5. **Resume attribution.** For every resume workload, run six callgrind
   processes per arm and diff every function per process.
   - Name each deterministic delta at its addresses.
   - The driver's 136-B typed-outcome move per typed transfer is known and
     owed to R. Count it; do not repair it.
6. **Timing** comes last, as an indication only. Run four full
   `scripts/core-language-microbench.sh` panels on the one-unit binaries, in
   the order base, candidate, base, candidate, under `taskset -c 2`, with
   profiling off.
   - A residual whose instructions are equal is a layout question. Settle
     it causally, as the director did for G14: rebuild both arms with
     `RUSTFLAGS="-C llvm-args=-align-all-functions=6"` and compare cycles
     per iteration on a probe cut from the benchmark program itself, with
     its prelude.
   - Check that the probe reproduces the label's per-iteration magnitude
     before reading its delta. The director's own S5 probe once hit a fast
     path 250x cheaper than the label.

A regression is a defect to fix or report. A stack or performance residual
is not a failing gate. It goes to the report with its attribution, and I
rule on it.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- No default-path cost. No owned-tree clones on hot paths.
- **The witness comes from the actual `Result`.** Never guess the input from
  a synthetic `Err` type.
- **The token** carries no `Arc`, is not `Clone`, and accepts fresh state of
  the same frame type only.
- Actions, codecs and modes stay sealed and opaque. Nothing public names a
  mode or constructs an erased entry.
- Profiling records stay balanced on every path, ordinary and resumed.
- **Prefer the caller-place macro arm.** An address-escaped local that is
  re-bound costs a copy on every ordinary pass.
- Today the frames clone their state on resume (`self.state.clone()`). A
  typed resume may take ownership where the API allows. Disclose which.
- **Module size.** New modules stay under 10k characters. `rhs_guards.rs` is
  13k today. Check `.temp/brief-606/file-size-assessments.tsv` before
  adding to it, and split by responsibility, never by arbitrary cuts.

## Checkpointing — protect the scene

- Everything goes under `.temp/brief-614/`.
- After each step, write `PROGRESS.md` and a machine-readable `state.json`.
  Update `state.json` when the work moves, not only at the end.
- Record:
  - the verified state;
  - the compile and test counters;
  - any half-edited files;
  - the exact next action.

A quota or usage-limit stop can end the session at any moment, and the next
session continues from these files.

## Hard rules

- **No `git`, in any form.** No network.
- Modify only the Scope. No new `unsafe`; no dependency, lockfile or
  toolchain change.
- Commands allowed:
  - scoped reads and edits, `rg`/`grep`, bounded `python3`;
  - the bounded Cargo commands above and bounded compiler fixtures;
  - `scripts/check-suspension-oracle.sh` and
    `scripts/core-language-microbench.sh`;
  - `valgrind`, `callgrind_annotate`, `objdump`, `nm`, `perf` and `taskset`
    on the measurement binaries.

  No probe that aborts a process.
- **The Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- No dual-arm decoder, per-frame mode override, transit record or public raw
  adapter. Type no row outside S6's. Never weaken an oracle `Checks` flag, a
  retention reject, a fault-absence gate or a documented test to get a pass.
- **Stop-loss is keyed on progress.** Compile errors you are reducing count
  as progress. Stop after two attempts that move none of the named failing
  gates.
- Paste real output. Say plainly what did not run or did not pass. Leave
  the tree uncommitted.

## Report format (append to this brief)

1. Files changed, each with its semantic change. Disclose every ripple,
   module root and pin update as old → new, under its derived-facts case.
2. Owned-row reconciliation:
   - each F/M row's transition;
   - the call-path and route table;
   - the removed transport maps;
   - the census output.
3. The harness workloads and oracle fixtures: what each exercises, its frame
   census, and its result on the base tree and the candidate in both builds.
   Include the director's 22 shapes.
4. All gates, verbatim. Fault-occurrence deltas. Inventory and certificate
   transitions. Every canonical tool output path written.
5. Allocation, instruction, copy, stack (all six shapes), resume-attribution
   and timing evidence.
6. Stop-loss counters, deviations and open questions.


## Executor report — S6 guards and binding completion

The five owned implementations and five insertion sites are converted. The
ordinary allocation/copy gates and semantic gates passed. Performance residuals
are submitted for the director's ruling, including two deterministic ordinary
instructions on guarded rewrite and 32 debug stack bytes per RHS-guard level.
The workspace run has nine native ZeroMQ failures caused by denied loopback
binding; the explicit skip path was verified separately. These are skips, not
functional passes. No git command, network access, unsafe code, dependency change,
or driver repair was used. The tree is left uncommitted.

All evidence paths below are relative to the repository. Own scratch is under
`.temp/brief-614/`. Raw commands, stdout/stderr, exit status, preflight results,
source hashes and measurement binary hashes are retained. Performance claims
refer to the frozen binaries in `perf/measured-binaries.json`; supplementary
debug CLI binaries only fill prologue coverage for the stdlib string adapter.

### 1. Files and derived facts

The file table below is the complete source/test/inventory edit list; this
appended report is the only documentation edit. Source snapshots and exact
old/new SHA256 values are in `source-changes-final-preperformance.json`.

`rhs_guards.rs` grows from 13,251 to 15,308 characters, within the existing warning
range. The frame module shrinks from 5,647 to 5,410. The new fixture module is
5,379 characters and the new census module is 4,331. The existing oversized
`frame_root_trace.rs` grows from 35,327 to 35,668 solely for the exact owned
finalizer signature pin; no arbitrary split was made. The existing file-size
assessment was consulted before editing.

No route-trait, public API, visibility/re-export, runtime module-root, compiler
control, obligation, retention frontier, shape receipt, core representation,
Cargo, lockfile, toolchain, or `typed/witness.rs` edit was needed. The fixture
module root adds one declaration; the census root adds the S6 module/receipt and
makes its output root step independent.

Derived changes follow the three allowed cases. Four old macro expansions and
two identity-transport closures retire with their owned source operations. Ten
ownership rows move and one macro row is paired through its owned rename. Two
new closures stay `UnknownConstruct`; neither gains a mark or coverage. All 79
`UnknownHostPayload` rows and all 59 cycle rows remain byte-identical.
`UnknownConstruct` changes 7,709 → 7,705. The row-by-row old/new text is retained
in `ownership-row-changes.json` and reproduced below.

The 68 local-contract fingerprints are canonical recomputations; every other
cell is unchanged. Four C01/C02 promoted certificate cells are copied from the
canonical default/no-default emissions. Semantic instance marks (three
production and four probe instances), coverage, promotion and exclusions remain
unchanged. Certificate digest strings do change with the replayed fingerprints;
they are not a gain or loss of semantic marks. The full emitted cells are in
`certificate-diff.json`. No fingerprint, certificate or ledger chain was
invented. The complete rejection frontier remains 950.


| File | Characters, before → after | Semantic change / derived-facts case |
| --- | --- | --- |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs | 3472 → 3503 | Add guards_binding_completion module declaration. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/guards_binding_completion.rs | 0 → 5379 | New 25 strict queued/resumed equality fixtures (22 director shapes plus three observable repeated routes). |
| crates/rum-runtime/src/runtime/definitions/rhs_guards.rs | 13251 → 15308 | Owned producer witnesses and four cold insertion helpers; existing ordinary/token route trait; caller-place M0277; no public signature change. |
| crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs | 5647 → 5410 | Four typed frame contracts, typed deferred completion/finalizer, removal of owned identity transports; state cloning retained as disclosed. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace.rs | 35327 → 35668 | Paired signature pin: add exact GuardOutcomeFinalizer SlotFinalizer::finish output allowlist entry; no derive population or coverage change. |
| crates/rum-stdlib/tests/typed_slot_census.rs | 7843 → 7879 | Declare S6 module; S5 → S6 stage; forced-red legacy pins 529/57 → 525/56; output root becomes .temp/typed-slot-census/. |
| crates/rum-stdlib/tests/typed_slot_census/receipts.rs | 13603 → 13662 | Owned census pins and S6 transition call; production (518,52,607,577,356,221) → (514,51,607,573,356,217); test-only unchanged. |
| crates/rum-stdlib/tests/typed_slot_census/reporting.rs | 11479 → 11477 | Paired scratch path .temp/brief-611/r3/census/ → .temp/typed-slot-census/. |
| crates/rum-stdlib/tests/typed_slot_census/s4.rs | 5957 → 5999 | Historical S4 population excludes later S5 and S6 rows for both frame and macro comparisons. |
| crates/rum-stdlib/tests/typed_slot_census/s6.rs | 0 → 4331 | New closed reconciliation for the five owned implementations, five insertion routes and transport removals. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs | 181 → 181 | Paired source-stage receipt S5 → S6. |
| crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv | 681514 → 681673 | Fourteen owned row transitions plus implemented S6 stage; all other rows unchanged. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv | 256929 → 256929 | Six paired location moves; source meaning/status unchanged. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv | 421917 → 421909 | Three paired moves; two retired owned identity closures; two new closures remain unknown. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv | 399310 → 398743 | One paired move; four owned macro retirements; one paired owned macro rename. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv | 91646 → 91646 | Canonical replay of 68 fingerprint cells; all other columns unchanged. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv | 18162 → 18162 | Canonical replay of four emitted C01/C02 certificate cells; semantic marks/coverage/promotion unchanged. |

This brief: append this report only. Measurement helpers, exports and probes remain uncommitted scratch.
Ownership replay, exact old → new rows:

| Case | Old row | New row |
| --- | --- | --- |
| paired moved source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	316	1	unresolved attribute/expansion derive (FrameRootTrace) | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	390	1	unresolved attribute/expansion derive (FrameRootTrace) |
| paired moved source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	338	1	unresolved attribute/expansion derive (Clone , FrameRootTrace) | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	412	1	unresolved attribute/expansion derive (Clone , FrameRootTrace) |
| paired moved source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	353	1	unresolved attribute/expansion derive (Clone , FrameRootTrace) | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	427	1	unresolved attribute/expansion derive (Clone , FrameRootTrace) |
| paired moved source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	362	1	unresolved attribute/expansion derive (Clone , FrameRootTrace) | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	436	1	unresolved attribute/expansion derive (Clone , FrameRootTrace) |
| paired moved source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs	56	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs	60	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) |
| paired moved source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs	128	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs	136	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) |
| paired moved source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	42	63	closure captures: \|nested , guard\| (capture types/modes unresolved) | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	46	63	closure captures: \|nested , guard\| (capture types/modes unresolved) |
| paired moved source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	66	53	closure captures: \|(left , right)\| (capture types/modes unresolved) | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	70	53	closure captures: \|(left , right)\| (capture types/modes unresolved) |
| retired owned source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs	75	26	closure captures: \|matched\| (capture types/modes unresolved) | retired |
| paired moved source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs	142	18	closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs	150	29	closure captures: \|\| (capture types/modes unresolved) |
| retired owned source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs	142	79	closure captures: \|value\| (capture types/modes unresolved) | retired |
| added owned source remains unknown | absent | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	335	18	closure captures: \|outcome\| (capture types/modes unresolved) |
| added owned source remains unknown | absent | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs	117	42	closure captures: \|completion\| (capture types/modes unresolved) |
| paired moved source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	49	57	unexpanded macro vec! (ends at 49:73) | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	53	57	unexpanded macro vec! (ends at 53:73) |
| retired owned source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	126	9	unexpanded macro crate :: resume_or_suspend! (ends at 126:126) | retired |
| retired owned source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	248	9	unexpanded macro crate :: resume_or_suspend! (ends at 248:93) | retired |
| retired owned source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	284	17	unexpanded macro crate :: resume_or_suspend! (ends at 284:101) | retired |
| retired owned source | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	308	17	unexpanded macro crate :: resume_or_suspend! (ends at 308:113) | retired |
| paired owned rename | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	394	5	unexpanded macro crate :: resume_or_suspend! (ends at 399:6) | UnknownConstruct	crates/rum-runtime/src/runtime/definitions/rhs_guards.rs	468	5	unexpanded macro crate :: resume_typed_or_suspend! (ends at 473:6) |

Local-contract fingerprint replay:

| ID | Before | After |
| --- | --- | --- |
| B01 | fnv1a64:0cb6c2c3e9fadf69 | fnv1a64:19d7fe16bbc1ce2b |
| B02 | fnv1a64:2d9d26c7fa3d53a8 | fnv1a64:56cceddbdf6cc0ee |
| B03 | fnv1a64:f4c8213f6e6e6c43 | fnv1a64:eb1d9f9e2d431c09 |
| B04 | fnv1a64:c9f2bee3b8a7e731 | fnv1a64:c5ba065b2f9df953 |
| B05 | fnv1a64:3f57a47461035acc | fnv1a64:5be9800fca60548a |
| B06 | fnv1a64:0a1de01e0b3d1f35 | fnv1a64:0aec4ed78e79ac4f |
| B07 | fnv1a64:453d24529115933c | fnv1a64:f068124d11ff38fa |
| B08 | fnv1a64:0775d21adccbb4b7 | fnv1a64:9f3577863cb47d35 |
| B09 | fnv1a64:6f7ab59ca26afe9d | fnv1a64:94f5a9ad6bb0e6d7 |
| B10 | fnv1a64:ef40e435fbcc6f02 | fnv1a64:00d76ff81146b0ac |
| B11 | fnv1a64:8b4a7040d29c0d31 | fnv1a64:8711b7b849921f53 |
| B12 | fnv1a64:2e8735987e73939d | fnv1a64:540229a947b97bd7 |
| I01 | fnv1a64:ea98adc156b2d5e9 | fnv1a64:f7b9e9142879c4ab |
| I02 | fnv1a64:969922085939e2cc | fnv1a64:b32afda3c296dc8a |
| I03 | fnv1a64:88e2720224c73cdd | fnv1a64:9ecf47c0c1b30317 |
| I04 | fnv1a64:00ed61a50b3b86c8 | fnv1a64:9b8b756f1beb498e |
| P01 | fnv1a64:2fefa43cdba02cd5 | fnv1a64:75a131dc70d5cbef |
| P02 | fnv1a64:a97c859e8095d852 | fnv1a64:afa9c3ab9afaf5bc |
| P03 | fnv1a64:f00b457dbdcc00bb | fnv1a64:36cb37e16674d7a1 |
| P04 | fnv1a64:227844dd7ee66fa8 | fnv1a64:4ba80bf16415dcee |
| P05 | fnv1a64:c02e105faef22e39 | fnv1a64:c2682ae06cc7ff3b |
| P06 | fnv1a64:7f065a690aa32d1d | fnv1a64:a4814e79d3e91557 |
| P070 | fnv1a64:b953f6aae918a71d | fnv1a64:deceeabbb25e8f57 |
| P080 | fnv1a64:9ec7497476b2d342 | fnv1a64:f9f3cb4066efbeec |
| P090 | fnv1a64:fe30794ebc79b4e8 | fnv1a64:eaf27152c911bb2e |
| P100 | fnv1a64:93226cf1587ff521 | fnv1a64:ee611c890c6e9203 |
| P110 | fnv1a64:81b568f176de973a | fnv1a64:e1185cc4cc28f0c4 |
| P120 | fnv1a64:f9ad94d1ab693e57 | fnv1a64:a257f815d1290855 |
| P130 | fnv1a64:e90a7dafbfc96d3c | fnv1a64:94356baa40b312fa |
| P140 | fnv1a64:99e9216d444f86e0 | fnv1a64:957f218091ed6b46 |
| P150 | fnv1a64:f08b819632683408 | fnv1a64:eb630cc715e8dcce |
| P071 | fnv1a64:19eb519f19277a5c | fnv1a64:db1b06996a05621a |
| P081 | fnv1a64:794951c34e3c253d | fnv1a64:0ea35ebea5b6ef77 |
| P091 | fnv1a64:41aa919e49d70015 | fnv1a64:9d69da7fda48132f |
| P101 | fnv1a64:79b859beac42343a | fnv1a64:d91b4d92018c8dc4 |
| P111 | fnv1a64:fb7d1b1e3673508d | fnv1a64:c9ec758c455ba247 |
| P121 | fnv1a64:200465981417379e | fnv1a64:3db5b238ff7c3f30 |
| P131 | fnv1a64:679ce149eefc08b1 | fnv1a64:636428c165f21ad3 |
| P141 | fnv1a64:817bff4a179a8675 | fnv1a64:dc581cc37cb6b88f |
| P151 | fnv1a64:a9e1dee2c2361b17 | fnv1a64:9213ae268c8b7515 |
| B13 | fnv1a64:982e73a73bd9db35 | fnv1a64:98fce260bf16684f |
| R01 | fnv1a64:229a7c9b8ae2fa18 | fnv1a64:ea2d385aedc6469e |
| R02 | fnv1a64:2b4b3a2e0db82dfb | fnv1a64:096e4300078542e1 |
| R03 | fnv1a64:c3ee8bae2105d8c4 | fnv1a64:cb9e1905e576a462 |
| R040 | fnv1a64:6495cc15340f7e38 | fnv1a64:36531f2ceca3b73e |
| R050 | fnv1a64:208e529b95eb7319 | fnv1a64:63eccba05d3f501b |
| R060 | fnv1a64:d036f09e0487e6d0 | fnv1a64:ebbb1524b3f093f6 |
| R070 | fnv1a64:e1b719dd2fdde43d | fnv1a64:771126d88758ae77 |
| R080 | fnv1a64:c7f1ca97dac04fc8 | fnv1a64:628fde61eb70128e |
| R090 | fnv1a64:2f7d2e350d253961 | fnv1a64:ca4349cc65a96643 |
| R100 | fnv1a64:b9edc592184416c3 | fnv1a64:b04343f0d718c689 |
| R110 | fnv1a64:c533b55ae55dc82a | fnv1a64:f8a577a5a7968b34 |
| R041 | fnv1a64:7e67d6d153019a35 | fnv1a64:7f36458ad63e274f |
| R051 | fnv1a64:8c52a76449a7c168 | fnv1a64:79149f68563fc7ae |
| R061 | fnv1a64:7072a656aea49e47 | fnv1a64:9f48febfa7dc93c5 |
| R071 | fnv1a64:f3bf62c58c5518c6 | fnv1a64:1992d74b043d3998 |
| R081 | fnv1a64:4f0cd645f42bca21 | fnv1a64:aa4b85dda81a6703 |
| R091 | fnv1a64:76a0ab84c732b2a0 | fnv1a64:135ce900653ea206 |
| R101 | fnv1a64:3bb669c673831510 | fnv1a64:093450dfe79d9636 |
| R111 | fnv1a64:3e38d4f9b6dfb63f | fnv1a64:2c4ec9085a93879d |
| R12 | fnv1a64:8c0a538a0650bdff | fnv1a64:74490b4b44b8b05d |
| R13 | fnv1a64:38d03d21ad26af93 | fnv1a64:a8626ee02ee08d59 |
| R14 | fnv1a64:ad7a7b673530b521 | fnv1a64:08b92afee91f5203 |
| R15 | fnv1a64:fffa26e02c7b69c5 | fnv1a64:c7ea53235c14b6df |
| O010 | fnv1a64:8b64d9a56d6d8327 | fnv1a64:561ad2e87789e3a5 |
| O0200 | fnv1a64:94b0c3419306ace0 | fnv1a64:9046c354e0a49146 |
| O011 | fnv1a64:12bfad261219ab50 | fnv1a64:2e43d1acc1825876 |
| O0210 | fnv1a64:596b64c8b64b155f | fnv1a64:48688f12456e803d |

Certificate body/digest changes (full cells: [certificate-diff.json](../../../.temp/brief-614/certificate-diff.json)):

| Body | hash feature | Before digests | After digests |
| --- | --- | --- | --- |
| C01 | true | fnv1a64:b28df1198b7f61bc; fnv1a64:71fe59f6b7d56e89; fnv1a64:51af0d9d0712feef | fnv1a64:100b5826970f8119; fnv1a64:b6536f64ad4523f8; fnv1a64:f09c40857bbfda66 |
| C02 | true | fnv1a64:507bc9bc29e731e6; fnv1a64:b1ce359c8c3d53cf | fnv1a64:3784b91870ee3517; fnv1a64:2001491e51a21e81 |
| C01 | false | fnv1a64:c90b0583388bf177; fnv1a64:a309c48dcb1cddab; fnv1a64:131f552d20944495 | fnv1a64:ef3db814857f389b; fnv1a64:9e75cce6dd584e7e; fnv1a64:3f328793dd23fc7c |
| C02 | false | fnv1a64:c3ef25525a57e77d; fnv1a64:3e8abab390e29f56 | fnv1a64:be07676aab7bd919; fnv1a64:a473e36eab5f4d37 |


### 2. Owned rows and routes

F0266–F0269 become `TypedFrame`; F0569 becomes `SlotFinalizer`. Inputs are derived
from actual producer `Result`s. M0273/M0274 create the witness immediately before
matching the result and pass only that zero-sized witness plus the request across
the cold helper. M0275/M0276 use the existing `DefinitionResumeRoute` with an
ordinary `()` instantiation and a same-frame token instantiation. M0277 uses the
caller-place typed macro on `guard_result`, keeping its public signature.

F0266's M0275 call inserts F0267 through the ordinary witnessed route and then
turns the returned error into `ResumeStop<CompiledMatchOutcome>`. There is no
chain, token reuse for a different frame type, or invented insertion route.
D0003 carries the frozen binding accumulator through deferred work and selects
the typed finalizer via `finish_work`. F0267 and F0268 retain their existing
`self.state.clone()` behavior. F0266 retains the guards/options clones and the
separate false-bindings clone. The finalizer consumes its `Box<Self>` and moves
its frozen bindings into the result; it does not clone that state.

T0022's bool-to-continuation identity map disappears. The other owned wire
decode/encode calls disappear from the four resumes/finalizer. F0269's
`outcome.matched` remains the semantic map on both ordinary and resumed paths.
The new cold M0275 total map describes successful completion as
`SuspendedGuardCompletion`; its two route implementations suspend by returning
an error, so it does not create an ordinary success conversion on that edge.

The discovered call-path table below includes ordinary calls made from other
families' resumed frames. Those remain ordinary insertions. In particular,
S5 DownValue/default scanning does not receive an S6 continuation token.
The public string guard adapter also serves `StringCases` through the same
stdlib helper (`cases.rs:107,242`, `replace.rs:109,304`); this needs no additional
conversion or caller change.

The production implementation census stays 607; typed implementations change 37 → 42.
Production legacy frames/finalizers change 518/52 → 514/51; test-only legacy
11/5 is unchanged. Production physical macro sites change 577 → 573, result-form sites
356 → 356, insert-form sites 221 → 217. The forced-red total legacy pins change
529/57 → 525/56. The typed registry remains one Value type and 21 Legacy types.
Only the 14 owned transition rows change; S6 adds its implemented stage receipt.
Historical S4 checks exclude later S5/S6 rows when comparing its old population.


| Row | Before → after | Before identity | After identity |
| --- | --- | --- | --- |
| D0003 | untyped → implemented | deferred::guard sequence | deferred::guard sequence |
| F0266 | untyped → typed | rum_runtime::runtime::definitions::rhs_guards::resume_frames::MatchCompiledDefinitionConditionOwnedSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::definitions::rhs_guards::resume_frames::MatchCompiledDefinitionConditionOwnedSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame |
| F0267 | untyped → typed | rum_runtime::runtime::definitions::rhs_guards::resume_frames::EvaluateOwnedGuardsSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::definitions::rhs_guards::resume_frames::EvaluateOwnedGuardsSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame |
| F0268 | untyped → typed | rum_runtime::runtime::definitions::rhs_guards::resume_frames::EvaluateReadonlySuspendedGuardsSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::definitions::rhs_guards::resume_frames::EvaluateReadonlySuspendedGuardsSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame |
| F0269 | untyped → typed | rum_runtime::runtime::definitions::rhs_guards::resume_frames::EvaluateDefinitionRhsGuardsForMatchSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::definitions::rhs_guards::resume_frames::EvaluateDefinitionRhsGuardsForMatchSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame |
| F0569 | untyped → typed | rum_runtime::runtime::definitions::rhs_guards::resume_frames::GuardOutcomeFinalizer implements rum_runtime::runtime::suspension::core::PureFrameFinalizer | rum_runtime::runtime::definitions::rhs_guards::resume_frames::GuardOutcomeFinalizer implements rum_runtime::runtime::suspension::typed::SlotFinalizer |
| M0273 | untyped → typed | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::suspend_definition_condition_match#macro1:resume_or_suspend:insert | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::match_compiled_definition_condition_owned#method4:suspend_definition_condition_match:self |
| M0274 | untyped → typed | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::suspend_owned_guards#macro1:resume_or_suspend:insert | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::evaluate_owned_guards#method5:suspend_owned_guards:self |
| M0275 | untyped → typed | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::evaluate_suspended_guards_deferred#macro1:resume_or_suspend:insert | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::evaluate_suspended_guards_deferred#method6:suspend_deferred_guards:self |
| M0276 | untyped → typed | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::evaluate_readonly_suspended_guards#macro1:resume_or_suspend:insert | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::evaluate_readonly_suspended_guards#method6:suspend_readonly_guards:self |
| M0277 | untyped → typed | rum_runtime::runtime::definitions::rhs_guards::evaluate_definition_rhs_guards_for_match#macro1:resume_or_suspend:result | rum_runtime::runtime::definitions::rhs_guards::evaluate_definition_rhs_guards_for_match#macro1:resume_typed_or_suspend:result |
| S6 | absent → implemented | absent | S6 guards and binding completion: 5 typed implementations and 5 witnessed insertion sites |
| T0022 | untyped → removed | rum_runtime::runtime::definitions::rhs_guards::resume_frames::<EvaluateReadonlySuspendedGuardsSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#method2:map:session . evaluate_readonly_suspended_guards (state) | rum_runtime::runtime::definitions::rhs_guards::resume_frames::<EvaluateReadonlySuspendedGuardsSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#method2:map:session . evaluate_readonly_suspended_guards (state) |
| Whole0027 | legacy-use → removed | rum_runtime::runtime::definitions::rhs_guards::resume_frames::<EvaluateDefinitionRhsGuardsForMatchSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr: | rum_runtime::runtime::definitions::rhs_guards::resume_frames::<EvaluateDefinitionRhsGuardsForMatchSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr: |
| Whole0028 | legacy-use → removed | rum_runtime::runtime::definitions::rhs_guards::resume_frames::<MatchCompiledDefinitionConditionOwnedSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr: | rum_runtime::runtime::definitions::rhs_guards::resume_frames::<MatchCompiledDefinitionConditionOwnedSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr: |


| Site | Call location | Caller | Route |
| --- | --- | --- | --- |
| M0274 | crates/rum-runtime/src/runtime/definitions/application/default_values.rs:144 | rum_runtime::runtime::definitions::application::default_values::<Session as inherent>::evaluate_first_exact_default_value_from_cursor | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0274 | crates/rum-runtime/src/runtime/definitions/application/default_values.rs:243 | rum_runtime::runtime::definitions::application::default_values::<Session as inherent>::resume_suspended_exact_default_value_scan | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0276 | crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:194 | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::evaluate_guard_sequence_readonly_suspended | ordinary () route |
| M0276 | crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:359 | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::evaluate_readonly_suspended_guards | carry the incoming route |
| M0275 | crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:308 | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::evaluate_suspended_guards_deferred | carry the incoming ordinary/token route |
| M0274 | crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:153 | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::finish_definition_condition_match_owned | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0275 | crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:285 | rum_runtime::runtime::definitions::rhs_guards::<Session as inherent>::resume_suspended_guard_deferred | carry the incoming ordinary/token route |
| M0274 | crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:462 | rum_runtime::runtime::definitions::rhs_guards::evaluate_definition_rhs_guards_for_match | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0276 | crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs:79 | rum_runtime::runtime::definitions::rhs_guards::resume_frames::<EvaluateReadonlySuspendedGuardsSuspensionFrame as TypedFrame>::resume | F0268 token |
| M0275 | crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs:109 | rum_runtime::runtime::definitions::rhs_guards::resume_frames::continue_guard_frame | ordinary () insertion of F0267; F0266 then stops |
| M0275 | crates/rum-runtime/src/runtime/definitions/rhs_guards/resume_frames.rs:98 | rum_runtime::runtime::definitions::rhs_guards::resume_frames::resume_guard_frame | F0267 token |
| M0274 | crates/rum-runtime/src/runtime/eval_core/symbol.rs:118 | rum_runtime::runtime::eval_core::symbol::<Session as inherent>::evaluate_stored_own_value | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0274 | crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:301 | rum_runtime::runtime::kernel_data::control_data::exceptions::<Session as inherent>::dispatch_exception_handlers_owned | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0274 | crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:406 | rum_runtime::runtime::kernel_data::control_data::exceptions::<Session as inherent>::resume_suspended_exception_match_outcome | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0274 | crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching.rs:209 | rum_runtime::runtime::kernel_data::rewrite_data::replace::list::matching::<Session as inherent>::filter_replace_list_guard_bindings_owned | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0274 | crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded.rs:262 | rum_runtime::runtime::kernel_data::structure_data::replace::guarded::<Session as inherent>::select_guarded_replace_part_targets_owned | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0276 | crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded.rs:480 | rum_runtime::runtime::kernel_data::structure_data::replace::guarded::<Session as inherent>::select_guarded_replace_part_targets_suspended | ordinary () route; insert F0268 |
| M0274 | crates/rum-runtime/src/runtime/pattern_matching/core.rs:400 | rum_runtime::runtime::pattern_matching::core::<Session as inherent>::evaluate_sequence_pattern_checks_owned | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0274 | crates/rum-runtime/src/runtime/pattern_matching/core.rs:166 | rum_runtime::runtime::pattern_matching::core::<Session as inherent>::match_compiled_condition_owned | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0273 | crates/rum-runtime/src/runtime/pattern_matching/core.rs:514 | rum_runtime::runtime::pattern_matching::core::<Session as inherent>::match_compiled_inner_owned | ordinary actual-result witness; insert F0266 |
| M0274 | crates/rum-runtime/src/runtime/pattern_matching/core/resume_frames.rs:249 | rum_runtime::runtime::pattern_matching::core::resume_frames::<MatchCompiledConditionOwnedSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume | ordinary actual-result witness; insert F0267, including calls made inside other-family resumes |
| M0277 | crates/rum-stdlib/src/strings/patterns/search/guarded.rs:23 | rum_stdlib::strings::patterns::search::guarded::evaluate_string_rule_guard_result | caller-place typed macro; ordinary insert F0269; semantic .matched map retained |
| M0274 | crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:44 | ordinary interpreted condition matcher | ordinary actual-result witness; insert F0267 |
| M0274 | crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:520 | interpreted condition frame resume | ordinary nested insertion of F0267; no S6 token |

Discovered census output (population lines from the initial discovery; final reconciled default/no-default census gates below both pass):

```text
S6 CENSUS production frames=514 finalizers=51 implementations=607 typed=42 macros=573 result=356 insert=217
S6 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S6 CENSUS test-support frames=7 finalizers=1 implementations=21 typed=13 macros=10 result=6 insert=4
S6 PARTITION implementations=623 macros=584
S6 TRANSITION T0022 removed; successor/provider=F0268
S6 TRANSITION Whole0027 removed; successor/provider=D049
S6 TRANSITION Whole0028 removed; successor/provider=D049
S6 FRAME F0266 MatchCompiledDefinitionConditionOwnedSuspensionFrame CompiledMatchOutcome -> CompiledMatchOutcome
S6 FRAME F0267 EvaluateOwnedGuardsSuspensionFrame Expr -> CompiledMatchOutcome
S6 FRAME F0268 EvaluateReadonlySuspendedGuardsSuspensionFrame Expr -> bool
S6 FRAME F0269 EvaluateDefinitionRhsGuardsForMatchSuspensionFrame CompiledMatchOutcome -> bool
S6 FRAME F0569 GuardOutcomeFinalizer  -> CompiledMatchOutcome
```
Final witnessed insertion identities: `.temp/typed-slot-census/{default,no-default}/witnessed-sites.tsv`; physical and receipt census totals must not be conflated. Production plus test-only partition is 623 implementations and 584 macros.

### 3. Harness and oracle

All 117 ordinary and 37 resume workloads pass result/pause-count preflights in
both arms at N and 2N. The five new workloads were built and measured on the
verified base before production edits. The first readonly harness used only one
target and reached F0267 alone; the retained initial census exposed that error.
It was repaired to use two targets and four pauses before conversion, proving
the required F0268 route. This was a harness repair, not a runtime change.

The 25 new oracle fixtures include the director's 22 shapes and three additional
observable-log/printed-output cases for cross-type insertion, repeated readonly
tokens and string matching. Every fixture uses `Checks::ALL` and unconditional
fault-absence assertions. All 25 pass on the corrected base in both builds and
on the candidate (focused default run plus both complete oracles). The initial
readonly fixture expected forward target log order; queued and resumed paths
both produced the existing backward walk. The expectation was corrected to
`List[2, -2, 1, -1]` based on the documented source order, without weakening any
check. The initial failure is retained.

The standalone director probe reports all 22 shapes `SAME`, each with a real
pause and final value. Both complete oracles report 161 passing tests, final
fault-occurrence delta zero, 90,618 observations, and no compiler warnings.
Ordinary controls pass all 28 cases in both feature builds with zero faults.


| New workload | Pauses/iteration | F0266 | F0267 | F0268 | F0269 | F0569 |
| --- | --- | --- | --- | --- | --- | --- |
| s6-rhs-single | 1 | 0.0 | 1.0 | 0.0 | 0.0 | 1.0 |
| s6-rhs-twice | 2 | 0.0 | 2.0 | 0.0 | 0.0 | 1.0 |
| s6-cross-type | 2 | 1.0 | 2.0 | 0.0 | 0.0 | 2.0 |
| s6-readonly-twice | 4 | 0.0 | 2.0 | 2.0 | 0.0 | 1.0 |
| s6-string | 1 | 0.0 | 1.0 | 0.0 | 1.0 | 1.0 |

The table is the base call census, repeated at 100/200 iterations before conversion. Candidate raw frame/finalizer and erased-adapter counts for all six processes are in [resume-callgrind/frame-calls.json](../../../.temp/brief-614/resume-callgrind/frame-calls.json). Exact adapter increases are independently reconciled below.
Director 22-shape output, verbatim:

```text
SAME | f[x_] := x + 1 /; (InputString["g"]; True); | f[1]
  resumed: pauses=1 value=2 messages=[]
  queued:  pauses=0 value=2 messages=[]
SAME | f[x_] := 1 /; (InputString["g"]; False); f[x_] := 2; | f[1]
  resumed: pauses=1 value=2 messages=[]
  queued:  pauses=0 value=2 messages=[]
SAME | f[x_] := 1 /; (InputString["g"]; 7); f[x_] := 2; | f[1]
  resumed: pauses=1 value=2 messages=[]
  queued:  pauses=0 value=2 messages=[]
SAME | f[x_] := (x /; (InputString["a"]; True)) /; (InputString["b"]; True); | f[5]
  resumed: pauses=2 value=5 messages=[]
  queued:  pauses=0 value=5 messages=[]
SAME | f[x_] := (x /; (InputString["a"]; False)) /; (InputString["b"]; True); f[x_] := 0; | f[5]
  resumed: pauses=1 value=0 messages=[]
  queued:  pauses=0 value=0 messages=[]
SAME | f[x_, y_] := {x, y} /; (InputString["p"]; x < y); | {f[1, 2], f[2, 1]}
  resumed: pauses=3 value=List[List[1, 2], f[2, 1]] messages=[]
  queued:  pauses=0 value=List[List[1, 2], f[2, 1]] messages=[]
SAME | f[x_] := x /; (InputString["p"]; OddQ[x]); f[x_] := 0; | Table[f[i], {i, 3}]
  resumed: pauses=3 value=List[1, 0, 3] messages=[]
  queued:  pauses=0 value=List[1, 0, 3] messages=[]
SAME | f[x_ /; (InputString["a"]; True)] := x /; (InputString["b"]; True); | f[4]
  resumed: pauses=2 value=4 messages=[]
  queued:  pauses=0 value=4 messages=[]
SAME | f[x_ /; (InputString["a"]; True)] := x /; (InputString["b"]; False); f[x_] := -1; | f[4]
  resumed: pauses=2 value=-1 messages=[]
  queued:  pauses=0 value=-1 messages=[]
SAME |  | MatchQ[3, x_ /; (InputString["p"]; x > 2)]
  resumed: pauses=1 value=True messages=[]
  queued:  pauses=0 value=True messages=[]
SAME |  | MatchQ[1, x_ /; (InputString["p"]; x > 2)]
  resumed: pauses=1 value=False messages=[]
  queued:  pauses=0 value=False messages=[]
SAME |  | Cases[{1, 2, 3}, x_ /; (InputString["p"]; x > 1)]
  resumed: pauses=3 value=List[2, 3] messages=[]
  queued:  pauses=0 value=List[2, 3] messages=[]
SAME |  | {1, 2, 3} /. x_Integer :> 10 x /; (InputString["p"]; x > 1)
  resumed: pauses=3 value=List[1, 20, 30] messages=[]
  queued:  pauses=0 value=List[1, 20, 30] messages=[]
SAME |  | ReplacePart[{1, 2, 3}, {i_} :> 0 /; (InputString["p"]; i > 1)]
  resumed: pauses=3 value=List[1, 0, 0] messages=[]
  queued:  pauses=0 value=List[1, 0, 0] messages=[]
SAME |  | StringReplace["abc", x_ :> "X" /; (InputString["p"]; x == "b")]
  resumed: pauses=3 value="aXc" messages=[]
  queued:  pauses=0 value="aXc" messages=[]
SAME | v := 5 /; (InputString["p"]; True); | v
  resumed: pauses=1 value=5 messages=[]
  queued:  pauses=0 value=5 messages=[]
SAME | v := 5 /; (InputString["p"]; False); | v
  resumed: pauses=1 value=v messages=[]
  queued:  pauses=0 value=v messages=[]
SAME | Default[d] := 1 /; (InputString["p"]; True); d[x_.] := {x}; | d[]
  resumed: pauses=1 value=List[1] messages=[]
  queued:  pauses=0 value=List[1] messages=[]
SAME | g[x_] := x /; (InputString["p"]; x > 0); g[x_] := -x; | Map[g, {1, -2, 3}]
  resumed: pauses=3 value=List[1, 2, 3] messages=[]
  queued:  pauses=0 value=List[1, 2, 3] messages=[]
SAME | k[1][x_] := x /; (InputString["p"]; True); | k[1][7]
  resumed: pauses=1 value=7 messages=[]
  queued:  pauses=0 value=7 messages=[]
SAME | h /: u[h, x_] := x /; (InputString["p"]; True); | u[h, 9]
  resumed: pauses=1 value=9 messages=[]
  queued:  pauses=0 value=9 messages=[]
SAME | f[x_] := x /; (InputString["p"]; True); | f[f[2]]
  resumed: pauses=2 value=2 messages=[]
  queued:  pauses=0 value=2 messages=[]
```

### 4. Gates, replay and output paths

The exact commands, raw output and exit records for every gate are under
`gates/`. The verbatim totals below include unsuccessful preliminary runs, whose
causes were reduced and resolved: the new fixture's expected log, the census
before discovery replay, formatting, and stale emitted certificate cells.
There were no two consecutive attempts making no progress on a named failing
gate. Compiler controls remain 34 positive/34 intended rejection pairs in each
feature build; no pair or pin was added. Census falsifiers all pass.

The full workspace completed with 3,584 passed, nine failed and 59 ignored
across 53 result lines. All failures are native ZeroMQ loopback binds returning
`Operation not permitted (os error 1)`. The existing helper requires
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`; unlike the brief's automatic-skip description,
the first run did not silently skip. A targeted workspace rerun with that
documented variable emitted nine explicit skips. Its Rust harness totals must
not be read as nine functional passes. No networking/escalation was attempted.

Three unused-helper warnings in the runtime lib-test builds were reproduced in
both base debug/release builds (`base-runtime-warning-evidence.json`); no new
warning is attributed to S6. Neither oracle build warns. The 48-level recursion
test remains green. The full workspace supplies the broader cross-crate check.

Canonical tools were left unchanged and used their own fixed output roots,
listed below. Mock `.rum` path strings in older test fixtures are not recorded
as filesystem writes. Own probes and emitted rum programs use `.rum`.
Required gates and preliminary history, verbatim recorded totals (each command inherits the bounded/offline/serial environment):

`base-s6-oracle-default`: `cargo test -p rum-stdlib --test suspension_oracle guards_binding_completion -- --nocapture`; exit 101.

```text
test result: FAILED. 24 passed; 1 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.21s
test oracle::fixtures::guards_binding_completion::cases ... continuation-slot fault-occurrence delta=0; observations=12
test oracle::fixtures::guards_binding_completion::cross_type_twice ... continuation-slot fault-occurrence delta=0; observations=24
test oracle::fixtures::guards_binding_completion::default_guard ... continuation-slot fault-occurrence delta=0; observations=36
test oracle::fixtures::guards_binding_completion::false_guard ... continuation-slot fault-occurrence delta=0; observations=48
test oracle::fixtures::guards_binding_completion::left_then_right_false ... continuation-slot fault-occurrence delta=0; observations=60
test oracle::fixtures::guards_binding_completion::left_then_right_true ... continuation-slot fault-occurrence delta=0; observations=72
test oracle::fixtures::guards_binding_completion::map_fallback ... continuation-slot fault-occurrence delta=0; observations=84
test oracle::fixtures::guards_binding_completion::match_false ... continuation-slot fault-occurrence delta=0; observations=96
test oracle::fixtures::guards_binding_completion::match_true ... continuation-slot fault-occurrence delta=0; observations=108
test oracle::fixtures::guards_binding_completion::multiple_bindings ... continuation-slot fault-occurrence delta=0; observations=120
test oracle::fixtures::guards_binding_completion::nested_calls ... continuation-slot fault-occurrence delta=0; observations=132
test oracle::fixtures::guards_binding_completion::nested_false ... continuation-slot fault-occurrence delta=0; observations=144
test oracle::fixtures::guards_binding_completion::nested_guards ... continuation-slot fault-occurrence delta=0; observations=156
test oracle::fixtures::guards_binding_completion::non_boolean_guard ... continuation-slot fault-occurrence delta=0; observations=168
test oracle::fixtures::guards_binding_completion::own_false ... continuation-slot fault-occurrence delta=0; observations=180
test oracle::fixtures::guards_binding_completion::own_true ... continuation-slot fault-occurrence delta=0; observations=192
continuation-slot fault-occurrence delta=0; observations=204
test oracle::fixtures::guards_binding_completion::replace_all ... continuation-slot fault-occurrence delta=0; observations=216
test oracle::fixtures::guards_binding_completion::replace_part ... continuation-slot fault-occurrence delta=0; observations=228
test oracle::fixtures::guards_binding_completion::single_guard ... continuation-slot fault-occurrence delta=0; observations=240
test oracle::fixtures::guards_binding_completion::string_replace ... continuation-slot fault-occurrence delta=0; observations=252
test oracle::fixtures::guards_binding_completion::string_route_twice ... continuation-slot fault-occurrence delta=0; observations=264
test oracle::fixtures::guards_binding_completion::subvalue ... continuation-slot fault-occurrence delta=0; observations=276
test oracle::fixtures::guards_binding_completion::table_guards ... continuation-slot fault-occurrence delta=0; observations=288
test oracle::fixtures::guards_binding_completion::upvalue ... continuation-slot fault-occurrence delta=0; observations=300
```

`base-s6-oracle-default-r2`: `cargo test -p rum-stdlib --test suspension_oracle guards_binding_completion -- --nocapture`; exit 0.

```text
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.18s
test oracle::fixtures::guards_binding_completion::cases ... continuation-slot fault-occurrence delta=0; observations=12
test oracle::fixtures::guards_binding_completion::cross_type_twice ... continuation-slot fault-occurrence delta=0; observations=24
test oracle::fixtures::guards_binding_completion::default_guard ... continuation-slot fault-occurrence delta=0; observations=36
test oracle::fixtures::guards_binding_completion::false_guard ... continuation-slot fault-occurrence delta=0; observations=48
test oracle::fixtures::guards_binding_completion::left_then_right_false ... continuation-slot fault-occurrence delta=0; observations=60
test oracle::fixtures::guards_binding_completion::left_then_right_true ... continuation-slot fault-occurrence delta=0; observations=72
test oracle::fixtures::guards_binding_completion::map_fallback ... continuation-slot fault-occurrence delta=0; observations=84
test oracle::fixtures::guards_binding_completion::match_false ... continuation-slot fault-occurrence delta=0; observations=96
test oracle::fixtures::guards_binding_completion::match_true ... continuation-slot fault-occurrence delta=0; observations=108
test oracle::fixtures::guards_binding_completion::multiple_bindings ... continuation-slot fault-occurrence delta=0; observations=120
test oracle::fixtures::guards_binding_completion::nested_calls ... continuation-slot fault-occurrence delta=0; observations=132
test oracle::fixtures::guards_binding_completion::nested_false ... continuation-slot fault-occurrence delta=0; observations=144
test oracle::fixtures::guards_binding_completion::nested_guards ... continuation-slot fault-occurrence delta=0; observations=156
test oracle::fixtures::guards_binding_completion::non_boolean_guard ... continuation-slot fault-occurrence delta=0; observations=168
test oracle::fixtures::guards_binding_completion::own_false ... continuation-slot fault-occurrence delta=0; observations=180
test oracle::fixtures::guards_binding_completion::own_true ... continuation-slot fault-occurrence delta=0; observations=192
test oracle::fixtures::guards_binding_completion::readonly_token_twice ... continuation-slot fault-occurrence delta=0; observations=204
test oracle::fixtures::guards_binding_completion::replace_all ... continuation-slot fault-occurrence delta=0; observations=216
test oracle::fixtures::guards_binding_completion::replace_part ... continuation-slot fault-occurrence delta=0; observations=228
test oracle::fixtures::guards_binding_completion::single_guard ... continuation-slot fault-occurrence delta=0; observations=240
test oracle::fixtures::guards_binding_completion::string_replace ... continuation-slot fault-occurrence delta=0; observations=252
test oracle::fixtures::guards_binding_completion::string_route_twice ... continuation-slot fault-occurrence delta=0; observations=264
test oracle::fixtures::guards_binding_completion::subvalue ... continuation-slot fault-occurrence delta=0; observations=276
test oracle::fixtures::guards_binding_completion::table_guards ... continuation-slot fault-occurrence delta=0; observations=288
test oracle::fixtures::guards_binding_completion::upvalue ... continuation-slot fault-occurrence delta=0; observations=300
```

`base-s6-oracle-no-default`: `cargo test -p rum-stdlib --no-default-features --test suspension_oracle guards_binding_completion -- --nocapture`; exit 0.

```text
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.19s
test oracle::fixtures::guards_binding_completion::cases ... continuation-slot fault-occurrence delta=0; observations=12
test oracle::fixtures::guards_binding_completion::cross_type_twice ... continuation-slot fault-occurrence delta=0; observations=24
test oracle::fixtures::guards_binding_completion::default_guard ... continuation-slot fault-occurrence delta=0; observations=36
test oracle::fixtures::guards_binding_completion::false_guard ... continuation-slot fault-occurrence delta=0; observations=48
test oracle::fixtures::guards_binding_completion::left_then_right_false ... continuation-slot fault-occurrence delta=0; observations=60
test oracle::fixtures::guards_binding_completion::left_then_right_true ... continuation-slot fault-occurrence delta=0; observations=72
test oracle::fixtures::guards_binding_completion::map_fallback ... continuation-slot fault-occurrence delta=0; observations=84
test oracle::fixtures::guards_binding_completion::match_false ... continuation-slot fault-occurrence delta=0; observations=96
test oracle::fixtures::guards_binding_completion::match_true ... continuation-slot fault-occurrence delta=0; observations=108
test oracle::fixtures::guards_binding_completion::multiple_bindings ... continuation-slot fault-occurrence delta=0; observations=120
test oracle::fixtures::guards_binding_completion::nested_calls ... continuation-slot fault-occurrence delta=0; observations=132
test oracle::fixtures::guards_binding_completion::nested_false ... continuation-slot fault-occurrence delta=0; observations=144
test oracle::fixtures::guards_binding_completion::nested_guards ... continuation-slot fault-occurrence delta=0; observations=156
test oracle::fixtures::guards_binding_completion::non_boolean_guard ... continuation-slot fault-occurrence delta=0; observations=168
test oracle::fixtures::guards_binding_completion::own_false ... continuation-slot fault-occurrence delta=0; observations=180
test oracle::fixtures::guards_binding_completion::own_true ... continuation-slot fault-occurrence delta=0; observations=192
test oracle::fixtures::guards_binding_completion::readonly_token_twice ... continuation-slot fault-occurrence delta=0; observations=204
test oracle::fixtures::guards_binding_completion::replace_all ... continuation-slot fault-occurrence delta=0; observations=216
test oracle::fixtures::guards_binding_completion::replace_part ... continuation-slot fault-occurrence delta=0; observations=228
test oracle::fixtures::guards_binding_completion::single_guard ... continuation-slot fault-occurrence delta=0; observations=240
test oracle::fixtures::guards_binding_completion::string_replace ... continuation-slot fault-occurrence delta=0; observations=252
test oracle::fixtures::guards_binding_completion::string_route_twice ... continuation-slot fault-occurrence delta=0; observations=264
test oracle::fixtures::guards_binding_completion::subvalue ... continuation-slot fault-occurrence delta=0; observations=276
test oracle::fixtures::guards_binding_completion::table_guards ... continuation-slot fault-occurrence delta=0; observations=288
test oracle::fixtures::guards_binding_completion::upvalue ... continuation-slot fault-occurrence delta=0; observations=300
```

`census-default`: `cargo test -p rum-stdlib --test typed_slot_census`; exit 0.

```text
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 236.90s
```

`census-discovery-initial`: `cargo test -p rum-stdlib --test typed_slot_census source_baseline_matches_earned_receipt -- --exact --nocapture`; exit 101.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 24.49s
```

`census-no-default`: `cargo test -p rum-stdlib --no-default-features --test typed_slot_census`; exit 0.

```text
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 235.84s
```

`compiler-controls-default`: `cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 47.68s
```

`compiler-controls-no-default`: `cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.60s
```

`depth-48`: `cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`director-22`: `python3 .temp/brief-614/run-director-probe.py`; exit 0.
No test-result line emitted; raw log retained.

`fmt-census`: `cargo fmt --check`; exit 1.
No test-result line emitted; raw log retained.

`fmt-final`: `cargo fmt --check`; exit 0.
No test-result line emitted; raw log retained.

`fmt-initial`: `cargo fmt --check`; exit 1.
No test-result line emitted; raw log retained.

`frame-root-trace`: `cargo test -p rum-runtime --lib frame_root_trace`; exit 0.

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 1.30s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`full-oracle`: `scripts/check-suspension-oracle.sh 1500`; exit 0.

```text
test result: ok. 161 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.75s
test result: ok. 161 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.13s
test oracle::fixtures::argument_traversal_resolution::s4_argument_twice ... continuation-slot fault-occurrence delta=0; observations=12
test oracle::fixtures::argument_traversal_resolution::s4_cases_shared ... continuation-slot fault-occurrence delta=0; observations=24
test oracle::fixtures::argument_traversal_resolution::s4_cases_spec ... continuation-slot fault-occurrence delta=0; observations=36
test oracle::fixtures::argument_traversal_resolution::s4_cases_traversal ... continuation-slot fault-occurrence delta=0; observations=48
test oracle::fixtures::argument_traversal_resolution::s4_count_traversal ... continuation-slot fault-occurrence delta=0; observations=60
test oracle::fixtures::argument_traversal_resolution::s4_position_traversal ... continuation-slot fault-occurrence delta=0; observations=72
test oracle::fixtures::argument_traversal_resolution::s4_rewrite_compiled ... continuation-slot fault-occurrence delta=0; observations=84
test oracle::fixtures::argument_traversal_resolution::s4_rewrite_exprs ... continuation-slot fault-occurrence delta=0; observations=96
test oracle::fixtures::definition_application::default_cursor_match_none ... continuation-slot fault-occurrence delta=0; observations=17271
test oracle::fixtures::definition_application::default_cursor_match_ordinary ... continuation-slot fault-occurrence delta=0; observations=17283
test oracle::fixtures::definition_application::default_cursor_match_some ... continuation-slot fault-occurrence delta=0; observations=17295
test oracle::fixtures::definition_application::default_cursor_next_none ... continuation-slot fault-occurrence delta=0; observations=17307
test oracle::fixtures::definition_application::default_cursor_next_ordinary ... continuation-slot fault-occurrence delta=0; observations=17319
test oracle::fixtures::definition_application::default_cursor_next_some ... continuation-slot fault-occurrence delta=0; observations=17331
test oracle::fixtures::definition_application::default_first_guarded_table_none ... continuation-slot fault-occurrence delta=0; observations=17343
test oracle::fixtures::definition_application::default_first_guarded_table_ordinary ... continuation-slot fault-occurrence delta=0; observations=17355
test oracle::fixtures::definition_application::default_first_guarded_table_some ... continuation-slot fault-occurrence delta=0; observations=17367
test oracle::fixtures::definition_application::default_first_plain_none ... continuation-slot fault-occurrence delta=0; observations=17379
test oracle::fixtures::definition_application::default_first_plain_ordinary ... continuation-slot fault-occurrence delta=0; observations=17391
test oracle::fixtures::definition_application::default_first_plain_some ... continuation-slot fault-occurrence delta=0; observations=17403
test oracle::fixtures::definition_application::default_resumed_match_none ... continuation-slot fault-occurrence delta=0; observations=17415
test oracle::fixtures::definition_application::default_resumed_match_ordinary ... continuation-slot fault-occurrence delta=0; observations=17427
test oracle::fixtures::definition_application::default_resumed_match_some ... continuation-slot fault-occurrence delta=0; observations=17439
test oracle::fixtures::definition_application::default_resumed_next_none ... continuation-slot fault-occurrence delta=0; observations=17451
test oracle::fixtures::definition_application::default_resumed_next_ordinary ... continuation-slot fault-occurrence delta=0; observations=17463
test oracle::fixtures::definition_application::default_resumed_next_some ... continuation-slot fault-occurrence delta=0; observations=17475
test oracle::fixtures::definition_application::down_exact_none ... continuation-slot fault-occurrence delta=0; observations=17487
test oracle::fixtures::definition_application::down_exact_ordinary ... continuation-slot fault-occurrence delta=0; observations=17499
test oracle::fixtures::definition_application::down_exact_some ... continuation-slot fault-occurrence delta=0; observations=17511
test oracle::fixtures::definition_application::down_pattern_none ... continuation-slot fault-occurrence delta=0; observations=17523
test oracle::fixtures::definition_application::down_pattern_ordinary ... continuation-slot fault-occurrence delta=0; observations=17535
test oracle::fixtures::definition_application::down_pattern_some ... continuation-slot fault-occurrence delta=0; observations=17547
test oracle::fixtures::definition_application::down_resumed_none ... continuation-slot fault-occurrence delta=0; observations=17559
test oracle::fixtures::definition_application::down_resumed_ordinary ... continuation-slot fault-occurrence delta=0; observations=17571
test oracle::fixtures::definition_application::down_resumed_some ... continuation-slot fault-occurrence delta=0; observations=17583
test oracle::fixtures::definition_application::sub_exact_none ... continuation-slot fault-occurrence delta=0; observations=17595
test oracle::fixtures::definition_application::sub_exact_ordinary ... continuation-slot fault-occurrence delta=0; observations=17607
test oracle::fixtures::definition_application::sub_exact_some ... continuation-slot fault-occurrence delta=0; observations=17619
test oracle::fixtures::definition_application::sub_pattern_none ... continuation-slot fault-occurrence delta=0; observations=17631
test oracle::fixtures::definition_application::sub_pattern_ordinary ... continuation-slot fault-occurrence delta=0; observations=17643
test oracle::fixtures::definition_application::sub_pattern_some ... continuation-slot fault-occurrence delta=0; observations=17655
test oracle::fixtures::definition_application::sub_resumed_none ... continuation-slot fault-occurrence delta=0; observations=17667
test oracle::fixtures::definition_application::sub_resumed_ordinary ... continuation-slot fault-occurrence delta=0; observations=17679
test oracle::fixtures::definition_application::sub_resumed_some ... continuation-slot fault-occurrence delta=0; observations=17691
test oracle::fixtures::definition_application::up_exact_none ... continuation-slot fault-occurrence delta=0; observations=17703
test oracle::fixtures::definition_application::up_exact_ordinary ... continuation-slot fault-occurrence delta=0; observations=17715
test oracle::fixtures::definition_application::up_exact_some ... continuation-slot fault-occurrence delta=0; observations=17727
test oracle::fixtures::definition_application::up_pattern_none ... continuation-slot fault-occurrence delta=0; observations=17739
test oracle::fixtures::definition_application::up_pattern_ordinary ... continuation-slot fault-occurrence delta=0; observations=17751
test oracle::fixtures::definition_application::up_pattern_some ... continuation-slot fault-occurrence delta=0; observations=17763
test oracle::fixtures::definition_application::up_resumed_none ... continuation-slot fault-occurrence delta=0; observations=17775
test oracle::fixtures::definition_application::up_resumed_ordinary ... continuation-slot fault-occurrence delta=0; observations=17787
test oracle::fixtures::definition_application::up_resumed_some ... continuation-slot fault-occurrence delta=0; observations=17799
test oracle::fixtures::definition_application::up_sync_guard_none ... continuation-slot fault-occurrence delta=0; observations=17811
test oracle::fixtures::definition_application::up_sync_guard_ordinary ... continuation-slot fault-occurrence delta=0; observations=17823
test oracle::fixtures::definition_application::up_sync_guard_some ... continuation-slot fault-occurrence delta=0; observations=17835
test oracle::fixtures::definition_storage::s5_default_fallback_rhs ... continuation-slot fault-occurrence delta=0; observations=17847
test oracle::fixtures::definition_storage::s5_default_guard ... continuation-slot fault-occurrence delta=0; observations=17859
test oracle::fixtures::definition_storage::s5_default_rhs ... continuation-slot fault-occurrence delta=0; observations=17871
test oracle::fixtures::definition_storage::s5_destructure ... continuation-slot fault-occurrence delta=0; observations=17883
test oracle::fixtures::definition_storage::s5_destructure_twice ... continuation-slot fault-occurrence delta=0; observations=17895
test oracle::fixtures::definition_storage::s5_down_guard ... continuation-slot fault-occurrence delta=0; observations=17907
test oracle::fixtures::definition_storage::s5_down_rhs ... continuation-slot fault-occurrence delta=0; observations=17919
test oracle::fixtures::definition_storage::s5_lhs_twice ... continuation-slot fault-occurrence delta=0; observations=17931
test oracle::fixtures::definition_storage::s5_own_rhs ... continuation-slot fault-occurrence delta=0; observations=17943
test oracle::fixtures::definition_storage::s5_rule_lhs ... continuation-slot fault-occurrence delta=0; observations=17955
test oracle::fixtures::definition_storage::s5_rule_lhs_twice ... continuation-slot fault-occurrence delta=0; observations=17967
test oracle::fixtures::definition_storage::s5_rule_rhs ... continuation-slot fault-occurrence delta=0; observations=17979
test oracle::fixtures::definition_storage::s5_set ... continuation-slot fault-occurrence delta=0; observations=17991
test oracle::fixtures::definition_storage::s5_sub_guard ... continuation-slot fault-occurrence delta=0; observations=18003
test oracle::fixtures::definition_storage::s5_sub_rhs ... continuation-slot fault-occurrence delta=0; observations=18015
test oracle::fixtures::definition_storage::s5_tagset ... continuation-slot fault-occurrence delta=0; observations=18027
test oracle::fixtures::definition_storage::s5_up_guard ... continuation-slot fault-occurrence delta=0; observations=18039
test oracle::fixtures::definition_storage::s5_up_rhs ... continuation-slot fault-occurrence delta=0; observations=18051
test oracle::fixtures::definition_storage::s5_upset ... continuation-slot fault-occurrence delta=0; observations=18063
test oracle::fixtures::evaluation_dispatch::head_map_twice ... continuation-slot fault-occurrence delta=0; observations=18075
test oracle::fixtures::evaluation_dispatch::prepared_twice ... continuation-slot fault-occurrence delta=0; observations=18087
test oracle::fixtures::evaluation_dispatch::s3_builtin_empty_some ... continuation-slot fault-occurrence delta=0; observations=18099
test oracle::fixtures::evaluation_dispatch::s3_builtin_none ... continuation-slot fault-occurrence delta=0; observations=18111
test oracle::fixtures::evaluation_dispatch::s3_builtin_some ... continuation-slot fault-occurrence delta=0; observations=18123
test oracle::fixtures::evaluation_dispatch::s3_threaded_empty_some ... continuation-slot fault-occurrence delta=0; observations=18135
test oracle::fixtures::evaluation_dispatch::s3_threaded_none ... continuation-slot fault-occurrence delta=0; observations=18147
test oracle::fixtures::evaluation_dispatch::s3_threaded_some ... continuation-slot fault-occurrence delta=0; observations=18159
continuation-slot fault-occurrence delta=0; observations=18825
continuation-slot fault-occurrence delta=0; observations=18949
continuation-slot fault-occurrence delta=0; observations=19561
test oracle::fixtures::guards_binding_completion::cases ... continuation-slot fault-occurrence delta=0; observations=22433
test oracle::fixtures::guards_binding_completion::cross_type_twice ... continuation-slot fault-occurrence delta=0; observations=22445
test oracle::fixtures::guards_binding_completion::default_guard ... continuation-slot fault-occurrence delta=0; observations=22457
test oracle::fixtures::guards_binding_completion::false_guard ... continuation-slot fault-occurrence delta=0; observations=22469
test oracle::fixtures::guards_binding_completion::left_then_right_false ... continuation-slot fault-occurrence delta=0; observations=22481
test oracle::fixtures::guards_binding_completion::left_then_right_true ... continuation-slot fault-occurrence delta=0; observations=22493
test oracle::fixtures::guards_binding_completion::map_fallback ... continuation-slot fault-occurrence delta=0; observations=22505
test oracle::fixtures::guards_binding_completion::match_false ... continuation-slot fault-occurrence delta=0; observations=22517
test oracle::fixtures::guards_binding_completion::match_true ... continuation-slot fault-occurrence delta=0; observations=22529
test oracle::fixtures::guards_binding_completion::multiple_bindings ... continuation-slot fault-occurrence delta=0; observations=22541
test oracle::fixtures::guards_binding_completion::nested_calls ... continuation-slot fault-occurrence delta=0; observations=22553
test oracle::fixtures::guards_binding_completion::nested_false ... continuation-slot fault-occurrence delta=0; observations=22565
test oracle::fixtures::guards_binding_completion::nested_guards ... continuation-slot fault-occurrence delta=0; observations=22577
test oracle::fixtures::guards_binding_completion::non_boolean_guard ... continuation-slot fault-occurrence delta=0; observations=22589
test oracle::fixtures::guards_binding_completion::own_false ... continuation-slot fault-occurrence delta=0; observations=22601
test oracle::fixtures::guards_binding_completion::own_true ... continuation-slot fault-occurrence delta=0; observations=22613
test oracle::fixtures::guards_binding_completion::readonly_token_twice ... continuation-slot fault-occurrence delta=0; observations=22625
test oracle::fixtures::guards_binding_completion::replace_all ... continuation-slot fault-occurrence delta=0; observations=22637
test oracle::fixtures::guards_binding_completion::replace_part ... continuation-slot fault-occurrence delta=0; observations=22649
test oracle::fixtures::guards_binding_completion::single_guard ... continuation-slot fault-occurrence delta=0; observations=22661
test oracle::fixtures::guards_binding_completion::string_replace ... continuation-slot fault-occurrence delta=0; observations=22673
test oracle::fixtures::guards_binding_completion::string_route_twice ... continuation-slot fault-occurrence delta=0; observations=22685
test oracle::fixtures::guards_binding_completion::subvalue ... continuation-slot fault-occurrence delta=0; observations=22697
test oracle::fixtures::guards_binding_completion::table_guards ... continuation-slot fault-occurrence delta=0; observations=22709
test oracle::fixtures::guards_binding_completion::upvalue ... continuation-slot fault-occurrence delta=0; observations=22721
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=90618
```

`ordinary-default`: `cargo test --manifest-path .temp/brief-614/ordinary-controls/Cargo.toml --test ordinary -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test ordinary_family_witnesses ... continuation-slot fault-occurrence delta=0; observations=12
continuation-slot fault-occurrence delta=0; observations=24
continuation-slot fault-occurrence delta=0; observations=36
continuation-slot fault-occurrence delta=0; observations=48
continuation-slot fault-occurrence delta=0; observations=60
continuation-slot fault-occurrence delta=0; observations=72
continuation-slot fault-occurrence delta=0; observations=84
continuation-slot fault-occurrence delta=0; observations=96
continuation-slot fault-occurrence delta=0; observations=108
continuation-slot fault-occurrence delta=0; observations=120
continuation-slot fault-occurrence delta=0; observations=132
continuation-slot fault-occurrence delta=0; observations=144
continuation-slot fault-occurrence delta=0; observations=156
continuation-slot fault-occurrence delta=0; observations=168
continuation-slot fault-occurrence delta=0; observations=180
continuation-slot fault-occurrence delta=0; observations=192
continuation-slot fault-occurrence delta=0; observations=204
continuation-slot fault-occurrence delta=0; observations=216
continuation-slot fault-occurrence delta=0; observations=228
continuation-slot fault-occurrence delta=0; observations=240
continuation-slot fault-occurrence delta=0; observations=252
continuation-slot fault-occurrence delta=0; observations=264
continuation-slot fault-occurrence delta=0; observations=276
continuation-slot fault-occurrence delta=0; observations=288
continuation-slot fault-occurrence delta=0; observations=300
continuation-slot fault-occurrence delta=0; observations=312
continuation-slot fault-occurrence delta=0; observations=324
continuation-slot fault-occurrence delta=0; observations=336
```

`ordinary-no-default`: `cargo test --manifest-path .temp/brief-614/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test ordinary_family_witnesses ... continuation-slot fault-occurrence delta=0; observations=12
continuation-slot fault-occurrence delta=0; observations=24
continuation-slot fault-occurrence delta=0; observations=36
continuation-slot fault-occurrence delta=0; observations=48
continuation-slot fault-occurrence delta=0; observations=60
continuation-slot fault-occurrence delta=0; observations=72
continuation-slot fault-occurrence delta=0; observations=84
continuation-slot fault-occurrence delta=0; observations=96
continuation-slot fault-occurrence delta=0; observations=108
continuation-slot fault-occurrence delta=0; observations=120
continuation-slot fault-occurrence delta=0; observations=132
continuation-slot fault-occurrence delta=0; observations=144
continuation-slot fault-occurrence delta=0; observations=156
continuation-slot fault-occurrence delta=0; observations=168
continuation-slot fault-occurrence delta=0; observations=180
continuation-slot fault-occurrence delta=0; observations=192
continuation-slot fault-occurrence delta=0; observations=204
continuation-slot fault-occurrence delta=0; observations=216
continuation-slot fault-occurrence delta=0; observations=228
continuation-slot fault-occurrence delta=0; observations=240
continuation-slot fault-occurrence delta=0; observations=252
continuation-slot fault-occurrence delta=0; observations=264
continuation-slot fault-occurrence delta=0; observations=276
continuation-slot fault-occurrence delta=0; observations=288
continuation-slot fault-occurrence delta=0; observations=300
continuation-slot fault-occurrence delta=0; observations=312
continuation-slot fault-occurrence delta=0; observations=324
continuation-slot fault-occurrence delta=0; observations=336
```

`ownership-discovery`: `cargo test -p rum-stdlib --lib ownership_graph::discovery_report_without_inventory_input -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 5.82s
```

`ownership-graph`: `cargo test -p rum-stdlib --lib ownership_graph`; exit 0.

```text
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 70.21s
```

`rejection-census`: `cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 26.03s
```

`resume-context`: `cargo test -p rum-runtime --lib resume_evaluation_context`; exit 0.

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1176 filtered out; finished in 0.16s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`retention-default`: `cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.41s
```

`retention-default-emit`: `cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`; exit 101.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.68s
```

`retention-no-default`: `cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.21s
```

`retention-no-default-emit`: `cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`; exit 101.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.23s
```

`typed-runtime-initial`: `cargo test -p rum-runtime --lib suspension::typed::tests`; exit 0.

```text
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 0.50s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`typed-s6-oracle-default`: `cargo test -p rum-stdlib --test suspension_oracle guards_binding_completion -- --nocapture`; exit 0.

```text
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.20s
test oracle::fixtures::guards_binding_completion::cases ... continuation-slot fault-occurrence delta=0; observations=12
test oracle::fixtures::guards_binding_completion::cross_type_twice ... continuation-slot fault-occurrence delta=0; observations=24
test oracle::fixtures::guards_binding_completion::default_guard ... continuation-slot fault-occurrence delta=0; observations=36
test oracle::fixtures::guards_binding_completion::false_guard ... continuation-slot fault-occurrence delta=0; observations=48
test oracle::fixtures::guards_binding_completion::left_then_right_false ... continuation-slot fault-occurrence delta=0; observations=60
test oracle::fixtures::guards_binding_completion::left_then_right_true ... continuation-slot fault-occurrence delta=0; observations=72
test oracle::fixtures::guards_binding_completion::map_fallback ... continuation-slot fault-occurrence delta=0; observations=84
test oracle::fixtures::guards_binding_completion::match_false ... continuation-slot fault-occurrence delta=0; observations=96
test oracle::fixtures::guards_binding_completion::match_true ... continuation-slot fault-occurrence delta=0; observations=108
test oracle::fixtures::guards_binding_completion::multiple_bindings ... continuation-slot fault-occurrence delta=0; observations=120
test oracle::fixtures::guards_binding_completion::nested_calls ... continuation-slot fault-occurrence delta=0; observations=132
test oracle::fixtures::guards_binding_completion::nested_false ... continuation-slot fault-occurrence delta=0; observations=144
test oracle::fixtures::guards_binding_completion::nested_guards ... continuation-slot fault-occurrence delta=0; observations=156
test oracle::fixtures::guards_binding_completion::non_boolean_guard ... continuation-slot fault-occurrence delta=0; observations=168
test oracle::fixtures::guards_binding_completion::own_false ... continuation-slot fault-occurrence delta=0; observations=180
test oracle::fixtures::guards_binding_completion::own_true ... continuation-slot fault-occurrence delta=0; observations=192
test oracle::fixtures::guards_binding_completion::readonly_token_twice ... continuation-slot fault-occurrence delta=0; observations=204
test oracle::fixtures::guards_binding_completion::replace_all ... continuation-slot fault-occurrence delta=0; observations=216
test oracle::fixtures::guards_binding_completion::replace_part ... continuation-slot fault-occurrence delta=0; observations=228
test oracle::fixtures::guards_binding_completion::single_guard ... continuation-slot fault-occurrence delta=0; observations=240
test oracle::fixtures::guards_binding_completion::string_replace ... continuation-slot fault-occurrence delta=0; observations=252
test oracle::fixtures::guards_binding_completion::string_route_twice ... continuation-slot fault-occurrence delta=0; observations=264
test oracle::fixtures::guards_binding_completion::subvalue ... continuation-slot fault-occurrence delta=0; observations=276
test oracle::fixtures::guards_binding_completion::table_guards ... continuation-slot fault-occurrence delta=0; observations=288
test oracle::fixtures::guards_binding_completion::upvalue ... continuation-slot fault-occurrence delta=0; observations=300
```

`workspace`: `cargo test --workspace --no-fail-fast`; exit 101.

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 51.34s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.99s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: FAILED. 142 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 1184 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 43.48s
test result: ok. 1109 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1290.95s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.13s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 161 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.94s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 235.74s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.37s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.55s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.65s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.54s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`workspace-native-skips`: `cargo test --workspace --no-fail-fast native_zeromq -- --nocapture`; exit 0.

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1162 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 100 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 71 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 40 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 193 filtered out; finished in 0.00s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```
Complete oracle final lines:

```text
test result: ok. 161 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.75s
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=90618
test result: ok. 161 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.13s
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=90618
```
Explicit native ZeroMQ skips:

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```
Standalone native-stack calibration, candidate and resident-backtrace probe totals, verbatim (one exact test invocation per file):

```text
stack/base/debug/backtrace-guarded-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-guarded-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.05s
stack/base/debug/backtrace-peel-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.15s
stack/base/debug/backtrace-peel-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-plus-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-plus-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-rhs-guard-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-rhs-guard-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-sub-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-sub-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.05s
stack/base/debug/backtrace-up-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-up-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.05s
stack/base/debug/guarded-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/guarded-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/peel-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/peel-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/peel-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/peel-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/peel-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/peel-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/peel-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/peel-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/peel-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/peel-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/peel-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/peel-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/plus-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/rhs-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/rhs-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/sub-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/sub-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/sub-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/sub-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/sub-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/sub-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/sub-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/sub-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/sub-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/sub-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/sub-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/sub-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/up-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/up-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/release/backtrace-guarded-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-guarded-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-peel-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-peel-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-plus-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-plus-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-rhs-guard-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-rhs-guard-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-sub-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-sub-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-up-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-up-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/guarded-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/peel-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/plus-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/rhs-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/sub-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/up-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/debug/backtrace-guarded-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-guarded-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.05s
stack/candidate/debug/backtrace-peel-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-peel-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-plus-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.05s
stack/candidate/debug/backtrace-plus-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-rhs-guard-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-rhs-guard-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.05s
stack/candidate/debug/backtrace-sub-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-sub-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-up-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-up-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/guarded-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/guarded-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/guarded-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/guarded-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/guarded-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/guarded-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/guarded-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/guarded-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/guarded-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/guarded-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/guarded-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/candidate/debug/guarded-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/peel-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/candidate/debug/peel-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/peel-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/peel-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/peel-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/peel-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/peel-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/peel-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/peel-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/peel-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/peel-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/peel-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/plus-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/candidate/debug/rhs-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/candidate/debug/sub-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/sub-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/sub-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/sub-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/sub-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/sub-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/sub-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/sub-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/sub-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/sub-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/sub-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/candidate/debug/sub-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/up-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/release/backtrace-guarded-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/backtrace-guarded-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/backtrace-peel-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/backtrace-peel-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/backtrace-plus-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/backtrace-plus-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/backtrace-rhs-guard-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/backtrace-rhs-guard-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/backtrace-sub-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/backtrace-sub-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/backtrace-up-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/backtrace-up-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/guarded-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/peel-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s
stack/candidate/release/peel-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/peel-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/peel-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/peel-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/peel-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/peel-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/peel-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/peel-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/peel-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/peel-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/peel-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/plus-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/rhs-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/sub-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/release/up-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
```
Canonical output roots written by unchanged gate tools:

```text
target/
target/brief-535/
target/brief-539/falsifiers/
target/brief-540/falsifiers/
target/brief-540/oracle/
target/brief-544/
.temp/brief-542/falsifiers/
.temp/brief-543/falsifiers/
.temp/brief-543/oracle-reads/
.temp/brief-546/certificates/
.temp/brief-546/falsifiers/
.temp/typed-slot-census/
.temp/typed-slot-compile-controls/
.temp/suspension-oracle/
```
The full command/exit index, including all measurement stages, is [gates-index.json](../../../.temp/brief-614/gates-index.json). Raw logs remain beside every indexed command.

### 5. Performance evidence and residuals

Every heavy measurement ran serially with bounded process groups. Candidate
builds use the supplied base commands, the same repository cwd and separate
target directories. Source hashes match the frozen candidate build inputs.
The supplied base CLI and allocation binary hashes were checked, and the
base export matched all 1,490 original source/build inputs before any edit.
Eight primary measurement binary hashes are frozen. Six-shape base stack
calibration matches the supplied G14 figures exactly. Supplementary debug CLI
and aligned builds have their own command/source/binary manifests.

Allocations ran first. All 117 ordinary labels are exactly equal in count and
bytes at original N and 2N and in slope; every constant intercept delta is zero.
All 37 resume labels reconcile exactly to one 24-byte outer adapter allocation
per additional typed entry, with zero allocation and byte intercepts. Twenty-eight
resume labels are unchanged. No further allocation difference needs a mode-wire
exemption. Frame states retain the existing clones; no hidden owned-tree clone
was introduced on the ordinary path.
Raw whole counts, slopes and result/pause preflights: [perf/allocation-comparison.tsv](../../../.temp/brief-614/perf/allocation-comparison.tsv), [perf/allocation-frame-proof.json](../../../.temp/brief-614/perf/allocation-frame-proof.json), [perf/measured-binaries.json](../../../.temp/brief-614/perf/measured-binaries.json).

| Resume workload | N | Pauses/iteration | Typed transfers base → candidate | Δ allocations | Δ allocated bytes | Δ driver copies / bytes |
| --- | --- | --- | --- | --- | --- | --- |
| resume-table | 1000 | 1 | 3 → 3 | 0.0 | 0.0 | 0 / 0 |
| resume-do-chain | 1000 | 1 | 3 → 3 | 0.0 | 0.0 | 0 / 0 |
| resume-call-repeat | 1000 | 1 | 4 → 4 | 0.0 | 0.0 | 0 / 0 |
| resume-cleanup | 1000 | 1 | 3 → 3 | 0.0 | 0.0 | 0 / 0 |
| resume-fold | 1000 | 1 | 5 → 5 | 0.0 | 0.0 | 0 / 0 |
| resume-argument-twice | 1000 | 2 | 17 → 17 | 0.0 | 0.0 | 0 / 0 |
| resume-rewrite-exprs | 1000 | 1 | 11 → 11 | 0.0 | 0.0 | 0 / 0 |
| resume-rewrite-compiled | 1000 | 1 | 10 → 10 | 0.0 | 0.0 | 0 / 0 |
| resume-cases-spec | 1000 | 1 | 14 → 14 | 0.0 | 0.0 | 0 / 0 |
| resume-cases-traversal | 1000 | 1 | 11 → 11 | 0.0 | 0.0 | 0 / 0 |
| resume-position-traversal | 1000 | 1 | 11 → 11 | 0.0 | 0.0 | 0 / 0 |
| resume-count-traversal | 1000 | 1 | 11 → 11 | 0.0 | 0.0 | 0 / 0 |
| resume-cases-shared | 1000 | 2 | 22 → 22 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-down-rhs | 100 | 1 | 12 → 12 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-down-guard | 100 | 2 | 19 → 20 | 1.0 | 24.0 | 1 / 136 |
| resume-s5-sub-rhs | 100 | 1 | 12 → 12 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-sub-guard | 100 | 2 | 19 → 20 | 1.0 | 24.0 | 1 / 136 |
| resume-s5-up-rhs | 100 | 1 | 12 → 12 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-up-guard | 100 | 2 | 19 → 20 | 1.0 | 24.0 | 1 / 136 |
| resume-s5-default-rhs | 100 | 1 | 13 → 13 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-default-guard | 100 | 2 | 20 → 21 | 1.0 | 24.0 | 1 / 136 |
| resume-s5-default-fallback-rhs | 100 | 1 | 13 → 13 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-set | 100 | 1 | 13 → 13 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-upset | 100 | 1 | 13 → 13 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-tagset | 100 | 1 | 13 → 13 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-rule-rhs | 100 | 1 | 16 → 16 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-rule-lhs | 100 | 1 | 17 → 17 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-destructure | 100 | 1 | 14 → 14 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-lhs-twice | 100 | 2 | 20 → 20 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-own-rhs | 100 | 1 | 11 → 11 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-destructure-twice | 100 | 2 | 22 → 22 | 0.0 | 0.0 | 0 / 0 |
| resume-s5-rule-lhs-twice | 100 | 2 | 24 → 24 | 0.0 | 0.0 | 0 / 0 |
| resume-s6-rhs-single | 100 | 1 | 12 → 13 | 1.0 | 24.0 | 1 / 136 |
| resume-s6-rhs-twice | 100 | 2 | 18 → 20 | 2.0 | 48.0 | 2 / 272 |
| resume-s6-cross-type | 100 | 2 | 18 → 21 | 3.0 | 72.0 | 3 / 408 |
| resume-s6-readonly-twice | 100 | 4 | 30 → 34 | 4.0 | 96.0 | 4 / 544 |
| resume-s6-string | 100 | 1 | 12 → 14 | 2.0 | 48.0 | 2 / 272 |

Two interleaved native perf N/2N pairs per arm cover all 154 labels, on CPU 2,
using `cpu_core/instructions/u` and stdin `/dev/null`. The exception set was
conservative: maximum paired increase was compared with
`max(0.05% of base mean, base range, candidate range)`, so it includes labels
whose mean alone would not be flagged. All 12 flagged ordinary labels and all
three leaves received three Callgrind N/2N pairs (six processes per arm).
Every ordinary label received at least one Callgrind pair. The original N is
retained for native instructions and allocations; the ordinary address/copy
profiles use the disclosed S5-style instruction budget (up to 2,000 iterations
and about 20 million instructions), with both N and 2N measured.

Every nonzero instruction address has assembly, including resolved PLT/GOT,
libc IFUNC and 72 cached-register copy-call sites. The initial copy resolver
failed closed on unresolved indirect calls; that log is retained. After the
resolution replay, memcpy and memmove counts are not above base on all 117
labels, in whole counts at profiled N and 2N and in slope. Ordinary copy
non-growth is not inferred from native allocation counts.
Full native slopes: [perf/instruction-comparison.tsv](../../../.temp/brief-614/perf/instruction-comparison.tsv). Complete copy proof: [ordinary-callgrind/copy-gate-resolved.tsv](../../../.temp/brief-614/ordinary-callgrind/copy-gate-resolved.tsv) and [ordinary-callgrind/copy-callers-resolved.tsv](../../../.temp/brief-614/ordinary-callgrind/copy-callers-resolved.tsv).

| Repeated ordinary label | Native mean Δ I | Flag threshold | CG mean Δ I | Stable function Δ I in three pairs | Variable functions outside 1 SE |
| --- | --- | --- | --- | --- | --- |
| lazy-range-normal-small | 4.980025 | 48.198900 | 35.809651 | none | 4 |
| lazy-foldlist-realize-small | 18.448975 | 60.102950 | -3.561111 | none | 2 |
| lazy-drop-realize-small | 128.111350 | 301.987150 | -183.691824 | none | 3 |
| lazy-table-construct-full | 110.698675 | 40.656100 | -34.122116 | none | 4 |
| lazy-subsets-take-small | 7.655800 | 16.503400 | -12.122653 | none | 3 |
| do-count-null | -0.018650 | 0.267520 | -0.305500 | none | 3 |
| do-iterator-null | 0.028200 | 0.576006 | -0.137167 | none | 3 |
| table-iterator-null | 0.108725 | 0.193554 | -0.680333 | none | 4 |
| lazy-depth-map-build-64 | 5.789000 | 17.595000 | -11.118408 | none | 4 |
| literal-integer | -0.184825 | 0.409100 | 0.076667 | rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom: -0.006500 | 5 |
| replace-leaf-rebuild-small | 1449.977000 | 1852.420000 | 1201.164234 | none | 5 |
| rewrite-guarded-hit | 139.013000 | 76.422000 | 40.097625 | rum_runtime::runtime::pattern_matching::core::<impl rum_runtime::runtime::model::session::Session>::match_compiled_owned: 1.000000; rum_runtime::runtime::definitions::rhs_guards::<impl rum_runtime::runtime::model::session::Session>::evaluate_owned_guards: 1.000000 | 3 |
| appendto-part-outer1000-small | 11387.185000 | 15717.210000 | -1468.074074 | none | 4 |
| upvalue-candidates-tiny | 10.078750 | 22.645500 | -6.556777 | none | 6 |
| stringreplace-guarded-hit | 36.712000 | 92.892000 | -7.817333 | rum_runtime::runtime::definitions::rhs_guards::evaluate_definition_rhs_guards_for_match: -2.000000; rum_runtime::runtime::definitions::rhs_guards::<impl rum_runtime::runtime::model::session::Session>::evaluate_owned_guards: 1.000000 | 4 |

The ordinary deterministic defect is **+2 instructions/iteration on
`rewrite-guarded-hit`**: +1 in `evaluate_owned_guards`, +1 in
`match_compiled_owned`. The former adds the flag spill at candidate `0x5a9a0e`
(`mov %eax,0xc(%rsp)`); the latter changes the two moves at base
`0x6da05e,0x6da061` into three at candidate
`0x6dab3e,0x6dab41,0x6dab44`. The latter function's source is unchanged.
The full aligned address ledger retains all compensating moves, not only those
isolated examples. `stringreplace-guarded-hit` has +1 in the same guard helper
and -2 in the public string adapter (net -1). These are not rounded to zero.

All three leaves show no deterministic instruction growth by address. In
`Session::evaluate`, every leaf measures exactly `[110,110,110]` instructions
per iteration in both arms with zero changed addresses. Whole-program counts
are not exactly equal: variable startup/hash/allocation residues remain, and
the raw whole-label no-growth booleans are false. Stable negative
`literal-integer` bloom-cache delta is also retained. Positive variable
differences (including `replace-leaf-rebuild-small` and lazy range) are submitted
as residuals, not dismissed because the deterministic subtotal is zero.
Address proofs: [ordinary-callgrind/leaf-proof.json](../../../.temp/brief-614/ordinary-callgrind/leaf-proof.json), [ordinary-callgrind/repeated-summary.json](../../../.temp/brief-614/ordinary-callgrind/repeated-summary.json). For each label, `attribution/cli/<label>-address-deltas.tsv` names every address delta and `<label>-address-proof.json` checks its sum against each pair total (rounding residual below 1e-5). `<label>-whole-process-functions.tsv` diffs every function at N and 2N, including constant overhead.
Stack measurements use heights 2, 4, 6 and 8, three samples each, in both profiles and arms. All fit residuals are zero. Resident backtraces and prologue multiplicities reconcile every measured delta, with zero unattributed bytes per level:

| Profile | Shape | Base bytes/level | Candidate bytes/level | Δ | Fit / resident residual |
| --- | --- | --- | --- | --- | --- |
| debug | peel | 19024 | 19024 | 0 | 0 / 0 |
| debug | plus | 34912 | 34912 | 0 | 0 / 0 |
| debug | sub | 29376 | 29376 | 0 | 0 / 0 |
| debug | up | 28064 | 28064 | 0 | 0 / 0 |
| debug | guarded | 35840 | 35840 | 0 | 0 / 0 |
| debug | rhs-guard | 49120 | 49152 | 32 | 0 / 0 |
| release | peel | 7344 | 7344 | 0 | 0 / 0 |
| release | plus | 13792 | 13792 | 0 | 0 / 0 |
| release | sub | 13168 | 13168 | 0 | 0 / 0 |
| release | up | 11456 | 11456 | 0 | 0 / 0 |
| release | guarded | 14240 | 14240 | 0 | 0 / 0 |
| release | rhs-guard | 17920 | 17920 | 0 | 0 / 0 |

**Debug RHS-guard growth is +32 bytes/level and requires the director's
ruling.** One resident `match_compiled_definition_condition_owned` frame grows
792 → 808 bytes, and one `evaluate_owned_guards` frame grows 1,048 → 1,064.
The producer Result slots remain at offsets 200 and 216 respectively, with
size 136. No producer Result is moved to the cold helper. The slot table below
distinguishes actual data from compiler reserve/padding: each frame has an
additional eight-byte gap before error storage and eight-byte reserve containing
the zero-sized witness's debug location. Two additional drop flags consume two
previous padding bytes. DWARF proves every witness is zero bytes; reserve bytes
are not claimed as witness payload or attributed to an invented source value.

| Function | Slot | Offset base → candidate | Bytes base → candidate | Δ |
| --- | --- | --- | --- | --- |
| match_compiled_definition_condition_owned | producer Result (inner_result) | 200 → 200 | 136 → 136 | 0 |
| match_compiled_definition_condition_owned | outcome | 448 → 448 | 40 → 40 | 0 |
| match_compiled_definition_condition_owned | request | 544 → 544 | 32 → 32 | 0 |
| match_compiled_definition_condition_owned | outgoing option scope | 576 → 576 | 48 → 48 | 0 |
| match_compiled_definition_condition_owned | unreferenced compiler reserve before error | 624 → 624 | 0 → 8 | 8 |
| match_compiled_definition_condition_owned | error | 624 → 632 | 136 → 136 | 0 |
| match_compiled_definition_condition_owned | padding before drop flags | 760 → 768 | 2 → 0 | -2 |
| match_compiled_definition_condition_owned | drop flags | 762 → 768 | 6 → 8 | 2 |
| match_compiled_definition_condition_owned | self debug spill | 768 → 776 | 8 → 8 | 0 |
| match_compiled_definition_condition_owned | reserve containing witness debug location | 776 → 784 | 0 → 8 | 8 |
| match_compiled_definition_condition_owned | unwind payload and selector | 776 → 792 | 12 → 12 | 0 |
| match_compiled_definition_condition_owned | tail alignment | 788 → 804 | 4 → 4 | 0 |
| evaluate_owned_guards | producer Result (guard_result) | 216 → 216 | 136 → 136 | 0 |
| evaluate_owned_guards | result | 432 → 432 | 32 → 32 | 0 |
| evaluate_owned_guards | request | 688 → 688 | 32 → 32 | 0 |
| evaluate_owned_guards | outgoing guard/binding state | 720 → 720 | 144 → 144 | 0 |
| evaluate_owned_guards | unreferenced compiler reserve before error | 864 → 864 | 0 → 8 | 8 |
| evaluate_owned_guards | error | 864 → 872 | 136 → 136 | 0 |
| evaluate_owned_guards | padding before drop flags | 1000 → 1008 | 6 → 4 | -2 |
| evaluate_owned_guards | drop flags | 1006 → 1012 | 10 → 12 | 2 |
| evaluate_owned_guards | self debug spill | 1016 → 1024 | 8 → 8 | 0 |
| evaluate_owned_guards | reserve containing witness debug location | 1024 → 1032 | 0 → 8 | 8 |
| evaluate_owned_guards | unwind payload and selector | 1024 → 1040 | 12 → 12 | 0 |
| evaluate_owned_guards | alignment before guard pointer | 1036 → 1052 | 4 → 4 | 0 |
| evaluate_owned_guards | guard pointer debug spill | 1040 → 1056 | 8 → 8 | 0 |

Prologue coverage for all 19 changed source functions, including absent/inlined status (never treated as a zero-byte measurement):

| Binary | Profile | Function | Base prologue bytes | Candidate prologue bytes |
| --- | --- | --- | --- | --- |
| runtime-probe | debug | match_compiled_definition_condition_owned | 792 | 808 |
| runtime-probe | debug | suspend_definition_condition_match | 440 | 312 |
| runtime-probe | debug | evaluate_guard_sequence_readonly_suspended | 136 | 136 |
| runtime-probe | debug | evaluate_owned_guards | 1048 | 1064 |
| runtime-probe | debug | suspend_owned_guards | 664 | 536 |
| runtime-probe | debug | resume_suspended_guard_deferred | 200 | 216, 216 |
| runtime-probe | debug | evaluate_suspended_guards_deferred | 1064 | 920, 936 |
| runtime-probe | debug | suspend_deferred_guards | new helper: absent in base | 296, 312 |
| runtime-probe | debug | evaluate_readonly_suspended_guards | 1000 | 904, 920 |
| runtime-probe | debug | suspend_readonly_guards | new helper: absent in base | 136, 136 |
| runtime-probe | debug | evaluate_definition_rhs_guards_for_match | not linked in runtime-only probe | not linked in runtime-only probe |
| runtime-probe | debug | resume_guard_frame | 664 | 680 |
| runtime-probe | debug | continue_guard_frame | 168 | 312 |
| runtime-probe | debug | complete_guard_frame | 312 | 8 |
| runtime-probe | debug | MatchCompiledDefinitionConditionOwnedSuspensionFrame::resume | 1032 | 1000 |
| runtime-probe | debug | EvaluateOwnedGuardsSuspensionFrame::resume | 328 | 344 |
| runtime-probe | debug | EvaluateReadonlySuspendedGuardsSuspensionFrame::resume | 872 | 888 |
| runtime-probe | debug | EvaluateDefinitionRhsGuardsForMatchSuspensionFrame::resume | not linked in runtime-only probe | not linked in runtime-only probe |
| runtime-probe | debug | GuardOutcomeFinalizer::finish | 168 | 136 |
| runtime-probe | release | match_compiled_definition_condition_owned | no separate symbol; optimized/inlined; not a zero-byte measurement | no separate symbol; optimized/inlined; not a zero-byte measurement |
| runtime-probe | release | suspend_definition_condition_match | 104 | 136 |
| runtime-probe | release | evaluate_guard_sequence_readonly_suspended | no separate symbol; optimized/inlined; not a zero-byte measurement | no separate symbol; optimized/inlined; not a zero-byte measurement |
| runtime-probe | release | evaluate_owned_guards | 440 | 440 |
| runtime-probe | release | suspend_owned_guards | 296 | 312 |
| runtime-probe | release | resume_suspended_guard_deferred | 152 | 520 |
| runtime-probe | release | evaluate_suspended_guards_deferred | 440 | 536 |
| runtime-probe | release | suspend_deferred_guards | new helper: absent in base | 24, 232 |
| runtime-probe | release | evaluate_readonly_suspended_guards | 376 | 440, 440 |
| runtime-probe | release | suspend_readonly_guards | new helper: absent in base | 120, 24 |
| runtime-probe | release | evaluate_definition_rhs_guards_for_match | not linked in runtime-only probe | not linked in runtime-only probe |
| runtime-probe | release | resume_guard_frame | no separate symbol; optimized/inlined; not a zero-byte measurement | no separate symbol; optimized/inlined; not a zero-byte measurement |
| runtime-probe | release | continue_guard_frame | no separate symbol; optimized/inlined; not a zero-byte measurement | no separate symbol; optimized/inlined; not a zero-byte measurement |
| runtime-probe | release | complete_guard_frame | no separate symbol; optimized/inlined; not a zero-byte measurement | no separate symbol; optimized/inlined; not a zero-byte measurement |
| runtime-probe | release | MatchCompiledDefinitionConditionOwnedSuspensionFrame::resume | 520 | no separate symbol; optimized/inlined; not a zero-byte measurement |
| runtime-probe | release | EvaluateOwnedGuardsSuspensionFrame::resume | 456 | no separate symbol; optimized/inlined; not a zero-byte measurement |
| runtime-probe | release | EvaluateReadonlySuspendedGuardsSuspensionFrame::resume | 520 | no separate symbol; optimized/inlined; not a zero-byte measurement |
| runtime-probe | release | EvaluateDefinitionRhsGuardsForMatchSuspensionFrame::resume | not linked in runtime-only probe | not linked in runtime-only probe |
| runtime-probe | release | GuardOutcomeFinalizer::finish | 104 | no separate symbol; optimized/inlined; not a zero-byte measurement |
| cli | debug | match_compiled_definition_condition_owned | 792 | 808 |
| cli | debug | suspend_definition_condition_match | 440 | 312 |
| cli | debug | evaluate_guard_sequence_readonly_suspended | 136 | 136 |
| cli | debug | evaluate_owned_guards | 1048 | 1064 |
| cli | debug | suspend_owned_guards | 664 | 536 |
| cli | debug | resume_suspended_guard_deferred | 200 | 216, 216 |
| cli | debug | evaluate_suspended_guards_deferred | 1064 | 920, 936 |
| cli | debug | suspend_deferred_guards | new helper: absent in base | 312, 296 |
| cli | debug | evaluate_readonly_suspended_guards | 1000 | 920, 904 |
| cli | debug | suspend_readonly_guards | new helper: absent in base | 136, 136 |
| cli | debug | evaluate_definition_rhs_guards_for_match | 872 | 744 |
| cli | debug | resume_guard_frame | 664 | 680 |
| cli | debug | continue_guard_frame | 168 | 312 |
| cli | debug | complete_guard_frame | 312 | 8 |
| cli | debug | MatchCompiledDefinitionConditionOwnedSuspensionFrame::resume | 1032 | 1000 |
| cli | debug | EvaluateOwnedGuardsSuspensionFrame::resume | 328 | 344 |
| cli | debug | EvaluateReadonlySuspendedGuardsSuspensionFrame::resume | 872 | 888 |
| cli | debug | EvaluateDefinitionRhsGuardsForMatchSuspensionFrame::resume | 696 | 536 |
| cli | debug | GuardOutcomeFinalizer::finish | 168 | 136 |
| cli | release | match_compiled_definition_condition_owned | no separate symbol; optimized/inlined; not a zero-byte measurement | no separate symbol; optimized/inlined; not a zero-byte measurement |
| cli | release | suspend_definition_condition_match | 104 | 136 |
| cli | release | evaluate_guard_sequence_readonly_suspended | no separate symbol; optimized/inlined; not a zero-byte measurement | no separate symbol; optimized/inlined; not a zero-byte measurement |
| cli | release | evaluate_owned_guards | 440 | 440 |
| cli | release | suspend_owned_guards | 296 | 312 |
| cli | release | resume_suspended_guard_deferred | 152 | 520 |
| cli | release | evaluate_suspended_guards_deferred | 440 | 536 |
| cli | release | suspend_deferred_guards | new helper: absent in base | 24, 232 |
| cli | release | evaluate_readonly_suspended_guards | 376 | 440, 440 |
| cli | release | suspend_readonly_guards | new helper: absent in base | 120, 24 |
| cli | release | evaluate_definition_rhs_guards_for_match | 344 | 360 |
| cli | release | resume_guard_frame | no separate symbol; optimized/inlined; not a zero-byte measurement | no separate symbol; optimized/inlined; not a zero-byte measurement |
| cli | release | continue_guard_frame | no separate symbol; optimized/inlined; not a zero-byte measurement | no separate symbol; optimized/inlined; not a zero-byte measurement |
| cli | release | complete_guard_frame | no separate symbol; optimized/inlined; not a zero-byte measurement | no separate symbol; optimized/inlined; not a zero-byte measurement |
| cli | release | MatchCompiledDefinitionConditionOwnedSuspensionFrame::resume | 520 | 424 |
| cli | release | EvaluateOwnedGuardsSuspensionFrame::resume | 456 | 472 |
| cli | release | EvaluateReadonlySuspendedGuardsSuspensionFrame::resume | 520 | 408 |
| cli | release | EvaluateDefinitionRhsGuardsForMatchSuspensionFrame::resume | 280 | 200 |
| cli | release | GuardOutcomeFinalizer::finish | 104 | no separate symbol; optimized/inlined; not a zero-byte measurement |

Closure/specialization prologues and exact addresses are in [stack/cli/changed-prologues.tsv](../../../.temp/brief-614/stack/cli/changed-prologues.tsv) and [stack/changed-prologues.tsv](../../../.temp/brief-614/stack/changed-prologues.tsv). The complete slot evidence is [stack/all-guard-dwarf-slots.tsv](../../../.temp/brief-614/stack/all-guard-dwarf-slots.tsv) (829 named debug slots), [stack/all-guard-stack-accesses.tsv](../../../.temp/brief-614/stack/all-guard-stack-accesses.tsv) (4,947 explicit stack-relative instructions), and [stack/resident-deltas.tsv](../../../.temp/brief-614/stack/resident-deltas.tsv).
Cold prologue increases are also residuals. In the release CLI, these include
F0267 resume +16, the public string adapter +16, readonly helper +64,
deferred helper +96, `resume_suspended_guard_deferred` +368,
`suspend_definition_condition_match` +32 and `suspend_owned_guards` +16.
Debug `continue_guard_frame` grows +144, while `complete_guard_frame` shrinks
304. The M0277 named macro closure environment is 40 bytes (request 32,
session reference 8; witness and F0269 frame zero), proved by the debug `make`
parameter type. Its new release cold helper prologue is 88 bytes. Other cold
growth is not justified by that closure environment. Compiler-generated
spill/reserve positions are retained in the slot ledger; where source names
are absent, the report does not invent them or claim the stack bar passed.

Every one of the 37 resume workloads has three N/2N pairs per arm, at original
N. Every function is diffed per process, every nonzero instruction address is
resolved, and address sums reconcile all three total deltas. Bodies, allocation
callers, mode-wire/adapter/frame/queue/driver categories, copy calls and SIMD
moves are retained separately under `attribution/resume/`. Stable deltas mean
constant in the three observed pairs; variable differences remain explicit.

| Resume workload | Mean total Δ I/iteration | Stable subtotal Δ I | Variable functions | Address ledger |
| --- | --- | --- | --- | --- |
| resume-table | 41.103667 | 0.000000 | 8 | [attribution/resume/resume-table-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-table-address-deltas.tsv) |
| resume-do-chain | -53.440333 | 0.000000 | 8 | [attribution/resume/resume-do-chain-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-do-chain-address-deltas.tsv) |
| resume-call-repeat | -26.239667 | 0.000000 | 9 | [attribution/resume/resume-call-repeat-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-call-repeat-address-deltas.tsv) |
| resume-cleanup | 18.266333 | 0.000000 | 7 | [attribution/resume/resume-cleanup-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-cleanup-address-deltas.tsv) |
| resume-fold | 27.983333 | 0.000000 | 9 | [attribution/resume/resume-fold-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-fold-address-deltas.tsv) |
| resume-argument-twice | -77.475000 | 0.000000 | 9 | [attribution/resume/resume-argument-twice-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-argument-twice-address-deltas.tsv) |
| resume-rewrite-exprs | 59.233667 | 0.000000 | 8 | [attribution/resume/resume-rewrite-exprs-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-rewrite-exprs-address-deltas.tsv) |
| resume-rewrite-compiled | -92.058333 | 0.000000 | 23 | [attribution/resume/resume-rewrite-compiled-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-rewrite-compiled-address-deltas.tsv) |
| resume-cases-spec | 82.006000 | 0.000000 | 10 | [attribution/resume/resume-cases-spec-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-cases-spec-address-deltas.tsv) |
| resume-cases-traversal | -9.299667 | 0.000000 | 24 | [attribution/resume/resume-cases-traversal-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-cases-traversal-address-deltas.tsv) |
| resume-position-traversal | -108.111000 | 0.000000 | 10 | [attribution/resume/resume-position-traversal-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-position-traversal-address-deltas.tsv) |
| resume-count-traversal | -24.168000 | 0.000000 | 23 | [attribution/resume/resume-count-traversal-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-count-traversal-address-deltas.tsv) |
| resume-cases-shared | -225.735000 | 0.000000 | 8 | [attribution/resume/resume-cases-shared-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-cases-shared-address-deltas.tsv) |
| resume-s5-down-rhs | 29.383333 | 0.000000 | 8 | [attribution/resume/resume-s5-down-rhs-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-down-rhs-address-deltas.tsv) |
| resume-s5-down-guard | 66.483333 | 178.000000 | 8 | [attribution/resume/resume-s5-down-guard-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-down-guard-address-deltas.tsv) |
| resume-s5-sub-rhs | 83.573333 | 0.000000 | 9 | [attribution/resume/resume-s5-sub-rhs-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-sub-rhs-address-deltas.tsv) |
| resume-s5-sub-guard | 264.680000 | 178.000000 | 8 | [attribution/resume/resume-s5-sub-guard-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-sub-guard-address-deltas.tsv) |
| resume-s5-up-rhs | -15.703333 | 0.000000 | 10 | [attribution/resume/resume-s5-up-rhs-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-up-rhs-address-deltas.tsv) |
| resume-s5-up-guard | -192.523333 | 178.000000 | 7 | [attribution/resume/resume-s5-up-guard-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-up-guard-address-deltas.tsv) |
| resume-s5-default-rhs | 72.260000 | 0.000000 | 10 | [attribution/resume/resume-s5-default-rhs-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-default-rhs-address-deltas.tsv) |
| resume-s5-default-guard | 138.606667 | 148.000000 | 9 | [attribution/resume/resume-s5-default-guard-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-default-guard-address-deltas.tsv) |
| resume-s5-default-fallback-rhs | -3.340000 | 0.000000 | 8 | [attribution/resume/resume-s5-default-fallback-rhs-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-default-fallback-rhs-address-deltas.tsv) |
| resume-s5-set | 20.306667 | 0.000000 | 23 | [attribution/resume/resume-s5-set-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-set-address-deltas.tsv) |
| resume-s5-upset | -14.386667 | 0.000000 | 8 | [attribution/resume/resume-s5-upset-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-upset-address-deltas.tsv) |
| resume-s5-tagset | 11.043333 | 0.000000 | 8 | [attribution/resume/resume-s5-tagset-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-tagset-address-deltas.tsv) |
| resume-s5-rule-rhs | 5.110000 | 0.000000 | 9 | [attribution/resume/resume-s5-rule-rhs-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-rule-rhs-address-deltas.tsv) |
| resume-s5-rule-lhs | 100.676667 | 0.000000 | 9 | [attribution/resume/resume-s5-rule-lhs-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-rule-lhs-address-deltas.tsv) |
| resume-s5-destructure | 77.243333 | 0.000000 | 8 | [attribution/resume/resume-s5-destructure-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-destructure-address-deltas.tsv) |
| resume-s5-lhs-twice | -81.286667 | 0.000000 | 23 | [attribution/resume/resume-s5-lhs-twice-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-lhs-twice-address-deltas.tsv) |
| resume-s5-own-rhs | -58.806667 | 0.000000 | 7 | [attribution/resume/resume-s5-own-rhs-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-own-rhs-address-deltas.tsv) |
| resume-s5-destructure-twice | -40.643333 | 0.000000 | 9 | [attribution/resume/resume-s5-destructure-twice-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-destructure-twice-address-deltas.tsv) |
| resume-s5-rule-lhs-twice | -101.866667 | 0.000000 | 9 | [attribution/resume/resume-s5-rule-lhs-twice-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s5-rule-lhs-twice-address-deltas.tsv) |
| resume-s6-rhs-single | 220.890000 | 179.040000 | 22 | [attribution/resume/resume-s6-rhs-single-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s6-rhs-single-address-deltas.tsv) |
| resume-s6-rhs-twice | 420.770000 | 315.190000 | 10 | [attribution/resume/resume-s6-rhs-twice-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s6-rhs-twice-address-deltas.tsv) |
| resume-s6-cross-type | 443.786667 | 526.000000 | 24 | [attribution/resume/resume-s6-cross-type-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s6-cross-type-address-deltas.tsv) |
| resume-s6-readonly-twice | 913.020000 | 768.070000 | 9 | [attribution/resume/resume-s6-readonly-twice-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s6-readonly-twice-address-deltas.tsv) |
| resume-s6-string | 374.186667 | 350.560000 | 24 | [attribution/resume/resume-s6-string-address-deltas.tsv](../../../.temp/brief-614/attribution/resume/resume-s6-string-address-deltas.tsv) |

The existing driver outcome move is counted, not repaired. The unique pair of
136-byte copy sites whose counts equal every typed adapter transfer in all 37
labels/all three pairs is base `0x74457d,0x74459d` and candidate
`0x74509d,0x7450bd`. Both use `edx=0x88`; the finalizer branch copies
`rsp+0x270 → rsp+0x300`, and the completed-outcome branch copies the pointer
held at `rsp+0x258 → rsp+0x300`. The per-label extra copies/bytes are in the
allocation table above. This is the known R debt; there is no driver edit.
Typed insertion/erasure/queue and finalizer changes, including the stable
guard-helper instruction growth, remain named in the address ledgers rather
than being charged to an unspecified transport bucket.
Driver proof: [attribution/resume/driver-copy-proof.json](../../../.temp/brief-614/attribution/resume/driver-copy-proof.json). All stable per-function deltas and their exact address rows: [attribution/resume/deterministic-function-index.tsv](../../../.temp/brief-614/attribution/resume/deterministic-function-index.tsv). Variable functions and their three samples are retained in each `<label>-semantic-summary.json`.
Timing was last. Four full unchanged core-language panels ran
base/candidate/base/candidate, one-unit binaries, CPU 2, profiling off,
scale 1, two internal repetitions, retained-module GC batches 3. All 120 labels
completed. Timing uses the kernel monotonic `AbsoluteTiming`, not differences
of wall-clock timestamps. Host monitors observed no competing CPU workload;
GPU telemetry was unavailable and is not asserted idle. Of 120 labels, 46
had positive means and 23 had candidate ranges wholly above the base ranges.
All rows, including overlapping/negative changes, remain in the full table.
Full timing values: [perf/timing-comparison.tsv](../../../.temp/brief-614/perf/timing-comparison.tsv). Causal builds use exactly `RUSTFLAGS="-C llvm-args=-align-all-functions=6"` on both arms; [alignment/alignment-proof.json](../../../.temp/brief-614/alignment/alignment-proof.json) confirms all 5,230 base and 5,247 candidate runtime text symbols are 64-byte aligned. Their original measurement binaries remain unchanged.
Each residual probe retains the original benchmark statement and preceding
setup; input width and expression are unchanged. An initial outer-loop probe
failed every magnitude check at about 2x and yielded no cycle inference.
Those raw results are archived under `alignment/probes-initial-wrapped/`.
The corrected calls are top-level, with 64 fixed original-call warmups before
the measured calls. These warmups cancel from the N/2N counter slope. A
three-label pilot showed the transient magnitude problem was reduced before
the full rerun. Five one-call preflights in each binary verify identical result
text and median nanoseconds within 0.75–1.3333 of the full panel. This range
is a probe-validity check, not a performance acceptance threshold. Failed
magnitudes receive no cycle inference; any full-prefix retries are disclosed
in the table's evidence paths. Five interleaved N/2N pairs per arm and binary
family then measure cycles and instructions per original benchmark work unit.
Fixed prelude costs cancel. The original warm call is included in the known
unit denominator (2n+1 for Bench, 3n for Once).

| Label | Full panel Δ % | Magnitude verified | Ordinary cycles/unit B → C (Δ %) | Aligned cycles/unit B → C (Δ %) | Disposition |
| --- | --- | --- | --- | --- | --- |
| if-true | 1.355694 | True | 1803.200 → 1916.803 (+6.300%) | 1867.749 → 1952.707 (+4.549%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| block-local | 1.451417 | True | 4969.739 → 5125.859 (+3.141%) | 4732.848 → 4944.917 (+4.481%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| d-nested-sin-small | 1.194666 | True | 134733.660 → 138227.423 (+2.593%) | 138101.788 → 141350.829 (+2.353%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| downvalue-pattern-miss | 2.021580 | True | 1509.374 → 1528.214 (+1.248%) | 1554.664 → 1506.806 (-3.078%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| downvalue-exact-miss-repeat | 2.777385 | True | 1225.774 → 1206.079 (-1.607%) | 1207.704 → 1270.030 (+5.161%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| downvalue-pattern-miss-repeat | 1.620559 | True | 1277.667 → 1209.253 (-5.355%) | 1292.046 → 1343.082 (+3.950%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| upvalue-guarded-hit | 0.403574 | True | 12158.765 → 12553.201 (+3.244%) | 12141.044 → 12109.356 (-0.261%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| subvalue-unguarded-hit | 0.446015 | True | 7140.144 → 7339.673 (+2.794%) | 6666.649 → 7792.301 (+16.885%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| subvalue-guarded-hit | 3.001361 | True | 12599.262 → 12655.692 (+0.448%) | 12381.238 → 12686.692 (+2.467%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| default-unguarded-hit | 9.706414 | True | 6772.634 → 6753.691 (-0.280%) | 7195.877 → 6614.855 (-8.074%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| default-guarded-hit | 0.758960 | True | 8362.993 → 8740.885 (+4.519%) | 8708.369 → 8483.197 (-2.586%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| replaceall-simple | 0.742596 | True | 3993.678 → 3981.086 (-0.315%) | 4036.263 → 4006.183 (-0.745%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| replacepart-guarded-hit | 0.738246 | True | 5839.623 → 5859.195 (+0.335%) | 6002.005 → 5917.544 (-1.407%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| stringreplace-guarded-hit | 1.027599 | True | 7405.763 → 7394.511 (-0.152%) | 7200.305 → 7266.787 (+0.923%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| stringreplace-guarded-decline-fallback | 1.110815 | True | 8720.776 → 8877.744 (+1.800%) | 8834.473 → 8990.570 (+1.767%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| sort-callback-large | 3.108280 | True | 10336.645 → 10480.507 (+1.392%) | 10755.585 → 10247.752 (-4.722%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| sortby-callback-small | 1.424525 | True | 11017.687 → 11358.723 (+3.095%) | 11182.394 → 11219.375 (+0.331%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| sortby-callback-large | 1.887018 | True | 12058.546 → 12172.088 (+0.942%) | 12059.960 → 11916.096 (-1.193%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| reversesort-callback-large | 0.736996 | True | 9760.707 → 10090.520 (+3.379%) | 9903.155 → 10016.437 (+1.144%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| keysort-callback-large | 3.213967 | True | 10786.029 → 10950.359 (+1.524%) | 10962.555 → 10812.820 (-1.366%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| keysortby-callback-large | 1.369025 | True | 12112.980 → 12375.366 (+2.166%) | 12447.793 → 12397.418 (-0.405%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| sortby-default-wide | 0.925655 | True | 1652.685 → 1774.058 (+7.344%) | 1782.396 → 1757.022 (-1.424%) | Ordinary positive point estimate disappears with both arms aligned, consistent with layout. The unadjusted five-pair interval is borderline and samples overlap; residual remains unaccepted. |
| valueq-undefined-symbol-definitions-2000 | 2.540455 | True | 7412.210 → 9436.356 (+27.308%) | 8025.355 → 7301.109 (-9.024%) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |

The paired 95% intervals (five pairs, t multiplier 2.776), raw sample ranges, instruction slopes and preflight magnitude ratios are in [alignment/comparison.json](../../../.temp/brief-614/alignment/comparison.json). All 23 instruction comparisons in both families are within the measured slope/spread bar; this is not exact-count equality. The intervals are unadjusted for 23 comparisons. Even sortby-default-wide only barely excludes zero before alignment and has overlapping samples, so its layout evidence is weak. Statistical compatibility is disclosed as uncertainty; it does not authorize accepting a regression. Positive residuals after alignment and failures to reproduce a full-panel slowdown remain open for the director.

### 6. Counters, deviations and director decisions

Final recorded counters: **41 compile attempts**,
**457 test/probe/gate commands**, **0 consecutive non-progress
attempts**. These are the persisted orchestration counters, not counts of Rust
test cases or every child process inside an oracle/compiler/measurement panel.
The exact subprocess commands remain in their raw manifests. There are no
half-edited files. The source audit verifies all 1,492 final source/build inputs
against the measured candidate, exactly 17 scoped source/test/inventory changes
(two new files), and unchanged Cargo/lock/toolchain/witness files. The appended
brief report is the eighteenth edited repository file. No git command ran and
no commit was made.

Final audit: [final-source-audit.json](../../../.temp/brief-614/final-source-audit.json).
The measured binary identities are in
[perf/measured-binaries.json](../../../.temp/brief-614/perf/measured-binaries.json).

Measurement repairs and limits are disclosed rather than hidden: the readonly
base workload needed two targets to reach F0268; its initial census is retained.
The new fixture's expected log was corrected to the existing backward walk.
Indirect copy-call resolution initially failed closed, then resolved every site.
The first causal timing construction failed all magnitude checks, and produced
no cycle inference. Top-level calls plus fixed warmup brought 20/23 probes into
range; retaining the complete preceding benchmark program brought the remaining
three into range. Raw rejected runs and pilots remain. Two internal timing
repetitions, the conservative native exception test, reduced ordinary Callgrind
budgets, supplementary debug CLI binaries, and unavailable GPU telemetry are
all explicit above. None changes an oracle check or semantic test contract.

The required gates and measurement panels have run; no mandatory panel is
silently omitted. The nine native ZeroMQ cases remain unexecuted functionally
in this sandbox and are reported as explicit skips after the initial failures.
The following residuals need the director's ruling:

1. `rewrite-guarded-hit` adds two deterministic ordinary instructions at the
   named addresses. The shared helper also adds one instruction in the string
   case, offset there by two removed adapter instructions. This is reported as
   a defect, without accepting a default-path increase.
2. Debug `rhs-guard` recursion grows by 32 bytes/level. The two resident frames,
   exact slot extents, compiler reserve gaps and zero-sized witness types are
   documented. Additional cold-function prologue growth is listed separately;
   unnamed compiler spills/reserves are not presented as a proved source-level
   justification. The stack bar is not declared passed.
3. Variable instruction/cycle residuals remain, including positive ordinary
   differences outside one standard error and resumed guard transport costs.
   All are retained by function and address. The known 136-byte driver move is
   counted as R debt and left untouched. Allocations and ordinary copy counts
   meet their exact respective bars.
4. The alignment experiment is inconclusive for most timing residuals. The
   `sortby-default-wide` point estimate is consistent with layout but has weak,
   borderline statistical evidence and overlapping samples. No timing slowdown
   is cleared solely by that experiment or by a small instruction subtotal.

The implementation and evidence are ready for the authoritative gate. This
report submits the residuals; it does not issue the director's acceptance.
