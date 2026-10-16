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
