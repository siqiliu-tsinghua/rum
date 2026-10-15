# brief-610 — G13 step S4: type argument and traversal-spec resolution

## Goal

Type the second production family through the typed API. S4 owns the frames
that suspend while rum resolves the arguments and specifications of the
rule-driven builtins:

- argument lists, such as option values, held `Part` selectors and level
  specifications;
- `Cases` pattern specifications;
- rewrite-rule expressions;
- the `Cases`, `Position` and limit-free traversal configurations.

When you are done:

- the 11 owned types (10 frames and 1 finalizer) implement the typed API;
- the 12 owned macro sites use the witnessed typed macro;
- every other family still runs through the transport it uses today;
- the one deferred obligation (D0002) is applied as its row specifies;
- the ordinary path of every program executes no more work than at S3, and
  the cost of S4's resume path is measured and attributed.

This is step S4 of G13 (design round 5, brief-606, ratified). S1 (brief-607,
`777c61cc`) built the dormant typed API. S2 (brief-608, `cb09bfde`) made the
driver run typed and legacy entries side by side. S3 (brief-609, `1a228f4b`)
typed evaluation, argument preparation and dispatch. S5-S54 type the
remaining families, F1-F23 flip transport per type, and R removes the legacy
machinery.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- Authority, in order: this brief; brief-606 (its D1/D2 and report) with
  `.temp/brief-606/design.md` and `census-spec.md`; `.temp/brief-605/design.md`
  §§2-5 for the API, token, fault, unwind and accounting design, **except
  §5's transit record, which D2 rejected**.
- **S3 as built is your template.** Read brief-609 in full: its body, each
  director amendment and the final director gate record. S3 solved the same
  problems you will meet, and each solution is now the rule:
  - **Two arms on the typed macro.** `resume_typed_or_suspend!` is defined
    in `suspension/typed/witness.rs`.
    - The caller-place arm (`$result:ident`) matches a local in place. Use
      it wherever the producer's result is already bound to a local.
      Re-binding a large `Result` costs a `memcpy` on every ordinary pass;
      S3 measured this.
    - The expression arm is for a producer written inline.
  - **The witness and the token.** The witness comes from the actual
    producer `Result`. `ContinueToken::suspend(session, witness, request,
    fresh)` is the only insertion route, and it carries no `Arc`.
  - **D0001 is the pattern D0002 copies.** It is the prepared-argument
    accumulator in `eval_call_prepare.rs` and
    `eval_call_prepare/resume_frames.rs`:
    - a frozen prefix plus an `AccumulatorChain` tail;
    - fresh cursor state through the token;
    - `ResumeStop` and `TypedAction::finish_work` selecting the pure
      finalizer;
    - the finalizer running after both originating `Arc`s drop.
  - **The finalizer erasure.** A typed finalizer is erased as its own box,
    through the blanket `impl<F: SlotFinalizer> ErasedFinalizer for F`.
  - **The retention analyzer reads the typed macro** through one checked
    adapter, `retention_closure/typed_resume_macro.rs`. It holds the exact
    definitions of both arms, the expansion rejection list and the drift
    falsifier. All four production views and the frontier view read it.
- **Rows, as input; never edit them.** All of these are in `.temp/brief-606/`:
  - `all-frame-migrations.tsv`: F0247-F0256 and F0568;
  - `macro-dataflow.tsv` and `macro-frame-ownership.tsv`: M0248-M0259;
  - `deferred-obligations.tsv`: D0002;
  - `draft-S4.md`: Codex's plan for this step.

  S4 has no transformation rows, chained inserts, insertion routes, State
  births or historical divergences. The draft is raw material, and where it
  and this brief differ, this brief governs. In particular, its Scope, its
  new-file list (those files were built in S1-S3) and its measurement
  section (timing does not decide) are superseded.
- Current definitions, checked by the director at S3's commit. All paths are
  under `crates/rum-runtime/src/runtime/definitions/definition_catalog/`:

  | Row | Type | Defined at |
  | --- | --- | --- |
  | F0247 | `ResolveArgumentExprsFromCursorSuspensionFrame` | `arguments.rs:171` |
  | F0248 | `ResumeSuspendedArgumentResolutionSuspensionFrame` | `arguments.rs:198` |
  | F0568 | `FinishArgumentResolutionFinalizer` | `arguments.rs:226` |
  | F0249 | `ResolveCasesSpecExprForMatchingOwnedSuspensionFrame` | `rewrite_rules.rs:328` |
  | F0250 | `ResolveRewriteRuleExprsOwnedSuspensionFrame` | `rewrite_rules.rs:360` |
  | F0251 | `DriveRewriteRuleExprsOwnedSuspensionFrame` | `rewrite_rules.rs:383` |
  | F0252 | `DriveRewriteRuleInfosOwnedSuspensionFrame` | `rewrite_rules.rs:411` |
  | F0253 | `DriveRewriteRulesOwnedSuspensionFrame` | `rewrite_rules.rs:434` |
  | F0254 | `ResolveCasesTraversalOwnedSuspensionFrame` | `traversal.rs:399` |
  | F0255 | `ResolvePositionTraversalOwnedSuspensionFrame` | `traversal.rs:432` |
  | F0256 | `ResolveTraversalWithoutLimitOwnedSuspensionFrame` | `traversal.rs:466` |

  The macro sites fall into three shapes:

  | Shape | Sites | Conversion |
  | --- | --- | --- |
  | An ordinary producer matched by hand, then `resume_or_suspend!(insert …)` | M0248, M0249 (`arguments.rs:97`, `:127`); M0255 (`rewrite_rules.rs:237`) | The typed macro on the producer's `Result`. The state the frame needs moves into the cold `suspend =>` arm. |
  | An insert inside the resumed helper | M0250 (`arguments.rs:154`, `resume_suspended_argument_resolution`) | D0002: fresh state through the token, as M0280 did in S3. |
  | The result form, already bound to a local | M0251-M0254 (`rewrite_rules.rs:150`, `:185`, `:201`, `:217`); M0256-M0259 (`traversal.rs:312`, `:335`, `:357`, `:380`) | The caller-place arm of the typed macro. |

- **A pure transport map disappears.** The `ok` arms of M0251 and M0252 call
  `<Expr as ContinuationValue>::from_continuation_expr` on the producer's
  value. That method is the identity (`suspension/primitive.rs:3`), and
  `Expr` is the one `Value`-mode type, so the call goes with the conversion.
  This is the only transport map in S4's rows. Any other map you find is
  semantic: it runs identically on the ordinary and resumed paths.
- **Every slot type S4 exchanges is already registered.** That covers
  `Vec<Expr>`, `Option<Vec<Expr>>`, `Option<Vec<RewriteRuleInfo>>`,
  `CasesTraversalResolution`, `TraversalWithoutLimitResolution` and `Expr`
  (all registered in `suspension/typed/legacy.rs`, except `Expr`, which is
  `Value`). Each keeps its single global mode. A typed frame that exchanges
  a Legacy type, including a typed-to-typed exchange such as M0252 → M0253,
  goes through that type's wire. The cost of doing so is mode work and is
  attributed as such.

## Scope

A listed file or class is permission to do this step's owned work there. It
is not permission to convert another step's rows, change evaluation beyond
the owned rows, or tidy. Disclose every file you edit.

**Classes.**

- **The family's source files**, under
  `crates/rum-runtime/src/runtime/definitions/`:
  - `definition_catalog/arguments.rs`;
  - `definition_catalog/rewrite_rules.rs`;
  - `definition_catalog/traversal.rs`;
  - `definition_catalog/traversal_continuation.rs`;
  - `definition_catalog/entries.rs`.
- **Module roots on an owned type's path.** Make only the visibility or
  re-export edit that the typed API needs, and disclose it. The director's
  `module-path-visibility.py` found every owned type private, behind
  `definitions/mod.rs:3` (`mod definition_catalog`) and
  `definition_catalog.rs:2`, `:5` and `:6` (`mod arguments`,
  `mod rewrite_rules`, `mod traversal`). S3 needed no visibility change for
  the same situation.
- **Constructor ripple.** Edit only the construct that changed, in any file
  where an owned frame or helper is constructed or named. The callers of
  `resolve_argument_exprs_owned` (`kernel_data/evaluated_part_selectors.rs`
  and `kernel_data/rewrite_data/replace/once.rs`) belong to later steps. If
  S4 changes no signature they use, they do not change.
- **`crates/rum-runtime/src/runtime/suspension/`**, only where S4's use
  exposes a gap in the API or driver. Disclose each edit and why it is
  needed. Sealing, opacity and the no-dual-arm rule stay intact.
- **The retention analyzer's typed-macro model**
  (`crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs`,
  `regions.rs` and the four views), only if an S4 site uses a macro shape
  the model does not yet read. Every R5 obligation from brief-609 then
  applies to that shape:
  - the exact definition;
  - the drift falsifier, watched again;
  - the expansion rejection list;
  - all four views reading it;
  - an A3 fixture.
- **Sibling unit tests.** A test that needs a family module's private items
  goes in that module's sibling `tests.rs`, declared as
  `#[cfg(test)] mod tests;`. Never use `#[path = "../…"]`: the ownership
  loader does not follow `..` and fails closed.
- **Census, oracle and controls.**
  - Census: everything under `crates/rum-stdlib/tests/typed_slot_census/`
    and `typed_slot_census.rs`.
  - Oracle: `crates/rum-stdlib/tests/suspension_oracle/fixtures/` for the
    new branch-observing fixtures and the `typed_slots.rs` receipt, and
    `suspension_oracle/inventory/typed-slot-stages.tsv` and
    `typed-slot-shapes.tsv`.
  - Compile controls: rerun all 31 pairs; S4 adds none unless a new API
    shape needs one.
- **Pins that follow code.** A test or inventory row that names a touched
  file's path, line, function or count may be updated only to follow the
  same construct after it moves. Disclose each update as old → new. A row
  whose construct disappears by an owned conversion is a planned
  transition; list it. Anything else is a STOP. The director's grep found
  these pinning files; the list is a starting point, not a boundary:
  - `runtime/tests/frame_root_trace.rs` and
    `runtime/tests/frame_root_trace/derived_inventory.rs`;
  - `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv`, by
    discovery replay only;
  - `crates/rum-stdlib/src/tests/retention_closure/inventory/{obligations,local-contracts,promoted}.tsv`,
    by replay only;
  - `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs`, if a
    frontier row moves. Classify every diff as brief-609 did: a moved pair,
    an attributed S4 change, or a construction revealed or hidden by the
    typed macro. An unpaired removal is a STOP.

`Cargo.toml` files, `Cargo.lock`, the toolchain, docs, `AGENTS.md`,
`TODO.md` and the core `Expr`/`Symbol` representation stay unchanged.
Scratch goes under `.temp/brief-610/`. The certificate replay may write
`.temp/brief-546/certificates/`.

**Disclosure duty.** If a needed change falls outside every list and class,
make no edit there. STOP and report the file, the line and why.

## Steps

1. **Harness and bases first, before any source edit.**
   - Copy the resume harness from `.temp/brief-608/resume-harness/` to
     `.temp/brief-610/resume-harness/`. Add workloads that suspend in S4's
     frames. Base and candidate both run this one harness source.
   - Each workload repeats its suspension N times and checks its final
     value, like the existing five. The requirements:
     - every owned frame shape that guest code can reach suspends in at
       least one workload;
     - one workload pauses twice inside a single argument resolution, so
       D0002's repeated suspension and its finalizer run each iteration;
     - each workload's frame-call census (callgrind, as in brief-609 R6)
       shows the intended S4 frame resuming once per resume.
   - An owned frame that guest code cannot reach is reported with the
     reason.
   - Then build the four measurement bases from the committed S3 tree with
     `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, each in its own target
     directory under `.temp/brief-610/`: `rum-cli`, `rum-cli-alloc`
     (`--features rum-cli/alloc-count`), `resume` and `resume-alloc`. Hash
     them.
2. **Labels.** Copy `.temp/brief-608/perf/labels.json` and its programs into
   `.temp/brief-610/perf/`.
   - Add every `scripts/core-language-microbench.sh` label whose statement
     reaches an S4-changed function, in the same N/2N format. Confirm
     reachability by the function appearing in a base callgrind profile.
   - At minimum add `cases-wide-no-match`, `count-wide-no-match`,
     `position-wide-no-match`, `deletecases-wide-no-match`,
     `deletecases-leaf-delete-small`, `replace-wide-no-match`,
     `replace-leaf-rebuild-small`, `rewrite-guarded-hit`,
     `rewrite-guarded-decline-fallback` and `part-list`.
   - Each program checks its own result, so a label cannot time a failing
     path.
   - Add the new resume workloads as resume labels.
3. **Type the owned rows.** Convert each F row to the typed API.
   - Derive `Input` from the actual producer `Result` before any
     transformation. Bind `Output` at the innermost function or closure
     return.
   - Use complete frame and helper bodies. Keep, exactly:
     - pending-generation checks;
     - cursor advancement;
     - owner drop order;
     - held and evaluation order;
     - profiling balance.
   - Each M site uses the typed macro arm its shape calls for (see the
     table above). Its ordinary path matches the producer `Result` once and
     constructs no frame, `Arc` or `Vec`. Frame state is built in the cold
     `suspend =>` arm.
   - Keep the `ok` bodies' semantics byte-for-byte. The `(|| { … })()`
     wrappers may stay or go, but do not restructure control flow on the
     ordinary path. Move locals, not control flow.
4. **D0002.** Argument resolution follows D0001's shape:
   - `resume_suspended_argument_resolution` continues through the adapter's
     token, with fresh `SuspendedArgumentResolution` state (cursor, shared
     expressions, accumulator) and no captured `Arc`;
   - `ResumeStop` and `finish_work` select `FinishArgumentResolutionFinalizer`,
     whose `Output` is `Vec<Expr>`;
   - the finalizer runs after both originating `Arc`s drop;
   - the accumulator's `try_unwrap` uniqueness assertion stays true on
     every path, and a fixture proves it after two pauses.
5. **Oracle fixtures.** Add branch-observing fixtures, each comparing the
   queued-input path with the resumed path in value, log, messages, prints
   and trace under `Checks::ALL`, with unconditional `FaultProbe` absence.
   They cover:
   - two pauses inside one argument resolution, at the first and at a
     later argument, so the cursor, prefix and finalizer state show;
   - a pause while rewrite rules are evaluated, through each driver
     reachable from guest code (expressions, infos, compiled rules);
   - a pause in a `Cases` pattern specification;
   - a pause in each traversal configuration (`Cases`, `Position`, limit
     free).

   The oracle has no observer of frame identity, so attribution comes from
   the harness. Each fixture uses the same guest shape as a step-1 harness
   workload whose frame-call census shows the owned frame resuming, and it
   cites that census. A fixture with no such census is reported as
   unattributed.

   These fixtures assert equality that already holds at S3, so they must
   pass on the base tree too. Run them there first and record that they
   pass.
6. **Ordinary controls.**
   - Rerun the 28 round-4 ordinary controls through brief-609's harness.
     Copy `.temp/brief-609/r2/ordinary-controls/` to `.temp/brief-610/` and
     re-sync its copied `suspension_oracle/` support from the committed
     tree first; its `support/drive.rs` copy is stale. Report any remaining
     difference.
   - All 28 pass, including the four this step touches directly:
     `replace.heads`, `replace.levels`, `held.part.twice` and
     `replace.arguments.twice`.
7. **Census.**
   - Advance the receipt to S4 in `typed_slot_census` and in
     `suspension_oracle/fixtures/typed_slots.rs`, and record every owned
     row's transition.
   - The production totals stay at 607 implementations and 589 macro sites.
     Report how many are now typed (expected 23 and 25, unless a row merges)
     and the actual new or split frame names.
8. **Inventories.** Replay fingerprints, both certificate sets and ownership
   discovery, and disclose every changed row. A new `UnknownHostPayload`
   without an approved disposition is a STOP, and so is a discovery failure.
   Never hand-write a ledger chain.

## Verification (MANDATORY — run to green or evidence)

Run everything serially, each command through `python3
scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-610/tmp …`. Bounds: workspace 3 600 s; oracle
1 500 s per build; everything else 900 s. These are the durations S3's gates
used.

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

**Performance.** S4 changes paths that every `Cases`, `Position`,
`Replace`, `ReplaceAll` and evaluated-`Part` call runs, so it is measured;
there is no exemption. `TEMPLATE.md` pitfall 8 governs, with the bar
brief-609 set.

1. **Allocations and bytes first**, on every label, against the bases.
   - Ordinary labels must be exactly equal.
   - On the resume workloads, a typed entry adds exactly one allocation, the
     outer adapter `Arc`, per typed frame per resume. Report which frame
     types suspend per resume, before and after.
   - Any further difference must be attributed, at its allocation sites, to
     a named type's mode wire, as brief-609 did for `Option<Expr>`'s
     `Some`. Anything else is a defect.
   - Check the whole-program counts at N and 2N as well as the slopes. A
     constant that cancels from a slope is still work: S3's finalizer
     rebox hid there.
2. **Instruction slopes.** Two interleaved `perf stat` pairs per arm on all
   labels: `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, stdin
   `/dev/null`, slope `(count(2N) - count(N)) / N`, raw output saved before
   parsing.
   - A label is flagged when its delta exceeds `max(0.05%, base range,
     candidate range)`.
   - A flagged ordinary label is cleared only by five runs per arm, or by
     three or more callgrind processes per arm whose mean difference does
     not exceed the larger arm range, with an address ledger showing no
     operation the base does not execute.
3. **Copies and leaves.** On every ordinary label, the per-label `memcpy`
   call count (callgrind, every caller of the resolved libc symbol) is at
   most the base's. `literal-integer`, `do-count-null` and
   `do-iterator-null` execute no more instructions per iteration than the
   base, by address.
4. **Stack.** Tabulate the prologue frame size of every function S4
   changes, in debug and release, base against candidate.
   - If any grows and sits on a path a guest recursion can repeat (for
     example a rule-driven builtin inside a recursive definition), measure
     bytes per level for that shape as brief-609 R4-R6 did, in both
     profiles.
   - Acceptance: no growth per level, and the 48-level test green.
5. **Resume attribution.** For every resume workload, do a callgrind
   per-function diff plus address-level attribution, as brief-609 R6 did.
   - Partition each typed frame's added instructions into:
     - decode and encode through the mode;
     - the adapter's dispatch;
     - the second `Arc`;
     - the mixed-queue entry.
   - Name each item at its addresses, with per-resume counts. A copy or move
     of an action, slot, payload or error that the legacy arm did not make
     is a defect: remove it, or report it with its addresses.
6. **Timing** last, as an indication only: the full
   `scripts/core-language-microbench.sh` panel on both one-unit binaries,
   profiling off, with no other work on the host. Timing never overrides an
   instruction result.

A regression is a defect to fix or report.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- No default-path cost. No owned-tree clones on hot paths.
- The witness comes from the actual `Result`. Never guess the input from a
  synthetic `Err` type.
- The token carries no `Arc`, is not `Clone`, and accepts fresh state of the
  same frame type only.
- Actions, codecs and modes stay sealed and opaque. Nothing public names a
  mode or constructs an erased entry.
- Profiling records stay balanced on every path, ordinary and resumed.
- Prefer the caller-place macro arm. An address-escaped local that is
  re-bound costs a copy on every ordinary pass.
- New modules stay under 10k characters. Check
  `.temp/brief-606/file-size-assessments.tsv` before adding to a large file
  (`traversal.rs` is 20k and `rewrite_rules.rs` is 16k). Split by
  responsibility, never by arbitrary cuts.

## Checkpointing — protect the scene

Everything goes under `.temp/brief-610/`. Write `PROGRESS.md` and a
machine-readable `state.json` after each step. Record the verified state,
the compile and test counters, any half-edited files and the exact next
action. A model-capacity or usage-limit stop can end the session at any
moment, and the next session continues from these files.

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
  - `valgrind`, `callgrind_annotate` and `objdump` on the measurement
    binaries, and the measurement commands named above.

  No probe that aborts a process.
- **The Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- No dual-arm decoder, per-frame mode override, transit record or public raw
  adapter. Type no row outside S4's. Never weaken an oracle `Checks` flag, a
  retention reject, a fault-absence gate or a documented test to get a pass.
- Stop-loss is keyed on progress. Compile errors you are reducing count as
  progress. Stop after two attempts that move none of the named failing
  gates.
- Paste real output. Say plainly what did not run or did not pass. Leave
  the tree uncommitted.

## Report format (append to this brief)

1. Files changed, each with its semantic change. List every ripple, module
   root and pin update as old → new.
2. Owned-row reconciliation: each F/M row's transition, the new or split
   frame names, D0002, the removed transport maps and the census output.
3. The harness workloads and the new oracle fixtures: what each exercises,
   its frame census, and its pass on the base tree and the candidate in
   both builds.
4. All gates, verbatim. Fault-occurrence deltas. Inventory and certificate
   transitions.
5. Allocation, instruction, copy, stack, resume-attribution and timing
   evidence.
6. Stop-loss counters, deviations and open questions.

## Execution report — conversion and correctness complete; performance acceptance withheld

S4 is implemented and its correctness, oracle, census, ownership and workspace
gates are green. The ordinary allocation, instruction, copy and deterministic
leaf gates are also green. Two performance issues are reported rather than
accepted: native stack grows on 7 of 14 profile/shape pairs, and the signed
resume attribution remainder is not proved inherent. No performance acceptance
is claimed. The tree remains uncommitted.

### 1. Files changed

Production and test sources:

- `crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs`:
  typed F0247 and F0568; merged repeated argument resolution into F0247's
  witnessed token route; implemented D0002 with fresh
  `SuspendedArgumentResolution`, `ResumeStop` and `TypedAction::finish_work`.
- `crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs`:
  typed F0249-F0253 and their witnessed macro sites, preserving compile and
  continuation semantics.
- `crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal.rs`:
  typed F0254-F0256 and their caller-place macro sites.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/argument_traversal_resolution.rs`
  and `fixtures/mod.rs`: added and registered eight strict-equality fixtures.
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs` and
  `suspension_oracle/inventory/typed-slot-stages.tsv`: advanced the receipt to
  S4 and recorded the planned removals.
- `crates/rum-stdlib/tests/typed_slot_census/s4.rs`: added the closed S4
  transition gate. `typed_slot_census.rs`, `receipts.rs`, `reporting.rs` and
  `routes.rs` now check S4, its totals and its scratch paths. S3 remains as the
  historical input.
- `crates/rum-stdlib/tests/typed_slot_compile_controls.rs`: only the scratch
  path pin moved, `.temp/brief-609/compile-controls` →
  `.temp/brief-610/compile-controls`; no pair was added or removed.
- `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs`: the typed
  finalizer contract now also checks `FinishArgumentResolutionFinalizer`;
  named frame count 564 → 563 after the F0248 merge.
- `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`:
  removed the two `ResumeSuspendedArgumentResolutionSuspensionFrame` rows that
  name the merged type.
- Discovery replay updated `ownership_graph/inventory/{unknown_attributes,
  unknown_captures,unknown_macros,host_payload_ledger,self_reaching}.tsv`.
  Every row-level old → new is in `.temp/brief-610/ownership-replay.patch`.
  Notably, `ResolvedArgumentNode` moved from `arguments.rs:15` → `:18`, and
  four existing witnesses now name S4's argument frame/finalizer instead of
  the S3 prepared-call examples.
- Retention replay updated `retention_closure/inventory/local-contracts.tsv`
  and `promoted.tsv`; all 68 fingerprint transitions are enumerated in
  `.temp/brief-610/contract-fingerprint-diff.tsv`.
- This brief received this report. No module root, visibility rule, Cargo
  manifest, lockfile, toolchain, dependency, suspension API, `Expr`/`Symbol`
  representation or `unsafe` block changed. The isolated stack trial modified
  only `.temp/brief-610/stack/candidate/tree/`, never the working source.

### 2. Owned-row reconciliation

| Row | Final transition |
| --- | --- |
| F0247 | `TypedFrame<Input=Expr, Output=Vec<Expr>>`; also owns same-type repeat suspension |
| F0248 | Merged into F0247; retained removal receipt, no split type remains |
| F0568 | `SlotFinalizer<Output=Vec<Expr>>`; drains and returns the vector directly |
| F0249 | `TypedFrame<Expr -> Expr>` |
| F0250 | `TypedFrame<Expr -> Option<Vec<Expr>>>` |
| F0251 | `TypedFrame<Option<Vec<Expr>> -> Expr>` |
| F0252 | `TypedFrame<Option<Vec<Expr>> -> Option<Vec<RewriteRuleInfo>>>`; host-only in this harness |
| F0253 | `TypedFrame<Expr -> Expr>` |
| F0254 | `TypedFrame<Vec<Expr> -> CasesTraversalResolution>` |
| F0255 | `TypedFrame<Vec<Expr> -> CasesTraversalResolution>` |
| F0256 | `TypedFrame<Vec<Expr> -> TraversalWithoutLimitResolution>` |
| M0248, M0249 | Typed macro on the actual argument-evaluation `Result`; state remains cold |
| M0250 | Removed as a macro site; D0002 uses `ContinueToken::suspend` with fresh state |
| M0251-M0254 | Caller-place typed macro; the identity `Expr` transport maps at M0251/M0252 disappeared |
| M0255 | Typed macro on the rewrite-info producer `Result` |
| M0256-M0259 | Caller-place typed macro on each bound traversal producer `Result` |

D0002 captures no original `Arc`. `ResumeStop` and `finish_work` select the
pure finalizer, which runs after both originating owners drop. The
`Arc::try_unwrap` uniqueness assertion remains unchanged and is exercised
after two pauses. No State birth, registry mode flip, dual-arm decoder, public
raw adapter or transit record was introduced.

Final census, identical in default and no-default builds:

```text
S4 CENSUS production frames=532 finalizers=52 implementations=606 typed=22 macros=588 result=356 insert=232
S4 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S4 CENSUS test-support frames=5 finalizers=0 implementations=15 typed=10 macros=10 result=5 insert=5
S4 PARTITION implementations=622 macros=599
S4 D0002 witnessed token live; typed=22 macros=24
S4 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
named_suspension_frames=563
```

The expected pre-merge 607/589 totals therefore became 606/588, with 22 typed
implementations and 24 typed macro sites. F0248 and M0250 are retained
removals. `Whole0018`-`Whole0024` and `Nested0005`/`Nested0006` are the other
planned removals; `s4.rs` proves the old identities absent.

### 3. Harness workloads and oracle fixtures

All 13 harness workloads pass on the S3 base and candidate. The five inherited
workloads are `table`, `do-chain`, `call-repeat`, `cleanup` and `fold`. The
following table gives the eight new workloads and their owned frame census.
Counts are per actual resume; F0568 runs once per completed argument
resolution, not once per input pause.

| Workload / fixture | Exercised path and owned frame census |
| --- | --- |
| `argument-twice` / `s4.argument-twice` | First and later argument pause. Base: F0247 and F0248 at 0.5 each; candidate: F0247 once. F0568 finishes after the second pause. |
| `rewrite-exprs` / `s4.rewrite-exprs` | Rewrite-rule expressions: F0250 once and F0251 once |
| `rewrite-compiled` / `s4.rewrite-compiled` | Compiled rewrite driver: F0253 once |
| `cases-spec` / `s4.cases-spec` | Cases pattern specification: F0249 once |
| `cases-traversal` / `s4.cases-traversal` | Argument resolution F0247 and Cases traversal F0254, once each |
| `position-traversal` / `s4.position-traversal` | F0247 and Position traversal F0255, once each |
| `count-traversal` / `s4.count-traversal` | F0247 and limit-free traversal F0256, once each |
| `cases-shared` / `s4.cases-shared` | Shared argument/spec/traversal path: F0247, F0249 and F0254 each once per iteration (0.5 per actual pause) |

F0252 is not guest-reachable in these workloads; it is reached by host rewrite
inspection. Its existing host test passes:

```text
test runtime::tests::definitions_and_rules_rewrite_inspect::inspect_rewrite_rules_can_suspend_and_resume_with_host_input ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1171 filtered out; finished in 0.00s
```

Each fixture uses the corresponding harness guest shape and checks value,
log, messages, prints and trace under `Checks::ALL`, with unconditional
`FaultProbe` absence. All eight passed on the unmodified S3 base and candidate,
in both feature builds. Every per-fixture fault-occurrence delta is zero.

```text
# base, default and no-default; candidate, default and no-default
running 8 tests
...
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 58 filtered out; finished in 0.06s
```

The candidate harness check printed:

```text
PASS table resumes=2
PASS do-chain resumes=2
PASS call-repeat resumes=2
PASS cleanup resumes=2
PASS fold resumes=2
PASS argument-twice resumes=4
PASS rewrite-exprs resumes=2
PASS rewrite-compiled resumes=2
PASS cases-spec resumes=2
PASS cases-traversal resumes=2
PASS position-traversal resumes=2
PASS count-traversal resumes=2
PASS cases-shared resumes=4
```

### 4. Gates, inventories and certificates

All commands ran serially, offline, with one build job and one test thread,
under `scripts/run-bounded.py`. Exit status is zero for every row.

| Gate | Verbatim result / summary |
| --- | --- |
| `cargo fmt --check` | No output |
| typed runtime | `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.50s` |
| frame roots | `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 2.28s` |
| exact 48-level test | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1171 filtered out; finished in 0.01s` |
| compile controls, default | `COMPILE CONTROLS: 31 positive / 31 intended rejections (22 S1 + 7 S2 + opaque fault + token witness); raw Cargo JSON and primary spans retained` |
| compile controls, no-default | Same 31/31 line; `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.38s` |
| census, default | `test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 138.11s` |
| census, no-default | `test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 140.48s` |
| ownership graph | `test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 70.28s` |
| local production, default | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 18.66s` |
| local production, no-default | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 18.51s` |
| production rejection census | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1157 filtered out; finished in 26.69s` |
| ordinary controls, default | `ORDINARY CONTROLS: 28 PASS, Checks::ALL; unconditional FaultProbe absence` |
| ordinary controls, no-default | Same 28-pass line; all 28 fault deltas zero, 336 observations |
| native ZeroMQ | `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.50s` |

The two complete oracle builds emitted no warnings and reported:

```text
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88112
test result: ok. 66 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.99s
...
test zz_continuation_fault_occurrences ... continuation-slot fault-occurrence delta=0; observations=88112
test result: ok. 66 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.18s
```

The catalog lines were `hash=true installed=643 manifest=643` and
`hash=false installed=637 manifest=637`; lifecycle obligations were 4 026 per
build. The nine native ZeroMQ results above are actual passes; there were no
skips in this environment. The only compiler warnings are the three existing
runtime test-helper dead-code warnings for `register_test_if_builtin`,
`register_test_return_builtin` and `register_test_vm_template_builtins`.

The full workspace rerun with the proper 3 600-second bound passed. Its
verbatim `test result:` lines are:

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 41.80s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.10s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.31s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 1169 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 42.94s
test result: ok. 1105 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1365.63s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.99s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.26s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 66 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.22s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 142.41s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.44s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.57s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.67s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.72s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Discovery found no new `UnknownHostPayload`: 79 host identities and 59 cycles
remain, and 31/31 ownership tests pass. Final unknown inventories are 1 513
attributes, 2 948 captures, 3 153 macros and 94 storage rows, 7 708 constructs
in total. The replay removed the F0248 derive row, six old `|value|` captures
and the old legacy macros; it added the D0002 `|resolved|` capture and the new
typed macros, with every moved row preserved in the patch. Existing negative
controls stayed byte-identical.

All 68 contract fingerprints were refreshed by replay, not hand-written. Four
C01/C02 certificate cells (hash true/false) replayed successfully; both local
production acceptance builds pass. The historical retention frontier is
byte-identical to S3: 937 rows, `RET001=10`, `RET002=901`, `RET003=26`, with no
added or removed row.

### 5. Performance evidence

Measurement binaries, reverified after all work:

```text
55f0214690dfef2a7b77f12db24690180a9937227e58d59729b241ff31cf8385  base/rum-cli
9d1482c6389c3715b19135b194c05892d778649b893c741164bf78b6ac953934  base/rum-cli-alloc
e795b10353c4658925e3bcd441f71ff4eb866c943f26c1f0f24641ff48918a32  base/resume
9908a2341f38df01e488218657f4755e70445914da5d46a297a57fdc8215c9c7  base/resume-alloc
b5b6abadd170bcd01cbf75b915ed2d47da3dc9dbbd070e1652cba277d7ebb194  candidate/rum-cli
77448691768cd39c91c9820815df1c466042eb7d8049c6f66aa716a4c4bb0bf2  candidate/rum-cli-alloc
ff3ff8139e3f7be3a84c2660d979c674e908ce18b888278fa654597ea03209f5  candidate/resume
d7060084db98dbd51f53b3fea483fbc61d7f14da7a396b352055faefd6530c91  candidate/resume-alloc
```

There are 77 ordinary and 13 resume labels; base and candidate value checks
pass 90/90. Seventeen added ordinary labels have owned-function base
Callgrind reachability; `part-list` is the required control without an owned
resolver in its reduced base profile.

**Allocations.** All 77 ordinary labels have exactly equal count and byte
slopes and exactly equal whole-program allocation counts at N and 2N. Their
whole-program byte counts differ only by +5 at both sizes, the constant length
of `/candidate` minus `/base` in the binary path. After that harness-path
adjustment they are exactly equal. Every raw count intercept is zero. Final
whole-program gate:

```text
ORDINARY WHOLE PROGRAM: 77/77 exact allocation counts at N and 2N; exact count and byte slopes; raw byte excess is a constant +5 harness-path bytes at both sizes
RESUME INTERCEPTS: 13/13 allocation-count intercept=0 and byte intercept=+5 harness-path bytes
```

The eight new resume allocation slopes per iteration are:

| Workload | Allocations | Bytes | Typed frames accounting for the slope |
| --- | ---: | ---: | --- |
| argument-twice | +2 | +48 | F0247 once per actual pause, two pauses |
| rewrite-exprs | +2 | +48 | F0250 and F0251 |
| rewrite-compiled | +1 | +24 | F0253 |
| cases-spec | +1 | +24 | F0249 |
| cases-traversal | +2 | +48 | F0247 and F0254 |
| position-traversal | +2 | +48 | F0247 and F0255 |
| count-traversal | +2 | +48 | F0247 and F0256 |
| cases-shared | +3 | +72 | F0247, F0249 and F0254 once per iteration |

These are the outer 24-byte adapter `Arc` allocations, one per typed frame per
entry. All five inherited resume labels have zero count and byte slopes.

**Ordinary instructions and copies.** Two interleaved hardware pairs per arm
flagged eight ordinary labels. Five additional repeats per arm cleared every
one; three-pair Callgrind means independently satisfy the larger-arm-range
rule:

| Label | Five-run delta / threshold | Callgrind mean delta / larger range |
| --- | ---: | ---: |
| lazy-range-normal-small | -18.330 / 74.335 | +6.263 / 150.190 |
| lazy-first-small | -13.435 / 49.200 | +1.911 / 98.508 |
| lazy-subsets-take-small | +30.623 / 63.317 | -37.806 / 35.202 |
| lazy-depth-nest-map-64 | -681.031 / 4 915.592 | +374.397 / 1 898.238 |
| replace-leaf-rebuild-small | +205.767 / 1 267.156 | -173.977 / 591.784 |
| rewrite-guarded-decline-fallback | +25.706 / 281.976 | +41.321 / 157.519 |
| set-part-fixed100-len800 | +512.956 / 1 379.320 | +689.800 / 1 824.333 |
| set-part-fill-small | -1 034.976 / 9 940.950 | +408.524 / 694.571 |

The complete ordinary address ledger has 59 436 base and 59 438 candidate
address rows and 2 623 differences, with every raw label delta reconciled to
addressed rows. Six objectless rows at `0x4071118`, `0x4071128` and
`0x4071160` per arm have identical count maps and zero signed contribution.
Library-only startup/IFUNC variations are retained rather than used as a
clearance claim; the five-run results above clear the flags independently.

The resolved libc copy target is `0x1b1980`,
`__memcpy_avx_unaligned_erms`. All 77 labels have candidate `memcpy` calls at
or below base; `Changed copy functions: 0`. The three deterministic leaves
have zero addressed changes and exactly 110 `Session::evaluate`
self-instructions in every sample on both arms:

```text
literal-integer {'base': 110.0, 'candidate': 110.0} 0 addressed changes; register/stack differences retained
do-count-null {'base': 110.0, 'candidate': 110.0} 0 addressed changes; register/stack differences retained
do-iterator-null {'base': 110.0, 'candidate': 110.0} 0 addressed changes; register/stack differences retained
```

**Resume instructions and address attribution.** Three Callgrind pairs per
arm cover every workload. The per-function mean differences and the complete
object/address ledger are preserved. New-workload totals are:

| Workload | Mean delta per iteration | Mean delta per actual resume |
| --- | ---: | ---: |
| argument-twice | +225.405 | +112.703 |
| rewrite-exprs | +357.232 | +357.232 |
| rewrite-compiled | +205.740 | +205.740 |
| cases-spec | +199.038 | +199.038 |
| cases-traversal | +265.990 | +265.990 |
| position-traversal | +224.022 | +224.022 |
| count-traversal | +350.782 | +350.782 |
| cases-shared | +646.383 | +323.192 |

Inherited workload deltas per iteration are `table=-38.658`,
`do-chain=-12.650`, `call-repeat=+20.250`, `cleanup=-12.110` and
`fold=-61.590`.

Physical ELF entries and resolved calls distinguish identical-code-folded
logical frames. The adapter partition names each physical body, its mode work
and second-`Arc` instructions:

| Physical adapter / logical frame(s) | Mode rows / decoder or encoder calls | Second-`Arc` addresses |
| --- | --- | --- |
| `0x758710`, F0253 | 37 inline mode rows | `0x75879b`, `0x75879e`, `0x7587a2` |
| `0x7591d0`, F0254 and folded F0255 | 43; `Vec<Expr>` decode at `0x759239` | `0x75925a`, `0x75925d`, `0x759261` |
| `0x7597d0`, F0256 | 43; `Vec<Expr>` decode at `0x759839` | `0x75985a`, `0x75985d`, `0x759861` |
| `0x7599d0`, F0251 | 45; `Option<Vec<Expr>>` decode at `0x759a3d` | `0x759a61`, `0x759a64`, `0x759a68` |
| `0x759be0`, F0247 | 37 inline mode rows | `0x759c6b`, `0x759c6e`, `0x759c72` |
| `0x759d70`, F0249 | 37 inline mode rows | `0x759dfb`, `0x759dfe`, `0x759e02` |
| `0x759f00`, F0250 | 37 inline mode rows | `0x759f89`, `0x759f8c`, `0x759f90` |
| `0x75a5d0`, F0568 finalizer adapter | 37; `Symbol::system` at `0x75a612`, `Expr::call` at `0x75a63e` | None |

`address-partition.tsv` assigns every self address to mode, adapter dispatch,
second `Arc`, mixed queue, or an explicit reported remainder. Inclusive,
non-additive call roots separately record external mode decode/encode, outer
adapter allocation/destruction and direct mixed-queue insertion. Per-actual-
resume signed contributions are:

| Workload | Mode | Adapter dispatch | Second `Arc` | Mixed queue | Reported remainder |
| --- | ---: | ---: | ---: | ---: | ---: |
| argument-twice | 20.852 | 18.500 | 66.425 | -30.765 | 37.691 |
| rewrite-exprs | -125.142 | 30.000 | 173.305 | -106.605 | 385.673 |
| rewrite-compiled | 73.715 | 13.000 | 57.380 | -63.315 | 124.960 |
| cases-spec | -25.307 | 13.000 | 80.910 | -33.120 | 163.555 |
| cases-traversal | 30.667 | 42.000 | 96.370 | -89.240 | 186.193 |
| position-traversal | -42.350 | 42.000 | 215.630 | -91.910 | 100.652 |
| count-traversal | 29.795 | 42.000 | 197.830 | -83.010 | 164.167 |
| cases-shared | 197.258 | 27.500 | 159.500 | -66.215 | 5.148 |

Negative entries are credits from removed baseline work, not negative resource
consumption. The remainder contains moved bodies, shared callees, hash
variation and helper ABI work. It is **not** declared inherent, and the table
is not presented as a zero-defect proof. Typed adapters contain no `memcpy`.
Cases-spec has the same two copy calls per resume in both arms and compiled
rewrite has the same one; argument resolution removes its former 0.5 copy per
actual pause. No additional action/slot/payload/error copy was found in the
owned physical bodies. Full paired bodies and copy windows are in
`paired-frame-differences.json` and `frame-copy-review.json`.

**Native stack.** Four runtime-test binaries (debug/release, base/candidate)
plus four matched probe binaries were built. Prologues for all owned symbols,
including inlined/absent identities, are in `stack/changed-prologues.tsv`.
The ten matched growth rows are:

```text
debug  504 -> 520   DriveRewriteRulesOwnedSuspensionFrame::resume
debug  888 -> 1048  resolve_argument_exprs_from_cursor
debug  888 -> 1064  resolve_argument_exprs_shared_from
debug  856 -> 984   resume_suspended_argument_resolution
debug  168 -> 312   drive_rewrite_rule_exprs_owned::{{closure}}
debug   40 -> 56    drive_rewrite_rule_infos_owned::{{closure}}
release 328 -> 360  resolve_argument_exprs_owned
release 312 -> 344  drive_rewrite_rules_owned
release 408 -> 424  resolve_cases_traversal_owned
release 488 -> 520  resolve_cases_traversal_shared_from
```

Seven recursive guest shapes were then measured at four heights and three
fresh repetitions per arm/profile. All slopes have zero residual. The
unchanged 1 048 576-byte native guard and libtest stack were used,
`RUST_MIN_STACK` was absent, and no aborting probe was used. Values are bytes
per guest level:

| Profile | Shape | Base | Candidate | Delta / pass |
| --- | --- | ---: | ---: | --- |
| debug | argument | 58 768 | 58 768 | 0 / PASS |
| debug | rewrite | 80 352 | 79 968 | -384 / PASS |
| debug | compiled | 79 760 | 79 568 | -192 / PASS |
| debug | cases-spec | 91 856 | 91 696 | -160 / PASS |
| debug | cases-traversal | 82 688 | 82 720 | **+32 / FAIL** |
| debug | position | 79 424 | 79 456 | **+32 / FAIL** |
| debug | count | 67 936 | 67 968 | **+32 / FAIL** |
| release | argument | 24 048 | 24 048 | 0 / PASS |
| release | rewrite | 31 248 | 31 216 | -32 / PASS |
| release | compiled | 31 104 | 31 136 | **+32 / FAIL** |
| release | cases-spec | 36 480 | 36 480 | 0 / PASS |
| release | cases-traversal | 31 488 | 31 536 | **+48 / FAIL** |
| release | position | 30 784 | 30 832 | **+48 / FAIL** |
| release | count | 25 568 | 25 616 | **+48 / FAIL** |

Thus native no-growth is 7/14, not a pass. The unchanged fully qualified
48-level test is nevertheless green. An isolated trial binding the two
argument producers at caller place did not change any of these 14 slopes; it
was not promoted. The remaining regression is reported for director review,
not rationalized as inherent typed-mode cost.

**Timing, indication only.** The final full panel ran last on the two CGU1
CLI binaries, scale 1, seven repetitions, profiling off, with five quiet
seconds before each arm and no observed competing process above 0.15 core.
GPU idleness was unavailable from sysfs and is not claimed. Both arms produced
120 in-kernel `AbsoluteTiming` rows:

```text
labels 120 median ratio 0.99905178104074 slower 58
```

The largest ratios include `do-iterator-null=2.62857`,
`table-count-null=2.52552`, `table-iterator-null=2.48258`,
`loop-empty=2.47484` and `literal-integer=2.44470`. These single-panel timing
outliers are reported, not hidden by the median; they do not override the
equal addressed instruction and copy results for the deterministic leaves.

### 6. Stop-loss, deviations and open questions

Stop-loss is **1**: the isolated caller-place stack trial moved none of the 14
native-stack gates. A second unproductive production attempt was not made.
Earlier scratch-only corrections fixed the Callgrind slope reducer, per-label
probe-template handling, resume-analysis import and adapter-count assertion;
none edited production behavior. Initial debug depth 16 and release depth 40
rewrite probes reached the guard and returned `StackLimit` normally, so the
measurement ranges were reduced to 2/4/6/8 and 5/10/15/20; no process-abort
probe was used. An administrative 300-second workspace bound was replaced by
the brief's 3 600-second bound, whose complete results appear above.

Open for director decision:

1. The seven native-stack growth rows in the table above. Performance
   acceptance is withheld.
2. The signed resume attribution remainder and helper ABI work are reported,
   not proved inherent. The full addressed ledgers remain available for
   further review.
3. Timing outliers are single-panel indications only. No timing-derived source
   change was attempted.

No correctness gate is open, no production file is half-edited, no background
task remains, and no git command was run.


## Director amendment — redispatch 1: the typed macro stops costing stack

The report is honest and thorough, and withholding acceptance was the right
call. The native-stack growth is real. Its pattern also names the cause as a
class rather than three sites.

### Accepted as reported (re-verified at my gate)

- Correctness: the census (606/588, typed 22/24, F0248 merged into F0247
  as a same-type repeat), oracle 66/66 in both builds, controls 28/28 in
  both, ownership 31/31, the workspace, and nine actual ZeroMQ passes.
- The ordinary allocation, instruction, copy and leaf gates.
- Resume allocations: one 24-byte outer adapter `Arc` per typed frame per
  entry, matched workload by workload. This is the brief's allowance.
- The timing outliers. I re-ran `literal-integer` and `do-iterator-null`
  under `taskset -c 2`, five interleaved N/2N pairs per arm. Instructions per
  iteration were equal, cycles per iteration were within ±8%, and nothing
  came near 2.5x. Your panel ran each arm whole and unpinned on a host whose
  P- and E-cores differ by 1.27x, so the outliers come from the host, not
  the code. C3 changes the method.

### A. The stack: the typed macro's expansion

**What the report shows.** In `traversal.rs` the four resolvers changed
only in the macro name, `resume_or_suspend!` → `resume_typed_or_suspend!`.
The frame impls are not on the guest-recursion path. Even so, in release
`resolve_cases_traversal_owned` grew from 408 to 424 bytes and
`resolve_cases_traversal_shared_from` from 488 to 520.
`resolve_argument_exprs_owned` (328 → 360) and `drive_rewrite_rules_owned`
(312 → 344) also hold converted macro sites.

So the expansion itself very likely costs 16-32 bytes in the frame that
contains it. S5-S54 will put that macro at roughly 560 more sites (588 production macro sites, 24 typed today), many of
them on guest-recursive paths. Fixing S4's three sites one by one would close
the instances seen and leave the class open.

The report does not yet reconcile the shapes. Debug `cases-traversal`,
`position` and `count` grew by 32 bytes per level, but no debug growth row in
your table lies on that path. Debug `argument` grew by zero even though three
argument-resolution functions grew by 128-176 bytes.

**Ruling A.**

1. **Reconcile before changing anything.** For each of the 14
   profile/shape pairs:
   - list every frame that stays live across one guest level, from the call
     path that shape actually takes;
   - show that the measured per-level delta equals the sum of those frames'
     prologue deltas, as brief-609 R4 did.

   Include generic instantiations, closures, and functions outside the
   owned files. A pair whose sum does not match is not yet understood. Do
   not fix it by trial.
2. **Attribute each growing frame to its slots.** Compare the base and
   candidate disassembly of the function and name the value that each new
   or enlarged stack slot holds.

   Test this hypothesis first; do not assume it. The cold helper takes its
   closure by value. That closure holds the moved `RuntimeInputRequest`,
   everything `$frame` captures, and the session reference, so it is built in
   the caller's frame, in a slot the legacy arm never needed. The legacy arm
   re-wrapped `request` in place and built its frame inline.
3. **Fix where the cause is.**
   - **If the cause is the macro expansion, fix the macro**
     (`suspension/typed/witness.rs`, authorized for this). Do not patch
     each site. These stay as they are:
     - the witness comes from the actual `Result`;
     - the producer is matched once;
     - nothing is constructed on the ordinary path;
     - the suspend arm stays cold and out of line;
     - the ordinary path makes no new copy (the caller-place rule stands).

     Every R5 obligation from brief-609 then applies to the new expansion:
     - the analyzer model's exact definition;
     - the drift falsifier, watched again;
     - the expansion rejection list;
     - all four views;
     - an A3 fixture.

     S3's sites move with the macro. Re-measure S3's two shapes, the
     `peel` test's shape and `f[k_] := f[k - 1] + 1`, with
     `.temp/brief-609/r6b/stack/`, in both profiles.
   - **If a cause is site-specific, fix it at the site.** Examples: an
     argument-resolution local that lives across the recursive call, or a
     debug-only closure layout.
   - Move locals, not control flow. The 48-level test, the guard, the stack
     size and `RUST_MIN_STACK` stay unchanged.
4. **Acceptance.**
   - All 14 pairs are at or below base, in both profiles. A pair that shrank
     stays at or below base.
   - If the macro changed, S3's two shapes are also at or below base in both
     profiles.
   - The 48-level test passes.

   If any growth remains, attribute it at its addresses to a named inherent
   item, then STOP. Do not accept it yourself.
5. **Stop-loss.** Progress means either of two numbers falls:
   - the summed excess over base across the 14 pairs, now 272 bytes
     (3 × 32 debug, 32 + 3 × 48 release);
   - the count of failing pairs, now 7.

   Stop after two attempts that lower neither, and report the reconciliation
   and attribution.

### B. Name the resume remainder

The brief asked for each item to be named at its addresses, with per-resume
counts. The partition leaves a signed remainder of up to 386 instructions per
resume (`rewrite-exprs`), which is larger than that workload's whole delta.

On the final binaries, split every remainder whose magnitude exceeds 25
instructions per actual resume into named items. Give each item its
addresses and per-resume count, and put it in exactly one class:

- **a moved body:** from which function to which, with the net count;
- **a callee whose call count changed:** which callee, and why;
- **process-to-process variation:** show it lies within the same arm's
  three-process range;
- **helper ABI work:** which helper, and which instructions.

A move or copy of an action, slot, payload or error that the legacy arm did
not make is a defect, and you remove it. Report anything that fits no class,
with its addresses, as unexplained. A remainder that shrinks because A
changed the code is fine. The attribution is of the final binaries.

### C. Then finish the step, on the final source

1. **Every correctness gate in the brief:**
   - fmt and the typed runtime tests;
   - frame roots and the 48-level test;
   - compile controls, the census, local production and ordinary controls,
     each in both builds;
   - ownership and the production rejection census;
   - both oracle builds, warning-free;
   - native ZeroMQ and the full workspace.

   Then replay the inventories, certificates and fingerprints, and disclose
   every changed row. If the macro changed, the A3 obligations apply too,
   and you classify the historical frontier against S3 as brief-609 did.
2. **Every performance section, against the same frozen bases:**
   - allocations, including whole-program counts at N and 2N;
   - the instruction pairs and the clearance of flagged labels;
   - copies and leaves;
   - the stack table;
   - resume attribution with B.

   Rebuild the candidate binaries. The bases are unchanged: verify their
   hashes.
3. **Timing last, pinned.** Run `scripts/core-language-microbench.sh` under
   `taskset -c 2` for both arms in the order base, candidate, base,
   candidate, which gives two full passes per arm. Report per-label ratios
   for each pairing.
   - A deterministic leaf (`literal-integer`, `do-count-null` or
     `do-iterator-null`) outside [0.8, 1.25] in either pairing is re-run
     alone before you report it: five interleaved runs per arm.
   - Timing remains an indication only.

### Process

- **Tool timeouts.** Your tool timeout must exceed the
  `run-bounded.py --timeout` you pass. Your first workspace run was killed
  by a 300-second tool timeout while the command's own bound was 3 600
  seconds.
- **Checkpoint as before.** Then append a new
  `## Execution report — redispatch 1` after this section.

**Newly authorized:** `crates/rum-runtime/src/runtime/suspension/typed/witness.rs`,
for A3 only. If the macro changes, the retention analyzer's typed-macro
model and its four views, already named in Scope, follow it. The disclosure
duty covers anything else.

## Execution report — redispatch 1

**STOP on A's two-attempt stop-loss.** The original working source and macro
remain unchanged; neither isolated macro trial was promoted. Both attempts
left seven failing pairs and increased the summed excess from 272 to 7 936
bytes, so neither lowered either progress metric. B's final-binary remainder
review, C's rebuild/gates and the pinned B,C,B,C timing panel were not run.
No performance acceptance is claimed.

### A1. Complete native-path reconciliation

Before either trial, I rebuilt the original base and candidate with a
scratch-only map/backtrace marker and captured all seven shapes at depth 3 in
both profiles. `stack/*/reconcile/*/resident-paths.json` contains every live
native frame, including generic instantiations, closures and functions outside
the owned files. `stack/frame-deltas.tsv` contains the base/candidate grouping
for every frame. Marker slopes at four heights and three repetitions have zero
residual.

All 14 slopes equal the complete resident-frame sum plus outgoing stack area:

```text
RECONCILED_WITH_OUTGOING 14 / 14
```

Thirteen pairs reconcile from prologue deltas alone. Release `count` has a
separate +16-byte outgoing/ABI residual:

```text
PROLOGUE_ONLY_RECONCILED 13 / 14
{"profile": "release", "shape": "count", "base_slope": 25568.0, "base_resident": 24896, "base_outgoing": 672.0, "candidate_slope": 25616.0, "candidate_resident": 24928, "candidate_outgoing": 688.0, "frame_delta": 32, "outgoing_delta": 16.0, "measured_delta": 48.0, "prologue_only_reconciled": false, "reconciled": true}
```

The nonzero frame deltas on the original binaries are:

| Profile / shape | Function delta | Net |
| --- | --- | ---: |
| debug rewrite | `drive_rewrite_rule_exprs_owned` 984 → 744; `resolve_rewrite_rule_exprs_owned` 616 → 472 | -384 |
| debug compiled | `drive_rewrite_rules_owned` 824 → 632 | -192 |
| debug cases-spec | `resolve_cases_spec_expr_for_matching_owned` 776 → 616 | -160 |
| debug cases-traversal | `resolve_argument_exprs_from_cursor` 888 → 1 048 (+160); `resolve_cases_traversal_owned` 648 → 520 (-128) | +32 |
| debug position | The same argument +160; `resolve_position_traversal_owned` 648 → 520 (-128) | +32 |
| debug count | The same argument +160; `resolve_traversal_without_limit_owned` 648 → 520 (-128) | +32 |
| release rewrite | `drive_rewrite_rule_exprs_owned` 360 → 328 | -32 |
| release compiled | `drive_rewrite_rules_owned` 312 → 344 | +32 |
| release cases-traversal | `resolve_argument_exprs_owned` 328 → 360 (+32); `resolve_cases_traversal_owned` 408 → 424 (+16) | +48 |
| release position | `resolve_argument_exprs_owned` +32; `tree_walkers::position::options::finish_position_option` 696 → 712 (+16) | +48 |
| release count | `resolve_argument_exprs_owned` +32, plus the +16 outgoing residual above | +48 |

Debug `argument` has no frame-size delta on its actual path, so its zero native
slope is now explained rather than contradicted by the unrelated growth table.
Release `cases-spec` and both `argument` slopes likewise have zero frame delta.
The original release `count` decomposition is fully accounted numerically, but
its +16 outgoing residual is not asserted inherent.

### A2. Stack-slot attribution

The converted macro's cold closure is present in the growing functions and
holds exactly the values identified by the amendment: the moved
`RuntimeInputRequest`, the frame's captured state and the session reference.
It is not a guessed type or a new ordinary-path construction.

- Debug `resolve_argument_exprs_from_cursor` (base entry `0x1b7c70`,
  candidate `0x1bb370`, 888 → 1 048): candidate instructions
  `0x1bb67d`–`0x1bb6e0` build a 96-byte closure at stack offsets
  `0x270`–`0x2cf`. The two 16-byte slots at `0x270`/`0x280` hold the request;
  `0x290`–`0x2a0` the shared expressions; `0x2a8`–`0x2b8` the frozen
  accumulator; `0x2c0` the session reference; and `0x2c8` the cursor. The call
  at `0x1bb6e8` passes that closure to the cold suspend helper.
- Release `resolve_argument_exprs_owned` (base `0x52fc10`, candidate
  `0x52f880`, 328 → 360): the same 96-byte environment is built at
  `0x52fa69`–`0x52fac4`, in slots `0x50`–`0xaf`, and passed at `0x52face`.
  Slot reuse makes the net prologue increase 32 rather than 96 bytes.
- Release `drive_rewrite_rules_owned` (base `0x5375d0`, candidate `0x5367f0`,
  312 → 344): `0x536917`–`0x536969` constructs the cold closure. The request
  occupies `0x68`–`0x87`, the `CompiledRewriteRuleContinuation` occupies
  `0x20`–`0x60`, and the session reference is at `0x88`; the suspend call is
  `0x536971`.
- Release `finish_position_option` (base `0x797130`, candidate `0x798150`,
  696 → 712): its prologue moves `sub $0x288,%rsp` → `sub $0x298,%rsp`.
  Candidate instructions `0x7984e7`–`0x798544` build the position closure:
  request at `0xe0`–`0xff`, `CasesTraversalConfig` at `0xb0`–`0xdf`, and session
  reference at `0x100`; `0x79854c` calls the cold position suspend helper.
- The release count's remaining +16 is not an added instruction. The
  `suspend_option_read` call moves from base `0x792fe8` to candidate
  `0x794008` inside `resume_count_after_pattern`; its surrounding argument
  construction is instruction-for-instruction identical. The measured
  decomposition is +32 from `resolve_argument_exprs_owned` and +16 outgoing
  ABI area. No distinct new slot or inherent item is claimed for the latter;
  it remains reported as an ABI residual.

The other release traversal growth (`resolve_cases_traversal_owned`,
`resolve_cases_traversal_shared_from`) has the same converted macro and config
capture shape. The native cycle table above, rather than a blanket assumption
that frame impls are recursive, drives this attribution: the typed frame impls
are not on these guest-recursive paths.

### A3. Isolated macro trials and stop-loss

Both trials changed only `.temp/brief-610/stack/candidate/tree/.../witness.rs`
and their scratch binaries. They removed the nested `FnOnce` closure from both
macro arms and called `witness.suspend` directly. `direct-cold` additionally
marked the method cold/out-of-line; `direct-inline` left the original method
attributes unchanged. No working-source macro, analyzer or production fixture
was edited.

The variants have identical debug results. Their seven release failures all
close, but every debug shape becomes worse:

```text
# direct-inline; direct-cold has the same debug values and failing count
Native no-growth rows: 7 / 14
debug argument        58768 -> 59760  (+992)
debug rewrite         80352 -> 81216  (+864)
debug compiled        79760 -> 80800  (+1040)
debug cases-spec      91856 -> 93136  (+1280)
debug cases-traversal 82688 -> 84032  (+1344)
debug position        79424 -> 80768  (+1344)
debug count           67936 -> 69008  (+1072)
```

Summed positive excess is therefore 7 936 bytes against the original 272,
with seven failures unchanged. This is two attempts that lower neither A
metric, so the required stop fired. S3's two shapes were not re-measured
because neither macro edit reached the final source; no A3 production
obligation is claimed complete.

### B and C: not run

Because A stopped, the following were not run on a new final binary:

- B's >25-instruction resume-remainder split;
- rebuilt candidates and repeated correctness, oracle, inventory,
  certificate and fingerprint gates;
- repeated allocation, instruction, copy, leaf and resume-attribution panels;
- S3 `peel`/arithmetic stack measurements on a promoted macro; and
- the pinned base, candidate, base, candidate timing panel and any required
  deterministic-leaf reruns.

The earlier green correctness evidence remains attached to the unchanged
working source, but it is not represented as redispatch-1 final-source
acceptance. No git command was run, no background task remains, and the tree
is uncommitted.


## Director amendment — redispatch 2: keep the cold closure, take the request and the session out of it

A1 and A2 are what I asked for, and they settle the cause. The reconciliation
is exact (14/14). In every growing release frame the growth is the cold
closure's environment, and A2 names its slots. Two of those slots duplicate
values the caller already holds:

- the moved `RuntimeInputRequest`, 32 bytes;
- the session reference, 8 bytes.

The legacy arm re-wrapped `request` in place and never copied it. Those 40
bytes, rounded by slot alignment and reuse, match the +16/+32/+48 frame
deltas.

**Why the two trials could not have worked.** Both removed the closure, and
the second changed only the method's attributes. Removing the closure closed
the release rows, but every debug shape grew by 864-1 344 bytes per level.
The likely reason is that the macro is shared. S3's sites on the evaluator's
recursive path, which every shape crosses, use the same expansion. S3
redispatch 4-5 bought its debug depth with exactly this cold closure: it
moves frame construction, and its debug temporaries, out of the caller.
**Confirm this attribution** by listing the debug frames that grew in either
trial. If it is not S3's sites, say what it is.

So the closure is right, and its environment is too big. The stop-loss fired
correctly. This amendment is a new hypothesis, so the counter restarts.

### Ruling

1. **The shape to try first.** Keep the cold, out-of-line helper and the
   lazy closure. Pass the request and the session to the helper as
   parameters, so the closure captures only what `$frame` uses. For example:

   ```rust
   return suspend(witness, $session, request, move || $frame);
   // where the cold, never-inlined helper is
   fn suspend<I, F>(witness: SlotWitness<I>, session: &mut Session,
                    request: RuntimeInputRequest, make: impl FnOnce() -> F)
       -> Result<F::Output, EvalError>
   where F: TypedFrame<Input = I> { witness.suspend(session, request, make()) }
   ```

   - Where the helper lives is your choice: inside the macro arm as today,
     or as a `#[doc(hidden)]` method or function next to `SlotWitness` in
     `witness.rs`. Nothing public may name a mode or construct an erased
     entry.
   - A `$frame` expression that itself uses the session would conflict with
     the separate `&mut`. The compiler will say so. Report any such site;
     do not restructure it.
2. **Measure it the same way, on more shapes.**
   - The 14 S4 pairs.
   - S3's two shapes, `peel` and `f[k_] := f[k - 1] + 1`
     (`.temp/brief-609/r6b/stack/`), in both profiles, against base. The
     macro change moves S3's 13 sites too, and the release side may gain.
   - The reconciliation and per-frame slot table for every frame that still
     grows.
3. **Acceptance** is unchanged: all 14 pairs and S3's two shapes at or below
   base in both profiles, and the 48-level test green.
4. **If a residual remains,** derive the next attempt from its slot table,
   not from an attribute variation:
   - capture by reference what `$frame` only reads;
   - build a `$frame` component inside the closure rather than before it;
   - for a site-specific temporary, fix it at the site.

   Stop-loss is unchanged: progress is the summed excess over base (now 272
   bytes across the 14 pairs; add S3's two shapes, at 0 today) or the count
   of failing pairs. Stop after two attempts that lower neither, with each
   attempt's slot table.
5. **Promote only what passes.** Then the R5 obligations from redispatch 1
   apply to the final expansion, followed by B and C from redispatch 1 in
   full, including the pinned timing panel.

Append `## Execution report — redispatch 2` after this section.
