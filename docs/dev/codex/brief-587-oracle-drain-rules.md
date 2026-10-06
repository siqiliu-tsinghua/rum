# brief-587 — E1 drain 1 of 8: the `rules` family (73 owed entries)

Implementation brief, **test-only**. **Do not run `git` in any form**,
including before you have read this brief. Leave every change uncommitted for
the director's gate.

## Goal

brief-586 built the catalog-wide suspension oracle
(`crates/rum-stdlib/tests/suspension_oracle/`) and left **500** declared
guest-effect entries in an exact owed ledger (`inventory/owed.tsv`). Eight
drain briefs empty it; the last deletes the ledger. **This one drains the 73
entries of the `rules` family** — exactly the IDs listed in
`.temp/brief-586/followups/draft-E1-13-rules.md` (compound assignments'
targets and updates, `Set`/`SetDelayed`/`TagSet`/`TagSetDelayed`/`TagUnset`
targets, destructuring, the containment predicates' option, guard,
pattern-test and preparation roles, `Level`'s options, `Position`, `ReplaceAt`,
`ReplacePart`, rule left-hand-side preparation, `Part`'s lazy consumer).

Each entry leaves the ledger **only in the same change that attaches and runs
its complete descriptor**: every required variant (`Middle`, `Twice`, and the
`Decline`, `Error`, `AlreadyResumed`, `Nested` variants its effect row
records), lifecycle at the first and second pause for every distinct
continuation owner (completion, cancellation, reset, a pending request
rejecting a new one), and a fresh request after.

That draft is the raw material; **this brief governs** where they differ.

## A failing descriptor is a finding, not a test to fix

If a correct descriptor shows the resumed path differing from the queued one —
a lost continuation the census did not see — **do not weaken the descriptor,
do not repair production code, and do not commit a failing test.** Keep the
entry owed, write the witness (source, answers, both observations) to
`.temp/brief-587/defects/<entry>.md`, and report it. It becomes a repair group
of this branch. The same holds for a manifest disposition you find wrong:
report it with source evidence; do not change it in this brief.

## Context pointers

- brief-586 and its report; `.temp/brief-586/followups/draft-E1-13-rules.md`.
- `crates/rum-stdlib/tests/suspension_oracle/{manifest,descriptors,catalog}.rs`,
  `support/`, `fixtures/`, `inventory/{catalog,effects,shared-effects,owed,historical-labels}.tsv`.
- The owning functions named in the draft; the guide
  (`evaluation.md`, `patterns-and-rules.md`) and the generated reference are
  the contract.
- The existing `destructuring_continuations.rs`, `rule_pattern_continuations.rs`
  and `option_reader_continuations.rs` cases, reusable as descriptors.

## Scope

**Edit surface:** the `suspension_oracle` test tree; the existing
continuation suites only to share descriptors (test names and specialized
assertions stay).

**Standing duty — always in scope** (`TEMPLATE.md`; test edits under `crates/`
expire the fingerprints): fingerprints, both certificates, ownership
reconciliation, every row disclosed. **STOP** on any existing row change or
disappearance, any new `UnknownHostPayload` or `self_reaching` row, any new
row in untouched code.

**Fence:** no production `.rs`, manifest disposition, `lib.rs`, Cargo
manifest, lockfile or dependency change; no other family's entries; no edits
to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier briefs, `scripts/` or
docs.

**Disclosure duty:** every entry drained, every entry kept owed and why, every
defect witness, every changed test name, every inventory row.

## Deliverable

Role-specific descriptor factories with explicit expected values, logs and
state projections and exact input kind, prompt and answer plans. A callback,
guard, option or held-target phase is exercised by its own role — ordinary
argument preparation never stands in. Loops run at least five operations and
pause in the middle and at a second position. Shared factories across the
family are welcome where they demonstrably exercise each entry's role.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix or resumed-only value the expected answer; expected
   values come from the documented contract and the queued path.
2. An eager-only or single-pause fixture proves nothing.
3. Support modules under 10k characters, scenario modules under 15k.
4. Stop-loss counters: entries drained, variants passing, lifecycle
   obligations passing, defects recorded. Stop after two consecutive compiling
   attempts that move none; fewer compiler errors is progress.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-587/tmp …`. Bounds: the oracle target 900 s per
build; runtime, ownership and retention 300 s; `cargo test --workspace
--no-fail-fast` 3 000 s.

```
cargo fmt --check
scripts/check-suspension-oracle.sh 900
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

The oracle's `CATALOG` lines must show `owed` reduced by exactly the drained
count in both builds. Report native ZeroMQ skips as skips.

## Checkpointing — protect the scene

Everything under `.temp/brief-587/`; `PROGRESS.md` after each milestone with
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
2. Entries drained and kept owed (with reasons); owed before and after, per
   build; variants and lifecycle obligations added.
3. Defect witnesses, if any; the broad suites verbatim; every inventory row
   change.
4. Stop-loss counters, open questions.

---

## Implementation report — brief 587

**70 of the 73 requested entries are drained. Three remain owed because their
manifest dispositions need director review.** Both feature builds now report
430 owed entries, down from 500. No production semantic mismatch was found.
All required verification commands completed successfully, with nine native
ZeroMQ permission skips disclosed below. Changes are left uncommitted.

### 1. Files changed and scope audit

```text
crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv
crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv
crates/rum-stdlib/tests/suspension_oracle/descriptors.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/primary.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rules/assignments.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rules/construction.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rules/falsifiers.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rules/lifecycle.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rules/mod.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rules/options.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rules/patterns.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rules/replacements.rs
crates/rum-stdlib/tests/suspension_oracle/fixtures/rules/support.rs
crates/rum-stdlib/tests/suspension_oracle/inventory/effects.tsv
crates/rum-stdlib/tests/suspension_oracle/inventory/owed.tsv
crates/rum-stdlib/tests/suspension_oracle/support/lifecycle.rs
docs/dev/codex/brief-587-oracle-drain-rules.md
```

The nine new `fixtures/rules/` modules separate assignments, matcher roles,
options, construction, replacements, shared setup, lifecycle, falsifiers and
the registry. Factories supply explicit values (or typed expected errors),
ordered logs and prints, state projections, and exact input kinds, prompts and
answers. `descriptors.rs` adds optional expected error/print checks.
`primary.rs` accepts an entry-aware factory and a lifecycle verifier; the six
existing descriptors retain their sources, expectations and variant sets.
`support/lifecycle.rs` adds an observer without changing the existing wrapper's
checks. No historical continuation suite was edited.

SHA256 comparison against the initial scene found no production Rust, `lib.rs`,
Cargo manifest/lockfile, dependency, script, benchmark, other documentation,
AGENTS, TODO or root-PROGRESS changes, and no removed baseline file. All changed
Rust modules are below 10k characters (largest: 5861). No new
unsafe. No git command, commit or external network access was used. No hot-path
change was made, so no performance gate was required or run. The audit and
baseline are [audit.txt](../../../.temp/brief-587/audit.txt) and
[baseline-hashes.json](../../../.temp/brief-587/baseline-hashes.json).

Existing test names were not changed or removed. The one added test is
`oracle::fixtures::rules::falsifiers::every_role_rejects_missing_pause_and_lost_log`.
The 209 historical descriptors and their ledger are unchanged. The exact live
historical name comparison is:

```text
suite	before	after	name set equal
default_fallback_continuations	21	21	true
destructuring_continuations	16	16	true
guest_loading_resume	47	47	true
lazy_pull_continuations	69	69	true
loop_continuations	21	21	true
option_reader_continuations	100	100	true
rule_pattern_continuations	71	71	true
schema_continuations	40	40	true
sow_trace_continuations	51	51	true
```

### 2. Entries, coverage and debt

Each ID below identifies both inventory edits: its `effects.tsv` descriptor
cell changes from `-` to that exact ID, and its original `owed.tsv` row is
removed. **Every other effect field is byte-identical**, including policy,
owner, variants and lifecycle owner. All other families are unchanged.

```text
AddTo.target
AddTo.update
AppendTo.target
AppendTo.update
ApplyTo.target
ApplyTo.update
Count.defaultOption
Count.explicitOption
Count.guard
Count.optionSyntax
Count.patternPreparation
Count.patternTest
Decrement.target
Decrement.update
DivideBy.target
DivideBy.update
FreeQ.defaultOption
FreeQ.explicitOption
FreeQ.guard
FreeQ.optionSyntax
FreeQ.patternPreparation
FreeQ.patternTest
Increment.target
Increment.update
Level.defaultOption
MemberQ.defaultOption
MemberQ.explicitOption
MemberQ.guard
MemberQ.optionSyntax
MemberQ.patternPreparation
MemberQ.patternTest
NormalizeRules.lhsPreparation
Position.defaultOption
Position.explicitOption
Position.guard
Position.heldSpec
Position.heldTarget
Position.optionSyntax
Position.patternPreparation
Position.patternTest
PreDecrement.target
PreDecrement.update
PreIncrement.target
PreIncrement.update
PrependTo.target
PrependTo.update
ReplaceAt.delayedRhs
ReplaceAt.guard
ReplaceAt.heldSpec
ReplaceAt.heldTarget
ReplaceAt.patternPreparation
ReplaceAt.patternTest
ReplacePart.delayedRhs
ReplacePart.guard
ReplacePart.patternPreparation
ReplacePart.patternTest
Rule.lhsPreparation
RuleDelayed.lhsPreparation
Set.destructuring
Set.target
SetDelayed.target
SubtractFrom.target
SubtractFrom.update
TagSet.target
TagSetDelayed.target
TagUnset.target
TimesBy.target
TimesBy.update
UpSet.target
UpSetDelayed.target
```

The exhaustive old/new effect rows are in
[effect-row-changes.tsv](../../../.temp/brief-587/effect-row-changes.tsv), and
every removed owed row is in
[owed-rows-removed.tsv](../../../.temp/brief-587/owed-rows-removed.tsv).

| Kept owed | Reason and witness |
| --- | --- |
| `Level.explicitOption` | Ordinary argument preparation evaluates the explicit immediate option rhs; the local Level handler only classifies it. [Source evidence](../../../.temp/brief-587/defects/Level.explicitOption.md). |
| `Level.optionSyntax` | Computed option syntax also belongs to ordinary argument preparation. The local classifier performs no guest evaluation. [Source evidence](../../../.temp/brief-587/defects/Level.optionSyntax.md). |
| `Part.lazyConsumer` | `eval_part` directly returns evaluation of `First[LazyDrop[...]]`, a tail handoff, while the manifest says Frame. Its candidate passed all five variants and six lifecycle checks, but the disposition remains unchanged and the entry remains owed. [Source evidence](../../../.temp/brief-587/defects/Part.lazyConsumer.md). |

For Level, `builtin_catalog/rules.rs:511` registers ordinary evaluation;
`tree_walkers/level/options.rs:42,61` calls the pure classifier in
`messages/parse_and_message_helpers.rs:367`. The distinct default read in
`options/core.rs:12` reads stored/default options and is covered by
`Level.defaultOption`. For Part, the direct return is at
`kernel_data/structure_data/selectors.rs:125`. No eager-preparation fixture
was substituted for a claimed local Level role, and no disposition was edited.

| Build | Owed before | Owed after | Drained | Complete descriptors | Completion variants | Lifecycle obligations |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| default | 500 | 430 | 70 | 76 | 381 | 432 |
| no-default | 500 | 430 | 70 | 76 | 381 | 432 |

The additions per build are **362 variants** (70 each Middle, Twice, Error,
AlreadyResumed and Nested, plus 12 Decline) and **420 lifecycle obligations**
(cancellation, reset and rejection of an overlapping request at both pauses
for each of 70 owners). Completion runs use the same descriptor, and every
terminal path admits a fresh request. Exact queue exhaustion is checked before
state projections. Reset uses Session replacement from identical configuration,
as in the existing runner; rum-host owns the host reset API tests.

Five-operation workloads pause at the second and fourth operations. Target
selectors and lhs arguments, update callbacks/upvalues, pattern preparation,
matcher predicates/guards, option values/syntax and replacement bodies run in
their own roles. Cancellation/reset projections check committed destructuring
prefixes, uncommitted assignment/update stores, callback logs and scope state.
ReplacePart guards use five competing rules at one position, exercising the
guide's documented last-to-first order. Each admitted entry also has missing-
pause and lost-log falsifiers, with weakened-checker and good controls:
**140 rejecting falsifiers per build**. Typed queued/resumed observations,
messages and cumulative output comparison remain enabled; no normalization or
weakened acceptance is used by admitted descriptors.

### 3. Findings, verification and every maintenance row

Three manifest witnesses are listed above; there are **zero production semantic
defect witnesses**. The first candidate run rejected an invalid assumption in
the initial ReplacePart.guard fixture: queued and resumed observations agreed
on reverse position traversal, while the fixture expected ascending positions.
That order is not promised by the guide. The final workload instead tests the
documented order of five competing guards, retaining all six required variants
and both pauses. No production code was repaired and no failing descriptor
remains attached. Investigation output is preserved in
[candidates-01.txt](../../../.temp/brief-587/candidates-01.txt),
[candidate-ReplacePart.guard.md](../../../.temp/brief-587/candidate-ReplacePart.guard.md)
and [candidates-02.txt](../../../.temp/brief-587/candidates-02.txt).

All Cargo commands ran serially, offline, with `CARGO_BUILD_JOBS=1`,
`RUST_TEST_THREADS=1`, and caller TMPDIR `.temp/brief-587/tmp`, through
`scripts/run-bounded.py`. Oracle builds were bounded at 900 seconds each;
runtime/ownership/retention at 300; workspace at 3000. The workspace command
was `cargo test --workspace --no-fail-fast -- --nocapture`, with the supported
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` permission-skip flag.

`cargo fmt --check` exited 0 after scoped formatting. The initial formatting
check failed only on the new edits; both logs are saved. No mandatory command
was omitted. The unchanged required oracle script internally overrides TMPDIR
and emits logs under `.temp/suspension-oracle`; those generated logs were copied
into this brief's directory. The mandatory fingerprint writer emits its review
copy under `target/brief-544`; certificate replay uses the prescribed
`.temp/brief-546/certificates/`. All authored scratch files/checkpoints are under
`.temp/brief-587/`.

Verbatim oracle results from `scripts/check-suspension-oracle.sh 900`:

```text
LIFECYCLE obligations=432 (plus completion for every variant)
CATALOG hash=true installed=643 manifest=643 GuestEntries=240 NoLocalGuestEffect=403 Boundary=4 entries=506 descriptors=76 historical=209 owed=430
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.78s
LIFECYCLE obligations=432 (plus completion for every variant)
CATALOG hash=false installed=637 manifest=637 GuestEntries=240 NoLocalGuestEffect=397 Boundary=4 entries=506 descriptors=76 historical=209 owed=430
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.72s
```

Verbatim `cargo test -p rum-runtime` results:

```text
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.84s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.97s
```

Verbatim ownership reconciliation and result:

```text
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7683 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 61.59s
```

Every ownership inventory is byte-identical. There are **zero ownership row
changes**, including positions; no new UnknownConstruct, UnknownHostPayload or
self-reaching row was added. No closed-frame list changed.

The maintenance writer regenerated all 68 contract fingerprints. Every
non-fingerprint field remains identical. All changed rows, verbatim:

```text
id	before	after
B01	fnv1a64:9cc55546787aed0e	fnv1a64:c7ca4497c1289b99
B02	fnv1a64:87653a1853c15bb7	fnv1a64:b9da2fea8b257e74
B03	fnv1a64:543f9347d8e3dae4	fnv1a64:08448190db456cfb
B04	fnv1a64:68c7bed338e9b386	fnv1a64:f0af8100526e03c1
B05	fnv1a64:aaacfade47b8268b	fnv1a64:cd7982cad800b0d8
B06	fnv1a64:eb3a6f2aafdf9dea	fnv1a64:019bb1612e355d35
B07	fnv1a64:91a81a4c9a257edb	fnv1a64:570a8b53dbb94f68
B08	fnv1a64:87ae900c59fbec08	fnv1a64:37bd6c9dfb972a3f
B09	fnv1a64:baeed2680d4d1d42	fnv1a64:eb683f91324357fd
B10	fnv1a64:dbe2e4dca7c78015	fnv1a64:eb530a0d02ab008e
B11	fnv1a64:2a1f703052ddd986	fnv1a64:b207325d6c6229c1
B12	fnv1a64:79fb5263e955b242	fnv1a64:aa74bf8d0e4becfd
I01	fnv1a64:7aa74043e532e38e	fnv1a64:a5ac2f952de09219
I02	fnv1a64:01ee78723feeae8b	fnv1a64:24bb005ed03738d8
I03	fnv1a64:76e6ac401bd1c102	fnv1a64:45764f3df2e6e0bd
I04	fnv1a64:64b4db5dc32855d7	fnv1a64:2f4f728f0dcd5414
P01	fnv1a64:b4222e76367c7f0a	fnv1a64:1b24abd49a60f655
P02	fnv1a64:35c63e7603b2e445	fnv1a64:53155d20d6b0d47e
P03	fnv1a64:795197df27a15a2c	fnv1a64:f6ee32dc1ad1c493
P04	fnv1a64:7c40582dd86a77b7	fnv1a64:aeb54e000fce9a74
P05	fnv1a64:5b78b9f288e52dfe	fnv1a64:d6660ab9ed319609
P06	fnv1a64:ca7a773475854bc2	fnv1a64:faf3e45d9a7b867d
P070	fnv1a64:04c8137653fac5c2	fnv1a64:3541809f78f1007d
P080	fnv1a64:34a0f197e21e87d5	fnv1a64:f0af8fea0e270f4e
P090	fnv1a64:bbe90c1c58b85c77	fnv1a64:a2c4566bb1d22834
P100	fnv1a64:2ec97dcbb4565296	fnv1a64:163181cb2e904f91
P110	fnv1a64:9a9ad29c2e141e1d	fnv1a64:4b000df1b95310a6
P120	fnv1a64:b98e0701308817a8	fnv1a64:5b50053e5ef5d55f
P130	fnv1a64:357573a9c8d958db	fnv1a64:fad7e4b10a6d2968
P140	fnv1a64:2d4dd9256747761f	fnv1a64:c016498e0a91606c
P150	fnv1a64:9a2914528af6f397	fnv1a64:1f0ca0315ae7cdd4
P071	fnv1a64:dd9a3565550d527b	fnv1a64:fc1c8e6f67701a88
P081	fnv1a64:ef29ce2aac0987e2	fnv1a64:40d451f223dcf19d
P091	fnv1a64:f4ef80adaee07dca	fnv1a64:93e14efea47c7415
P101	fnv1a64:929dc3696377bb1d	fnv1a64:4302febeeeb6ada6
P111	fnv1a64:0c7f257946488ed2	fnv1a64:a91dd27f16760b0d
P121	fnv1a64:1ed585c9e8faf791	fnv1a64:6345378c86c9fa9a
P131	fnv1a64:0671e1396f3dd506	fnv1a64:8e59a36688c22541
P141	fnv1a64:0364eeb8d56d7faa	fnv1a64:78613c127bd639f5
P151	fnv1a64:05d20700c914ece8	fnv1a64:4cce183ff5554d9f
B13	fnv1a64:794b02b3e07c59ea	fnv1a64:8fac44ea5ed21935
R01	fnv1a64:cdab66b698497287	fnv1a64:9c9734e1b2ddd984
R02	fnv1a64:ab975470ad6171ec	fnv1a64:762d4fbfa21b2453
R03	fnv1a64:ff8d961bac621153	fnv1a64:d8321bd640c36850
R040	fnv1a64:2b7452027c9b88a7	fnv1a64:9c3e44335cf9af24
R050	fnv1a64:f7ae3995e02a55de	fnv1a64:aec7a37cb299a069
R060	fnv1a64:ead2b5bd86755aef	fnv1a64:bbc4f8ada0044c3c
R070	fnv1a64:579796448dab46e2	fnv1a64:a9421a0c057eb09d
R080	fnv1a64:2bb9445092ad1ed7	fnv1a64:f653db81dd521d14
R090	fnv1a64:bb6b3204a2d67056	fnv1a64:ee6de186fb5c7751
R100	fnv1a64:1965379a82b98564	fnv1a64:cd6a25e3851b177b
R110	fnv1a64:0875c80c8de71fed	fnv1a64:e94a722266c93cf6
R041	fnv1a64:5f8465ddf7a418ea	fnv1a64:75e5a81475f9d835
R051	fnv1a64:4a0b3a31e5e668f7	fnv1a64:30e684813f0034b4
R061	fnv1a64:e12983b695ffff78	fnv1a64:a4fbda2ec37f8aef
R071	fnv1a64:4f28020b3253e869	fnv1a64:1e8f5073d7d7e082
R081	fnv1a64:eab3e72050022796	fnv1a64:d21beb1fca3c2491
R091	fnv1a64:58cfc016c856ff5f	fnv1a64:443f8e0c649dabac
R101	fnv1a64:ef4747e4ab30a5af	fnv1a64:7548888d3953e0fc
R111	fnv1a64:9faacffe41d23640	fnv1a64:435fa8fc55a9b327
R12	fnv1a64:e785687dca663180	fnv1a64:e6012e79e06cbe67
R13	fnv1a64:3ac040d81ac8a894	fnv1a64:4ca57bf089dfb3ab
R14	fnv1a64:49218c4191071296	fnv1a64:308990410b410f91
R15	fnv1a64:a25738e923bf4b1a	fnv1a64:68985b307cd7f1e5
O010	fnv1a64:6fa6637790c4ffd8	fnv1a64:81f1537f35e2c84f
O0200	fnv1a64:28157af9b5fe9c1f	fnv1a64:baddeb625948866c
O011	fnv1a64:2d5b724594071f6f	fnv1a64:fe4db535ad9610bc
O0210	fnv1a64:ae12e7c4fd6d76e0	fnv1a64:9fd4dfe0f234fac7
```

Both initial certificate replays rejected stale C01 cells, as expected after
the fingerprint changes, while emitting the current C01/C02 certificates. Four
cells were copied from those outputs; neither identity nor status, source,
preconditions, strength, exclusions, body/domain/build fingerprints or instance
marks changed. Exact string comparison restricts changes to these direct-
contract fingerprints:

```text
body	feature	changed direct-contract IDs
C01	hash=true	O010,O0200,O011,O0210
C01	hash=false	O010,O0200,O011,O0210
C02	hash=true	B05,I01,I02,I04,B13,R01,R02
C02	hash=false	B05,I01,I02,I04,B13,R01,R02
```

Verbatim initial rejection evidence (subsequently replayed to green):

```text
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.38s
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.33s
```

The complete old/new cells are in
[certificate-changes.tsv](../../../.temp/brief-587/certificate-changes.tsv).
Both final required `local_production_acceptance` runs passed:

```text
default:
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.34s
no-default:
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.30s
```

Workspace exited 0. Its harness reports 3362 passed, 0 failed and
58 ignored across 49 targets. **Nine of those harness-success
returns are native ZeroMQ permission skips, not executed native validations.**
Their complete names and messages are in
[native-skips.txt](../../../.temp/brief-587/native-skips.txt). The existing ignored
tests retain their status; the ignored fingerprint writer was also explicitly
run as maintenance. Verbatim workspace target/result summaries:

```text
Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.15s
Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.29s
Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.98s
Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1139 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 29.47s
Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1103.55s
Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.07s
Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s
Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-1756073e73a214f6)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.24s
Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.62s
Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.34s
Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.84s
Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Raw logs: [workspace](../../../.temp/brief-587/workspace.txt),
[oracle](../../../.temp/brief-587/oracle-gate.txt),
[runtime](../../../.temp/brief-587/runtime.txt),
[ownership](../../../.temp/brief-587/ownership.txt),
[maintenance](../../../.temp/brief-587/maintenance.txt).
`catalog.tsv`, `shared-effects.tsv`, `historical-labels.tsv` and every other
inventory are unchanged; only the 70 effect attachments, 70 owed removals,
68 fingerprint fields and four certificate cells described above changed.

### 4. Stop-loss counters and handoff

Final admitted counters: **70 entries drained; 362 completion variants and 420
lifecycle obligations passing per build; three manifest witnesses recorded;
zero semantic mismatches; 140 falsifiers per build; 436 historical test names
preserved.** The separate Part candidate contributes five passing variants and
six lifecycle checks to investigation evidence, without discharging its debt.
The compiling candidate attempts advanced these counters; there were no two
consecutive implementation attempts without progress. Mandatory verification
and the expected stale-certificate/replay controls are recorded separately.

The only open decisions for this drain are the three unchanged manifest rows
above. E1 remains open with 430 owed entries, including those three rules
entries. No production repair or disposition change was attempted. The tree
is ready for the director's authoritative gate; all changes remain uncommitted.
The durable checkpoint is
[PROGRESS.md](../../../.temp/brief-587/PROGRESS.md).
