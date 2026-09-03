# brief-426 — can we afford to know an expression's height?

Shape: **implementation, and a measured spike**. Worker: **cdx (Codex)**.

Third step of the traversal line. brief-424 measured, brief-425 converted, and
this answers the question that decides whether the bound is affordable at all.
**It installs no bound.** Enforcement is the next brief, and only if this one's
numbers permit it.

## Why a spike and not the bound

Your brief-423 design puts a saturating physical height inside the private
`Arc` allocations so that admission is O(1) instead of a re-walk. That is the
right shape, and it is also a change to the hottest path in the tree: every
`SharedExpr::new` and `SharedArgs::new` would compute and store it. Your own
plan says to stop the line here if construction or memory labels regress, and I
agree — this project does not trade a hot-path regression for a
pathological-path benefit.

So the deliverable is the height, correct and measured, with nothing depending
on it yet.

## The target the numbers have to serve

`MAX_PHYSICAL_EXPR_DEPTH` is ruled at **`L >= 4097`** (user, 2026-09-03). My
brief-425 gate then pinned the other end: six of the ten converted traversals
complete at 4,500 and give out at 4,700 on a 2 MiB stack, and it is neither the
matcher nor structural equality — `MatchQ[x, _]` and `SameQ[x, x]` both clear
20,000 there. So `L` will land in roughly **[4,097, 4,500]**, with about ten
percent of margin at the ruled minimum. You do not need that here; it is context
for why the height must be exact rather than approximate near the boundary.

## Deliverable

### 1. The cached height

A saturating physical height in the private allocations —
`SharedExpr(Arc<Expr>)` (`crates/rum-core/src/expr.rs:573-574`) and
`SharedArgs(Arc<Vec<Expr>>)` (`:637-638`), with `WeakSharedArgs`
(`:686-687`) pointing at the same inner allocation. Define height as your design
did: `h(Atom) = 1`, `h(Call) = 1 + max(h(head), h(each raw argument))`.

**Saturating**, so an already-deep tree built before any bound exists cannot
wrap to a small number.

### 2. Four properties that must not move

These are derived today and each one is a way to break something quietly:

- **`SharedExpr` and `SharedArgs` and `WeakSharedArgs` stay pointer-sized.**
  Report `size_of` for all three, before and after.
- **`Eq` and `PartialEq` compare exactly what they compare today.** The height
  is a function of the value, so including it is harmless in result and wrong in
  cost — it adds a comparison to a hot path for no information.
- **`Hash` produces the same hash for the same expression.** Exclude the height.
- **`Serialize`/`Deserialize` produce byte-identical output.** These derives are
  on both wrappers; a changed shape is a stop-loss, not a migration to perform
  here. Prove it with a round-trip over a non-trivial expression, comparing
  serialized bytes before and after.

### 3. The measurement trap, which you must defeat before measuring

**Nothing reads the height in this brief, so the compiler may elide computing
it, and you will measure a change that is not there.** A neutral A/B obtained
that way is worse than no A/B: it would license the next brief on false
evidence.

So make the height observable in a way that cannot be optimized away, and say
what you chose — a `Diagnostic`-gated builtin reporting it, a debug assertion,
whatever fits. Then **show the computation actually happens**: an expression
whose height you can predict, read back through your observation point, at
several shapes including a deep one. If you cannot demonstrate the height is
being computed during the timed runs, the A/B does not count and you should say
so rather than report it.

### 4. Correctness against a reference

The cached height must equal a reference computed by direct recursion, for:
atoms, a call with a deep head and shallow arguments, a call with shallow head
and deep arguments, an `Association`, and a tree assembled from shared subtrees
where the same child appears at two positions. That last one is where an
incremental height is most likely to be wrong.

### 5. The A/B, and the stop-loss it feeds

Base against candidate in one window, profiling off:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>
bash scripts/core-language-microbench.sh --output-dir <cand-dir>
```

Report the construction and allocation labels, the list-literal rows, the
matcher rows, the definition-dispatch rows, and their same-tier neighbours.
**Name your control labels before looking at the candidate numbers.** If the
spread is noisy, raise the iteration count rather than reading the noise — that
worked in brief-420, where the same controls went from a -17%..+7% spread at 1x
to under 2% at 5x.

Also report the allocation counters, which are load-immune where timings are
not: `shared_expr_alloc` and `shared_args_alloc` must not move at all, since
this brief adds no allocation.

**If construction or memory labels regress, STOP and report.** Do not tune, do
not try a second layout, do not proceed to enforcement. A regression here is the
answer to the question this brief asks, and it is a useful answer.

## Scope

Files you MAY modify:

- `crates/rum-core/src/expr.rs`
- `crates/rum-core/src/lib.rs` if an export changes
- whatever the mechanical follow-on of the wrapper change touches — that is
  expected; anything beyond it, **STOP and report**
- a new test module, and the test module roots it needs
- this brief, to append your report

Files you MUST NOT touch: the memory-growth classifications or the `Preflight`
allowlist, `debug_tools/resources/`, any `ResourceLimits` default or the stride,
the traversal conversions brief-425 landed, `AGENTS.md`, `PROGRESS.md`,
`TODO.md`, `docs/**` except appending your report, `.git`, git config.

## Hard rules

- **Install no bound and refuse nothing.** No construction fails in this brief.
  If you find yourself writing a limit check, that is the next brief.
- **No new `unsafe`.** `rum-core` is zero-`unsafe`.
- **No default-path cost beyond the height itself.** The observation point from
  deliverable 3 must not be on the ordinary path when it is not being observed.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **This host can bind loopback**; the nine native ZeroMQ tests genuinely run
  here — I observed 9 of 9 at the brief-425 gate.
- **Append your report to this brief file** before finishing.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline --workspace
cargo test --offline --release --workspace -- --ignored
```

The third covers plan-330's deployment-stack probes and brief-425's ten depth
probes; a height field that changes frame sizes could move them, and I want to
know if it does.

## Report format

1. What changed — per file, line ranges, one-line why.
2. `size_of` for the three wrappers, before and after.
3. How you proved `Eq`, `Hash` and serde are unmoved, with the serde bytes.
4. **How you made the height observable, and your evidence that it is actually
   computed during the timed runs.**
5. Correctness against the reference, including the shared-subtree case.
6. The A/B rows with named controls, and the two allocation counters.
7. Verification — every command, verbatim output.
8. Your verdict: is the height affordable, and does the next brief proceed?

---

# Report — 2026-09-03 — Codex

## 1. What changed

No bound was installed and no construction can fail.

- `crates/rum-core/src/expr.rs:573-896` — replaced each shared wrapper's Arc
  payload with a private inner value carrying saturating `u16` physical-height
  metadata; added O(1) `Expr::physical_height()`/`SharedExpr::physical_height()`;
  kept pointer identity and weak identity; implemented logical `Eq`, `Hash`,
  serde, and `Debug` manually; and changed raw `SharedArgs` mutation into a
  scoped closure that refreshes metadata on return or unwind.
- `crates/rum-core/src/expr.rs:1053-1055` and
  `crates/rum-core/src/expr/height_cache_tests.rs:1-125` — added the test module
  and the layout, reference-height, shared-DAG, 4,097-deep observation,
  saturation, mutation, equality, and hash tests. Splitting the test module
  leaves `expr.rs` at 30,683 bytes, below the 35k architecture-debt line.
- `crates/rum-runtime/src/runtime/definitions/incremental_append.rs:52-95,212-234`
  — mechanically scoped leaf mutation so every enclosing argument cache is
  refreshed.
- `crates/rum-runtime/src/runtime/definitions/part_assignment.rs:185-215,335-356`
  — mechanically scoped positional mutation for the same invariant.
- `crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs:90-124` —
  mechanically scoped direct Association mutation.
- `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs:164-202`,
  `crates/rum-runtime/src/runtime/tests/association_lookup_cache.rs:52-60`, and
  `crates/rum-runtime/src/runtime/tests/association_part_cursor.rs:92-100` —
  mechanical test call-site migration to the scoped mutation API.
- This report is the only documentation change. `rum-core/src/lib.rs`, resource
  defaults, memory-growth classifications, the `Preflight` allowlist,
  traversal code, and debug resources were not changed.

The recurrence is exactly `h(Atom)=1` and
`h(Call)=1+max(h(head),h(each raw argument))`, with `saturating_add(1)`.
`SharedArgs` caches the maximum immediate child height, including zero for an
empty argument vector. No recursive scan is used during construction.

## 2. Representation

The same standalone Rust/serde probe was compiled offline against base and
candidate:

| Wrapper | Base `size_of` | Candidate `size_of` |
|---|---:|---:|
| `SharedExpr` | 8 | 8 |
| `SharedArgs` | 8 | 8 |
| `WeakSharedArgs` | 8 | 8 |

All three remain one pointer. The private allocation payload does grow:
release disassembly shows the complete Arc allocation request for
`SharedExpr::new` moving from `0x30` (48) to `0x38` (56) bytes and
`SharedArgs::new` from `0x28` (40) to `0x30` (48) bytes. Thus each allocation
gains 8 bytes even though the public wrappers do not.

## 3. Equality, hash, and serde

`PartialEq` retains the exact old pointer-fast-path followed by logical-value
comparison. `Eq` is empty. `Hash` delegates only to `Expr`/`[Expr]`; height is
never hashed. A core test deliberately constructs equal private inners with
different cached metadata (`1` versus `u16::MAX`) and proves both equality and
hash equality. The independent base/candidate probe produced identical
`DefaultHasher` results:

```text
shared_expr_hash=18203567883814912169
shared_args_hash=4376554985300920199
```

Manual serde implementations delegate to the old logical `Expr` and
`Vec<Expr>` shapes. Deserialization calls `SharedExpr::new`/`SharedArgs::new`,
so metadata is recomputed rather than accepted from input. The non-trivial
round-trip contains a call-valued head, string and integer atoms, an
`Association`, and a nested `Rule`. Base and candidate bytes were identical:

```text
shared_expr_json bytes=551 sha256=143cc3d5c0de6a36c33e7c7bc34e68faa25badf6108095aead9da263bafd8a5f
{"Call":{"head":{"Call":{"head":{"Atom":{"Symbol":{"context":"Global`","name":"HeadFactory"}}},"args":[{"Call":{"head":{"Atom":{"Symbol":{"context":"Global`","name":"InnerHead"}}},"args":[{"Atom":{"String":"head-leaf"}}]}}]}},"args":[{"Atom":{"Integer":[-1,[7]]}},{"Call":{"head":{"Atom":{"Symbol":{"context":"System`","name":"Association"}}},"args":[{"Call":{"head":{"Atom":{"Symbol":{"context":"System`","name":"Rule"}}},"args":[{"Atom":{"String":"key"}},{"Atom":{"Integer":[1,[42]]}}]}}]}},{"Atom":{"Symbol":{"context":"Global`","name":"tail"}}}]}}

shared_args_json bytes=584 sha256=4fbc271d46f01625015fef6a73d0ff443c6e4e3c7f2e57f44bafcf2c2289ce5f
[{"Call":{"head":{"Call":{"head":{"Atom":{"Symbol":{"context":"Global`","name":"HeadFactory"}}},"args":[{"Call":{"head":{"Atom":{"Symbol":{"context":"Global`","name":"InnerHead"}}},"args":[{"Atom":{"String":"head-leaf"}}]}}]}},"args":[{"Atom":{"Integer":[-1,[7]]}},{"Call":{"head":{"Atom":{"Symbol":{"context":"System`","name":"Association"}}},"args":[{"Call":{"head":{"Atom":{"Symbol":{"context":"System`","name":"Rule"}}},"args":[{"Atom":{"String":"key"}},{"Atom":{"Integer":[1,[42]]}}]}}]}},{"Atom":{"Symbol":{"context":"Global`","name":"tail"}}}]}},{"Atom":{"String":"argument"}}]
```

The candidate round-trip compared equal to the source and reported height 4
before and after deserialization.

## 4. Observation and proof against dead-code elimination

The observation point is the public, constant-time
`Expr::physical_height()` (and the shared-head accessor). It is invoked only
when requested, so it adds no ordinary-path work beyond maintaining the cache.
The release probe read these predicted values:

```text
shared_expr_height=4
roundtrip_height=4
atom_height=1
deep_head_height=18
deep_argument_height=20
association_height=13
shared_dag_height=15
deep_4097_height=4097
```

The exact candidate binary used for timing had SHA-256
`496c075808e9c3b4c0192d46970a2ff4c7af46d3ac7ae03f6896686dff61f369`.
`nm -C` showed both constructors as out-of-line text symbols:

```text
00000000007a69e0 T rum_core::expr::SharedArgs::new
00000000007a6300 T rum_core::expr::SharedExpr::new
```

`objdump -d -C` showed `SharedExpr::new` loading the cached head/argument
heights, selecting the maximum, incrementing, and conditionally selecting
`0xffff` before allocation. `SharedArgs::new` contains the loop over 32-byte
`Expr` slots, the same cached child loads/max/saturating increment, and only
then allocation and the existing env-gated repr counter. This code is
unconditional in the constructor; it is not a debug assertion and is present
in the release binary.

Finally, the same generated benchmark source and same candidate binary were
rerun with repr counting enabled. It reached these constructors
`shared_args_alloc=270462` and `shared_expr_alloc=196090` times. Since the
disassembled height work precedes those counter sites in each out-of-line
constructor, the profiling-off timing run necessarily executed the height
work too. The timing A/B therefore does not fall into the compiler-elision
trap.

## 5. Correctness against the reference

`cached_physical_height_matches_recursive_reference` uses an independent
direct recursive walker. It compares atoms, a height-17 head with a shallow
argument, a shallow head with a height-19 argument, an `Association` whose
keys and values are both included, and a parent containing two clones of one
shared call subtree. Pointer identity is asserted in the last case before the
height comparison. Separate tests cover unique and COW argument mutation,
`u16::MAX` saturation, and the 4,097-deep observation.

Focused result:

```text
running 13 tests
test expr::height_cache_tests::shared_wrapper_layout_stays_pointer_sized ... ok
test expr::height_cache_tests::shared_args_mutation_refreshes_height_for_unique_and_cow_values ... ok
test expr::height_cache_tests::cached_physical_height_matches_recursive_reference ... ok
test expr::height_cache_tests::physical_height_saturates_instead_of_wrapping ... ok
test expr::height_cache_tests::shared_wrapper_equality_and_hash_ignore_cached_height ... ok
test expr::height_cache_tests::physical_height_observation_reaches_the_deployment_depth ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 6. A/B and allocation counters

Before inspecting candidate numbers, the named controls were
`literal-integer`, `literal-string`, `symbol-ownvalue-read`, `loop-empty`,
`sameq-integer`, `head-list`, `length-list`, `part-list`, `sameq-list`,
`if-true`, `module-local`, `block-local`, `with-local`, `table-small`, and
`do-small`. `table-iterator-pair-list` was the same-tier list-construction
neighbor. Targets were every construction/mutation row, every
`matchq-*`/`matcher-single-*` row, and all downvalue/upvalue/subvalue/default/
valueq dispatch rows.

The initial 5x window had broad control noise (-19% to +15%), so it was not
used. A 20x attempt was invalid because the unchanged base reached the existing
100,000 iteration-work limit:

```text
warning: rum exited with 1; see /tmp/brief426-base-20x/raw/rum.stderr
error: core microbench failed because rum exited with 1; no timings.tsv written
```

The final window used the largest viable 10x scale and 21 repetitions.
Profiling variables were absent from the environment and the script explicitly
disabled its profiling switches. Commands were bounded; the inner benchmark
commands were:

```text
bash scripts/core-language-microbench.sh --output-dir /tmp/brief426-base-10x-21r --rum-cli /tmp/brief426-base-rum-cli --scale 10 --reps 21
core microbench output: /tmp/brief426-base-10x-21r
bash scripts/core-language-microbench.sh --output-dir /tmp/brief426-candidate-10x-21r --rum-cli /tmp/brief426-candidate-rum-cli --scale 10 --reps 21
core microbench output: /tmp/brief426-candidate-10x-21r
```

Whole-process elapsed time was 36.228745 seconds base and 36.676747 seconds
candidate.

### Controls, list construction, and allocation/mutation rows

| Label | Iterations | Base s | Candidate s | Delta |
|---|---:|---:|---:|---:|
| `loop-empty` | 50000 | 0.00228923 | 0.00229249 | +0.14% |
| `literal-integer` | 20000 | 0.000728272 | 0.000750843 | +3.10% |
| `literal-string` | 20000 | 0.000812070 | 0.000780805 | -3.85% |
| `symbol-ownvalue-read` | 20000 | 0.00209467 | 0.00206443 | -1.44% |
| `list-literal-small` | 10000 | 0.000769459 | 0.000768988 | -0.06% |
| `head-list` | 20000 | 0.0112265 | 0.0112397 | +0.12% |
| `length-list` | 20000 | 0.0112847 | 0.0113230 | +0.34% |
| `part-list` | 20000 | 0.0131398 | 0.0130609 | -0.60% |
| `sameq-integer` | 20000 | 0.00622213 | 0.00635234 | +2.09% |
| `sameq-list` | 10000 | 0.00860467 | 0.00824931 | -4.13% |
| `if-true` | 20000 | 0.00714922 | 0.00713292 | -0.23% |
| `module-local` | 5000 | 0.00747698 | 0.00749022 | +0.18% |
| `block-local` | 5000 | 0.00463495 | 0.00461973 | -0.33% |
| `with-local` | 5000 | 0.00487055 | 0.00488797 | +0.36% |
| `table-small` | 2000 | 0.00184758 | 0.00186761 | +1.08% |
| `table-iterator-pair-list` | 10000 | 0.000710864 | 0.000764147 | **+7.50%** |
| `do-small` | 2000 | 0.00900604 | 0.00897176 | -0.38% |
| `appendto-list-incremental-small` | 2500 | 0.0270856 | 0.0295070 | **+8.94%** |
| `appendto-list-incremental-large` | 5000 | 0.103957 | 0.112823 | **+8.53%** |
| `appendto-part-outer1000-small` | 1000 | 0.0230635 | 0.0220521 | -4.39% |
| `appendto-part-outer1000-large` | 2000 | 0.0500780 | 0.0529318 | **+5.70%** |
| `association-set-key-incremental-small` | 1000 | 0.133748 | 0.134293 | +0.41% |
| `association-set-key-incremental-large` | 2000 | 0.538978 | 0.536281 | -0.50% |
| `association-small-literal-repeat` | 5000 | 0.00739224 | 0.00770015 | +4.17% |
| `set-part-fixed100-len100` | 100 | 0.000128524 | 0.000134467 | +4.62% |
| `set-part-fixed100-len800` | 100 | 0.000477678 | 0.000621161 | **+30.04%** |
| `set-part-fill-small` | 200 | 0.000369102 | 0.000420047 | **+13.80%** |
| `set-part-fill-large` | 400 | 0.00114619 | 0.00126635 | **+10.48%** |

### Matcher rows

| Label | Iterations | Base s | Candidate s | Delta |
|---|---:|---:|---:|---:|
| `matchq-integer-success` | 10000 | 0.00308842 | 0.00307689 | -0.37% |
| `matchq-integer-fail` | 10000 | 0.00313321 | 0.00311574 | -0.56% |
| `matcher-single-tiny` | 10000 | 0.00677066 | 0.00672382 | -0.69% |
| `matcher-single-wide-small` | 800 | 0.000038041 | 0.000037931 | -0.29% |
| `matcher-single-wide-large` | 1600 | 0.000075384 | 0.000074411 | -1.29% |

### Definition-dispatch rows

| Label | Iterations | Base s | Candidate s | Delta |
|---|---:|---:|---:|---:|
| `downvalue-exact-hit` | 20000 | 0.00773674 | 0.00775390 | +0.22% |
| `downvalue-exact-miss` | 20000 | 0.00548983 | 0.00549895 | +0.17% |
| `downvalue-pattern-hit` | 10000 | 0.00971568 | 0.00995650 | +2.48% |
| `downvalue-pattern-miss` | 10000 | 0.00284093 | 0.00287768 | +1.29% |
| `downvalue-exact-miss-repeat` | 50000 | 0.0124963 | 0.0124858 | -0.08% |
| `downvalue-pattern-miss-repeat` | 50000 | 0.0129109 | 0.0130392 | +0.99% |
| `downvalue-guarded-hit` | 10000 | 0.0192601 | 0.0195962 | +1.75% |
| `downvalue-guarded-decline-fallback` | 10000 | 0.0143423 | 0.0149195 | +4.02% |
| `upvalue-guarded-hit` | 10000 | 0.0238321 | 0.0242131 | +1.60% |
| `subvalue-unguarded-hit` | 10000 | 0.0139616 | 0.0142852 | +2.32% |
| `subvalue-unguarded-miss` | 10000 | 0.00767422 | 0.00793009 | +3.33% |
| `subvalue-guarded-hit` | 10000 | 0.0245836 | 0.0253421 | +3.09% |
| `default-unguarded-hit` | 10000 | 0.0130627 | 0.0132358 | +1.33% |
| `default-unguarded-miss` | 10000 | 0.0107565 | 0.0108337 | +0.72% |
| `default-guarded-hit` | 10000 | 0.0167407 | 0.0168168 | +0.45% |
| `upvalue-candidates-tiny` | 10000 | 0.0198672 | 0.0198347 | -0.16% |
| `upvalue-candidates-wide-small` | 1600 | 0.000156794 | 0.000165080 | **+5.28%** |
| `upvalue-candidates-wide-large` | 3200 | 0.000309635 | 0.000326603 | **+5.48%** |
| `valueq-undefined-symbol-definitions-2000` | 200 | 0.000066451 | 0.000064779 | -2.52% |
| `valueq-undefined-symbol-definitions-8000` | 200 | 0.000066868 | 0.000064459 | -3.60% |

The controls are centered near zero, while list construction, incremental
append, fixed/fill Part mutation, and wide upvalue candidate construction show
a coherent cost. The width-sensitive Part result is also mechanically
consistent with rescanning the immediate argument vector after mutation. This
is the brief's construction stop-loss; no alternate layout or tuning was
attempted.

The load-immune repr counters did not move:

| Counter | Base | Candidate | Delta |
|---|---:|---:|---:|
| `shared_expr_alloc` | 196090 | 196090 | 0 |
| `shared_args_alloc` | 270462 | 270462 | 0 |

For completeness, a profiling-off core-memory panel showed identical modeled
`MemoryInUse` retained/peak values for every label. RSS varied in both
directions and was not used as a decision signal. The deterministic allocation
requests above establish the real +8-byte payload cost:

| Memory label | Base retained/peak | Candidate retained/peak |
|---|---:|---:|
| `baseline` | 205 / 483 | 205 / 483 |
| `matcher-single-wide-small` | 66343 / 66901 | 66343 / 66901 |
| `matcher-single-wide-large` | 131879 / 132437 | 131879 / 132437 |
| `module-leak-20000` | 3005 / 3283 | 3005 / 3283 |
| `retain-range-1000` | 64909 / 65187 | 64909 / 65187 |
| `retain-range-10000` | 640909 / 641187 | 640909 / 641187 |
| `retain-range-100000` | 6400909 / 6401187 | 6400909 / 6401187 |
| `transient-total-200000` | 12802494 / 19202587 | 12802494 / 19202587 |

## 7. Verification

All cargo invocations were offline and long runs were process-group bounded.

`cargo --offline fmt --check`

```text
```

Exit status 0.

Focused core command, `cargo test --offline -p rum-core`:

```text
running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Mandatory `cargo test --offline --workspace` was run unmodified. Contrary to
the brief's host note, this execution sandbox denied every loopback bind:

```text
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip

failures:
    zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket
    zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets
    zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip
    zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages
    zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata
    zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown
    zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply
    zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end
    zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages

test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

error: test failed, to rerun pass `-p rum-jupyter --lib`
```

The explicit repository opt-out was used only to verify everything else:
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --offline --workspace`. Verbatim
crate result lines:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 946 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.82s
test result: ok. 843 passed; 0 failed; 13 ignored; 0 measured; 0 filtered out; finished in 4.15s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 190 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.80s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Aggregate: 2,325 passed, 0 failed, 17 ignored. The nine native ZeroMQ cases
are unobserved, not counted as behavioral passes.

Mandatory `cargo test --offline --release --workspace -- --ignored`:

```text
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.26s

running 2 tests
test runtime::tests::left_chain_lowering::left_chain_two_mib_full_round_trip_accepts_and_rejects_without_abort ... ok
test runtime::tests::left_chain_lowering::left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 945 filtered out; finished in 0.05s

running 13 tests
test tests::iterative_traversal_depth::tostring_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::apply_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::map_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::level_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::position_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::freeq_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::count_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::memberq_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::replaceall_completes_at_physical_height_4097_on_two_mib ... ok
test tests::iterative_traversal_depth::cases_completes_at_physical_height_4097_on_two_mib ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 843 filtered out; finished in 1.34s
```

Aggregate: 17 passed, 0 failed. The height metadata did not move the
deployment-stack probe boundary.

Final source checks: `git diff --check` produced no output; no new `unsafe`,
limit check, refusal, resource-setting change, or default-path instrumentation
was introduced.

## 8. Verdict

**The height is not affordable in this layout.** Public wrapper sizes, logical
behavior, serialization, allocation counts, matcher timings, and deployment
stack probes are preserved, but both private Arc allocations grow by 8 bytes
and stabilized construction/mutation rows regress materially—up to 30.04%,
with repeated list append consistently about 8.5-8.9% slower.

The stop-loss fires. I did not tune the layout, try a second representation, or
install enforcement. **The next enforcement brief must not proceed on these
numbers.**
