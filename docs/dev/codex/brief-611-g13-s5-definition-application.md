# brief-611 — G13 step S5: type definition application, defaults and fallback

## Goal

Type the third production family through the typed API. S5 owns the frames
that suspend while rum applies a user definition or stores one:

- **application:** DownValues, SubValues, UpValues, default values
  (`Default[…]`), the direct default fallback, and the right-hand side those
  applications evaluate;
- **storage:** `Set`, `UpSet`, `TagSet`, `Rule` with a suspending left side,
  list destructuring, and definition left-hand-side preparation.

When you are done:

- the 14 owned frames implement the typed API, and the 17 owned macro sites
  use the witnessed typed macro or the token, as their shape requires;
- **the skipped-`Some` class below is repaired at all 15 sites**, each
  behind a fixture that you watch fail on the S4 tree first;
- every other family still runs through the transport it uses today;
- the ordinary path of every program executes no more work than at S4, and
  the cost of S5's resume path is measured and attributed.

This is step S5 of G13 (design round 5, brief-606, ratified). S1 (brief-607)
built the dormant typed API, S2 (brief-608, `cb09bfde`) the driver that runs
typed and legacy entries side by side, S3 (brief-609, `1a228f4b`) evaluation
and dispatch, and S4 (brief-610, `24149a1a`) argument, rewrite-rule and
traversal resolution. S6-S54 type the remaining families, F1-F23 flip
transport per type, and R removes the legacy machinery.

**S5 sits on the hottest path in rum.** Every user function call applies a
DownValue and evaluates its right-hand side through these files, and every
guest recursion level runs through them. Treat every ordinary-path
instruction and every stack byte as owed.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- Authority, in order: this brief; brief-606 (its D1/D2 and report) with
  `.temp/brief-606/design.md` and `census-spec.md`; `.temp/brief-605/design.md`
  §§2-5 for the API, token, fault, unwind and accounting design, **except
  §5's transit record, which D2 rejected**.
- **S3 and S4 as built are your template.** Read brief-609 and brief-610 in
  full: their bodies, every director amendment and both director gate
  records. Their rulings are now the rules:
  - **Two arms on the typed macro.** `resume_typed_or_suspend!` is defined
    in `suspension/typed/witness.rs`.
    - Use the caller-place arm (`$result:ident`) wherever the producer's
      result is already bound to a local.
    - The expression arm is for a producer written inline.
  - **The macro expansion is fixed.** Do not change `witness.rs`.
    brief-610 built and measured three other expansions, and all three were
    worse. Its lazy cold closure costs a named, slot-attributed 16-48 B in
    some callers, and that is the accepted cost. A shape that needs a macro
    change is a STOP with the slot table.
  - **The witness and the token.** The witness comes from the actual
    producer `Result`. `ContinueToken::suspend(session, witness, request,
    fresh)` is the only insertion route inside a typed resume, and it
    carries no `Arc`. brief-610's D0002 (`arguments.rs`) is the newest
    example.
  - **The finalizer erasure.** A typed finalizer is erased as its own box,
    through the blanket `impl<F: SlotFinalizer> ErasedFinalizer for F`.
  - **The retention analyzer reads the typed macro** through one checked
    adapter, `retention_closure/typed_resume_macro.rs`.
- **Rows, as input; never edit them.** All of these are in `.temp/brief-606/`:
  - `all-frame-migrations.tsv`: F0237-F0239, F0244-F0246, F0257-F0259,
    F0262-F0264, F0270, F0271;
  - `macro-dataflow.tsv` and `macro-frame-ownership.tsv`: M0237-M0239,
    M0245-M0247, M0260-M0264, M0267-M0271, M0278;
  - `transformation-dispositions.tsv`: T0011-T0017, T0020, T0021;
  - `insertion-routes.tsv`: I0004;
  - `compile-controls.tsv`: `fallback-output`;
  - `witness-step-gates.tsv`: `default.rhs.twice`, `default.decline.twice`;
  - `microbench-step-labels.tsv`: the 32 S5 labels;
  - `draft-S5.md`: Codex's plan for this step.

  S5 has no chained inserts, deferred obligations, State births or
  historical divergences. The draft is raw material, and where it and this
  brief differ, this brief governs. In particular, its Scope, its new-file
  list (those files were built in S1-S4), its scratch path and its
  measurement section (timing does not decide) are superseded.
- **Every slot type S5 exchanges is already registered.**
  - `Expr` is `Value`.
  - `Option<Expr>` and `CompiledMatchOutcome` are Legacy, registered in
    `suspension/typed/legacy.rs`.

  Each keeps its single global mode.
- Current definitions, checked by the director at `24149a1a`. All paths are
  under `crates/rum-runtime/src/runtime/definitions/`:

  | Row | Type | Defined at |
  | --- | --- | --- |
  | F0237 | `EvaluateFirstExactDefaultValueFromCursorSuspensionFrame` | `application/default_values.rs:271` |
  | F0238 | `DirectDefaultFallbackFrame` | `application/down_values/default_fallback.rs:8` |
  | F0239 | `TryApplyDownValuesFromSymbolSuspensionFrame<Scan>` | `application/down_values.rs:457` |
  | F0244 | `EvaluateDefinitionRhsSuspensionFrame` | `application/rhs.rs:74` |
  | F0245 | `TryApplySubValuesFromSymbolSuspensionFrame` | `application/sub_values.rs:368` |
  | F0246 | `TryApplyUpValuesFromListSuspensionFrame` | `application/up_values.rs:406` |
  | F0257 | `DefinitionLhsPreparationSuspensionFrame` | `definition_lhs_preparation/resume_frames.rs:151` |
  | F0258 | `EvalUpSetSuspensionFrame` | `definition_tags.rs:254` |
  | F0259 | `EvalTagSetSuspensionFrame` | `definition_tags.rs:273` |
  | F0262 | `DestructuringSuspensionFrame` | `definitions/destructuring.rs:115` |
  | F0263 | `EvalSetSuspensionFrame` | `definitions/resume_frames.rs:5` |
  | F0264 | `EvalRuleSuspensionFrame` | `definitions/resume_frames.rs:26` |
  | F0270 | `RulePatternSuspensionFrame` | `rule_lhs/continuation.rs:5` |
  | F0271 | `RuleLhsCompletionFrame` | `rule_lhs/continuation.rs:41` |

- The 17 macro sites, at `24149a1a`:

  | Site | Location | Shape |
  | --- | --- | --- |
  | M0237 | `application/default_values.rs:198`, in cold `suspend_exact_default_value_scan` | Reached from the ordinary scan (through `suspend_exact_default_value_scan_from_owned`) and from the resumed helper `resume_suspended_exact_default_value_scan` |
  | M0238 | `application/down_values/default_fallback.rs:47`, cold `suspend` | I0004: the direct lookup's `Option` witness, returning `Result<Option<Expr>, _>` |
  | M0239 | `application/down_values.rs:383`, cold `suspend_down_value_application` | Reached from the ordinary scan, and again from `finish_suspended_down_value_application` when a resumed match declines and the scan continues |
  | M0245 | `application/rhs.rs:53`, cold `suspend_definition_rhs_evaluation` | Hand-matched producer `rhs_result` |
  | M0246 | `application/sub_values.rs:294`, cold `suspend_sub_value_application` | As M0239 |
  | M0247 | `application/up_values.rs:257`, cold `suspend_up_value_scan` | As M0237 (`_from_owned` plus the resumed scan) |
  | M0260-M0262 | `definition_lhs_preparation.rs:264`, `:325`, `:384` | Three inserts of F0257 |
  | M0263, M0264 | `definition_tags.rs:15`, `:58` | Result form (`let rhs = resume_or_suspend!(…, rhs_result, …)`) |
  | M0267 | `definitions/destructuring.rs:143`, cold `suspend_destructuring` | Hand-matched producer |
  | M0268-M0271 | `definitions.rs:42`, `:86`, `:103`, `:122` | Result form |
  | M0278 | `rule_lhs.rs:138`, cold `suspend_rule_lhs` | Hand-matched producer |

  Before converting, classify every site and report the table, using the
  three shapes brief-610 used plus the dual route. A site that a typed
  resume reaches and that re-inserts its own frame type continues through
  the token with fresh state, as M0250/D0002 did in S4. A site that is
  also reached from the ordinary path keeps the macro there. Report which
  route each call path takes.

### The skipped-`Some` class (a real defect, verified by the director)

Every definition application evaluates its right-hand side with
`evaluate_definition_rhs` (`application/rhs.rs:5`) and then returns
`.map(Some)`. If the right-hand side pauses, `rhs.rs:53` inserts F0244. The
`.map(Some)` at the caller runs only on the ordinary path. On resume, F0244
completes with a bare `Expr`, and the next frame out expects an
`Option<Expr>`. The legacy decoder (`suspension/primitive.rs:36-63`) then
decodes leniently:

- `System`InternalContinuationNone[]` becomes `None`;
- `System`InternalContinuationSome[x]` becomes `Some(x)`;
- anything else becomes `Some(it)`.

Guest code can write both heads. So an ordinary value round-trips by
accident, and those two do not. The director's probe
(`.temp/director/s5-probe/`, the resumed path against the queued-input path
on the same program, at `24149a1a`), with `N` standing for
`System`InternalContinuationNone[]`:

| Program | Resumed | Queued |
| --- | --- | --- |
| `f[x_] := (InputString["a"]; N); f[1]` | `TerminatedEvaluation["IterationLimit"]`, `General::passlim` re-entering `f[1]` | `InternalContinuationNone[]` |
| `f[x_] := (InputString["a"]; System`InternalContinuationSome[5]); f[1]` | `5` | `InternalContinuationSome[5]` |
| `f[1] := (InputString["a"]; N); {f[1]}` | `IterationLimit` | `{InternalContinuationNone[]}` |
| `f[x_ /; (InputString["c"]; True)] := (InputString["a"]; N); {f[1]}` | `IterationLimit` after 2 pauses | as above |
| `k[1][2] := …`, and `k[1][x_ /; …] := …` (SubValues) | `IterationLimit` | correct |
| `g /: h[g, 1] := …`, and `g /: h[x_ /; …, g] := …` (UpValues) | `IterationLimit` | correct |
| `Default[d] := (InputString["a"]; N); d[x_.] := {x}; d[]`, and `Default[e, 1]` | `IterationLimit` | correct |
| `f[x_] := (InputString["a"]; 7); f[1]`, and `Default[d] := (…; 3)` | equal | equal |

`None` makes the direct fallback (F0238) believe that no rule applied. It
re-enters the call, and the cycle guard ends the evaluation. Every storage
shape probed was equal: `Set`, `SetDelayed`, `UpSet`, `TagSet`, `Rule`,
`RuleDelayed`, destructuring, and a suspending left side.

The 15 sites are the whole class: every caller of `evaluate_definition_rhs`
except the OwnValue reader (`eval_core/symbol.rs:150`, which wants `Expr`
and is correct). Line numbers are the producer call:

- `application/default_values.rs:83`, `:94`, `:122`, `:141`, `:210`, `:229`
  (rows T0011-T0016);
- `application/down_values.rs:221`, `:322`, `:438`;
- `application/sub_values.rs:129`, `:230`, `:349`;
- `application/up_values.rs:103`, `:183`, `:299`.

The round-5 analysis listed only the six default-value sites. It could not
link the other nine `let result = …; … result.map(Some)` receivers to their
producer and marked them unresolved. This brief owns all 15.

**Ruling on the repair's shape: the map moves into the frame the pause
inserts.**

- The 15 application callers get a right-hand-side evaluation whose typed
  frame has `Output = Option<Expr>` and applies `Some` in its resume.
- The OwnValue caller keeps `Output = Expr`.
- One frame per pause, as today. The 15 call sites gain no macro and no
  extra frame, and their ordinary path does the same work.
- The `.map(Some)` moves into the helper and runs on both paths.
- Choose a second frame type or a type parameter on F0244, and disclose
  which. Both are `TypedFrame` with `Input = Expr`, derived from the actual
  producer.
- Profiling stays balanced. The caller's `record_profile_definition_rule_timing`
  is recorded with `false` at the pause, as it is today, and is not
  recorded again on resume.

If you find a reason this shape is wrong, STOP and report it with evidence.
Do not substitute an extra frame per pause.

## Scope

A listed file or class is permission to do this step's owned work there. It
is not permission to convert another step's rows, change evaluation beyond
the owned rows, or tidy. Disclose every file you edit.

**Classes.**

- **The family's source files.** These are, under
  `crates/rum-runtime/src/runtime/definitions/`:
  - `application/{default_values,down_values,rhs,sub_values,up_values,fast_match,mod}.rs`
    and `application/down_values/default_fallback.rs`;
  - `definition_lhs_preparation.rs` and `definition_lhs_preparation/{resume_frames,prepared_store,syntax_policy}.rs`;
  - `definition_tags.rs`;
  - `definitions.rs` and `definitions/{destructuring,resume_frames}.rs`;
  - `rule_lhs.rs` and `rule_lhs/{continuation,state}.rs`.
- **The OwnValue caller**, `eval_core/symbol.rs`. Only the call that selects
  F0244's `Expr` form may change, and only if the shape you choose renames
  it.
- **Module roots on an owned type's path.** Make only the visibility or
  re-export edit that the typed API needs, and disclose it.
- **Constructor ripple.** Edit only the construct that changed, in any file
  where an owned frame or helper is constructed or named.
- **`crates/rum-runtime/src/runtime/suspension/`**, only where S5's use
  exposes a gap in the API or driver. Disclose each edit and why it is
  needed. `typed/witness.rs` is excluded (see above). Sealing, opacity and
  the no-dual-arm rule stay intact.
- **Sibling unit tests.** A test that needs a family module's private items
  goes in that module's sibling `tests.rs`, declared as
  `#[cfg(test)] mod tests;`. Never use `#[path = "../…"]`: the ownership
  loader does not follow `..` and fails closed.
- **Census, oracle and controls.**
  - Census: everything under `crates/rum-stdlib/tests/typed_slot_census/`
    and `typed_slot_census.rs`.
  - Oracle: `crates/rum-stdlib/tests/suspension_oracle/fixtures/` for the
    new fixtures and the `typed_slots.rs` receipt, and
    `suspension_oracle/inventory/typed-slot-stages.tsv` and
    `typed-slot-shapes.tsv`.
  - Compile controls: `crates/rum-stdlib/tests/typed_slot_compile_controls.rs`.
    Add the `fallback-output` pair, for 32 in all. Also make its scratch
    path step-independent: it pins `.temp/brief-610/compile-controls` today.
    Use `.temp/typed-slot-compile-controls`, so later steps need no edit.
- **Pins that follow code.** A test or inventory row that names a touched
  file's path, line, function or count may be updated only to follow the
  same construct after it moves. Disclose each update as old → new. A row
  whose construct disappears by an owned conversion is a planned
  transition; list it. Anything else is a STOP. The starting points are:
  - `runtime/tests/frame_root_trace.rs` and
    `runtime/tests/frame_root_trace/derived_inventory.rs`;
  - `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv`, by
    discovery replay only;
  - `crates/rum-stdlib/src/tests/retention_closure/inventory/{obligations,local-contracts,promoted}.tsv`,
    by replay only;
  - `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs`, if a
    frontier row moves. Classify every diff as brief-609/610 did: a moved
    pair, an attributed S5 change, or a construction revealed or hidden by
    the typed macro. An unpaired removal is a STOP.

`Cargo.toml` files, `Cargo.lock`, the toolchain, docs, `AGENTS.md`,
`TODO.md` and the core `Expr`/`Symbol` representation stay unchanged.
Scratch goes under `.temp/brief-611/`. The certificate replay may write
`.temp/brief-546/certificates/`, and the compile controls
`.temp/typed-slot-compile-controls/`.

**Disclosure duty.** If a needed change falls outside every list and class,
make no edit there. STOP and report the file, the line and why.

## Steps

1. **Harness and bases first, before any source edit.**
   - Copy brief-610's resume harness (`.temp/brief-610/resume-harness/`) to
     `.temp/brief-611/resume-harness/`. Add workloads that pause in S5's
     frames. Base and candidate both run this one harness source.
   - Each workload repeats its pause N times and checks its final value.
     The requirements:
     - every owned frame that guest code can reach pauses in at least one
       workload, including a DownValue right-hand side, a guarded DownValue
       whose condition pauses before its right-hand side does, SubValues,
       UpValues, a default value, the direct default fallback, `Set`,
       `UpSet`, `TagSet`, `Rule` with a pausing left side, destructuring,
       and a pausing definition left side;
     - each workload's frame-call census (callgrind, as in brief-609 R6 and
       brief-610) shows the intended S5 frame resuming once per resume.
   - An owned frame that guest code cannot reach is reported with the
     reason.
   - Build the four measurement bases from the committed S4 tree
     (`24149a1a`) with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, each in its
     own target directory under `.temp/brief-611/`: `rum-cli`,
     `rum-cli-alloc` (`--features rum-cli/alloc-count`), `resume` and
     `resume-alloc`. Hash them, and record each base's source hashes.
2. **Labels.** Copy `.temp/brief-610/perf/labels.json` and its programs into
   `.temp/brief-611/perf/`.
   - Add the 32 S5 labels from `microbench-step-labels.tsv` that are not
     already there, in the same N/2N format. Confirm reachability by the
     owned function appearing in a base callgrind profile.
   - Each program checks its own result, so a label cannot time a failing
     path.
   - Add the new resume workloads as resume labels. Expect each resume line
     to say how many resumes ran: a workload may pause more than once per
     iteration.
3. **Fixtures that fail first.** Before converting anything, add the
   skipped-`Some` fixtures to the oracle and run them on the S4 tree. Each
   compares the queued-input path with the resumed path in value, log,
   messages, prints and trace under `Checks::ALL`, with unconditional
   `FaultProbe` absence.
   - Cover all 15 sites. For each, the right-hand side pauses and returns
     `System`InternalContinuationNone[]`, then
     `System`InternalContinuationSome[5]`, then an ordinary value.
   - Reach each site's own code path:
     - exact against pattern DownValues;
     - a DownValue whose condition pauses first, so the right-hand side runs
       from `finish_suspended_down_value_application`;
     - the same three for SubValues and for UpValues;
     - a default value through the first exact scan, through the cursor
       scan, and through the resumed scan after a pausing guard.
   - **Watch each one fail on the S4 tree** with the director's observation
     or an equivalent one, and record it.
   - A site no guest program can reach is reported with the reason. A
     fixture that passes on the S4 tree is reported, not deleted.
4. **Type the owned rows.**
   - Convert each F row to the typed API. Derive `Input` from the actual
     producer `Result` before any transformation, and bind `Output` at the
     innermost function or closure return.
   - The transport encodings in T0017, T0020, T0021 and the frames' other
     `into_continuation_expr` / `from_continuation_expr` calls disappear.
     Only identity transport goes; a semantic transform runs identically on
     both paths.
   - Keep, exactly: pending-generation checks, cursor advancement, owner
     drop order, held and evaluation order, and profiling balance.
   - Each M site follows its classified shape. Its ordinary path matches
     the producer `Result` once and constructs no frame, `Arc` or `Vec`.
     Frame state is built in the cold path. The existing hand-written cold
     helpers (`suspend_*`) may stay as the cold path if the typed route
     fits inside them. Move locals, not control flow.
   - The skipped-`Some` repair follows the ruling above.
   - The `fallback-output` control proves the default fallback's output
     type cannot float by declaration. Its positive case returns
     `Result<Option<Expr>, _>` from the actual `Option` witness. Its single
     invalid change returns `Result<Expr, _>` instead, and that must not
     compile.
5. **Ordinary controls.**
   - Rerun the 28 round-4 ordinary controls through brief-610's harness:
     copy `.temp/brief-610/ordinary-controls/` to `.temp/brief-611/`, and
     re-sync its copied `suspension_oracle/` support from the committed tree
     first.
   - All 28 pass, including the two this step touches directly:
     `default.rhs.twice` and `default.decline.twice`.
6. **Census.**
   - Advance the receipt to S5 in `typed_slot_census` and in
     `suspension_oracle/fixtures/typed_slots.rs`, and record every owned
     row's transition.
   - Production is 606 implementations and 588 macro sites at S4, with 22
     and 24 typed. Report the new totals.
   - Expect 36 typed implementations if no row splits or merges. A split
     of F0244 into two types is a planned transition: disclose it, with its
     effect on the totals.
7. **Inventories.** Replay fingerprints, both certificate sets and ownership
   discovery, and disclose every changed row. A new `UnknownHostPayload`
   without an approved disposition is a STOP, and so is a discovery failure.
   Never hand-write a ledger chain.

## Verification (MANDATORY — run to green or evidence)

Run everything serially, each command through `python3
scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-611/tmp …`. Bounds: workspace 3 600 s; oracle
1 500 s per build; everything else 900 s. **Your shell tool's own timeout
must exceed the command's bound**; a workspace run needs about 3 700 s.

```
cargo fmt --check
cargo test -p rum-runtime --lib suspension::typed::tests
cargo test -p rum-runtime --lib frame_root_trace
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

Also run the ordinary-controls harness in both builds.

Report:
- every `test result:` total;
- the oracle's fault-occurrence delta for each build;
- native ZeroMQ skips, as skips, not passes;
- any compiler warning the diff introduces. Neither oracle build may warn.

**Performance.** `TEMPLATE.md` pitfall 8 governs, with the bars brief-609
and brief-610 set.

1. **Allocations and bytes first**, on every label, against the bases.
   - Ordinary labels must be exactly equal, in slope and in the
     whole-program counts at N and 2N. A constant that cancels from a slope
     is still work: S3's finalizer rebox hid there.
   - On the resume workloads, a typed entry adds exactly one allocation,
     the outer adapter `Arc` (24 B), per typed frame per resume. Report
     which frame types pause per resume, before and after.
   - The skipped-`Some` repair adds no frame. Any further difference must
     be attributed, at its allocation sites, to a named type's mode wire.
     For example, `Option<Expr>`'s strict `Some` encoding, which brief-609
     recorded. Anything else is a defect.
2. **Instruction slopes.** Two interleaved `perf stat` pairs per arm on all
   labels: `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, stdin
   `/dev/null`, slope `(count(2N) - count(N)) / N`, raw output saved before
   parsing.
   - A label is flagged when its delta exceeds `max(0.05%, base range,
     candidate range)`.
   - A flagged ordinary label is cleared only in one of two ways:
     - five runs per arm;
     - six or more callgrind processes per arm, with a per-function diff in
       which every deterministic delta (identical in every process) is
       named at its addresses, and every process-variable delta is within
       its standard error of zero.

     brief-610's gate record shows why three processes are not enough when
     a residue sits in hash-seeded code such as the symbol interner.
3. **Copies and leaves.** On every ordinary label, the per-label `memcpy`
   call count (callgrind, every caller of the resolved libc symbol) is at
   most the base's. `literal-integer`, `do-count-null` and
   `do-iterator-null` execute no more instructions per iteration than the
   base, by address.
4. **Stack.** Measure bytes per level, as brief-610 did, in debug and
   release, base against candidate:
   - **S3's guest recursions, `peel` and `plus`.** They run through
     DownValue application and `evaluate_definition_rhs` on every level.
     brief-609's probe is in `.temp/brief-609/r6b/stack/`, and brief-610's
     method is in `.temp/brief-610/stack/`.
   - **Three S5 shapes:** a SubValue recursion (`k[1][n_] := k[1][n - 1] + 1`),
     an UpValue recursion (`g /: h[g, n_] := h[g, n - 1] + 1`) and a guarded
     DownValue recursion (`f[n_ /; n > 0] := f[n - 1] + 1`).

   Use four heights and three samples per shape, and report the residuals.
   - Tabulate the prologue frame size of every function S5 changes, in
     both profiles.
   - **Bar:** no growth beyond the typed macro's named closure environment,
     attributed per slot as in brief-610's redispatch 1.
   - **Any growth on `peel` or `plus`, in either profile, is reported with
     its slot table for the director to rule on.** Do not accept it
     yourself. These two are every guest call's depth.
   - The 48-level test must stay green.
5. **Resume attribution.** For every resume workload, run six callgrind
   processes per arm and diff every function per process, as brief-610's
   director gate did.
   - Name each deterministic delta at its addresses: typed frame and
     adapter bodies against legacy bodies, the typed queue, helper ABI,
     typed-outcome copies, the adapter `Arc`, and mode wire.
   - Report each process-variable delta with its standard error.
   - A copy or move of an action, slot, payload or error that the legacy
     arm did not make is reported with its addresses. The driver's extra
     136-B typed-outcome move per typed transfer is known and owed to R.
     Count it; do not repair it here.
6. **Timing** last, as an indication only. Run four full
   `scripts/core-language-microbench.sh` panels on the one-unit binaries,
   in the order base, candidate, base, candidate, under `taskset -c 2`, with
   profiling off and no other work on the host. A deterministic leaf
   outside [0.8, 1.25] is rerun five times, interleaved. Timing never
   overrides an instruction result.

A regression is a defect to fix or report.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- No default-path cost. No owned-tree clones on hot paths. S5 is the hottest
  path in rum.
- The witness comes from the actual `Result`. Never guess the input from a
  synthetic `Err` type.
- The token carries no `Arc`, is not `Clone`, and accepts fresh state of the
  same frame type only.
- Actions, codecs and modes stay sealed and opaque. Nothing public names a
  mode or constructs an erased entry.
- Profiling records stay balanced on every path, ordinary and resumed.
- Prefer the caller-place macro arm. An address-escaped local that is
  re-bound costs a copy on every ordinary pass.
- A generic frame (`TryApplyDownValuesFromSymbolSuspensionFrame<Scan>`)
  stays generic. Each instantiation the program uses is one type for the
  census; report them.
- New modules stay under 10k characters. Check
  `.temp/brief-606/file-size-assessments.tsv` before adding to a large file:
  `definition_lhs_preparation.rs` is 30k, `definitions.rs` 22k,
  `down_values.rs` 19k, `sub_values.rs` 16k and `up_values.rs` 16k. Split
  by responsibility, never by arbitrary cuts.

## Checkpointing — protect the scene

Everything goes under `.temp/brief-611/`. After each step, write
`PROGRESS.md` and a machine-readable `state.json`. Update `state.json` when
the work moves, not only at the end. Record the verified state, the compile
and test counters, any half-edited files and the exact next action. A quota
or usage-limit stop can end the session at any moment, and the next session
continues from these files.

## Hard rules

- **No `git`, in any form.** No network. Write nothing outside this
  repository except the certificate replay output named above.
- Modify only the Scope. No new `unsafe`; no dependency, lockfile or
  toolchain change.
- Commands allowed:
  - scoped reads and edits, `rg`/`grep`, bounded `python3`;
  - the bounded Cargo commands above and bounded compiler fixtures;
  - `scripts/check-suspension-oracle.sh` and
    `scripts/core-language-microbench.sh`;
  - `valgrind`, `callgrind_annotate`, `objdump`, `perf` and `taskset` on
    the measurement binaries.

  No probe that aborts a process.
- **The Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- No dual-arm decoder, per-frame mode override, transit record or public raw
  adapter. Type no row outside S5's. Never weaken an oracle `Checks` flag, a
  retention reject, a fault-absence gate or a documented test to get a pass.
- Stop-loss is keyed on progress. Compile errors you are reducing count as
  progress. Stop after two attempts that move none of the named failing
  gates.
- Paste real output. Say plainly what did not run or did not pass. Leave
  the tree uncommitted.

## Report format (append to this brief)

1. Files changed, each with its semantic change. List every ripple, module
   root and pin update as old → new.
2. Owned-row reconciliation:
   - each F/M row's transition and the site classification table;
   - the new or split frame names and the generic instantiations;
   - the removed transport maps;
   - the census output.
3. The skipped-`Some` repair:
   - the shape you chose;
   - each of the 15 sites' fixtures, with the failing observation on the S4
     tree and the pass after, in both builds.
4. The harness workloads and other oracle fixtures: what each exercises,
   its frame census, and its result on the base tree and the candidate in
   both builds.
5. All gates, verbatim. Fault-occurrence deltas. Inventory and certificate
   transitions.
6. Allocation, instruction, copy, stack, resume-attribution and timing
   evidence.
7. Stop-loss counters, deviations and open questions.


## S5 execution report — scope STOP, not accepted

The S5 implementation and skipped-Some repair are present. The final workspace fails one structural source audit whose required rule expansion exceeds the permitted pin-only scope. That audit remains unchanged. The instruction and native-stack defects below also remain unresolved. This report records completed work and names every missing gate.

### 1. Files changed and pin/ripple disclosures

- `crates/rum-runtime/src/runtime/suspension.rs` — Test-only re-export of the private real-adapter completion helper; constructor ripple for converted frame tests.
- `crates/rum-runtime/src/runtime/suspension/typed.rs` — Test-only helper enters the existing erased adapter and decodes its completed output; it does not mint a second token or expose a raw adapter.
- `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` — Named frame total 563 → 564 for the planned RHS split.
- `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` — Pattern walk specializes initial/token error routing; empty-stack suspension still propagates directly, and fresh state is inserted for later paused nodes.
- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs` — Three actual Expr producers use the macro initially and a fresh-state token on resumed scans. The head producer is witnessed before shared-head reuse; its old single-use transformed producer helper disappears. Explicitly drops the consumed head before both InputRequired and other errors, matching the removed helper's owner order.
- `crates/rum-runtime/src/runtime/definitions/definitions.rs` — Set, Rule, and RuleDelayed bound results use caller-place typed macros. The Set closure preserves the existing suspend timing record despite the macro return.
- `crates/rum-runtime/src/runtime/definitions/definition_tags.rs` — UpSet and TagSet bound RHS results use the caller-place typed macro; their frames are Expr-to-Expr.
- `crates/rum-runtime/src/runtime/definitions/application/mod.rs` — Family-private DefinitionResumeRoute specializes the initial EvalError route and the affine ContinueToken route. suspend_again witnesses the actual producer and inserts fresh same-type state. The initial name repeat was renamed after a concrete retention-checker collision.
- `crates/rum-runtime/src/runtime/definitions/application/default_values.rs` — All six application RHS callers select the Option output helper. The ordinary cursor uses the typed macro; resumed declines consume a token with fresh cursor state. Removes legacy output conversion.
- `crates/rum-runtime/src/runtime/definitions/application/down_values.rs` — All three RHS callers select the Option helper. Preserves generic Linear/IndexedDownValueScan and specializes initial/token drivers; default fallback passes its actual Option producer to the cold helper.
- `crates/rum-runtime/src/runtime/definitions/application/rhs.rs` — Splits application RHS completion from OwnValue completion: the new EvaluateDefinitionApplicationRhsSuspensionFrame returns Option<Expr>, while the old name retains Expr. Both complete through finish_definition_rhs_evaluation; application Some runs on both ordinary and resumed paths. Both cold helpers witness the actual RHS result and profiling stays balanced.
- `crates/rum-runtime/src/runtime/definitions/application/sub_values.rs` — All three RHS callers select the Option helper, including the guest-unreachable pending branch. Matcher declines preserve cursor/generation handling through initial/token specializations. Removes legacy output conversion.
- `crates/rum-runtime/src/runtime/definitions/application/up_values.rs` — All three RHS callers select the Option helper. Ordinary owned scan uses the macro; resumed shared scan consumes the token with fresh state. Removes legacy output conversion.
- `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/resume_frames.rs` — Typed Expr-to-Expr LHS preparation frame drives continuation through its token.
- `crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs` — Constructor ripple: abandon fixture uses the actual producer with the typed macro; error-resume fixture uses the private real-adapter helper.
- `crates/rum-runtime/src/runtime/definitions/rule_lhs/continuation.rs` — Pattern and Rule-LHS completion frames become typed Expr-to-Expr.
- `crates/rum-runtime/src/runtime/definitions/definitions/resume_frames.rs` — Set and Rule frames become typed Expr-to-Expr.
- `crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs` — Typed Expr-to-Expr destructuring preserves cursor order and uses the token for another paused leaf.
- `crates/rum-runtime/src/runtime/definitions/rule_lhs/tests/base_cases.rs` — Constructor ripple: direct driver call selects the initial () route.
- `crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs` — I0004 witnesses Result<Option<Expr>, EvalError> and fixes Output=Option<Expr>; removes explicit transport encode/decode.
- `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` — Adds the new application RHS frame to the root-trace and PureFrameAbandon lists.
- `crates/rum-runtime/src/runtime/suspension/typed/action.rs` — Adds runtime-private complete_work for an immediate typed completion or an existing ResumeStop; avoids a needless finalizer allocation.
- `crates/rum-runtime/src/runtime/suspension/typed/tests/composition.rs` — Checks complete_work success and Abort preservation.
- `crates/rum-stdlib/tests/typed_slot_census.rs` — Advances receipt/pins to S5 (607 implementations/589 macros, 37/42 typed); preserves historical S4 cohort checks and forced-red coverage. The shared token route follows the owned rename repeat → suspend_again.
- `crates/rum-stdlib/tests/typed_slot_compile_controls.rs` — Adds fallback-output positive/negative pair (31 → 32); scratch path brief-610/compile-controls → .temp/typed-slot-compile-controls.
- `crates/rum-stdlib/tests/typed_slot_census/reporting.rs` — Advances receipt/pins to S5 (607 implementations/589 macros, 37/42 typed); preserves historical S4 cohort checks and forced-red coverage. The shared token route follows the owned rename repeat → suspend_again.
- `crates/rum-stdlib/tests/typed_slot_census/routes.rs` — I0004 moves from the untyped-family list to an exact implemented/S5 receipt check; the other route ownership checks remain intact.
- `crates/rum-stdlib/tests/typed_slot_census/receipts.rs` — Advances receipt/pins to S5 (607 implementations/589 macros, 37/42 typed); preserves historical S4 cohort checks and forced-red coverage. The shared token route follows the owned rename repeat → suspend_again.
- `crates/rum-stdlib/tests/typed_slot_census/s4.rs` — Excludes precisely the S5 receipt cohort from the historical S4 set comparison; s5.rs independently checks the closed S5 set. Historical S4 frame/macro identities remain checked.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs` — Earned receipt S4 → S5.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` — Declares the two S5 oracle fixture modules.
- `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` — Records all 14 owned transitions, planned RHS split F0244A/M0245A, implemented I0004/S5, and 15 precise transformation/test-use removals.
- `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` — Replays all 68 exact contract fingerprints; all other columns unchanged. Old → new cells: contract-fingerprint-final-diff.tsv.
- `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` — Replays four emitted C01/C02 certificate cells (both hash configurations); seven other columns unchanged. certificate-diff.json records old → new.
- `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv` — Replays 31 anchor occurrences across nine unique moved constructs; all dispositions unchanged. Exact old → new rows: obligation-anchor-replay.json.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` — Discovery-only replay; exact changed rows are in ownership-replay.patch and host-payload-replay.json. No new host payload identity or disposition.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` — Discovery-only replay; exact changed rows are in ownership-replay.patch and host-payload-replay.json. No new host payload identity or disposition.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv` — Discovery-only replay; exact changed rows are in ownership-replay.patch and host-payload-replay.json. No new host payload identity or disposition.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` — Discovery-only replay; exact changed rows are in ownership-replay.patch and host-payload-replay.json. No new host payload identity or disposition.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` — Discovery-only replay; exact changed rows are in ownership-replay.patch and host-payload-replay.json. No new host payload identity or disposition.
- `crates/rum-stdlib/tests/typed_slot_census/s5.rs` — Closed S5 frame/macro/transform reconciliation and actual input/output/token evidence checks.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/definition_storage.rs` — New 19 storage, OwnValue, repeated-pause, and sentinel controls; all also pass on S4.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/definition_application.rs` — New 48 Checks::ALL oracle fixtures for the skipped-Some class, with unconditional fault absence; watched on the S4 tree first.

No changes to witness.rs, global slot modes, OwnValue caller, core values, manifests, lockfile, toolchain, AGENTS.md, or TODO.md. No unsafe was introduced. Full byte/hash scope audit is in current-source-diff.json; final-source.patch is a plain difflib review patch, generated without git. All new modules are below 10k characters. Existing large owned files remain explicit: LHS preparation 29,885 → 32,755; definitions.rs 21,905 → 22,169; down_values.rs 18,786 → 19,808; sub_values.rs 16,134 → 17,129; up_values.rs 15,905 → 16,462. default_values.rs moves 9,914 → 10,481 into the warning range. No arbitrary file split was introduced.

The final additional document edit is this appended report in `docs/dev/codex/brief-611-g13-s5-definition-application.md`; the brief body is preserved byte for byte. The stopped-source audit checks 1,482 entry files: 40 modified plus three declared new source/inventory files. The failing G4 audit file is byte-identical to entry. See [scope-stop-source-audit.json](../../../.temp/brief-611/scope-stop-source-audit.json). The full code/inventory review patch is [final-source.patch](../../../.temp/brief-611/final-source.patch).

Exact pin/replay transitions are retained in [ownership-replay.patch](../../../.temp/brief-611/ownership-replay.patch), [host-payload-replay.json](../../../.temp/brief-611/host-payload-replay.json), [ownership-drop-order-replay.patch](../../../.temp/brief-611/ownership-drop-order-replay.patch), [obligation-anchor-replay.json](../../../.temp/brief-611/obligation-anchor-replay.json), [obligation-drop-order-replay.json](../../../.temp/brief-611/obligation-drop-order-replay.json), [contract-fingerprint-final-diff.tsv](../../../.temp/brief-611/contract-fingerprint-final-diff.tsv), [certificate-diff.json](../../../.temp/brief-611/certificate-diff.json). These show every old → new cell, including the final three-line head-drop displacement; none is an invented ledger chain.

### 2. Owned-row reconciliation and census

| Row | Frame | Transition | Input → Output |
| --- | --- | --- | --- |
| F0237 | `EvaluateFirstExactDefaultValueFromCursorSuspensionFrame` | legacy → typed | `CompiledMatchOutcome → Option<Expr>` |
| F0238 | `DirectDefaultFallbackFrame` | legacy → typed | `Option<Expr> → Option<Expr>` |
| F0239 | `TryApplyDownValuesFromSymbolSuspensionFrame <Scan>` | legacy → typed | `CompiledMatchOutcome → Option<Expr>` |
| F0244 | `EvaluateDefinitionRhsSuspensionFrame` | legacy → typed | `Expr → Expr` |
| F0245 | `TryApplySubValuesFromSymbolSuspensionFrame` | legacy → typed | `CompiledMatchOutcome → Option<Expr>` |
| F0246 | `TryApplyUpValuesFromListSuspensionFrame` | legacy → typed | `CompiledMatchOutcome → Option<Expr>` |
| F0257 | `DefinitionLhsPreparationSuspensionFrame` | legacy → typed | `Expr → Expr` |
| F0258 | `EvalUpSetSuspensionFrame` | legacy → typed | `Expr → Expr` |
| F0259 | `EvalTagSetSuspensionFrame` | legacy → typed | `Expr → Expr` |
| F0262 | `DestructuringSuspensionFrame` | legacy → typed | `Expr → Expr` |
| F0263 | `EvalSetSuspensionFrame` | legacy → typed | `Expr → Expr` |
| F0264 | `EvalRuleSuspensionFrame` | legacy → typed | `Expr → Expr` |
| F0270 | `RulePatternSuspensionFrame` | legacy → typed | `Expr → Expr` |
| F0271 | `RuleLhsCompletionFrame` | legacy → typed | `Expr → Expr` |
| F0244A | `EvaluateDefinitionApplicationRhsSuspensionFrame` | new split of F0244 | `Expr → Option<Expr>` |

| Site | Shape | Input | Output | Routes |
| --- | --- | --- | --- | --- |
| M0237 | dual route | CompiledMatchOutcome | Option<Expr> | ordinary cursor -> typed macro; resumed exact scan -> token with advanced state |
| M0238 | ordinary hand match / I0004 | Option<Expr> | Option<Expr> | direct lookup witness before match -> cold fallback helper returning Result<Option<Expr>, EvalError> |
| M0239 | dual route | CompiledMatchOutcome | Option<Expr> | ordinary DownValue scan -> typed macro; resumed decline re-enters scan -> same-Scan token with new application state |
| M0245 | ordinary hand match | Expr | Expr or Option<Expr> | RHS producer -> typed macro; split own-value and application completion, one frame per pause |
| M0246 | dual route | CompiledMatchOutcome | Option<Expr> | ordinary SubValue scan -> typed macro; resumed decline re-enters scan -> token with new application state |
| M0247 | dual route | CompiledMatchOutcome | Option<Expr> | ordinary owned scan -> typed macro; resumed shared scan -> token with advanced state |
| M0260 | dual route | Expr | Expr | ordinary active pattern atom -> typed macro; resumed LHS driver -> token |
| M0261 | dual route | Expr (before reuse_definition_lhs_call_head) | Expr | witness raw evaluate_definition_lhs_active_expr; ordinary head -> typed macro; resumed LHS driver -> token; shared-head reuse remains in each success path |
| M0262 | dual route | Expr | Expr | ordinary call argument -> typed macro; resumed LHS driver -> token |
| M0263 | bound result | Expr | Expr | caller-place typed macro on rhs_result; resumed UpSet continues storage |
| M0264 | bound result | Expr | Expr | caller-place typed macro on rhs_result; resumed TagSet continues storage |
| M0267 | dual route | Expr | Expr | ordinary leaf store -> typed macro; resumed destructuring suffix -> token with advanced cursor |
| M0268 | bound result | Expr | Expr | caller-place typed macro on rhs_result; resumed Set continues storage |
| M0269 | bound result | Expr | Expr | caller-place typed macro on lhs_result; resumed Rule completion may enter distinct F0264 |
| M0270 | bound result | Expr | Expr | caller-place typed macro on lhs_result; resumed RuleDelayed constructs held RHS |
| M0271 | bound result | Expr | Expr | caller-place typed macro on rhs_result; reached from ordinary Rule and distinct F0271 resume, inserts F0264 |
| M0278 | dual route | Expr | Expr | ordinary pattern-LHS evaluation -> typed macro; resumed pattern walk -> token with fresh state; empty stack still propagates directly |

No site is exclusively resumed; nine sites have both routes. The six bound-result sites use the caller-place arm. SharedExpr is not registered: M0261 witnesses the underlying Expr result before its semantic reuse helper, matching the existing resumed success path.

Production: 606 → 607 implementations; 588 → 589 macro sites; 22 → 37 typed implementations; 24 → 42 typed sites. Test-only population remains 16 implementations / 11 macro sites. F0244A and M0245A are the planned RHS split. The generic DownValue frame stays generic; both LinearDownValueScan and IndexedDownValueScan run in the harness.

Removed maps: T0011–T0016 move application Some into the RHS helper/frame; T0017/T0020/T0021 remove identity wire maps. Whole0009/0010/0016/0017 and Whole0011 remove the now-typed decoders; X0002 follows the converted test insertion. Exact receipts remain in typed-slot-stages.tsv.

The census counts a generic implementation block once. Its two concrete DownValue scan instantiations are `LinearDownValueScan` and `IndexedDownValueScan`; both are used by the harness. This is one source F row and two concrete types, not a claim that a generic frame is a single runtime instantiation. M0245 keeps the OwnValue helper and the new M0245A is the application RHS helper. Expr remains Value; Option<Expr> and CompiledMatchOutcome remain Legacy. No decoder accepts both modes.

Verbatim census/transition output (the later forced-red child runs are separately included with the gate totals):

```text
S5 CENSUS production frames=518 finalizers=52 implementations=607 typed=37 macros=589 result=368 insert=221
S5 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S5 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=6 insert=4
S5 PARTITION implementations=623 macros=600
S4 TRANSITION F0248 removed; successor/provider=F0247
S4 TRANSITION M0250 removed; successor/provider=F0247
S4 TRANSITION Nested0005 removed; successor/provider=D122
S4 TRANSITION Nested0006 removed; successor/provider=D122
S4 TRANSITION Whole0018 removed; successor/provider=D122
S4 TRANSITION Whole0019 removed; successor/provider=D122
S4 TRANSITION Whole0020 removed; successor/provider=D128
S4 TRANSITION Whole0021 removed; successor/provider=D128
S4 TRANSITION Whole0022 removed; successor/provider=D126
S4 TRANSITION Whole0023 removed; successor/provider=D126
S4 TRANSITION Whole0024 removed; successor/provider=D126
S4 FRAME F0204 BuiltinEvaluateSuspensionFrame Expr -> Expr
S4 FRAME F0247 ResolveArgumentExprsFromCursorSuspensionFrame Expr -> Vec<Expr>
S4 FRAME F0249 ResolveCasesSpecExprForMatchingOwnedSuspensionFrame Expr -> Expr
S4 FRAME F0250 ResolveRewriteRuleExprsOwnedSuspensionFrame Expr -> Option<Vec<Expr>>
S4 FRAME F0251 DriveRewriteRuleExprsOwnedSuspensionFrame Option<Vec<Expr>> -> Expr
S4 FRAME F0252 DriveRewriteRuleInfosOwnedSuspensionFrame Option<Vec<Expr>> -> Option<Vec<RewriteRuleInfo>>
S4 FRAME F0253 DriveRewriteRulesOwnedSuspensionFrame Expr -> Expr
S4 FRAME F0254 ResolveCasesTraversalOwnedSuspensionFrame Vec<Expr> -> CasesTraversalResolution
S4 FRAME F0255 ResolvePositionTraversalOwnedSuspensionFrame Vec<Expr> -> CasesTraversalResolution
S4 FRAME F0256 ResolveTraversalWithoutLimitOwnedSuspensionFrame Vec<Expr> -> TraversalWithoutLimitResolution
S4 FRAME F0272 PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrame Expr -> PreparedCallArguments
S4 FRAME F0274 EvaluateCallSuspensionFrame Expr -> Expr
S4 FRAME F0275 ResumeCallWithDispatchPlanSuspensionFrame Option<Expr> -> Expr
S4 FRAME F0276 ResumeSuspendedCallWithDispatchPlanSuspensionFrame Option<Expr> -> Expr
S4 FRAME F0275P PreparedDispatchFrame PreparedCallArguments -> Expr
S4 FRAME F0277 EvaluateStoredOwnValueSuspensionFrame CompiledMatchOutcome -> Expr
S4 FRAME F0278 RunCallDispatchStageSuspensionFrame Expr -> Option<Expr>
S4 FRAME F0279 EvalReleaseHoldSuspensionFrame Expr -> Expr
S4 FRAME F0526 EvaluateProfiledFixedPointSuspensionFrame Expr -> Expr
S4 FRAME F0527 ResumeFixedPointFromSuspensionFrame Expr -> Expr
S4 FRAME F0568 FinishArgumentResolutionFinalizer  -> Vec<Expr>
S4 FRAME F0570 FinishPreparedCallArgumentsFinalizer  -> PreparedCallArguments
S4 D0002 witnessed token live; typed=22 macros=24
S5 TRANSITION T0011 removed; successor/provider=F0244A
S5 TRANSITION T0012 removed; successor/provider=F0244A
S5 TRANSITION T0013 removed; successor/provider=F0244A
S5 TRANSITION T0014 removed; successor/provider=F0244A
S5 TRANSITION T0015 removed; successor/provider=F0244A
S5 TRANSITION T0016 removed; successor/provider=F0244A
S5 TRANSITION T0017 removed; successor/provider=F0237
S5 TRANSITION T0020 removed; successor/provider=F0245
S5 TRANSITION T0021 removed; successor/provider=F0246
S5 TRANSITION Whole0009 removed; successor/provider=D049
S5 TRANSITION Whole0010 removed; successor/provider=D049
S5 TRANSITION Whole0011 removed; successor/provider=D125
S5 TRANSITION Whole0016 removed; successor/provider=D049
S5 TRANSITION Whole0017 removed; successor/provider=D049
S5 TRANSITION X0002 removed; successor/provider=F0270
S5 FRAME F0237 EvaluateFirstExactDefaultValueFromCursorSuspensionFrame CompiledMatchOutcome -> Option<Expr>
S5 FRAME F0238 DirectDefaultFallbackFrame Option<Expr> -> Option<Expr>
S5 FRAME F0239 TryApplyDownValuesFromSymbolSuspensionFrame CompiledMatchOutcome -> Option<Expr>
S5 FRAME F0244 EvaluateDefinitionRhsSuspensionFrame Expr -> Expr
S5 FRAME F0244A EvaluateDefinitionApplicationRhsSuspensionFrame Expr -> Option<Expr>
S5 FRAME F0245 TryApplySubValuesFromSymbolSuspensionFrame CompiledMatchOutcome -> Option<Expr>
S5 FRAME F0246 TryApplyUpValuesFromListSuspensionFrame CompiledMatchOutcome -> Option<Expr>
S5 FRAME F0257 DefinitionLhsPreparationSuspensionFrame Expr -> Expr
S5 FRAME F0258 EvalUpSetSuspensionFrame Expr -> Expr
S5 FRAME F0259 EvalTagSetSuspensionFrame Expr -> Expr
S5 FRAME F0262 DestructuringSuspensionFrame Expr -> Expr
S5 FRAME F0263 EvalSetSuspensionFrame Expr -> Expr
S5 FRAME F0264 EvalRuleSuspensionFrame Expr -> Expr
S5 FRAME F0270 RulePatternSuspensionFrame Expr -> Expr
S5 FRAME F0271 RuleLhsCompletionFrame Expr -> Expr
S5 SPLIT F0244 Expr / F0244A Option<Expr>; one frame per pause; generic F0239 retains Scan
S5 TOTAL typed=37 typed_macros=42; modes unchanged; preflip remains refused
```

### 3. Skipped-Some repair and fail-first fixtures

Application RHS owns Some on both completion paths; OwnValue retains Expr. This creates one frame per pause. All 15 callers are repaired. sub_values.rs:129 is guest-unreachable: both callers pass pending=None, the loop only takes pending, and successful matches enter the finish helper. The exact/pattern SubValue fixtures reach line230; exact/pattern UpValue fixtures both reach line183. This exception is reported, not counted as covered.

Both S4 oracle configurations: 16 ordinary fixtures pass, 32 sentinel fixtures fail. None produces UnexpectedPause(1), or UnexpectedPause(2) after a paused guard; Some produces a Value mismatch. Both candidate focused configurations: 67 passed (48 application + 19 storage controls), zero failures and zero fault occurrences. Per-case evidence: skipped-some-before-after.json and the four raw baseline/candidate logs.


| Original → final caller | Fixture family | S4 None / Some / ordinary, both builds | Candidate, both builds |
| --- | --- | --- | --- |
| `application/down_values.rs:221` → `228` | `down_exact` | UnexpectedPause(1) / Value / pass | all three pass |
| `application/down_values.rs:322` → `331` | `down_pattern` | UnexpectedPause(1) / Value / pass | all three pass |
| `application/down_values.rs:438` → `470` | `down_resumed` | UnexpectedPause(2) / Value / pass | all three pass |
| `application/sub_values.rs:230` → `234` | `sub_exact` | UnexpectedPause(1) / Value / pass | all three pass |
| `application/sub_values.rs:230` → `234` | `sub_pattern` | UnexpectedPause(1) / Value / pass | all three pass |
| `application/sub_values.rs:349` → `376` | `sub_resumed` | UnexpectedPause(2) / Value / pass | all three pass |
| `application/up_values.rs:183` → `186` | `up_exact` | UnexpectedPause(1) / Value / pass | all three pass |
| `application/up_values.rs:183` → `186` | `up_pattern` | UnexpectedPause(1) / Value / pass | all three pass |
| `application/up_values.rs:103` → `106` | `up_sync_guard` | UnexpectedPause(1) / Value / pass | all three pass |
| `application/up_values.rs:299` → `305` | `up_resumed` | UnexpectedPause(2) / Value / pass | all three pass |
| `application/default_values.rs:83` → `86` | `default_first_plain` | UnexpectedPause(1) / Value / pass | all three pass |
| `application/default_values.rs:94` → `96` | `default_first_guarded_table` | UnexpectedPause(1) / Value / pass | all three pass |
| `application/default_values.rs:122` → `123` | `default_cursor_match` | UnexpectedPause(1) / Value / pass | all three pass |
| `application/default_values.rs:141` → `141` | `default_cursor_next` | UnexpectedPause(1) / Value / pass | all three pass |
| `application/default_values.rs:210` → `213` | `default_resumed_match` | UnexpectedPause(2) / Value / pass | all three pass |
| `application/default_values.rs:229` → `233` | `default_resumed_next` | UnexpectedPause(2) / Value / pass | all three pass |
| `application/sub_values.rs:129` → `133` | guest-unreachable pending branch | not run; reason above | helper selection repaired; no coverage claim |

Every fixture uses Checks::ALL, checks value/log/messages/prints/trace, and makes fault absence unconditional. The ordinary cases that already passed on S4 were retained. The final complete oracle runs include every candidate fixture after the explicit head-drop correction; the earlier 67-test focused receipts are also retained. Exact case names and both-build baseline/candidate observations: [skipped-some-before-after.json](../../../.temp/brief-611/skipped-some-before-after.json); guest programs and caller mapping: [skipped-some-cases.json](../../../.temp/brief-611/skipped-some-cases.json); unreachable branch proof: [unreachable-sites.json](../../../.temp/brief-611/unreachable-sites.json).

### 4. Harness workloads and other oracle fixtures

One frozen harness source serves the four S4 measurement bases and the final candidate binaries. All 32 default-build workload checks pass on both arms in the measured N/2N allocation and instruction runs. The 19 new workloads cover all 14 original owned frames; their S4 reachability census is below. Each frame is once per relevant resume, while guard/RHS workloads have two actual pauses per iteration. The distinction is preserved in both columns. Candidate frame-call attribution and the separate no-default-feature harness runs did **not** run before the scope STOP. No claim is made that the default-build checks cover that missing configuration.

The inherited workloads are `table`, `do-chain`, `call-repeat`, `cleanup`, `fold`, `argument-twice`, `rewrite-exprs`, `rewrite-compiled`, `cases-spec`, `cases-traversal`, `position-traversal`, `count-traversal`, `cases-shared`. Their source is unchanged from the copied S4 harness.

| New workload | Actual resumes/iteration | S4 owned frame calls: per iteration / per actual resume |
| --- | ---: | --- |
| `s5-down-rhs` | 1 | F0244: 1 / 1 |
| `s5-down-guard` | 2 | F0239: 1 / 0.5; F0244: 1 / 0.5 |
| `s5-sub-rhs` | 1 | F0244: 1 / 1 |
| `s5-sub-guard` | 2 | F0244: 1 / 0.5; F0245: 1 / 0.5 |
| `s5-up-rhs` | 1 | F0244: 1 / 1 |
| `s5-up-guard` | 2 | F0244: 1 / 0.5; F0246: 1 / 0.5 |
| `s5-default-rhs` | 1 | F0239: 1 / 1; F0244: 1 / 1 |
| `s5-default-guard` | 2 | F0237: 1 / 0.5; F0238: 1 / 0.5; F0244: 1 / 0.5 |
| `s5-default-fallback-rhs` | 1 | F0238: 1 / 1; F0244: 1 / 1 |
| `s5-set` | 1 | F0263: 1 / 1 |
| `s5-upset` | 1 | F0258: 1 / 1 |
| `s5-tagset` | 1 | F0259: 1 / 1 |
| `s5-rule-rhs` | 1 | F0264: 1 / 1 |
| `s5-rule-lhs` | 1 | F0271: 1 / 1; F0270: 1 / 1 |
| `s5-destructure` | 1 | F0262: 1 / 1 |
| `s5-lhs-twice` | 2 | F0257: 2 / 1 |
| `s5-own-rhs` | 1 | F0244: 1 / 1 |
| `s5-destructure-twice` | 2 | F0262: 2 / 1 |
| `s5-rule-lhs-twice` | 2 | F0271: 1 / 0.5; F0270: 2 / 1 |

The new workloads return 7, True, List[7], or List[7, 8] as specified in [resume-harness/src/s5.rs](../../../.temp/brief-611/resume-harness/src/s5.rs); the harness checks every final value and reports its actual resume count. The raw S4 census is [harness-frame-census-base-final/results.json](../../../.temp/brief-611/harness-frame-census-base-final/results.json). The indexed DownValue guard workload installs 32 exact neighbors; the default workload reaches the linear scan. The table includes OwnValue separately, which retains the original Expr RHS frame. Candidate application RHS pauses select F0244A by construction, with no extra frame per pause; the required measured candidate census remains unrun.

The initial 19 new performance labels used N=1,000. The first S4 2N run failed after 1,021 resumes because of the existing RecursionLimit=1,024 (confirmed against the frozen S4 host rlib). Only that first N=2,000 new-label case was attempted. All 19 new labels were changed uniformly to N/2N=100/200; no source, binary, check, or limit changed. All 38 corrected S4 new-label cells pass, and all final candidate cells pass. The failed sample and 240 earlier valid inherited/ordinary cells remain preserved. Provenance: [perf/resume-size-correction.json](../../../.temp/brief-611/perf/resume-size-correction.json). Callgrind reachability used its separate small N/2N census, not a timing inference.

There are 107 ordinary labels (77 inherited plus 30 missing S5 labels, covering all 32 S5 labels when the two inherited overlaps are counted), and 32 resume labels: 139 total. Every ordinary program checks its own result. Base profiles confirm the intended owned function is present; on some neighboring labels it occurs during setup rather than the timed statement, which is a reachability limitation, not evidence of per-iteration cost. See [base-reachability.log](../../../.temp/brief-611/base-reachability.log) and [base-label-check.log](../../../.temp/brief-611/base-label-check.log).

All 19 additional storage/OwnValue/repeated-pause oracle fixtures pass on S4 in both feature configurations and on the candidate in both configurations. The 48 application fixtures have the baseline failures described in section 3. The 28 ordinary controls pass in each candidate build, including default.rhs.twice and default.decline.twice; each uses Checks::ALL and unconditional fault absence. The complete oracle passes 133 tests in each build.

### 5. Gates, fault records, inventories and certificates

The final workspace exits 101. Its only failing target is rum-stdlib --lib: 1,104 passed, one failed, 53 ignored. The failed test is `tests::association_depth_contract::rule_pattern_depth_contract::structural_rule_pattern_depth_contract`. It detects these eight new outbound edges:

```text
unaudited G4 outbound calls: {"RuleLhsCompletionFrame::resume -> crate::runtime::kernel_api::TypedAction::complete", "RulePatternSuspensionFrame::resume -> ResumeStop::failure", "RulePatternSuspensionFrame::resume -> TypedAction::complete_work", "Session::drive_rule_lhs -> Route::failure", "Session::suspend_rule_lhs -> <indirect-call>", "Session::suspend_rule_lhs -> map_err", "Session::suspend_rule_lhs -> resume_typed_or_suspend!", "Session::suspend_rule_lhs -> suspend_again"}
```

Ownership discovery remains exact: 79 normalized host payload identities and 59 cycles, with 7,708 → 7,715 constructs. Initial replay pairs 90 moved rows, removes 22 owned constructs (17 legacy macro uses, four wire closures, one equivalent direct-result match), and adds 29 (18 typed macro uses, the new RHS derive, two route-unreachable constructs, eight cold/witness closures). The last head-drop edit moves 12 exact LHS rows without changing identities or dispositions. The two host-payload coordinate moves and the computed typed-adapter ledger witnesses are recorded; no new UnknownHostPayload was admitted.

The obligation replay first moves 31 anchor occurrences across nine unique constructs; the final drop-order correction advances ten occurrences by three lines (573 → 576, 598 → 601, 605 → 608). All dispositions stay fixed. All 68 local-contract fingerprint cells were recomputed; their other 18 columns remain unchanged. Final context fingerprint: fnv1a64:982ba42ade98c054. Both configurations emitted and replayed the four C01/C02 certificate cells; their other seven columns remain unchanged. Final local_production_acceptance passes in both builds. The frontier inventory remains byte-identical to S4, 937 rows, so no frontier pin changed. Final raw production-rejection census: 48,510 edges and 7,345 roots; the explicit head drop explains the last +1 edge from the earlier 48,509 run.

The first retention replay exposed the family trait's new `repeat` method as a competing string-repeat candidate inside the existing compiler probe. A scratch diagnosis identified that exact collision. Renaming only the S5 method to `suspend_again` restored all four checked probe marks. Subsequent emissions failed only at the expected stale certificate check, then both fresh certificate acceptances passed. No retention checker or reject condition changed.

Exact replay files: [ownership-classification-initial.json](../../../.temp/brief-611/ownership-classification-initial.json), [ownership-replay.patch](../../../.temp/brief-611/ownership-replay.patch), [host-payload-replay.json](../../../.temp/brief-611/host-payload-replay.json), [ownership-drop-order-replay.json](../../../.temp/brief-611/ownership-drop-order-replay.json), [obligation-anchor-replay.json](../../../.temp/brief-611/obligation-anchor-replay.json), [obligation-drop-order-replay.json](../../../.temp/brief-611/obligation-drop-order-replay.json), [contract-fingerprint-final-diff.tsv](../../../.temp/brief-611/contract-fingerprint-final-diff.tsv), [certificate-diff.json](../../../.temp/brief-611/certificate-diff.json), [certificate-replay.patch](../../../.temp/brief-611/certificate-replay.patch).

Both final oracle builds report exactly:

```text
final-oracle-default
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.56s
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88916
final-oracle-no-default
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.43s
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88916
```

Each has 84 fault-absence records, all delta=0, with the final counter at 88,916 observations. Both are warning-free. The focused runtime/workspace build reports three preexisting unused test helpers (`register_test_if_builtin`, `register_test_return_builtin`, `register_test_vm_template_builtins`); the frozen S4 native-build diagnostics contain the same warnings. Nested compiler-oracle fixtures deliberately emit their own diagnostics/warnings. No new production compiler warning was identified; those fixture diagnostics are not counted as oracle-build warnings.

The following nine native ZeroMQ cases **skipped**, even though Rust counts their early returns inside the enclosing 151-passed total. They are not exercised passes:

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

All commands use the brief environment (one Cargo build job, one Rust test thread, Cargo offline, TMPDIR under brief-611) and process-group bounds. The native test skip flag was explicit. Final mandatory gate commands, exits, and **every** test-result total are copied verbatim below. The failed 0-passed/1-failed census children are intentional red controls; their enclosing census gates pass. The stdlib 1,104/1/53 failure is the real unresolved failure.

```text
$ cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.84s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.93s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 144.79s

$ cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 20.34s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.89s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 146.29s

$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 19.01s

$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 19.55s

$ cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.50s

$ cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.91s

$ cargo fmt --check
exit=0


$ cargo test -p rum-runtime --lib frame_root_trace
exit=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1158 filtered out; finished in 1.29s

$ cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.01s

$ cargo test -p rum-stdlib --test suspension_oracle -- --nocapture
exit=0
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.56s

$ cargo test -p rum-stdlib --no-default-features --test suspension_oracle -- --nocapture
exit=0
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.43s

$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/ordinary-controls/Cargo.toml --test ordinary -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

$ cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
exit=0
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 73.51s

$ cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 25.98s

$ cargo test -p rum-runtime --lib suspension::typed::tests
exit=0
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.47s

$ cargo test --workspace --no-fail-fast -- --nocapture
exit=101
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.92s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 1170 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 42.75s
test result: FAILED. 1104 passed; 1 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1307.99s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.10s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.63s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.95s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.98s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 146.15s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.65s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.58s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.29s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.58s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Earlier fixture/iteration/replay test totals, also verbatim (kept rather than replaced by later passes):

```text
adapter-witnesses
$ cargo test -p rum-stdlib --lib adapter_production_witnesses_are_computed -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 5.94s

base-skipped-some-default
$ cargo test -p rum-stdlib --test suspension_oracle fixtures::definition_application -- --nocapture
exit=101
test result: FAILED. 16 passed; 32 failed; 0 ignored; 0 measured; 66 filtered out; finished in 0.33s

base-skipped-some-no-default
$ cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::definition_application -- --nocapture
exit=101
test result: FAILED. 16 passed; 32 failed; 0 ignored; 0 measured; 66 filtered out; finished in 0.30s

base-storage-default
$ cargo test -p rum-stdlib --test suspension_oracle fixtures::definition_storage -- --nocapture
exit=0
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 114 filtered out; finished in 0.15s

base-storage-no-default
$ cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::definition_storage -- --nocapture
exit=0
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 114 filtered out; finished in 0.14s

candidate-definition-default
$ cargo test -p rum-stdlib --test suspension_oracle fixtures::definition_ -- --nocapture
exit=0
test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 66 filtered out; finished in 0.55s

candidate-definition-no-default
$ cargo test -p rum-stdlib --no-default-features --test suspension_oracle fixtures::definition_ -- --nocapture
exit=0
test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 66 filtered out; finished in 0.50s

census-default-1
$ cargo test -p rum-stdlib --test typed_slot_census source_baseline_matches_earned_receipt -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 23.28s

census-default
$ cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.09s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.03s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 142.32s

census-no-default
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 20.06s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 20.18s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 145.10s

census-renamed
$ cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.88s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 20.00s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 144.27s

certificate-emission-default-drop-order
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 18.23s

certificate-emission-no-default-drop-order
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 18.68s

compile-controls-default
$ cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 79.70s

compile-controls-no-default
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.64s

compile-typed-4
$ cargo test -p rum-runtime --lib suspension::typed::tests -- --nocapture
exit=0
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.48s

frame-root-trace
$ cargo test -p rum-runtime --lib frame_root_trace -- --nocapture
exit=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1158 filtered out; finished in 1.20s

frontier-inventory
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/frontier-fixture/Cargo.toml --test frontier checker::r6_production_views_inventory -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 10.45s

local-production-default-green
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 18.50s

local-production-default-rename
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 18.41s

local-production-default
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 17.85s

local-production-no-default-green
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 18.61s

local-production-no-default
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 18.59s

native-base-debug-measure
$ python3 /home/remi/dev/rum/.temp/brief-611/measure-native-stack.py base debug
exit=1
AssertionError: ('guarded', 2, 1, "\nrunning 1 test\ntest runtime::tests::holding_and_control_part2::brief611_stack_probe ... FAILED\n\nfailures:\n\nfailures:\n    runtime::tests::holding_and_control_part2::brief611_stack_probe\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.01s\n\n\nthread 'runtime::tests::holding_and_control_part2::brief611_stack_probe' panicked at crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs:670:5:\nunexpected result: f[2]\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n")

oracle-default
$ cargo test -p rum-stdlib --test suspension_oracle -- --nocapture
exit=0
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.51s

oracle-no-default
$ cargo test -p rum-stdlib --no-default-features --test suspension_oracle -- --nocapture
exit=0
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.96s

ordinary-controls-default
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/ordinary-controls/Cargo.toml --test ordinary -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

ordinary-controls-no-default
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

ownership-discovery-drop-order
$ cargo test -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 6.06s

ownership-discovery-initial
$ cargo test -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 5.71s

ownership-final
$ cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
exit=0
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 72.43s

production-rejections
$ cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 27.36s

recursive-48
$ cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.01s

retention-repeat-diagnostic
$ cargo test --manifest-path .temp/brief-611/frontier-fixture/Cargo.toml --test frontier checker::s5_probe_repeat_candidates -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 14.75s

```

Complete bounded gate index (105 top-level records; the four frozen base builds are recorded separately under base/):

| Gate | Exit | Log |
| --- | ---: | --- |
| adapter-witnesses | 0 | [verbatim log](../../../.temp/brief-611/adapter-witnesses.log) |
| allocation-comparison | 0 | [verbatim log](../../../.temp/brief-611/allocation-comparison.log) |
| base-allocations-s5-N100 | 0 | [verbatim log](../../../.temp/brief-611/base-allocations-s5-N100.log) |
| base-allocations | 1 | [verbatim log](../../../.temp/brief-611/base-allocations.log) |
| base-frame-census-final | 0 | [verbatim log](../../../.temp/brief-611/base-frame-census-final.log) |
| base-frame-census | 0 | [verbatim log](../../../.temp/brief-611/base-frame-census.log) |
| base-label-check | 0 | [verbatim log](../../../.temp/brief-611/base-label-check.log) |
| base-reachability | 0 | [verbatim log](../../../.temp/brief-611/base-reachability.log) |
| base-skipped-some-default | 101 | [verbatim log](../../../.temp/brief-611/base-skipped-some-default.log) |
| base-skipped-some-no-default | 101 | [verbatim log](../../../.temp/brief-611/base-skipped-some-no-default.log) |
| base-storage-default | 0 | [verbatim log](../../../.temp/brief-611/base-storage-default.log) |
| base-storage-no-default | 0 | [verbatim log](../../../.temp/brief-611/base-storage-no-default.log) |
| candidate-allocations | 0 | [verbatim log](../../../.temp/brief-611/candidate-allocations.log) |
| candidate-build-cli-alloc-drop-order | 0 | [verbatim log](../../../.temp/brief-611/candidate-build-cli-alloc-drop-order.log) |
| candidate-build-cli-alloc | 0 | [verbatim log](../../../.temp/brief-611/candidate-build-cli-alloc.log) |
| candidate-build-cli-drop-order | 0 | [verbatim log](../../../.temp/brief-611/candidate-build-cli-drop-order.log) |
| candidate-build-cli | 0 | [verbatim log](../../../.temp/brief-611/candidate-build-cli.log) |
| candidate-build-resume-alloc-drop-order | 0 | [verbatim log](../../../.temp/brief-611/candidate-build-resume-alloc-drop-order.log) |
| candidate-build-resume-alloc | 0 | [verbatim log](../../../.temp/brief-611/candidate-build-resume-alloc.log) |
| candidate-build-resume-drop-order | 0 | [verbatim log](../../../.temp/brief-611/candidate-build-resume-drop-order.log) |
| candidate-build-resume | 0 | [verbatim log](../../../.temp/brief-611/candidate-build-resume.log) |
| candidate-definition-default | 0 | [verbatim log](../../../.temp/brief-611/candidate-definition-default.log) |
| candidate-definition-no-default | 0 | [verbatim log](../../../.temp/brief-611/candidate-definition-no-default.log) |
| candidate-label-check | 0 | [verbatim log](../../../.temp/brief-611/candidate-label-check.log) |
| census-default-1 | 0 | [verbatim log](../../../.temp/brief-611/census-default-1.log) |
| census-default | 0 | [verbatim log](../../../.temp/brief-611/census-default.log) |
| census-no-default | 0 | [verbatim log](../../../.temp/brief-611/census-no-default.log) |
| census-renamed | 0 | [verbatim log](../../../.temp/brief-611/census-renamed.log) |
| certificate-emission-default-drop-order | 101 | [verbatim log](../../../.temp/brief-611/certificate-emission-default-drop-order.log) |
| certificate-emission-no-default-drop-order | 101 | [verbatim log](../../../.temp/brief-611/certificate-emission-no-default-drop-order.log) |
| compile-controls-default | 0 | [verbatim log](../../../.temp/brief-611/compile-controls-default.log) |
| compile-controls-no-default | 0 | [verbatim log](../../../.temp/brief-611/compile-controls-no-default.log) |
| compile-typed-1 | 101 | [verbatim log](../../../.temp/brief-611/compile-typed-1.log) |
| compile-typed-2 | 101 | [verbatim log](../../../.temp/brief-611/compile-typed-2.log) |
| compile-typed-3 | 101 | [verbatim log](../../../.temp/brief-611/compile-typed-3.log) |
| compile-typed-4 | 0 | [verbatim log](../../../.temp/brief-611/compile-typed-4.log) |
| final-census-default | 0 | [verbatim log](../../../.temp/brief-611/final-census-default.log) |
| final-census-no-default | 0 | [verbatim log](../../../.temp/brief-611/final-census-no-default.log) |
| final-certificate-default | 0 | [verbatim log](../../../.temp/brief-611/final-certificate-default.log) |
| final-certificate-no-default | 0 | [verbatim log](../../../.temp/brief-611/final-certificate-no-default.log) |
| final-controls-default | 0 | [verbatim log](../../../.temp/brief-611/final-controls-default.log) |
| final-controls-no-default | 0 | [verbatim log](../../../.temp/brief-611/final-controls-no-default.log) |
| final-fmt | 0 | [verbatim log](../../../.temp/brief-611/final-fmt.log) |
| final-frame-roots | 0 | [verbatim log](../../../.temp/brief-611/final-frame-roots.log) |
| final-native48 | 0 | [verbatim log](../../../.temp/brief-611/final-native48.log) |
| final-oracle-default | 0 | [verbatim log](../../../.temp/brief-611/final-oracle-default.log) |
| final-oracle-no-default | 0 | [verbatim log](../../../.temp/brief-611/final-oracle-no-default.log) |
| final-ordinary-default | 0 | [verbatim log](../../../.temp/brief-611/final-ordinary-default.log) |
| final-ordinary-no-default | 0 | [verbatim log](../../../.temp/brief-611/final-ordinary-no-default.log) |
| final-ownership | 0 | [verbatim log](../../../.temp/brief-611/final-ownership.log) |
| final-production-rejections | 0 | [verbatim log](../../../.temp/brief-611/final-production-rejections.log) |
| final-typed-api | 0 | [verbatim log](../../../.temp/brief-611/final-typed-api.log) |
| final-workspace | 101 | [verbatim log](../../../.temp/brief-611/final-workspace.log) |
| fmt-final | 0 | [verbatim log](../../../.temp/brief-611/fmt-final.log) |
| format-source | 0 | [verbatim log](../../../.temp/brief-611/format-source.log) |
| frame-root-trace | 0 | [verbatim log](../../../.temp/brief-611/frame-root-trace.log) |
| frontier-inventory | 0 | [verbatim log](../../../.temp/brief-611/frontier-inventory.log) |
| instruction-comparison | 0 | [verbatim log](../../../.temp/brief-611/instruction-comparison.log) |
| instructions-base-1 | 0 | [verbatim log](../../../.temp/brief-611/instructions-base-1.log) |
| instructions-base-2 | 0 | [verbatim log](../../../.temp/brief-611/instructions-base-2.log) |
| instructions-candidate-1 | 0 | [verbatim log](../../../.temp/brief-611/instructions-candidate-1.log) |
| instructions-candidate-2 | 0 | [verbatim log](../../../.temp/brief-611/instructions-candidate-2.log) |
| instructions-extra-base-1 | 0 | [verbatim log](../../../.temp/brief-611/instructions-extra-base-1.log) |
| instructions-extra-base-2 | 0 | [verbatim log](../../../.temp/brief-611/instructions-extra-base-2.log) |
| instructions-extra-base-3 | 0 | [verbatim log](../../../.temp/brief-611/instructions-extra-base-3.log) |
| instructions-extra-base-4 | 0 | [verbatim log](../../../.temp/brief-611/instructions-extra-base-4.log) |
| instructions-extra-base-5 | 0 | [verbatim log](../../../.temp/brief-611/instructions-extra-base-5.log) |
| instructions-extra-candidate-1 | 0 | [verbatim log](../../../.temp/brief-611/instructions-extra-candidate-1.log) |
| instructions-extra-candidate-2 | 0 | [verbatim log](../../../.temp/brief-611/instructions-extra-candidate-2.log) |
| instructions-extra-candidate-3 | 0 | [verbatim log](../../../.temp/brief-611/instructions-extra-candidate-3.log) |
| instructions-extra-candidate-4 | 0 | [verbatim log](../../../.temp/brief-611/instructions-extra-candidate-4.log) |
| instructions-extra-candidate-5 | 0 | [verbatim log](../../../.temp/brief-611/instructions-extra-candidate-5.log) |
| instructions-extra-comparison | 0 | [verbatim log](../../../.temp/brief-611/instructions-extra-comparison.log) |
| local-production-default-green | 0 | [verbatim log](../../../.temp/brief-611/local-production-default-green.log) |
| local-production-default-rename | 101 | [verbatim log](../../../.temp/brief-611/local-production-default-rename.log) |
| local-production-default | 101 | [verbatim log](../../../.temp/brief-611/local-production-default.log) |
| local-production-no-default-green | 0 | [verbatim log](../../../.temp/brief-611/local-production-no-default-green.log) |
| local-production-no-default | 101 | [verbatim log](../../../.temp/brief-611/local-production-no-default.log) |
| native-base-debug-build-integer-guard | 0 | [verbatim log](../../../.temp/brief-611/native-base-debug-build-integer-guard.log) |
| native-base-debug-build-with-greater | 1 | [verbatim log](../../../.temp/brief-611/native-base-debug-build-with-greater.log) |
| native-base-debug-build | 0 | [verbatim log](../../../.temp/brief-611/native-base-debug-build.log) |
| native-base-debug-measure-integer-guard | 0 | [verbatim log](../../../.temp/brief-611/native-base-debug-measure-integer-guard.log) |
| native-base-debug-measure | 1 | [verbatim log](../../../.temp/brief-611/native-base-debug-measure.log) |
| native-base-debug-prologues-integer-guard | 0 | [verbatim log](../../../.temp/brief-611/native-base-debug-prologues-integer-guard.log) |
| native-base-release-build | 0 | [verbatim log](../../../.temp/brief-611/native-base-release-build.log) |
| native-base-release-measure | 0 | [verbatim log](../../../.temp/brief-611/native-base-release-measure.log) |
| native-base-release-prologues | 0 | [verbatim log](../../../.temp/brief-611/native-base-release-prologues.log) |
| native-candidate-debug-build-integer-guard | 0 | [verbatim log](../../../.temp/brief-611/native-candidate-debug-build-integer-guard.log) |
| native-candidate-debug-measure-integer-guard | 0 | [verbatim log](../../../.temp/brief-611/native-candidate-debug-measure-integer-guard.log) |
| native-candidate-debug-prologues-integer-guard | 0 | [verbatim log](../../../.temp/brief-611/native-candidate-debug-prologues-integer-guard.log) |
| native-candidate-release-build | 0 | [verbatim log](../../../.temp/brief-611/native-candidate-release-build.log) |
| native-candidate-release-measure | 0 | [verbatim log](../../../.temp/brief-611/native-candidate-release-measure.log) |
| native-candidate-release-prologues | 0 | [verbatim log](../../../.temp/brief-611/native-candidate-release-prologues.log) |
| native-prologue-table-final | 0 | [verbatim log](../../../.temp/brief-611/native-prologue-table-final.log) |
| oracle-default | 0 | [verbatim log](../../../.temp/brief-611/oracle-default.log) |
| oracle-no-default | 0 | [verbatim log](../../../.temp/brief-611/oracle-no-default.log) |
| ordinary-controls-default | 0 | [verbatim log](../../../.temp/brief-611/ordinary-controls-default.log) |
| ordinary-controls-no-default | 0 | [verbatim log](../../../.temp/brief-611/ordinary-controls-no-default.log) |
| ownership-discovery-drop-order | 0 | [verbatim log](../../../.temp/brief-611/ownership-discovery-drop-order.log) |
| ownership-discovery-initial | 0 | [verbatim log](../../../.temp/brief-611/ownership-discovery-initial.log) |
| ownership-final | 0 | [verbatim log](../../../.temp/brief-611/ownership-final.log) |
| production-rejections | 0 | [verbatim log](../../../.temp/brief-611/production-rejections.log) |
| recursive-48 | 0 | [verbatim log](../../../.temp/brief-611/recursive-48.log) |
| resume-limit-diagnostic | 0 | [verbatim log](../../../.temp/brief-611/resume-limit-diagnostic.log) |
| retention-repeat-diagnostic | 0 | [verbatim log](../../../.temp/brief-611/retention-repeat-diagnostic.log) |

The machine-readable commands, all 114 test-result lines, warnings, fault records and skips are in [gate-results.json](../../../.temp/brief-611/gate-results.json). The workspace finished naturally; no command was abandoned or left running.


### 6. Performance evidence and unrun requirements

All four S4 bases were built before production conversion, with CODEGEN_UNITS=1 and separate targets. Their 1,430 Rust-source hashes equal the director's S4 candidate snapshot; no git command was used to infer the baseline. The final four candidate builds include the explicit head-drop correction. Every source hash still matches that frozen candidate. The earlier candidate binaries were archived before measurements and are not used for these results.

| Frozen binary | S4 SHA-256 | Final candidate SHA-256 |
| --- | --- | --- |
| rum-cli | `7881a7149a3c5a5d8e6d792f39d46338b365987d5c02908a65b6f4648d950e80` | `454ca3a71773f385571b02ca92c0a48990f831be5390686a8be5b884051ebb3e` |
| rum-cli-alloc | `adf80f3eeebbbe39dcba9b6295191c9462893540a0fdce48215fba3499284b24` | `a7fa9a4e83e9f4d89f84a68e6de997dbf779875f6944f341a1451745ac2cb658` |
| resume | `40f5c75c6b42d698523822959b9f0a39372c5d4700f0bad55e21b8c2f2aade3d` | `032531bb21a5b04a2335965de5318b1914f6e385e731d397f8852335c26e3fc1` |
| resume-alloc | `0414b33d5cee14192e786f2ca705ab250c35ccf76b257f5ea69da0dfaece04c5` | `1609c967d99a4c6078069b35b29d333fa08f72a57fb229d7ab2f90a0561d9cc4` |

Allocations: all 107 ordinary labels are exactly equal in whole-program N and 2N counts/bytes and in slopes. All 32 resume intercept differences are zero. Extra allocations are the outer typed adapter Arc (24 B) plus, on application RHS completion, the strict Option<Expr> Some wire (three allocations, 120 B). The measured per-label deltas follow; the required measured candidate frame-call reconciliation did not run before the STOP.

| Resume workload | Allocation delta / iteration | Byte delta / iteration |
| --- | ---: | ---: |
| resume-table | 0 | 0 |
| resume-do-chain | 0 | 0 |
| resume-call-repeat | 0 | 0 |
| resume-cleanup | 0 | 0 |
| resume-fold | 0 | 0 |
| resume-argument-twice | 0 | 0 |
| resume-rewrite-exprs | 0 | 0 |
| resume-rewrite-compiled | 0 | 0 |
| resume-cases-spec | 1 | 24 |
| resume-cases-traversal | 0 | 0 |
| resume-position-traversal | 0 | 0 |
| resume-count-traversal | 0 | 0 |
| resume-cases-shared | 1 | 24 |
| resume-s5-down-rhs | 4 | 144 |
| resume-s5-down-guard | 5 | 168 |
| resume-s5-sub-rhs | 4 | 144 |
| resume-s5-sub-guard | 5 | 168 |
| resume-s5-up-rhs | 4 | 144 |
| resume-s5-up-guard | 5 | 168 |
| resume-s5-default-rhs | 5 | 168 |
| resume-s5-default-guard | 6 | 192 |
| resume-s5-default-fallback-rhs | 5 | 168 |
| resume-s5-set | 1 | 24 |
| resume-s5-upset | 1 | 24 |
| resume-s5-tagset | 1 | 24 |
| resume-s5-rule-rhs | 1 | 24 |
| resume-s5-rule-lhs | 2 | 48 |
| resume-s5-destructure | 1 | 24 |
| resume-s5-lhs-twice | 2 | 48 |
| resume-s5-own-rhs | 1 | 24 |
| resume-s5-destructure-twice | 2 | 48 |
| resume-s5-rule-lhs-twice | 3 | 72 |

Allocation sites: typed/adapter.rs outer Arc; suspension/primitive.rs Some encoding creates a one-Expr Vec and calls Expr::call, whose SharedExpr Arc and SharedArgs Arc account for the other two allocations. The outer adapter allocation is typed/adapter.rs:62. The three Some-wire allocations are primitive.rs:39 (the Expr Vec), rum-core expr.rs:577–580 (SharedExpr Arc), and expr.rs:641–645 (SharedArgs Arc), reached through Expr::call at :813. These source sites explain the observed sizes; executed-address/all-frame reconciliation is not claimed because the final call census is unrun.

Hardware instructions: two complete interleaved pairs cover all 139 labels. Fourteen ordinary labels were flagged conservatively using the maximum pair delta. Five additional interleaved measurements per arm clear thirteen by the larger observed range; default-unguarded-miss remains a defect: base mean 12,786.8649, candidate mean 12,800.0395, delta +13.1746 instructions/iteration, bar 6.39343245. The wide ValueQ label has large setup subtraction noise, including one negative slope; its clearing range is measurement resolution, not proof of a speedup. Six-process address attribution did not run. No source fix for the instruction regression was attempted; it remains an unresolved defect.

| Profile | Recursion | Base bytes/level | Candidate bytes/level | Delta | Maximum residual, both arms |
| --- | --- | ---: | ---: | ---: | ---: |
| debug | peel | 19152 | 19360 | +208 | 0 |
| debug | plus | 34896 | 35376 | +480 | 0 |
| debug | sub | 30352 | 30528 | +176 | 0 |
| debug | up | 28528 | 28288 | -240 | 0 |
| debug | guarded | 35824 | 36304 | +480 | 0 |
| release | peel | 7312 | 7440 | +128 | 0 |
| release | plus | 13744 | 13920 | +176 | 0 |
| release | sub | 12928 | 13264 | +336 | 0 |
| release | up | 11424 | 11584 | +160 | 0 |
| release | guarded | 14192 | 14368 | +176 | 0 |

All native measurements use four heights and three samples per shape, per profile and arm. Actual resident prologue deltas exactly reconcile every measured slope. The detailed slot table is stack/slot-attribution.md; complete touched-family prologues are stack/native-changed-prologues.tsv. This is not acceptance: peel and plus growth requires the director's ruling, and the additional SubValue/UpValue growth is reported as a defect. No macro expansion was changed or retried. The canonical 48-level native test passes.

**Not run before STOP:** the 107-label ordinary memcpy comparison; six-process instruction/address proof for the three leaves and the default-miss defect; six-process per-function resume attribution for all 32 workloads; measured candidate frame counts and allocation reconciliation; counts/addresses of the driver's owed 136-B typed-outcome transfers; all four timing panels and any required leaf reruns. Earlier S4 reachability profiles are not substitutes for these gates. No timing acceptance, copy acceptance, or complete resume-cost attribution is claimed.

The full allocation table, with whole-program N and 2N cells and intercepts, is [perf/allocation-comparison.json](../../../.temp/brief-611/perf/allocation-comparison.json); ordinary allocation defects: [perf/ordinary-allocation-defects.json](../../../.temp/brief-611/perf/ordinary-allocation-defects.json) (empty). All hardware instruction rows are in [perf/instruction-comparison.json](../../../.temp/brief-611/perf/instruction-comparison.json). The original two pairs and five additional interleaved runs per flagged arm retain their raw perf outputs under perf/instructions/. Five-run decisions are:

| Ordinary label | Base mean | Candidate mean | Delta | Bar | Cleared |
| --- | ---: | ---: | ---: | ---: | --- |
| lazy-range-normal-small | 53585.078280 | 53545.781530 | -39.296750 | 238.712750 | True |
| lazy-select-realize-small | 112484.738800 | 112485.544030 | +0.805230 | 124.942200 | True |
| lazy-nestlist-realize-small | 142997.637800 | 142944.453730 | -53.184070 | 183.653100 | True |
| lazy-table-construct-full | 23907.761850 | 23859.748620 | -48.013230 | 168.043150 | True |
| lazy-tuples-take-small | 26269.837600 | 26279.469390 | +9.631790 | 171.171550 | True |
| differences-no-input | 9915.924930 | 9917.438650 | +1.513720 | 21.344650 | True |
| lazy-depth-map-build-64 | 29809.112200 | 29815.569200 | +6.457000 | 46.323000 | True |
| lazy-depth-nest-map-64 | 944618.646200 | 943641.524200 | -977.122000 | 3330.903000 | True |
| lazy-depth-map-first-256 | 3668165.452400 | 3661573.003800 | -6592.448600 | 13707.492000 | True |
| appendto-part-outer1000-large | 3606686.248000 | 3619939.493000 | +13253.245000 | 85215.675000 | True |
| set-part-fill-small | 2781234.645000 | 2785399.523000 | +4164.878000 | 13578.650000 | True |
| downvalue-exact-miss | 998.083900 | 998.050000 | -0.033900 | 2.623000 | True |
| default-unguarded-miss | 12786.864900 | 12800.039500 | +13.174600 | 6.393432 | False |
| valueq-undefined-symbol-definitions-8000 | 278.540000 | 3058.000000 | +2779.460000 | 24600.950000 | True |

Raw five-run samples and ranges: [perf/ordinary-five-run-comparison.json](../../../.temp/brief-611/perf/ordinary-five-run-comparison.json). The default-miss samples are base [12786.883, 12787.404, 12786.7125, 12785.9105, 12787.4145], candidate [12800.6905, 12799.575, 12801.5035, 12800.032, 12798.3965]. Their ranges are 1.504 and 3.107; the 0.05% threshold is the larger 6.39343245.

Complete stack prologue table: [stack/native-changed-prologues.tsv](../../../.temp/brief-611/stack/native-changed-prologues.tsv) (610 symbol/profile groups: 504 matched, 30 base-only, 76 candidate-only). It is a touched-family superset including unchanged functions and compiler-generated bodies, not 610 claimed source edits. Per-symbol maxima may combine ordinary and resumed monomorphs; actual recursive resident paths are separated in [stack/frame-deltas.tsv](../../../.temp/brief-611/stack/frame-deltas.tsv). Every measured residual is zero. Scratch native probes use safe Rust stack markers, identical four heights/three samples per arm, the standard native guard, and a high guest recursion limit to isolate the native measurement. An identical scratch-only integer Greater test handler supplies the stdlib behavior missing from the runtime unit session. Neither the canonical limit nor test contract was weakened.

The slot attribution and its limitations follow verbatim:

All offsets below are relative to `%rsp` after the function prologue. Addresses
refer to the frozen `stack/{base,candidate}/native/{debug,release}/runtime-tests`
binaries, not the CLI binaries. Full disassembly is retained beside each binary;
the relevant functions are also in `slot-excerpts/`.

#### Resident cycles

Every one of the 120 measurements per arm (five shapes, two profiles, four
heights, three samples) has a zero fit residual. `frame-deltas.tsv` reconciles
every measured difference from resident prologues, including repeated DownValue
scan calls. A return address is eight bytes on both arms and cancels.

| Profile | Shape | Base B/level | Candidate B/level | Delta |
|---|---|---:|---:|---:|
| debug | peel | 19152 | 19360 | +208 |
| debug | plus | 34896 | 35376 | +480 |
| debug | SubValue | 30352 | 30528 | +176 |
| debug | UpValue | 28528 | 28288 | -240 |
| debug | guarded DownValue | 35824 | 36304 | +480 |
| release | peel | 7312 | 7440 | +128 |
| release | plus | 13744 | 13920 | +176 |
| release | SubValue | 12928 | 13264 | +336 |
| release | UpValue | 11424 | 11584 | +160 |
| release | guarded DownValue | 14192 | 14368 | +176 |

#### DownValue scan slots

The debug ordinary linear scan is base `0xa7a3d0`, candidate `0xa5e700`.
Its frame grows 3800 → 4072 bytes. This is not a lazy closure environment in
the recursive function: the macro is in the separate cold suspend helper.

| Purpose | Base slot / instructions | Candidate slot / instructions |
|---|---|---|
| Actual compiled-match producer result, 136 B | `0xa28..0xaaf`; return pointer at `0xa7b4a9`, producer call `0xa7b4c9` | `0xa48..0xacf`; return pointer at `0xa5f811`, producer call `0xa5f831` |
| Cold helper application argument, 168 B | `0xc18..0xcbf`; copy `0xa7b6b6..0xa7b6d7` | `0xc10..0xcb7`; copy `0xa5fa0c..0xa5fa32` |
| Cold helper input | extracted 32-B request at `0xbf0..0xc0f`, stores `0xa7b68e..0xa7b6a6`; 8-B gap before application argument | actual 136-B result at `0xcb8..0xd3f`, copy `0xa5fa5e..0xa5fa78` |
| Cold helper call | `0xa7b6fe` | `0xa5faa1` |
| Other-error forwarding | one 136-B temporary `0xcc0..0xd47`, copies at `0xa7b715..0xa7b744` | two 136-B temporaries `0xd40..0xdc7` and `0xdc8..0xe4f`, copies at `0xa5fab0..0xa5fb00` |
| RHS return forwarding | `Result::map(Some)`, argument copy and call `0xa7be3f..0xa7be6f` | `Result::map_err(Route::failure)`, argument copy and call `0xa60176..0xa601a6` |

The cold argument region grows by 96 B (304 versus 208 including the old
8-B gap); error forwarding adds 136 B. Common later locals shift by 32 B
before this region, and the final frame rounding contributes 8 B: together
32 + 96 + 136 + 8 = 272. The 32-B prefix change is compiler spill/temporary
space; individual Rust-local identities are not claimed for those bytes.
This is an explicit attribution limitation, not accepted closure cost.

Release ordinary linear scan is base `0x51c0c0`, candidate `0x51f000`,
1272 → 1320 B. Base extracts the request to `0x440..0x45f` at
`0x51e119..0x51e134`, copies the 168-B application to `0x50..0xf7`, and
calls the cold helper at `0x51e176`. Candidate copies the application to
`0xa0..0x147` at `0x521076..0x521098`, copies the actual 136-B result from
`0x320..0x3a7` to `0x260..0x2e7` at `0x52109b..0x5210b3`, and calls at
`0x5210c7`. These named slots prove the enlarged cold argument. Optimized
slot reuse makes the net increase 48 B; no claim is made that all 48 B
are inevitable or are the macro closure.

#### Outer DownValue lookup

Debug `try_apply_down_values` grows 2088 → 2104 B (`0x1ad340` →
`0x19df00`). The former fallback helper accepts only session, lookup,
symbol and a Boolean, and returns the request. The typed helper receives
the actual Option result and returns that result with its fixed output.
Candidate copies 136 B from `0x2a0` to `0x368` at `0x19efc7..0x19efe8`
and passes that address in `%r9` at `0x19f00f`; the cold call is
`0x19f018`. Additional saved Boolean/address slots are `0xc` and `0x10`
(`0x19efa3`, `0x19efcf`). The frame's other common offsets shift 16 B.
The release lookup has no frame-size delta.

#### Application RHS

The original Expr helper is compared with the new Option helper because
those are the functions actually resident in guest recursion. Debug frames
shrink 1336 → 1256 B; release frames grow 840 → 920 B.

Release base is `0x5254e0`, candidate `0x52e340`:

| Result storage | Base | Candidate |
|---|---|---|
| Actual evaluator result, 136 B | `0xe0..0x167`, producer call `0x525670` | `0x258..0x2df`, producer call `0x52e4e2` |
| Argument passed to finish helper, 136 B | `0x290..0x317`, copy/call `0x52588f..0x5258b5` | `0x2e0..0x367`, copy/call `0x52e650..0x52e679` |
| Completed Expr result before Some, 136 B | `0x170..0x1f7` | `0x1d0..0x257` |
| Cold suspension input | extracted request in the base helper | actual 136-B result copied to `0x80..0x107` at `0x52e860..0x52e878` |
| Cold suspension output, 136 B | reconstructed Expr error before shared profiling tail | Option result at `0x110..0x197`, cold call `0x52e88e`, profiled before its early return |

The candidate places the finish-helper argument 80 B higher, reaching
`0x367` instead of `0x317`; the prologue increases by exactly 80 B. The
storage layout has different reuse across profiling and cold-result paths.
This evidence identifies the live results and their addresses, but does not
justify the extra bytes as a macro environment. No macro variant was tried.

#### Ruling needed

`peel`: debug +272 scan +16 lookup -80 RHS = +208; release +48 scan
+80 RHS = +128. `plus` and guarded recursion retain two scan invocations,
so their deltas are debug 2×272 +16 -80 = +480 and release 2×48 +80 = +176.
These increases remain defects/reportable exceptions under the brief.
The director must rule on `peel` and `plus`; this report does not accept them.

#### Other S5 recursion shapes

The SubValue debug scan (`0x1b4ca0` → `0xa642c0`) grows 3656 → 3912 B.
The cold input changes from a 32-B request at `0xbc0..0xbdf` (stores
`0x1b64d4..0x1b64ec`) to the actual 136-B result at `0xc78..0xcff`
(copy `0xa65b67..0xa65b81`). The 168-B application argument moves from
`0xbe8..0xc8f` to `0xbd0..0xc77`. Error forwarding has one 136-B
temporary at base `0xc90`, versus two at candidate `0xd00` and `0xd88`
(`0xa65bb4..0xa65c04`). The same region accounting is +16 prefix, +96
cold argument region, +136 error forwarding, +8 rounding = +256 B.
After the RHS helper's -80 B, the measured cycle grows 176 B.

Release SubValue grows 1336 → 1592 B (`0x5268c0` → `0x52f700`). The
base cold request is `0x480..0x49f` (`0x528b28..0x528b43`), with the
application argument at `0xd0..0x177`; call `0x528b86`. Candidate's
application argument is `0x70..0x117`, and its actual result is copied
from `0x440..0x4c7` to `0x1a0..0x227` at `0x5321ed..0x532205`; call
`0x532218`. Optimized storage reuse and spills remain part of the +256-B
frame difference; these bytes are not claimed as an accepted environment.
Together with RHS +80 B, this is the measured +336 B per level.

Release UpValue inlines the list scan into `try_apply_up_values` and grows
1016 → 1096 B (`0x529ef0` → `0x535030`). The base passes the extracted
32-B request at `0xc0` through an outgoing pointer at `%rsp` (`0x52b1e3`)
to the cold helper (`0x52b216`). Candidate copies 136 B from `0x330` to
`0x280` (`0x53625f..0x536277`) and passes that address through `%rsp`
(`0x53627d`), calling `0x5362a8`. Reuse reduces the enlarged input's net
frame cost to +80 B; RHS +80 B gives the measured +160 B per level.
Debug UpValue's actual resident list scan shrinks by 160 B, with another
80-B reduction from the RHS helper, for the measured -240 B.


### 7. Stop-loss, deviations and required director rulings

**Required STOP:** `crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:103` declares the outbound source boundary; its assertion at :162 rejects eight newly introduced call edges. A correct update needs a checked audit of the new route/helpers and the immediate closure edge. A generic `<indirect-call>` exemption would weaken that boundary. This is an expansion of audit rules, beyond a pin following the same construct after it moves. I therefore applied Scope's “Anything else is a STOP” and Disclosure duty's “make no edit there. STOP and report the file, the line and why.” The audit file is unchanged. The already running workspace command was allowed to finish; only evidence collection and this report followed the stop. A director scope ruling is required before further implementation or acceptance work. Evidence: [scope-stop.json](../../../.temp/brief-611/scope-stop.json).

Stop-loss: zero consecutive unsuccessful repair attempts without progress. The initial typed API compile errors reduced 54 → 6 → 2 → 0; the retention method collision was diagnosed and fixed in the owned helper. Native scratch-probe setup first exposed the absent Greater handler, then two compile errors in that handler; both were resolved without canonical changes. The S4 large-N resume failure was isolated to the guest recursion limit and corrected uniformly at the measurement-size level. No performance-fix trial was attempted. The structural-audit failure triggered scope STOP immediately after its cause and scope were checked; no test was relaxed or patched.

Counters: 22 dedicated build/typed-compilation iteration attempts (four S4 base builds, eight candidate measurement builds including the rebuilt final four, six native-probe builds, four initial typed-API compilation iterations; the latter two iteration commands also request tests). The 105 recorded top-level gate invocations include 52 direct cargo-test commands, two of which are --no-run, and three cargo-fmt commands. Nested positive/negative compiler fixtures and measurement processes are not counted as source repair attempts. All counters have explicit units; the prior checkpoint's 48 test-run counter excluded the four typed compilation iterations. Both that history and the final command index are preserved.

Deviations and limits:

- No git command or network tool was used. All work is left uncommitted; the director owns git. No new unsafe, dependency, lockfile, toolchain, core representation, or witness macro change.
- The requested oracle script hardcodes .temp/suspension-oracle outside the brief's scratch fence. Its exact two underlying Cargo test invocations were run with brief-611 temporary/log paths instead. Both final builds pass warning-free.
- Agent-authored scratch stays under brief-611, apart from the explicitly allowed certificate and compile-control destinations. The unchanged mandatory workspace tests also use their existing Cargo target/brief-* compiler-fixture directories; their implementations were not edited to relocate them.
- The guest-unreachable pending SubValue site is repaired but untested through guest code. Some ordinary neighbor labels reach their owned function during setup only. Both limitations remain explicit.
- The 19 new resume N values were reduced uniformly to 100 after the first failing S4 N=2,000 cell; all original raw samples are preserved. The measurement source and limits were not changed.
- The no-default-feature resume harness and the remaining copy, leaf, resume-attribution, and timing gates did not run. Prepared scripts are not results. No complete performance acceptance is claimed.
- default-unguarded-miss remains an instruction regression. Native stack growth remains unresolved; peel and plus require the director's ruling on the slot tables. Even a scope ruling on the G4 audit would not accept those costs.

No half-edited files or running commands remain. The final code/inventory hashes match the measured candidate. The next action is a director ruling on the G4 source-boundary audit; only then can a scoped repair and the missing verification proceed. This report does not mark S5 accepted.


## Director amendment — redispatch 1: the G4 audit, the cold-helper copies, and the rest of the step

The STOP was correct, and so was refusing a generic `<indirect-call>`
exemption. The report is honest about what did not run. The skipped-`Some`
repair works: my own resumed-against-queued probe of nine shapes (exact,
pattern and guarded DownValue; exact and pattern SubValue; exact and pattern
UpValue; `Default` fallback; optional default), each returning a guest-written
`System`InternalContinuationSome[5]` or `System`InternalContinuationNone[]`, gives
eight DIFFs on the S4 tree and nine SAMEs on yours. (The guarded DownValue shape
was already SAME on S4.)

### Accepted as reported (I re-verify all of it at my gate)

- The census: 607/589 production, 37/42 typed. F0244A/M0245A is the planned
  split, with one frame per pause, and OwnValue keeps `Expr`.
- The 48 application fixtures, watched failing on S4 (32 sentinels), the 19
  storage controls, the 133/133 oracle in both builds, the 32 compile pairs,
  ownership 31/31, frame roots and the 48-level test.
- The rename `repeat` → `suspend_again`.
- The ordinary allocations: 107/107 labels are equal.
- **The `Some` wire on the application-RHS resume.** Each such resume makes
  three allocations (120 B): `primitive.rs:39` and the two `Arc`s in
  `rum-core`'s `Expr::call`. `Option<Expr>` is still `Legacy`. Its flip owes
  this, and must show it gone, exactly as for F0278. Do not repair it here.

### A. Ruling on the G4 source-boundary audit

A generic `<indirect-call>` would admit any closure or function-pointer
call, including one that re-enters a G4 function. The precise rule is
narrower:

1. **The walker.** In `recursion_inventory.rs`, change
   `FlatCalls::visit_expr_call` so that a call whose callee is a closure
   *literal* records no `<indirect-call>`. The literal may be parenthesized:
   `(|| body)()` or `(move || body)()`. The visitor already descends into
   `call.func`, so every call in that body is attributed to the caller and
   audited like the caller's own code. Any other non-path callee still
   records `<indirect-call>`. That covers a closure bound to a local and called
   later, a field or a function pointer.
2. **Its falsifier.** Add walker tests on small source strings:
   - An immediately-invoked closure whose body calls an owned function yields
     that edge, and no `<indirect-call>`. A cycle closed through such a body is
     reported by `flat_walk_recursive_edges`.
   - `let c = || f(); c()` still records `c`.
   - `(self.callback)()` still records `<indirect-call>`.

   Watch the first test fail on the current walker before you change it.
3. **G4's `OUTBOUND`.** Add the seven new callees:
   - `TypedAction::complete`
   - `TypedAction::complete_work`
   - `ResumeStop::failure`
   - `Route::failure`
   - `map_err`
   - `suspend_again`
   - `resume_typed_or_suspend!`

   Read each definition first. In the report, state in one line per callee
   that it neither evaluates guest code nor calls a G4 function. Extend the
   comment above `OUTBOUND` to name the typed continuation API.
   `resume_typed_or_suspend!` is the typed sibling of the already-audited
   `resume_or_suspend!`, and the walker does not descend into macro tokens
   for either.
4. **G7.** Since `033711f9`, `g7_continuation_contract.rs:372` has allowed
   `<indirect-call>` generically.
   - If G7 passes without it after the walker change, remove it.
   - If G7 still needs it, report the exact caller and callee expression, and
     leave it in.
5. **Other users.** Report every other test that uses `flat_walk_call_graph`
   or `flat_walk_recursive_edges` and changes outcome.

**Newly authorized:**

- `crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs`:
  the walker rule and its tests, nothing else.
- `rule_pattern_depth_contract.rs`: `OUTBOUND` and its comment.
- `g7_continuation_contract.rs`: that one token.

### B. The stack: the cold helpers carry the producer's whole `Result`

Your slot tables name the cause, and it is not the macro's closure
environment:

- **The whole `Result` crosses the call.** Base passed the extracted 32-B
  request to each cold helper. The candidate passes the producer's whole
  136-B `Result` instead. This happens at:
  - the DownValue and SubValue scans;
  - release UpValue;
  - the outer DownValue lookup;
  - the application-RHS helper.
- **A second error temporary.** Other-error forwarding uses two 136-B
  temporaries where base used one.
- **Both are on the guest-recursive path.** `plus` and the guarded recursion
  hold two scan frames per level, so they pay twice.

| Profile | Shape | Base B/level | Candidate B/level | Delta |
|---|---|---:|---:|---:|
| debug | peel | 19152 | 19360 | +208 |
| debug | plus | 34896 | 35376 | +480 |
| debug | SubValue | 30352 | 30528 | +176 |
| debug | UpValue | 28528 | 28288 | −240 |
| debug | guarded DownValue | 35824 | 36304 | +480 |
| release | peel | 7312 | 7440 | +128 |
| release | plus | 13744 | 13920 | +176 |
| release | SubValue | 12928 | 13264 | +336 |
| release | UpValue | 11424 | 11584 | +160 |
| release | guarded DownValue | 14192 | 14368 | +176 |

The summed excess over base is 2,320 B, and 9 of 10 pairs are above it.

**Ruling B.**

1. **The bar stands:** no growth beyond the typed macro's named closure
   environment, attributed per slot. Neither a producer `Result` passed by
   value across a call nor a second error temporary is that environment.
2. **Try these shapes in order.** Measure each and attribute it per slot.
   - **(a) Put the macro at the producer's site.** Use the caller-place arm
     in the ordinary driver, so that the macro's own cold closure is the only
     cold capture. That closure takes the extracted request, not the
     `Result`. If the generic `Route` forces the ordinary suspension across a
     call boundary, give the ordinary route its own arm at the site instead
     of passing the `Result`. Either put the macro inline under
     `if Route::ORDINARY`, or split the ordinary and resumed drivers, as S4's
     `arguments.rs` does.
   - **(b) Forward other errors once.** Do it at the match, with no re-match
     after a call.
   - **(c) The constraints are unchanged.** `witness.rs` stays as it is. The
     witness comes from the actual producer `Result`. Nothing is constructed
     on the ordinary path. The insertion route is the macro on the ordinary
     route and `ContinueToken::suspend` inside a typed resume.
3. **Acceptance.**
   - Every pair is at or below base, or its growth is attributed per slot to
     the macro's named closure environment.
   - Debug UpValue stays at or below base.
   - Any growth left on `peel` or `plus`, in either profile, is reported with
     its slot table, and I rule on it.
   - The 48-level test, the guard and the stack configuration stay
     unchanged. Move locals, not control flow.
4. **Stop-loss.** Progress means either of two numbers falls: the summed
   excess, now 2,320 B, or the count of failing pairs, now 9. Stop after two
   attempts that lower neither, and report.

### C. The ordinary instruction regression: `default-unguarded-miss`

The label costs +13.17 instructions per iteration (12,786.86 → 12,800.04)
over five runs per arm. The ranges are 1.50 and 3.11 against a 6.39 bar, and
nothing has attributed the difference yet.

- **Re-measure after B.** If the label is still flagged, run six callgrind
  processes per arm and diff every function. Name each deterministic delta
  at its addresses, with call counts and callers.
- **Repair or report.** A copy or move that base did not make is a defect,
  so remove it. Report anything else with its addresses.
- **One suspect to test.** The default lookup in `try_apply_down_values`
  now moves `direct_default` into the fallback call. Verify this; do not
  assume it.

### D. Remove the dead `pending` branch

`try_apply_sub_values_from_symbol` takes
`pending: Option<CompiledMatchOutcome>`. Both callers pass `None`, and
nothing ever sets it, as your `unreachable-sites.json` shows. Its doc comment
describes a resume path the frame no longer takes. The resume now enters
`finish_suspended_sub_value_application`.

A repaired branch that no guest program can reach is code that can rot.
Remove the parameter, the branch and the stale sentence of the doc comment.
The 15 application callers become 14. Follow every pin, stage row and census
receipt this moves, and disclose each.

### E. Director finding, not yours to fix: the 1,021-resume `RecursionLimit`

Your `N=2000` cell did not hit a size limit. It hit a defect that has existed
since `eee42d2f` (2026-03-21). At suspension, the resume driver keeps the
evaluation stack's active entries minus the top two
(`suspension/core.rs:493-505`). On resume it sets the depth to that stack's
length. As the resumed frames complete, it never pops the entries or lowers
the depth.

So an expression re-evaluated after a pause is reported as a cycle. On the
S4 tree each of these fails after one pause with `General::passlim` and
`TerminatedEvaluation["IterationLimit"]`, while the queued-input path
succeeds:

- `f[1] + f[1]` with `f[x_] := (InputString["rhs"]; 7)`;
- `Table[{{InputString["x"]}}, {3}]`.

A changing expression leaks one depth level per resume until
`RecursionLimit`. That is your cell. This becomes its own group after S5
lands.

In S5: keep `N=100/200` for the 19 new resume labels. Do not repair the
defect, do not add tests for it, and do not shape any fixture around it.

### F. Then finish the step, on the final source

1. **Every correctness gate**, on the final source:
   - fmt and the typed runtime tests;
   - frame roots and the 48-level test;
   - the compile controls, the census, local production and the ordinary
     controls, each in both builds;
   - ownership and the production rejection census;
   - both oracle builds, warning-free;
   - native ZeroMQ and the full workspace.

   Replay the inventories, certificates and fingerprints, and disclose every
   changed row.
2. **Every performance section, against the same frozen S4 bases.** Verify
   the base hashes and rebuild the candidates. Run what did not run before:
   - the no-default-feature resume harness;
   - the measured candidate frame-call census and allocation reconciliation;
   - the 107-label ordinary `memcpy` comparison;
   - the six-process address proof for the three leaves;
   - six-process per-function resume attribution for all 32 workloads, with
     the owed 136-B typed-outcome transfers counted at their addresses;
   - the stack table, with B;
   - the full instruction panel again.
3. **Timing last.** Run four pinned panels: base, candidate, base,
   candidate. Rerun a deterministic leaf outside [0.8, 1.25] five times,
   interleaved. Timing remains an indication only.

### Process

- This run forks your session, so your scratch, state and frozen bases are
  where you left them in `.temp/brief-611/`.
- Your tool timeout must exceed every `--timeout` you pass.
- Checkpoint as before. Then append `## Execution report — redispatch 1`
  after this section.
