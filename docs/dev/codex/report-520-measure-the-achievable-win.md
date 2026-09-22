**1. Three-way comparison and fraction removed**

The measured ceiling supports continuing the maintained-total line at the stated approximately 0.7% unarmed price. Removing the session walk eliminates 99.79%–99.99% of the armed overhead in these nine shapes, measured against fresh unarmed controls. This is an upper bound on the benefit, before charge/release bookkeeping.

The stub is **deliberately wrong: it does not enforce anything correctly**. Its only source change replaced the body of `current_memory_proxy_bytes()` with `self.peak_memory_proxy_bytes.get()`. That existing `Cell<usize>` starts at zero and the stub never maintains it. No field or layout was added. The expression term, constraint code, and all other source remained unchanged. The stub has been removed and the normal release binary restored.

All counts below are medians of three raw instruction measurements. B and F are brief-514's historical base (strided) and final (every checkpoint); M is today's main, `78ce151130bd6a15e30ca47b56af0b4ec95d1222`; S is the throwaway stub. The historical records were checked against their raw perf CSVs, stdout and stderr.

“Added removed” is `100 × (M − S) / (M − B)`, using today's main against the historical strided baseline. “Armed removed” is `100 × [1 − (S − U_stub)/(M − U_main)]`, using fresh matching unarmed controls in each build. These denominators answer different questions explicitly.

| Retained root / N | 514 base B | 514 final F | Main M | Stub S | Added removed | Armed removed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Bootstrap / 16 | 3,827,120 | 8,661,387 | 8,660,408 | 3,574,179 | 105.233% | 99.794% |
| Bootstrap / 128 | 4,392,850 | 37,222,136 | 37,220,519 | 3,633,301 | 102.314% | 99.958% |
| Bootstrap / 1024 | 8,406,144 | 265,700,367 | 265,700,870 | 4,125,501 | 101.664% | 99.977% |
| Range[1000] / 16 | 4,021,776 | 9,848,047 | 9,846,447 | 3,713,715 | 105.289% | 99.850% |
| Range[1000] / 128 | 4,691,417 | 44,254,099 | 44,255,362 | 3,776,340 | 102.313% | 99.962% |
| Range[1000] / 1024 | 9,435,123 | 319,502,628 | 319,503,212 | 4,267,124 | 101.667% | 99.980% |
| Range[10000] / 16 | 5,597,630 | 20,315,567 | 20,314,703 | 4,823,146 | 105.262% | 99.929% |
| Range[10000] / 128 | 7,203,868 | 107,139,604 | 107,138,340 | 4,885,906 | 102.319% | 99.983% |
| Range[10000] / 1024 | 18,499,514 | 801,717,142 | 801,714,469 | 5,374,916 | 101.676% | 99.992% |

Values above 100% are expected: brief-514's base still paid for a session walk every 64 checkpoints. The stub also removes those older walks. This is not a negative checkpoint cost. Using the original `(F − B)` denominator instead gives 101.664%–105.260%; the precise alternative values are in [comparison.json](comparison.json). Today's main differs from F by at most 0.016247% in these nine medians.

The original nine `target/brief-514/armed-*.rum` files were read and executed unchanged. Each retains `gRoot = 0`, `Range[1000]` or `Range[10000]`, prints ByteCount, and evaluates `MemoryConstrained[Do[Null, {N}], 1000000000]`. N counts loop iterations; each steady `Null` iteration reaches one memory checkpoint. Fixed entry/exit checks and setup are included in whole-process counts.

Both arms were built offline using Rust 1.85.0 / LLVM 19.1.7, the ordinary release profile, and the exact same build/execution path `/home/remi/dev/rum/target/release/rum-cli`. All `RUM_*` profiling variables were absent and `RUST_LOG` was removed. Stdin was closed. Every run used CPU 2 and the required event:

```text
scripts/run-bounded.py --timeout 60 -- taskset -c 2 perf stat -e instructions:u -x, -o <capture>.perf.csv -- /home/remi/dev/rum/target/release/rum-cli <source>.rum < /dev/null
```

Only `cpu_core/instructions/u`, coverage `100.00`, was accepted. The uncounted hybrid `cpu_atom` row was ignored. Measurements were serial, with no concurrent task-local build or test job. No timing panel, network operation, brief-518 prototype run, unsafe addition, Git mutation, or write into the earlier evidence directories occurred. Temporary files stayed in this repository.

Verbatim verification output from [analysis-evidence.txt](analysis-evidence.txt):

```text
NEW_RAW_RECORDS_VERIFIED 144
HISTORICAL_RAW_RECORDS_VERIFIED 54
ALL_CORE_EVENTS cpu_core/instructions/u
ALL_COVERAGE 100.00
ALL_EXIT_STATUS 0
ALL_STDERR empty
ALL_COMPLETION_MARKERS verified
```

The 144 new records comprise 54 required armed samples, 54 corresponding unarmed samples, and 36 longer residual-control samples. [all-runs.tsv](all-runs.tsv) contains every raw instruction count, including all three repetitions; [main-instructions.json](main-instructions.json), [stub-instructions.json](stub-instructions.json), and [stub-residual-instructions.json](stub-residual-instructions.json) retain commands, outputs, hashes and coverage. Individual perf/stdout/stderr files are alongside this report.

For example, verbatim core-event rows from `main-armed-list10k-1024-1.perf.csv` and `stub-armed-list10k-1024-1.perf.csv`:

```text
801713491,,cpu_core/instructions/u,73153477,100.00,,
5374916,,cpu_core/instructions/u,2671263,100.00,,
```

Both corresponding stdout files contain, verbatim:

```text
320075
PASS: armed-list10k-1024
Null
```

**2. Residual cost, composition, and scaling checks**

All nine armed medians increase with N. The incremental main cost has the session-size dependence the walk predicts. The stub removes that dependence:

| Retained root | Main instructions / added N, 16→128 | Main, 128→1024 | Stub, 16→128 | Stub, 128→1024 |
| --- | ---: | ---: | ---: | ---: |
| Bootstrap | 255,000.99 | 255,000.39 | 527.88 | 549.33 |
| Range[1000] | 307,222.46 | 307,196.26 | 559.15 | 547.75 |
| Range[10000] | 775,211.04 | 775,196.57 | 560.36 | 545.77 |

The small-N stub totals are dominated by fixed process/session setup, so their whole-process ratios cannot scale as N's ratio. Their short-interval slopes are also sensitive to the few-thousand-instruction variation visible in the raw runs. To resolve this before accepting the residual, separate controls extended only N to 8192 and 65536, and compared armed with unarmed in the same stub binary, three runs each. The original nine sources remained unchanged. Unarmed controls replace only the `MemoryConstrained[...]` wrapper with its `Do[...]` body, retaining setup, ByteCount, result checks and output markers.

The resulting medians and slopes are:

| Root | Armed 8192 → 65536 | Unarmed 8192 → 65536 | Armed instructions / N | Unarmed instructions / N | Extra instructions / checkpoint |
| --- | --- | --- | ---: | ---: | ---: |
| Bootstrap | 8,049,380 → 39,419,035 | 7,627,430 → 36,128,902 | 547.043 | 497.026 | 50.017 |
| Range[1000] | 8,189,747 → 39,559,330 | 7,768,807 → 36,262,967 | 547.042 | 496.899 | 50.143 |
| Range[10000] | 9,298,685 → 40,666,842 | 8,872,147 → 37,374,017 | 547.017 | 497.033 | 49.984 |

Each slope divides the count difference by 57,344. The armed steady loop is about 10.06%–10.09% more instructions than the matching unarmed loop. The whole-process residual estimate is **about 50 extra instructions per checkpoint**, above about 497 ordinary loop/evaluator instructions. The decimal variation is retained rather than treated as an exact function measurement: these counters include process setup and N-dependent parsing/output as well as evaluation.

Disassembly independently explains the approximately 50-instruction residual, without another instrumented build:

| Success-path component for `Null` | Static executed instruction count |
| --- | ---: |
| Caller: load limit and cached total, pass arguments, call, inspect result | 8 |
| Existing cold helper: register saves, expression call, saturating addition, limit comparison, result and return | 19 |
| Unchanged expression walker: variant dispatch, symbol context/name lengths, saturation, return | 23 |
| Total beyond the common armed/unarmed guard | 50 |

[checkpoint-composition.txt](checkpoint-composition.txt) preserves the exact instruction lines counted. [stub-expression-jump-table.json](stub-expression-jump-table.json) verifies the symbol dispatch target directly from the measured ELF. The field load remains real, the helper stays cold and non-inlined, and both saturating addition and the comparison are present.

Brief-519's inlining anomaly was specifically checked. Ignoring relocation addresses, branch byte offsets and padding, `evaluate_inner_unstable`, `finish_do_spec_eval`, and `expr_memory_proxy_bytes` have identical assembly. `evaluate` and `evaluate_once_unstable_after_input_check_with_cycle_tracking` differ only in substituting the cached-value load for the session-pointer argument; their instruction counts and the call to `evaluate_inner_unstable` are retained. The `normalized-*.diff` and per-function `.asm` files preserve these comparisons. No new favourable inlining change is being counted as the walk's benefit.

This floor is specific to an atomic active expression. `expr_memory_proxy_bytes(expr)` still walks the current expression recursively, including its occurrence-based children/capacity and special cases. The large Range values here are retained session state, not the repeatedly checked expression. A maintained session total alone cannot make a checkpoint on a large active expression O(1). Time/interrupt behavior and all other evaluator work also remain; these workloads do not arm a timeout.

**3. Verdict on the approximately 0.7%**

**Yes: the measured payoff is large enough to justify that enabling cost and proceeding to C, D and E.** The largest required case drops from 801,714,469 to 5,374,916 instructions, a 149.16× whole-process reduction in this deliberately idealized experiment. The old approximately 43× penalty was dominated by the repeated session walk; the unchanged expression check does not leave a comparable penalty on these shapes.

The measured floor is approximately 50 extra instructions on a roughly 497-instruction `Null` loop, not a zero-cost armed checkpoint. The 0.7% figure is the brief's accepted budget from the earlier route measurement; this experiment neither re-runs that route nor establishes that the complete maintained-total implementation fits that budget. Final acceptance needs its full bookkeeping costs and correctness evidence.

**4. Why the stub over-estimates the achievable win**

The real implementation must pay for initial accounting when a limit is armed, and for charge/release updates when roots are inserted, replaced, shared, imported, removed or rolled back. That includes definitions and lazy indexes, caches, evaluation and dynamic-binding stacks, vector/string capacity changes, buffered prints/messages/traces/reaps, suspension/resumption/abandonment and host extension paths. Occurrence-based accounting, ledger storage, conservative saturation and correct release after saturation all remain obligations. The stub performs none of them, and also skips the old walk's peak update. A correct maintained-total/peak implementation must supply the corresponding behavior.

These read-mostly `Do[Null]` loops are favourable to the design: a large retained root is unchanged across many checkpoints. They do not bound maintenance cost on mutation-heavy workloads. I cannot give a measured numeric estimate or upper bound for an implementation that does not exist. Its true benefit with the other work held fixed is strictly smaller than this stub's benefit.

There is a useful conditional instruction budget. Let K be additional real maintenance/arming/release work amortized per steady checkpoint, above this stub. To retain 99% of today's incremental armed overhead reduction, the measured slopes allow approximately the following K; allowing only 90% removal gives the second column:

| Root | K ceiling retaining 99% removal | K ceiling retaining 90% removal |
| --- | ---: | ---: |
| Bootstrap | 2,495 instructions/checkpoint | 25,400 instructions/checkpoint |
| Range[1000] | 3,017 | 30,620 |
| Range[10000] | 7,697 | 77,420 |

These are derived headroom limits, not predictions of bookkeeping cost: `K ≤ (1 − fraction) × (main_slope − unarmed_slope) − residual`. They apply to the measured steady-loop shapes; short requests must also amortize initial accounting, and write-heavy requests require separate measurement. The original default-path price remains a separate constraint.

**5. Stub removal and clean-tree evidence**

The saved original `memory.rs` bytes were copied back after verifying the file still matched this task's stub hash. All **1,510 tracked-file hashes** match their initial values. The nine original workload hashes and `target/accounting-route-518/prototype.patch` hash are unchanged. No Git index/config/branch operation was used. The temporary 32,019-byte memory module was not split during this measurement; a future organizational boundary would separate session summation from value/container sizing, and this task leaves the original module intact.

The restored release binary is byte-identical to the initially measured main binary:

```text
beda3aa055222aa5ca4db7320684e05848ad2bbad5f247431e84e6d02d103376
```

`cargo fmt --check` and `cargo test --offline --locked -p rum-runtime` ran after restoration. Verbatim result lines from [restored-runtime-tests.log](restored-runtime-tests.log):

```text
test result: ok. 1012 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 5.31s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

Evidence and this report are in a new ignored `target/measure-achievable-520/` directory. The production stub and runnable stub build are gone. The following command produced no output (exit 0); [final-status.txt](final-status.txt) is its zero-byte capture:

```console
$ git status --porcelain
```
