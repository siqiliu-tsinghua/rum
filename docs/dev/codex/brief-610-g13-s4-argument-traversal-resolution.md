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
