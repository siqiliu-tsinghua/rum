# brief-447 — fully iterative `Replace`

## Goal

`Replace` is the last traversal in the deep-rebuild class that still ends the
process. Remove every depth-proportional Rust recursion from it — the traversal
cycle *and* the two depth helpers — so that a stable deepest-leaf
`Replace[value, 1 -> 2, {0, Infinity}]` at physical height 4,097 completes on a
2 MiB stack, and a value whose spine is active fails with a recoverable in-band
resource error rather than terminating the process. Every documented semantic
stays exactly as it is: top-down rule attempt order, level and depth handling,
`Heads`, association values, `Sequence` splicing, messages, and suspend/resume
parity.

This is step 2 of brief-439's plan, re-derived against measurements taken after
step 1 landed. It is a depth-bound change, not a work-removal change — see
"Explicitly out of scope".

## What I measured — treat as given, do not re-run

Release `rum-cli` at `59410b75`, `ulimit -s 2048`, 1 MiB evaluator budget. The
probe builds the value with `x = 1; Do[x = {x}, {i, N}]` and asserts liveness on
the rebuilt result (`MemberQ[r, 2, Infinity]` plus the exact depth).

| case | result |
|---|---|
| `Replace[x, 1 -> 2, {0, Infinity}]` | **process kill**, last ok 1,308, first fail 1,316 |
| `ReplaceAll[x, 1 -> 2]` (control) | ok at 4,097, depth 4,098, hit `True` |
| `Apply[gg, x, {0, Infinity}]` (control) | ok at 4,097 |
| `Cases` / `Count` / `Position` / `ToString` / `Level` (controls) | ok at 4,097 |

1,308 is **identical** to the bracket brief-439 measured before step 1 landed,
which is the evidence that step 1 never touched this site: the kill is
`Replace`'s own recursion, not the shared post-result stability walk that
briefs 440-446 repaired. Confirmed in source — the strings
`poll_native_stack_limit` and `evaluation_depth` occur **zero** times anywhere
under `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/`, so
this descent has no poll point and cannot fail in band.

Do not re-measure any of the above, and do not write a probe that drives the
recursion past its ceiling — the pre-change deep case terminates the test
binary, which is why the falsifier below is a shallow test instead.

## The recursion edges to remove

All in
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs`:

- the traversal cycle — `replace_owned` (`:270`) → `replace_children_owned`
  (`:318`) → `replace_call_children_owned` (`:395`, `:416`) → `replace_owned`
  (`:490`); the association arm `replace_children_owned` → 
  `replace_association_children_owned` (`:347`, `:368`) → `replace_owned`
  (`:573`); and the head arms `replace_children_owned` → `replace_owned`
  (`:339`, `:387`).
- the resume drivers — `drive_suspended_replace_call_children` (`:633`) and
  `drive_suspended_replace_association_children` (`:731`) re-enter
  `replace_owned` at `:678` and `:775`.
- the two depth helpers — `replace_expr_depth` (`:802`, self-recursive at
  `:809` and `:815`) and `replace_expr_depth_with_heads` (`:820`, self-recursive
  at `:827`, `:830`, `:835`, `:838`). They are called per node from `:279-281`,
  `:295-297`, `:466` and `:649`, so each is an independent depth-proportional
  descent and either one alone can end the process.

A conversion that leaves any of these three groups recursive does not meet the
Goal.

## Context pointers

- `AGENTS.md` first, including the Testing Policy hard lines.
- The structural pattern to follow: the iterative `ReplaceAll` state machine
  and its `ReplaceAllTraversalWork`
  (`crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:136-314`).
  Follow its *shape*, not its semantics — see the pitfalls.
- The bounded-front-plus-explicit-stack pattern used elsewhere in this line:
  `crates/rum-runtime/src/runtime/tree_walkers/level.rs:71-87` and
  `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:3-61`.
- The suspension frames you must keep working:
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs`,
  and the tests that exercise them,
  `crates/rum-runtime/src/runtime/tests/session_requests/transforms_resume.rs:55,64,196,219,247`.
- The certificate panel and its conventions:
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:1-64` (helpers,
  the 2 MiB declared thread, `TARGET_PHYSICAL_HEIGHT`), `:117-127` (the
  `ReplaceAll` deep certificate) and `:145-152` (its shallow companion).

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/resume_frames.rs`
- new files under
  `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/`
  (declare them from `once.rs`)
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace.rs` — only if
  a type this change introduces has to cross that module boundary
- `crates/rum-runtime/src/runtime/tests/session_requests/transforms_resume.rs`
  — additive suspension-parity cases only
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
- `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` and
  `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
  — ONLY to record a newly derived frame type, never to weaken the allowlist
- `scripts/core-language-microbench.sh` — only to add the one new label named
  below

Everything else is out of scope, including matcher semantics, `ReplaceAll`,
level-spec parsing, and any public resource limit
(`.../replace/once.rs:251-268`). The Scope list and the Hard rules are the
fence; if you believe you must go outside it, STOP and say so in the report
instead of doing it.

## Explicitly out of scope

The depth helpers run once per visited node, so `Replace` also does repeated
depth work. **Do not remove or cache that work.** Keep the same call schedule
and make each individual call non-recursive. Changing what is computed, and
changing how deep the computation nests, are two different changes and this
brief is only the second one; a combined diff cannot be gated.

## Deliverable

1. `Replace`'s traversal driven by owned visit/finish work items on an explicit
   stack, with level, depth, change state and the rule-plan cursor carried in
   the work item so a suspension can resume in the middle of a deep traversal
   exactly as it does today.
2. Both depth helpers computed without recursion.
3. No new suspension frame that holds an `Expr` without deriving
   `FrameRootTrace`. If a new frame type is needed, derive it — never hand-write
   a traversal, never add an allowlist entry. If the frame-root invariant test
   cannot pass without weakening it, STOP and report.
4. Certificates, following the rule this line settled in briefs 439-442:
   - a release-only `#[ignore]` deep certificate on a 2 MiB declared thread, at
     `TARGET_PHYSICAL_HEIGHT`, with a **unique deepest target whose replacement
     must propagate** and an assertion on the **exact rebuilt `InputForm`
     string** — not on `Depth` or any other summary. Model it on
     `replaceall_completes_at_physical_height_4097_on_two_mib`
     (`iterative_traversal_depth.rs:117-127`);
   - a shallow companion, running in the ordinary suite, that pins the
     `{0, Infinity}` selector on a three-level fixture, modelled on
     `replaceall_certificate_companion_pins_matching_rule` (`:145-152`);
   - an active-spine case asserting a recoverable in-band error, never process
     termination.
5. One new microbench label `replace-leaf-rebuild-small` next to
   `replace-wide-no-match` (`scripts/core-language-microbench.sh:594-598`),
   exercising a small deep-leaf rebuild — the shape this change actually
   alters, since the existing label matches nothing.

## The falsifier — required section

Before changing any source, run the **shallow companion** against unmodified
HEAD and paste its output. Then break it deliberately once — point it at a
selector that does not descend, e.g. `{1}` instead of `{0, Infinity}` — and
paste the failure. That is the proof the companion can fail. Restore it before
continuing.

Do **not** run the deep certificate against unmodified HEAD: at 4,097 the
pre-change code terminates the test binary rather than failing a test, which is
exactly why the companion is the falsifier and why the panel needs both.

## rum / Rust pitfalls (read twice)

1. **`Replace` is not `ReplaceAll`.** A rule is attempted at a node *before* its
   children when the node's level matches, and a successful replacement is
   **not** traversed into. Borrow `ReplaceAllTraversalWork`'s structure only;
   its visit order is not yours (`.../replace/once.rs:270-319`).
2. **Levels are computed from a depth, not just a counter.** `:279-281` picks
   the helper by `include_heads`, and `:295-297` recomputes it for the
   *replacement*. Preserve both, including the recomputation after a successful
   replacement.
3. **Suspension parity is exact.** Every suspended callback resumes exactly once
   with the same rule-plan cursor and partial result; do not turn a
   `suspend_with_frame*` into an eager evaluator call
   (`.../replace/once.rs:270-319`, `:453-512`,
   `.../once/resume_frames.rs`).
4. **Preserve `Heads`, association-value treatment and `Sequence` splicing**,
   including the computed depth of a spliced result.
5. **Two functionally complete conversions were already rejected on this line**
   (briefs 440 and 441) — both passed every test and cost 2-7% on hot labels.
   Correctness is necessary and not sufficient here; the A/B decides.
6. **Do not add default-path cost.** No counter, allocation, `Mutex` or extra
   branch on the ordinary shallow path when nothing deep is happening.
7. **No new `unsafe`**, and no growth of `Expr`/`SharedExpr`/`SharedArgs`
   representation.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the release certificate, after the shallow suite is green:

```
cargo test -p rum-stdlib --release -- --ignored replace
```

Then the A/B. Base = unmodified HEAD build, candidate = your build, both run
from the same working tree with profiling env OFF:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <new-dir>
```

Paste the per-label `rum_seconds` rows for the new `replace-leaf-rebuild-small`
label and for these neighbors: `replace-wide-no-match`,
`replacepart-wide-rule-list`, `replacepart-guarded-hit`,
`replacepart-guarded-decline-fallback`, `rewrite-guarded-hit`,
`rewrite-guarded-decline-fallback`. Best-of-2 per arm; state the cap on total
probe runs you used.

## Stop-loss

Stop and report, with what you have, if:

- exact top-down semantics cannot be preserved without a change outside Scope;
- suspension parity fails twice for the same reason;
- two isolated revisions do not remove an A/B regression on any listed label;
- the frame-root invariant test would have to be weakened;
- the certificate can only be made to pass by capping depth, changing the level
  spec, or lowering `TARGET_PHYSICAL_HEIGHT`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave everything in
  the working tree for the gate.
- Do NOT modify files outside Scope. Touch nothing outside this repository; no
  network.
- Testing Policy is a hard line (`AGENTS.md`): rum's own documented behavior
  only; no oracle harnesses, no third-party engine outputs.
- No new `unsafe` in `rum-core`/`rum-runtime`.
- Use `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything that
  can run long or spawn children; derive the bound from a measured run of the
  same shape.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. What changed — per file, line ranges, one line of why each recursive edge is
   gone. List the three edge groups explicitly and say how each was removed.
2. Verification — every command with verbatim output, including the falsifier
   run against HEAD and its deliberate failure.
3. The deep certificate result and the active-spine result reported as two
   separate outcomes: safety (no process termination) and contract (4,097
   reached).
4. The A/B rows for the target label and every listed neighbor.
5. Stop-loss / open questions.
