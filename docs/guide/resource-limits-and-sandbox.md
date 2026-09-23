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

### Occurrence memory model

The proxy counts **occurrences**, not unique allocations. Each retained
expression root, head, argument, rule, cache entry, and traced frame root is
charged where it occurs. Two edges to the same shared subtree each charge the
whole subtree. Interned symbol text is likewise charged at each modeled symbol
occurrence. No `Arc` identity set or sharing discount is part of the model.
The two accepted-request wrappers remain explicit exceptions: their interrupt
state allocation is charged through `request_interrupt_control` alone.

`ByteCount[expr]` measures an expression with the following formula. Write
`S(T)` for Rust's `size_of::<T>()`, `E(x)` for the expression charge, and
`I(n)` for the length of an integer's signed little-endian bytes (at least one).
Sizes depend on the target layout; this is a structural proxy, not a portable
serialization size.

| Expression | Charge |
| --- | --- |
| Small integer | `S(Expr)` |
| Big integer | `S(Expr) + S(BigInt) + I(n)` |
| Rational | `S(Expr) + S(Rational) + I(numerator) + I(denominator)` |
| String | `S(Expr) + String.capacity()` |
| Symbol | `S(Expr) + context.len() + name.len()` |
| Call | `S(Expr) + E(head) + sum(E(argument)) + (args.capacity() - args.len()) * S(Expr)` |

`ByteArray` uses the call formula, including the **base64 String's capacity**
and its `Expr` slot, not decoded byte length. Malformed text and non-string
payloads use the same structural formula. Shared expression allocation headers
and allocator metadata are omitted.

The session total begins with `S(Session)`, including its inline bookkeeping,
then adds each classified field's contribution. An independent test oracle
reports those contributions without sampling the peak, clearing caches, or
building indexes. Its exhaustive schema has 52 walked fields and 42 exemptions
in a test build (41 exemptions in a production build, which omits the test-only
gensym sweep counter). Adding an unclassified field fails compilation of the
tests. The rules for the field contributions are:

| Fields or payload | Contribution beyond `S(Session)` |
| --- | --- |
| `builtins` | Live entry count times `S((Symbol, BuiltinSpec))`; static catalog text omitted |
| Attribute, builtin-dispatch, stable-symbol and message-count caches; installed ownvalue and GC symbol sets | Container capacity times entry size, plus context/name text for every live key |
| Symbol call-effect cache | Capacity times key/value size, plus each live key's symbol text |
| Call-stability cache | Capacity times key/value size, plus each retained head and argument occurrence and argument spare slots |
| Association cache | Entry-vector capacity times entry size, plus each live index's hash capacity times `S((u64, usize))` and collision-vector capacity times `S(usize)`; stale entries remain charged until the owner prunes them |
| `current_context`, string vectors/deque | String capacities; vectors/deque also charge capacity times `S(String)` |
| Path vectors | Path **lengths**, plus vector capacity times `S(PathBuf)` |
| Context, evaluation-stack, dynamic-ownvalue, module, option, monitor and trace frames; message and selector vectors; sealed units | Each live element's inline size and specified payload, plus outer vector spare slots |
| `attributes` | Each live key's formatted symbol length |
| `definitions`, `options`, `syntax_information` | Each key's formatted symbol length plus definition, options, or expression charge |
| `declared_stubs`, `removed_symbols` | Each live symbol's `S(Symbol)` and context/name text; unused hash capacity omitted |
| `symbol_generations` | Each `S(((String, String), u64))` and the two String capacities |
| `once_values` | Each key's String capacity and value expression |
| Installed downvalues | Hash capacity times entry size, key text, each rule pair's expression occurrences and nested rule-vector spare slots |
| Message definitions | Formatted symbol length, tag **length**, immediate and delayed expression occurrences |
| Exception-type registry | Each `S((String, Option<String>))`, name capacity and optional parent capacity |
| Trace depths | Vector capacity times `S(usize)` |
| Reap scopes | Outer vector capacity times `S(ReapScope)`, pattern and handler expressions, bucket tags and values |
| Input request | When present, `S(RuntimeInputRequest) + prompt.capacity() + 1` |
| Pending evaluation request | When present, an additional `S(PendingEvalRequest)` |
| Pending/resuming suspension and abandonment worklist | Each owner, saved live stack entries, frame queue capacity, frame allocations and traced expression/symbol occurrences; abandonment vector spare slots too |
| File/entropy providers and request interrupt control | Two word-sized Arc counters plus the concrete object's immediate `size_of_val` |
| Print sink | Concrete boxed object's immediate `size_of_val` |
| Resource limits | An additional `S(ResourceLimits)` |

A definition charges `S(SymbolDefinition)`, immediate/delayed ownvalues and
ownvalue guards, and each rule's inline size, left/right expressions and guards.
Options charge `S(SymbolOptions)`, entry-vector capacity times
`S(SymbolOptionEntry)`, each option name's formatted length and its value
expression. An option scope additionally charges its own inline size and target
name. Context frames charge their two String capacities and saved context-path
vector. Module activations charge their gensym vector's capacity and live symbol
text. Dynamic frames charge symbol text and optional value. Monitor frames
charge display/callback expressions and optional display String capacity.
Messages/selectors charge their String capacities. Trace events charge their
expression occurrences and any symbol String capacity. Sealed units charge
package String capacity, held wrapper, two symbol vectors and each private
index entry's `S((String, Symbol))`, String capacity and symbol text.

Definition dispatch metadata charges its inline size when present. A built
index adds its inline size, hash capacity times `S((u64, usize))`, and collision,
non-ground and wrapped-ground vector capacities times `S(usize)`. Each optional
tagged upvalue/subvalue index adds its own inline size, hash capacity and
collision-vector capacity. Merely measuring a lazy index does not build it.

A suspension charges `S(SuspendedEvaluation)`, each saved live stack entry's
`S(EvaluationStackEntry) + E(expr)`, frame-queue capacity times the frame Arc
slot size, and, for every queued frame, two word-sized Arc counters and its
concrete inline size. `FrameRootTrace` then contributes `E(expr)` for each
expression root and `S(Symbol) + context.len() + name.len()` for each bare
symbol root, including expressions inside pending throws/exceptions. Repeated
roots are charged again. The traversal protocol is shared with rooting; the
oracle's arithmetic is independent of the production memory walker.

Several conservative terms intentionally charge inline storage again: resource
limits, pending request wrappers, frame expressions, definition expressions,
and option structures. These are part of the existing ceiling, not deductions
to make when interpreting it as physical memory.

Deliberate omissions include allocator overhead, hash control bytes and B-tree
node overhead, process-global interner storage, unobserved temporaries, opaque
host objects' nested heaps, bigint limb spare capacity, definition matcher
payloads and rule-vector spare capacity, dispatch shallow-signature heaps,
Reap bucket/group storage and tag-position indexes beyond the expression roots,
and saved suspension-stack spare capacity. A traced frame's nested container
capacity, strings and other non-root heap payloads are omitted unless its
concrete inline size already covers them. Paths and message-definition tags
use length rather than capacity. These omissions are boundaries of this proxy;
they are not claims that the omitted storage is free.

**Arithmetic specification:** every charge, product and intermediate sum must
saturate conservatively on overflow. Exact `usize::MAX` is distinct from an
invalid (information-losing) total. An invalid total projects to `usize::MAX`
and must reject every ceiling, including `usize::MAX`. Releases must either
retain non-lossy information or leave the total explicitly invalid and too high;
subtracting from a saturated number is insufficient. An over-release or a
non-representable release also invalidates the result. Only a fresh independent
walk can re-establish an exact total. Synthetic tests exercise this rule without
large allocations. The current production walker still contains unchecked
intermediate sums/additions (which can panic or wrap). Its saturated `usize`
also loses the invalid-state distinction, so `projected > limit` can admit an
overflow when the ceiling is `usize::MAX`. These are recorded gaps against the
specification, not changes to runtime enforcement in this step.

Accounting metadata is charged too: inline bookkeeping grows `S(Session)` or
its containing owner, and added heap storage and spare capacity receive a
contribution. The accounting route includes the per-owner ledger and its total.
Existing omissions do not authorize an exemption for new accounting metadata.

Armed checkpoints read a maintained total for all modeled Session owners, with
no session traversal, stride or adaptive sampling. Maintenance runs only while
a memory limit is active. Each arming seeds every owner once from its actual
retained state, including roots retained before entry. Repeated local
`MemoryConstrained` entries therefore repeat that cold walk. Mutation hooks
reconcile changed owners; replacements and map updates can still walk the
affected owner. Evaluation-stack pushes, Sow appends, buffered diagnostic
appends and Monitor display replacement charge their incoming occurrences and
modeled capacity changes without revisiting an existing append prefix. Shared
expressions remain charged per occurrence, and retained capacity remains
charged after removals. Resume, abandonment and rollback reconcile actual
ownership transfers; a queued resume frame remains charged during its callback.
Checked ledger arithmetic stays invalid after overflow or over-release until a
fresh cold seed. The production payload-weigher arithmetic gaps above remain.

`MemoryInUse[]` always samples the complete authoritative walker.
`MaxMemoryUsed[]` returns the high-water mark sampled by observations and armed
checkpoints, not a continuously observed allocator peak. Maintaining Session
owners does not close the separately documented transient-builder omissions or
certify individual builtin `Incremental` declarations.

Explicit retention scopes add a separate **R** term to armed admission:
expression checkpoints use `S + W + R`, and projected-byte checks use
`S + projected bytes + R`. R counts leased local expression occurrences and
their vector spare capacity. Shared children count once per occurrence.
`MemoryInUse[]`, `MaxMemoryUsed[]` and `ByteCount` exclude R. Session accountant
controls and registry/bridge capacity are ordinary S storage. A frozen receipt's
separate control allocation stays in its R complement.

The lease protocol currently supports one branded expression accumulator per
scope. A callback bridge exposes the complete live prefix before guest code,
including an unarmed prefix when a later nested constraint arms. Callback
outcomes own the context and its leased prefix until adoption or scope exit.
A raw callback error cannot be extracted and retained across another step.
Only scope completion publishes the outgoing expression. Normal return,
errors and unwinding release local receipts. A real input suspension moves the
accumulator into a kernel-owned traced frame: its expressions become existing
S roots, and its spare capacity stays in R. Resumption charges any working copy while the
frozen occurrence remains owned; abandonment releases on actual destruction.
These mechanisms are proved on a test-only builtin. No production accumulator
has been converted and no production retention region is certified.

The kernel's option loop invokes only reentry types registered on that Session
by the host/stdlib installer. An unregistered type produces
`General::optreentry` and `$Failed`. This enforces membership, not the behavior
of a registered implementation. External direct calls to `OptionReentry::resume`
are outside that guarantee.

## Poison Recovery

Public `HostSession` entrypoints catch panics from guest execution.

A caught panic returns `HostFailureCode::Panic` and poisons the session.

Further calls fail with `HostFailureCode::Poisoned` until the host calls
`reset()`.

Stack-overflow aborts are not caught by this boundary. The parser and matcher
depth guards prevent the covered source-evaluation paths that would otherwise
reach the known deep-source crash class.
