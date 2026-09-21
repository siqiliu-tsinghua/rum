# Resource Limits And Sandbox

The runtime is designed to be embedded by a caller that chooses limits and
capabilities.

Limits bound work.

Capabilities bound access to builtin families.

The host hardening document gives the implementation background:
[host embedding and hardening](../host-embedding-and-hardening.md).

## Caller-Set Limits

`HostResourceLimits` exposes these caller-set budgets:

- `recursion_limit`
- `native_stack_limit` (default: 1,048,576 bytes)
- `replace_repeated_limit`
- `evaluation_pass_limit`
- `eval_memory_limit` (optional modeled-memory ceiling per request)
- `generated_element_limit`
- `integer_work_limit`
- `iteration_work_limit`

The parser also has a source nesting depth guard.

The orderless sequence matcher has a match-depth guard.

The default CLI session uses the runtime defaults.

## Generated Elements

Generated-element limits apply to builtins that would construct too many
elements.

```text
$ cargo run -q -p rum-cli -- --eval 'Range[1000000000000]'
TerminatedEvaluation["GeneratedElementLimit"]
! General::reslim: evaluation exceeded generated element limit (100000)
```

The value identifies the reason.

The message includes the configured limit.

## Iteration Work

Iteration-work limits apply to loops and traversal-style work.

```text
$ cargo run -q -p rum-cli -- --eval 'Do[x = 1, 100000000]'
TerminatedEvaluation["IterationWorkLimit"]
! General::reslim: evaluation exceeded iteration work limit (100000)
```

The session remains usable after a recoverable terminated evaluation.

## Integer Work

Integer-work limits apply to selected arithmetic and number-theory loops.

For example, large prime-index work can trip the budget:

```text
$ cargo run -q -p rum-cli -- --eval 'Prime[1000000000]'
TerminatedEvaluation["IntegerWorkLimit"]
! General::reslim: evaluation exceeded integer work limit (100000)
```

The exact builtin coverage is in the standard library and reference limit
notes.

## Recursion And Pass Limits

`recursion_limit` guards recursive evaluator re-entry.

`native_stack_limit` guards the host thread's native stack while evaluator
recursion is active. Its default is the named calibration equation:

```text
DEFAULT_NATIVE_STACK_LIMIT
  = MIN_SUPPORTED_NATIVE_STACK_BYTES
  - NATIVE_STACK_POLL_HEADROOM_BYTES
  - NATIVE_STACK_NON_EVALUATOR_RESERVE_BYTES
  = 2 MiB - (4 * 128 KiB) - 512 KiB
  = 1 MiB
```

The four-level poll headroom follows `NATIVE_STACK_DEPTH_POLL_STRIDE`; the
non-evaluator reserve covers caller/host frames below the evaluator's base plus
error and drop paths above it. The default is calibrated for ordinary shallow
entry on a thread of at least `MIN_SUPPORTED_NATIVE_STACK_BYTES` (2 MiB). A
host with a smaller thread stack, or substantial native frames below the entry,
must lower the budget.

`native_stack_limit_for_declared_stack(stack_bytes)` derives a matching budget
for a caller-provisioned stack and returns `None` below the 2 MiB supported
minimum. This is a declared-stack contract, not stack discovery: safe Rust has
no portable way to verify the current thread's actual bounds.

The default `recursion_limit` of 1024 is a logical ceiling, not a guarantee
that every evaluation shape can reach that depth. The native byte guard can
bind first.

`evaluation_pass_limit` guards top-level fixed-point evaluation.

An evaluation cycle in the fixed-point loop is reported separately, but the
recoverable top-level value reason is currently `IterationLimit`.

Use these limits for recursive definitions, generated rewrite systems, and
hosted workloads that should not monopolize a session.

```text
$ cargo run -q -p rum-cli -- --eval 'f[n_] := f[n + 1]; f[0]'
TerminatedEvaluation["StackLimit"]
! General::reslim: evaluation exceeded native stack limit (1048576 bytes)
```

## ReplaceRepeated Limit

`replace_repeated_limit` bounds `ReplaceRepeated` and the `//.` sugar.

If repeated replacement does not stabilize within the limit, the runtime
reports an iteration-limit condition.

This is separate from traversal work limits.

```text
$ cargo run -q -p rum-cli -- --eval '1 //. x_ :> x + 1'
TerminatedEvaluation["IterationLimit"]
! General::iterlim: ReplaceRepeated exceeded iteration limit (256)
```

## Parse Depth

The parser has four independent guards.

`MAX_DELIMITER_NESTING_DEPTH = 128` bounds delimiter nesting —
`[...]`, `[[...]]`, `(...)`, `{...}`, and `<|...|>`.

`MAX_OPERATOR_CHAIN_DEPTH = 128` bounds operator-chain recursion —
right-associative chains such as `^` spines and prefix-operator runs. This
cap is sized by the native stack budget, not by the evaluator's recursion
limit: the parser spends about 4.6 KiB of stack per chain link, and a
2 MiB thread — Rust's default for a spawned thread — aborts at roughly
452 links. 128 fails gracefully well before the stack runs out.

`MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS = 4096` bounds the binary links in one
precedence-climbing lhs spine, including implicit multiplication. Exactly
4,096 links are accepted; link 4,097 reports its explicit operator token, or
the first token of the rejected operand for juxtaposition. Nested and rhs
spines start fresh. The same bound applies during lowering to syntax trees
built directly by a host, including the total operators collected from
comparison subtrees. Hitting it reports —

```text
left-associative chain exceeded limit 4096 at bytes 8193..8194
```

`MAX_STATEMENT_CHAIN_DEPTH = 256` bounds how many `;`-separated statements
a single form may hold. The parser collects `;` chains iteratively, so
statements consume neither depth budget; a chain's length is bounded by
this cap instead, counted along the whole spine so that consecutive empty
statements cannot slip past it. Hitting it reports its own error —

```text
statement chain exceeded limit 256 at bytes 0..4321; split the statements across separate lines
```

— and the advice is literal: script sources are split into forms line by
line, so writing the statements on separate lines parses and runs the same
program.

All four source guards fail closed at parse time: overflow is a syntax failure
with a source span, not a normal evaluated expression. A host-built syntax tree
that bypasses parsing receives the same syntax failure during lowering.

## Match Depth

The current orderless sequence matcher has `MAX_MATCH_DEPTH = 512`.

When the match-depth guard trips through a recoverable top-level evaluation,
the terminated reason is `PatternMatchRecursionLimit` and the message tag is
`General::matchreclim`.

The patterns chapter and generated reference define the supported pattern
interactions; the former `Flat` plus `Orderless` plus `Optional` debt is closed.

## Interrupt And Cancel

`rum-host` exposes `HostSession::cancel_handle()`.

The host obtains a handle before starting evaluation and can call `cancel()`
from another thread.

The evaluator checks the cancel flag at cooperative poll points.

A cancelled evaluation returns `TerminatedEvaluation["Cancelled"]` on the
recoverable value path.

The handle is invalidated by `reset()`.

## Elapsed-Time Timeout

`HostSession::eval_with_timeout(source, duration)` installs a per-evaluation
deadline measured with Rust's monotonic `Instant`, not the civil wall clock.

When the deadline is exceeded, the recoverable value path returns
`TerminatedEvaluation["TimeConstraint"]`.

The same mechanism is also used by timeout-aware host adapters.

## Capability Classes

Every builtin is classified by capability in the generated reference.

The current capability classes are:

- `Pure`
- `FileRead`
- `FileWrite`
- `Clock`
- `Random`
- `Diagnostic`
- `Process`
- `Interactive`

Most language and rewrite work is `Pure`.

Filesystem readers such as `Get`, `Needs`, `FindFile`, and `FileNames` are
`FileRead`.

Filesystem writers such as `Save` and `DumpSave` are `FileWrite`.

Wall-clock and time-observing builtins such as `AbsoluteTime`, `Timing`,
`AbsoluteTiming`, `RepeatedTiming`, `TimeConstrained`, `Pause`,
`EvaluationData`, and `TestReport` are `Clock`. `AbsoluteTime` reads the civil
wall clock; elapsed-time operations use Rust's monotonic `Instant`. The shared
capability class controls access to time effects, not the choice of clock used
to measure an interval.

`AbsoluteTime[]` is the current-time reader and requires `Clock`. `DateList` and
`DateString` are pure UTC calendar conversions (`DateList[ts]`,
`DateString[ts]`, or `DateString[{y, mo, d, h, mi, s}]`) and remain available
under a `{Pure}` policy.

Entropy-effect builtins are `Random`: `RandomBytes` and `RandomPassword` draw
directly from the host entropy provider, and `ReseedRandom[]` reseeds the
deterministic session PRNG from that provider. `SeedRandom` takes an exact
integer seed and is deterministic/`Pure`.

General file utilities are deliberately outside the kernel; the filesystem
surface is the module system and session persistence only.

Instrumentation and resource-disclosure builtins such as `RumProfileSnapshot`,
`RumProfileReset`, `MemoryInUse`, and `MaxMemoryUsed` are `Diagnostic`.

`Process` is reserved for future downstream process-control integrations and
has no core builtins today.

`Interactive` covers builtins such as `Input` and `InputString` that may ask the
host for input or suspend a request pending a host response.

## Sandboxed Host Configuration

`HostConfig::sandboxed()` installs a `{Pure}` policy.

Sandboxed sessions are deterministic: results depend only on session state and
source. The deterministic PRNG is session state, starts from the fixed default
seed, and remains available to `Pure` random builtins. Grant `{Pure, Clock}`
when scripts need time, or `{Pure, Random}` when scripts need host entropy.

That denies every non-allowed builtin at dispatch.

Dispatch enforcement matters: runtime-constructed calls such as
`Symbol["Get"][...]` or parsed strings through `ToExpression[...]` are still
checked when the builtin dispatch is reached.

A denied call emits a `General::sandbox` message and returns `$Failed`.

Sandboxed host configuration also omits the file I/O and entropy providers, so
provider absence is a construction-level layer after capability dispatch.

Fine-grained policies can allow, for example, `{Pure, FileRead}` for read-only
file access, `{Pure, Clock}` for time, `{Pure, Random}` for entropy, or
`{Pure, Diagnostic}` for instrumentation without file access.

Role or identity mapping is the caller's job. `rum` supplies the capability
partition, policy object, and dispatch enforcement.

## Memory Ceilings And Hard RSS Limits

`HostResourceLimits::eval_memory_limit` installs a modeled-memory ceiling for
each request. A breach returns `TerminatedEvaluation["MemoryLimit"]`, classifies
the termination as `MemoryLimit`, emits `General::memlim`, and leaves the
session usable. Nested `MemoryConstrained` frames use the lower of their local
limit and the request ceiling while active.

While a memory limit is active, every existing per-expression resource
checkpoint checks the session-plus-expression model against that limit.
Projected-byte prechecks also check every time they are called. A single
construction larger than the limit is rejected before allocation only where
such a precheck exists; otherwise its excess is detected at the next existing
checkpoint that observes it. This does not bound transient allocations between
checkpoints or memory omitted from the model.

The model counts deterministic session/expression structures. It is not an
allocator meter or process RSS measurement, so a native host that requires a
hard process-wide memory bound must enforce one outside the kernel. One pattern
is to monitor process RSS with platform code and trigger a previously obtained
`cancel_handle()` when the process exceeds its budget.

This keeps `rum-core`, `rum-runtime`, and `rum-host` free of platform-specific
RSS polling while allowing embedders to combine an in-kernel deterministic
ceiling with an OS-level hard boundary.

Deliberate contract note: a tripped memory constraint is CONSUMED by
`MemoryConstrained` itself (it returns its fallback or `$Aborted` with
`General::memlim`). That local frame does not surface as a top-level
`TerminatedEvaluation[...]` value because the construct that requested the
bound is also its handler. A breach of the outer request ceiling does surface
as `TerminatedEvaluation["MemoryLimit"]`.

## Poison Recovery

Public `HostSession` entrypoints catch panics from guest execution.

A caught panic returns `HostFailureCode::Panic` and poisons the session.

Further calls fail with `HostFailureCode::Poisoned` until the host calls
`reset()`.

Stack-overflow aborts are not caught by this boundary. The parser and matcher
depth guards prevent the covered source-evaluation paths that would otherwise
reach the known deep-source crash class.
