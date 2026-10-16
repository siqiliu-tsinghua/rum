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

## Codex execution report — brief-615

### 1. Files and semantic changes

The implementation types ten S7 frames and removes the eleventh, F0348, with its selector publishing protocol. No git command or network access was used. The fixed `suspension/typed/witness.rs` is byte-for-byte equal to the supplied base. No new unsafe, dependency, lockfile, toolchain, core representation, or non-S7 production transport change is made.

The doc-hidden builtin-facing producer `Session::evaluate_part_selectors_owned_for_builtin` changes from `EvaluatedPartSelectorsEvaluation` to `Result<EvaluatedPartSelectors, EvalError>`. The doc-hidden re-exports `EvaluatedPartSelectorsContinuation` and `EvaluatedPartSelectorsEvaluation` are removed with those types. `EvaluatedPartSelectors` remains opaque: fields and construction are private, and its continuation decoder remains runtime-private. No public raw adapter, codec, mode, or route trait is added.

`DefinitionResumeRoute` widens from `pub(in crate::runtime::definitions)` to `pub(in crate::runtime)`. The `definitions::application` module changes from `mod application` to `pub(in crate::runtime) mod application` so `association_data` can use it. Neither is public outside runtime. All new Rust modules are below 10k characters. Existing large family files retain their established responsibility boundaries; their exact before/after sizes follow below. The complete source-only patch is retained in [source.patch](../../../.temp/brief-615/source.patch).

The owned deletion updates census pins: production implementations/macros 607/573 → 606/566 and combined implementations/macros 623/584 → 622/577. Test-only 16/11 is unchanged. Forced-red output markers follow the same source deletion: combined legacy frames/finalizers/implementations 525/56/623 → 514/56/622, and the deliberately one-frame-short partition 622 → 621. The root-trace population is 564 → 563; compile-control pairs are 34 → 35; the stage receipt and diagnostic stage labels are S6 → S7. These are derived-source transitions, not relaxed rejection conditions.


| File | Characters, before → after | Semantic change / derived-facts case |
| --- | --- | --- |
| crates/rum-runtime/src/runtime/definitions/application/mod.rs | 6682 → 6666 | Widen DefinitionResumeRoute from runtime::definitions to runtime; no trait method or implementation change. |
| crates/rum-runtime/src/runtime/definitions/definition_updates.rs | 26305 → 26430 | Witness actual selector/update results once at M0265/M0266; retain ordinary HeldUpdateCurrent and move frozen frame construction into cold helpers. |
| crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs | 2283 → 3628 | Type F0260/F0261; add cold witness/request insertion helpers; F0260 stops through ResumeStop when it inserts the different F0261 frame. |
| crates/rum-runtime/src/runtime/definitions/mod.rs | 886 → 909 | Widen application module visibility only to runtime so association_data can use the internal route trait. |
| crates/rum-runtime/src/runtime/definitions/part_assignment.rs | 13874 → 14034 | M0272 witnesses the actual local selector result once; direct error returns and named cold Expr-result helper preserve the ordinary Option lvalue probe without an extra ordinary memcpy. |
| crates/rum-runtime/src/runtime/definitions/part_assignment/resume_frames.rs | 947 → 1444 | Type F0265 as EvaluatedPartSelectors -> Expr; remove identity selector decoding. Add named cold suspend_positional_part_selectors receiving only the witness, request and frame inputs. |
| crates/rum-runtime/src/runtime/kernel_api.rs | 5604 → 5525 | Retire doc-hidden re-exports EvaluatedPartSelectorsContinuation and EvaluatedPartSelectorsEvaluation; keep opaque EvaluatedPartSelectors. |
| crates/rum-runtime/src/runtime/kernel_data/association_data.rs | 33759 → 34543 | M0290 uses the actual witness and named cold Expr-result helper within the ordinary Option probe; M0291/M0292 witness actual results; suspended selector driver is generic over the internal ordinary/token route. |
| crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs | 2860 → 4755 | Type F0280/F0281/F0282; cold initial insertion and shared route helper; F0281 inserts a different frame with a witness, F0282 repeats with its own token and fresh state. Add named cold suspend_association_set_part_selectors receiving witness/request and binding Expr completion. |
| crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs | 7404 → 4255 | Type F0347 as Vec<Expr> -> EvaluatedPartSelectors; delete F0348, continuation/evaluation types and Arc<OnceLock> publishing protocol. Doc-hidden builtin producer now returns Result<EvaluatedPartSelectors, EvalError>. Fields, constructor and decoder remain private; Legacy mode unchanged. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace.rs | 35668 → 35723 | Owned deletion pin: derive-managed frame population 564 -> 563; add brief-615 explanation. |
| crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs | 175536 → 175161 | Retire deleted publishing continuation type and the two F0348 inventory entries with their source. |
| crates/rum-stdlib/src/associations/mod.rs | 2099 → 2025 | Remove imports of the two retired publishing protocol types. |
| crates/rum-stdlib/src/associations/mutation.rs | 23055 → 22697 | M0041/M0042 use actual-result caller-place typed macros; M0043 witnesses actual Result<EvaluatedPartSelectors, EvalError> and replaces publishing handshake with typed completion. |
| crates/rum-stdlib/src/associations/mutation/resume_frames.rs | 3213 → 3309 | Type F0047/F0048/F0049; F0047 receives opaque selectors directly and retires its continuation field. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv | 23520 → 23520 | Pair the unchanged AssociationSelectorNode cycle location 620 -> 622; all 59 cycle meanings unchanged. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv | 256929 → 256578 | Replay owned-source discovery: 20 paired moved source, 2 retired owned source; final unknown count 1513; no changed status or exclusion. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv | 421909 → 421601 | Replay owned-source discovery: 58 paired moved source, 2 retired owned source; final unknown count 2956; no changed status or exclusion. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv | 398743 → 397571 | Replay owned-source discovery: 29 paired moved source, 8 retired owned source, 4 paired owned rename; final unknown count 3130; no changed status or exclusion. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv | 91646 → 91646 | Replay 68 fingerprints emitted by the canonical regeneration test; all other 18 columns unchanged. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv | 18162 → 18162 | Replay four emitted C01/C02 hash-feature certificate cells; all seven other columns unchanged; no changed mark, coverage or promotion. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs | 3503 → 3539 | Declare the new S7 fixture module. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/part_selectors_held_assignment/director.rs | 0 → 7914 | 32 strict queued/resumed director shapes; positional association cases have Q5 labels and no expected silent-no-op value. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/part_selectors_held_assignment/held.rs | 0 → 4091 | 12 harness-shape fixtures and three ordered/frozen/error held-update controls. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/part_selectors_held_assignment/markers.rs | 0 → 2064 | Six strict historical marker promotions with assignment/readback checks; association expected values use explicit key selectors. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/part_selectors_held_assignment/mod.rs | 0 → 1168 | Shared Checks::ALL fixture driver with unconditional fault-absence checks, including error paths. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs | 181 → 181 | Advance the typing receipt S6 -> S7. |
| crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv | 681673 → 682675 | 29 scoped transitions: 11 F, 11 M, D0014, five retired owned decoder uses, and implemented S7. Other production receipts unchanged. |
| crates/rum-stdlib/tests/typed_slot_census.rs | 7879 → 7923 | Declare S7 module and stage; update combined/forced-red pins for the owned F0348 deletion; preserve all falsifiers. |
| crates/rum-stdlib/tests/typed_slot_census/receipts.rs | 13662 → 13713 | Advance stage and pins: (514,51,607,573,356,217) → (503,51,606,566,354,212); include S7 witnessed sites and closed transition check. |
| crates/rum-stdlib/tests/typed_slot_census/reporting.rs | 11477 → 11561 | Include association_data in the diagnostic list of witnessed calls; no discovery or rejection exemption. |
| crates/rum-stdlib/tests/typed_slot_census/s4.rs | 5999 → 6013 | Exclude the later S7 cohort from the historical S4 closed frame/macro set; S7 has its own exact check. |
| crates/rum-stdlib/tests/typed_slot_census/s7.rs | 0 → 4852 | New closed frame/input/output, insertion, deletion, producer opacity and unchanged-mode reconciliation. |
| crates/rum-stdlib/tests/typed_slot_compile_controls.rs | 23890 → 24541 | Add actual builtin selector producer positive/negative witness pair; pair count 34 -> 35. |

This brief is the only documentation edit: this report is appended. All measurement helpers and temporary probes stay under `.temp/brief-615/`.
Ownership discovery replay, exact old → new rows:

| Inventory / case | Old row | New row |
| --- | --- | --- |
| self_reaching.tsv / paired moved source | rum_runtime::runtime::kernel_data::association_data::AssociationSelectorNode · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 620 · rum_runtime::runtime::kernel_data::association_data::AssociationSelectorNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::kernel_data::association_data::AssociationSelectorNode | rum_runtime::runtime::kernel_data::association_data::AssociationSelectorNode · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 622 · rum_runtime::runtime::kernel_data::association_data::AssociationSelectorNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::kernel_data::association_data::AssociationSelectorNode |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs · 4 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs · 5 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs · 35 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs · 37 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment/resume_frames.rs · 4 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment/resume_frames.rs · 5 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_api.rs · 135 · 1 · unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots")) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_api.rs · 133 · 1 · unresolved attribute/expansion cfg (any (test , feature = "test-continuation-slots")) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 14 · 1 · unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , FrameRootTrace) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 16 · 1 · unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , FrameRootTrace) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 600 · 1 · unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , FrameRootTrace) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 602 · 1 · unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , FrameRootTrace) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 612 · 1 · unresolved attribute/expansion derive (Clone , FrameRootTrace) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 614 · 1 · unresolved attribute/expansion derive (Clone , FrameRootTrace) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 619 · 1 · unresolved attribute/expansion derive (Clone , FrameRootTrace , AccumulatorChainNode) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 621 · 1 · unresolved attribute/expansion derive (Clone , FrameRootTrace , AccumulatorChainNode) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 621 · 5 · unresolved attribute/expansion accumulator_link | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 623 · 5 · unresolved attribute/expansion accumulator_link |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 626 · 1 · unresolved attribute/expansion derive (Clone , FrameRootTrace) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 628 · 1 · unresolved attribute/expansion derive (Clone , FrameRootTrace) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 688 · 1 · unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 690 · 1 · unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 4 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 5 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 35 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 37 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 59 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 62 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 9 · 1 · unresolved attribute/expansion derive (Clone , FrameRootTrace) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 10 · 1 · unresolved attribute/expansion derive (Clone , FrameRootTrace) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 14 · 1 · unresolved attribute/expansion derive (Clone , FrameRootTrace) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 15 · 1 · unresolved attribute/expansion derive (Clone , FrameRootTrace) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 159 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 105 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 5 · 1 · unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 6 · 1 · unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 41 · 1 · unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 42 · 1 · unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon) |
| unknown_attributes.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 65 · 1 · unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 69 · 1 · unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon) |
| unknown_attributes.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 26 · 1 · unresolved attribute/expansion derive (Clone , FrameRootTrace) | retired |
| unknown_attributes.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 176 · 1 · unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) | retired |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 537 · 27 · closure captures: \|definition\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 542 · 27 · closure captures: \|definition\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 538 · 33 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 543 · 33 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 594 · 27 · closure captures: \|definition\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 599 · 27 · closure captures: \|definition\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 595 · 33 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 600 · 33 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 678 · 14 · closure captures: \|(path , value)\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 683 · 14 · closure captures: \|(path , value)\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 689 · 15 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 694 · 15 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 691 · 17 · closure captures: \|values\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 696 · 17 · closure captures: \|values\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 746 · 22 · closure captures: \|(selection_arg , updated_arg)\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 751 · 22 · closure captures: \|(selection_arg , updated_arg)\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 750 · 22 · closure captures: \|values\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 755 · 22 · closure captures: \|values\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs · 22 · 24 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs · 24 · 24 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs · 54 · 24 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates/resume_frames.rs · 59 · 24 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 77 · 23 · closure captures: \|definition\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 78 · 23 · closure captures: \|definition\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 89 · 19 · closure captures: \|selector\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 90 · 19 · closure captures: \|selector\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 140 · 23 · closure captures: \|definition\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 141 · 23 · closure captures: \|definition\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 226 · 23 · closure captures: \|definition\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 227 · 23 · closure captures: \|definition\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 237 · 23 · closure captures: \|definition\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 238 · 23 · closure captures: \|definition\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 238 · 26 · closure captures: \|current\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 239 · 26 · closure captures: \|current\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 247 · 36 · closure captures: \|key\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 248 · 36 · closure captures: \|key\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 250 · 29 · closure captures: \|entry\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 251 · 29 · closure captures: \|entry\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 383 · 48 · closure captures: \|index\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 384 · 48 · closure captures: \|index\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 398 · 23 · closure captures: \|index\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 399 · 23 · closure captures: \|index\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment/resume_frames.rs · 20 · 24 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment/resume_frames.rs · 22 · 24 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 47 · 43 · closure captures: \|(key , value)\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 49 · 43 · closure captures: \|(key , value)\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 53 · 29 · closure captures: \|position\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 55 · 29 · closure captures: \|position\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 134 · 18 · closure captures: \|(key , value)\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 136 · 18 · closure captures: \|(key , value)\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 146 · 15 · closure captures: \|(entry_key , _)\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 148 · 15 · closure captures: \|(entry_key , _)\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 147 · 14 · closure captures: \|(_ , value)\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 149 · 14 · closure captures: \|(_ , value)\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 157 · 15 · closure captures: \|(_ , (entry_key , _))\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 159 · 15 · closure captures: \|(_ , (entry_key , _))\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 158 · 14 · closure captures: \|(index , (_ , value))\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 160 · 14 · closure captures: \|(index , (_ , value))\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 213 · 23 · closure captures: \|(existing_key , _)\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 215 · 23 · closure captures: \|(existing_key , _)\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 224 · 30 · closure captures: \|(index , (key , _))\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 226 · 30 · closure captures: \|(index , (key , _))\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 348 · 23 · closure captures: \|existing\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 350 · 23 · closure captures: \|existing\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 359 · 30 · closure captures: \|(index , entry)\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 361 · 30 · closure captures: \|(index , entry)\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 384 · 59 · closure captures: \|_\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 386 · 59 · closure captures: \|_\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 393 · 33 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 395 · 33 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 410 · 23 · closure captures: \|(entry_key , _)\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 412 · 23 · closure captures: \|(entry_key , _)\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 411 · 22 · closure captures: \|(_ , value)\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 413 · 22 · closure captures: \|(_ , value)\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 449 · 26 · closure captures: \|item\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 451 · 26 · closure captures: \|item\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 456 · 41 · closure captures: \|item\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 458 · 41 · closure captures: \|item\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 471 · 22 · closure captures: \|item\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 473 · 22 · closure captures: \|item\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 502 · 26 · closure captures: \|item\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 504 · 26 · closure captures: \|item\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 509 · 41 · closure captures: \|item\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 511 · 41 · closure captures: \|item\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 794 · 23 · closure captures: \|definition\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 803 · 23 · closure captures: \|definition\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 887 · 56 · closure captures: \|depth\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 908 · 56 · closure captures: \|depth\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 931 · 56 · closure captures: \|depth\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 952 · 56 · closure captures: \|depth\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 963 · 37 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 984 · 37 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 1030 · 47 · closure captures: \|entry\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 1051 · 47 · closure captures: \|entry\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 1058 · 50 · closure captures: \|entry\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 1079 · 50 · closure captures: \|entry\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 22 · 24 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 24 · 24 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 51 · 24 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 54 · 24 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 74 · 24 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data/resume_frames.rs · 78 · 24 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 68 · 22 · closure captures: \|selector\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 33 · 22 · closure captures: \|selector\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 68 · 76 · closure captures: \|index\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 33 · 76 · closure captures: \|index\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 674 · 14 · closure captures: \|(_ , updated_part)\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 657 · 14 · closure captures: \|(_ , updated_part)\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 675 · 25 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 658 · 25 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 26 · 24 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 27 · 24 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 58 · 26 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 62 · 26 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 82 · 26 · closure captures: \|\| (capture types/modes unresolved) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation/resume_frames.rs · 89 · 26 · closure captures: \|\| (capture types/modes unresolved) |
| unknown_captures.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 168 · 35 · closure captures: \|value\| (capture types/modes unresolved) | retired |
| unknown_captures.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 189 · 35 · closure captures: \|value\| (capture types/modes unresolved) | retired |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 702 · 18 · unexpanded macro vec! (ends at 702:43) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 707 · 18 · unexpanded macro vec! (ends at 707:43) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 719 · 5 · unexpanded macro matches! (ends at 723:6) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 724 · 5 · unexpanded macro matches! (ends at 728:6) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 752 · 19 · unexpanded macro vec! (ends at 752:40) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 757 · 19 · unexpanded macro vec! (ends at 757:40) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 30 · 58 · unexpanded macro format! (ends at 32:14) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 29 · 58 · unexpanded macro format! (ends at 31:14) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 68 · 58 · unexpanded macro format! (ends at 71:14) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 69 · 58 · unexpanded macro format! (ends at 72:14) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 79 · 58 · unexpanded macro format! (ends at 82:14) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 80 · 58 · unexpanded macro format! (ends at 83:14) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 91 · 58 · unexpanded macro format! (ends at 93:14) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 92 · 58 · unexpanded macro format! (ends at 94:14) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 124 · 58 · unexpanded macro format! (ends at 126:14) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 125 · 58 · unexpanded macro format! (ends at 127:14) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 142 · 58 · unexpanded macro format! (ends at 145:14) | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 143 · 58 · unexpanded macro format! (ends at 146:14) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 110 · 21 · unexpanded macro vec! (ends at 110:32) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 112 · 21 · unexpanded macro vec! (ends at 112:32) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 134 · 65 · unexpanded macro vec! (ends at 134:81) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 136 · 65 · unexpanded macro vec! (ends at 136:81) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 165 · 13 · unexpanded macro vec! (ends at 165:53) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 167 · 13 · unexpanded macro vec! (ends at 167:53) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 294 · 21 · unexpanded macro vec! (ends at 294:32) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 296 · 21 · unexpanded macro vec! (ends at 296:32) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 324 · 9 · unexpanded macro vec! (ends at 324:37) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 326 · 9 · unexpanded macro vec! (ends at 326:37) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 370 · 5 · unexpanded macro format! (ends at 370:63) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 372 · 5 · unexpanded macro format! (ends at 372:63) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 374 · 5 · unexpanded macro matches! (ends at 374:90) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 376 · 5 · unexpanded macro matches! (ends at 376:90) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 527 · 21 · unexpanded macro unreachable! (ends at 527:88) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 529 · 21 · unexpanded macro unreachable! (ends at 529:88) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 593 · 40 · unexpanded macro vec! (ends at 593:76) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 595 · 40 · unexpanded macro vec! (ends at 595:76) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 597 · 5 · unexpanded macro matches! (ends at 597:86) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 599 · 5 · unexpanded macro matches! (ends at 599:86) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 706 · 17 · unexpanded macro matches! (ends at 706:116) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 708 · 17 · unexpanded macro matches! (ends at 708:116) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 742 · 41 · unexpanded macro matches! (ends at 745:18) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 744 · 41 · unexpanded macro matches! (ends at 747:18) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 902 · 21 · unexpanded macro format! (ends at 902:75) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 923 · 21 · unexpanded macro format! (ends at 923:75) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 945 · 21 · unexpanded macro format! (ends at 945:75) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 966 · 21 · unexpanded macro format! (ends at 966:75) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 997 · 8 · unexpanded macro matches! (ends at 997:55) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 1018 · 8 · unexpanded macro matches! (ends at 1018:55) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 110 · 22 · unexpanded macro panic! (ends at 110:90) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 75 · 22 · unexpanded macro panic! (ends at 75:90) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 573 · 21 · unexpanded macro vec! (ends at 573:62) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 556 · 21 · unexpanded macro vec! (ends at 556:62) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 592 · 17 · unexpanded macro format! (ends at 592:93) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 575 · 17 · unexpanded macro format! (ends at 575:93) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 625 · 21 · unexpanded macro format! (ends at 625:69) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 608 · 21 · unexpanded macro format! (ends at 608:69) |
| unknown_macros.tsv / paired moved source | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 722 · 5 · unexpanded macro format! (ends at 722:95) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 705 · 5 · unexpanded macro format! (ends at 705:95) |
| unknown_macros.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 254 · 20 · unexpanded macro crate :: resume_or_suspend! (ends at 268:14) | retired |
| unknown_macros.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/definition_updates.rs · 516 · 17 · unexpanded macro crate :: resume_or_suspend! (ends at 516:123) | retired |
| unknown_macros.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/definitions/part_assignment.rs · 43 · 21 · unexpanded macro crate :: resume_or_suspend! (ends at 52:10) | retired |
| unknown_macros.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 747 · 17 · unexpanded macro crate :: resume_or_suspend! (ends at 761:18) | retired |
| unknown_macros.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 830 · 21 · unexpanded macro crate :: resume_or_suspend! (ends at 830:134) | retired |
| unknown_macros.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/association_data.rs · 862 · 21 · unexpanded macro crate :: resume_or_suspend! (ends at 862:121) | retired |
| unknown_macros.tsv / paired owned rename | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 122 · 9 · unexpanded macro crate :: resume_or_suspend! (ends at 127:10) | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 87 · 9 · unexpanded macro crate :: resume_typed_or_suspend! (ends at 92:10) |
| unknown_macros.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 143 · 17 · unexpanded macro crate :: resume_or_suspend! (ends at 148:18) | retired |
| unknown_macros.tsv / retired owned source | UnknownConstruct · crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs · 191 · 13 · unexpanded macro assert! (ends at 196:14) | retired |
| unknown_macros.tsv / paired owned rename | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 235 · 5 · unexpanded macro rum_runtime :: resume_or_suspend! (ends at 243:6) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 235 · 5 · unexpanded macro rum_runtime :: resume_typed_or_suspend! (ends at 243:6) |
| unknown_macros.tsv / paired owned rename | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 364 · 5 · unexpanded macro rum_runtime :: resume_or_suspend! (ends at 372:6) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 364 · 5 · unexpanded macro rum_runtime :: resume_typed_or_suspend! (ends at 372:6) |
| unknown_macros.tsv / paired owned rename | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 545 · 13 · unexpanded macro rum_runtime :: resume_or_suspend! (ends at 554:14) | UnknownConstruct · crates/rum-stdlib/src/associations/mutation.rs · 531 · 5 · unexpanded macro rum_runtime :: resume_typed_or_suspend! (ends at 540:6) |

Canonical local-contract fingerprint replay:

| ID | Before | After |
| --- | --- | --- |
| B01 | fnv1a64:19d7fe16bbc1ce2b | fnv1a64:3d1bec37bb2a7957 |
| B02 | fnv1a64:56cceddbdf6cc0ee | fnv1a64:32aaa4812eb293d6 |
| B03 | fnv1a64:eb1d9f9e2d431c09 | fnv1a64:29665e09fb003061 |
| B04 | fnv1a64:c5ba065b2f9df953 | fnv1a64:7b57a9ebcd96819f |
| B05 | fnv1a64:5be9800fca60548a | fnv1a64:f2b749063d152cba |
| B06 | fnv1a64:0aec4ed78e79ac4f | fnv1a64:6acc8583ed1e0343 |
| B07 | fnv1a64:f068124d11ff38fa | fnv1a64:4a14aca3315494ea |
| B08 | fnv1a64:9f3577863cb47d35 | fnv1a64:e017bd39ba9abee5 |
| B09 | fnv1a64:94f5a9ad6bb0e6d7 | fnv1a64:de373a886d79270b |
| B10 | fnv1a64:00d76ff81146b0ac | fnv1a64:b7d20eaa316900b8 |
| B11 | fnv1a64:8711b7b849921f53 | fnv1a64:3caf5b48e78aa79f |
| B12 | fnv1a64:540229a947b97bd7 | fnv1a64:9d43ba844981bc0b |
| I01 | fnv1a64:f7b9e9142879c4ab | fnv1a64:1afdd73527e26fd7 |
| I02 | fnv1a64:b32afda3c296dc8a | fnv1a64:49f8c69a354bb4ba |
| I03 | fnv1a64:9ecf47c0c1b30317 | fnv1a64:2d7e1cb048ad7d4b |
| I04 | fnv1a64:9b8b756f1beb498e | fnv1a64:5dfa8d63919ad376 |
| P01 | fnv1a64:75a131dc70d5cbef | fnv1a64:7f3e3121f66c0be3 |
| P02 | fnv1a64:afa9c3ab9afaf5bc | fnv1a64:8b0b4d588fbfb488 |
| P03 | fnv1a64:36cb37e16674d7a1 | fnv1a64:31ab253bde4a1239 |
| P04 | fnv1a64:4ba80bf16415dcee | fnv1a64:2785c296b35bafd6 |
| P05 | fnv1a64:c2682ae06cc7ff3b | fnv1a64:ef05cff88033e367 |
| P06 | fnv1a64:a4814e79d3e91557 | fnv1a64:edc2df54d5b1558b |
| P070 | fnv1a64:deceeabbb25e8f57 | fnv1a64:28107b96b426cf8b |
| P080 | fnv1a64:f9f3cb4066efbeec | fnv1a64:8f8453e979a31ef8 |
| P090 | fnv1a64:eaf27152c911bb2e | fnv1a64:a69870a5f3b2bb16 |
| P100 | fnv1a64:ee611c890c6e9203 | fnv1a64:714e4d361548398f |
| P110 | fnv1a64:e1185cc4cc28f0c4 | fnv1a64:bd6b023a352183d0 |
| P120 | fnv1a64:a257f815d1290855 | fnv1a64:2b7f52ba450feb05 |
| P130 | fnv1a64:94356baa40b312fa | fnv1a64:ede2060060086eea |
| P140 | fnv1a64:957f218091ed6b46 | fnv1a64:96a463c49fa252ce |
| P150 | fnv1a64:eb630cc715e8dcce | fnv1a64:3a3ee8a41a4588b6 |
| P071 | fnv1a64:db1b06996a05621a | fnv1a64:0caf461a6719840a |
| P081 | fnv1a64:0ea35ebea5b6ef77 | fnv1a64:2f1bae8078671f2b |
| P091 | fnv1a64:9d69da7fda48132f | fnv1a64:22fa540fa2d83e23 |
| P101 | fnv1a64:d91b4d92018c8dc4 | fnv1a64:b56df3076a8520d0 |
| P111 | fnv1a64:c9ec758c455ba247 | fnv1a64:3197f37f5d41827b |
| P121 | fnv1a64:3db5b238ff7c3f30 | fnv1a64:84e2c3105492cf84 |
| P131 | fnv1a64:636428c165f21ad3 | fnv1a64:1901cc5203eaa31f |
| P141 | fnv1a64:dc581cc37cb6b88f | fnv1a64:0f01548d8a043783 |
| P151 | fnv1a64:9213ae268c8b7515 | fnv1a64:31b1992325d0c3c5 |
| B13 | fnv1a64:98fce260bf16684f | fnv1a64:f8dd190d1dbabf43 |
| R01 | fnv1a64:ea2d385aedc6469e | fnv1a64:a384cfa7c3dbd446 |
| R02 | fnv1a64:096e4300078542e1 | fnv1a64:8d6e484d297b4179 |
| R03 | fnv1a64:cb9e1905e576a462 | fnv1a64:eed06cd6609986b2 |
| R040 | fnv1a64:36531f2ceca3b73e | fnv1a64:ecbe44e010d1f7e6 |
| R050 | fnv1a64:63eccba05d3f501b | fnv1a64:73c3ceb78e9ee647 |
| R060 | fnv1a64:ebbb1524b3f093f6 | fnv1a64:a39c478a875cf3fe |
| R070 | fnv1a64:771126d88758ae77 | fnv1a64:9789769a5a08de2b |
| R080 | fnv1a64:628fde61eb70128e | fnv1a64:24fef656611f9c76 |
| R090 | fnv1a64:ca4349cc65a96643 | fnv1a64:e3e9c793fb00bdcf |
| R100 | fnv1a64:b04343f0d718c689 | fnv1a64:ee8c025ca4d5dae1 |
| R110 | fnv1a64:f8a577a5a7968b34 | fnv1a64:760e053826342ec0 |
| R041 | fnv1a64:7f36458ad63e274f | fnv1a64:df167c3734e27e43 |
| R051 | fnv1a64:79149f68563fc7ae | fnv1a64:34ba9ebb80e0c796 |
| R061 | fnv1a64:9f48febfa7dc93c5 | fnv1a64:82369e6cc28a9e35 |
| R071 | fnv1a64:1992d74b043d3998 | fnv1a64:28f1eba9a752d16c |
| R081 | fnv1a64:aa4b85dda81a6703 | fnv1a64:2d38b68ab0f40e8f |
| R091 | fnv1a64:135ce900653ea206 | fnv1a64:58beee0d2b9cab8e |
| R101 | fnv1a64:093450dfe79d9636 | fnv1a64:e27e994ec877333e |
| R111 | fnv1a64:2c4ec9085a93879d | fnv1a64:f109104e1b87decd |
| R12 | fnv1a64:74490b4b44b8b05d | fnv1a64:476c08f9c953538d |
| R13 | fnv1a64:a8626ee02ee08d59 | fnv1a64:a1a71e24ebe725b1 |
| R14 | fnv1a64:08b92afee91f5203 | fnv1a64:8ba65babf1f8f98f |
| R15 | fnv1a64:c7ea53235c14b6df | fnv1a64:81856b3a2c31a3d3 |
| O010 | fnv1a64:561ad2e87789e3a5 | fnv1a64:ba7b55b59f79ea15 |
| O0200 | fnv1a64:9046c354e0a49146 | fnv1a64:916c0598ee5978ce |
| O011 | fnv1a64:2e43d1acc1825876 | fnv1a64:e625041294eeb87e |
| O0210 | fnv1a64:48688f12456e803d | fnv1a64:0b8062daf2a27aed |

Certificate cells (complete emitted cells: [certificate-diff.json](../../../.temp/brief-615/certificate-diff.json)):

| Body | hash feature | Before digests | After digests |
| --- | --- | --- | --- |
| C01 | true | fnv1a64:100b5826970f8119; fnv1a64:b6536f64ad4523f8; fnv1a64:f09c40857bbfda66 | fnv1a64:df77b4b3bcbdf927; fnv1a64:c0aea7ab152c718d; fnv1a64:6b3d2e72c0bc02c7 |
| C01 | false | fnv1a64:ef3db814857f389b; fnv1a64:9e75cce6dd584e7e; fnv1a64:3f328793dd23fc7c | fnv1a64:e711c794b75571e4; fnv1a64:ab39c6d427bf53d6; fnv1a64:2eaf668580e45ff8 |
| C02 | true | fnv1a64:3784b91870ee3517; fnv1a64:2001491e51a21e81 | fnv1a64:2fcdd6cbea3b226d; fnv1a64:73ad9c9382ccba67 |
| C02 | false | fnv1a64:be07676aab7bd919; fnv1a64:a473e36eab5f4d37 | fnv1a64:5deeb05e95faf07e; fnv1a64:32f0c951d39eabe2 |

Retention diagnostics are fully paired, including every retired/new rejected boundary, in [retention-replay/audit-summary.json](../../../.temp/brief-615/retention-replay/audit-summary.json), [retention-replay/rows.tsv.audit.json](../../../.temp/brief-615/retention-replay/rows.tsv.audit.json) and [retention-replay/rows.tsv.paired.json](../../../.temp/brief-615/retention-replay/rows.tsv.paired.json). The first-failure replay is [retention-replay/first-failure.tsv.audit.json](../../../.temp/brief-615/retention-replay/first-failure.tsv.audit.json) with [retention-replay/first-failure.tsv.paired.json](../../../.temp/brief-615/retention-replay/first-failure.tsv.paired.json).

### 2. Owned rows and routes

Every producer witness comes from its actual bound Result before the match. Each ordinary site matches that Result once. The cold paths construct frames and frozen state; no producer Result crosses the cold boundary. Identity selector decoding moves into the existing typed Legacy codec. `Expr` remains Value; `Vec<Expr>` and `EvaluatedPartSelectors` remain globally Legacy. `HeldUpdateCurrent` is never registered as a slot type.

For M0272/M0290 the typed consumer completion is Expr. Named cold Result<Expr, EvalError> helpers bind that completion. Each caller witnesses the actual producer Result, matches it once, returns errors directly, and uses a total map(Some) for the outer ordinary Set-lvalue probe. M0265's cold helper returns the existing ordinary helper type through a total `map(HeldUpdateCurrent::Immediate)`; it does not add a new slot or an unreachable branch.

F0260 may insert the different F0261 frame: it uses the witnessed path, then `complete_work` with `ResumeStop::failure`. F0281 similarly uses the unit route to insert the different F0282 frame. A resumed F0282 consumes its own continuation token with fresh F0282 state. The token carries no Arc. None of the stdlib frames needs a public dual-route trait.

The OnceLock protocol is deleted, including the publishing continuation/evaluation types, F0348 and M0359. The builtin producer delegates to the ordinary typed selector producer; F0347 maps Vec<Expr> to the opaque selectors, and F0047 receives those selectors through the typed slot. Source discovery verifies exactly the owned transitions. Physical macros fall by seven, 573 → 566, because six insertion sites become named witnessed cold calls and M0359 is deleted. There are 35 physical typed macro sites and 22 witnessed insertion sites.


| Row | Before → after | Before identity | After identity / contract |
| --- | --- | --- | --- |
| F0047 | untyped → typed | rum_stdlib::associations::mutation::resume_frames::FinishHeldAssociationPartSelectorEvaluationSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_stdlib::associations::mutation::resume_frames::FinishHeldAssociationPartSelectorEvaluationSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame / EvaluatedPartSelectors -> Expr |
| F0048 | untyped → typed | rum_stdlib::associations::mutation::resume_frames::EvalAssociateToSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_stdlib::associations::mutation::resume_frames::EvalAssociateToSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame / Expr -> Expr |
| F0049 | untyped → typed | rum_stdlib::associations::mutation::resume_frames::EvalKeyDropFromSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_stdlib::associations::mutation::resume_frames::EvalKeyDropFromSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame / Expr -> Expr |
| F0260 | untyped → typed | rum_runtime::runtime::definitions::definition_updates::resume_frames::FinishHeldPartSelectorEvaluationSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::definitions::definition_updates::resume_frames::FinishHeldPartSelectorEvaluationSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame / EvaluatedPartSelectors -> Expr |
| F0261 | untyped → typed | rum_runtime::runtime::definitions::definition_updates::resume_frames::FinishHeldUpdateSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::definitions::definition_updates::resume_frames::FinishHeldUpdateSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame / Expr -> Expr |
| F0265 | untyped → typed | rum_runtime::runtime::definitions::part_assignment::resume_frames::FinishPositionalPartSelectorEvaluationSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::definitions::part_assignment::resume_frames::FinishPositionalPartSelectorEvaluationSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame / EvaluatedPartSelectors -> Expr |
| F0280 | untyped → typed | rum_runtime::runtime::kernel_data::association_data::resume_frames::FinishAssociationSetPartSelectorEvaluationSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::kernel_data::association_data::resume_frames::FinishAssociationSetPartSelectorEvaluationSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame / EvaluatedPartSelectors -> Expr |
| F0281 | untyped → typed | rum_runtime::runtime::kernel_data::association_data::resume_frames::ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::kernel_data::association_data::resume_frames::ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame / Expr -> Expr |
| F0282 | untyped → typed | rum_runtime::runtime::kernel_data::association_data::resume_frames::ResumeSuspendedAssociationSetLvalueSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::kernel_data::association_data::resume_frames::ResumeSuspendedAssociationSetLvalueSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame / Expr -> Expr |
| F0347 | untyped → typed | rum_runtime::runtime::kernel_data::evaluated_part_selectors::FinishEvaluatedPartSelectorsSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::kernel_data::evaluated_part_selectors::FinishEvaluatedPartSelectorsSuspensionFrame implements rum_runtime::runtime::suspension::typed::TypedFrame / Vec < Expr > -> EvaluatedPartSelectors |
| F0348 | untyped → removed | rum_runtime::runtime::kernel_data::evaluated_part_selectors::FinishPublishedEvaluatedPartSelectorsSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame | rum_runtime::runtime::kernel_data::evaluated_part_selectors::FinishPublishedEvaluatedPartSelectorsSuspensionFrame implements rum_runtime::runtime::suspension::core::SuspensionFrame / F0347 |
| M0041 | untyped → typed | rum_stdlib::associations::mutation::resume_associate_to_after_target_preparation#macro1:resume_or_suspend:result | rum_stdlib::associations::mutation::resume_associate_to_after_target_preparation#macro1:resume_typed_or_suspend:result / caller-place typed macro; fixed input/output |
| M0042 | untyped → typed | rum_stdlib::associations::mutation::resume_key_drop_from_after_target_preparation#macro1:resume_or_suspend:result | rum_stdlib::associations::mutation::resume_key_drop_from_after_target_preparation#macro1:resume_typed_or_suspend:result / caller-place typed macro; fixed input/output |
| M0043 | untyped → typed | rum_stdlib::associations::mutation::eval_held_association_part_mutation#macro1:resume_or_suspend:insert | rum_stdlib::associations::mutation::eval_held_association_part_mutation#macro1:resume_typed_or_suspend:result / caller-place typed macro; fixed input/output |
| M0265 | untyped → typed | rum_runtime::runtime::definitions::definition_updates::<Session as inherent>::current_held_update_value#macro1:resume_or_suspend:result | rum_runtime::runtime::definitions::definition_updates::<Session as inherent>::current_held_update_value#call5:suspend_held_part_selectors: / actual producer witness and request cross named cold helper |
| M0266 | untyped → typed | rum_runtime::runtime::definitions::definition_updates::<Session as inherent>::finish_held_update#macro1:resume_or_suspend:insert | rum_runtime::runtime::definitions::definition_updates::<Session as inherent>::finish_held_update#call2:suspend_held_update: / actual producer witness and request cross named cold helper |
| M0272 | untyped → typed | rum_runtime::runtime::definitions::part_assignment::<Session as inherent>::try_set_positional_part_lvalue#macro1:resume_or_suspend:result | rum_runtime::runtime::definitions::part_assignment::<Session as inherent>::try_set_positional_part_lvalue#call12:suspend_positional_part_selectors: / actual producer witness and request cross named cold helper; cold Expr result maps to outer Option |
| M0290 | untyped → typed | rum_runtime::runtime::kernel_data::association_data::<Session as inherent>::try_set_association_lvalue#macro1:resume_or_suspend:result | rum_runtime::runtime::kernel_data::association_data::<Session as inherent>::try_set_association_lvalue#call7:suspend_association_set_part_selectors: / actual producer witness and request cross named cold helper; cold Expr result maps to outer Option |
| M0291 | untyped → typed | rum_runtime::runtime::kernel_data::association_data::<Session as inherent>::resume_association_set_lvalue_selector_evaluation#macro1:resume_or_suspend:insert | rum_runtime::runtime::kernel_data::association_data::<Session as inherent>::resume_association_set_lvalue_selector_evaluation#call2:suspend_association_set_lvalue_selectors: / actual producer witness and request cross named cold helper |
| M0292 | untyped → typed | rum_runtime::runtime::kernel_data::association_data::<Session as inherent>::resume_suspended_association_set_lvalue#macro1:resume_or_suspend:insert | rum_runtime::runtime::kernel_data::association_data::<Session as inherent>::resume_suspended_association_set_lvalue#call2:suspend_suspended_association_set_lvalue: / actual producer witness and request cross named cold helper |
| M0358 | untyped → typed | rum_runtime::runtime::kernel_data::evaluated_part_selectors::<Session as inherent>::evaluate_part_selectors_owned#macro1:resume_or_suspend:result | rum_runtime::runtime::kernel_data::evaluated_part_selectors::<Session as inherent>::evaluate_part_selectors_owned#macro1:resume_typed_or_suspend:result / caller-place typed macro; fixed input/output |
| M0359 | untyped → removed | rum_runtime::runtime::kernel_data::evaluated_part_selectors::<Session as inherent>::evaluate_part_selectors_owned_for_builtin#macro1:resume_or_suspend:insert | rum_runtime::runtime::kernel_data::evaluated_part_selectors::<Session as inherent>::evaluate_part_selectors_owned_for_builtin#macro1:resume_or_suspend:insert / M0358 |
| D0014 | untyped → implemented | deferred::held update selectors | deferred::held update selectors / F0047;F0048;F0049;F0260;F0261;F0265;F0280;F0281;F0282;F0347;F0348 |
| Whole0025 | legacy-use → removed | rum_runtime::runtime::definitions::definition_updates::resume_frames::<FinishHeldPartSelectorEvaluationSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call2:from_continuation_expr: | rum_runtime::runtime::definitions::definition_updates::resume_frames::<FinishHeldPartSelectorEvaluationSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call2:from_continuation_expr: / D011 |
| Whole0026 | legacy-use → removed | rum_runtime::runtime::definitions::part_assignment::resume_frames::<FinishPositionalPartSelectorEvaluationSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call2:from_continuation_expr: | rum_runtime::runtime::definitions::part_assignment::resume_frames::<FinishPositionalPartSelectorEvaluationSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call2:from_continuation_expr: / D011 |
| Whole0033 | legacy-use → removed | rum_runtime::runtime::kernel_data::association_data::resume_frames::<FinishAssociationSetPartSelectorEvaluationSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call2:from_continuation_expr: | rum_runtime::runtime::kernel_data::association_data::resume_frames::<FinishAssociationSetPartSelectorEvaluationSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call2:from_continuation_expr: / D011 |
| Whole0050 | legacy-use → removed | rum_runtime::runtime::kernel_data::evaluated_part_selectors::<FinishEvaluatedPartSelectorsSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr: | rum_runtime::runtime::kernel_data::evaluated_part_selectors::<FinishEvaluatedPartSelectorsSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr: / D126 |
| Whole0051 | legacy-use → removed | rum_runtime::runtime::kernel_data::evaluated_part_selectors::<FinishPublishedEvaluatedPartSelectorsSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr: | rum_runtime::runtime::kernel_data::evaluated_part_selectors::<FinishPublishedEvaluatedPartSelectorsSuspensionFrame as crate :: runtime :: suspension :: SuspensionFrame>::resume#call1:from_continuation_expr: / D126 |
| S7 | absent → implemented | absent | S7 typed Part selectors and held assignment; publishing protocol retired / ten typed frames; F0348 and M0359 retired; global modes unchanged |


| Site / frame | Actual producer | Input → Output | Call path | Route |
| --- | --- | --- | --- | --- |
| M0041 / F0048 | evaluate_prepared_definition_lhs | Expr → Expr | AssociateTo after immediate or resumed PreparedDefinitionLhs continuation; F0048 resumes finish_associate_to_value and does not revisit this insertion. | caller-place witnessed macro |
| M0042 / F0049 | evaluate_prepared_definition_lhs | Expr → Expr | KeyDropFrom after immediate or resumed PreparedDefinitionLhs continuation; F0049 resumes finish_key_drop_from_value and does not revisit this insertion. | caller-place witnessed macro |
| M0043 / F0047 | evaluate_part_selectors_owned_for_builtin | EvaluatedPartSelectors → Expr | AssociateTo/KeyDropFrom held Part entry; F0047 runs finish_held_association_part_mutation directly. | caller-place witnessed macro |
| M0265 / F0260 | evaluate_part_selectors_owned | EvaluatedPartSelectors → Expr | current_held_update_value from ordinary held operation; F0260 skips this producer and continues the held update. HeldUpdateCurrent remains an ordinary helper result. | actual witness/request to cold suspend_held_part_selectors; total map to HeldUpdateCurrent::Immediate |
| M0266 / F0261 | evaluate(update) | Expr → Expr | finish_held_update from ordinary held operation. | actual witness/request to cold suspend_held_update |
| M0266 / F0261 | evaluate(update) | Expr → Expr | F0260 resume -> finish_suspended_held_part_selector_evaluation -> finish_held_update_operation -> finish_held_update. F0261 is a different frame. | witnessed cold insertion; F0260 complete_work maps EvalError to ResumeStop::failure |
| M0272 / F0265 | evaluate_part_selectors_owned | EvaluatedPartSelectors → Expr | try_set_positional_part_lvalue ordinary Set probe; F0265 resumes finish_positional_part_assignment directly. Outer Option is only the lvalue probe. | actual producer witness/request to named cold Result<Expr, EvalError> helper; direct error returns and total map(Some) for the outer ordinary lvalue probe |
| M0290 / F0280 | evaluate_part_selectors_owned | EvaluatedPartSelectors → Expr | try_set_association_lvalue Part branch; F0280 resumes finish_association_set_part_lvalue directly. Outer Option is only the lvalue probe. | actual producer witness/request to named cold Result<Expr, EvalError> helper; direct error returns and total map(Some) for the outer ordinary lvalue probe |
| M0291 / F0281 | evaluate(selector.clone()) | Expr → Expr | initial association function-call lvalue selector loop; F0281 resumes the separate suspended driver. | actual witness/request to cold suspend_association_set_lvalue_selectors |
| M0292 / F0282 | evaluate(selector.clone()) | Expr → Expr | F0281 resume -> resume_suspended_association_set_lvalue(..., ()); F0282 is a different frame. | DefinitionResumeRoute with ordinary route (); witnessed cold insertion; F0281 complete_work maps failure to ResumeStop |
| M0292 / F0282 | evaluate(selector.clone()) | Expr → Expr | F0282 resume -> resume_suspended_association_set_lvalue(..., token), including subsequent selector pauses. | DefinitionResumeRoute with ContinueToken<F0282>; token consumes fresh F0282 state through the cold helper |
| M0358 / F0347 | resolve_argument_exprs_owned | Vec<Expr> → EvaluatedPartSelectors | runtime Part/held selectors and builtin-facing delegating producer; F0347 maps completed values directly and never revisits producer. | caller-place witnessed macro |
| M0359 / F0348 | deleted publishing API | deleted → deleted | All builtin consumers now use the actual typed selector result. | retired with publishing protocol |

Reconciled census output, verbatim before the forced-red scratch controls:

```text

```
Final raw inventories live at `.temp/typed-slot-census/{default,no-default}/`; physical macro counts and logical receipt rows are distinct.

### 3. Harnesses and fixtures

All five supplied hashes were verified first. All 1,492 original source/build inputs matched the supplied base export before source edits. The supplied stack binaries reproduced every required S6 slope exactly, at heights 2/4/6/8 with three samples each and zero fit residual.

The original measurement set had 174 labels (125 ordinary and 49 resumed). After the first candidate exposed an ordinary Part copy defect, a separate explicit Key Part-assignment investigation was added to the final panel: 175 labels (126 ordinary and 49 resumed). It checks its result and receives all ordinary allocation, instruction, copy, and repeated address-attribution checks. This additional probe is recorded as later investigation, not as a before-conversion fixture.

The S6 resume harness was copied into this brief's scratch directory and extended with twelve S7 workloads. Base native, allocation and Callgrind runs checked each at N=100 and 200 before production edits. Pausing mutator arguments alone do not exercise F0048/F0049, so separate pausing prepared-target workloads prove those frame calls. The F0261 workload uses ApplyTo with a pausing function body; the initial plain right-hand-side input paused during argument preparation and was not counted as F0261 evidence. KeyDropFrom's held-target workload uses an explicit System`Key leaf selector. Initial probe results and binaries remain under `harness-initial/`.

LLVM merged the baseline F0281 and F0282 resume implementations at address `0x5ae510`; nm evidence is retained. Their measured aggregate is reported separately from the logical source-path split: one initial F0281, then two F0282 resumptions in the three-selector case. Under held.part.twice, F0260/F0347 complete once after the selector sequence completes; the workload has two actual input resumes per iteration. Raw pause counts and frame counts are separate quantities.

The 53 new strict oracle fixtures comprise the director's 32 shapes, six marker promotions, twelve harness shapes and three additional held-update controls (ordered two-selector effects, frozen target state, and an error after input). They all passed on the unedited production tree in both feature builds. Every fixture uses Checks::ALL and an unconditional fault-absence check. The candidate's full oracle passes 214 tests in each build, with zero continuation-slot fault deltas across 91,254 observations per build and no compiler warning. The copied ordinary harness passes all 28 controls in both builds.

The six marker promotions cover `part.none`, `part.some`, `part.empty_some`, `assoc.none`, `assoc.some`, and `assoc.empty_some`, including assigned-state readback. Association expected-value fixtures use key selectors. The director's positional association shapes remain equality-only, labelled Q5; no expected value codifies the silent no-op and no Q5 implementation is repaired.

The copied director probe reports SAME for all 32 S7 pairs and all 22 S6 regression pairs. Every pair actually pauses. Commands, verbatim output and per-pair pause counts are retained under `director-s7-probe/` and `director-s6-probe/` in this brief's scratch directory.


| New workload | Actual pauses / iteration | Measured base frame calls / iteration |
| --- | --- | --- |
| s7-list-part | 1 | F0265: 1, F0347: 1 |
| s7-assoc-part | 1 | F0280: 1, F0347: 1 |
| s7-assoc-call | 1 | F0282: 1 |
| s7-assoc-call-thrice | 3 | F0282: 3 |
| s7-held-twice | 2 | F0260: 1, F0347: 1 |
| s7-held-rhs | 1 | F0261: 1 |
| s7-associate-argument | 1 | no S7 frame |
| s7-keydrop-argument | 1 | no S7 frame |
| s7-associate-target | 1 | F0048: 1 |
| s7-keydrop-target | 1 | F0049: 1 |
| s7-associate-held | 1 | F0047: 1, F0348: 1 |
| s7-keydrop-held | 1 | F0047: 1, F0348: 1 |

F0281/F0282 counts above use the compiler-merged measured symbol; the logical initial/later split is described above. All candidate frame and adapter counts, for all three N/2N pairs, are in [resume-callgrind/frame-calls.json](../../../.temp/brief-615/resume-callgrind/frame-calls.json).
Director S7 probe, verbatim:

```text
SAME | sweepList={0}; | sweepList[[(InputString["sweep"];1)]]=System`InternalContinuationNone[]
  resumed: pauses=1 value=InternalContinuationNone[] messages=[]
  queued:  pauses=0 value=InternalContinuationNone[] messages=[]
SAME | swA=<|"k"->{0}|>; | swA[[(InputString["sweep"];1)]]=System`InternalContinuationNone[]
  resumed: pauses=1 value=InternalContinuationNone[] messages=[]
  queued:  pauses=0 value=InternalContinuationNone[] messages=[]
SAME | sweepList={0}; | sweepList[[(InputString["sweep"];1)]]=System`InternalContinuationSome[5]
  resumed: pauses=1 value=InternalContinuationSome[5] messages=[]
  queued:  pauses=0 value=InternalContinuationSome[5] messages=[]
SAME | swA=<|"k"->{0}|>; | swA[[(InputString["sweep"];1)]]=System`InternalContinuationSome[5]
  resumed: pauses=1 value=InternalContinuationSome[5] messages=[]
  queued:  pauses=0 value=InternalContinuationSome[5] messages=[]
SAME | sweepList={0}; | sweepList[[(InputString["sweep"];1)]]=System`InternalContinuationSome[]
  resumed: pauses=1 value=InternalContinuationSome[] messages=[]
  queued:  pauses=0 value=InternalContinuationSome[] messages=[]
SAME | swA=<|"k"->{0}|>; | swA[[(InputString["sweep"];1)]]=System`InternalContinuationSome[]
  resumed: pauses=1 value=InternalContinuationSome[] messages=[]
  queued:  pauses=0 value=InternalContinuationSome[] messages=[]
SAME | l={0,0}; | {l[[(InputString["i"];2)]]=System`InternalContinuationNone[], l}
  resumed: pauses=1 value=List[InternalContinuationNone[], List[0, InternalContinuatio…(len 69) messages=[]
  queued:  pauses=0 value=List[InternalContinuationNone[], List[0, InternalContinuatio…(len 69) messages=[]
SAME | a=<|"k"->0|>; | {a[[(InputString["i"];1)]]=System`InternalContinuationSome[5], a}
  resumed: pauses=1 value=List[InternalContinuationSome[5], Association[Rule["k", 0]]] messages=[]
  queued:  pauses=0 value=List[InternalContinuationSome[5], Association[Rule["k", 0]]] messages=[]
SAME | l={0,0,0}; | {l[[(InputString["i"];2)]]=7, l}
  resumed: pauses=1 value=List[7, List[0, 7, 0]] messages=[]
  queued:  pauses=0 value=List[7, List[0, 7, 0]] messages=[]
SAME | l={{0,0},{0,0}}; | {l[[(InputString["i"];2),(InputString["j"];1)]]=7, l}
  resumed: pauses=2 value=List[7, List[List[0, 0], List[7, 0]]] messages=[]
  queued:  pauses=0 value=List[7, List[List[0, 0], List[7, 0]]] messages=[]
SAME | a=<|"k"->0,"j"->1|>; | {a[[(InputString["i"];2)]]=9, a}
  resumed: pauses=1 value=List[9, Association[Rule["k", 0], Rule["j", 1]]] messages=[]
  queued:  pauses=0 value=List[9, Association[Rule["k", 0], Rule["j", 1]]] messages=[]
SAME | a=<|"k"->0|>; | {a[(InputString["k"];"k")]=5, a}
  resumed: pauses=1 value=List[5, Association[Rule["k", 5]]] messages=[]
  queued:  pauses=0 value=List[5, Association[Rule["k", 5]]] messages=[]
SAME | a=<|"k"->0|>; | {a[(InputString["k"];"new")]=5, a}
  resumed: pauses=1 value=List[5, Association[Rule["k", 0], Rule["new", 5]]] messages=[]
  queued:  pauses=0 value=List[5, Association[Rule["k", 0], Rule["new", 5]]] messages=[]
SAME | a=<|"k"-><|"j"->0|>|>; | {a[["k",(InputString["k"];"j")]]=5, a}
  resumed: pauses=1 value=List[5, Association[Rule["k", Association[Rule["j", 5]]]]] messages=[]
  queued:  pauses=0 value=List[5, Association[Rule["k", Association[Rule["j", 5]]]]] messages=[]
SAME | x={1,2}; | {x[[(InputString["i"];1)]]+=10, x}
  resumed: pauses=1 value=List[11, List[11, 2]] messages=[]
  queued:  pauses=0 value=List[11, List[11, 2]] messages=[]
SAME | x={1,2}; | {x[[(InputString["i"];2)]]*=3, x}
  resumed: pauses=1 value=List[6, List[1, 6]] messages=[]
  queued:  pauses=0 value=List[6, List[1, 6]] messages=[]
SAME | x={1,2}; | {x[[(InputString["i"];1)]]++, x}
  resumed: pauses=1 value=List[1, List[2, 2]] messages=[]
  queued:  pauses=0 value=List[1, List[2, 2]] messages=[]
SAME | x={1,2}; | {++x[[(InputString["i"];1)]], x}
  resumed: pauses=1 value=List[2, List[2, 2]] messages=[]
  queued:  pauses=0 value=List[2, List[2, 2]] messages=[]
SAME | x={{1},{2}}; | {AppendTo[x[[(InputString["i"];1)]], 5], x}
  resumed: pauses=1 value=List[List[1, 5], List[List[1, 5], List[2]]] messages=[]
  queued:  pauses=0 value=List[List[1, 5], List[List[1, 5], List[2]]] messages=[]
SAME | x={{1},{2}}; | {PrependTo[x[[(InputString["i"];2)]], 0], x}
  resumed: pauses=1 value=List[List[0, 2], List[List[1], List[0, 2]]] messages=[]
  queued:  pauses=0 value=List[List[0, 2], List[List[1], List[0, 2]]] messages=[]
SAME | x={1,2}; | {x[[(InputString["i"];1)]]+=(InputString["v"];5), x}
  resumed: pauses=2 value=List[6, List[6, 2]] messages=[]
  queued:  pauses=0 value=List[6, List[6, 2]] messages=[]
SAME | a=<||>; | {AssociateTo[a, (InputString["k"];"k"->1)], a}
  resumed: pauses=1 value=List[Association[Rule["k", 1]], Association[Rule["k", 1]]] messages=[]
  queued:  pauses=0 value=List[Association[Rule["k", 1]], Association[Rule["k", 1]]] messages=[]
SAME | a=<|"k"->1,"j"->2|>; | {KeyDropFrom[a, (InputString["k"];"k")], a}
  resumed: pauses=1 value=List[Association[Rule["j", 2]], Association[Rule["j", 2]]] messages=[]
  queued:  pauses=0 value=List[Association[Rule["j", 2]], Association[Rule["j", 2]]] messages=[]
SAME | a=<|"k"-><|"j"->1|>|>; | {AssociateTo[a[[(InputString["s"];"k")]], "m"->2], a}
  resumed: pauses=1 value=List[Association[Rule["k", Association[Rule["j", 1], Rule["m…(len 134) messages=[]
  queued:  pauses=0 value=List[Association[Rule["k", Association[Rule["j", 1], Rule["m…(len 134) messages=[]
SAME | a=<|"k"-><|"j"->1,"m"->2|>|>; | {KeyDropFrom[a[[(InputString["s"];"k")]], "j"], a}
  resumed: pauses=1 value=List[KeyDropFrom[Part[a, "k"], "j"], Association[Rule["k", A…(len 101) messages=[HostMessage { symbol: "KeyDropFrom", tag: "invak", text: "The argument Part[a, CompoundExpression[InputString[\"s\"], \"k\"]] is not a valid Association." }, HostMessage { symbol: "KeyDropFrom", tag: "invak", text: "The argument Part[a, \"k\"] is not a valid Association." }]
  queued:  pauses=0 value=List[KeyDropFrom[Part[a, "k"], "j"], Association[Rule["k", A…(len 101) messages=[HostMessage { symbol: "KeyDropFrom", tag: "invak", text: "The argument Part[a, CompoundExpression[InputString[\"s\"], \"k\"]] is not a valid Association." }, HostMessage { symbol: "KeyDropFrom", tag: "invak", text: "The argument Part[a, \"k\"] is not a valid Association." }]
SAME | a=<|"k"->1|>; | {AssociateTo[a, (InputString["k"];System`InternalContinuationNone[])], a}
  resumed: pauses=1 value=List[AssociateTo[a, InternalContinuationNone[]], Association…(len 75) messages=[HostMessage { symbol: "AssociateTo", tag: "invlb", text: "The argument InternalContinuationNone[] is not a list, Rule or Association." }, HostMessage { symbol: "AssociateTo", tag: "invlb", text: "The argument InternalContinuationNone[] is not a list, Rule or Association." }]
  queued:  pauses=0 value=List[AssociateTo[a, InternalContinuationNone[]], Association…(len 75) messages=[HostMessage { symbol: "AssociateTo", tag: "invlb", text: "The argument InternalContinuationNone[] is not a list, Rule or Association." }, HostMessage { symbol: "AssociateTo", tag: "invlb", text: "The argument InternalContinuationNone[] is not a list, Rule or Association." }]
SAME | a=<|"k"->1|>; | {KeyDropFrom[a, (InputString["k"];System`InternalContinuationSome[5])], a}
  resumed: pauses=1 value=List[Association[Rule["k", 1]], Association[Rule["k", 1]]] messages=[]
  queued:  pauses=0 value=List[Association[Rule["k", 1]], Association[Rule["k", 1]]] messages=[]
SAME | l=Table[0,{3}]; | Do[l[[(InputString["i"];i)]]=i, {i,3}]; l
  resumed: pauses=3 value=List[1, 2, 3] messages=[]
  queued:  pauses=0 value=List[1, 2, 3] messages=[]
SAME | x={0}; | Do[x[[(InputString["i"];1)]]+=1, {3}]; x
  resumed: pauses=3 value=List[3] messages=[]
  queued:  pauses=0 value=List[3] messages=[]
SAME | a=<||>; | Do[a[(InputString["k"];i)]=i, {i,3}]; a
  resumed: pauses=3 value=Association[Rule[1, 1], Rule[2, 2], Rule[3, 3]] messages=[]
  queued:  pauses=0 value=Association[Rule[1, 1], Rule[2, 2], Rule[3, 3]] messages=[]
SAME | l={0}; | {l[[1]]=(InputString["v"];System`InternalContinuationNone[]), l}
  resumed: pauses=1 value=List[InternalContinuationNone[], List[InternalContinuationNo…(len 66) messages=[]
  queued:  pauses=0 value=List[InternalContinuationNone[], List[InternalContinuationNo…(len 66) messages=[]
SAME | a=<|"k"->0|>; | {a["k"]=(InputString["v"];System`InternalContinuationSome[5]), a}
  resumed: pauses=1 value=List[InternalContinuationSome[5], Association[Rule["k", Inte…(len 86) messages=[]
  queued:  pauses=0 value=List[InternalContinuationSome[5], Association[Rule["k", Inte…(len 86) messages=[]
```
Director S6 probe, verbatim:

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

### 4. Gates and derived evidence

Both compile-control configurations pass 35 positive/negative pairs. The new builtin selector producer pair succeeds with the actual selector witness and rejects the single change to an Expr-result witness with E0271 at the typed consumer. Both full census configurations pass eight tests with one intentionally ignored direct forced-red entry point. Their logged failed scratch-copy test totals are expected negative-control evidence, and the enclosing tests pass.

Ownership replay records 108 moved rows, four owned macro renames, twelve retired owned operations and zero new unknown rows. All 79 host-payload rows and all 59 cycle meanings remain unchanged. No host-payload ledger cell is edited. All 31 ownership tests pass.

Retention fingerprints are copied from the unchanged canonical regeneration test's emitted file: 68 fingerprint cells change, with all other 18 columns unchanged. Four certificate cells are copied from the two canonical acceptance emissions, with all other seven columns unchanged. Both final acceptance builds pass with the same two production body rows, three production instance marks and four probe marks; no mark, coverage or promotion is gained or lost.

An additional base/candidate rejection replay checks unchanged-operation meaning: 48,544 -> 48,556 raw diagnostics, with 608 moved pairs, 22 retired owned-source diagnostics and 34 new rejected boundaries. Every other diagnostic meaning key pairs exactly; the same three first-ok roots remain. Roots/definitions move from 7,357 to 7,362, and first-failure rows from 7,354 to 7,359, with every changed signature or added/deleted helper disclosed in the first-failure replay. The legacy option-frontier pin stays 950 and needs no source edit. Its pre-existing console summary still prints OPTION_FRONTIER=927; the report preserves that verbatim output rather than changing unrelated reporting code.

The full workspace command exits 0 after 1946.00 seconds. The runtime library total is 1184 passed / 3 ignored; the stdlib library total is 1109 passed / 53 ignored. The nine native ZeroMQ checks explicitly skip because loopback binding is unavailable. They are reported as skips, although their Rust test harness entries read `ok`: the Jupyter library harness reports 151 passed, comprising 142 executed tests and nine sandbox skips. The full per-target totals, including the two intentional nested census failures, are preserved below. The three runtime test-helper dead-code warnings also occurred in S6; the unchanged retention compiler-fixture warnings are included in the raw workspace log. Neither oracle feature build warns.
Every recorded test total, fault delta and compiler warning follows verbatim. The command environment is bounded, offline and serial as required by the brief. Preliminary failures remain visible beside the later green gates.

`acceptance-default`: `cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.39s
```

`acceptance-default-emission`: `cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`; exit 101.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.40s
```

`acceptance-no-default`: `cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.43s
```

`acceptance-no-default-emission`: `cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`; exit 101.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.37s
```

`allocation-comparison`: `python3 .temp/brief-615/compare-performance.py alloc`; exit 0.
No test-result line emitted; raw output is retained in [gates/allocation-comparison.log](../../../.temp/brief-615/gates/allocation-comparison.log).

`analyze-part-residual`: `python3 .temp/brief-615/analyze-cg.py cli set-part-fixed100-len100 set-part-fixed100-len800`; exit 0.
No test-result line emitted; raw output is retained in [gates/analyze-part-residual.log](../../../.temp/brief-615/gates/analyze-part-residual.log).

`base-retention-census`: `cargo test --manifest-path /home/remi/dev/rum/.temp/brief-615/base-source/Cargo.toml -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 25.45s
```

`base-s7-fixtures-default`: `cargo test -p rum-stdlib --test suspension_oracle part_selectors_held_assignment -- --nocapture`; exit 101.
No test-result line emitted; raw output is retained in [gates/base-s7-fixtures-default.log](../../../.temp/brief-615/gates/base-s7-fixtures-default.log).

`base-s7-fixtures-default-2`: `cargo test -p rum-stdlib --test suspension_oracle part_selectors_held_assignment -- --nocapture`; exit 101.

```text
test result: FAILED. 52 passed; 1 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.37s
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=12
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=24
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=36
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=48
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=60
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=72
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=84
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=96
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=108
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=120
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=132
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=144
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=156
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=168
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=180
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=183
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=195
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=207
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=219
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=231
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=243
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=255
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=267
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=279
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=291
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=303
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=315
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=327
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=339
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=351
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=363
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=375
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=387
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=399
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=411
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=423
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=435
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=447
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=459
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=471
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=483
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=495
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=507
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=519
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=531
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=543
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=555
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=567
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=579
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=591
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=603
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=615
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=627
```

`base-s7-fixtures-default-3`: `cargo test -p rum-stdlib --test suspension_oracle part_selectors_held_assignment -- --nocapture`; exit 0.

```text
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.43s
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=12
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=24
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=36
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=48
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=60
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=72
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=84
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=96
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=108
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=120
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=132
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=144
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=156
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=168
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=180
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=192
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=204
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=216
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=228
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=240
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=252
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=264
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=276
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=288
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=300
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=312
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=324
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=336
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=348
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=360
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=372
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=384
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=396
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=408
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=420
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=432
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=444
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=456
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=468
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=480
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=492
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=504
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=516
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=528
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=540
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=552
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=564
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=576
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=588
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=600
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=612
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=624
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=636
```

`base-s7-fixtures-no-default`: `cargo test -p rum-stdlib --no-default-features --test suspension_oracle part_selectors_held_assignment -- --nocapture`; exit 0.

```text
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.41s
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=12
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=24
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=36
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=48
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=60
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=72
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=84
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=96
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=108
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=120
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=132
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=144
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=156
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=168
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=180
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=192
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=204
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=216
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=228
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=240
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=252
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=264
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=276
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=288
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=300
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=312
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=324
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=336
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=348
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=360
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=372
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=384
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=396
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=408
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=420
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=432
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=444
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=456
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=468
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=480
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=492
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=504
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=516
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=528
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=540
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=552
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=564
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=576
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=588
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=600
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=612
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=624
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=636
```

`bind-part-libc`: `python3 .temp/brief-615/resolve-libc.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/bind-part-libc.log](../../../.temp/brief-615/gates/bind-part-libc.log).

`bind-part-plt`: `python3 .temp/brief-615/resolve-plt-attribution.py cli`; exit 0.
No test-result line emitted; raw output is retained in [gates/bind-part-plt.log](../../../.temp/brief-615/gates/bind-part-plt.log).

`callgrind-cli-1a`: `python3 .temp/brief-615/cg-chunk.py cli 1 0 63`; exit 0.
No test-result line emitted; raw output is retained in [gates/callgrind-cli-1a.log](../../../.temp/brief-615/gates/callgrind-cli-1a.log).

`callgrind-cli-1b`: `python3 .temp/brief-615/cg-chunk.py cli 1 63 125`; exit 0.
No test-result line emitted; raw output is retained in [gates/callgrind-cli-1b.log](../../../.temp/brief-615/gates/callgrind-cli-1b.log).

`callgrind-part-residual`: `python3 .temp/brief-615/cg-part-residual.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/callgrind-part-residual.log](../../../.temp/brief-615/gates/callgrind-part-residual.log).

`census-discovery-initial`: `cargo test -p rum-stdlib --test typed_slot_census source_baseline_matches_earned_receipt -- --exact --nocapture`; exit 101.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 21.49s
```

`census-s7-default-2`: `cargo test -p rum-stdlib --test typed_slot_census -- --nocapture`; exit 0.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.60s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.49s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 243.92s
```

`census-s7-default-initial`: `cargo test -p rum-stdlib --test typed_slot_census source_baseline_matches_earned_receipt -- --exact --nocapture`; exit 101.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 25.91s
```

`census-s7-no-default`: `cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture`; exit 0.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 20.14s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.58s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 246.02s
```

`cold-helper-association-part`: `python3 .temp/brief-615/profile-association-part-experiment.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/cold-helper-association-part.log](../../../.temp/brief-615/gates/cold-helper-association-part.log).

`cold-helper-build`: `python3 .temp/brief-615/cold-helper-experiment.py`; exit 0.

```text
warning: unused import: `resume_frames::*`
warning: `rum-runtime` (lib) generated 1 warning (run `cargo fix --lib -p rum-runtime` to apply 1 suggestion)
```

`cold-helper-comparison`: `python3 .temp/brief-615/analyze-local-move.py cold-helper`; exit 0.
No test-result line emitted; raw output is retained in [gates/cold-helper-comparison.log](../../../.temp/brief-615/gates/cold-helper-comparison.log).

`cold-helper-profiles`: `python3 .temp/brief-615/local-move-experiment.py profile cold-helper-experiment`; exit 0.
No test-result line emitted; raw output is retained in [gates/cold-helper-profiles.log](../../../.temp/brief-615/gates/cold-helper-profiles.log).

`compile-controls-default`: `cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 44.71s
```

`compile-controls-no-default`: `cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.75s
```

`director-build`: `cargo build --manifest-path .temp/brief-615/director-probe-source/Cargo.toml`; exit 0.
No test-result line emitted; raw output is retained in [gates/director-build.log](../../../.temp/brief-615/gates/director-build.log).

`director-s6`: `python3 .temp/brief-615/run-director-probe.py s6`; exit 0.
No test-result line emitted; raw output is retained in [gates/director-s6.log](../../../.temp/brief-615/gates/director-s6.log).

`director-s7`: `python3 .temp/brief-615/run-director-probe.py s7`; exit 0.
No test-result line emitted; raw output is retained in [gates/director-s7.log](../../../.temp/brief-615/gates/director-s7.log).

`early-return-association-part`: `python3 .temp/brief-615/profile-association-part-experiment.py early-return`; exit 0.
No test-result line emitted; raw output is retained in [gates/early-return-association-part.log](../../../.temp/brief-615/gates/early-return-association-part.log).

`early-return-build`: `python3 .temp/brief-615/early-return-experiment.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/early-return-build.log](../../../.temp/brief-615/gates/early-return-build.log).

`early-return-comparison`: `python3 .temp/brief-615/analyze-local-move.py early-return`; exit 0.
No test-result line emitted; raw output is retained in [gates/early-return-comparison.log](../../../.temp/brief-615/gates/early-return-comparison.log).

`early-return-profiles`: `python3 .temp/brief-615/local-move-experiment.py profile early-return-experiment`; exit 0.
No test-result line emitted; raw output is retained in [gates/early-return-profiles.log](../../../.temp/brief-615/gates/early-return-profiles.log).

`fmt-check`: `cargo fmt --check`; exit 0.
No test-result line emitted; raw output is retained in [gates/fmt-check.log](../../../.temp/brief-615/gates/fmt-check.log).

`instruction-comparison`: `python3 .temp/brief-615/compare-performance.py instructions`; exit 0.
No test-result line emitted; raw output is retained in [gates/instruction-comparison.log](../../../.temp/brief-615/gates/instruction-comparison.log).

`local-move-build`: `python3 .temp/brief-615/local-move-experiment.py build`; exit 0.
No test-result line emitted; raw output is retained in [gates/local-move-build.log](../../../.temp/brief-615/gates/local-move-build.log).

`local-move-comparison`: `python3 .temp/brief-615/analyze-local-move.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/local-move-comparison.log](../../../.temp/brief-615/gates/local-move-comparison.log).

`local-move-profiles`: `python3 .temp/brief-615/local-move-experiment.py profile`; exit 0.
No test-result line emitted; raw output is retained in [gates/local-move-profiles.log](../../../.temp/brief-615/gates/local-move-profiles.log).

`oracle`: `scripts/check-suspension-oracle.sh 1500`; exit 0.

```text
test result: ok. 214 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.50s
test result: ok. 214 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.57s
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
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=25241
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=25253
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=25265
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=25277
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=25289
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=25301
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=25313
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=25325
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=25337
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=25349
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=25361
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=25373
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=25385
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=25397
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=25409
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=25421
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=25433
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=25445
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=25457
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=25469
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=25481
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=25493
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=25505
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=25517
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=25529
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=25541
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=25553
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=25565
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=25577
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=25589
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=25601
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=25613
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=25625
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=25637
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=25649
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=25661
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=25673
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=25685
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=25697
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=25709
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=25721
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=25733
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=25745
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=25757
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=25769
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=25781
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=25793
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=25805
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=25817
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=25829
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=25841
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=25853
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=25865
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=91254
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
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=25241
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=25253
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=25265
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=25277
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=25289
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=25301
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=25313
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=25325
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=25337
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=25349
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=25361
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=25373
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=25385
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=25397
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=25409
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=25421
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=25433
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=25445
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=25457
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=25469
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=25481
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=25493
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=25505
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=25517
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=25529
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=25541
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=25553
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=25565
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=25577
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=25589
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=25601
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=25613
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=25625
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=25637
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=25649
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=25661
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=25673
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=25685
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=25697
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=25709
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=25721
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=25733
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=25745
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=25757
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=25769
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=25781
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=25793
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=25805
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=25817
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=25829
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=25841
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=25853
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=25865
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=91254
```

`ordinary-default`: `cargo test --manifest-path .temp/brief-615/ordinary-controls/Cargo.toml --test ordinary -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
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

`ordinary-no-default`: `cargo test --manifest-path .temp/brief-615/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
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

`ownership`: `cargo test -p rum-stdlib --lib ownership_graph -- --nocapture`; exit 0.

```text
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 71.07s
```

`ownership-discovery`: `cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 5.94s
```

`part-address-proof`: `python3 .temp/brief-615/address-deltas.py cli set-part-fixed100-len100 set-part-fixed100-len800`; exit 0.
No test-result line emitted; raw output is retained in [gates/part-address-proof.log](../../../.temp/brief-615/gates/part-address-proof.log).

`performance-allocations`: `python3 .temp/brief-615/perf-chunk.py alloc 0 174`; exit 0.
No test-result line emitted; raw output is retained in [gates/performance-allocations.log](../../../.temp/brief-615/gates/performance-allocations.log).

`performance-instructions-1`: `python3 .temp/brief-615/perf-chunk.py instructions 0 70`; exit 0.
No test-result line emitted; raw output is retained in [gates/performance-instructions-1.log](../../../.temp/brief-615/gates/performance-instructions-1.log).

`performance-instructions-2`: `python3 .temp/brief-615/perf-chunk.py instructions 70 174`; exit 0.
No test-result line emitted; raw output is retained in [gates/performance-instructions-2.log](../../../.temp/brief-615/gates/performance-instructions-2.log).

`performance-preflight`: `python3 .temp/brief-615/perf-chunk.py check 0 174`; exit 0.
No test-result line emitted; raw output is retained in [gates/performance-preflight.log](../../../.temp/brief-615/gates/performance-preflight.log).

`r2-acceptance-default`: `cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.51s
```

`r2-acceptance-default-emission`: `cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`; exit 101.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.19s
```

`r2-acceptance-no-default`: `cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.38s
```

`r2-acceptance-no-default-emission`: `cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`; exit 101.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.28s
```

`r2-aligned-builds`: `python3 -c import subprocess; w=".temp/brief-615/"; subprocess.run(["python3",w+"build-batch.py","aligned"],check=True); subprocess.run(["python3",w+"verify-alignment.py"],check=True)`; exit 1.
No test-result line emitted; raw output is retained in [gates/r2-aligned-builds.log](../../../.temp/brief-615/gates/r2-aligned-builds.log).

`r2-alignment-analysis`: `python3 .temp/brief-615/analyze-alignment.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-alignment-analysis.log](../../../.temp/brief-615/gates/r2-alignment-analysis.log).

`r2-alignment-exact-prefix-00-02`: `python3 .temp/brief-615/alignment-chunk-idle.py .temp/brief-615/alignment/probes-exact-prefix 0 2`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-alignment-exact-prefix-00-02.log](../../../.temp/brief-615/gates/r2-alignment-exact-prefix-00-02.log).

`r2-alignment-full-prefix-00-04`: `python3 .temp/brief-615/alignment-chunk-idle.py .temp/brief-615/alignment/probes-full-prefix 0 4`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-alignment-full-prefix-00-04.log](../../../.temp/brief-615/gates/r2-alignment-full-prefix-00-04.log).

`r2-alignment-full-prefix-preparation`: `python3 .temp/brief-615/prepare-full-prefix-probes.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-alignment-full-prefix-preparation.log](../../../.temp/brief-615/gates/r2-alignment-full-prefix-preparation.log).

`r2-alignment-probe-preparation`: `python3 .temp/brief-615/prepare-alignment-probes.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-alignment-probe-preparation.log](../../../.temp/brief-615/gates/r2-alignment-probe-preparation.log).

`r2-alignment-probes-00-05`: `python3 .temp/brief-615/alignment-chunk-idle.py .temp/brief-615/alignment/probes 0 5`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-alignment-probes-00-05.log](../../../.temp/brief-615/gates/r2-alignment-probes-00-05.log).

`r2-alignment-probes-05-10`: `python3 .temp/brief-615/alignment-chunk-idle.py .temp/brief-615/alignment/probes 5 10`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-alignment-probes-05-10.log](../../../.temp/brief-615/gates/r2-alignment-probes-05-10.log).

`r2-alignment-probes-10-15`: `python3 .temp/brief-615/alignment-chunk-idle.py .temp/brief-615/alignment/probes 10 15`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-alignment-probes-10-15.log](../../../.temp/brief-615/gates/r2-alignment-probes-10-15.log).

`r2-alignment-probes-15-20`: `python3 .temp/brief-615/alignment-chunk-idle.py .temp/brief-615/alignment/probes 15 20`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-alignment-probes-15-20.log](../../../.temp/brief-615/gates/r2-alignment-probes-15-20.log).

`r2-alignment-probes-20-22`: `python3 .temp/brief-615/alignment-chunk-idle.py .temp/brief-615/alignment/probes 20 22`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-alignment-probes-20-22.log](../../../.temp/brief-615/gates/r2-alignment-probes-20-22.log).

`r2-alignment-single-call-00-02`: `python3 .temp/brief-615/alignment-chunk-idle.py .temp/brief-615/alignment/probes-single-call 0 2`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-alignment-single-call-00-02.log](../../../.temp/brief-615/gates/r2-alignment-single-call-00-02.log).

`r2-allocation-comparison`: `python3 .temp/brief-615/compare-performance.py alloc`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-allocation-comparison.log](../../../.temp/brief-615/gates/r2-allocation-comparison.log).

`r2-allocation-slot-reconciliation`: `python3 -c import subprocess; w=".temp/brief-615/"; run=lambda name,*args:subprocess.run(["python3",w+name,*args],check=True); run("reconcile-publishing-allocations.py"); run("check-repeated-copies.py"); [run("dwarf-owned.py",arm,w+"stack/"+arm+"/debug/runtime-tests",w+"stack/dwarf-runtime/"+arm) for arm in ["base","candidate"]]; run("runtime-slot-ledger.py")`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-allocation-slot-reconciliation.log](../../../.temp/brief-615/gates/r2-allocation-slot-reconciliation.log).

`r2-analyze-cli`: `python3 .temp/brief-615/analyze-cg.py cli`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-analyze-cli.log](../../../.temp/brief-615/gates/r2-analyze-cli.log).

`r2-appendto-repeats`: `python3 -c import subprocess; labels=["appendto-part-outer1000-small","appendto-part-outer1000-large"]; [subprocess.run(["python3",".temp/brief-615/run-cg.py","cli",str(s),*labels],check=True) for s in [2,3]]; subprocess.run(["python3",".temp/brief-615/analyze-cg.py","cli",*labels],check=True)`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-appendto-repeats.log](../../../.temp/brief-615/gates/r2-appendto-repeats.log).

`r2-association-experiment-address-proof`: `python3 .temp/brief-615/association-residual-addresses.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-association-experiment-address-proof.log](../../../.temp/brief-615/gates/r2-association-experiment-address-proof.log).

`r2-callgrind-cli-1a`: `python3 .temp/brief-615/cg-chunk.py cli 1 0 63`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-callgrind-cli-1a.log](../../../.temp/brief-615/gates/r2-callgrind-cli-1a.log).

`r2-callgrind-cli-1b`: `python3 .temp/brief-615/cg-chunk.py cli 1 63 126`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-callgrind-cli-1b.log](../../../.temp/brief-615/gates/r2-callgrind-cli-1b.log).

`r2-callgrind-cli-2`: `python3 .temp/brief-615/cg-selected.py 2`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-callgrind-cli-2.log](../../../.temp/brief-615/gates/r2-callgrind-cli-2.log).

`r2-callgrind-cli-3`: `python3 .temp/brief-615/cg-selected.py 3`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-callgrind-cli-3.log](../../../.temp/brief-615/gates/r2-callgrind-cli-3.log).

`r2-callgrind-resume-1`: `python3 .temp/brief-615/cg-chunk.py resume 1 0 49`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-callgrind-resume-1.log](../../../.temp/brief-615/gates/r2-callgrind-resume-1.log).

`r2-callgrind-resume-2`: `python3 .temp/brief-615/cg-chunk.py resume 2 0 49`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-callgrind-resume-2.log](../../../.temp/brief-615/gates/r2-callgrind-resume-2.log).

`r2-callgrind-resume-3`: `python3 .temp/brief-615/cg-chunk.py resume 3 0 49`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-callgrind-resume-3.log](../../../.temp/brief-615/gates/r2-callgrind-resume-3.log).

`r2-census-default`: `cargo test -p rum-stdlib --test typed_slot_census -- --nocapture`; exit 0.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.44s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.52s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 244.00s
```

`r2-census-discovery`: `cargo test -p rum-stdlib --test typed_slot_census source_baseline_matches_earned_receipt -- --exact --nocapture`; exit 101.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 21.40s
```

`r2-census-no-default`: `cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture`; exit 0.

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.60s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.61s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 245.46s
```

`r2-certificate-replay`: `python3 .temp/brief-615/r2-replay-certificates.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-certificate-replay.log](../../../.temp/brief-615/gates/r2-certificate-replay.log).

`r2-cli-stack`: `python3 .temp/brief-615/stack-batch.py cli`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-cli-stack.log](../../../.temp/brief-615/gates/r2-cli-stack.log).

`r2-compile-controls-default`: `cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 42.31s
```

`r2-compile-controls-no-default`: `cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.68s
```

`r2-contract-replay`: `python3 .temp/brief-615/r2-replay-contracts.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-contract-replay.log](../../../.temp/brief-615/gates/r2-contract-replay.log).

`r2-director-build`: `cargo build --manifest-path .temp/brief-615/director-probe-source/Cargo.toml`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-director-build.log](../../../.temp/brief-615/gates/r2-director-build.log).

`r2-director-s6`: `python3 .temp/brief-615/run-director-probe.py s6`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-director-s6.log](../../../.temp/brief-615/gates/r2-director-s6.log).

`r2-director-s7`: `python3 .temp/brief-615/run-director-probe.py s7`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-director-s7.log](../../../.temp/brief-615/gates/r2-director-s7.log).

`r2-final-alignment-analysis`: `python3 .temp/brief-615/analyze-alignment.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-final-alignment-analysis.log](../../../.temp/brief-615/gates/r2-final-alignment-analysis.log).

`r2-final-binary-evidence`: `python3 -c import subprocess; w=".temp/brief-615/"; [subprocess.run(["python3",w+s],check=True) for s in ["verify-alignment.py","slot-ledger.py","runtime-slot-ledger.py","final-evidence.py"]]`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-final-binary-evidence.log](../../../.temp/brief-615/gates/r2-final-binary-evidence.log).

`r2-final-fmt`: `cargo fmt --check`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-final-fmt.log](../../../.temp/brief-615/gates/r2-final-fmt.log).

`r2-fmt-check`: `cargo fmt --check`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-fmt-check.log](../../../.temp/brief-615/gates/r2-fmt-check.log).

`r2-instruction-comparison`: `python3 .temp/brief-615/compare-performance.py instructions`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-instruction-comparison.log](../../../.temp/brief-615/gates/r2-instruction-comparison.log).

`r2-oracle`: `scripts/check-suspension-oracle.sh 1500`; exit 0.

```text
test result: ok. 214 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.47s
test result: ok. 214 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.34s
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
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=25241
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=25253
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=25265
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=25277
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=25289
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=25301
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=25313
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=25325
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=25337
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=25349
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=25361
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=25373
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=25385
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=25397
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=25409
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=25421
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=25433
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=25445
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=25457
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=25469
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=25481
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=25493
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=25505
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=25517
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=25529
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=25541
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=25553
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=25565
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=25577
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=25589
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=25601
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=25613
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=25625
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=25637
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=25649
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=25661
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=25673
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=25685
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=25697
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=25709
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=25721
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=25733
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=25745
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=25757
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=25769
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=25781
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=25793
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=25805
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=25817
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=25829
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=25841
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=25853
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=25865
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=91254
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
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=25241
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=25253
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=25265
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=25277
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=25289
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=25301
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=25313
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=25325
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=25337
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=25349
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=25361
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=25373
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=25385
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=25397
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=25409
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=25421
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=25433
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=25445
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=25457
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=25469
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=25481
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=25493
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=25505
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=25517
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=25529
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=25541
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=25553
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=25565
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=25577
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=25589
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=25601
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=25613
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=25625
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=25637
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=25649
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=25661
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=25673
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=25685
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=25697
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=25709
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=25721
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=25733
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=25745
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=25757
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=25769
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=25781
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=25793
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=25805
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=25817
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=25829
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=25841
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=25853
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=25865
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=91254
```

`r2-ordinary-copy-attribution`: `python3 -c import subprocess,pathlib; w=pathlib.Path(".temp/brief-615"); run=lambda s,*a:subprocess.run(["python3",str(w/s),*a],check=True); run("resolve-plt-attribution.py","cli"); run("resolve-libc.py"); p=subprocess.run(["python3",str(w/"copy-gates-exact.py")],capture_output=True,text=True); (w/"ordinary-callgrind/copy-exact-resolution.log").write_text(p.stdout+p.stderr); assert p.returncode==0 or (p.returncode==1 and "Shared IFUNC call sites require further binding attribution" in p.stderr); run("resolve-copy-indirect.py"); run("leaf-proof.py"); run("address-deltas.py","cli")`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-ordinary-copy-attribution.log](../../../.temp/brief-615/gates/r2-ordinary-copy-attribution.log).

`r2-ordinary-default`: `cargo test --manifest-path .temp/brief-615/ordinary-controls/Cargo.toml --test ordinary -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
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

`r2-ordinary-no-default`: `cargo test --manifest-path .temp/brief-615/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
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

`r2-ownership`: `cargo test -p rum-stdlib --lib ownership_graph -- --nocapture`; exit 0.

```text
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 69.99s
```

`r2-ownership-audit`: `python3 .temp/brief-615/r2-audit-ownership.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-ownership-audit.log](../../../.temp/brief-615/gates/r2-ownership-audit.log).

`r2-ownership-discovery`: `cargo test -p rum-stdlib --lib discovery_report_without_inventory_input -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 5.86s
```

`r2-ownership-replay`: `python3 .temp/brief-615/r2-replay-ownership.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-ownership-replay.log](../../../.temp/brief-615/gates/r2-ownership-replay.log).

`r2-performance-allocations`: `python3 .temp/brief-615/perf-chunk.py alloc 0 175`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-performance-allocations.log](../../../.temp/brief-615/gates/r2-performance-allocations.log).

`r2-performance-instructions-1`: `python3 .temp/brief-615/perf-chunk.py instructions 0 70`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-performance-instructions-1.log](../../../.temp/brief-615/gates/r2-performance-instructions-1.log).

`r2-performance-instructions-2`: `python3 .temp/brief-615/perf-chunk.py instructions 70 175`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-performance-instructions-2.log](../../../.temp/brief-615/gates/r2-performance-instructions-2.log).

`r2-performance-preflight`: `python3 .temp/brief-615/perf-chunk.py check 0 175`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-performance-preflight.log](../../../.temp/brief-615/gates/r2-performance-preflight.log).

`r2-primary-builds`: `python3 .temp/brief-615/build-batch.py primary`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-primary-builds.log](../../../.temp/brief-615/gates/r2-primary-builds.log).

`r2-regenerate-contracts`: `cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 0.15s
```

`r2-rejection-census`: `cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 25.58s
```

`r2-report-evidence`: `python3 -c import subprocess; w=".temp/brief-615/"; [subprocess.run(["python3",w+s],check=True) for s in ["make-performance-report.py","collect-canonical-outputs.py","final-evidence.py"]]`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-report-evidence.log](../../../.temp/brief-615/gates/r2-report-evidence.log).

`r2-report-final-check`: `python3 .temp/brief-615/check-report.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-report-final-check.log](../../../.temp/brief-615/gates/r2-report-final-check.log).

`r2-resume-attribution`: `python3 .temp/brief-615/resume-attribution-batch.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-resume-attribution.log](../../../.temp/brief-615/gates/r2-resume-attribution.log).

`r2-runtime-context`: `cargo test -p rum-runtime --lib resume_evaluation_context -- --nocapture`; exit 0.

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1176 filtered out; finished in 0.16s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`r2-runtime-recursion`: `cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`r2-runtime-roots`: `cargo test -p rum-runtime --lib frame_root_trace -- --nocapture`; exit 0.

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 1.32s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`r2-runtime-stack`: `python3 .temp/brief-615/stack-batch.py runtime`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-runtime-stack.log](../../../.temp/brief-615/gates/r2-runtime-stack.log).

`r2-runtime-typed`: `cargo test -p rum-runtime --lib suspension::typed::tests -- --nocapture`; exit 0.

```text
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 0.57s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`r2-s7-fixtures-default`: `cargo test -p rum-stdlib --test suspension_oracle part_selectors_held_assignment -- --nocapture`; exit 0.

```text
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.39s
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=12
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=24
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=36
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=48
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=60
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=72
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=84
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=96
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=108
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=120
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=132
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=144
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=156
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=168
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=180
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=192
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=204
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=216
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=228
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=240
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=252
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=264
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=276
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=288
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=300
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=312
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=324
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=336
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=348
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=360
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=372
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=384
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=396
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=408
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=420
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=432
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=444
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=456
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=468
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=480
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=492
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=504
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=516
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=528
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=540
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=552
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=564
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=576
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=588
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=600
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=612
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=624
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=636
```

`r2-stack-growth-ledger`: `python3 .temp/brief-615/stack-growth-attribution.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-stack-growth-ledger.log](../../../.temp/brief-615/gates/r2-stack-growth-ledger.log).

`r2-timing-01-base`: `python3 .temp/brief-615/timing-pinned-idle.py 01-base base`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-timing-01-base.log](../../../.temp/brief-615/gates/r2-timing-01-base.log).

`r2-timing-02-candidate`: `python3 .temp/brief-615/timing-pinned-idle.py 02-candidate candidate`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-timing-02-candidate.log](../../../.temp/brief-615/gates/r2-timing-02-candidate.log).

`r2-timing-03-base`: `python3 .temp/brief-615/timing-pinned-idle.py 03-base base`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-timing-03-base.log](../../../.temp/brief-615/gates/r2-timing-03-base.log).

`r2-timing-04-candidate`: `python3 .temp/brief-615/timing-pinned-idle.py 04-candidate candidate`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-timing-04-candidate.log](../../../.temp/brief-615/gates/r2-timing-04-candidate.log).

`r2-timing-comparison`: `python3 -c import subprocess; w=".temp/brief-615/"; [subprocess.run(["python3",w+s],check=True) for s in ["compare-timing.py","prepare-alignment-probes.py"]]`; exit 1.
No test-result line emitted; raw output is retained in [gates/r2-timing-comparison.log](../../../.temp/brief-615/gates/r2-timing-comparison.log).

`r2-validation-freeze`: `python3 .temp/brief-615/finalize-validation.py`; exit 0.
No test-result line emitted; raw output is retained in [gates/r2-validation-freeze.log](../../../.temp/brief-615/gates/r2-validation-freeze.log).

`r2-workspace`: `cargo test --workspace --no-fail-fast -- --nocapture`; exit 0.

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 39.34s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.98s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 1184 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 40.64s
test result: ok. 1109 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1264.55s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.95s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 214 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.61s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.43s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.62s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 244.29s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.72s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.50s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.49s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.68s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
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
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=25241
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=25253
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=25265
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=25277
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=25289
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=25301
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=25313
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=25325
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=25337
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=25349
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=25361
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=25373
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=25385
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=25397
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=25409
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=25421
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=25433
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=25445
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=25457
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=25469
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=25481
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=25493
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=25505
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=25517
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=25529
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=25541
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=25553
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=25565
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=25577
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=25589
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=25601
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=25613
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=25625
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=25637
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=25649
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=25661
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=25673
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=25685
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=25697
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=25709
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=25721
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=25733
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=25745
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=25757
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=25769
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=25781
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=25793
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=25805
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=25817
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=25829
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=25841
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=25853
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=25865
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=91254
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: unused import: `rum_core::Symbol`
warning: unreachable call
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: unused import: `rum_core::Symbol`
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: function `consume` is never used
warning: 1 warning emitted
```

`regenerate-contracts`: `cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 0.21s
```

`rejection-census`: `cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 25.77s
```

`runtime-context`: `cargo test -p rum-runtime --lib resume_evaluation_context -- --nocapture`; exit 0.

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1176 filtered out; finished in 0.16s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`runtime-recursion`: `cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact`; exit 0.

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`runtime-roots`: `cargo test -p rum-runtime --lib frame_root_trace -- --nocapture`; exit 0.

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 1.24s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`runtime-typed`: `cargo test -p rum-runtime --lib suspension::typed::tests -- --nocapture`; exit 0.

```text
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 0.50s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`s7-fixtures-candidate-default`: `cargo test -p rum-stdlib --test suspension_oracle part_selectors_held_assignment -- --nocapture`; exit 101.

```text
test result: FAILED. 35 passed; 18 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.41s
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=12
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=24
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=36
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=48
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=60
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=72
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=84
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=96
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=108
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=120
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=132
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=144
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=156
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=168
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=180
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=192
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=204
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=216
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=228
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=240
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=252
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=264
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=276
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=288
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=300
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=312
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=324
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=336
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=348
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=360
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=372
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=384
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=396
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=408
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=420
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=432
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=444
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=456
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=468
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=480
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=492
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=504
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=516
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=528
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=540
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=552
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=564
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=576
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=588
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=600
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=612
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=624
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=636
```

`s7-fixtures-candidate-default-2`: `cargo test -p rum-stdlib --test suspension_oracle part_selectors_held_assignment -- --nocapture`; exit 0.

```text
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 0.39s
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=12
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=24
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=36
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=48
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=60
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=72
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=84
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=96
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=108
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=120
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=132
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=144
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=156
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=168
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=180
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=192
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=204
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=216
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=228
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=240
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=252
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=264
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=276
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=288
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=300
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=312
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=324
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=336
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=348
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=360
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=372
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=384
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=396
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=408
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=420
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=432
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=444
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=456
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=468
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=480
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=492
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=504
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=516
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=528
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=540
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=552
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=564
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=576
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=588
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=600
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=612
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=624
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=636
```

`typed-runtime-2`: `cargo test -p rum-runtime --lib suspension::typed::tests`; exit 0.

```text
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 0.50s
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
```

`typed-runtime-initial`: `cargo test -p rum-runtime --lib suspension::typed::tests`; exit 101.

```text
warning: unused import: `resume_frames::*`
warning: `rum-runtime` (lib test) generated 1 warning
```

`workspace`: `cargo test --workspace --no-fail-fast -- --nocapture`; exit 0.

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 52.88s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.03s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 1184 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 44.93s
test result: ok. 1109 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1297.26s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.73s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.96s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 214 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.44s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.58s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.34s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 245.39s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.91s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.28s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.64s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
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
test oracle::fixtures::part_selectors_held_assignment::director::director_01 ... continuation-slot fault-occurrence delta=0; observations=25241
test oracle::fixtures::part_selectors_held_assignment::director::director_03 ... continuation-slot fault-occurrence delta=0; observations=25253
test oracle::fixtures::part_selectors_held_assignment::director::director_05 ... continuation-slot fault-occurrence delta=0; observations=25265
test oracle::fixtures::part_selectors_held_assignment::director::director_07 ... continuation-slot fault-occurrence delta=0; observations=25277
test oracle::fixtures::part_selectors_held_assignment::director::director_09 ... continuation-slot fault-occurrence delta=0; observations=25289
test oracle::fixtures::part_selectors_held_assignment::director::director_10 ... continuation-slot fault-occurrence delta=0; observations=25301
test oracle::fixtures::part_selectors_held_assignment::director::director_12 ... continuation-slot fault-occurrence delta=0; observations=25313
test oracle::fixtures::part_selectors_held_assignment::director::director_13 ... continuation-slot fault-occurrence delta=0; observations=25325
test oracle::fixtures::part_selectors_held_assignment::director::director_14 ... continuation-slot fault-occurrence delta=0; observations=25337
test oracle::fixtures::part_selectors_held_assignment::director::director_15 ... continuation-slot fault-occurrence delta=0; observations=25349
test oracle::fixtures::part_selectors_held_assignment::director::director_16 ... continuation-slot fault-occurrence delta=0; observations=25361
test oracle::fixtures::part_selectors_held_assignment::director::director_17 ... continuation-slot fault-occurrence delta=0; observations=25373
test oracle::fixtures::part_selectors_held_assignment::director::director_18 ... continuation-slot fault-occurrence delta=0; observations=25385
test oracle::fixtures::part_selectors_held_assignment::director::director_19 ... continuation-slot fault-occurrence delta=0; observations=25397
test oracle::fixtures::part_selectors_held_assignment::director::director_20 ... continuation-slot fault-occurrence delta=0; observations=25409
test oracle::fixtures::part_selectors_held_assignment::director::director_21 ... continuation-slot fault-occurrence delta=0; observations=25421
test oracle::fixtures::part_selectors_held_assignment::director::director_22 ... continuation-slot fault-occurrence delta=0; observations=25433
test oracle::fixtures::part_selectors_held_assignment::director::director_23 ... continuation-slot fault-occurrence delta=0; observations=25445
test oracle::fixtures::part_selectors_held_assignment::director::director_24 ... continuation-slot fault-occurrence delta=0; observations=25457
test oracle::fixtures::part_selectors_held_assignment::director::director_25 ... continuation-slot fault-occurrence delta=0; observations=25469
test oracle::fixtures::part_selectors_held_assignment::director::director_26 ... continuation-slot fault-occurrence delta=0; observations=25481
test oracle::fixtures::part_selectors_held_assignment::director::director_27 ... continuation-slot fault-occurrence delta=0; observations=25493
test oracle::fixtures::part_selectors_held_assignment::director::director_28 ... continuation-slot fault-occurrence delta=0; observations=25505
test oracle::fixtures::part_selectors_held_assignment::director::director_29 ... continuation-slot fault-occurrence delta=0; observations=25517
test oracle::fixtures::part_selectors_held_assignment::director::director_30 ... continuation-slot fault-occurrence delta=0; observations=25529
test oracle::fixtures::part_selectors_held_assignment::director::director_31 ... continuation-slot fault-occurrence delta=0; observations=25541
test oracle::fixtures::part_selectors_held_assignment::director::director_32 ... continuation-slot fault-occurrence delta=0; observations=25553
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_02 ... continuation-slot fault-occurrence delta=0; observations=25565
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_04 ... continuation-slot fault-occurrence delta=0; observations=25577
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_06 ... continuation-slot fault-occurrence delta=0; observations=25589
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_08 ... continuation-slot fault-occurrence delta=0; observations=25601
test oracle::fixtures::part_selectors_held_assignment::director::q5_director_11 ... continuation-slot fault-occurrence delta=0; observations=25613
test oracle::fixtures::part_selectors_held_assignment::held::frozen_target_during_update ... continuation-slot fault-occurrence delta=0; observations=25625
test oracle::fixtures::part_selectors_held_assignment::held::held_part_twice_order ... continuation-slot fault-occurrence delta=0; observations=25637
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call ... continuation-slot fault-occurrence delta=0; observations=25649
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_call_thrice ... continuation-slot fault-occurrence delta=0; observations=25661
test oracle::fixtures::part_selectors_held_assignment::held::s7_assoc_part ... continuation-slot fault-occurrence delta=0; observations=25673
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_argument ... continuation-slot fault-occurrence delta=0; observations=25685
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_held ... continuation-slot fault-occurrence delta=0; observations=25697
test oracle::fixtures::part_selectors_held_assignment::held::s7_associate_target ... continuation-slot fault-occurrence delta=0; observations=25709
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_rhs ... continuation-slot fault-occurrence delta=0; observations=25721
test oracle::fixtures::part_selectors_held_assignment::held::s7_held_twice ... continuation-slot fault-occurrence delta=0; observations=25733
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_argument ... continuation-slot fault-occurrence delta=0; observations=25745
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_held ... continuation-slot fault-occurrence delta=0; observations=25757
test oracle::fixtures::part_selectors_held_assignment::held::s7_keydrop_target ... continuation-slot fault-occurrence delta=0; observations=25769
test oracle::fixtures::part_selectors_held_assignment::held::s7_list_part ... continuation-slot fault-occurrence delta=0; observations=25781
test oracle::fixtures::part_selectors_held_assignment::held::selector_throw_after_resume ... continuation-slot fault-occurrence delta=0; observations=25793
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_empty_some ... continuation-slot fault-occurrence delta=0; observations=25805
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_none ... continuation-slot fault-occurrence delta=0; observations=25817
test oracle::fixtures::part_selectors_held_assignment::markers::assoc_some ... continuation-slot fault-occurrence delta=0; observations=25829
test oracle::fixtures::part_selectors_held_assignment::markers::part_empty_some ... continuation-slot fault-occurrence delta=0; observations=25841
test oracle::fixtures::part_selectors_held_assignment::markers::part_none ... continuation-slot fault-occurrence delta=0; observations=25853
test oracle::fixtures::part_selectors_held_assignment::markers::part_some ... continuation-slot fault-occurrence delta=0; observations=25865
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=91254
warning: function `register_test_if_builtin` is never used
warning: function `register_test_return_builtin` is never used
warning: function `register_test_vm_template_builtins` is never used
warning: `rum-runtime` (lib test) generated 3 warnings
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: unused import: `rum_core::Symbol`
warning: unreachable call
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: function `consume` is never used
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: struct `Capture` is never constructed
warning: struct `Frozen` is never constructed
warning: `option-read-proof` (bin "option-read-proof") generated 2 warnings
warning: unused import: `rum_core::Symbol`
warning: `option-read-proof` (bin "option-read-proof") generated 1 warning
warning: function `consume` is never used
warning: 1 warning emitted
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
Standalone calibration, candidate stack and resident-backtrace test totals, verbatim:

```text
stack-calibration/debug/guarded-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/guarded-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/guarded-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/guarded-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/guarded-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/guarded-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/guarded-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/guarded-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/guarded-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/guarded-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/guarded-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/guarded-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/peel-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack-calibration/debug/peel-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/peel-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/peel-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/peel-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/peel-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/peel-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/peel-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/peel-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/peel-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/peel-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/peel-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/plus-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/rhs-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/rhs-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/rhs-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/rhs-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/rhs-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/rhs-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/rhs-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/rhs-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/rhs-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/rhs-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack-calibration/debug/rhs-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/rhs-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/sub-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/up-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/up-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/up-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/up-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/up-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/up-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/up-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/up-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/up-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/up-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack-calibration/debug/up-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/debug/up-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack-calibration/release/guarded-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/guarded-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/guarded-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/guarded-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/guarded-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/guarded-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/guarded-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/guarded-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/guarded-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/guarded-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/guarded-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/guarded-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/peel-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/plus-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/rhs-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/sub-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack-calibration/release/up-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/debug/backtrace-guarded-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-guarded-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-peel-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.15s
stack/base/debug/backtrace-peel-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-plus-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.05s
stack/base/debug/backtrace-plus-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-rhs-guard-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-rhs-guard-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-sub-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.05s
stack/base/debug/backtrace-sub-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-up-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/debug/backtrace-up-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/base/release/backtrace-guarded-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-guarded-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-peel-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s
stack/base/release/backtrace-peel-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-plus-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-plus-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-rhs-guard-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-rhs-guard-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-sub-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-sub-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-up-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/backtrace-up-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/candidate/debug/backtrace-guarded-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-guarded-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-peel-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.08s
stack/candidate/debug/backtrace-peel-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.03s
stack/candidate/debug/backtrace-plus-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-plus-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-rhs-guard-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-rhs-guard-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-sub-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.05s
stack/candidate/debug/backtrace-sub-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.04s
stack/candidate/debug/backtrace-up-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.05s
stack/candidate/debug/backtrace-up-4.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.05s
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
stack/candidate/debug/guarded-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
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
stack/candidate/debug/rhs-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/candidate/debug/rhs-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
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
stack/candidate/debug/sub-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
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
stack/candidate/release/peel-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
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
Canonical output roots written by unchanged required tools:

```text
.temp/brief-542
.temp/brief-543
.temp/brief-546
.temp/typed-slot-census
.temp/typed-slot-compile-controls
target/brief-535
target/brief-539
target/brief-540
target/brief-544
.temp/suspension-oracle
target/debug
```
The complete file-level canonical output index is [canonical-output-files.json](../../../.temp/brief-615/canonical-output-files.json). The full command/exit index is [gates-index.json](../../../.temp/brief-615/gates-index.json); raw logs remain beside each command.

### 5. Allocation, instructions, copies, stack and timing

All data below use the final corrected source. The rejected wrapper candidate is retained separately and is not substituted into any final comparison. The final panel has 175 labels: 126 ordinary and 49 resumed. Every native allocation/instruction process and every Callgrind process passed its result check; resumed workloads also passed the exact pause-count check. Allocation and native instruction panels retain original N/2N. Ordinary address profiling uses the disclosed 20M-instruction budget (at most original N or 2000, at least one); resumed profiles use N=100/200. Raw commands, budgets, preflights and per-process results are retained beside each profile.

The primary release builds use one codegen unit and the required cwd. Source snapshots match 1497 candidate inputs and the 1492-input base; all eight primary measurement binaries retain their recorded hashes. [final-source-audit.json](../../../.temp/brief-615/final-source-audit.json) and [perf/measured-binaries.json](../../../.temp/brief-615/perf/measured-binaries.json) record the exact source/binary proof.

| Binary | SHA-256 |
| --- | --- |
| base/rum-cli | 14c221bf6e8e6fc92e9aa26ef6837dda4332f95ea8814b0a73ec77123e4f1c35 |
| base/rum-cli-alloc | 38c43f9c4031a6b3f2b1690d8b513a5912f28963ab2618c4f75e440e915b0297 |
| base/resume | 3b400b1a4556ff5c962da4880514f9d8662ca3508846d293c0c6505039b1c4c0 |
| base/resume-alloc | c797907f4b12b4f0e20131b4b00eb05ea7cc7def47ac8a4bb27c033e2bfa674c |
| cand/rum-cli | 0b4dc295480c717110d7db754c19df92d87a669e4681918fa15e9bc5236d2e13 |
| cand/rum-cli-alloc | 6612931f16dede74336b386fadc1c1798777fed18db17f195ef0a762fae8e231 |
| cand/resume | 96973ce2e58c4081a2b61c559738e9faacf196f4f11f834b11d0bea55d413940 |
| cand/resume-alloc | 4e0bba57edb422a9393d539ea76deec3c1fced98b0e3c098231b16ce11f45606 |

**Allocations.** All 126 ordinary labels have exactly equal counts and requested bytes at N and 2N, and equal differential slopes (700 correct allocation processes across the complete panel). Full values: [perf/allocation-comparison.json](../../../.temp/brief-615/perf/allocation-comparison.json). The three inherited workloads argument-twice, s5-destructure and s5-destructure-twice also enter newly typed S7 frames; they are not claimed allocation-unchanged.

For 47 resumed labels, the measured increment is exactly one 24-byte adapter Arc per new typed transfer, with zero intercept change. The two publishing workloads are reconciled below at actual allocator addresses. Every resumed label follows; counts/bytes are per workload iteration, and the pause column makes conversion to per-resume cost explicit.

| Workload | Pauses/iteration | New typed entries | Allocation delta | Byte delta | Count/byte intercept delta | New 136B driver copies |
| --- | --- | --- | --- | --- | --- | --- |
| resume-table | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-do-chain | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-call-repeat | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-cleanup | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-fold | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-argument-twice | 2 | 2.0 | 2.0 | 48.0 | 0.0 / 0.0 | 2.0 |
| resume-rewrite-exprs | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-rewrite-compiled | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-cases-spec | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-cases-traversal | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-position-traversal | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-count-traversal | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-cases-shared | 2 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-down-rhs | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-down-guard | 2 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-sub-rhs | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-sub-guard | 2 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-up-rhs | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-up-guard | 2 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-default-rhs | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-default-guard | 2 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-default-fallback-rhs | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-set | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-upset | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-tagset | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-rule-rhs | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-rule-lhs | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-destructure | 1 | 2.0 | 2.0 | 48.0 | 0.0 / 0.0 | 2.0 |
| resume-s5-lhs-twice | 2 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-own-rhs | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s5-destructure-twice | 2 | 4.0 | 4.0 | 96.0 | 0.0 / 0.0 | 4.0 |
| resume-s5-rule-lhs-twice | 2 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s6-rhs-single | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s6-rhs-twice | 2 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s6-cross-type | 2 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s6-readonly-twice | 4 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s6-string | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s7-list-part | 1 | 2.0 | 2.0 | 48.0 | 0.0 / 0.0 | 2.0 |
| resume-s7-assoc-part | 1 | 2.0 | 2.0 | 48.0 | 0.0 / 0.0 | 2.0 |
| resume-s7-assoc-call | 1 | 1.0 | 1.0 | 24.0 | 0.0 / 0.0 | 1.0 |
| resume-s7-assoc-call-thrice | 3 | 3.0 | 3.0 | 72.0 | 0.0 / 0.0 | 3.0 |
| resume-s7-held-twice | 2 | 2.0 | 2.0 | 48.0 | 0.0 / 0.0 | 2.0 |
| resume-s7-held-rhs | 1 | 1.0 | 1.0 | 24.0 | 0.0 / 0.0 | 1.0 |
| resume-s7-associate-argument | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s7-keydrop-argument | 1 | 0.0 | 0.0 | 0.0 | 0.0 / 0.0 | 0.0 |
| resume-s7-associate-target | 1 | 1.0 | 1.0 | 24.0 | 0.0 / 0.0 | 1.0 |
| resume-s7-keydrop-target | 1 | 1.0 | 1.0 | 24.0 | 0.0 / 0.0 | 1.0 |
| resume-s7-associate-held | 1 | 2.0 | 1.0 | 39.0 | 2.0 / 82.0 | 2.0 |
| resume-s7-keydrop-held | 1 | 2.0 | 2.0 | 40.0 | 2.0 / 82.0 | 2.0 |

The publishing removal deletes the 48-byte Arc<OnceLock> at base allocation call `0x5f8c13`, once per held-association iteration. The complete allocation balance is: 

| Operation | Count delta | Byte delta | Base allocation call(s) | Candidate allocation call(s) |
| --- | --- | --- | --- | --- |
| Two typed adapter Arcs | 2 | 48 | absent | 0x115a5d, 0xa9512 |
| Removed Arc<OnceLock<EvaluatedPartSelectors>> | -1 | -48 | 0x5f8c13 | absent |
| F0348 frame replaced by zero-sized F0347 frame (Arc header remains) | 0 | -8 | 0xe9cf3 | 0x115a18 |
| F0047 frame loses publishing continuation field | 0 | -8 | 0x7f36f | 0xa949e |
| Removed one-selector Vec clone | -1 | -32 | 0x47f782 | absent |
| Legacy selector wire SharedExpr and SharedArgs headers | 2 | 88 | absent | 0x7bd94e, 0x7bdbad |
| Removed one-byte string selector clone | -1 | -1 | 0x80b018 | absent |

KeyDropFrom has every row except the one-byte string clone: its selector is a shared Key call. Thus AssociateTo is +1 allocation/+39 bytes and KeyDropFrom is +2/+40 per iteration. F0047’s actual Arc allocation shrinks 120 → 112 bytes; F0348’s 24-byte Arc is replaced by the 16-byte zero-sized F0347 Arc. The new selector Legacy wire contributes 48-byte SharedExpr and 40-byte SharedArgs allocations. The removed one-selector Vec is 32 bytes: the base allocator argument is length shifted left five at `0x47f731`, passed at `0x47f77f` into `0x47f782`. Its old caller is the deleted continuation finish at `0x5f880e`.

Both publishing workloads add two allocations/82 bytes once per process: the exact 42-byte `InternalContinuationEvaluatedPartSelectors` literal, plus its 40-byte interned SymbolText Arc. The final encoder loads literal `0x849e2d`, length 42 at `0x77bc90`, and calls Symbol::system at `0x77bc95`; allocation sites are `0x7ba955` and `0x7bab9e`. All three pairs’ allocation-site counts, immediate argument proofs, dynamic-size source/call-chain proof and the N/2N equations are in [attribution/publishing-allocations/reconciled.json](../../../.temp/brief-615/attribution/publishing-allocations/reconciled.json). There is no unexplained allocation or byte delta.

**Ordinary instructions and copies.** The native panel contains 1400 correct processes: two interleaved N/2N pairs per arm, pinned to CPU 2 with cpu_core/instructions/u and stdin /dev/null. Its flag threshold is max(0.05% of base slope, the observed base range, the candidate range). Fifteen ordinary labels flagged; all received three Callgrind pairs (six processes per arm). The three leaves, both positional assignment probes, explicit Key Part investigation and two AppendTo Part neighbors are also repeated: 22 labels total. All 126 labels have at least one full N/2N Callgrind pair, for 680 ordinary profiles. Per-process function counts, exact instructions and all changed addresses are in [attribution/cli](../../../.temp/brief-615/attribution/cli); each address ledger sums to its measured delta within 1e-5 rounding. Single-pair values are not described as stable.

Every ordinary label passes memcpy and memmove at N, 2N and slope. Indirect GOT, cached-register and libc IFUNC bindings are resolved, not guessed from a shared memcpy/memmove target. The 22 repeated labels also pass in all 66 pairs. [ordinary-callgrind/copy-gate-resolved.tsv](../../../.temp/brief-615/ordinary-callgrind/copy-gate-resolved.tsv), [ordinary-callgrind/repeated-copy-proof.json](../../../.temp/brief-615/ordinary-callgrind/repeated-copy-proof.json) and [attribution/copy-register-bindings.json](../../../.temp/brief-615/attribution/copy-register-bindings.json) contain the whole-program counts and caller addresses.

| Ordinary label | Native base I/iteration | Native delta | Native flag | Callgrind pairs | Stable function delta subtotal (3 pairs only) | Whole N/2N allocation + memcpy gates |
| --- | --- | --- | --- | --- | --- | --- |
| lazy-range-take-small | 41864.599 | +1.837 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-range-normal-small | 53544.993 | +53.281 | True | 3 | +0.000 | exact / pass |
| lazy-map-realize-small | 176352.293 | +214.885 | True | 3 | +0.000 | exact / pass |
| lazy-select-realize-small | 112569.245 | -91.411 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-select-scan-small | 793592.474 | -344.930 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-nestlist-realize-small | 142955.697 | -28.221 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-foldlist-realize-small | 110790.976 | +12.488 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-drop-realize-small | 188619.757 | -81.680 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-first-small | 40597.113 | -0.693 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-part-small | 87052.013 | -44.843 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-transparent-adapters-small | 178616.480 | -144.047 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-table-construct-short | 23405.753 | -39.447 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-table-construct-full | 23835.903 | +40.361 | True | 3 | +0.000 | exact / pass |
| lazy-table-realize-small | 115746.712 | +78.993 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-permutations-take-small | 27881.107 | -23.780 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-subsets-take-small | 24969.715 | +61.085 | True | 3 | +0.000 | exact / pass |
| lazy-tuples-take-small | 26279.759 | -26.867 | False | 1 | one pair; no stability claim | exact / pass |
| scan-no-input | 28965.677 | -5.407 | False | 1 | one pair; no stability claim | exact / pass |
| optionvalue-three-no-input | 9492.581 | -1.322 | False | 1 | one pair; no stability claim | exact / pass |
| replacerepeated-no-input | 38996.778 | -14.450 | False | 1 | one pair; no stability claim | exact / pass |
| dot-three-no-input | 249967.925 | -0.919 | False | 1 | one pair; no stability claim | exact / pass |
| differences-no-input | 9909.793 | -0.074 | False | 1 | one pair; no stability claim | exact / pass |
| do-count-null | 534.940 | +0.030 | False | 3 | +0.000 | exact / pass |
| do-iterator-null | 1151.980 | +0.032 | False | 3 | +0.000 | exact / pass |
| table-iterator-null | 387.073 | -0.053 | False | 1 | one pair; no stability claim | exact / pass |
| replaceall-simple | 11322.146 | -0.039 | False | 1 | one pair; no stability claim | exact / pass |
| table-small | 14509.712 | -0.338 | False | 1 | one pair; no stability claim | exact / pass |
| table-iterator-plus | 563.440 | -0.010 | False | 1 | one pair; no stability claim | exact / pass |
| table-iterator-pair-list | 1640.433 | +0.143 | False | 1 | one pair; no stability claim | exact / pass |
| take-multispec-small | 49714.208 | +0.454 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-map-build-8 | 17792.555 | +3.033 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-map-first-8 | 128203.605 | +103.506 | True | 3 | +0.000 | exact / pass |
| lazy-depth-map-eight-8 | 945232.882 | +20.789 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-select-restart-8 | 177929.693 | +259.710 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-drop-all-one-8 | 46371.745 | +1.184 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-take-map-8 | 136857.704 | -18.245 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-fold-through-source-8 | 581139.283 | -29.603 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-nest-map-8 | 138568.445 | +20.790 | True | 3 | +0.000 | exact / pass |
| lazy-depth-map-build-64 | 29801.984 | +13.308 | True | 3 | +0.000 | exact / pass |
| lazy-depth-map-first-64 | 933206.275 | +1657.827 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-map-eight-64 | 7298198.434 | +478.312 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-select-restart-64 | 970202.863 | +382.130 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-drop-all-one-64 | 327355.007 | +2.387 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-take-map-64 | 993040.484 | -7.591 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-fold-through-source-64 | 33526935.541 | -77.483 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-nest-map-64 | 943969.670 | +1661.237 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-map-build-256 | 70454.583 | -7.838 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-map-first-256 | 3663022.647 | -3378.215 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-map-eight-256 | 29093023.096 | -574.720 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-select-restart-256 | 3646398.699 | +6062.745 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-drop-all-one-256 | 2218619.143 | -9.201 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-take-map-256 | 3925904.922 | +67.700 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-fold-through-source-256 | 523298981.544 | +19741.685 | False | 1 | one pair; no stability claim | exact / pass |
| lazy-depth-nest-map-256 | 3674635.251 | -20.702 | False | 1 | one pair; no stability claim | exact / pass |
| literal-integer | 291.911 | +0.164 | True | 3 | +0.000 | exact / pass |
| downvalue-pattern-hit | 13647.114 | -0.071 | False | 1 | one pair; no stability claim | exact / pass |
| module-local | 22433.321 | -0.095 | False | 1 | one pair; no stability claim | exact / pass |
| block-local | 14463.027 | -0.092 | False | 1 | one pair; no stability claim | exact / pass |
| matcher-single-wide-small | 760073.292 | +12.892 | False | 1 | one pair; no stability claim | exact / pass |
| cases-wide-no-match | 2400973.256 | +1.133 | False | 1 | one pair; no stability claim | exact / pass |
| count-wide-no-match | 2179477.875 | -21.762 | False | 1 | one pair; no stability claim | exact / pass |
| position-wide-no-match | 2574069.927 | -77.193 | False | 1 | one pair; no stability claim | exact / pass |
| deletecases-wide-no-match | 2670999.972 | -54.065 | False | 1 | one pair; no stability claim | exact / pass |
| deletecases-wide-association-no-match | 7501684.606 | -49.563 | False | 1 | one pair; no stability claim | exact / pass |
| deletecases-leaf-delete-small | 63193.779 | -127.839 | False | 1 | one pair; no stability claim | exact / pass |
| replace-wide-no-match | 1164301.744 | +126.479 | False | 1 | one pair; no stability claim | exact / pass |
| replace-leaf-rebuild-small | 73241.752 | +408.302 | False | 1 | one pair; no stability claim | exact / pass |
| rewrite-guarded-hit | 52747.279 | -9.046 | False | 1 | one pair; no stability claim | exact / pass |
| rewrite-guarded-decline-fallback | 77009.957 | +60.200 | False | 1 | one pair; no stability claim | exact / pass |
| rule-rhs-deep-small | 82500.510 | -78.920 | False | 1 | one pair; no stability claim | exact / pass |
| part-list | 8304.424 | +0.655 | False | 1 | one pair; no stability claim | exact / pass |
| appendto-part-outer1000-small | 2207196.425 | +10674.965 | False | 3 | +0.000 | exact / pass |
| appendto-part-outer1000-large | 3614906.538 | -29827.708 | False | 3 | +0.000 | exact / pass |
| set-part-fixed100-len100 | 1267777.930 | -2957.640 | False | 3 | -800.000 | exact / pass |
| set-part-fixed100-len800 | 1267550.975 | -2404.480 | False | 3 | -800.000 | exact / pass |
| set-part-fill-small | 2786414.007 | -15017.897 | False | 1 | one pair; no stability claim | exact / pass |
| set-part-fill-large | 5590595.015 | -9591.420 | False | 1 | one pair; no stability claim | exact / pass |
| table-iterator-symbol | 332.137 | +0.956 | True | 3 | +0.000 | exact / pass |
| symbol-ownvalue-read | 2207.116 | +0.699 | True | 3 | +0.000 | exact / pass |
| symbol-ownvalue-guarded-read | 3566.898 | +0.076 | False | 1 | one pair; no stability claim | exact / pass |
| block-symbol-only-repeat | 10687.039 | -0.553 | False | 1 | one pair; no stability claim | exact / pass |
| downvalue-exact-hit | 5751.969 | -2.173 | False | 1 | one pair; no stability claim | exact / pass |
| downvalue-exact-miss | 997.844 | +0.346 | True | 3 | +0.000 | exact / pass |
| downvalue-pattern-miss | 998.508 | -1.710 | False | 1 | one pair; no stability claim | exact / pass |
| downvalue-guarded-hit | 27571.678 | +0.449 | False | 1 | one pair; no stability claim | exact / pass |
| downvalue-guarded-decline-fallback | 20873.234 | -7.079 | False | 1 | one pair; no stability claim | exact / pass |
| upvalue-guarded-hit | 33853.231 | +0.227 | False | 1 | one pair; no stability claim | exact / pass |
| subvalue-unguarded-hit | 20066.568 | +0.245 | False | 1 | one pair; no stability claim | exact / pass |
| subvalue-unguarded-miss | 11281.229 | -0.007 | False | 1 | one pair; no stability claim | exact / pass |
| subvalue-guarded-hit | 35217.392 | +0.117 | False | 1 | one pair; no stability claim | exact / pass |
| default-unguarded-hit | 17192.549 | +0.761 | False | 1 | one pair; no stability claim | exact / pass |
| default-unguarded-miss | 12784.970 | +0.267 | False | 1 | one pair; no stability claim | exact / pass |
| default-guarded-hit | 22363.217 | -0.601 | False | 1 | one pair; no stability claim | exact / pass |
| downvalue-exact-miss-repeat | 776.019 | -0.215 | False | 1 | one pair; no stability claim | exact / pass |
| downvalue-pattern-miss-repeat | 776.854 | -0.904 | False | 1 | one pair; no stability claim | exact / pass |
| upvalue-candidates-tiny | 31377.147 | -7.889 | False | 1 | one pair; no stability claim | exact / pass |
| upvalue-candidates-wide-small | 3063771.650 | +167.750 | False | 1 | one pair; no stability claim | exact / pass |
| upvalue-candidates-wide-large | 6121404.000 | -1829.675 | False | 1 | one pair; no stability claim | exact / pass |
| sort-default-wide | 342582.500 | -89.100 | False | 1 | one pair; no stability claim | exact / pass |
| sortby-default-wide | 7426070.325 | +242.900 | False | 1 | one pair; no stability claim | exact / pass |
| reversesort-default-wide | 340910.675 | +110.450 | True | 3 | +0.000 | exact / pass |
| keysort-default-wide | 8258809.800 | +813.450 | False | 1 | one pair; no stability claim | exact / pass |
| keysortby-default-wide | 15448369.500 | +189.700 | False | 1 | one pair; no stability claim | exact / pass |
| sort-symbol-default-width4000 | 5885397.350 | -1713.500 | False | 1 | one pair; no stability claim | exact / pass |
| sort-symbol-default-width16000 | 23366788.150 | +7337.250 | True | 3 | +0.000 | exact / pass |
| valueq-undefined-symbol-definitions-2000 | 4044.175 | -475.400 | False | 1 | one pair; no stability claim | exact / pass |
| valueq-undefined-symbol-definitions-8000 | 9093.400 | -3793.675 | False | 1 | one pair; no stability claim | exact / pass |
| matcher-single-tiny | 11576.876 | -0.266 | False | 1 | one pair; no stability claim | exact / pass |
| matcher-single-wide-large | 1510880.922 | -3.156 | False | 1 | one pair; no stability claim | exact / pass |
| replacepart-deep-path-small | 40821.946 | +5.996 | False | 1 | one pair; no stability claim | exact / pass |
| replacepart-wide-rule-list | 5968110.475 | +11.900 | False | 1 | one pair; no stability claim | exact / pass |
| replacepart-guarded-hit | 15790.522 | -13.255 | False | 1 | one pair; no stability claim | exact / pass |
| replacepart-guarded-decline-fallback | 18170.965 | +8.345 | True | 3 | +0.000 | exact / pass |
| stringreplace-single-literal-unguarded | 37189.053 | +23.477 | False | 1 | one pair; no stability claim | exact / pass |
| stringreplace-pattern-unguarded | 31653.759 | +2.933 | False | 1 | one pair; no stability claim | exact / pass |
| stringreplace-guarded-hit | 21594.562 | +7.489 | False | 1 | one pair; no stability claim | exact / pass |
| stringreplace-guarded-decline-fallback | 25889.183 | +36.272 | True | 3 | +0.000 | exact / pass |
| association-part-key-read-width4000 | 14204.050 | -19.662 | False | 1 | one pair; no stability claim | exact / pass |
| association-part-positional-read-width4000 | 9989.862 | -43.662 | False | 1 | one pair; no stability claim | exact / pass |
| association-part-key-read-width32000 | 14274.300 | -102.175 | False | 1 | one pair; no stability claim | exact / pass |
| association-part-positional-read-width32000 | 10168.825 | -65.075 | False | 1 | one pair; no stability claim | exact / pass |
| appendto-list-incremental-small | 10588.324 | -24.624 | False | 1 | one pair; no stability claim | exact / pass |
| appendto-list-incremental-large | 10603.357 | -51.678 | False | 1 | one pair; no stability claim | exact / pass |
| association-set-key-incremental-small | 17249.540 | -5.610 | False | 1 | one pair; no stability claim | exact / pass |
| association-set-key-incremental-large | 17204.022 | -3.110 | False | 1 | one pair; no stability claim | exact / pass |
| association-set-part-key-investigation | 14577.805 | +7.370 | False | 3 | +5.000 | exact / pass |

All three leaves (`literal-integer`, `do-count-null`, `do-iterator-null`) have zero deterministic function growth across three pairs. Session::evaluate is exactly 110 instructions per iteration in both arms and every pair. Small variable hashing/interning costs are retained, including positive means outside one standard error; they are not rounded into deterministic zeros. [ordinary-callgrind/leaf-proof.json](../../../.temp/brief-615/ordinary-callgrind/leaf-proof.json) preserves the full totals, variation and address evidence. The AppendTo neighbors likewise have no stable function delta; their variable cache/symbol costs remain disclosed in [ordinary-callgrind/final-function-summary.json](../../../.temp/brief-615/ordinary-callgrind/final-function-summary.json).

The positional assignment function falls 100 → 93 own instructions per assignment, and the association-lvalue decline on those workloads falls 123 → 122. Each fixed100 outer iteration therefore has a stable -800 own-function subtotal. The explicit pure-Key Part assignment retains **312 → 317 own instructions (+5 per assignment)**. Its stable address ledger has +9 register/memory movements, -3 control transfers and -1 comparison; arithmetic/other is equal. Examples of changed Session reloads are base `0x5d970f` versus candidate `0x5db335`, `0x5db365`, `0x5db578`, `0x5db5af`, `0x5db668`, `0x5db67a`, `0x5db778`, `0x5dbb24`, `0x5dbcdd`. The complete 263 changed-address rows and their balanced instruction sums are in [attribution/cli/association-set-part-key-investigation-address-deltas.tsv](../../../.temp/brief-615/attribution/cli/association-set-part-key-investigation-address-deltas.tsv) and [attribution/cli/association-part-classified-addresses.json](../../../.temp/brief-615/attribution/cli/association-part-classified-addresses.json). This is a director performance residual; it is not called zero cost or attributed solely to register allocation.

**Resumed instruction attribution.** All 49 workloads have three N/2N pairs per arm: 588 correct Callgrind processes, with six processes per arm per label. [attribution/resume/deterministic-function-index.tsv](../../../.temp/brief-615/attribution/resume/deterministic-function-index.tsv) names every one of the 724 observed stable function-delta occurrences at its original addresses and checks each address sum. Per-process function tables, per-pair address ledgers, body categories, SIMD moves and copy-call argument windows are under [attribution/resume](../../../.temp/brief-615/attribution/resume). Variable costs and confidence limits remain separate from stable changes. The stable subtotal below can include offsetting renamed/merged bodies; the whole delta includes measured process variation.

| Resume workload | Native instruction delta/iteration | Callgrind mean delta/iteration | Stable function subtotal | Variable function groups |
| --- | --- | --- | --- | --- |
| resume-table | -25.920 | +36.463 | +0.000 | 8 |
| resume-do-chain | +22.808 | -30.330 | +0.000 | 8 |
| resume-call-repeat | -6.049 | +14.648 | +0.000 | 24 |
| resume-cleanup | -99.114 | -3.480 | +0.000 | 8 |
| resume-fold | +12.224 | -15.182 | +0.000 | 8 |
| resume-argument-twice | +362.007 | +374.879 | +427.983 | 10 |
| resume-rewrite-exprs | +30.924 | -216.706 | +0.000 | 8 |
| resume-rewrite-compiled | +50.704 | -7.065 | +0.000 | 22 |
| resume-cases-spec | +272.924 | +37.129 | +0.000 | 9 |
| resume-cases-traversal | +71.355 | -46.782 | +0.000 | 10 |
| resume-position-traversal | +32.784 | +51.641 | +0.000 | 10 |
| resume-count-traversal | +73.107 | -13.147 | +0.000 | 10 |
| resume-cases-shared | +50.158 | -53.528 | +0.000 | 24 |
| resume-s5-down-rhs | -126.725 | -88.500 | +0.000 | 8 |
| resume-s5-down-guard | -164.560 | -141.100 | +0.000 | 22 |
| resume-s5-sub-rhs | +17.405 | -449.103 | +0.000 | 24 |
| resume-s5-sub-guard | +65.495 | -62.643 | +0.000 | 10 |
| resume-s5-up-rhs | +70.555 | -106.617 | +0.000 | 9 |
| resume-s5-up-guard | -233.240 | -17.700 | +0.000 | 8 |
| resume-s5-default-rhs | -129.515 | +187.317 | +0.000 | 12 |
| resume-s5-default-guard | +292.305 | +77.880 | +0.000 | 7 |
| resume-s5-default-fallback-rhs | -109.405 | -308.833 | +0.000 | 9 |
| resume-s5-set | -119.020 | +33.780 | +0.000 | 8 |
| resume-s5-upset | +0.975 | +18.087 | +0.000 | 10 |
| resume-s5-tagset | +144.510 | +73.647 | +0.000 | 8 |
| resume-s5-rule-rhs | +82.860 | -71.960 | +0.000 | 10 |
| resume-s5-rule-lhs | -12.975 | -86.713 | +0.000 | 23 |
| resume-s5-destructure | +288.710 | +423.930 | +400.080 | 10 |
| resume-s5-lhs-twice | +139.690 | +19.383 | -3.000 | 8 |
| resume-s5-own-rhs | -2.940 | -52.813 | +0.000 | 8 |
| resume-s5-destructure-twice | +1127.050 | +1011.773 | +791.060 | 9 |
| resume-s5-rule-lhs-twice | +19.145 | -286.933 | +0.000 | 9 |
| resume-s6-rhs-single | +81.660 | -92.930 | +0.000 | 8 |
| resume-s6-rhs-twice | -149.245 | +117.707 | +0.000 | 8 |
| resume-s6-cross-type | -210.025 | -10.910 | +0.000 | 9 |
| resume-s6-readonly-twice | +3.320 | +90.873 | +0.000 | 24 |
| resume-s6-string | +56.355 | +57.040 | +0.000 | 9 |
| resume-s7-list-part | +401.225 | +437.770 | +397.260 | 23 |
| resume-s7-assoc-part | +619.810 | +353.887 | +397.080 | 9 |
| resume-s7-assoc-call | +113.545 | +252.737 | +278.640 | 26 |
| resume-s7-assoc-call-thrice | +111.605 | +930.127 | +540.000 | 13 |
| resume-s7-held-twice | +410.205 | +411.143 | +430.820 | 9 |
| resume-s7-held-rhs | +235.460 | +135.657 | +208.600 | 9 |
| resume-s7-associate-argument | +44.565 | +60.750 | +0.000 | 10 |
| resume-s7-keydrop-argument | -80.905 | -85.423 | +0.000 | 9 |
| resume-s7-associate-target | +115.415 | +228.223 | +197.000 | 12 |
| resume-s7-keydrop-target | +164.740 | -102.797 | +212.840 | 12 |
| resume-s7-associate-held | +1075.340 | +1085.020 | +711.250 | 9 |
| resume-s7-keydrop-held | +1280.580 | +1364.987 | +958.430 | 11 |

The driver’s existing 136-byte typed-outcome move is owed to R and is unchanged. Its unique measured site set is base `{0x74547d, 0x74545d}` and candidate `{0x74692d, 0x74690d}`. The sum equals every TypedAdapter resume count for every label and all three pairs. The allocation table above gives the additional copies per iteration; multiply by 136 for moved bytes. [attribution/resume/driver-copy-proof.json](../../../.temp/brief-615/attribution/resume/driver-copy-proof.json) includes call-site windows and the all-label matrix. This is not confused with the rejected candidate’s extra ordinary Part copy, which is gone.

**Stack.** All twelve supplied-base calibrations and final candidate slopes are exact, with zero fit residual at heights 2/4/6/8 and three samples each. All 48 resident-backtrace checks reconcile the slopes to emitted native frames with zero residual. In particular peel, plus and rhs-guard do not grow.

| Profile | Shape | S6 bytes/level | S7 bytes/level | Delta | Unattributed residual |
| --- | --- | --- | --- | --- | --- |
| debug | peel | 19024 | 19024 | 0 | 0 |
| debug | plus | 34912 | 34912 | 0 | 0 |
| debug | sub | 29376 | 29376 | 0 | 0 |
| debug | up | 28064 | 28064 | 0 | 0 |
| debug | guarded | 35840 | 35840 | 0 | 0 |
| debug | rhs-guard | 49152 | 49152 | 0 | 0 |
| release | peel | 7344 | 7344 | 0 | 0 |
| release | plus | 13792 | 13792 | 0 | 0 |
| release | sub | 13168 | 13168 | 0 | 0 |
| release | up | 11456 | 11456 | 0 | 0 |
| release | guarded | 14240 | 14240 | 0 | 0 |
| release | rhs-guard | 17920 | 17920 | 0 | 0 |

The complete source-function coverage is below. nm aliases are included: baseline F0281 and F0282 share one emitted body. “Not emitted” means removed, inlined or optimized code, not a zero-byte frame. Prologue bytes exclude the return address and include saved registers. Macro-generated closures and all generic variants are additionally tabulated in [stack/cli/changed-prologues.tsv](../../../.temp/brief-615/stack/cli/changed-prologues.tsv); the independent runtime-probe table is [stack/changed-prologues.tsv](../../../.temp/brief-615/stack/changed-prologues.tsv).

| Profile | Changed source function | Base address: bytes | Candidate address: bytes |
| --- | --- | --- | --- |
| debug | current_held_update_value | 0xa00d50: 1480 | 0xa095d0: 1304 |
| debug | finish_held_update | 0xa04410: 1224 | 0xa0cc00: 792 |
| debug | suspend_held_part_selectors | not emitted | 0x1001b00: 376 |
| debug | suspend_held_update | not emitted | 0x1001ca0: 456 |
| debug | try_set_positional_part_lvalue | 0xa0ec20: 1816 | 0xa17340: 1432 |
| debug | try_set_association_lvalue | 0xa29090: 2408 | 0xa315e0: 2376 |
| debug | resume_association_set_lvalue_selector_evaluation | 0xa2a5b0: 1128 | 0xa32a70: 760 |
| debug | resume_suspended_association_set_lvalue | 0xa2acd0: 1096 | 0xfa0bd0: 1112, 0xfa1480: 1096 |
| debug | suspend_association_set_lvalue_selectors | not emitted | 0xc5d820: 376 |
| debug | suspend_suspended_association_set_lvalue | not emitted | 0xc6a7a0: 168, 0xc6a810: 168 |
| debug | evaluate_part_selectors_owned | 0xa5e980: 584 | 0xa66500: 472 |
| debug | evaluate_part_selectors_owned_for_builtin | 0xa5ec80: 984 | 0xa66800: 24 |
| debug | resume_associate_to_after_target_preparation | 0x5243a0: 1464 | 0x4ad330: 1144 |
| debug | resume_key_drop_from_after_target_preparation | 0x5260e0: 1176 | 0x4af070: 856 |
| debug | eval_held_association_part_mutation | 0x527c00: 1272 | 0x4b0b60: 680 |
| debug | suspend_positional_part_selectors | not emitted | 0xd309e0: 88 |
| debug | suspend_association_set_part_selectors | not emitted | 0xc5d9d0: 104 |
| debug | FinishHeldAssociationPartSelectorEvaluationSuspensionFrame::resume | 0x638210: 568 | 0x454e10: 568 |
| debug | EvalAssociateToSuspensionFrame::resume | 0x638480: 968 | 0x455080: 968 |
| debug | EvalKeyDropFromSuspensionFrame::resume | 0x6387f0: 968 | 0x4553f0: 968 |
| debug | FinishHeldPartSelectorEvaluationSuspensionFrame::resume | 0xf21f80: 472 | 0x1001690: 616 |
| debug | FinishHeldUpdateSuspensionFrame::resume | 0xf22140: 728 | 0x1001860: 728 |
| debug | FinishPositionalPartSelectorEvaluationSuspensionFrame::resume | 0xd1c5d0: 472 | 0xd30850: 488 |
| debug | FinishAssociationSetPartSelectorEvaluationSuspensionFrame::resume | 0xc4f270: 504 | 0xc5d310: 504 |
| debug | ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame::resume | 0xc4f430: 568 | 0xc5d4d0: 728 |
| debug | ResumeSuspendedAssociationSetLvalueSuspensionFrame::resume | 0xc4f5c0: 568 | 0xc5d680: 600 |
| debug | FinishEvaluatedPartSelectorsSuspensionFrame::resume | 0x963ca0: 200 | 0xc3e670: 200 |
| debug | FinishPublishedEvaluatedPartSelectorsSuspensionFrame::resume | 0x963d40: 200 | not emitted |
| debug | EvaluatedPartSelectorsContinuation::finish | 0x9633e0: 408 | not emitted |
| release | current_held_update_value | not emitted | not emitted |
| release | finish_held_update | not emitted | not emitted |
| release | suspend_held_part_selectors | not emitted | 0xc5a70: 120 |
| release | suspend_held_update | not emitted | 0xc5c60: 568 |
| release | try_set_positional_part_lvalue | 0x5a50b0: 664 | 0x5a5740: 520 |
| release | try_set_association_lvalue | 0x5d9490: 1016 | 0x5db070: 872 |
| release | resume_association_set_lvalue_selector_evaluation | not emitted | not emitted |
| release | resume_suspended_association_set_lvalue | 0x5db030: 552 | not emitted |
| release | suspend_association_set_lvalue_selectors | not emitted | 0xccdf0: 280 |
| release | suspend_suspended_association_set_lvalue | not emitted | 0xcd0e0: 120, 0xcd1a0: 24 |
| release | evaluate_part_selectors_owned | not emitted | not emitted |
| release | evaluate_part_selectors_owned_for_builtin | 0x61eea0: 344 | 0x61f7e0: 200 |
| release | resume_associate_to_after_target_preparation | 0x22a520: 568 | 0x22af00: 600 |
| release | resume_key_drop_from_after_target_preparation | 0x22ba30: 440 | 0x22c3f0: 456 |
| release | eval_held_association_part_mutation | 0x22d490: 488 | 0x22de40: 408 |
| release | suspend_positional_part_selectors | not emitted | 0xc6c60: 184 |
| release | suspend_association_set_part_selectors | not emitted | 0xcd210: 184 |
| release | FinishHeldAssociationPartSelectorEvaluationSuspensionFrame::resume | 0x228f60: 888 | 0x229690: 792 |
| release | EvalAssociateToSuspensionFrame::resume | 0x229370: 584 | 0x229b50: 600 |
| release | EvalKeyDropFromSuspensionFrame::resume | 0x2294f0: 584 | 0x229dd0: 600 |
| release | FinishHeldPartSelectorEvaluationSuspensionFrame::resume | 0x599c10: 808 | 0x59a650: 792 |
| release | FinishHeldUpdateSuspensionFrame::resume | 0x59a010: 1208 | 0x59aad0: 1288 |
| release | FinishPositionalPartSelectorEvaluationSuspensionFrame::resume | 0x5a4de0: 552 | 0x5a5390: 664 |
| release | FinishAssociationSetPartSelectorEvaluationSuspensionFrame::resume | 0x5d4560: 552 | 0x5d4a40: 680 |
| release | ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame::resume | 0x5d4830: 696 (alias) | 0x5d4e10: 1032 |
| release | ResumeSuspendedAssociationSetLvalueSuspensionFrame::resume | 0x5d4830: 696 | 0x5d5860: 1048 |
| release | FinishEvaluatedPartSelectorsSuspensionFrame::resume | 0x61f020: 264 | not emitted |
| release | FinishPublishedEvaluatedPartSelectorsSuspensionFrame::resume | 0x61f180: 280 | not emitted |
| release | EvaluatedPartSelectorsContinuation::finish | 0x61ea40: 88 | not emitted |

Individual-frame growth is **not cleared** by the unchanged recursion slopes. There are six debug function/closure groups and nine release function groups in the complete CLI comparison. All growing frames, including the independently built runtime probe, are partitioned at actual machine stack offsets in [stack/growth-slot-ledger.tsv](../../../.temp/brief-615/stack/growth-slot-ledger.tsv); [stack/growth-slot-ledger.json](../../../.temp/brief-615/stack/growth-slot-ledger.json) retains each instruction, call argument window, named DWARF slot, anonymous region and saved-register contribution. Every partition sums to its prologue. These machine regions are not invented source variables or assertions of disjoint variable lifetimes. DWARF location lists/dereferences may describe captured references or pointees; their type sizes are not added as resident stack. All witnesses and tokens have zero-byte types.

| Profile | Function / closure | Prologue growth | Base reserve + saved registers | Candidate reserve + saved registers |
| --- | --- | --- | --- | --- |
| debug | ResumeSuspendedAssociationSetLvalueSuspensionFrame::resume::{{closure}} | 160 | 0xb93a30: 696 + 0 | 0xc6a530: 856 + 0 |
| debug | resume_suspended_association_set_lvalue | 16 | 0xa2acd0: 1096 + 0 | 0xfa0bd0: 1112 + 0, 0xfa1480: 1096 + 0 |
| debug | FinishHeldPartSelectorEvaluationSuspensionFrame::resume | 144 | 0xf21f80: 472 + 0 | 0x1001690: 616 + 0 |
| debug | FinishPositionalPartSelectorEvaluationSuspensionFrame::resume | 16 | 0xd1c5d0: 472 + 0 | 0xd30850: 488 + 0 |
| debug | ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame::resume | 160 | 0xc4f430: 568 + 0 | 0xc5d4d0: 728 + 0 |
| debug | ResumeSuspendedAssociationSetLvalueSuspensionFrame::resume | 32 | 0xc4f5c0: 568 + 0 | 0xc5d680: 600 + 0 |
| release | resume_associate_to_after_target_preparation | 32 | 0x22a520: 520 + 48 | 0x22af00: 552 + 48 |
| release | resume_key_drop_from_after_target_preparation | 16 | 0x22ba30: 392 + 48 | 0x22c3f0: 408 + 48 |
| release | EvalAssociateToSuspensionFrame::resume | 16 | 0x229370: 536 + 48 | 0x229b50: 552 + 48 |
| release | EvalKeyDropFromSuspensionFrame::resume | 16 | 0x2294f0: 536 + 48 | 0x229dd0: 552 + 48 |
| release | FinishHeldUpdateSuspensionFrame::resume | 80 | 0x59a010: 1160 + 48 | 0x59aad0: 1240 + 48 |
| release | FinishPositionalPartSelectorEvaluationSuspensionFrame::resume | 112 | 0x5a4de0: 528 + 24 | 0x5a5390: 640 + 24 |
| release | FinishAssociationSetPartSelectorEvaluationSuspensionFrame::resume | 128 | 0x5d4560: 528 + 24 | 0x5d4a40: 648 + 32 |
| release | ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrame::resume | 336 | 0x5d4830: 648 + 48 | 0x5d4e10: 984 + 48 |
| release | ResumeSuspendedAssociationSetLvalueSuspensionFrame::resume | 352 | 0x5d4830: 648 + 48 | 0x5d5860: 1000 + 48 |

Debug slot examples: F0260’s resumed Result remains 136 bytes (rsp+128 → rsp+120); its zero-sized token is recorded at rsp+591 and the session pointer moves 440 → 576. F0281’s resumed Result remains 136 bytes at rsp+152; its zero-sized token is at rsp+711. F0282’s resumed Result changes 136 → 144 bytes at rsp+152 → rsp+160, and its closure residual changes 136 → 144 bytes at rsp+360 → rsp+512. The dual-route helper’s ordinary unit instantiation stays 1096 bytes; its token instantiation is 1112. These result/stop conversions and compiler temporary placement are recorded per slot, not attributed to a nonzero witness. Full debug slots: [stack/all-owned-dwarf-slots.tsv](../../../.temp/brief-615/stack/all-owned-dwarf-slots.tsv) and [stack/all-owned-runtime-dwarf-slots.tsv](../../../.temp/brief-615/stack/all-owned-runtime-dwarf-slots.tsv).

Release examples: the two mutator target frames reserve 536 → 552 bytes plus 48 saved-register bytes; their PreparedDefinitionLhs clone and outgoing argument each remain 176 bytes and their Result remains 136 bytes. For AssociateTo, the old regions are self/args [0,16), value [16,48), prepared [48,224), Result [224,360), outgoing prepared [360,536). The new layout is self/args [0,16), Result [16,152), padding [152,160), value [160,192), padding [192,200), prepared [200,376), outgoing prepared [376,552): the +16 is padding. The positional selector frame’s incoming 136-byte copy moves rsp+0xd0 → rsp+0x1e0; its finish result moves rsp+0x188 → rsp+0xd8, with additional completion staging at rsp+0x50. The association selector frame has analogous staging plus another saved register. F0261’s copied 136-byte region moves rsp+0x3e0 → rsp+0x430, with its high spills shifted by 80 bytes. F0281/F0282 now inline the formerly called 552-byte resume_suspended_association_set_lvalue body; this explains relocation of its accumulator/result regions into the parent, but is not a measured whole-path peak-stack clearance. Exact instruction bodies are retained in [stack/release-growth-bodies](../../../.temp/brief-615/stack/release-growth-bodies) and the complete slot ledger. All listed growth remains for the director’s stack-budget ruling; there is no claim that it fits only the typed macro’s named closure environment.

**Timing and alignment.** Timing was last: four unchanged full core-language-microbench panels in base/candidate/base/candidate order, 120 rows each, taskset CPU 2, profiling disabled, scale 1, two repetitions and three retained-module GC batches. The values are in-kernel monotonic AbsoluteTiming, not differenced wall time. All host monitors reported no competing CPU activity; GPU telemetry was unavailable. The full row comparison is [perf/timing-comparison.tsv](../../../.temp/brief-615/perf/timing-comparison.tsv). Fifty-eight labels had positive means and 22 had disjoint slower candidate ranges; these are indications, not acceptance gates. The known width32000 key read was -0.0522% by the full-panel means and was not a slowdown residual.

Both aligned arms were rebuilt with one codegen unit and RUSTFLAGS="-C llvm-args=-align-all-functions=6". Actual Cargo fingerprint records prove the flag; the original environment-summary logging omission is preserved and disclosed. All 7524 base and 7548 candidate rum function symbols are 64-byte aligned, including collect_upvalue_candidate_symbols_in_matching_order at base 0x5a2900 and candidate 0x5a3380. Proof: [alignment/function-alignment.json](../../../.temp/brief-615/alignment/function-alignment.json).

Each residual probe retains its original expression, input sizes, setup and result checks. The module-GC row retains the complete original three-batch block. Initial probes retain preceding setup; failed magnitudes are retried with the complete original program prefix. Warmups are fixed identically across arms (2–64 original calls, budgeted from the panel), and cancel from N/2N slopes. Each arm first has five timing preflights. Every magnitude must be within [0.75, 4/3] of its full-panel value before cycles are interpreted. Valid probes then have five interleaved N/2N cycles/instructions pairs in both normal and aligned builds, all pinned and monitored. Raw manifests, exact .rum sources, host samples and results remain in [alignment](../../../.temp/brief-615/alignment).


| Label | Full panel delta % | Magnitude valid | Normal cycles delta % | Aligned cycles delta % | Normal instruction delta % | Aligned instruction delta % | Interpretation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| association-part-key-read-width4000 | +0.303 | yes | +4.846 | -3.541 | +0.006 | +0.006 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| loop-empty | +3.812 | yes | +0.948 | -2.903 | +0.006 | +0.005 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| do-iterator-null | +8.855 | no | not interpreted | not interpreted | not interpreted | not interpreted | unresolved magnitude; stop-loss |
| table-count-null | +5.466 | no | not interpreted | not interpreted | not interpreted | not interpreted | unresolved magnitude; stop-loss |
| part-list | +1.770 | yes | -0.383 | +1.786 | -0.004 | +0.129 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| sameq-integer | +0.294 | yes | -1.890 | +0.762 | +0.010 | +0.025 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| sameq-list | +1.919 | yes | +6.744 | +0.145 | -0.169 | +0.028 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| d-nested-sin-small | +4.499 | yes | +2.154 | -1.322 | -0.000 | +0.012 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| downvalue-guarded-decline-fallback | +0.291 | yes | +2.932 | +1.070 | -0.046 | -0.004 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| default-unguarded-miss | +1.000 | yes | -0.836 | -3.446 | -0.577 | -0.520 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| default-guarded-hit | +1.323 | yes | -0.911 | -3.642 | +0.004 | +0.122 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| matcher-single-tiny | +1.127 | yes | -0.571 | -1.401 | +0.004 | -0.063 (outside bar) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| replaceall-simple | +1.927 | yes | +2.462 | +1.929 | -0.070 | +0.142 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| rewrite-guarded-hit | +1.078 | yes | +1.275 | -1.058 | -0.021 | -0.021 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| replacepart-guarded-decline-fallback | +0.461 | yes | +1.415 | -2.967 | -0.002 | +0.037 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| stringreplace-single-literal-unguarded | +2.332 | yes | +2.455 | +0.788 | -0.090 | +0.075 | The positive ordinary cycle delta is no longer resolved with both arms aligned. This supports alignment sensitivity; the measured intervals and instruction slopes are retained for the director, without an automatic acceptance or sole-layout claim. |
| stringreplace-guarded-hit | +1.453 | yes | +3.877 | +1.243 | -0.263 | +0.027 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| stringreplace-guarded-decline-fallback | +1.602 | yes | +0.232 | +1.832 | +0.026 | -0.004 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| sortby-callback-small | +1.500 | yes | -2.488 | -2.718 | +0.028 | +0.196 (outside bar) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| keysortby-callback-small | +2.794 | yes | +1.018 | +0.642 | -0.060 | +0.241 (outside bar) | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| sortby-default-wide | +0.302 | yes | +0.928 | -4.377 | +0.018 | +0.179 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |
| module-retained-gensym-gc-batch | +2.581 | yes | +0.543 | +0.852 | +0.043 | -0.028 | Full-panel slowdown is not reproduced as a positive resolved cycle delta by this exact-expression warmed probe; no causal layout claim. |

Twenty probes reproduce magnitude. Nineteen do not reproduce a resolved positive normal-build cycle delta using the unadjusted five-pair 95% interval. The literal StringReplace point estimate is +2.455% normally and +0.788% aligned, with the aligned interval including zero; this supports alignment sensitivity but does not establish a sole-layout explanation or automatic acceptance. Instruction residuals marked outside the native slope bar remain disclosed. All samples, paired intervals and attempt histories are in [alignment/comparison.json](../../../.temp/brief-615/alignment/comparison.json).

do-iterator-null and table-count-null remain **unresolved timing residuals**. Complete-prefix probes still missed magnitude. Two final follow-up attempts (single original call with scale assignment, then exact prefix with only loop literals varied and the marker moved before the prefix) cleared neither magnitude gate. Their cycle panels were therefore not run or interpreted. The brief’s two-attempt stop-loss ended this investigation; every failed magnitude remains in [alignment/probes-single-call/magnitude-preflights.json](../../../.temp/brief-615/alignment/probes-single-call/magnitude-preflights.json) and [alignment/probes-exact-prefix/magnitude-preflights.json](../../../.temp/brief-615/alignment/probes-exact-prefix/magnitude-preflights.json), alongside the earlier attempts. The do-iterator-null ordinary instruction/leaf gate still independently passes its six-process-per-arm Callgrind evidence; that does not clear its timing indication. No further measurement experiment followed the stop-loss.

### 6. Stop-loss, deviations and open questions

Driver-tracked top-level compile/build attempts: 74. Driver-tracked verification commands: 609. These counters count explicit orchestration entries; the compiler-control subprocess pairs and measurement processes are separately enumerated in the retained outputs. Consecutive source-conversion attempts with no movement on a named failing gate: **0**. The final two follow-up attempts to reproduce the tiny timing controls cleared neither remaining magnitude gate: **2**, triggering the brief’s stop-loss for that investigation. No further measurement experiment was run. No file is left half edited.

The initial conversion incorrectly bound F0265/F0280 completion to the outer `Option<Expr>` lvalue probe. Eighteen strict fixtures exposed that mistake. Binding the frame completion to `Expr` restored all 53 fixtures. The resulting wrapper-closure candidate then failed the ordinary-copy bar: three Callgrind pairs found one extra 136-byte copy per ordinary Part assignment. It was rejected; its binaries, profiles, source snapshots and attribution remain intact under `rejected-wrapper-candidate/`.

Three isolated experiments followed, each moving the named failing gates: moving the producer local removed the copy; named witness/request cold helpers reduced the remaining own-body instruction cost; direct error returns reduced it further. The selected four-file correction is the final source. For the positional Part probe, own-body instructions per assignment moved from 100 at S6 to 112 in the rejected wrapper candidate, then 111, 103 and 93 across the experiments; the added copy fell from one to zero. The explicit association Key Part probe likewise lost its extra copy, with a remaining instruction residual reserved for final-binary attribution. These isolated experiments are history, not substitutes for the final measurements in section 5. One unused import warning in an isolated cold-helper experiment was removed before applying the selected source.

The original 174-label measurement set was extended with one later explicit Key Part-assignment investigation. It receives the same final ordinary gates and three Callgrind pairs. The earlier harness corrections and the baseline F0281/F0282 compiler alias are disclosed in section 3. The fixed macro expansion, global slot modes, driver, core representations and Q5 semantics remain untouched.

No scope exception, public raw adapter, public route trait, new unknown host payload, changed retention mark, changed oracle check, dependency change, unsafe addition, network access or git operation was needed. The nine native ZeroMQ checks remain for the director's native environment. The pre-existing `OPTION_FRONTIER=927` console wording versus actual pin 950 is preserved and disclosed.

The association Key Part ordinary path retains a stable +5 own-body instructions per assignment, with exact address attribution. Individual stack frames grow as tabulated in section 5; all twelve recursion slopes nevertheless match S6 exactly. Twenty timing residuals have magnitude-validated normal/aligned comparisons; do-iterator-null and table-count-null remain unresolved because their probes did not reproduce the original magnitude. No cycle inference or acceptance is claimed for those two. These performance/stack residuals are submitted to the director, as this brief permits; they are not silently cleared by the correctness gates.

Two measurement-helper defects were corrected without changing product source: the aligned-build environment summary omitted RUSTFLAGS (actual Cargo fingerprints and emitted alignment prove it), and the initial timing-probe extractor lacked the retained-module GC block (the complete original block was added). Failed attempts remain in the command index. Early chunked probe logs printed a cumulative passing numerator against the current chunk size; final totals are recomputed from the full preflight records, not that cosmetic line. A single-call timing attempt inserted a scale assignment; the following exact-prefix attempt removed it and varied only the loop literals. Both failed the magnitude criterion and remain excluded.

The working tree is left uncommitted. Canonical gate outputs remain at their prescribed locations; all own probes and intermediate evidence stay under `.temp/brief-615/`.
