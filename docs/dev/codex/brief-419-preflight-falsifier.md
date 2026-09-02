# brief-419 — make `Preflight` mean something, then tell me who is lying

Shape: **implementation**, and deliberately a small one. Worker: **cdx (Codex)**.

Gap B, step 2. Not your brief-414 draft for this step: that draft went straight
to repairing "every inventory site whose maximum output bytes can be computed
before construction", which is a scope the worker decides. This brief measures
first.

## Why this shape

`MemoryGrowthContract::Preflight`'s own documentation says "a conservative
result-size projection **can** run before construction"
(`metadata.rs:301-302`), and the type's doc adds that `Preflight` "is the
attachment point for the projector functions added by the subsequent
memory-accounting work" (`:293-295`). That was honest of brief-417. It also
means the declaration is currently a statement of possibility, not of fact.

Director-measured: **136 builtins are declared `Preflight`**, and there are
**18 calls to `enforce_active_memory_limit_for_projected_bytes`** in the entire
tree outside its own definition (`constraints.rs:175`). So at most eighteen of
the hundred and thirty-six actually project bytes before allocating, and I do
not know which, nor how badly the rest amplify.

brief-417 taught the lesson I am applying here. Its static inventory did not
find the eleven wrong declarations; the falsifier did, in one run, with numbers.
Scoping a hundred-odd repairs from a source reading would repeat exactly the
mistake that instrument avoided — so build the instrument, run it, and report.

## Goal

A `Preflight` builtin that returns materially more than its arguments **without
having projected those bytes first** fails a test, and I get the list of every
builtin that does so, with numbers.

**Repair nothing.** That is brief-420, and I will scope it from your list.

## Context pointers

- `AGENTS.md` first.
- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:189-333` — the existing
  `NonAmplifying` falsifier, its arming predicate, and
  `assert_non_amplifying_result`. Extend this machinery; do not build a second
  one beside it.
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:174-186`
  — `enforce_active_memory_limit_for_projected_bytes`, the single byte-projection
  choke point, with 18 callers.
- `docs/dev/codex/brief-414-memory-accounting-gaps.md`, appended report §3 —
  the inventory. Useful for interpreting what you find; **not** a work list here.
- `crates/rum-runtime/src/runtime/tests/session_memory.rs` — brief-415's
  invariant. If you add a `Session` field for this, that macro will stop
  compiling until you classify it. That is the structure working; give it an
  accurate row rather than routing around it.

## Deliverable

### 1. The `Preflight` falsifier

Under the same arming as the existing one (test builds through the
`test-builtin-memory-growth` feature, or `RUM_TEST_BUILTIN_MEMORY_GROWTH`, and
**not** a production memory ceiling — brief-418 settled that), after a
`Preflight` handler returns `Ok`:

- if the result's proxy bytes exceed the arguments' by more than the existing
  allowance, **and** the handler made no byte projection during the call, that
  is a failure naming the builtin and both figures.

The cheap way is a monotonic counter on `Session` bumped inside
`enforce_active_memory_limit_for_projected_bytes`, snapshotted before the
handler and compared after. A handler that returns early, passes through
symbolically, or produces something small is untouched by this rule, which is
the point: it only fires when real growth happened with no projection behind it.

Suspension matters here and is easy to get wrong: a handler that suspends and
resumes must be judged across the whole call, not across one fragment. Say what
you did about that. brief-417 kept its contract on the dispatch-stage frame; do
the same for the counter snapshot if that is what correctness needs.

### 2. Evidence the falsifier can fire

A checked-in `#[should_panic]` test with a deliberately unprojected `Preflight`
builtin, in the shape of brief-417's two. An instrument that has never been
seen to fail is not known to work — this project has been caught by that four
times, and brief-417's own falsifier only became credible when it was watched
failing.

### 3. The list

Run the armed workspace suite and report **every** builtin the falsifier
catches: name, result bytes, argument bytes, and the test that surfaced it.
Sort by the ratio. This list is the deliverable I care about most; brief-420 is
scoped from it.

If the list is empty, do not celebrate — say so plainly and then explain how
that squares with 136 declarations against 18 projection sites, because those
two numbers cannot both be right without an explanation. Some `Preflight`
builtins may bound their output by element count or integer work instead of
bytes; if that is what you find, say which, and say whether a count bound
discharges a *byte* promise. My view is that it does not — bounding counts
while bytes are the quantity at risk is the original defect of this whole line —
but I want your reading with evidence, not my assumption echoed back.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
- `crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs` — the
  counter bump only. **Do not change any limit, the stride, or what the ceiling
  admits.**
- `crates/rum-runtime/src/runtime/model/session.rs` and
  `crates/rum-runtime/src/runtime/model/mod.rs` — the counter field
- `crates/rum-runtime/src/runtime/session/session_state.rs` — if the frame or
  the reset path needs it
- `crates/rum-runtime/src/runtime/tests/session_memory.rs` — the field's
  inventory row
- `crates/rum-runtime/src/runtime/tests/memory_growth_contract.rs` and
  `crates/rum-runtime/src/runtime/tests/mod.rs`
- this brief, to append your report

If the mechanism needs a file outside this list, **STOP and report** before
editing it.

Files you MUST NOT touch: the growth classifications themselves, any builtin
handler, `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`,
`ResourceLimits` defaults, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`
except appending your report, `.git`, git config.

## Hard rules

- **Repair nothing.** No handler gains a projection in this brief, and no
  declaration changes. If the list is long, that is the finding, not a reason
  to start fixing. A builtin you cannot leave alone is a stop-loss.
- **No default-path cost.** Disarmed, the check stays a single cheap predicate
  that returns immediately. The counter bump inside the projection helper runs
  on the armed path only if you can do it for free; if it would cost the
  unarmed path, gate it and say how.
- **No behaviour change.** Nothing refuses anything new. No message changes, no
  limit changes, no stride change.
- **No new `unsafe`.** Do not weaken an existing test.
- Do NOT `git commit`, branch, push, stash, checkout, or touch git config.
  Leave everything in the working tree. Do not touch `stash@{0}`.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network; pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **This host can bind loopback**; the nine native ZeroMQ tests genuinely run
  here — I observed 9 of 9 at the brief-418 gate. If your sandbox denies it and
  you need `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, say so and mark those nine results
  unobserved.
- **Append your report to this brief file.** brief-414 and brief-418 both left
  theirs in the dispatch log only; the first cost a wasted dispatch.

## Verification (MANDATORY — you run it, iterate to green)

```
cargo fmt --check
cargo test --offline --workspace
cargo build --offline --release
```

The workspace suite is expected to FAIL while the falsifier is catching real
liars — that is deliverable 3, not a defect. Report those failures as the list.
Then, so the tree can be committed, put the falsifier behind an explicit
opt-in for the catches you found: a named allowlist of the builtins currently
known to be unprojected, checked in and sorted, so the suite is green while the
list is visible and brief-420 can delete entries as it repairs them. **Do not
silence a catch by reclassifying the builtin or loosening the allowance.**

No microbench A/B is required if the disarmed path gains nothing measurable;
say so explicitly, and if you had to add anything to the unarmed path, stop and
tell me instead.

## Report format

1. What changed — per file, line ranges, one-line why.
2. How the falsifier works, including what you did about suspension.
3. Deliverable 2's proof, verbatim.
4. **The list**, sorted by ratio, with the test that surfaced each.
5. Your reading of the 136-vs-18 arithmetic, with evidence.
6. The allowlist you checked in, and its exact contents.
7. Verification — every command, verbatim output.
8. Stop-loss / open questions.
