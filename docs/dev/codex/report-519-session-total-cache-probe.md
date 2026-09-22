# Brief 519 — session-total cache design probe

## 1. Verdict: REFUTE

The favourable retained instruction counts have an explanation: **adding the
counter changed compiler inlining**. The base calls a separate
`evaluate_inner_unstable`; the counter build inlines that function into its
caller. Fresh copies of the current source reproduce both old evaluator
bodies byte-for-byte. The reductions are real aggregate instruction counts,
but they do not show that maintaining the counter is free.

Holding the baseline's inlining choice constant, and then also controlling
for the added field's layout, exposes the cost of just the mandatory
evaluation-stack mutation signal: **+0.182073% smoke, +0.470698% ownvalue
reads, +0.134833% pattern hits**. All three workloads scale with N. The
brief's zero-unarmed-cost bar rejects this measured signal.

Existing counters cannot replace it: **42 of the 52 walked fields have no
covering existing counter**, eight have coverage before counter saturation,
and two have invariant proxy contributions. The existing-counter tuple
actually reuses a stale low total in the reproduction. The more extensive
retained observer's hit rates are useful measurements, but its signal still
covers only a subset of mutations.

The authoritative occurrence walker is unchanged. No production source was
edited or timing panel run. This refutes the measured design under the
standing constraints, without proving every future implementation impossible.

## 2. Field-versus-counter cross

The retained enumeration matches the schema's **52 `Accounted` fields**,
and all **935** field-reference lines match current source. The audit covers
mutation helpers/callers, host setters, rollback, lazy indexes and suspension.
The machine-readable cross is
[field-cross.tsv](../../../target/session-total-cache-519-resume/field-cross.tsv);
the raw cross-reference is
[field-references.tsv](../../../target/session-total-cache-519/field-references.tsv).

`E` = `evaluation_epoch`; `A` = `attribute_revision`;
`S` = `symbol_cache_generation`; `C` = `call_stability_generation`.
The eight-component key examined in the observer is `(E, S, C, A,
next_generated_symbol_id, next_sealed_symbol_id, next_source_unit_epoch,
next_request_generation)`.

**Covered** means every proxy-changing live mutation has the stated existing
counter change before the next observable checkpoint. Initialization starts
with an invalid cache. This classification requires fail-closed handling of
counter saturation: E/A/S/C use `saturating_add`, so equality at `u64::MAX`
cannot certify freshness. **None** names an uncovered field, even when some
of its mutation paths happen to bump a counter. **Constant** means its
current formula does not change when the field's value changes.

Source links point into `crates/rum-runtime/src/runtime/`. Each None row
identifies a missing path. All eight positive rows also cover loader
restoration through `bump_attribute_revision`, hence E/S/C.

| Walked field | Covering counter | Source witness and coverage result |
| --- | --- | --- |
| `abandonment_worklist` | **None** | [core.rs:544](../../../crates/rum-runtime/src/runtime/suspension/core.rs#L544) — Abandonment pushes, pops and cleanup requeues have no revision; frames can change while owned by the worklist. |
| `association_lookup_cache` | **None** | [association_data.rs:447](../../../crates/rum-runtime/src/runtime/kernel_data/association_data.rs#L447) — Index insertion/eviction and lvalue detach/reattach change charged owners without a session epoch bump. |
| `attribute_set_cache` | **None** | [resolution.rs:190](../../../crates/rum-runtime/src/runtime/resolution.rs#L190) — Lookup inserts on a miss without A/E/S/C; A covers invalidation only. |
| `attributes` | A and E | [resolution.rs:214](../../../crates/rum-runtime/src/runtime/resolution.rs#L214) — Mutation helpers, clear/block/iterator restoration and loader rollback call bump_attribute_revision; it also bumps E/S/C. |
| `builtin_dispatch_spec_cache` | **None** | [resolution.rs:95](../../../crates/rum-runtime/src/runtime/resolution.rs#L95) — Lookup inserts on a miss without a revision; registration clears it with E but that is not every change. |
| `builtins` | E | [session_api.rs:271](../../../crates/rum-runtime/src/runtime/session/session_api.rs#L271) — The only live writer is register_builtin; both attribute branches bump E, with S/C through full invalidation. |
| `call_stability_cache` | **None** | [stability.rs:343](../../../crates/rum-runtime/src/runtime/evaluation_cache/stability.rs#L343) — Lazy insertion, mutation-proof insertion/removal and size-cap clearing can change storage without E/S/C advancing. |
| `context_path` | **None** | [session_api.rs:103](../../../crates/rum-runtime/src/runtime/session/session_api.rs#L103) — apply_config and Begin/End replace owned strings without a revision; some package/assignment routes do bump E. |
| `context_stack` | **None** | [session_api.rs:761](../../../crates/rum-runtime/src/runtime/session/session_api.rs#L761) — Begin/End push/pop frames without an epoch, including repeated package entries. |
| `current_context` | **None** | [session_api.rs:102](../../../crates/rum-runtime/src/runtime/session/session_api.rs#L102) — apply_config and Begin/End replace the String without a revision. |
| `declared_stubs` | E | [package_tools.rs:68](../../../crates/rum-runtime/src/runtime/package_tools.rs#L68) — DeclarePackage bumps E when a stub is inserted; Remove bumps E when one is removed; loader restore bumps A/E. |
| `definitions` | **None** | [down_value_dispatch.rs:248](../../../crates/rum-runtime/src/runtime/model/definitions/down_value_dispatch.rs#L248) — Ordinary writes bump E, but a first indexed read populates a charged OnceLock through &self with no session revision; Unique also uses the separate generated-symbol ID. |
| `disabled_message_selectors` | **None** | [message_switch.rs:55](../../../crates/rum-runtime/src/runtime/debug_tools/message_switch.rs#L55) — Off/On edit selectors without advancing an epoch. |
| `dynamic_own_values` | **None** | [symbol.rs:166](../../../crates/rum-runtime/src/runtime/eval_core/symbol.rs#L166) — Dynamic-binding push/truncate and loop/lvalue replacement bypass definition epochs. |
| `entropy_source` | **None** | [session_api.rs:584](../../../crates/rum-runtime/src/runtime/session/session_api.rs#L584) — Host setter replaces the optional concrete Arc object without a revision; its internal allocations are outside the present proxy. |
| `evaluation_stack` | **None** | [mod.rs:46](../../../crates/rum-runtime/src/runtime/eval_core/mod.rs#L46) — Cycle-tracked push/pop, rollback and suspension replacement have no covering counter; depth is reversible and cannot be a generation. |
| `exception_types` | **None** | [exceptions.rs:79](../../../crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs#L79) — RegisterExceptionType inserts a String/parent entry without any revision. |
| `file_io` | **None** | [session_api.rs:572](../../../crates/rum-runtime/src/runtime/session/session_api.rs#L572) — Host setter changes optional provider/object size without a revision. |
| `gc_candidate_gensyms` | **None** | [module.rs:336](../../../crates/rum-runtime/src/runtime/scoping/lexical/module.rs#L336) — Escape registration and advisory removal/sweep alter the set without a guaranteed epoch; the watermark is not a monotone version. |
| `handled_non_loader_eval_errors` | **None** | [session_helpers.rs:412](../../../crates/rum-runtime/src/runtime/session/session_helpers.rs#L412) — Handled errors append messages independently of definition changes. |
| `installed_system_down_values` | E | [session_api.rs:300](../../../crates/rum-runtime/src/runtime/session/session_api.rs#L300) — Install goes through store_trusted_system_down_value and its E bump before recording the marker; loader restore bumps A/E; no intervening checkpoint. |
| `installed_system_own_values` | E | [session_api.rs:292](../../../crates/rum-runtime/src/runtime/session/session_api.rs#L292) — Install and marker-removing assignment paths bump E; loader restoration bumps A/E. |
| `last_message_list` | **None** | [session_eval.rs:256](../../../crates/rum-runtime/src/runtime/session/session_eval.rs#L256) — Evaluation/request completion and $MessageList observation replace the message vector without a revision. |
| `load_stack` | **None** | [session_eval.rs:377](../../../crates/rum-runtime/src/runtime/session/session_eval.rs#L377) — Loader push/pop and rollback replace path owners without an epoch. |
| `loaded_packages` | E | [session_api.rs:719](../../../crates/rum-runtime/src/runtime/session/session_api.rs#L719) — mark_package_loaded, $Packages assignment, failed dependency truncation and loader restoration bump E. |
| `message_definitions` | **None** | [message_catalog.rs:87](../../../crates/rum-runtime/src/runtime/messages/message_catalog.rs#L87) — define_message and Unset edit templates independently of evaluation epochs. |
| `message_symbol_counts` | **None** | [message_catalog.rs:89](../../../crates/rum-runtime/src/runtime/messages/message_catalog.rs#L89) — Template insertion, discount and rebuild change counted hash storage without a revision. |
| `messages` | **None** | [session_helpers.rs:294](../../../crates/rum-runtime/src/runtime/session/session_helpers.rs#L294) — Message emission and take/split/drain change strings/capacity without a revision. |
| `module_activations` | **None** | [module.rs:60](../../../crates/rum-runtime/src/runtime/scoping/lexical/module.rs#L60) — Push/truncate/rollback change owned gensym vectors without a guaranteed counter; an empty Module allocates no generated ID. |
| `monitor_scopes` | **None** | [monitor.rs:43](../../../crates/rum-runtime/src/runtime/debug_tools/monitor.rs#L43) — Push/pop and display updates have no covering epoch. |
| `once_values` | **None** | [package_tools.rs:141](../../../crates/rum-runtime/src/runtime/package_tools.rs#L141) — Once caches a result (including resume) without an epoch. |
| `option_value_scopes` | **None** | [patterns.rs:20](../../../crates/rum-runtime/src/runtime/options/patterns.rs#L20) — Scope push/pop and continuation restoration do not bump epochs. |
| `options` | **None** | [core.rs:35](../../../crates/rum-runtime/src/runtime/options/core.rs#L35) — set_options_for_symbol inserts/removes without an epoch; note_active_modules_escaped is not a version. |
| `package_search_paths` | **None** | [session_api.rs:104](../../../crates/rum-runtime/src/runtime/session/session_api.rs#L104) — Configuration/path setters replace PathBuf vectors without a covering counter. |
| `pending_eval_request` | **None** | [session_requests.rs:234](../../../crates/rum-runtime/src/runtime/session/session_requests.rs#L234) — Occupancy changes on suspension/completion inside the same request generation. |
| `pending_input_request` | **None** | [core.rs:644](../../../crates/rum-runtime/src/runtime/suspension/core.rs#L644) — Prompt install/take/queue response changes owned prompt bytes inside one request generation. |
| `pending_suspension` | **None** | [core.rs:432](../../../crates/rum-runtime/src/runtime/suspension/core.rs#L432) — SuspensionAccess installs/takes/mutates the owner and frame payloads without a session memory revision. |
| `print_sink` | **None** | [session.rs:141](../../../crates/rum-runtime/src/runtime/model/session.rs#L141) — Host set/clear changes optional concrete boxed size without an epoch; captured heap internals are not counted by this proxy. |
| `prints` | **None** | [session_helpers.rs:306](../../../crates/rum-runtime/src/runtime/session/session_helpers.rs#L306) — Buffer push and take/split/truncate change storage without an epoch. |
| `queued_input_responses` | **None** | [session_api.rs:266](../../../crates/rum-runtime/src/runtime/session/session_api.rs#L266) — Host enqueue and guest dequeue change strings and queue capacity without a generation bump. |
| `reap_scopes` | **None** | [reap.rs:47](../../../crates/rum-runtime/src/runtime/debug_tools/reap.rs#L47) — Scope push/pop and Sow bucket append mutate expression roots without a revision. |
| `removed_symbols` | E | [definition_clear.rs:212](../../../crates/rum-runtime/src/runtime/definitions/definition_clear.rs#L212) — Each new removal marker bumps E; loader rollback also bumps A/E. |
| `request_interrupt_control` | Constant charge | [schema.rs:166](../../../crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/schema.rs#L166) — Always charges the same Arc header plus concrete RequestInterruptState size, including Arc rotation; token/state changes do not change this field's proxy bytes. |
| `resource_limits` | Constant charge | [schema.rs:168](../../../crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/schema.rs#L168) — The deliberately conservative size_of_val term is constant; changing any configured value leaves its charge unchanged. |
| `resuming_suspension` | **None** | [core.rs:492](../../../crates/rum-runtime/src/runtime/suspension/core.rs#L492) — Resume installs/takes the saved owner; continuation work changes its charged payload without a request generation bump. |
| `sealed_units` | **None** | [packages.rs:500](../../../crates/rum-runtime/src/runtime/scoping/packages.rs#L500) — Install increments next_source_unit_epoch, but rollback removes an existing unit without that increment; cleanup of an empty package need not bump E. |
| `stable_symbol_cache` | **None** | [stability.rs:367](../../../crates/rum-runtime/src/runtime/eval_core/stability.rs#L367) — Lazy symbol-cache insertion has no epoch; S covers clear, not fill. |
| `symbol_call_effect_cache` | **None** | [stability.rs:515](../../../crates/rum-runtime/src/runtime/eval_core/stability.rs#L515) — Lazy effect-cache insertion has no epoch; S covers clear, not fill. |
| `symbol_generations` | E | [definition_clear.rs:225](../../../crates/rum-runtime/src/runtime/definitions/definition_clear.rs#L225) — New context/name entries accompany a new removed-symbol marker and E; subsequent u64-only value changes do not change the proxy; loader rollback bumps A/E. |
| `syntax_information` | **None** | [syntax.rs:37](../../../crates/rum-runtime/src/runtime/namespace_tools/syntax.rs#L37) — Setter and Unset replace expression roots without an epoch. |
| `trace_event_depths` | **None** | [mod.rs:95](../../../crates/rum-runtime/src/runtime/definitions/application/mod.rs#L95) — Trace push can grow capacity without an epoch; clear/take paths likewise lack coverage. |
| `trace_events` | **None** | [mod.rs:94](../../../crates/rum-runtime/src/runtime/definitions/application/mod.rs#L94) — Trace push/take/truncate change expression/string owners independently of definition epochs. |

S and C invalidate evaluation proofs; cache fill/removal is not guaranteed to
advance them. Generated-symbol IDs observe allocation of identities, not
scope exit or advisory GC-set changes. `next_source_unit_epoch` observes a
successful installation, not all rollback removal. Request generations stay
constant across suspension/resumption within one request. Depths, poll
ticks, GC watermarks, side-table length stamps and optional diagnostics do
not supply a missing monotone memory version. In particular, equal stack
depth or collection length permits different payloads and capacities.

In particular, a read initializes `definitions`' charged dispatch `OnceLock`
through `&self`. Covering `Set` and `Clear` leaves this mutation uncovered.

There is also direct measured failure. These are columns 1–7 of the first
two retained reproduction observer records, followed by the four changed
field charges extracted from those same records:

```text
RAW_PREFIX_COLUMNS_1_TO_7
B519	expr	671,661,661,657,0,0,0,2	1361	71694	0	0
B519	expr	671,661,661,657,0,0,0,2	1365	71738	0	71694
attribute_set_cache	320	331
builtin_dispatch_spec_cache	712	723
stable_symbol_cache	387	398
symbol_call_effect_cache	432	443
```

The eight existing counters are identical, while the walker rises from
71,694 to 71,738 bytes. The individual cache-field changes sum to 44 bytes.
An existing-counter cache would hit with a low value here. Over the entire
reproduction it would claim 91 hits, **all 91 stale low**. The throwaway
revision does change between these records, which is precisely why the
existing counters cannot substitute for it.

## 3. Unarmed signal cost, scaling, and provenance

### What was verified and retained

The starting production tree was clean at
`5114bfca` (`docs: brief-519 resume note after an upstream capacity failure`).
Comparing the interrupted run's 1,507-file SHA-256 manifest with the current
tree finds only the expected change to the brief itself. Production sources,
Cargo manifests/lockfile and toolchain declaration match. The saved workspace
has 15 differences from production; inspection identifies them as the final
throwaway observer, not a baseline source snapshot.

Every retained instruction record was matched to its actual perf CSV,
stdout/stderr and preserved executable hash. All 112 records passed these
checks: four calibration records and 27 records in each of base, counter,
signal and candidate. Each command names the **same original executable
path**, `target/session-total-cache-519/build/release/rum-cli`. The recorded
counter hash matches `rum-stack-counter`; the base hash matches `rum-base`.
The medians and full slopes in `instruction-comparison.json` recompute from
the raw samples. Verbatim audit output:

```text
INITIAL_MANIFEST_ENTRIES 1507
INITIAL_MANIFEST_CHANGED ["docs/dev/codex/brief-519-session-total-cache-probe.md"]
WALKED_FIELDS_MATCH 52
FIELD_REFERENCE_LINES_MATCH 935
RAW_INSTRUCTION_RECORDS_VERIFIED 112
RETAINED_ARTIFACTS_HASHED 1436
RETAINED_WORKSPACE_DIFFERS_FROM_PRODUCTION_IN 15
OLD_MEDIANS_AND_COMPARISON_MATCH True
family	base_slope	counter_slope	percent
smoke	27458.805120	27349.861627	-0.396752
symbol-ownvalue-read	2124.942414	2110.943695	-0.658781
downvalue-pattern-hit	14568.603676	14530.383352	-0.262347
```

The fresh base copies unchanged production source. The reconstructed stack
arm adds a u64, initialization, tracked push/pop increments and a cold armed
read, reproducing the old hot function exactly. The rebuilt observer's
evaluator and `brief519_observe` also match. Per-build source hashes and
diffs are preserved. Rust is **1.85.0, LLVM 19.1.7**, release profile, x86-64.

Verbatim [assembly audit](../../../target/session-total-cache-519-resume/assembly-audit.log):

```text
EXACT_FUNCTION_BYTES old-base base evaluate_once_unstable_after_input_check_with_cycle_tracking True
EXACT_FUNCTION_BYTES old-base base evaluate_inner_unstable True
EXACT_FUNCTION_BYTES base fixed-base evaluate_once_unstable_after_input_check_with_cycle_tracking True
EXACT_FUNCTION_BYTES base fixed-base evaluate_inner_unstable True
EXACT_FUNCTION_BYTES old-stack stack evaluate_once_unstable_after_input_check_with_cycle_tracking True
EXACT_FUNCTION_BYTES old-observer observer evaluate_once_unstable_after_input_check_with_cycle_tracking True
EXACT_FUNCTION_BYTES old-observer observer brief519_observe True
```

| Build | Outer evaluator bytes | Separate inner evaluator bytes |
| --- | ---: | ---: |
| Old base / fresh base | 1,854 | 9,971 |
| Old stack / reconstructed stack | 11,805 | absent: inlined |
| Inlining-controlled base | 1,854 | 9,971 |
| Inlining-controlled field-only control | 1,854 | 9,971 |
| Inlining-controlled stack signal | 1,910 | 9,971 |

The base's actual call instruction is:

```asm
  4827df:	e8 0c 05 00 00       	call   482cf0 <rum_runtime::runtime::eval_core::<impl rum_runtime::runtime::model::session::Session>::evaluate_inner_unstable>
```

This identifies a compiler inlining change. The unrelated main
`target/release/rum-cli` has a different cold diagnostic-string relocation;
whole-file hashes across build directories differ. The fresh copied-workspace
function comparisons above are byte-exact. Disassembly and addresses are in
[assembly/](../../../target/session-total-cache-519-resume/assembly/).

The old negative counts remain valid for those binaries; **their
interpretation as a free counter is discarded**. The old signal/candidate
arms also have incomplete mutation coverage. Their numbers are retained,
without accepting them as correct caches. New measurements isolate this
confound; section 4 repeats only the failed mixed-program hit test.

### Controlled measurement

All three controlled arms add `#[inline(never)]` to
`evaluate_inner_unstable` in the disposable copy. The base already compiled
that way: both of its evaluator bodies remain byte-identical. This prevents
the counter from obtaining an unrelated saving by eliminating that call.

The controls are:

1. `fixed-base`: unchanged baseline plus the common inlining constraint.
2. `fixed-layout`: the same constraint, added u64, initializer and cold read,
   with no increments. This controls for the field's layout and observability.
3. `fixed-stack`: identical to `fixed-layout`, with a saturating increment at
   each tracked evaluation-stack push and pop.

The cold armed read prevents dead-code elimination; push/pop increments also
execute unarmed. No mode route, global, thread-local or pointer cache is
introduced. Saturation invalidates freshness in the hit observer.

This **minimum representative signal** omits suspension stack replacement,
other owners and cache-total bookkeeping. It isolates a required cost with
the baseline's call boundary fixed; the original freely-inlined binary's
aggregate counts did not regress.

All new arms were built sequentially in
`target/session-total-cache-519-resume/workspace` into the same
`target/session-total-cache-519-resume/build/release/rum-cli` path and measured
at that path. Cargo was offline and locked, `TMPDIR` stayed in the repo,
`RUM_*` variables were removed, and builds did not overlap measurements.
The copied sources are now removed; their diffs, hashes and binaries remain
as labelled forensic artifacts.

The executed per-sample recipe was:

```text
scripts/run-bounded.py --timeout 15 -- taskset -c 2 perf stat -e instructions:u -x, -o <sample>.perf.csv -- <same-build-path>/rum-cli <original-workload>.rum
```

The original calibration observed a maximum 0.405348 seconds across baseline
samples. New samples took at most 0.341590 seconds. Each 27-sample driver was
bounded at 60 seconds, each release build at 120 seconds with an encompassing
150-second bound; prior identical-shape builds took 23–34 seconds. Elapsed
time was used solely to size bounds. No wall-time performance comparison or
timing panel was run.

The workloads are the original assertion-bearing `.rum` files. Smoke uses
12,500 / 25,000 / 50,000 iterations. Ownvalue reads use 10,000 / 20,000 /
40,000 iterations per inner `Do`, repeated seven times; pattern hits use
5,000 / 10,000 / 20,000 per inner `Do`, also repeated seven times. Their
aggregate N values below are 70,000 / 140,000 / 280,000 and 35,000 / 70,000 /
140,000 respectively. **No single Do exceeds 50,000.** All runs completed
the exact result assertions; every PMU core reading has 100.00 coverage,
with no abort, termination result or stderr.

Verbatim medians, within-arm spreads and adjacent/full-slope analysis:

```text
family	arm	N	median_instructions	spread_percent
smoke	fixed-base	12500	346774880	0.000325283
smoke	fixed-base	25000	690010963	0.000493905
smoke	fixed-base	50000	1376481728	0.000139123
smoke	fixed-layout	12500	346776029	0.001623815
smoke	fixed-layout	25000	690011462	0.000245793
smoke	fixed-layout	50000	1376483279	0.000268801
smoke	fixed-stack	12500	347399450	0.000387450
smoke	fixed-stack	25000	691260382	0.000397101
smoke	fixed-stack	50000	1378981523	0.000155695
symbol-ownvalue-read	fixed-base	70000	153206219	0.003146739
symbol-ownvalue-read	fixed-base	140000	301955859	0.001441933
symbol-ownvalue-read	fixed-base	280000	599437243	0.000180169
symbol-ownvalue-read	fixed-layout	70000	153201382	0.002513685
symbol-ownvalue-read	fixed-layout	140000	301954362	0.000436490
symbol-ownvalue-read	fixed-layout	280000	599438546	0.000637930
symbol-ownvalue-read	fixed-stack	70000	153903260	0.001681576
symbol-ownvalue-read	fixed-stack	140000	303357695	0.001088154
symbol-ownvalue-read	fixed-stack	280000	602240853	0.000507272
downvalue-pattern-hit	fixed-base	35000	514391609	0.002527452
downvalue-pattern-hit	fixed-base	70000	1024268132	0.006122811
downvalue-pattern-hit	fixed-base	140000	2044030001	0.005979902
downvalue-pattern-hit	fixed-layout	35000	514404585	0.000481139
downvalue-pattern-hit	fixed-layout	70000	1024268349	0.001485939
downvalue-pattern-hit	fixed-layout	140000	2044107373	0.001488278
downvalue-pattern-hit	fixed-stack	35000	515098287	0.002179002
downvalue-pattern-hit	fixed-stack	70000	1025647665	0.001638282
downvalue-pattern-hit	fixed-stack	140000	2046863612	0.004217575
family	base_slope_lo	base_slope_hi	layout_slope_lo	layout_slope_hi	stack_slope_lo	stack_slope_hi	added_per_N_vs_layout	percent_vs_base	percent_vs_layout
smoke	27458.886640	27458.830600	27458.834640	27458.872680	27508.874560	27508.845640	49.995280	0.182113	0.182073
symbol-ownvalue-read	2124.994857	2124.867029	2125.042571	2124.887029	2135.063357	2134.879700	10.002043	0.472080	0.470698
downvalue-pattern-hit	14567.900657	14568.026700	14567.536114	14569.128914	14587.125086	14588.799243	19.643210	0.139048	0.134833
VALID_SAMPLES 81
EXIT_CODES [0]
COVERAGE ['100.00']
MAX_SAMPLE_SECONDS 0.3415899600004195
```

Both adjacent N intervals give nearly identical per-unit slopes; counts
scale with work. For the signal versus the field-only control, full-interval
differences are approximately
**50 / 10 / 20 instructions per unit of work**, respectively. The emitted
saturating increment is visible on the unarmed route:

```asm
  482766:	49 8b 87 50 06 00 00 	mov    0x650(%r15),%rax
  48276d:	48 ff c0             	inc    %rax
  482770:	48 c7 c1 ff ff ff ff 	mov    $0xffffffffffffffff,%rcx
  482777:	48 0f 45 c8          	cmovne %rax,%rcx
  48277b:	49 89 8f 50 06 00 00 	mov    %rcx,0x650(%r15)
```

There are corresponding increments on push and pop. The positive slopes are
well outside the within-arm instruction spreads. Against `fixed-base`, the
full-slope regressions are +0.182113%, +0.472080% and +0.139048%; against the
identical-field control they are the verdict's +0.182073%, +0.470698% and
+0.134833%. None is waived as small. Adding the required signal incurs a
default-path cost that the original compiler inlining change had masked.

The [driver](../../../target/session-total-cache-519-resume/measure_fixed.py),
[raw records](../../../target/session-total-cache-519-resume/fixed-stack-instructions.json),
[comparison](../../../target/session-total-cache-519-resume/fixed-comparison.json)
and [exact patches](../../../target/session-total-cache-519-resume/patches/)
make the controls reviewable. No complete caching implementation was built
or benchmarked.

## 4. Measured armed hit rates

The retained observer runs **both authoritative traversals** at each armed
checkpoint and asserts equality: `current_memory_proxy_bytes()` and the
independent brief-517 `session_breakdown`. It always returns the walker's
total to enforcement. It logs all 52 field charges, the eight existing
versions and the partial added revision. It never makes a limit decision
from cached data. The observer's own inline storage is included through
`size_of::<Session>()`; the log totals minus all 52 charges equal 1,952
bytes. No occurrence deduplication or accounting-formula change was made.

The partial revision tracks ordinary evaluation-stack push/pop, E bumps,
dynamic-binding push/truncate, five evaluation/dispatch cache fills and
selected call-cache removes/clears. Its source is preserved in
[observer.patch](../../../target/session-total-cache-519-resume/patches/observer.patch).
It does **not** claim the missing paths in section 2. Its recorded hits are
an upper bound for a completed version of that same mutation-invalidation
scheme. No stale partial-signal hit was observed in these workloads; this
is an observation, not proof of general correctness.

For an independent ceiling on the opportunity, I also compared the complete
52-field charge vector at successive checkpoints. Its equality coincides
with exact total equality in these four runs. This measured oracle requires
a walk and therefore supplies no cheap implementation; equal totals also
do not prove that no intervening mutation occurred.

The original mixed workload failed its assertion in **both** control and
observer (exit **2**). Its verbatim stdout was:

```text
{20100 * Switch["A", "A", 125, "B", 250, 100], 10}
```

`Switch` remained symbolic. That run is discarded as a successful realistic
program. The only source correction was to use supported nested `If` with
`SameQ` in `gRate`; the expected `{2512500, 10}` was preserved. It still
prices 200 invoice lines with `Map`, `Part` and a pattern definition, sums
them inside `Module`, writes `gTotal`/`gAudit`, and creates an audit batch
with `Reap`, `Do`, `If` and `Sow` under a 1,000,000,000-byte ceiling.
The repaired source is
[mixed-fixed.rum](../../../target/session-total-cache-519-resume/mixed-fixed.rum).
Both new runs exit zero with identical stdout:

```text
PASS: mixed invoice total and audit
Null
```

Their bounds were 10 seconds each, based on the old same-shape control and
observer runs (0.393 and 1.337 seconds); the enclosing driver was bounded at
30 seconds. The three successful old control/observer pairs were reused.
The rebuilt observer matches the original observer's machine code as
shown in section 3.

Verbatim [hit summary](../../../target/session-total-cache-519-resume/hit-summary.tsv),
including the first compulsory cache miss in each denominator:

```text
workload	checkpoints	expr	projected	partial_hits	partial_misses	partial_hit_percent	oracle_equal	oracle_equal_percent	old_key_stale_low
reproduction	92	92	0	75	17	81.521739	87	94.565217	91
mutating-128	2308	2308	0	1153	1155	49.956672	1664	72.097054	1795
mutating-512	9220	9220	0	4609	4611	49.989154	6656	72.190889	7171
mixed-fixed	5561	5561	0	4188	1373	75.310196	4445	79.931667	5542
```

* **Reproduction:** `Nest[StringJoin[#, #] &, gSeed, 16]`, 1,000-character
  seed, 8,000,000-byte ceiling; the asserted outcome is `$Aborted`. Counts
  cover execution through that refusal, not sixteen completed expansions.
  Cache warm-up changes five walked fields even though the user holds only
  the seed/output symbols. The partial signal hits 81.52%; equality of the
  authoritative field charges reaches 94.57%.
* **Mutating:** `gAcc = gAcc + i*i` on every iteration under a ceiling, at
  N=128 and N=512, with exact sum assertions. The partial signal hits about
  50%, and actual charge-vector equality about 72%. Four times the loop
  work gives 2,308 → 9,220 checkpoints, so the rate is not dominated by
  initialization.
* **Mixed:** the repaired invoice/audit program gives 75.31% partial-signal
  hits and 79.93% charge-vector equality. Unchanged existing versions
  coexist with changing `module_activations`, `gc_candidate_gensyms` and
  `reap_scopes`, as well as the ordinary caches/stacks.

All **17,181** observed checkpoints in these runs were expression checks;
the observer also instruments projected-byte checks, but these workloads
produced none. No coverage of that unexecuted checkpoint class, host setter
edges, exception registration or suspension is inferred from these rates.
Full counts, stale-low/high checks and witnesses are in
[hit-analysis.json](../../../target/session-total-cache-519-resume/hit-analysis.json).

Reuse exists, especially after cache warm-up. Hit percentages establish
neither free invalidation nor removal of the documented 43x armed cost.
No armed speedup measurement is claimed.

## 5. Refutation and implications

The whole-session key cannot be built from the existing counters safely.
An observed unchanged eight-counter tuple with a larger authoritative total
is the falsifier, already witnessed failing. One missing owner would have
been sufficient; the source audit finds 42.

The representative new signal is also rejected: even the ordinary stack
mutation subset costs the unarmed path when compiler inlining and field
layout are controlled. Filling in the remaining owners is neither a proof
that this cost disappears nor work authorized as production development by
this brief. The original negative aggregate numbers cannot certify erasure.

Accordingly, **do not implement this session-total cache or draft a cutover
brief from this evidence**. Together with
[brief-518's mode-route refutation](./report-518-accounting-mode-route.md),
this leaves the unstrided occurrence walker and its documented armed cost
as the accepted outcome of this design line. The 43x characterization comes
from the preceding briefs; it was not remeasured here. The public resource
contract, checkpoint frequency and independent oracle stay unchanged.

## 6. Proof of removal and unchanged production tree

**The production tree is clean of throwaway instrumentation.** It was never
edited during this dispatch. The newly created disposable source workspace
was deleted after measurements. Patches, binaries and logs are retained only
as ignored probe evidence under `target/session-total-cache-519-resume/`.
The pre-existing interrupted-run evidence, including its saved instrumented
workspace, was preserved byte-for-byte as requested; it is not active
production source. No existing `target/brief-5*/` or
`target/accounting-route-518/` artifact was overwritten.

Verbatim final integrity check:

```text
start-sha256.json CHECKED 1507 CHANGED []
retained-evidence-sha256.json CHECKED 1436 CHANGED []
RETAINED_OBSERVER_BINARY_UNCHANGED True
MATRIX_EXACTLY_COVERS_WALKED_FIELDS 52
NEW_INSTRUCTION_RECORDS_AND_HASHES_VERIFIED 81
REPAIRED_MIXED_CONTROL_AND_OBSERVER_PASS True
OBSERVED_CHECKPOINTS 17181
NEW_DISPOSABLE_WORKSPACE_ABSENT True
PRODUCTION_INSTRUMENTATION_MARKERS []
PRODUCTION_SOURCE_EDITS 0
NEW_UNSAFE 0
```

No commit, branch, switch, merge, push, stash, checkout, restore, git-config
or index operation was performed. Cargo ran offline; all new files and
temporary build output stayed in this repository. No new unsafe code or
third-party-engine comparison was introduced.

With no runtime change, verification used hashes, schema/table completeness,
exact machine code, 81 assertion-bearing instruction runs and the repaired
mixed-program pair. The workspace suite and timing panel were not run. The
only repository deliverable is this report.

Verbatim final repository status (optional git locks disabled):

```text
?? docs/dev/codex/report-519-session-total-cache-probe.md
```
