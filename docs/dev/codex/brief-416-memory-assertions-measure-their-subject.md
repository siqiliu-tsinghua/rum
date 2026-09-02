# brief-416 — four memory assertions must measure what they claim

Shape: **implementation**. Worker: **cdx (Codex)**.

brief-415 made the session memory walk honest. Four pre-existing assertions
went red as a result. I gated brief-415 myself and reproduced all four: none of
them is a defect in the accounting, and none of them is a defect in the kernel.
All four were asserting properties that only held while the walk was blind.

**This brief restores the workspace gate to green by fixing the assertions, and
by nothing else.**

## The working tree already holds brief-415's change, uncommitted

`crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`,
`crates/rum-runtime/src/runtime/tests/mod.rs` and the new
`crates/rum-runtime/src/runtime/tests/session_memory.rs` are present and
uncommitted. **Do not revert, modify, stash, or "clean up" any of them.** They
are the correct state; this brief builds on top. Do not touch `stash@{0}`.

## Goal

`cargo test --offline --workspace` green, with every one of the four assertions
still asserting something real — in most cases something stronger than before,
because a differential against a named control is a better instrument than a
constant that happened to hold.

## The four, with my ruling on each

### 1 & 2 — the two host abandonment tests

`crates/rum-host/tests/suspension_abandonment_contract.rs`:
`public_cancelled_suspension_restores_reap_monitor_trace_and_constraint_state`
(the assertion at `:478-481`) and
`public_monitor_matrix_restores_reap_monitor_trace_constraints_random_and_abort_state`.

Both assert `memory_growth < 16_000` with the message "cancelled Reap buckets
retained {n} proxy bytes". Observed now: 68 312 and 260 758.

**Director-measured, and it settles what these tests are actually seeing:** a
*completed* — not cancelled — `Reap[Sow[Range[2000]]; 1]` grows the proxy by
**259 807 bytes** in a fresh session (`rum-cli --eval 'b = MemoryInUse[];
Reap[Sow[Range[2000]]; 1]; a = MemoryInUse[]; a - b'`). That is *more* than the
cancelled run retains. So the growth these assertions now trip on is ordinary
evaluation-cache retention, present whether or not the request is cancelled. A
constant threshold cannot separate the two; it never could, it was merely below
the noise while the cache was invisible.

**Ruling: replace the constant with a differential against a completed
control.** In the same test, run the equivalent workload to *completion* in an
equivalent fresh session, measure its growth the same way, and assert that the
cancelled run does not retain materially more than the completed one. Pick and
state the margin; justify it from the two measured numbers rather than choosing
a round figure. Print both numbers in the existing `println!` style so a future
reader sees the control, not just the subject.

The semantic assertions in both tests — the sentinel staying `0`, the monitor
refresh being omitted, the trace and constraint state restored — are unaffected
and must remain exactly as they are. Do not weaken them, and do not delete the
memory assertion instead of repairing it.

### 3 — `MaxMemoryUsed[] >= MemoryInUse[]` is not an invariant

`crates/rum-stdlib/src/tests/incidental_debug_tools_resources.rs:185-192`
asserts that as a single expression.

The high-water mark is updated inside the walk itself
(`memory.rs:335-336`), so both operands sample and both update it — but at
different times. `MaxMemoryUsed[]` evaluates first and returns the peak as of
*its* walk; `MemoryInUse[]` then walks again, and evaluation between the two
samples has grown state, so it reports more. A peak read at T1 cannot cover
growth at T2. The assertion held only because nothing used to grow in between.

Note the observer effect while you are here: evaluating `MemoryInUse[]`
populates the call-stability cache, so the measurement grows the quantity it
measures. That is worth a comment in the test, not a repair.

**Ruling: assert the ordering that is actually invariant** — sample
`MemoryInUse[]` first, then `MaxMemoryUsed[]`, so the peak observation is the
later one. Add a comment stating why the original order is not an invariant, so
this is not "fixed" back later. The rest of that test (the `Do[gmem[i] = i, …]`
before/after growth check) is unaffected; leave it.

### 4 — `byte_count_and_memory_in_use_use_the_current_structural_proxy_subset`

Same file, from `:13`. Its `after_on < after_off` relation flipped to `False`.

I have not ruled on this one, because what it intends to prove is not obvious
from the assertion alone. **Determine what property the test exists to
establish, say so in the report in one sentence, and make it assert that
property.** If the honest conclusion is that the property no longer holds and
the test was encoding an accident, say that plainly rather than finding a
formulation that passes — that is a stop-loss, and I want it as one.

## Scope

Files you MAY modify:

- `crates/rum-host/tests/suspension_abandonment_contract.rs`
- `crates/rum-stdlib/src/tests/incidental_debug_tools_resources.rs`
- this brief, to append your report

Files you MUST NOT touch — this list is the point of the brief:

- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs` and the
  other two brief-415 files. The accounting is settled and correct.
- Anything under `crates/rum-runtime/src/runtime/model/` or the resource-limit
  defaults.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, git config,
  `Cargo.lock`.

## Hard rules

- **Do not make a number smaller to make a test pass.** No raising a bound, no
  clearing a cache before a measurement to hide retention, no removing a term
  from the walk, no retuning a limit or the stride. If you conclude one of
  those is genuinely the right repair, STOP and report — that is my decision,
  and it would need its own brief with a hot-path A/B.
- **Do not delete an assertion.** Repair it or stop-loss it.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree; I gate and commit.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **This host can bind loopback**, so the nine native ZeroMQ tests in
  `rum-jupyter` genuinely run here — I verified 9 of 9 passing during the
  brief-415 gate. If your sandbox cannot bind and you need
  `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, say so explicitly in the report and mark
  every native result as unobserved. Never report a rate for a test you did not
  watch run.
- No new `unsafe`. Minimal diff; no drive-by reformatting.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline -p rum-host --test suspension_abandonment_contract
cargo test --offline -p rum-stdlib --lib
cargo test --offline --workspace
```

Paste the verbatim `test result:` lines for each. The workspace run is the
gate; a green narrow run is not a green tree.

## Report format

1. What changed — per file, line ranges, one-line why.
2. For each of the four assertions: what it now asserts, and the measured
   numbers behind any margin you chose.
3. Your one-sentence statement of what assertion 4 exists to establish.
4. Verification — every command, verbatim output.
5. Stop-loss / open questions.

## Worker report

### 1. What changed

- `crates/rum-host/tests/suspension_abandonment_contract.rs:8-11,267-287,481-501` — replaced both fixed retention ceilings with completed-`Reap` controls in equivalent fresh sessions, including a measured margin and subject/control diagnostics.
- `crates/rum-stdlib/src/tests/incidental_debug_tools_resources.rs:87-133` — isolated disabled-message-selector storage with equivalent fresh sessions whose identical `On`/`Off` request sets leave opposite final selector states.
- `crates/rum-stdlib/src/tests/incidental_debug_tools_resources.rs:209-218` — sampled `MemoryInUse[]` before `MaxMemoryUsed[]` and documented the measurement's call-stability-cache observer effect.

### 2. The four repaired assertions

1. `public_cancelled_suspension_restores_reap_monitor_trace_and_constraint_state` now asserts that cancelled `Reap` growth does not exceed a completed fresh-session control plus 1,902 bytes. The observed cancelled growth was 68,312 bytes and the in-test completed control was 259,090 bytes.
2. `public_monitor_matrix_restores_reap_monitor_trace_constraints_random_and_abort_state` now asserts that abandoned `Reap` growth does not exceed the same completed fresh-session control plus 1,902 bytes. The observed abandoned growth was 260,758 bytes and the in-test completed control was 259,090 bytes. The 1,902-byte margin is exactly twice the director-measured gap `260,758 - 259,807 = 951`: one additional observed gap of headroom, while remaining below 0.74% of the completed control. Both tests print subject, control, and margin.
3. `max_memory_used_tracks_session_memory_proxy_high_water_mark` now asserts `MemoryInUse[] <= MaxMemoryUsed[]`, so current use is sampled before the later peak observation; the comment records why the reverse sampling order is not invariant.
4. `byte_count_and_memory_in_use_use_the_current_structural_proxy_subset` retains the assertion that `Off` grows measured state and replaces the confounded same-session `On` comparison with two fresh sessions that execute the same `On` and `Off` calls in opposite order, asserting that the session ending with the selector removed uses less proxy memory than the one ending with it retained.

### 3. Assertion 4's purpose

Assertion 4 exists to establish that `MemoryInUse[]` accounts for disabled-message-selector storage: disabling a selector increases the proxy, and removing it leaves less retained proxy memory than an equivalent session that keeps it disabled.

### 4. Verification

All cargo invocations were run through `scripts/run-bounded.py`; the cargo commands and their verbatim `test result:` output follow.

`cargo fmt --check`

No output; exit status 0.

`cargo test --offline -p rum-host --test suspension_abandonment_contract`

```text
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

`cargo test --offline -p rum-stdlib --lib`

```text
test result: ok. 828 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.70s
```

The first `cargo test --offline --workspace` attempt reached the nine native ZeroMQ tests; all nine failed before exercising their subjects because this sandbox rejected loopback bind with `Operation not permitted (os error 1)`. Its failing `rum-jupyter` result was:

```text
test result: FAILED. 97 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

As explicitly permitted by the brief for that condition, the workspace gate was rerun as `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --offline --workspace`. Its verbatim result lines were:

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 934 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.15s
test result: ok. 828 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.58s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.78s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 5. Stop-loss / open questions

No assertion required a stop-loss and no implementation or accounting code was changed. The nine native ZeroMQ subjects are unobserved in this sandbox because loopback bind is denied; the skip-enabled green workspace result is not a native ZeroMQ pass rate, and an unrestricted gate still needs to observe those tests.
