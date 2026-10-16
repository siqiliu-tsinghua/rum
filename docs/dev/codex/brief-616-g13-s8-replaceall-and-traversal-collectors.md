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
     - `position-guard`: `p[0] := <leaf>`, `p[n_] := Position[{n},
       x_Integer /; p[x - 1] >= 0][[1, 1]] + n - 1` (amendment 1).

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

## Codex checkpoint report — brief-616 (blocked before conversion)

Step 1 cannot be completed as written. The prescribed `position-guard` hits `StackLimit` at depth 2 in both debug and release without reaching its leaf marker. The other nine shapes pass, including exact reproduction of all twelve S7 slopes. Clarification is pending on changing the Position pattern from `x_` to `x_Integer`; that correction has **not** been applied or tested. No production source has been edited.

### 1. Files changed and scope

This appended report is the only edited repository document. All preparation and evidence are under `.temp/brief-616/`. All 1,497 production source/build inputs still match the supplied export byte for byte. No git command or network operation was run. No workspace manifest, lockfile, toolchain, public signature, module root, receipt, inventory or pin changed.

- `stack-base/probe.rs`: added the four exact requested formulas and reused `brief614_stack_greater_equal` for their guard comparison. Count, Position and Part already exist in runtime test registrations; Replace and ReplaceAll come from the existing test session.
- `stack/base/tree/`: copied the supplied export; only its copied `crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs` has the extended probe appended. The production file is unchanged.
- `resume-harness/`: copied S7 harness; `src/main.rs` adds S8 dispatch and new `src/s8.rs` holds 19 value/prompt-checking workloads. The copied manifest, lockfile and S5–S7 modules keep the predecessor setup.
- `perf/labels.json`, `perf/*.rum`: copied 175 S7 labels/programs, added 12 ordinary N/2N labels and 19 resume labels. Total 206 = 138 ordinary + 68 resume. `table-count-null` uses N=8 to stay below 100,000 iterations at 2N including checks.
- `build.py`, `measure-base-stack.py`, `base-resume-census.py`, `run-perf.py`: bounded scratch build/calibration/census/preflight runners. The census derives physical symbol aliases from `nm`.
- `hash-verification.json`, `base-source-hashes.json`, `s8-resume-workloads.json`, `s8-ordinary-labels.json`, `builds/`, `stack/base/{debug,release}/`, `base-resume-census/`, `perf/check/base/s8/`, `compiler-warnings.json`, `evidence-summary.json`, `test-result-totals.txt`, `scratch-artifacts.json`, `REPORT.md`, `PROGRESS.md`, `state.json`: retained evidence/checkpoints. The artifact manifest lists individual scratch outputs outside copied source and Cargo target trees.

Build target paths written: `.temp/brief-616/target-base-stack-debug/`, `target-base-stack-release/`, `target-base-resume/`, `target-base-resume-alloc/`. No canonical gate output outside `.temp/brief-616/` was written.

### 2. Owned-row reconciliation

Unchanged frames: F0240, F0241, F0242, F0243, F0551, F0552, F0553, F0554, F0555, F0556, F0557, F0558, F0559, F0560, F0561, F0562, F0563, F0564. Unchanged sites: M0240, M0241, M0242, M0243, M0244, M0586, M0587, M0588, M0589, M0590, M0591, M0592, M0593, M0594, M0595, M0596, M0597, M0598, M0599, M0600. Unchanged encodings: T0018, T0019, T0089, T0090. No row was typed or retired. The call-path/route table and production census have not been produced; there are no derived-fact transitions. The brief’s S7 census totals were not independently rerun.

### 3. Harnesses, stack shapes and fixtures

All 19 resume workloads passed exact value/prompt/pause assertions at N=100 and 200 on both baseline binaries (76 runs). All 38 Callgrind processes passed their physical-frame census. Counts below are per iteration and multiply by N.

| Workload | Pauses | Owned frame calls | Final item |
| --- | ---: | --- | --- |
| `s8-replaceall-node` | 1 | F0240=1, F0242=1, F0243=1 | `f[2]` |
| `s8-replaceall-atom` | 1 | F0240=1, F0242=1, F0243=1 | `2` |
| `s8-replaceall-twice` | 2 | F0240=1, F0242=2, F0243=2 | `List[2, 3]` |
| `s8-repeated-guard` | 1 | F0241=1, F0242=1, F0243=1 | `f[0]` |
| `s8-replace-decline` | 1 | F0243=1 | `9` |
| `s8-replace-both` | 2 | F0243=2 | `9` |
| `s8-replaceat-guard` | 1 | F0243=1 | `List[1, 3, 3]` |
| `s8-count-target` | 1 | F0551=1 | `1` |
| `s8-count-pattern` | 1 | F0554=1 | `1` |
| `s8-memberq-target` | 1 | F0552=1 | `True` |
| `s8-memberq-pattern` | 1 | F0555=1 | `True` |
| `s8-freeq-target` | 1 | F0553=1 | `False` |
| `s8-freeq-pattern` | 1 | F0556=1 | `False` |
| `s8-count-level` | 1 | F0557=1 | `1` |
| `s8-count-guard-twice` | 2 | F0558=1, F0559=2 | `2` |
| `s8-position-target` | 1 | F0560=1 | `List[List[1]]` |
| `s8-position-pattern` | 1 | F0561=1 | `List[List[1]]` |
| `s8-position-level` | 1 | F0562=1 | `List[List[1]]` |
| `s8-position-guard-twice` | 2 | F0563=1, F0564=2 | `List[List[1], List[2]]` |

F0240 and F0241 share emitted address `0x5450e0`, proved by `nm`. Callgrind names the body F0240 even for ReplaceRepeated, so that physical group is checked as F0240+F0241; no separate observed F0241 count is claimed. Every other owned symbol is distinct. See `base-resume-census/{frame-symbol-groups,frame-census}.json`.

Stack bytes per recursive level at heights 2/4/6/8, three samples each; all completed fits have zero residual:

| Shape | Debug | Release | Status |
| --- | ---: | ---: | --- |
| peel | 19024 | 7344 | Exact S7 match |
| plus | 34912 | 13792 | Exact S7 match |
| sub | 29376 | 13168 | Exact S7 match |
| up | 28064 | 11456 | Exact S7 match |
| guarded | 35840 | 14240 | Exact S7 match |
| rhs-guard | 49152 | 17920 | Exact S7 match |
| replace-guard | 65056 | 26960 | Base established |
| replaceall-guard | 64080 | 25264 | Base established |
| count-guard | 74928 | 28560 | Base established |
| position-guard | unavailable | unavailable | StackLimit at first height, no marker |

The exact prescribed formula used:

```rum
p[0] := Brief611StackMarker[];
p[n_] := Position[{n}, x_ /; p[x - 1] >= 0][[1, 1]] + n - 1;
p[2]
```

Source diagnosis: `CasesTraversalConfig::default_position` (`runtime/model/traversal.rs:98`) includes heads and levels 0 through Infinity. `PostOrderPathTraversal::schedule_standard_call` (`runtime/tree_walkers/post_order.rs:285`) schedules the head for the next LIFO visit. Thus `x_` admits `List` and its guard recurses through `p[List - 1]`. This explanation follows from source; the measured facts are StackLimit and the missing marker. `x_Integer` is only a proposed correction.

All 12 new ordinary labels passed standalone checks plus N and 2N (36 runs). No oracle fixture was added while the required baseline stack prerequisite remains unresolved. The director’s 48 S8 / 32 S7 / 22 S6 pairs, both oracle feature builds, and 28 ordinary controls have not run. No candidate exists.

### 4. Gates and verbatim evidence

All build/measurement runners used `python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-616/tmp ...`. Builds set `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, use cwd `/home/remi/dev/rum` and separate targets, and retain command/environment/source hashes. Shell sessions retained commands without a shorter timeout. Builds and measurement workloads ran serially.

`base-resume`: exit 0, SHA-256 `a3839128f29faf3e89c1f29055156c232213642ff45d9b8e38ee65eacfde3407`.

```text
cargo build --offline --release --manifest-path /home/remi/dev/rum/.temp/brief-616/resume-harness/Cargo.toml --message-format=json
    Finished `release` profile [optimized] target(s) in 1m 38s
```

`base-resume-alloc`: exit 0, SHA-256 `49446fa5f638eb460c77786a6487efaa0aa854746ad172d439df0c7b8383dafa`.

```text
cargo build --offline --release --manifest-path /home/remi/dev/rum/.temp/brief-616/resume-harness/Cargo.toml --message-format=json --features alloc-count
    Finished `release` profile [optimized] target(s) in 1m 34s
```

`base-stack-debug`: exit 0, SHA-256 `36f7088022e740a10b71e33ef808b0da3b1e2935545496b124b86188d7907ca9`.

```text
cargo test --offline --manifest-path /home/remi/dev/rum/.temp/brief-616/stack/base/tree/Cargo.toml -p rum-runtime --lib --no-run --message-format=json
    Finished `test` profile [unoptimized + debuginfo] target(s) in 47.63s
```

`base-stack-release`: exit 0, SHA-256 `0ee208f9b60e04aa6c7f29c0dff65b5f8536072ec8241acfe69a11c25f1caf02`.

```text
cargo test --offline --manifest-path /home/remi/dev/rum/.temp/brief-616/stack/base/tree/Cargo.toml -p rum-runtime --lib --no-run --message-format=json --release
    Finished `release` profile [optimized] target(s) in 1m 26s
```

Both resume builds emitted zero warnings. Each stack build emitted the following three warnings, identical to `.temp/brief-615/builds/candidate-stack-{debug,release}/stdout.jsonl`; none is introduced by S8’s scratch extension:

```text
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

```

The original-six calibration runners exited 0. Each new-shape runner established three slopes, then exited 1 at Position’s result/marker check. Neither test process aborted: the supplied probe permits StackLimit as a reportable result, but the outer calibration correctly rejects it. Exact failures:

`debug`:

```text
running 1 test
test runtime::tests::holding_and_control_part2::brief611_stack_probe ... STACK_PROBE shape=position-guard depth=2 limit=1048576 result=StackLimit
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
```

`release`:

```text
running 1 test
test runtime::tests::holding_and_control_part2::brief611_stack_probe ... STACK_PROBE shape=position-guard depth=2 limit=1048576 result=StackLimit
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
```

Base resume-census runner: exit 0. Verbatim stdout for all 114 executions:

```text
resume N=100: PASS s8-replaceall-node resumes=100
resume-alloc N=100: PASS s8-replaceall-node resumes=100
callgrind N=100: PASS s8-replaceall-node resumes=100
resume N=200: PASS s8-replaceall-node resumes=200
resume-alloc N=200: PASS s8-replaceall-node resumes=200
callgrind N=200: PASS s8-replaceall-node resumes=200
resume N=100: PASS s8-replaceall-atom resumes=100
resume-alloc N=100: PASS s8-replaceall-atom resumes=100
callgrind N=100: PASS s8-replaceall-atom resumes=100
resume N=200: PASS s8-replaceall-atom resumes=200
resume-alloc N=200: PASS s8-replaceall-atom resumes=200
callgrind N=200: PASS s8-replaceall-atom resumes=200
resume N=100: PASS s8-replaceall-twice resumes=200
resume-alloc N=100: PASS s8-replaceall-twice resumes=200
callgrind N=100: PASS s8-replaceall-twice resumes=200
resume N=200: PASS s8-replaceall-twice resumes=400
resume-alloc N=200: PASS s8-replaceall-twice resumes=400
callgrind N=200: PASS s8-replaceall-twice resumes=400
resume N=100: PASS s8-repeated-guard resumes=100
resume-alloc N=100: PASS s8-repeated-guard resumes=100
callgrind N=100: PASS s8-repeated-guard resumes=100
resume N=200: PASS s8-repeated-guard resumes=200
resume-alloc N=200: PASS s8-repeated-guard resumes=200
callgrind N=200: PASS s8-repeated-guard resumes=200
resume N=100: PASS s8-replace-decline resumes=100
resume-alloc N=100: PASS s8-replace-decline resumes=100
callgrind N=100: PASS s8-replace-decline resumes=100
resume N=200: PASS s8-replace-decline resumes=200
resume-alloc N=200: PASS s8-replace-decline resumes=200
callgrind N=200: PASS s8-replace-decline resumes=200
resume N=100: PASS s8-replace-both resumes=200
resume-alloc N=100: PASS s8-replace-both resumes=200
callgrind N=100: PASS s8-replace-both resumes=200
resume N=200: PASS s8-replace-both resumes=400
resume-alloc N=200: PASS s8-replace-both resumes=400
callgrind N=200: PASS s8-replace-both resumes=400
resume N=100: PASS s8-replaceat-guard resumes=100
resume-alloc N=100: PASS s8-replaceat-guard resumes=100
callgrind N=100: PASS s8-replaceat-guard resumes=100
resume N=200: PASS s8-replaceat-guard resumes=200
resume-alloc N=200: PASS s8-replaceat-guard resumes=200
callgrind N=200: PASS s8-replaceat-guard resumes=200
resume N=100: PASS s8-count-target resumes=100
resume-alloc N=100: PASS s8-count-target resumes=100
callgrind N=100: PASS s8-count-target resumes=100
resume N=200: PASS s8-count-target resumes=200
resume-alloc N=200: PASS s8-count-target resumes=200
callgrind N=200: PASS s8-count-target resumes=200
resume N=100: PASS s8-count-pattern resumes=100
resume-alloc N=100: PASS s8-count-pattern resumes=100
callgrind N=100: PASS s8-count-pattern resumes=100
resume N=200: PASS s8-count-pattern resumes=200
resume-alloc N=200: PASS s8-count-pattern resumes=200
callgrind N=200: PASS s8-count-pattern resumes=200
resume N=100: PASS s8-memberq-target resumes=100
resume-alloc N=100: PASS s8-memberq-target resumes=100
callgrind N=100: PASS s8-memberq-target resumes=100
resume N=200: PASS s8-memberq-target resumes=200
resume-alloc N=200: PASS s8-memberq-target resumes=200
callgrind N=200: PASS s8-memberq-target resumes=200
resume N=100: PASS s8-memberq-pattern resumes=100
resume-alloc N=100: PASS s8-memberq-pattern resumes=100
callgrind N=100: PASS s8-memberq-pattern resumes=100
resume N=200: PASS s8-memberq-pattern resumes=200
resume-alloc N=200: PASS s8-memberq-pattern resumes=200
callgrind N=200: PASS s8-memberq-pattern resumes=200
resume N=100: PASS s8-freeq-target resumes=100
resume-alloc N=100: PASS s8-freeq-target resumes=100
callgrind N=100: PASS s8-freeq-target resumes=100
resume N=200: PASS s8-freeq-target resumes=200
resume-alloc N=200: PASS s8-freeq-target resumes=200
callgrind N=200: PASS s8-freeq-target resumes=200
resume N=100: PASS s8-freeq-pattern resumes=100
resume-alloc N=100: PASS s8-freeq-pattern resumes=100
callgrind N=100: PASS s8-freeq-pattern resumes=100
resume N=200: PASS s8-freeq-pattern resumes=200
resume-alloc N=200: PASS s8-freeq-pattern resumes=200
callgrind N=200: PASS s8-freeq-pattern resumes=200
resume N=100: PASS s8-count-level resumes=100
resume-alloc N=100: PASS s8-count-level resumes=100
callgrind N=100: PASS s8-count-level resumes=100
resume N=200: PASS s8-count-level resumes=200
resume-alloc N=200: PASS s8-count-level resumes=200
callgrind N=200: PASS s8-count-level resumes=200
resume N=100: PASS s8-count-guard-twice resumes=200
resume-alloc N=100: PASS s8-count-guard-twice resumes=200
callgrind N=100: PASS s8-count-guard-twice resumes=200
resume N=200: PASS s8-count-guard-twice resumes=400
resume-alloc N=200: PASS s8-count-guard-twice resumes=400
callgrind N=200: PASS s8-count-guard-twice resumes=400
resume N=100: PASS s8-position-target resumes=100
resume-alloc N=100: PASS s8-position-target resumes=100
callgrind N=100: PASS s8-position-target resumes=100
resume N=200: PASS s8-position-target resumes=200
resume-alloc N=200: PASS s8-position-target resumes=200
callgrind N=200: PASS s8-position-target resumes=200
resume N=100: PASS s8-position-pattern resumes=100
resume-alloc N=100: PASS s8-position-pattern resumes=100
callgrind N=100: PASS s8-position-pattern resumes=100
resume N=200: PASS s8-position-pattern resumes=200
resume-alloc N=200: PASS s8-position-pattern resumes=200
callgrind N=200: PASS s8-position-pattern resumes=200
resume N=100: PASS s8-position-level resumes=100
resume-alloc N=100: PASS s8-position-level resumes=100
callgrind N=100: PASS s8-position-level resumes=100
resume N=200: PASS s8-position-level resumes=200
resume-alloc N=200: PASS s8-position-level resumes=200
callgrind N=200: PASS s8-position-level resumes=200
resume N=100: PASS s8-position-guard-twice resumes=200
resume-alloc N=100: PASS s8-position-guard-twice resumes=200
callgrind N=100: PASS s8-position-guard-twice resumes=200
resume N=200: PASS s8-position-guard-twice resumes=400
resume-alloc N=200: PASS s8-position-guard-twice resumes=400
callgrind N=200: PASS s8-position-guard-twice resumes=400
```

New ordinary-preflight runner: exit 0. Verbatim stdout:

```text
table-count-null check: True
table-count-null 8: Null
table-count-null 16: Null
replaceall-wide-no-match check: True
replaceall-wide-no-match 400: Null
replaceall-wide-no-match 800: Null
replaceall-leaf-rebuild-small check: True
replaceall-leaf-rebuild-small 500: Null
replaceall-leaf-rebuild-small 1000: Null
replaceall-guarded-hit-small check: List[1, 3, 4]
replaceall-guarded-hit-small 500: Null
replaceall-guarded-hit-small 1000: Null
replace-rule-list-decline check: 9
replace-rule-list-decline 500: Null
replace-rule-list-decline 1000: Null
replacerepeated-guarded-small check: f[0]
replacerepeated-guarded-small 500: Null
replacerepeated-guarded-small 1000: Null
replaceat-small check: List[1, 3, 3]
replaceat-small 500: Null
replaceat-small 1000: Null
memberq-wide-no-match check: False
memberq-wide-no-match 400: Null
memberq-wide-no-match 800: Null
freeq-wide-no-match check: True
freeq-wide-no-match 400: Null
freeq-wide-no-match 800: Null
count-wide-match check: 1600
count-wide-match 400: Null
count-wide-match 800: Null
count-guarded-small check: 2
count-guarded-small 500: Null
count-guarded-small 1000: Null
position-small-match check: List[List[1], List[3]]
position-small-match 500: Null
position-small-match 1000: Null
```

Every stack `test result:` total, verbatim. The two Position test-process passes do not clear the outer calibration failure.

```text
stack/base/debug/count-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/count-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/count-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/count-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/count-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/count-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/count-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/count-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/count-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/count-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/count-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/count-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/guarded-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/guarded-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/peel-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.02s
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
stack/base/debug/position-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/replace-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.02s
stack/base/debug/replace-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replace-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replace-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replace-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/replace-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/replace-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replace-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replace-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replace-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/replace-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replace-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/replaceall-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replaceall-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replaceall-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replaceall-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/replaceall-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replaceall-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/replaceall-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/replaceall-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replaceall-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/replaceall-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/replaceall-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/replaceall-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/rhs-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/debug/rhs-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
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
stack/base/debug/up-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
stack/base/debug/up-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.01s
stack/base/release/count-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/count-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/count-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/count-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/count-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/count-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/count-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/count-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/count-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/count-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/count-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/count-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
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
stack/base/release/peel-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s
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
stack/base/release/position-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replace-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-2-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-2-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-2-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-4-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-4-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-4-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-6-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-6-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-6-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-8-0.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-8-1.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
stack/base/release/replaceall-guard-8-2.stdout: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.00s
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
```

Not run: formatting; all listed repository unit/integration/compile-control/census gates; either suspension oracle; ownership discovery; retention fingerprint/certificate replay; workspace tests; ordinary-controls harness; director probes. Oracle fault deltas are **unmeasured**, not zero. Native ZeroMQ did not run; no skip count is claimed. No certificate/inventory transition is claimed.

### 5. Performance evidence and limits

Only base stack calibration, result/pause preflights and one base Callgrind process per resume workload/size are complete. Allocation-enabled resume preflights retain counts in stderr; no A/B allocation slope or attribution is claimed. All-label allocation comparisons, perf pairs, ordinary memcpy and hot-function own-instruction gates, changed-function prologue/slot tables, six-process-per-arm resume attribution, four timing panels and alignment checks have not run. No timing conclusion is drawn from runner-duration metadata.

### 6. Stop-loss, deviations and next action

Counters: 4 successful builds; 216 valid stack measurements plus 2 invalid Position samples; 76 successful resume preflights; 38 successful frame-census processes; 36 successful ordinary preflights; 218 test-result lines; 0 production edits; 0 conversions. Stop-loss no-progress counter: 0. Both failed new-shape commands established three requested slopes, so this is a prerequisite/formula conflict, not exhaustion of stop-loss. No half-edited production file exists.

Independent resume-base and label work proceeded while clarification was pending. Conversion, fixture writes and dependent gates did not. Full predecessor-brief reading and conversion route analysis remain unfinished; those prerequisites are not claimed complete. One report-assembly validation initially rejected an incorrect zero-warning assumption; the actual warnings above were then checked against S7 logs and recorded.

Required next decision: authorize `x_Integer` for the Position stack probe or supply another depth-returning formula that keeps its guard path resident. If authorized, preserve these failures, rebuild corrected stack bases into separate targets, remeasure all ten shapes, finish predecessor reading, and then add/run base oracle fixtures before conversion. The pending clarification has not been treated as approval.

## Director amendment 1 (2026-10-17) — `position-guard` uses `x_Integer`

The checkpoint above is correct, and stopping was the right call. My
`position-guard` formula was wrong: `Position` defaults to levels 0 through
`Infinity` with heads included (`CasesTraversalConfig::default_position`),
so `x_` admits the `List` head and the guard recurses through `p[List - 1]`.
I wrote the shape without running it once.

**Ruling.** The shape is `p[n_] := Position[{n}, x_Integer /; p[x - 1] >=
0][[1, 1]] + n - 1`; Step 1 is updated. `x_Integer` excludes the head and the
list itself, so the guard runs once per level, on `n`, and the result is
still the depth. Measure it on the same base tree with the same calibration
rule. The other nine shapes, their base values, the 19 resume workloads and
the 12 ordinary labels stand as recorded above.

Then continue from Step 2's remaining work: fixtures, conversion, and the
full verification. Append your final report after this amendment.

## Codex report — brief-616 (redispatch 1, after amendment 1)

_Director note: this run was cut by the Codex usage limit at 05:37 on
2026-10-17, after every functional, allocation, instruction, stack and
resume-attribution gate and during the timing investigation, before the report
was appended. Sections 1-4 and the stack section below are the worker's own,
copied verbatim from `.temp/brief-616/report-functional.md` and
`report-stack.md`. Sections 5-6 are the director's gate record._

### 1. Files changed and scope

The 25 source/test/inventory files below are the complete source delta against the supplied S7 export. This appended report is the only document edit. The 1,497 baseline inputs were verified; all six candidate builds used the same frozen set of 1,502 source/build inputs and the repository cwd. No public signature, module-root visibility, slot registration/mode, `typed/witness.rs`, dependency, lockfile, toolchain, core representation or `unsafe` changed. No git or network operation was run.

| File | Chars before → after | Semantic change |
| --- | --- | --- |
| crates/rum-runtime/src/runtime/definitions/application/replace_all.rs | 16228 → 16819 | M0240/M0241 use caller-place typed macros. M0242 uses an actual-result witness and cold atom helper. M0243/M0244 use DefinitionResumeRoute with () on ordinary calls and the same frame token on resume. Cursor, prefilter, match accounting, hold/Sequence behavior and unchanged-subtree sharing remain intact. |
| crates/rum-runtime/src/runtime/definitions/application/replace_all/resume_frames.rs | 4823 → 5697 | F0240–F0243 implement TypedFrame; delete T0018/T0019 and owned identity decodes. Three named cold helpers build frame state. Same-frame retries consume fresh-state tokens. |
| crates/rum-runtime/src/runtime/tree_walkers/count_like.rs | 11092 → 11564 | M0586–M0593 use caller-place macros or actual-result witnesses. M0594 uses DefinitionResumeRoute for ordinary and own-frame resume paths. Count/MemberQ/FreeQ finishing and traversal limits remain unchanged. |
| crates/rum-runtime/src/runtime/tree_walkers/count_like/resume_frames.rs | 8492 → 9077 | F0551–F0559 implement TypedFrame; delete T0089 and owned identity decodes. Two named cold helpers construct frames; F0559 consumes its token when a later node pauses. |
| crates/rum-runtime/src/runtime/tree_walkers/position.rs | 10366 → 10440 | M0596–M0598 use caller-place macros. M0599 freezes positions only in its cold helper; M0600 uses the existing frame token. Traversal order, Heads, limits and path accumulation remain unchanged. |
| crates/rum-runtime/src/runtime/tree_walkers/position/options.rs | 2046 → 2438 | M0595 takes the actual traversal-result witness and passes it plus the request to a local named cold helper; SharedArgs cloning and frame construction stay cold. |
| crates/rum-runtime/src/runtime/tree_walkers/position/resume_frames.rs | 4721 → 5672 | F0560–F0564 implement TypedFrame; delete T0090 and owned identity decodes. Two named cold helpers freeze initial state or consume a fresh-state token. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv | 23520 → 23520 | Discovery replay: the PersistentPositionNode definition moves from line 19 to 20; its cycle meaning and coverage are unchanged. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv | 256578 → 256580 | Discovery replay: 22 coordinate moves, unchanged classifications. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv | 421601 → 417750 | Discovery replay: three coordinate moves and 25 closures retired with owned S8 conversions. |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv | 397571 → 396447 | Discovery replay: eight coordinate moves, 12 owned macro renames and eight insert macros retired for witnessed cold helpers. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv | 91646 → 91646 | Replay 68 canonical emitted context fingerprints; all 18 other columns unchanged. |
| crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv | 18162 → 18162 | Replay four emitted C01/C02 certificate cells for hash=true/false; the other seven columns and all marks remain unchanged. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs | 3539 → 3576 | Register the new S8 fixture module. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/replaceall_traversal_collectors/collectors.rs | 0 → 6652 | New fixtures cover the director’s 29 Count/MemberQ/FreeQ/Position shapes, including target/pattern/level/limit/Heads pauses, nested guards and loops. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/replaceall_traversal_collectors/harness.rs | 0 → 4338 | New oracle fixtures cover all 19 S8 resume-harness workloads. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/replaceall_traversal_collectors/mod.rs | 0 → 1084 | New S8 fixture runner requires Checks::ALL and unconditional fault absence; registers rewrite, collector and harness fixture modules. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/replaceall_traversal_collectors/rewrite.rs | 0 → 4598 | New fixtures cover the director’s 19 rewrite shapes, including guard re-entry, cursor decline, holds, Sequence and guest-written internal heads. |
| crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs | 181 → 181 | Earned stage assertion advances S7 → S8; no slot mode changes. |
| crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv | 682675 → 682791 | Derived receipt replay: 18 frames and 20 sites become typed; four transports and ten owned whole-slot identity decodes retire; S8 becomes implemented. |
| crates/rum-stdlib/tests/typed_slot_census.rs | 7923 → 7967 | Register the S8 census module; stage S7 → S8; forced-red guard-loss pin legacy frame count 514 → 496, finalizer count 56 and implementation count 622 unchanged. |
| crates/rum-stdlib/tests/typed_slot_census/receipts.rs | 13713 → 13764 | Derived S8 baseline counts: production legacy frames 503 → 485; implementations stay 606; physical macros 566 → 558. Include S8 transition/witness checks and retain fixed registry/preflip rules. |
| crates/rum-stdlib/tests/typed_slot_census/reporting.rs | 11561 → 11713 | Include the owned Count/Position witness sites in the emitted witnessed-site inventory; no witness-proof semantics change. |
| crates/rum-stdlib/tests/typed_slot_census/s4.rs | 6013 → 6027 | Exclude the now-earned S8 cohort from the historical S4 ownership comparison, as with prior later cohorts. |
| crates/rum-stdlib/tests/typed_slot_census/s8.rs | 0 → 5185 | New closed S8 reconciliation for exact frame types, 20 insertion identities, four fresh-state token routes and 14 owned transport/identity removals; preserves modes and preflip refusal. |

All new Rust modules are below 10k characters. The seven production files contain 52 changed source functions: 18 frame resumes and 34 methods/helpers, including eight new cold helpers. The internal route generics remain private to the owned family; no caller ripple was needed. A Count trait import is explicit. The Position level/limit cold helper stays in `position/options.rs` so the existing closed witness resolver can prove it; no proof semantics were changed. See [complete source patch](../../../.temp/brief-616/candidate-final.patch), [function bodies](../../../.temp/brief-616/changed-functions.json) and [build/source/binary hashes](../../../.temp/brief-616/candidate-build-verification.json).

### 2. Owned-row reconciliation and routes

| Row | Old owner/status | New owner/status | Provider or identity |
| --- | --- | --- | --- |
| F0240 | S8/untyped | S8/typed | ApplyReplaceAllSuspensionFrame |
| F0241 | S8/untyped | S8/typed | ApplyReplaceAllWithPlanSuspensionFrame |
| F0242 | S8/untyped | S8/typed | ApplyReplaceAllOwnedSuspensionFrame |
| F0243 | S8/untyped | S8/typed | ApplyRulesOnceFromListSuspensionFrame |
| F0551 | S8/untyped | S8/typed | EvalCountSuspensionFrame |
| F0552 | S8/untyped | S8/typed | EvalMemberQSuspensionFrame |
| F0553 | S8/untyped | S8/typed | EvalFreeQSuspensionFrame |
| F0554 | S8/untyped | S8/typed | ResumeCountAfterTargetSuspensionFrame |
| F0555 | S8/untyped | S8/typed | ResumeMemberQAfterTargetSuspensionFrame |
| F0556 | S8/untyped | S8/typed | ResumeFreeQAfterTargetSuspensionFrame |
| F0557 | S8/untyped | S8/typed | FinishCountLikeAfterPatternSuspensionFrame |
| F0558 | S8/untyped | S8/typed | FinishCountLikeAfterTraversalSuspensionFrame |
| F0559 | S8/untyped | S8/typed | ContinueCountTraversalSuspensionFrame |
| F0560 | S8/untyped | S8/typed | EvalPositionSuspensionFrame |
| F0561 | S8/untyped | S8/typed | ResumePositionAfterTargetSuspensionFrame |
| F0562 | S8/untyped | S8/typed | ResumePositionAfterPatternSuspensionFrame |
| F0563 | S8/untyped | S8/typed | FinishPositionAfterResolvedTraversalSuspensionFrame |
| F0564 | S8/untyped | S8/typed | ContinuePositionTraversalSuspensionFrame |
| M0240 | S8/untyped | S8/typed | result |
| M0241 | S8/untyped | S8/typed | result |
| M0242 | S8/untyped | S8/typed | witnessed |
| M0243 | S8/untyped | S8/typed | witnessed |
| M0244 | S8/untyped | S8/typed | witnessed |
| M0586 | S8/untyped | S8/typed | result |
| M0587 | S8/untyped | S8/typed | result |
| M0588 | S8/untyped | S8/typed | result |
| M0589 | S8/untyped | S8/typed | result |
| M0590 | S8/untyped | S8/typed | result |
| M0591 | S8/untyped | S8/typed | result |
| M0592 | S8/untyped | S8/typed | witnessed |
| M0593 | S8/untyped | S8/typed | result |
| M0594 | S8/untyped | S8/typed | witnessed |
| M0595 | S8/untyped | S8/typed | witnessed |
| M0596 | S8/untyped | S8/typed | result |
| M0597 | S8/untyped | S8/typed | result |
| M0598 | S8/untyped | S8/typed | result |
| M0599 | S8/untyped | S8/typed | witnessed |
| M0600 | S8/untyped | S8/typed | witnessed |
| T0018 | S8/untyped | S8/removed | map |
| T0019 | S8/untyped | S8/removed | map |
| T0089 | S8/untyped | S8/removed | map |
| T0090 | S8/untyped | S8/removed | map |
| Whole0012 | R/legacy-use | S8/removed | from_continuation_expr |
| Whole0013 | R/legacy-use | S8/removed | from_continuation_expr |
| Whole0014 | R/legacy-use | S8/removed | from_continuation_expr |
| Whole0015 | R/legacy-use | S8/removed | from_continuation_expr |
| Whole0180 | R/legacy-use | S8/removed | from_continuation_expr |
| Whole0181 | R/legacy-use | S8/removed | from_continuation_expr |
| Whole0182 | R/legacy-use | S8/removed | from_continuation_expr |
| Whole0183 | R/legacy-use | S8/removed | from_continuation_expr |
| Whole0184 | R/legacy-use | S8/removed | from_continuation_expr |
| Whole0185 | R/legacy-use | S8/removed | from_continuation_expr |

The ten Whole rows retire only owned identity decodes; their receipt owner changes from R to S8 because the deletion is earned here. T0018/T0019/T0089/T0090 delete only transport maps. Semantic finishing stays in the same return position. All other production receipts are unchanged. The exact old and new row cells are in [receipt-transitions.json](../../../.temp/brief-616/receipt-transitions.json).

| Site / frame | Actual producer; Input → Output | Call paths | Insertion route |
| --- | --- | --- | --- |
| M0240 / F0240 | apply_replace_all_owned; ReplaceAllTraversalOutcome → Expr | ReplaceAll / ReplaceAllUnheld completion, including their resumed target/rule stages; F0240 only extracts outcome.expr | caller-place witnessed macro |
| M0241 / F0241 | apply_replace_all_owned; ReplaceAllTraversalOutcome → Expr | ReplaceRepeated ordinary loop and its resumed continuation; F0241 only extracts outcome.expr | caller-place witnessed macro |
| M0242 / F0242 | apply_rewrite_rule_plan_once; Option < Expr > → ReplaceAllTraversalOutcome | apply_replace_all and apply_replace_all_with_plan; no own-frame return to atom branch | actual Result witness/request to cold atom helper |
| M0243 / F0242 | apply_rewrite_rule_plan_once; Option < Expr > → ReplaceAllTraversalOutcome | ordinary apply_replace_all_owned via (); F0242 -> resume_replace_all_traversal via token | DefinitionResumeRoute<F0242>, fresh frame on pause |
| M0244 / F0243 | match_compiled_owned; CompiledMatchOutcome → Option < Expr > | ordinary apply_rewrite_rule_plan_once via (); F0243 -> finish_suspended_rewrite_rule_match via token; F0242 -> drive_replace_all_traversal -> apply_rewrite_rule_plan_once via (); Replace and ReplaceAt callers likewise use () | DefinitionResumeRoute<F0243>; F0242 inserts different F0243 before resuspending F0242 with its token |
| M0586 / F0551 | evaluate; Expr → Expr | builtin evaluation entry; own frame proceeds after target, never re-enters this site | caller-place witnessed macro |
| M0587 / F0552 | evaluate; Expr → Expr | builtin evaluation entry; own frame proceeds after target, never re-enters this site | caller-place witnessed macro |
| M0588 / F0553 | evaluate; Expr → Expr | builtin evaluation entry; own frame proceeds after target, never re-enters this site | caller-place witnessed macro |
| M0589 / F0554 | resolve_matcher_pattern_expr_owned; Expr → Expr | ordinary eval_count; F0551 resume inserts this different frame then returns ResumeStop; own frame proceeds after pattern | caller-place witnessed macro |
| M0590 / F0555 | resolve_matcher_pattern_expr_owned; Expr → Expr | ordinary eval_member_q; F0552 resume inserts this different frame then returns ResumeStop; own frame proceeds after pattern | caller-place witnessed macro |
| M0591 / F0556 | resolve_matcher_pattern_expr_owned; Expr → Expr | ordinary eval_free_q; F0553 resume inserts this different frame then returns ResumeStop; own frame proceeds after pattern | caller-place witnessed macro |
| M0592 / F0557 | resolve_traversal_without_limit_owned; TraversalWithoutLimitResolution → Expr | ordinary Count/MemberQ/FreeQ after-pattern Heads completion; F0551-F0556 resumed tails; FrozenOptionReadContinuation after pausing Heads; own F0557 proceeds after traversal resolution | actual Result witness/request to cold helper; enclosing different-frame resume maps to ResumeStop |
| M0593 / F0558 | count_pattern_matches_owned; TraversalCountOutcome → Expr | ordinary and F0551-F0556 tails through option finish; F0557 resume after level resolution; F0558 only applies finish.finish | caller-place witnessed macro; enclosing different-frame resume maps to ResumeStop |
| M0594 / F0559 | match_expr_owned; CompiledMatchOutcome → TraversalCountOutcome | count_pattern_matches_owned -> driver via (); F0559 -> resume_count_traversal via token; outer resumed collectors enter count_pattern_matches_owned with () | DefinitionResumeRoute<F0559>, fresh frame on pause |
| M0595 / F0562 | resolve_position_traversal_owned; CasesTraversalResolution → Expr | ordinary Position after-pattern Heads completion; F0560/F0561 resumed tails; frozen Heads option completion; own F0562 proceeds after traversal resolution | actual Result witness/request to cold helper; enclosing different-frame resume maps to ResumeStop |
| M0596 / F0560 | evaluate; Expr → Expr | builtin evaluation entry; own frame proceeds after target | caller-place witnessed macro |
| M0597 / F0561 | resolve_matcher_pattern_expr_owned; Expr → Expr | ordinary eval_position; F0560 resume; own F0561 proceeds after pattern | caller-place witnessed macro; F0560 maps to ResumeStop |
| M0598 / F0563 | collect_positions_owned; PositionCollectionOutcome → Expr | ordinary and F0560/F0561 tails through option finish; F0562 resume after level/limit; own F0563 only wraps positions in List | caller-place witnessed macro; enclosing different-frame resume maps to ResumeStop |
| M0599 / F0564 | match_expr_owned; CompiledMatchOutcome → PositionCollectionOutcome | finish_position_after_resolved_traversal, including different-frame resumed tails; F0564 never returns here | actual Result witness/request to cold helper; Vec freezes only inside that helper |
| M0600 / F0564 | match_expr_owned; CompiledMatchOutcome → PositionCollectionOutcome | F0564 -> resume_position_traversal only | ContinueToken<F0564> consumes fresh state via cold helper |

Every producer Result is matched once. Ordinary routes build no frame, Arc or Vec. Each cold helper receives the actual producer witness and request plus frozen state. Own-frame resumes consume a token with fresh state; different-frame routes use witnesses and map stopping to ResumeStop. In particular F0242 reaches the F0243 scan through `()` before its own F0242 token handles a new node pause; Position M0600 is own-frame only. The witness and token are ZSTs.

Verbatim S8 census output (identical repetitions omitted):

```text
S8 negative control: preflip refused: S54 completed receipt is required first; no zero-legacy claim
S8 CENSUS production frames=485 finalizers=51 implementations=606 typed=70 macros=558 result=354 insert=204
S8 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S8 CENSUS test-support frames=7 finalizers=1 implementations=21 typed=13 macros=10 result=6 insert=4
S8 PARTITION implementations=622 macros=569
S8 TRANSITION T0018 removed; successor/provider=F0242
S8 TRANSITION T0019 removed; successor/provider=F0243
S8 TRANSITION T0089 removed; successor/provider=F0559
S8 TRANSITION T0090 removed; successor/provider=F0564
S8 TRANSITION Whole0012 removed; successor/provider=D049
S8 TRANSITION Whole0013 removed; successor/provider=D005
S8 TRANSITION Whole0014 removed; successor/provider=D005
S8 TRANSITION Whole0015 removed; successor/provider=D125
S8 TRANSITION Whole0180 removed; successor/provider=D006
S8 TRANSITION Whole0181 removed; successor/provider=D112
S8 TRANSITION Whole0182 removed; successor/provider=D049
S8 TRANSITION Whole0183 removed; successor/provider=D049
S8 TRANSITION Whole0184 removed; successor/provider=D134
S8 TRANSITION Whole0185 removed; successor/provider=D111
S8 FRAME F0240 ApplyReplaceAllSuspensionFrame ReplaceAllTraversalOutcome -> Expr
S8 FRAME F0241 ApplyReplaceAllWithPlanSuspensionFrame ReplaceAllTraversalOutcome -> Expr
S8 FRAME F0242 ApplyReplaceAllOwnedSuspensionFrame Option<Expr> -> ReplaceAllTraversalOutcome
S8 FRAME F0243 ApplyRulesOnceFromListSuspensionFrame CompiledMatchOutcome -> Option<Expr>
S8 FRAME F0551 EvalCountSuspensionFrame Expr -> Expr
S8 FRAME F0552 EvalMemberQSuspensionFrame Expr -> Expr
S8 FRAME F0553 EvalFreeQSuspensionFrame Expr -> Expr
S8 FRAME F0554 ResumeCountAfterTargetSuspensionFrame Expr -> Expr
S8 FRAME F0555 ResumeMemberQAfterTargetSuspensionFrame Expr -> Expr
S8 FRAME F0556 ResumeFreeQAfterTargetSuspensionFrame Expr -> Expr
S8 FRAME F0557 FinishCountLikeAfterPatternSuspensionFrame TraversalWithoutLimitResolution -> Expr
S8 FRAME F0558 FinishCountLikeAfterTraversalSuspensionFrame TraversalCountOutcome -> Expr
S8 FRAME F0559 ContinueCountTraversalSuspensionFrame CompiledMatchOutcome -> TraversalCountOutcome
S8 FRAME F0560 EvalPositionSuspensionFrame Expr -> Expr
S8 FRAME F0561 ResumePositionAfterTargetSuspensionFrame Expr -> Expr
S8 FRAME F0562 ResumePositionAfterPatternSuspensionFrame CasesTraversalResolution -> Expr
S8 FRAME F0563 FinishPositionAfterResolvedTraversalSuspensionFrame PositionCollectionOutcome -> Expr
S8 FRAME F0564 ContinuePositionTraversalSuspensionFrame CompiledMatchOutcome -> PositionCollectionOutcome
S8 INSERTION M0240 rum_runtime::runtime::definitions::application::replace_all::<Session as inherent>::apply_replace_all#macro1:resume_typed_or_suspend:result
S8 INSERTION M0241 rum_runtime::runtime::definitions::application::replace_all::<Session as inherent>::apply_replace_all_with_plan#macro1:resume_typed_or_suspend:result
S8 INSERTION M0242 rum_runtime::runtime::definitions::application::replace_all::<Session as inherent>::apply_replace_all_owned#call5:suspend_replace_all_atom:
S8 INSERTION M0243 rum_runtime::runtime::definitions::application::replace_all::<Session as inherent>::drive_replace_all_traversal#call2:suspend_replace_all_traversal:
S8 INSERTION M0244 rum_runtime::runtime::definitions::application::replace_all::<Session as inherent>::apply_rules_once_from_list#call6:suspend_rewrite_rule_match:
S8 INSERTION M0586 rum_runtime::runtime::tree_walkers::count_like::<Session as inherent>::eval_count#macro1:resume_typed_or_suspend:result
S8 INSERTION M0587 rum_runtime::runtime::tree_walkers::count_like::<Session as inherent>::eval_member_q#macro1:resume_typed_or_suspend:result
S8 INSERTION M0588 rum_runtime::runtime::tree_walkers::count_like::<Session as inherent>::eval_free_q#macro1:resume_typed_or_suspend:result
S8 INSERTION M0589 rum_runtime::runtime::tree_walkers::count_like::<Session as inherent>::resume_count_after_target#macro1:resume_typed_or_suspend:result
S8 INSERTION M0590 rum_runtime::runtime::tree_walkers::count_like::<Session as inherent>::resume_member_q_after_target#macro1:resume_typed_or_suspend:result
S8 INSERTION M0591 rum_runtime::runtime::tree_walkers::count_like::<Session as inherent>::resume_free_q_after_target#macro1:resume_typed_or_suspend:result
S8 INSERTION M0592 rum_runtime::runtime::tree_walkers::count_like::<Session as inherent>::finish_count_like_after_pattern#call2:suspend_count_like_pattern:
S8 INSERTION M0593 rum_runtime::runtime::tree_walkers::count_like::<Session as inherent>::finish_count_like_after_traversal#macro1:resume_typed_or_suspend:result
S8 INSERTION M0594 rum_runtime::runtime::tree_walkers::count_like::<Session as inherent>::drive_count_pattern_matches#call8:suspend_count_traversal:
S8 INSERTION M0595 rum_runtime::runtime::tree_walkers::position::options::<Session as inherent>::finish_position_option#call3:suspend_position_pattern:
S8 INSERTION M0596 rum_runtime::runtime::tree_walkers::position::<Session as inherent>::eval_position#macro1:resume_typed_or_suspend:result
S8 INSERTION M0597 rum_runtime::runtime::tree_walkers::position::<Session as inherent>::resume_position_after_target#macro1:resume_typed_or_suspend:result
S8 INSERTION M0598 rum_runtime::runtime::tree_walkers::position::<Session as inherent>::finish_position_after_resolved_traversal#macro1:resume_typed_or_suspend:result
S8 INSERTION M0599 rum_runtime::runtime::tree_walkers::position::<Session as inherent>::collect_positions_owned#call9:suspend_position_collection:
S8 INSERTION M0600 rum_runtime::runtime::tree_walkers::position::<Session as inherent>::drive_suspended_position_traversal#call7:suspend_position_traversal:
S8 TOTAL typed=70 typed_macros=47 witnessed_sites=30; four transport maps removed; modes unchanged; preflip remains refused
S8 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; typed evaluation uses unchanged global modes
S8 PARTITION implementations=621 macros=569
S8 CENSUS test-only frames=0 finalizers=0 implementations=0 typed=0 macros=0 result=0 insert=0
```

Pin changes are derived from this census: production legacy frames 503 → 485, typed implementations 52 → 70, physical macros 566 → 558, typed macros 35 → 47, witnessed sites 22 → 30. Production implementations remain 606. Forced guard-loss legacy-frame pin 514 → 496; finalizers 56 and implementations 622 are unchanged. Stage S7 → S8. Historical S4 reconciliation excludes the earned S8 cohort; preflip still requires S54 and refuses.

### 3. Harnesses, shapes and fixtures

The amendment-1 Position shape uses `x_Integer`; the earlier invalid `x_` attempt and STOP report above remain preserved. The scratch stack session reuses the disclosed GreaterEqual handler from S7. All other registrations and all six historical formulas are unchanged. Debug and release reproduce all twelve required S7 stack slopes exactly before conversion. Base and candidate stack probes are copied test trees; no production test file was changed.

The 19 added resume workloads use N=100/200 and check values, prompt order and exact pauses. All 114 initial base runs (native, alloc, Callgrind) passed. The final 206-label preflight passed 550 runs per arm: 138 ordinary labels at check/N/2N and 68 resume labels at N/2N.

| Added workload | Pauses / iteration | Intended owned frame calls / iteration | Final item |
| --- | --- | --- | --- |
| s8-replaceall-node | 1 | F0240=1, F0242=1, F0243=1 | see workload record |
| s8-replaceall-atom | 1 | F0240=1, F0242=1, F0243=1 | see workload record |
| s8-replaceall-twice | 2 | F0240=1, F0242=2, F0243=2 | see workload record |
| s8-repeated-guard | 1 | F0241=1, F0242=1, F0243=1 | see workload record |
| s8-replace-decline | 1 | F0243=1 | see workload record |
| s8-replace-both | 2 | F0243=2 | see workload record |
| s8-replaceat-guard | 1 | F0243=1 | see workload record |
| s8-count-target | 1 | F0551=1 | see workload record |
| s8-count-pattern | 1 | F0554=1 | see workload record |
| s8-memberq-target | 1 | F0552=1 | see workload record |
| s8-memberq-pattern | 1 | F0555=1 | see workload record |
| s8-freeq-target | 1 | F0553=1 | see workload record |
| s8-freeq-pattern | 1 | F0556=1 | see workload record |
| s8-count-level | 1 | F0557=1 | see workload record |
| s8-count-guard-twice | 2 | F0558=1, F0559=2 | see workload record |
| s8-position-target | 1 | F0560=1 | see workload record |
| s8-position-pattern | 1 | F0561=1 | see workload record |
| s8-position-level | 1 | F0562=1 | see workload record |
| s8-position-guard-twice | 2 | F0563=1, F0564=2 | see workload record |

The final measured frame census and all-ten-shape candidate comparison are reported in section 5. Base F0240/F0241 share native address `0x5450e0`; that physical group is checked as their sum, never as separate invented counts. Exact workload sources and expected results are in [s8-resume-workloads.json](../../../.temp/brief-616/s8-resume-workloads.json).

All 67 oracle fixtures passed before conversion in both builds (67 passed, 214 filtered out per build; zero faults across 804 observations). The final full oracle runs cover all 67 again under Checks::ALL and unconditional FaultProbe absence. The fixtures below include every director shape (director_01–director_48) and all 19 harness shapes.

| Fixture | Setup | Source | Prompts | Expected value |
| --- | --- | --- | --- | --- |
| director_01 |  | f[1] /. f[x_] /; (InputString["g"]; True) :> x | g | 1 |
| director_02 |  | {1, 2, 3} /. x_Integer /; (InputString["g"]; x > 1) :> 10 x | g, g, g | List[1, 20, 30] |
| director_03 |  | 1 /. x_ /; (InputString["g"]; True) :> 7 | g | 7 |
| director_04 |  | {1, 2} /. {x_ /; (InputString["g"]; False) :> 0, x_Integer :> x + 1} | g, g, g, g | List[2, 3] |
| director_05 |  | {{1}, {2, {3}}} /. x_Integer /; (InputString["g"]; True) :> -x | g, g, g | List[List[-1], List[-2, List[-3]]] |
| director_06 |  | g[h[1], 2] /. {h[x_] /; (InputString["g"]; True) :> x, 2 -> 3} | g | g[1, 3] |
| director_07 |  | {1, 2} /. x_Integer /; (InputString["g"]; True) :> Sequence[x, x] | g, g | List[1, 1, 2, 2] |
| director_08 |  | Hold[1 + 1] /. x_Integer /; (InputString["g"]; True) :> 5 | g, g | Hold[Plus[5, 5]] |
| director_09 |  | f[3] //. f[x_ /; (InputString["g"]; x > 0)] :> f[x - 1] | g, g, g, g | f[0] |
| director_10 |  | {a, b} //. {a /; (InputString["g"]; True) -> b, b -> c} | g | List[c, c] |
| director_11 |  | Replace[5, x_ /; (InputString["g"]; True) :> x + 1] | g | 6 |
| director_12 |  | Replace[{1, 2}, x_Integer /; (InputString["g"]; True) :> x + 1, {1}] | g, g | List[2, 3] |
| director_13 |  | Replace[3, {x_ /; (InputString["g"]; False) :> 0, x_ /; (InputString["g"]; True) :> 9}] | g, g | 9 |
| director_14 |  | ReplaceAt[{1, 2, 3}, x_ /; (InputString["g"]; True) :> 0, 2] | g | List[1, 0, 3] |
| director_15 |  | {1} /. x_Integer /; (InputString["g"]; True) :> System`InternalContinuationNone[] | g | List[InternalContinuationNone[]] |
| director_16 |  | {1} /. x_Integer /; (InputString["g"]; True) :> System`InternalContinuationSome[5] | g | List[InternalContinuationSome[5]] |
| director_17 |  | {1} /. x_Integer /; (InputString["g"]; True) :> System`InternalContinuationReplaceAllTraversalOutcome[x, True] | g | List[InternalContinuationReplaceAllTraversalOutcome[1, True]] |
| director_18 |  | {System`InternalContinuationNone[], 1} /. x_Integer /; (InputString["g"]; True) :> 2 | g | List[InternalContinuationNone[], 2] |
| director_19 |  | Replace[1, x_ /; (InputString["g"]; True) :> System`InternalContinuationReplaceAllTraversalOutcome[2, False]] | g | InternalContinuationReplaceAllTraversalOutcome[2, False] |
| director_20 |  | Count[(InputString["t"]; {1, 2, 1}), 1] | t | 2 |
| director_21 |  | MemberQ[(InputString["t"]; {1, 2}), 2] | t | True |
| director_22 |  | FreeQ[(InputString["t"]; {1, 2}), 3] | t | True |
| director_23 |  | Count[{1, 2, 1}, (InputString["p"]; 1)] | p | 2 |
| director_24 |  | MemberQ[{1, 2}, (InputString["p"]; 2)] | p | True |
| director_25 |  | FreeQ[{1, 2}, (InputString["p"]; 2)] | p | False |
| director_26 |  | Count[{1, {1}}, 1, (InputString["l"]; {2})] | l | 1 |
| director_27 |  | MemberQ[{1, {2}}, 2, (InputString["l"]; {2})] | l | True |
| director_28 |  | FreeQ[{1, {2}}, 2, (InputString["l"]; {1})] | l | True |
| director_29 |  | Count[{1, 2, 3}, x_ /; (InputString["t"]; x > 1)] | t, t, t | 2 |
| director_30 |  | MemberQ[{1, 2, 3}, x_ /; (InputString["t"]; x > 2)] | t, t, t | True |
| director_31 |  | FreeQ[{1, 2, 3}, x_ /; (InputString["t"]; x > 5)] | t, t, t, t, t | True |
| director_32 |  | Count[f[f, 1], f, {0, Infinity}, Heads -> (InputString["h"]; True)] | h | 2 |
| director_33 |  | Count[f[g, 1], s_Symbol /; (InputString["g"]; True), {0, Infinity}, Heads -> True] | g, g | 2 |
| director_34 |  | Count[{System`InternalContinuationNone[], System`InternalContinuationSome[5], 1}, x_ /; (InputString["g"]; True)] | g, g, g | 3 |
| director_35 |  | MemberQ[{System`InternalContinuationSome[5]}, (InputString["p"]; System`InternalContinuationSome[5])] | p | True |
| director_36 |  | Position[(InputString["t"]; {1, 2, 1}), 1] | t | List[List[1], List[3]] |
| director_37 |  | Position[{1, 2, 1}, (InputString["p"]; 1)] | p | List[List[1], List[3]] |
| director_38 |  | Position[{1, {1}}, 1, (InputString["l"]; {2})] | l | List[List[2, 1]] |
| director_39 |  | Position[{1, 1, 1}, 1, {1}, (InputString["n"]; 2)] | n | List[List[1], List[2]] |
| director_40 |  | Position[{1, 2, 3}, x_ /; (InputString["t"]; x > 1)] | t, t, t, t, t | List[List[2], List[3]] |
| director_41 |  | Position[{{1}, {2, 1}}, x_Integer /; (InputString["t"]; x == 1)] | t, t, t | List[List[1, 1], List[2, 2]] |
| director_42 |  | Position[{1, 2, 3}, x_ /; (InputString["g"]; True), {1}, 2] | g, g | List[List[0], List[1]] |
| director_43 |  | Position[{System`InternalContinuationNone[], System`InternalContinuationSome[5]}, x_ /; (InputString["g"]; True)] | g, g, g, g, g, g, g | List[List[0], List[1, 0], List[1], List[2, 0], List[2, 1], List[2], List[]] |
| director_44 |  | Position[f[f, 1], f, Heads -> (InputString["h"]; True)] | h | List[List[0], List[1]] |
| director_45 | c = 0; | Do[c += Count[{1, 2}, x_ /; (InputString["t"]; True)], {3}]; c | t, t, t, t, t, t | 6 |
| director_46 |  | Table[{i} /. x_Integer /; (InputString["g"]; True) :> x + 1, {i, 3}] | g, g, g | List[List[2], List[3], List[4]] |
| director_47 |  | Table[Position[{i, 0}, x_ /; (InputString["t"]; x > 0)], {i, 2}] | t, t, t, t, t, t, t, t | List[List[List[1]], List[List[1]]] |
| director_48 |  | Table[MemberQ[{i}, x_ /; (InputString["g"]; True)] && FreeQ[{i}, x_ /; (InputString["g"]; True)], {i, 2}] | g, g, g, g | List[False, False] |
| s8_replaceall_node |  | f[1] /. x_Integer /; (InputString["guard"];True) :> x+1 | guard | f[2] |
| s8_replaceall_atom |  | 1 /. x_Integer /; (InputString["guard"];True) :> x+1 | guard | 2 |
| s8_replaceall_twice |  | {1,2} /. x_Integer /; (InputString["guard"];True) :> x+1 | guard, guard | List[2, 3] |
| s8_repeated_guard |  | ReplaceRepeated[f[1], f[1] /; (InputString["guard"];True) :> f[0]] | guard | f[0] |
| s8_replace_decline |  | Replace[3,{x_ /; (InputString["first"];False) :> 0,x_ :> 9}] | first | 9 |
| s8_replace_both |  | Replace[3,{x_ /; (InputString["first"];False) :> 0,x_ /; (InputString["later"];True) :> 9}] | first, later | 9 |
| s8_replaceat_guard |  | ReplaceAt[{1,2,3},x_ /; (InputString["guard"];True) :> x+1,2] | guard | List[1, 3, 3] |
| s8_count_target |  | Count[(InputString["target"];{1}),_Integer] | target | 1 |
| s8_count_pattern |  | Count[{1},(InputString["pattern"];_Integer)] | pattern | 1 |
| s8_memberq_target |  | MemberQ[(InputString["target"];{1}),_Integer] | target | True |
| s8_memberq_pattern |  | MemberQ[{1},(InputString["pattern"];_Integer)] | pattern | True |
| s8_freeq_target |  | FreeQ[(InputString["target"];{1}),_Integer] | target | False |
| s8_freeq_pattern |  | FreeQ[{1},(InputString["pattern"];_Integer)] | pattern | False |
| s8_count_level |  | Count[{1},_Integer,(InputString["level"];{1})] | level | 1 |
| s8_count_guard_twice |  | Count[{1,2},x_Integer /; (InputString["guard"];True)] | guard, guard | 2 |
| s8_position_target |  | Position[(InputString["target"];{1}),1] | target | List[List[1]] |
| s8_position_pattern |  | Position[{1},(InputString["pattern"];1)] | pattern | List[List[1]] |
| s8_position_level |  | Position[{1},1,(InputString["level"];{1})] | level | List[List[1]] |
| s8_position_guard_twice |  | Position[{1,2},x_Integer /; (InputString["guard"];True)] | guard, guard | List[List[1], List[2]] |

The independent director probes also pass: S6 22/22 SAME (1–3 pauses), S7 32/32 SAME (1–3), S8 48/48 SAME (1–8). Every shape paused. The copied runner preserves all formulas; only scratch build paths changed. Both ordinary-control builds pass all 28 controls with zero faults across 336 observations each.

### 4. Gates, replay and canonical outputs

All mandatory functional gates pass on the frozen source. Commands use the prescribed bounded serial environment (`CARGO_BUILD_JOBS=1`, `RUST_TEST_THREADS=1`, `CARGO_NET_OFFLINE=true`, scratch TMPDIR). Workspace bound is 3600 seconds; the unchanged oracle script has 1500 seconds per build (3000 aggregate wrapper); other commands use at most 900 seconds. Commands below retain exact executed arguments, including nocapture where used.

**base-s8-fixtures-default** — exit 0; 15.33 s. [Raw output](../../../.temp/brief-616/gates/base-s8-fixtures-default/output.txt).

```text
cargo test -p rum-stdlib --test suspension_oracle replaceall_traversal_collectors -- --nocapture
```

```text
test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 214 filtered out; finished in 0.51s
```

**base-s8-fixtures-no-default** — exit 0; 13.10 s. [Raw output](../../../.temp/brief-616/gates/base-s8-fixtures-no-default/output.txt).

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle replaceall_traversal_collectors -- --nocapture
```

```text
test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 214 filtered out; finished in 0.51s
```

**candidate-s8-fixtures-default-1** — exit 0; 37.49 s. [Raw output](../../../.temp/brief-616/gates/candidate-s8-fixtures-default-1/output.txt).

```text
cargo test -p rum-stdlib --test suspension_oracle replaceall_traversal_collectors -- --nocapture
```

```text
test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 214 filtered out; finished in 0.56s
```

**candidate-s8-fixtures-no-default-1** — exit 0; 24.49 s. [Raw output](../../../.temp/brief-616/gates/candidate-s8-fixtures-no-default-1/output.txt).

```text
cargo test -p rum-stdlib --no-default-features --test suspension_oracle replaceall_traversal_collectors -- --nocapture
```

```text
test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 214 filtered out; finished in 0.50s
```

**census-discovery-s8-1** — exit 101; 33.45 s. [Raw output](../../../.temp/brief-616/gates/census-discovery-s8-1/output.txt).

```text
cargo test -p rum-stdlib --test typed_slot_census source_baseline_matches_earned_receipt -- --exact --nocapture
```

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 21.52s
```

**census-s8-baseline-1** — exit 0; 46.61 s. [Raw output](../../../.temp/brief-616/gates/census-s8-baseline-1/output.txt).

```text
cargo test -p rum-stdlib --test typed_slot_census source_baseline_matches_earned_receipt -- --exact --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 29.70s
```

**ownership-discovery** — exit 0; 41.38 s. [Raw output](../../../.temp/brief-616/gates/ownership-discovery/output.txt).

```text
cargo test -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 5.74s
```

**regenerate-contracts** — exit 0; 15.52 s. [Raw output](../../../.temp/brief-616/gates/regenerate-contracts/output.txt).

```text
cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 0.17s
```

**certificate-emission-default** — exit 101; 35.01 s. [Raw output](../../../.temp/brief-616/gates/certificate-emission-default/output.txt).

```text
cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.59s
```

**certificate-emission-no-default** — exit 101; 52.91 s. [Raw output](../../../.temp/brief-616/gates/certificate-emission-no-default/output.txt).

```text
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.32s
```

**fmt-final** — exit 0; 1.91 s. [Raw output](../../../.temp/brief-616/gates/fmt-final/output.txt).

```text
cargo fmt --check
```

```text
(No test result line: command/build or standalone probe.)
```

**typed-runtime** — exit 0; 23.59 s. [Raw output](../../../.temp/brief-616/gates/typed-runtime/output.txt).

```text
cargo test -p rum-runtime --lib suspension::typed::tests -- --nocapture
```

```text
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 0.53s
```

**frame-root-trace** — exit 0; 1.32 s. [Raw output](../../../.temp/brief-616/gates/frame-root-trace/output.txt).

```text
cargo test -p rum-runtime --lib frame_root_trace -- --nocapture
```

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 1.26s
```

**resume-context** — exit 0; 0.22 s. [Raw output](../../../.temp/brief-616/gates/resume-context/output.txt).

```text
cargo test -p rum-runtime --lib resume_evaluation_context -- --nocapture
```

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1176 filtered out; finished in 0.16s
```

**bounded-recursion** — exit 0; 0.06 s. [Raw output](../../../.temp/brief-616/gates/bounded-recursion/output.txt).

```text
cargo test -p rum-runtime --lib runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds -- --nocapture --exact
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s
```

**controls-default** — exit 0; 48.94 s. [Raw output](../../../.temp/brief-616/gates/controls-default/output.txt).

```text
cargo test -p rum-stdlib --test typed_slot_compile_controls -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 48.59s
```

**controls-no-default** — exit 0; 34.60 s. [Raw output](../../../.temp/brief-616/gates/controls-no-default/output.txt).

```text
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.29s
```

**census-default** — exit 0; 265.43 s. [Raw output](../../../.temp/brief-616/gates/census-default/output.txt).

```text
cargo test -p rum-stdlib --test typed_slot_census -- --nocapture
```

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.62s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.62s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 265.34s
```

**census-no-default** — exit 0; 272.43 s. [Raw output](../../../.temp/brief-616/gates/census-no-default/output.txt).

```text
cargo test -p rum-stdlib --no-default-features --test typed_slot_census -- --nocapture
```

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.87s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.95s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 266.52s
```

**suspension-oracle** — exit 0; 82.75 s. [Raw output](../../../.temp/brief-616/gates/suspension-oracle/output.txt).

```text
scripts/check-suspension-oracle.sh 1500
```

```text
test result: ok. 281 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.85s
test result: ok. 281 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.75s
```

**ownership-graph** — exit 0; 70.07 s. [Raw output](../../../.temp/brief-616/gates/ownership-graph/output.txt).

```text
cargo test -p rum-stdlib --lib ownership_graph -- --nocapture
```

```text
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 69.99s
```

**certificates-default** — exit 0; 18.55 s. [Raw output](../../../.temp/brief-616/gates/certificates-default/output.txt).

```text
cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.46s
```

**certificates-no-default** — exit 0; 33.67 s. [Raw output](../../../.temp/brief-616/gates/certificates-no-default/output.txt).

```text
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.37s
```

**production-rejection-census** — exit 0; 40.75 s. [Raw output](../../../.temp/brief-616/gates/production-rejection-census/output.txt).

```text
cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 25.60s
```

**ordinary-controls-default** — exit 0; 58.24 s. [Raw output](../../../.temp/brief-616/gates/ordinary-controls-default/output.txt).

```text
env CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-616/target-ordinary-controls cargo test --offline --manifest-path /home/remi/dev/rum/.temp/brief-616/ordinary-controls/Cargo.toml --test ordinary -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

**ordinary-controls-no-default** — exit 0; 21.52 s. [Raw output](../../../.temp/brief-616/gates/ordinary-controls-no-default/output.txt).

```text
env CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-616/target-ordinary-controls cargo test --offline --manifest-path /home/remi/dev/rum/.temp/brief-616/ordinary-controls/Cargo.toml --no-default-features --test ordinary -- --nocapture
```

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

**director-probe-build** — exit 0; 56.65 s. [Raw output](../../../.temp/brief-616/gates/director-probe-build/output.txt).

```text
env CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-616/target-director-probe cargo build --offline --manifest-path /home/remi/dev/rum/.temp/brief-616/director-probe/Cargo.toml
```

```text
(No test result line: command/build or standalone probe.)
```

**director-probe-s6** — exit 0; 0.17 s. [Raw output](../../../.temp/brief-616/gates/director-probe-s6/output.txt).

```text
python3 .temp/brief-616/run-director-probe.py s6
```

```text
(No test result line: command/build or standalone probe.)
```

**director-probe-s7** — exit 0; 0.18 s. [Raw output](../../../.temp/brief-616/gates/director-probe-s7/output.txt).

```text
python3 .temp/brief-616/run-director-probe.py s7
```

```text
(No test result line: command/build or standalone probe.)
```

**director-probe-s8** — exit 0; 0.20 s. [Raw output](../../../.temp/brief-616/gates/director-probe-s8/output.txt).

```text
python3 .temp/brief-616/run-director-probe.py s8
```

```text
(No test result line: command/build or standalone probe.)
```

**workspace** — exit 0; 2076.34 s. [Raw output](../../../.temp/brief-616/gates/workspace/output.txt).

```text
cargo test --workspace --no-fail-fast -- --nocapture
```

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 40.95s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.04s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 1184 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 43.57s
test result: ok. 1109 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1320.64s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.21s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.16s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 281 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.06s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.68s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.54s
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 265.85s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.37s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.61s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.87s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.70s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The preliminary census-discovery failure was the stale owned Whole pin; the two certificate-emission failures emitted the new canonical cells. They were replayed and all final gates are green. Each final census intentionally runs two failing scratch children (missing frame and wrong result binding); their FAILED totals above are expected falsifiers, not failed parent gates. Workspace also includes those two expected child failures.

Workspace raw totals are 3713 passed and 59 ignored. Nine native ZeroMQ tests explicitly skipped under the required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, so executed passes are 3704; the director owns native loopback execution. No new warning text versus S7: three existing runtime test helpers and existing compiler-proof warnings remain. Both standalone oracle builds have no compiler warnings.

| Oracle build | Fault delta | Observations | Warnings |
| --- | --- | --- | --- |
| default | 0 | 92058 | 0 |
| no-default | 0 | 92058 | 0 |

Both certificate builds retain exactly 3 production instance marks and 4 probe body marks, no fixture/helper marks, and all seven certificate entries have zero rejections. Four C01/C02 hash=true/false cells are copied from canonical emission; the other seven TSV columns are byte-identical. The 68 context fingerprints below are canonical replays; the other 18 contract columns are unchanged. No fingerprint, certificate cell or ledger chain was handwritten.

| Contract | Old fingerprint | New fingerprint |
| --- | --- | --- |
| B01 | fnv1a64:3d1bec37bb2a7957 | fnv1a64:088e19a7e4c8a0d2 |
| B02 | fnv1a64:32aaa4812eb293d6 | fnv1a64:b96077e4386cccbf |
| B03 | fnv1a64:29665e09fb003061 | fnv1a64:d875f3ac9366dacc |
| B04 | fnv1a64:7b57a9ebcd96819f | fnv1a64:614ad680aaa4de9a |
| B05 | fnv1a64:f2b749063d152cba | fnv1a64:451f3196034af68b |
| B06 | fnv1a64:6acc8583ed1e0343 | fnv1a64:329cb27fb700cad6 |
| B07 | fnv1a64:4a14aca3315494ea | fnv1a64:14dc70aa71023ebb |
| B08 | fnv1a64:e017bd39ba9abee5 | fnv1a64:031637e031b9c678 |
| B09 | fnv1a64:de373a886d79270b | fnv1a64:150896baa3e4d31e |
| B10 | fnv1a64:b7d20eaa316900b8 | fnv1a64:1a636da44e6fed21 |
| B11 | fnv1a64:3caf5b48e78aa79f | fnv1a64:22a287ddc499049a |
| B12 | fnv1a64:9d43ba844981bc0b | fnv1a64:d41516b67fed681e |
| I01 | fnv1a64:1afdd73527e26fd7 | fnv1a64:e67004a551809752 |
| I02 | fnv1a64:49f8c69a354bb4ba | fnv1a64:9c60af29fb817e8b |
| I03 | fnv1a64:2d7e1cb048ad7d4b | fnv1a64:3a3e80463d06d25e |
| I04 | fnv1a64:5dfa8d63919ad376 | fnv1a64:fddc8e67dcfa4ddf |
| P01 | fnv1a64:7f3e3121f66c0be3 | fnv1a64:2a44c2426ad533f6 |
| P02 | fnv1a64:8b0b4d588fbfb488 | fnv1a64:f3f86a5e6fb821f1 |
| P03 | fnv1a64:31ab253bde4a1239 | fnv1a64:82d72bdfdd2d12e4 |
| P04 | fnv1a64:2785c296b35bafd6 | fnv1a64:ae3b95f9bd15e8bf |
| P05 | fnv1a64:ef05cff88033e367 | fnv1a64:8b484d85250555e2 |
| P06 | fnv1a64:edc2df54d5b1558b | fnv1a64:24943b870c1d019e |
| P070 | fnv1a64:28107b96b426cf8b | fnv1a64:5ee1d7c8ea927b9e |
| P080 | fnv1a64:8f8453e979a31ef8 | fnv1a64:f7255ce35ce81561 |
| P090 | fnv1a64:a69870a5f3b2bb16 | fnv1a64:66f80ee67a49eaff |
| P100 | fnv1a64:714e4d361548398f | fnv1a64:e64504de4e4a624a |
| P110 | fnv1a64:bd6b023a352183d0 | fnv1a64:fd5a21117394ed99 |
| P120 | fnv1a64:2b7f52ba450feb05 | fnv1a64:23d0d412ef70cf98 |
| P130 | fnv1a64:ede2060060086eea | fnv1a64:b8a9ca079fb618bb |
| P140 | fnv1a64:96a463c49fa252ce | fnv1a64:788789f2ee6efe57 |
| P150 | fnv1a64:3a3ee8a41a4588b6 | fnv1a64:20cafc160f35b21f |
| P071 | fnv1a64:0caf461a6719840a | fnv1a64:b735acf1d4d0105b |
| P081 | fnv1a64:2f1bae8078671f2b | fnv1a64:9b46b476bfee0cbe |
| P091 | fnv1a64:22fa540fa2d83e23 | fnv1a64:7baea7d38de66036 |
| P101 | fnv1a64:b56df3076a8520d0 | fnv1a64:f55d11dea8f88a99 |
| P111 | fnv1a64:3197f37f5d41827b | fnv1a64:28ec459cf8340f4e |
| P121 | fnv1a64:84e2c3105492cf84 | fnv1a64:d257f850be4bd585 |
| P131 | fnv1a64:1901cc5203eaa31f | fnv1a64:fef4f8e6e0f9001a |
| P141 | fnv1a64:0f01548d8a043783 | fnv1a64:8e2eccb4d16e2b16 |
| P151 | fnv1a64:31b1992325d0c3c5 | fnv1a64:3072794ab9aea358 |
| B13 | fnv1a64:f8dd190d1dbabf43 | fnv1a64:c0ad4608e79d86d6 |
| R01 | fnv1a64:a384cfa7c3dbd446 | fnv1a64:c2ec18319fdb506f |
| R02 | fnv1a64:8d6e484d297b4179 | fnv1a64:4aaea926bbb24a24 |
| R03 | fnv1a64:eed06cd6609986b2 | fnv1a64:590458c0d3c4c8c3 |
| R040 | fnv1a64:ecbe44e010d1f7e6 | fnv1a64:93597c69d9b6408f |
| R050 | fnv1a64:73c3ceb78e9ee647 | fnv1a64:87ce9cb857e2eec2 |
| R060 | fnv1a64:a39c478a875cf3fe | fnv1a64:f7cbbe58d6b0a5c7 |
| R070 | fnv1a64:9789769a5a08de2b | fnv1a64:03b47c90a18fcbbe |
| R080 | fnv1a64:24fef656611f9c76 | fnv1a64:c4e0f75aac7f16df |
| R090 | fnv1a64:e3e9c793fb00bdcf | fnv1a64:f9d2f418ffe4628a |
| R100 | fnv1a64:ee8c025ca4d5dae1 | fnv1a64:9d9b97ff3d3c854c |
| R110 | fnv1a64:760e053826342ec0 | fnv1a64:e76dc2c615b48049 |
| R041 | fnv1a64:df167c3734e27e43 | fnv1a64:a6e6a932fec545d6 |
| R051 | fnv1a64:34ba9ebb80e0c796 | fnv1a64:f51a3cfc0777f77f |
| R061 | fnv1a64:82369e6cc28a9e35 | fnv1a64:49f5f39e73ba95c8 |
| R071 | fnv1a64:28f1eba9a752d16c | fnv1a64:82ee0742bc064ded |
| R081 | fnv1a64:2d38b68ab0f40e8f | fnv1a64:a22f6e32e9f6374a |
| R091 | fnv1a64:58beee0d2b9cab8e | fnv1a64:8be736b375751617 |
| R101 | fnv1a64:e27e994ec877333e | fnv1a64:7415b5ec6c773b07 |
| R111 | fnv1a64:f109104e1b87decd | fnv1a64:0a215d0b36bbb3e0 |
| R12 | fnv1a64:476c08f9c953538d | fnv1a64:76d97d82441c9ea0 |
| R13 | fnv1a64:a1a71e24ebe725b1 | fnv1a64:75f92c51822c129c |
| R14 | fnv1a64:8ba65babf1f8f98f | fnv1a64:009d13542afb224a |
| R15 | fnv1a64:81856b3a2c31a3d3 | fnv1a64:49d348db1a2eb966 |
| O010 | fnv1a64:ba7b55b59f79ea15 | fnv1a64:fa5da8eb3a233d28 |
| O0200 | fnv1a64:916c0598ee5978ce | fnv1a64:734f2bc73d262457 |
| O011 | fnv1a64:e625041294eeb87e | fnv1a64:3a547ae0e4426a47 |
| O0210 | fnv1a64:0b8062daf2a27aed | fnv1a64:c281e56c96884e00 |

Every certificate cell, ownership row and retention row is disclosed with its old/new evidence in [certificate-diff.json](../../../.temp/brief-616/certificate-diff.json), [ownership-row-changes.json](../../../.temp/brief-616/ownership-row-changes.json) and [retention-replay/reconciliation.json](../../../.temp/brief-616/retention-replay/reconciliation.json). These are linked exact row ledgers, not sampling summaries.

Ownership discovery reconciles 79 changed rows: 34 coordinate moves, 12 macro renames, 33 owned deletions. Unknown attributes remain 1513; captures become 2931; macros become 3122; self-reaching cycles remain 59. Storage 94, host payloads 79 and their ledger are unchanged. Retention rows 48556 → 48627: 213 paired moves/renames, 20 retired owned macro rows, 91 newly rejected rows. First-failure rows 7359 → 7367: 36 moves, eight changed owned route signatures still RET002, and eight new cold helpers RET002. No unchanged operation disappears or changes meaning. Marks, coverage and promotion stay fixed; no new UnknownHostPayload. The checked option frontier is 950 = 10 RET001 + 913 RET002 + 27 RET003; wrapper frontier stays 16. Historical console text still says 927/890; the unchanged diagnostic wording is not the asserted count.

Unchanged canonical tools wrote their fixed roots: `.temp/brief-542/`, `.temp/brief-543/`, `.temp/brief-546/`, `.temp/typed-slot-census/`, `.temp/typed-slot-compile-controls/`, `.temp/suspension-oracle/`, `target/brief-535/`, `target/brief-539/`, `target/brief-540/`, `target/brief-544/`, and normal Cargo `target/debug/`. The exact file inventory is recorded separately at final audit. All agent scratch, copied harnesses/source trees, candidate/base measurement targets, profiles and reports are under `.temp/brief-616/`. No canonical output path was redirected.

### 5a. Stack (worker)

**Stack measurements and attribution.** All 240 candidate marker tests passed (ten shapes × two profiles × four heights × three samples), against the 240 calibrated amendment-1 base samples. All fits have zero residual. The six historical shapes retain their exact required slopes; all four S8 guard shapes decrease in both profiles. Eighty retained native backtraces (base/candidate × debug/release × ten shapes × depths 2/4) reconcile every measured delta. No recursive shape grows.

| Profile | Shape | Base bytes/level | Candidate | Delta | Unattributed |
| --- | --- | --- | --- | --- | --- |
| debug | peel | 19024 | 19024 | 0 | 0 |
| debug | plus | 34912 | 34912 | 0 | 0 |
| debug | sub | 29376 | 29376 | 0 | 0 |
| debug | up | 28064 | 28064 | 0 | 0 |
| debug | guarded | 35840 | 35840 | 0 | 0 |
| debug | rhs-guard | 49152 | 49152 | 0 | 0 |
| debug | replace-guard | 65056 | 64880 | -176 | 0 |
| debug | replaceall-guard | 64080 | 63296 | -784 | 0 |
| debug | count-guard | 74928 | 74064 | -864 | 0 |
| debug | position-guard | 86832 | 85648 | -1184 | 0 |
| release | peel | 7344 | 7344 | 0 | 0 |
| release | plus | 13792 | 13792 | 0 | 0 |
| release | sub | 13168 | 13168 | 0 | 0 |
| release | up | 11456 | 11456 | 0 | 0 |
| release | guarded | 14240 | 14240 | 0 | 0 |
| release | rhs-guard | 17920 | 17920 | 0 | 0 |
| release | replace-guard | 26960 | 26768 | -192 | 0 |
| release | replaceall-guard | 25264 | 24640 | -624 | 0 |
| release | count-guard | 28560 | 28416 | -144 | 0 |
| release | position-guard | 33664 | 33632 | -32 | 0 |

The first backtrace parser omitted `<rum_runtime… as Trait>` names, leaving +16 bytes unexplained for release Count. Re-reading the same saved backtraces with trait names included identifies `CountLikeOptionWorking::finish` (680 → 696 prologue bytes) and closes that residual. The original `stack-compare` output is retained; `stack-compare-traits` is the corrected final analysis. No probe or production source changed.

All 52 changed source functions are tabulated below in both profiles. Each cell is native address:prologue bytes; multiple generic instantiations and compiler aliases are retained. “Not emitted” means optimized/inlined or removed, never a measured zero-byte frame. Full names/aliases are in `stack/complete-changed-function-coverage.json`; closures and generated cold insertions add 148 named groups in `stack/changed-prologues.json`.

| Profile | Source function | Base addresses:bytes | Candidate addresses:bytes |
| --- | --- | --- | --- |
| debug | apply_replace_all | 0x1b5420:696 | 0x1a0b10:568 |
| debug | apply_replace_all_owned | 0x1b5e20:1192 | 0x1a1590:824 |
| debug | apply_replace_all_with_plan | 0x1b57e0:632 | 0x1a0f10:504 |
| debug | apply_rewrite_rule_plan_once | 0x1b5c40:248 | 0x1a13b0:248 |
| debug | apply_rules_once_from_list | 0x1b84f0:1416 | 0xa6a830:1240, 0xa6b260:1240 |
| debug | collect_positions_owned | 0x304cc0:1944 | 0x2edc60:1416 |
| debug | count_pattern_matches_owned | 0x300ef0:328 | 0x2ea780:328 |
| debug | drive_count_pattern_matches | 0x301080:1368 | 0xab6100:1176, 0xab6970:1176 |
| debug | drive_replace_all_traversal | 0x1b6690:2088 | 0xa68760:1976, 0xa697c0:2008 |
| debug | drive_suspended_position_traversal | 0x305c40:1752 | 0x2eeae0:1528 |
| debug | eval_count | 0x2fc920:1272 | 0x2e6500:1160 |
| debug | eval_free_q | 0x2fd660:1272 | 0x2e71a0:1160 |
| debug | eval_member_q | 0x2fcfc0:1272 | 0x2e6b50:1160 |
| debug | eval_position | 0x303210:1272 | 0x2ec1c0:1160 |
| debug | finish_count_like_after_pattern | 0x300220:1080 | 0x2e9b00:808 |
| debug | finish_count_like_after_traversal | 0x300870:1080 | 0x2ea0d0:1000 |
| debug | finish_position_after_resolved_traversal | 0x304500:1256 | 0x2ed3b0:1192 |
| debug | finish_position_option | 0x302c10:1064 | 0x2ebc40:792 |
| debug | finish_suspended_rewrite_rule_match | 0x1b8f40:360 | 0x1a2b20:360 |
| debug | resume_count_after_target | 0x2fdd00:1320 | 0x2e77f0:1112 |
| debug | resume_count_traversal | 0x3018e0:344 | 0x2ea910:360 |
| debug | resume_free_q_after_target | 0x2ff5c0:1320 | 0x2e8f50:1112 |
| debug | resume_member_q_after_target | 0x2fe960:1320 | 0x2e83a0:1112 |
| debug | resume_position_after_target | 0x3038b0:1320 | 0x2ec810:1112 |
| debug | resume_position_traversal | 0x3058b0:504 | 0x2ee750:520 |
| debug | resume_replace_all_traversal | 0x1b7710:152 | 0x1a1d40:152 |
| debug | suspend_count_like_pattern | not emitted | 0x6b3b60:344 |
| debug | suspend_count_traversal | not emitted | 0xc5e8b0:216, 0xc5e920:216 |
| debug | suspend_position_collection | not emitted | 0x3cd7d0:744 |
| debug | suspend_position_pattern | not emitted | 0x3ad4f0:312 |
| debug | suspend_position_traversal | not emitted | 0x3cda80:408 |
| debug | suspend_replace_all_atom | not emitted | 0x107290:376 |
| debug | suspend_replace_all_traversal | not emitted | 0x87e5a0:152, 0x87e630:152 |
| debug | suspend_rewrite_rule_match | not emitted | 0x87e6c0:280, 0x87e810:280 |
| debug | ApplyReplaceAllOwnedSuspensionFrame::resume | 0xa4b6f0:920 | 0x106b40:920 |
| debug | ApplyReplaceAllSuspensionFrame::resume | 0xa4b3f0:712 | 0x106820:552 |
| debug | ApplyReplaceAllWithPlanSuspensionFrame::resume | 0xa4b570:712 | 0x1069b0:552 |
| debug | ApplyRulesOnceFromListSuspensionFrame::resume | 0xa4ba70:792 | 0x106eb0:1016 |
| debug | ContinueCountTraversalSuspensionFrame::resume | 0x697ce0:840 | 0x6b38c0:1032 |
| debug | ContinuePositionTraversalSuspensionFrame::resume | 0x3520d0:792 | 0x3cd530:1144 |
| debug | EvalCountSuspensionFrame::resume | 0x696740:600 | 0x6b2290:696 |
| debug | EvalFreeQSuspensionFrame::resume | 0x696be0:600 | 0x6b2710:696 |
| debug | EvalMemberQSuspensionFrame::resume | 0x696990:600 | 0x6b24d0:696 |
| debug | EvalPositionSuspensionFrame::resume | 0x351730:600 | 0x3cc9e0:696 |
| debug | FinishCountLikeAfterPatternSuspensionFrame::resume | 0x697760:632 | 0x6b3220:1016 |
| debug | FinishCountLikeAfterTraversalSuspensionFrame::resume | 0x697ac0:728 | 0x6b36e0:568 |
| debug | FinishPositionAfterResolvedTraversalSuspensionFrame::resume | 0x351f50:712 | 0x3cd310:632 |
| debug | ResumeCountAfterTargetSuspensionFrame::resume | 0x696e30:680 | 0x6b2950:776 |
| debug | ResumeFreeQAfterTargetSuspensionFrame::resume | 0x697450:680 | 0x6b2f30:776 |
| debug | ResumeMemberQAfterTargetSuspensionFrame::resume | 0x697140:680 | 0x6b2c40:776 |
| debug | ResumePositionAfterPatternSuspensionFrame::resume | 0x351c90:536 | 0x3ccf10:920 |
| debug | ResumePositionAfterTargetSuspensionFrame::resume | 0x351980:680 | 0x3ccc20:776 |
| release | apply_replace_all | 0x5351f0:216 | not emitted |
| release | apply_replace_all_owned | 0x535720:296 | 0x5355d0:632 |
| release | apply_replace_all_with_plan | not emitted | not emitted |
| release | apply_rewrite_rule_plan_once | 0x535420:280 | 0x534be0:632 |
| release | apply_rules_once_from_list | 0x537350:584 | not emitted |
| release | collect_positions_owned | not emitted | not emitted |
| release | count_pattern_matches_owned | not emitted | not emitted |
| release | drive_count_pattern_matches | 0x7bbcc0:504 | not emitted |
| release | drive_replace_all_traversal | 0x535a40:616 | not emitted |
| release | drive_suspended_position_traversal | not emitted | not emitted |
| release | eval_count | not emitted | not emitted |
| release | eval_free_q | not emitted | not emitted |
| release | eval_member_q | not emitted | not emitted |
| release | eval_position | not emitted | not emitted |
| release | finish_count_like_after_pattern | not emitted | not emitted |
| release | finish_count_like_after_traversal | 0x7bb470:600 | 0x7c4f50:1000 |
| release | finish_position_after_resolved_traversal | 0x7c1fb0:1032 | 0x7c9110:1016 |
| release | finish_position_option | 0x7bdeb0:712 | 0x7c79f0:744 |
| release | finish_suspended_rewrite_rule_match | not emitted | not emitted |
| release | resume_count_after_target | 0x7b9330:344 | 0x7c2f60:296 |
| release | resume_count_traversal | not emitted | not emitted |
| release | resume_free_q_after_target | 0x7ba950:344 | 0x7c44a0:296 |
| release | resume_member_q_after_target | 0x7b9e40:344 | 0x7c3a00:296 |
| release | resume_position_after_target | 0x7c1450:344 | 0x7c8620:296 |
| release | resume_position_traversal | not emitted | not emitted |
| release | resume_replace_all_traversal | not emitted | not emitted |
| release | suspend_count_like_pattern | not emitted | 0x12fa50:216 |
| release | suspend_count_traversal | not emitted | 0x12fc50:24, 0x12fcc0:120 |
| release | suspend_position_collection | not emitted | 0x12ff50:312 |
| release | suspend_position_pattern | not emitted | 0x12fd70:200 |
| release | suspend_position_traversal | not emitted | 0x130110:120 |
| release | suspend_replace_all_atom | not emitted | 0xc6fe0:120 |
| release | suspend_replace_all_traversal | not emitted | 0xc70a0:120, 0xc7150:216 |
| release | suspend_rewrite_rule_match | not emitted | 0xc7250:296, 0xc73a0:200 |
| release | ApplyReplaceAllOwnedSuspensionFrame::resume | 0x533490:808 | not emitted |
| release | ApplyReplaceAllSuspensionFrame::resume | 0x533350:248 | not emitted |
| release | ApplyReplaceAllWithPlanSuspensionFrame::resume | not emitted | not emitted |
| release | ApplyRulesOnceFromListSuspensionFrame::resume | 0x534000:936 | not emitted |
| release | ContinueCountTraversalSuspensionFrame::resume | 0x7b8b60:1224 | not emitted |
| release | ContinuePositionTraversalSuspensionFrame::resume | 0x7bf4b0:1752 | not emitted |
| release | EvalCountSuspensionFrame::resume | 0x7b6ba0:216 | not emitted |
| release | EvalFreeQSuspensionFrame::resume | 0x7b6e00:216 | not emitted |
| release | EvalMemberQSuspensionFrame::resume | 0x7b6cd0:216 | not emitted |
| release | EvalPositionSuspensionFrame::resume | 0x7be4d0:216 | not emitted |
| release | FinishCountLikeAfterPatternSuspensionFrame::resume | 0x7b75f0:888 | not emitted |
| release | FinishCountLikeAfterTraversalSuspensionFrame::resume | 0x7b84a0:440 | not emitted |
| release | FinishPositionAfterResolvedTraversalSuspensionFrame::resume | 0x7bec20:536 | not emitted |
| release | ResumeCountAfterTargetSuspensionFrame::resume | 0x7b6f30:312 | not emitted |
| release | ResumeFreeQAfterTargetSuspensionFrame::resume | 0x7b73b0:312 | not emitted |
| release | ResumeMemberQAfterTargetSuspensionFrame::resume | 0x7b7170:312 | not emitted |
| release | ResumePositionAfterPatternSuspensionFrame::resume | 0x7be840:680 | not emitted |
| release | ResumePositionAfterTargetSuspensionFrame::resume | 0x7be600:312 | not emitted |

**Individual prologue growth remains for director review.** The following 24 emitted groups grow even though all recursive slopes are flat or lower. The two debug ReplaceAll closure groups pair a removed semantic-map closure with the new cold insertion closure by emitted group name; they are different closure roles, not a same-body growth claim. Three unchanged-source resident functions also changed emitted layout and are disclosed. No claim is made that all these reserves fit solely within a named typed-macro environment.

| Profile | Function/group | Delta bytes | Base entry:total/pushes/reserve | Candidate entry:total/pushes/reserve |
| --- | --- | --- | --- | --- |
| debug | apply_replace_all::{{closure}} | 56 | 0x458b90:0/0/0 | 0xcffc80:56/0/56 |
| debug | apply_replace_all_with_plan::{{closure}} | 56 | 0x458bf0:0/0/0 | 0xcffcf0:56/0/56 |
| debug | resume_count_traversal | 16 | 0x3018e0:344/0/344 | 0x2ea910:360/0/360 |
| debug | resume_position_traversal | 16 | 0x3058b0:504/0/504 | 0x2ee750:520/0/520 |
| debug | ApplyRulesOnceFromListSuspensionFrame::resume | 224 | 0xa4ba70:792/0/792 | 0x106eb0:1016/0/1016 |
| debug | ContinueCountTraversalSuspensionFrame::resume | 192 | 0x697ce0:840/0/840 | 0x6b38c0:1032/0/1032 |
| debug | ContinuePositionTraversalSuspensionFrame::resume | 352 | 0x3520d0:792/0/792 | 0x3cd530:1144/0/1144 |
| debug | EvalCountSuspensionFrame::resume | 96 | 0x696740:600/0/600 | 0x6b2290:696/0/696 |
| debug | EvalFreeQSuspensionFrame::resume | 96 | 0x696be0:600/0/600 | 0x6b2710:696/0/696 |
| debug | EvalMemberQSuspensionFrame::resume | 96 | 0x696990:600/0/600 | 0x6b24d0:696/0/696 |
| debug | EvalPositionSuspensionFrame::resume | 96 | 0x351730:600/0/600 | 0x3cc9e0:696/0/696 |
| debug | FinishCountLikeAfterPatternSuspensionFrame::resume | 384 | 0x697760:632/0/632 | 0x6b3220:1016/0/1016 |
| debug | ResumeCountAfterTargetSuspensionFrame::resume | 96 | 0x696e30:680/0/680 | 0x6b2950:776/0/776 |
| debug | ResumeFreeQAfterTargetSuspensionFrame::resume | 96 | 0x697450:680/0/680 | 0x6b2f30:776/0/776 |
| debug | ResumeMemberQAfterTargetSuspensionFrame::resume | 96 | 0x697140:680/0/680 | 0x6b2c40:776/0/776 |
| debug | ResumePositionAfterPatternSuspensionFrame::resume | 384 | 0x351c90:536/0/536 | 0x3ccf10:920/0/920 |
| debug | ResumePositionAfterTargetSuspensionFrame::resume | 96 | 0x351980:680/0/680 | 0x3ccc20:776/0/776 |
| release | apply_replace_all_owned | 336 | 0x535720:296/48/248 | 0x5355d0:632/48/584 |
| release | apply_rewrite_rule_plan_once | 352 | 0x535420:280/48/232 | 0x534be0:632/48/584 |
| release | finish_count_like_after_traversal | 400 | 0x7bb470:600/48/552 | 0x7c4f50:1000/48/952 |
| release | finish_position_option | 32 | 0x7bdeb0:712/48/664 | 0x7c79f0:744/48/696 |
| release | rum_runtime::runtime::definitions::definition_catalog::rewrite_rules::RewriteRuleExprContinuation::resume | 48 | 0x545dc0:1352/48/1304 | 0x545290:1400/48/1352 |
| release | rum_runtime::runtime::definitions::definition_catalog::rewrite_rules::CompiledRewriteRuleContinuation::resume | 128 | 0x5493b0:376/48/328 | 0x548700:504/48/456 |
| release | <rum_runtime::runtime::tree_walkers::count_like::options::CountLikeOptionWorking as rum_runtime::runtime::options::read_phase::OptionReadContinuation<bool>>::finish | 16 | 0x7b6490:680/48/632 | 0x7c2760:696/48/648 |

`stack/growth-slot-ledger.json` and `.tsv` partition every growing reserve at actual rsp-relative accesses and direct DWARF slot offsets, with saved-register bytes accounted separately; the regions sum exactly to each prologue. Every instruction, call argument window, named slot/type/location and anonymous compiler region is retained. There are 1,121 named debug slots for the changed native bodies and 16,226 explicit debug/release stack instructions. Witness and token types are zero bytes; their displayed source offsets do not add storage. Reference/pointee types and overlapping variable lifetimes are never summed as independent resident stack. Release-only slots without emitted DWARF names stay anonymous, with concrete assembly offsets.

For example, debug `ApplyRulesOnceFromListSuspensionFrame::resume` grows 792 → 1016: the old `resumed` Result is 136 bytes at fbreg 376; the typed Result carrying ResumeStop is 144 bytes at fbreg 240, with a 40-byte match outcome at 384 and 136-byte error at 504. Its token is zero bytes. Debug `EvalCountSuspensionFrame::resume` grows 600 → 696 while `resumed` remains 136 bytes at 72; error storage moves 296 → 240 and the zero-byte token is described at 679. The remaining reserved bytes are identified as machine regions in the ledger, not invented source variables.

### 5. Performance evidence (worker gates, then director reproduction)

**Worker, before the cut** (`.temp/brief-616/gates/`):
- 550 native preflights per arm passed: 138 ordinary labels at check/N/2N and
  68 resume labels at N/2N with exact resume counts.
- Allocations: all 138 ordinary labels equal in count, bytes and slope at N
  and 2N; all 68 resume budgets reconcile exactly to one 24-B adapter `Arc`
  per typed entry (`resume-frame-allocation-proof`).
- Instructions: 19 ordinary labels flagged on two pairs, all cleared by five
  runs per arm (`ordinary-five-runs`, `compare-instructions-extra`: "Ordinary
  flags: []").
- Resume attribution: six callgrind processes per arm for every resume
  workload; every per-address delta reconciles, and the driver's 136-B
  typed-outcome move (owed to R) counts one per typed entry. The 228
  process frame censuses pass; F0240/F0241 (shared address) and F0558 are
  inlined into adapters and are counted at their emitted addresses.
- Position's containing body has a +3 local instruction delta, fully
  accounted in its address ledger; the label's deterministic total is
  -12 instructions per iteration.
- Timing (four full panels, base/candidate/base/candidate): 21 labels had a
  disjoint slower candidate range, all with equal deterministic
  instructions; the largest was `association-part-key-read-width4000`
  +21.8%. The worker had started the aligned rebuild when the run was cut.

**Director reproduction** (`.temp/director/gate-616/`):
- Base binaries' hashes unchanged; all four candidates rebuild as no-ops to
  their measured hashes.
- Allocations, 206 labels: 138 ordinary equal; 21 resume labels differ, each
  by exactly 24 B per typed transfer, matching the frame census (for
  example `s8-replaceall-twice` +5 / +120 B = F0240 + 2 F0242 + 2 F0243;
  `s8-count-guard-twice` +3 / +72 B; `resume-count-traversal` and
  `resume-position-traversal` +1 / +24 B); the other 47 resume labels equal.
- Instructions, two interleaved pairs: improvements on the rule scan and
  the collectors (`replaceall-wide-no-match` -3.92%, `replace-wide-no-match`
  -2.75%, `count`/`memberq`/`freeq`-wide about -0.3%, `table-small` -0.33%).
  S8's own resume workloads +0.15-0.52%, the typed-entry cost attributed
  above. Fifteen labels flagged upward were rerun five times per arm and
  every one cleared inside its own spread, including
  `association-part-positional-read-width4000` (+0.36%, spread 2.0/1.1%),
  `resume-s6-readonly-twice` (+0.30%, 0.22/0.80%) and the noisy
  `valueq-undefined-symbol-definitions-2000` (-9.5%, spread 67/93%).
- Stack: all twenty profile/shape slopes reproduce the worker's values
  exactly (table in 5a).
- Probes: S8 48/48, S7 32/32, S6 22/22 and G14 23/23 SAME, every one
  paused.
- Timing, the whole generated panel program with `rumMicroReps` raised to
  25, ten alternating rounds per arm under `taskset -c 2`: on the ordinary
  build the median delta is +0.16% and three labels are disjoint slower
  (`association-part-key-read-width32000` +21.3%, `module-local` +2.2%,
  `with-deep-body-small` +1.5%). Rebuilt with every function 64-byte
  aligned in both arms, the median is -0.02% and **no** label is disjoint
  slower; `association-part-key-read` goes +21.7/+21.3% → +1.6/+0.4%. The
  residuals are layout. Running the whole program needs no cutting, and so
  cannot drop a label's prelude.

### 6. Director rulings

- **Accepted.** Source review: every site follows the S5-S8 routes; the
  Position family keeps its two drivers, the resumed one taking only the
  token.
- **Prologue growth (5a, 24 groups) accepted.** Every function that grows on
  an ordinary path is resident in one of the ten measured recursions, and
  every recursion is flat or lower (release `finish_count_like_after_traversal`
  +400 B sits in `count-guard`, which falls 144 B per level): the growth is
  inlining redistribution. The rest is resume-side bodies carrying the
  typed `Result` with `ResumeStop`, as S7's were.
- **Director gate:** workspace 3 713/0/59 with 0 ZeroMQ skips; oracle
  281/281 in both builds, warning-free; census, compile controls and local
  production in both builds; ownership 31/31; the bounded-recursion test;
  inventories byte-identical after the canonical replay; base source equal
  to `a1359ba8` (2 066 files).
- No stop-loss fired. Deviation: none in source; the report was completed
  by the director because the worker's run was cut.
