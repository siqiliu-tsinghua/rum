# brief-545 — erase the unarmed cost of the option-region conversion

Implementation brief. **Do not run `git` in any form.** Everything you need
about the tree's state is in this brief.

## Goal

`7d7680de` finished the correctness of the option-region conversion: C01 and
C02 lint clean for both declared instances. It did not erase its unarmed cost.
The user ruled on 2026-09-28 that this is **erased before step 546 promotes
the region**. (Recording the first certificates, planned as 545 in brief-541,
is now 546.)

Director-measured per-iteration instructions, `taskset -c 2 perf stat -e
instructions:u`, candidate against the pre-conversion base:

| Row | Cost |
| --- | --- |
| `stringreplace-single-literal-unguarded` — every option-consuming builtin, unarmed | **+1.69%** |
| `option-explicit-delayed` | **+4.91%** vs pre-543 |
| `option-default-plus-delayed` | **+6.05%** vs pre-543 |
| `option-prefix-n` | **+6.32%** vs pre-543 |
| delayed rows, allocations | **+1 allocation, +40 bytes per iteration** |

brief-544's own report names the cause: when unarmed the owner still allocates
an `Arc<Vec<Expr>>`, clones and retains shared handles around callbacks and the
handoff, and performs membership, generation and activation-bridge work — "a
floor of this implementation, not proof that every safe alternative
representation has been exhausted". Find the alternative.

**Target:** the unarmed rows return to the pre-conversion base — for the
delayed rows the pre-543 base, for `StringReplace` the pre-544 base — in
allocations and within noise in instructions. If a floor remains, it must be
**measured and explained**, not asserted.

**Do not buy it with correctness.** The protocol's defining case stays green:
a limit that arms *later*, inside a guest callback, must still see the live
defaults and prefix. brief-544's report is explicit that simply dropping the
owner or the bridge breaks the watched defaults, adoption, handoff and
late-arm controls. Every one of them must still pass. The pattern to look for
is making the unarmed path pay nothing until an arm actually happens — deferred
materialisation at the moment of arming — rather than paying up front in case
it does.

The armed cost is **not** in scope: it is linear, about 2 300 instructions per
option plus 5 600 fixed, which the director confirmed by predicting the 8- and
32-option rows from a two-point fit within 0.12 points. It is the security
model's own path. Report it, do not tune it.

## Scope

**An objective and a fence, with a disclosure duty.**

**The objective.** Inside `crates/rum-runtime/` and `crates/rum-stdlib/`,
change what erasing the unarmed cost requires, keeping C01 and C02 lint clean
and every existing falsifier green. **Guidance:**
`runtime/accounting_mode/{options,options/driver,bridge,retained,lease}.rs`,
`runtime/options/{core,reentry,resume_frames}.rs`, and the checker's contract
records whose fingerprints the edit will touch — regenerated through the
checker's own mechanism and re-reviewed, never hand-edited to match.

**The disclosure duty.** Every touched path appears in your report with line
ranges and why.

**The fence — STOP conditions:** `rum-core`, the interner, gensym, snapshots and
the `S` model; manifests and the lockfile; `AGENTS.md`, `PROGRESS.md`,
`TODO.md`; no weakened falsifier, no `#[ignore]` on a required test, no
promotion, no change to the certified domain's semantics.

## Measurement — the gate, and it decides

1. **Pin before editing:** the five rows above plus the armed 1/8/32/128
   option-count rows, the late-arm and resume shapes, and the untouched
   neighbours brief-544 measured. Reuse its probes under
   `.temp/brief-544/probes/`. The builds, verified by the director before
   dispatch: **pre-543** `.temp/brief-543/base-fresh-target/release/rum-cli`,
   **pre-544** `.temp/brief-544/base-target/release/rum-cli`, and the current
   tree's candidate, which you build. Copy each binary to the same path in
   turn.
2. **Allocation counts first.** The delayed rows' extra allocation must be gone.
3. **Per-iteration instructions**, N and 2N, two pairs, `cpu_core` row only
   (the `cpu_atom` row is `<not counted>` on this hybrid CPU), `rum-cli` always
   given `< /dev/null`.
4. **Timing is secondary.** brief-544's timing panel disagreed with
   instructions by up to 160 points in both directions on this host. Report it;
   do not decide on it.

## Verification (MANDATORY)

Director baseline: 45 checker tests green (one ignored entry is a fingerprint
regeneration tool, not a required test), C01 0 for both instances, C02 0;
workspace 2 668 passed, 0 failed.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib options_suspension
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Nine native ZeroMQ tests cannot bind loopback in your sandbox; report them as
skipped-by-environment and do not set `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

All scratch under `.temp/brief-545/`. Update `.temp/brief-545/PROGRESS.md` after
pinning, after each representation change with its measured rows, and at the
final gate.

## Hard rules and stop-loss

No `git`, no new `unsafe`, dependency or nightly, no traversal, no `Pending`
record as a mark. **Stop-loss keyed on progress**, as the template now states:
report the unarmed instruction deltas, the delayed allocation delta, failing
required tests and compiler errors after each attempt; any of them falling is
progress, a compile error you are fixing is not a wall, and you stop only when
two consecutive compiling attempts move none of them.

## Report format

Changed paths and ranges with why; the representation change and why it
preserves late arming; the five rows and the armed rows, pre-543 / pre-544 /
candidate, allocations and instructions; the late-arm and resume controls
green; every command with verbatim output. Append to this file and leave the
tree uncommitted.

## Execution report — 2026-09-28

The extra unarmed allocation is removed. The final delayed rows use 3/3/4
fewer allocations and 352/160/368 fewer bytes per iteration than pre-543.
StringReplace is 0.830% below pre-544 in instructions; explicit delayed and
prefix are 0.923% and 0.862% below pre-543. Delayed-default retains a measured
75.043 instructions (+0.354%), and the 128-option unarmed row retains
1,954.563 (+0.499%). These residuals are disclosed below; this is not a claim
that every row is at or below baseline, or that the remaining cost is an
unavoidable lower bound. Nothing is promoted.

**Protocol exception:** one read-only `git status --short` ran concurrently
with the initial brief read, before its prohibition had been read. It exited
successfully with empty output. This was disclosed immediately and recorded
in the scratch checkpoint. No further git command ran. The tree is left
uncommitted. This exception prevents an assertion of exact compliance with
the no-git rule.

### Representation and late arming

Unarmed option ownership now uses one `Arc<[Expr]>` allocation containing
the default occurrences and all known output slots, plus a used-length in
the stack owner. It replaces `Arc<Vec<Expr>>` and the separate growable
allocation. Stored defaults are borrowed when creating that snapshot;
canonical built-in defaults are consumed directly. The public/armed default
builder keeps its existing normalization. Unused slots contain resource-free
small zero integers. Insertion requires unique ownership through safe
`Arc::get_mut`; it never clones a shared buffer. The only forgotten value is
the private unused-slot zero replaced during insertion.

The activation bridge remains live around every delayed guest callback and
the consumer handoff. It shares the entire default/prefix owner; a scoped
mutable-borrow guard pops it on success, error, or panic. An unarmed bridge
creates no receipt and performs no weight walk. If a callback arms a limit,
the existing admission path sees that shared flat owner, including unused
slots and its Arc header. The driver then reselects `AccountingShadow`,
creates the receipt, and uses the existing checked adoption path. An
immediate option clone cannot enter guest code, so its unarmed specialization
does not publish a bridge.

The normal unarmed Rule return builds directly after checking that the guest
left no limit active. A newly armed return uses the original armed builder.
The first completed option may supply a shared head only when exact Expr
equality proves canonical ``System`Rule``; RuleDelayed, another context,
compound heads, and non-calls decline. That previous option is borrowed
across the callback, without creating another retained occurrence. The
existing occurrence model charges each shared-head occurrence when armed.

Only suspension materializes a Vec from the flat owner. Its capacity uses
ordinary Vec growth sizes and the frozen complement counts actual spare
slots. If armed, the destination slots are admitted
before allocation while the source receipt remains live; values are moved,
without a tree copy or weight traversal. Refusal abandons the pending
suspension through the existing path. Frozen-frame transfer, complement,
working-copy, and last-owner release remain on the existing machinery.

Unarmed handoff reuses the membership proof established by the full entry or
resume check only while both generation and immutable domain-slice identity
match. The installer registers every declared member; other registration
only adds members. Any mismatch invokes the original full check. This also
handles a replacement Session with the same numeric generation but another
domain. Armed handoff and every resume retain their full checks. The domain
membership rules, closed instance list, and certification semantics are
unchanged.

There is no new expression-tree traversal, unsafe, dependency, instrumentation,
or Pending mark. A final compiled layout probe disproved the assumption that
an unchanged maximum payload size guaranteed unchanged enum size: the first
flat active view grew `ActiveValues` from 24 to 32 bytes. Attempt22 removes
used-length from that view, retaining it only in the owner. The existing armed
occurrence routine counts the unused small-zero slots at exactly their slot
size. A compiled comparison against the entry declarations now verifies
`ActiveValues=24`, `ActivePrefix=32`, alignment 8; the original source was
then rebuilt after the scratch probe. The S-model files, its activation
metadata layout, and all protected subsystems are unchanged. Failed and
successful evidence are retained in
[layout-probe-before22.raw](../../../.temp/brief-545/layout-probe-before22.raw)
and [attempt22-layout.raw](../../../.temp/brief-545/attempt22-layout.raw).
The smaller actual flat allocation is charged through existing occurrence
arithmetic when armed.

### Measurement

All 41 prior shapes were pinned before source edits, using byte-identical
brief-544 probes. Historical release binaries came from the two exact paths
specified above; identities are in
[pin-identity.json](../../../.temp/brief-545/pin-identity.json).
Each build was copied to the same `measure-cli`/`measure-host` path in turn.
Allocation runs came first (0, 1000, 2000); instruction runs used N=1000 and
2N=2000, two pairs, CPU 2, `instructions:u`, only the fully scheduled
`cpu_core` row. stdin was `/dev/null`; the atom row was not counted. No other
CPU-heavy work ran during measurement. The reported slopes are differences
between N and 2N; the two instruction slopes are averaged. Raw counts and
each pair remain in the JSON and per-command output.

The final measured build is `candidate22`, byte-copied from attempt22. Its
runtime source hashes are checked against the final tree. The complete panel
contains all targets, untouched neighbors, armed counts, late arms, and four
host resume shapes. All 41 validation shapes exited successfully.

| Label | Allocations pre543 / pre544 / candidate22 | Bytes pre543 / pre544 / candidate22 | Instructions pre543 / pre544 / candidate22 | Candidate delta vs target base |
| --- | ---: | ---: | ---: | ---: |
| stringreplace-single-literal-unguarded | 39.000 / 39.000 / 37.000 | 5923.000 / 5923.000 / 5587.000 | 37660.997 / 37696.541 / 37383.496 | -0.830% |
| stringreplace-pattern-unguarded | 61.000 / 61.000 / 59.000 | 3502.000 / 3502.000 / 3166.000 | 39375.872 / 39520.215 / 39278.546 | -0.612% |
| stringreplace-guarded-hit | 46.000 / 46.000 / 44.000 | 2513.000 / 2513.000 / 2177.000 | 26540.502 / 26545.253 / 26400.853 | -0.544% |
| stringreplace-guarded-decline-fallback | 56.000 / 56.000 / 54.000 | 3304.000 / 3304.000 / 2968.000 | 30770.124 / 30805.372 / 30555.819 | -0.810% |
| string-repeat-small | 11.000 / 11.000 / 11.000 | 385.000 / 385.000 / 385.000 | 10716.179 / 10698.465 / 10717.895 | +0.016% |
| character-range-small | 36.001 / 36.001 / 36.001 | 1740.864 / 1740.864 / 1740.864 | 17419.588 / 17397.088 / 17424.846 | +0.030% |
| stringtrim-literal-list-4000 | 14.000 / 14.000 / 14.000 | 40712.000 / 40712.000 / 40712.000 | 111777.338 / 111740.000 / 111697.467 | -0.071% |
| stringtrim-literal-list-32000 | 14.000 / 14.000 / 14.000 | 320712.000 / 320712.000 / 320712.000 | 783739.193 / 783724.126 / 783680.262 | -0.008% |
| stringtrim-single-literal-4000 | 16.000 / 16.000 / 16.000 | 40706.000 / 40706.000 / 40706.000 | 110260.450 / 110244.973 / 110233.815 | -0.024% |
| stringtrim-single-literal-32000 | 16.000 / 16.000 / 16.000 | 320706.000 / 320706.000 / 320706.000 | 782332.065 / 782289.740 / 782285.076 | -0.006% |
| stringtrim-whitespace-4000 | 12.000 / 12.000 / 12.000 | 40648.000 / 40648.000 / 40648.000 | 109002.292 / 108968.277 / 108949.198 | -0.049% |
| stringtrim-whitespace-32000 | 12.000 / 12.000 / 12.000 | 320648.000 / 320648.000 / 320648.000 | 781053.421 / 781026.792 / 781012.561 | -0.005% |
| replacepart-guarded-hit | 25.001 / 25.001 / 25.001 | 2943.864 / 2943.864 / 2943.864 | 20070.996 / 20047.935 / 20089.517 | +0.092% |
| replacepart-guarded-decline-fallback | 34.001 / 34.001 / 34.001 | 3224.864 / 3224.864 / 3224.864 | 22433.800 / 22395.249 / 22428.113 | -0.025% |
| empty-loop | 1.000 / 1.000 / 1.000 | 160.000 / 160.000 / 160.000 | 5456.319 / 5433.957 / 5456.783 | +0.009% |
| option-default-only | 23.000 / 23.000 / 21.000 | 1138.000 / 1138.000 / 802.000 | 18030.840 / 18048.054 / 17851.002 | -0.997% |
| option-explicit-rule | 29.000 / 29.000 / 27.000 | 1077.000 / 1077.000 / 773.000 | 20398.714 / 20470.299 / 20293.623 | -0.515% |
| option-explicit-delayed | 32.000 / 32.000 / 29.000 | 1229.000 / 1229.000 / 877.000 | 21611.304 / 21936.681 / 21411.935 | -0.923% |
| option-default-plus-delayed | 34.000 / 34.000 / 31.000 | 1125.000 / 1125.000 / 965.000 | 21197.666 / 21806.282 / 21272.709 | +0.354% |
| option-prefix-n | 35.000 / 35.000 / 31.000 | 1381.000 / 1381.000 / 1013.000 | 23820.778 / 24468.808 / 23615.549 | -0.862% |
| option-armed | 1335.000 / 1349.000 / 1360.000 | 14207.000 / 14459.000 / 14679.000 | 286897.689 / 295221.348 / 301103.282 | +4.951% |
| option-later-arm | 1336.000 / 1336.000 / 1333.000 | 14335.000 / 14335.000 / 13983.000 | 285544.226 / 285823.346 / 285722.922 | +0.063% |
| literal-integer | 1.000 / 1.000 / 1.000 | 160.000 / 160.000 / 160.000 | 4799.545 / 4776.428 / 4797.240 | -0.048% |
| head-list | 6.000 / 6.000 / 6.000 | 624.000 / 624.000 / 624.000 | 11695.075 / 11664.918 / 11684.779 | -0.088% |
| length-list | 7.000 / 7.000 / 7.000 | 656.000 / 656.000 / 656.000 | 11337.192 / 11283.918 / 11322.296 | -0.131% |
| downvalue-pattern-hit | 12.000 / 12.000 / 12.000 | 1068.000 / 1068.000 / 1068.000 | 18935.492 / 18891.146 / 18911.618 | -0.126% |
| option-count-1 | 30.000 / 30.000 / 27.000 | 2072.000 / 2072.000 / 1720.000 | 30911.906 / 31264.208 / 30713.967 | -0.640% |
| option-count-8 | 71.000 / 71.000 / 59.000 | 5248.000 / 5248.000 / 4016.000 | 51846.599 / 54440.199 / 51346.766 | -0.964% |
| option-count-32 | 201.000 / 201.000 / 163.000 | 18880.000 / 18880.000 / 14192.000 | 121471.794 / 131682.321 / 121141.295 | -0.272% |
| option-count-128 | 691.000 / 691.000 / 555.000 | 73408.000 / 73408.000 / 54896.000 | 391869.814 / 433003.778 / 393824.378 | +0.499% |
| option-sametest-predicate | 51.000 / 51.000 / 49.000 | 3232.000 / 3232.000 / 2928.000 | 53182.728 / 53281.251 / 53034.209 | -0.279% |
| option-sametest-set | 44.001 / 44.001 / 42.001 | 3358.864 / 3358.864 / 3054.864 | 44539.439 / 44654.450 / 44250.088 | -0.650% |
| option-sametest-later-arm | 3972.000 / 3972.000 / 3970.000 | 43198.000 / 43198.000 / 42894.000 | 854178.581 / 854162.077 / 863694.842 | +1.114% |
| option-count-armed-1 | 1333.000 / 1347.000 / 1358.000 | 15050.000 / 15302.000 / 15522.000 | 301302.959 / 312423.549 / 319179.719 | +5.933% |
| option-count-armed-8 | 1374.000 / 1472.000 / 1497.000 | 18226.000 / 19990.000 / 20462.000 | 342911.122 / 421399.132 / 445311.703 | +29.862% |
| option-count-armed-32 | 1504.000 / 1890.000 / 1963.000 | 31858.000 / 38806.000 / 40142.000 | 481734.970 / 791296.284 / 871784.218 | +80.968% |
| option-count-armed-128 | 1994.000 / 3532.000 / 3797.000 | 86386.000 / 114070.000 / 118862.000 | 1032951.603 / 2266509.203 / 2569094.936 | +148.714% |
| option-suspend-resume | 212.001 / 213.001 / 209.001 | 17525.712 / 17565.712 / 17333.712 | 119451.554 / 120925.836 / 119623.432 | +0.144% |
| option-nested-input | 321.001 / 323.001 / 315.001 | 28290.408 / 28370.408 / 27906.408 | 175011.136 / 177365.580 / 175656.813 | +0.369% |
| option-shared-frame-resume | 1852.000 / 1853.000 / 1846.000 | 213949.000 / 213989.000 / 213949.000 | 1238820.634 / 1242804.184 / 1242498.461 | +0.297% |
| option-suspended-later-arm | 1509.001 / 1510.001 / 1506.001 | 30652.712 / 30692.712 / 30524.712 | 1505968.183 / 1506771.080 / 1506032.751 | +0.004% |

StringReplace deltas use pre-544; all other deltas use pre-543. Fractional
allocation slopes arise from the finite N/2N panel, not fractional allocation
operations. Full samples and pair slopes:
[candidate22-summary.json](../../../.temp/brief-545/candidate22-summary.json).

The default-plus residual is reported without selecting the more favorable
pair: final slopes are 21,235.442 and 21,309.976 instructions, against pre-543
21,207.721 and 21,187.611. Their mean difference is +75.043 (+0.354%).
Earlier attempt21 panels measured +0.287% and +0.199%; before its explicit
slice-typing repair, the same attempt20 binary measured −0.094% and +0.213%.
There is observable variation between runs and pairs, but this final result
is a small positive residual, not proof of strictly zero added instructions.

The unarmed 128-option residual (+0.499%) remains above the smaller neighbor
variation. This implementation still publishes and retires a shared owner
for every delayed callback, checks unique slot access and used-length, checks
for a newly active limit, and checks canonical head identity when reusing a
head. Its allocation saving is 136 allocations and 18,512 bytes at 128
options. The measured net difference rises from -330.499 instructions at 32
options to +1,954.563 at 128 (about 23.8 extra instructions per additional
option over that interval). Attribution to those retained per-option
operations is an implementation explanation, not an isolated cycle-level
measurement or proof that another safe representation cannot improve it.

Untouched instruction neighbors range from approximately −0.131% to +0.092%
against pre-543 with unchanged allocation slopes. The four host resume
shapes are +0.144%, +0.369%, +0.297%, and +0.004% against pre-543 and have
fewer allocations. SameTest later-arm remains +1.114% against pre-543
(the entry-tree pin was +1.223%); its armed work remains out of optimization
scope. Armed 128 is 2,569,094.936 instructions versus entry-tree pin
2,564,147.587 (+0.193%), with four fewer allocations. The historical armed
overheads are reported in the table and were not the tuning objective.

All 120 core-language timing labels and 23 option/host timing labels were
also measured (seven repetitions, kernel AbsoluteTiming or the existing
host Instant harness). The core comparison against pre-544 has 45 positive
deltas and spans −49.302% to +5.810%; positive examples include
module-chain-escape-small +5.810% and literal-string +5.595%.
StringReplace timing is −11.714%, while the three delayed timings are
+7.086%/+3.933%/+3.508% against pre-543 despite their much smaller instruction
changes. Every label and timing delta is retained in
[the timing comparison](../../../.temp/brief-545/candidate22-timing-comparison.tsv).
These timing regressions and sign conflicts are reported, not used to choose
the implementation.

### Checker maintenance and verification

The local audited declaration set now includes the concrete flat buffer,
mode/domain proof, shared active view, and direct default builders. Payload
enum fields are compared exactly, including names, types, attributes and
generics; variant removal, type substitution, conditional fields and an
unconditional handoff admission have rejecting controls. The checker's own
regeneration tool produced all 68 fingerprints. Only field 19 changed from
entry; the first 18 fields, ordering, statuses and instance domains remain
unchanged. `promoted.tsv` stays empty.

Direct unarmed evaluator entry expands the historical unresolved frontier.
It now records 924 rejected sites (RET001 10, RET002 887, RET003 27), with
PUBLIC_WRAPPER still 16 and production coverage still zero. Exact production
constructor claims increase to 172, plus the two existing preserved
clone-return contexts. No unknown is admitted to make the count pass.

The expanded exact oracle initially exceeded its unchanged 700-second
bound. It now groups at most eight disjoint claims with distinct diagnostic
lines; each file in a group has one constructor spelling. Every claim still
requires the invariant positive type check, a located E0308 wrong-type
failure, and a live independent same-name negative with no error at the
claimed site. No claim is sampled or dropped. A disagreement makes the
harness fail after collecting all batches.

This exposed one real probe/context discrepancy at
`expr_runtime_helpers.rs:23`: freezing `Some(args)` removed the surrounding
return coercion and inferred `Option<&SharedArgs>` instead of
`Option<&[Expr]>`. The production helper now spells the slice borrow
explicitly as `Some(&args[..])`; the invariant probe was not weakened.
The eight-row batch was conservatively reported as disagreement; its only
underlying compiler-error site was that helper. The new explicit index adds
one located RET002 row, now independently asserted. The complete failing
compiler evidence is retained under
[oracle-evidence/coercion-discrepancy](../../../.temp/brief-545/oracle-evidence/coercion-discrepancy/).

Ownership inventory reconciliation retained unknown classifications and all
host-payload dispositions. Counts changed only for the new source shapes:
attributes 1414 unchanged, captures 2925→2929, macros 3061→3071, storage
82→84; host payloads remain 63 and self-reaching rows remain 60.
The [located reconciliation](../../../.temp/brief-545/ownership-reconciliation-review.json)
records the last macro-location update; the
[earlier reconciliation](../../../.temp/brief-545/ownership-reconciliation-review-before22.json)
records the entry-to-attempt21 additions/removals. No ownership coverage is added.

| Command (bounded wrapper and exact argv linked with raw output) | Result |
| --- | --- |
| `cargo fmt --check` ([raw](../../../.temp/brief-545/final22-format.raw), [argv](../../../.temp/brief-545/final22-format.command)) | exit 0 |
| `cargo test -p rum-runtime` ([raw](../../../.temp/brief-545/final22-runtime.raw), [argv](../../../.temp/brief-545/final22-runtime.command)) | 1091 passed; 0 failed; 3 existing ignored; exit 0 |
| `cargo test -p rum-stdlib options_suspension` ([raw](../../../.temp/brief-545/final22-options-suspension.raw), [argv](../../../.temp/brief-545/final22-options-suspension.command)) | 32 passed; 0 failed; 0 existing ignored; exit 0 |
| `cargo test -p rum-stdlib retention_closure -- --nocapture` ([raw](../../../.temp/brief-545/final22-checker-default.raw), [argv](../../../.temp/brief-545/final22-checker-default.command)) | 45 passed; 0 failed; 1 existing ignored; exit 0 |
| `cargo test -p rum-stdlib --no-default-features retention_closure` ([raw](../../../.temp/brief-545/final22-checker-no-default.raw), [argv](../../../.temp/brief-545/final22-checker-no-default.command)) | 45 passed; 0 failed; 1 existing ignored; exit 0 |
| `cargo test --workspace` ([raw](../../../.temp/brief-545/final22-workspace.raw), [argv](../../../.temp/brief-545/final22-workspace.command)) | 299 passed; 9 failed; 2 existing ignored; exit 101; nine native tests classified skipped-by-environment below |
| `cargo test --workspace --no-fail-fast` ([raw](../../../.temp/brief-545/final22-workspace-all.raw), [argv](../../../.temp/brief-545/final22-workspace-all.command)) | 2662 passed; 9 failed; 58 existing ignored; exit 101; nine native tests classified skipped-by-environment below |
| `cargo test -p rum-stdlib ownership_graph` ([raw](../../../.temp/brief-545/final22-ownership.raw), [argv](../../../.temp/brief-545/final22-ownership.command)) | 20 passed; 0 failed; 0 existing ignored; exit 0 |
| `cargo fmt --check (after all gates)` ([raw](../../../.temp/brief-545/final22-format-last.raw), [argv](../../../.temp/brief-545/final22-format-last.command)) | exit 0 |

Runtime totals include 1069 unit tests and 22 rustdocs. The three existing runtime ignores are unchanged; the checker’s one ignore is its fingerprint regeneration tool. The option suspension suite passes all 32 tests. Both checker configurations pass 45 tests and each exact oracle records 287 agreements, zero disagreements.

Verbatim acceptance rows, final22-no-default:

```text
C01	clean within reconciled domain	0 rejections
C01 M=rum_runtime::runtime::accounting_mode::counter::Unarmed R=rum_stdlib::option_consumers::OptionConsumer hash=false	clean	0 rejections
C01 M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow R=rum_stdlib::option_consumers::OptionConsumer hash=false	clean	0 rejections
C02	clean within reconciled domain	0 rejections
```

Verbatim acceptance rows, final22-default:

```text
C01	clean within reconciled domain	0 rejections
C01 M=rum_runtime::runtime::accounting_mode::counter::Unarmed R=rum_stdlib::option_consumers::OptionConsumer hash=true	clean	0 rejections
C01 M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow R=rum_stdlib::option_consumers::OptionConsumer hash=true	clean	0 rejections
C02	clean within reconciled domain	0 rejections
```

The nine native ZeroMQ tests are **skipped-by-environment for this report**: loopback bind returns `Operation not permitted`. The unchanged harness actually reports nine failures and Cargo exits 101, because the forbidden skip variable was not set. The mandatory workspace command was run exactly; its fail-fast exit was followed by `--no-fail-fast` to execute the remaining crates and rustdocs. No non-environment failure remains in that complete run.

```text
zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket
zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets
zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip
zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages
zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata
zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown
zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply
zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end
zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages
```

All 28 compiled mutation triples pass their controls, fail semantically with the protection removed (exit 101, no compiler error), and pass after restoration: 84 compiled test executions. Existing watched assertions are unchanged. The extra controls cover flat freeze allocation preflight, equal-generation/different-domain admission, canonical shared-head decline, and new option-bridge unwind cleanup.

```text
clone-handoff: 0 -> 101 -> 0
rule-literal: 0 -> 101 -> 0
shell-charge: 0 -> 101 -> 0
clone-before-growth: 0 -> 101 -> 0
callback-adoption: 0 -> 101 -> 0
error-payload: 0 -> 101 -> 0
panic-cleanup: 0 -> 101 -> 0
transfer-once: 0 -> 101 -> 0
frozen-complement: 0 -> 101 -> 0
abandon-reset-release: 0 -> 101 -> 0
overflow-stays-invalid: 0 -> 101 -> 0
late-arm-prefix: 0 -> 101 -> 0
late-arm-reselection: 0 -> 101 -> 0
uninitialized-suspension: 0 -> 101 -> 0
refusal-abandons: 0 -> 101 -> 0
shared-resume-adoption: 0 -> 101 -> 0
defaults: 0 -> 101 -> 0
capacity: 0 -> 101 -> 0
adoption: 0 -> 101 -> 0
registration: 0 -> 101 -> 0
resume-domain: 0 -> 101 -> 0
frame-complement: 0 -> 101 -> 0
working-copy: 0 -> 101 -> 0
flat-freeze-preflight: 0 -> 101 -> 0
handoff: 0 -> 101 -> 0
handoff-domain-identity: 0 -> 101 -> 0
canonical-shared-head: 0 -> 101 -> 0
option-bridge-unwind: 0 -> 101 -> 0
```

The first inherited replay stopped when its error-payload mutation hit the nonempty bridge branch bypassed by C02’s existing empty-owner path. That green omission and its raw output are preserved under `mutations-first-pass/`. The replay targets the active `adopt_outcome` error path; nonempty panic cleanup uses the existing nonempty bridge test. A premature retargeting of late-arm reselection to `evaluate_unarmed` also stayed green: that test deliberately exercises `CallbackValue::into_rule` directly. Its original omission was restored and checked; the mistargeted run is retained under `mutations-candidate21/late-arm-new-helper-miss/`. No test assertion was weakened to make a control red.

Verbatim mandatory gate summaries (full output remains linked above):

```text
$ cargo fmt --check
exit 0
$ cargo test -p rum-runtime
test result: ok. 1069 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.41s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.79s
exit 0
$ cargo test -p rum-stdlib options_suspension
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 1042 filtered out; finished in 1.84s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit 0
$ cargo test -p rum-stdlib retention_closure -- --nocapture
test result: ok. 45 passed; 0 failed; 1 ignored; 0 measured; 1028 filtered out; finished in 224.98s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit 0
$ cargo test -p rum-stdlib --no-default-features retention_closure
test result: ok. 45 passed; 0 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 232.08s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit 0
$ cargo test --workspace
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 38.89s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
exit 101
$ cargo test --workspace --no-fail-fast
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.05s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 1069 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.84s
test result: ok. 1021 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 55.78s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.35s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.34s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.16s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit 101
```

Earlier unsuccessful attempts are retained, including the frozen-Vec spare
capacity mismatch (fixed without changing its assertion), enum declaration
audit omissions, stale frontier counts, oracle timeout, and the slice-type
discrepancy. The
[checkpoint](../../../.temp/brief-545/PROGRESS.md)
records each of the 22 attempts, its instruction/allocation deltas, known
failing tests, compiler errors, and stop-loss status. Regressive inlining,
explicit normal bridge cleanup, moving the Rule builder outside the bridge,
and Arc::make_mut experiments were removed. The first allocator-harness
build selected an incompatible cached mimalloc artifact; selecting the
artifact already used by the existing CLI feature repaired it without any
dependency change.

### Changed paths and ranges

Ranges below refer to the final source. Exact zero-context diff hunks,
including removed locations, are in
[changed-paths-final.json](../../../.temp/brief-545/changed-paths-final.json).

| Changed path | Final line ranges | Why |
| --- | --- | --- |
| [crates/rum-runtime/src/runtime/accounting_mode/bridge.rs](../../../crates/rum-runtime/src/runtime/accounting_mode/bridge.rs) | 203–213 | Direct terminal unarmed Rule construction after the existing late-arm check. |
| [crates/rum-runtime/src/runtime/accounting_mode/lease.rs](../../../crates/rum-runtime/src/runtime/accounting_mode/lease.rs) | 129–139, 145–171, 182–199 | Mode-specific buffers, receipt byte access, and reuse of the validated immutable domain proof. |
| [crates/rum-runtime/src/runtime/accounting_mode/options.rs](../../../crates/rum-runtime/src/runtime/accounting_mode/options.rs) | 8–14, 60–65, 70–112, 120, 127–134, 148–157, 162, 172, 179–191, 197–201, 212–224 | Flat owner selection, scoped bridge cleanup, checked cold freeze and deferred armed admission. |
| [crates/rum-runtime/src/runtime/accounting_mode/options/buffer.rs](../../../crates/rum-runtime/src/runtime/accounting_mode/options/buffer.rs) | 1–201 | New private flat/growable buffer boundary, exact slots, safe unique insertion, and checked suspension materialization. |
| [crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs](../../../crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs) | 20–48, 115, 155–159 | Unarmed delayed specialization, inert clone path, canonical-head borrowing, domain handoff and resumed buffer selection. |
| [crates/rum-runtime/src/runtime/accounting_mode/retained.rs](../../../crates/rum-runtime/src/runtime/accounting_mode/retained.rs) | 40, 50–55, 61–65 | Flat shared activation view and reuse of existing occurrence arithmetic for slice capacity. |
| [crates/rum-runtime/src/runtime/accounting_mode/rule.rs](../../../crates/rum-runtime/src/runtime/accounting_mode/rule.rs) | 97, 106, 114–155 | Direct unarmed success path, exact canonical-head sharing, and cold failure/late-arm delegation. |
| [crates/rum-runtime/src/runtime/accounting_mode/tests/options.rs](../../../crates/rum-runtime/src/runtime/accounting_mode/tests/options.rs) | 54, 85, 164–195, 202–205, 211, 224–260 | Snapshot arguments and new exact late-arm/freeze-preflight and domain-identity collision controls. |
| [crates/rum-runtime/src/runtime/accounting_mode/tests/rule.rs](../../../crates/rum-runtime/src/runtime/accounting_mode/tests/rule.rs) | 27–48 | Canonical shared-head acceptance and RuleDelayed/context/compound/non-call decline control. |
| [crates/rum-runtime/src/runtime/expr_runtime_helpers.rs](../../../crates/rum-runtime/src/runtime/expr_runtime_helpers.rs) | 23 | Explicit slice borrow at the newly reached exact-type oracle site; runtime behavior unchanged. |
| [crates/rum-runtime/src/runtime/options.rs](../../../crates/rum-runtime/src/runtime/options.rs) | 19, 39, 133–167 | Borrowed stored-table and consuming canonical-default shared snapshot construction; inline length accessor. |
| [crates/rum-runtime/src/runtime/options/core.rs](../../../crates/rum-runtime/src/runtime/options/core.rs) | 98–99, 190 | Pass known explicit slot count into C01; small C02 dispatch inline hint. |
| [crates/rum-runtime/src/runtime/options/defaults.rs](../../../crates/rum-runtime/src/runtime/options/defaults.rs) | 16–21, 143–148 | Expose canonical declared entries for the unarmed snapshot while retaining normal normalization. |
| [crates/rum-runtime/src/runtime/options/reentry.rs](../../../crates/rum-runtime/src/runtime/options/reentry.rs) | 87, 94 | Inline existing generation/domain reads; full validation semantics unchanged. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv) | 15–16, 32 | Only three source-location updates; all existing disposition/witness columns preserved. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv) | 14–15, 31 | Reconcile the same 63 payload rows to moved source locations. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv) | 591–592 | Reconcile moved attribute locations; unchanged classification and count. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv) | 11–12, 26–31, 39–62, 70–74, 1153–1169, 1180–1182, 1189–1190 | Located closure additions/removals from specialization; all remain unknown. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv) | 17–19, 25–34, 1081–1095, 1118–1153 | Located macro additions/removals for buffer/default construction; all remain unknown. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv) | 20–22 | Record the two new opaque associated storage types; no coverage added. |
| [crates/rum-stdlib/src/tests/retention_closure/census/tests.rs](../../../crates/rum-stdlib/src/tests/retention_closure/census/tests.rs) | 161–165, 176, 185–194, 212–214 | Exact expanded frontier and moved protected sites; independently assert the newly explicit rejected index. |
| [crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.py](../../../crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.py) | 201–274, 286 | Batch every existing production invariant/negative probe within the unchanged bound; fail on any disagreement. |
| [crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.rs](../../../crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.rs) | 170–175 | Assert all 172 live contextual claims and both preserved clone-return fixtures. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv) | 2–69 | Checker-generated field-19 fingerprints only; first 18 fields and ordering unchanged. |
| [crates/rum-stdlib/src/tests/retention_closure/local/option_boundaries.rs](../../../crates/rum-stdlib/src/tests/retention_closure/local/option_boundaries.rs) | 9–33 | Add the concrete buffer/default/mode/shared-view declarations to the existing flat audit. |
| [crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs](../../../crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs) | 47–79, 84, 129 | New enum-field/conditional-field/domain-guard mutants and updated concrete snapshot signature. |
| [crates/rum-stdlib/src/tests/retention_closure/local/rules.rs](../../../crates/rum-stdlib/src/tests/retention_closure/local/rules.rs) | 142–179, 184–185 | Exact payload-enum schema checking with conditional fields rejected. |
| `docs/dev/codex/brief-545-erase-unarmed-option-cost.md` | 128–594 | Required execution report, measurements, disclosure and raw-output index. |

The new buffer module is under 6.4k characters. Rust files above the preferred 10k
size remain within the warning band; the expanded Python oracle is 16.9k.
Generated exact inventories are data tables. No unrelated source was
reverted. The final fence audit verifies unchanged protected roots,
manifests, rum-core/interner/gensym/snapshot/S-model files, runtime hashes
matching the measured candidate, unchanged contract meaning and host-ledger
dispositions, no added unsafe or ignore, and byte-identical probes.

### Commands and verbatim output

[COMMANDS.md](../../../.temp/brief-545/COMMANDS.md) indexes recorded build,
measurement, checker, mutation, inventory, and verification commands and
their unabridged outputs, including failed experiments. The exact argument
arrays and evidence paths are also in
[commands-index.json](../../../.temp/brief-545/commands-index.json).
Per-sample perf and allocation outputs are preserved separately; no
environmental or negative-control failure is relabeled in the raw evidence.
Compiler evidence is archived before the feature configuration changes.

All scratch is under `.temp/brief-545/`. Existing checker paths that write
under `target/` or earlier brief scratch directories run through a read-only
root/bwrap overlay whose writable upper/work directories are under this
brief's scratch. Each long or child-spawning command is bounded through
`scripts/run-bounded.py`. The ZeroMQ skip variable is never set.
