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
