# brief-438 — calibrate the native stack budget to the ruled 2 MiB stack

**Shape: implementation.** One variable: the default native stack budget and
the named relationship it is derived from. No frame reduction, no evaluator
restructuring.

## Goal

`SECURITY.md:43-60` publishes a 6 MiB native stack default and then promises
"Exceeding one yields an in-band `TerminatedEvaluation[...]` value, not a
crash." On the ruled 2 MiB embedder stack that promise is false — the process
takes SIGABRT, because a 6 MiB budget cannot be reached on a 2 MiB stack. When
you are done, the default budget is derived by a named, checkable equation from
the minimum stack rum claims to support, that promise is true on the ruled
configuration, and a test fails if anyone detaches the default from the
equation again.

This does **not** make deep evaluation work. It makes deep evaluation fail
recoverably instead of killing the host. Reaching the ruled physical height of
4,097 for a strict residual call needs an explicit task stack and is a separate
project (brief-437 §3, projected floor 1.5-2.6 KiB per level against the
511.875 B that height 4,097 on 2 MiB would allow).

## Context pointers

Read `AGENTS.md` first, then brief-437 in full — the design report and my gate
note are the reasoning behind every number below
(`docs/dev/codex/brief-437-native-stack-budget-design.md`).

- `crates/rum-runtime/src/runtime/metadata.rs:4-11` — every resource default,
  including the `DEFAULT_NATIVE_STACK_LIMIT = 6 * 1024 * 1024` being replaced
  and `NATIVE_STACK_DEPTH_POLL_STRIDE = 4`.
- `crates/rum-runtime/src/runtime/eval_core/mod.rs:34-79` — the depth check,
  the strided poll, and `poll_native_stack_limit`, which measures bytes
  consumed. The mechanism is correct and is not being changed.
- `crates/rum-runtime/src/runtime/model/config.rs:3-31` — `ResourceLimits` and
  its `Default`.
- `crates/rum-runtime/src/runtime/mod.rs:70-73` — the runtime's public
  re-export surface.
- `crates/rum-runtime/src/runtime/session/session_api.rs:98-120` —
  `set_native_stack_limit` and its neighbours.
- `crates/rum-host/src/config.rs:55-117,153-212` — `HostResourceLimits` and the
  four profiles that inherit it.
- `crates/rum-runtime/src/runtime/tests/session_config.rs:3-18` — where the
  falsifier goes.
- `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:5-12` and
  `crates/rum-stdlib/src/tests/stress_corpus.rs:191-197` — the two existing
  patterns for a test that declares its own thread stack.

## The measured facts you are calibrating against

All measured by me at the brief-437 gate; do not re-measure and do not run any
stack-exhaustion probe of your own (see Hard rules).

| profile and path | native stack per level |
|---|---|
| release, strict residual call (`Power[deep, "a"]`) | 4.7 KiB (static packet 4,736 B) |
| release, user recursion (`peel[wrap[x_]] := peel[x]`) | 7.5 KiB |
| **debug, user recursion** | **20.6 KiB** |

The debug number is the one that governs `cargo test`, which runs a debug build
on a 2 MiB libtest thread. Any prediction about a test's behaviour uses 20.6
KiB, not 7.5 and not 4.7. brief-437's report priced its ripple inventory on an
unqualified "approximately 20 KiB" quoted from `PROGRESS.md:621-628`; that
figure is the debug cost and is still accurate, but the report did not say so,
so treat every per-test prediction in its §6 as unverified.

## Deliverable

### 1. The calibration constants and the derived default

In `crates/rum-runtime/src/runtime/metadata.rs`, replace the 6 MiB literal with
a derivation from named public constants:

| name | value | role |
|---|---:|---|
| `MIN_SUPPORTED_NATIVE_STACK_BYTES` | `2 * 1024 * 1024` | The smallest thread stack the default profile supports. This is the ruled embedder floor (`docs/dev/codex/brief-426-height-cache-spike.md:25-30`). |
| `NATIVE_STACK_UNPOLLED_LEVEL_RESERVE_BYTES` | `128 * 1024` | Allowance for one evaluation level between byte polls. 6.2x the measured debug worst case of 20.6 KiB. |
| `NATIVE_STACK_POLL_HEADROOM_BYTES` | `NATIVE_STACK_DEPTH_POLL_STRIDE * NATIVE_STACK_UNPOLLED_LEVEL_RESERVE_BYTES` | Covers all four levels the stride can skip. |
| `NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES` | `512 * 1024` | Host and caller frames below `native_stack_base`, plus the error and drop paths above it. |
| `DEFAULT_NATIVE_STACK_LIMIT` | `MIN_SUPPORTED_NATIVE_STACK_BYTES - NATIVE_STACK_POLL_HEADROOM_BYTES - NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES` | 1 MiB. |

Add `pub const fn native_stack_limit_for_declared_stack(stack_bytes: usize) ->
Option<usize>`: `None` below `MIN_SUPPORTED_NATIVE_STACK_BYTES`, otherwise the
declared size minus the same two reserves, with checked arithmetic. Export the
constants and the helper through the runtime's existing public surface
(`crates/rum-runtime/src/runtime/mod.rs:70-73`) and re-export from `rum-host`
alongside `HostResourceLimits`.

This is a *declared*-stack contract, not stack discovery: safe Rust has no
portable way to ask the OS how much stack a thread has, and platform FFI is
ruled out by the zero-`unsafe` line. Say so in the rustdoc rather than implying
the kernel can verify it.

### 2. The falsifier, watched to fail

Standing rule: a check nobody has seen fail is not known to work. Add
`default_native_stack_budget_is_derived_from_minimum_supported_stack` beside
`crates/rum-runtime/src/runtime/tests/session_config.rs:3-11`, asserting the
poll-headroom product, the three-term equation, that the helper returns the
default at exactly the minimum, that it returns `None` one byte below, and that
`ResourceLimits::default().native_stack_limit` equals the derived constant. It
is pure arithmetic and cannot overflow a stack.

**Procedure, and the report must show both halves.** Introduce the constants
and the test while `DEFAULT_NATIVE_STACK_LIMIT` still selects the old 6 MiB
value, run only this test, and paste the failure verbatim. Then switch to the
derived value and paste it green. A report that shows only the green run has
not demonstrated the check works.

Add a host-side assertion that `HostResourceLimits::default()` and all four
profiles (`default`, `sandboxed`, `hermetic`, `scripting`) expose the same
derived value, so a profile cannot silently pin a stale number
(`crates/rum-host/src/config.rs:85-117,153-212`).

### 3. The test ripple — run it, do not predict it

Run the full workspace suite and collect **every** failure the new default
produces. For each one, paste the verbatim failure before you change anything.

Two repairs are permitted, per failing test:

- **Declare the thread.** If the test's purpose needs that depth, run it on a
  thread with an explicit `stack_size` and set the session's budget from
  `native_stack_limit_for_declared_stack` with the *same* value. That makes the
  declaration true by construction rather than a number someone matched by
  hand. The pattern already exists at
  `crates/rum-stdlib/src/tests/iterative_traversal_depth.rs:5-12` and
  `crates/rum-stdlib/src/tests/stress_corpus.rs:191-197`.
- **Lower the depth**, but only where the depth is incidental to what the test
  asserts.

Three things are **not** permitted, and any of them is a stop:

- Raising `DEFAULT_NATIVE_STACK_LIMIT`, widening a reserve, or lowering
  `MIN_SUPPORTED_NATIVE_STACK_BYTES` so a test passes.
- Weakening an assertion — in particular, changing an expected `reclim` to
  `reslim`. If a test that documents the *logical* recursion boundary now trips
  the *byte* guard first, it has lost its coverage; give it a thread, do not
  relabel what it proves.
- Shrinking a guard-adjacent test's configured limits to fit the budget.
  brief-437 proposed taking `guard_adjacent_user_recursion_*`
  (`crates/rum-stdlib/src/tests/stress_corpus.rs:44-70`) from 96/72/120 down to
  32/24/40. Rejected: those tests are not correct behaviour this change breaks,
  they are code that passes today on 74% of a 2 MiB stack with no guard between
  it and the edge, and the repair is to give them the stack they need.

Two known candidates, both **predictions to verify, not instructions**:
`guard_adjacent_user_recursion_below_limit_succeeds` (depth 72, about 1.48 MiB
in debug) and `from_xml_depth_limit_accepts_deep_element_nesting`
(`crates/rum-stdlib/src/tests/codecs_xml.rs:305-313`), whose comment already
records this defect being worked around at the call site — retire that
workaround with a declared thread rather than adding a second one. Also check
`bounded_recursive_definition_below_native_stack_limit_succeeds`
(`crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs:489-505`,
depth 48, about 989 KiB in debug), which is marginal by prediction and must be
decided by the run.

Two ignored tests provision 8 MiB threads and then silently inherit the
default budget; wire both to the helper with their own declared size:
`crates/rum-stdlib/src/tests/stress_corpus.rs:187-207` and
`crates/rum-host/tests/stress.rs:290-324` (which hard-codes 6 MiB twice).

Rerun the release-only 2 MiB certificates in
`crates/rum-stdlib/src/tests/iterative_traversal_depth.rs`. They are designed
to pass at their stated heights; an unexpected `StackLimit` there is a
stop-loss, not an expectation to rewrite.

### 4. Documentation

- `SECURITY.md:43-60` — correct the native stack row and make the "not a crash"
  promise true by stating the precondition it depends on: the default is
  calibrated for a thread of at least `MIN_SUPPORTED_NATIVE_STACK_BYTES`, and a
  host on a smaller stack must lower the budget.
- `docs/guide/resource-limits-and-sandbox.md:13-30,74-94` — the numeric
  default, the named equation, the helper, and the precondition.
- `docs/guide/evaluation.md:261-273` — state plainly that the byte guard may
  bind before the recursion limit, so 1024 is a ceiling and not a reachable
  guarantee. Give the three measured per-level costs as the reason.
- `docs/guide/embedding.md:49-76,453-462` and
  `docs/host-embedding-and-hardening.md:52-71` — the declared-stack contract
  for embedders.
- `docs/dev/publication-audit/capability-consistency-findings.md:702-715` —
  only where it states the value as a current contract; leave historical
  measurements alone.

Do not touch `PROGRESS.md`, `CHANGELOG.md`, or any completed brief: their 6 MiB
values are historical measurements and stay as written.

## Decisions already made — do not reopen

- `DEFAULT_RECURSION_LIMIT` stays 1024, as a ceiling rather than a promise.
  With three per-level costs spanning 4.7 to 20.6 KiB a byte-derived depth
  limit would be fiction.
- `NATIVE_STACK_DEPTH_POLL_STRIDE` stays 4.
- Message tags and structured outcomes are unchanged: `reclim` for the logical
  limit, `reslim` for the byte guard, and both `EvalTerminationReason` variants
  keep their text.
- The default profile becoming shallow — roughly 48 debug and 140 release
  levels of user recursion on a 2 MiB host, against a nominal 1024 — is the
  accepted price of failing closed, not a problem to engineer around here.

## Scope

- Files you MAY modify: `crates/rum-runtime/src/runtime/metadata.rs`,
  `crates/rum-runtime/src/runtime/model/config.rs`,
  `crates/rum-runtime/src/runtime/mod.rs`,
  `crates/rum-runtime/src/runtime/session/session_api.rs` (rustdoc only),
  `crates/rum-runtime/src/runtime/tests/session_config.rs`,
  `crates/rum-host/src/config.rs`, `crates/rum-host/src/lib.rs` (re-export
  only), a host profile test file, `SECURITY.md`, and the four `docs/guide/`
  and `docs/` files named in Deliverable 4.
- **Plus any test file the gate shows failing**, modified only in the two ways
  Deliverable 3 permits.
- **The general accept-vs-stop criterion, which governs anything I failed to
  list**: a change to a *test* that gives it the stack it needs, or lowers a
  depth incidental to its purpose, is in scope even if I did not name the file.
  A change to *non-test source* outside the named list is a stop — write it in
  the report instead of making it.
- Do not touch `Expr`/`Symbol` representation, serde shapes, generated docs, or
  `Cargo.lock`.

## rum / Rust pitfalls

- The equation must be computed from the constants, not restated as a literal.
  A hard-coded `1_048_576` beside the equation is the defect this brief exists
  to remove.
- `native_stack_limit_for_declared_stack` must be `const fn` with checked
  arithmetic so a nonsensical declared size cannot wrap.
- Resumed requests capture a fresh `native_stack_base`
  (`crates/rum-runtime/src/runtime/suspension/core.rs:482-510`). Do not cache a
  process-global base.
- `poll_native_stack_limit` stays `#[cold] #[inline(never)]`. No new work on
  the default path.
- Known limit to record in the report, not to fix: the guard polls only at
  evaluation-depth increments, so a recursion running entirely inside one
  builtin advances no depth and is invisible to it for its whole descent.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test --workspace
cargo test -p rum-stdlib --release tests::iterative_traversal_depth -- --ignored
```

No microbench A/B is required and you should not run one: this changes a
constant and the rustdoc around it, adds no default-path work, and does not
touch the poll site or its frequency. Say that in the report rather than
performing the ceremony.

**Stop-loss.** Stop and report, without further tuning, if: the falsifier
cannot be made to fail on the old default; a release certificate in
`iterative_traversal_depth.rs` regresses; the same test fails twice for the
same reason after a permitted repair; or the only way to get a test green is
one of the three forbidden moves.

## Hard rules

- **Do not write, generate, or run any program whose purpose is to exhaust the
  stack.** Every per-level number you need is in this brief. A crash
  reproduction would take the test binary with it.
- Do NOT `git commit`, branch, push, or touch git config. Leave the tree
  uncommitted for my gate.
- No network. No third-party engine, oracle, or comparison corpus
  (`AGENTS.md` Testing Policy).
- No new `unsafe` in `rum-core` or `rum-runtime`. This is why the stack budget
  is declared rather than discovered.
- No default-path cost for diagnostics.
- Do not modify files outside Scope beyond the stated criterion. If you believe
  you must, STOP and write that in the report.

## Report format

Per-file changes; the falsifier's red run and green run both verbatim; the
complete list of tests the new default broke with their verbatim failures and
which permitted repair each received; verbatim `cargo fmt --check`,
`cargo test --workspace`, and the release certificate run; the note on why no
A/B was run; and any stop-loss or open question.
