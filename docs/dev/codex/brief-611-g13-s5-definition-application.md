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


## Execution report — redispatch 1

**STOP under the Hard rules; S5 is not accepted or complete.** A is implemented
and its shared audit suite passes. B(a) made the native stack worse. B(b) then
failed to compile twice with the same two `E0658` errors on the required Rust
toolchain. I stopped instead of making a third compile experiment. The six
application files changed by the experiments have been restored byte-for-byte
to their redispatch-entry contents. The four-file A repair remains. C, D and
the final-source F gates have not been executed in this redispatch.

This report supplements, and does not replace, the original report or the
director amendment above. No git command, network operation, dependency,
lockfile, toolchain or `unsafe` change was made. No sub-agent was used.
`typed/witness.rs` is unchanged. Scratch is under `.temp/brief-611/r1/`;
`TMPDIR` remained `.temp/brief-611/tmp`. All expensive commands ran serially
through `run-bounded.py`, with one Cargo job, one test thread and offline mode.

The checkpoint is `.temp/brief-611/r1/state.json`; `PROGRESS.md`, the entry
snapshot, the retained patch, the rejected patches/source, all command records,
raw output and exit records are beside it. The root brief-611 checkpoint now
points to this STOP, rather than the earlier G4 scope STOP that the amendment
cleared. The four original performance bases and both frozen native stack
bases were hash-verified. No new performance candidate is presented as final.

**Retained files and source boundaries.** These are the only four source files
different from the 1,486-file redispatch-entry snapshot. This appended report
is the only documentation edit.

| File | Retained change |
|---|---|
| `crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs` | At the non-path callee branch in `FlatCalls::visit_expr_call`, unwrap parentheses solely to recognize a closure literal. Omit its generic indirect marker and retain the normal recursive visit of its body. Other non-path callees retain the marker. Add the three requested falsifier tests. |
| `crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs` | Extend the comment to name the typed continuation API; add exactly the seven authorized `OUTBOUND` names. No other audit rule changes. |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | Delete exactly the one `<indirect-call>` token at line 372. G7 passes without it after the walker change. |
| `crates/rum-runtime/src/runtime/definitions/rule_lhs/continuation.rs` | At line 61, `crate::runtime::kernel_api::TypedAction::complete` → the already imported `TypedAction::complete`. This names the exact audited callee without adding an eighth outbound alias. Its behavior, arguments and line count are unchanged. |

The complete retained diff and before/after hashes are
`r1/retained-source.patch` and `r1/retained-source-changes.json`. No frame-root
pin, F/M row, stage row, fixture, census receipt or inventory row was changed
in this redispatch. Fingerprints and certificates have **not** been replayed
after A; they must not be treated as current acceptance evidence.

**A: falsifier and shared users.** Before changing the walker, the new first
test failed exactly as required:

```text
assertion `left == right` failed: the literal body remains audited: fn caller() { (|| owned())(); } fn owned() {}
  left: {"<indirect-call>", "owned"}
 right: {"owned"}
```

The three tests cover `(|| owned())()`, nested parentheses, a parenthesized
`move` closure, a cycle through the closure body, `let c = || owned(); c()`
(edges `c` and `owned`), and `(self.callback)()` (the indirect marker).
The shared suite changed from 59 passed / 2 failed to 61 passed / 0 failed.
The two outcome changes are the new first walker test and the existing
`structural_rule_pattern_depth_contract`, both failed → passed. The cycle
test and stored-closure/callback test already passed on the old walker and
still pass. No falsifier was weakened or deleted.

The other users found by the source search are G7, the destructuring contract,
G4's other graph/recursion checks and the recursion-inventory tests themselves.
There are **no other existing test outcome changes**. In particular, G7 and
destructuring pass both before and after; G7's after-run has no generic
indirect exemption. Expected panics from the suite's falsifiers remain in the
raw `--nocapture` logs and are not additional test failures.

All seven definitions were read before extending `OUTBOUND`:

| Callee | Semantic audit |
|---|---|
| `TypedAction::complete` | Encodes/completes the already produced value; the G4 `Expr` slot is Value. It neither evaluates guest code nor calls a G4 function. |
| `TypedAction::complete_work` | Selects immediate completion or an existing stop action. It neither evaluates guest code nor calls a G4 function. |
| `ResumeStop::failure` | Constructs error completion. It neither evaluates guest code nor calls a G4 function. |
| `Route::failure` | Ordinary identity or `ResumeStop::failure` on the token route. It neither evaluates guest code nor calls a G4 function. |
| `map_err` | The local Rust 1.85 `Result` implementation forwards `Ok` or calls the supplied error mapper; this audited use supplies `Route::failure`. It neither evaluates guest code nor calls a G4 function. |
| `suspend_again` | Witnesses the actual producer, extracts its request and queues fresh same-type state through the token. It neither evaluates guest code nor calls a G4 function. |
| `resume_typed_or_suspend!` | Matches the actual producer and inserts only on InputRequired; the audited G4 success arm is unreachable. It neither evaluates guest code nor calls a G4 function. The walker remains opaque to both this macro's tokens and its legacy sibling's tokens. |

**Owned rows, fixtures and the accepted earlier evidence.** A retains the
previous S5 implementation: the accepted production totals are 607 frames /
589 macro sites, with 37 typed frames / 42 typed macro sites. These totals
were not rerun in this redispatch. F0244A/M0245A and the application RHS
`Option<Expr>` frame remain the previously reported split; OwnValue retains
its `Expr` frame. The generic DownValue instantiations, one-frame-per-pause
repair, removed transport maps, all fixture observations, and original row
classification tables remain as reported above. There is no new owned-row
transition in the retained redispatch patch.

The 48 application fixtures, 19 storage controls, 133-test oracle builds,
32 compile pairs, 28 ordinary controls and their S4 observations are **earlier
evidence**, not reruns here. No fixture or harness source was changed. The
three allocations / 120 B owed to the application RHS Some wire are untouched.
The amendment's diagnosis of the 1,021-resume failure supersedes the earlier
report's size-limit explanation: it is the pre-existing resume-depth leak.
No repair, test or fixture workaround for that leak was attempted. The 19
new resume labels remain at N=100/200.

**B(a): measured and rejected.** The trial moved the typed macros from the
whole-result cold helpers into the DownValue, SubValue, UpValue and direct
default producer sites, and introduced inline RHS bodies so the outer
profiling tail still executes after suspension. For the generic DownValue
and SubValue return types it used an immediately invoked conversion closure
under `Route::ORDINARY`, followed by `map_err(Route::failure)`. That closure
still captured the entire result; it did not achieve the required shape.
No macro expansion, guard, recursion limit or native stack configuration was
changed.

The experiment touched these six application files, all now restored:
`down_values.rs`, `down_values/default_fallback.rs`, `sub_values.rs`,
`up_values.rs`, `rhs.rs`, and (in B(b) only) `mod.rs`. The exact B(a) source
snapshot contains all 1,433 Rust files and matches both native builds' source
hash maps. Its scratch-only probe appendix is retained and unchanged from
the previous native method. It is in `r1/b-a-source/`; the rejected B(b)
application sources and diff are in `r1/b-b-source/` and
`r1/b-b-rejected.patch`. `r1/b-rollback.json` proves the six restored files
equal their redispatch-entry hashes.

The first native debug build failed because the scratch synchronization list
contained only 12 application-root files and missed the nested fallback
module. Its diagnostic was `E0425: cannot find function frame in module
default_fallback`. I retained that failed build under
`stack/candidate/b-a/debug-incomplete`, synchronized the full 1,433-file
manifest, and rebuilt successfully. This setup failure was corrected before
measurement and was not counted as a measured stack attempt.

The candidate has 120 successful samples: five shapes × two profiles × four
heights × three repeats. Debug heights are 2/4/6/8 and release heights are
5/10/15/20. The guard remains 1,048,576 B and the canonical 48-level test is
unchanged. Every candidate fit and every reused frozen-base fit has zero
residual. The frozen base measurements were reused, not rerun during r1.
The new measurements are exclusively for the rejected B(a) source, not the
restored retained source.

| Profile | Shape | Frozen S4 B/level | B(a) B/level | Delta |
|---|---|---:|---:|---:|
| debug | peel | 19152 | 19920 | +768 |
| debug | plus | 34896 | 36080 | +1184 |
| debug | sub | 30352 | 30912 | +560 |
| debug | up | 28528 | 28672 | +144 |
| debug | guarded | 35824 | 37008 | +1184 |
| release | peel | 7312 | 7664 | +352 |
| release | plus | 13744 | 14512 | +768 |
| release | sub | 12928 | 13456 | +528 |
| release | up | 11424 | 11504 | +80 |
| release | guarded | 14192 | 14960 | +768 |

The summed positive excess is **6,336 B**, versus the amendment's 2,320 B;
the failing-pair count is **10**, versus 9. Debug UpValue also violates its
explicit no-growth bar. B(a) is one measured non-progress attempt. No growth
in this table is accepted by this report.

The resident prologues reconcile every measured delta. `plus` and `guarded`
retain two DownValue scans per level; `peel` retains one. The following sizes
exclude the equally present eight-byte return addresses. The two differently
named RHS helpers are paired because they are the actual resident functions.

| Profile | Resident function | Base prologue B | B(a) prologue B | Delta |
|---|---|---:|---:|---:|
| debug | `try_apply_down_values` | 2088 | 2264 | +176 |
| debug | `try_apply_down_values_from_symbol` | 3800 | 4216 | +416 |
| debug | `try_apply_sub_values_from_symbol` | 3656 | 4040 | +384 |
| debug | `try_apply_up_values_from_list` | 2632 | 2600 | -32 |
| debug | RHS: Expr helper → application Option helper | 1336 | 1512 | +176 |
| release | `try_apply_down_values` | 776 | 728 | -48 |
| release | `try_apply_down_values_from_symbol` | 1272 | 1688 | +416 |
| release | `try_apply_sub_values_from_symbol` | 1336 | 1880 | +544 |
| release | `try_apply_up_values` | 1016 | 1112 | +96 |
| release | RHS: Expr helper → application Option helper | 840 | 824 | -16 |

The complete touched-family superset, including individual monomorph
addresses, has 606 profile/function rows in
`r1/stack/b-a-changed-prologues.{json,tsv}`. Every resident frame and call path
is in `b-a-frame-deltas.{json,tsv}` and each profile's `resident-paths.json`.
Unmatched functions are explicitly marked base-only or candidate-only;
they are not silently compared as zero-sized functions.

**B(a) slot evidence.** All offsets below are from `%rsp` after the prologue;
the full disassembly and selected complete functions are preserved beside
the native binaries. Base slot excerpts are also retained in the original
`brief-611/stack/slot-excerpts/`.

| Profile / resident function | Named storage and instructions in B(a) |
|---|---|
| Debug DownValue scan `0xa2a7b0` | Actual matcher result at `0xa38..0xabf`, producer call `0xa2b8f6`. Conversion-closure result at `0xc08..0xc8f` (136 B). Its 320-B capture is application `0xc90..0xd37` (168 B), actual Result `0xd38..0xdbf` (136 B), session at `0xdc0`, scan reference at `0xdc8`; Result copy `0xa2bb33..0xa2bb44`, closure call `0xa2bb94`, remapping call `0xa2bbab`. Other-error temporaries `0xdd0..0xe57` and `0xe58..0xedf`, copies `0xa2bada..0xa2baf9`. |
| Debug SubValue scan `0xa30000` | Actual result at `0x9f8`. Conversion result `0xbc0..0xc47`; 312-B capture: application `0xc48..0xcef`, Result `0xcf0..0xd77`, session at `0xd78`. Result copy `0xa318be..0xa318cf`, closure call `0xa3190f`, remapping call `0xa31926`. Other-error temporaries `0xd80..0xe07` and `0xe08..0xe8f`, copies `0xa31865..0xa31884`. |
| Debug outer DownValue `0x1794d0` | Actual Option producer at `0x290`. Extracted request at `0x390..0x3af`; the macro capture is `0x3b0..0x3ff`: request 32 B, session 8 B, lookup identity 24 B, symbol reference 8 B, Boolean 1 B plus padding. Capture stores `0x17a674..0x17a6e4`; macro cold call `0x17a6f3`. The outer error arm uses `0x488..0x50f` (`0x17a5a3..0x17a5c2`); the macro's other-error arm adds `0x400..0x487` (`0x17a71a..0x17a739`). |
| Debug application RHS `0x180580` | Actual result at `0x258`, intermediate outcome at `0x338`, other-error temporary at `0x430`, finish argument at `0x540` (each 136 B). Error copies `0x180b32..0x180b51`; argument copy `0x180a7c..0x180a88`, finish call `0x180a9c`. Request temporary `0x3e0..0x3ff`; macro capture `0x400..0x427` contains the 32-B request and session reference; cold call `0x180b0b`. The outer profiled Option result is at `0xc8`. |
| Release DownValue scan `0x51e240` | Actual result at `0x3a0`; whole-result copy to conversion capture `0x308..0x38f` at `0x51fc8f..0x51fc97`. Capture application is `0x260..0x307`, session `0x390`, scan reference `0x398`. The conversion closure has been inlined, but its capture stores remain. A separate macro capture begins at `0x180`, with application 168 B, extracted request at `0x228..0x247`, session `0x248`, and scan at `0x250` (`0x520400..0x52044f`). This is not solely the macro's capture. |
| Release SubValue scan `0x52ed00` | The macro capture begins at `0x120`: 168-B application, extracted request at `0x1c8`, session at `0x1e8`; application copy `0x5319c5..0x5319cd`, macro call `0x5319de`, conversion output at `0x480`. Full disassembly retains the separate conversion-result/capture paths. |
| Release UpValue `0x534800` | Macro capture at `0x1a0..0x227`: two Expr values (64 B), extracted request (32 B), owned Vec (24 B), session and index (8 B each). Capture stores `0x5356c0..0x535716`, cold call `0x53572b`. The actual result remains at `0x320`; the other-error forwarding copy is `0x53575e..0x535768`. |
| Release application RHS `0x52d890` | Actual result at `0x110`; payload staging at `0xf0` and `0x230..0x28f`; finish argument at `0x40`, finish output at `0x1a0`; staging/copies `0x52dbee..0x52dcdc`, finish call `0x52dcf5`. Net frame size is 16 B below base, despite those moves. |

The debug DownValue region accounting is exact: the old 208-B cold input
region becomes 456 B (136-B conversion result plus 320-B capture), adding
248 B; the extra error temporary adds 136 B; the region begins 24 B later,
and final rounding contributes another 8 B: **24 + 248 + 136 + 8 = 416 B**.
The 24-B prefix shift is not attributed to a named Rust local or accepted as
macro capture. For debug SubValue, 208 B becomes 448 B (136 + 312), plus
136 B for the second error temporary and 8 B rounding: **240 + 136 + 8 =
384 B**. The 320/312-B conversion captures contain the producer Result and
are **not** the typed macro's permitted extracted-request closure.

For `peel`, the complete resident differences are debug
`+176 lookup +416 scan +176 RHS = +768 B`, release
`−48 lookup +416 scan −16 RHS = +352 B`. For `plus` and `guarded`, they
are debug `176 + 2×416 + 176 = 1184 B`, release
`−48 + 2×416 −16 = 768 B`. These slot and resident tables identify a failed
attempt; they are not an acceptance argument. Release slot reuse/spills and
the complete debug lookup/RHS net-layout attribution were not closed before
the STOP. In particular, I do not label any unexplained net growth a macro
environment cost. Any future retained `peel`/`plus` growth still needs the
director's ruling.

**B(b): compile STOP and rollback.** The next proposed shape added an inline
`DefinitionResumeRoute::failure_result` so the ordinary error arm constructs
its `Result` directly; changed the RHS body to forward the actual non-pause
outcome directly to the finish helper; and tried to force the generic-route
conversion closure inline. The first placement was
`return (#[inline(always)] || { ... })()`. The second placed the same
attribute on the closure in a `let insert_ordinary = ...` initializer.
Both are rejected as experimental expression attributes by Rust 1.85, at
both the DownValue and SubValue sites. No nightly feature or toolchain change
was attempted. These were my introduced compile errors, not pre-existing
errors in the repository.

`b-b-typed.log` (exit 101), verbatim:

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
error[E0658]: attributes on expressions are experimental
   --> crates/rum-runtime/src/runtime/definitions/application/down_values.rs:380:29
    |
380 |                     return (#[inline(always)] || {
    |                             ^^^^^^^^^^^^^^^^^
    |
    = note: see issue #15701 <https://github.com/rust-lang/rust/issues/15701> for more information

error[E0658]: attributes on expressions are experimental
   --> crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:289:33
    |
289 |                         return (#[inline(always)] || {
    |                                 ^^^^^^^^^^^^^^^^^
    |
    = note: see issue #15701 <https://github.com/rust-lang/rust/issues/15701> for more information

For more information about this error, try `rustc --explain E0658`.
error: could not compile `rum-runtime` (lib test) due to 2 previous errors
```

`b-b-typed-closure-attribute.log` (exit 101), verbatim:

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
error[E0658]: attributes on expressions are experimental
   --> crates/rum-runtime/src/runtime/definitions/application/down_values.rs:380:43
    |
380 |                     let insert_ordinary = #[inline(always)] || {
    |                                           ^^^^^^^^^^^^^^^^^
    |
    = note: see issue #15701 <https://github.com/rust-lang/rust/issues/15701> for more information

error[E0658]: attributes on expressions are experimental
   --> crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:289:47
    |
289 |                         let insert_ordinary = #[inline(always)] || {
    |                                               ^^^^^^^^^^^^^^^^^
    |
    = note: see issue #15701 <https://github.com/rust-lang/rust/issues/15701> for more information

For more information about this error, try `rustc --explain E0658`.
error: could not compile `rum-runtime` (lib test) due to 2 previous errors
```

The required Hard rule is: “Stop after two attempts that move none of the
named failing gates.” The two B(b) compile placements had the identical two
errors, so there was no reduction to justify another attempt. Separately,
the one completed native trial lowered neither B stack metric. **B(b) never
produced a measurement binary, and its error-forwarding proposal has no
performance result.** Splitting ordinary and resumed drivers was authorized
but was not attempted before this STOP. Restoring the entry application
files preserved the verified A source state and removed all rejected code;
it was not a third optimization experiment. No acceptance suite was rerun
after that byte-for-byte restoration.

**Commands and all test totals.** There were nine Cargo invocations: three A
test commands, the B(a) typed test, three native build invocations (one setup
failure and two successes), and two rejected B(b) compile attempts. Four
Cargo test invocations reached their test harness, plus 120 native probe
invocations. Complete command/environment/exit records are in
`r1/gate-index.json` and each gate's `.command.json` / `.exit.json`.

| Gate | Command after the common bounded/offline environment | Exit |
|---|---|---:|
| `a-shared-users-after` | `cargo test -p rum-stdlib --lib association_depth_contract -- --nocapture` | 0 |
| `a-shared-users-before` | `cargo test -p rum-stdlib --lib association_depth_contract -- --nocapture` | 101 |
| `a-walker-before` | `cargo test -p rum-stdlib --lib flat_walk_ -- --nocapture` | 101 |
| `b-a-native-debug-build-retry` | `python3 .temp/brief-611/r1/build-native.py candidate debug b-a` | 0 |
| `b-a-native-debug-build` | `python3 .temp/brief-611/r1/build-native.py candidate debug b-a` | 1 |
| `b-a-native-debug-measure` | `python3 .temp/brief-611/r1/measure-native-stack.py candidate debug b-a` | 0 |
| `b-a-native-release-build` | `python3 .temp/brief-611/r1/build-native.py candidate release b-a` | 0 |
| `b-a-native-release-measure` | `python3 .temp/brief-611/r1/measure-native-stack.py candidate release b-a` | 0 |
| `b-a-typed` | `cargo test -p rum-runtime --lib suspension::typed::tests -- --nocapture` | 0 |
| `b-b-typed-closure-attribute` | `cargo test -p rum-runtime --lib suspension::typed::tests` | 101 |
| `b-b-typed` | `cargo test -p rum-runtime --lib suspension::typed::tests` | 101 |

The B(a) typed run emitted the same three pre-existing unused test-helper
warnings as the original `final-typed-api.log`: `register_test_if_builtin`,
`register_test_return_builtin`, and `register_test_vm_template_builtins`.
They are not new warnings from this patch. No oracle build ran in r1, so
there is no new warning-free oracle claim or fault-occurrence delta.

All 124 emitted `test result:` lines follow. Each is labelled with its raw
log; the 120 marker tests are the B(a) native probe, not the canonical
48-level test.

```text
a-shared-users-after.log
test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 1100 filtered out; finished in 1.71s
a-shared-users-before.log
test result: FAILED. 59 passed; 2 failed; 0 ignored; 0 measured; 1100 filtered out; finished in 1.70s
a-walker-before.log
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 1158 filtered out; finished in 0.00s
b-a-typed.log
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.48s
stack/candidate/b-a/debug/marker-guarded-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.04s
stack/candidate/b-a/debug/marker-guarded-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-guarded-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-guarded-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-guarded-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-guarded-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-guarded-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-guarded-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-guarded-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-guarded-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-guarded-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/b-a/debug/marker-guarded-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-peel-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.07s
stack/candidate/b-a/debug/marker-peel-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-peel-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-peel-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-peel-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-peel-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/b-a/debug/marker-peel-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-peel-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-peel-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-peel-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-peel-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-peel-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-plus-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.05s
stack/candidate/b-a/debug/marker-plus-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-plus-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-plus-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-plus-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-plus-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-plus-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-plus-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-plus-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-plus-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-plus-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-plus-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-sub-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.05s
stack/candidate/b-a/debug/marker-sub-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-sub-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-sub-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-sub-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-sub-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-sub-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-sub-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/b-a/debug/marker-sub-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-sub-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/b-a/debug/marker-sub-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-sub-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-up-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.05s
stack/candidate/b-a/debug/marker-up-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-up-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-up-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-up-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-up-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-up-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-up-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-up-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-up-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-up-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/debug/marker-up-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-guarded-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-peel-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-plus-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-sub-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/b-a/release/marker-up-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
```

**Remaining gates, explicitly not run on the retained final source.**

- B acceptance and a final-source stack table; the canonical 48-level test
  and frame roots were not rerun in this redispatch.
- C: `default-unguarded-miss` was not remeasured, and its six-process
  address/caller attribution was not run. The earlier instruction regression
  remains unresolved; the suspect direct-default move is not declared fixed.
- D: the dead SubValue `pending` parameter/branch/doc sentence remain.
  The original application-site count is still 15; the requested removal to
  14 and its pins/stage/census changes are unfinished.
- F correctness: final fmt, typed tests, frame roots, native48, both compile
  control configurations, both census configurations, both local production
  configurations, both ordinary-control configurations, ownership,
  production rejection census, both warning-free oracle builds, native
  ZeroMQ and the full workspace. The A suite and rejected-trial typed result
  above are the only fresh correctness evidence. Native ZeroMQ has no fresh
  result here; its previously reported skips remain skips.
- F replay: ownership discovery, fingerprints, both certificate sets and
  other moved-row receipts. No handwritten ledger or relaxed rejection was
  introduced to skip this work.
- F performance: no-default-feature resume harness; final allocation panel
  and measured candidate frame-call reconciliation; all 107 ordinary memcpy
  comparisons; six-process address proofs for the three leaves; six-process
  per-function attribution for all 32 resume workloads including the owed
  136-B driver copies; and the full instruction panel.
- Timing: none. Four pinned panels and any required five interleaved leaf
  reruns remain unexecuted.

The earlier 107-label allocation equality and the accepted fixture/oracle
results remain archived, but are not substituted for these requested
final-source reruns. The task is left uncommitted and stopped for a director
redispatch, with a verified A repair and preserved evidence of the rejected
B work.

Verified SHA-256 values (also in `r1/binary-verification.json`):

| Binary | SHA-256 |
|---|---|
| `.temp/brief-611/base/rum-cli` | `7881a7149a3c5a5d8e6d792f39d46338b365987d5c02908a65b6f4648d950e80` |
| `.temp/brief-611/base/rum-cli-alloc` | `adf80f3eeebbbe39dcba9b6295191c9462893540a0fdce48215fba3499284b24` |
| `.temp/brief-611/base/resume` | `40f5c75c6b42d698523822959b9f0a39372c5d4700f0bad55e21b8c2f2aade3d` |
| `.temp/brief-611/base/resume-alloc` | `0414b33d5cee14192e786f2ca705ab250c35ccf76b257f5ea69da0dfaece04c5` |
| `.temp/brief-611/r1/stack/base/native/debug/runtime-tests` | `b02963e59ccf6ef3f83f52c6e860c09d63e5e1ca0c072555a9dff1d3a11c4018` |
| `.temp/brief-611/r1/stack/candidate/b-a/debug/runtime-tests` | `cc77a9843957a486cd8f0fc108df2e32d16ca5f716c3a5b5dae8b8d392ea42e6` |
| `.temp/brief-611/r1/stack/base/native/release/runtime-tests` | `778ed46cd14c17c57738529f8e6673b6bc4d674ae447031fe44d7bc27bd2ae2c` |
| `.temp/brief-611/r1/stack/candidate/b-a/release/runtime-tests` | `70e399720a50e9ba2daead4a4385f2cde8196c65490fd4d112b233dee581ce61` |

## Director amendment — redispatch 2: split the ordinary driver, then close the step either way

A is accepted as reported. The walker rule is precise and was watched
failing first. G7 lost its generic `<indirect-call>` and still passes. The
seven callees are audited one line each. I re-run all of it at my gate.

B(a) was a fair measurement and it settles one question. A closure that
converts the generic route's return type always captures the producer's
`Result`. Its 320-B and 312-B captures are the whole cost. So no placement
of the macro inside such a closure can meet the bar.

B(b) was not a shape. Attributes on expressions are unstable on 1.85, which
can be checked before compiling. The shape that Ruling B(a) also authorized,
separate ordinary and resumed drivers, was never tried. That is this
redispatch's only stack attempt.

The run also stopped the whole step on B's stop-loss, leaving C, D and F
unrun for a second time. From here, a stop-loss in B ends B, not the step.

### Ruling

1. **One stack attempt: the split drivers (S4's `arguments.rs` shape).**
   - **The ordinary driver.** For the DownValue scan, the SubValue scan and
     wherever else the generic `Route` forced the conversion closure, the
     ordinary driver is a non-generic function. Its error type is
     `EvalError`, and the caller-place macro sits inline at the producer's
     site with its own `return`. There is no conversion closure, no
     `map_err` on the ordinary path, and no `Result` passed across a call.
   - **The resumed driver.** It keeps the token route and `suspend_again`.
     It is not on the ordinary recursion's resident path. Report its prologue
     anyway.
   - **Shared code goes into helpers that contain no suspension point.**
     Candidate lookup, prefilter, substitution, trace and profiling are
     examples. Do not duplicate a loop body you can factor. Do not route the
     producer `Result` through a helper's return value either; that is the
     same copy moved elsewhere.
   - **The other sites.** For the outer DownValue lookup, UpValue and the
     application-RHS helper, keep run 1's shape unless the split itself
     changes them.
   - **Measure it** on all ten pairs, with the per-slot attribution.
2. **Pre-committed rule.** Keep the split source only if it lowers run 1's
   summed excess (2,320 B) or its failing-pair count (9), without making
   any `peel` or `plus` pair worse than run 1. Otherwise restore run 1's
   application source byte for byte and verify the hashes. Either way, B ends
   there.
   - **Report the retained source's residual** per slot, with the
     `peel`/`plus` table. I rule on it at my gate.
   - **Do not try another shape.** If you see one you believe would work,
     describe it in the report with your reasoning, but do not build it.
3. **Then C, D and F, on the retained source, all of them.**
   - **C** stands as written. Re-measure `default-unguarded-miss` and
     attribute it over six callgrind processes per arm. Remove a copy that
     base did not make.
   - **D** stands as written. Remove the `pending` parameter and its branch
     (15 callers → 14), and follow every pin it moves.
   - **F** stands as written: every correctness gate, every replay, every
     performance section that has not run, and timing last.

   A failing gate in C, D or F is repaired under the brief's normal
   stop-loss, which counts compile errors and failing tests as progress
   when they fall. A stack residual is not a failing gate; it goes to the
   report.
4. **Process.** This run forks your session again. Your tool timeout must
   exceed every `--timeout` you pass. Checkpoint as before. Then append
   `## Execution report — redispatch 2` after this section.


## Execution report — redispatch 2

**All requested redispatch sections and final measurements ran. S5 is not
self-accepted: the stack residual, ordinary default-miss regression, strict
whole-leaf proof limitations and timing indications below remain for the
director.** Every final correctness command passed. The one authorized split
attempt was rejected under the precommitted rule and restored byte for byte;
C was measured and attributed; D removed the dead branch; F ran through timing.

No git command, network operation, sub-agent, new unsafe, repository dependency,
lockfile or toolchain change was made. `typed/witness.rs` is unchanged. Work is
uncommitted. This report supplements the earlier reports and preserves the
entire redispatch-entry brief as its byte-identical prefix. Evidence is under
`.temp/brief-611/r2/`; C's pre-D measurements are preserved in `c-initial/`.
`state.json`, `PROGRESS.md`, command/exit records and raw logs are there.

### Retained files, pins and replay

Relative to the 1,486-file redispatch-2 entry snapshot, these are the six source
or inventory files changed; this appended brief is the sole documentation edit.
The accepted four-file A repair was already present and remains unchanged.

| File | Change |
|---|---|
| `crates/rum-runtime/src/runtime/definitions/application/sub_values.rs` | Delete the unused `pending` parameter, its entire branch, both `None` arguments and the stale resume sentence. Resume still enters `finish_suspended_sub_value_application`. Application RHS calls: 15 → 14 (six default, three DownValue, two SubValue, three UpValue). |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | Discovery replay moves the SubValue frame's derive row 394:1 → 337:1. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | Discovery replay moves the five SubValue rows: 81:27 → 78:27, 247:58 → 191:58, 257:57 → 201:57, 306:21 → 250:21, 411:24 → 354:24. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | Same typed macro: 307:17–310:18 → 251:17–254:18. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | Refresh all 68 source-context fingerprints using the checker formula; all other fields remain unchanged. Every old → new fingerprint is reproduced below. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | Replay the four emitted C01/C02 certificate cells, both hash configurations. Identity, source, preconditions, strength and exclusions remain unchanged. Exact old → new cells are in `certificate-diff.json` and `certificate-replay.patch`. |

Discovery retained the exact 79 host identities, 59 cycles and 7,715 constructs;
no new UnknownHostPayload appeared. All construct movement is paired and
attributed to D. Four adapter witnesses were re-emitted. The frontier's 937
rows are byte-identical to run 1, so no frontier pin changed. The frame-root,
stage and census identities/counts remain unchanged: removing a dead ordinary
branch removes neither a frame nor a macro site. No fixture was deleted.

Evidence: [D's fourteen call sites](../../../.temp/brief-611/r2/d-removal.json),
[ownership replay](../../../.temp/brief-611/r2/ownership-replay.patch),
[certificate cells](../../../.temp/brief-611/r2/certificate-diff.json),
[frontier](../../../.temp/brief-611/r2/frontier-candidate.tsv), and
[final source/binary verification](../../../.temp/brief-611/r2/final-verification.json).

The accepted F/M transitions, two Scan instantiations, ordinary/token site
classification and F0244 Expr / F0244A Option split remain as reported earlier.
The skipped-`Some` transform still runs inside the application RHS frame with
one frame per pause. The original failing-first S4 observations were not redone
or substituted; both final 133-test oracle builds exercised the retained fixtures.
D removes the previously reported guest-unreachable pending site.

Final census output (the same cohort passed in both feature configurations):

```text
S5 CENSUS production frames=518 finalizers=52 implementations=607 typed=37 macros=589 result=368 insert=221
S5 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S5 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=6 insert=4
S5 PARTITION implementations=623 macros=600
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
S5 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
S5 PARTITION implementations=622 macros=600
S5 CENSUS test-only frames=0 finalizers=0 implementations=0 typed=0 macros=0 result=0 insert=0
```

### B — one split attempt, rejected and restored

The attempted ordinary routes used concrete `EvalError`, caller-place macros
at the producer and direct error forwarding. Only the Route genericity was
removed; the required DownValue `Scan` genericity remained. Resumed drivers
used the token and `suspend_again`. Pure preparation helpers factored the
candidate, prefilter, substitution and tracing work without carrying producer
Results. The outer DownValue lookup, UpValue and RHS retained run 1's shape.

The trial changed and then restored these eight existing files beneath
`crates/rum-runtime/src/runtime/definitions/`: `application/down_values.rs`,
`application/sub_values.rs`, `application/mod.rs`, `definitions/destructuring.rs`,
`rule_lhs.rs`, `rule_lhs/continuation.rs`, `rule_lhs/tests/base_cases.rs`, and
`definition_lhs_preparation.rs`. Its six new modules were removed:
`application/down_values/{preparation,resume}.rs`,
`application/sub_values/{preparation,resume}.rs`, and
`definition_lhs_preparation/{drive_step,resumed}.rs`. Each new module was under
10k characters. Full trial source, hashes and the restoration proof are in
`split-source/`, `split-source-hashes.json` and `b-decision.json`.

| Profile / shape | S4 B/level | Split B/level | Split delta |
|---|---:|---:|---:|
| debug peel | 19152 | 20912 | +1760 |
| debug plus | 34896 | 38480 | +3584 |
| debug sub | 30352 | 32048 | +1696 |
| debug up | 28528 | 28288 | -240 |
| debug guarded | 35824 | 39696 | +3872 |
| release peel | 7312 | 7408 | +96 |
| release plus | 13744 | 13856 | +112 |
| release sub | 12928 | 13008 | +80 |
| release up | 11424 | 11584 | +160 |
| release guarded | 14192 | 13856 | -336 |

The split reduced failing-pair count 9 → 8 but increased summed positive
excess 2,320 → 11,360 B, and made debug peel/plus worse. It therefore failed
the conjunction in the keep rule. All eight files were hash-verified against
entry and all application files were restored before C and D. No second shape
was built. The named pure-helper aggregate temporaries, cold closure captures,
other-error slots and every resumed-driver prologue are in
[split slot attribution](../../../.temp/brief-611/r2/stack/slot-attribution.md).
Some compiler spill identities remain unassigned; they are not claimed as
accepted macro cost.

Final-source verification after D gives:

| Profile / shape | S4 B/level | Final B/level | Final delta |
|---|---:|---:|---:|
| debug peel | 19152 | 19360 | +208 |
| debug plus | 34896 | 35376 | +480 |
| debug sub | 30352 | 29600 | -752 |
| debug up | 28528 | 28288 | -240 |
| debug guarded | 35824 | 36304 | +480 |
| release peel | 7312 | 7440 | +128 |
| release plus | 13744 | 13920 | +176 |
| release sub | 12928 | 13152 | +224 |
| release up | 11424 | 11584 | +160 |
| release guarded | 14192 | 14368 | +176 |

All four-height/three-sample fits have zero residual. D improves SubValue stack
use; summed positive excess is now 2,032 B in eight pairs. Peel and plus retain
their run-1 residuals. Debug peel is +272 scan +16 outer lookup −80 RHS = +208;
debug plus is 2×272 +16 −80 = +480. Release peel is +48 scan +80 RHS = +128;
release plus is 2×48 +80 = +176. These include the producer Result crossing a
cold call and the extra error temporary, not just the macro environment.

The final per-slot report names current addresses, SubValue's remaining slots
and all attribution limitations:
[final slot attribution](../../../.temp/brief-611/r2/stack/final-slot-attribution.md),
[whole resident cycles](../../../.temp/brief-611/r2/stack/final-frame-deltas.tsv),
[all owned prologues](../../../.temp/brief-611/r2/stack/final-changed-prologues.tsv).
The unchanged 48-level test passed. No stack residual is accepted here.

### C and F — final ordinary measurements

The original four S4 binary hashes were reverified, and all four CGU1 candidates
were rebuilt from the final source. Every candidate build's complete Rust
source hash map agrees with that source. Equal-depth `base`/`cand` executable
paths avoid a constant argv allocation discrepancy.

| Binary | Frozen S4 SHA-256 | Final SHA-256 |
|---|---|---|
| rum-cli | `7881a7149a3c5a5d8e6d792f39d46338b365987d5c02908a65b6f4648d950e80` | `669b82e9788749f1be6c7d26522bfcd76090a683f44c2f12308b4ee90e79948f` |
| rum-cli-alloc | `adf80f3eeebbbe39dcba9b6295191c9462893540a0fdce48215fba3499284b24` | `563d9bdd0d8528c2205f9df95b4f35d4e29af54e68b86d757a5756609c4ea4fe` |
| resume | `40f5c75c6b42d698523822959b9f0a39372c5d4700f0bad55e21b8c2f2aade3d` | `e325d992080e015f5645d2f8a8b038ff1faca6123ba9513e814d4dcd9febc31a` |
| resume-alloc | `0414b33d5cee14192e786f2ca705ab250c35ccf76b257f5ea69da0dfaece04c5` | `6353758a1ef259b0cbdd43c725ae7ec641f9c766528a45ffeff1f3729cd23841` |

All 139 result checks passed. On all 107 ordinary labels, allocation counts
and bytes match exactly at N and 2N and in slope. All 107 labels also pass the
resolved-libc memcpy gate at N, 2N and in slope, counting every caller. The
target binding is proved through real GOT call sites, not a nearest exported
symbol in stripped libc. See `perf/allocation-comparison.json`,
`ordinary-callgrind/copy-gate.json`, `copy-callers.tsv` and
`attribution/libc-binding.json`.

Two interleaved perf pairs covered all 139 labels. Eleven ordinary labels
were flagged. Five additional interleaved runs per arm cleared ten; the
remaining `default-unguarded-miss` result is base 12,787.03390, candidate
12,801.02240, delta +13.98850 instructions/iteration against a 6.39351695 bar.
This remains a regression. The exact flags, all slopes/ranges and five-run
decisions are in `perf/{instruction-comparison,ordinary-five-run-comparison}.json`.

Six final-source callgrind processes per arm again show the default miss's
dominant increase in libc `bcmp` at 0x1b1220: +24.046035806, SE 0.029858034.
The owned DownValue and default scans contribute deterministic −9 and −1.
The outer DownValue lookup stays exactly 283 instructions/iteration. The
base vector return 0x1b1533 and candidate short-path return 0x1b157a each
execute three times/iteration in every sample, following the page-boundary
test against 0xfe0. The exact compared string/pointer remains unidentified.
All twelve memcpy measurements have slope 12; whole counts are base
19,838/38,606 and candidate 19,837/38,605. The suspect direct-default cold
move does not execute on this miss. There was no added executed copy to
remove, so no speculative C source edit was made.

The three leaves have identical `Session::evaluate` cost, 110 instructions
per iteration in every sample, with zero aligned address changes in that
body. No leaf has a positive deterministic per-function delta. However,
whole-label sample ranges overlap and positive process-variable residues
remain outside one standard error. **The strict whole-label no-growth proof
is not declared green.** All addresses, caller counts, per-function samples
and standard errors are disclosed in
[final ordinary attribution](../../../.temp/brief-611/r2/final-ordinary-attribution.md)
and its linked JSON/TSV files. Synthetic `???` disassembly gaps have zero delta
and are explicitly identified.

### Resume census, allocation and instruction attribution

Both no-default-feature harnesses passed all 32 workloads. The 19 new labels
remain N=100/200. All 32 workloads have six independent callgrind processes
per arm, full per-function/address/caller diffs, and a measured before/after
frame census including calls per actual resume.

All 32 allocation deltas reconcile exactly to one 24-B outer adapter Arc per
newly typed entry plus three allocations/120 B for each new application
`Some` wire. Count and byte intercepts are zero. The latter consists of the
32-B element buffer, 48-B shared head and 40-B shared argument owner; the
source and allocation instruction sites are named in the report.

The unique driver 136-B typed-outcome copy is base 0x72d414, candidate
0x73bde4. Its count equals typed-adapter calls for every label in every sample.
Each newly typed entry adds exactly one such transfer. It is counted and owed
to R; the driver was not changed. The per-label copies and SIMD/rep moves,
including action/input/state/error and helper ABI transfers, are retained
with addresses and source/destination windows. Existing transfers are paired
with their legacy bodies rather than all being labelled new cost.

The full report and combined ledgers are
[resume attribution](../../../.temp/brief-611/r2/final-resume-attribution.md),
[frame calls](../../../.temp/brief-611/r2/resume-callgrind/frame-calls.json),
[allocation reconciliation](../../../.temp/brief-611/r2/perf/allocation-frame-proof.json),
[function deltas](../../../.temp/brief-611/r2/attribution/resume/resume-function-deltas.tsv),
[copy sites](../../../.temp/brief-611/r2/attribution/resume/resume-copy-sites.tsv),
[movement sites](../../../.temp/brief-611/r2/attribution/resume/resume-simd-moves.tsv),
and [driver proof](../../../.temp/brief-611/r2/attribution/resume/driver-copy-proof.json).
Every process-variable delta is reported with its SE; none is silently cleared.

| Workload | New typed entries | New Some wires | Extra allocs / bytes | Added 136-B transfers | Instruction delta | Deterministic part | Variable functions outside 1 SE |
|---|---:|---:|---:|---:|---:|---:|---:|
| resume-table | 0 | 0 | 0 / 0 | 0 | -10.025000 | 0.000000 | 6 |
| resume-do-chain | 0 | 0 | 0 / 0 | 0 | -26.725000 | 0.000000 | 2 |
| resume-call-repeat | 0 | 0 | 0 / 0 | 0 | -76.873333 | 0.000000 | 3 |
| resume-cleanup | 0 | 0 | 0 / 0 | 0 | 1.313333 | 0.000000 | 3 |
| resume-fold | 0 | 0 | 0 / 0 | 0 | -62.870000 | 0.000000 | 6 |
| resume-argument-twice | 0 | 0 | 0 / 0 | 0 | 32.241667 | 0.000000 | 5 |
| resume-rewrite-exprs | 0 | 0 | 0 / 0 | 0 | -211.456667 | 0.000000 | 4 |
| resume-rewrite-compiled | 0 | 0 | 0 / 0 | 0 | -56.198333 | 0.000000 | 2 |
| resume-cases-spec | 1 | 0 | 1 / 24 | 1 | 155.465000 | 166.000000 | 5 |
| resume-cases-traversal | 0 | 0 | 0 / 0 | 0 | -63.893333 | 0.000000 | 5 |
| resume-position-traversal | 0 | 0 | 0 / 0 | 0 | 89.863333 | 0.000000 | 6 |
| resume-count-traversal | 0 | 0 | 0 / 0 | 0 | -71.461667 | 0.000000 | 2 |
| resume-cases-shared | 1 | 0 | 1 / 24 | 1 | 336.975000 | 170.010000 | 5 |
| resume-s5-down-rhs | 1 | 1 | 4 / 144 | 1 | 1854.958333 | 1430.980000 | 8 |
| resume-s5-down-guard | 2 | 1 | 5 / 168 | 2 | 2232.701667 | 1575.260000 | 10 |
| resume-s5-sub-rhs | 1 | 1 | 4 / 144 | 1 | 1828.588333 | 1409.780000 | 4 |
| resume-s5-sub-guard | 2 | 1 | 5 / 168 | 2 | 2345.328333 | 1823.570000 | 6 |
| resume-s5-up-rhs | 1 | 1 | 4 / 144 | 1 | 1909.275000 | 1427.340000 | 6 |
| resume-s5-up-guard | 2 | 1 | 5 / 168 | 2 | 2245.180000 | 1838.000000 | 6 |
| resume-s5-default-rhs | 2 | 1 | 5 / 168 | 2 | 1932.806667 | 1722.730000 | 8 |
| resume-s5-default-guard | 3 | 1 | 6 / 192 | 3 | 2206.573333 | 1870.700000 | 5 |
| resume-s5-default-fallback-rhs | 2 | 1 | 5 / 168 | 2 | 1928.723333 | 1612.700000 | 5 |
| resume-s5-set | 1 | 0 | 1 / 24 | 1 | 133.948333 | 108.000000 | 3 |
| resume-s5-upset | 1 | 0 | 1 / 24 | 1 | 94.881667 | 230.100000 | 3 |
| resume-s5-tagset | 1 | 0 | 1 / 24 | 1 | 226.060000 | 247.100000 | 1 |
| resume-s5-rule-rhs | 1 | 0 | 1 / 24 | 1 | 138.130000 | 117.560000 | 6 |
| resume-s5-rule-lhs | 2 | 0 | 2 / 48 | 2 | 279.010000 | 263.480000 | 3 |
| resume-s5-destructure | 1 | 0 | 1 / 24 | 1 | 336.501667 | 263.640000 | 6 |
| resume-s5-lhs-twice | 2 | 0 | 2 / 48 | 2 | 763.950000 | 660.370000 | 4 |
| resume-s5-own-rhs | 1 | 0 | 1 / 24 | 1 | 166.503333 | 165.000000 | 2 |
| resume-s5-destructure-twice | 2 | 0 | 2 / 48 | 2 | 429.353333 | 501.780000 | 3 |
| resume-s5-rule-lhs-twice | 3 | 0 | 3 / 72 | 3 | 262.018333 | 381.280000 | 4 |

### Timing, last and indicative

Four full pinned panels ran B/C/B/C with profiling disabled, seven repetitions,
scale 1 and retained-module GC batches 3. Each produced 120 in-kernel
`AbsoluteTiming` rows. All preflights observed quiet CPU and all monitored
measurement samples found no competing CPU process above 0.15 cores. GPU
activity was unobservable through the available provider; universal host
quiescence is not claimed.

Median candidate/base ratios were 0.996671875 and 0.999653720. No deterministic
leaf was outside [0.8, 1.25], so the conditional five-run rerun was not required.
The two association key-read labels were about 21–23% slower in both pairs;
those indications remain reported, not hidden by the medians. Timing does
not override the instruction findings.

| Label | Pair 1 candidate/base | Pair 2 candidate/base |
|---|---:|---:|
| literal-integer | 0.986012573 | 1.092480165 |
| do-count-null | 0.972684498 | 0.975839615 |
| do-iterator-null | 0.963230172 | 1.042230373 |
| association-part-key-read-width4000 | 1.216490287 | 1.226468927 |
| association-part-key-read-width32000 | 1.211501791 | 1.219572326 |
| appendto-list-incremental-small | 1.232194117 | 0.904875337 |

Raw `timings.tsv` panels, generated `.rum`, host-monitor records and pairwise
ratios are under `perf/timing-pinned/` and `timing-pinned-*-host.json`.

### Correctness results and warnings

Final fmt, typed API, frame roots, native48, both compile-control/census/local
production/ordinary-control builds, ownership, rejection census, both oracle
builds, native ZeroMQ command and full workspace all exited zero. Compile
controls ran 32 positive/negative pairs per build; ordinary controls checked
28 cases with Checks::ALL and unconditional FaultProbe absence per build.
Both oracle builds passed 133 tests, emitted no compiler warning and ended
with fault-occurrence delta=0, observations=88916; all 84 captured fault
assertions per build are zero.

The native ZeroMQ command's harness says nine passed because its wrappers
return after reporting unavailable loopback. **All nine are skips of native
transport coverage, not passes of that coverage.** The same limitation applies
to the workspace run. The literal skip output is reproduced below.

The runtime test binary reports the three existing dead-code warnings for
`register_test_if_builtin`, `register_test_return_builtin`, and
`register_test_vm_template_builtins`. No warning was introduced by the retained
change; both oracle builds and all four production candidate builds are clean.
Workspace also prints warnings from its deliberate `option-read-proof`
compiler fixtures; raw warnings and context are preserved in the log.

The `FAILED` totals within successful census/workspace logs are expected
forced-red child controls. They are reproduced verbatim, not removed. The
earlier certificate-emission failures are expected stale-certificate checks
after seven zero-rejection rows, followed by emitted-cell replay and green
final acceptance tests.

```text
b-split-fmt-initial
$ cargo fmt --check
exit=1

b-split-format
$ cargo fmt
exit=0

b-split-typed-1
$ cargo test -p rum-runtime --lib suspension::typed::tests
exit=0
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.46s

certificate-emission-default-drop-order
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 18.35s

certificate-emission-no-default-drop-order
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1154 filtered out; finished in 18.16s

f-adapter-witnesses
$ cargo test -p rum-stdlib --lib adapter_production_witnesses_are_computed -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 5.97s

f-discovery
$ cargo test -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 6.06s

f-frontier-replay-template
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/frontier-fixture/Cargo.toml --test frontier checker::r6_production_views_inventory -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 10.27s

final-census-default
$ cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.99s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.98s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 141.40s

final-census-no-default
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.95s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.22s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 139.86s

final-certificate-default
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 18.16s

final-certificate-no-default
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1154 filtered out; finished in 18.14s

final-controls-default
$ cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.90s

final-controls-no-default
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.43s

final-fmt
$ cargo fmt --check
exit=0

final-frame-roots
$ cargo test -p rum-runtime --lib frame_root_trace
exit=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1158 filtered out; finished in 1.36s

final-native-zeromq
$ cargo test -p rum-jupyter --lib native_zeromq -- --nocapture
exit=0
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s

final-native48
$ cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.01s

final-oracle-default
$ cargo test -p rum-stdlib --test suspension_oracle -- --nocapture
exit=0
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.33s

final-oracle-no-default
$ cargo test -p rum-stdlib --no-default-features --test suspension_oracle -- --nocapture
exit=0
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.44s

final-ordinary-default
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/ordinary-controls/Cargo.toml --test ordinary -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

final-ordinary-no-default
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

final-ownership
$ cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
exit=0
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1130 filtered out; finished in 68.96s

final-production-rejections
$ cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 25.50s

final-typed-api
$ cargo test -p rum-runtime --lib suspension::typed::tests
exit=0
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.45s

final-workspace
$ cargo test --workspace --no-fail-fast -- --nocapture
exit=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 37.89s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.07s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 1170 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 37.30s
test result: ok. 1108 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1270.50s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.44s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.60s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.50s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 19.84s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 146.49s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.61s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.59s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.27s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.74s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

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

### Stop-loss, repairs, deviations and remaining decisions

- B: one parse error corrected, then 45 compile errors → 0. The sole measured
  split fails the keep rule and is restored. No second shape, unstable
  expression attribute or witness macro change was tried.
- C: the first analysis misclassified the symlinked base object. Canonicalizing
  both paths repaired the reader; the same raw data was reanalysed. The initial
  invalid analysis is archived and is not evidence. Final bases are copied
  binaries with reverified original hashes.
- D: a scratch call counter initially counted only 12 direct `self.` lines;
  following multiline calls found the actual 14. No source was changed to
  satisfy the counter.
- F frontier scratch harness: duplicate module compilation had one error;
  the first repair left three unresolved imports; restoring the existing S4
  scratch module selection reduced them to zero and the 937-row replay passed.
  Only one repair failed to make progress; canonical checker code was unchanged.
- No-default scratch harness: `--locked` rejected the copied default-feature
  lock before compilation. One repair allowed offline feature selection only
  in that scratch copy. Package/version/source/checksum identities were checked
  to be a subset of the frozen harness lock. Both arms then built and all
  32 workloads passed. The repository manifests/lockfile and frozen harness
  source/lock remain unchanged. The initial failure remains in the gate ledger.
- The trial used `cargo check -p rum-runtime --tests` once while reducing
  compile errors. The oracle was run with the exact two Cargo test commands
  used by the script, bounded separately at 1500 s, to keep scratch output in
  the authorized fence. Persistent shell sessions imposed no shorter deadline
  than `run-bounded.py`. Cargo jobs/test threads were one, offline mode was on,
  and expensive commands and all measurements ran serially.

No mandatory correctness, replay or performance section remains unrun.
Remaining findings are the final eight positive stack pairs (especially all
peel/plus pairs), the +13.9885 default-miss instruction regression, the strict
whole-leaf proof's variable residues, and the indicated key-read timing
slowdowns. B is closed; none of these is self-approved. E's depth defect was
not fixed, tested afresh or worked around. The director owns acceptance and git.

### Exact fingerprint transitions

```tsv
id	before	after
B01	fnv1a64:61ec6da8509828cc	fnv1a64:d84cec17eacedac2
B02	fnv1a64:24b7a4e2536c9b1d	fnv1a64:aaf861f067661c3b
B03	fnv1a64:64c876414d7dcb8a	fnv1a64:015b2428c15389b0
B04	fnv1a64:16d310aad2d33624	fnv1a64:5f479ff0607eff7a
B05	fnv1a64:39d9ba91553d7d09	fnv1a64:3bfd8f1d16e68a97
B06	fnv1a64:96d1aab07ec8eec0	fnv1a64:d3f713603fe426b6
B07	fnv1a64:30c85fd4bb7f0b79	fnv1a64:ea254397fa00cbe7
B08	fnv1a64:bfdfb056ecc56d16	fnv1a64:97f9b04824a6abdc
B09	fnv1a64:ce8d286f635a9718	fnv1a64:081d2d2b9ba2036e
B10	fnv1a64:ff027c5aed91d823	fnv1a64:d3ace321f0cbd031
B11	fnv1a64:d82ac207ecc75c24	fnv1a64:209f514d7a73257a
B12	fnv1a64:8d99a86b3f632c18	fnv1a64:c729ad2777aa986e
I01	fnv1a64:3fce58a5bd501f4c	fnv1a64:b62ed7155786d142
I02	fnv1a64:911b38254d740509	fnv1a64:933f0cb10f1d1297
I03	fnv1a64:75721f8c04be5c58	fnv1a64:d6b6d73dbc53842e
I04	fnv1a64:777013f22ccd14bd	fnv1a64:5d69c08448c1c8db
P01	fnv1a64:9b92851d28290660	fnv1a64:837941d0d679fbd6
P02	fnv1a64:5925c7e58a1a99f3	fnv1a64:02eb6637b0d81061
P03	fnv1a64:b87ecf0ee689c112	fnv1a64:107c9c36d8a18058
P04	fnv1a64:1992c2f7d815b71d	fnv1a64:9fd38005ec0f383b
P05	fnv1a64:c02677d596f9551c	fnv1a64:a47b5b8cf2ed9ab2
P06	fnv1a64:de18cd3bcb92c598	fnv1a64:17a8d1f803da31ee
P070	fnv1a64:1866697daa083f98	fnv1a64:51f66e39e24fabee
P080	fnv1a64:cf06babc546a2063	fnv1a64:829160720b8a18f1
P090	fnv1a64:bf502e6fbc8c875d	fnv1a64:38bc0c0d5447eefb
P100	fnv1a64:8c57a8c58d5dbd54	fnv1a64:902bfac6139d198a
P110	fnv1a64:f41d1e9c2d82604b	fnv1a64:08d4ec77e2ceaa39
P120	fnv1a64:f1d4504dcaeaab36	fnv1a64:559ea35fa0391bfc
P130	fnv1a64:d495b931ea32e579	fnv1a64:8df29cf528b4a5e7
P140	fnv1a64:d9354095d750a085	fnv1a64:c298a00cd72518e3
P150	fnv1a64:f48b6e9a730148fd	fnv1a64:f61a81055a6ce39b
P071	fnv1a64:0e9d354feab91d19	fnv1a64:820dd7e99fe33787
P081	fnv1a64:8a377b8471edfb38	fnv1a64:7d9fc860d16f8a8e
P091	fnv1a64:8b4c26e29d3210a0	fnv1a64:464439abbb7db896
P101	fnv1a64:ec200f6962e5fd4b	fnv1a64:00d7dd4518324739
P111	fnv1a64:0140c89d6543c988	fnv1a64:9a25cf0a27df0ffe
P121	fnv1a64:6fdd582b0e87da97	fnv1a64:b5544ec750df37f5
P131	fnv1a64:b47d3311092757a4	fnv1a64:fcf1c25696d320fa
P141	fnv1a64:95ba7d1c0cbbb200	fnv1a64:92e0bad57d858376
P151	fnv1a64:a980ff8091e3daf6	fnv1a64:fb237b7820b2763c
B13	fnv1a64:24e23e39af65aac0	fnv1a64:6207a6e97080e2b6
R01	fnv1a64:1dd7021694e4b5cd	fnv1a64:7fc8ba9caff04d4b
R02	fnv1a64:c7c3eb7bdfca6f52	fnv1a64:e98180af76751318
R03	fnv1a64:a7cbb673f7171e11	fnv1a64:f73e8e6428bbb67f
R040	fnv1a64:d5e701eb07f723ed	fnv1a64:ab4fbff6e1902e6b
R050	fnv1a64:41209a4b1c6c3f7c	fnv1a64:1c73e40a66e95192
R060	fnv1a64:aa33d11019262535	fnv1a64:d494a6f42b612fb3
R070	fnv1a64:f2a5439e538fba38	fnv1a64:e60d907ab311498e
R080	fnv1a64:3e747ce4fc51ddbd	fnv1a64:246e2977184691db
R090	fnv1a64:4dc83ed58d74bb94	fnv1a64:21d64a1131ed284a
R100	fnv1a64:29ee1a93f753760a	fnv1a64:c680c87b6b293430
R110	fnv1a64:fed0c43fa5d0b13b	fnv1a64:e21e754af04fc149
R041	fnv1a64:0b1ba163c68d69c0	fnv1a64:48410a1387a8a1b6
R051	fnv1a64:4d725c8549ba93dd	fnv1a64:c6de3a22e175fb7b
R061	fnv1a64:25caeefdda0a6726	fnv1a64:26e58a2139c28b4c
R071	fnv1a64:26fd2eb9dbdd88ef	fnv1a64:0a9a4517174f31cd
R081	fnv1a64:4842121a29099254	fnv1a64:4c16641aaf48ee8a
R091	fnv1a64:e4ebef9b6cf39445	fnv1a64:b2bae939a152d923
R101	fnv1a64:f1168df5ab912775	fnv1a64:3c45c0a799f8f773
R111	fnv1a64:07ed3bc6d034abae	fnv1a64:10e49ba1a31c2954
R12	fnv1a64:ed8d5ee2e2185c6e	fnv1a64:941ed1e5c291ed94
R13	fnv1a64:171c4f87ef12675a	fnv1a64:0effd0a4dfd7b060
R14	fnv1a64:a6afb73b6a0e7d54	fnv1a64:aa84093bf04dd98a
R15	fnv1a64:36ea39000506a490	fnv1a64:25909dcea10f4f26
O010	fnv1a64:7f8ec3d664a17f86	fnv1a64:843a4008bc9ad5ac
O0200	fnv1a64:d3fce26a2607c685	fnv1a64:bd6041e125dc3ee3
O011	fnv1a64:ecbc8d9826b7e9b5	fnv1a64:171d637c38f2f433
O0210	fnv1a64:ad57105c2852c8ce	fnv1a64:e7020576ec3b0ff4
```

### All native-probe test totals

The following are the real `test result:` lines for the rejected split and
final-source stack samples. Base samples are frozen earlier evidence and were
hash-verified rather than rerun. The probe guard and stack size are unchanged.

```text
stack/candidate/split/debug/marker-guarded-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.04s
stack/candidate/split/debug/marker-guarded-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-guarded-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-guarded-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-guarded-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-guarded-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-guarded-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-guarded-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-guarded-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-guarded-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-guarded-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-guarded-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/split/debug/marker-peel-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.05s
stack/candidate/split/debug/marker-peel-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-peel-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-peel-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-peel-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-peel-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-peel-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-peel-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-peel-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-peel-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-peel-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-peel-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-plus-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.05s
stack/candidate/split/debug/marker-plus-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-plus-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-plus-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-plus-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-plus-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-plus-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-plus-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-plus-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-plus-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-plus-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/split/debug/marker-plus-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/split/debug/marker-sub-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.05s
stack/candidate/split/debug/marker-sub-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-sub-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-sub-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/split/debug/marker-sub-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/split/debug/marker-sub-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-sub-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-sub-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-sub-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-sub-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-sub-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-sub-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-up-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.05s
stack/candidate/split/debug/marker-up-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-up-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-up-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-up-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-up-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-up-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/split/debug/marker-up-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-up-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-up-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-up-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/debug/marker-up-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-guarded-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-peel-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-plus-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-sub-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/split/release/marker-up-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-guarded-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.05s
stack/candidate/final/debug/marker-guarded-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-guarded-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-guarded-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-guarded-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-guarded-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-guarded-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-guarded-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-guarded-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-guarded-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-guarded-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/final/debug/marker-guarded-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.01s
stack/candidate/final/debug/marker-peel-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.06s
stack/candidate/final/debug/marker-peel-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-peel-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-peel-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-peel-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-peel-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-peel-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-peel-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-peel-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-peel-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-peel-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-peel-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-plus-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.04s
stack/candidate/final/debug/marker-plus-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-plus-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-plus-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-plus-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-plus-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-plus-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-plus-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-plus-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-plus-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-plus-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-plus-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-sub-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.06s
stack/candidate/final/debug/marker-sub-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-sub-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-sub-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-sub-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-sub-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-sub-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-sub-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-sub-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-sub-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-sub-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-sub-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-up-2-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.05s
stack/candidate/final/debug/marker-up-2-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-up-2-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-up-4-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-up-4-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-up-4-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-up-6-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-up-6-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-up-6-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-up-8-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-up-8-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/debug/marker-up-8-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-guarded-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-peel-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-plus-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-sub-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-10-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-10-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-10-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-15-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-15-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-15-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-20-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-20-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-20-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-5-1.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-5-2.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
stack/candidate/final/release/marker-up-5-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
```

## Director amendment — redispatch 3: the witness crosses the cold call, not the `Result`

### Rulings on redispatch 2

- **B's split is closed.** It was measured honestly, rejected under the
  pre-committed rule and restored byte for byte.
- **D is accepted.** The dead `pending` branch is gone (15 → 14 callers),
  and every moved pin is paired.
- **C is attributed at my gate, and needs no source change.** The +24 in
  `bcmp` is glibc's small-size page-cross test. It takes the bitwise OR of
  both operands' page offsets and compares it with `0xfe0`. Both operands
  are `"Default"` literals:
  - the probe, `heads::patterns::DEFAULT`, passed by `default_value_lhs`;
  - the key, from `CANONICAL_CORE_HEADS`.

  | Arm | Probe | Key | OR of page offsets | Path |
  |---|---|---|---|---|
  | base | `0x860b97` | `0x8779db` | `0xb97 \| 0x9db = 0xbdf` (≤ `0xfe0`) | vector |
  | candidate | `0x873bf7` | `0x88ae0b` | `0xbf7 \| 0xe0b = 0xfff` (> `0xfe0`) | page-cross |

  This is read-only data layout, not executed code; your owned functions are
  −10. If this label flags again, report the same OR for the two literals
  and stop there.
- **The key-read timing is cleared at my gate.** I ran width 4000 at
  N = 400/800, five runs per arm, with the redispatch-2 binaries.
  Instructions per iteration are equal (about 19,940 in both arms). Median
  cycles per iteration are base 6,779 and candidate 6,505. Timing is not
  required in this redispatch.
- **The stack is not accepted yet, because one shape is still unwalked.**
  Your slot tables name the cause: each cold helper receives the producer's
  whole `Result`. That shape also needs an impossible state. The tree now
  has twelve `ok(_) => unreachable!` arms, and two `unreachable!` arms in
  `DefinitionResumeRoute`. HEAD has none: S3 and S4 put the macro at the
  producer. No run has tried the shape the design itself describes. The
  witness is a ZST, borrowed from the actual producer `Result` before any
  transformation, and it is what crosses the call. The `Result` never does.

### Ruling

1. **One shape: the witness and the request cross the call.** Apply it at
   every S5 site where a producer `Result` now crosses into a cold helper or
   a route method. Today those are:
   - `application/mod.rs`: `DefinitionResumeRoute::suspend_again`, both
     impls;
   - `application/rhs.rs`: both cold RHS helpers;
   - `application/down_values.rs`: `suspend_down_value_application`;
   - `application/down_values/default_fallback.rs`: `suspend`;
   - `application/sub_values.rs`, `application/default_values.rs` and
     `application/up_values.rs`: their cold scan helpers;
   - `definition_lhs_preparation.rs`: the three LHS branches;
   - `rule_lhs.rs`: the cold helper;
   - `definitions/destructuring.rs`: the cold helper.

   The shape at each site:
   - **At the producer.** Take `let witness = slot_witness(&actual);`
     immediately after the producer returns, before any match. The
     `InputRequired { request }` arm extracts the request and passes
     `(witness, request)` together with the frame's parts. The `Ok` arm and
     the other-error arm stay at the site. Other errors are forwarded once.
   - **The route method.** Replace `suspend_again` with one total method:

     ```rust
     fn suspend(
         self,
         session: &mut Session,
         witness: SlotWitness<F::Input>,
         request: RuntimeInputRequest,
         fresh: F,
     ) -> Result<F::Output, Self::Error>
     ```

     - The ordinary route `()` is `witness.suspend(session, request, fresh)`.
     - The token route is `Err(self.suspend(session, witness, request, fresh))`.
     - A cold helper that is generic over `Route` calls this method. It has
       no `Route::ORDINARY` branch and no closure.
     - A helper that is not generic over `Route` calls
       `witness.suspend(self, request, frame)` directly. The RHS helpers and
       the default fallback are examples.
   - **Afterwards.** The tree has no `ok(_) => unreachable!` arm, and the
     route trait has no `unreachable!`. If a site cannot take this shape,
     leave that site as it is and report why. Do not build a third shape.
   - **Constraints, unchanged.** `witness.rs` stays as it is. Nothing is
     constructed on the ordinary path: the witness is a ZST, and the frame
     is built only inside the cold helper.
   - **Before editing**, snapshot the current application source with
     hashes under `.temp/brief-611/r3/entry/`. Redispatch 2's F evidence
     belongs to that source, so I can restore it exactly at my gate.
2. **The census follows the insertion route.** The census now proves that
   each owned S5 `M` receipt has a `resume_typed_or_suspend` use. Change it
   to accept either of two forms:
   - that macro at the receipt's identity; or
   - a witnessed insertion, meaning a `suspend` whose witness argument is
     `slot_witness(&x)` taken from the same binding `x` that the site
     matches on.

   It must still refuse a receipt with neither form. Watch it refuse twice
   in a scratch copy: once for a site with no insertion, and once for a
   witness taken from a different binding.

   The printed TOTAL and SPLIT lines and the token-route suffixes may change
   with the route. They are pins that follow code: disclose each one as
   old → new.
3. **Audits and pins.**
   - **G4 `OUTBOUND` and G7.** Add the new callees with one-line audits.
     Remove the edges that become dead, for example `suspend_again`, the
     typed macro's closure edges and `map_err`.
   - **Ownership and retention inventories.** Replay them only. A row the
     new route method introduces, such as its by-value `fresh: F`, is an
     attributed S5 change. Give it the disposition of the nearest existing
     equivalent (`ContinueToken::suspend` or `SlotWitness::suspend`), and
     disclose it. An unpaired removal is still a STOP.
4. **Measure the ten pairs against the current tree** (run 1 with D, the
   final table above). Attribute each pair per slot.
   - **Keep the shape whatever the bytes are, and run F on it.** It removes
     the impossible states even if the stack does not move.
   - **If any pair is worse than the current tree,** say so with its slot
     table. I will choose between this tree and the `entry/` snapshot at my
     gate.
   - **The debug-only second error temporary** may survive this shape.
     Attribute it, and do not chase it.
5. **F on the retained source.**
   - **Correctness:** every correctness command in redispatch 2's ledger.
   - **Allocations:** all 107 ordinary labels at N and 2N.
   - **Instructions:** the 139-label perf panel, with five runs per arm for
     any flagged label.
   - **The resume census:** all 32 workloads, six callgrind processes per
     arm, with the allocation reconciliation.
   - **The memcpy gate.**
   - **Timing** is not required.
   - **C** is closed, as ruled above.
6. **Process.** This run forks your session. Your tool timeout must exceed
   every `--timeout` you pass. Scratch and checkpoints go under
   `.temp/brief-611/r3/`. Append `## Execution report — redispatch 3` after
   this section.


## Execution report — redispatch 3

The required witness/request shape is retained at every owned site, and F has finished on that source. All final correctness gates pass. All 107 ordinary allocation and memcpy gates pass; all 13 initially flagged ordinary instruction labels clear after five extra runs per arm. All 32 resume allocation deltas reconcile, and the driver transfer count is proved for every workload and sample.

**Director decisions remain.** Release SubValue grows by **16 B/level** against the r2-final tree: its scan grows 48 B and its application RHS shrinks 32 B. The shape is kept as ordered. The three leaf evaluate bodies are instruction-identical, but the strict whole-label no-growth proof is not established; the two loop leaves have six positive variable function residues outside one standard error. The resume census also reports every variable residue rather than declaring it zero. No stack or performance acceptance is claimed on the director’s behalf.

No git command, network request or sub-agent was used. The tree is left uncommitted. No new unsafe, dependency, root manifest, lockfile, toolchain, slot-mode, decoder or witness.rs change was made. Timing was not run, as the redispatch-3 amendment directs; C did not flag and remains closed. Redispatch-2 D remains applied.

### Redispatch 3 file disclosure

Paths below are relative to the repository root. Changes are relative to the r3 entry snapshot (r2 final); the earlier S5 repair is retained. The brief itself receives the requested execution report.

| File | Semantic change / pin transition |
|---|---|
| `crates/rum-runtime/src/runtime/definitions/application/default_values.rs` | The guard producer creates its witness immediately; both cold scan helpers carry witness/request and retain ordinary/token routing. |
| `crates/rum-runtime/src/runtime/definitions/application/down_values.rs` | Both direct-default and matcher producers match locally; their cold calls receive witness/request. The generic Scan and existing profiling remain. |
| `crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs` | Cold fallback receives SlotWitness<Option<Expr>> and RuntimeInputRequest, builds its frame there, and calls witness.suspend. |
| `crates/rum-runtime/src/runtime/definitions/application/mod.rs` | Replace suspend_again and ORDINARY branching with the total suspend method; () delegates to witness.suspend and ContinueToken delegates to its inherent suspend. |
| `crates/rum-runtime/src/runtime/definitions/application/rhs.rs` | Both Expr and Option<Expr> RHS producers retain Ok/error matching at the producer; cold helpers take witness/request. The Some semantic transform remains on both paths. |
| `crates/rum-runtime/src/runtime/definitions/application/sub_values.rs` | The matcher producer owns the match and forwards witness/request to the cold helper. Redispatch 2 D stays applied. |
| `crates/rum-runtime/src/runtime/definitions/application/up_values.rs` | The matcher producer and resumed scan take the witness before profiling/matching; both cold scan helpers carry witness/request. |
| `crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation.rs` | All three LHS producers take immediate witnesses. One cold helper builds DefinitionLhsPreparationSuspensionFrame and invokes the total route. |
| `crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs` | The producer takes the witness and extracts the request; the cold helper builds and inserts through the total route. |
| `crates/rum-runtime/src/runtime/definitions/mod.rs` | Authorized re-export ripple: application::* -> application::{ReplaceAllTraversalOutcome, RewriteRulePlan}. This stops DefinitionResumeRoute::suspend from becoming a competing imported method in the retention probe; no checker change. |
| `crates/rum-runtime/src/runtime/definitions/rule_lhs.rs` | The producer owns the match; cold insertion takes witness/request. An empty stack forwards InputRequired once through Route::failure. |
| `crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs` | G4 OUTBOUND removes dead map_err, suspend_again, resume_typed_or_suspend edges and adds slot_witness and suspend with one-line audits. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` | Discovery replay of moved adapter-witness coordinates; all 79 payload identities/dispositions and 59 cycles retained. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | Discovery replay of moved owned derive locations. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | Discovery replay of moved closures and removal of the seven ordinary-route wrapper closures replaced by total routing. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | Discovery replay removes twelve replaced typed macro uses and two impossible route unreachable macros; remaining moved uses are paired. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | Replay 68 fingerprint fields; the other 18 columns stay unchanged. Context fingerprint becomes fnv1a64:b27fcfb9c5bd127f. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | Replay four C01/C02 certificate cells (both feature builds); all seven other columns unchanged. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | Replay twelve M identities from cold-helper macro sites to the actual producer witnessed insertion sites, result -> witnessed. F rows and the other six S5 M receipts unchanged. |
| `crates/rum-stdlib/tests/typed_slot_census.rs` | Register witnessed proof module, use r3 scratch paths, and add the required missing-insertion, wrong-binding, and shadowed-witness scratch refusal checks. |
| `crates/rum-stdlib/tests/typed_slot_census/receipts.rs` | Accept a witnessed M receipt only at its proved identity while preserving the macro route and frame checks. |
| `crates/rum-stdlib/tests/typed_slot_census/reporting.rs` | Report proved witnessed sites and updated totals alongside macro sites. |
| `crates/rum-stdlib/tests/typed_slot_census/s5.rs` | Require the complete S5 receipt set, enforce macro-or-witnessed insertion, update route suffix to ::suspend, and print the new TOTAL; SPLIT unchanged. |
| `crates/rum-stdlib/tests/typed_slot_census/scan.rs` | Retain function signature/body AST and cold status for witnessed flow verification. |
| `crates/rum-stdlib/tests/typed_slot_census/scan/sites.rs` | Populate the function AST facts used by witnessed verification. |
| `crates/rum-stdlib/tests/typed_slot_census/witnessed.rs` | New 9616-character module proves immediate slot_witness(&actual), the match on that same binding, the InputRequired request binding, and unchanged propagation through named cold helpers to insertion. Rejects shadows, unrelated witnesses, uncalled closures, cycles and missing insertion. |

G7 was examined and its 18 checks passed without an inventory edit. The retention frontier remains byte-identical (937 rows). There is no new host-payload disposition, no unpaired ownership removal, no changed checker rejection, and no change to witness.rs or the core representation. Exact old/new row data are in receipt-transitions.json, ownership-pairs.json, ownership-replay.patch, contract-fingerprint-diff.tsv, and certificate-diff.json.

The additional changed document is this brief, by append only. The complete entry snapshot has 1,486 files and matches all 1,433 r2-final Rust hashes. The retained source has 1,434 Rust files. All 13 protected build inputs and the explicitly protected files match their entry hashes. The new witness checker is 9,616 characters; the existing large LHS file shrinks from 32,755 to 31,371 characters. No new module exceeds 10k.

Review artifacts: [entry/hashes.json](../../../.temp/brief-611/r3/entry/hashes.json), [retained-source-hashes.json](../../../.temp/brief-611/r3/retained-source-hashes.json), [implementation.patch](../../../.temp/brief-611/r3/implementation.patch), [implementation-files.json](../../../.temp/brief-611/r3/implementation-files.json), [file-sizes.json](../../../.temp/brief-611/r3/file-sizes.json), [final-scope-audit.json](../../../.temp/brief-611/r3/final-scope-audit.json).

### Redispatch 3 census and insertion routes

All 15 S5 frame implementations, the F0244/F0244A split, the generic F0239 Scan parameter, slot modes and the skipped-Some repair are retained from r2. Twelve M receipts move from a macro inside a cold helper to the actual producer call carrying its witness and extracted request. The six other S5 M receipts retain the typed macro.

F0239 retains both concrete Scan instantiations, IndexedDownValueScan and LinearDownValueScan. Each uses the ordinary route () on entry and the corresponding ContinueToken on resume. There is still one frame per pause. The fifteen skipped-Some fixture histories and the inherited transport-map removals remain as recorded in the earlier S5 reports; this redispatch changes their insertion plumbing, and reruns both complete oracle builds on it.

The ordinary route calls witness.suspend; a generic helper calls the total DefinitionResumeRoute::suspend, whose ContinueToken implementation returns Err(token.suspend(...)). The cold helper constructs the frame. Ok and other errors stay at the producer; no whole producer Result crosses these cold calls. witness.rs remains byte-identical.

| Receipt | Old anchor | New producer anchor |
|---|---|---|
| M0237 | `application::default_values::<Session as inherent>::suspend_exact_default_value_scan#macro1:resume_typed_or_suspend:result` | `application::default_values::<Session as inherent>::evaluate_first_exact_default_value_from_cursor#method10:suspend_exact_default_value_scan_from_owned:self` |
| M0238 | `application::down_values::default_fallback::suspend#macro1:resume_typed_or_suspend:result` | `application::down_values::<Session as inherent>::try_apply_down_values#call19:suspend:` |
| M0239 | `application::down_values::<Session as inherent>::suspend_down_value_application#macro1:resume_typed_or_suspend:result` | `application::down_values::<Session as inherent>::try_apply_down_values_from_symbol#method37:suspend_down_value_application:self` |
| M0245 | `application::rhs::<Session as inherent>::suspend_definition_rhs_evaluation#macro1:resume_typed_or_suspend:result` | `application::rhs::<Session as inherent>::evaluate_definition_rhs#method3:suspend_definition_rhs_evaluation:self` |
| M0246 | `application::sub_values::<Session as inherent>::suspend_sub_value_application#macro1:resume_typed_or_suspend:result` | `application::sub_values::<Session as inherent>::try_apply_sub_values_from_symbol#method24:suspend_sub_value_application:self` |
| M0247 | `application::up_values::<Session as inherent>::suspend_up_value_scan#macro1:resume_typed_or_suspend:result` | `application::up_values::<Session as inherent>::try_apply_up_values_from_list#method20:suspend_up_value_scan_from_owned:self` |
| M0260 | `definition_lhs_preparation::<Session as inherent>::drive_definition_lhs_preparation#macro1:resume_typed_or_suspend:result` | `definition_lhs_preparation::<Session as inherent>::drive_definition_lhs_preparation#method2:suspend_definition_lhs_preparation:self` |
| M0261 | `definition_lhs_preparation::<Session as inherent>::drive_definition_lhs_preparation#macro2:resume_typed_or_suspend:result` | `definition_lhs_preparation::<Session as inherent>::drive_definition_lhs_preparation#method10:suspend_definition_lhs_preparation:self` |
| M0262 | `definition_lhs_preparation::<Session as inherent>::drive_definition_lhs_preparation#macro3:resume_typed_or_suspend:result` | `definition_lhs_preparation::<Session as inherent>::drive_definition_lhs_preparation#method21:suspend_definition_lhs_preparation:self` |
| M0267 | `definitions::destructuring::suspend_destructuring#macro1:resume_typed_or_suspend:result` | `definitions::destructuring::<Session as inherent>::drive_list_destructuring#call2:suspend_destructuring:` |
| M0278 | `rule_lhs::<Session as inherent>::suspend_rule_lhs#macro1:resume_typed_or_suspend:result` | `rule_lhs::<Session as inherent>::drive_rule_lhs#method2:suspend_rule_lhs:self` |
| M0245A | `application::rhs::<Session as inherent>::suspend_definition_application_rhs_evaluation#macro1:resume_typed_or_suspend:result` | `application::rhs::<Session as inherent>::evaluate_definition_application_rhs#method3:suspend_definition_application_rhs_evaluation:self` |

For each row above, shape is result -> witnessed. Exact identities, files and all unchanged receipt fields are in receipt-transitions.json. M0238 moves from down_values/default_fallback.rs to its producer in down_values.rs; the other receipts stay in their source files. M0260/M0261/M0262 share the new cold LHS helper. The unchanged macro receipts are M0263, M0264, M0268, M0269, M0270 and M0271.

Pins relative to entry: production implementations 607 -> 607; typed implementations 37 -> 37; production macros 589 -> 577 (result 368 -> 356; insert 221 -> 221); test-only implementations 16 -> 16 and macros 11 -> 11; combined implementations 623 -> 623, macros 600 -> 588. Typed macros 42 -> 30 plus 12 proved witnessed sites. The token-route suffix ::suspend_again -> ::suspend. SPLIT is unchanged.

Old TOTAL:
```text
S5 TOTAL typed=37 typed_macros=42; modes unchanged; preflip remains refused
```

Final real output:
```text
S5 CENSUS production frames=518 finalizers=52 implementations=607 typed=37 macros=577 result=356 insert=221
S5 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S5 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=6 insert=4
S5 PARTITION implementations=623 macros=588
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
S5 TOTAL typed=37 typed_macros=30 witnessed_sites=12; modes unchanged; preflip remains refused
```

#### Refusal evidence

Both feature builds passed 8 tests, with one ignored probe that the existing scratch falsifiers invoke explicitly. Missing insertion and a witness from a different producer binding are refused at M0245. Review added a cold-helper shadowing falsifier: it first failed against the old checker because the mutated source was accepted (review-shadow-before, unwrap_err on Ok). After adding the conservative rebinding guard, it passed by refusing that same mutation. The final checker is 9616 characters, below the 10k limit.

```text
FORCED RED no-insertion: M0245: witnessed typed insertion absent
FORCED RED wrong-binding: M0245: witnessed typed insertion absent
FORCED RED shadowed-witness: M0245: witnessed typed insertion absent
```

The proof checks the immediate producer/witness local pair, the match on that same binding, the InputRequired request binding, and unchanged witness/request propagation through named cold helpers to insertion. Rebindings in a matched arm or helper are refused, as are uncalled closures, cyclic helper paths, missing insertion and ambiguous helper resolution. Ordinary and token paths share the verified cold insertion helpers. No preflip or zero-legacy claim is made.

The twelve impossible success arms in the replaced S5 cold helpers and the two route unreachable arms are removed. Existing unrelated LHS/destructuring invariants are unchanged. No whole producer Result crosses these cold calls. The fresh frame is constructed only inside the cold helper; the ordinary and token route methods are total.

The fifteen inherited fail-first histories are retained in [Skipped-Some repair and fail-first fixtures](#3-skipped-some-repair-and-fail-first-fixtures). This redispatch changes no fixture, Checks flag or fault-absence contract. Both complete oracle builds were rerun on the final source, as recorded below. The inherited workload definitions are in [Harness workloads and other oracle fixtures](#4-harness-workloads-and-other-oracle-fixtures). Both fresh no-default-feature harnesses pass all 32 workloads; their raw results are [S4 results](../../../.temp/brief-611/r3/harness-no-default/base/results.json) and [candidate results](../../../.temp/brief-611/r3/harness-no-default/candidate/results.json).

### Redispatch 3 audits and inventory replay

G4 OUTBOUND removes map_err, suspend_again and resume_typed_or_suspend and adds slot_witness and suspend with one-line audits. G7 was examined and all 18 checks passed; its source did not need an edit. The module-root re-export is narrowed from application::* to application::{ReplaceAllTraversalOutcome, RewriteRulePlan}; this prevents the newly named route method from becoming a competing imported method in the retention probe. The retention checker is unchanged.

Ownership discovery keeps all 79 normalized host-payload identities and their dispositions and all 59 self-reaching types. Constructs change 7715 -> 7694: twelve replaced typed macro sites, seven ordinary-route closures and two impossible route arms disappear. All 58 coordinate movements are paired, the 21 eliminations are attributed to S5, and there is no new host payload or unpaired removal. The by-value route frame produces no new inventory identity. Exact old/new rows and dispositions are in ownership-pairs.json and ownership-replay.patch.

The retention frontier remains byte-identical: 937 rows, SHA-256 91672dfb87f6143c95c234387d8ed61dc5f38260c7a5a0a7198b782f4599028b. No frontier disposition or rejection rule changed. The final production rejection census retains roots=7346, definitions=7346, raw=48456, deduplicated=48456 and reason classes=60. Its printed OPTION_FRONTIER=927 is the pre-existing historical label; the canonical inventory has 937 rows.

The global contract context now hashes to fnv1a64:b27fcfb9c5bd127f. Every .rs/.py/.toml under crates participates, including the two census files changed during hardening. Only the 68 fingerprint fields below changed; all other 18 contract columns are unchanged. Both final certificate emissions produced seven zero-rejection rows. Four C01/C02 cells were replayed from those emissions, leaving the other seven certificate columns unchanged. Intermediate replay artifacts remain under review/.

| Contract | Entry fingerprint | Retained fingerprint |
|---|---|---|
| B01 | fnv1a64:d84cec17eacedac2 | fnv1a64:44e9e9ae5732c9d0 |
| B02 | fnv1a64:aaf861f067661c3b | fnv1a64:375d1a5b5d197599 |
| B03 | fnv1a64:015b2428c15389b0 | fnv1a64:3fce90dcc88713de |
| B04 | fnv1a64:5f479ff0607eff7a | fnv1a64:f70fd72437a7f118 |
| B05 | fnv1a64:3bfd8f1d16e68a97 | fnv1a64:487a184d8ab3f645 |
| B06 | fnv1a64:d3f713603fe426b6 | fnv1a64:df1d68f399d5b984 |
| B07 | fnv1a64:ea254397fa00cbe7 | fnv1a64:3a282f979cd9d995 |
| B08 | fnv1a64:97f9b04824a6abdc | fnv1a64:fa9874031f46ea8a |
| B09 | fnv1a64:081d2d2b9ba2036e | fnv1a64:4ddf77400e78d76c |
| B10 | fnv1a64:d3ace321f0cbd031 | fnv1a64:1227b8d516deb13f |
| B11 | fnv1a64:209f514d7a73257a | fnv1a64:b8678881519c1718 |
| B12 | fnv1a64:c729ad2777aa986e | fnv1a64:0cebf73bea816c6c |
| I01 | fnv1a64:b62ed7155786d142 | fnv1a64:22cbd4abc3eac050 |
| I02 | fnv1a64:933f0cb10f1d1297 | fnv1a64:9fbb95e182ea7e45 |
| I03 | fnv1a64:d6b6d73dbc53842e | fnv1a64:0520af80e6e8b72c |
| I04 | fnv1a64:5d69c08448c1c8db | fnv1a64:2d746c9ba77b7139 |
| P01 | fnv1a64:837941d0d679fbd6 | fnv1a64:66fbf48f412c8ba4 |
| P02 | fnv1a64:02eb6637b0d81061 | fnv1a64:51706cd4c516096f |
| P03 | fnv1a64:107c9c36d8a18058 | fnv1a64:888be0564eac23f6 |
| P04 | fnv1a64:9fd38005ec0f383b | fnv1a64:2c383870e1c29199 |
| P05 | fnv1a64:a47b5b8cf2ed9ab2 | fnv1a64:552e56edc8191240 |
| P06 | fnv1a64:17a8d1f803da31ee | fnv1a64:5d6b1c0c76b105ec |
| P070 | fnv1a64:51f66e39e24fabee | fnv1a64:97b8b84e55267fec |
| P080 | fnv1a64:829160720b8a18f1 | fnv1a64:790150160e33bfff |
| P090 | fnv1a64:38bc0c0d5447eefb | fnv1a64:f159872b70926c59 |
| P100 | fnv1a64:902bfac6139d198a | fnv1a64:6f41e6eeceb771a8 |
| P110 | fnv1a64:08d4ec77e2ceaa39 | fnv1a64:be9ad46e19a48617 |
| P120 | fnv1a64:559ea35fa0391bfc | fnv1a64:1b8d9c21cb87fbaa |
| P130 | fnv1a64:8df29cf528b4a5e7 | fnv1a64:ddf588f4cb8db395 |
| P140 | fnv1a64:c298a00cd72518e3 | fnv1a64:6072122428d433b1 |
| P150 | fnv1a64:f61a81055a6ce39b | fnv1a64:bf7a2702166665f9 |
| P071 | fnv1a64:820dd7e99fe33787 | fnv1a64:847c37349e0af3b5 |
| P081 | fnv1a64:7d9fc860d16f8a8e | fnv1a64:010250404c10c60c |
| P091 | fnv1a64:464439abbb7db896 | fnv1a64:4788d057b492d664 |
| P101 | fnv1a64:00d7dd4518324739 | fnv1a64:b69dc53b4f082317 |
| P111 | fnv1a64:9a25cf0a27df0ffe | fnv1a64:4719a0d6831c693c |
| P121 | fnv1a64:b5544ec750df37f5 | fnv1a64:cc12c38f9283a423 |
| P131 | fnv1a64:fcf1c25696d320fa | fnv1a64:94b9f98a6dfc1298 |
| P141 | fnv1a64:92e0bad57d858376 | fnv1a64:d50e886532666944 |
| P151 | fnv1a64:fb237b7820b2763c | fnv1a64:b7533790e61ca0ea |
| B13 | fnv1a64:6207a6e97080e2b6 | fnv1a64:6d2dfc7cca727584 |
| R01 | fnv1a64:7fc8ba9caff04d4b | fnv1a64:0b744d91a98e15a9 |
| R02 | fnv1a64:e98180af76751318 | fnv1a64:c4c18104340467b6 |
| R03 | fnv1a64:f73e8e6428bbb67f | fnv1a64:700b7fb962a32b7d |
| R040 | fnv1a64:ab4fbff6e1902e6b | fnv1a64:1f57a7ae01de3449 |
| R050 | fnv1a64:1c73e40a66e95192 | fnv1a64:8939ad35ee824b20 |
| R060 | fnv1a64:d494a6f42b612fb3 | fnv1a64:df13effd16872901 |
| R070 | fnv1a64:e60d907ab311498e | fnv1a64:6970185a2db2850c |
| R080 | fnv1a64:246e2977184691db | fnv1a64:f478d58e77003a39 |
| R090 | fnv1a64:21d64a1131ed284a | fnv1a64:d430345d4ea8ac68 |
| R100 | fnv1a64:c680c87b6b293430 | fnv1a64:04f4352f725cbe5e |
| R110 | fnv1a64:e21e754af04fc149 | fnv1a64:dc7ba34a96557d27 |
| R041 | fnv1a64:48410a1387a8a1b6 | fnv1a64:53675fa6e19a3484 |
| R051 | fnv1a64:c6de3a22e175fb7b | fnv1a64:7f7bb540fdc078d9 |
| R061 | fnv1a64:26e58a2139c28b4c | fnv1a64:510efb9e0db1f73a |
| R071 | fnv1a64:0a9a4517174f31cd | fnv1a64:3ee38dcf05b86e8b |
| R081 | fnv1a64:4c16641aaf48ee8a | fnv1a64:2b2c50436a6346a8 |
| R091 | fnv1a64:b2bae939a152d923 | fnv1a64:4a23ddafb2eb5ef1 |
| R101 | fnv1a64:3c45c0a799f8f773 | fnv1a64:5f4016ef495e02c1 |
| R111 | fnv1a64:10e49ba1a31c2954 | fnv1a64:6ec779229b08c132 |
| R12 | fnv1a64:941ed1e5c291ed94 | fnv1a64:c3e0b4c6a9030872 |
| R13 | fnv1a64:0effd0a4dfd7b060 | fnv1a64:e5228f8170cfee4e |
| R14 | fnv1a64:aa84093bf04dd98a | fnv1a64:8999f564ab6831a8 |
| R15 | fnv1a64:25909dcea10f4f26 | fnv1a64:a4e7be52107a1f34 |
| O010 | fnv1a64:843a4008bc9ad5ac | fnv1a64:12b75408c0f89e9a |
| O0200 | fnv1a64:bd6041e125dc3ee3 | fnv1a64:5b39b3f8778b59b1 |
| O011 | fnv1a64:171d637c38f2f433 | fnv1a64:219cac852418ed81 |
| O0210 | fnv1a64:e7020576ec3b0ff4 | fnv1a64:7172d6eea3d28952 |

The certificate table lists all body fingerprints and instance marks. The full four old/new cells, including each direct-contract fingerprint, are preserved in certificate-diff.json and certificate-replay.patch.

| Certificate / feature | Body fingerprint old -> new | Instance marks old -> new |
|---|---|---|
| C01 hash=true | fnv1a64:a8213b4a1c564752 -> fnv1a64:4a7518d46d0e6829 | fnv1a64:91c1aca7ec803066 / fnv1a64:93226f679c70ebf4 -> fnv1a64:c2e6456886d5597b / fnv1a64:abb40cd8a4a6ed85 |
| C01 hash=false | fnv1a64:0adfd107d478087d -> fnv1a64:1b67d5871cecf831 | fnv1a64:cf936ae90fca28fc / fnv1a64:c0ed60a5f6b724f6 -> fnv1a64:be550c8b2ffec4bd / fnv1a64:c0357c69ebf12d5f |
| C02 hash=true | fnv1a64:a4cba7438e7f979c -> fnv1a64:fa1e4d38afa3afe7 | fnv1a64:47aa91bf05534a81 -> fnv1a64:d42dff5111f1bd4c |
| C02 hash=false | fnv1a64:6e3e09e954537db3 -> fnv1a64:21173f3b2ae911df | fnv1a64:6078f6324c3cedb7 -> fnv1a64:5065a3cb49f8891a |

Exact row evidence: [receipt-transitions.json](../../../.temp/brief-611/r3/receipt-transitions.json), [ownership-pairs.json](../../../.temp/brief-611/r3/ownership-pairs.json), [ownership-replay.patch](../../../.temp/brief-611/r3/ownership-replay.patch), [contract-fingerprint-diff.tsv](../../../.temp/brief-611/r3/contract-fingerprint-diff.tsv), [certificate-diff.json](../../../.temp/brief-611/r3/certificate-diff.json), [certificate-replay.patch](../../../.temp/brief-611/r3/certificate-replay.patch), [frontier-verification.json](../../../.temp/brief-611/r3/frontier-verification.json).

### Redispatch 3 final correctness

Every final correctness command exited zero. The full workspace passed with runtime 1170 passed/3 ignored and stdlib 1108 passed/53 ignored. Both compile-control builds checked 32 positive/negative pairs; both ordinary-control builds checked 28 cases with the existing complete checks and fault-absence assertions.

Both oracle builds passed 133 tests with no compiler warnings. Each emitted 84 fault-occurrence assertions, all delta=0; the final observation count was 88916. All individual fault records are retained in gate-results.json and the raw logs.

Native ZeroMQ coverage was skipped nine times because loopback was unavailable. Its Rust wrappers return successfully after printing the skip; the printed "9 passed" total does not mean those native transport cases ran.

The final census passes 8 tests with 1 ignored scratch probe in each feature build. The FAILED child totals below are the existing deliberate frame/guard falsifiers. The no-insertion, wrong-binding and shadowed-witness mutations are also refused.

The runtime test build still reports the three pre-existing unused test helpers register_test_if_builtin, register_test_return_builtin and register_test_vm_template_builtins. The workspace also prints diagnostics from its deliberate option-read-proof compiler fixtures. No new retained-source compiler warning was identified. Both oracle builds, all four measurement candidate builds and both no-default harness builds are warning-free. Exact warning lines and their gates are retained in correctness-summary.json and gate-results.json.

The earlier shadowing falsifier exposed a real checker defect (review-shadow-before: unwrap_err on Ok). A conservative rebinding guard repaired it. The two changed census integration files then expired the global retention fingerprints, causing 17 stale-pin failures in final-workspace-reviewed. Replaying 68 fingerprint fields and four C01/C02 cells cleared both acceptance builds, the rejection census and the complete workspace. No rejection rule or contract column was weakened. The complete earlier totals remain in correctness-all-attempts.md; the 17 identities are in review/expired-pin-failures.json.

The commands below are the final passing selection. Every recorded command has its exact command JSON, bounded exit record and raw log under r3/. Commands used the offline local toolchain with one Cargo build job and one test thread. Workspace was bounded at 3600 seconds, each oracle at 1500 seconds, and the other gate commands at 900 seconds; the process-session lifetime did not truncate those bounds.

```text
final-fmt-reviewed
$ cargo fmt --check
exit=0

final-typed-api
$ cargo test -p rum-runtime --lib suspension::typed::tests
exit=0
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.46s

final-frame-roots
$ cargo test -p rum-runtime --lib frame_root_trace
exit=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1158 filtered out; finished in 1.25s

final-native48
$ cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.01s

final-controls-default
$ cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 39.29s

final-controls-no-default
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.03s

final-census-default-reviewed
$ cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.60s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.55s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 231.95s

final-census-no-default-reviewed
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.38s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.40s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 230.81s

final-ownership
$ cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
exit=0
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1130 filtered out; finished in 69.42s

final-certificate-default-reviewed
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 18.12s

final-certificate-no-default-reviewed
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1154 filtered out; finished in 18.22s

final-production-rejections-reviewed
$ cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 25.35s

final-ordinary-default
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/r3/ordinary-controls/Cargo.toml --test ordinary -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

final-ordinary-no-default
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/r3/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

final-oracle-default
$ cargo test -p rum-stdlib --test suspension_oracle -- --nocapture
exit=0
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.75s

final-oracle-no-default
$ cargo test -p rum-stdlib --no-default-features --test suspension_oracle -- --nocapture
exit=0
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.82s

final-workspace-replayed
$ cargo test --workspace --no-fail-fast -- --nocapture
exit=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.00s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.10s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 1170 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.02s
test result: ok. 1108 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1200.24s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.21s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.96s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.88s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.49s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.57s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 232.08s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.94s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.61s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.65s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

final-native-zeromq-replayed
$ cargo test -p rum-jupyter --lib native_zeromq -- --nocapture
exit=0
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.01s
```

Final oracle fault records:

```text
final-oracle-default
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88916
final-oracle-no-default
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88916
```

Native ZeroMQ skip output:

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

Every recorded correctness attempt and test-result line is reproduced in the final ledger below, including expected refusal children and superseded failures. Machine records with all warnings and fault assertions are in [gate-results.json](../../../.temp/brief-611/r3/gate-results.json).

### Measurement identity and method

Performance compares the frozen S4 binaries with freshly built r3 CGU1 binaries. Native stack compares the frozen r2-final binaries (including D) with r3. These are different baselines by instruction. The final binary audit verifies all fourteen binaries and all four measurement build-source manifests. The native scratch source has only the recorded 3,989-byte probe extension in its test file. All 529 runtime source hashes are unchanged since native measurement; only two census integration files changed afterward. The audit verifies that distinction explicitly.

| Arm / purpose | SHA-256 |
|---|---|
| S4 base rum-cli | `7881a7149a3c5a5d8e6d792f39d46338b365987d5c02908a65b6f4648d950e80` |
| S4 base rum-cli-alloc | `adf80f3eeebbbe39dcba9b6295191c9462893540a0fdce48215fba3499284b24` |
| S4 base resume | `40f5c75c6b42d698523822959b9f0a39372c5d4700f0bad55e21b8c2f2aade3d` |
| S4 base resume-alloc | `0414b33d5cee14192e786f2ca705ab250c35ccf76b257f5ea69da0dfaece04c5` |
| r3 candidate rum-cli | `0a63eab606e439b6f1d3c7f42fd1c66899114332456542b7760ea071669b406b` |
| r3 candidate rum-cli-alloc | `87df7df340a4d47805c8f3dbd68d73477b8ed8fbefa8a7b242419453333a2635` |
| r3 candidate resume | `92d9455fd3c71622cbbd267bf261d771b51c4ecd5a3a15b6e716c767aeb0a37b` |
| r3 candidate resume-alloc | `bc61d1222cce52c08492ca5baa4476cb27557af2b40f1c6140c603efc0c30403` |
| native base debug | `8465b366463dcb87056858de1a90c6558ac8e86c0188b34b68a0cce5bcf29c94` |
| native base release | `013e44fc1bfe20058a3c250dc82fa6dad22e05d6bddcd9f03531f296be7a3121` |
| native candidate debug | `e14d5c16e506c25c3027d591411cfa08b8b9af7b7c97a6866e4c0bd344d14d8a` |
| native candidate release | `f1900a19c94c0beba8e927400d1b1ca6fc9d18094091ddb0d70166793c1ac92f` |
| no-default harness base | `cd544c5aa0106a16f2f7e67976064b43a062eb63e4a56db12177880735b34d5a` |
| no-default harness candidate | `1389ef2972f60045919228c7bdcc6e855d6c34e401db9d29ff05f0b9d347ed59` |

Hash/source evidence: [base-verification.json](../../../.temp/brief-611/r3/base-verification.json), [measurement-build-summary.json](../../../.temp/brief-611/r3/measurement-build-summary.json), [final-binary-audit.json](../../../.temp/brief-611/r3/final-binary-audit.json), [review/source-hash-delta.json](../../../.temp/brief-611/r3/review/source-hash-delta.json).

The 139-label instruction panel ran base/candidate/base/candidate on CPU 2 with cpu_core/instructions/u, stdin /dev/null and profiling off. All raw perf output was saved before parsing. Each slope is (count(2N)-count(N))/N. Initial flag selection uses the maximum paired delta against max(0.05% of base, base range, candidate range), which can select more labels than the mean-delta rule. All 32 selected labels received five additional interleaved runs per arm; those comparisons use the mean delta and the stated threshold. All checked outputs matched.

Callgrind used separate N and 2N processes. Ordinary copy coverage is 107 labels; the three leaves and all 32 resume workloads each have six N/2N pairs per arm. Resume callgrind uses N=100/200 throughout; ordinary callgrind sizes are recorded per label and bounded by the existing instruction-budget rule. The tools ran serially. All final measurement and attribution gates exited zero.

### Redispatch 3 ordinary evidence

Ordinary allocations: 107/107 labels exactly equal in whole counts and bytes at N and 2N. Slopes and intercepts are retained in perf/allocation-comparison.json.

Resolved-libc memcpy gate: 107/107 labels have no increase at N, 2N or in slope. Every caller is listed in ordinary-callgrind/copy-callers.tsv; the actual GOT bindings and symbol resolution are in attribution/libc-binding.json and ordinary-callgrind/copy-symbol-resolution.json.

The 139-label instruction panel used two interleaved pairs per arm. Every one of the 32 initially flagged labels received five additional runs per arm, including resume labels. This table lists the ordinary subset; complete results are in perf/all-flagged-five-run-comparison.json.

| Initially flagged ordinary label | Base mean | Candidate mean | Delta | Threshold | Below flag threshold after five extra runs |
|---|---:|---:|---:|---:|---|
| lazy-foldlist-realize-small | 110766.087700 | 110803.678660 | 37.590960 | 233.596450 | True |
| lazy-depth-map-first-8 | 128299.904600 | 128297.337000 | -2.567600 | 64.149952 | True |
| lazy-depth-map-build-64 | 29818.346400 | 29809.150000 | -9.196400 | 22.998000 | True |
| lazy-depth-map-first-256 | 3664110.871200 | 3664218.028400 | 107.157200 | 20435.363000 | True |
| deletecases-leaf-delete-small | 63116.456800 | 63197.712800 | 81.256000 | 498.160000 | True |
| replace-leaf-rebuild-small | 73673.194000 | 74313.496800 | 640.302800 | 1876.960000 | True |
| rewrite-guarded-decline-fallback | 77027.328000 | 76955.252400 | -72.075600 | 218.910000 | True |
| part-list | 8302.904500 | 8298.660500 | -4.244000 | 38.960000 | True |
| set-part-fill-large | 5587482.056000 | 5584844.350000 | -2637.706000 | 16150.830000 | True |
| table-iterator-symbol | 333.234400 | 332.469500 | -0.764900 | 3.636500 | True |
| downvalue-exact-miss | 997.855600 | 997.434900 | -0.420700 | 2.846500 | True |
| downvalue-pattern-miss-repeat | 776.145000 | 776.005100 | -0.139900 | 3.692000 | True |
| reversesort-default-wide | 340860.070000 | 340933.160000 | 73.090000 | 447.100000 | True |

#### Leaves

The full address alignment and function deltas for all three leaves are in attribution/cli/*-leaf-proof.json. Equal evaluate bodies alone do not establish the whole-label no-growth bar. Variable residues outside one standard error remain explicitly reported.

| Leaf | Base mean | Candidate mean | Delta | Whole-label no-growth | Evaluate-body no-growth | Evaluate address changes | Positive function residues outside SE or deterministic growth |
|---|---:|---:|---:|---|---|---:|---:|
| literal-integer | 291.863583333 | 291.464916667 | -0.398666667 | False | True | 0 | 0 |
| do-count-null | 534.771500000 | 535.647250000 | 0.875750000 | False | True | 0 | 4 |
| do-iterator-null | 1152.196250000 | 1152.522416667 | 0.326166667 | False | True | 0 | 2 |

Every evaluate-body sample is 110 instructions per iteration in both arms, with zero changed executed instructions after address alignment. All leaf function deltas are process-variable; there is no deterministic function growth. The whole-label samples overlap, so the strict whole-label no-growth proof is not established. The positive variable residues outside one standard error are:

| Leaf | Function / resolved body | Delta | Standard error |
|---|---|---:|---:|
| do-count-null | `hashbrown::map::HashMap<K,V,S,A>::insert` | 0.341000000 | 0.217975527 |
| do-count-null | `rum_core::expr::intern_symbol_text` | 0.256166667 | 0.249660942 |
| do-count-null | `rum_core::expr::intern_text` | 0.243833333 | 0.149049554 |
| do-count-null | `rum_runtime::runtime::builtin_catalog::default_builtin_dispatch_spec_for_symbol` | 0.008416667 | 0.005385294 |
| do-iterator-null | `/usr/lib/libc.so.6 0x00000000001b1220` | 0.054083333 | 0.038153437 |
| do-iterator-null | `rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom` | 0.003250000 | 0.002422407 |

The libc body at 0x1b1220 is the resolved bcmp/memcmp alias in libc-binding.json. The full function and address ledgers retain all six samples, including negative residues; no variable remainder is rounded to zero. The copy alias audit separately proves that memcpy/memmove share body 0x1b1980 and that every caller of that proven memcpy target was counted (copy-target-alias-audit.json).

#### Default miss

C remains closed by the director amendment. default-unguarded-miss did not flag in this panel; default-literal-or.json records that result. No new literal-OR analysis, C source repair, additional C callgrind experiment, or timing experiment was performed.

Ordinary measurement evidence: [perf/allocation-comparison.json](../../../.temp/brief-611/r3/perf/allocation-comparison.json), [perf/instruction-comparison.json](../../../.temp/brief-611/r3/perf/instruction-comparison.json), [perf/all-flagged-five-run-comparison.json](../../../.temp/brief-611/r3/perf/all-flagged-five-run-comparison.json), [ordinary-callgrind/copy-gate.json](../../../.temp/brief-611/r3/ordinary-callgrind/copy-gate.json), [ordinary-callgrind/copy-callers.tsv](../../../.temp/brief-611/r3/ordinary-callgrind/copy-callers.tsv), [ordinary-callgrind/copy-target-alias-audit.json](../../../.temp/brief-611/r3/ordinary-callgrind/copy-target-alias-audit.json), [ordinary-callgrind/leaf-proof.json](../../../.temp/brief-611/r3/ordinary-callgrind/leaf-proof.json), [default-literal-or.json](../../../.temp/brief-611/r3/default-literal-or.json).

### Native stack: redispatch 3 versus redispatch 2 final

The base is the r2 final tree, including D. The candidate is the retained witness/request tree. Each pair uses four heights and three samples per height. Every linear fit has zero residual. Native binaries, source hashes, raw samples, fits, complete changed-function prologues and disassembly excerpts are preserved under stack/.

| Shape | Debug base | Debug candidate | Delta | Release base | Release candidate | Delta |
|---|---:|---:|---:|---:|---:|---:|
| peel | 19360 | 19024 | -336 | 7440 | 7344 | -96 |
| plus | 35376 | 34912 | -464 | 13920 | 13792 | -128 |
| SubValue | 29600 | 29376 | -224 | 13152 | 13168 | +16 |
| UpValue | 28288 | 28064 | -224 | 11584 | 11456 | -128 |
| guarded DownValue | 36304 | 35840 | -464 | 14368 | 14240 | -128 |

The +16 release SubValue pair requires the director's ruling. The requested shape is retained; no further source experiment was made. All other changed resident frames below shrink. frame-deltas.json/tsv records every resident function and its multiplicity, including unchanged functions. The change sums exactly to every measured pair: debug peel = -96-128-112; plus/guard = -96-2*128-112; sub = -112-112; up = -112-112. Release peel = -32-32-32; plus/guard = -32-2*32-32; sub = +48-32; up = -96-32.

Offsets below are from rsp after the prologue. Payload sizes are 136 B for the producer Result, 32 B for RuntimeInputRequest, and 168 B for SuspendedDownValueApplication / the analogous application state. SlotWitness is zero-sized. A reduced payload does not imply that the entire frame shrinks by that amount: release code reuses low slots and rearranges live return areas. Saved-register counts and alignment are included in the measured prologues.

| Profile / resident function | Entry base -> candidate | Frame bytes | Producer / cold-input slots and evidence |
|---|---|---|---|
| debug outer DownValue | 19df00 -> 1a3c10 | 2104 -> 2008 (-96) | Result 2a0 -> 298; cold Result 368..3ef becomes request 360..37f; cold calls 19f018 -> 1a4d26. The cold payload saves 104 B; remaining frame layout differs by +8 B. |
| debug DownValue scan | a5e6e0 -> a57080 | 4072 -> 3944 (-128) | Actual Result a48 -> a28; cold application c10..cb7 -> c18..cbf; cold Result cb8..d3f becomes request bf0..c0f. Cold calls a5fa81 -> a583ed. The payload saves 104 B; other slot placement/alignment saves 24 B. |
| debug application RHS | 1a4e30 -> 1aac10 | 1256 -> 1144 (-112) | Actual Result 1a8 -> 198; cold Result 310..397 becomes request 270..28f. Cold output 288 -> 298; cold calls 1a51c6 -> 1aafd9. The payload saves 104 B; other slot layout saves 8 B. |
| debug SubValue scan | a626b0 -> a5aea0 | 3048 -> 2936 (-112) | Actual Result 708 -> 6f8; application 8d0..977 -> 8e8..98f; cold Result 978..9ff becomes request 8c0..8df; calls a638ca -> a5c0a2. The payload saves 104 B; other slot layout saves 8 B. |
| debug UpValue list scan | 1a67d0 -> 1ac1d0 | 2472 -> 2360 (-112) | Actual Result 638 -> 628; cold Result 7e8..86f becomes request 780..79f; candidate Expr arguments 7a0/7c0 and vector 7e0; cold calls 1a7ade -> 1ad4c9. The payload saves 104 B; other slot layout saves 8 B. |
| release outer DownValue | 51ada0 -> 519820 | 776 -> 744 (-32) | Actual Result 1d0 -> 120; cold Result copied to reused a0 at 51c63f becomes request 290..2af at 51b086..51b0a1. Timing return area 2a0..2d7 -> 258..28f, followed by the new request. Cold calls 51c65f -> 51b0cc. The timing area moves down 72 B; a new 32-B request plus an 8-B frame-tail gap gives the net -32 B. |
| release DownValue scan | 51f000 -> 51dba0 | 1320 -> 1288 (-32) | Actual Result 320 -> 180; cold application a0 -> 60; cold Result at reused 260 becomes request 450..46f (extract 51fc22..51fc3d). OptionValueScope return slots 498/4c8 -> 478/4a8, each 48 B; highest end 4f8 -> 4d8. Cold calls 5210c7 -> 51fc7f. |
| release application RHS | 52e340 -> 52d160 | 920 -> 888 (-32) | Actual Result 258 -> c0; cold Result at reused 80 becomes request 2a0..2bf; cold output 110 -> 150; cold calls 52e88e -> 52d6db. The highest 136-B finish-helper argument moves 2e0..367 -> 2c0..347, lowering the frame end by 32 B. |
| release SubValue scan | 52f6e0 -> 52e5a0 | 1528 -> 1576 (+48) | Detailed high-slot map below; actual Result 4a0 -> 420. Cold application remains 70. The old cold Result reuses 150; the new request has a dedicated 5a0..5bf slot. Cold calls 5319d7 -> 530900. |
| release UpValue (list scan inlined) | 534620 -> 533690 | 1096 -> 1000 (-96) | Cold Result at 280, copied from actual 330 at 535867, becomes request 310..32f (534889..534898). OptionValueScope slots 3b8/3e8 -> 358/388, each 48 B; highest end 418 -> 3b8. Cold calls 535898 -> 534911. |

The debug SubValue outer (744 B), debug UpValue outer (696 B), release SubValue outer (504 B), and finish_suspended_down_value_application (debug 920 B, release 440 B) are unchanged. Other resident functions cancel exactly; see frame-deltas.tsv. native-changed-prologues.tsv includes cold/nonresident functions as well and does not substitute their frame sizes for resident-cycle measurements.

#### Release SubValue high-slot layout (+48 B)

Both prologues push six registers (48 B); the rsp subtraction changes 0x5c8 -> 0x5f8. Both layouts reserve a lower region ending at 0x420; the high-area extent differs as follows. The following high-area slots explain the entire difference:

| Slot | Base range | Candidate range | Evidence |
|---|---|---|---|
| Producer Result (136 B) | 4a0..527 | 420..4a7 | matcher calls 5306c1 -> 52f581 |
| Alignment gap before SIMD mask (8 B) | absent | 4a8..4af | mask needs 16-byte alignment after relocated 136-B Result |
| Hash SIMD mask (16 B) | 420..42f | 4b0..4bf | stores 52f751 -> 52e611 |
| Existing 8-B gap | 430..437 | 4c0..4c7 | no direct accesses in either function |
| Matcher return area (48 B) | 438..467 | 4c8..4f7 | compile_down_value_matcher calls 530525 -> 52f3e5 |
| First timing return area (56 B) | 468..49f | 4f8..52f | runtime_path_timing_start_slow calls 52f9d4 -> 52e894 |
| Second timing return area (56 B) | 528..55f | 530..567 | calls 52fb78 -> 52ea38 |
| Third timing return area (56 B) | 560..597 | 568..59f | calls 52fdcf -> 52ec8f |
| Dedicated cold request (32 B) | absent: old Result reused low 150 | 5a0..5bf | extract 5308a4..5308bf, cold call 530900 |
| New 8-B gap | absent | 5c0..5c7 | no direct access; precedes highest OptionValueScope slot |
| Option<OptionValueScope> (48 B) | 598..5c7 | 5c8..5f7 | construct 53073a -> 52f606; RHS argument 530ff6 -> 52fee1; drop 531e06 -> 530d65 |

The new high region is larger by 8 + 32 + 8 = 48 B. The RHS frame shrinks by 32 B, giving the measured +16 B/level. This is compiler slot layout, not a new frame or allocation on the ordinary path.

#### Debug error forwarding

The debug-only second 136-B error temporary persists. DownValue scan copies actual a28 -> cc0 at a58419, cc0 -> d48 at a58433, then forwards out. Base temporaries were d40/dc8. SubValue scan copies actual 6f8 -> 990 at a5c0ce, 990 -> a18 at a5c0e8, and out at a5c109; base temporaries were a00/a88. Each error is forwarded once semantically; the extra compiler temporary has not been chased. Full instruction streams are in final-slot-excerpts/.

The full 584-row owned-function prologue table includes cold and nonresident functions as well as the resident cycle; it is not restricted to the ten changing resident rows shown above. Evidence: [stack/native-comparison.json](../../../.temp/brief-611/r3/stack/native-comparison.json), [stack/frame-deltas.tsv](../../../.temp/brief-611/r3/stack/frame-deltas.tsv), [stack/native-changed-prologues.tsv](../../../.temp/brief-611/r3/stack/native-changed-prologues.tsv), [stack/final-slot-excerpts/index.json](../../../.temp/brief-611/r3/stack/final-slot-excerpts/index.json).

### Redispatch 3 resume attribution

All 32 workloads ran at N=100/200 in six independent callgrind N/2N pairs per arm (twelve processes per arm per workload). The base is the frozen S4 binary and the candidate is the retained r3 source. Raw stdout preserves the checked final values and actual resume counts. No timing panel was run.

#### Allocations and frames

resume-callgrind/frame-calls.json records each measured frame before/after, its whole N/2N counts, calls per iteration, and calls per actual resume. All 32 allocation deltas reconcile to delta(typed entries) + 3*new application Some wires; byte deltas are 24*delta(typed entries) + 120*new wires. Every count and byte intercept is zero. Allocation instruction sites are disclosed separately in allocation-site-evidence.json.

#### Driver and payload copies

driver-copy-proof.json proves the unique 136-B typed-outcome transfer site set in each binary: its execution count equals the typed-adapter resume count for every workload in all six N/2N samples. Exact source/destination instruction windows are retained there. The extra copies equal the new typed entries, 136 B each, and remain owed to R. No driver change was made.

Every resolved memcpy call is listed in resume-copy-sites.tsv with the binary arm, exact body entry/name, category, address, nearby-immediate size where proved, and all six execution slopes. Per-label *-copies.json preserves the full instruction windows; unknown sizes are not guessed. resume-simd-moves.tsv records executed vector/rep moves in the transport, frame, helper and allocation categories. All other instructions are retained in the full address tables. Corresponding legacy and typed bodies must be paired before calling a movement additional.

#### Complete instruction attribution

Each workload .json contains all per-function samples and standard errors, including unchanged functions in all_functions. Each *-addresses.tsv contains every executed address, disassembly and six slopes. Each *-calls.tsv contains callers, callees, call addresses and six counts. *-bodies.json resolves aliased names to actual ELF entries. Category totals in semantic-summary.json sum exactly to the whole-label delta. Categories organize the evidence; the named functions and addresses remain the attribution.

Variable residues are listed with their standard errors and are not silently accepted as zero. Missing-disassembly rows, if any, are disclosed in the workload JSON and summarized below.

| Workload | New typed entries | New Some wires | Extra allocs / bytes | Added 136-B transfers | Instruction delta | Deterministic part | Variable functions outside 1 SE |
|---|---:|---:|---:|---:|---:|---:|---:|
| resume-table | 0 | 0 | 0 / 0 | 0 | 8.188333 | 0.000000 | 3 |
| resume-do-chain | 0 | 0 | 0 / 0 | 0 | -43.868333 | 0.000000 | 3 |
| resume-call-repeat | 0 | 0 | 0 / 0 | 0 | -4.260000 | 0.000000 | 5 |
| resume-cleanup | 0 | 0 | 0 / 0 | 0 | 2.271667 | 0.000000 | 3 |
| resume-fold | 0 | 0 | 0 / 0 | 0 | -36.078333 | 0.000000 | 4 |
| resume-argument-twice | 0 | 0 | 0 / 0 | 0 | -10.905000 | 0.000000 | 1 |
| resume-rewrite-exprs | 0 | 0 | 0 / 0 | 0 | 41.166667 | 0.000000 | 1 |
| resume-rewrite-compiled | 0 | 0 | 0 / 0 | 0 | 17.636667 | 0.000000 | 3 |
| resume-cases-spec | 1 | 0 | 1 / 24 | 1 | 282.920000 | 167.960000 | 6 |
| resume-cases-traversal | 0 | 0 | 0 / 0 | 0 | -19.081667 | 0.000000 | 6 |
| resume-position-traversal | 0 | 0 | 0 / 0 | 0 | -155.106667 | 0.000000 | 5 |
| resume-count-traversal | 0 | 0 | 0 / 0 | 0 | -46.811667 | 0.000000 | 2 |
| resume-cases-shared | 1 | 0 | 1 / 24 | 1 | 157.420000 | 170.010000 | 3 |
| resume-s5-down-rhs | 1 | 1 | 4 / 144 | 1 | 1947.955000 | 1407.840000 | 6 |
| resume-s5-down-guard | 2 | 1 | 5 / 168 | 2 | 1999.863333 | 1435.350000 | 8 |
| resume-s5-sub-rhs | 1 | 1 | 4 / 144 | 1 | 1881.460000 | 1360.780000 | 5 |
| resume-s5-sub-guard | 2 | 1 | 5 / 168 | 2 | 2114.156667 | 1567.570000 | 6 |
| resume-s5-up-rhs | 1 | 1 | 4 / 144 | 1 | 2027.991667 | 1428.340000 | 6 |
| resume-s5-up-guard | 2 | 1 | 5 / 168 | 2 | 2161.160000 | 1671.000000 | 5 |
| resume-s5-default-rhs | 2 | 1 | 5 / 168 | 2 | 1999.631667 | 1455.860000 | 9 |
| resume-s5-default-guard | 3 | 1 | 6 / 192 | 3 | 2015.020000 | 1655.540000 | 8 |
| resume-s5-default-fallback-rhs | 2 | 1 | 5 / 168 | 2 | 1934.955000 | 1506.960000 | 6 |
| resume-s5-set | 1 | 0 | 1 / 24 | 1 | 170.095000 | 108.000000 | 3 |
| resume-s5-upset | 1 | 0 | 1 / 24 | 1 | 173.181667 | 232.100000 | 0 |
| resume-s5-tagset | 1 | 0 | 1 / 24 | 1 | 264.998333 | 249.100000 | 4 |
| resume-s5-rule-rhs | 1 | 0 | 1 / 24 | 1 | 153.880000 | 117.560000 | 4 |
| resume-s5-rule-lhs | 2 | 0 | 2 / 48 | 2 | 83.368333 | 127.480000 | 4 |
| resume-s5-destructure | 1 | 0 | 1 / 24 | 1 | 0.295000 | 100.040000 | 4 |
| resume-s5-lhs-twice | 2 | 0 | 2 / 48 | 2 | 347.965000 | 385.560000 | 4 |
| resume-s5-own-rhs | 1 | 0 | 1 / 24 | 1 | 209.231667 | 117.000000 | 4 |
| resume-s5-destructure-twice | 2 | 0 | 2 / 48 | 2 | 326.978333 | 228.460000 | 6 |
| resume-s5-rule-lhs-twice | 3 | 0 | 3 / 72 | 3 | 107.960000 | 210.280000 | 5 |

The complete ledgers contain 1286 function-delta rows, 2453 resolved copy sites, 44690 vector/rep movement rows and 19808 body rows. Of 463 variable function records, 142 lie outside one standard error; each measured value and standard error is retained in resume-function-deltas.tsv. These residues are reported, not accepted as zero.

There are 120 missing-disassembly rows, at 0x406e128, 0x406e160. They are Callgrind object ??? entries. All 60 corresponding workload/address pairs have identical base and candidate samples, so their delta is exactly zero (missing-assembly-comparison.json). No source name or disassembly is invented for them.

The unique driver transfer sites are:

| Arm | memcpy call address | Bytes | Stack source -> destination |
|---|---|---:|---|
| base | 0x72d414 | 136 | rsp+0x310 -> rsp+0x228 |
| candidate | 0x73b484 | 136 | rsp+0x310 -> rsp+0x228 |

Representative allocation sites for resume-s5-down-rhs follow. Every listed call count is identical across all six samples. The three Some-wire sites each increase by one call per iteration; the typed adapter adds one new 24-byte allocation. The full windows and source locations are in allocation-site-evidence.json.

| Arm | Allocation | Bytes | Size instruction | Allocator call | Calls / iteration |
|---|---|---:|---|---|---:|
| base | Some wire one-element Vec<Expr> | 32 | 0x754759 | 0x754763 | 3 |
| base | Some wire SharedExpr Arc | 48 | 0x79d904 | 0x79d90e | 4 |
| base | Some wire SharedArgs Arc | 40 | 0x79db63 | 0x79db6d | 7 |
| candidate | outer typed adapter Arc | 24 | 0xb9bd9 | 0xb9be3 | 1 |
| candidate | Some wire one-element Vec<Expr> | 32 | 0x7627c9 | 0x7627d3 | 4 |
| candidate | Some wire SharedExpr Arc | 48 | 0x7af674 | 0x7af67e | 5 |
| candidate | Some wire SharedArgs Arc | 40 | 0x7af8d3 | 0x7af8dd | 8 |

Resume measurement evidence: [resume-callgrind/frame-calls.json](../../../.temp/brief-611/r3/resume-callgrind/frame-calls.json), [perf/allocation-frame-proof.json](../../../.temp/brief-611/r3/perf/allocation-frame-proof.json), [allocation-site-evidence.json](../../../.temp/brief-611/r3/allocation-site-evidence.json), [attribution/resume/semantic-summary.json](../../../.temp/brief-611/r3/attribution/resume/semantic-summary.json), [attribution/resume/resume-function-deltas.tsv](../../../.temp/brief-611/r3/attribution/resume/resume-function-deltas.tsv), [attribution/resume/resume-copy-sites.tsv](../../../.temp/brief-611/r3/attribution/resume/resume-copy-sites.tsv), [attribution/resume/resume-simd-moves.tsv](../../../.temp/brief-611/r3/attribution/resume/resume-simd-moves.tsv), [attribution/resume/resume-body-costs.tsv](../../../.temp/brief-611/r3/attribution/resume/resume-body-costs.tsv), [attribution/resume/driver-copy-proof.json](../../../.temp/brief-611/r3/attribution/resume/driver-copy-proof.json), [attribution/resume/missing-assembly-comparison.json](../../../.temp/brief-611/r3/attribution/resume/missing-assembly-comparison.json).

### Stop-loss, repairs and remaining limits

The final consecutive no-progress repair count is zero. Initial compilation reduced one doc-comment-placement error to zero; the duplicate cold attribute warning was removed in the same repair. Two receipt-discovery invocations still used the old M0237 anchor and refused it; the twelve witnessed insertion identities were proved before replaying the receipts. G4/G7 and ownership replay then passed. An initial ownership classification assertion stopped before any inventory write and was corrected by restricting the removal classification to the owned S5 constructs.

The route-name collision was repaired through the authorized module-root re-export, with no retention-checker edit. The shadowed-witness falsifier first demonstrated false acceptance, then passed with the rebinding guard. The resulting integration source changes expired global fingerprints: one workspace run had 17 stale-pin failures. A single fingerprint/certificate replay cycle cleared both acceptances, the production rejection census and the final full workspace. The failed identities and exact cause are in [review/expired-pin-failures.json](../../../.temp/brief-611/r3/review/expired-pin-failures.json).

The final binary-audit script initially compared the probe-augmented native manifest directly with the canonical test source. Its corrected check proves the exact canonical prefix plus the saved probe and separately compares every canonical runtime hash. This was an audit-script correction, not a source repair. The memcpy alias audit confirms that the saved resolution selected the proven memcpy target even though memmove shares it; no copy caller was omitted.

Recorded bounded gate counters: 139 completed parent records, including 48 direct Cargo test invocations and 8 formatting invocations. The complete correctness ledger contains 219 test-result lines. These include expected refusal controls, discovery emissions and superseded attempts; they are not counts of unsuccessful source repairs. Final selected correctness exits are all zero. The historical final-workspace-reviewed failure remains in the ledger and is superseded by final-workspace-replayed.

Scratch remains under r3 except the pre-authorized step-independent compile-control scratch and certificate replay output. The oracle’s exact two Cargo invocations were used with the r3 scratch/log paths instead of its wrapper’s fixed scratch path. No unsafe probe or abort was used. No second stack shape, debug error-temporary chase, new C experiment or timing panel was attempted.

Remaining limits are explicit: release SubValue +16 B/level needs the director’s choice between this tree and entry/; the strict leaf whole-label proof remains uncleared; 142 of 463 variable resume function records lie outside one standard error and remain reported; native ZeroMQ coverage has nine skips. The 120 unmapped Callgrind address rows have exactly zero base/candidate delta. All required commands have finished; no gate is left running.

### Redispatch 3 complete recorded correctness ledger

These are all recorded parent gates with test-result output, plus formatting commands. Expected forced-red child totals and superseded failures are preserved verbatim. The final passing selection is in correctness-final.md.

```text
adapter-witnesses
$ cargo test -p rum-stdlib --lib adapter_production_witnesses_are_computed -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 5.99s

census-discovery1
$ cargo test -p rum-stdlib --test typed_slot_census source_baseline_matches_earned_receipt -- --exact --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 22.51s

census-discovery2
$ cargo test -p rum-stdlib --test typed_slot_census source_baseline_matches_earned_receipt -- --exact --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 22.63s

census-final-format
$ cargo fmt
exit=0

census-format
$ cargo fmt
exit=0

census-witnessed
$ cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 19.27s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 19.58s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 202.81s

certificate-emission-default-census-hardening
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 18.08s

certificate-emission-default-drop-order
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 17.85s

certificate-emission-default-route-scope
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 18.05s

certificate-emission-no-default-census-hardening
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1154 filtered out; finished in 18.28s

certificate-emission-no-default-route-scope
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1154 filtered out; finished in 18.47s

discovery
$ cargo test -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 5.70s

final-census-default-reviewed
$ cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.60s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.55s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 231.95s

final-census-default
$ cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 19.41s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 19.34s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 202.15s

final-census-no-default-reviewed
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.38s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.40s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 230.81s

final-census-no-default
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
exit=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 19.69s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 19.41s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 202.23s

final-certificate-default-reviewed
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 18.12s

final-certificate-default
$ cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 18.28s

final-certificate-no-default-reviewed
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1154 filtered out; finished in 18.22s

final-certificate-no-default
$ cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1154 filtered out; finished in 18.30s

final-controls-default
$ cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 39.29s

final-controls-no-default
$ cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.03s

final-fmt-reviewed
$ cargo fmt --check
exit=0

final-fmt
$ cargo fmt --check
exit=0

final-frame-roots
$ cargo test -p rum-runtime --lib frame_root_trace
exit=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1158 filtered out; finished in 1.25s

final-native-zeromq-replayed
$ cargo test -p rum-jupyter --lib native_zeromq -- --nocapture
exit=0
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.01s

final-native-zeromq
$ cargo test -p rum-jupyter --lib native_zeromq -- --nocapture
exit=0
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s

final-native48
$ cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --exact
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.01s

final-oracle-default
$ cargo test -p rum-stdlib --test suspension_oracle -- --nocapture
exit=0
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.75s

final-oracle-no-default
$ cargo test -p rum-stdlib --no-default-features --test suspension_oracle -- --nocapture
exit=0
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.82s

final-ordinary-default
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/r3/ordinary-controls/Cargo.toml --test ordinary -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

final-ordinary-no-default
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/r3/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

final-ownership
$ cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
exit=0
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1130 filtered out; finished in 69.42s

final-production-rejections-reviewed
$ cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 25.35s

final-production-rejections
$ cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 25.22s

final-shadow-refusal
$ cargo test -p rum-stdlib --test typed_slot_census shadowing_a_witness_in_a_cold_helper_is_red -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 28.96s

final-typed-api
$ cargo test -p rum-runtime --lib suspension::typed::tests
exit=0
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.46s

final-workspace-replayed
$ cargo test --workspace --no-fail-fast -- --nocapture
exit=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.00s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.10s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 1170 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.02s
test result: ok. 1108 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1200.24s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.21s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.96s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.88s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.49s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.57s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 232.08s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.94s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.61s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.65s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

final-workspace-reviewed
$ cargo test --workspace --no-fail-fast -- --nocapture
exit=101
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.45s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.10s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.81s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 1170 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.52s
test result: FAILED. 1091 passed; 17 failed; 53 ignored; 0 measured; 0 filtered out; finished in 949.39s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.13s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.68s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.30s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.51s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 230.87s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.34s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.49s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.93s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.58s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

final-workspace
$ cargo test --workspace --no-fail-fast -- --nocapture
exit=0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.50s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 1170 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 40.69s
test result: ok. 1108 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1301.36s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.31s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 133 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.69s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 19.54s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 19.35s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 201.79s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.49s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.55s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.63s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

format-repair1
$ cargo fmt
exit=0

frontier
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/r3/frontier-fixture/Cargo.toml --test frontier checker::r6_production_views_inventory -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 10.26s

g4-audit
$ cargo test -p rum-stdlib --lib structural_rule_pattern_depth_contract -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 0.13s

g7-audit
$ cargo test -p rum-stdlib --lib g7_continuation_contract -- --nocapture
exit=0
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1143 filtered out; finished in 0.35s

initial-format
$ cargo fmt
exit=1

initial-typed-api
$ cargo test -p rum-runtime --lib suspension::typed::tests
exit=101

review-shadow-before
$ cargo test -p rum-stdlib --test typed_slot_census shadowing_a_witness_in_a_cold_helper_is_red -- --exact --nocapture
exit=101
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 28.99s

review-shadow-format
$ cargo fmt --all
exit=0

route-scope-adapter-witnesses
$ cargo test -p rum-stdlib --lib adapter_production_witnesses_are_computed -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 5.91s

route-scope-candidates
$ cargo test --manifest-path .temp/brief-611/r3/frontier-fixture/Cargo.toml --test frontier checker::s5_probe_repeat_candidates -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 14.56s

route-scope-discovery
$ cargo test -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 5.70s

route-scope-format
$ cargo fmt
exit=0

route-scope-frontier
$ cargo test --manifest-path /home/remi/dev/rum/.temp/brief-611/r3/frontier-fixture/Cargo.toml --test frontier checker::r6_production_views_inventory -- --exact --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 10.20s

route-scope-g4-audit
$ cargo test -p rum-stdlib --lib structural_rule_pattern_depth_contract -- --nocapture
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 0.12s

route-scope-g7-audit
$ cargo test -p rum-stdlib --lib g7_continuation_contract -- --nocapture
exit=0
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1143 filtered out; finished in 0.30s

typed-api-repair1
$ cargo test -p rum-runtime --lib suspension::typed::tests
exit=0
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.49s
```
