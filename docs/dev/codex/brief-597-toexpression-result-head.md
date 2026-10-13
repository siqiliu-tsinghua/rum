# brief-597 — G12: `ToExpression` resumes into its result head, and the fieldless-frame sweep

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

The E1 oracle's seventh drain (brief-596) found group G12. `ToExpression[str,
form, head]` evaluates the parsed source, then evaluates `head[value]`
(`crates/rum-stdlib/src/strings/text/builtins.rs:460-479`). When the parsed
source asks for input, the fieldless `EvalToExpressionSuspensionFrame1`
(`strings/text/builtins/resume_frames.rs:6-21`) completes with the resumed
value and the result head is never applied: the resumed path returns `7`
where the queued path returns `107`
(`.temp/brief-596/defects/ToExpression.parsedSource.md`, with the exact
`Middle`, `Twice`, `AlreadyResumed` and `Nested` witnesses).

1. **Repair.** The first frame carries the result head (captured only on
   suspension) and, on resume, enters the same completion helper as the eager
   path: apply the head once (its own suspension still carried by the second
   frame), or return the value when there is none. Attach the
   `ToExpression.parsedSource` descriptor (owed 39 -> 38).
2. **The class.** This is a frame with no fields whose function still has work
   after the `?` — the census classified it as a correct tail. Sweep the whole
   repository (`rum-runtime`, `rum-stdlib`, and any crate using
   `resume_or_suspend!` or inserting a frame): list every site whose suspension
   frame is a unit struct or carries less state than the work that follows it
   in the eager path, with `path:line`, the work that follows, and a verdict
   (correct tail / **lost suffix**). Each lost suffix gets an oracle witness
   (resumed against queued) in the scratch test and is **reported, not
   repaired** here — it becomes a group of this branch. A site is a correct
   tail only when nothing runs after the suspension point in the eager path.

## Context pointers

- brief-596's report and `.temp/brief-596/defects/ToExpression.parsedSource.md`.
- brief-548's census and its `fieldless-frames.tsv`; brief-583's
  `reviewed-census.tsv` (which classified this site); the accepted
  continuation pattern of briefs 559-595.
- The oracle `crates/rum-stdlib/tests/suspension_oracle/` (its
  `fixtures/strings_roles/` descriptor for this entry).

## Scope

**Edit surface:** `crates/rum-stdlib/src/strings/text/builtins.rs` (only
`ToExpression` and a shared completion helper) and
`strings/text/builtins/resume_frames.rs`; the oracle's
`fixtures/strings_roles/` and `inventory/{effects,owed}.tsv` — attach exactly
`ToExpression.parsedSource` and delete exactly its owed row after its gates pass
in both builds; rum-runtime's frame lists only if a frame type is added (a
field added to an existing frame needs none). Scratch: `.temp/brief-597/`, with
one scratch integration test `crates/rum-stdlib/tests/zz_brief597_sweep.rs` for
the sweep's witnesses, **deleted before the report**, its content copied to
`.temp/brief-597/sweep.rs`.

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints, both
certificates, ownership reconciliation, every row disclosed. **STOP** on any
existing row change or disappearance, any new `UnknownHostPayload` or
`self_reaching` row, any new row in untouched code.

**Fence:** no other production change; a lost suffix found by the sweep is a
report, not a repair; no leases; `scripts/` unchanged. No edits to
`AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier briefs or docs.

**Disclosure duty:** every changed function and field, every inventory row,
every measured label, every sweep row.

## Tests

Red first, recorded before any production edit: brief-596's four witnesses
through the oracle. Green: the full `ToExpression.parsedSource` descriptor
(`Middle`, `Twice`, `Error`, `AlreadyResumed`, `Nested`), lifecycle at the
first and second pause, both builds; plus a head that itself suspends after a
suspending source, and no head.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. The head is applied exactly once, on whichever path completes.
3. Stop after two consecutive compiling candidates that move none of: reds
   turned green, sweep sites classified, regressing labels.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-597/tmp …`. Bounds: focused 300 s; the oracle 900 s
per build; runtime, ownership and retention 300 s; a one-unit release build
900 s; a metric process 150 s; `cargo test --workspace --no-fail-fast` 3 600 s.

```
cargo fmt --check
scripts/check-suspension-oracle.sh 900
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Instruction gate.** Before any production edit, build the base from the
repository tree with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` into
`.temp/brief-597/base/target-default` and `target-alloc` (`--features
rum-cli/alloc-count`, two arguments); candidates the same way in fresh
directories. Panel (scratch workloads, each result checked first):
`ToExpression` without a head, with a head, and a small arithmetic source; the
controls `dot-three-no-input`, `differences-no-input`, `take-multispec-small`
(`.temp/brief-563/perf/`). Noise floor first (base against itself, two pairs);
a label regresses when both candidate samples exceed `max(+0.05%, floor)`.
Allocation counts, then two paired samples of `taskset -c 2 perf stat -x, -e
cpu_core/instructions/u`, one label per process, stdin `/dev/null`,
`(count(2N) - count(N)) / N`, raw output first. No timing suite. At most four
measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-597/`; `PROGRESS.md` after the red capture, each
compiling candidate, the sweep and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the oracle gate script, the release `rum-cli` on passing programs,
  `taskset`/`perf`, `valgrind --tool=callgrind`. No probe that aborts a
  process.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files, functions and fields.
2. Verbatim red and green output, the oracle in both builds, ownership,
   certificates and the broad suites; every inventory row change.
3. The sweep table, every lost suffix with its witness; the noise floors and
   every measured row.
4. Stop-loss counters, open questions.

---
