# Evaluation

Evaluation starts after source has been parsed and lowered.

At a high level, a session:

1. parses source into syntax
2. lowers syntax into core expressions
3. evaluates expressions to a fixed point
4. returns the final value and any runtime messages

The runtime audit records the current evaluator model in more detail:
[runtime evaluation audit](../runtime-evaluation-audit.md).

## Fixed Point

Top-level evaluation repeats until the expression stops changing.

User definitions re-enter the same evaluator as ordinary source.

```text
$ cargo run -q -p rum-cli -- --eval 'f[x_] := x + 1; f[3]'
4
```

The first expression stores a delayed downvalue.

The second expression evaluates through that downvalue and then through exact
arithmetic.

A common memoization idiom is `f[x_] := f[x] = rhs`: the delayed definition
computes a value once for a matching argument, then stores an immediate
definition for that concrete call.

```text
$ cargo run -q -p rum-cli -- --eval 'calls = 0; fib[0] = 0; fib[1] = 1; fib[n_Integer] := fib[n] = (calls = calls + 1; fib[n - 1] + fib[n - 2]); {fib[8], fib[8], calls}'
List[21, 21, 7]
```

The counter increments for `fib[2]` through `fib[8]`; the second `fib[8]` reads
the stored immediate value.

## Call Pipeline

For calls, the evaluator currently follows this staged shape:

1. evaluate the head
2. build a dispatch plan from builtin metadata, attributes, and head traits
3. let special forms inspect raw arguments
4. evaluate ordinary arguments according to hold attributes
5. apply attribute-driven preprocessing
6. try upvalues attached to evaluated arguments
7. try downvalues on the evaluated head
8. try subvalues on nested heads
9. try builtin dispatch
10. try pure-function application
11. reconstruct the call if nothing fires

This order is important for rules, held forms, and embedding outcomes.

## Held Arguments

Hold attributes control which arguments evaluate before a builtin or definition
sees them.

`Hold` has `HoldAll`, so it holds its argument unevaluated.

`ReleaseHold` then evaluates the held expression.

`Abs` is `Listable`, so it threads over lists.

```text
$ cargo run -q -p rum-cli -- --eval 'x = 3; {Hold[x + 1], ReleaseHold[Hold[x + 1]], Abs[{-2, 3, -4}]}'
List[Hold[Plus[x, 1]], 4, List[2, 3, 4]]
```

The held expression keeps `x` as a symbol.

The released expression evaluates `x + 1` to `4`.

The listable call maps `Abs` over each list element.

## Attribute Families

Attributes are data on heads.

The current core includes useful coverage for:

- hold behavior: `HoldAll`, `HoldFirst`, `HoldRest`, `HoldAllComplete`
- threading behavior: `Listable`
- associative and ordering behavior: `Flat`, `Orderless`, `OneIdentity`
- sequence behavior: `SequenceHold`
- selected protection and numeric metadata used by the builtin catalog

The implemented attribute set is deliberately bounded to these documented
families. The generated reference lists the exact attributes for each builtin
entry.

Attribute-driven behavior is part of evaluation, not syntax lowering.

## Evaluation Order

The evaluator is head first.

After the head is stable, the call's argument evaluation is controlled by the
evaluated head's hold behavior.

Definitions see evaluated arguments unless a hold attribute or special form
keeps an argument raw.

Upvalues run before ordinary downvalues for ordinary calls unless
`HoldAllComplete` blocks them.

Subvalues run after downvalues once the nested head is stable.

Definition assignment prepares the identity it stores without applying the
outer call. `Set` evaluates its right-hand side first; `SetDelayed` keeps its
right-hand side raw. Both then evaluate a callable left-hand-side head, take
one snapshot of that head's attributes, and prepare arguments from left to
right under the same hold, `Evaluate`, `Unevaluated`, `Sequence`, `Flat`, and
`Orderless` rules used for ordinary call lookup. Active positions complete
evaluation, including delayed own-values and nested definitions. Held
positions retain their raw subtree. Preparation classifies pattern syntax from
the same head registry as the matcher. Pattern binder names and deferred
`PatternTest`/`Condition` predicates remain raw, while their pattern operands
and typed blank constraints are prepared. `PatternSequence`,
`OrderlessPatternSequence`, `KeyValuePattern`, `Longest`, and `Shortest`
traverse their pattern structure without evaluating literal calls inside it;
nested typed constraints are still prepared. `Verbatim` contents and
`OptionsPattern` targets are entirely raw. Opaque assignment forms such as
`HoldPattern`, `MessageName`, and `Default` are not user-evaluated.

List-destructuring `Set` validates the complete left/right list shape before
storing anything. It then stores leaves from left to right, descending into
nested lists in that order, and returns the complete evaluated right-hand side.
If a target asks for input, its pending store resumes once before the remaining
stores continue. Completed stores remain committed if a later target fails or
the request is cancelled or abandoned; pending stores do not run on abandonment.

The resulting definition identity is a snapshot. Later own-value or attribute
changes affect identities prepared by later assignments and lookups, but do
not rewrite definitions that are already stored.

Tagged assignments use that same prepared identity. `TagSet` normalizes its
explicit tag, evaluates an immediate right-hand side, and then prepares the
left-hand side; `TagSetDelayed` skips right-hand-side evaluation. Ownership is
classified only after preparation. `UpSet` and `UpSetDelayed` likewise infer
their tag symbols from the prepared left-hand side. Consequently, if preparing
an `UpSet` left-hand side leaves no valid tag symbol, the assignment reports
`UpSet::nosym` (or `UpSetDelayed::nosym`) and stores nothing. `TagUnset`
prepares its left-hand side under current state before classifying and removing
the explicitly tagged identity.

`Unset[lhs]` and its `lhs =.` syntax prepare an ordinary definition lhs by the
same rules before removing it. Removal therefore addresses the identity named
by current state: if `i = 2`, then `f[i] = 1; Unset[f[i]]` removes `f[2]`, but
rebinding `i` to `3` before `Unset[f[i]]` addresses `f[3]` and leaves the
stored `f[2]` definition intact. `Clear[f]` remains the owner-wide operation
that removes every identity stored for `f`.

Assignments to `OwnValues`, `DownValues`, `UpValues`, and `SubValues` are
definition stores. Every imported rule lhs is prepared under the same identity
rules before installation. Replacement is transactional: the runtime parses,
prepares, and validates the entire incoming table and rechecks protection
before clearing the old table. If any entry is invalid, or preparation
suspends and later fails validation, the existing table is left intact.

`AssociateTo` and `KeyDropFrom` keep dedicated symbol and `Part` mutation
paths. For a definition-lvalue fallback such as `f[i]` or `f[p][q]`, they
prepare the target once, read it by continuing the captured outer-call plan
after argument preparation, and store a successful update under that same
prepared identity. Active target arguments and the selected outer definition
therefore run at most once, including when either phase suspends, while held
target positions remain raw.

Assignment-style `Part` mutations have a stricter callback order. For the
binary heads `AddTo`, `SubtractFrom`, `TimesBy`, `DivideBy`, `ApplyTo`,
`AppendTo`, and `PrependTo`, ordinary `HoldFirst` preparation evaluates the
second argument once before target selection. When a mutation reaches target
selection, the `Part` selectors evaluate exactly once from left to right. Only
after the final selector does the mutation read the owner's removal,
protection, and value state; both the selected paths and their current values
come from that post-selector value. The update evaluates once, and protection
is checked again immediately before the result is stored. `AssociateTo` and
`KeyDropFrom` use the same selector-then-owner order for `Part` targets after
their evaluated mutation specification passes validation.

`Increment` and `Decrement` return the selection from before their update.
`PreIncrement`, `PreDecrement`, and the seven binary update heads return the
updated selection. Selector diagnostics describe the evaluated selector value,
not the held source expression that produced it.

## Ordering Callbacks

The binary ordering callback accepted by `Sort`, the three-argument form of
`SortBy`, and `ReverseSort` is expected to define a deterministic strict weak
order over the values or evaluated keys it compares. For a callback satisfying
that contract, the ordered result is defined; the implementation's comparison
schedule is not. The implementation may select, omit, reorder, or reverse
comparison pairs. Comparator call count, direction, order, and relative timing
are therefore not a sequencing API, and programs must not depend on comparator
side effects occurring in a particular schedule.

`Sort` and custom `SortBy` preserve input order when the callback does not say
that either tied value strictly precedes the other. `SortBy` evaluates its key
specification exactly once per input element, in input order, before comparator
sorting begins. `ReverseSort` produces the complete ascending arrangement and
then reverses it, so reversing also reverses the input order of a tied run.

If a selected callback comparison requests input, evaluation preserves that
pending comparison and the partial sort state. Resuming continues that
invocation, applies its result, and proceeds without restarting the sort.

## Messages

Messages are runtime outputs attached to an evaluation.

The CLI prints the final value and then the emitted message lines.

```text
$ cargo run -q -p rum-cli -- --eval 'Check[Message[General::stop]; 1, fallback]'
fallback
! General::stop: message emitted
```

`Message` emits `General::stop`.

`Check` sees that a message was emitted and returns the fallback.

`Quiet` suppresses selected message output. If a message is quieted, `Check`
does not treat it as a visible message for fallback selection.

## Limits As Values

Several evaluator limits are converted into
[`TerminatedEvaluation`](../reference/builtins.md#terminatedevaluation) values
when the top-level request path handles them as recoverable evaluation
outcomes.

```text
$ cargo run -q -p rum-cli -- --eval 'Do[x = 1, 100000000]'
TerminatedEvaluation["IterationWorkLimit"]
! General::reslim: evaluation exceeded iteration work limit (100000)
```

The value says which budget was tripped.

The message gives the concrete limit and category.

Host APIs expose the same rendered value in successful outcomes for these
recoverable cases.

## Cycles And Pass Limits

Fixed-point evaluation has guards.

If the same expression reappears in the same fixed-point loop, the runtime
reports an evaluation cycle.

If evaluation keeps changing without stabilizing past the configured pass
limit, the runtime reports an evaluation pass limit.

Both map to the `IterationLimit` terminated-evaluation reason in the current
top-level value path.

The resource chapter lists the caller-set limits.

## Recursion

Recursive evaluator re-entry is guarded by `recursion_limit`.

Native stack headroom is guarded separately by `native_stack_limit`; unbounded
definitions can therefore terminate as `TerminatedEvaluation["StackLimit"]`
before the logical recursion counter reaches `recursion_limit`.

The default recursion limit of 1024 is a ceiling, not a reachable guarantee.
Native stack consumed per level depends on the evaluation path and build
profile: calibration measured about 4.7 KiB for a release strict residual call,
7.5 KiB for release user recursion, and 20.6 KiB for debug user recursion. The
byte guard may therefore bind at very different logical depths, and can bind
well before 1024 on the default 1 MiB evaluator budget.

Those limits are separate from the fixed-point pass limit and the pattern-match
depth guard.

Hosts should set resource limits according to their workload, especially when
evaluating untrusted source.

## Suspension

Host-driven input is part of the evaluator model.

`Input` and `InputString` can suspend a request. The host then resumes the same
request with input instead of replaying the whole source string.

This includes guest files reached through `Get`, `Needs`, `BeginPackage`
dependencies, and `DeclarePackage` autoloads within a resumable evaluation
request. The paused form resumes before the rest of the file and its caller
complete. File forms and resumes keep the enclosing resource budget and
absolute deadline; they do not start a fresh timeout.

Buffered output inside an unsettled capture is provisional. At an input pause
it remains available to the capture and nested observers, but is absent from
the host response. Completion settles it once using that capture's normal
consume, filter, or release policy; abandonment discards the provisional
suffix. An inner capture completing cannot expose output still owned by an
outer capture.

`EvaluationData` and a `TestReport` input capture own buffered prints, messages,
and raw trace events while their bodies are unsettled. `EvaluationData[expr,
properties]` applies the prepared property selector once after capture completion,
including after resume. A successful `TestReport` input consumes its prints and
messages into the case result. An input evaluation that fails keeps the existing
copy-and-release policy: its output is copied into the case result and remains
public after the capture settles. A wrong expected value or false `SameTest`
alone does not release a successfully evaluated input's captured output.
Expected-output and `SameTest` evaluations belong to the surrounding request.

`Check`, `Quiet`, and `ConfirmQuiet` hold messages provisionally while their
bodies are unsettled; their prints pass through. Their normal message matching,
suppression, and release rules run on completion. A `Check` fallback or
confirmation information expression runs after that body capture has settled.
Abandonment runs neither callback. `Trace`, `TraceDetail`, and `TracePrint` own
the raw trace channel until their capture settles; `TracePrint`'s rendered
lines follow the print channel's rules. `Reap`, timing, constraints, and
`Monitor` own no buffered output.

Output outside every active capture becomes public at the next response, in
order within each channel. Pause and terminal responses are cumulative:
already exposed output remains a prefix, with no duplicated elements in the
cumulative arrays. Prints, messages, and raw traces remain separate channels.
The embedding chapter shows the `HostEvalOutcome::InputRequired` path.

## Structured Exceptions

Structured exceptions are a typed, hierarchical alternative to tagged
`Throw`/`Catch`.

An exception type is a string registered in a single-parent hierarchy.

```text
$ cargo run -q -p rum-cli -- --eval 'RegisterExceptionType["NetError"]; RegisterExceptionType["Timeout", "NetError"]; ExceptionTypes[]'
List[Rule["NetError", "Exception"], Rule["Timeout", "NetError"]]
```

Registration is required. Every registered type descends from the implicit
root `"Exception"`. Re-registering a type with a different parent is refused
and leaves the registration unchanged.

`ThrowException[type, payload]` raises. The payload is an `Association`.
`CatchExceptions[expr, handlers]` runs `expr` and matches each raised
`Exception[type, payload]` object against the handler rules.

A handler registered for a parent type catches its registered subtypes.

```text
$ cargo run -q -p rum-cli -- --eval 'RegisterExceptionType["NetError"]; RegisterExceptionType["Timeout", "NetError"]; CatchExceptions[ThrowException["Timeout", <|"ms" -> 30|>], {Exception["NetError", p_] :> {"caught-by-parent", p}}]'
List["caught-by-parent", Association[Rule["ms", 30]]]
```

Handlers are tried in list order. Within a handler, the nearest matching
ancestor binds. A handler list that matches nothing re-raises the exception
outward. `CatchExceptions[expr]` without handlers catches any exception and
returns the exception object.

An uncaught exception is an in-band value at the request boundary, not a host
failure. It becomes a `Failure["UncaughtException", …]` result carrying the
type and payload, plus a `ThrowException::nocatch` message.

```text
$ cargo run -q -p rum-cli -- --eval 'RegisterExceptionType["NetError"]; ThrowException["NetError", <|"url" -> "x"|>]'
Failure["UncaughtException", Association[Rule["Type", "NetError"], Rule["Payload", Association[Rule["url", "x"]]]]]
! ThrowException::nocatch: Uncaught Exception["NetError", Association[Rule["url", "x"]]] returned to top level.
```

Throwing an unregistered type fails closed the same way: a message plus a root
`Exception` carrying the original type and payload.

The two channels are separate. `Catch` never catches a `ThrowException`, and
`CatchExceptions` never catches a classic `Throw`. `WithCleanup` still runs its
cleanup when an exception unwinds through it. `Enclose` does not confine
exceptions.

The registry is session state. It is captured and restored with the loader
snapshot like other definitions.

## Lazy Sequences

A lazy sequence is an opaque value that describes elements without computing
them. Generators build one; realizers force as many as you ask for.

`LazyRange` is the basic generator. It is infinite, and it prints as a
descriptor, not as elements.

```text
$ cargo run -q -p rum-cli -- --eval 'LazyRange[1, 1]'
LazyList[LazyRange[1, 1]]
```

`LazyList[...]` is not a `List`. It does not match `_List`, and no ordinary
list operation forces it — it stays symbolic until a realizer touches it.

```text
$ cargo run -q -p rum-cli -- --eval 'MatchQ[LazyRange[1, 1], _List]'
False
$ cargo run -q -p rum-cli -- --eval 'Total[LazyRange[1, 1]]'
Total[LazyList[LazyRange[1, 1]]]
```

The realizers are `Normal`, `Take`, `First`, `Rest`, and `Part` (`[[i]]`).
`Take`/`First`/`Part` pull a bounded number of elements and return an ordinary
`List` or element, so they are safe on an infinite sequence.

```text
$ cargo run -q -p rum-cli -- --eval 'Take[LazyRange[10, 5], 4]'
List[10, 15, 20, 25]
$ cargo run -q -p rum-cli -- --eval 'First[LazyRange[7, 2]]'
7
```

`Normal` realizes everything. On an infinite sequence it fails closed at the
iteration work limit — it returns a `TerminatedEvaluation`, it does not hang.

```text
$ cargo run -q -p rum-cli -- --eval 'Normal[LazyRange[1, 1]]'
! General::reslim: evaluation exceeded iteration work limit (100000)
TerminatedEvaluation["IterationWorkLimit"]
```

The `Lazy*` transformers stay lazy — they take a lazy sequence and return a
lazy sequence, so they compose without computing anything. `LazyTake` is what
caps an infinite sequence down to a finite one; `Normal` of that finite
sequence then terminates naturally.

```text
$ cargo run -q -p rum-cli -- --eval 'Normal[LazyTake[LazyMap[Function[x, x^2], LazyRange[1, 1]], 5]]'
List[1, 4, 9, 16, 25]
```

`LazySelect` keeps elements whose predicate returns `True`. Under a bounded
realizer it pulls the source until enough have passed.

```text
$ cargo run -q -p rum-cli -- --eval 'Take[LazySelect[EvenQ, LazyRange[1, 1]], 4]'
List[2, 4, 6, 8]
```

`LazyTable` is a lazy `Table`; its body is held and evaluated once per realized
element. `LazyNestList` iterates a function; `LazyFoldList` is a lazy
`FoldList` that emits the initial value first.

```text
$ cargo run -q -p rum-cli -- --eval 'Normal[LazyTable[i^2, {i, 1, 5}]]'
List[1, 4, 9, 16, 25]
$ cargo run -q -p rum-cli -- --eval 'Normal[LazyTake[LazyNestList[Function[x, 2 x], 1], 6]]'
List[1, 2, 4, 8, 16, 32]
$ cargo run -q -p rum-cli -- --eval 'Normal[LazyFoldList[Plus, 0, LazyTake[LazyRange[1, 1], 5]]]'
List[0, 1, 3, 6, 10, 15]
```

The combinatorial sources — `LazySubsets`, `LazyTuples`, `LazyPermutations` —
enumerate finite but often enormous spaces. They never materialize the whole
space, so a bounded `Take` is cheap even when the full result would be
astronomically large. Their order matches the eager `Subsets`/`Tuples`/
`Permutations` builtins exactly.

```text
$ cargo run -q -p rum-cli -- --eval 'Take[LazySubsets[Range[100]], 4]'
List[List[], List[1], List[2], List[3]]
$ cargo run -q -p rum-cli -- --eval 'Normal[LazyTuples[{{1, 2}, {3, 4}}]]'
List[List[1, 3], List[1, 4], List[2, 3], List[2, 4]]
$ cargo run -q -p rum-cli -- --eval 'Take[LazyPermutations[Range[20]], 2]'
List[List[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20], List[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 20, 19]]
```

Lazy sequences are pure. A realizer runs under the iteration work ceiling and
observes cancellation at each source pull and after each callback returns,
before applying its value. An unbounded pull fails closed. If a callback asks
for input, the suspended realization resumes into its remaining work and then
completes its `Normal`, `Take`, `First`, or `Part` consumer; completed callbacks
and source pulls are not repeated. A lazy value is an ordinary immutable
expression, so it is captured and restored with the session snapshot and
re-derives identically. There is no mutable cursor: `Rest` returns a new lazy
tail, and realizing the same value twice recomputes it.
