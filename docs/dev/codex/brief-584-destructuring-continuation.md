# brief-584 — G9: list-destructuring assignment resumes into its remaining stores

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-583's re-census found group G9: a list-destructuring `Set` walks its
left-hand side recursively, storing leaf by leaf, and a leaf whose target asks
for input resumes **past** the rest — the remaining sibling and nested stores
are skipped and the result is the leaf's value instead of the complete
right-hand side:

```
log = {}; a = {0}; b = 0;
{a[[(AppendTo[log, 1]; InputString["p: "]; 1)]], b} = {7, 9}
```

resumes to `7` where the queued path returns `List[7, 9]` (and `b` stays `0`).
With two suspending targets
(`r = ({a[[(…"p: "…; 1)]], a[[(…"q: "…; 2)]], b} = {7, 8, 9}); {r, a, b}`),
the resumed path reaches only the first prompt and returns
`List[7, List[7, 0], 0]`, log `{1}`; the queued path returns
`List[List[7, 8, 9], List[7, 8], 9]`, log `{1, 2}`. Three obligations in one
mechanism (`crates/rum-runtime/src/runtime/definitions/definitions.rs:467`,
`:480`, `:485` — the full-RHS return, the sibling cursor, the leaf
completion): lost continuations **4 -> 1** by the re-census.

Replace the recursive store continuation with an owned cursor and parent
stack, keeping the original right-hand side as the final result: resume the
pending leaf once, perform the remaining stores in order, return the complete
evaluated right-hand side. Shape validation still runs before any store;
completed stores stay committed; ordinary errors propagate unchanged.

brief-583's report and `.temp/brief-583/draft-G9.md` are the raw material;
**this brief governs** where they differ.

## Context pointers

- brief-583 and its report; `.temp/brief-583/{draft-G9.md,new-lost.tsv,new-probes.json,new-probes.tsv,reviewed-census.tsv}`.
- `definitions/definitions.rs` (the destructuring entry and the lvalue
  dispatcher at `:312`), `finish_definition_assignment`, the Part assignment
  continuations; `docs/guide/evaluation.md:116-176`, `:286-295`; the generated
  `Set` reference.
- The accepted continuation and flat-walk patterns of briefs 563, 568, 572-582;
  their structural depth contracts
  (`tests/association_depth_contract/*_contract.rs`).

## Scope

**Objective:** the resumable destructuring driver, its frame, its tests, and
the standing and performance gates.

**Edit surface:** `definitions/definitions.rs` (only the destructuring entry,
the driver and module wiring); new `definitions/definitions/destructuring.rs`
(and `destructuring/resume_frames.rs` if size requires); new
`crates/rum-stdlib/tests/destructuring_continuations.rs` and its support
children (reuse the existing oracle helpers without weakening them); a
structural depth contract for the new module beside the existing ones under
`tests/association_depth_contract/` with its `mod` line;
`docs/guide/evaluation.md`, only to state the existing store order, the
full-RHS result and the suspension behaviour; rum-runtime's frame lists
(exactly the new types).

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints, both
certificates, ownership reconciliation, every row disclosed. **STOP** on any
existing row change or disappearance, any new `UnknownHostPayload` or
`self_reaching` row, any new row in untouched code.

**Fence:** no change to `Expr`, definition precedence, Part selector
semantics, other assignment paths, public host APIs, manifests or the
lockfile; no leases; **`scripts/core-language-microbench.sh` stays
unchanged** (labels are scratch workloads). No edits to `AGENTS.md`, `TODO.md`,
root `PROGRESS.md`, earlier briefs or other docs.

**Disclosure duty:** every changed function, state field and root, inventory
row, measured label, adjacent defect.

## Deliverable

An owned traversal — cursor plus parent stack, no recursion — with the
original right-hand side retained separately. Once a leaf is selected, its
pending completion is recorded before `finish_definition_assignment` runs; on
`InputRequired` (on the error edge only) the remaining cursor and the original
right-hand side freeze in a rooted frame; on resume the child's outcome is
consumed once and the same next-leaf helper continues. A second suspension
freezes the new state once; pure abandonment drops pending work and leaves
committed stores intact (no new rollback semantics). No `Session` borrow or
borrowed iterator crosses a suspension; no deep owned-tree clone; no frame
allocation on an ordinary symbol `Set`.

## Tests

Red first, recorded before any production edit: the two witnesses above, the
nested witness in `new-probes.json`, a completed store before the first pause,
a pause in the middle and in the last leaf, an ordinary error in a later
target after a resume, a shape mismatch (validated before any store),
cancellation at a suspended target, answer-free abandonment, and a fresh
`1 + 1` after — through the rum-vs-rum oracle (value, error, termination,
ordered messages and prints, `log`, the stored variables, actual suspension,
exact prompts). The structural contract for the new module: the recursion
scanner per file and combined, no self-reaching state, root completeness, each
with a falsifier. Every earlier group's suite stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. The result is the complete right-hand side, not the last leaf.
3. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-584/tmp …`. Bounds: focused 150 s; runtime,
ownership and retention 300 s; a one-unit release build 900 s; a metric
process 150 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test destructuring_continuations
cargo test -p rum-stdlib --lib association_depth_contract
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Performance gate.** Base: `.temp/brief-584/base/rum-cli` and
`rum-cli-alloc` — brief-582's selected one-unit candidate, whose production
source is the incoming tree (hashes in `sha256.txt`); do not rebuild them.
Candidates with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and
`--features rum-cli/alloc-count` (two arguments), fresh target directory
each, hash checked. Panel (scratch workloads, each result checked first):
destructuring of symbols (`{x, y} = {1, 2}`), of held parts
(`{a[[1]], b} = {7, 9}` with setup), nested destructuring, an ordinary
`x = 1`, a part assignment, and the controls `dot-three-no-input`,
`differences-no-input`, `take-multispec-small` (`.temp/brief-563/perf/`).
**Noise floor first** (base against itself, two pairs); a label regresses
when both candidate samples exceed `max(+0.05%, floor)`. Allocation counts,
then two paired samples of `taskset -c 2 perf stat -x, -e
cpu_core/instructions/u`, one label per process, stdin `/dev/null`,
`(count(2N) - count(N)) / N`, raw output first. No timing suite. At most eight
measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-584/`; `PROGRESS.md` after the red capture, each
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
2. Verbatim red and green output, the structural contract and falsifiers,
   ownership and certificate results, the broad suites; every inventory row
   change.
3. Three obligations repaired (4 -> 1 by the re-census); the noise floors and
   every measured row.
4. Stop-loss counters, residuals, open questions.

---
