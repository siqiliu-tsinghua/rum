# brief-606 — design round 5: G13's sequence as type-then-flip

`Scope: NONE — no production change.` Design-only, with one optional scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

Round 4 (brief-605, your design) is accepted except for its green sequence and
one addition. This round replaces the single atomic conversion with a sequence
in which **every step lands green and is measured on its own**, and produces
the implementation drafts for that sequence. Nothing else in round 4 is
reopened.

## The director's decisions on round 4

**Accepted as designed** (`docs/dev/codex/brief-605-g13-design-round-4.md`,
`.temp/brief-605/design.md`): the closed channel and its visibility
arrangement with the codec key; the witness, `FrameChain<I, O>`, the
fresh-state `ContinueToken`, `ResumeStop`/`finish_work` and every deferred
family in `deferred-obligations.tsv`; the default-fallback, retention and
public-file-rollback routes; the private `DriverFault`, its structural unwind
and the request latch; two-Arc accounting in the maintained and independent
models; the test-only `FaultProbe` behind the runtime feature
`test-continuation-slots`; the 26 compile-control pairs; the inventory
transitions; the E2 boundary; the full per-label A/B on every step that
changes a hot function. The 28 ordinary witnesses and all 16 raw tables were
reproduced byte-identically by the director, and 623 implementations and 600
production macro sites match an independent scan.

**D1 — the atomic 608 is replaced by type-then-flip.** Round 4 argued that a
smaller step needs bridges and an edge proof. That holds only if a step
changes the transport. Separate the two:

- **Per-type transport mode, on the sealed codec.** Every registered slot type
  has exactly one transport at any time, a compile-time property of its sealed
  implementation: `Expr` is always `Value`; a type that has an existing
  in-band codec starts as *legacy* (it travels as `Value(Expr)` through that
  existing encoder and decoder, exactly as today); a new nominal carrier is
  `State` from birth. A decoder accepts only its type's one arm. This is not
  the dual-arm codec round 4 rightly forbids, and it is deleted with the legacy
  codecs in the last step.
- **Typing steps (phase 1).** Convert frames, finalizers and macro sites to the
  typed API family by family. A typed frame whose types are all legacy or
  `Expr` exchanges only `Value` with its neighbours, so it is compatible with
  legacy frames on both sides whatever the call graph says: no edge proof is
  needed for these steps. Each step is green in both builds, runs the whole
  oracle and the full A/B, and fixes the transformations, chains and deferred
  obligations of the frames it owns. The six raw completers are corrected by
  their return binding in the step that types them.
- **New carriers.** A carrier that is `State` from birth replaces an existing
  encoded payload between specific frames. The step that introduces it owns
  every frame that produces or consumes that payload today: the frames that
  name the carrier (the compiler finds them) and every old decoder use of the
  payload it replaces (`whole-slot-consumer-sites.tsv`,
  `nested-decoder-use-sites.tsv`). A missed consumer is still a legacy frame
  receiving `State`, which the legacy entry rejects as a fault: loud, never a
  silent decode.
- **Flip steps (phase 2).** Once every frame that consumes or produces a
  legacy type is typed, switch that type's transport to `State`. Any subset of
  types may flip in any order, because each type's mode is global. Each flip
  runs that type's internal-encoding corpus (each descriptor, position and
  shape separately) and unconditional `FaultProbe` absence in both builds; a
  frameless transformation that the tables missed shows up there as a fault,
  which is exactly when the atomic plan would have found it too.
- **Removal.** Then the legacy exports, codecs, internal heads, raw insertion,
  the transport mode itself and the temporary macro name go, as in draft 609.

Why: in both plans a missed frameless edge is caught only at run time, and
loudly, so the atomic step buys no detection. What it costs is the gate per
step, a per-label A/B attributable to one family (one variable per A/B), and
a diff a reviewer can read. If you find a population for which this argument
fails, say so with source evidence; do not paper over it.

**D2 — no `TransitRoots` in G13.** A consumer's decoded input is a Rust local
while the consumer evaluates guest code, exactly as today's `Expr` input is
(`suspension/core.rs:736`); typed state was already encoded inside that
`Expr`, so G13 neither creates nor widens this gap. It is one instance of a
wider pre-existing class the director has just demonstrated on the ordinary
path, with no suspension involved: the gensym sweep's roots
(`scoping/gensym_gc.rs:9`, `:136`) are Session fields, so a value held only in
a builtin's Rust accumulator is not a root.
`Table[Module[{x}, x = i; Hold[x]], {i, 4}]` released element by element gives
`{1, 2, 3, 4}`, and `{x$1, x$2, x$3, 4}` with `RUM_GENSYM_GC_THRESHOLD=1`.
That class is recorded for its own repair. G13 states the synchronous transit
contract in one paragraph, adds no transit record, and leaves the maintained
and independent accounting of transit unchanged.

## What this round must produce

1. **The transport mode** in Rust: how the sealed codec declares it, how the
   adapter encodes and decodes per mode, why no decoder can accept both arms,
   how the compile controls cover it, and its deletion. Revise
   `api_prototype.rs` into a compiled model that includes the mode, and add
   compile-control pairs for it to `compile-controls.tsv`.
2. **The slot table by mode**: for each of the 67 whole slots, its initial
   mode (legacy because an existing codec carries it, or `State` from birth),
   and for each `State`-from-birth carrier the frames, sites and old decoder
   uses its introducing step must own.
3. **The partition.** Every row of `all-frame-migrations.tsv` (623), every
   macro site in `macro-dataflow.tsv` (600), every chained insert, deferred
   obligation, insertion route and transformation-disposition row assigned to
   exactly one typing step, with a reconciliation script that proves the
   assignment is a partition of each table. Order the steps; for each give its
   exact file scope, the rows it owns, why it is green beside legacy
   neighbours, which of the 21 recorded divergences and 28 witnesses it
   changes (so the gate knows which comparison must move from divergent to
   equal), and the microbench labels its hot functions touch. **Size every
   step so one dispatch can finish it**: say what you used as the size
   measure and why it is safe; no step may be the whole conversion again.
4. **The flip steps**: which types flip together and why (a cycle among
   legacy types that must flip at once is a reason; convenience is not), the
   precondition each checks (no legacy frame or decoder still names the type,
   by a census that runs in the gate), and the corpus positions per type.
5. **Drafts in `TEMPLATE.md` shape**: revise 606 (the dormant API, now with
   the mode) and 607 (the compatible driver, now mode-aware and without
   transit), and write a full draft for every typing step, every flip step and
   the removal step. Shared sections may be identical across drafts; each
   draft must still stand alone when dispatched. Name drafts by step label
   (`S1`, `S2`, … `F1`, … `R`), not by brief number.
6. Open questions for the director.

## Evidence rules

- Witnesses, if you add any, are rum-vs-rum equality controls with ordinary
  values only, run as the oracle runs them
  (`crates/rum-stdlib/tests/suspension_oracle/descriptors.rs:191`), with the
  pause attributed to a source line. This round runs no internal-encoding
  shapes.
- Every count reconciles against a raw source scan or against your round-4
  tables; say which.
- Reuse the round-4 tables; copy them, do not edit them.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-606/`; **one** optional scratch integration test,
  `crates/rum-stdlib/tests/zz_brief606_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-606/probe.rs`.
- Everything else is read-only, including `.temp/brief-60*/` other than your
  own directory and `.temp/director/`.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-606/`.
- Run no probe that panics or aborts.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief606_probe`
  (900 s); `python3` scripts (300 s each). File reads, `rg`/`grep`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No production code.** No new `unsafe`.
- Checkpoint to `.temp/brief-606/PROGRESS.md` after each section with the
  exact next action.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the optional test.
2. The transport mode and the slot table by mode.
3. The partition and its reconciliation; the flip steps.
4. The drafts, with the scope table.
5. Open questions for the director.

---

## Round 5 report

### 1. What changed

**Design only.** All artifacts are under `.temp/brief-606/`; production code
is unchanged. The one optional integration test compiled the revised mode/API
model, was copied to `probe.rs`, and was **deleted before this report was
appended**. The scope check confirms all 1,427 baseline crate files unchanged
and no added crate source/inventory files. No git, network, internal-shape
probe, gensym-GC reproduction, whole-workspace/oracle test or benchmark command was run.

The first model compile reported E0507 in a test guard; that move was fixed.
The final model also removes per-slot channel-method overrides. Its actual
authorized test output was:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.69s
     Running tests/zz_brief606_probe.rs (target/debug/deps/zz_brief606_probe-6ad92f58638fdaeb)

running 4 tests
test after_flip::downstream::public_surface_and_zst_witness_compile ... ok
test after_flip::kernel::sealed::mode_is_single_arm_with_ordinary_values ... ok
test before_flip::downstream::public_surface_and_zst_witness_compile ... ok
test before_flip::kernel::sealed::mode_is_single_arm_with_ordinary_values ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

This is positive model evidence in the authorized default build. It is not
production driver/accounting evidence, a negative-compiler run, a whole-oracle
run or an A/B result. The implementation drafts require those later gates.

### 2. Transport mode and slot table

The sealed keyed Codec declares only one associated Mode. Central functions
perform channel encode/decode, so an individual slot implementation cannot
override them. ValueMode accepts only Value for Expr; LegacyMode accepts only
Value and delegates to the existing wire codec; StateMode accepts only State
and then the exact variant. Legacy providers receive Expr, state providers
receive ContinuationState, and neither can inspect both channel arms. There
is no runtime switch, per-frame choice or dual-arm fallback. R deletes the
temporary mode layer with the legacy codecs.

[slot-modes.tsv](../../../.temp/brief-606/slot-modes.tsv) enumerates all 67:
**1 Value, 23 initially Legacy, 43 State from birth**. The two existing
PreparedInterpreted enums at `pattern_matching_interpreted.rs:732,850` have
existing decoders at `:741,860` and finalizer encoder branches at `:713,829`;
they are treated as Legacy, registered in S33. The other 21 legacy types
register in S1. Each new carrier registers only in its introducing family.

State-birth ownership includes 148 direct frame/finalizer links, 156 producer/
definition site links, and all 47 slot-to-decoder-use links (44 distinct use
rows) from **both** whole-slot and nested-use tables. The introducing family
also owns its adjacent/helper frames; those are a conservative closure, not
an assertion that each frame has the carrier as its actual input/output.

The [design](../../../.temp/brief-606/design.md) states D2's synchronous Rust
local transit contract. It introduces no transit record or accounting change.
The accepted two-Arc queued adapter accounting remains required. There are
35 paired compile controls: the accepted 26 plus nine mode/deletion controls,
with phase-only pairs retired explicitly when R removes their positive API.

### 3. Partition, reconciliation and flips

S1 is dormant API; S2 is the compatible driver; S3–S54 are **52 family typing
steps**. Every one of the 1,399 required table rows has exactly one typing
owner. S2 owns the umbrella insertion mechanisms; individual occurrences stay
in their family steps. M0284 at `eval_core/symbol.rs:63` migrates in S14 with
DeclaredStubLoadFrame. All 600 macro constructor expressions are source-linked,
including six local-variable expressions resolved from their bodies/signatures.

The independent checker compares complete row payloads to the immutable
round-4 copies, not just counts. Its verbatim result is:

```text
PASS immutable input copies: 40
PASS partition all-frame-migrations: 623 rows, exactly one typing owner each
PASS partition macro-dataflow: 600 rows, exactly one typing owner each
PASS partition chained-inserts: 62 rows, exactly one typing owner each
PASS partition deferred-obligations: 14 rows, exactly one typing owner each
PASS partition insertion-routes: 10 rows, exactly one typing owner each
PASS partition transformation-dispositions: 90 rows, exactly one typing owner each
PASS total partition rows: 1399
PASS frame kinds: {'frame': 564, 'finalizer': 59}
PASS macro arms: {'result': 348, 'insert': 252}
PASS macro constructors: 600 rows; unresolved/generic factories: 0
PASS modes: {'Value': 1, 'Legacy': 23, 'State': 43} ; State-birth frame links: 148 ; decoder-use links: 47 ; unique decoder-use rows: 44
PASS gates: 35 compile pairs; 21 divergence promotions; 28 ordinary controls; 23 singleton flips
PASS corpus plan: 438 descriptors; 86724 position rows; 546 shape rows; no shapes executed
PASS standalone drafts: 78 ; exact scope rows: 3359
PASS family maxima: impls 26 ; macros 24 ; impl-body chars 19822
PASS all reconciliation checks
```

The bounded family maxima are **26 implementations, 24 macros and 19,822
implementation-body characters**. Separate producer-body sizes, transformations,
deferred obligations, carrier closures and file counts are recorded in
[step-sizes.tsv](../../../.temp/brief-606/step-sizes.tsv). S2's change scope is
the actual driver/storage/error-boundary population; the much larger catchall
tables remain a read-only review population. R is dead-code deletion and
macro spelling replacement, with no postponed family typing.

All six raw completers get their fixed typed Option return binding during
typing. The 21 historical comparisons promote from divergent to equal in
S3 (six Fold/Nest), S7 (six Part/association), S10 (three Failure Message),
S13 (three Function), and S19 (three Cases). The 28 ordinary controls remain
PASS throughout, with per-step source attribution. No oracle Checks are
weakened and no expected-divergence waiver is introduced.

F1–F23 flip **one legacy type each**. There is no required transport cycle:
nested private wire codecs retain their field grammar independently of the
whole-slot mode. The gate rejects a wire provider that calls channel transport.
Before every flip, the source census must find no production legacy frame/raw
insertion and no live decoder boundary for the target type; exact pure codec
definitions/dependencies are distinguished from live calls. The
[census specification](../../../.temp/brief-606/census-spec.md) defines the
gate outputs and unresolved-use rejection.

The corpus plan enumerates 438 descriptor roles and each declared Middle
pause / first Twice pause / second Twice pause separately for every non-Expr
slot: 86,724 position rows joined with 546 per-head shape rows. These are
**planned cells**, not executed fixtures. Each actual fixture must preserve
role grammar, prove its pause attribution and pass strict queued/resumed
equality plus unconditional FaultProbe absence in both builds; exclusions
require exact source evidence. Each hot-function step also runs the complete
119-label A/B; the family label mapping does not reduce that gate.

### 4. Drafts and scope table

There are **78 full TEMPLATE-shaped drafts**: S1–S54, F1–F23 and R. Each
contains its exact file fence, owned row IDs and names, changes, applicable
controls, both-build gates, full A/B instructions, maintenance duty, progress
checkpoint and hard rules. File counts include shared verification and
maintenance files, not a claim that every scoped file must change. The
director rewrites these drafts before dispatch under the usual workflow.

[Artifact index](../../../.temp/brief-606/INDEX.md) ·
[Exact file scopes](../../../.temp/brief-606/implementation-file-scopes.tsv)

| Draft | Family/type | Impls | Macros | Exact files |
|---|---|---:|---:|---:|
| [S1](../../../.temp/brief-606/draft-S1.md) | Dormant typed API and sealed transport strategies | 0 | 0 | 59 |
| [S2](../../../.temp/brief-606/draft-S2.md) | Compatible driver, faults and accounting | 0 | 0 | 80 |
| [S3](../../../.temp/brief-606/draft-S3.md) | Evaluation, argument preparation and dispatch | 12 | 13 | 46 |
| [S4](../../../.temp/brief-606/draft-S4.md) | Argument and traversal specification resolution | 11 | 12 | 39 |
| [S5](../../../.temp/brief-606/draft-S5.md) | Definitions, defaults and fallback | 14 | 17 | 58 |
| [S6](../../../.temp/brief-606/draft-S6.md) | Guards and binding completion | 5 | 5 | 35 |
| [S7](../../../.temp/brief-606/draft-S7.md) | Part selectors and held assignment | 11 | 11 | 44 |
| [S8](../../../.temp/brief-606/draft-S8.md) | ReplaceAll and traversal collectors | 18 | 20 | 46 |
| [S9](../../../.temp/brief-606/draft-S9.md) | General control and cleanup | 25 | 24 | 43 |
| [S10](../../../.temp/brief-606/draft-S10.md) | Confirmation and status properties | 18 | 19 | 38 |
| [S11](../../../.temp/brief-606/draft-S11.md) | Loops and iterator scope | 23 | 22 | 45 |
| [S12](../../../.temp/brief-606/draft-S12.md) | Lexical block scope | 12 | 8 | 39 |
| [S13](../../../.temp/brief-606/draft-S13.md) | Other lexical and callable scope | 7 | 7 | 39 |
| [S14](../../../.temp/brief-606/draft-S14.md) | Packages, loading and structural rollback | 13 | 12 | 55 |
| [S15](../../../.temp/brief-606/draft-S15.md) | Reap and Sow deferred rendering | 9 | 7 | 35 |
| [S16](../../../.temp/brief-606/draft-S16.md) | Captured tests and reports | 7 | 7 | 41 |
| [S17](../../../.temp/brief-606/draft-S17.md) | Debugging, resource wrappers and tracing | 12 | 12 | 66 |
| [S18](../../../.temp/brief-606/draft-S18.md) | Retention, attributes, options, input and messages | 13 | 14 | 62 |
| [S19](../../../.temp/brief-606/draft-S19.md) | Cases collection and preparation | 15 | 19 | 44 |
| [S20](../../../.temp/brief-606/draft-S20.md) | DeleteCases traversal and preparation | 16 | 20 | 42 |
| [S21](../../../.temp/brief-606/draft-S21.md) | Replace traversal and setup | 18 | 19 | 41 |
| [S22](../../../.temp/brief-606/draft-S22.md) | ReplaceList setup and scanning | 8 | 7 | 37 |
| [S23](../../../.temp/brief-606/draft-S23.md) | ReplaceList matching and guard collection | 13 | 9 | 38 |
| [S24](../../../.temp/brief-606/draft-S24.md) | Replace repeated and rule resolution | 11 | 12 | 43 |
| [S25](../../../.temp/brief-606/draft-S25.md) | ReplacePart targets and structure replacement | 20 | 19 | 59 |
| [S26](../../../.temp/brief-606/draft-S26.md) | Compiled matcher core preparation | 26 | 18 | 39 |
| [S27](../../../.temp/brief-606/draft-S27.md) | Sequence alternatives and collection | 10 | 8 | 38 |
| [S28](../../../.temp/brief-606/draft-S28.md) | Optional sequence defaults | 14 | 9 | 38 |
| [S29](../../../.temp/brief-606/draft-S29.md) | Sequence pattern and ordered preparation | 11 | 8 | 40 |
| [S30](../../../.temp/brief-606/draft-S30.md) | Flat sequence argument and rest | 17 | 12 | 39 |
| [S31](../../../.temp/brief-606/draft-S31.md) | Orderless matching and collection | 10 | 8 | 42 |
| [S32](../../../.temp/brief-606/draft-S32.md) | Orderless optional match and collect split | 10 | 13 | 38 |
| [S33](../../../.temp/brief-606/draft-S33.md) | Interpreted matcher preparation | 16 | 11 | 40 |
| [S34](../../../.temp/brief-606/draft-S34.md) | Association basic and missing | 15 | 9 | 50 |
| [S35](../../../.temp/brief-606/draft-S35.md) | Association grouping lists | 9 | 9 | 35 |
| [S36](../../../.temp/brief-606/draft-S36.md) | Association grouping entries | 9 | 9 | 36 |
| [S37](../../../.temp/brief-606/draft-S37.md) | Association aggregates and transforms | 18 | 18 | 45 |
| [S38](../../../.temp/brief-606/draft-S38.md) | Association callback phases | 8 | 10 | 37 |
| [S39](../../../.temp/brief-606/draft-S39.md) | Arrays, tensor operations and argument threading | 11 | 12 | 42 |
| [S40](../../../.temp/brief-606/draft-S40.md) | Apply, map and targeted updates | 6 | 6 | 46 |
| [S41](../../../.temp/brief-606/draft-S41.md) | Iterated function application | 8 | 8 | 35 |
| [S42](../../../.temp/brief-606/draft-S42.md) | Fold and while folds | 16 | 16 | 38 |
| [S43](../../../.temp/brief-606/draft-S43.md) | Select, window and sequence operations | 10 | 10 | 38 |
| [S44](../../../.temp/brief-606/draft-S44.md) | Lazy realization and outer captures | 5 | 5 | 49 |
| [S45](../../../.temp/brief-606/draft-S45.md) | Schema callback machine | 3 | 3 | 39 |
| [S46](../../../.temp/brief-606/draft-S46.md) | Truth, array and collection predicates | 17 | 17 | 46 |
| [S47](../../../.temp/brief-606/draft-S47.md) | Predicate ordering callbacks | 9 | 9 | 35 |
| [S48](../../../.temp/brief-606/draft-S48.md) | Guarded string Cases | 6 | 6 | 35 |
| [S49](../../../.temp/brief-606/draft-S49.md) | Guarded string replacement | 6 | 6 | 35 |
| [S50](../../../.temp/brief-606/draft-S50.md) | String search, rules and encoding | 16 | 16 | 72 |
| [S51](../../../.temp/brief-606/draft-S51.md) | Collection statistics and deduplication | 10 | 10 | 36 |
| [S52](../../../.temp/brief-606/draft-S52.md) | Ordering, sets and merge sorting | 10 | 11 | 37 |
| [S53](../../../.temp/brief-606/draft-S53.md) | Differences, lookups and remaining structure operations | 4 | 6 | 50 |
| [S54](../../../.temp/brief-606/draft-S54.md) | Arithmetic and remaining builtin wrappers | 2 | 2 | 35 |
| [F1](../../../.temp/brief-606/draft-F1.md) | Flip Option<Expr> | 0 | 0 | 30 |
| [F2](../../../.temp/brief-606/draft-F2.md) | Flip Vec<Expr> | 0 | 0 | 30 |
| [F3](../../../.temp/brief-606/draft-F3.md) | Flip Vec<SymbolExprMap> | 0 | 0 | 30 |
| [F4](../../../.temp/brief-606/draft-F4.md) | Flip Option<Vec<Expr>> | 0 | 0 | 30 |
| [F5](../../../.temp/brief-606/draft-F5.md) | Flip Option<Vec<RewriteRuleInfo>> | 0 | 0 | 30 |
| [F6](../../../.temp/brief-606/draft-F6.md) | Flip Option<CasesLevelSpec> | 0 | 0 | 30 |
| [F7](../../../.temp/brief-606/draft-F7.md) | Flip bool | 0 | 0 | 30 |
| [F8](../../../.temp/brief-606/draft-F8.md) | Flip String | 0 | 0 | 30 |
| [F9](../../../.temp/brief-606/draft-F9.md) | Flip CompiledMatchOutcome | 0 | 0 | 30 |
| [F10](../../../.temp/brief-606/draft-F10.md) | Flip CasesTraversalResolution | 0 | 0 | 30 |
| [F11](../../../.temp/brief-606/draft-F11.md) | Flip TraversalWithoutLimitResolution | 0 | 0 | 30 |
| [F12](../../../.temp/brief-606/draft-F12.md) | Flip CasesCollectionOutcome | 0 | 0 | 30 |
| [F13](../../../.temp/brief-606/draft-F13.md) | Flip PositionCollectionOutcome | 0 | 0 | 30 |
| [F14](../../../.temp/brief-606/draft-F14.md) | Flip DeleteCasesOutcome | 0 | 0 | 30 |
| [F15](../../../.temp/brief-606/draft-F15.md) | Flip TraversalCountOutcome | 0 | 0 | 30 |
| [F16](../../../.temp/brief-606/draft-F16.md) | Flip ReplaceOutcome | 0 | 0 | 30 |
| [F17](../../../.temp/brief-606/draft-F17.md) | Flip ReplaceAllTraversalOutcome | 0 | 0 | 30 |
| [F18](../../../.temp/brief-606/draft-F18.md) | Flip Vec<ReplacePartTarget> | 0 | 0 | 30 |
| [F19](../../../.temp/brief-606/draft-F19.md) | Flip EvalCapturedOutcome | 0 | 0 | 30 |
| [F20](../../../.temp/brief-606/draft-F20.md) | Flip PreparedCallArguments | 0 | 0 | 30 |
| [F21](../../../.temp/brief-606/draft-F21.md) | Flip EvaluatedPartSelectors | 0 | 0 | 30 |
| [F22](../../../.temp/brief-606/draft-F22.md) | Flip PreparedInterpretedCall | 0 | 0 | 30 |
| [F23](../../../.temp/brief-606/draft-F23.md) | Flip PreparedInterpretedAlternative | 0 | 0 | 30 |
| [R](../../../.temp/brief-606/draft-R.md) | Remove legacy transport/mode and rename macro | 0 | 600 spelling-only renames | 310 |

### 5. Open questions for the director

Does the intended slot registry retain the existing Rust identities of
PreparedInterpretedCall and PreparedInterpretedAlternative? This plan does,
and therefore starts both in Legacy under D1. If the intended carriers are
instead distinct new nominal identities, both must be State from birth in
S33; the counts become 21 Legacy / 45 new State and F22/F23 disappear. The
source evidence and existing logical sum shapes favor retaining the identities.
No counterexample to D1 was found in the audited populations.

Draft timeout ceilings carry forward the prior drafts; the dispatching
director must check them against recent equivalent gate logs as the drafts
require. This design round did not authorize new workspace or benchmark runs.
