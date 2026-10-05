# brief-568 — G3: a flat, resumable schema validator

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

`ConformSchema` and `SchemaErrors` (`crates/rum-stdlib/src/schema.rs`) resume
**past** their work when a predicate or a default asks for input, at seven
sites — P2056, P2061, P2065, P2066, P2067, P2068, P2069 — and their validator
walks the value and schema by native recursion, which aborts the process on
the ruled 2 MiB stack (release, one codegen unit: a literal list schema aborts
at 6 000 levels, an association at 5 000; `.temp/brief-567/depth-runs.jsonl`).
Replace the recursion with brief-567's design: one heap-held depth-first work
stack, one mutable path buffer, one suspension frame. Lost continuations
**66 -> 59**.

brief-567's report and `.temp/brief-567/draft-G3-implementation.md` are the
raw material; **this brief governs** where they differ (the depth boundary,
the depth tests, the performance gate).

## Director rulings on brief-567's questions

1. **The depth contract covers the walk the validator owns**: the validation
   loop, path building and rendering, and cloning, tracing, dropping and
   abandoning the validator's own state types. The `Expr`-depth helpers
   brief-567 found outside the module — `Expr`'s destructor, the memory and
   gensym root visitors' recursive `Expr` walks, association-data flattening,
   `Expr`'s `Display` — are existing recorded debt, not G3's; they are not
   touched here and are recorded for the hardening line. A deep test that
   aborts in one of them rather than in the validator is a **STOP with
   evidence**, never a pass, and never a reason to lower n silently.
2. **Performance**: the standard gate, one codegen unit on both arms; a
   residual is reported with its attribution for a director decision.
3. **Inventory**: brief-567's plan (`.temp/brief-567/inventory-plan.md`) is
   authorized exactly as written — listed under Scope.

## Context pointers

- brief-567 and its report; `.temp/brief-567/design.md` (types, loop,
  semantics to preserve), `inventory-plan.md`, `inventory-before.tsv`,
  `perf-plan.md`, `verification.md`, `probe.rs` (the red, control and deep
  tests), `post-fix-depth/` (release programs), `depth-runs.jsonl`.
- The accepted G2 pattern: `crates/rum-stdlib/src/lazy/continuation.rs` (cold
  capture on the error edge, immutable Session-rooted frame, cold working copy
  on an actual resume), `tests/association_depth_contract/lazy_depth_contract.rs`
  (the structural depth checks and falsifiers), `tests/lazy_pull_continuations.rs`.
- `docs/guide/structured-data.md:555-610`; `docs/dev/codex/TEMPLATE.md`
  (standing duty; pitfall 8).

## Scope

**Objective:** the flat validator, its suspension frame and finisher
completion, the structural and empirical depth checks, the owning tests, one
narrow guide paragraph, and the standing and performance gates.

**Edit surface:** `crates/rum-stdlib/src/schema.rs` and new children
`schema/{state,parse,format,association,machine,continuation}.rs` (design.md's
layout; split further only at a named responsibility boundary); new
`crates/rum-stdlib/tests/schema_continuations.rs` (and children by
responsibility, e.g. a depth child); new
`crates/rum-stdlib/src/tests/association_depth_contract/schema_depth_contract.rs`
and its `mod` line; `docs/guide/structured-data.md`, one paragraph saying a
schema callback or default that asks for input resumes into the validation —
no claim about depth beyond the validator's own walk, none about the matcher.
**`scripts/core-language-microbench.sh` stays unchanged**; the schema labels
live as scratch workloads.

**Inventory exceptions, authorized by name** (`inventory-plan.md`): the 41
schema rows mapped as written — the four old data types (`SchemaOptions`,
`SchemaError`, `SchemaEntry`, `SchemaPath`) gain `FrameRootTrace`, the other
data derives and the named captures and macros relocate, the two existing
input frames keep their identity, the key-path `format!` at `schema.rs:363:27`
is removed and the index-path `format!` at `:370:23` becomes a `write!` row in
`schema/format.rs`; the new `FrameRootTrace` types (`SchemaMode`,
`SchemaState`, `SchemaContainer`, `ListWork`, `AssociationWork`,
`SchemaDestination`, `SchemaAwait`, `SchemaValidationSuspensionFrame`) and the
frame's `PureFrameAbandon`; rum-runtime's lists get exactly the 12 root-trace
entries and one abandon entry, and the named-frame count goes **555 -> 556**.
`Unsplit:ConformSchema` and `Unsplit:SchemaErrors` keep class, status, owner
and exclusions; only their evidence sites relocate.

**Standing maintenance duty — always in scope** (`TEMPLATE.md`): replay the 68
fingerprints and both certificates, reconcile every ownership inventory from
discovery, disclose every changed row. **STOP** on any other existing row
change or disappearance, any new `UnknownHostPayload`, any self-reaching
schema type, any new row in untouched code, or any retention reclassification.

**Fence:** no rum-runtime or rum-core production change, no matcher change, no
`Expr`, serde or public representation change, no other suspension group, no
dependency, no lease. No edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`,
earlier briefs or other docs.

**Disclosure duty:** every changed file and moved item, every frame and state
field with its roots, every changed inventory row, every measured label, and
any adjacent defect.

## Deliverable

As design.md specifies, with every observable semantic preserved — output
shape and order, error order, paths and messages, Optional laziness, defaults
evaluated only when the key is missing and before their validation, strict
extra keys after the specified ones, malformed-schema and `Eval` abort
precedence, full traversal under the 64-error presentation cap,
`MatchQ`'s True-only interpretation with the original leaf value kept. One
mutable path buffer with byte-length checkpoints replaces the per-level path
copies (an O(n^2) cost today). Capture only on `InputRequired`, on the error
edge; no continuation allocation or clone on a run that never suspends; a cold
working copy on an actual `Ok` resume; a second suspension freezes the new
state; abandonment runs no skipped validation, default or message. Both
finishers complete through one shared helper on the ordinary and resumed paths.

## Tests

Red first, recorded on the incoming tree before any production edit: brief-567's
scratch suite (`.temp/brief-567/probe.rs`) moved into
`schema_continuations.rs` — the nine unchanged brief-548 G3 witnesses and the
eleven further semantic cases through the rum-vs-rum oracle (value, `log`,
errors and termination, messages, prints, actual suspension, exact prompts,
pending request identity, no owner after, a working `1 + 1`), the
cancellation and control cases, and the structural checks. Pre-fix: 20
semantic reds, one structural red, the rest controls.

**Structural depth checks** in `schema_depth_contract.rs`, each with a
falsifier that is seen to fire: the recursion scanner over every schema source
file individually and as one combined unit reports no recursive edge (direct
and cross-file falsifiers); no `rum_stdlib::schema::` row in
`ownership_graph/inventory/self_reaching.tsv` (injected-row falsifier); a
root-completeness test visits every `Expr`-bearing field of the state and
frame.

**Depth tests — run controls first, never lower n silently:**

- in-process, on a `2 * 1024 * 1024` thread that also owns the Session's
  teardown: construction controls at **list 1 000 / association 500** (they
  pass today), then `ConformSchema` exact result, `SchemaErrors` empty result,
  and `SchemaErrors` on a terminal scalar mismatch with the exact full path,
  for both container kinds; plus a deep suspension, a second suspension and an
  abandonment at the same depths. Assert every level and the leaf; never
  format a deep `Expr` in an assertion message;
- release, through the candidate one-unit `rum-cli` under
  `bash -c 'ulimit -s 2048; ulimit -c 0; …'`, stdin `/dev/null`, each program
  run alone: construction controls, then the same three validations at **list
  6 000 / association 5 000** — the depths at which construction passes today
  and the validator aborts (brief-567's `post-fix-depth/` programs, with n set
  to these depths). Do not run any validation pre-fix; the pre-fix evidence is
  complete.

Every existing test stays as it is and green.

## rum / Rust pitfalls (read twice)

1. An ordinary error is not `InputRequired` and creates no frame.
2. Share `Expr`/`SharedArgs`; the state owns its buffers, not a borrowed view
   of itself; no stage or container reference survives an evaluation.
3. One change per measured candidate.
4. Stop-loss is keyed on progress: failing tests, remaining Lost sites,
   regressing labels, the worst regression. Stop after two consecutive
   compiling candidates that move none of them. Falling compile-error counts
   are progress before the tree compiles.
5. No recursion as a fallback, no depth cap, no fail-closed result counted as
   a depth pass.
6. **Both performance arms use one codegen unit** (`TEMPLATE.md` pitfall 8).

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-568/tmp …`. Bounds from measured serial history: a
focused test target 150 s; `ownership_graph`, `local_production_acceptance`
(either feature set) and `cargo test -p rum-runtime` 300 s; `cargo test
--workspace --no-fail-fast` 2 400 s (observed 651-1 012 s); a one-unit release
build 600 s (observed 98-105 s), checking the binary's hash changed; a release
depth program 60 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test schema_continuations
cargo test -p rum-stdlib --lib schema_depth_contract
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test -p rum-runtime
cargo test --workspace --no-fail-fast
```

plus the release depth programs, each alone. Report native ZeroMQ skips as
skips.

**Performance gate.** Base: `.temp/brief-568/base/rum-cli` and
`rum-cli-alloc` — the incoming tree built with one codegen unit by the
director (hashes in `.temp/brief-568/base/sha256.txt`); do not rebuild them.
Candidates: the working tree built with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`
into a fresh target directory each, default and `--features
rum-cli/alloc-count`. Panel: the 18 schema labels of
`.temp/brief-567/perf-plan.md` as scratch workloads (`Do[<exact statement>,
{i, 1, N}]` at N and 2N, setup outside the loop, each workload's result checked
first), plus the neighbours it names (`list-literal-small`,
`association-small-literal-repeat`, `matchq-integer-success`,
`matchq-integer-fail`, `table-small`, `table-iterator-null`) and the controls
`dot-three-no-input`, `differences-no-input`, `take-multispec-small` from
`.temp/brief-563/perf/`. Allocation counts first, then two paired samples of
`taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one label per
process, stdin `/dev/null`, `(count(2N) - count(N)) / N`, raw output saved
before parsing. A label regresses when **both** samples exceed +0.05%. Report
every row. No timing suite. At most six measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-568/`. After each milestone update
`.temp/brief-568/PROGRESS.md`: what is done and verified, the exact next
action, whether the tree compiles, half-edited files, the counters, and any
depth or dependency blocker.

## Hard rules

- **No `git`, in any form.**
- No network; nothing outside this repository; all scratch, `.rum` programs,
  `TMPDIR` and targets under `.temp/brief-568/`.
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli`, `taskset`/`perf`, `valgrind --tool=callgrind`.
- Scope is the objective, the fence and the disclosure duty; STOP and report
  before editing past the fence.
- **Testing Policy is a hard line** (`AGENTS.md`).
- No new `unsafe`; never store a `Session` borrow, the cancellation observer
  or a `NativeStackDepth` in a frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Files, items and fields changed or moved, roots, module sizes.
2. Verbatim red and green output: the oracle suite, cancellation and controls,
   structural checks and falsifiers, in-process and release depth (exact
   results, exit statuses), the broad suites; every inventory row change.
3. Lost continuations before and after (66 -> 59 predicted).
4. Every performance row for base and every candidate, both samples.
5. Stop-loss counters, residuals with attribution, open questions.

---
