# brief-530 — D2a: make the option path suspension-safe, before any lease

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Goal

brief-529 could not convert the option region because the region is not
suspension-safe, and it found the defect that proves it: **queued input inside
a delayed `StringReplace` option produces `"b"`, while suspending and resuming
the same source produces `True`.** The builtin never runs; the option's own
value becomes the call's result.

Fix that. **Build no leases in this brief** — D2b does the accounting. This
one restores correctness, because converting a path whose continuation is
broken would build the accounting on a defect.

## The mechanism, confirmed by the director from source

- `evaluate_runtime_option_expr` evaluates a `RuleDelayed` option's
  right-hand side with `?` — `let value = self.evaluate(args[1].clone())?;`
  (`crates/rum-runtime/src/runtime/options/core.rs:171`). That runs guest
  code and can suspend.
- `merged_option_exprs_for_symbol` propagates it (`:94`, `:97`), and so does
  the consumer, for example
  `crates/rum-stdlib/src/strings/patterns/search.rs:215`.
- Nobody installs a frame, so the generic dispatch frame is what resumes.
  `RunCallDispatchStageSuspensionFrame::resume`
  (`crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:466-492`) takes the
  resumed `Ok(value)` and calls `FrameAction::complete(resumed)` — it
  **completes the whole builtin call with the inner value** and never
  re-enters the handler.

So the Rust continuation is lost: which option we were on, the prefix already
accumulated, and the consumer that was going to use the list. With queued
input no suspension occurs, which is why the two paths disagree.

## What I want

1. **The defect as a red test first.** Reproduce it before repairing it, and
   record the pre-fix value verbatim. After the fix the suspended-and-resumed
   path must produce the same answer as the queued-input path, and both must
   be the answer `StringReplace` is documented to give. A test that passes
   because nothing suspended is not a test — assert that the suspension
   actually happened.

2. **Continuation frames for the option loop.** C01's loop state — the target
   symbol, the explicit options, the position reached, and the prefix already
   evaluated — must survive a suspension and resume into the loop, not past
   it. Frames derive `FrameRootTrace`; the accumulated prefix is a root, and a
   missing traversal is a compile error by design. Preserve the existing
   suspend/resume/abandon contract.

3. **The continuation must reach the consumer.** This is the part brief-529
   stopped at: resuming the option loop yields the option list, and the
   consumer still has to run with it. Design that, and state which consumers
   are covered. There are 18 call sites of the public wrapper across 7 files
   in `crates/rum-stdlib/src/`. If your design covers some and not others,
   **say exactly which remain broken and why** — a partial repair that is
   documented is acceptable here; one that is silent is not.

4. **A bounded inventory of the class, as discovery, not repair.** The option
   path is one instance of a shape: a builtin whose handler drives the
   evaluator through a helper and propagates with `?`, installing no frame of
   its own, so a suspension completes the call with the inner value. Find the
   other instances and list them with `path:line` and the shape of the loss.
   **Do not fix them here.** If the inventory is large, that is a finding I
   need, not a reason to widen this brief.

5. **The behaviour that must not change**: cancellation, abandonment, the
   memory ceiling's existing readings, and the answers every currently green
   test asserts.

## Scope

**This section states an objective and a fence, not an enumeration.** A MAY
list has been short six times on this project. The enumeration is guidance;
it is not the boundary.

**The objective.** Inside `crates/rum-runtime/` and `crates/rum-stdlib/`,
change what the Goal requires — including files not named here.

**The disclosure duty, which replaces asking permission.** Every file you
touch that the guidance does not name must appear in your report's first
section with its `path:line` ranges and one line on why the objective required
it. **An undisclosed edit is the violation; a disclosed necessary edit is
not.** The director diffs the tree against your disclosure list.

**Where the work is expected to live** (guidance):

- `crates/rum-runtime/src/runtime/options/core.rs` and `options.rs`.
- `crates/rum-runtime/src/runtime/suspension/` and the frame definitions,
  plus whatever `FrameRootTrace` and the frame inventory require.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`, if the dispatch
  frame must stop swallowing a handler's continuation.
- The consumer call sites in `crates/rum-stdlib/src/`.
- Owning tests in `crates/rum-runtime/src/runtime/tests/` and
  `crates/rum-stdlib/src/tests/`.
- `docs/guide/` only if a documented behaviour changes; name the file in the
  report.

**The fence — STOP conditions:**

- **`crates/rum-core/`**, in particular `expr.rs`. Representation is fixed.
- All `Cargo.toml` files and `Cargo.lock`. No new dependency; the pin is
  stable 1.85.0.
- The interner, gensym, and the snapshot format.
- **No leases, no `R` term, no admission change.** That is D2b. If you find
  yourself adding accounting, you have left this brief.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`.

## rum / Rust pitfalls (read twice)

1. **A resumed frame must not assume uniqueness.** `Arc::try_unwrap`
   demanding a unique owner is a shape that has panicked this kernel before;
   a suspension frame legitimately breaks it.
2. **A lease-free prefix is still a root.** The accumulated option values are
   live `Expr`s; a frame holding them must traverse them or the derive fails
   the build, which is the point.
3. **Do not collapse a `suspend_with_frame*` into an eager call** to make the
   ownership story simpler.
4. **The checker is in the tree and runs in the gate.** Eight
   `retention_closure` tests must stay green. If your change makes a
   previously resolved edge unresolved, that shows up there — report it, do
   not silence it.
5. **A certificate must force the transformation it names.** Assert the exact
   resumed result, and assert that a suspension occurred; a `Length` or a
   `True` that is correct when nothing ran is the trap this project has hit
   twelve times.
6. **Queued input does not suspend.** Any control that relies on queued input
   is testing the non-suspending path; you need both.

## Verification (MANDATORY — you run it; iterate to green)

Director measurements on this host, warm tree, taken immediately before
dispatch: `cargo test -p rum-stdlib` green, eight checker tests in **8.2 s**,
`cargo test --workspace` green in **84 s** with no failures. The tree is green
at your starting point; any red is yours.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1200 -- cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 900  -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Nine native ZeroMQ tests cannot bind loopback in your sandbox. Report them as
skipped-by-environment; the director settles them. **Do not** set
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP` or otherwise mask them.

**Measurement.** You are touching the option path and possibly the dispatch
frame, so the gate applies: **allocation counts first**, via
`scripts/core-memory-microbench.sh`; this host's cores are 1.27x apart and one
timing pair cannot gate the panel. If the counters do not move, do not quote a
time. If they do, run `scripts/core-language-microbench.sh` base and candidate
at the same binary path, four passes each, profiling env off, normalized
against **named untouched labels** — never the suite median — and report the
option-consuming labels with their same-tier neighbors.

## Hard rules

- **No `git`, in any form.**
- Do NOT commit, branch, push, or touch git config. The director gates and
  commits.
- Respect the fence. Outside it, the rule is the disclosure duty.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- No default-path cost for off-by-default machinery.
- No `#[ignore]`, no feature gate on a required test, no test weakened to get
  green, no answer changed to match a broken implementation.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behaviour only.
- Scratch goes under `target/brief-530/`.
- **Stop-loss, keyed on progress.** Report two numbers after each attempt:
  consumers whose resumed answer matches their queued-input answer, and
  required gates passing. Stop only when **two consecutive** attempts move
  neither. A compile error you are actively reducing is not a wall.

## Report format (your final message)

1. What changed — per file with line ranges, including every file the
   guidance did not name, with why.
2. The defect: the red test before the repair with its verbatim pre-fix
   value, and the green one after.
3. The continuation design: what each frame carries, how it resumes into the
   loop and then into the consumer, and which consumers are covered.
4. The class inventory — other instances of the shape, `path:line`, not
   fixed.
5. Verification — every command with verbatim output.
6. Measurement — allocation counts first; timing only if the counters moved.
7. Stop-loss / open questions.
