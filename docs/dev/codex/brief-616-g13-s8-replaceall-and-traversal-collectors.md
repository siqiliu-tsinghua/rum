# brief-616 — G13 step S8: type ReplaceAll and the traversal collectors

## Goal

Type the sixth production family through the typed API. S8 owns the frames
that suspend while rum applies rewrite rules or walks a tree to collect
matches:
- `ReplaceAll` (`/.`), `ReplaceAllUnheld` and `ReplaceRepeated` (`//.`),
  through the ReplaceAll traversal;
- the rule-list scan that `Replace`, `ReplaceAt` and every traversal node
  use;
- `Count`, `MemberQ` and `FreeQ`;
- `Position`.

A pause can come from the target argument, the pattern argument, a level
spec or limit, or a pattern guard partway through a traversal.

When you are done:

- the owned rows implement the typed API, and the owned macro sites use the
  witnessed typed macro or the token, as their call path requires;
- the four owned transport encodings (T0018, T0019, T0089, T0090) are gone.
  Each frame completes its typed helper result directly;
- every other family still runs through the transport it uses today;
- the ordinary path of every program executes no more work than at S7, and
  the cost of S8's resume path is measured and attributed.

This is step S8 of G13 (design round 5, brief-606, ratified). S1-S7 (briefs
607-611, 614 and 615) and G14 (612-613) are done; S7 is `a1359ba8`. S9-S54
type the remaining families, F1-F23 flip transport per type, and R removes
the legacy machinery.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- Authority, in order: this brief; brief-606 (its D1/D2 and report) with
  `.temp/brief-606/design.md` and `census-spec.md`; `.temp/brief-605/design.md`
  §§2-5, **except §5's transit record, which D2 rejected**.
- **S3-S7 as built are your template.** Read brief-611, brief-614 and
  brief-615 in full, with their director amendments and gate records. Their
  rulings are now the rules:
  - **Two arms on the typed macro.** `resume_typed_or_suspend!` is defined in
    `suspension/typed/witness.rs`. Use the caller-place arm (`$result:ident`)
    wherever the producer's result is already bound to a local. Its third
    arm (`evaluation_return =>`) is for G14's two fixed-point sites only.
  - **The macro expansion is fixed.** Do not change `witness.rs`.
  - **The witness and the request cross a cold call, never the producer's
    `Result`.** Take the ZST witness at the producer, then pass
    `(witness, request)` and the frame's frozen state to a named
    `#[cold] #[inline(never)]` helper. A shape that needs an `unreachable!`
    arm for a state the types allow is the wrong shape.
  - **Start from S7's final shape, not its first one.** S7's first candidate
    bound the completion inside a wrapper closure; it cost one extra 136-B
    copy on every ordinary Part assignment, and S7's copy gate rejected it.
    Its final shape (`part_assignment.rs`, `association_data.rs`): bind the
    producer `Result`, take the witness, `match` once, return errors
    directly, and call the cold helper only on `InputRequired`.
  - **The routes** (S7's table is the newest example):
    - a site reached only from the ordinary path uses the witnessed route;
    - a site reached both from the ordinary path and from its own frame's
      resume uses `DefinitionResumeRoute`
      (`definitions/application/mod.rs`, now `pub(in crate::runtime)`). Its
      ordinary instantiation is `()` and is unchanged; the resumed
      instantiation consumes the token with fresh state;
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
- **This family chains.** A guard that pauses during a ReplaceAll traversal
  leaves three frames: F0243 (the rule scan, completing `Option<Expr>`)
  above F0242 (the traversal, completing `ReplaceAllTraversalOutcome`) above
  F0240 or F0241 (completing `Expr`). Count and Position chain the same way:
  F0559 → F0558 and F0564 → F0563. Several sites are reached from more than
  one frame's resume. For example, `apply_rules_once_from_list` (M0244) is
  reached on the ordinary path, from F0243's own resume, and from F0242's
  resume through `drive_replace_all_traversal`. **Classify every call path
  of every site under the route rules, and report the table, as S6 and S7
  did.** A site that fits no rule is a STOP with its call paths.
- **Rows, as input; never edit them.** All of these are in `.temp/brief-606/`:
  - `all-frame-migrations.tsv`: F0240-F0243, F0551-F0564;
  - `macro-dataflow.tsv` and `macro-frame-ownership.tsv`: M0240-M0244,
    M0586-M0600;
  - `transformation-dispositions.tsv`: T0018, T0019, T0089, T0090;
  - `microbench-step-labels.tsv`: the 11 S8 labels;
  - `draft-S8.md`: Codex's plan for this step.

  S8 has no chained-insert rows, insertion-route rows, deferred
  obligations, State births, promoted divergences or witness gates. The
  draft is raw material, and where it and this brief differ, this brief
  governs. Its Scope, scratch path, file list and measurement section are
  superseded.
- **Slot types.** `Expr` is `Value`. `Option<Expr>`, `CompiledMatchOutcome`,
  `ReplaceAllTraversalOutcome`, `TraversalWithoutLimitResolution`,
  `CasesTraversalResolution`, `TraversalCountOutcome` and
  `PositionCollectionOutcome` are registered Legacy
  (`suspension/typed/registry.rs`, `legacy.rs`). Each keeps its single global
  mode until its F step. Register no new slot type.

### The director's re-probe, at `a1359ba8`

48 shapes were run through the resumed path against the queued-input path
(`.temp/director/s8-probe/`, `pairs.py` and `run1.txt`). **All 48 are SAME,
and every one paused**, between 1 and 8 times.

- **ReplaceAll and ReplaceRepeated:**
  - a guard pausing at one node, at several nodes, and on an atom target;
  - a rule list whose first guard pauses and declines, so the scan
    continues to the next rule;
  - nested targets, rule lists with a head rule;
  - a `Sequence` splice;
  - a `Hold` target;
  - `//.` with a repeatedly pausing guard.
- **Replace and ReplaceAt:** at level 0, at a level spec, a two-rule list
  where both guards pause, and `ReplaceAt` at one position.
- **Count, MemberQ and FreeQ:** a pause in the target, in the pattern, in
  the level spec, in a traversal guard, and in the `Heads` option value.
- **Position:** a pause in the target, the pattern, the level spec, the
  limit, a traversal guard (flat and nested), and the `Heads` option value.
- **Loops:** `Do` and `Table` over each.
- **Internal heads:**
  - guest-written `System`InternalContinuationNone[]`, `…Some[5]` and
    `…ReplaceAllTraversalOutcome[x, True]` as a rule's right-hand side
    across the pause;
  - the same heads as traversed values and as the pattern.

Like S6 and S7, the typing must keep all 48 equal. This step has no defect
to repair.

### The owned rows, at `a1359ba8`

Frames (`replace_all/` is under `crates/rum-runtime/src/runtime/definitions/application/`;
`count_like/` and `position/` are under `crates/rum-runtime/src/runtime/tree_walkers/`):

| Row | Type | File:line (impl) |
| --- | --- | --- |
| F0240 | `ApplyReplaceAllSuspensionFrame` | `replace_all/resume_frames.rs:7` |
| F0241 | `ApplyReplaceAllWithPlanSuspensionFrame` | same `:28` |
| F0242 | `ApplyReplaceAllOwnedSuspensionFrame` | same `:53` |
| F0243 | `ApplyRulesOnceFromListSuspensionFrame` | same `:83` |
| F0551 | `EvalCountSuspensionFrame` | `count_like/resume_frames.rs:9` |
| F0552 | `EvalMemberQSuspensionFrame` | same `:30` |
| F0553 | `EvalFreeQSuspensionFrame` | same `:51` |
| F0554 | `ResumeCountAfterTargetSuspensionFrame` | same `:73` |
| F0555 | `ResumeMemberQAfterTargetSuspensionFrame` | same `:97` |
| F0556 | `ResumeFreeQAfterTargetSuspensionFrame` | same `:121` |
| F0557 | `FinishCountLikeAfterPatternSuspensionFrame` | same `:148` |
| F0558 | `FinishCountLikeAfterTraversalSuspensionFrame` | same `:184` |
| F0559 | `ContinueCountTraversalSuspensionFrame` | same `:211` |
| F0560 | `EvalPositionSuspensionFrame` | `position/resume_frames.rs:9` |
| F0561 | `ResumePositionAfterTargetSuspensionFrame` | same `:31` |
| F0562 | `ResumePositionAfterPatternSuspensionFrame` | same `:56` |
| F0563 | `FinishPositionAfterResolvedTraversalSuspensionFrame` | same `:79` |
| F0564 | `ContinuePositionTraversalSuspensionFrame` | same `:106` |

Macro sites (`replace_all.rs` is `definitions/application/replace_all.rs`;
the others are under `tree_walkers/`):

| Site | File:line | What pauses |
| --- | --- | --- |
| M0240 | `replace_all.rs:88` | `ReplaceAll` traversal, result form |
| M0241 | same `:103` | `ReplaceRepeated` traversal, result form |
| M0242 | same `:161` | rule scan on an atom target, insert form |
| M0243 | same `:193` | rule scan at a traversal node, insert form |
| M0244 | same `:394` | a rule's match (its guard), insert form |
| M0586-M0588 | `count_like.rs:45`, `:71`, `:94` | `Count`/`MemberQ`/`FreeQ` target |
| M0589-M0591 | same `:113`, `:152`, `:191` | their pattern |
| M0592 | same `:246` | level spec, insert form |
| M0593 | same `:274` | the count traversal, result form |
| M0594 | same `:323` | a traversal node's match, insert form |
| M0595 | `position/options.rs:49` | `Position` level spec or limit, insert form |
| M0596, M0597 | `position.rs:108`, `:127` | `Position` target, pattern |
| M0598 | same `:178` | the position traversal, result form |
| M0599, M0600 | same `:227`, `:296` | a traversal node's match, insert form |

Transport transforms, each **delete only the transport encoding**; the frame
completes its typed helper's exact result:

| Row | Site | Today |
| --- | --- | --- |
| T0018 | `replace_all/resume_frames.rs:65` (F0242) | `.map(into_continuation_expr)` on `ReplaceAllTraversalOutcome` |
| T0019 | same `:111` (F0243) | `.map(into_continuation_expr)` on `Option<Expr>` |
| T0089 | `count_like/resume_frames.rs:223` (F0559) | `.map(into_continuation_expr)` on `TraversalCountOutcome` |
| T0090 | `position/resume_frames.rs:117` (F0564) | `.map(into_continuation_expr)` on `PositionCollectionOutcome` |

Derive each `Input` from the actual producer `Result` before any
transformation, and bind each `Output` at the innermost function or closure
return. The identity decodes (`from_continuation_expr` on the frame's own
input) and identity re-encodes (`Expr::into_continuation_expr`) disappear.
A semantic map, such as `finish.finish(count)` or wrapping positions in
`List`, runs identically on both paths.

## Scope

A listed file or class is permission to do this step's owned work there.
It is not permission to:
- convert another step's rows;
- change evaluation beyond the owned rows;
- tidy.

Disclose every file you edit.

**Classes.**

- **The family's source files:**
  - under `crates/rum-runtime/src/runtime/definitions/application/`:
    `replace_all.rs`, `replace_all/resume_frames.rs` and
    `replace_all/plan.rs`;
  - under `crates/rum-runtime/src/runtime/tree_walkers/`:
    `count_like.rs`, `count_like/resume_frames.rs`, `count_like/options.rs`,
    `position.rs`, `position/resume_frames.rs` and `position/options.rs`.
- **Traversal support**, `tree_walkers/{level.rs, level/options.rs,
  post_order.rs}`, only for a visibility or trait edit that a typed frame's
  state needs. Disclose it.
- **Callers of a changed signature.** Edit only the construct that changed,
  in any file where an owned frame, helper or signature is named, including
  `kernel_api.rs` re-exports and the `rewrite_data/replace/` and
  `structure_data/replace/` callers.
- **Module roots on an owned type's path.** Make only the visibility or
  re-export edit the typed API needs, and disclose it.
- **`crates/rum-runtime/src/runtime/suspension/`**, only where S8's use
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
    new fixtures and the `typed_slots.rs` receipt, and
    `suspension_oracle/inventory/typed-slot-stages.tsv` and
    `typed-slot-shapes.tsv`.
  - **Compile controls:** `crates/rum-stdlib/tests/typed_slot_compile_controls.rs`,
    only for a pin that follows a derived count. No new pair is required.
- **Derived facts follow the diff** (brief-613's class rule). For every
  derived inventory, frontier, census receipt, fingerprint, certificate or
  test pin that names a touched construct, each row falls into exactly one
  case:

  | Its operation in this diff | Ruling |
  | --- | --- |
  | Deleted by an owned conversion (including the four transport encodings) | Retired with its source. Disclose it. |
  | Added by this diff | A new row. It stays rejected or unknown; claim no mark or coverage. Disclose it. |
  | Moved, or its text follows an owned rename | Paired. Disclose it old → new. |

  The class covers:
  - `runtime/tests/frame_root_trace{.rs,/derived_inventory.rs}`;
  - ownership inventories, by discovery replay;
  - retention inventories, by replay;
  - `retention_closure/census/tests.rs`, with a per-brief comment;
  - census receipts;
  - count and layout pins.

  A **STOP** is a change of *meaning* where the source did not change:
  - a row whose operation's source text is unchanged disappears, or changes
    kind, reason, status, disposition or exclusion;
  - a mark, coverage or promotion is gained or lost;
  - a new `UnknownHostPayload`;
  - a production receipt change outside S8's owned rows.

  Never hand-write a fingerprint, certificate cell or ledger chain.
- **Canonical tool outputs.** Any unchanged canonical gate tool this brief
  requires may write to its own fixed output paths under `target/` or
  `.temp/`. You may not modify, redirect or wrap it to move its output.
  Disclose each such path. Your own scratch stays in `.temp/brief-616/`.

`Cargo.toml` files, `Cargo.lock`, the toolchain, docs, `AGENTS.md`,
`TODO.md` and the core `Expr`/`Symbol` representation stay unchanged.

**Disclosure duty.** If a needed change falls outside every list and class,
make no edit there. STOP and report the file, the line and why.

## Steps

1. **Harness, probe and bases first, before any source edit.**
   - **Supplied.** `.temp/brief-616/` holds:
     - `base/rum-cli` and `base/rum-cli-alloc`: the S7 one-codegen-unit
       binaries, with their build commands. The director proved them fresh.
     - `stack-base/probe.rs`: the six-shape stack probe.
     - `base-source/`: the `a1359ba8` export.
     - `base-hashes.txt`.

     Verify the hashes first.
   - **Stack probe and bases.** Extend `probe.rs` with four shapes in which
     the recursion runs inside a guard of this family, so that its ordinary
     path functions stay resident on the native stack:
     - `replace-guard`: `r[0] := <leaf>`, `r[n_] := Replace[n, x_ /;
       r[x - 1] >= 0 :> x]`;
     - `replaceall-guard`: `a[0] := <leaf>`, `a[n_] := w[n] /. w[x_] /;
       a[x - 1] >= 0 :> x`;
     - `count-guard`: `c[0] := <leaf>`, `c[n_] := Count[{n}, x_ /;
       c[x - 1] >= 0] + n - 1`;
     - `position-guard`: `p[0] := <leaf>`, `p[n_] := Position[{n}, x_ /;
       p[x - 1] >= 0][[1, 1]] + n - 1`.

     Each must return its depth. `<leaf>` is the marker when measuring and
     `0` otherwise, as for the existing shapes. If the runtime-only test
     session lacks a builtin a shape needs, register a scratch handler, as
     `rhs-guard` does, and disclose it. Then build the debug and release
     `runtime-tests` bases from the unedited tree with
     `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, each in its own target
     directory under `.temp/brief-616/`. **The six existing shapes must
     reproduce S7's bytes per level exactly. If they do not, STOP.**

     | Profile | peel | plus | sub | up | guarded | rhs-guard |
     | --- | --- | --- | --- | --- | --- | --- |
     | debug | 19 024 | 34 912 | 29 376 | 28 064 | 35 840 | 49 152 |
     | release | 7 344 | 13 792 | 13 168 | 11 456 | 14 240 | 17 920 |

     Record the base values for the four new shapes.
   - **Resume harness.** Copy `.temp/brief-615/resume-harness/` to
     `.temp/brief-616/resume-harness/`. Add workloads that pause in S8's
     frames:
     - `ReplaceAll` with a pausing guard at one node, on an atom target,
       and at two nodes (F0240, F0242, F0243);
     - `ReplaceRepeated` with a pausing guard (F0241);
     - `Replace` with a two-rule list whose first guard pauses and declines,
       and one where both guards pause (F0243's cursor and its own
       re-insertion);
     - `ReplaceAt` with a pausing guard;
     - `Count`, `MemberQ` and `FreeQ` pausing in the target (F0551-F0553)
       and in the pattern (F0554-F0556);
     - `Count` pausing in the level spec (F0557) and in a guard at two
       traversal nodes (F0559, F0558);
     - `Position` pausing in the target, the pattern, the level spec, and a
       guard at two traversal nodes (F0560-F0564).

     Each repeats its pauses N times and checks its final value. Then build
     `resume` and `resume-alloc` from the unedited tree, with
     `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, each in its own target
     directory under `.temp/brief-616/`. Record their source hashes; they
     must match `base-source/`. Each workload's callgrind frame-call census
     must show the intended S8 frames resuming the intended number of times
     per pause.
2. **Labels.** Copy `.temp/brief-615/perf/labels.json` and its programs into
   `.temp/brief-616/perf/`. Add these ordinary labels, in the same N/2N
   format, each checking its own result:
   - `table-count-null`: `Table[Null, {5000}]`;
   - `replaceall-wide-no-match`: `gMicroWideTraversal /. z -> 0`, with
     `gMicroWideTraversal = Range[1600]`;
   - `replaceall-leaf-rebuild-small`: `gMicroReplaceLeafRebuildSmall /. 1
     -> 2`, with `gMicroReplaceLeafRebuildSmall = Nest[List, 1, 8]`;
   - `replaceall-guarded-hit-small`: `{1, 2, 3} /. x_Integer /; x > 1 :> x
     + 1`;
   - `replace-rule-list-decline`: `Replace[3, {x_ /; x < 0 :> 0, x_ :>
     9}]`;
   - `replacerepeated-guarded-small`: `ReplaceRepeated[f[8], f[x_ /; x >
     0] :> f[x - 1]]`;
   - `replaceat-small`: `ReplaceAt[{1, 2, 3}, x_ :> x + 1, 2]`;
   - `memberq-wide-no-match` and `freeq-wide-no-match`:
     `MemberQ[gMicroWideTraversal, _String]`,
     `FreeQ[gMicroWideTraversal, _String]`;
   - `count-wide-match`: `Count[gMicroWideTraversal, _Integer, {1}]`;
   - `count-guarded-small`: `Count[{1, 2, 3}, x_ /; x > 1]`;
   - `position-small-match`: `Position[{1, 2, 1, 2}, 1]`.

   Add the new resume workloads as resume labels, each with its
   `resumes_per_iteration`.
3. **Fixtures.** Before converting anything, add S8's oracle fixtures and
   run them on the unedited tree. Each compares the queued-input path with
   the resumed path under `Checks::ALL`, with unconditional `FaultProbe`
   absence.
   - Cover the director's 48 shapes and the harness shapes.
   - Each passes on the unedited tree. Record that.
4. **Type the owned rows and delete the four transport encodings**,
   following the rulings above.
   - Keep exactly:
     - the rule scan's cursor and its `would_reject` prefilter accounting;
     - the traversal order, the `ReplaceAllUnheld` hold handling, the
       `Sequence` splice into the parent and the unchanged-subtree sharing
       (`Expr::call_shared` on the original arguments);
     - the count/position limits, level matching and `Heads` handling;
     - held and evaluation order;
     - owner drop order;
     - profiling balance, including the rewrite prefilter and match-outcome
       records.
   - Each M site's ordinary path matches the producer `Result` once and
     constructs no frame, `Arc` or `Vec`. Frame state is built in the cold
     path. Move locals, not control flow.
5. **Ordinary controls.** Rerun the 28 round-4 ordinary controls through
   brief-611's harness:
   - copy `.temp/brief-615/ordinary-controls/` to `.temp/brief-616/`;
   - re-sync its copied `suspension_oracle/` support from the committed tree
     first;
   - all 28 must pass.
6. **Census.** Advance the receipt to S8 in `typed_slot_census` and in
   `suspension_oracle/fixtures/typed_slots.rs`, and record every owned row's
   transition.
   - Production at S7 is 606 implementations and 566 macro sites, with 52
     typed.
   - With the eighteen frames typed, expect 606 implementations with 70
     typed. Report the actual totals, including the macro sites.
7. **Inventories.** Replay fingerprints, both certificate sets and ownership
   discovery, and disclose every changed row under the derived-facts class.

## Verification (MANDATORY — run to green or evidence)

**How to run.**
- Run everything serially, each command through `python3
  scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
  RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
  TMPDIR=<repo>/.temp/brief-616/tmp …`.
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
- Rerun the director's 48-shape probe (`.temp/director/s8-probe/pairs.py`,
  through a copy of `.temp/director/depth-leak-probe/` built under
  `.temp/brief-616/`). Also rerun S7's 32 shapes and S6's 22
  (`.temp/director/{s7,s6}-probe/pairs.py`) as a regression check.
- **Native ZeroMQ.** In your sandbox the nine native ZeroMQ tests cannot
  bind loopback. Run the workspace with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`,
  report the nine as skips, and report every other count. The director runs
  them natively.

Report:
- every `test result:` total;
- the oracle's fault-occurrence delta for each build;
- any compiler warning the diff introduces. Neither oracle build may warn.

**Performance.** `TEMPLATE.md` pitfall 8 governs, with the bars of
brief-611, 613, 614 and 615. Build the candidates with the base commands and
the same cwd, into separate target directories. Preflight every N/2N run:
the result, and the exact resume count (`resumes_per_iteration` × N).

1. **Allocations and bytes first**, on every label.
   - Ordinary labels must be exactly equal, in slope and in the
     whole-program counts at N and 2N.
   - On the resume workloads, a typed entry adds one 24-B adapter `Arc` per
     newly typed transfer. Any other difference is attributed at its
     allocation sites, or it is a defect.
2. **Instruction slopes.** Run two interleaved `perf stat` pairs per arm on
   all labels: `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`,
   stdin `/dev/null`.
   - A flagged ordinary label is cleared by five runs per arm, or by six or
     more callgrind processes per arm with every deterministic delta named
     at its addresses.
   - A deterministic register-allocation residual is reported with its
     addresses for the director to rule on, as S6's +2 and S7's +5 were.
3. **Copies and leaves.** The `memcpy` count on every ordinary label is at
   most the base's. `literal-integer`, `do-count-null` and
   `do-iterator-null` show no deterministic instruction growth.
   `apply_rules_once_from_list`, `drive_replace_all_traversal`,
   `drive_count_pattern_matches` and the position traversal are hot on the
   ordinary path; report each one's own instructions per call, base and
   candidate.
4. **Stack.** Measure all ten shapes, debug and release, at four heights
   with three samples each, against your calibrated bases.
   - Tabulate the prologue frame size of every function S8 changes, in both
     profiles.
   - **Bar:** no growth beyond the typed macro's named closure environment,
     attributed per slot.
   - **Any growth on any of the ten shapes is reported with its slot table
     for the director to rule on.**
5. **Resume attribution.** For every resume workload, run six callgrind
   processes per arm and diff every function per process. Name each
   deterministic delta at its addresses. The driver's 136-B typed-outcome
   move is owed to R: count it, do not repair it.
6. **Timing** comes last, as an indication only. Run four full
   `scripts/core-language-microbench.sh` panels, base, candidate, base,
   candidate, under `taskset -c 2`, with profiling off.
   - A residual whose instructions are equal is a layout question. Settle
     it causally with both arms rebuilt using `RUSTFLAGS="-C
     llvm-args=-align-all-functions=6"`.
   - **Probe the panel's own harness.** Cut the label's line from the
     generated benchmark program, keep its prelude, and raise
     `rumMicroReps`; the panel reports a minimum over warm repetitions, so
     a single cold call is not the same quantity (S7's single-call probes
     ran at twice the panel's cost and never cleared the magnitude check).
     Check the probe reproduces the label's per-iteration magnitude before
     reading its delta.
   - Keep every probe's total iteration count below the 100 000 iteration
     work limit, and check its output for completion. A probe that exceeds
     it bails out and measures startup; the tell is a zero slope.

A regression is a defect to fix or report. A stack or performance residual
is not a failing gate; it goes to the report with its attribution, and I
rule on it.

## rum / Rust pitfalls (read twice)

`TEMPLATE.md` applies. Rum against rum only.

- No default-path cost. No owned-tree clones on hot paths. The rule scan and
  the traversals are among the hottest code in rum: every `Replace`,
  `ReplaceAll`, `Cases`-family call and every rewrite goes through
  `apply_rules_once_from_list`. An unchanged subtree must stay shared, not
  rebuilt.
- **The witness comes from the actual `Result`.** Never guess the input from
  a synthetic `Err` type.
- **The token** carries no `Arc`, is not `Clone`, and accepts fresh state of
  the same frame type only.
- Actions, codecs and modes stay sealed and opaque. Nothing public names a
  mode or constructs an erased entry.
- Profiling records stay balanced on every path.
- Prefer the caller-place macro arm.
- **Module size.** New modules stay under 10k characters. Check
  `.temp/brief-606/file-size-assessments.tsv` first. `replace_all.rs` is
  16k and `count_like.rs` 11k. Split only by responsibility.

## Checkpointing — protect the scene

- Everything goes under `.temp/brief-616/`.
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
  adapter. Type no row outside S8's. Never weaken an oracle `Checks` flag, a
  retention reject, a fault-absence gate or a documented test to get a pass.
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
   - each F/M/T row's transition;
   - the call-path and route table;
   - the census output.
3. The harness workloads, stack shapes and oracle fixtures: what each
   exercises, its frame census, and its result on the base tree and the
   candidate in both builds. Include the director's 48 shapes.
4. All gates, verbatim. Fault-occurrence deltas. Inventory and certificate
   transitions. Every canonical tool output path written.
5. Allocation, instruction, copy, stack and resume-attribution evidence, and
   timing with any alignment check.
6. Stop-loss counters, deviations and open questions.
