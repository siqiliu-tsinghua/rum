# brief-418 — the falsifier belongs in tests, not in production

Shape: **implementation**. Worker: **cdx (Codex)**.

A correction to brief-417's landed work, found at my gate rather than by you.
Your implementation was sound; its arming condition is wrong in both
directions, and I only established that by measuring after the commit.

## The two defects

### A — it runs in production, and panics there

`validate_builtin_memory_growth` (`eval_dispatch/mod.rs:189-201`) arms whenever
`self.current_memory_limit.is_some()`. That field is set from
`resource_limits.eval_memory_limit` in the request prologue
(`session/session_helpers.rs:33`), so **every request of every host that
configures a memory ceiling** — the recommended sandbox configuration, and the
whole point of this line — runs the falsifier on every builtin call.

Two consequences, and the second is worse than the first.

**It costs.** Director-measured, release build, one variable (the
`current_memory_limit.is_none()` clause removed, nothing else), same binary
shape, `AbsoluteTiming` in-kernel, profiling off:

```
big = Range[3000];
Do[Length[big], {n}]   plain   vs   MemoryConstrained[same, 100000000]
```

| n | ceiling, falsifier armed | ceiling, falsifier off | ratio | plain control (armed) | plain control (off) | control delta |
|---:|---:|---:|---:|---:|---:|---:|
| 300 | 0.001711 | 0.000798 | **2.15x** | 0.000134 | 0.000138 | -2.6% |
| 600 | 0.003621 | 0.001633 | **2.22x** | 0.000242 | 0.000246 | -1.7% |
| 1200 | 0.007077 | 0.003233 | **2.19x** | 0.000554 | 0.000489 | +13.2% |

Both arms scale linearly with `n`. The affected arm moves 2.15-2.22x
consistently across three sizes; the unaffected control moves within
-2.6%..+13.2%, which is the noise floor. `Length` is `NonAmplifying` and its
argument is large, so each call walks the 3 000-element list and the result.

**It panics.** `assert_non_amplifying_result` (`:320-333`) is an `assert!`. In a
production sandbox a misdeclared builtin therefore kills the request through
the panic boundary, when the honest consequence of a wrong declaration is an
under-count. brief-417's own hard rule was "no behaviour change"; this is one,
and it is the kind that only shows up in a host that armed a ceiling.

A diagnostic postcondition must not be load-bearing in production. This project
has a standing hard line about diagnostics not costing the paths that do not
ask for them, and this is that line with a ceiling in place of the default
path.

### B — it does not run where it is most needed

`builtin_memory_growth_test_enabled` (`:299-309`) returns true under
`cfg!(test)`. That is evaluated when **`rum-runtime` itself** is compiled.
`rum-stdlib`, `rum-host` and `rum-jupyter` test binaries link `rum-runtime` as
an ordinary dependency, compiled without `cfg(test)`, so the falsifier is off
for all of them unless `RUM_TEST_BUILTIN_MEMORY_GROWTH` is set by hand.

Director-measured. I misdeclared `Rule` as `NonAmplifying` in its declaration
and in both checked-in inventories, so the falsifier was the only remaining
guard:

```
cargo test --offline --workspace --no-fail-fast     ->  2295 passed; 0 failed
RUM_TEST_BUILTIN_MEMORY_GROWTH=1 <same command>     ->  2293 passed; 2 failed
    builtin Rule declared NonAmplifying but returned 64289 proxy bytes
    from 424 argument proxy bytes (allowance: 1024 bytes)
```

The ordinary gate does not catch a builtin that amplifies 151x. The checked-in
inventory catches a *reclassification*, but not a **new** builtin declared
wrongly, nor an existing handler that starts amplifying — which are the only
two things the falsifier uniquely guards.

## Goal

The falsifier is armed automatically for every test binary in the workspace,
and not armed by a memory ceiling. `RUM_TEST_BUILTIN_MEMORY_GROWTH` stays as an
explicit opt-in for diagnosing a release build.

**Making the gate remember an environment variable is not an acceptable
answer.** The standing rule on this project is that a fix the compiler or the
build system enforces beats one that relies on people remembering, and a gate
step that must be typed correctly is the second kind. I would forget it; so
would the next reader of `AGENTS.md`.

## Deliverable

1. **Remove `current_memory_limit.is_some()` from the arming condition.** The
   fast path becomes a single check of the test-mode predicate.
2. **Structural test arming.** Every test binary in the workspace arms the
   falsifier without an environment variable — all ten crates (`rum-cli`,
   `rum-core`, `rum-derive`, `rum-host`, `rum-jupyter`, `rum-repl`,
   `rum-runtime`, `rum-stdlib`, `rum-syntax`, `rum-terminal`) and their
   integration-test targets, not only the ones that happen to exercise builtins
   today. A `rum-runtime` cargo feature that the other crates
   enable through a `dev-dependencies` entry is the obvious route and the
   workspace is on `resolver = "3"`, so dev-dependency features are not applied
   when building non-test targets. Use it, or something better, and say why.
3. **A proof it does not leak into a release build.** Assert it, do not assume
   it: build the release `rum-cli` and show the falsifier is inert there. The
   timing shape above is one way — under a ceiling the `Length[Range[3000]]`
   workload must return to the ~0.0008/0.0016/0.0032 column, not the ~2.2x one.
   State whichever proof you use and paste its output.
4. **A proof arming actually reaches the other crates.** Repeat my `Rule`
   experiment: misdeclare it in the declaration and both inventories, run
   `cargo test --offline --workspace` with **no** environment variable, show
   the falsifier firing, then revert all three edits and show the tree green
   again. Paste both results. A falsifier that has never been seen to fire in
   the configuration you just built is not known to work there.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
- `crates/rum-runtime/Cargo.toml`, and the `Cargo.toml` of any workspace crate
  that needs the dev-dependency entry
- `Cargo.toml` (workspace root) only if a shared dependency entry requires it
- `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs` and a new
  test module if you need one

If the mechanism you choose needs a file outside this list, **STOP and report**
before editing it.

Files you MUST NOT touch: `crates/rum-runtime/src/runtime/debug_tools/resources/`
(the accounting is settled), the growth classifications themselves, any
`ResourceLimits` default or the stride, `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
`docs/**` except appending your report, `.git`, git config, `Cargo.lock` beyond
what cargo rewrites on its own.

## Hard rules

- **No classification changes.** If arming the falsifier across the workspace
  reveals a builtin that amplifies and is declared `NonAmplifying`, that is a
  **finding to report, not to fix here** — it is exactly what this brief exists
  to surface, and I want to see the list before anything is reclassified. Stop
  and report it with the numbers.
- **No default-path cost.** With the falsifier disarmed the check must remain a
  single cheap predicate that returns immediately; do not reintroduce a walk, an
  allocation, or a lock on the ordinary path.
- **The feature must not be on in a normal `cargo build` or `cargo build
  --release`.** If you cannot establish that, STOP and report rather than
  shipping something that might.
- **No new `unsafe`.** Do not weaken an existing test.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **This host can bind loopback**; the nine native ZeroMQ tests genuinely run
  here — I observed 9 of 9 at the brief-417 gate. If your sandbox denies it and
  you need `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, say so and mark every native
  result unobserved.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline --workspace
cargo build --offline --release
```

Plus deliverables 3 and 4 above, with their verbatim output. No microbench A/B
is required: this brief only *removes* work from an armed path and adds none to
the default one. If you find yourself adding work anywhere, that is a stop-loss.

## Report format

1. What changed — per file, line ranges, one-line why.
2. The arming mechanism you chose, and why, including how you know it cannot
   reach a release build.
3. Deliverable 3's proof, verbatim.
4. Deliverable 4's proof, verbatim — the falsifier firing with no environment
   variable set, and the tree green after reverting.
5. Any builtin the newly-armed workspace suite caught. Report, do not fix.
6. Verification — every command, verbatim output.
7. Stop-loss / open questions.

---

# Report — 2026-09-03

## 1. What changed

| File | Lines | Change |
|---|---:|---|
| [eval_dispatch/mod.rs](/home/remi/dev/rum/crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:189) | 189–200, 298–308 | Removed memory-ceiling arming; enabled the falsifier only for `cfg(test)`, the test feature, or explicit environment opt-in. |
| [rum-runtime/Cargo.toml](/home/remi/dev/rum/crates/rum-runtime/Cargo.toml:12) | 12–13 | Added `test-builtin-memory-growth`. |
| [rum-host/Cargo.toml](/home/remi/dev/rum/crates/rum-host/Cargo.toml:15) | 15, 22–23 | Forwarded the feature for Jupyter and enabled it in host tests. |
| [rum-jupyter/Cargo.toml](/home/remi/dev/rum/crates/rum-jupyter/Cargo.toml:22) | 22–23 | Enabled the feature through `rum-host`, preserving the host-facade boundary. |
| [rum-cli/Cargo.toml](/home/remi/dev/rum/crates/rum-cli/Cargo.toml:26) | 26–27 | Added the dev-only runtime feature dependency. |
| [rum-core/Cargo.toml](/home/remi/dev/rum/crates/rum-core/Cargo.toml:16) | 16–17 | Same. |
| [rum-derive/Cargo.toml](/home/remi/dev/rum/crates/rum-derive/Cargo.toml:18) | 18–19 | Same. |
| [rum-repl/Cargo.toml](/home/remi/dev/rum/crates/rum-repl/Cargo.toml:21) | 21–22 | Same. |
| [rum-stdlib/Cargo.toml](/home/remi/dev/rum/crates/rum-stdlib/Cargo.toml:25) | 25–26 | Same. |
| [rum-syntax/Cargo.toml](/home/remi/dev/rum/crates/rum-syntax/Cargo.toml:15) | 15–16 | Same. |
| [rum-terminal/Cargo.toml](/home/remi/dev/rum/crates/rum-terminal/Cargo.toml:19) | 19–20 | Same. |
| [Cargo.lock](/home/remi/dev/rum/Cargo.lock:618) | 618–716 | Cargo-generated path dev-dependency entries only. |

No classifications, tests, documentation, or accounting code remain changed.

## 2. Arming mechanism

`builtin_memory_growth_test_enabled()` now returns true for:

```rust
cfg!(any(test, feature = "test-builtin-memory-growth"))
```

Otherwise it retains the cold-loaded atomic environment check for `RUM_TEST_BUILTIN_MEMORY_GROWTH`.

Eight crates directly enable the feature through `dev-dependencies`; `rum-jupyter` enables it through `rum-host` to preserve its architecture test; `rum-runtime` unit tests use `cfg(test)`. Resolver 3 keeps these dev features out of normal targets.

The all-target test graph showed structural activation:

```text
└── rum-runtime feature "test-builtin-memory-growth"
    [dev-dependencies]
    ├── rum-cli v0.2.0
    ├── rum-core v0.2.0
    ├── rum-derive v0.2.0
    ├── rum-host v0.2.0
    ├── rum-repl v0.2.0
    ├── rum-stdlib v0.2.0
    ├── rum-syntax v0.2.0
    └── rum-terminal v0.2.0
    └── rum-host feature "test-builtin-memory-growth"
```

The normal `rum-cli` graph contained only:

```text
rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
└── rum-runtime feature "default"
```

Both ordinary debug and release builds completed without the test feature.

## 3. Release-build inertness proof

After `cargo build --offline --release`, the exact release binary ran five under-ceiling samples per size with the opt-in variable unset:

```text
List[List[300, List["0.000817", "0.000809", "0.000796", "0.000814", "0.000821"]], List[600, List["0.001641", "0.001660", "0.001581", "0.001611", "0.001644"]], List[1200, List["0.003390", "0.003228", "0.003291", "0.003427", "0.003266"]]]
```

These are the expected inert bands: approximately 0.0008, 0.0016, and 0.0032 seconds.

As a control, the same release binary with explicit opt-in produced:

```text
List[List[300, List["0.001723", "0.001719", "0.001714", "0.001742", "0.001744"]], List[600, List["0.003443", "0.003405", "0.003413", "0.003432", "0.003401"]], List[1200, List["0.007099", "0.006972", "0.006838", "0.007293", "0.007528"]]]
```

Thus the release build is inert by default while `RUM_TEST_BUILTIN_MEMORY_GROWTH=1` remains functional.

## 4. Structural falsifier proof

I temporarily changed `Rule` to `NonAmplifying` in its declaration and both inventories, then ran the workspace suite with `RUM_TEST_BUILTIN_MEMORY_GROWTH` unset.

Verbatim failure:

```text
---- tests::associations::associations_key_sort::custom_key_sorts_obey_the_merge_bound_without_growing_the_stack stdout ----

thread 'tests::associations::associations_key_sort::custom_key_sorts_obey_the_merge_bound_without_growing_the_stack' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:327:5:
builtin Rule declared NonAmplifying but returned 64289 proxy bytes from 424 argument proxy bytes (allowance: 1024 bytes)

---- tests::codecs_text::ini_wide_key_and_section_indexes_preserve_round_trips stdout ----

thread 'tests::codecs_text::ini_wide_key_and_section_indexes_preserve_round_trips' panicked at crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:327:5:
builtin Rule declared NonAmplifying but returned 4696 proxy bytes from 273 argument proxy bytes (allowance: 1024 bytes)

failures:
    tests::associations::associations_key_sort::custom_key_sorts_obey_the_merge_bound_without_growing_the_stack
    tests::codecs_text::ini_wide_key_and_section_indexes_preserve_round_trips

test result: FAILED. 827 passed; 2 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.97s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

All three temporary edits were manually restored. The post-revert compact rerun included:

```text
running 939 tests
test result: ok. 937 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.73s

running 832 tests
test result: ok. 829 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.99s

running 189 tests
test result: ok. 189 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

The complete workspace result was 2,295 passed, 7 ignored, 0 failed.

## 5. Newly caught builtins

None. The ordinary newly armed suite found no real `NonAmplifying` misclassification. `Rule` was only the mandated temporary falsifier experiment and remains `Delegated`.

## 6. Verification

Formatting:

```text
$ scripts/run-bounded.py --timeout 120 -- cargo --offline fmt --check
```

No output; exit 0.

Workspace tests:

```text
$ env -u RUM_TEST_BUILTIN_MEMORY_GROWTH RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 scripts/run-bounded.py --timeout 1200 -- cargo test --offline --workspace
```

Exit 0; 2,295 passed, 7 ignored, 0 failed. The compact verbatim target summaries are reported above.

Release build:

```text
$ env -u RUM_TEST_BUILTIN_MEMORY_GROWTH scripts/run-bounded.py --timeout 1200 -- cargo build --offline --release
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `release` profile [optimized] target(s) in 21.66s
```

Ordinary build:

```text
$ env -u RUM_TEST_BUILTIN_MEMORY_GROWTH scripts/run-bounded.py --timeout 600 -- cargo build --offline
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.41s
```

`git diff --check` also returned no output and exit 0.

## 7. Stop-loss / open questions

This sandbox denied loopback binding:

```text
native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1); set RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 to explicitly allow this test to skip
```

Therefore these nine native results are unobserved:

- `native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket`
- `native_zeromq_backend_round_trips_execute_over_loopback_sockets`
- `native_zeromq_backend_frontend_style_control_interrupt_round_trip`
- `native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages`
- `native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata`
- `native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown`
- `native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply`
- `native_zeromq_backend_rejects_tampered_signed_requests_end_to_end`
- `native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages`

No other stop-loss or open question. No git operations, network access, `unsafe`, weakened tests, or classification changes were made.
