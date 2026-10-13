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

## Execution report

### 1. Files changed and production audit

E1 is closed: both oracle builds pass with 438 declared entries, 438 covered entries and 546 executed descriptors. The real manifest gate was observed rejecting the required missing attachment before restoration. All changes remain uncommitted. No git command or network command was run.

- `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv`

- `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv`

- `crates/rum-stdlib/tests/suspension_oracle/catalog.rs`

- `crates/rum-stdlib/tests/suspension_oracle/inventory/catalog.tsv`

- `crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv`

- `crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv`

- `crates/rum-stdlib/tests/suspension_oracle/manifest.rs`

- `crates/rum-stdlib/tests/suspension_oracle/mutations/mod.rs`

- `docs/dev/codex/brief-599-oracle-ledger-closure.md`

- `docs/testing-strategy.md`


The audit compares the entry snapshot of 1,989 files under `crates/`, `docs/`, `scripts/` and repository-root files. Only the listed scoped files changed. No production Rust, `lib.rs`, descriptor, fixture, variant, Cargo manifest, lockfile, dependency, script, ownership inventory, AGENTS/TODO/root PROGRESS, or earlier brief changed. No unsafe was added. All unchanged falsifiers, including `omitted_registration`, `non_suspending_fixture`, `false_tail`, `missing_prefix`, `capture_output` and `driver`, remain byte-identical. The code changes are test-only; no evaluation hot path was touched.

Evidence: [final-scope-audit.json](../../../.temp/brief-599/final-scope-audit.json), [implementation.diff](../../../.temp/brief-599/implementation.diff), [ownership-reconciliation.json](../../../.temp/brief-599/ownership-reconciliation.json). Agent-created scratch evidence and checkpoints are under `.temp/brief-599/`. The unchanged gate script also uses its existing `.temp/suspension-oracle/` scratch paths and overrides TMPDIR there; its outputs were copied into this brief's directory. Existing test-owned artifact paths were preserved, including the required legacy certificate output.

### 2. Inventory changes, mechanism replacement and documentation

Deleted exactly these five effect rows; their after-state is absent:

```tsv
AssociationThread.delayedValue	System`AssociationThread	delayedValue	crates/rum-stdlib/src/associations/transforms.rs	eval_association_thread	Frame	-	Middle,Twice,Error,AlreadyResumed,Nested	crates/rum-stdlib/src/associations/transforms.rs#eval_association_thread#delayedValue	associations_transform	always
DeleteMissing.defaultOption	System`DeleteMissing	defaultOption	crates/rum-stdlib/src/associations/missing.rs	eval_delete_missing	Frame	-	Middle,Twice,Error,AlreadyResumed,Nested	crates/rum-stdlib/src/associations/missing.rs#eval_delete_missing#defaultOption	associations_transform	always
DeleteMissing.explicitOption	System`DeleteMissing	explicitOption	crates/rum-stdlib/src/associations/missing.rs	eval_delete_missing	Frame	-	Middle,Twice,Error,AlreadyResumed,Nested	crates/rum-stdlib/src/associations/missing.rs#eval_delete_missing#explicitOption	associations_transform	always
DeleteMissing.optionSyntax	System`DeleteMissing	optionSyntax	crates/rum-stdlib/src/associations/missing.rs	eval_delete_missing	Frame	-	Middle,Twice,Error,AlreadyResumed,Nested	crates/rum-stdlib/src/associations/missing.rs#eval_delete_missing#optionSyntax	associations_transform	always
Normal.delayedValue	System`Normal	delayedValue	crates/rum-stdlib/src/associations/basic.rs	eval_normal	Frame	-	Middle,Twice,Error,AlreadyResumed,Nested	crates/rum-stdlib/src/associations/basic.rs#eval_normal#delayedValue	associations_transform	always
```

`inventory/owed.tsv` was deleted, including its header and all five data rows. These are every deleted ledger row; their after-state is absent:

```tsv
AssociationThread.delayedValue	associations_transform	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
DeleteMissing.defaultOption	associations_transform	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
DeleteMissing.explicitOption	associations_transform	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
DeleteMissing.optionSyntax	associations_transform	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
Normal.delayedValue	associations_transform	Role-specific variants and lifecycle remain owed; historical descriptors are supplemental witnesses, not automatic admission.
```

Exactly three catalog rows changed. Columns 1–8 are byte-identical; before and after follow in pairs:

```tsv
BEFORE	System`AssociationThread	always	stdlib	Ordinary	Evaluator.callPreparation	crates/rum-stdlib/src/catalog/associations_transform.rs	crates/rum-stdlib/src/associations/transforms.rs	eval_association_thread	GuestEntries	AssociationThread.delayedValue	Local phases: delayedValue. External installed handlers/providers are outside the repository proof.
AFTER	System`AssociationThread	always	stdlib	Ordinary	Evaluator.callPreparation	crates/rum-stdlib/src/catalog/associations_transform.rs	crates/rum-stdlib/src/associations/transforms.rs	eval_association_thread	NoLocalGuestEffect	-	Removed after source review: AssociationThread threads keys and values into rules and keeps a RuleDelayed rhs unevaluated; a later consumer such as Values or Lookup reads it. No local guest evaluation in the installed form.
```

```tsv
BEFORE	System`DeleteMissing	always	stdlib	Ordinary	Evaluator.callPreparation	crates/rum-stdlib/src/catalog/associations_transform.rs	crates/rum-stdlib/src/associations/missing.rs	eval_delete_missing	GuestEntries	DeleteMissing.defaultOption,DeleteMissing.explicitOption,DeleteMissing.optionSyntax	Local phases: defaultOption, explicitOption, optionSyntax. External installed handlers/providers are outside the repository proof.
AFTER	System`DeleteMissing	always	stdlib	Ordinary	Evaluator.callPreparation	crates/rum-stdlib/src/catalog/associations_transform.rs	crates/rum-stdlib/src/associations/missing.rs	eval_delete_missing	NoLocalGuestEffect	-	Removed after source review: DeleteMissing is Ordinary; its second and third arguments are positional depths, not options, and the handler evaluates no guest code.
```

```tsv
BEFORE	System`Normal	always	stdlib	Ordinary	Evaluator.callPreparation	crates/rum-stdlib/src/catalog/associations_transform.rs	crates/rum-stdlib/src/associations/basic.rs	eval_normal	GuestEntries	Normal.delayedValue,Normal.lazyConsumer	Local phases: delayedValue, lazyConsumer. External installed handlers/providers are outside the repository proof.
AFTER	System`Normal	always	stdlib	Ordinary	Evaluator.callPreparation	crates/rum-stdlib/src/catalog/associations_transform.rs	crates/rum-stdlib/src/associations/basic.rs	eval_normal	GuestEntries	Normal.lazyConsumer	Local phases: lazyConsumer. Removed after source review: normalizing an association returns its rules with every RuleDelayed rhs held; only lazy realization evaluates guest code. External installed handlers/providers are outside the repository proof.
```

Every other oracle inventory row is byte-identical. Full row records: [oracle-row-changes.json](../../../.temp/brief-599/oracle-row-changes.json). The temporary forced-red change to `Normal.lazyConsumer` was restored exactly and is absent from the final diff.

Deleted mechanism items:


- `inventory/owed.tsv`

- `catalog::reconcile_owed`

- `CatalogError::StaleOwed`

- `CatalogError::MissingOwed`

- `mutations::owed_entry_gains_descriptor`

- `mutations::entry_missing_from_both`

- `manifest owed-ledger read`

- `manifest debt-set construction`

- `manifest duplicate owed check`

- `manifest reconcile_owed call`

- `manifest non-empty assertion`

- `manifest family aggregation`

- `CATALOG owed= field`

- `OWED families= output`


Added `CatalogError::UncoveredEntry`, `catalog::reconcile_coverage`, a strict manifest call, the `covered=` CATALOG field, and `declared_entry_without_descriptor` / `descriptor_for_undeclared_entry`. Each falsifier proves WEAK pass, the exact STRICT error, and a restored good CONTROL. The real call is strict and runs before the unattached-descriptor assertion so a missing attachment identifies its uncovered entry. The weakened mode is used only by falsifiers. Renamed `installed_catalog_and_owed_ledger_are_exact` to `installed_catalog_and_descriptor_coverage_are_exact`.

The only testing-strategy edits are this exact diff:

```diff
--- before/docs/testing-strategy.md
+++ after/docs/testing-strategy.md
@@ -89,15 +89,19 @@
 and raw traces; a final-only comparison cannot detect provisional output leaking
 and later disappearing. Lifecycle checks at both pauses cover buffered output
 and exclusive print sinks. Catalog, no-pause, false-tail, lost-prefix,
-message-prefix, trace-prefix and owed-ledger falsifiers exercise the acceptance
+message-prefix, trace-prefix and coverage falsifiers exercise the acceptance
 functions. Historical continuation suites
 retain their specialized assertions. Declared boundary sinks must retire input.
 
-Step A keeps an exact owed-entry ledger for role descriptors still to be added;
-a descriptor cannot silently discharge an owed entry, and an undeclared gap
-fails. Passing this gate does not claim that owed roles have been exercised or
-that arbitrary guest programs/external Rust handlers are proven safe. E1 closes
-only when all owed entries are covered and the ledger mechanism is deleted.
+Coverage is exact and keeps no list of debt. Every declared guest-effect
+entry, in `inventory/effects.tsv` and `inventory/shared-effects.tsv` after
+feature filtering, carries at least one executable descriptor that runs its
+full declared variant set and, unless it is a declared boundary, its
+lifecycle obligations at both pauses. An entry without a descriptor, or a
+descriptor attached to an undeclared entry, fails the gate in either build.
+Entries are declared by source review of each owner, not by source-edge
+discovery, and passing the gate does not claim that arbitrary guest
+programs or external Rust handlers are proven safe.
 A default-only workspace run does not satisfy the two-feature obligation.
 
 ## Performance Gates
```

Standing maintenance replay changed only column 19 in all 68 local-contract rows. The exact Rust length-delimited FNV algorithm first reproduced all 68 baseline fingerprints with zero differences. Identities, statuses, guarantees and exclusions are unchanged.

```text
baseline: 68 contract fingerprints; 0 changed; 1408 source files; context=fnv1a64:bd43ea2a32076899
refresh: 68 contract fingerprints; 68 changed; 1408 source files; context=fnv1a64:2c76e1245cd3b2ec
```

| Row | Contract | Before | Replayed |
|---|---|---|---|

| 2 | B01 | `fnv1a64:a802cf752c0f8b99` | `fnv1a64:6ba3a829d90a71d2` |

| 3 | B02 | `fnv1a64:45d09df5fb1a571c` | `fnv1a64:d5864c150058e9e3` |

| 4 | B03 | `fnv1a64:651737748a60467b` | `fnv1a64:4dd190f2affe95d0` |

| 5 | B04 | `fnv1a64:bc1b7ce26e329e41` | `fnv1a64:04c483be8753bb8a` |

| 6 | B05 | `fnv1a64:3e109306fd7ee168` | `fnv1a64:4f9c40fbfe7ae6bf` |

| 7 | B06 | `fnv1a64:be92836b1f02310d` | `fnv1a64:42d68f2867f5a686` |

| 8 | B07 | `fnv1a64:fc87b408ac162a78` | `fnv1a64:3c190a4f137e60cf` |

| 9 | B08 | `fnv1a64:85c02de849876a67` | `fnv1a64:8260102b08e7e0fc` |

| 10 | B09 | `fnv1a64:45e59fdc66beaeb5` | `fnv1a64:7e4d2a522243a23e` |

| 11 | B10 | `fnv1a64:bd4ce8bf7ad15e8e` | `fnv1a64:833f123778ad35b9` |

| 12 | B11 | `fnv1a64:7d732e3f8826c441` | `fnv1a64:c61c351ba147e18a` |

| 13 | B12 | `fnv1a64:04f21fd842c743b5` | `fnv1a64:3d59aa4dfe4c373e` |

| 14 | I01 | `fnv1a64:85e4ba7298c78219` | `fnv1a64:4985932745c26852` |

| 15 | I02 | `fnv1a64:9552109af5b56968` | `fnv1a64:a6ddbe8ff6b16ebf` |

| 16 | I03 | `fnv1a64:68d36839d0c51475` | `fnv1a64:803dc77a012d61fe` |

| 17 | I04 | `fnv1a64:ec35f55d4ff17ebc` | `fnv1a64:ea595d2b9acf4783` |

| 18 | P01 | `fnv1a64:17630c64aa246b2d` | `fnv1a64:a045396b6faddda6` |

| 19 | P02 | `fnv1a64:97d305ee1d76a73e` | `fnv1a64:27ea4f9a6a9b28a9` |

| 20 | P03 | `fnv1a64:37c9d9e97a66ab13` | `fnv1a64:407686e71eac08b8` |

| 21 | P04 | `fnv1a64:3aabbc0b7fc3731c` | `fnv1a64:ca616a2a850205e3` |

| 22 | P05 | `fnv1a64:32c2eeafe7dba789` | `fnv1a64:8f72a2c3def51142` |

| 23 | P06 | `fnv1a64:557144a8cef6dd35` | `fnv1a64:8dd8cf1e8a7bd0be` |

| 24 | P070 | `fnv1a64:8fbee0eaad6c5735` | `fnv1a64:c8266b6068f14abe` |

| 25 | P080 | `fnv1a64:021c1464f524004e` | `fnv1a64:948b370e6b036879` |

| 26 | P090 | `fnv1a64:df60269270ab5adc` | `fnv1a64:26fa7b6255a3e8a3` |

| 27 | P100 | `fnv1a64:ae72c45675cd5a51` | `fnv1a64:90a2145e06a8865a` |

| 28 | P110 | `fnv1a64:4f79167dfa6a5206` | `fnv1a64:c0b16ebb79d1d861` |

| 29 | P120 | `fnv1a64:ef4d7be66e935487` | `fnv1a64:93d4d51c5ce6ad9c` |

| 30 | P130 | `fnv1a64:a0550d65daca0478` | `fnv1a64:dfe663ac42323acf` |

| 31 | P140 | `fnv1a64:0550bcad9fe6b394` | `fnv1a64:b3dbf46f30702e6b` |

| 32 | P150 | `fnv1a64:09c0da594b8a527c` | `fnv1a64:969bcaf1bf5c4a43` |

| 33 | P071 | `fnv1a64:547ca08c42b98c18` | `fnv1a64:06ec8fd60c4f3eef` |

| 34 | P081 | `fnv1a64:cf8ffaa70d3020d5` | `fnv1a64:24bd37fd1573a9de` |

| 35 | P091 | `fnv1a64:03da1d1368e061ed` | `fnv1a64:793b40c890ebee66` |

| 36 | P101 | `fnv1a64:477c074b2fcdef06` | `fnv1a64:b8b45f88af357561` |

| 37 | P111 | `fnv1a64:792fdfdf0ab84905` | `fnv1a64:48b1cebe422a8f0e` |

| 38 | P121 | `fnv1a64:93fc1b35a4f78552` | `fnv1a64:0d5e596dbc00e38d` |

| 39 | P131 | `fnv1a64:59c59f48a486bfc1` | `fnv1a64:a26ea624bda7dd0a` |

| 40 | P141 | `fnv1a64:71b14c60d3f60fcd` | `fnv1a64:e589864821c5b046` |

| 41 | P151 | `fnv1a64:a5a1d1ef6a272fc7` | `fnv1a64:7ba78a41a130bddc` |

| 42 | B13 | `fnv1a64:4ca316f44f9eed0d` | `fnv1a64:d0e722b198926286` |

| 43 | R01 | `fnv1a64:9a026b20e7d5caac` | `fnv1a64:406cc605086c6833` |

| 44 | R02 | `fnv1a64:de3222a8c2cf5dd3` | `fnv1a64:245420fb023c9e78` |

| 45 | R03 | `fnv1a64:20c050dfa65c2ae0` | `fnv1a64:676866f6f0706407` |

| 46 | R040 | `fnv1a64:389cd784fcd0d5cc` | `fnv1a64:7268ac0696fa0cd3` |

| 47 | R050 | `fnv1a64:e4bb8e1aba45cde9` | `fnv1a64:2e8a6ab4354c16a2` |

| 48 | R060 | `fnv1a64:cec9e9f248fd4f24` | `fnv1a64:6c0a72c64e2d193b` |

| 49 | R070 | `fnv1a64:37fdc2c0eed1dfd5` | `fnv1a64:8d2b0016f71568de` |

| 50 | R080 | `fnv1a64:b33a5e501f7647bc` | `fnv1a64:b15dc61e6a541083` |

| 51 | R090 | `fnv1a64:6898fcde51381e11` | `fnv1a64:1b1a3e1dee5a1e1a` |

| 52 | R100 | `fnv1a64:2a3cdbc73435f0fb` | `fnv1a64:12f7354559d44050` |

| 53 | R110 | `fnv1a64:8844202d5efd5f16` | `fnv1a64:7f6086d64c6d2d71` |

| 54 | R041 | `fnv1a64:32dc7a1e66c6ac0d` | `fnv1a64:b72085dbafba2186` |

| 55 | R051 | `fnv1a64:6d8254a7fdd9675c` | `fnv1a64:b51ca977e2d1f523` |

| 56 | R061 | `fnv1a64:682df1def61b7417` | `fnv1a64:b0602ff3ad8463ac` |

| 57 | R071 | `fnv1a64:a05d6d4f91e1325a` | `fnv1a64:221bb4bfb5d00b45` |

| 58 | R081 | `fnv1a64:6a5d2dab11792f51` | `fnv1a64:4c8c7db2a2545b5a` |

| 59 | R091 | `fnv1a64:1c03b72a3c45b6d4` | `fnv1a64:8a6c2e19bb7ebeab` |

| 60 | R101 | `fnv1a64:a3a9bcd45f844ee4` | `fnv1a64:c92166ee833fdafb` |

| 61 | R111 | `fnv1a64:c9aaf1a78d9a2a4f` | `fnv1a64:14b567908c30ae34` |

| 62 | R12 | `fnv1a64:7376d93b7f42e48f` | `fnv1a64:5b9f2305140deb74` |

| 63 | R13 | `fnv1a64:11554469470c41eb` | `fnv1a64:63c8adfcea9a61c0` |

| 64 | R14 | `fnv1a64:c8cad2cc527e1a51` | `fnv1a64:aafa22d3e359465a` |

| 65 | R15 | `fnv1a64:51466a3e86d1f1fd` | `fnv1a64:62cfbf2573566676` |

| 66 | O010 | `fnv1a64:d5b04114ba0cfff7` | `fnv1a64:f788d24499d3940c` |

| 67 | O0200 | `fnv1a64:00185e81ee9dd994` | `fnv1a64:aea396437f27546b` |

| 68 | O011 | `fnv1a64:1152a67a568f13a4` | `fnv1a64:ae932f4e5bbeddbb` |

| 69 | O0210 | `fnv1a64:f957b4a1fd532fef` | `fnv1a64:532459524bc5e754` |


Complete replay record: [contract-refresh.json](../../../.temp/brief-599/contract-refresh.json).

Both certificate builds independently earned their cells. The first replay in each build rejected the old certificate inventory; after copying the exact generated cells, both reruns passed. Only these four cells in two existing `promoted.tsv` rows changed:


| Row | Body | Build | SHA-256 before | SHA-256 after |
|---|---|---|---|---|

| 2 | C01 | hash=true | `60912241a1682786b571935f405e27936d3c9aaf7db14362d2e9ae370a2a702d` | `a3abce30e96da595c327ab03d96f9764dce09a038192337b356937edf57d1625` |

| 3 | C02 | hash=true | `805c63c09354eb02efe508976148ea36b870e88d7deaf63b68824853efe154c4` | `d245f07b271347f8448c38c97c287d570892977b87c959f5c7e0abfcbe2e8c65` |

| 2 | C01 | hash=false | `b46e700c17d7ec832c590da168acbc3c3108daeb3be6accefd9dc62166dd3283` | `d851785655877a82dd6a29727b8581a3d908e12949443a42467aa47a0fc54fc3` |

| 3 | C02 | hash=false | `b01657ccb8b9cd68fc403b4e9b9aa5164e1115fccec0c153dc1dfc2aed268358` | `45100018c8def351dab34d60772d29fd22264dbf93b050af96b8c43843c257cc` |


These hashes identify long cells; they are not substituted into the inventory. Full old/new cell contents are in [certificate-cell-changes-true.json](../../../.temp/brief-599/certificate-cell-changes-true.json) and [certificate-cell-changes-false.json](../../../.temp/brief-599/certificate-cell-changes-false.json). Generated source copies: [promoted-hash-true.tsv](../../../.temp/brief-599/promoted-hash-true.tsv) and [promoted-hash-false.tsv](../../../.temp/brief-599/promoted-hash-false.tsv). Every other certificate field is byte-identical. Ownership reconciliation added, removed and changed zero rows: 59 self-reaching, 65 UnknownHostPayload, 7,680 UnknownConstruct.

### 3. Verification evidence and test-name sets

All Cargo commands ran serially through `python3 scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-599/tmp ...`. Oracle runs were bounded at 900 seconds per build; ownership/certificate runs at 300 seconds; workspace at 3,600 seconds. The two-feature script was also enclosed in a 1,800-second wrapper and retained its own two 900-second bounds. `--nocapture` retained evidence without changing selected tests.

`cargo fmt --check`: exit 0, no output ([fmt-check.log](../../../.temp/brief-599/fmt-check.log)). The initial check found four assertion line wraps; only those were fixed, and its output is retained in [fmt-check-initial.log](../../../.temp/brief-599/fmt-check-initial.log).

Forced red: temporarily changed only the descriptor column of `Normal.lazyConsumer` to `-`, then ran the whole default oracle. Exit 101, expected failure ([forced-red-default.log](../../../.temp/brief-599/forced-red-default.log)):

```text
thread 'oracle::manifest::installed_catalog_and_descriptor_coverage_are_exact' panicked at crates/rum-stdlib/tests/suspension_oracle/manifest.rs:135:60:
called `Result::unwrap()` on an `Err` value: UncoveredEntry("Normal.lazyConsumer")
FAILED
test result: FAILED. 48 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.41s
```

Restored the entire effects file byte-for-byte to its post-retirement state, then reran green ([restored-green-default.log](../../../.temp/brief-599/restored-green-default.log)):

```text
LIFECYCLE obligations=4026 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=235 NoLocalGuestEffect=408 Boundary=5 entries=438 descriptors=546 historical=209 covered=438
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.51s
```

New falsifier output from the restored green run:

```text
test oracle::mutations::declared_entry_without_descriptor ... WEAK declared_entry_without_descriptor: PASS
STRICT declared_entry_without_descriptor: Err(UncoveredEntry("Scan.callback"))
CONTROL declared_entry_without_descriptor: PASS
test oracle::mutations::descriptor_for_undeclared_entry ... WEAK descriptor_for_undeclared_entry: PASS
STRICT descriptor_for_undeclared_entry: Err(UnknownDescriptorEntry("OracleUndeclared.callback"))
CONTROL descriptor_for_undeclared_entry: PASS
```

`scripts/check-suspension-oracle.sh 900`: exit 0; both runs retain 4026 lifecycle obligations ([suspension-gate.log](../../../.temp/brief-599/suspension-gate.log), [gate-default.log](../../../.temp/brief-599/gate-default.log), [gate-no-default.log](../../../.temp/brief-599/gate-no-default.log)):

```text
LIFECYCLE obligations=4026 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=235 NoLocalGuestEffect=408 Boundary=5 entries=438 descriptors=546 historical=209 covered=438
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.18s
LIFECYCLE obligations=4026 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=235 NoLocalGuestEffect=402 Boundary=5 entries=438 descriptors=546 historical=209 covered=438
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.32s
```

Residue command: `rg -n -i -w "owed|debt|reconcile_owed|StaleOwed|MissingOwed" crates/rum-stdlib/tests/suspension_oracle docs/testing-strategy.md`. Exit 0 because the required documentation sentence contains `debt`; no match remains in the oracle tree ([residue-search.log](../../../.temp/brief-599/residue-search.log)):

```text
docs/testing-strategy.md:96:Coverage is exact and keeps no list of debt. Every declared guest-effect
```

The sole match explicitly states that there is no debt list. It carries or reports no tolerated debt.

`cargo test -p rum-stdlib --lib ownership_graph -- --nocapture`: exit 0 ([ownership-graph.log](../../../.temp/brief-599/ownership-graph.log)). Verbatim evidence:

```text
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7680 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 63.09s
```

`cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`: exit 0 ([certificates-true-green.log](../../../.temp/brief-599/certificates-true-green.log)). Verbatim evidence:

```text
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.57s
```

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`: exit 0 ([certificates-false-green.log](../../../.temp/brief-599/certificates-false-green.log)). Verbatim evidence:

```text
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 17.90s
```

Expected stale-certificate replay failures, before generated-cell installation:

[certificates-true-replay.log](../../../.temp/brief-599/certificates-true-replay.log) (exit 101):

```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.85s
```

[certificates-false-replay.log](../../../.temp/brief-599/certificates-false-replay.log) (exit 101):

```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.90s
```

`cargo test --workspace --no-fail-fast -- --nocapture`: exit 0 ([workspace.log](../../../.temp/brief-599/workspace.log)). The environment additionally set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, allowing the existing harness to skip native tests only when loopback binding is denied. Verbatim suite results:

```text
     Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/capture_output_lifecycle.rs (target/debug/deps/capture_output_lifecycle-c541c492bd454062)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.65s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/shared_accumulator_completion.rs (target/debug/deps/shared_accumulator_completion-067001aa2ecf9629)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.35s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.18s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.83s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1149 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.70s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1070.43s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.38s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-1756073e73a214f6)
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.27s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.54s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.48s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.55s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Native ZeroMQ skips: 9. These transport paths were skipped, not exercised passes, even where libtest reports the returning test as `ok`. Verbatim skip output:

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

Test-name sets are identical between feature builds within each phase: 49 before and 49 after, 46 unchanged. Exactly these three names leave and enter (one manifest rename and two falsifier replacements):

```text
REMOVED
oracle::manifest::installed_catalog_and_owed_ledger_are_exact
oracle::mutations::entry_missing_from_both
oracle::mutations::owed_entry_gains_descriptor
ADDED
oracle::manifest::installed_catalog_and_descriptor_coverage_are_exact
oracle::mutations::declared_entry_without_descriptor
oracle::mutations::descriptor_for_undeclared_entry
```

Full before set, valid for both builds ([tests-before-default.log](../../../.temp/brief-599/tests-before-default.log), [tests-before-no-default.log](../../../.temp/brief-599/tests-before-no-default.log)):

```text
host_facade::file_io::tests::normalize_path_falls_back_to_current_dir_for_missing_relative_paths: test
host_facade::file_io::tests::std_file_io_converts_read_and_write_errors: test
host_facade::file_io::tests::std_file_io_lists_only_immediate_directory_entries: test
host_facade::file_io::tests::std_file_io_probe_distinguishes_files_directories_and_missing_paths: test
host_facade::model::outcome::tests::failure_code_maps_runtime_request_errors_to_host_codes: test
host_facade::session::tests::panic_is_complete_response_marks_session_poisoned_and_invalid: test
oracle::fixtures::associations_roles::falsifiers::every_owner_rejects_missing_pause_and_lost_log: test
oracle::fixtures::capture_output::adjacent_trace_capture_prefix: test
oracle::fixtures::capture_output::evaluation_data_list_selector: test
oracle::fixtures::capture_output::evaluation_data_print_prefix: test
oracle::fixtures::capture_output::evaluation_data_scalar_selector: test
oracle::fixtures::capture_output::messages::body_errors_settle_matching_and_nonmatching_message_policies: test
oracle::fixtures::capture_output::messages::message_and_trace_supplemental_matrix: test
oracle::fixtures::capture_output::messages::nonmatching_messages_and_settled_fallback_and_info_are_public: test
oracle::fixtures::capture_output::quiet_message_prefix: test
oracle::fixtures::capture_output::roles::g11_admission_matrix: test
oracle::fixtures::capture_output::selectors::evaluation_data_selectors_and_preparation_run_once: test
oracle::fixtures::capture_output::selectors::report_failed_input_copy_and_release_wrong_output_and_empty_report: test
oracle::fixtures::capture_output::test_report_input_print_prefix: test
oracle::fixtures::control_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::debug_roles::audit::complete_role_descriptors: test
oracle::fixtures::debug_roles::falsifiers::duration_normalization_preserves_value_and_rejects_non_numeric_leaves: test
oracle::fixtures::debug_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::functional_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::historical_label_ledger_and_equality: test
oracle::fixtures::predicate_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::rewrite_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::rules::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::shared_completion::original_panic_witnesses_complete: test
oracle::fixtures::shared_completion::shared_completion_variants_and_lifecycle: test
oracle::fixtures::strings_roles::falsifiers::every_owner_rejects_missing_pause_and_lost_log: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_already_resumed: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_error: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_lifecycle: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_middle: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_nested: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_then_suspending_head: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_twice: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_without_head: test
oracle::manifest::installed_catalog_and_owed_ledger_are_exact: test
oracle::mutations::capture_output::lost_message_prefix_is_rejected_when_old_print_only_check_passes: test
oracle::mutations::capture_output::lost_trace_prefix_is_rejected_when_old_print_only_check_passes: test
oracle::mutations::entry_missing_from_both: test
oracle::mutations::false_tail: test
oracle::mutations::missing_prefix: test
oracle::mutations::non_suspending_fixture: test
oracle::mutations::omitted_registration: test
oracle::mutations::owed_entry_gains_descriptor: test
runner_scan_middle_and_twice: test

49 tests, 0 benchmarks
```

Full after set, valid for both builds ([tests-after-default.log](../../../.temp/brief-599/tests-after-default.log), [tests-after-no-default.log](../../../.temp/brief-599/tests-after-no-default.log)):

```text
host_facade::file_io::tests::normalize_path_falls_back_to_current_dir_for_missing_relative_paths: test
host_facade::file_io::tests::std_file_io_converts_read_and_write_errors: test
host_facade::file_io::tests::std_file_io_lists_only_immediate_directory_entries: test
host_facade::file_io::tests::std_file_io_probe_distinguishes_files_directories_and_missing_paths: test
host_facade::model::outcome::tests::failure_code_maps_runtime_request_errors_to_host_codes: test
host_facade::session::tests::panic_is_complete_response_marks_session_poisoned_and_invalid: test
oracle::fixtures::associations_roles::falsifiers::every_owner_rejects_missing_pause_and_lost_log: test
oracle::fixtures::capture_output::adjacent_trace_capture_prefix: test
oracle::fixtures::capture_output::evaluation_data_list_selector: test
oracle::fixtures::capture_output::evaluation_data_print_prefix: test
oracle::fixtures::capture_output::evaluation_data_scalar_selector: test
oracle::fixtures::capture_output::messages::body_errors_settle_matching_and_nonmatching_message_policies: test
oracle::fixtures::capture_output::messages::message_and_trace_supplemental_matrix: test
oracle::fixtures::capture_output::messages::nonmatching_messages_and_settled_fallback_and_info_are_public: test
oracle::fixtures::capture_output::quiet_message_prefix: test
oracle::fixtures::capture_output::roles::g11_admission_matrix: test
oracle::fixtures::capture_output::selectors::evaluation_data_selectors_and_preparation_run_once: test
oracle::fixtures::capture_output::selectors::report_failed_input_copy_and_release_wrong_output_and_empty_report: test
oracle::fixtures::capture_output::test_report_input_print_prefix: test
oracle::fixtures::control_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::debug_roles::audit::complete_role_descriptors: test
oracle::fixtures::debug_roles::falsifiers::duration_normalization_preserves_value_and_rejects_non_numeric_leaves: test
oracle::fixtures::debug_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::functional_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::historical_label_ledger_and_equality: test
oracle::fixtures::predicate_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::rewrite_roles::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::rules::falsifiers::every_role_rejects_missing_pause_and_lost_log: test
oracle::fixtures::shared_completion::original_panic_witnesses_complete: test
oracle::fixtures::shared_completion::shared_completion_variants_and_lifecycle: test
oracle::fixtures::strings_roles::falsifiers::every_owner_rejects_missing_pause_and_lost_log: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_already_resumed: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_error: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_lifecycle: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_middle: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_nested: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_then_suspending_head: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_twice: test
oracle::fixtures::strings_roles::parsed_source::parsed_source_without_head: test
oracle::manifest::installed_catalog_and_descriptor_coverage_are_exact: test
oracle::mutations::capture_output::lost_message_prefix_is_rejected_when_old_print_only_check_passes: test
oracle::mutations::capture_output::lost_trace_prefix_is_rejected_when_old_print_only_check_passes: test
oracle::mutations::declared_entry_without_descriptor: test
oracle::mutations::descriptor_for_undeclared_entry: test
oracle::mutations::false_tail: test
oracle::mutations::missing_prefix: test
oracle::mutations::non_suspending_fixture: test
oracle::mutations::omitted_registration: test
runner_scan_middle_and_twice: test

49 tests, 0 benchmarks
```

### 4. Stop-loss counters and open questions

Final counters: 8/8 disposition row edits (five retirements and three catalog changes); 14/14 ledger mechanism items deleted; 4/4 checks added (coverage function, strict manifest call, two falsifiers); 8 strict gate passes (format, restored default oracle, two script builds, ownership, two certificate builds, workspace); all six mandatory format/test gates green; residue search reviewed. Maintenance: 68/68 contract fingerprint cells and 4/4 certificate cells replayed; zero ownership rows changed. Two expected stale-certificate replay failures each reached green immediately after generated-cell installation; no two consecutive compiling attempts stalled, and the final stall count is zero.

Attempt record: [attempt-counters.json](../../../.temp/brief-599/attempt-counters.json); counters: [counters.json](../../../.temp/brief-599/counters.json); checkpoint: [PROGRESS.md](../../../.temp/brief-599/PROGRESS.md). The required forced red, initial formatting failure and two stale-certificate replay failures are disclosed above. Final required checks all passed. Open questions: none. Native transport coverage remains limited by the explicitly reported sandbox skips.
