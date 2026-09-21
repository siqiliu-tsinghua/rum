# brief-507 — design: can a `Module` local stop minting interned text?

`Scope: NONE — no source change.` Design-only. You survey, reason, and
return a plan plus drafted follow-up briefs. You write no code.

## Goal

brief-506 measured that **every `Module` activation permanently adds one
interned string and one pair, about 120 bytes, and nothing ever reclaims
it** — the interner has no removal path. `Module` is the most ordinary
construct in the language, so on a long-lived host the interner grows with
work done, not with anything abusive.

Design whether a `Module` local can be made distinct **without minting fresh
interned text**, using the identity mechanism that already exists, and what
that costs and breaks. Return a plan, its open questions, and the follow-up
briefs.

## Why this, before the quota

The security-hardening line's HIGH finding is that interner bytes are
invisible to the memory ceiling and never reclaimed. The ruled mitigation is
a process-wide byte cap with a recoverable refusal (see `TODO.md` §The
interner ruling of 2026-09-21 and the D1-D7 rulings). brief-506's census
showed that with `Module` leaking, **a cap is a timer on ordinary
operation** rather than a defence against abuse: a long-lived host refuses
after enough legitimate work. Remove the dominant legitimate consumer and
the cap becomes a real backstop.

The cap is not cancelled. It resumes after this, sized against a system
where ordinary code does not leak.

## What is already settled — do not reopen

- **`Symbol` does not change**: not its size, `Copy`-ness, id-based
  fast-accept equality, or its lifetime-free `'static`-compatible public
  type (user ruling 2026-09-21). Any proposal that changes the
  representation is refuted before it is written.
- No reclamation from the interner, and no per-session attribution of
  interner bytes.
- **`Unique[]` is not in scope for removal.** Returning a symbol with a
  fresh *name* the caller can use is its documented behavior. If your design
  happens to help it too, say so, but do not make it a requirement.

## The mechanism that already exists, and the crux

`Symbol::with_identity(context, name, identity)` interns `(context, name)`
**once** and varies only a `u64` identity
(`crates/rum-core/src/expr.rs:80-96,115-122`). `active_symbol_for_name`
already uses it (`crates/rum-runtime/src/runtime/resolution.rs:128-138`),
and serde already serializes `identity` as its own field
(`crates/rum-core/src/expr.rs:287-320`), so snapshots already carry it.

`Module` does not use it. Its chain is `scoping/lexical/module.rs:46-50` →
`allocate_fresh_symbol` (`scoping/unique.rs:81-82`) →
`allocate_generated_symbol` → **`Symbol::new` with `format!` fresh text**
(`scoping/unique.rs:36-42`).

**The crux: today the `$n` suffix is interned text. Can it become a display
concern instead** — the symbol carries `("Global\`", "x")` plus identity
`n`, and renders as `x$n` when shown? Answer that with evidence, and if the
answer is no, say exactly what forbids it.

## The questions that decide the plan

### 1. What actually depends on a gensym's interned text

Inventory it, do not reason about it in the abstract. At minimum:
`Display`/`qualified_name`/`fully_qualified_name`
(`crates/rum-core/src/expr.rs:140-180`), trace and message rendering,
`Information` and name introspection, `SymbolName`-style predicates and any
pattern that matches on a name string, the gensym GC's candidate tracking
(`crates/rum-runtime/src/runtime/scoping/gensym_gc.rs`), `symbol_generations`
(`crates/rum-runtime/src/runtime/resolution.rs:128-138`), snapshot and
restore, and sealed scoping's bit-62 identity partition
(`docs/dev/sealed-scoping-design.md`).

The director found **five** source locations asserting a printed `name$n`
(`crates/rum-host/src/lifecycle_contract_tests.rs:183`,
`crates/rum-runtime/src/runtime/tests/scoping_unique.rs:17,21,50`,
`crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs:250`) — treat
that as a lower bound found by one regex, not as the inventory.

### 2. Does anything round-trip the text back into a symbol

A rendered `x$3` that can be **re-parsed** into a symbol is the dangerous
case: display-only identity breaks it, because parsing `x$3` would produce
the interned text `x$3` rather than `x` with identity 3. Search for it
honestly — source parsing, snapshot restore, `ToExpression`, message
substitution, anything that takes a printed name back. If such a path
exists, say whether it must keep working and what that forces.

### 3. Identity allocation and the bit-62 partition

Sealed scoping partitions identity space at bit 62 and keeps sealed
identities out of `symbol_generations`. Say how a per-activation `Module`
identity is allocated, whether it collides with that partition or with
`Unique[]`'s, and whether identities can be **reused** after a gensym is
swept — and if they can, what stops a stale `Symbol` from silently denoting
a live one. That hazard killed the reclaimable-interner option in brief-504
and it applies here in miniature.

### 4. Cost

`module-local` is a microbench label
(`scripts/core-language-microbench.sh:497`). Identity allocation should be
cheaper than `format!` plus interning, but say whether it is measured,
inferred or a guess — and specify the A/B the implementation must pass,
using the instruction-count recipe rather than the timing panel.

### 5. What it does to the census

If `Module` stops interning, restate what the census's numbers become:
which workloads still grow the interner, and what the dominant remaining
legitimate consumer is. That number is the input to sizing the cap later.

## What I want back

1. The plan: ordered steps, files and functions per step, gate test per
   step, and how each ruled invariant stays intact.
2. The dependency inventory from §1 and §2, produced rather than estimated.
3. A clear verdict on the crux, with what would falsify it.
4. Risks and open questions, each named as a decision for the director
   rather than resolved silently.
5. Drafted follow-up briefs in `TEMPLATE.md` shape, as raw material.

Every claim carries `path:line` evidence.

## Method requirements

- Read `AGENTS.md`, `TODO.md` §The interner ruling of 2026-09-21 including
  the census entry, and brief-506's report.
- **Re-probe rather than inherit.** The director's last premise about
  `Module` was wrong and brief-506 caught it; assume nothing stated here
  about a call path without confirming it, including the five assertion
  sites above.
- Where you assert a cost, say whether it is measured, inferred or a guess.
- Name what would falsify your recommendation. **A refutation is an
  acceptable answer** — if `Module` cannot stop interning without breaking
  documented behavior, say so, and the line goes back to sizing the cap with
  that fact documented.

## Hard rules

- **No source change.** No `.rs` file, no script, no fixture, no doc except
  appending your report to this file. Scratch under `target/brief-507/`.
- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. Bound anything long with
  `scripts/run-bounded.py --timeout SECONDS -- <command>`, sized from a
  measured run of the same shape.
- Paste verbatim evidence for anything you ran; never quote a number you
  could not take.

## Report format

1. The plan.
2. The dependency and round-trip inventories.
3. The verdict on the crux and what would falsify it.
4. Risks and open questions, as director decisions.
5. The drafted follow-up briefs.

## Execution report — brief-507 (2026-09-22)

### 1. The plan

**Recommendation: conditional feasibility, not a two-line replacement.** A
Module local can share its original interned pair and carry a distinct
numeric identity without changing `Symbol`'s representation or equality.
But **changing only allocation and Display is refuted**: ordinary source,
name-based operations, and the actual host snapshot implementation depend on
the old text. The proposed implementation below requires the director's
explicit decisions in section 4; none of those decisions is made by this
report. Evidence: `crates/rum-core/src/expr.rs:118`,
`crates/rum-core/src/expr.rs:250`,
`crates/rum-runtime/src/runtime/namespace_tools/names.rs:152`,
`crates/rum-host/src/session.rs:185`.

There is also a newly confirmed remaining automatic consumer: **one- and
two-argument `Enclose` allocate fresh interned tag text**. A Module-only
repair cannot justify the claim that ordinary execution no longer grows
the interner. This is a separate director decision, not an expansion of
this brief's implementation scope. Evidence:
`crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:14`,
`crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:118`;
the executed census is reproduced in section 2.

#### Ordered implementation steps and gates

These steps describe the proposed **display-only Module name** branch. If
the director requires existing spelling-based access to remain an unlimited
identity round-trip, step 1 must select a different design before any of
steps 2–5 is dispatched. The current language guide promises fresh locals
and closure capture, but does not specify a suffix or `$ModuleNumber`;
other surfaces still have their own contracts. Evidence:
`docs/guide/scoping-and-contexts.md:44`,
`docs/reference/builtins.md:5306`, `crates/rum-host/src/session.rs:339`.

| Step | Files and functions; proposed work | Gate before moving on |
|---|---|---|
| 1. Set the observable contract | Decide source/string round-trips, `SymbolName` versus raw Rust `name()`, typed host symbols, name collisions, ordering, reset and import identity lifetime, and persistence. Record the selected semantics in the follow-up brief and update the guide/catalog/rustdoc with the implementation. Existing boundaries: `docs/guide/scoping-and-contexts.md:44`, `docs/reference/builtins.md:3331`, `crates/rum-host/src/session.rs:177`, `crates/rum-host/src/session.rs:344`. | The director resolves M1–M6 below. Do not reinterpret failed compatibility probes as success. No source change is needed for this decision. |
| 2. Add a distinct identity kind and name operations, initially without switching Module | In `crates/rum-core/src/expr.rs`, add classification and a safe same-text identity constructor/helper; add a small identity/name helper module if needed. Preserve the three fields at `crates/rum-core/src/expr.rs:36`, the equality at `crates/rum-core/src/expr.rs:250`, and hashing at `crates/rum-core/src/expr.rs:258`. Add a rendered short-name writer and comparison operations; update `qualified_name`, `fully_qualified_name`, `Display`, and `cmp_qualified_name` only for the approved Module tag (`crates/rum-core/src/expr.rs:170`, `crates/rum-core/src/expr.rs:187`, `crates/rum-core/src/expr.rs:322`). Keep the existing serde fields (`crates/rum-core/src/expr.rs:288`). | Core tests prove identical interned id across many Module identities, unequal full symbols, 24-byte `Copy + Send + Sync + 'static` layout, no Drop, unchanged ordinary/Remove/sealed/alpha rendering, suffix rendering, comparator consistency, and serde round-trip. A diagnostic census proves no new pair/string after the base pair exists. Run the instruction gate described below because common symbol helpers can affect the default path. |
| 3. Make consumers distinguish base text from the logical displayed name | Audit and convert the inventory in section 2. Principal owners: `namespace_tools/names.rs` (`eval_symbol_name`, `names_matching_patterns`, `resolve_names_matching_expr`, `exact_symbol_name_exists`), `messages/message_format.rs`, `messages/builtins_messages_inspect.rs`, `session/session_api.rs` (completion/inspection), core/system-name recognizers, matcher atomic head tests, options, ordering and text-size preflights. Return matching **Symbols** from internal name selection rather than rendering and resolving them again (`crates/rum-runtime/src/runtime/namespace_tools/names.rs:152`). Do not implement `name()` by leaking a synthesized string. | SymbolName/Names/NameQ/Information/message/trace/completion tests; wildcard Clear/Remove/Save targets affect the correct local; ordinary `x`, a local based on `x`, and ordinary `x$N` remain distinct as ruled. Localized `True`, `Integer`, `List`, `Slot`, `$Context`, option names, and pattern heads do not acquire the original system symbol's behavior. Core/runtime/stdlib/host tests and reference drift tests; instruction gate on ordinary evaluation and sorting as well as Module. |
| 4. Preserve textual persistence before enabling the allocator | In `session/session_api.rs::{snapshot_definitions_source,render_snapshot_statement}` and `package_tools.rs::{render_saved_symbols,render_saved_symbol,symbols_in_context}`, introduce a shared Module-reference rendering plan. Collect full Module Symbols across selected definition keys and expressions, map each distinct identity to one image-local placeholder, and render a lexical restoration wrapper. Compose with the existing sealed relinking path; keep sealed-free, Module-free output unchanged. Relevant current paths: `crates/rum-runtime/src/runtime/session/session_api.rs:352`, `crates/rum-runtime/src/runtime/session/session_api.rs:432`, `crates/rum-runtime/src/runtime/package_tools.rs:410`, `crates/rum-runtime/src/runtime/package_tools.rs:393`. | Actual `HostSession::snapshot_to_bytes`/`restore_from_bytes` round-trips of two same-base closures, shared references, cyclic/mutually referring definitions, held references and call heads; repeated restore into a used process; a fresh Module after restore; sealed package plus Module closure; Save/DumpSave/Get selection behavior. No test substitutes raw serde for the public text snapshot. |
| 5. Switch only Module and maintain lifetime invariants | Add a Module-specific allocator in `scoping/lexical/module.rs` or a sibling; replace its call at `crates/rum-runtime/src/runtime/scoping/lexical/module.rs:49`. Do **not** globally replace `allocate_fresh_symbol`: Unique and Enclose also call it (`crates/rum-runtime/src/runtime/scoping/unique.rs:93`, `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:120`). Update `definition_clear.rs::mark_symbol_removed` so a Module tag cannot enter the base name's `symbol_generations` (`crates/rum-runtime/src/runtime/definitions/definition_clear.rs:212`); retain the GC's full-Symbol candidate/root machinery (`crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:9`, `crates/rum-runtime/src/runtime/scoping/lexical/module.rs:306`). Implement collision policy without constructing an ordinary fresh-text candidate on every activation. | Existing lexical capture, nested scopes, all initializer/body suspension and abandonment, GC reachability/watermark/rollback, Remove isolation, Unique freshness/collision/list/attribute behavior, and the new stale-Symbol tests. Cross-crate minimum: `cargo test --workspace`, bounded, plus fmt and the instruction gate. No permission to weaken the documented closure or host lifecycle contract. |
| 6. Independently falsify and recount | Fresh-process census at bootstrap, repeated Module, Enclose, Unique, dynamic Symbol, ordinary workspace binaries and the unchanged microbench source. Use a separate diagnostic build and count maximum over test processes. Resume the quota line only with this report and the director's disposition of Enclose. Existing census method: `docs/dev/codex/brief-506-interner-census.md:185`; quota rulings: `TODO.md:1213`. | Module's string/pair totals flatten after setup; the memory has not merely moved into an unbounded alias registry; all remaining positive slopes are named; independent instruction A/B passes. No cap default is selected by the worker (`TODO.md:1217`). |

**Persistence proposal, not a proven complete implementation.** The cold
renderer can emit a single `With` whose bindings each evaluate
``Module[{originalContext`baseName}, originalContext`baseName]``. Its body
contains all selected saved statements, with each old Module identity
replaced consistently by that binding's placeholder. Two different locals
with base `counter` get two bindings; repeated references to one local get
one binding. Choose placeholders absent from **all** names in the saved
image, including held/nested binders, and render them directly instead of
interning an activation-specific spelling during snapshot creation. A
deterministic slot vocabulary can be reused across images; it grows with
image vocabulary/width, not every historical Module activation. This is an
inference from lexical substitution and the existing source persistence
boundary, not a measured flat-census result. Evidence:
`crates/rum-runtime/src/runtime/scoping/lexical/module.rs:43`,
`crates/rum-runtime/src/runtime/package_tools.rs:432`,
`crates/rum-runtime/src/runtime/session/session_api.rs:453`.

A hand-written two-binding version ran successfully on current rum, with
two independently incrementing closures and shared held references
(`target/brief-507/probes/persistence-wrapper.rum:1`, output below).
That proves this basic source form works; it does **not** prove every
pattern binder, delayed definition, cyclic graph, capture-avoidance case,
selected Save subset, or composition with sealed-unit relinking. Those are
step-4 gates. If this lexical image form cannot preserve one of them, stop
for a revised persistence design; do not silently fall back to printing
`x$N` and reparsing it as an unrelated symbol. Existing sealed images already
need a separate relinking mechanism:
`crates/rum-runtime/src/runtime/session/session_api.rs:396`,
`docs/guide/scoping-and-contexts.md:166`.

#### Identity allocation and invariant ledger

The literal proposal “identity = small n” is insufficient: Remove already
uses small per-name identities. Sealed identities test bit 62, and rule-RHS
alpha identities use bit 63. A candidate encoding is **Module marker bit
61, with bits 62 and 63 clear**, and a nonzero payload below `2^61`.
Classification must test the whole reserved prefix, not just bit 61: a
large sealed or alpha counter can also have bit 61 set. This is a proposed
partition for the director, not an existing reservation. Evidence:
`crates/rum-runtime/src/runtime/definitions/definition_clear.rs:225`,
`crates/rum-runtime/src/runtime/scoping/packages.rs:8`,
`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:27`,
`docs/dev/sealed-scoping-design.md:63`.

For that encoding:

- Allocate **one identity per local binding**, not one per whole activation;
  the current loop already allocates separately for every item
  (`crates/rum-runtime/src/runtime/scoping/lexical/module.rs:46`). A payload is never returned to a free
  list on scope completion or GC. Keeping the original pair preserves the
  dense id; changing only the existing identity field preserves the ruled
  layout, Copy-ness and integer equality (`crates/rum-core/src/expr.rs:36`,
  `crates/rum-core/src/expr.rs:250`, paths prefixed by `crates/`).
- Preferred strong lifetime policy: a process-wide monotonically allocated
  Module payload, checked before exhaustion, with no reset/rollback rewind.
  Observe imported Module ids before issuing further fresh ids, or remap
  an imported graph consistently. A plain `fetch_add` that wraps is not a
  proof. A global atomic's cost and cross-worker contention are **unmeasured**.
  The existing alpha counter provides an implementation precedent, not a
  proof of safe overflow (`crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:669`).
- That preference conflicts with current **observable reset numbering**.
  Host reset reconstructs the session; the tests compare reset/fresh Module
  output. The director must authorize a contract change, choose a checked
  encoding that separates display serial from lifetime identity, or retain
  the existing session-relative equality domain explicitly. A bare
  session counter cannot prevent a stale public `Symbol` from comparing
  equal after reset, rollback, or import. Evidence:
  `crates/rum-host/src/session.rs:172`,
  `crates/rum-host/src/lifecycle_contract_tests.rs:94`,
  `crates/rum-host/src/lifecycle_contract_tests.rs:129`,
  `crates/rum-runtime/src/runtime/session/session_helpers.rs:150`.
- The existing sealed allocator explicitly permits reuse across independent
  sessions. This design does not pretend that convention is a process-wide
  uniqueness guarantee, or change it incidentally. Evidence:
  `crates/rum-runtime/src/runtime/scoping/packages.rs:420`.
- Cap Remove generations below the proposed Module partition and prevent
  Module identities from being inherited by `mark_symbol_removed`.
  Otherwise `Remove[local]` advances the **base** `x` generation into the
  Module region. Existing sealed exclusion is the model, not sufficient
  coverage (`crates/rum-runtime/src/runtime/definitions/definition_clear.rs:221`). Check sealed and alpha
  counter exhaustion against their existing partitions too; a wrap or a
  carry into another kind is an error, not reuse
  (`crates/rum-runtime/src/runtime/scoping/packages.rs:424`, `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:670`).
- Unique keeps fresh interned names and `identity: None`; that cannot equal
  a tagged Module symbol, but **display-name collisions are still possible**.
  Preserve the selected collision policy through a lookup that does not
  intern on a miss, or explicitly change that policy. Calling today's
  `generated_symbol_candidate` to check every Module spelling simply
  recreates the leak (`crates/rum-runtime/src/runtime/scoping/unique.rs:36`, `crates/rum-runtime/src/runtime/scoping/unique.rs:155`).
- No per-activation `(String,String) -> id` entry in `symbol_generations`,
  no process-lifetime display-name cache, and no reuse after sweep. These
  would respectively move the growth elsewhere or reintroduce stale-value
  aliasing. Full-Symbol sets/maps and traces already carry identity and need
  no representation change (`crates/rum-runtime/src/runtime/model/session.rs:49`,
  `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:73`,
  `crates/rum-runtime/src/runtime/model/messages.rs:27`, all under `crates/rum-runtime/src/`).

The same-text helper should copy the existing interned pointer/id and set
the checked identity, rather than call the interner again unnecessarily.
`with_identity(context,name,id)` already avoids **new retained text** for an
existing pair, but still takes the interner mutex on every call. Avoiding
that lock is a proposed optimization, not an already measured property.
Evidence: `crates/rum-core/src/expr.rs:80`,
`crates/rum-core/src/expr.rs:118`. No unsafe, lifetime parameter, extra
Symbol field, Arc ownership, interner reclamation or tenant attribution is
needed or proposed; the frozen invariants are at `TODO.md:1188` and
`TODO.md:1196`.

#### Cost and the required instruction A/B

**Measured here:** current retained interner growth, semantic probe results,
and successful build completion. **Not measured:** a candidate's speed,
instruction count, allocation count, flat interner growth or revised suite
maximum. **Inferred:** dropping fresh-name construction and insertion should
remove work from Module activation. The overall result is unknown because
identity-kind checks, name comparisons, collision checks and allocation
policy can add work elsewhere. Current work is visible at
`crates/rum-runtime/src/runtime/scoping/unique.rs:42` and
`crates/rum-core/src/expr.rs:80`; no guessed percentage is supplied.

Use the ruled instruction recipe, **not a timing-panel A/B**
(`TODO.md:1261`, `docs/dev/codex/brief-503-split-the-fourteen-percent.md:102`):

```sh
scripts/run-bounded.py --timeout <bound-derived-from-recorded-same-workload> -- \
  taskset -c 2 perf stat -e instructions:u -x, -- <plain-release-binary> <workload>.rum < /dev/null
```

Build base and candidate at the same path/toolchain/profile, save the base
binary before edits, and use two arms. Turn profiling/census/alloc-count
instrumentation off; require the `cpu_core/instructions/u` event at
`100.00` coverage, exit zero, and the workload's liveness output. Take
three samples per arm for full, half-size and matching setup controls;
report raw rows, median, within-arm spread, gross and setup-subtracted
counts. Reject silent iteration-limit truncation. These are the existing
method's requirements, not new performance results:
`docs/dev/codex/brief-502-stack-slope-and-a1-cost.md:128`,
`docs/dev/codex/brief-502-stack-slope-and-a1-cost.md:559`,
`docs/dev/codex/brief-503-split-the-fourteen-percent.md:110`.

At minimum reuse all twelve workloads listed at
`docs/dev/codex/brief-502-stack-slope-and-a1-cost.md:498`: cold/warm intern,
literal-integer, symbol-ownvalue-read, set-ownvalue, module-local,
downvalue-exact-hit, matcher-single-tiny, replaceall-simple, both symbol
sort widths, and ordinary smoke. Module's exact statement is
`Module[{x = 1}, x + 1]`, full 500 and half 250; no sink assignment or
Print is added around it (`scripts/core-language-microbench.sh:497`). Add
the exact block-local and with-local neighbors, both Module escape-chain
labels, and retained-gensym-GC workload from the same script
(`scripts/core-language-microbench.sh:498`, `:508`, `:512`, `:1066`).
Preserve whole setup/control sources for those additions. This defines
seventeen workload families; full, half and their respective setup controls
(at most four source shapes), at three repeats and two arms, is a maximum
**408 instruction process runs**, with duplicate controls
reused rather than inventing more runs. Report a regression beyond spread
on any target or neighbor; do not offset it with an average win. If counters
are unavailable, report a missing gate rather than substituting wall time.

#### What the census would become

The following is a **conditional prediction**, not a candidate measurement.
For the exact repeated one-local Module source used here, all source/base
pairs are already present in the zero-call control. A successful change
therefore removes the `+N` strings, `+N` pairs and per-suffix payload, leaving
the control totals constant across activation count. Under the unchanged
old storage formula that would be **665 strings, 665 pairs,
77,182–85,211 bytes** at 0, 1,000 and 10,000 calls, plus any separately
measured fixed new vocabulary/setup. Today's 10,000-call retained totals
are 1,173,676–1,289,225 bytes; the measured difference from its zero-call
control is 1,096,494–1,204,014 bytes, or 109.6494–120.4014 bytes/call.
This is per **allocated local** in this particular workload, not a universal
per-Module invocation constant. Evidence: `target/brief-507/census-summary.txt:1`,
the allocation loop at `crates/rum-runtime/src/runtime/scoping/lexical/module.rs:46`, and the original
formula at `docs/dev/codex/brief-506-interner-census.md:185`.

| Workload after a Module-only repair | Expected remaining growth and evidence |
|---|---|
| Fixed source repeatedly evaluates Module | No fresh interner entry per local after setup, conditional on the allocation, collision and renderer gates above. Returned closures still retain their real definitions; that is live runtime state, not interned suffix text (`crates/rum-runtime/src/runtime/scoping/lexical/module.rs:306`, `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:137`). |
| Unique | Still unbounded by design. Brief-506's **historical** 240,000-call row is 29,560,226–33,231,103 retained proxy bytes, averaging 122.846396–138.108262 incremental bytes/call. No new Unique census was run here (`docs/dev/codex/brief-506-interner-census.md:463`). |
| Dynamic Symbol/new source names or contexts | Still grows. The default unqualified lookup constructs System and Global candidate pairs; Module does not repair that ingress (`crates/rum-runtime/src/runtime/resolution.rs:101`). Brief-506's **historical** Symbol row at 240,000 calls is 52,810,274–58,578,303 bytes, 219.721596–243.721596 incremental bytes/call (`docs/dev/codex/brief-506-interner-census.md:468`, under `docs/dev/codex/`). |
| Untagged Enclose | Still an automatic leak. This report measured 10,665 strings / 10,664 pairs and 1,243,643–1,359,192 bytes at 10,000 calls, versus 664 / 663 and 77,091–85,120 at zero. That is one lazy seed plus one string/pair per call, 116.6552–127.4072 incremental proxy bytes/call at this size (`target/brief-507/enclose-census-summary.txt:1`, `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:118`). |
| Bootstrap | Module-free bootstrap does not lose any of its existing vocabulary. Brief-506's historical floor remains 662 strings / 661 pairs and 77,004–85,033 bytes before any new fixed vocabulary is added (`docs/dev/codex/brief-506-interner-census.md:258`). Recount it; do not subtract Module bytes from bootstrap. |
| Suite and panel | Must be measured again, not estimated by subtracting a guessed Module frequency. Historical maximum-over-binaries was 14,413,346–15,855,999 bytes; panel retained was 6,918,928–7,640,685. Different capacities, overlapping names, test sessions and remaining Enclose tags prevent deriving new exact totals (`docs/dev/codex/brief-506-interner-census.md:314`, `:365`). |
| Explicitly parsing generated display names; restored old snapshots | Under the display-only branch these can still deliberately create ordinary fresh text. Repeatedly rendering a name alone need not intern it. A new snapshot renderer must avoid exporting a fresh suffix as an identifier on every save/restore (`crates/rum-core/src/expr.rs:170`, `crates/rum-runtime/src/runtime/session/session_api.rs:380`, under `crates/`). |

Thus **Unique remains the intentional fresh-name builtin consumer**, but
there is no measured basis to call it the dominant remaining consumer of
all legitimate execution. Enclose is a counterexample to “only callers
explicitly requesting fresh names still grow.” No host workload-frequency
distribution was measured. The director should either schedule Enclose's
separate repair before sizing, or carry its measured demand into the cap
decision. The quota, its process-global proxy units and recoverable refusal
remain required; this report selects no default (`TODO.md:1201`, `:1217`,
`:1230`, `:1293`).

### 2. The dependency and round-trip inventories

This is an inspected inventory, not the director's regex estimate. The raw
name-call search is saved at `target/brief-507/name-call-sites.txt:1`; its
181 matching lines include a few non-Symbol `.name()` calls and inline unit
tests, so **181 is not a count of semantic dependencies**. The complete
numeric-dollar search produced **36 matching lines in 10 files**, including
inputs and helper-created Symbols as well as assertions. Those are search
results, not a claim that there are 36 separate tests. The exact searches
were:

```sh
rg -n '\.(name|qualified_name|fully_qualified_name|cmp_qualified_name)\(' crates -g '*.rs' -g '!**/tests/**' -g '!**/*tests.rs'
rg -n '\$[0-9]+' crates -g '*.rs'
rg -n 'allocate_fresh_symbol\(' crates
rg -n 'resolve_symbol_name\(' crates -g '*.rs' -g '!**/tests/**' -g '!**/*tests.rs'
```

Searches for identity producers, snapshots, string parsing, generated-name
recognition and the callers of the selected helpers supplemented those
searches. The findings below cite the actual definitions, including cases
where the name method call itself is harmless but its caller is not.

#### Name, value, rendering and lifecycle dependencies

| Dependency | Observed behavior and consequence of identity-only Module locals |
|---|---|
| Interning and equality | `Symbol::new` gives `identity: None`; `with_identity` uses the same interner and attaches a nonzero integer. Interning keys contain only the two text pointers. Equality includes id **and identity**, and hash includes both. Therefore sharing text is sufficient for distinct values; changing equality is unnecessary. `crates/rum-core/src/expr.rs:80`, `:109`, `:118`, `:250`, `:258`. |
| Raw name and context API | `name()` returns borrowed interned text; `context()` returns the existing context. A virtual suffix cannot be returned from this signature without other storage. Keep a raw base-name accessor and introduce a distinct writer/owned display-name operation; do not quietly leak on first display. `crates/rum-core/src/expr.rs:140`. |
| Display and qualified names | Both qualified-name methods currently omit **every** numeric identity; Display delegates to `qualified_name`. Rendering all `Some(id)` values with a suffix would incorrectly expose Remove generations, alpha renames and sealed ids. Tag-specific rendering is required. `crates/rum-core/src/expr.rs:170`, `:322`; `crates/rum-runtime/src/runtime/scoping/packages.rs:425`; `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs:671`. |
| ToString/InputForm/output/FullForm | Atom rendering and FullForm delegate to `Expr::to_string`, ultimately Symbol Display. Input-like messages use the same renderer. A Display change reaches these surfaces, but does not make them identity-preserving source encodings. `crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs:19`, `:31`, `:51`, `:82`. |
| SymbolName, Names and NameQ | SymbolName uses raw `name`; Names matches wildcard/regex/StringExpression/Alternatives against qualified rendered text and deduplicates strings; NameQ resolves a string back to a symbol. All three need an explicit coherent contract for virtual names, including identical displayed text belonging to distinct values. `crates/rum-runtime/src/runtime/namespace_tools/names.rs:116`, `:186`, `:322`, `:333`; `docs/reference/builtins.md:3331`, `:5306`. |
| Information, Definition, inspection and completion | Information's FullName and fallback Usage use fully-qualified text; stored definition expressions contain full Symbols. Inspection and completion expose rendered names. Their string-input resolution can lose the local separately from their correct rendering. `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:238`, `:328`, `:416`; `crates/rum-runtime/src/runtime/session/session_eval.rs:87`, `:117`; `crates/rum-runtime/src/runtime/session/session_api.rs:332`, `:728`. |
| Messages and substitutions | Placeholder `:name` uses raw `name`, `:full` and `:fullsymbol` use FQN, ordinary symbol placeholders use Display. Symbolic message tags also use raw `name`. Message keys themselves contain full Symbols. The template formatter emits text; it does not parse the substituted text. Separate string query/selector inputs do resolve names. `crates/rum-runtime/src/runtime/messages/message_format.rs:111`, `:146`; `crates/rum-runtime/src/runtime/messages/message_catalog.rs:157`, `:173`, `:223`; `crates/rum-runtime/src/runtime/messages/message_selector_parsing.rs:100`; `crates/rum-core/src/expr.rs:886`. |
| Trace and adapter rendering | Runtime trace events hold Expr/Symbol values; host conversion calls `to_string` for before/after and definitions. Thus rendering changes propagate without a new trace identity representation. No text-to-Symbol conversion is performed by the trace conversion itself. `crates/rum-runtime/src/runtime/model/messages.rs:27`; `crates/rum-host/src/model/outcome.rs:355`. |
| Equality-based matching, substitution and GC roots | Symbol maps and lexical presence compare the complete Symbol; call-head matching compares complete Symbols. Sharing an interned id alone does not merge these values. `crates/rum-runtime/src/runtime/substitution/lexical_presence.rs:65`; `crates/rum-runtime/src/runtime/matcher_and_symbols.rs:608`; `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:24`, `:64`. |
| Name-sensitive predicates and pattern heads | Atomic head matching compares `expected_symbol.name()` to `Integer`, `Rational`, `String`, `Symbol` in **two** implementations. Pattern-language recognition also matches raw name text. A local based on `Integer` must not start matching integer atoms after the change. `crates/rum-runtime/src/runtime/matcher_and_symbols.rs:598`; `crates/rum-runtime/src/runtime/model/definitions.rs:264`; `crates/rum-runtime/src/runtime/pattern_syntax.rs:34`; `crates/rum-stdlib/src/predicates.rs:411`. The current probe returns False for a Module-local `_Integer`. |
| System-symbol shortcuts | `is_system_name` ignores identity; `evaluate_symbol` checks System context plus raw name for `$Context` and other session variables. Default attributes, options, syntax metadata, stability, slot substitution, hold release and some fast paths likewise match raw names. A tagged ``System`True`` cannot become boolean True, and a tagged List cannot acquire List rendering/semantics. `crates/rum-core/src/expr.rs:156`; `crates/rum-runtime/src/runtime/eval_core/symbol.rs:10`; `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs:247`; `crates/rum-runtime/src/runtime/options/defaults.rs:20`; `crates/rum-runtime/src/runtime/namespace_tools/syntax.rs:121`; `crates/rum-runtime/src/runtime/substitution/slot_substitution.rs:49`; `crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs:34`; `crates/rum-host/src/model/outcome.rs:633`. |
| Option/property/string-key recognizers | Name-only fallback exists even outside System context, so fixing `is_system_name` alone is insufficient. Audit options' `contains_name`/`entry_by_name`, replacement options, confirmation-head recognition, and stdlib symbol-as-option-name adapters. Avoid allocating a display string on ordinary calls; use a Module-aware name comparison/view. `crates/rum-runtime/src/runtime/options.rs:42`, `:58`; `crates/rum-runtime/src/runtime/options/core.rs:50`; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:908`; `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:199`; `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:311`; `crates/rum-stdlib/src/strings.rs:117`; `crates/rum-stdlib/src/strings/template.rs:294`; codec examples `crates/rum-stdlib/src/json.rs:107`, `crates/rum-stdlib/src/xml.rs:104`. The raw call-site file includes the other codec/scripting/property adapters. |
| Ordering | `Symbol::Ord` orders stored context/name/identity. `cmp_qualified_name` promises comparison of rendered bytes but has an equal-id early return and deliberately ignores identity. Stdlib Sort uses this comparator; runtime canonical order separately compares `name()` and then identity. With virtual suffixes these are different obligations. In particular decimal lexical order of `$10` versus `$2` is not numeric identity order. Preserve the chosen visible order and Eq/Ord consistency explicitly. `crates/rum-core/src/expr.rs:182`, `:271`; `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:1044`; `crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:270`; tests `crates/rum-stdlib/src/tests/structure_ops/ordering_sets.rs:76`, `:101`. |
| Unique and nested generated names | Unique's symbol and list forms use the same fresh-text machinery as Module; the list form deliberately uses one serial for a batch. Unique of a Module value currently incorporates that value's full short spelling as its prefix. Keeping `symbol.name()` unchanged here would instead use only the base. Preserve Unique's fresh-name behavior and decide nested-local display policy explicitly. `crates/rum-runtime/src/runtime/scoping/unique.rs:81`, `:90`, `:125`; `crates/rum-stdlib/src/tests/incidental_scoping_unique.rs:24`. |
| GC candidate tracking | Candidate registration derives from the activation's actual generated Symbols; immediate reclaim and mark/sweep operate on sets of full Symbols and definition/root graphs, not a `$digits` regex. Identity reuse is neither necessary nor performed by that code. No string-based replacement of this membership proof is warranted. `crates/rum-runtime/src/runtime/scoping/lexical/module.rs:54`, `:329`, `:357`; `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:9`, `:73`, `:133`. Unique deliberately remains outside Module GC (`crates/rum-runtime/src/runtime/tests/scoping_gensym_gc/reachability.rs:525`). |
| Save context filtering | Unlike GC, `symbols_in_context` really does filter by a textual `$` followed by a digit. It would miss tagged Module locals whose base is `x`. Its current heuristic also excludes some hand-written names, so replacing it with provenance alone is an independent observable change. `crates/rum-runtime/src/runtime/package_tools.rs:393`, `:803`; `crates/rum-runtime/src/runtime/tests/package_tools.rs:302`, `:338`. |
| Remove generations | Generation keys are owned `(context,name)` strings; active lookup rebuilds a Symbol with that generation. Module ids must stay out of this map, just as sealed ids already do. Otherwise removing a local can alter the ordinary base symbol and conflict with subsequent allocation. `crates/rum-runtime/src/runtime/model/session.rs:49`; `crates/rum-runtime/src/runtime/resolution.rs:129`; `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:212`. |
| Cache identity and generated classification | The stability bloom mixes interned id with numeric identity and already classifies any `Some(identity)` as generated. This supports the representation; suffix-based family/profiling classification still changes. Do not replace full keys with base-name-only keys. `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs:65`, `:83`; `crates/rum-runtime/src/runtime/evaluation_cache/mod.rs:44`; `crates/rum-runtime/src/runtime/profiling/fingerprints.rs:3`, `:136`. |
| Diagnostics' reconstructed Symbols | Several profiling records deliberately call `Symbol::new(context,name)` to drop identity. With virtual locals they group by base instead of the previously distinct suffix. Decide the intended aggregation and label it; do not restore old grouping by permanently interning each rendered suffix. Keep diagnostics cold. `crates/rum-runtime/src/runtime/profiling/timing.rs:79`; `crates/rum-runtime/src/runtime/profiling/records.rs:282`; `crates/rum-runtime/src/runtime/profiling/presence.rs:268`; `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:279`; `AGENTS.md:112`. |
| Size/proxy calculations | Several expression/text-size estimates use context/name lengths. If an estimate bounds rendered text, it must include the virtual suffix without interning it; actual backing-text accounting must not pretend the suffix is retained. Audit the purpose of each estimate. `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:364`, `:414`; `crates/rum-runtime/src/runtime/arithmetic/canonical.rs:413`; `crates/rum-stdlib/src/structure_ops/surface.rs:280`; `crates/rum-stdlib/src/associations/missing/traversal/projection.rs:43`; other sites are in the saved name-call inventory. |

#### Actual text round-trips, including the snapshot correction

| Route | Evidence and required treatment |
|---|---|
| Direct source spelling | Syntax symbols lower through the session resolver. It currently constructs the textual name unchanged, so a previously generated `x$1` is reachable. The new probe gives `source={True, 42}`. Retaining this identity round-trip requires more than Display; losing it requires M1's explicit decision. `crates/rum-runtime/src/runtime/lowering.rs:100`; `crates/rum-runtime/src/runtime/resolution.rs:66`, `:101`; `target/brief-507/probes/direct-spelling.rum:1`. |
| Symbol[string] | Calls `resolve_symbol_name(name,true)`. The probe `SameQ[x,Symbol[Context[x]<>SymbolName[x]]]` is True today. With just the proposed allocator and renderer it would produce ordinary text `x$N`, not the tagged base. `crates/rum-stdlib/src/structure_ops/surface.rs:314`; `target/brief-507/probes/roundtrips-v2.rum:1`. |
| ToExpression/ToString | ToExpression parses, lowers using the session, evaluates, and only then applies an optional third-argument wrapper. It is not an identity decoder. The corresponding SameQ is True in the current probe; a plain display-only replacement breaks it. `crates/rum-stdlib/src/strings/text/builtins.rs:438`; `target/brief-507/probes/roundtrips-v2.rum:1`. |
| Internal Names-to-symbol path | `resolve_names_matching_expr` calls `names_matching_patterns`, then reparses the returned name strings through `resolve_symbol_name`. Wildcard Clear/Remove and pattern-based Save use it. This must be changed to retain full Symbols even if general source reparsing is deliberately no longer an identity round-trip. `crates/rum-runtime/src/runtime/namespace_tools/names.rs:152`; `crates/rum-runtime/src/runtime/definitions/definition_analysis_helpers.rs:114`; `crates/rum-runtime/src/runtime/package_tools.rs:344`. |
| String inspection/messages/attribute targets | Symbol-target normalization and message-catalog queries resolve strings. They must follow the selected exact-name policy; direct Symbol arguments already retain identity. Template substitution itself is one-way rendering, not a parser. `crates/rum-runtime/src/runtime/messages/message_catalog.rs:173`, `:223`; `crates/rum-runtime/src/runtime/attributes.rs:181`; `crates/rum-runtime/src/runtime/messages/message_format.rs:146`. |
| Public host snapshot and restore | **The brief's serde-based snapshot premise is false for this API.** `snapshot_to_bytes` calls `snapshot_definitions_source` and takes UTF-8 bytes; restore resets and evaluates that source. The fast snapshot path renders saved statements using Display. No Symbol serde field is consulted. Persisted definitions/attributes must continue to work; dropping local identity silently is not acceptable. `crates/rum-host/src/session.rs:177`, `:185`, `:201`; `crates/rum-runtime/src/runtime/session/session_api.rs:74`, `:352`, `:380`; `docs/host-embedding-and-hardening.md:95`. |
| Save/DumpSave/Get | Save and DumpSave emit textual statements; Get evaluates source. The executed explicit-symbol/pattern-selected dump contains `counter$1` in both closure and definition, and Get resumes at 2 then 3. The new representation needs an explicit persistence plan here too, including the selection contract. `crates/rum-runtime/src/runtime/package_tools.rs:284`, `:410`, `:432`; `target/brief-507/probes/persist-v2.rum:1`; `target/brief-507/probes/closure-save.rum:1`; `docs/reference/builtins.md:1469`, `:4564`. |
| Host typed symbols and function heads | Returned Symbol values become `HostTyped::Symbol(symbol.to_string())`; typed input resolves that string. `HostSession::call` documents equality of resolution with source text. Changing only Module makes a returned local fail to designate itself when passed back. The current string-shaped host variant cannot carry an extra numeric identity implicitly. Decide the limitation, or authorize an explicit opaque/structured symbol value extension; do not treat the current variant as an identity handle. `crates/rum-host/src/model/outcome.rs:509`, `:622`; `crates/rum-host/src/session.rs:339`, `:435`. |
| InputForm transport payload | `restore_transport_expr` parses UTF-8 when the original form is InputForm, then lowers it. This is another real source round-trip, not serde. It follows M1 unless separately given an identity-preserving format. `crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:67`. |
| Symbol serde | This route really does serialize raw context, raw name and identity and reconstruct with `with_identity`. Shape can remain unchanged; freshness after importing ids still needs allocation reservation/remapping. It does not repair the other rows. `crates/rum-core/src/expr.rs:288`. |
| Loader/evaluation rollback snapshots | These are in-memory clones of full Symbol-bearing structures and numeric counters. They preserve values directly, but restoring the generator counter can reuse identities if a new allocator is placed there naively. Restore GC candidates too, without rewinding a never-reused allocator. `crates/rum-runtime/src/runtime/session/session_helpers.rs:95`, `:128`; `crates/rum-runtime/src/runtime/session/session_state.rs:290`; `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:92`. |
| Sealed-source persistence | Separate from ordinary text and serde: source-unit registration is emitted, and escaped sealed references are relinked. Adding a generic parser for numeric identity must not expose bit-62 private values. `crates/rum-runtime/src/runtime/session/session_api.rs:396`, `:442`; `docs/dev/sealed-scoping-design.md:164`; `docs/guide/scoping-and-contexts.md:166`. |

#### The five cited sites, and the larger test surface

All five cited locations exist, but they have different roles. The host
test really asserts Module's `t$1`; the three `scoping_unique.rs` sites
assert **Unique**, and the GC site constructs a raw ordinary Symbol named
`gcImmediateModule$1` to seed a stale advisory entry. That last test is an
identity-coupling test, not just a printed-string assertion. Evidence:
`crates/rum-host/src/lifecycle_contract_tests.rs:183`,
`crates/rum-runtime/src/runtime/tests/scoping_unique.rs:17`, `:21`, `:50`,
`crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs:250`.

The complete numeric-dollar search output is retained at
`target/brief-507/dollar-number-sites.txt:1`. In addition to those five it
finds the following dependencies; the nonliteral helper/round-trip checks
are listed separately so they are not missed by another literal regex:

- Module's custom context and reset/fresh output comparison:
  `crates/rum-host/src/lifecycle_contract_tests.rs:95`, `:129`, `:209`.
- Five capture-avoidance/nested-scope printed expectations:
  `crates/rum-runtime/src/runtime/tests/definitions_and_rules_part3.rs:611`,
  `:641`, `:651`, `:664`, `:697`.
- Occupied generated spellings must be skipped:
  `crates/rum-runtime/src/runtime/tests/thread_and_dispatch.rs:61`.
- Per-binding serial progression survives initializer suspension:
  `crates/rum-runtime/src/runtime/tests/session_requests/input_resume.rs:189`.
- GC tests reconstruct expected locals from spelling, including error
  payloads, nested activations and RHS guards:
  `crates/rum-runtime/src/runtime/tests/scoping_gensym_gc/reachability.rs:192`,
  `:202`, `:212`, `:227`, `:242`, `:243`, `:253`, `:254`, `:269`, `:284`,
  `:300`, `:316`; suspension print assertions at
  `crates/rum-runtime/src/runtime/tests/scoping_gensym_gc/suspension_frames.rs:108`,
  `:140`. Replace textual fabrication with actual captured symbols or an
  explicit allocator fixture only after the contract decision; preserve
  the root/watermark assertions.
- Literal dollar names can be ordinary names and explicit Save targets:
  `crates/rum-runtime/src/runtime/tests/package_tools.rs:338`.
- Unique's default spelling, collision input, batch and mixed prefixes:
  `crates/rum-runtime/src/runtime/tests/scoping_unique.rs:9`, `:38`;
  `crates/rum-stdlib/src/tests/incidental_scoping_unique.rs:24`, `:32`.
- Unique's generated name is also interpolated back into an assignment and
  evaluated after GC, without any literal `$1` assertion:
  `crates/rum-runtime/src/runtime/tests/scoping_gensym_gc/reachability.rs:525`.
- Core rendered-name comparator tests exercise identities without numeric
  suffix literals; sealed snapshot/visibility tests exercise a different
  identity domain. Keep them:
  `crates/rum-stdlib/src/tests/structure_ops/ordering_sets.rs:76`, `:101`;
  `crates/rum-runtime/src/runtime/tests/sealed_snapshot.rs:55`, `:107`, `:210`;
  `crates/rum-runtime/src/runtime/tests/sealed_packages.rs:131`, `:256`.

#### Executed evidence and limits of verification

Starting checkout was clean, HEAD
`ef5118c26bfedc40745d6eb41a97cbb91a097bb9`. Read-only
`git diff --stat f7a4217c2f9798b9909c2b4b114474f9b5ba6b79 -- crates`
produced no output: the production crate sources match the base used by
brief-506. The current core source and that census's uninstrumented saved
source have the same hash. This justified reusing its **existing** census
helper without adding or editing instrumentation. Semantics probes instead
use a newly built, uninstrumented executable. Verbatim identity evidence:

```text
ef5118c26bfedc40745d6eb41a97cbb91a097bb9
8d74114e9a3ad1b69e98a95d4095afc91048b9193894b91f74e00479ed8a67de  crates/rum-core/src/expr.rs
8d74114e9a3ad1b69e98a95d4095afc91048b9193894b91f74e00479ed8a67de  target/brief-506/expr.original.rs
```

Build command:

```sh
env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER CARGO_NET_OFFLINE=true CARGO_TERM_COLOR=never CARGO_TARGET_DIR=/home/remi/dev/rum/target/brief-507/build TMPDIR=/home/remi/dev/rum/target/brief-507/tmp scripts/run-bounded.py --timeout 300 -- cargo build --offline --locked --release -p rum-cli > target/brief-507/logs/build.log 2>&1
```

Exit 0; verbatim completion (not a performance result):

```text
    Finished `release` profile [optimized] target(s) in 25.86s
```

The 300-second build bound uses the recorded same release build's 26.68 s
completion (`target/brief-506/logs/release-build.log`). Small language
probes and census probes use the previously recorded 60-second envelope
for this same installed-session/short-source process shape; the exact
Module loop at the same sizes was already measured by brief-506
(`docs/dev/codex/brief-506-interner-census.md:507`). The new Enclose series
was run zero calls first, then 1,000, then 10,000; those bounded process
calls completed in 0.089769515, 0.128455958 and 0.624200381 seconds as
recorded by the execution tool. These are completion/bound evidence only,
not a cost comparison. No timing decision or instruction measurement was
made.

Each of the following seven semantic invocations used this exact command
shape, replacing `<label>` with `roundtrips`, `collision`, `persist`,
`roundtrips-v2`, `persist-v2`, `direct-spelling`, `persistence-wrapper`:

```sh
env -u RUM_INTERNER_CENSUS -u RUM_PROFILE_ALLOC TMPDIR=/home/remi/dev/rum/target/brief-507/tmp scripts/run-bounded.py --timeout 60 -- target/brief-507/build/release/rum-cli --file target/brief-507/probes/<label>.rum > target/brief-507/logs/<label>.log 2>&1
```

Every invocation exited 0; **the first roundtrips and persist probes had
probe errors and are not claimed as passing**. The original inputs/logs
were retained. One used an unsupported two-argument Information call; the
other supplied a relative file path that the file loader resolved relative
to the probe's directory. Corrected inputs use one-argument Information and
an absolute path inside `target/brief-507/`. Verbatim original outputs:

```text
roundtrip={"x$1", "x$1", True, True, True}
names={{"listed$2"}, True, Information[listed$2, "FullName"]}
clear={}
system-local={Hold[True$4], False, False}
remove={Hold[removedLocal$6], removedLocal}
closure={1, 2}
! Information::argtu: expected between 1 and 1 argument(s), got 2
! Information::argtu: expected between 1 and 1 argument(s), got 2
! Information::argtu: expected between 1 and 1 argument(s), got 2
! Information::argtu: expected between 1 and 1 argument(s), got 2
! msg$8::note: short=msg$8; full=Global`msg$8; symbol=msg$8
Null
```

```text
before=1
saved=$Failed
after={inc[], inc[]}
! DumpSave::io: could not write `target/brief-507/probes/target/brief-507/probes/closure-save.rum`: No such file or directory (os error 2)
! Get::notfound: could not locate `target/brief-507/probes/closure-save.rum` in target/brief-507/probes/target/brief-507/probes/closure-save.rum, ./target/brief-507/probes/closure-save.rum
Null
```

The corrected `roundtrips-v2.rum` input:

```text
Print["roundtrip=", Module[{x}, {SymbolName[x], ToString[x], SameQ[x, ToExpression[ToString[x]]], SameQ[x, Symbol[StringJoin[Context[x], SymbolName[x]]]], StringMatchQ[SymbolName[x], "x$*"]}]];
Print["names=", Module[{listed}, listed[t_] := t; {Names["listed$*"], NameQ[SymbolName[listed]], Information[listed]}]];
Print["clear=", Module[{cleared}, cleared[z_] := z; Clear["cleared$*"]; DownValues[cleared]]];
Print["system-local=", Module[{True, Integer}, {Hold[True], TrueQ[True], MatchQ[1, _Integer]}]];
Print["remove=", {Module[{removedLocal}, Remove[removedLocal]; Hold[removedLocal]], removedLocal}];
Print["closure=", (make[] := Module[{n = 0}, Function[{}, n = n + 1]]; c = make[]; {c[], c[]})];
Module[{msg}, msg::note = "short=`1:name`; full=`1:full`; symbol=`0:symbol`"; Message[msg::note, msg]];
```

Verbatim corrected output; the final `msg` diagnostic is intentional:

```text
roundtrip={"x$1", "x$1", True, True, True}
names={{"listed$2"}, True, InformationData[<|"ObjectType" -> "Symbol", "Usage" -> "Global`listed$2", "Documentation" -> None, "Messages" -> {}, "OwnValues" -> None, "UpValues" -> None, "DownValues" -> Information`InformationValueForm[DownValues, listed$2, {HoldPattern[listed$2[t_]] :> t}], "SubValues" -> None, "DefaultValues" -> None, "NValues" -> None, "FormatValues" -> None, "Options" -> None, "Attributes" -> {}, "FullName" -> "Global`listed$2"|>]}
clear={}
system-local={Hold[True$4], False, False}
remove={Hold[removedLocal$6], removedLocal}
closure={1, 2}
! msg$8::note: short=msg$8; full=Global`msg$8; symbol=msg$8
Null
```

`collision.rum` sets `x$1` and `x$2` before Module, then evaluates
`{Unique[],Unique[x],Unique[{a,b}]}`. Verbatim output:

```text
collision=Hold[x$3]
unique={$4, x$5, {a$6, b$6}}
Null
```

`direct-spelling.rum` creates a held local with a downvalue, then spells
`probeLocal$1` directly in the following statement and traces another
Module. Verbatim output:

```text
source={True, 42}
trace={Hold[Module[{traced}, Hold[traced]]], Hold[Hold[traced$2]]}
Null
```

`persist-v2.rum` increments a captured local once, DumpSaves `inc` and the
``Global`counter$*`` selection, clears `inc`, Gets the file, then calls the
closure twice. Verbatim output and generated source:

```text
before=1
saved={"Global`inc", "Global`counter$1"}
after={2, 3}
Null
```

```text
Set[inc, Function[List[], Set[counter$1, Plus[counter$1, 1]]]]

Set[counter$1, 1]
```

`persistence-wrapper.rum` exercises the proposed lexical image shape,
**on the current textual-gensym implementation**:

```text
With[{snapshotSlot1 = Module[{counter}, counter], snapshotSlot2 = Module[{counter}, counter]},
  Set[incA, Function[{}, snapshotSlot1 = snapshotSlot1 + 1]];
  Set[incB, Function[{}, snapshotSlot2 = snapshotSlot2 + 1]];
  Set[snapshotSlot1, 1];
  Set[snapshotSlot2, 10];
  Set[savedHeld, Hold[snapshotSlot1, snapshotSlot2]]
];
Print["restored-closures=", {incA[], incB[], incA[], incB[]}];
Print["shared-refs=", savedHeld];
Print["not-captured=", Hold[counter, snapshotSlot1, snapshotSlot2]];
```

Verbatim output:

```text
restored-closures={2, 11, 3, 12}
shared-refs=Hold[counter$1, counter$2]
not-captured=Hold[counter, snapshotSlot1, snapshotSlot2]
Null
```

The six census invocations used the following exact shape, with `<kind>`
equal to `module` then `enclose`, and `<N>` equal to 0, 1000, 10000 in that
order. Every run used a fresh process and exited 0; the helper asserts
the final integer, no messages and no termination
(`target/brief-506/probes/consumer.rs:1`).

```sh
env RUM_INTERNER_CENSUS=1 RUM_INTERNER_CENSUS_DIR=/home/remi/dev/rum/target/brief-507/census/<kind>-<N> TMPDIR=/home/remi/dev/rum/target/brief-507/tmp scripts/run-bounded.py --timeout 60 -- target/brief-506/probes/consumer target/brief-507/probes/<kind>-<N>.rum <N> > target/brief-507/logs/<kind>-<N>.log 2>&1
```

Exact source templates, with N replaced by its integer:

```text
Do[Module[{censusLocal}, censusLocal], {censusI, 1, N}]; N
Do[Enclose[1], {censusI, 1, N}]; N
```

Verbatim Module completion and last census rows:

```text
CONSUMER source=target/brief-507/probes/module-0.rum calls=0 complete=true messages=0 termination=None
# symbol_size=24 symbol_text_size=40 fixed_state_size=112 canonical_owner_size=56 reserve=4096
665	665	6166	26600	896	896	112	14336	17439	21504	25631	4480	5279	4096	77182	85211	87299	""	""
CONSUMER source=target/brief-507/probes/module-1000.rum calls=1000 complete=true messages=0 termination=None
# symbol_size=24 symbol_text_size=40 fixed_state_size=112 canonical_owner_size=56 reserve=4096
1665	1665	21059	66600	1792	1792	112	28672	34847	43008	51231	4480	5279	4096	167915	183112	183112	""	""
CONSUMER source=target/brief-507/probes/module-10000.rum calls=10000 complete=true messages=0 termination=None
# symbol_size=24 symbol_text_size=40 fixed_state_size=112 canonical_owner_size=56 reserve=4096
10665	10665	165060	426600	14336	14336	112	229376	278559	344064	409631	4480	5279	4096	1173676	1289225	1298279	""	""
```

Verbatim Enclose completion and last census rows:

```text
CONSUMER source=target/brief-507/probes/enclose-0.rum calls=0 complete=true messages=0 termination=None
664	663	6155	26520	896	896	112	14336	17439	21504	25631	4480	5279	4096	77091	85120	87299	""	""
CONSUMER source=target/brief-507/probes/enclose-1000.rum calls=1000 complete=true messages=0 termination=None
1665	1664	28066	66560	1792	1792	112	28672	34847	43008	51231	4480	5279	4096	174882	190079	190079	""	""
CONSUMER source=target/brief-507/probes/enclose-10000.rum calls=10000 complete=true messages=0 termination=None
10665	10664	235067	426560	14336	14336	112	229376	278559	344064	409631	4480	5279	4096	1243643	1359192	1359192	""	""
```

These use brief-506's existing proxy: text bytes + 40 bytes/pair + the
lower/upper table-storage terms + a 4096-byte reserve. The upper table term
is implementation-conditioned; neither column bounds RSS. TSV field order
is strings, pairs, text bytes, pair bytes, three capacities, six table
bounds, reserve, total lower/upper, peak upper, context, name
(`docs/dev/codex/brief-506-interner-census.md:194`, `:205`, `:296`).
For Enclose the additional one-time seed is `InternalEncloseTag`, created
at `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:119`;
each following generated spelling uses the same allocator as Module.

Verbatim arithmetic/search-count output:

```text
dollar_number_matching_lines=36
dollar_number_distinct_files=10
module_10000_incremental_proxy_lower=1096494
module_10000_incremental_proxy_upper=1204014
module_10000_average_proxy_lower=109.6494
module_10000_average_proxy_upper=120.4014
enclose_10000_incremental_proxy_lower=1166552
enclose_10000_incremental_proxy_upper=1274072
enclose_10000_average_proxy_lower=116.6552
enclose_10000_average_proxy_upper=127.4072
```

No workspace test suite, perf gate or candidate census was run; there is
no candidate source change to validate. The current semantic build and
probes are evidence, not a claim that the proposed implementation passes
its future gates. All authored inputs, logs, hashes and build artifacts
are under `target/brief-507/`; no Rust source, repository script or committed
fixture was edited. The report's verification appendix at the end records
the final append-only and tracked-file checks.

### 3. Verdict on the crux and what would falsify it

**Yes to representation feasibility; no to a transparent display-only
patch.** The frozen Symbol has exactly the necessary independent identity
field. Reusing one pair while changing a checked identity preserves the
ruled equality and layout and avoids retained suffix text. The repeated
Module census confirms the cost that this would remove; it does not
measure an implementation of the replacement.
`crates/rum-core/src/expr.rs:36`, `:80`, `:118`, `:250`;
`target/brief-507/census-summary.txt:1`.

The exact obstruction to the naive patch is **text-to-value identity**.
``("Global`", "x", Module-tag|3)`` and ``("Global`", "x$3", None)`` have
different interned ids and identities. Printing the former as `x$3`
does not instruct today's resolver to create it. This is observed behavior,
not a hypothetical use: source, Symbol and ToExpression probes all reach
the current local, internal Names targeting performs this conversion, and
host snapshots actually use source text. Symbol serde is not evidence to
the contrary. `crates/rum-core/src/expr.rs:250`;
`crates/rum-runtime/src/runtime/resolution.rs:129`;
`crates/rum-runtime/src/runtime/namespace_tools/names.rs:152`;
`crates/rum-host/src/session.rs:185`;
`target/brief-507/logs/direct-spelling.log:1`;
`target/brief-507/logs/roundtrips-v2.log:1`.

The recommendation is to have the director consider explicitly separating
Module's display label from a source-resolvable name, preserve actual
symbol values and lexical closures, and provide real identity-preserving
persistence. It is **not** permission to break name APIs or the host
contract during implementation. If that semantic direction is unacceptable,
this design must return to an alias/reserved-encoding design or proceed
with the quota while retaining Module's measured demand. A lossless alias
map retained forever would trade an interner leak for another
activation-count-dependent store; collecting aliases changes what old
strings can recover. A reserved textual encoding would change the accepted
symbol-name/Unique namespace and must not decode sealed ids. These are
tradeoffs requiring a new ruling, not free implementations of Display.
Evidence for the affected existing surfaces:
`docs/reference/builtins.md:5306`, `:5628`, `:5861`;
`crates/rum-host/src/session.rs:344`;
`docs/dev/sealed-scoping-design.md:63`, `:164`.

The following would falsify the proposed branch or stop its implementation:

1. A normative requirement that every Module display label remain an
   identity-preserving source/typed-symbol round-trip, with no authorized
   change. The Module paragraph itself does not promise that, but the
   director must judge the surrounding public surfaces rather than infer
   permission from silence (`docs/guide/scoping-and-contexts.md:44`,
   `crates/rum-host/src/session.rs:339`).
2. A persisted closure, shared/held/head reference, definition cycle or
   sealed relink that the new textual image fails to preserve. The current
   host snapshot contract and closure contract must remain intact
   (`docs/host-embedding-and-hardening.md:95`,
   `docs/guide/scoping-and-contexts.md:48`). The small wrapper probe is not
   sufficient evidence to waive that gate.
3. Any stale Symbol becoming equal to a newly allocated local after sweep,
   rollback, import or within the lifetime domain selected by M2, or a
   Module/Remove/alpha identity reaching the sealed domain. Allocation must
   fail before overlap or wrap, not merely rely on the large size of u64
   (`crates/rum-runtime/src/runtime/session/session_helpers.rs:150`,
   `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:225`,
   `crates/rum-runtime/src/runtime/scoping/packages.rs:8`).
4. A positive post-warmup string/pair slope in the fixed-source Module
   workload, or an equivalent unbounded auxiliary name registry. Rendering,
   collision checking and snapshot code must not secretly call the old
   fresh-text constructor (`crates/rum-runtime/src/runtime/scoping/unique.rs:42`,
   `crates/rum-runtime/src/runtime/session/session_api.rs:380`, under `crates/rum-runtime/src/`).
5. A regression on ordinary evaluation or any relevant neighbor in the
   required instruction A/B. The representation ruling cannot be evaded to
   fix it; report the failure (`TODO.md:1188`, `:1261`).

The claim that a **Module-only** change removes all automatic ordinary
interner growth is already falsified by the Enclose census. That does not
falsify identity-based Module locals; it changes the sequence and the
honesty required of the subsequent cap-sizing report
(`target/brief-507/enclose-census-summary.txt:1`).

### 4. Risks and open questions — decisions for the director

| Decision | Proposed disposition and alternatives; no implicit ruling |
|---|---|
| **M1 — Are Module labels still resolvable identities?** | Decide separately for direct source, Symbol/ToExpression, exact string inspection/message/attribute targets, NameQ/Names, typed host arguments and InputForm transport. Recommended branch: labels are presentation, full values carry identity, internal name selection preserves actual Symbols, and persistence explicitly reconstructs them. This requires documenting the affected limits and a coherent exact-name/ambiguity policy. If existing text round-trips are mandatory, commission a separate resolver/encoding design; do not dispatch the allocation switch. Evidence: the round-trip table and `crates/rum-host/src/model/outcome.rs:509`. |
| **M2 — What is the freshness lifetime and display-number contract?** | Never reuse after GC. Prefer process-wide Module identity allocation and checked exhaustion to protect lifetime-free stale Symbols, including across reset/rollback/import. This changes reset-visible suffixes unless display serials are separately encoded. If session-relative identity is chosen to preserve the current numbering contract, explicitly accept that stale raw Symbols can alias across session rebuilds; do not call that globally safe. Do not silently adopt a guessed bit split for epoch/serial. Evidence: `crates/rum-host/src/lifecycle_contract_tests.rs:129`, `crates/rum-runtime/src/runtime/scoping/packages.rs:420`, `crates/rum-runtime/src/runtime/session/session_helpers.rs:150`. |
| **M3 — Approve the identity partition and exhaustion outcome** | Proposed Module region: bit 61 set, bits 62/63 clear, payload nonzero and below bit 61. Require checked bounds on every allocator and on Remove generation advancement, and exclude Module from `symbol_generations`. Decide the recoverable identity-exhaustion error and import validation/reservation surface; do not panic or conflate it with the later interner quota. Existing unrestricted `with_identity` is a trusted host construction surface, so its accepted domain and serde import behavior need an explicit policy. Evidence: `crates/rum-core/src/expr.rs:118`, `:312`; `docs/dev/sealed-scoping-design.md:63`, `:72`; `TODO.md:1238`. |
| **M4 — Collision, ordering, name accessor and host value behavior** | Prefer preserving visible name comparisons and Unique's fresh-name semantics. Decide whether the current occupied-name skip and same-text display ambiguity must be prevented, including later creation of an ordinary literal with the same display. A lookup-only interner operation can avoid inserting a candidate on a miss, but still costs formatting/lookup; it must be measured. Decide raw Rust `name()` versus owned logical short name, nested generated prefixes, and whether the string-shaped HostTyped surface gains an explicit identity-carrying variant or documents a limit. Evidence: `crates/rum-runtime/src/runtime/tests/thread_and_dispatch.rs:61`, `crates/rum-core/src/expr.rs:144`, `:182`, `crates/rum-runtime/src/runtime/scoping/unique.rs:90`, `crates/rum-host/src/model/outcome.rs:509`. |
| **M5 — Persistence image and backward behavior** | Approve or reject the lexical wrapper proposal only after the comprehensive proof gates. Specify old text-image loading, repeated loads, selected Save dependency behavior, byte stability for unaffected images, placeholder vocabulary, and composition with sealed-unit registration. Prefer fresh identities on restore with graph-wide relinking, consistent with the fact that the public snapshot does not capture the generator counter. A stable opaque numeric import format is another design, not a hidden extension to this proposal. Evidence: `crates/rum-host/src/session.rs:177`, `crates/rum-runtime/src/runtime/session/session_api.rs:396`, `crates/rum-runtime/src/runtime/tests/sealed_snapshot.rs:210`. |
| **M6 — Classification and diagnostic policy** | Decide whether Save's current textual gensym heuristic is preserved for ordinary dollar names while adding tagged locals, and whether profiling continues grouping by old displayed spellings or explicitly groups by base. Do not recreate a process-lifetime text cache just to preserve diagnostic grouping. All disabled instrumentation must remain cold; default-path changes must pass the instruction gate. Evidence: `crates/rum-runtime/src/runtime/package_tools.rs:393`, `:803`, `crates/rum-runtime/src/runtime/profiling/records.rs:282`, `AGENTS.md:112`. |
| **M7 — Enclose sequencing and the meaning of “dominant remaining”** | Schedule a separate internal-tag repair/design before claiming ordinary execution is flat, or include Enclose's positive slope in cap sizing. The same-text mechanism may help this internal tag, but a blind shared allocator change would also change Unique and GC provenance. No Enclose repair is performed here. Frequency-based dominance remains unmeasured. Evidence: `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:118`, `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:14`, `target/brief-507/enclose-census-summary.txt:1`. |

The general restore-termination defect remains outside this work, as already
ruled. Nothing here makes restore transactional or repairs unrelated
termination kinds (`TODO.md:1253`, `crates/rum-host/src/session.rs:209`).
Quota storage/admission work resumes under D1–D7; its default must use new
measurements rather than the historical suite maximum or an assumed
elimination of Enclose (`TODO.md:1213`, `:1293`).

### 5. Drafted follow-up briefs

These are **raw material for the director to rewrite**, not authorized
implementation work. In particular A–C cannot be dispatched with M1–M6
unresolved. The template requires this review step
(`docs/dev/codex/TEMPLATE.md:32`). Letter labels deliberately do not reserve
brief numbers. Each implementation brief below inherits the instruction
recipe in section 1, replacing the template's timing-panel example under
the specific D6 ruling (`TODO.md:1261`).

#### Draft A — checked identity and virtual-name substrate

##### Goal

Implement the director-approved Module identity encoding and raw-versus-
rendered name operations while leaving Module on its current allocator.
The existing Symbol representation, Copy behavior, lifetime-free public
type, id-plus-identity equality and serde fields remain unchanged.
Prepare the substrate; do not claim that Module's leak is fixed yet.

##### Context pointers

- `AGENTS.md`, `TODO.md:1188`, `TODO.md:1213`, and brief-507 sections 1–4.
- `crates/rum-core/src/expr.rs:36`: frozen representation;
  `:80`: interner; `:118`: identity constructor; `:140`: names;
  `:250`: equality/hash/order; `:288`: serde.
- `docs/dev/sealed-scoping-design.md:63`: existing identity partitions.
- The director must insert the actual M1–M4 rulings before dispatch,
  including exhaustion/import behavior and whether allocation is
  process-wide. Stop if they are absent or contradictory.

##### Scope

- MAY modify `crates/rum-core/src/expr.rs`, `crates/rum-core/src/lib.rs`,
  and a new `crates/rum-core/src/symbol_identity.rs` helper/test module.
- MAY update `crates/rum-stdlib/src/tests/structure_ops/ordering_sets.rs`
  only for approved virtual-name comparator coverage; ordinary identity
  cases must retain their existing contract.
- Scratch only under the new brief's `target/brief-NNN/`; append its report.
- MUST NOT switch Module, change any field in Symbol/Expr/Atom, change
  equality/hash, change serde field shape, add an owning string or lifetime
  to Symbol, modify the interner's retention policy, or implement the quota.
- MUST NOT modify scripts, Cargo manifests/lock, AGENTS/TODO/PROGRESS,
  unrelated docs, ownership inventory fixtures or `.git`.

##### Deliverable

- Checked kind/payload helpers with tests at zero, partition boundaries and
  exhaustion; a safe constructor that reuses an existing text pair.
- A nonleaking rendered-name writer, rendered length, name comparison and
  Module-aware system-name predicate, under the approved API contract.
- Tests that ordinary, Remove, sealed and alpha symbols retain their
  specified rendering, and that a Module-local system base cannot become
  the underlying system constant through name comparison.
- Layout/Copy/needs-drop/type-bound results, pair/string diagnostic
  evidence, and the instruction A/B rows for shared-path changes.

##### rum / Rust pitfalls (read twice)

1. `Some(identity)` is not synonymous with Module; classification must not
   treat sealed or alpha payload bits as a Module marker.
2. `with_identity` still calls the interner. Demonstrate retained-text
   behavior separately from any lock/allocation-cost claim.
3. `name() -> &str` cannot synthesize a borrowed suffix without storage.
   No leaked display cache; no unsafe.
4. `cmp_qualified_name` must agree with actual rendered bytes, and Eq/Ord
   must remain consistent even when two distinct symbols display alike.
5. Raw serde import must follow the ruled freshness policy. Copying an id
   into a value does not advance a separate allocator automatically.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run bounded `cargo fmt --check`, `cargo test -p rum-core` and
`cargo test -p rum-stdlib structure_ops::ordering_sets`, then the shared-path
instruction A/B defined in brief-507 section 1. Save a base binary built
at the same build path before editing. Use diagnostic instrumentation only
in separately authorized scratch copies; do not measure it for speed.
Paste layout/type checks, core/comparator results and raw instruction rows.
If the helper cannot be implemented inside this scope without widening a
public shape or error-propagation graph, stop with the concrete ripple.

##### Hard rules

- No commits, branches, switches, merges, pushes, stashes, checkout/restore,
  index or git-config writes. Leave work uncommitted.
- No edits outside Scope, no network, no files outside the repo; bound
  builds/probes with `scripts/run-bounded.py` using recorded same-shape
  runs to choose bounds. Rust 1.85.0, offline builds.
- Rum's documented contracts only; no vendor oracle or external outputs.
- No new unsafe, no change to the frozen Symbol representation/equality,
  no interner reclamation or tenant attribution.
- No unconditional diagnostic work; preserve the env-gated cold pattern.
- Instruction regression on a target or neighbor is a defect to fix or
  report. Do not substitute timing or relax liveness checks. Maximum 408
  instruction process runs for the complete section-1 matrix; stop after
  two failures for the same reason and report instead of tuning blindly.

##### Report format

1. Files/functions changed and the approved decisions implemented.
2. Verbatim type/layout, unit-test, census and instruction evidence.
3. What remains unwired; invariant checks and any measured regressions.
4. Stop-loss, missing gates and concrete scope/error-propagation issues.

#### Draft B — name consumers and identity-preserving text persistence

##### Goal

Make the identified consumers correct for Module-tagged Symbols before the
runtime starts producing them. Preserve logical name comparisons without
re-interning suffixes, keep full Symbols through internal name selection,
and implement the director-approved textual persistence plan. Do not change
the Module allocator in this brief.

##### Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, and brief-507's complete inventory.
- `crates/rum-runtime/src/runtime/namespace_tools/names.rs:152`: lossy
  render/resolve selection; `crates/rum-runtime/src/runtime/messages/message_format.rs:153`: raw names.
- `crates/rum-runtime/src/runtime/matcher_and_symbols.rs:598` and
  `crates/rum-runtime/src/runtime/model/definitions.rs:264`: duplicate atomic-head name checks.
- `crates/rum-runtime/src/runtime/session/session_api.rs:352` and
  `crates/rum-runtime/src/runtime/package_tools.rs:410`: real source persistence.
- `crates/rum-host/src/session.rs:177`, `:339` and
  `crates/rum-host/src/model/outcome.rs:509`: public host contracts.
- Insert M1, M4, M5 and M6 rulings verbatim. If the director chooses an
  alias/reserved-source encoding instead, replace this draft; do not add
  that substantially different design during execution.

##### Scope

MAY modify the following existing production files for the precise
Symbol-name uses inventoried in brief-507:

- Under `crates/rum-runtime/src/runtime/`: `attributes.rs`,
  `expr_runtime_helpers.rs`, `pattern_syntax.rs`, `matcher_and_symbols.rs`,
  `model/definitions.rs`, `builtin_catalog.rs`,
  `builtin_catalog/stdlib_attributes.rs`, `metadata.rs`, `builtins_input.rs`,
  `namespace_tools/names.rs`, `namespace_tools/syntax.rs`, `options.rs`,
  `options/core.rs`, `options/arguments.rs`, `options/defaults.rs`,
  `options/builtins.rs`, `messages/message_selector_parsing.rs`,
  `messages/message_catalog.rs`, `messages/message_format.rs`,
  `messages/builtins_messages_inspect.rs`,
  `messages/parse_and_message_helpers.rs`, `session/session_api.rs`,
  `session/session_eval.rs`, `package_tools.rs`, `eval_core/symbol.rs`,
  `eval_core/stability.rs`, `eval_dispatch/mod.rs`,
  `substitution/slot_substitution.rs`,
  `substitution/function_and_subst_helpers.rs`,
  `definitions/definitions.rs`, `definitions/value_table_assignment.rs`,
  `definitions/definition_lhs_preparation/syntax_policy.rs`,
  `definitions/definition_lhs_preparation/prepared_store.rs`,
  `scoping/packages.rs`, `scoping/unique.rs`,
  `kernel_data/control_data/status.rs`,
  `kernel_data/control_data/confirmation.rs`,
  `kernel_data/control_data/abort_cleanup.rs`,
  `kernel_data/control_data/branching_loops/iteration/table_fast_path.rs`,
  `kernel_data/rewrite_data/replace/once.rs`,
  `kernel_data/rewrite_data/replace/repeated.rs`,
  `kernel_data/text_rendering.rs`, `debug_tools/resources/memory.rs`,
  `debug_tools/properties.rs`, `debug_tools/monitor.rs`,
  `debug_tools/test_report/evaluation.rs`,
  `debug_tools/test_report/parsing.rs`, `arithmetic/canonical.rs`,
  `evaluation_cache/mod.rs`, `evaluation_cache/profile/mod.rs`,
  `profiling/fingerprints.rs`, `profiling/timing.rs`,
  `profiling/presence.rs`, `profiling/records.rs`.
- A new focused `crates/rum-runtime/src/runtime/package_tools/module_image.rs`
  for snapshot planning/rendering; declare it from `package_tools.rs`.
- Under `crates/rum-stdlib/src/`: `strings.rs`, `predicates.rs`,
  `predicates/collections.rs`, `structure_ops/ordering_sets.rs`,
  `structure_ops/surface.rs`, `arithmetic/stats.rs`, `xml.rs`, `csv.rs`,
  `json.rs`, `schema.rs`, `polynomials/mod.rs`,
  `associations/missing/traversal/projection.rs`, `strings/template.rs`,
  `strings/similarity/mod.rs`, `strings/scripting/args.rs`,
  `strings/scripting/url.rs`, `strings/scripting/text_shape.rs`,
  `functional_ops/arrays/spec.rs`.
- `crates/rum-host/src/model/outcome.rs` and `crates/rum-host/src/session.rs`
  for the approved existing-interface behavior/rustdoc, not an unapproved
  new host symbol-handle type.
- New focused tests:
  `crates/rum-runtime/src/runtime/tests/module_identity_consumers.rs`,
  `crates/rum-runtime/src/runtime/tests/module_identity_snapshots.rs`,
  `crates/rum-stdlib/src/tests/module_identity_consumers.rs`,
  `crates/rum-host/src/module_identity_contract_tests.rs`, and their existing
  parent test-module declarations.
- Approved contract docs: `docs/guide/scoping-and-contexts.md`,
  `docs/host-embedding-and-hardening.md`; changes to generated
  `docs/reference/builtins.md` must come with the precise catalog entries
  named by the director before dispatch. No blanket catalog edit permission.
- Scratch under the new brief's target directory and its appended report.

This is a ripple allowlist, not a request to edit every listed file. Do not
rewrite non-Symbol `.name()` calls or unrelated code. If the director wants
smaller implementation units, split runtime consumers, stdlib consumers
and persistence into sequential briefs before dispatch; each must remain
green with Module still unwired. Do not change Symbol layout/equality,
parser syntax, Unique's fresh-name allocator, Enclose's allocator, quota
logic, scripts, Cargo files, AGENTS/TODO/PROGRESS, `.git`, or unrelated docs.

##### Deliverable

- An annotated disposition for every real Symbol-name consumer in the
  saved inventory: preserved raw identity text, logical comparison,
  presentation, size estimate or deliberate diagnostic grouping.
- Internal name matching returns actual Symbols and retains context/sealed
  filtering; it never recovers those values by reparsing printed labels.
- An explicit persistence plan with identity-keyed slots, deterministic
  collision-free placeholders and shared/cyclic reference preservation.
- Tests using constructed Module-tagged symbols, plus public text snapshot
  and host-value coverage. The existing plain/Unique/Remove/sealed/alpha
  tests remain meaningful, not mechanically weakened.

##### rum / Rust pitfalls (read twice)

1. Fixing `Display` does not fix `SymbolName`, `:name`, options, atomic head
   constraints, or system-name fast paths. Fixing `is_system_name` alone
   does not fix callers comparing `.name()` directly.
2. Name-pattern operations must keep the matched Symbol; a string
   deduplication/reparse step loses identity and may target a different value.
3. Measure any branch added to common name/matcher/order helpers. Do not
   allocate rendered names on every ordinary predicate call.
4. Prove snapshot closure/sharing/held/head/binder behavior at the actual
   host source boundary. A serde round-trip is a different test.
5. Avoid capture by placeholders anywhere in the saved graph. Keep
   definition planning separate from rendering and preserve sealed relinking.
6. Raw name byte accounting and rendered text preflight are different
   quantities. Do not undercharge new output or claim suffix bytes are
   retained in the interner when they are not.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run bounded `cargo fmt --check` and `cargo test --workspace`. Include
reference drift and host lifecycle tests. The targeted matrix includes
Module-tagged system constants, both atomic-head matcher paths, option
fallbacks, wildcard and exact name queries, messages, trace, completion,
Sort/canonical order, Unique with a tagged prefix, GC-visible persisted
closures, repeated and old-image restore, and sealed snapshot composition.
Execute the instruction A/B from brief-507 section 1 with plain release
binaries. No unsupported native transport scenario is reported as executed.
If the lexical image form fails twice on the same contract, stop with the
smallest counterexample; do not add an identity-forging parser shortcut.

##### Hard rules

- No git mutation operations; leave the tree uncommitted. Preserve unrelated
  user edits. No scope expansion without a revised brief.
- No network or writes outside the repo; all long/child-spawning commands
  use `scripts/run-bounded.py` with measured same-shape bounds.
- No vendor oracle, unsafe, Symbol representation/equality change,
  interner reclamation, quota/default choice or permanent display-name cache.
- No hidden default-path diagnostics. Preserve suspension frames/root
  tracing and borrowed/shared expression traversal.
- Use the instruction gate, not the timing panel; maximum 408 matrix runs,
  raw captures before reduction, profiling off, liveness and scaling checked.
  A target/neighbor regression is a defect to fix or report.
- Documentation changes require the director's concrete M1/M4/M5/M6 ruling;
  never weaken an existing contract just because a test fails.

##### Report format

1. Per-file changes and the completed consumer disposition inventory.
2. Verbatim tests, persistence source/output, and per-workload A/B rows.
3. Remaining compatibility differences against the approved rulings.
4. Stop-loss, residual scope needs and unexecuted/missing gates.

#### Draft C — switch Module to checked identities

##### Goal

After A and B pass, make every new Module local reuse its already interned
base pair and allocate only its approved numeric identity. Preserve lexical
capture, all suspension/error/GC paths, and the selected name, persistence
and lifetime contracts. No new interned suffix or per-activation permanent
alias entry is permitted. Unique and Enclose keep their separate allocators.

##### Context pointers

- `AGENTS.md`, brief-507 sections 1–4, and A/B's passing reports.
- `crates/rum-runtime/src/runtime/scoping/lexical/module.rs:46`, `:306`:
  allocation, scope finish and candidate registration.
- `crates/rum-runtime/src/runtime/scoping/unique.rs:81`, `:149`: shared
  allocator that must not be replaced globally.
- `crates/rum-runtime/src/runtime/definitions/definition_clear.rs:212`:
  generation pollution hazard.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:95`, `:128`:
  rollback and counter lifetime; `crates/rum-runtime/src/runtime/scoping/packages.rs:420`: sealed counter.
- Insert the exact M2/M3/M4 decisions and recoverable exhaustion behavior
  before dispatch; do not silently choose a process/session lifetime.

##### Scope

- MAY modify `crates/rum-runtime/src/runtime/scoping/lexical/module.rs`,
  a new sibling `module_identity.rs` allocator, and its parent declaration;
  `scoping/unique.rs` only for the approved shared display-collision checks;
  `definitions/definition_clear.rs`; `scoping/gensym_gc.rs` only for
  required identity-lifetime integration, preserving its root contract;
  `session/session_helpers.rs`, `session/session_state.rs`,
  `runtime/model/session.rs` (relative to `crates/rum-runtime/src/` for that
  last path) for the approved counter/snapshot model.
- Partition/exhaustion enforcement may touch
  `crates/rum-runtime/src/runtime/scoping/packages.rs` and
  `crates/rum-runtime/src/runtime/substitution/binding_substitution.rs`
  only as explicitly ruled in M3. If error propagation forces a wider
  graph, stop and enumerate it; this is not a compiler-forced wildcard.
- MAY update the exact affected tests from brief-507's inventory:
  `crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs`,
  `tests/scoping_gensym_gc/reachability.rs`,
  `tests/scoping_gensym_gc/suspension_frames.rs`,
  `tests/thread_and_dispatch.rs`, `tests/definitions_and_rules_part3.rs`,
  `tests/session_requests/input_resume.rs`, `tests/scoping_unique.rs`,
  `tests/sealed_packages.rs`, `tests/session_memory.rs` (all `tests/` paths
  relative to `crates/rum-runtime/src/runtime/`);
  `crates/rum-host/src/lifecycle_contract_tests.rs`, and the new focused
  A/B test modules. Add tests for freshness, imports and exhaustion there.
- Update only the specific guide/rustdoc contract paragraphs named in the
  director's rulings, with regenerated reference entries if applicable.
- Scratch under this brief's target directory; append its own report.
- MUST NOT change Symbol fields/Copy/equality, Unique's documented naming,
  Enclose's allocation, interner retention/quota, scripts, Cargo files,
  AGENTS/TODO/PROGRESS, `.git`, or unrelated docs/fixtures.

##### Deliverable

- Module-only allocation switch and checked identity lifetime integration.
- Collision checks that do not intern candidate text on a miss; no
  base-name generation-map pollution on Remove.
- Tests retaining a stale Symbol across sweep and every lifetime boundary
  in the approved policy, including imported identities and exhaustion.
- Existing capture/root/suspension assertions preserved with actual
  generated symbols where old helpers fabricated them from text.
- Flat fixed-source Module census and the instruction A/B, both with raw
  evidence; no claim about Enclose or cap sizing yet.

##### rum / Rust pitfalls (read twice)

1. Allocate per binding, not per activation. Do not reuse an identity on
   success, error, abandon, sweep or a rolled-back allocation.
2. `symbol_generations` is keyed by base text. A Module id must not be
   inherited into it by Remove.
3. A display collision check that calls `Symbol::new` on `x$N` reinstates
   the entire interner leak before the successful allocation.
4. Preserve pending initializer/body continuation state and root visits;
   do not re-evaluate initializers or allocate a second set of locals on resume.
5. Source images and raw serde imports are different freshness paths. Test
   a new allocation after both, not only the imported value's equality.
6. Do not “repair” the raw-symbol lifetime problem by changing Symbol's
   representation, branding, equality or reclamation policy.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run bounded `cargo fmt --check`, focused scoping/GC/Remove/sealed/suspension
tests, then `cargo test --workspace`. Use separately instrumented fresh
processes at Module N=0, 1000, 10000; compare counts and retained proxy after
setup. Add same-base multiple locals/closures, immediate reclaim, root
escape, cancellation/error, repeated source-image restore, and collision
trials. A whole-session memory check must rule out an accumulating alias
table in addition to the interner counter check. Run the complete section-1
instruction A/B and paste target and neighbor rows.

##### Hard rules

- No commits/branches/switches/merges/pushes/stashes/checkout/restore,
  git-config or index writes; leave the patch reviewable and uncommitted.
- Scope is closed. Stop if new error propagation requires an unlisted file.
- No unsafe, external oracle, network, outside-repo writes, representation
  change, interner reclamation or tenant attribution.
- Bound every long/child-spawning command from recorded same-shape data;
  offline toolchain/build use; no timing-based acceptance.
- Profiling stays cold; timing/instruction builds contain no census or
  allocation instrumentation. At most 408 instruction matrix runs; stop on
  repeat failure and report a regression rather than hiding it in an average.
- No quota/default selection and no opportunistic Enclose repair.

##### Report format

1. Per-file changes and implementation of M2–M4.
2. Verbatim focused/workspace tests, stale/import/exhaustion cases, census
   rows, memory evidence and instruction A/B.
3. Whether Module's per-local retained text slope is actually zero.
4. Remaining differences, regressions, environmental omissions and stops.

#### Draft D — independent Module falsification and post-change census

##### Goal

Independently challenge the approved Module implementation's identity,
name/persistence, lifetime and performance claims. Recount the process-wide
interner on ordinary and intentional-growth workloads, including Enclose.
Supply cap-sizing inputs; choose no quota/default and land no diagnostics.

##### Context pointers

- `AGENTS.md`, `TODO.md:1188`, `TODO.md:1213`, A–C reports and this report.
- `docs/dev/codex/brief-506-interner-census.md:185`: exact proxy formula,
  table uncertainty, retained versus growth-overlap columns.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:118`:
  measured remaining automatic consumer unless separately repaired.
- `docs/dev/codex/brief-503-split-the-fourteen-percent.md:102`: instruction
  instrument and acceptance discipline.

##### Scope

- NONE — no production source, script, fixture or other documentation
  change. Append only this new brief's report.
- MAY create diagnostic source copies, instrumentation, `.rum` workloads,
  logs and builds under its own `target/brief-NNN/`. Instrumentation is
  explicitly throwaway and must not be installed into the production tree.
- Read previous artifacts; never overwrite them. No git mutation or quota work.

##### Deliverable

- Attempts to falsify the exact semantics/lifetime decisions: ordinary
  lookalikes, localized system heads, Remove/alpha/sealed interactions,
  stale values, wrap boundaries, imports, text snapshots and typed host values.
- Fresh-process bootstrap census; maximum over individual workspace test
  binaries; unchanged default microbench-source census; Module, Enclose,
  Unique and dynamic-Symbol loops at calibrated sizes.
- Separate intentional name growth, automatic tag growth and retained
  runtime state. If Module is flat but an alias table grows, say the goal
  failed. If Enclose remains positive, do not call ordinary code flat.
- Independent two-arm instruction A/B and post-change cap-budget arithmetic
  under the actual storage model, with its uncertainty. No selected default.

##### rum / Rust pitfalls (read twice)

1. Maximum over test processes is not their sum; each has its own interner.
2. Count strings, pairs and bytes. Context probing can create more than one
   pair for one input spelling, and table capacity changes are discontinuous.
3. Reused sessions, fresh processes and repeat restores test different
   lifetime boundaries. Name each one explicitly.
4. Interner proxy bytes are not RSS. Do not quote a portable physical bound
   from opaque std HashMap capacity.
5. Census and instruction builds are separate. A timed-out or terminated
   workload is incomplete even if its CLI output contains a final row.

##### Verification (MANDATORY — run it; iterate to green / evidence)

Run exactly the named falsification probes, bounded
`cargo test --workspace --no-fail-fast` with census enabled, and the
unchanged microbench source with census enabled **for storage only**.
Record unavailable native transport cases honestly. Perform the instruction
matrix with the plain base and candidate binaries built at the same path;
three repeats/full/half/control, profiling off, exit/liveness/coverage
checks. Paste every census endpoint, largest-process identity, raw perf
rows and arithmetic used. Revalidate bounds before increasing trial sizes.

##### Hard rules

- No production changes or git mutation operations; no network or
  outside-repo writes; no unsafe or vendor oracle.
- All scratch under the assigned target directory; bound all long commands
  with `scripts/run-bounded.py` from measured same-shape data.
- No Symbol representation/equality changes, reclamation, attribution or
  default selection. The census cannot turn a proxy into an RSS guarantee.
- Do not weaken failed contracts, excuse an unexecuted test as green, or
  replace missing perf access with timing-panel acceptance.
- Maximum 408 instruction matrix processes; suite once unless a concrete
  failure correction justifies a rerun. Preserve raw evidence for failures
  as well as successes; stop after two identical failures and report.

##### Report format

1. Falsification attempts and observed outcomes, with path:line evidence.
2. Verbatim census/test/perf output, byte model, uncertainty and process scope.
3. Revised legitimate-growth inputs and remaining automatic consumers.
4. Pass/fail/missing gates and the remaining choices for the director.

#### Draft E — separate Enclose tag design before the quota resumes

##### Goal

Decide whether Enclose's internal lexical confirmation tag can use the
approved same-text identity mechanism without changing confirmation
selection, nested boundaries, suspension or exposed failure payloads.
Return its bounded implementation brief and gate plan. This is a distinct
repair line; do not change Unique or broaden the Module patch.

##### Context pointers

- `AGENTS.md`, `TODO.md:1213`, brief-507's Enclose source/census evidence.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation.rs:118`:
  seed/fresh tag; `:299`: rewriting untagged confirmations.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:7`:
  allocation only when the third form argument is absent; suspension state.
- `crates/rum-runtime/src/runtime/scoping/unique.rs:81`: current shared
  fresh-text generator. `docs/reference/builtins.md:1532`: own contract.
- Director M7 disposition and the final Module identity-kind/lifetime rules.

##### Scope

- NONE — no source change. Append this new brief's report only.
- Scratch observations and `.rum` probes under its own target directory;
  read existing sources/builds and census helpers without editing them.
- No quota/default change, no automatic reuse of Module GC provenance,
  no source edit to either allocator, and no changes to other docs or `.git`.

##### Deliverable

- Enumerated tag creation, rewrite, matching, escape/failure rendering and
  resume/abandon paths, with path:line evidence and actual rum probes.
- An explicit choice whether the existing new identity kind is suitable
  or an additional kind is required. Do not treat a non-Module internal tag
  as a Module GC candidate merely because both use numeric identities.
- A drafted implementation brief naming exact files/functions, tests,
  repeated Enclose flat-census gate and instruction neighbors.
- A corrected statement of remaining legitimate interner consumers.

##### rum / Rust pitfalls (read twice)

1. The third argument suppresses lexical tag generation; test both forms.
2. Nested Enclose deliberately stops the outer confirmation rewrite.
3. A tag can survive in a suspension or expression even when ordinary
   success returns only a number. No identity reuse after lifetime end is
   assumed without proving the approved lifetime domain.
4. Replacing the shared allocator would also change Unique and Module;
   scope the eventual repair to the internal tag call site.

##### Verification (MANDATORY — run it; iterate to evidence)

Re-probe success, failure, handler, explicit third form, nested confirmation
boundaries and suspension using rum's own documented behavior. Reproduce
the zero/1,000/10,000 census with the existing method, preserving raw rows.
No source change means no implementation test gate; do not claim a fixed
candidate. Define the later Enclose-specific instruction A/B and ordinary
neighbor controls, using the section-1 method rather than the timing panel.

##### Hard rules

- No source/script/fixture edits; no git mutation, network, outside-repo
  writes, unsafe or vendor comparisons. Report-only outside scratch.
- Bound every long or child-spawning command from recorded same-shape
  measurements; stop after two identical failures and report.
- No Symbol representation/equality change, interner reclamation,
  attribution, quota/default choice or assumed hidden-name exemption.
- Paste actual evidence; label prospective savings as inferred, not measured.

##### Report format

1. Plan with exact files/functions and per-step gates.
2. Tag dependency/round-trip inventory and verbatim probe/census evidence.
3. Feasibility verdict and falsifiers.
4. Director decisions and a TEMPLATE-shaped implementation draft.

#### Handoff verification for brief-507

The original brief prefix was saved before any report write at
`target/brief-507/brief-prefix.txt:1`; tracked-file hashes were saved at
`target/brief-507/start-hashes.json:1`. A final comparison found only this
report changed, with that prefix byte-identical. The report contains every
executed probe's complete stdout/stderr verbatim, including the two invalid
initial probes. Exact invocation/result records are in
`target/brief-507/run-results.json:1`; raw logs and census files remain
under the same scratch root. Verbatim verification output:

```text
Executed probe outputs reproduced verbatim: True
Changed tracked files:
docs/dev/codex/brief-507-module-gensym-text-leak.md
Original brief prefix intact: True
Recorded probe process exits: 13 of 13 returned 0; two initial semantic probes had documented messages/failures.
```

`git diff --check` exited 0 with no output. Verbatim `git status --short`:

```text
 M docs/dev/codex/brief-507-module-gensym-text-leak.md
```

Executable hashes, recorded in `target/brief-507/binary-hashes.json:1`:

```text
target/brief-507/build/release/rum-cli
eb306640457ca20fdb6dd6fcebb2dd9ddb7abca453e85f88575a39aa29b1abb6
target/brief-506/probes/consumer
4df1b56f218e72aea94d86015812b9f0780d1d40f5eaa1292220af00cf22f0e5
```

No production source, script, fixture or other document changed. No
implementation, quota choice, network call, unsafe addition, vendor
comparison, commit, branch/switch, merge, push, stash, checkout/restore,
git-config or index-edit operation was performed. Git use was limited to
read-only status, diff, rev-parse and ls-files. No sub-agents were used.
Exploratory file reads included some nonexistent guessed paths, followed
by `rg --files`/actual-source searches; those read failures are not
verification successes. The scoped task is complete: the implementation
choices remain explicitly with the director, as the brief requested.
