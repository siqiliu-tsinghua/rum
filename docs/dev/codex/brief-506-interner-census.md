# brief-506 — how much does rum actually intern?

## Goal

Before anything is built, measure what the real system interns. brief-505's
Draft A says outright that it cannot answer this — "do not infer a legitimate
workload working set from that subtraction" — and the director's D1 ruling
(2026-09-22) turned the proposed 8 MiB default from a number into a
measurement obligation. This brief is that measurement.

The deliverable is a census, in bytes and distinct pairs, of what the
process-global symbol interner holds under four conditions. The director sets
the quota default from it. **No production change lands. Nothing is
proposed. Instrumentation built here is throwaway and must be stated as
such.**

## Why this comes first

The mitigation is a process-wide byte cap on the interner. If the ordinary
test suite or the microbench panel already interns more than a plausible cap,
the design is not merely mis-tuned, it is wrong, and that must be known before
Draft A's storage proof is written rather than after. A premise check precedes
the build.

## What is already settled, so you do not re-derive it

- `Symbol` does not change — not its size, `Copy`-ness, id-based fast-accept
  equality, or its lifetime-free `'static`-compatible public type
  (user ruling 2026-09-21, `TODO.md` §The interner ruling of 2026-09-21).
- There is no reclamation and no per-session attribution. The quota is
  process-wide.
- **The charging model is a proxy, and only a proxy** (director ruling D2):
  exact Rust allocation-layout bytes plus a conservative fixed reserve,
  explicitly excluding allocator metadata, rounding and RSS. Use that model
  here so the census is denominated in the same units the quota will be. Say
  so, and say where it under- or over-counts.
- Director-verified 2026-09-22, and you should confirm rather than assume:
  **`Module` gensyms do not grow the interner** — `Symbol::with_identity`
  varies only the numeric identity while `intern_symbol_text` is keyed on
  `(context, name)` (`crates/rum-core/src/expr.rs:80-96,115-122`) — while
  **`Unique[]` mints fresh text on every call**
  (`crates/rum-runtime/src/runtime/scoping/unique.rs:42`).

## The instrumentation

Add a throwaway census to the interner: counts of distinct strings and pairs,
and the D2-model byte total, reported at process exit behind an env var
(`RUM_INTERNER_CENSUS=1` or similar). Keep it simple — this build is never
committed and never measured for speed, so an atomic or even a mutex inside
the existing interner lock is fine. **Do not propose it for landing** and do
not shape it as if it were production diagnostics.

Report the exact byte formula you used, term by term: the `str` bytes, the
`SymbolText` allocations, the `HashSet`/`HashMap` backing storage at the
capacity actually reached, and anything else the model charges. Where the
real container's backing bytes are not exactly knowable from safe Rust, say
so and give a bound with its direction, rather than a number that looks
exact.

## The four measurements

Write each into your report before starting the next.

### 1. Bootstrap

A session created and `install_core` run, nothing else evaluated. Report
distinct strings, distinct pairs, and bytes. This is the floor any cap must
clear by a wide margin, and it is the number the reserved-vocabulary work in
the next brief sizes itself against.

Also report the breakdown between `System\`` and other contexts, and how many
of the pairs come from the stdlib catalog rather than from
`CANONICAL_CORE_HEADS` (`crates/rum-core/src/heads.rs:112`).

### 2. The ordinary test suite

`cargo test --workspace --no-fail-fast`, census enabled. Report the **maximum
over test binaries**, not a sum — the quota is per process and each test
binary is one. Name the binary that holds the maximum.

Expect one pre-existing failure and do not treat it as yours:
`rum-stdlib`'s `tests::ownership_graph::production_inventory_is_exact` is
**green on `main`**; if you see it fail you are not on `main`, so stop and say
so. Run **without** `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; if loopback binding is
unavailable in your sandbox, report those native ZeroMQ failures as
environmental and say plainly that those tests did not execute.

Derive the timeout from a recorded run of the same shape; brief-500 used 900 s
against a ~97 s workload and did not time out.

### 3. The microbench panel

`scripts/core-language-microbench.sh`, census enabled, release build. Report
the total at exit. This is the heaviest ordinary workload the repository has,
and it deliberately builds symbol-sorting fixtures at widths 4 000 and 16 000
(`scripts/core-language-microbench.sh:884-891`) — so it is also a realistic
upper bound on how much a legitimate program interns on purpose.

### 4. The language's own unbounded consumer

`Unique[]` in a loop, and separately `Symbol[StringJoin[…, ToString[i]]]` in
a loop, at a few sizes. Report **bytes per call** for each, and then the
question the director actually needs answered:

> at a candidate cap of 8 MiB, and at whatever the bootstrap floor turns out
> to be, **how many `Unique[]` calls does a process get before refusal?**

Give the arithmetic. Give it for 8 MiB and for two other plausible caps of
your choosing, so the director can see the shape of the trade rather than one
point.

## What I want back

1. The four measurements, each with its byte formula and its uncertainty.
2. The `Unique[]` budget table.
3. **Your judgement on whether a process-wide byte cap is a workable
   mitigation at all**, given those numbers — and if it is not, say so
   plainly. A refutation here is worth more than a tuned number, and this
   line has already accepted one refutation on evidence.
4. Anything you found that changes the plan in brief-505 §1.

## Scope

- You MAY add throwaway census instrumentation to
  `crates/rum-core/src/expr.rs`, and whatever the compiler forces from it.
  Every file in the report.
- You MAY write under `target/brief-506/**` and append your report to this
  file.
- **MUST NOT touch**: `scripts/**` — the panel must stay byte-identical or
  its numbers are not comparable to recorded ones; `docs/**` except this
  file; `AGENTS.md`; `TODO.md`; `PROGRESS.md`; `Cargo.toml`; `Cargo.lock`;
  `.git`; the ownership-graph inventory fixtures.
- The instrumentation is throwaway. Leave it in the working tree; the
  director discards it.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index. (`git checkout` restores
  from the index, which on uncommitted output is a delete.)
- **No new `unsafe`**, including to inspect a container's backing storage. If
  a number needs `unsafe` to obtain, bound it instead and say which direction
  the bound errs.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo. Bound anything long with
  `scripts/run-bounded.py --timeout SECONDS -- <command>`, sized from a
  measured run of the same shape.
- **Stop-loss, keyed on progress.** If a measurement fails twice and the
  second attempt produced no more information than the first, stop and report
  what you have. Partial census plus the `Unique[]` budget is still a useful
  result.
- Paste real verbatim output. Never fake or interpolate a number, and never
  quote one you could not take.

## Report format

1. The census: four measurements, byte formulas, uncertainties.
2. The `Unique[]` budget table with its arithmetic.
3. Your verdict on whether the cap is workable.
4. What changed in your understanding of brief-505 §1.
5. Every command run, and the list of files you touched.

---

## Execution report — brief-506 (2026-09-22)

### 1. Census: method and measurements

This is a **throwaway measurement build**, not a production change or a
proposal to land diagnostics. Starting checkout: clean `main`,
`f7a4217c2f9798b9909c2b4b114474f9b5ba6b79`. No quota is implemented and no
default is selected here. Measurements below are appended in the requested
order, before starting the next workload.

The unit is director D2's **Rust allocation-layout proxy plus a fixed
reserve**, explicitly excluding allocator metadata, allocator rounding and
RSS. The reserve in this census is **4,096 bytes**, an explicit accounting
allowance for the original static interner/cache owners, their synchronization
state and the two static context strings. It is not a measured allocation.
Instrumentation memory, log files, sessions, parsed source, caller buffers,
executable mappings and the rest of the runtime are excluded.

For every process, let `T` be the sum of byte lengths of strings actually
allocated by `intern_text`, `P` the distinct `(context, name)` pair count,
and `cs`, `cp`, `cc` the actual public capacities of the string set, pair
map and canonical-symbol cache. Static `Global\`` and `System\`` are counted
as distinct strings but allocate no `str` payload. Every other distinct
string is charged once. No per-occurrence or per-session multiplication.

The term-by-term formula is:

```text
lower = T + P * size_of::<SymbolText>()
          + L(cs, size_of::<&str>())
          + L(cp, size_of::<((usize, usize), &SymbolText)>())
          + L(cc, size_of::<(&str, Symbol)>()) + 4096
upper = the same formula with U instead of L
L(c,s) = c*s
U(0,s) = 0
U(c,s) = next_power_of_two(c+1)*(s+1) + 31  (c > 0)
```

`L` is a downward bound on table backing storage, **not** a conservative
charge. `U` is an upward, **implementation-conditioned** bound, not a
portable property of `HashMap`. This toolchain's local Rust 1.85.0 sources
identify hashbrown 0.15.2. Its insertion-only capacity rule gives the bucket
count above; each bucket has one control byte. Its layout rounds
`buckets*stride` to control alignment and adds one SSE2 group of 16 bytes.
The extra 31 bytes allow up to 15 alignment bytes plus that group. The
observed strides/capacities make padding zero, so this deliberately adds 15
bytes per nonempty table above the source-derived layout. Public safe Rust
does not expose the actual backing allocation; these bounds do not pretend
to measure it. They would need revalidation for a different standard-library
implementation. Neither bound is a bound on RSS. The fixed reserve can
overcount static state; omitted allocator overhead can make physical usage
higher than the upper proxy.

The census also records an upward growth-overlap bound: final retained
`upper` plus the larger old backing store among tables that grew during that
insertion, maximized over the process. String and pair tables grow
sequentially. Charging final payload while an older table still exists can
overcount temporary overlap; this is intentional. This does not substitute
for Draft A's storage proof for its different proposed index representation.

To avoid an `unsafe` exit hook, instrumentation writes an unbuffered census
row under the interner lock whenever retained state changes. The bounded
launcher reads the last complete row **after process exit** and emits
`CENSUS_AT_EXIT`; child processes have separate PID files. This covers test
harness capture and explicit process exit without relying on destructors.
No interner deletion exists, so the last row is also the retained high-water
mark. Aborted or timed-out runs would be marked incomplete, not successful
exit measurements.

The ownership inventory reads production source locations. Therefore a
scratch `RUSTC_WRAPPER` compiles a copied `rum-core` with instrumentation
under `target/brief-506/core-src`, while the suite reads unchanged production
sources. No inventory fixture or test is altered. The instrumentation is
left in the working-tree `expr.rs` after all measurements, as requested.
This temporary source-copy arrangement is not proposed for production.

Timeout evidence: the recorded workspace run
`target/brief-498/logs/workspace.log` contains 24.83 seconds of compilation
and 72.15 seconds across test summaries (96.98 seconds, as recorded in
brief-500); the workspace bound is 900 seconds. The recorded release builds
in `target/brief-503/logs/{varc,veq}-build.log` took 28.50 and 24.45 seconds;
the release build bound is 300 seconds. Bounds are process-group bounds via
the unchanged `scripts/run-bounded.py`. Cargo is offline throughout;
temporary files and new build outputs are directed into `target/brief-506`.

#### Measurement 1 — bootstrap

Completed before starting measurement 2. A fresh standalone release helper
calls `Session::new()` and `rum_stdlib::install_core`, then reads catalog
metadata; **nothing is evaluated**. Exit 0, launcher elapsed 0.178826616 s.

| Quantity | Observed value |
|---|---:|
| Distinct strings | 662 |
| Distinct pairs | 661 |
| `System\`` pairs / other-context pairs | 661 / 0 |
| Heap `str` payload bytes | 6,148 |
| `SymbolText` bytes (`661 * 40`) | 26,440 |
| String set capacity; backing lower–upper | 896; 14,336–17,439 |
| Pair map capacity; backing lower–upper | 896; 21,504–25,631 |
| Canonical cache capacity; backing lower–upper | 112; 4,480–5,279 |
| Fixed reserve | 4,096 |
| **Retained proxy lower–upper** | **77,004–85,033 bytes** |
| Observed-path growth-overlap upper bound | 87,299 bytes |

The measured layouts are `Symbol=24`, `SymbolText=40`, original static
interner owner=112, canonical cache owner=56 bytes. Thus the reserve exceeds
the 168 owner bytes and the two seven-byte static contexts by 3,914 bytes.
It is deliberately upward. The table-bound uncertainty is the method above;
the string/pair counts and heap payload terms are exact for this execution.

There are **81 canonical-head pairs** and **643 installed builtin catalog
pairs**, with an overlap of **63**. Hence **580 additional pairs come from
the installed catalog outside `CANONICAL_CORE_HEADS`**, and the remaining
18 canonical heads are not catalog builtins: `81 + 643 - 63 = 661`. No other
pair is needed by this bootstrap. “Installed catalog” here includes the
runtime's `core_builtin_specs()` that `install_core` registers, as well as
the stdlib-owned family catalogs. This is a membership breakdown of the
actual installed set, not guessed provenance from insertion order. Default
stdlib features include `hash`.

Verbatim output (`target/brief-506/logs/bootstrap.log` and the census file):

```text
BOOTSTRAP no evaluation; canonical_heads=81 catalog=643 overlap=63 catalog_noncanonical=580
# symbol_size=24 symbol_text_size=40 fixed_state_size=112 canonical_owner_size=56 reserve=4096
662	661	6148	26440	896	896	112	14336	17439	21504	25631	4480	5279	4096	77004	85033	87299	"System`"	"ToYaml"
```

The TSV columns are, in order: strings, pairs, text bytes, pair bytes,
string/pair/canonical capacities, string lower/upper, pair lower/upper,
canonical lower/upper, reserve, total lower/upper, peak upper, context, name.
The complete bootstrap name list and independently computed membership
breakdown are in `target/brief-506/census/bootstrap/` and
`target/brief-506/bootstrap-breakdown.json`.

#### Measurement 2 — ordinary workspace suite

Completed and recorded before starting measurement 3. Exact requested
workload: `cargo test --workspace --no-fail-fast`, census enabled,
**`RUM_ALLOW_NATIVE_ZEROMQ_SKIP` absent**. Exit 101 after 159.713787271 s;
32 test-summary groups reported **2,465 passed, 9 failed, 57 ignored**.
All nine failures are the native ZeroMQ loopback permission probe. Those
native transport scenarios **did not execute**. This is not a green
workspace result. The ownership inventory test **passed**; there is no
ownership-inventory failure being excused as pre-existing.

The **maximum over individual test binaries**, not their sum, is
`rum_runtime-562b258c6eef9b02` (the `rum-runtime` unit-test binary):

| Quantity | Maximum binary's observed value |
|---|---:|
| Distinct strings / pairs | 92,267 / 153,237 |
| Heap `str` payload bytes | 935,258 |
| `SymbolText` bytes (`153237 * 40`) | 6,129,480 |
| String set capacity; backing lower–upper | 114,688; 1,835,008–2,228,255 |
| Pair map capacity; backing lower–upper | 229,376; 5,505,024–6,553,631 |
| Canonical cache capacity; backing lower–upper | 112; 4,480–5,279 |
| Fixed reserve | 4,096 |
| **Retained proxy lower–upper** | **14,413,346–15,855,999 bytes** |
| Observed-path growth-overlap upper bound | 17,366,680 bytes |

Formula: `935258 + 153237*40 + table(cs=114688,16) +
table(cp=229376,24) + table(cc=112,40) + 4096`, with the previously stated
lower/upper table bounds. Counts and allocated payload sizes are exact for
this run; the opaque backing-store and allocator qualifications remain.
The growth-overlap bound also depends on the concurrent test insertion
order and is not claimed to be identical for every scheduling order.

The runner captured 61 processes that actually interned, including test
children/doctests. None exceeded that test binary. The next largest was
`rum_stdlib-c8c5a89643371dc7`, with 19,739 strings, 36,319 pairs and a
3,582,558–3,943,867-byte retained proxy. Binaries that never intern create
no census file and cannot be the maximum.

Verbatim evidence (`target/brief-506/logs/workspace.log`; the last row for
each of the two largest processes):

```text
test tests::ownership_graph::production_inventory_is_exact ... ok
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
92267	153237	935258	6129480	114688	229376	112	1835008	2228255	5505024	6553631	4480	5279	4096	14413346	15855999	17366680	""	""
19739	36319	286214	1452760	28672	57344	112	458752	557087	1376256	1638431	4480	5279	4096	3582558	3943867	4457258	""	""
```

The lower bound alone is above 8 MiB (8,388,608 bytes). Thus **8 MiB does
not cover this ordinary test binary under the current storage layout**;
this conclusion does not depend on interpreting the upper bound as exact.
Full per-process results: `target/brief-506/census/workspace/exit-totals.json`.

#### Measurement 3 — unchanged release microbench panel

Completed and recorded before starting measurement 4. Ran the original
`scripts/core-language-microbench.sh` at its defaults (scale 1, seven
repetitions), using the instrumented release `rum-cli`. Exit 0; 120 labeled
rows and `RUM_CORE_MICRO_END` were emitted. The script's SHA-256 is unchanged:
`02b9e0b547400b7beac5285de0b87a39d488c98585f55e5639880fa90d4aa541`.

| Quantity | Observed value at process exit |
|---|---:|
| Distinct strings / pairs | 36,091 / 63,593 |
| Heap `str` payload bytes | 696,616 |
| `SymbolText` bytes (`63593 * 40`) | 2,543,720 |
| String set capacity; backing lower–upper | 57,344; 917,504–1,114,143 |
| Pair map capacity; backing lower–upper | 114,688; 2,752,512–3,276,831 |
| Canonical cache capacity; backing lower–upper | 112; 4,480–5,279 |
| Fixed reserve | 4,096 |
| **Retained proxy lower–upper** | **6,918,928–7,640,685 bytes** |
| Observed-path growth-overlap upper bound | 8,948,851 bytes |

Formula: `696616 + 63593*40 + table(57344,16) + table(114688,24) +
table(112,40) + 4096`, with exactly the same table uncertainty and physical
memory exclusions as measurements 1–2. Retained upper is below 8 MiB, but
the overlap-aware upper is above it; retained exit bytes alone cannot
establish that an 8 MiB preflight budget admits every growth transaction.

The width-4,000 and width-16,000 symbol fixtures ran as written; their
shared prefix means the first 4,000 names are reused in the larger fixture,
not an extra disjoint 4,000-name set. This panel is a substantial ordinary
workload, but the observed workspace maximum is larger.

The 180-second inner bound (300-second outer script bound) uses the recorded
same panel in `target/brief-497/candidate-microbench/runs.tsv`, which
completed in 1.143569 s. The instrumented run's launcher took 5.263558043 s.
This is timeout/completion evidence only: **no timing comparison or speed
claim is made from this throwaway build**.

Verbatim evidence:

```text
rum	ok	0	4.716491	target/brief-506/microbench/raw/rum.stdout	target/brief-506/microbench/raw/rum.stderr
RUM_CORE_MICRO_END
Null
36091	63593	696616	2543720	57344	114688	112	917504	1114143	2752512	3276831	4480	5279	4096	6918928	7640685	8948851	""	""
```

#### Measurement 4 — unbounded consumers, and the Module premise

Each row below comes from a **fresh release process and session** after
`install_core`. The helper evaluates one `.rum` input, asserts no termination
or messages and checks the final integer result. Its host iteration-work
limit is explicitly 1,000,000 for every size, permitting the trials above
the ordinary 100,000-iteration default. No memory limit or interner quota is
implemented or bypassed. All runs exited 0.

The two requested sources are:

```text
Do[Unique[], {censusI, 1, N}]; N
Do[Symbol[StringJoin["censusSymbol", ToString[censusI]]], {censusI, 1, N}]; N
```

The measured zero-call control for both is 664 strings, 663 pairs,
6,155 text bytes and a 77,091–85,120-byte proxy. Relative to bootstrap,
lowering the loop introduces `censusI` in two contexts and the static
`Global\`` string: **87 extra charged bytes** (7 text bytes + 2*40 pair
bytes), without growing the tables.

Define `D(N) = sum(decimal_digits(i), i=1..N)`. The exact observed payload
and count formulas are:

```text
Unique: strings=664+N; pairs=663+N; T=6155+N+D(N)
Symbol: strings=664+N; pairs=663+2N; T=6155+12N+D(N)
```

For every row, substitute these into the section-1 byte formula, using
the actual reached capacities recorded in its raw census. Equivalently
for these insertion-only trials, each index reaches capacity `7*b/8`,
where `b` is the smallest power of two for which `7*b/8` covers its count.
The canonical cache remains at 112; the fixed reserve remains 4,096.
`analyze.py` checks every observed count/payload against these formulas and
independently reconstructs both Unique byte bounds from the capacity rule.

The **two pairs per Symbol call are real**: name resolution constructs
candidate symbols in the context path before testing whether they exist,
so a fresh unqualified name creates both `System\`` and `Global\`` pairs.
See `crates/rum-runtime/src/runtime/resolution.rs:101-138`. The text payload
is shared once between them. A differently qualified name or context path
can have a different cost; this census is for the requested unqualified
workload with the concrete prefix shown above.

Bytes per call below mean `(exit bytes - same-source zero-call bytes)/N`.
They include actual table growth and exclude the one-time loop setup.
They are **finite-run averages, not a constant allocation rate**. Between
table growths, Unique adds exactly `40 + 1 + decimal_digits(i)` payload
bytes per new call; Symbol adds `80 + 12 + decimal_digits(i)`. Table
resizes cause discontinuities. Uncertainty is the same lower/upper backing
bound, not a statistical confidence interval or an RSS estimate.

| Consumer | Calls | Distinct strings | Distinct pairs | Retained proxy lower–upper (bytes) | Incremental bytes/call lower–upper |
|---|---:|---:|---:|---:|---:|
| Unique | 1,000 | 1,664 | 1,663 | 156,824–172,021 | 79.733–86.901 |
| Unique | 10,000 | 10,664 | 10,663 | 1,063,585–1,179,134 | 98.6494–109.4014 |
| Unique | 60,000 | 60,664 | 60,663 | 7,377,665–8,296,030 | 121.676233–136.848500 |
| Unique | 120,000 | 120,664 | 120,663 | 14,745,186–16,581,055 | 122.234125–137.466125 |
| Unique | 240,000 | 240,664 | 240,663 | 29,560,226–33,231,103 | 122.846396–138.108262 |
| Symbol | 1,000 | 1,664 | 2,663 | 250,832–274,221 | 173.741–189.101 |
| Symbol | 10,000 | 10,664 | 20,663 | 1,917,649–2,098,734 | 184.0558–201.3614 |
| Symbol | 60,000 | 60,664 | 120,663 | 13,190,177–14,632,830 | 218.551433–242.461833 |
| Symbol | 120,000 | 120,664 | 240,663 | 26,370,210–29,254,655 | 219.109325–243.079458 |
| Symbol | 240,000 | 240,664 | 480,663 | 52,810,274–58,578,303 | 219.721596–243.721596 |

**The claimed Module premise is false in the measured `main` checkout.**
`with_identity` itself does reuse the same interned pair, but this Module
implementation does not use it for local generation. The call chain is
`scoping/lexical/module.rs:46-50` → `allocate_fresh_symbol`
(`scoping/unique.rs:81-82`) → `allocate_generated_symbol` →
`generated_symbol_candidate` → **`Symbol::new` with formatted fresh text**
(`scoping/unique.rs:36-42`). The additional source was:

```text
Do[Module[{censusLocal}, censusLocal], {censusI, 1, N}]; N
```

| Module calls | Strings | Pairs | Retained lower–upper (bytes) |
|---:|---:|---:|---:|
| 0 | 665 | 665 | 77,182–85,211 |
| 1,000 | 1,665 | 1,665 | 167,915–183,112 |
| 10,000 | 10,665 | 10,665 | 1,173,676–1,289,225 |

Thus even discarded Module results add one string and one pair per
activation here. Its measured formula is `strings=pairs=665+N`,
`T=6166+12N+D(N)`, with the same table terms. Module GC does not reclaim
the leaked interner allocations. This refutes the stated premise with
both the current call chain and a direct rum-only measurement; it does
not reopen the ruled representation constraints.

Verbatim completion output and final census rows for the largest requested
consumer runs and the Module confirmation:

```text
CONSUMER source=target/brief-506/probes/unique-240000.rum calls=240000 complete=true messages=0 termination=None
CONSUMER source=target/brief-506/probes/symbol-240000.rum calls=240000 complete=true messages=0 termination=None
CONSUMER source=target/brief-506/probes/module-10000.rum calls=10000 complete=true messages=0 termination=None
240664	240663	1575050	9626520	458752	458752	112	7340032	8912927	11010048	13107231	4480	5279	4096	29560226	33231103	39254292	""	""
240664	480663	4215050	19226520	458752	917504	112	7340032	8912927	22020096	26214431	4480	5279	4096	52810274	58578303	70611944	""	""
10665	10665	165060	426600	14336	14336	112	229376	278559	344064	409631	4480	5279	4096	1173676	1289225	1298279	""	""
```

Timeout calibration is recorded, not inferred from an arbitrary larger
workload. The bootstrap helper completed in 0.178827 s; the initial
zero/1,000-call probes used a 60 s envelope, also exceeding the recorded
1.143569 s panel that exercises generation. Unique at 1,000/10,000 calls
took 0.277432/0.645992 s. Scaling the 10,000-call result by trial size and
10x headroom, rounded up, gave 60/120/180 s for 60,000/120,000/240,000.
Symbol at 1,000 calls took 0.191409 s; scaling that by size and 10x,
rounded up to a minute, gave 120/240/480 s for those larger sizes.
Observed largest-run completion times were 13.925734 s (Unique) and
27.525094 s (Symbol). These are timeout records, not speed measurements.

### 2. Unique budget table and arithmetic

**These are conditional budgets for the measured current layout, not
observed quota refusals.** No cap exists in this build. Draft A's explicitly
sized indexes will have different slot layouts, load factors and growth
temporaries; their actual refusal count cannot honestly be measured here.
The following gives the director the concrete budget/trade under this
census's upward charging rule. It assumes one fresh process/session,
default context path, no intervening symbol work, and fresh `$1`, `$2`, …
names. Existing interned text, collisions, context changes, other sessions
or longer prefixes change the available budget.

Take the measured bootstrap upper floor `B=85,033`. The actual loop has
`B0=B+87=85,120`. With `cs(N)=capacity(664+N)` and
`cp(N)=capacity(663+N)`, retained usage is:

```text
R(N) = 85120 + 40*N + N + D(N)
       + U(cs(N),16) - U(896,16)
       + U(cp(N),24) - U(896,24)
```

For example, at 8 MiB the bootstrap headroom is
`8,388,608 - 85,033 = 8,303,575` bytes; loop setup leaves `8,303,488`.
This is not divided by one average slope. The capacity jumps and decimal
name lengths are included exactly in the calculation. The analysis
reconstructs every sampled Unique exit row from this formula.

`R(62,012)=8,388,582 <= 8,388,608`, whereas
`R(62,013)=8,388,628 > 8,388,608`. That is the retained-only threshold.
For admission that also charges growth overlap, use the recorded running
peak bound instead. After 56,681 calls it is 7,619,099 bytes; the next
call would require the upward allowance
`8,143,402 + 1,638,431 = 9,781,833` bytes (new retained upper + old pair
table upper), above 8 MiB. Its retained state alone still fits.

| Candidate total cap | Headroom after bootstrap upper | Calls under overlap-aware upper charge | First call exceeding that charge | Retained-only upper-charge budget |
|---|---:|---:|---:|---:|
| Bootstrap upper floor: 85,033 B | 0 B | 0 | First new Unique name; loop setup itself also exceeds this floor | 0 |
| 8 MiB = 8,388,608 B | 8,303,575 B | **56,681** | 56,682 | 62,012 |
| 16 MiB = 16,777,216 B | 16,692,183 B | **114,025** | 114,026 | 124,173 |
| 32 MiB = 33,554,432 B | 33,469,399 B | **228,713** | 228,714 | 246,879 |

The 16/32 MiB overlap budgets have the same arithmetic: the following
pair-table growth takes the allowances to
`16,300,277 + 3,276,831 = 19,577,108` and
`32,700,661 + 6,553,631 = 39,254,292` bytes, respectively.
All three overlap transitions are present in the **measured per-insertion
log** of the 240,000-call run. The 32 MiB retained-only threshold is an
algebraic projection from the validated formula, beyond that run's endpoint,
and is **not** quoted as an executed sample. These budgets retain the
actual measured loop's setup costs rather than postulating a different
host-side call sequence.

Verbatim output of the budget calculation (`budget-summary.tsv`):

```text
cap_bytes	headroom_after_bootstrap	overlap_budget_calls	peak_upper_through_budget	next_call_peak_upper	retained_only_calls	retained_at_budget	retained_at_next_call
8388608	8303575	56681	7619099	9781833	62012	8388582	8388628
16777216	16692183	114025	15251685	19577108	124173	16777186	16777233
33554432	33469399	228713	30603493	39254292	246879	33554416	33554463
```

The conservative reserve/backing bound reduces these conditional budgets;
it is not a claim of exact underlying allocator behavior. The upward bound
adds 15 spare bytes per nonempty table; growth additionally overcounts at
most the final pair/payload not yet published when the old table exists.
The much larger limitation is the as-yet-unbuilt replacement index layout,
not that small allowance. A fixed cap gives no unlimited lifetime budget:
the loop's byte demand is unbounded.

### 3. Verdict: is the process-wide cap workable?

**Yes, as an explicit finite process resource limit with recoverable
refusal. The numbers refute 8 MiB as a ceiling covering this ordinary
workspace suite under the measured layout.** The bootstrap is small;
ordinary accumulated process vocabulary is not. The runtime test binary's
lower proxy is already 14,413,346 bytes. A 16 MiB candidate clears its
retained upper but does not clear the 17,366,680-byte conservative growth
allowance. A 32 MiB envelope clears the bootstrap, recorded suite maximum
and panel allowances in this census. These are comparisons, **not a
selected or proposed default**.

No finite process cap can transparently preserve arbitrarily many supported
Unique or Module activations, arbitrary dynamically generated names, or an
unbounded succession of distinct programs. The 240,000-call trials make
that concrete. If “workable” means bounding permanently retained process
storage while reporting exhaustion, the premise survives. If it means
guaranteeing ordinary language constructs never exhaust a long-lived shared
process, that requirement is refuted. Reset cannot restore the used budget,
and the proxy is not a physical-memory ceiling. This census does not claim
otherwise and does not close the non-reclamation finding.

### 4. What changes in the understanding of brief-505 §1

1. **The Module exemption is unavailable on current main.** Its actual
   generator formats new text. Fallible arbitrary-name construction and
   refusal propagation through Module remain relevant; a claim that only
   Unique is the language-level unbounded generator is unsupported here.
   `with_identity` reuse is true as an API property, not evidence about this
   Module call path.
2. **Bootstrap closure is much larger than the canonical head list:**
   661 actual pairs, with 580 installed noncanonical catalog names. This
   sizes the observed bootstrap portion of the reserved vocabulary only;
   it does not prove the full finite error/options/lazy-construction closure
   is exhausted by bootstrap.
3. **Lookup is an insertion ingress.** Default unqualified Symbol resolution
   charges two pairs per fresh name. The inventory/refusal plan must account
   for candidate construction in every probed context, including candidates
   for which `symbol_exists` returns false. Per-call budgets that assume
   every textual symbol has one pair are wrong for this measured ingress.
4. **The 8 MiB subtraction in Draft A is not calibration.** The measured
   ordinary-suite maximum is larger; even the panel's retained-vs-growth
   difference matters. These capacities/payloads are input to Draft A's
   storage proof. Copying a present-container bytes-per-call average into
   the replacement index design would not establish its budget.

No representation change, reclamation, per-session attribution, default
choice, diagnostic landing or alternative implementation is proposed here.



### 5. Commands, touched files and handoff checks

The two changed tracked files are:

- `crates/rum-core/src/expr.rs`: **throwaway instrumentation**, byte-identical
  to the copied source used for every measurement. It preserves Symbol's
  fields, size, Copy implementation and public signatures. It is intentionally
  unsuitable for landing or speed testing and must be discarded by the
  director. No other production source was changed.
- `docs/dev/codex/brief-506-interner-census.md`: this appended report.

Authored scratch files are `target/brief-506/setup.py`, `instrumentation.rs`,
`rustc-wrapper.py`, `run.py`, `make-consumers.py`, `analyze.py`, `finalize.py`,
`probes/bootstrap.rs`, `probes/consumer.rs`, generated `probes/*.rum`, and
the command/evidence manifests. `core-src/**` is the copied rum-core source,
including the measured instrumented `expr.rs`; `expr.original.rs` is the
unmodified starting copy. Builds are under `build/**`; executable probes,
raw census rows, exit summaries, analysis tables, logs, microbench output and
temporary compiler/test files remain under `target/brief-506/**`.

The mandatory workspace test also invokes the **unchanged**
`scripts/check-host-payload-destruction.py`, which writes its generated
consumer manifests, locks, diagnostics and builds to its fixed default
`target/host-payload-destruction-contract/**`. These are incidental outputs
of the requested suite, inside the repository; the harness and fixtures were
not edited. The complete artifact-tree file listing, including that existing
test-output tree, is `target/brief-506/artifact-files.txt`. No claim is made
that every pre-existing artifact in that tree was newly created.

`rustfmt --edition 2024 --check crates/rum-core/src/expr.rs` passed after
the measured source was left in place. `git diff --check` also passed;
the original brief prefix still matches its starting SHA-256. All 15
consumer runs completed successfully, and the only nonzero measured-workload
exit is the already explained workspace result. The instrumented file is 33,194 bytes:
no organization refactor is appropriate for code the director discards;
the real interner/module separation belongs to the already specified
implementation work. The production ownership inventory was tested against
the original source, with the instrumented copy supplying compiled behavior.
Re-running that source-location inventory against the deliberately left
throwaway additions would see those additions; this is not a claim that
the final throwaway source is an updated production inventory baseline.

Protected-file hashes were checked for all 449 recorded files (all scripts,
docs, ownership inventory fixtures and the named root files). **Only this
allowed report changed.** The panel, other scripts/docs, AGENTS, TODO,
PROGRESS, workspace Cargo.toml and Cargo.lock remain byte-identical. No
commit, branch creation/switch, merge, push, stash, checkout/restore, index
or git-configuration write was performed. Git usage was read-only. No
network or new unsafe code was used. External toolchain/cache sources were
read only; no file outside the repository was intentionally created or
edited. No performance A/B was run: the brief explicitly says this build is
never measured for speed. No black-box vendor comparison was performed.

The suite's single invocation completed with the nine environmental failures
already reported; it was not retried with a skip variable. The only failed
analysis attempt was a bootstrap-name parser treating CSV-unquoted strings
as Python literals; correcting that parser produced the complete breakdown.
Several initial read-only searches used nonexistent guessed paths; all are
retained in the command transcript. No census measurement failed twice, so
the stop-loss did not trigger.

The local compiler identified itself verbatim as:

```text
rustc 1.85.0 (4d91de4e4 2025-02-17)
host: x86_64-unknown-linux-gnu
release: 1.85.0
LLVM version: 19.1.7
```

Its cfg output includes `target_feature="sse2"` and
`target_pointer_width="64"`. The backing-storage inference uses the local
toolchain's `library/Cargo.lock` (`hashbrown` 0.15.2), `std` hash map/set
delegation, and cached hashbrown `src/raw/mod.rs` capacity/layout code and
`src/control/group/{mod,sse2}.rs`; the exact read paths are below.

Every agent-issued shell command is reproduced below, including failed
read-only searches and the final manifest command. Structured `apply_patch`
edits created/updated only the authored scratch files and this report;
`setup.py`/the explicitly logged copy command created the measured source
copy and final instrumentation. Tool waits merely collected process output.
The bounded launcher's exact subprocess argv/environment records are in
`target/brief-506/commands.jsonl`, with exit codes and monotonic durations
in `logs/*.result.json`. Cargo/compiler subprocesses and the unchanged
scripts' internally generated commands are retained in their corresponding
raw logs rather than represented as separately issued agent shell commands.

<details>
<summary>Complete agent-issued shell command transcript</summary>

Command 1:

```bash
pwd && git status --short && rg --files -g AGENTS.md -g '!target' -g '!vendor'
```

Command 2:

```bash
cat docs/dev/codex/brief-506-interner-census.md && cat AGENTS.md
```

Command 3:

```bash
git branch --show-current && git rev-parse HEAD && git status --porcelain=v1 && rg --files docs/dev/codex target -g '*500*' -g '*505*' -g '*506*' -g '*timing*' -g '*summary*' -g '*command*' -g '*microbench*' -g '*gate*'
```

Command 4:

```bash
cat docs/dev/codex/brief-505-interner-bounded-or-disproof.md
```

Command 5:

```bash
sed -n '1,260p' crates/rum-core/src/expr.rs && sed -n '1,260p' crates/rum-core/src/heads.rs && cat crates/rum-runtime/src/runtime/scoping/unique.rs
```

Command 6:

```bash
cat README.md && cat docs/testing-strategy.md && cat docs/code-organization-guidelines.md && sed -n '1,220p' scripts/core-language-microbench.sh
```

Command 7:

```bash
cat docs/dev/codex/brief-505-interner-quota-design.md
```

Command 8:

```bash
sed -n '1,235p' crates/rum-core/src/expr.rs && sed -n '95,180p' crates/rum-core/src/heads.rs && cat crates/rum-runtime/src/runtime/scoping/unique.rs
```

Command 9:

```bash
rg -n '97|900|elapsed|timeout|cargo test|microbench|release' docs/dev/codex/brief-500-non-copy-migration-to-green.md target/brief-500 target/brief-504 target/brief-497/base-microbench.log target/brief-497/candidate-microbench.log && cat target/brief-497/candidate-microbench/runs.tsv && cat rust-toolchain.toml && sed -n '1,170p' crates/rum-stdlib/src/lib.rs && cat crates/rum-core/Cargo.toml
```

Command 10:

```bash
sed -n '1,200p' ARCHITECTURE.md && rg -n 'interner ruling|D1|D2|2026-09-2[12]|interner' TODO.md && rg -n 'install_core|BuiltinSpec|catalog|all_specs' crates/rum-stdlib/src/catalog* crates/rum-runtime/src/lib.rs && sed -n '1,145p' scripts/run-bounded.py
```

Command 11:

```bash
rg -n 'D1|D2|fixed reserve|8 MiB|Draft A|2026-09-22' docs/dev/codex/brief-505-interner-quota-design.md && sed -n '250,360p' docs/dev/codex/brief-505-interner-quota-design.md && sed -n '1190,1240p' TODO.md && cat rust-toolchain.toml && cat target/brief-497/candidate-microbench/runs.tsv
```

Command 12:

```bash
sed -n '1,170p' crates/rum-stdlib/src/lib.rs && sed -n '1,120p' crates/rum-stdlib/src/catalog.rs && cat crates/rum-core/Cargo.toml && rg -n 'thread_local|impl Drop|exit|stderr' crates/rum-core/src/profiling.rs crates/rum-runtime/src/runtime/profiling/mod.rs && rg -n 'pub fn (new|eval|builtin)|builtin_registry|pub struct Builtin' crates/rum-runtime/src/session.rs crates/rum-runtime/src/runtime.rs crates/rum-stdlib/src/catalog.rs
```

Command 13:

```bash
rg -n 'Finished|test result:' target/brief-498/logs/workspace.log && rg -n 'elapsed|wall|Finished|timeout' target/brief-503/logs/build-commands.jsonl target/brief-503/logs/measurement-commands.jsonl target/brief-502/logs/measurement-commands.jsonl && sed -n '870,910p' scripts/core-language-microbench.sh && sed -n '1060,1170p' scripts/core-language-microbench.sh
```

Command 14:

```bash
cat crates/rum-core/Cargo.toml && sed -n '1,140p' crates/rum-core/src/profiling.rs && sed -n '1,110p' crates/rum-stdlib/src/docgen.rs && sed -n '1,110p' crates/rum-stdlib/src/catalog/mod.rs && rg -n 'builtin|pub fn|pub struct' crates/rum-runtime/src/runtime/session/session_api.rs | head -60
```

Command 15:

```bash
sed -n '870,910p' scripts/core-language-microbench.sh && tail -140 scripts/core-language-microbench.sh && head -5 target/brief-503/logs/build-commands.jsonl && head -5 target/brief-503/logs/measurement-commands.jsonl && rg --files /home/remi/.rustup/toolchains -g 'raw/mod.rs' -g 'mod.rs.html' -g rustc -g rustfmt -g cargo | head -35
```

Command 16:

```bash
rg -n 'pub fn (new|eval)|next_generated_symbol_id|with_identity' crates/rum-runtime/src/runtime/session/session_state.rs crates/rum-runtime/src/runtime/session/session_api.rs crates/rum-runtime/src/runtime/scoping/lexical/module.rs && sed -n '1,100p' crates/rum-runtime/src/lib.rs && cat Cargo.toml && cat crates/rum-stdlib/Cargo.toml && sed -n '1,105p' ARCHITECTURE.md
```

Command 17:

```bash
rg --files /home/remi/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu /home/remi/.cargo/registry/src -g '*.rs' -g '*.html' | rg '/(hashbrown[^/]*|raw|table|alloc)/' | head -50
```

Command 18:

```bash
rg -n 'eval_str|evaluate_source|pub fn eval' crates/rum-runtime/src/runtime/session crates/rum-runtime/src/runtime/request* crates/rum-runtime/src/runtime/mod.rs && head -60 crates/rum-stdlib/src/tests.rs && sed -n '1436,1463p' docs/dev/codex/brief-505-interner-quota-design.md && rg -n 'Finished|elapsed' target/brief-503/logs/*build* target/brief-497/*build* target/brief-502/logs/*build* && sed -n '1,110p' crates/rum-jupyter/src/zeromq_native/tests.rs
```

Command 19:

```bash
rg --files /home/remi/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust /home/remi/.cargo/registry/src | rg 'hashbrown[^/]*/(src/raw/mod.rs|src/control/group/|Cargo.toml)$|hashbrown[^/]*/src/raw/mod.rs|library/std/Cargo.toml' && rg -n -A5 'name = "hashbrown"' /home/remi/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/Cargo.lock && rg -n 'Finished|elapsed' target/brief-497/*build* target/brief-503/logs/*build*
```

Command 20:

```bash
sed -n '195,245p' crates/rum-runtime/src/runtime/session/session_eval.rs && head -45 crates/rum-stdlib/src/tests.rs && rg -n 'Finished|elapsed|timeout' docs/dev/codex/brief-503* | head -25 && rg -n 'with_identity' crates/rum-runtime/src/runtime/scoping && sed -n '55,82p' crates/rum-runtime/src/runtime/session/session_api.rs && rg -n 'pub struct EvalOutcome|pub struct BuiltinSpec' crates/rum-runtime/src/runtime -g '*.rs'
```

Command 21:

```bash
sed -n '90,210p' /home/remi/.cargo/registry/src/mirrors.ustc.edu.cn-38d0e5eb5da2abae/hashbrown-0.15.2/src/raw/mod.rs && rg -n -A12 'pub fn capacity|fn capacity|growth_left|fn new' /home/remi/.cargo/registry/src/mirrors.ustc.edu.cn-38d0e5eb5da2abae/hashbrown-0.15.2/src/raw/mod.rs | head -95 && cat /home/remi/.cargo/registry/src/mirrors.ustc.edu.cn-38d0e5eb5da2abae/hashbrown-0.15.2/src/control/group/mod.rs && sed -n '20,50p' /home/remi/.cargo/registry/src/mirrors.ustc.edu.cn-38d0e5eb5da2abae/hashbrown-0.15.2/src/control/group/sse2.rs
```

Command 22:

```bash
sed -n '85,110p' crates/rum-runtime/src/runtime/model/requests.rs && sed -n '377,403p' crates/rum-runtime/src/runtime/metadata.rs && sed -n '407,438p' crates/rum-runtime/src/runtime/scoping/packages.rs && wc -c crates/rum-core/src/expr.rs && head -35 crates/rum-core/src/lib.rs && cat crates/rum-stdlib/src/tests/ownership_graph/mod.rs | head -75
```

Command 23:

```bash
cat crates/rum-stdlib/src/tests/ownership_graph/source.rs | head -150 && rg -n 'expr.rs|rum-core|rum_core' crates/rum-stdlib/src/tests/ownership_graph/source.rs && rg -n 'Module|allocate_module|with_identity' crates/rum-runtime/src/runtime/scoping/lexical/module.rs crates/rum-runtime/src/runtime -g '*gensym*'
```

Command 24:

```bash
rg -n 'pub fn capacity|self.table.growth_left|pub fn allocation' /home/remi/.cargo/registry/src/mirrors.ustc.edu.cn-38d0e5eb5da2abae/hashbrown-0.15.2/src/raw/mod.rs && rg -n 'hashbrown' /home/remi/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/map.rs /home/remi/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/set.rs /home/remi/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/Cargo.toml
```

Command 25:

```bash
python3 target/brief-506/setup.py && chmod +x target/brief-506/rustc-wrapper.py && scripts/run-bounded.py --timeout 120 -- rustfmt --edition 2024 target/brief-506/core-src/expr.rs target/brief-506/probes/bootstrap.rs
```

Command 26:

```bash
python3 target/brief-506/run.py --label release-build --timeout 300 -- cargo build --locked --release -p rum-stdlib -p rum-cli
```

Command 27:

```bash
tail -12 target/brief-506/logs/release-build.log && rg -n 'with_identity|fn allocate_module|identity' crates/rum-runtime/src/runtime/module_lifecycle.rs crates/rum-runtime/src/runtime/scoping/lexical/module.rs && sed -n '1,100p' crates/rum-runtime/src/runtime/module_lifecycle.rs
```

Command 28:

```bash
rg --files target/brief-506/build/release/deps -g 'librum_core-*.rlib' -g 'librum_runtime-*.rlib' && rg -n 'with_identity' crates/rum-runtime/src/runtime -g '*.rs' | head -15 && rg -n 'pub fn empty|fn empty|impl Default' crates/rum-runtime/src/runtime/session/session_state.rs
```

Command 29:

```bash
python3 target/brief-506/run.py --label bootstrap-build --timeout 300 -- rustc --edition 2024 -O target/brief-506/probes/bootstrap.rs --extern rum_stdlib=target/brief-506/build/release/librum_stdlib.rlib --extern rum_runtime=target/brief-506/build/release/deps/librum_runtime-c71407e195689fc9.rlib --extern rum_core=target/brief-506/build/release/deps/librum_core-04527ae728ff9c02.rlib -L dependency=target/brief-506/build/release/deps -o target/brief-506/probes/bootstrap
```

Command 30:

```bash
python3 target/brief-506/run.py --label bootstrap --timeout 60 --census --names -- target/brief-506/probes/bootstrap
```

Command 31:

```bash
python3 - <<'PY'
import ast, collections, csv, json
from pathlib import Path
base=Path('target/brief-506')
lines=(base/'logs/bootstrap.log').read_text().splitlines()
print(lines[0])
heads={s.split('\t',1)[1] for s in lines if s.startswith('CANONICAL\t')}
catalog={s.split('\t',1)[1] for s in lines if s.startswith('CATALOG\t')}
path=next((base/'census/bootstrap').glob('*.tsv'))
with path.open() as stream:
    print(next(stream).strip()); print(next(stream).strip())
    rows=list(csv.DictReader(stream,delimiter='\t'))
pairs={(ast.literal_eval(row['context']),ast.literal_eval(row['name'])) for row in rows if row['context']!='""'}
summary={'contexts':dict(collections.Counter(context for context,name in pairs)), 'pairs':len(pairs), 'canonical_pairs':sum(context=='System`' and name in heads for context,name in pairs), 'catalog_pairs':sum(context=='System`' and name in catalog for context,name in pairs), 'catalog_noncanonical_pairs':sum(context=='System`' and name in catalog-heads for context,name in pairs), 'other_pairs':sorted((c,n) for c,n in pairs if c!='System`' or n not in heads|catalog)}
text=json.dumps(summary,indent=2)
(base/'bootstrap-breakdown.json').write_text(text+'\n')
print(text)
print('EXIT_ROW_VERBATIM')
print(path.read_text().splitlines()[-1])
PY
```

Command 32:

```bash
python3 - <<'PY'
import collections, csv, json
from pathlib import Path
base=Path('target/brief-506')
lines=(base/'logs/bootstrap.log').read_text().splitlines()
print(lines[0])
heads={s.split('\t',1)[1] for s in lines if s.startswith('CANONICAL\t')}
catalog={s.split('\t',1)[1] for s in lines if s.startswith('CATALOG\t')}
path=next((base/'census/bootstrap').glob('*.tsv'))
with path.open() as stream:
    print(next(stream).strip()); print(next(stream).strip())
    rows=list(csv.DictReader(stream,delimiter='\t'))
pairs={(row['context'],row['name']) for row in rows if row['context']}
summary={'contexts':dict(collections.Counter(context for context,name in pairs)), 'pairs':len(pairs), 'canonical_pairs':sum(context=='System`' and name in heads for context,name in pairs), 'catalog_pairs':sum(context=='System`' and name in catalog for context,name in pairs), 'catalog_noncanonical_pairs':sum(context=='System`' and name in catalog-heads for context,name in pairs), 'other_pairs':sorted((c,n) for c,n in pairs if c!='System`' or n not in heads|catalog)}
text=json.dumps(summary,indent=2)
(base/'bootstrap-breakdown.json').write_text(text+'\n')
print(text)
print('EXIT_ROW_VERBATIM')
print(path.read_text().splitlines()[-1])
PY
```

Command 33:

```bash
python3 target/brief-506/run.py --label workspace --timeout 900 --census -- cargo test --workspace --no-fail-fast
```

Command 34:

```bash
tail -15 target/brief-506/logs/workspace.log && rg -n 'iteration_work|generated_element|memory_limit' crates/rum-runtime/src/runtime/config.rs crates/rum-runtime/src/runtime/session/session_state.rs && sed -n '657,680p' crates/rum-runtime/src/runtime/substitution/binding_substitution.rs
```

Command 35:

```bash
sed -n '193,216p' crates/rum-runtime/src/runtime/model/requests.rs && rg -n 'fn default|iteration_work|generated_element' crates/rum-runtime/src/runtime/session/session_config.rs crates/rum-runtime/src/runtime/resource_limits.rs && rg -n 'RUM_ALLOW_NATIVE_ZEROMQ_SKIP|ALLOW_NATIVE|native_loopback' crates/rum-jupyter/src/zeromq_native -g '*.rs' | head -25
```

Command 36:

```bash
rg -n 'iteration_work_limit:|generated_element_limit:|with_identity\(' crates/rum-runtime/src/runtime/session crates/rum-runtime/src/runtime/model crates/rum-runtime/src/runtime/substitution/binding_substitution.rs && rg -n 'RUM_ALLOW_NATIVE_ZEROMQ_SKIP' crates/rum-jupyter && sed -n '657,680p' crates/rum-runtime/src/runtime/substitution/binding_substitution.rs && rg -n 'Finished|test result:|production_inventory_is_exact' target/brief-506/logs/workspace.log | tail -20
```

Command 37:

```bash
python3 target/brief-506/make-consumers.py && sed -n '1,78p' crates/rum-runtime/src/runtime/scoping/lexical/module.rs && rg -n 'fresh.*symbol|generated.*symbol|with_identity|identity' crates/rum-runtime/src/runtime/module_gc* crates/rum-runtime/src/runtime/resolution.rs && cat crates/rum-runtime/src/runtime/model/config.rs | head -45 && cat crates/rum-jupyter/src/zeromq_native_test_support.rs | head -100
```

Command 38:

```bash
tail -12 target/brief-506/logs/workspace.log && rg -n '^error:|FAILED|production_inventory_is_exact' target/brief-506/logs/workspace.log | tail -15
```

Command 39:

```bash
cat crates/rum-runtime/src/runtime/model/config.rs | head -45 && rg -n 'DEFAULT_ITERATION_WORK_LIMIT|DEFAULT_GENERATED_ELEMENT_LIMIT' crates/rum-runtime/src/runtime && cat crates/rum-jupyter/src/zeromq_native_test_support.rs | head -100 && sed -n '100,166p' crates/rum-runtime/src/runtime/resolution.rs
```

Command 40:

```bash
tail -18 target/brief-506/logs/workspace.log && rg -n '^error:|FAILED|production_inventory_is_exact' target/brief-506/logs/workspace.log | tail -20
```

Command 41:

```bash
sed -n '1,210p' crates/rum-host/tests/host_payload_destruction_contract.rs && rg -n 'RUM_ALLOW_NATIVE_ZEROMQ_SKIP|loopback' crates/rum-jupyter/src/zeromq_native_test_support.rs | tail -15 && rg -n 'fn .*fresh|allocate_fresh_symbol' crates/rum-runtime/src/runtime/scoping/unique.rs && tail -5 target/brief-506/logs/workspace.log
```

Command 42:

```bash
sed -n '1,145p' scripts/check-host-payload-destruction.py && rg -n 'target|Temporary|temp|env|CARGO_TARGET' scripts/check-host-payload-destruction.py
```

Command 43:

```bash
tail -10 target/brief-506/logs/workspace.log && rg -n '^error:|FAILED|production_inventory_is_exact' target/brief-506/logs/workspace.log | tail -20 && rg --files target/host-payload-destruction-contract -g cargo.stderr.txt | wc -l
```

Command 44:

```bash
python3 - <<'PY'
from pathlib import Path
import json,re
base=Path('target/brief-506')
text=(base/'logs/workspace.log').read_text()
results=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',text)
print('SUITE_SUMMARY', json.dumps(dict(groups=len(results), passed=sum(int(x[0]) for x in results), failed=sum(int(x[1]) for x in results), ignored=sum(int(x[2]) for x in results))))
for line in text.splitlines():
    if 'production_inventory_is_exact' in line or 'cannot bind loopback:' in line:
        print(line)
for record in json.loads((base/'census/workspace/exit-totals.json').read_text())[:2]:
    print('BINARY',record['executable'])
    print('EXIT_ROW_VERBATIM', (base/'census/workspace'/f"{record['pid']}.tsv").read_text().splitlines()[-1], sep='\n')
print('RECORDED_MICROBENCH',Path('target/brief-497/candidate-microbench/runs.tsv').read_text(),sep='\n')
PY
```

Command 45:

```bash
python3 target/brief-506/run.py --label microbench --timeout 300 --census -- scripts/core-language-microbench.sh --output-dir target/brief-506/microbench --timeout 180 --rum-cli target/brief-506/build/release/rum-cli
```

Command 46:

```bash
cat target/brief-506/microbench/runs.tsv && tail -3 target/brief-506/microbench/raw/rum.stdout && tail -1 target/brief-506/census/microbench/22.tsv && wc -l target/brief-506/microbench/timings.tsv && python3 - <<'PY'
from pathlib import Path
import hashlib,json
before=json.loads(Path('target/brief-506/protected-before.json').read_text())
p=Path('scripts/core-language-microbench.sh')
current=hashlib.sha256(p.read_bytes()).hexdigest()
print('PANEL_SHA256',current,'UNCHANGED',current==before[str(p)])
PY
```

Command 47:

```bash
python3 target/brief-506/run.py --label consumer-build --timeout 60 -- rustc --edition 2024 -O target/brief-506/probes/consumer.rs --extern rum_stdlib=target/brief-506/build/release/librum_stdlib.rlib --extern rum_runtime=target/brief-506/build/release/deps/librum_runtime-c71407e195689fc9.rlib --extern rum_core=target/brief-506/build/release/deps/librum_core-04527ae728ff9c02.rlib -L dependency=target/brief-506/build/release/deps -o target/brief-506/probes/consumer
```

Command 48:

```bash
python3 target/brief-506/run.py --label unique-0 --timeout 60 --census -- target/brief-506/probes/consumer target/brief-506/probes/unique-0.rum 0
```

Command 49:

```bash
python3 target/brief-506/run.py --label unique-1000 --timeout 60 --census -- target/brief-506/probes/consumer target/brief-506/probes/unique-1000.rum 1000
```

Command 50:

```bash
python3 target/brief-506/run.py --label unique-10000 --timeout 60 --census -- target/brief-506/probes/consumer target/brief-506/probes/unique-10000.rum 10000
```

Command 51:

```bash
python3 target/brief-506/run.py --label symbol-0 --timeout 60 --census -- target/brief-506/probes/consumer target/brief-506/probes/symbol-0.rum 0
```

Command 52:

```bash
python3 target/brief-506/run.py --label symbol-1000 --timeout 60 --census -- target/brief-506/probes/consumer target/brief-506/probes/symbol-1000.rum 1000
```

Command 53:

```bash
python3 target/brief-506/run.py --label module-0 --timeout 60 --census -- target/brief-506/probes/consumer target/brief-506/probes/module-0.rum 0
```

Command 54:

```bash
python3 target/brief-506/run.py --label module-1000 --timeout 60 --census -- target/brief-506/probes/consumer target/brief-506/probes/module-1000.rum 1000
```

Command 55:

```bash
python3 target/brief-506/run.py --label unique-60000 --timeout 60 --census -- target/brief-506/probes/consumer target/brief-506/probes/unique-60000.rum 60000
```

Command 56:

```bash
python3 target/brief-506/run.py --label unique-120000 --timeout 120 --census -- target/brief-506/probes/consumer target/brief-506/probes/unique-120000.rum 120000
```

Command 57:

```bash
python3 target/brief-506/run.py --label unique-240000 --timeout 180 --census -- target/brief-506/probes/consumer target/brief-506/probes/unique-240000.rum 240000
```

Command 58:

```bash
python3 target/brief-506/run.py --label symbol-10000 --timeout 60 --census -- target/brief-506/probes/consumer target/brief-506/probes/symbol-10000.rum 10000
```

Command 59:

```bash
python3 target/brief-506/run.py --label symbol-60000 --timeout 120 --census -- target/brief-506/probes/consumer target/brief-506/probes/symbol-60000.rum 60000
```

Command 60:

```bash
python3 target/brief-506/run.py --label symbol-120000 --timeout 240 --census -- target/brief-506/probes/consumer target/brief-506/probes/symbol-120000.rum 120000
```

Command 61:

```bash
python3 target/brief-506/run.py --label symbol-240000 --timeout 480 --census -- target/brief-506/probes/consumer target/brief-506/probes/symbol-240000.rum 240000
```

Command 62:

```bash
python3 target/brief-506/run.py --label module-10000 --timeout 60 --census -- target/brief-506/probes/consumer target/brief-506/probes/module-10000.rum 10000
```

Command 63:

```bash
scripts/run-bounded.py --timeout 60 -- python3 target/brief-506/analyze.py > target/brief-506/analysis.txt
```

Command 64:

```bash
cat target/brief-506/analysis.txt && cat target/brief-506/logs/module-10000.log && cat target/brief-506/logs/unique-240000.log && cat target/brief-506/logs/symbol-240000.log
```

Command 65:

```bash
python3 - <<'PY'
import json
from pathlib import Path
base=Path('target/brief-506')
lines=['cap_bytes\theadroom_after_bootstrap\toverlap_budget_calls\tpeak_upper_through_budget\tnext_call_peak_upper\tretained_only_calls\tretained_at_budget\tretained_at_next_call']
for r in json.loads((base/'budgets.json').read_text()):
    lines.append('\t'.join(str(x) for x in [r['cap_bytes'],r['bootstrap_headroom'],r['overlap_upper_calls'],r['accepted']['peak_upper'],r['first_over_budget']['peak_upper'],r['retained_upper_calls'],r['retained_upper_at_n'],r['retained_upper_at_next']]))
result='\n'.join(lines)+'\n'
(base/'budget-summary.tsv').write_text(result)
print(result,end='')
for mode in ('unique','symbol','module'):
    n=10000 if mode=='module' else 240000
    path=next((base/'census'/f'{mode}-{n}').glob('*.tsv'))
    print('EXIT_ROW',mode,n)
    print(path.read_text().splitlines()[-1])
print('Module call-chain evidence:')
PY
sed -n '42,55p' crates/rum-runtime/src/runtime/scoping/lexical/module.rs
sed -n '36,43p' crates/rum-runtime/src/runtime/scoping/unique.rs
sed -n '80,90p' crates/rum-runtime/src/runtime/scoping/unique.rs
```

Command 66:

```bash
python3 - <<'PY'
from pathlib import Path
base=Path('target/brief-506')
path=Path('crates/rum-core/src/expr.rs')
assert path.read_bytes() == (base/'expr.original.rs').read_bytes(), 'production expr.rs changed unexpectedly; refusing to overwrite'
path.write_bytes((base/'core-src/expr.rs').read_bytes())
print('Left the measured throwaway instrumentation in crates/rum-core/src/expr.rs.')
print('expr.rs bytes:',path.stat().st_size)
PY
scripts/run-bounded.py --timeout 60 -- rustfmt --edition 2024 --check crates/rum-core/src/expr.rs
git diff --stat
git status --short
```

Command 67:

```bash
scripts/run-bounded.py --timeout 60 -- rustc --version --verbose && scripts/run-bounded.py --timeout 60 -- rustc --print cfg
```

Command 68:

```bash
git diff -- crates/rum-core/src/expr.rs && python3 - <<'PY'
from pathlib import Path
import hashlib,json
base=Path('target/brief-506')
before=json.loads((base/'protected-before.json').read_text())
changed=[name for name,digest in before.items() if hashlib.sha256(Path(name).read_bytes()).hexdigest()!=digest]
allowed=['docs/dev/codex/brief-506-interner-census.md']
print('PROTECTED_CHANGED',json.dumps(changed))
assert changed==allowed,changed
path=Path('crates/rum-core/src/expr.rs')
assert path.read_bytes()==(base/'core-src/expr.rs').read_bytes()
print('FINAL_INSTRUMENTATION_MATCHES_MEASURED_SOURCE True')
print('PROTECTED_FILES_CHECKED',len(before))
PY
```

Command 69:

```bash
scripts/run-bounded.py --timeout 60 -- python3 target/brief-506/finalize.py
```

Command 70:

```bash
scripts/run-bounded.py --timeout 60 -- python3 target/brief-506/analyze.py > target/brief-506/analysis.txt
```

Command 71:

```bash
git diff --check && python3 - <<'PY'
from pathlib import Path
import hashlib,json
base=Path('target/brief-506')
report=Path('docs/dev/codex/brief-506-interner-census.md').read_text()
original=report.split('\n---\n\n## Execution report — brief-506',1)[0]
manifest=json.loads((base/'protected-before.json').read_text())
assert hashlib.sha256(original.encode()).hexdigest()==manifest['docs/dev/codex/brief-506-interner-census.md']
results=[json.loads(p.read_text()) for p in sorted((base/'logs').glob('*.result.json'))]
assert all(r['exit_code']==(101 if r['label']=='workspace' else 0) for r in results)
assert len([r for r in results if r['label'].startswith(('unique-','symbol-','module-'))])==15
print('ORIGINAL_BRIEF_PRESERVED True')
print('RUN_RESULTS',[(r['label'],r['exit_code']) for r in results])
print('MEASUREMENT_LOGS_COMPLETE True')
print('REPORT_SECTIONS',[line for line in report.splitlines() if line.startswith(('### ','#### Measurement'))][:9])
PY
```

Command 72:

```bash
scripts/run-bounded.py --timeout 60 -- python3 target/brief-506/finalize.py
```

</details>
