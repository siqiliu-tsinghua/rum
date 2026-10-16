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
