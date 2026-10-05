# brief-569 — erase the schema validator's ordinary-path cost (G3)

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

brief-568 (`70108834`) made the schema validator a flat work stack that
resumes into its walk and cannot overflow the native stack. It is correct and
gated. But a validation that never suspends got dearer on small schemas.
Per-iteration instructions, both arms built with one codegen unit (base
`.temp/brief-568/base/`, candidate `.temp/brief-568/candidate-1/`), from
brief-568's report §4 (the director reproduced the first, second, fourth and
the control):

| Label | Delta, sample 1 / 2 |
| --- | --- |
| `schema-conform-association-defaults-small` | +3.15% / +3.25% (director: +3.27% / +3.27%) |
| `schema-errors-association-defaults-small` | +3.43% / +2.83% |
| `schema-conform-list-small` | +0.76% / +1.02% (director: +1.02% / +1.30%) |
| `schema-errors-list-small` | +0.55% / +0.47% |
| `schema-conform-list-depth-8` / `schema-errors-list-depth-8` | +0.85% / +0.82%; +0.85% / +0.89% |
| `schema-conform-association-index-33` / `-errors-` | +0.52% / +0.49%; +0.55% / +0.57% (director: +0.49%) |
| `schema-conform-association-depth-8` / `-errors-` | +0.26% / +0.27%; +0.35% / +0.35% |

Deeper schemas got faster (list depth 128 -5.2%), and every non-schema
neighbour and control sits at the base (`dot-three-no-input` +0.00%).
Allocation bytes per small validation rose (list-small 1 144 -> 1 960,
association-defaults 3 139 -> 3 977) while counts fell by one.

**Target:** every one of the 27 labels in `.temp/brief-568/perf/labels.json`
within +0.05% of the base (a label regresses when **both** samples exceed
+0.05%), with every test unchanged and green. If an honest effort leaves a
residual, attribute it to the specific work the flat walk must do that the
recursion did not.

## Fixed — not negotiable in this brief

- **The flat walk and its depth contract.** One heap-held work stack, no
  native recursion in proportion to depth; `schema_depth_contract.rs` (the
  recursion scanner per file and combined, no self-reaching schema type, root
  completeness) and the depth tests stay unchanged and green. **No regression
  is bought back by restoring recursion.**
- **Semantics.** `crates/rum-stdlib/tests/schema_continuations.rs` and its
  children, and every other test, stay exactly as they are.

## Where the director would look first — hypotheses, measure each

brief-568 §4's attribution: for the small list, schema self-cost rose 867 ->
1 534 instructions per validation, `Expr::clone` self-cost +168, partly offset
by less formatting; for the 33-key association, the owned `String`-keyed
`HashMap` replaced the old borrowed-key local index.

1. **Build what a flat, non-nested validation does not need lazily**: the work
   stack `Vec` and the path buffer for a schema with no nested container (a
   root container handled before the stack is ever pushed, or the stack and
   path allocated on first descent).
2. **Paths only on errors.** The path text is needed only when an error is
   recorded; a stack of segment references (key index or list index) rendered
   on the first error may cost less than maintaining text on every descent.
3. **Fewer `Expr` clones on the ordinary path.** Share `SharedArgs`/`Expr`
   handles where a work record now clones items it only reads.
4. **The association index without owned `String` keys** — e.g. indices into
   the value entries the state already owns, sorted by key and searched with
   borrowed comparisons. If this removes `AssociationWork.positions`, the
   `RandomState` row in `unknown_storage.tsv` that brief-568 admitted
   disappears: **that one removal is authorized**, from discovery, disclosed.

**Attribute before you change**: `valgrind --tool=callgrind` on base and
candidate, each total calibrated against `perf stat` on the same binary.

## Measurement

- **Base:** `.temp/brief-568/base/rum-cli` and `rum-cli-alloc` (hashes in
  `sha256.txt`); do not rebuild them.
- **Candidates:** the working tree built with
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
  rum-cli/alloc-count`, into a fresh target directory each under
  `.temp/brief-569/`; check the binary's hash changed; keep every measured
  binary, its hash and source. Measure the incoming tree once, the same way,
  as candidate 00.
- **Panel:** `.temp/brief-568/perf/` (27 labels, workloads and checks; check
  each result first). brief-568's and brief-566's scripts are a working
  harness to copy into `.temp/brief-569/`.
- **Method:** allocation counts first, then two paired samples of `taskset -c
  2 perf stat -x, -e cpu_core/instructions/u`, one label per process, stdin
  `/dev/null`, `(count(2N) - count(N)) / N`, raw output saved before parsing.
  Any candidate you keep, and the final one, runs all 27 labels. No timing.

## Scope

- **Objective:** the ordinary-path shape of `crates/rum-stdlib/src/schema.rs`
  and `crates/rum-stdlib/src/schema/`.
- **Standing duty — always in scope** (`TEMPLATE.md`): replay the
  fingerprints and both certificates, reconcile the ownership inventories,
  keep rum-runtime's frame lists exact. New `UnknownConstruct` rows for code
  you add or change are expected and disclosed; the one removal above is
  authorized; STOP on any other changed or vanished existing row, a new
  `UnknownHostPayload` or `self_reaching` row, or a row in code you did not
  touch.
- **Fence:** no semantic change and no test edit; no rum-runtime or rum-core
  production change; no matcher change; no other suspension group; no
  dependency, no `unsafe`, no depth cap, no recursion; the release profile in
  `Cargo.toml` unchanged. No edits to `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  earlier briefs or docs.
- **Disclosure duty:** every changed file with line ranges and why; every
  candidate — what it changed, its attribution, its rows.

## rum / Rust pitfalls (read twice)

1. **One variable per candidate.**
2. **A candidate that makes another label regress is rejected**, a deep label
   pushed above the base included.
3. **Instruction counts decide.**
4. **Stop-loss is keyed on progress:** the number of regressing labels or the
   largest remaining delta must fall. Stop after two consecutive compiling
   candidates that reduce neither; at most ten measured candidates.
5. A second ordinary code path kept beside the general one (a "flat-schema
   fast path") is a drift risk: it needs a falsifier proving both paths agree
   on every semantic case, or do not add it.
6. `#[inline(always)]` is not a fix by itself (brief-560: +1.17%).

## Verification (MANDATORY — you run it; iterate to green)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-569/tmp …`. Bounds: a focused test target 150 s; a
one-unit release build 600 s; a metric process 150 s;
`local_production_acceptance` and `cargo test -p rum-runtime` 300 s;
`cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test schema_continuations
cargo test -p rum-stdlib --lib schema_depth_contract
cargo test -p rum-runtime
cargo test --workspace --no-fail-fast
```

Plus the retention replay in both feature sets. Paste commands, bounds, exit
statuses and counts verbatim; list native ZeroMQ skips as skips.

## Checkpointing — protect the scene

Scratch under `.temp/brief-569/`. After each candidate write
`.temp/brief-569/PROGRESS.md`: its change, its attribution, its rows, the
counters, the exact next action and the tree's state.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository. No network. Commands: file reads and
  edits within scope, `rg`/`grep`, `python3`, bounded `cargo`, the release
  `rum-cli`, bounded `taskset`/`perf`, bounded `valgrind --tool=callgrind` with
  `callgrind_annotate`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges.
2. The attribution for each regressing family, base against incoming.
3. Every candidate measured, with its rows (all 27 for kept candidates), both
   samples.
4. Verification — every command's verbatim result.
5. Progress counters, disclosures, and the attribution of anything left.

---
