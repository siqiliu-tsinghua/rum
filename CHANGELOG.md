# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Pre-1.0,
minor versions may include breaking changes; the project aims to follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) once it reaches 1.0.

## [Unreleased]

### Fixed

- A deep rebuilt expression no longer ends the process. The
  evaluation-stability walk descends a freshly rebuilt tree without crossing
  `Session::evaluate`, so the native-stack guard had no poll point during that
  descent and a value past the walk's reach aborted the host. The walk now
  polls the same byte budget on a stride, at the one place that can grow the
  stack — a call-cache miss, immediately before it recurses — and on exhaustion
  returns a conservative "not stable", which is always safe because stability
  only decides whether evaluation may be skipped. The value is then evaluated
  through the evaluator's own guarded path and stops in band. At physical
  height 20,001 on a 2 MiB stack the result is
  `TerminatedEvaluation["StackLimit"]` where the process previously aborted,
  and the walk still succeeds to about 5,340 levels — 30% above the ruled
  4,097. Cache contents, counters and profiling output are unchanged.
- The TOML and YAML codec depth-acceptance tests now declare the stack they
  need, as their XML sibling already did. Both were passing by consuming more
  native stack than the declared budget permits, which only became visible once
  the stability walk began enforcing it. The accepted depth is unchanged.

- Off-by-default profiling state no longer occupies the evaluation-stability
  walk's recursive frame. The walk computed a source/head profiling key and a
  start instant at the top of each level and held both across the recursive
  call. The key constructor returns immediately when profiling is disabled, so
  no work was done on the default path — but the locals still occupied about 64
  bytes of frame per level, and on a recursive walk stack bytes per level are
  what the depth contract is made of. The key is now derived inside the cold
  profiling-enabled path from the head it already has, and the start instant
  moved to a thread-local stack, taking the walk from 256 to 192 bytes per
  level. A rebuilt result of physical height 9,500 now completes on a 2 MiB
  stack where 8,008 was previously the ceiling. Profiling output, cache
  contents and counters are unchanged.

- A traversal that rebuilds a deep result no longer ends the process. The
  rebuilt tree carries fresh identities at every level, so every
  evaluation-stability cache key misses and the stability walk descends the
  whole tree natively — and because that walk never crosses `Session::evaluate`,
  the native-stack guard has no poll point during the descent. `ReplaceAll` on a
  rule that matches, `Apply` at a descending level spec, and `Replace` at all
  levels all aborted at roughly 1,300 levels on a 2 MiB stack, while the same
  operations with a rule that never matches survived, because nothing was
  rebuilt. The walk's frame was the union of every branch it can take —
  thirteen data-constructor arms, cache insertion, head-effect computation and
  the profiling tokens — even though a deep descent executes almost none of
  them. Outlining those branches took it from 811 to 256 bytes per level, so a
  rebuilt result of physical height 4,097 now completes with room to spare.
  Semantics, cache contents and counters are unchanged.
- Three release depth certificates could not fail for the reason they existed.
  The `Map` and `Apply` certificates used a level spec that selects only the
  outermost level, so they wrapped the value once and handed it to an
  already-iterative `Depth`; the `ReplaceAll` certificate replaced a symbol
  absent from the value, so no rebuild ever ran. All three now assert the exact
  rebuilt expression, and each is paired with a shallow companion that pins its
  selector's semantics on a three-level value.

- The native stack guard no longer fails open on the stack an embedder
  actually gets. Its default budget was 6 MiB against the 2 MiB a spawned
  thread gets by default, so on that stack the guard could never fire and a
  deep evaluation ended the process instead of returning a value — while
  `SECURITY.md` promised the opposite. The budget is now derived by a checked
  equation from the smallest stack the default profile supports, minus poll
  headroom and non-evaluator reserve, and `native_stack_limit_for_declared_stack`
  lets a host that provisions a larger thread derive a matching budget from the
  same number it passed to `stack_size`. A strict residual construction on a
  2 MiB stack now returns `TerminatedEvaluation["StackLimit"]` where it
  previously aborted. The default profile is correspondingly shallower —
  measured at about 4.7 KiB of stack per level for a release strict call, 7.5
  KiB for release user recursion and 20.6 KiB for debug — so `recursion_limit`
  is documented as a ceiling rather than a reachable depth.

- Deeply nested input no longer ends the process in four recursive descents.
  `FromXml` had no depth bound where `FromJson`, `FromYaml` and `FromToml` were
  bounded at 256 in the same repair that skipped it; `RegularExpression` group
  nesting, the postfix-blank chain in the parser, and the four codec encoders
  had none either. All now refuse in band, naming the limit and the byte offset
  where the existing codecs already did.
- Pattern matching no longer grows the native stack in proportion to the height
  of the value being matched. Normalizing a match candidate was recursive, so a
  single match against a deeply nested value ended the process at about height
  4,600 on the 2 MiB stack a spawned thread gets by default — and that, not the
  traversals themselves, was the ceiling `Cases`, `Count`, `Position`,
  `MemberQ`, `FreeQ` and `ReplaceAll` shared at about 4,500. A single
  `MatchQ` against a non-trivial pattern now clears height 43,000 and those
  traversals clear 20,000. Results, argument order, `Flat` and `Orderless`
  normalization, and which values are rebuilt rather than shared are all
  unchanged.

- Ten traversals now handle values far deeper than before on the 2 MiB stack a
  spawned thread gets by default: `ToString` from about 1,550 to 43,000,
  `Level` from 4,000 past 20,000, `Map` and `Apply` from 3,600 to 20,000, and
  `ReplaceAll`, `Cases`, `MemberQ`, `Count`, `Position` and `FreeQ` from
  1,100-1,450 to 4,500. Results, traversal order, level semantics and where the
  short-circuiting predicates stop are unchanged.
- The per-request memory ceiling now measures the whole session. The proxy
  walked 34 of `Session`'s 93 fields, missing every cache, the installed
  system down-values, sealed package units and declared exception types; a
  `ByteArray` was charged its decoded payload while it holds base64 text.

### Changed

- Every builtin declares how it can grow, as a required argument to every
  `BuiltinSpec` constructor, so a new builtin cannot be added without choosing.
  Declaring `NonAmplifying` or `Preflight` falsely now fails a test: eleven
  declarations were wrong when the first check was added and eighteen more when
  the second was, and the ten repaired since project their result bytes before
  allocating.
- `ToJson`, `ToYaml`, `ToToml` and `ToXml` refuse to encode a value nested
  deeper than 256, matching what their own decoders already refuse to read.


- Discarding a suspended evaluation no longer leaks the kernel state its frames
  own. Abandoning a suspension dropped its saved continuation frames without
  unwinding them, so `Block` bindings, `Module` activations and gensym
  reclamation, loop iterator scopes, `OptionValue` scopes, open `Reap` and
  `Monitor` scopes, time and memory constraint nesting, trace and capture
  checkpoints, `AbortProtect` depth, `BlockRandom` state, and partial package
  installs all survived into the next request. The leaked `AbortProtect` depth
  let evaluation continue past `Abort[]`; a leaked deadline terminated an
  unrelated later evaluation. Abandonment is now a closed protocol that every
  suspension frame must classify — a supertrait with no default body, so a new
  frame does not compile until its behaviour is chosen — and `WithCleanup` is
  its one allowlisted evaluation.

- A cancellation is no longer erased by the evaluation it was meant to stop.
  The prologue cleared a session-global interrupt flag as its first act, so a
  cancel that arrived between obtaining a handle and starting the request was
  lost. Cancellation is now request-scoped and generation-qualified: a handle
  affects only the request that produced it, is inert after that request
  completes or the session is reset, and survives its own request's prologue.

- A cancellation accepted while an evaluation is suspended for host input now
  abandons that continuation before any input is converted, wrapped, or
  consumed, returns the documented structured termination, and leaves the
  session reusable.

### Changed

- A new evaluation request arriving while a single-source input continuation is
  pending is rejected with a documented failure instead of silently replacing
  it. The pending continuation is untouched and remains resumable through
  `resume_with_input`; `queue_input_response` remains pre-supply only. A
  script-shaped suspension, which has never been resumable, is escaped with
  `reset()` or by cancelling. Superseding a suspended request is no longer an
  accident of consuming queued input, and remains unimplemented as an explicit
  operation.

- Cancellation is observed through one mechanism. The legacy `AtomicBool`
  interrupt lane, its mirror synchronization, and its `Arc`-holder arming
  heuristic are gone; `rum-stdlib`'s lazy steppers observe the request phase
  word directly.

## [0.2.0] — 2026-08-31

### Fixed

- Comparison operators are not associative, and parentheses bound a chain.
  `(a == b) == c`, `a == (b == c)` and `(a < b) < c` all flattened to the n-ary
  form, so `(1 == 1) == 1` evaluated to `True` where it should reduce to
  `True == 1` and stay unevaluated. Chains are now collected while parsing, so a
  parenthesised operand is one operand of the outer chain by construction. A
  chain of one relation keeps its n-ary head (`a == b == c` is
  `Equal[a, b, c]`); a mixed chain lowers to `Inequality[e1, op1, e2, op2, …]`,
  which carries exactly `Protected` and no hold attribute — so every operand
  evaluates exactly once and nothing is skipped. Previously a mixed chain
  lowered to `And`, which repeated the middle operand and short-circuited.
  `Inequality[a, Less, b]` degenerates to `Less[a, b]`, and an undetermined
  chain is preserved.

- Preparation no longer evaluates inside pattern regions. The policy listed the
  wrappers it protected and the list was short: `Verbatim[Blank[typeAlias]]`
  stored `Verbatim[_Integer]`, evaluating the contents of a wrapper whose
  entire purpose is not to, and a call inside `PatternSequence` ran and was
  replaced by its result. `OrderlessPatternSequence`, `KeyValuePattern`,
  `Longest`, `Shortest` and `OptionsPattern` were equally exposed. Pattern
  syntax is now named once in a registry shared with the matcher, so the two
  cannot drift apart.

- Syntax errors report the location of the offending text. Spans were computed
  per form and rendered against the whole file, so a script failing in its
  fourth form reported the position of an unrelated character in its second.
  `Get` reported no line or column at all. Spans now leave the parser in file
  coordinates, so `--file`, stdin scripts, host script requests, `Get` and
  `Needs` are all correct, and the caret in the rendered excerpt points at the
  right character.

- A definition's left-hand side is prepared consistently, so the identity a
  definition is stored under matches the one a lookup constructs. Seven
  defects shared this shape and pointed in both directions. Where preparation
  was due but skipped: `AssociateTo[af[ii], …]` with `ii = 2` installed a
  second definition under `af[ii]` and left `af[2]` untouched; `TagSet`,
  `UpSet` and their delayed forms stored a raw lhs, so the rule they installed
  could not be reached from any input; `f[iD] = 8` under `iD := 1 + 1` stored
  `f[1 + 1]`, which `f[2]` could not reach; `Unset[f[i]]` could not remove what
  `Set` had just stored; and `DownValues[f] = {…}` imported raw rule
  left-hand sides. In the other direction, `f[xH] = 9` under a `HoldFirst`
  head normalized a position that had to stay raw, so the natural `f[xH]`
  lookup missed the rule it had just installed. A held ground nested `Flat`
  call was unreachable from both its source and its flattened form, because
  candidate matching normalized `Flat`/`Orderless` while the ground index did
  not.

  Arguments are now prepared by the head's hold attributes — active positions
  evaluate once, including delayed own-values; held positions keep their raw
  subtrees — and the same preparation serves storage, removal and lookup. A
  stored identity is a state snapshot: later own-value or attribute changes
  affect future preparation, never existing definitions, so removing after
  rebinding addresses a different identity and leaves the original in place,
  while `Clear` still removes everything. `m[ww] ^= 22` with `ww = 5` now
  prepares to `m[5]`, finds no symbol able to serve as a tag, and reports
  `UpSet::nosym` instead of silently installing an unreachable rule.
  Value-table assignment is transactional: an invalid entry late in the list
  leaves the previous table intact, including across a suspension.

  The repair is enforced rather than maintained. Storage and removal travel as
  tokens whose constructors are private to a single module, so storing a raw
  left-hand side from source-language evaluation does not compile; only static
  catalog installation and whole-state restoration bypass preparation, both
  explicitly.

- `lhs := rhs /; test` evaluates its guard. Right-hand-side `Condition` guards
  were stored as part of the body and never examined, so
  `b[x_] := x^2 /; x > 0` made `b[3]` return `9 /; True` and `b[-3]` return
  `9 /; False`. All eight candidate-selecting lanes now evaluate them —
  `DownValues`, `UpValues`, `SubValues`, `DefaultValues`, `Rule`/`RuleDelayed`
  through the replacement system, `ReplacePart`, string rules, and
  `OwnValues` — with only `True` accepting and a decline indistinguishable
  from the rule not matching. `Cases` no longer collects an element whose
  guard declined. Rules that differ only by their guard now coexist and are
  tried in order, where previously the later one replaced the earlier.
  Per-lane ordering contracts are documented in `docs/guide/patterns-and-rules.md`.
- Iterator variables are visible on an assignment left-hand side.
  `Do[f[i] = i, {i, 1, 3}]` installed a single `HoldPattern[f[i]] :> 3`
  without a message; it now installs `f[1]`, `f[2]` and `f[3]`. `Table`, `Sum`
  and `Product` share the repair. `SetDelayed` resolves the lhs key and leaves
  its rhs delayed on the iterator symbol.
- `ValueQ` no longer commits side effects during its trial evaluation: the
  deterministic PRNG is restored, host print output is suppressed for the
  trial's duration, and `FileWrite` and `Process` are refused. As a
  consequence `ValueQ[DumpSave[path, x]]` now answers `False`.

### Changed

- `Session::install_system_down_value` takes a `TrustedSystemDownValueLhs`
  rather than a name and an arbitrary expression. It previously accepted any
  expression and stored it without preparation, so a caller outside the crate
  could install a definition whose identity no lookup could reach. Constructing
  a valid left-hand side is now the caller's typed obligation. The
  `EvaluatedPartSelectors` wrapper likewise no longer participates in the
  continuation protocol, which had offered a third way to build one.


- Sorting with a user comparator is `n log n`. `Sort`, `SortBy` and
  `ReverseSort` built a full all-pairs table — 639 200 comparisons at n=800 —
  and now run on an iterative, stable, resumable merge engine that stays within
  `n*ceil(log2 n)`. Ties keep their input order.
- The ordering-callback schedule is documented as unspecified: a valid strict
  weak order determines the result, while comparison count, order and direction
  are not promised. Key functions still run exactly once per entry, in input
  order.
- Arithmetic, composition and variadic chains of three or more operands lower
  to one n-ary call rather than nested binary ones: `a + b - c + d` is
  `Plus[a, b, Times[-1, c], d]`, and `.`, `<>`, `|` and `~~` flatten likewise.
  Explicitly written canonical calls stay opaque, so `Plus[a,b] + c` remains
  `Plus[Plus[a, b], c]`.
- Left-associative operator chains are bounded at 4 096 links, reported with a
  dedicated diagnostic. The limit is enforced for programmatically built syntax
  trees as well as for parsed source.
- `ValueQ` answers without evaluating when the evaluator already proves the
  target unchanged, so its cost no longer scales with session history: at
  8 000 definitions, 332.6 us to 0.76 us per call.
- Reading one value out of an association no longer materializes every entry.
  Allocations per read are now independent of width (14 at both 4 000 and
  32 000 keys, against 12 083 and 96 101 before); positional reads are flat
  across width and keyed reads sublinear.
- `StringTrim` with a list of exact string literals takes the fast path its
  single-literal form already had, 31.6x at 4 000 characters and 36.1x at
  32 000.
- Sorting symbols no longer allocates two strings per comparison. The order is
  unchanged, including for nested contexts.
- The REPL runs one full completeness check per form rather than one per
  appended line, so pasting a multi-line definition is linear in its length.
- The Jupyter kernel sleeps at its launcher's idle interval while a cell runs
  instead of spinning a core.
- The generated-symbol sweep rearms at a watermark after a pass that reclaims
  nothing, so repeated `Module` calls that retain their symbols no longer each
  pay a full mark. Reclamation is still bounded: an unreachable symbol is
  collected within one further threshold of new candidates.

- Completed the P0 tranche of a safe-Rust repository-wide algorithmic
  performance sweep. Wide
  evaluation, matching, replacement, collection, association, string, and
  polynomial paths now avoid numerous repeated scans, suffix copies, and
  per-step continuation clones; small-input paths retain adaptive linear tiers.
- Ordered, non-`Flat` multi-argument matching now advances all-`Single`
  pattern/value cursors in place instead of rebuilding the remaining vectors
  at every argument, including across suspended matcher conditions.
- Incremental `AppendTo`, association mutation, and positional `Part`
  assignment now use conservative in-place/COW fast paths when ownership and
  evaluator-cache invariants permit it.
- The core-language microbench measures elapsed time with the kernel's
  monotonic Rust `Instant` clock rather than differencing the host wall clock.
- Native ZeroMQ tests skip when the environment denies loopback access instead
  of requiring an out-of-sandbox rerun.
- Loading a script is linear in its length. The splitter re-parsed the whole
  accumulated text once per line while a form stayed unterminated, so a
  10 000-line data literal took 7.75 s and the same data on one line took
  0.01 s; it now lexes once and drives the parser form by form, taking 0.01 s.
  Each form also carries the syntax tree that parse produced, so evaluation no
  longer parses it a second time.
- A further round of complexity reductions on wide inputs, each measured
  before and after: `ToToml` and `FromToml` no longer rescan or rebuild per
  entry; `StringReplace`, `StringCases` and `StringApply` no longer copy their
  remaining input and accumulated output per element; `KeyTake`, `KeyDrop` and
  `KeyDropFrom` index their key membership tests; `ReplacePart` merges exact
  positional rules into a single pass; `Together` merges addends pairwise
  instead of renormalizing after every one; `Reap` indexes its tag buckets; and
  UpValue dispatch discovers candidates only at the positions where a stored
  tag can live, which is what made installing many UpValues on one symbol
  quadratic.

### Fixed

- `While`, `For`, `Until` and `Module` bindings with initializers no longer
  consume a native stack frame per iteration or binding. A 20 000-iteration
  loop terminated the process; loops now run iteratively and an unbounded loop
  reaches an existing deadline or interrupt instead of aborting.
- Sixteen left-associative operators no longer exhaust the native stack on long
  chains. The parser's recursion guard never saw them, because such chains are
  parsed iteratively and the depth was deferred into the syntax tree.
- `FromYaml` and `FromToml` refuse over-deep documents with a message instead
  of exhausting the native stack, matching `FromJson`. All three accept 256
  nested collections and refuse the 257th.
- `KeySort` and `KeySortBy` with a custom ordering no longer overflow the stack
  — previously at 110 and 100 keys respectively — and no longer clone the whole
  sort state per comparison.
- `StringPadLeft`, `StringPadRight`, `DecimalString`, numeric `StringIndent`,
  `StringExpandTabs` and `Fibonacci` now charge their work against the existing
  resource budgets, as their siblings already did.
- Multi-iterator `Table`, `Do`, `Sum` and `Product` charge the product of their
  iterator counts, so iterators that each pass the limit can no longer multiply
  past it. Counts that come from a variable or from an enclosing iterator are
  included; constructs that discard their inner results are not charged for
  sequential repetition.
- Comparison-chain lowering no longer recurses, so long comparison chains no
  longer overflow the stack while being lowered.

- Added the previously missing `list[[i]] = value` assignment semantics.
- Bounded recursive operator parsing and made uninitialized `Module` binding
  processing iterative. The parser's single depth limit is now three separate
  caps — delimiter nesting, operator chains, and `;`-separated statements —
  because one number could not serve all three: statement count is not nesting
  depth. `;` chains are collected iteratively in both the parser and lowering,
  and a form holding too many statements reports its own error naming the limit
  and suggesting the fix (splitting the line), rather than a nesting message.
- Wide inputs no longer abort the process. `Replace`, `ReplaceAll`, `Cases`,
  `Count`, `Position`, `DeleteCases`, `ReplaceList`, `Reap`, `StringReplace`,
  `StringCases`, `StringApply`, and `UpValue`/`SubValue` dispatch each walked
  their input by recursing once per element, rule, or definition, so a flat list
  of a few thousand entries overflowed the native stack. All of them now
  iterate, with suspension and resume preserved; the boundary is the documented
  resource limit, which refuses gracefully.
- Building an association one key at a time no longer falls off a performance
  cliff. Writes stability-checked a freshly built rule on every store, so the
  cache filled with single-use entries; crossing its cap wiped the proof the
  direct write path needs, and the slow path never re-seeded it, disabling the
  fast path permanently. Twenty thousand keys took 55 seconds and thirty-two
  thousand took over eleven minutes.
- Replaced wide symbolic `Plus`/`Times` like-term linear scans with an adaptive
  group index; the former unconditional quadratic wide path is no longer
  current.

## [0.1.0] — 2026-07-21

Initial public source release. `rum` is an embeddable, sandbox-first symbolic
term-rewriting language and runtime written in Rust.

### Language and runtime
- Symbolic expression trees with exact rational arithmetic (no floating point).
- Definitions, rules, and pattern matching, including Orderless/Flat/OneIdentity
  and optional/default patterns.
- Scoping and held evaluation, sealed packages (real, kernel-enforced
  encapsulation), lazy sequences, and structured exceptions.
- A folk-scope standard library of 642 builtins; see
  [`docs/reference/builtins.md`](./docs/reference/builtins.md).

### Embedding and sandbox
- `rum-host`: a Rust embedding facade with structured outcomes, resumable input,
  a bidirectional typed value bridge, and named capability profiles
  (`sandboxed` / `hermetic` / `scripting`).
- Per-session capability policy, always-on resource limits, opt-in request
  memory ceiling and monotonic elapsed-time timeout, crash-only per-request isolation,
  interrupt/cancel, reset, and snapshot/restore. See [`SECURITY.md`](./SECURITY.md).

### Tooling
- `rum-cli` and `rum-repl` terminal front ends, and an optional `rum-jupyter`
  adapter.

[0.1.0]: the initial public snapshot; this project has no prior public release.
