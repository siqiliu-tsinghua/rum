# brief-460 — the association-depth enforcement layers

## Goal

Make the association depth class a **closed declaration** instead of six
remembered guards. When this is done:

- every association builtin, and every cross-family axis that descends an
  association, carries a row saying how it is bounded and whether it meets the
  ruled physical height 4,097 — and a new catalog entry without a row fails a
  test with the exact name;
- every recursive descent in the association implementation surface carries a
  row saying which declaration it implements — and a new or changed recursive
  edge fails a test with the exact `(file, caller, callee, count)`;
- both checks ship with a proof they can fire.

**No behavior changes.** Not one operation is guarded, converted, or otherwise
altered by this brief. It installs the structure that the five conversions and
the two remaining guards will then flip, one row at a time.

## Ruling: two layers in this step, not three

brief-458 proposed a third layer — a `rum-derive` `guarded_native_recursion!`
procedural macro that parses a recursion group, computes its SCCs, and rejects
any edge not syntactically wrapped, making an unguarded edge a **compile error**
rather than a test failure. That is a stronger guarantee and it is **deferred to
its own step, not dropped.**

The reason is gateability, which is this line's standing criterion — brief-378's
split was "for gateability, not size": each step has to be independently
verifiable and revertible. An SCC analyser inside a proc macro is a substantial
artifact with its own correctness surface, and bundling it with the registry
would put two unrelated risks behind one gate. It is also *covered* meanwhile:
layer 2 below computes the same recursion rows from source, so an edge added
inside an already-guarded function changes that function's occurrence count and
turns the inventory red. Weaker than a compile error, but enforcement, not
convention.

**Re-open the macro step when the guarded population stops being small** — today
it is `Merge` (landed in brief-459) plus `Scan` and nested key-path `Set` (both
measured guard-eligible, see below), so three — or when a conversion is rejected
and a site stays natively recursive by decision rather than by schedule.

## Ruling: a row states what is true now, never what a later brief will make true

brief-458's inventory proposed `GuardedNative` for `Scan` and nested key-path
`Set`, which are not guarded. A declaration that names a future state is a
deferral wearing a repair's clothes — brief-417 put five declarations into an
unenforced contract and brief-419 caught all five the day a falsifier existed.

So each row carries **two orthogonal fields**, and neither may borrow from the
other:

- **Mechanism**, what the code does today: `NoProportionalDescent` |
  `BoundedHeap` | `GuardedNative` | `UnguardedNative` | `EvaluatorPolled`.
- **Contract status**, measured: `MeetsContract` | `ShortOfContract`. Every row
  that is not `NoProportionalDescent` also records its **failure direction above
  its ceiling** (`FailsOpen` | `FailsClosed`) and, when work remains, the brief
  or queue item that owes it.

A row reading `UnguardedNative + MeetsContract + FailsOpen + <owed brief>` is
the honest current state of six sites. It is visibly unfinished, which is the
point.

## What is given — measured, do not re-run, do not probe past a ceiling

Director-measured, release, contract height (raw physical 4,097), three runs per
cell. "1 MiB" halves the thread to emulate the operation boundary that
brief-459 ruled.

| site | 2 MiB | 1 MiB | consequence |
|---|---|---|---|
| `Merge` collection parser | pass | **pass** | guarded in brief-459 |
| association `Scan` | pass | **pass** | **guard-eligible** |
| nested key-path `Set` | pass | **pass** | **guard-eligible** |
| `Map`, `MergeNested` | pass | **pass** | already `BoundedHeap` |
| `DeleteMissing` | pass | **kill** | conversion, not a guard |
| `AssociationKeyDeflatten` | pass | **kill** | conversion |
| association `MapIndexed` | pass | **kill** | conversion |
| `AssociateTo` Part target | pass | **kill** | conversion |
| `AssociationKeyFlatten` | pass | kill 2/3 | conversion |

Ceilings on a 2 MiB thread, physical levels, and the direction above them:
`DeleteMissing` 6,272 / 6,304 kill; `AssociationKeyDeflatten` and association
`MapIndexed` 7,344 / 7,376 kill; `AssociateTo` and `KeyDropFrom` 8,048 / 8,080
kill; `AssociationKeyFlatten` 8,292 / 8,334 kill; `Merge` collection parser
16,072 / 16,128 kill. `Scan`, nested key-path `Set`, `Map` and `MergeNested`
clear the association fixture's own cap (about 10,800 physical) and their
direction above it is **unmeasured** — record that as unmeasured, do not guess.

Short of the contract, with direction: multi-spec `GroupBy` 2,016 fails open;
`KeyTake` over nested lists 2,856 fails open; generic-shape `MapIndexed` 2,944
fails closed; `Keys` over nested lists 3,056 fails open;
`ConformSchema`/`SchemaErrors` 208 fails closed **on the schema-depth axis, not
the value axis** — a deep value against a shallow schema clears outright.

A ceiling is not deterministic: inside a band tens of levels wide the same
height flips run to run. No test may depend on one.

## Deliverable

### Layer 3 — the association surface registry

A sorted, exhaustive registry, separate from `BuiltinSpec` so dispatch gains no
field, declaring a row for:

- every name in the association catalogs
  (`crates/rum-stdlib/src/catalog/associations.rs`,
  `crates/rum-stdlib/src/catalog/associations_transform.rs`);
- the cross-family axes that descend an association: `Map`, `Apply`,
  `MapIndexed`, `Scan`, `Set` (nested key path), and keyed `Part`.

Exact-equality against the catalogs, so a **new association builtin without a
row fails with its own name in the message**, in the shape of
`crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:199-215`.

The registry must live where it can see the stdlib catalogs. `rum-runtime` has
no `[dev-dependencies]` and cannot depend on `rum-stdlib`, so this is a
`rum-stdlib` test; say in the report where you put it and why.

### Layer 2 — the recursion inventory

A `syn`-based test that scans production Rust under:

- `crates/rum-stdlib/src/associations/**`;
- `crates/rum-stdlib/src/functional_ops/apply_map/**`;
- `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`;
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/exact_paths.rs`;

computes direct-self and mutual recursion within each file, and compares a
sorted exact inventory of `(path, caller, callee, occurrence count,
declaration it implements)` rows. **Exhaustive within those roots — no
per-function allowlist**, and a row whose declaration is absent from layer 3 is
a failure.

Those roots are the implementation of the surface layer 3 declares, which is why
they are the boundary; that correspondence is the justification, not
convenience. **State the limitation plainly in the report and in the test's own
documentation**: recursion reached through a trait object or function pointer,
recursion generated by a macro, and recursion outside those roots are not seen.
A scan that reports clean must be shown capable of finding something —
brief-388 closed the two instances it had seen rather than the class, and the
frame-roots line lost a round to a regex silently truncated by a comma.

`syn` is already in the workspace (`crates/rum-derive/Cargo.toml:16`); adding it
to `rum-stdlib`'s dev-dependencies is in scope, and so is the resulting
`Cargo.lock` change if there is one.

### The falsifiers — both, checked in, and reproduced at the gate

- **Registry.** A checked-in `#[should_panic]` that presents a *real* catalog
  entry under the wrong mechanism — misdeclare `AssociationKeyFlatten` as
  `NoProportionalDescent` — and asserts the validator names its real recursive
  edge. Following the real-catalog standard at
  `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs:253-319`;
  brief-419 caught five bad declarations the day this existed.
- **Inventory.** A checked-in unit test over a synthetic source string
  containing one unregistered self-recursive function and one unregistered
  mutual pair, asserting the exact rows reported. **Then, in your report,**
  temporarily add one extra recursive call inside a real function in the scanned
  roots, capture the inventory red verbatim with the changed occurrence count,
  restore it, and capture green. Do not leave that edit in the tree.

## Scope

- Files you MAY modify:
  - new test files under `crates/rum-stdlib/src/tests/` plus their module wiring
    in `crates/rum-stdlib/src/tests.rs`;
  - a new non-test module under `crates/rum-stdlib/src/` **only** if the
    registry data must be shared with a test in another file — justify it;
  - `crates/rum-stdlib/Cargo.toml` (dev-dependencies only);
  - `Cargo.lock`, only if the dev-dependency actually changes it.
- Files you MUST NOT touch: every operation implementation, the guard substrate
  from brief-459, `kernel_api.rs`, catalogs (except to *read* them), `Expr`,
  `metadata.rs`, `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other `docs/**`.
- **The accept-vs-stop criterion, general so a file I failed to list still
  cannot admit a regression:** this brief adds tests and declarations only. Any
  change to what any builtin computes, accepts, or reports — or to any
  non-test code path — is a **stop**. Report it instead of shipping it.

## rum / Rust pitfalls (read twice)

1. **The registry is test-only metadata.** No `BuiltinSpec` field, no dispatch
   cost, no normal-build allocation.
2. **Do not classify by file location.** Read the mechanism from the code. I
   once told the user `Trace` was `Diagnostic`-gated because it lives in
   `debug.rs`; it is `Capability::Pure`.
3. **A count derived by a script is a claim, not a fact.** Cross-check the
   inventory's row count against a few functions you read by hand, and say which
   ones. A regex over `docs/reference/builtins.md` once said "527 unprotected"
   when the truth was 483.
4. **Sorted and deterministic.** The inventory must not depend on filesystem
   order or on `HashMap` iteration order.
5. Wall-clock timing is not evidence on this host, and nothing here needs a
   timing number.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

Both falsifiers, with the registry `#[should_panic]` output and the inventory's
red capture and green capture all pasted verbatim.

Then confirm the no-behavior-change claim mechanically: `git diff --stat` shows
only test files, module wiring, `Cargo.toml` and possibly `Cargo.lock`. Paste
it.

No performance gate: no non-test code path changes. Say that explicitly rather
than running the panel — and if you find yourself needing to change a non-test
path, that is the stop condition above.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- **No new `unsafe`.**
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling.** Every
  number you need is given above.
- **Stop-loss.** Stop and report if: a registry row cannot be settled from
  source without guessing; the inventory cannot be made deterministic; either
  falsifier does not fire; or the same step fails twice for the same reason. An
  honest "this row is unmeasured" or "this mechanism is unclear" is a correct
  deliverable, not a shortfall — do not invent a disposition to fill a cell.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. The registry: its row count, the vocabulary, and every row whose mechanism
   you could not settle from source.
3. The inventory: its roots, its row count, the hand cross-check you did and on
   which functions, and its stated blind spots.
4. Verification: every command with verbatim output, plus `git diff --stat`.
5. Both falsifiers, including the inventory's red capture and green capture.
6. Stop-loss and open questions.
