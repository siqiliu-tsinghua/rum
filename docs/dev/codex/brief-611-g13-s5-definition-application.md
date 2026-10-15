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
