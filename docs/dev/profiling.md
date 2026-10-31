# Logical profiling across suspension

This is the **target contract**, approved in brief-624 and its director review
of 2026-10-20. LP01 freezes the contract and a test-only counter oracle; it
changes no production counting. The staged losses below remain defects owned
by LP02–LP08. The developer profiler does not extend the guest language or the
public embedding API.

## Logical operations and keys

One logical phase is one invocation of the operation at its instrumentation
site, for its actual input or cursor, through its continuation. Native reentry
continues that invocation. Recursive invocations, different arguments or loop
iterations, and later dispatch stages remain distinct operations. A phase key
aggregates operations; it is never an operation's identity. The target record
identity is `(epoch, owner, record)`, independent of addresses and aggregate keys.

The existing aggregate dimensions remain:

| Family | Key | Compared counters |
| --- | --- | --- |
| Runtime path | phase, source symbol | calls, ok, err, some, none, return_err, throw_err |
| Builtin | symbol, source symbol, argument count | calls, ok, err |
| Definition rule | kind, symbol, rule index | calls, ok, err |
| Rule RHS | phase, source symbol | calls, changed, unchanged |
| Definition dispatch | kind, symbol, source symbol | candidates, prefilter skips, fast matches/misses, matcher matches/misses |
| Evaluation-cache entry events | event, applicable symbol/source | logical evaluate entries, definition scans/applications, builtin applications and Module entries |

For a completed isolated program, queued answers and genuine host-driven
suspension must produce the same logical vector at every key. Setup, limits,
flags, input kinds, prompts, answer order and program must match. Collection is
exhaustive before treating an absent operation as a zero vector. Removing a
required key is a schema failure; top-N output is not a counter receipt.

Completion publishes the specified outcome exactly once. `InputRequired`
closes an executing interval and leaves a record open; it publishes no terminal
call or error. `Some` and `None` are subsets of `ok`, determined by the actual
boundary's result channel rather than its final guest value. Actual errors
contribute `err`. Return and both Throw carriers additionally contribute their
respective error subsets.

Sampling boundaries retain their meanings. `definition.rhs.finish` and
`function.apply.finish` observe Return before absorbing it.
`catch.resume_after_body` observes its incoming Throw, even when a handler later
succeeds. P0817/P0818 already have correct incoming-Throw sampling; ancestor
losses do not justify changing it.

## Lifecycle and epochs

For ordinary completed operations:

```text
calls = ok + err
some + none <= ok
return_err <= err
throw_err <= err
```

After a lifecycle boundary retires all of an owner's open records:

```text
open = 0
calls = ok + err + abandoned
cancelled <= err
```

An actual cancellation outcome contributes `err` and `cancelled`. Structural
discard without an actual result contributes `abandoned` with its reason;
it never fabricates `ok`. While execution is paused, open records have no
published terminal outcome. Equivalently, entered operations consist of
published terminal calls plus open operations; the equations above describe
completed or retired records, not a fabricated outcome for an open call.

Cancellation, Session reset/drop, failed restore, checkpoint rollback, explicit
helper discard and profiling reset must retire the applicable leases exactly
once, with no suffix effects or invented return values. Cleanup can itself
suspend and be abandoned. Invalid carrier or token pairings invalidate the
profiling receipt visibly; they cannot change guest evaluation or silently
consume another record.

Logical collection uses one enabled, process-wide epoch collector. An owner can
resume its Session on another thread without losing its records. TLS diagnostic
maps may remain physical telemetry. Comparison isolates one epoch: unrelated
profiled requests are not part of a program's vector. The epoch's rows are keyed
by logical key, not by owner, so requests profiled at the same time in one
epoch add up per key. Nested or interleaved sessions cannot inherit parked
native stacks.

Reset retires the old epoch, discharges its open records with the
profile-reset abandonment reason and creates an empty epoch. Old tokens and
queue markers retain their epoch identity and cannot publish into the new
one. Reset's own entered scopes belong to the old epoch. Snapshot neither
finishes nor restarts an open record.

Enabled diagnostic storage is separately bounded. Registry and marker storage
does not count against the guest memory ceiling. Exhausting the diagnostic cap
invalidates the epoch visibly and retires its leases without changing the guest
resource outcome. An invalid epoch cannot certify parity.

Profiling tests share the process with ordinary tests. Under the test cfg,
every test that reads or writes the process-wide profiling state holds the
one `TestLease` (`profiling/lease.rs`). While a lease is current, a thread
that has not registered with it cannot construct, open, charge or reset
the epoch, reset the lexical counters, or write a lexical counter or the
summary once-flag. A lease becomes current only after every lexical write
or reset admitted before it has landed. Production builds compile none of the
lease. The cold-load lexical profile flags, and the evaluation-cache
`ACTIVE` flag that `RumProfileReset[]` also writes, publish over their
uninitialized state only, so a value stored after a thread's first load is kept.

## Execution telemetry and durations

Telemetry is reported independently and never compared as logical parity:
pauses, executing segments, host input pauses/resumes, abandonment reasons,
instrumentation faults, Module native resume entries/max index, presence scans,
copy/allocation/sharing counts, fingerprint/reuse/cardinality data, cache
hits/misses/invalidation visits and incremental-print frequency. The legacy
`input_required` field belongs here. This classification does not excuse a
failing runtime, builtin, definition or logical entry counter.

Duration is the sum of monotonic `Instant` executing intervals while a record
is active, including child execution. Host waiting is excluded. Self duration
subtracts child intervals in the same profiling domain, once per segment.
Paused ancestors restart when their saved context actually executes. A
boundary-only decision does not acquire earlier child time retroactively.
Totals, self totals, averages and segment counts remain observable and are
never parity inputs. Declared guest timing leaves are normalized without
removing other values, trace events or ordering.

## Additive snapshot schema v2

`RumProfileSnapshot` continues to return `Null` and emit diagnostics. The target
adds a labelled schema-v2 epoch header with validity, followed by every logical
key exactly once in stable key order. Logical rows include the family's final
vector and explicit open, abandoned and cancelled accounting. Separate labelled
sections carry execution telemetry and durations. Existing ranked human timing
displays remain additional summaries; they cannot substitute for exhaustive
rows. A snapshot's own entered invocation is open while the snapshot runs.
No new guest builtin or public host type is required.

LP01's hidden native observer is available only in tests or with the existing
`test-continuation-slots` feature. It reads all storage maps and flags, including
empty maps, preserving the current field shapes. Its logical table excludes
pause and duration fields; durations and raw physical storage are retained
separately. It does not implement schema v2, manufacture open records, repair
counters or evaluate a guest projection. Reset is native after setup, and the
snapshot is taken immediately after the subject, before queue sentinels,
log/state projections or fresh-request probes.

## Frozen oracle and staged losses

The suspension oracle retains 1,115 original tuples without changing their
original columns. There are 875 executable bindings: the original 813 over
650 programs plus all 62 previously unbound non-encoding tuples. The 240
encoding-owned tuples retain their original disposition and explicit G13
ownership; LP01 executes none of them. Sixteen minimized witnesses and 48
correctness programs are separate fixtures. The 62 new bindings use 82 programs
because every cycle tuple has distinct matching and nonmatching controls.

Required tuples, bindings and actual executed receipts are independently frozen.
Exact fixture fingerprints cover setup, subject, requests, projections, timing
paths, files and trace selection. Producer fingerprints cover the owning source
bytes. The manifest also freezes all original requirement columns. Sharing
execution preserves every original tuple identity, route and variant. Both
library builds run disabled, lexical-only, timing-family, evaluation-cache and
all-detail/source modes independently. Semantic values, effects, ordered traces,
request plans, prefixes, pending ownership, queue exhaustion, fresh requests and
zero continuation-fault deltas remain gates. Missing a planned second pause is
invalid, including when an early Throw prevented it.

The indexed `logical-profile-losses.json` under the oracle inventory freezes
118,991 counter-difference cells across the five flag modes, together with
schema keys and independently executed receipts. Strings and integer vectors
are interned to avoid duplicating long routes; records retain multiplicity one.
Tag 0 is `(mode, program, delivery, source fingerprint, key vector)`, tag 1 is
`(mode, program, key, producer, consumer route, phase, phase sites/routes, owner,
resumed vector, queued vector)`, and tag 2 is the executed tuple receipt
`(mode, row, consumer route, variant, context, source fingerprints)`. Each
record's string fields index `strings`; vector fields index `vectors`. A vector
of schema keys contains string indices. The build identity is recorded in each
native execution artifact; the same frozen expectations are checked separately
in both builds. Counts are per mode and tuple attribution, not distinct kernel
sites or unique logical invocations. No duration or pause field enters a loss.

| Owner | Staged responsibility |
| --- | --- |
| LP02 | Enabled record/segment engine, exact queue boundaries, identity, epoch and lifecycle machinery; diagnostic cap and isolation |
| LP03 | Evaluator/fixed-point/head/dispatch/argument completions and resumed synchronous equality, stability and trace observations |
| LP04 | Definition RHS, all rule timing arms, matching/default phases, source aliases and dispatch decisions |
| LP05 | Assignment RHS, forwarded suspension, finish and LHS/list/association boundaries |
| LP06 | Builtin handler, control and callable completions, retaining incoming-outcome sampling |
| LP07 | Module total/body/initializers, native initializer reentry and remaining source/entry aggregation |
| LP08 | Public diagnostic schema v2 and final non-encoding counter-manifest promotion |

Current code still treats `InputRequired` as an error at many terminal hooks,
misses resumed ancestor completions and synchronous observations, loses source
context, and counts Module native reentry physically. LP01 pins the exact
current resumed/queued vectors with phase sites, source/consumer route and
owner. A changed loss, unexpected key/vector, missing execution or duplicate is
red. Each later adoption removes only its approved cells while retaining every
other owned loss. Current losses are not killed-mutant evidence. Final promotion
requires complete non-encoding parity and the owning steps' counter-only resume
mutants, outcome-classification controls, identity and lifecycle gates.
