# brief-449 — iterative `DeleteCases`

## Goal

`DeleteCases` is the largest remaining fail-open shortfall in the deep-rebuild
class: over a deep value it ends the process at 1,808 levels, 2.3x short of the
ruled 4,097. Remove the depth-proportional Rust recursion from its traversal so
that a stable deepest-match deletion at physical height 4,097 completes on a
2 MiB stack, and anything deeper fails in band rather than terminating the
process. Every documented semantic stays exactly as it is: post-order visit
order, the running deletion count and its limit short-circuit, computed depth,
association handling, and suspend/resume parity.

This is the first half of brief-439's draft 6, split on measurement — see
"Why this brief is half of draft 6".

## What I measured — treat as given, do not re-run

Release `rum-cli` at `a75d6b52`, `ulimit -s 2048`, 1 MiB evaluator budget. Each
probe carries a liveness assertion on the exact rebuilt value.

| case | last ok / first fail | mode |
|---|---|---|
| `DeleteCases[x, 1, Infinity]` | 1,808 / 1,816 | **process kill** |
| `Function` slot substitution over a deep body | 3,685 / 3,693 | process kill |
| `Module` lexical substitution over a deep body | 3,677 / 3,685 | process kill |
| `With` lexical substitution over a deep body | 3,677 / 3,685 | process kill |
| rule-RHS binding substitution over a deep template | clears 4,097; kills by 8,001 | process kill |

`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/` contains
**zero** occurrences of `poll_native_stack_limit` and `evaluation_depth`, which
is why this is a process kill and not an in-band error.

For reference, the two sites this line has already converted now behave like
this: `Replace` reaches 5,843 and then fails closed, and `D` completes at 4,097.

## Why this brief is half of draft 6

Draft 6 bundled `DeleteCases` with all four substitution engines and a caller
list spanning ten files. The measurements above split that bundle cleanly:

- `DeleteCases` is 2.3x short, lives in its own subtree, and has no caller
  ripple.
- Slot and lexical substitution are 1.11x short and share a wide consumer
  surface (down/up/subvalue application, guards, `callable`, `with`, `module`,
  packages, exceptions, string rules).
- Rule-RHS binding substitution — the engine draft 6 named first — is **not
  short of the contract at all**. It is fail-open above roughly 6,000, which is
  a different and lesser problem.

So this brief is `DeleteCases` alone, which is independently verifiable. The
substitution engines get their own brief with their own numbers.

## The recursion edges to remove

All in
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs`:

- `delete_cases_owned` (`:55`) → `delete_cases_call_children` (`:99`) and
  `delete_cases_association_children` (`:274`);
- both children functions run their queues in
  `delete_cases_call_children_from_queue` (`:139`) and
  `delete_cases_association_children_from_queue` (`:329`), which re-enter
  `delete_cases_owned` at `:247` and `:454`;
- the resume drivers `resume_suspended_delete_call_children` (`:480`) and
  `resume_suspended_delete_association_children` (`:604`) re-enter it at `:586`
  and `:721`.

One mutual recursion group, one native level per physical level.

## Semantics that must not move

Read these off the code before designing the driver.

- **Visit order is post-order.** `delete_cases_owned` on a call goes straight to
  its children; the node's own match runs afterwards in
  `finish_delete_cases_at_current_node` (`:4`). Children are matched before
  their parent, and an iterative driver must keep that exact order.
- **`deleted_before` is a running count and the order it sees is observable.**
  `traversal.limit_reached(deleted_before)` (`:13`) short-circuits with
  `exhausted: true` and `DeleteCasesResult::Keep`, so with an `n` limit *which*
  elements survive depends on visit order. Thread the count exactly as today.
- **`exhausted` propagates.** Once the limit is reached the walk stops deleting;
  it does not stop rebuilding.
- **`depth` is computed as children are processed** (the `1` seeds at `:78`,
  `:88`) and is what `traversal.levels.matches(level, depth)` (`:22`) consults.
  A negative level spec is decided by it, so an off-by-one here is a silent
  semantic change, not a crash.
- **The match can suspend.** `finish_delete_cases_at_current_node` wraps
  `match_expr_owned` in `resume_or_suspend!` with
  `FinishDeleteCasesAtCurrentNodeSuspensionFrame` (`:31-52`). Every suspended
  match must resume exactly once, with the same node, level, depth and
  `deleted_before`. Do not turn a suspension into an eager call.
- Association entries and call args are separate arms today and stay separate.

## Deliverable

1. The traversal driven by owned visit/finish work items on an explicit stack,
   with level, depth, deletion count and partial rebuild carried in the work
   item so a suspension resumes mid-traversal exactly as it does now. Follow the
   depth-16 recursive front plus explicit tail that `Level`, `Map`, `Replace`
   (`.../replace/once/traversal.rs`) and `D`
   (`crates/rum-stdlib/src/calculus/differentiation/iterative.rs`) already use —
   a shallow deletion must not pay for the deep machinery.
2. If a new suspension frame is needed, **derive `FrameRootTrace`** — never
   hand-write a traversal, never add an allowlist entry. Record it in the
   derived inventory; if the frame-root invariant test cannot pass without
   weakening it, STOP and report.
3. Certificates, following the rule this line settled in briefs 439-442:
   - a release-only `#[ignore]` deep certificate on a 2 MiB declared thread at
     `TARGET_PHYSICAL_HEIGHT`, deleting a **unique deepest** element and
     asserting the **exact rebuilt `InputForm`**. The panel helper
     `nested_input_form(TARGET_PHYSICAL_HEIGHT - 1, "")` already spells the
     expected value for `DeleteCases[x, 1, Infinity]`
     (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:43-49`);
   - a shallow companion in the ordinary suite pinning the level selector on a
     three-level fixture, modelled on
     `replace_certificate_companion_pins_descending_selector` (`:196-202`);
   - an active-spine case asserting a recoverable in-band error and a session
     that still works afterwards, modelled on
     `replace_active_spine_returns_in_band_resource_error_on_two_mib`
     (`:150-169`).
4. A shallow regression pinning the **`n`-limit visit order**: a value with
   several matches at different depths, an `n` smaller than the match count, and
   an assertion on exactly which ones survive. This is the semantic most at risk
   from reordering and nothing else in the panel covers it.
5. One new microbench label `deletecases-leaf-delete-small` beside the existing
   `deletecases-*` rows (`scripts/core-language-microbench.sh:584-593`), with a
   sink liveness assertion following the convention at `:546,552,558`.

## The falsifier — required section

Before changing any source, add the shallow companion and the `n`-limit
regression, run both against unmodified HEAD, and paste the passes. Then break
each one deliberately — a selector that does not descend for the companion, a
wrong surviving-element set for the limit test — and paste the failures. Restore
both before continuing.

Do **not** run the deep certificate against unmodified HEAD: at 4,097 the
pre-change code ends the process rather than failing a test.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/traversal.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/resume_frames.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/suspended_state.rs`
- new files under
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/`,
  declared from `delete.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases.rs` — only if a
  type this change introduces has to cross that module boundary
- `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` and
  `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` —
  ONLY to record a newly derived frame type
- focused runtime tests for `DeleteCases` and its suspension behavior
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
- `scripts/core-language-microbench.sh` — only for the one new label

Out of scope: `Cases` collection, `Replace*`, the substitution engines, matcher
semantics, level-spec parsing, and any public resource limit.

## Explicitly out of scope

Do not remove repeated work, cache match results, or change how often
`match_expr_owned` is called. This brief bounds depth and nothing else; a
combined diff cannot be gated.

## rum / Rust pitfalls (read twice)

1. Post-order, not pre-order. `Replace` is top-down and `DeleteCases` is not;
   do not carry the shape over from `.../replace/once/traversal.rs` without
   re-reading this file's order.
2. The deletion limit makes visit order observable. A driver that processes
   children in a different sequence passes most tests and changes which
   elements survive under `n`.
3. Suspension parity is exact: one resume per suspended match, same node, level,
   depth and count.
4. Hot paths avoid owned-tree clones — `Expr`/`SharedExpr`/`SharedArgs` are
   `Arc`-shared; keep unchanged siblings pointer-identical.
5. No default-path cost, no new `unsafe`.
6. **Two functionally complete conversions were rejected on this line already**
   (briefs 440 and 441): correct is necessary, not sufficient. The A/B decides.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

`cargo test --workspace` may fail nine `rum-jupyter` native ZeroMQ tests if your
sandbox cannot bind loopback. That is an environment limit, not a result: say so
explicitly and also run `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace`,
pasting both.

Then, after the shallow suite is green:

```
cargo test -p rum-stdlib --release -- --ignored
```

Note this panel now takes about 460 s end to end because of the `D` certificate
added by brief-448; size your timeout from that, not from a guess.

Then the A/B, base = unmodified HEAD build, candidate = yours, profiling env
OFF:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <new-dir>
```

Paste rows for the new label and for `deletecases-wide-no-match`,
`deletecases-wide-association-no-match`, `cases-wide`, `replaceall-simple`, and
three untouched same-tier neighbors you pick by measured magnitude. Best-of-2
per arm; report each label's same-binary spread beside its cross-binary
movement. **A movement inside its own same-binary spread is not a result** — say
so rather than tuning against it.

## Stop-loss

Stop and report if:

- visit order cannot be preserved without changing the deletion-limit contract;
- suspension parity fails twice for the same reason;
- two isolated revisions do not remove an A/B regression on a listed label;
- the frame-root invariant test would have to be weakened;
- the certificate can only pass by lowering `TARGET_PHYSICAL_HEIGHT` or changing
  the level spec.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the tree for the
  gate.
- Do NOT modify files outside Scope. Touch nothing outside this repository; no
  network.
- Testing Policy is a hard line (`AGENTS.md`): rum's own documented behavior
  only; no oracle harnesses, no third-party engine outputs.
- No new `unsafe` in `rum-core`/`rum-runtime`.
- Use `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything that
  can run long or spawn children; derive the bound from a measured run.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file and line range, naming each removed recursive edge.
2. Verification — every command with verbatim output, including both falsifier
   passes and both deliberate failures, and both workspace runs.
3. Safety and contract as separate outcomes: no process termination, and the
   height reached.
4. A/B rows with same-binary spreads.
5. Stop-loss / open questions.
