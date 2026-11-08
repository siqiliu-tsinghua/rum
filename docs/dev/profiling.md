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

`RumProfileSnapshot` continues to return `Null` and emit the legacy diagnostics.
When lexical profiling is configured, each valid call additionally emits one
line before the legacy lexical snapshot header:

```text
rum-profile schema-v2 {"schema",2,"label",L,"epoch",E,"validity",V,"mode",M,"logical",{ROW,...},"telemetry",{"faults",F,"rows",{TELEMETRY_ROW,...}},"durations",{"basis","epoch-accrued+observing-thread-running","rows",{DURATION_ROW,...}}}
```

The expression is guest list syntax built as text, never evaluated. `L` is the
label: no argument or a nonstring argument uses `"snapshot"`; a string argument
uses its text, including an empty string. Excess arity retains the existing
invalid-call behavior and emits no snapshot. Strings escape quotes, backslashes,
newline, CR and tab as `\"`, `\\`, `\n`, `\r`, `\t`; other characters are literal.
There is no raw newline in the schema line. Legacy label formatting is unchanged.

`E` is the captured epoch's decimal id, or `Null` if none exists. `V` is
`"valid"`, `"invalid"` or `"absent"`. A failed initial allocation publishes its
sticky failed id, `"validity","invalid","reason","diagnostic-cap","mode",M`,
with empty row lists. Only initial allocation failure carries that reason field;
freeing diagnostic memory does not repair it, and reset clears it. An absent
epoch is never created by publication. An existing invalid epoch publishes its
retained rows and fault count without a reason field; it cannot certify parity.
The telemetry and duration wrappers remain present even when their rows are `{}`.

`M` lists the effective family gates, joined by `+` in this order: `lexical`,
`builtin-timing`, `rule-rhs-timing`, `definition-rule-timing`, `cache`. If none
is active it is `"none"`. Legacy detail flags do not enter this token. A
lexical-configured but deferred process still emits the line, with absent epoch
and mode `none` when every family is inactive. With lexical unconfigured, no
schema line is emitted. A snapshot's own entered invocation stays open while
the snapshot runs **where its family's gate is enabled**; its builtin row has
`open = 1` and no terminal outcome. The header states those effective gates.
Snapshot neither finishes nor restarts any open record.

Rows have these exact forms:

```text
ROW = {"<key>","vector",{v,...},"open",o,"abandoned",a,"cancelled",c}
TELEMETRY_ROW = {"<key>",{v,...}}
DURATION_ROW = {"<key>",{v,...}}
```

Every logical key appears exactly once in each section, in the same bytewise
order of encoded key text. There is no top-N, row cap or deduplication of distinct
semantic identities. Ranked human timing displays remain additional summaries.

Symbols encode their components injectively, including semantic identity:

```text
symbol(s) = "s(" ESC(s.context()) "," ESC(s.name()) "," (s.identity() decimal | "n") ")"
source(None) = "-"
source(Some(s)) = symbol(s)
ESC: '%'->"%25"  '|'->"%7C"  ','->"%2C"  '('->"%28"  ')'->"%29"  '#'->"%23"

runtime|{phase}|{source}
builtin|{symbol}|{source}|{arity}
definition-rule|{kind}|{symbol}|{index}
rule-rhs|{phase}|{source}
definition-dispatch|{kind}|{symbol}|{source}
counter|{name}
cache|{NAME}
cache-symbol|{event}|{symbol}
cache-source|{event}|{symbol}
```

The trailing backtick in stored contexts is literal: for example,
``builtin|s(System`,RumProfileSnapshot,n)|-|1`` and
``definition-rule|down_value_rhs_eval|s(Global`,witness,n)|0``.
Context/name concatenations, interner ids, addresses and thread identities never
define key identity or ordering. Phase, kind, event and scalar names are the
closed recorder identifiers; arity and index are decimal text.

The publication union contains every native `Epoch::rows()` row, every stored
synchronous side-index row, Module aliases and eight fixed scalar defaults.
Native `runtime|module.eval.total|<source>` rows project once into
`counter|eval_module`, summing calls and open/abandoned/cancelled across sources;
each present source also projects `cache-source|eval_module_by_source|<symbol>`
with calls, accounting and native durations. There is no second alias writer.

The defaults are `counter|definition_rhs_eval`, `counter|eval_module`,
`cache|EVALUATE_ENTRY_COUNT`, `cache|DOWN_VALUE_SCAN_COUNT`,
`cache|DOWN_VALUE_APPLICATION_COUNT`, `cache|SUB_VALUE_APPLICATION_COUNT`,
`cache|BUILTIN_APPLICATION_COUNT` and `cache|EVAL_MODULE_ENTRY_COUNT`.
For an existing epoch each is present with vector `{0}`, accounting zero,
empty telemetry and durations `{0,0}` until replaced by its actual scalar or
Module projection. Defaults never add duplicate rows. A second owner of one
encoded key visibly faults the epoch. The unused native `RuleRhs` variant also
faults: native `ok/err` cannot stand for RHS `changed/unchanged`.

Logical vector widths are runtime 7, builtin and definition-rule 3, rule-rhs 3,
definition-dispatch 6 (in the family table's order), and counter/cache/
cache-symbol/cache-source 1. Synchronous side rows have zero lifecycle accounting.
Native telemetry has 14 fields: `input_required`, pauses, segments, then eleven
abandonment reasons in native `Retire` index order. Side telemetry is the one
`input_required` field; defaults and alias-only rows have empty telemetry.

Native durations and Module source aliases have `{nanos,self_nanos}`. All side
rows have four stored slots: runtime/source `{nanos,self_nanos,0,0}`, RHS/scalar
`{nanos,0,0,0}`, dispatch `{prefilter_nanos,fast_match_nanos,compiled_matcher_nanos,
rhs_substitution_nanos}`, and cache zeros. Reserved slots are not logical fields.
The duration basis **epoch-accrued+observing-thread-running** is the native fold:
accrued intervals plus the observing thread's currently running intervals.
Other threads' in-flight intervals are outside that basis. After closure the
native and rendered duration vectors agree exactly.

Publication captures the epoch once, copies each index under its own lock, and
encodes and formats after unlocking. No cross-family atomicity is claimed.
Validity and faults are read after copying, so a reset during the copy invalidates
the captured receipt. Temporary copies and text are bounded by the retained
epoch's storage and are not charged to its 16 MiB diagnostic budget a second
time. Publication adds no public builtin, host type or guest evaluation.

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
the surviving 76,460 counter-difference cells across the five flag modes
(88,795 records including 1,592 keys and 875 receipts per mode), together
with schema keys and independently executed receipts. Strings and integer vectors
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

The staged adoptions are complete. LP01 pinned the exact initial resumed/queued
vectors with phase sites, source/consumer route and owner; LP02–LP08 removed
exactly their owned cells, each proved SAME per cell. `InputRequired` now
closes an executing interval and leaves its record open instead of publishing a
terminal error, and resumed ancestor completions, source context and Module
native reentry are recorded logically. Every surviving cell is still red on a
changed loss, unexpected key/vector, missing execution or duplicate.

The public schema-v2 snapshot publishes the epoch-wide union described above;
`RumProfileSnapshot` still returns `Null` and retains the legacy diagnostics.
The counter promotion and its frozen oracle remain independent of publication.
