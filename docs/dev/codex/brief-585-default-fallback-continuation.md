# brief-585 — G10: a suspended direct `Default` lookup continues into the ordinary scan

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-583's re-census found group G10, the last Lost site: in
`try_apply_down_values`
(`crates/rum-runtime/src/runtime/definitions/application/down_values.rs:62-79`),
a call whose head is `Default` first tries the direct default lookup; if that
returns `None` the eager path continues into the ordinary down-value scan. When
a guard inside the direct lookup asks for input, the callee's own frame resumes
the lookup, but nothing holds the caller's decision — *`Some` returns, `None`
continues* — so a resumed decline ends the call instead of continuing:

```
log = {}; n = 0;
Default[d] := 7 /; (n = n + 1; AppendTo[log, n]; If[n <= 2, InputString["p: "]]; n >= 3);
Default[d]
```

With answers `x`, `y` both paths reach two prompts; the resumed path returns
`Default[d]` with log `{1, 2}`, the queued path `7` with log `{1, 2, 3}`.
Lost continuations **1 -> 0** by the re-census.

Hold the caller's decision in a frame and enter **one** ordinary-fallback
helper shared by the eager and the resumed path: `Some` completes, an ordinary
error forwards, `None` continues into the normal scan — exactly once, never
restarting the direct lookup or replaying a declined guard.

brief-583's report and `.temp/brief-583/draft-G10.md` are the raw material;
**this brief governs** where they differ.

## Context pointers

- brief-583 and its report; `.temp/brief-583/{draft-G10.md,new-lost.tsv,new-probes.json,new-probes.tsv}` (P0226).
- `definitions/application/down_values.rs:16-145`, `default_values.rs:15-258`,
  `eval_core/call.rs:291-340`, the `Option<Expr>` continuation codec in
  `suspension/primitive.rs`; `docs/guide/patterns-and-rules.md:165-184`, the
  guide's Suspension section.
- The accepted continuation pattern of briefs 559-584 (cold capture on the
  error edge, immutable frame, working copy only on resume).

## Scope

**Objective:** the caller's completion phase, its frame, the tests, and the
standing and performance gates.

**Edit surface:** `down_values.rs` (only the split into a prelude and the
shared fallback helper, and module wiring); a new responsibility submodule
`down_values/default_fallback.rs` (a `resume_frames.rs` child if size
requires); `default_values.rs` only for a narrowly required shared completion
type or helper; new `crates/rum-stdlib/tests/default_fallback_continuations.rs`
and its support children (reuse the existing oracle helpers without weakening
them); `docs/guide/patterns-and-rules.md`, only to state the existing
direct-`Default` guard-decline continuation; rum-runtime's frame lists
(exactly the new types).

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints, both
certificates, ownership reconciliation, every row disclosed. **STOP** on any
existing row change or disappearance, any new `UnknownHostPayload` or
`self_reaching` row, any new row in untouched code.

**Fence:** exact- and implicit-default lookup order and snapshots, definition
precedence, the candidate cache, the matcher, `Expr`, host APIs, catalog
metadata, manifests and the lockfile unchanged; **no refactor of the down-value
dispatcher beyond the split**; no leases; `scripts/core-language-microbench.sh`
unchanged. No edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier
briefs or other docs.

**Disclosure duty:** every changed function, state field and root, inventory
row, measured label, adjacent defect — including any semantic conflict a live
definition change in a guard exposes (report it; do not change precedence).

## Deliverable

The eager path keeps its behaviour: the prelude computes what it computes today
(including the `down_values_cannot_match` decision) and the shared helper
performs the ordinary scan, re-reading live definitions exactly as the eager
path already does after the direct lookup runs guest code. On `InputRequired`
from the direct lookup (on the error edge only), a cold helper freezes the
call's head, shared arguments, attributes, symbol and the prelude's decision —
no borrowed `Definition`, no candidate built or cloned before the cold branch.
On resume the child's raw outcome is decoded explicitly as `Option<Expr>`:
`Some` completes, `None` enters the same helper once, an error forwards. A
second suspension inside the fallback is carried by its own existing frames.
Pure abandonment runs no later guard, fallback or body.

## Tests

Red first, recorded before any production edit, through the rum-vs-rum oracle
(value, error, termination, ordered messages and prints, `log`, actual
suspension, exact prompts, no owner after, fresh `1 + 1`): the witness above;
its accepting control (`n == 1` / `n >= 2`, `7` and `{1, 2}` on both paths);
brief-583's optional-default declining control; a terminal `None` (no other
candidate); a fallback candidate that is first and one that is in the middle;
unguarded hit and miss; a nested enclosing caller; two pauses with distinct
logs; an ordinary error and a guard side effect that finishes normally;
cancellation and reset at both pauses. Every earlier group's suite stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. The callee's frame does not complete its caller; `None` is a live resumed
   outcome here, not a preflight result.
3. `try_apply_down_values` is on the hottest path of every call: the split must
   not move work onto it. Capture state only in the cold branch.
4. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-585/tmp …`. Bounds: focused 150 s; runtime,
ownership and retention 300 s; a one-unit release build 900 s; a metric
process 150 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test default_fallback_continuations
cargo test -p rum-stdlib --no-default-features --test default_fallback_continuations
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Performance gate.** Base: `.temp/brief-585/base/rum-cli` and
`rum-cli-alloc` — brief-584's selected one-unit candidate, whose production
source is the incoming tree (hashes in `sha256.txt`); do not rebuild them.
Candidates with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and
`--features rum-cli/alloc-count` (two arguments), fresh target directory
each, hash checked. Panel (scratch workloads, each result checked first):
direct `Default[d]` unguarded hit and miss, a guarded direct `Default` that
declines into a fallback candidate, the ordinary down-value hit, miss and
guarded hit of an ordinary head (`f[1]`), an optional-pattern default consumer
(`g[x_, y_.]`), an own-value read, and the controls `dot-three-no-input`,
`differences-no-input`, `take-multispec-small` (`.temp/brief-563/perf/`).
**Noise floor first** (base against itself, two pairs); a label regresses when
both candidate samples exceed `max(+0.05%, floor)`. Allocation counts, then two
paired samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one
label per process, stdin `/dev/null`, `(count(2N) - count(N)) / N`, raw output
first. No timing suite. At most eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-585/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli` on passing programs, `taskset`/`perf`,
  `valgrind --tool=callgrind`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files and functions, state fields and roots.
2. Verbatim red and green output, ownership and certificate results, the broad
   suites; every inventory row change.
3. The repaired obligation (1 -> 0 by the re-census); the noise floors and every
   measured row.
4. Stop-loss counters, residuals, open questions.

---
