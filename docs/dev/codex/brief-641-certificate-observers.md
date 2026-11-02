# Brief 641 — certificate observers: force the Take tail and witness the natural drop

**Shape: IMPLEMENTATION, test-only. Scope: two ignored certificate tests in
`rum-runtime` and `rum-stdlib`. No production change, no new test, no
behaviour-test edit.**

## Goal

Close the two certificate gaps the Stage B campaign confirmed:

- **D03 / I31.** The Take depth certificate cannot force the transformation it
  names. Its fixture repeats `{1}` at every level, which is the identity on the
  nested singleton chain, so the no-op tail handoff at
  `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:320`
  (`apply_take_drop_specs_iterative`) is statically predicted to survive. Make
  the fixture select a *different* shape than it receives: a unique deepest
  sentinel must survive to the physical bottom, and at least one excluded
  sibling must sit at the transition into the iterative tail — inside the
  expression the tail receives after the bounded recursive front
  (`RECURSIVE_TAKE_DROP_SPEC_DEPTH = 16` applications) — so only the tail can
  remove it and the literal expected output differs from the literal input.
- **D01 / I07.** The natural-drop certificate proves completion at the 2 MiB
  boundary but does not witness what it drops. The test never asserts the
  lowered raw shape and never establishes that the lowered tree is uniquely
  owned, so "natural `Expr` drop" could be a refcount decrement on a shared
  DAG. Add a raw-shape and unique-ownership witness *before* the drop,
  walked iteratively so the witness itself costs no stack depth.

Both certificates keep their current names, `#[ignore]` attributes, release
profile, 2 MiB worker thread and physical height.

## Context pointers

- `docs/dev/codex/brief-639-test-audit-inventory.md` — the I31 row, the M31
  mutant row, the D01 and D03 entries in the obligation table.
- `docs/dev/codex/brief-640-stage-b-mutant-campaign.md` — gate C: M31 KILLED
  (a damaged reconstruction is observed) but the identity shortcut at line 320
  was never exercised; D01 open because I07 has no honest localized mutant.
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs` — `I31`:
  `take_multispec_completes_at_physical_height_4097_on_two_mib` and the
  helpers `evaluate_string_certificate`, `nested_input_form`,
  `evaluate_certificate_value_at_height` (the `x` chain builder).
- `crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs` — `:310-340`
  (`apply_take_specs`, the front/tail split at `:319-321`), `:455-520` (the
  iterative `Visit`/`FinishCall` tail).
- `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs` — `I07`:
  `left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary`, the
  seven-token loop, `repeated_chain_source`, `run_on_two_mib_thread`, and
  `lower_syntax_expr`.
- `crates/rum-core/src/expr.rs:573-739` — `SharedExpr`/`SharedArgs` are
  `Arc` wrappers; both expose `ptr_key()`. `Expr` is at `:749-753`.
- `docs/dev/codex/TEMPLATE.md` — the standing-inventory duty under Scope.

## Scope

- Files you MAY modify: **only**
  - `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`
  - `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs`
  A test-local helper needed by either witness may live in the same file.
- Files/areas you MUST NOT touch:
  - any production file, including `take_drop.rs`, `lookup.rs`,
    `traversal.rs`, `differentiation/`, `rum-core/src/expr.rs` and anything
    under `crates/rum-runtime/src/runtime/` other than the named test file;
  - any existing assertion's meaning in either certificate (a change that
    makes a certificate weaker, wider or different in claim is a STOP, not a
    repair);
  - any other test, any `#[ignore]` attribute, any feature, any inventory
    file by hand, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`,
    `Cargo.lock`.
- **Standing maintenance duty (always in scope).** The retention contract
  fingerprints hash every `.rs` under `crates/`, so these two test edits
  expire `inventory/local-contracts.tsv` and the certificate cells in
  `inventory/promoted.tsv`. Replay exactly as `TEMPLATE.md` describes —
  recompute the fingerprints, run `local_production_acceptance` in both the
  default and `--no-default-features` builds and copy the cells to
  `.temp/brief-641/certificates/`, reconcile the ownership rows against the
  discovery the tests print. Replay is the **last step of the edit stage,
  before the first acceptance suite**, and every changed row is disclosed. A
  red whose only cause is a stale derived cell is an ordering artifact, never
  a regression. STOP instead if a row committed at HEAD would change its
  identity, status, disposition or exclusion, or would disappear, or if a new
  `UnknownHostPayload` row appears, or if a new row points at code this brief
  did not touch. New `UnknownConstruct` rows for code this brief added are
  expected and are added with file and line disclosed.

## Deliverable

Stage 1 — **D03 / I31 fixture, then its falsifier.**

1. Change the `take_multispec` fixture so that:
   - the input still builds the 4096-deep `x` chain at physical height 4097
     and still runs on the 2 MiB thread;
   - a **unique deepest sentinel** (a value distinct from every other value)
     remains at the physical bottom of the selected path, and the expected
     output contains it exactly once at the expected depth;
   - at least one **excluded sibling** sits at the transition into the
     iterative tail — inside the expression the tail receives after the 16
     recursive spec applications — so the iterative tail is the only code
     that can drop it;
   - the literal expected output **differs from the literal input** in shape
     (not merely in length), and no identity shortcut at `take_drop.rs:320`
     can produce it.
   Keep the test name, `#[ignore]`, profile and the 2 MiB thread. State in the
   test's own comment what the fixture now forces.
2. Calibrate the falsifier in a scratch copy under `.temp/brief-641/`:
   copy the tree, apply exactly one production change — line 320 returns
   `Some(expr.clone())` instead of calling the iterative tail — and run the
   exact fixture in release on the 2 MiB thread. The fixture must be green in
   the untouched copy and red in the mutated copy at the fixture's own
   assertion, with the changed value matching the missing sibling.
   The mutation lives only in the copy; the authoritative tree stays clean.

Stage 2 — **D01 / I07 witness, then its negative control.**

3. In the same test, before `drop(lowered)`, walk the lowered expression
   **iteratively** and assert:
   - the raw shape: the expected nesting structure with a unique sentinel at
     the physical bottom (the same `path`-style observation the audit asked
     for), and the depth the fixture actually builds;
   - unique ownership: no `SharedExpr` or `SharedArgs` reachable from the
     lowered root is reached twice (collect `ptr_key()` values). A DAG would
     mean the natural drop decrements a shared refcount instead of destroying
     each node. If this assertion fails, **STOP and report** — it is a
     finding about the lowering, never a reason to weaken the witness.
   The witness must not grow the stack (iterative walk) and must not clone the
   tree.
4. Calibrate the negative control in a scratch copy: make one deep subtree
   reachable twice in the *test only* (a test-side clone inserted before the
   witness) and show the uniqueness assertion fires. The authoritative tree
   must be green with the witness in place.

Stage 3 — **replay, tests, disclosure.**

5. Replay the standing inventories as step 3 of Scope describes, then run the
   exact fixtures plus the workspace suite. Report every changed row.

## rum / Rust pitfalls (read twice)

1. The certificates run in release only: `--release`, the exact ignored test
   name, `--exact --ignored --nocapture`, inside the named 2 MiB thread. A
   debug run is not the certificate.
2. The witness must be worth what it asserts: a check that iterates the tree
   twice or allocates a copy is fine at 4096 nodes, a recursive walk is not —
   the whole point is the 2 MiB stack.
3. Do not confuse atom values with structure. `Symbol` atoms may be shared by
   interning; the uniqueness witness is about `SharedExpr`/`SharedArgs`
   structural reachability, not about symbol identity.
4. The expected string must be computed literally, not by re-running
   production selection: the fixture's job is to disagree with a broken
   selection.
5. If the excluded sibling cannot be placed without changing the certificate's
   claim (height, thread, release profile), STOP and say so; do not silently
   lower the depth.
6. A fixture that passes is not a calibrated fixture. The only calibration is
   the no-op mutation red in Stage 1 and the deliberate sharing red in Stage
   2, each quoted verbatim.
7. No new `unsafe`, no new dependency, no new feature, no default-path cost.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
# D03 fixture, authoritative tree, release, 2 MiB thread
cargo test --offline --locked --release -p rum-stdlib --lib -- \
  tests::iterative_traversal_depth::take_multispec_completes_at_physical_height_4097_on_two_mib \
  --exact --ignored --nocapture
# D01 witness, authoritative tree, release, 2 MiB thread
cargo test --offline --locked --release -p rum-runtime --lib -- \
  runtime::tests::left_chain_lowering::left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary \
  --exact --ignored --nocapture
# falsifier (scratch copy only): the same D03 command in the mutated copy, red
# negative control (scratch copy only): the same D01 command with the test-side
#   deliberate sharing, red at the uniqueness assertion
cargo test --workspace
```

The authoritative tree must have both certificates green and both scratch
copies must red at the named assertions. Paste the exact verdict lines, the
red's assertion text and the changed values. Bounds: derive each timeout from
a measured run of the same shape and use `scripts/run-bounded.py`; do not
guess.

## Checkpointing — protect the scene

All scratch under `.temp/brief-641/` (copies, logs, PROGRESS.md, report). After
each stage write `.temp/brief-641/PROGRESS.md`: done and verified with saved
evidence, the exact next action, the tree's state (compiling, which tests are
green, any half-edited file), and what the director needs to take over. Leave
the tree compiling at every checkpoint.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Leave the tree for
  the director's gate.
- No production edit, no test weakening, no assertion widening, no new
  `#[ignore]`, no new test function, no `unsafe`, no network, nothing outside
  this repo.
- The two scratch copies are evidence containers: they may differ from the
  tree only by the one authorized change each (the line-320 no-op, or the
  test-side deliberate sharing), and they are deleted after their receipts.
- Testing Policy is a hard line: rum's own documented behaviour only.
- Minimal diff: no drive-by formatting, renames or comments outside the two
  certificates.
- Paste real verbatim output. A run that did not reach the evidence says so.

## Report format

`rum-report/1` at `.temp/brief-641/report.json`; every run with its command,
log, exit code and test counts; `changes` lists both test files with line
ranges and the reason; `disclosures` lists every inventory row that moved;
`questions` carries any STOP. Final message is exactly
`python3 scripts/report.py summary 641`.
