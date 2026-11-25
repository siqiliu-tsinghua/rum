# Brief 796 — G15-CM4 the probe gate on the ordinary path: the complete zero-unknown inventory and repair plan

**Design-only brief (`Scope: NONE`; write only under `.temp/brief-796/`).** One gate. It continues
ruling 809 (gate c157). The released source is read-only: this brief inventories the confirmed
ordinary-path cost of the generated `cm740_probe` gate, designs the repair to the standing
cold-load pattern, measures a prototype of that repair in a scratch tree, and drafts the
implementation and re-measurement briefs. It edits nothing in live or in the sealed trees.

## Goal

When this brief is done, in `.temp/brief-796/` only:

- **The closed ordinary-path probe surface.** Every `cm740_probe::` occurrence in the armed tree
  classified by hook kind and enclosing function, with its authoring artifact (generator, composer,
  overlay row, addition or carried file), whether it can execute in a non-driven run, the panel
  labels that reach it, and its measured per-visit cost. A closure statement with zero unexplained
  occurrences, in the shape of brief 794's `PROBE CLOSURE`.
- **The driven-run lifecycle and the gate design.** Every install, transition and retirement edge
  of a driven run, its thread scope, nesting and reentrancy, and a cold-load static gate that is
  provably conservative (it may over-take the slow path, never miss a drive), with the fast path
  at one relaxed load and the ledger walk behind a `#[cold]` slow path.
- **A measured prototype.** The proposed gate applied in a scratch copy of the armed tree, built
  as one release binary with one codegen unit at a stable path, and measured at the two
  `stringreplace-guarded-hit` sites at N=50/100: the slopes must fall from 112 inclusive Ir per
  call to the declared budget; the driven path must still drive and record; the focused probe
  tests must stay green.
- **The render and seal cascade.** The exact source of truth that authors the module and the hook
  sites, the edit set, the re-emission sequence through the 165-row overlay table, the nine
  additions, the class table, the 152 chain, F13, the four postimages, the seal `57e9acbd…`, the
  role manifest and the R4 re-install, with the 794 attribution rows and witnesses reconciled.
- **The drafted briefs.** The implementation/re-render brief and the A/B completion brief in
  `TEMPLATE.md` shape, and the open questions with a recommended answer each.

## Context pointers

- Ruling 809 and brief 795: `evidence/decision.json`, `evidence/callsite-attribution.json`,
  `evidence/probe-selection.json`, `evidence/ir-reduction.json`, `logs/verdict.stdout`, the 24
  profiles under `callgrind/stringreplace-guarded-hit/`, and the saved binaries `bins/{base,new,
  control}`; the module's measured cost is `driven` self 2 / `run_is_driven` self 82 /
  `lexical_current` self 86 per call pair, 112 inclusive Ir per call at
  `replace.rs:38`→`0x693890` and `:775`→`0x6960cf` (target `0xbbbcf0`), one call per site per
  iteration, base and control zero.
- The module in the armed tree: `crates/rum-runtime/src/lib.rs`, the `pub mod cm740_probe` span
  (armed line 22 to 6,173); `run_is_driven` (`384-405`), `driven` (`403-405`), `lexical_current`
  (`315-321`), `enter` (`~430-445`) and the `enter_slow` cold path, the `around*` family, the
  install site (`~254-255`), `retire_actual_run` (`112-121`) and every `current.set(false)`
  (`1344, 1568, 2019, 2187, 2270, 4523, 4536, 4663, 5131, 5517, 5533`).
- The sanctioned pattern: `crates/rum-runtime/src/runtime/profiling/config.rs:549`
  `publish_cold_load` (the 0/1/2 `AtomicU8` form) and its users; `evaluation_cache/profile/mod.rs`
  `EVALUATION_CACHE_PROFILE_ACTIVE` for the "active" publication.
- The renderer: `.temp/brief-794/tools/render_tools/{ablate.py (396d9474…), canonical.py
  (6f39d69b…), fn_split.py (4971a553…), recorder_removal.py (0f1d3c7b…)}` — the function-level
  composer and its inverse passes (they locate the module span and emit the `driven()` dispatch);
  `.temp/brief-768/generator.py` and `.temp/brief-769/generator.py` (the class-table generator;
  the frozen 768 copy is not edited); the 763/784 render-repair chain; the seal tools
  `.temp/brief-792/tools/{adopt.py,seal.py}`.
- The sealed tables and receipts: `crates/rum-stdlib/tests/suspension_oracle/mutations/
  source_auth/overlays.tsv` (165 data rows; the `crates/rum-runtime/src/lib.rs` payload is
  243,761 B / 6,173 lines, hex in columns 4-5), `additions.tsv` (9), the class table (2,422
  targets / 20,625 rows), `emission_chain.tsv` (152), the F13 table (68 records, `5d27e91b…`),
  the four postimages, the role manifest (92 rows / 90 files); brief 794's `attribution.tsv`
  (120 rows, `280e9ee3…`), `probe-attribution.tsv` (`a98f1bfe…`), the 26 shapes and 14 owners.
- Briefs 703 and 708 (the timing reducers, the layout control, the Callgrind slope settlement)
  and brief 795's stage B/C protocol, which the re-measurement reuses.

## Scope

- You MAY write only under `.temp/brief-796/`: `evidence/`, `scratch/` (the prototype source copy,
  external target and bins), `logs/`, `PROGRESS.md`, `report.json`.
- MUST NOT touch: live, `crates/`, the four 793 images, the sealed tables, `.git`, `Cargo.lock`,
  any `docs/` file, brief 795's evidence, or the 794 instrument's inputs. No network, no new
  dependency, no new `unsafe`, no default-path cost in live.
- The prototype edit is confined to a scratch copy under `.temp/brief-796/scratch/`; the source of
  truth is never edited. No workspace suite, no epoch, no standing-inventory replay (the released
  source is read-only).

## Stage A — the closed ordinary-path probe surface (1,000 s)

1. Scan the armed tree for every `cm740_probe::` occurrence (the module's own definitions and
   every call outside it), and classify each call by hook kind: `driven`, `enter` / `inactive`,
   `around` / `around_ok` / `around_some` / `around_hit`, the `hit` / `ordinary` / `pause` /
   `returned` recorders, and the four `session.rs` facades (`cm740_enter`, `cm740_ordinary`,
   `cm740_hit`, `cm740_driven`). For each: `path:line`, enclosing function, the emitted dispatch
   text, and whether a non-driven run can execute it.
2. For each ordinary-reachable site, give the measured per-visit cost: the inclusive and self Ir
   of the called hook in the saved `bins/new` (disassembly plus the 24 profiles where present),
   and the number of calls per panel iteration for every panel label that reaches it (the 120
   labels: `valueq-undefined-symbol-definitions-8000` excluded per 637; the four selected
   StringReplace labels and their same-tier neighbours). Where a site is not covered by any
   profile, say so and bound it from the disassembly and the call edges — never by silence.
3. Name the authoring artifact for every site and every module text region: which generator,
   composer or carrying file writes it, which overlay row or addition holds it, and which frozen
   pin (the 794 tool hashes, the 768 generator, the 792 seal tools) governs it.
4. Closure: one line in 794's shape, e.g. `PROBE GATE SITES expected=N observed=N outside=0
   unexplained=0`, plus the per-kind counts and the per-kind ordinary reachability. A site set
   that does not close is a STOP.

## Stage B — the driven-run lifecycle and the gate design (1,200 s)

1. Enumerate every edge that makes a run driven or not: `l.current`, `run_authority`,
   `RunAuthority.current` / `.queued` / `.episode`, `pair_stages`, `retire_actual_run`, the
   `pending` handoff, and every `current.set(false)` site; the thread-local `LEDGER` and the
   harness entry that installs a driven run (name the exact test/harness paths and functions).
   State, for each edge, whether the run is visible to the thread that evaluates the guarded
   StringReplace loop, and give the nesting/reentrancy model.
2. Specify the exact predicate `run_is_driven` must preserve (`lexical_current`'s five equalities,
   `run_current`, `pair.episode.is_some()`), and design a gate that is a **conservative
   super-set**: it may route a non-driven caller to the slow path, but it must never answer
   "not driven" while any driven run exists that the slow path would have accepted.
3. Design the fast path in the existing `publish_cold_load` shape: one relaxed load of a static
   count/flag in front of the TLS ledger; the ledger walk in a `#[cold] #[inline(never)]` slow
   path; the placement of `#[inline]` / `#[inline(never)]` on `driven`, `enter` and the
   `around*` family so the fast branch inlines and the slow path stays cold and attributable;
   the install/retire maintenance of the count (fetch_add/sub, Relaxed; the balance argument);
   and the hoist question — may a per-iteration site hoist the check to function entry? Prove the
   harness cannot install a drive mid-evaluation on the same thread, or reject hoisting with the
   measured reason.
4. Declare the fast-path budget: the maximum instructions per site visit beyond the call edge
   (target: a relaxed load, a compare and a predicted-not-taken branch; state the number the
   prototype must meet and the Callgrind symbol that will show it).
5. Falsifiers, each shipped with the design: (i) a driven run still drives — name the existing
   probe tests and the harness that must stay green, and the byte-identity bar for the recorded
   ledger; (ii) the ordinary fast branch is taken — a Callgrind/self-Ir control on a non-driven
   run and its driven twin, showing zero ledger-walk Ir in the former; (iii) a forced-slow mutant
   (gate always true, or the count forced nonzero) turns the cost gate red at the two
   StringReplace sites; (iv) the count's install/retire balance is asserted by a test.

## Stage C — the cascade, the reconciliation and the prototype (1,600 s)

1. Write the exact edit set in the source of truth: the one-function change to `run_is_driven`
   (and its slow helper) plus the count maintenance at the named edges; confirm whether any other
   emitted text changes. State which artifact must be edited so the next render reproduces the
   bytes (the composer template or the carried module text), and that no generated table or
   inventory is ever hand-edited.
2. Walk the re-emission sequence with its tools and byte bars: the `lib.rs` overlay payload, the
   165-row overlay table, the additions, the class table, the 152 chain, F13 (68 records,
   `5d27e91b…`), the four postimages, the role manifest, the seal (`57e9acbd…`), the four role
   pins, and the R4 re-install (190 crates paths) with the live suite. For each step, name the
   tool, the input, the expected receipt shape and the STOP on a mismatch.
3. Reconcile brief 794: say which of the 120 rows, 26 shapes, 14 owners and three probe witnesses
   change with the new module text, which absorb it unchanged, and what the classifier's
   `PROBE CLOSURE` must print after the re-render; name the `Local` rows (the ledger
   `check_entry` and the two `reconcile.rs` functions) explicitly.
4. Prototype, in `scratch/` only: copy the armed tree, apply the proposed gate, build one
   `rum-cli` release binary with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` into an external target
   at a stable path, and run the 50/100 Callgrind probe on `stringreplace-guarded-hit` (at least
   two paired draws per arm), reusing brief 795's saved `bins/new` profiles as the unmodified
   arm. The two sites' slopes must fall to the declared fast-path budget (compare against
   21,905.12 Ir per iteration and 112 inclusive Ir per call); report every slope, the residual
   fast-path self Ir at the gate symbol, and the driven-path check. Run the focused probe tests
   in the scratch copy and record the exact verdict lines. If the prototype does not meet the
   budget, STOP with the measured reason and the revised design.

## Stage D — the plan and the drafts (700 s)

- Decide among: (a) the runtime cold-load gate (default), (b) a compile-time default flip for the
  release profile (`ENABLED` false unless an explicit measurement cfg is set), (c) hoisting the
  per-iteration checks, and any combination, with the measured evidence and the hard line as the
  bar. Recommend one and say what it costs the campaign's driven measurements.
- Draft the implementation/re-render brief and the A/B completion brief in `TEMPLATE.md` shape:
  goal, context, scope, stages with time carves, the exact bars (the fast-path budget; the two
  slopes; the whole-panel instruction delta against the control envelope; all four selected
  labels settled; zero unexplained), the STOP rules, the report format, and the budget drawn
  from this brief's measured receipts.
- List every open question with a recommended answer; a question without a recommendation is not
  a plan.

## Verification and acceptance

A pass requires: the closed surface with zero unexplained occurrences; the lifecycle enumeration
with every edge cited; the gate design with the conservative-super-set proof and all four
falsifiers; the cascade with each step's tool and bar; the 794 reconciliation naming every
affected row; the prototype's slopes at or under the declared budget with the driven path green;
and the drafted briefs complete enough to dispatch without invention. Records must quote the
exact lines they rely on. A missing site, an unproved gate property, an unmet prototype budget or
an inexpressible cascade step is a STOP.

## Checkpointing and STOP rules

After A, B, C and D, update `.temp/brief-796/PROGRESS.md` with the completed proofs, saved
evidence, the exact next command and the director handoff. Bound every long or child-spawning
command with `scripts/run-bounded.py --timeout SECONDS -- …`, sizing limits from the previous
same-shape receipt and the stage carve.

STOP, with the fact named, on: a probe occurrence that cannot be classified or attributed; an
ordinary-reachable site with no cost bound; a gate property that cannot be proved conservative;
a prototype that misses the declared budget; a cascade step that cannot be expressed without a
hand edit of a generated table; a required live or sealed edit; any write outside
`.temp/brief-796/`; any refused git attempt (disclose it); or a projected overrun.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config; the fence logs every attempt and the
  report discloses it. The director owns git.
- No network, new dependency, new `unsafe`, live source edit, sealed edit or inventory hand edit.
  Preserve brief 794's tools and evidence, brief 795's evidence, and all earlier trees.
- `.temp` only, never `/tmp`; never read `~/.config/ccds/api-key`.
- This is design only: no workspace suite, no epoch, no re-freeze, no R4 re-install (the
  implementation brief does those under its own gate). Q25's thirteen rows stay named stops;
  briefs 744/745/746 remain held; CM5 is unauthorized.
- The prototype is disposable: its sources, target and bins stay under `.temp/brief-796/scratch/`
  and are never installed.

## Budget

Focused **4,500 s** (A 1,000 / B 1,200 / C 1,600 / D 700) + workspace 0 + epoch 0 = **4,500 s**.
Funded by the **2,986 s returned by brief 795** plus an explicit **1,514 s** top-up under the
standing 2026-11-19 ruling → reserve 0 at the carve. Unspent returns; a projected overrun stops
for a director ruling, never a scope cut.

## Report

`rum-report/1` at `.temp/brief-796/report.json`; every attempt with its actual argv, exit status,
receipts, checks and measurements; the observed counts, scope and any refused git attempt;
verbatim verdict lines from saved brief-local logs, including STOP evidence. Run
`python3 scripts/report.py check 796` before handoff.

## Amendment 1 (gate c158) — the Stage A STOP accepted; Stage A reopens with a measured-bound method, the surface names the module's internal families, and the brief resumes in the same session

The first dispatch stopped correctly at Stage A's cost rule. It delivered:

- the textual census — `907` literals (`839` Rust, `43` Markdown, `25` TSV) plus `8` supplemental
  facade spellings, every one carried by `evidence/occurrences-initial.tsv`;
- the measured panel surface — `120` labels (`119` eligible, the `637` exclusion
  `valueq-undefined-symbol-definitions-8000` present), `32` physical `driven` sites in `77`
  labels, `122` timed windows, `149` label×site rows at **112 inclusive Ir per visit** (self
  `1`), the two guarded StringReplace sites once per iteration, and the fixed `859`-Ir first-entry
  cost at `eval_request` in the eight armed captures with base and control zero;
- and the STOP — `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:52` runs
  `control::resume::<F, _>` before `driven()` at `:57`; `TypedAdapter::resume` is the ordinary
  `ErasedFrame` implementation (the driver calls it at `suspension/core/driver.rs:79` and `:117`);
  `control::resume` (armed `lib.rs:624-646`) unconditionally touches `FRAMES` and `RESUMED`; it
  has no standalone symbol and is inlined into `97` adapter bodies; none of the `24` profiles or
  `149` panel rows reaches any of them.

The bar was mine and it was incomplete: Stage A item 1's hook-kind list named the
`driven`/`enter`/`around`/recorder families and the four `session.rs` facades but not the
module's internal `control`/`class_lease`/`q25` helpers, and Stage A item 2's "bound it from the
disassembly and the call edges" assumed a helper with a symbol to disassemble. The site is real,
ordinary-reachable and unbounded; the STOP is the rule working. The continuation keeps the
delivered evidence byte-untouched and reopens Stage A with:

1. **The complete family list.** Every `cm740_probe::` helper family is in scope, including
   `control::{resume, active, emit_resumed, authority, source}`, `class_lease::*` (with
   `engaged()`'s `ENABLED && TLS FLAG` shape), `q25::*`, `frame`,
   `driven`/`run_is_driven`/`lexical_current`, `enter`/`inactive`, the `around` family, the
   recorders and the four `session.rs` facades (plus `call_unprotected_cm740_driven`). For each
   site: `path:line`, enclosing function, guard shape (none, compile-time `ENABLED`, TLS flag,
   TLS ledger), ordinary reachability, and the per-visit cost.
2. **The measured bound for a symbol-less helper.** A positive-control **undriven**
   suspend/resume workload that executes at least one of the 97 `ErasedFrame::resume` bodies; the
   typed suspension tests (`crates/rum-runtime/src/runtime/suspension/typed/tests.rs`,
   `tests/live.rs`) are candidates, and the run must print the entry count so a non-executed body
   cannot be mistaken for zero cost. Measure the armed source against a scratch mutant whose only
   difference is the hook call itself (for example `control::resume` → `None`,
   `note_typed_consumer` → nothing, `control::active` → `false`), both at one path with
   `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, with at least two paired draws; the per-visit cost is
   the inclusive-Ir delta divided by the paired call count. The instantiations not reached are
   bounded by the shared generic body (`F` enters only `type_name::<F>()` on the non-empty path)
   or measured individually where that argument fails. Where a helper does have a symbol (for
   example `class_lease::note_typed_consumer`, size `0x18`), a verified static bound computed
   from its disassembly and callee spans is acceptable.
3. **The panel table covers every gate entry.** Calls to `run_is_driven` reached from
   `enter`/`around`/`driven` are priced as the `driven` calls are, or proven not to execute; the
   whole-panel `+80,382,406` delta is attributed by site or the residual is named with its
   measured bound.
4. **The closure.** `ORDINARY SURFACE sites=N measured=M bounded=K unexplained=0`, with the
   per-family guard shape and reachability. A site without a bound is still a STOP.
5. **Stages B–D gain the surface.** Stage B's cold-load gate covers every hook-family entry, not
   only `run_is_driven` — the fast path one relaxed load plus a predicted-not-taken branch, the
   TLS ledger and class-lease state behind the cold path, the install/retire counter balanced;
   Stage C's prototype applies the gate to the whole surface and measures the repaired
   typed-adapter family as well as the two `stringreplace-guarded-hit` sites (against `21,905.12`
   Ir/iteration and `112` Ir/call), with the driven path and the focused probe tests green; Stage
   D's decision states its cost to the driven measurements.

Resume in the same session with `CDX_FORK=01a21034-20a9-75b2-8f3b-e631a20e0a16`, at Stage A
with the items restated above (the delivered census and panel rows stand), then Stages B–D.
Preserve the first run's evidence, tables, logs, scratch and `report.json`; the new report
includes this amendment and the accepted STOP as a deviation. The four trees stay read-only;
`.temp/brief-796/` remains the only writable root.

Charged ≈1,164 s of the 4,500 s carve (unit `brief-796-cdx-20261125-085620-2577714`,
08:56:20–09:15:44, peak 2,818,207,744 B, oom 0); **3,336 s returned**; this amendment carves
**5,300 s** (A 1,800 / B 1,200 / C 1,600 / D 700) = the returned 3,336 s plus an explicit
**1,964 s** top-up. Unspent returns; a projected overrun stops for a ruling, never a scope cut.
