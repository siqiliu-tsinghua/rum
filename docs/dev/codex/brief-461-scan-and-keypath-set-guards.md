# brief-461 — guard association `Scan` and nested key-path `Set`

## Goal

Install the brief-459 native-descent guard on the two remaining guard-eligible
association sites, so each returns an in-band `StackLimitExceeded` with a usable
session where it currently terminates the process, **while still completing at
the ruled physical height 4,097**. Then flip their registry rows from
`UnguardedNative` to `GuardedNative` and replace their `Unmeasured` failure
direction with the established one.

After this, the guarded population is three and every remaining
`UnguardedNative` row in the registry is a conversion, not a guard.

## Why these two and not the other five

Director-measured, release, contract height, three runs per cell, with "1 MiB"
halving the thread to emulate the operation boundary:

| site | 2 MiB | 1 MiB |
|---|---|---|
| association `Scan` | pass | **pass 3/3** |
| nested key-path `Set` | pass | **pass 3/3** |
| `DeleteMissing`, `AssociationKeyDeflatten`, association `MapIndexed`, `AssociateTo` | pass | **kill 3/3** |
| `AssociationKeyFlatten` | pass | kill 2/3 |

A guard refuses at the budget; it cannot make work fit inside it. These two fit,
so a guard converts their fail-open into fail-closed at no cost to the contract.
The other five do not fit, so guarding them would convert contract-meeting sites
into contract-failing ones — which is why they are being converted instead. Do
not touch them here.

## The sites — verified, treat as given

**Association `Scan`.** `eval_scan` holds the session
(`crates/rum-stdlib/src/functional_ops/apply_map/mod.rs:113-131`) and calls
`collect_scan_parts_with_config`
(`crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs:411-412`), whose
recursive worker `collect_scan_parts_recursive` (`:415`) has **three** recursive
edges: association values at `:435`, the head at `:470`, and arguments at
`:476`. It returns `ScanCollection`, not a `Result`.

**Nested key-path `Set`.** `set_association_key_path`
(`crates/rum-runtime/src/runtime/kernel_data/association_data.rs:976`) recurses
at `:996` and returns `AssociationSetUpdate`, not a `Result`. It has two roots,
both `&mut self` methods returning `Result<Expr, EvalError>`: the key-path
finisher whose call is at `:859`, and `finish_association_set_part_lvalue`
(`:881-887`) which reaches it at `:901` through `set_association_part_path`
(`:934-940`). This half is inside `rum-runtime`, so it uses the guard module
directly and needs no `kernel_api` addition.

**Neither recursive function returns a `Result`, and neither should be made to.**
Use the substrate's throwaway-fallback path: when `depth.child()` yields `None`,
stop descending and return a type-correct placeholder —
`ScanCollection { parts: Vec::new(), depth: 1 }` and
`AssociationSetUpdate::Unchanged` — and let the scoped runner's sticky bit turn
the operation into `Err(StackLimitExceeded)`. Threading a `Result` through every
node is the shape that cost 6-15% in brief-431, and the placeholder never
reaches a caller because the runner takes precedence.

## Two requirements that are easy to get wrong

1. **The scope must close before any effect.** For `Scan`, it covers the
   collection only and must be dropped before `evaluate_scan_parts`
   (`mod.rs:128-129`) runs the user callbacks — the token is not
   suspension-safe, and user code may suspend. For `Set`, it covers the path
   descent only and must close before the symbol store at `:859-878` and
   `:881-919`, so an exhausted path can never half-write. Cite the lines that
   show this in your report; do not assert it.
2. **The exhausted path must leave no partial effect.** Say in the report what
   each site does on the `None` branch and why it cannot be observed.

## What is given — do not probe past a ceiling

- Both sites clear the association fixture's own cap (about 10,800 physical
  levels), so **their direction above their own ceiling has never been
  observed**; that is why their registry rows read `Unmeasured` today.
- On a 2 MiB thread the default `native_stack_limit` is 1 MiB, so a certificate
  at physical height 4,097 exercises the guard at its **real** boundary. That is
  the direct proof that guarding does not cost the contract; no deeper probe is
  needed or permitted.
- A ceiling is not deterministic; no test may depend on one.

## Deliverable

### 1. The two guards

As above, using `run_with_native_stack_guard` and `NativeStackDepth` unchanged.
Do not modify the substrate. If you believe the substrate needs a change, STOP
and report — that is a design decision, not an implementation detail.

### 2. Certificates, release-only, on declared 2 MiB threads

Follow the three-part rule; brief-459's file is the model
(`crates/rum-stdlib/src/tests/merge_native_stack_guard.rs`).

- **`Scan`** returns `Null`, so its observable is the side effect. Assert
  **both**: a visit count, and a flag set only when the **unique deepest
  sentinel** is visited — a count alone is a summary that is correct even if the
  deepest level was never reached. Do not build a 4,097-element log to inspect.
  A three-level companion pins the level specification, association position and
  visit order.
- **`Set`** already has a 4,097 exact post-write certificate
  (`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:266-292`); keep it
  and add the unique-deepest-sentinel assertion on the exact written value. Its
  shallow companion must pin **both selector spellings** — a bare string and
  `Key[...]` — because they are not interchangeable across association mutation
  heads and a companion that tests only one pins half the contract.

### 3. Falsifiers — per site, both kinds

- The zero-boundary override (`test-native-stack-guard-zero`) at modest depth
  must return the in-band result with a usable session, for **each** site.
- In your report, remove the guard wrapper from **each** real edge in turn —
  `traversal.rs:435`, `:470`, `:476` and `association_data.rs:996` — capture the
  focused test red verbatim, restore, capture green. Four removals, four reds.
  If removing an edge's wrapper does not turn a test red, that edge has no
  coverage: say so rather than leaving it.

### 4. Registry and inventory rows

- Flip `Scan (association axis)` and `Set (nested association key-path axis)` to
  `Mechanism::GuardedNative`, and replace `FailureDirection::Unmeasured` with
  the direction the guard establishes, clearing `owed_by`. **State in the report
  what that direction rests on** — the guard plus the override test, not a
  ceiling observation.
- Update the recursion inventory rows for the four edges to name the
  declaration they now implement. The inventory test must stay green without
  weakening any assertion.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/functional_ops/apply_map/traversal.rs` and
    `crates/rum-stdlib/src/functional_ops/apply_map/mod.rs`;
  - `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`;
  - `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs` and
    `recursion_inventory.rs` (row updates only);
  - `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`;
  - a new focused test file plus its wiring in `crates/rum-stdlib/src/tests.rs`;
  - module roots whose exported items change.
- Files you MUST NOT touch: the guard substrate
  (`crates/rum-runtime/src/runtime/native_stack_guard.rs`), `kernel_api.rs`,
  the five conversion sites (`missing.rs`, `key_reshape.rs`, `mutation.rs`,
  `exact_paths.rs`, and the `MapIndexed` walkers), `metadata.rs`, `Expr`,
  `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other `docs/**`, `Cargo.lock`.
- **The accept-vs-stop criterion, general so a file I failed to list still
  cannot admit a regression:** the ONLY intended behavior change is that these
  two operations return an in-band `StackLimitExceeded` where they previously
  terminated the process. Any change to what `Scan` visits or in what order,
  to what `Set` writes, to either one's diagnostics, or to any other builtin,
  is a **stop**.
- **If exactly one of the two sites proves infeasible**, land the other
  completely and report the blocker. These are independent files and
  independent certificates; do not hold a working half hostage to a blocked one.

## rum / Rust pitfalls (read twice)

1. `Scan` collects first and evaluates callbacks afterwards
   (`mod.rs:128-129`). Preserve that order; the guard must not extend across the
   evaluation.
2. **Suspension safety.** Both halves sit near resumable paths. The token cannot
   live in a suspension frame — that is a compile error by construction — so a
   scope must not be open across a suspend.
3. **No default-path cost.** Three of the four edges are on `Scan`'s general
   traversal, which is shared machinery: the disabled case must be the strided
   cold call and nothing more.
4. Assert deep structure iteratively, never through `PartialEq`/`Display` on a
   deep value.
5. Logical wrappers are not physical height; the association fixture is two
   physical levels per wrapper, so raw 4,097 is `N=2048`.
6. Wall-clock timing is not evidence on this host.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, and if so
say plainly that the nine native tests did not run in your gate.

Then the release certificates on their declared 2 MiB threads, the two
zero-boundary falsifiers, and the four red/green captures.

Then the cost gate, **counters first**:

- Allocation counts (`RUM_PROFILE_ALLOC=1`, `--features alloc-count`), base
  against candidate, for a `Scan` probe, a `Set` key-path probe, and one
  untouched control. Calibrate the counter's sensitivity and state the
  same-binary spread before believing any delta.
- `perf stat -e instructions:u`, minimum of the tight clean cluster, migrated
  runs discarded, **both arms built at the same filesystem path**.

`Scan` and `Map` share `apply_map`, so include a `Map` probe among the controls:
it must not move.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- **No new `unsafe`.**
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling.**
- **Stop-loss.** Stop and report if: a 4,097 certificate does not complete with
  its exact value; a wrapper removal does not turn its test red; allocation
  counts move on the `Map` control; or the same step fails twice for the same
  reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. The scope-closure argument for each site, with `path:line`, and what the
   exhausted path leaves behind.
3. Verification: every command with verbatim output, and the ZeroMQ statement.
4. The certificates, and the two zero-boundary falsifiers.
5. The four red/green captures, one per real edge.
6. The registry and inventory row changes, and what the new failure direction
   rests on.
7. The cost gate: allocation rows first with calibration and spread, then
   instruction counts, including the `Map` control.
8. Stop-loss and open questions.
