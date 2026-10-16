# brief-615 — G13 step S7: type Part selectors and held assignment

## Goal

Type the fifth production family through the typed API. S7 owns the frames
that suspend while rum evaluates **Part selectors** for an assignment or a
held update. That means:
- `l[[i, j]] = v`;
- `a[[k]] = v` and `a[k] = v` on associations;
- the held updates `x[[i]] += v`, `x[[i]]++`, `AppendTo[x[[i]], v]` and
  their siblings;
- the stdlib association mutators `AssociateTo` and `KeyDropFrom`, including
  a held Part target.

When you are done:

- the owned rows implement the typed API, and the owned macro sites use the
  witnessed typed macro or the token, as their call path requires;
- **the selector publishing side channel is gone.** It is the
  `Arc<OnceLock<EvaluatedPartSelectors>>` handshake
  (`EvaluatedPartSelectorsContinuation`, `EvaluatedPartSelectorsEvaluation`)
  between the runtime and stdlib. The selectors travel in the typed slot;
- six historical divergences are promoted to strict equality gates;
- every other family still runs through the transport it uses today;
- the ordinary path of every program executes no more work than at S6, and
  the cost of S7's resume path is measured and attributed.

This is step S7 of G13 (design round 5, brief-606, ratified). S1-S6 (briefs
607-611 and 614) and G14 (612-613) are done; S6 is `e9707d15`. S8-S54 type
the remaining families, F1-F23 flip transport per type, and R removes the
legacy machinery.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- Authority, in order: this brief; brief-606 (its D1/D2 and report) with
  `.temp/brief-606/design.md` and `census-spec.md`; `.temp/brief-605/design.md`
  §§2-5, **except §5's transit record, which D2 rejected**. §2 ruled the
  removal of the OnceLock publishing protocol, together with the runtime
  producer and the `associations/mutation{,/resume_frames}.rs` consumer, in
  one migration.
- **S3-S6 as built are your template.** Read brief-611 and brief-614 in
  full, with their director amendments and gate records. Their rulings are
  now the rules:
  - **Two arms on the typed macro.** `resume_typed_or_suspend!` is defined in
    `suspension/typed/witness.rs`. Use the caller-place arm (`$result:ident`)
    wherever the producer's result is already bound to a local. Its third
    arm (`evaluation_return =>`) is for G14's two fixed-point sites only.
  - **The macro expansion is fixed.** Do not change `witness.rs`.
  - **The witness and the request cross a cold call, never the producer's
    `Result`.** Take the ZST witness at the producer, then pass
    `(witness, request)` to a cold helper. A shape that needs an
    `unreachable!` arm for a state the types allow is the wrong shape.
  - **The routes** (S6's table is the newest example):
    - a site reached only from the ordinary path uses the witnessed route;
    - a site reached both from the ordinary path and from its own frame's
      resume uses a route trait. Its ordinary instantiation is `()` and is
      unchanged; the resumed instantiation consumes the token with fresh
      state;
    - a resume that inserts a *different* frame type uses the witnessed
      route and then stops with `ResumeStop`.
  - **The token** re-inserts fresh state of the resuming frame's own type,
    and it carries no `Arc`.
  - **The finalizer erasure.** A typed finalizer is erased as its own box.
    Deferred work returns `Result<T, ResumeStop<O>>`, and `finish_work`
    selects the finalizer.
  - **The retention analyzer reads the typed macro** through
    `retention_closure/typed_resume_macro.rs`.
  - **G14:** typed frames get their evaluation context at the common
    insertion endpoints. A family has nothing to do for it.
- **This step crosses the crate edge.** F0047-F0049 and M0041-M0043 live in
  `rum-stdlib` and use the typed API through `rum_runtime::kernel_api` and
  the exported macro. They are the first stdlib frames to be typed.
  - Nothing that a stdlib frame needs may require a new public raw adapter,
    mode or codec.
  - `DefinitionResumeRoute` (`definitions/application/mod.rs:173`) is
    `pub(in crate::runtime::definitions)`. A runtime site outside
    `definitions/` that needs the dual route may widen it to
    `pub(in crate::runtime)`; disclose that.
  - A stdlib site that would need the dual route is a STOP with its call
    paths, because that would make the route trait public API.
- **Rows, as input; never edit them.** All of these are in `.temp/brief-606/`:
  - `all-frame-migrations.tsv`: F0047-F0049, F0260, F0261, F0265, F0280-F0282,
    F0347, F0348;
  - `macro-dataflow.tsv` and `macro-frame-ownership.tsv`: M0041-M0043,
    M0265, M0266, M0272, M0290-M0292, M0358, M0359;
  - `deferred-obligations.tsv`: D0014;
  - `divergence-step-gates.tsv`: `part.none`, `part.some`,
    `part.empty_some`, `assoc.none`, `assoc.some`, `assoc.empty_some`;
  - `witness-step-gates.tsv`: `held.part.twice`;
  - `microbench-step-labels.tsv`: the 19 S7 labels;
  - `draft-S7.md`: Codex's plan for this step. Its step 7 is the publishing
    removal.

  S7 has no chained inserts, insertion routes, State births or
  transformation dispositions. The draft is raw material, and where it and
  this brief differ, this brief governs. Its Scope, scratch path and
  measurement section are superseded.
- **Slot types.**
  - `Expr` is `Value`.
  - `Vec<Expr>` and `EvaluatedPartSelectors` are registered Legacy
    (`suspension/typed/registry.rs`, `legacy.rs`). Each keeps its single
    global mode; the selector codec stays Legacy until its F step.
  - `HeldUpdateCurrent` (D0014's helper) is not a slot type and must not
    become one.

### The director's re-probe, at `e9707d15`

32 shapes were run through the resumed path against the queued-input path
(`.temp/director/s7-probe/`, `pairs.py` and `run1.txt`). **All 32 are SAME,
and every one paused.**

- They include the six recorded divergences. A selector pauses, and the
  right-hand side is a guest-written
  `System`InternalContinuationNone[]`, `…Some[5]` or `…Some[]`, on a list
  and on an association. S3's dispatch-output fix already made them equal;
  promoting them is this step's gate.
- The rest:
  - read-backs of the assigned state;
  - list and association Part assignment, with one and two pausing
    selectors;
  - `a[k] = v` for an existing and a new key, and a nested key path;
  - `+=`, `*=`, `x[[i]]++`, `++x[[i]]`, `AppendTo` and `PrependTo` on a
    pausing Part;
  - an update whose right-hand side also pauses;
  - `AssociateTo` and `KeyDropFrom` with a pausing argument, a held Part
    target, and guest-written internal heads;
  - `Do` loops of each;
  - internal heads assigned through a pausing right-hand side.

Like S6, the typing must keep all 32 equal.

**One defect the probe exposed is not this step's, and you must not repair
it (Q5).** Part assignment into an association by *position* is a silent
no-op on the ordinary path. `a = <|"k" -> 0, "j" -> 1|>; a[[2]] = 9` returns
`9` and leaves `a` unchanged, with no message. So do
`e[["k", 2]] = 8` (key, then list position) and `h[["k", 1]] = 2`. The
cause is that `set_association_part_path` (`kernel_data/association_data.rs`)
handles only pure key paths and returns `Unchanged` otherwise.
`g[[1, "k"]] = 4` fails loudly with `Set::argform`. This is queued separately.

- Do not add a fixture whose **expected value** encodes the silent no-op.
- Use key selectors for association expected values.
- A positional shape may appear only as a resumed-against-queued equality
  pair, labelled as Q5.

### The owned rows, at `e9707d15`

Frames:

| Row | Type | File:line |
| --- | --- | --- |
| F0047 | `FinishHeldAssociationPartSelectorEvaluationSuspensionFrame` | `crates/rum-stdlib/src/associations/mutation/resume_frames.rs:6` |
| F0048 | `EvalAssociateToSuspensionFrame` | same file `:42` |
| F0049 | `EvalKeyDropFromSuspensionFrame` | same file `:66` |
| F0260 | `FinishHeldPartSelectorEvaluationSuspensionFrame` | `crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs:5` |
| F0261 | `FinishHeldUpdateSuspensionFrame` | same file `:36` |
| F0265 | `FinishPositionalPartSelectorEvaluationSuspensionFrame` | `crates/rum-runtime/src/runtime/definitions/part_assignment/resume_frames.rs:5` |
| F0280 | `FinishAssociationSetPartSelectorEvaluationSuspensionFrame` | `crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs:5` |
| F0281 | `ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame` | same file `:36` |
| F0282 | `ResumeSuspendedAssociationSetLvalueSuspensionFrame` | same file `:60` |
| F0347 | `FinishEvaluatedPartSelectorsSuspensionFrame` | `crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs:160` |
| F0348 | `FinishPublishedEvaluatedPartSelectorsSuspensionFrame` | same file `:177` |

Macro sites:

| Site | File:line | Form |
| --- | --- | --- |
| M0041 | `rum-stdlib/src/associations/mutation.rs:235` | `AssociateTo` target, result form |
| M0042 | same `:364` | `KeyDropFrom` target, result form |
| M0043 | same `:545` | held association Part selectors, insert form over the publishing API |
| M0265 | `rum-runtime/src/runtime/definitions/definition_updates.rs:254` | held update selectors, result form (D0014) |
| M0266 | same `:516` | held update right-hand side, insert form |
| M0272 | `rum-runtime/src/runtime/definitions/part_assignment.rs:43` | positional Part assignment, result form |
| M0290 | `rum-runtime/src/runtime/kernel_data/association_data.rs:747` | association Part set |
| M0291 | same `:830` | association lvalue selector |
| M0292 | same `:862` | suspended association lvalue |
| M0358 | `rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs:122` | selector evaluation, result form |
| M0359 | same `:143` | selector publishing, insert form |

Derive each `Input` from the actual producer `Result` before any
transformation, and bind each `Output` at the innermost function or closure
return. Classify every site's call paths under the route rules above, and
report a table of every call path and its route, as S6 did. A site that fits
no rule is a STOP with its call paths.

### Ruling: the publishing removal

- **The protocol goes.**
  `evaluate_part_selectors_owned_for_builtin` stops returning
  `EvaluatedPartSelectorsEvaluation`. The builtin-facing producer returns
  `Result<EvaluatedPartSelectors, EvalError>`, inserting F0347's typed frame
  (`Input = Vec<Expr>`, `Output = EvaluatedPartSelectors`) when the
  selectors pause. Delete these:
  - `EvaluatedPartSelectorsContinuation`;
  - `EvaluatedPartSelectorsEvaluation`;
  - the `OnceLock`;
  - F0348.
- **The stdlib consumer** (M0043, F0047) takes its witness from that actual
  `Result`. F0047 becomes a typed frame with `Input =
  EvaluatedPartSelectors`.
- **`EvaluatedPartSelectors` stays opaque.** Its fields and construction
  stay private, and `from_continuation_expr` stays crate-private.
- **The public items whose signatures change are doc-hidden builtin-facing
  API.** Changing them is authorized; disclose each.
- **F0348's removal is a planned transition.** The production
  implementation count falls by one. Report it in the census.

## Scope

A listed file or class is permission to do this step's owned work there.
It is not permission to:
- convert another step's rows;
- change evaluation beyond the owned rows;
- tidy;
- repair Q5.

Disclose every file you edit.

**Classes.**

- **The family's source files:**
  - `crates/rum-stdlib/src/associations/mutation.rs` and
    `associations/mutation/resume_frames.rs`;
  - under `crates/rum-runtime/src/runtime/`:
    - `definitions/definition_updates.rs` and
      `definitions/definition_updates/resume_frames.rs`;
    - `definitions/part_assignment.rs` and
      `definitions/part_assignment/resume_frames.rs`;
    - `kernel_data/association_data.rs` and
      `kernel_data/association_data/resume_frames.rs`;
    - `kernel_data/evaluated_part_selectors.rs`.
- **The route trait**, `definitions/application/mod.rs`, only for a
  visibility widening to `pub(in crate::runtime)`. Disclose it.
- **Callers of a changed signature.** Edit only the construct that changed,
  in any file where an owned frame, helper or signature is named, including
  `kernel_api.rs` re-exports.
- **Module roots on an owned type's path.** Make only the visibility or
  re-export edit the typed API needs, and disclose it.
- **`crates/rum-runtime/src/runtime/suspension/`**, only where S7's use
  exposes a gap in the API or driver. Disclose each edit and why.
  `typed/witness.rs` is excluded. Sealing, opacity and the no-dual-arm rule
  stay intact.
- **Sibling unit tests.** A test needing a family module's private items
  goes in that module's sibling `tests.rs`, declared as `#[cfg(test)] mod
  tests;`. Never use `#[path = "../…"]`.
- **Census, oracle and controls.**
  - **Census:** everything under `crates/rum-stdlib/tests/typed_slot_census/`
    and `typed_slot_census.rs`.
  - **Oracle:** `crates/rum-stdlib/tests/suspension_oracle/fixtures/` for the
    new fixtures, the promoted divergences and the `typed_slots.rs` receipt,
    and `suspension_oracle/inventory/typed-slot-stages.tsv` and
    `typed-slot-shapes.tsv`.
  - **Compile controls:** `crates/rum-stdlib/tests/typed_slot_compile_controls.rs`.
    Add a positive and a negative pair proving that the stdlib consumer's
    `Input` cannot float. The positive pair takes the witness from the
    actual `Result<EvaluatedPartSelectors, _>`; the single invalid change
    takes it from a `Result<Expr, _>`, and that must not compile.
- **Derived facts follow the diff** (brief-613's class rule). For every
  derived inventory, frontier, census receipt, fingerprint, certificate or
  test pin that names a touched construct, each row falls into exactly one
  case:

  | Its operation in this diff | Ruling |
  | --- | --- |
  | Deleted by an owned conversion (including the publishing protocol and F0348) | Retired with its source. Disclose it. |
  | Added by this diff | A new row. It stays rejected or unknown; claim no mark or coverage. Disclose it. |
  | Moved, or its text follows an owned rename | Paired. Disclose it old → new. |

  The class covers:
  - `runtime/tests/frame_root_trace{.rs,/derived_inventory.rs}`;
  - ownership inventories, by discovery replay. The OnceLock removal may
    retire an `UnknownHostPayload` or `UnknownConstruct` row with its
    source: that is the first case;
  - retention inventories, by replay;
  - `retention_closure/census/tests.rs`, with a per-brief comment;
  - census receipts;
  - count and layout pins.

  A **STOP** is a change of *meaning* where the source did not change:
  - a row whose operation's source text is unchanged disappears, or changes
    kind, reason, status, disposition or exclusion;
  - a mark, coverage or promotion is gained or lost;
  - a new `UnknownHostPayload`;
  - a production receipt change outside S7's owned rows.

  Never hand-write a fingerprint, certificate cell or ledger chain.
- **Canonical tool outputs.** Any unchanged canonical gate tool this brief
  requires may write to its own fixed output paths under `target/` or
  `.temp/`. You may not modify, redirect or wrap it to move its output.
  Disclose each such path. Your own scratch stays in `.temp/brief-615/`.

`Cargo.toml` files, `Cargo.lock`, the toolchain, docs, `AGENTS.md`,
`TODO.md` and the core `Expr`/`Symbol` representation stay unchanged.

**Disclosure duty.** If a needed change falls outside every list and class,
make no edit there. STOP and report the file, the line and why.

## Steps

1. **Harness, probe and bases first, before any source edit.**
   - **Supplied.** `.temp/brief-615/` holds:
     - `base/rum-cli` and `base/rum-cli-alloc`: the S6 one-codegen-unit
       binaries, with their build commands. The director proved them fresh.
     - `stack-base/runtime-tests-{debug,release}` with `probe.rs`: the
       six-shape S6 stack probe binaries, built by the director from S6's
       code plus brief-614's probe.
     - `base-source/`: the `e9707d15` export.
     - `base-hashes.txt`.

     Verify the hashes first.
   - **Stack calibration.** The supplied stack bases must reproduce S6's
     bytes per level exactly. If they do not, STOP.

     | Profile | peel | plus | sub | up | guarded | rhs-guard |
     | --- | --- | --- | --- | --- | --- | --- |
     | debug | 19 024 | 34 912 | 29 376 | 28 064 | 35 840 | 49 152 |
     | release | 7 344 | 13 792 | 13 168 | 11 456 | 14 240 | 17 920 |
   - **Resume harness.** Copy `.temp/brief-614/resume-harness/` to
     `.temp/brief-615/resume-harness/`. Add workloads that pause in S7's
     frames:
     - a positional list assignment (F0265);
     - an association key assignment through Part and through `a[k]`
       (F0280-F0282);
     - `held.part.twice`'s two pausing selectors under `+=` (F0260, F0347);
     - a held update whose right-hand side pauses (F0261);
     - `AssociateTo` and `KeyDropFrom` with a pausing argument (F0048,
       F0049);
     - `AssociateTo` on a held Part target with a pausing selector (F0047).

     Each repeats its pause N times and checks its final value. Then build
     `resume` and `resume-alloc` from the unedited tree, with
     `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, each in its own target
     directory under `.temp/brief-615/`. Record their source hashes; they
     must match `base-source/`. Each workload's callgrind frame-call census
     must show the intended S7 frame resuming once per resume.
2. **Labels.** Copy `.temp/brief-614/perf/labels.json` and its programs into
   `.temp/brief-615/perf/`.
   - Add the eight S7 labels that are missing, in the same N/2N format:
     - `association-part-key-read-width4000`,
       `association-part-key-read-width32000`;
     - `association-part-positional-read-width4000`,
       `association-part-positional-read-width32000`;
     - `appendto-list-incremental-small`,
       `appendto-list-incremental-large`;
     - `association-set-key-incremental-small`,
       `association-set-key-incremental-large`.
   - Each program checks its own result.
   - Add the new resume workloads as resume labels, each with its
     `resumes_per_iteration`.
3. **Fixtures.** Before converting anything, add S7's oracle fixtures and
   run them on the unedited tree. Each compares the queued-input path with
   the resumed path under `Checks::ALL`, with unconditional `FaultProbe`
   absence.
   - Promote the six divergences to strict equality.
   - Cover the director's 32 shapes (Q5's two positional shapes only as
     equality pairs).
   - Each passes on the unedited tree. Record that.
4. **Type the owned rows and remove the publishing protocol**, following the
   rulings above.
   - Keep exactly:
     - the exactly-once, left-to-right selector evaluation;
     - the held targets' frozen state;
     - the held updates' return values (`x++` returns the old value);
     - the association lvalue normalization;
     - held and evaluation order;
     - owner drop order;
     - profiling balance.
   - Each M site's ordinary path matches the producer `Result` once and
     constructs no frame, `Arc` or `Vec`. Frame state is built in the cold
     path. Move locals, not control flow.
   - The frames' identity `into_continuation_expr` / `from_continuation_expr`
     calls disappear. A semantic map runs on both paths.
5. **Ordinary controls.** Rerun the 28 round-4 ordinary controls through
   brief-611's harness:
   - copy `.temp/brief-614/ordinary-controls/` to `.temp/brief-615/`;
   - re-sync its copied `suspension_oracle/` support from the committed tree
     first;
   - all 28 must pass, including `held.part.twice`.
6. **Census.** Advance the receipt to S7 in `typed_slot_census` and in
   `suspension_oracle/fixtures/typed_slots.rs`, and record every owned row's
   transition.
   - Production at S6 is 607 implementations and 573 macro sites, with 42
     typed.
   - With F0348 removed and the other ten typed, expect 606 implementations
     with 52 typed. Report the actual totals.
7. **Inventories.** Replay fingerprints, both certificate sets and ownership
   discovery, and disclose every changed row under the derived-facts class.

## Verification (MANDATORY — run to green or evidence)

**How to run.**
- Run everything serially, each command through `python3
  scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
  RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
  TMPDIR=<repo>/.temp/brief-615/tmp …`.
- Bounds: workspace 3 600 s; oracle 1 500 s per build; everything else
  900 s.
- **Your shell tool's own timeout must exceed the command's bound.**

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

- Also run the ordinary-controls harness in both builds.
- Rerun the director's 32-shape probe (`.temp/director/s7-probe/pairs.py`,
  through a copy of `.temp/director/depth-leak-probe/` built under
  `.temp/brief-615/`). Also rerun S6's 22 shapes
  (`.temp/director/s6-probe/pairs.py`) as a regression check.
- **Native ZeroMQ.** In your sandbox the nine native ZeroMQ tests cannot
  bind loopback. Run the workspace with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`,
  report the nine as skips, and report every other count. The director runs
  them natively.

Report:
- every `test result:` total;
- the oracle's fault-occurrence delta for each build;
- any compiler warning the diff introduces. Neither oracle build may warn.

**Performance.** `TEMPLATE.md` pitfall 8 governs, with the bars of
brief-611, 613 and 614. Build the candidates with the base commands and the
same cwd, into separate target directories. Preflight every N/2N run: the
result, and the exact resume count (`resumes_per_iteration` × N).

1. **Allocations and bytes first**, on every label.
   - Ordinary labels must be exactly equal, in slope and in the
     whole-program counts at N and 2N.
   - On the resume workloads, a typed entry adds one 24-B adapter `Arc` per
     newly typed transfer. **The publishing removal should remove the
     `Arc<OnceLock>` allocation on the held-association path.** Report that
     reduction.
   - Any other difference is attributed at its allocation sites, or it is a
     defect.
2. **Instruction slopes.** Run two interleaved `perf stat` pairs per arm on
   all labels: `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`,
   stdin `/dev/null`.
   - A flagged ordinary label is cleared by five runs per arm, or by six or
     more callgrind processes per arm with every deterministic delta named
     at its addresses.
   - A deterministic register-allocation residual is reported with its
     addresses for the director to rule on, as S6's +2 was.
3. **Copies and leaves.** The `memcpy` count on every ordinary label is at
   most the base's. `literal-integer`, `do-count-null` and
   `do-iterator-null` show no deterministic instruction growth.
4. **Stack.** Measure the six supplied shapes, debug and release, at four
   heights with three samples each, against the supplied bases.
   - Tabulate the prologue frame size of every function S7 changes, in both
     profiles.
   - **Bar:** no growth beyond the typed macro's named closure environment,
     attributed per slot.
   - **Any growth on `peel`, `plus` or `rhs-guard` is reported with its slot
     table for the director to rule on.**
5. **Resume attribution.** For every resume workload, run six callgrind
   processes per arm and diff every function per process. Name each
   deterministic delta at its addresses. The driver's 136-B typed-outcome
   move is owed to R: count it, do not repair it.
6. **Timing** comes last, as an indication only. Run four full
   `scripts/core-language-microbench.sh` panels, base, candidate, base,
   candidate, under `taskset -c 2`, with profiling off.
   - A residual whose instructions are equal is a layout question. Settle
     it causally with both arms rebuilt using `RUSTFLAGS="-C
     llvm-args=-align-all-functions=6"`, on a probe cut from the benchmark
     program with its prelude.
   - Check that the probe reproduces the label's per-iteration magnitude
     first. `association-part-key-read-width32000` spends 75% of its cycles
     in `collect_upvalue_candidate_symbols_in_matching_order`, about 5.17M
     instructions per read, and is known to be alignment-sensitive (G14).

A regression is a defect to fix or report. A stack or performance residual
is not a failing gate; it goes to the report with its attribution, and I
rule on it.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- No default-path cost. No owned-tree clones on hot paths. Part assignment
  and `AppendTo` are measured hot paths. The O(1) direct association-key
  update (`try_direct_association_rule_update`) and the exclusive-owner
  fast paths must stay exactly as cheap.
- **The witness comes from the actual `Result`.** Never guess the input from
  a synthetic `Err` type.
- **The token** carries no `Arc`, is not `Clone`, and accepts fresh state of
  the same frame type only.
- Actions, codecs and modes stay sealed and opaque. Nothing public names a
  mode or constructs an erased entry. **This is the first stdlib-side
  conversion: check every new `kernel_api` use against that rule.**
- Profiling records stay balanced on every path.
- Prefer the caller-place macro arm.
- **Module size.** New modules stay under 10k characters. Check
  `.temp/brief-606/file-size-assessments.tsv` first. `association_data.rs`
  is 34k, `definition_updates.rs` 26k and `mutation.rs` 23k. Split only by
  responsibility.

## Checkpointing — protect the scene

- Everything goes under `.temp/brief-615/`.
- After each step, write `PROGRESS.md` and a machine-readable `state.json`.
  Update `state.json` when the work moves, not only at the end.
- Record:
  - the verified state;
  - the compile and test counters;
  - any half-edited files;
  - the exact next action.

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
  adapter. Type no row outside S7's. Do not repair Q5. Never weaken an
  oracle `Checks` flag, a retention reject, a fault-absence gate or a
  documented test to get a pass.
- **Stop-loss is keyed on progress.** Compile errors you are reducing count
  as progress. Stop after two attempts that move none of the named failing
  gates.
- Paste real output. Say plainly what did not run or did not pass. Leave
  the tree uncommitted.

## Report format (append to this brief)

1. Files changed, each with its semantic change. Disclose every ripple,
   public signature change, module root and pin update as old → new, under
   its derived-facts case.
2. Owned-row reconciliation:
   - each F/M row's transition;
   - the call-path and route table;
   - the publishing protocol's removal (what was deleted, what replaced it);
   - the census output.
3. The harness workloads and oracle fixtures: what each exercises, its frame
   census, and its result on the base tree and the candidate in both builds.
   Include the six promoted divergences and the director's 32 shapes.
4. All gates, verbatim. Fault-occurrence deltas. Inventory and certificate
   transitions. Every canonical tool output path written.
5. Allocation, instruction, copy, stack and resume-attribution evidence, and
   timing with any alignment check.
6. Stop-loss counters, deviations and open questions.
