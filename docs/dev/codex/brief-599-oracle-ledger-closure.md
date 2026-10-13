# brief-599 — E1 closure: retire the five nonexistent roles and delete the owed ledger

Implementation brief, **test-only plus one documentation paragraph**. **Do not
run `git` in any form**, including before you have read this brief. Leave every
change uncommitted for the director's gate.

## Goal

brief-586 built the catalog-wide suspension oracle
(`crates/rum-stdlib/tests/suspension_oracle/`) with an exact owed ledger
(`inventory/owed.tsv`) for declared guest-effect entries whose descriptors were
still to be written. Drains 1-8 (briefs 587-592, 596, 598) attached a complete
descriptor to every entry that exists. **Five entries remain owed, and the
director has verified from source that none of them is a real role.** E1 closes
in this brief: retire those five declarations, then delete the ledger mechanism
and put an exact coverage equality in its place. After this brief there is no
list of debt the oracle tolerates; a declared entry without an executed
descriptor fails the gate in either build.

The manifest test currently asserts that the ledger is non-empty
(`manifest.rs:145-148`, "E1 closes by deleting the ledger mechanism, not
preserving an empty escape hatch"). That is why the retirement and the deletion
are one change: removing the five rows alone would fail that assertion.

## Step 1 — the director's dispositions (apply exactly)

Verified at the 598 gate against the cited source:

| Entry | Why it is not a role |
|---|---|
| `AssociationThread.delayedValue` | `eval_association_thread` threads keys and values into rules and copies a `RuleDelayed` rhs unevaluated (`crates/rum-stdlib/src/associations/transforms.rs:127-224`); a later consumer reads it. |
| `DeleteMissing.defaultOption`, `.explicitOption`, `.optionSyntax` | `DeleteMissing` is `Ordinary`; its second and third arguments are positional depths, not options; the handler evaluates no guest code (`crates/rum-stdlib/src/associations/missing.rs:24-77`, `catalog/associations_transform.rs:345-349`). |
| `Normal.delayedValue` | `normalize_association_expr` takes no `Session` and returns the rules with every `RuleDelayed` rhs held (`crates/rum-stdlib/src/associations/basic.rs:217-234,276-290`); only lazy realization evaluates guest code. |

Edits:

1. `inventory/effects.tsv`: delete exactly the five rows whose first field is
   one of the five IDs above. No other row changes.
2. `inventory/owed.tsv`: delete the file (Step 2).
3. `inventory/catalog.tsv`: replace exactly these three rows. Columns 1-8 are
   unchanged; columns 9-11 (disposition, entries, rationale) become:
   - ``System`AssociationThread`` → `NoLocalGuestEffect` · `-` ·
     `Removed after source review: AssociationThread threads keys and values into rules and keeps a RuleDelayed rhs unevaluated; a later consumer such as Values or Lookup reads it. No local guest evaluation in the installed form.`
   - ``System`DeleteMissing`` → `NoLocalGuestEffect` · `-` ·
     `Removed after source review: DeleteMissing is Ordinary; its second and third arguments are positional depths, not options, and the handler evaluates no guest code.`
   - ``System`Normal`` → `GuestEntries` · `Normal.lazyConsumer` ·
     `Local phases: lazyConsumer. Removed after source review: normalizing an association returns its rules with every RuleDelayed rhs held; only lazy realization evaluates guest code. External installed handlers/providers are outside the repository proof.`

Expected counts afterwards: `entries` 443 → **438**, `GuestEntries` 237 →
**235**, `NoLocalGuestEffect` 406 → **408** (default) and 400 → **402**
(`--no-default-features`), `Boundary` **5**, `descriptors` **546** in both
builds.

## Step 2 — delete the ledger mechanism

Delete, in the `suspension_oracle` test tree:

- `inventory/owed.tsv`;
- `catalog::reconcile_owed` and the `CatalogError::{StaleOwed, MissingOwed}`
  variants;
- the falsifiers `mutations::owed_entry_gains_descriptor` and
  `mutations::entry_missing_from_both`;
- in the manifest test, the owed-ledger read, its duplicate check, the
  non-empty assertion, the `OWED families=` line and the `owed=` field of the
  `CATALOG` line;
- any other path that exists only to carry or report debt.

## Step 3 — exact coverage equality in its place

- Add `catalog::reconcile_coverage(declared, covered, exact) ->
  Result<(), CatalogError>` in the shape of `catalog::reconcile`: with `exact`
  false it returns `Ok(())` (the deliberately weakened checker, used only by
  falsifiers, as `reconcile` already documents); with `exact` true it returns
  `UnknownDescriptorEntry(id)` for an attached entry that is not declared and a
  new `UncoveredEntry(id)` for a declared entry with no attached descriptor.
- The manifest test calls it strictly with the feature-filtered declared set
  (both inventories, as now) and the covered set it already builds. Replace
  `owed=` in the `CATALOG` line with `covered=<n>`; the gate checks
  `covered == entries` in both builds.
- Falsifiers, in the existing WEAK / STRICT / CONTROL shape of `mutations/`:
  `declared_entry_without_descriptor` (strict → `UncoveredEntry`) and
  `descriptor_for_undeclared_entry` (strict → `UnknownDescriptorEntry`), each
  with its weak pass and a restored good control.
- Rename the manifest test `installed_catalog_and_owed_ledger_are_exact` to
  `installed_catalog_and_descriptor_coverage_are_exact`.
- Every other falsifier stays exactly as it is — `omitted_registration`,
  `non_suspending_fixture`, `false_tail`, `missing_prefix`, and the
  `mutations/capture_output` and `mutations/driver` modules — and so does the
  two-feature gate in `scripts/check-suspension-oracle.sh`.

**The real gate must be seen to fail (required).** Falsifiers prove the
function; they do not prove the manifest test calls it. Before the final runs,
temporarily set the descriptor column of the `Normal.lazyConsumer` row in
`effects.tsv` to `-`, run the oracle in the default build, and record the
failure naming `UncoveredEntry("Normal.lazyConsumer")`; restore the row and
record the green run. Keep both outputs under `.temp/brief-599/`.

## Step 4 — the documentation paragraph

In `docs/testing-strategy.md` §Installed-Catalog Suspension Gate, and nowhere
else:

- in the paragraph ending "...exercise the acceptance functions.", change
  "and owed-ledger falsifiers" to "and coverage falsifiers";
- replace the whole paragraph beginning "Step A keeps an exact owed-entry
  ledger" (it ends "...the ledger mechanism is deleted.") with:

  > Coverage is exact and keeps no list of debt. Every declared guest-effect
  > entry, in `inventory/effects.tsv` and `inventory/shared-effects.tsv` after
  > feature filtering, carries at least one executable descriptor that runs its
  > full declared variant set and, unless it is a declared boundary, its
  > lifecycle obligations at both pauses. An entry without a descriptor, or a
  > descriptor attached to an undeclared entry, fails the gate in either build.
  > Entries are declared by source review of each owner, not by source-edge
  > discovery, and passing the gate does not claim that arbitrary guest
  > programs or external Rust handlers are proven safe.

The following sentence ("A default-only workspace run does not satisfy the
two-feature obligation.") stays.

## Scope

**Edit surface:** the `suspension_oracle` test tree (the files named above);
`docs/testing-strategy.md`, only the two edits in Step 4.

**Standing duty — always in scope** (`TEMPLATE.md`; test edits under `crates/`
may expire the fingerprints): fingerprints, both certificates, ownership
reconciliation, every row disclosed. **STOP** on any existing ownership row
change or disappearance, any new `UnknownHostPayload` or `self_reaching` row,
any new row in untouched code.

**Fence:** no production `.rs`, `lib.rs`, Cargo manifest, lockfile or
dependency change; no descriptor, fixture or variant change; no other
inventory row; no edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier
briefs, `scripts/` or any other doc.

**Disclosure duty:** every inventory row changed or deleted, every deleted and
added item, every test name added, removed or renamed (capture `--list` in
both builds before and after), and the output of the residue search below.

## rum / Rust pitfalls

1. The non-empty assertion is a guard against stopping at an empty ledger;
   deleting it together with the ledger is the intended close, not a
   weakening. Do not keep any switch that lets an uncovered entry pass.
2. A falsifier that only calls the function is half the proof; the forced red
   on the real manifest test is the other half.
3. Stop-loss: stop after two consecutive compiling attempts that move none of
   (rows applied, mechanism items deleted, checks added, gates green); fewer
   compiler errors is progress.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-599/tmp …`. Bounds: the oracle 900 s per build;
ownership and certificates 300 s; `cargo test --workspace --no-fail-fast`
3 600 s.

```
cargo fmt --check
scripts/check-suspension-oracle.sh 900
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
rg -n -i -w "owed|debt|reconcile_owed|StaleOwed|MissingOwed" crates/rum-stdlib/tests/suspension_oracle docs/testing-strategy.md
```

The `CATALOG` lines must show the expected counts above and `covered` equal to
`entries`, in both builds. The residue search must return nothing that carries
or reports debt; list and explain anything it does return. Report native
ZeroMQ skips as skips.

## Checkpointing — protect the scene

Everything under `.temp/brief-599/`; `PROGRESS.md` there after each step with
the counters and the exact next action.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, `scripts/check-suspension-oracle.sh`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only. No
  new `unsafe`.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Files changed; the no-production-edit audit.
2. The five retirements and three catalog rows, before and after; the deleted
   mechanism items; the added check, falsifiers and rename; the doc diff.
3. The forced red and its green; both `CATALOG` lines; the residue search; the
   broad suites verbatim; test-name sets before and after.
4. Stop-loss counters, open questions.

---
