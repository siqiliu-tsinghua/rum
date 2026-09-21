# brief-503 — split A1's +14.33% into atomics and equality

## Goal

brief-502 measured the A1 candidate at **+14.33% instructions on ordinary
evaluation** against `main`, and +10.43% to +18.85% on every other workload,
while the ownership migration alone costs +0.06% to +0.84%. So the whole cost
is the representation — but brief-502 said plainly that it did not isolate
*which part* of the representation, and that is the question this brief
answers.

Two candidate mechanisms with completely different remedies:

- **`Arc` refcount traffic.** `Symbol` clone and drop became an atomic
  increment and decrement, on a type that is cloned constantly, plus one more
  pointer hop to reach owned `String`s. This is intrinsic to any owning
  representation.
- **The equality demotion.** `eq` stopped being a fast *accept* on a dense
  `u32` and became a pointer compare, then a fingerprint reject, then a text
  comparison; `hash` and `Ord` changed with it. This one might be repairable.

Build the two variants, measure them on brief-502's own workloads, and
attribute the number. **Diagnostic. Nothing lands, nothing is tuned, no
repair is attempted even if the answer is obvious.**

## Where the tree is

Branch `spike/brief-499-symbol-non-copy` at `357d3332`, carrying the
candidate representation and brief-502's report. `main` is clean at
`3024c33c`. **Do not switch, merge, rebase, commit or stash.**

Everything brief-502 built is still in place and you should reuse all of it
rather than rebuild it: the workloads under `target/brief-502/cost/*.rum`
(including the `-full`/`-half`/`-zero` setup-subtraction triples), the
measurement scripts `cost_measure.py` and `cost_summarize.py`, and the
reference binaries `rum-base`, `rum-stage1`, `rum-candidate` and their
`-alloc` counterparts. Reusing the exact workload files is what makes this
brief's numbers comparable to brief-502's.

## The two variants

Both are edits **from the candidate that is in the tree**, and both must be
**correct programs**, not measurement hacks.

### V-arc — the Arc, without the equality demotion

Keep the candidate's `Arc<SymbolText>` with owned `String`s. Restore the
dense `u32 id` field on `Symbol` and `SymbolText`, drop the fingerprint field,
and restore stage1's `eq`, `hash` and `Ord` exactly — fast accept on `id`,
hash on `id`, `Ord` shortcut on `id` equality. This stays correct because the
interner is still process-global and still assigns one dense id per distinct
text, which is the same guarantee stage1 relied on.

`Symbol` must remain 24 bytes: `u32` id, `Option<NonZeroU64>`, `Arc`.

### V-eq — the equality demotion, without the Arc

Keep the candidate's fingerprint, `eq`, `hash` and `Ord`, but put the text
back behind `&'static SymbolText` with leaked `&'static str` fields, exactly
as stage1 stored it. `Arc::ptr_eq` becomes `std::ptr::eq`. The interner goes
back to leaking, which is fine — this variant is a measurement, not a
proposal.

`Symbol` must remain 24 bytes: `u64` fingerprint, `Option<NonZeroU64>`,
`&'static`.

### The sum check

`V-arc` and `V-eq` should account for the candidate between them. **Report
whether they add up**, per workload, and treat a large residual as a finding
in its own right — it would mean the two changes interact in codegen, which
is a different conclusion from either mechanism dominating.

## How to handle the tree

You will edit `crates/rum-core/src/expr.rs` twice. Before building the second
variant, **write the first variant's complete `git diff` to
`target/brief-503/veq.diff` or `varc.diff`** — the tree is restored from git
at the end by the director, so anything you have not saved under
`target/brief-503/` is gone.

Leave the tree in whatever state the last variant left it. Do not try to
restore the candidate yourself, and do not use git to do it.

## Per-variant correctness guard

Before measuring a variant, and pasting the output for each:

```
cargo test -p rum-core
```

must be green, and every workload's own liveness assertion must pass in the
run (they `Exit[2]` otherwise, so a non-zero exit invalidates that cell).
That is the whole gate — do **not** run the workspace suite, whose state is
already known and recorded in brief-502.

Also report `size_of::<Symbol>()`, `::<Atom>()` and `::<Expr>()` per variant.
A variant whose `Symbol` is not 24 bytes has introduced a confounder; say so
and stop rather than measuring it.

## The measurement

Identical instrument and discipline to brief-502, so the numbers compose:

```
taskset -c 2 perf stat -e instructions:u -x, -- <binary> <workload>.rum < /dev/null
```

`cpu_core/instructions/u`, `100.00` multiplex coverage, three runs per arm per
workload, profiling env OFF, stdin closed, release profile, all binaries built
at this same path. Judge against the within-arm spread, which brief-502
observed at 0.000050% to 0.117%. **Do not run the timing panel.**

Use brief-502's full workload set and its setup-subtraction method, so each
row can be placed beside the row it already produced. Allocation counts are
not required — brief-502 established that ordinary evaluation and warm lookup
add none — but if a variant moves them, report it.

Deliver one table per workload:

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---|---|---|---|---|---|---|---|---|

where `Δ arc` is `stage1 → V-arc`, `Δ eq` is `stage1 → V-eq`, `Δ both` is
`stage1 → candidate`, and `residual` is `Δ both − Δ arc − Δ eq`.

## Scope

- Files you MAY modify: `crates/rum-core/src/expr.rs` only, plus anything the
  compiler forces from it — and if it forces anything outside `rum-core`,
  report that, because it would mean a variant changed a public shape it
  should not have.
- You MAY write under `target/brief-503/**` and append your report to this
  file.
- **MUST NOT touch**: every other `crates/**` file unless compiler-forced,
  `scripts/**`, `docs/**` except this file, `AGENTS.md`, `TODO.md`,
  `PROGRESS.md`, `Cargo.toml`, `Cargo.lock`, `.git`, the ownership-graph
  inventory fixtures, and anything under `target/brief-501/` or
  `target/brief-502/` — read those, never overwrite them.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`.**
- **No repair.** If the split shows one mechanism dominating and an obvious
  fix occurs to you, write it in the open-questions section and do not build
  it. The lever belongs to the director.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no third-party engine comparisons.
- No network. Touch nothing outside this repo.
- Do not measure while another CPU-heavy workload runs. Build both variants
  first, then measure.
- **Stop-loss, keyed on progress.** If a variant fails to build twice and the
  second attempt produced no more information than the first, stop and report
  with the variant you have — one variant plus the sum check missing is still
  a useful result.
- Paste real verbatim output. Never fake or interpolate a number.

## Report format (your final message, and appended to this file)

1. **The attribution table**, per workload, with the residual column. Written
   first.
2. **The verdict on the split**: which mechanism dominates, on which
   workloads, and whether the two add up.
3. **Per-variant evidence**: `size_of` values, `cargo test -p rum-core`
   output, and the paths of the saved diffs.
4. **What changed** — the two variants described precisely enough to rebuild
   them from the report alone.
5. **Open questions** — including any repair you thought of and did not
   build.


---

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| cold-intern (gross) | 411,254,429 | 411,661,920 | 451,410,663 | 447,151,053 | 486,746,814 | +9.6557% | +8.6209% | +18.2395% | -0.0372 pp |
| cold-intern (payload) | 407,792,916 | 408,184,043 | 447,720,543 | 443,329,018 | 482,721,028 | +9.6859% | +8.6101% | +18.2606% | -0.0354 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| warm-intern (gross) | 788,928,686 | 789,787,738 | 863,755,876 | 838,437,810 | 912,763,946 | +9.3656% | +6.1599% | +15.5708% | +0.0453 pp |
| warm-intern (payload) | 372,927,125 | 373,319,024 | 407,614,632 | 386,057,532 | 420,713,632 | +9.1867% | +3.4122% | +12.6955% | +0.0966 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| literal-integer (gross) | 5,177,175 | 5,219,141 | 5,578,663 | 5,570,972 | 5,911,133 | +6.8885% | +6.7412% | +13.2587% | -0.3710 pp |
| literal-integer (payload) | 1,752,306 | 1,773,493 | 1,921,220 | 1,782,030 | 1,916,000 | +8.3297% | +0.4814% | +8.0354% | -0.7757 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| symbol-ownvalue-read (gross) | 7,694,857 | 7,731,659 | 8,460,424 | 8,214,011 | 8,896,710 | +9.4257% | +6.2387% | +15.0686% | -0.5958 pp |
| symbol-ownvalue-read (payload) | 4,237,696 | 4,256,754 | 4,775,858 | 4,392,532 | 4,870,025 | +12.1948% | +3.1897% | +14.4070% | -0.9775 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| set-ownvalue (gross) | 8,052,441 | 8,062,752 | 8,988,201 | 8,644,448 | 9,585,425 | +11.4781% | +7.2146% | +18.8853% | +0.1926 pp |
| set-ownvalue (payload) | 4,592,053 | 4,584,260 | 5,297,350 | 4,819,495 | 5,552,232 | +15.5552% | +5.1314% | +21.1151% | +0.4286 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| module-local (gross) | 15,039,867 | 15,083,233 | 16,456,733 | 16,053,568 | 17,361,642 | +9.1061% | +6.4332% | +15.1056% | -0.4338 pp |
| module-local (payload) | 11,587,617 | 11,611,715 | 12,772,833 | 12,236,708 | 13,338,563 | +9.9995% | +5.3824% | +14.8716% | -0.5104 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| downvalue-exact-hit (gross) | 15,330,772 | 15,429,032 | 17,213,026 | 16,475,802 | 18,203,936 | +11.5626% | +6.7844% | +17.9850% | -0.3620 pp |
| downvalue-exact-hit (payload) | 11,858,217 | 11,938,188 | 13,513,403 | 12,637,110 | 14,164,959 | +13.1948% | +5.8545% | +18.6525% | -0.3968 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| matcher-single-tiny (gross) | 15,051,041 | 15,142,077 | 16,086,417 | 15,717,110 | 16,731,688 | +6.2365% | +3.7976% | +10.4980% | +0.4639 pp |
| matcher-single-tiny (payload) | 11,577,580 | 11,649,938 | 12,384,055 | 11,877,064 | 12,688,423 | +6.3015% | +1.9496% | +8.9141% | +0.6630 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| replaceall-simple (gross) | 9,195,455 | 9,241,946 | 9,973,121 | 9,728,694 | 10,399,109 | +7.9115% | +5.2667% | +12.5208% | -0.6574 pp |
| replaceall-simple (payload) | 5,686,946 | 5,713,543 | 6,232,331 | 5,852,246 | 6,315,757 | +9.0800% | +2.4276% | +10.5401% | -0.9675 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| sort-symbol-default-width4000 (gross) | 113,001,600 | 113,138,280 | 123,306,728 | 122,870,806 | 132,990,842 | +8.9876% | +8.6023% | +17.5472% | -0.0428 pp |
| sort-symbol-default-width4000 (payload) | 5,274,256 | 5,288,637 | 5,320,116 | 5,604,717 | 5,595,582 | +0.5952% | +5.9766% | +5.8039% | -0.7679 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| sort-symbol-default-width16000 (gross) | 443,051,386 | 443,485,485 | 483,444,983 | 481,680,595 | 521,423,537 | +9.0103% | +8.6125% | +17.5740% | -0.0488 pp |
| sort-symbol-default-width16000 (payload) | 20,964,611 | 20,931,561 | 21,100,395 | 22,192,620 | 22,223,942 | +0.8066% | +6.0247% | +6.1743% | -0.6570 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| ordinary-smoke (gross) | 1,376,123,225 | 1,376,893,664 | 1,527,227,336 | 1,424,139,142 | 1,573,313,524 | +10.9183% | +3.4313% | +14.2654% | -0.0842 pp |
| ordinary-smoke (payload) | 1,372,607,777 | 1,373,357,986 | 1,523,480,087 | 1,420,259,057 | 1,569,227,602 | +10.9310% | +3.4151% | +14.2621% | -0.0840 pp |

| workload | base | stage1 | V-arc | V-eq | candidate | Δ arc | Δ eq | Δ both | residual |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| empty-process (gross) | 3,363,719 | 3,383,457 | 3,592,798 | 3,725,059 | 3,930,740 | +6.1872% | +10.0962% | +16.1753% | -0.1082 pp |


Counts are **median user-space instructions from three fresh runs per arm**.
`gross` is the complete process. `payload` is median(full) minus
median(the matching zero/setup control), using brief-502's method without
changing any workload. Warm lookup subtracts its cold seed; each sort
subtracts its prepared input at the same width. The empty process has no
payload subtraction. All three deltas use **stage1** as their denominator;
the residual is **Δ both − Δ arc − Δ eq**, in percentage points (pp).
These are newly measured five-arm counts, not a mixture of old and new runs.

**2. Verdict on the split.** The Arc/owned-text side dominates ordinary
evaluation: **+10.9183% gross**, versus **+3.4313%** for the fingerprint and
comparison side. The candidate is **+14.2654%** over stage1, with a
**−0.0842 pp** residual. Relative to base, the rerun gives **+14.3294%**,
reproducing brief-502's +14.3293%; the ownership migration is +0.0560%.
After setup subtraction, Arc contributes +10.9310%, comparison/fingerprint
+3.4151%, and both +14.2621%, with −0.0840 pp residual. Those isolated
effects are about **76.64% and 23.95%** of the combined payload penalty;
the negative interaction is −0.59% of that penalty.

Arc/owned text also dominates warm lookup and all seven loop payloads.
The literal loop's small V-eq effect (+0.4814%) is within its combined
stage1/V-eq observed-range bound (0.5363 pp), so it is not resolved.
Cold interning is much more evenly split: +9.6859% Arc and +8.6101%
comparison/fingerprint. **Sort payloads reverse the result**: V-eq costs
+5.9766% / +6.0247% at widths 4,000 / 16,000; the V-arc estimates
(+0.5952% / +0.8066%) fall inside their stage1/V-arc range bounds
(2.3961 / 0.9791 pp). Startup also favors the comparison/fingerprint side:
empty-process V-eq is +10.0962%, versus +6.1872% for V-arc.

The effects approximately account for the candidate, but **they do not
add exactly**. No residual exceeds one percentage point. Several are
nonetheless clearly larger than the measured variation. The largest
payload residual magnitudes are ownvalue read (−0.9775 pp) and replacement
(−0.9675 pp); matching adds +0.6630 pp. Replacement's residual is −9.18%
of its combined payload penalty, and matching's is +7.44%, so those
interactions should not be dismissed. The ordinary residual is smaller
in proportion but very well resolved. These findings show nonadditivity
of the representation changes, consistent with code-generation/layout
interaction; this experiment does not identify the responsible instructions.

For each residual, the comparison bound below is the sum of the observed
instruction ranges of stage1, V-arc, V-eq, and candidate, divided by the
stage1 median. Payload ranges include both full and setup ranges. This
is an observed-range bound, not a confidence interval. “Resolved” means
the residual magnitude exceeds that bound; set-ownvalue payload only
narrowly does so (0.4286 versus 0.4017 pp).

| workload | gross residual bound (pp) | gross resolved | payload residual bound (pp) | payload resolved |
|---|---:|---|---:|---|
| cold-intern | 0.0632 | no | 0.0661 | no |
| warm-intern | 0.0339 | yes | 0.1674 | no |
| literal-integer | 0.2115 | yes | 1.2211 | no |
| symbol-ownvalue-read | 0.1036 | yes | 0.3733 | yes |
| set-ownvalue | 0.1266 | yes | 0.4017 | yes |
| module-local | 0.0797 | yes | 0.1553 | yes |
| downvalue-exact-hit | 0.0628 | yes | 0.1741 | yes |
| matcher-single-tiny | 0.1180 | yes | 0.2368 | yes |
| replaceall-simple | 0.2911 | yes | 0.6701 | yes |
| sort-symbol-default-width4000 | 0.0759 | no | 3.2754 | no |
| sort-symbol-default-width16000 | 0.0699 | no | 2.1313 | no |
| ordinary-smoke | 0.0007 | yes | 0.0017 | yes |
| empty-process | 0.2729 | no | — | — |


**3. Per-variant evidence.** Both variants passed `cargo test -p rum-core`
(seven unit tests and zero doc tests each) before measurement. Their
measured layouts are identical:

| variant | `size_of::<Symbol>()` | `size_of::<Atom>()` | `size_of::<Expr>()` |
|---|---:|---:|---:|
| V-arc | 24 | 32 | 32 |
| V-eq | 24 | 32 | 32 |

The safe Rust probe is `target/brief-503/sizes.rs`, linked separately
against each variant's built `rum-core` rlib. Verbatim V-arc size output:

```text
size_of::<Symbol>() = 24
size_of::<Atom>() = 32
size_of::<Expr>() = 32
```

Verbatim V-eq size output:

```text
size_of::<Symbol>() = 24
size_of::<Atom>() = 32
size_of::<Expr>() = 32
```


The exact test command was `cargo test -p rum-core --offline --locked`,
with `CARGO_NET_OFFLINE=true`, `CARGO_TARGET_DIR` set to the absolute
`target/brief-503/build` path, and `TMPDIR` to the absolute
`target/brief-503/tmp` path. All builds used Rust/Cargo 1.85.0 at
`/home/remi/dev/rum`; release binaries used the ordinary optimized profile.

Verbatim successful V-arc test output:

```text
   Compiling proc-macro2 v1.0.106
   Compiling quote v1.0.45
   Compiling unicode-ident v1.0.24
   Compiling autocfg v1.5.0
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling thiserror v2.0.18
   Compiling num-traits v0.2.19
   Compiling syn v2.0.117
   Compiling num-integer v0.1.46
   Compiling serde_derive v1.0.228
   Compiling thiserror-impl v2.0.18
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling num-bigint v0.4.6
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.93s
     Running unittests src/lib.rs (target/brief-503/build/debug/deps/rum_core-a204b80b601ed806)

running 7 tests
test expr::tests::displays_global_symbol_without_context_prefix ... ok
test expr::tests::displays_string_atoms_with_escaped_special_characters ... ok
test expr::tests::normalizes_rational_atoms ... ok
test expr::tests::normalizes_int_values_and_orders_by_numeric_value ... ok
test expr::tests::displays_nested_call_expression ... ok
test expr::tests::displays_message_name_shorthand ... ok
test expr::tests::displays_system_symbol_without_context_prefix ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rum_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Verbatim successful V-eq test output:

```text
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.11s
     Running unittests src/lib.rs (target/brief-503/build/debug/deps/rum_core-a204b80b601ed806)

running 7 tests
test expr::tests::displays_nested_call_expression ... ok
test expr::tests::normalizes_int_values_and_orders_by_numeric_value ... ok
test expr::tests::normalizes_rational_atoms ... ok
test expr::tests::displays_message_name_shorthand ... ok
test expr::tests::displays_string_atoms_with_escaped_special_characters ... ok
test expr::tests::displays_system_symbol_without_context_prefix ... ok
test expr::tests::displays_global_symbol_without_context_prefix ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rum_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```


The complete saved diffs against the candidate are
[`target/brief-503/varc.diff`](../../../target/brief-503/varc.diff) and
[`target/brief-503/veq.diff`](../../../target/brief-503/veq.diff).
V-arc's diff was saved before writing V-eq. Complete sources are also
retained as `expr-varc.rs` and `expr-veq.rs` in the same directory.
No compiler-forced edit outside `rum-core` was needed.

All **615 instruction runs** (41 original workload/control files × five
arms × three repetitions) exited 0, passed the exact required liveness
output, and reported `cpu_core/instructions/u` at **100.00** coverage.
Both variants, including their optional allocator binaries, were built
before any measurement. The three retained plain reference binaries and
their allocator counterparts were reused without rebuilding; hashes match
brief-502's binary manifest. Profiling variables were removed for instruction
runs, stdin was `/dev/null`, and every command used this instrument:

```sh
taskset -c 2 perf stat -e instructions:u -x, -- <binary> <original-workload>.rum < /dev/null
```

Verbatim ordinary V-arc run 1 perf output:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
1527223813,,cpu_core/instructions/u,170825436,100.00,,
```

Verbatim ordinary V-eq run 1 perf output:

```text
<not counted>,,cpu_atom/instructions/u,0,0.00,,
1424139142,,cpu_core/instructions/u,153644344,100.00,,
```


The extra `cpu_atom/instructions/u` row is uncounted on CPU 2, as in
brief-502; the requested core event has full coverage. Across the full
workload cells, the within-arm spread is **0.000054%–0.110948%**.
The largest absolute median drift of a reused reference versus brief-502
is 0.053735% (stage1 matcher), below its current 0.066787% spread.
The original half-size and zero/setup controls were all run, not inferred.
Ordinary payload full/half ratios are 1.999982 for V-arc and 1.999969 for
V-eq. Module and width-4,000 sort retain their roughly 1.93 ratios and
are not claimed to double exactly. All raw controls, ranges, and scaling
ratios are retained in `cost/summary.json` and `cost/spreads-and-scaling.json`.

The original `cost_measure.run` was imported with bytecode writes disabled
and only its output root/TMPDIR redirected to brief-503; `generate()` was
never called. The summarizer was copied with only its output root, five-arm
list, and original three-reference delta selection adapted. Its median and
setup-subtraction logic is unchanged. There were 69 quiet-machine checks
before phases and workload/control groups, all reporting no competing
CPU-heavy process. No timing panel or workspace test suite ran.

An additional **390 allocation runs** (26 full/zero/setup-full cells ×
five arms × three repetitions) all passed liveness. These used separate
`--features alloc-count` binaries, `RUM_PROFILE_ALLOC=1`, and no perf.
Every allocation count repeated identically across three runs, including
controls: **V-arc equals candidate; V-eq equals stage1 and base in every
cell**. Payload counts follow (empty-process is gross):

| workload | base | stage1 | V-arc | V-eq | candidate |
|---|---:|---:|---:|---:|---:|
| cold-intern | 480,032 | 480,032 | 544,032 | 480,032 | 544,032 |
| warm-intern | 432,094 | 432,094 | 432,094 | 432,094 | 432,094 |
| literal-integer | 6 | 6 | 6 | 6 | 6 |
| symbol-ownvalue-read | 2,006 | 2,006 | 2,006 | 2,006 | 2,006 |
| set-ownvalue | 1,006 | 1,006 | 1,006 | 1,006 | 1,006 |
| module-local | 11,512 | 11,512 | 12,512 | 11,512 | 12,512 |
| downvalue-exact-hit | 6,006 | 6,006 | 6,006 | 6,006 | 6,006 |
| matcher-single-tiny | 3,010 | 3,010 | 3,010 | 3,010 | 3,010 |
| replaceall-simple | 2,506 | 2,506 | 2,506 | 2,506 | 2,506 |
| sort-symbol-default-width4000 | 22 | 22 | 22 | 22 | 22 |
| sort-symbol-default-width16000 | 22 | 22 | 22 | 22 | 22 |
| ordinary-smoke | 2,200,002 | 2,200,002 | 2,200,002 | 2,200,002 | 2,200,002 |
| empty-process | 1,551 | 1,551 | 2,877 | 1,551 | 2,877 |


Thus the Arc side carries the candidate's +64,000 cold-intern payload
allocations and +1,000 Module payload allocations. Ordinary and warm
payload allocation counts do not change. V-arc/candidate ordinary gross
is 2,203,097, versus 2,201,763 for V-eq/stage1/base: the +1,334 difference
is entirely in startup/setup. The word “Arc” here includes the prescribed
owned strings and their access/layout costs, not just atomic instructions.
Likewise, “eq” includes fingerprint construction, hashing, and ordering.

Raw stdout/stderr/JSON captures are under
[`target/brief-503/cost/`](../../../target/brief-503/cost/).
The complete measurement argv, profiling setup, and stdin settings are
in `logs/measurement-commands.jsonl`; build commands are in
`logs/build-commands.jsonl`, and binary hashes are in `logs/binaries.jsonl`.
`reference-sha256.json` records every original workload/control, the retained
scripts/results used, and all six reference binaries; every recorded hash
was rechecked unchanged after measurement. `cost/attribution.json` records
unrounded deltas, integer residuals, and range bounds.

**4. What changed.** Both variants start from the saved candidate
`expr-candidate.rs` and retain `#[derive(Clone, Debug)]` without `Copy`.

- **V-arc:** retain `Arc<SymbolText>`, its owned `String` context/name,
  and the process-global interner's strong Arcs. Replace the fingerprint
  fields in both `Symbol` and `SymbolText` with `u32 id`; remove the FNV
  fingerprint function. Restore stage1's `next_id = u32::try_from(pairs.len())`
  before interner insertion and initialize symbols from `text.id`.
  Restore `interned_id()` to the dense ID and the qualified-name comparison's
  equal-ID shortcut. `PartialEq` and `Hash` are verbatim stage1: equal ID
  plus generated identity; hash `(u64::from(id) * 0x9e3779b97f4a7c15) ^ identity`
  with wrapping multiplication. `Ord` follows stage1 exactly: equal ID
  returns identity order; otherwise pointer-equal context strings compare
  equal, then lexical context, name, and generated identity. The only type
  adaptations are `as_str()` for pointer comparison and borrowing owned
  strings for lexical comparison. No fingerprint remains.
- **V-eq:** retain both `u64` fingerprints, the complete FNV function,
  candidate hash, pointer/fingerprint/text equality, and candidate lexical
  `Ord`. Change symbol/interner text handles to `&'static SymbolText`,
  context/name fields to interned `&'static str`, and allocate each text
  pair through `Box::leak(Box::new(...))`. Replace all three relevant
  `Arc::ptr_eq` calls with `std::ptr::eq(self.text, other.text)` and adjust
  access, comparisons, and serialization borrows to the static strings.
  The serde input helper keeps its owned `String` fields and deserialization
  still interns the input; no public method signature changes.

The working tree is intentionally left in **V-eq** state. No candidate
restoration, branch creation/change, switch/merge/rebase/commit/stash/push,
explicit index/config edit, network access, new unsafe code, or performance
repair was performed.
HEAD was `745b35ca`, one documentation-only commit after the brief's
`357d3332`; candidate source is identical, and `main` remains `3024c33c`.

Procedural deviation: initial inspection used the read-only
`git branch --show-current`, despite the brief's blanket prohibition on
`git branch`. It only reported the current branch; no branch mutation
was attempted. Subsequent Git inspection used `GIT_OPTIONAL_LOCKS=0`.

The first V-eq compile exposed an overly broad local replacement that
also changed the serde input helper to static references. Restoring that
helper's original owned fields resolved the compiler's lifetime error;
the second compile/test passed. No stop-loss condition was reached.
An initial V-arc formatting check and the build helper's first copy from
`release/rum` (actual filename `release/rum-cli`) were corrected before
continuing; the release compilation itself succeeded. Initial outputs
remain in `logs/veq-first-test.log`, `logs/varc-initial-format-check.log`,
and `logs/varc-transcript.log`.

File-size assessment: the original `expr.rs` has 26,072 implementation
bytes before its in-file tests (28,223 total). A real future boundary is
symbol interning/identity/serialization versus expression storage and
numeric values, with tests moved to a sibling module. No split was made
because this diagnostic explicitly permits only the representation edit;
organizational churn would add an uncontrolled variable.

**5. Open questions.** The primary ordinary-evaluation cost remains on
the owning-representation side, even with dense-ID operations restored.
These variants do not separate atomic refcounts from owned-string access,
drop glue, layout, or compiler effects; instruction counts also do not
measure cycle cost. Cold-intern V-eq includes fingerprint construction,
so its result cannot be attributed solely to equality checks. The resolved
residuals warrant instruction/code-generation investigation if the director
chooses to pursue that question.

A possible repair direction, **not implemented**, is an owning symbol
representation with a sound fast identity accept/hash path that remains
correct across interner/session domains. V-arc relies on the current
process-global dense-ID guarantee; it does not solve domain identity,
reclamation, escaped-symbol lifetime, or the remaining owning cost.
The director retains that decision. No third representation, tuning,
repair, or landing is proposed by this diagnostic.
