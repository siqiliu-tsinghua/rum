# Testing Strategy

`rum`'s tests verify exactly two things:

1. **`rum`'s behavior matches its own documented design.** The language guide
   (`docs/guide/`), the generated builtin reference (`docs/reference/`), and
   the host crates' rustdoc are the normative contract.
2. **Performance does not regress between `rum` builds.** Optimization work is
   gated by rum-vs-rum A/B on the microbench suites.

There is no external oracle for rum's language behavior. The project does not
run black-box differential testing against other vendors' engines, does not
maintain compatibility matrices or conformance reports, and does not commit third-party engine
outputs. This is a permanent policy; `DISCLAIMER.md` records its history.

## Test Layers

Use the lowest layer that can prove the contract being tested.

1. **Crate-local unit tests.** Own small helper invariants, planners,
   parser/formatter pieces, matcher branches, normalization functions, codec
   helpers, and host facades. Prefer direct helper tests when they make a
   high-risk policy branch reachable without large evaluator setup.
2. **Runtime regression tests.** Own evaluator/session semantics, state
   mutation, messages, held evaluation, suspension/resume, and `rum`-native
   request behavior, asserted against the documented contract.
3. **Host and adapter contract tests.** Own the embedding lifecycle in
   `rum-host` (interrupt/cancel, monotonic elapsed-time timeout, panic isolation and
   poison recovery, `reset()`, snapshot/restore round-trips, typed
   `HostValue`, capability sandbox, `Send`/concurrency contract) and the
   protocol surface in `rum-jupyter`. These are the layers a host depends on;
   coverage here is prioritized.
4. **Integration and stress tests.** rum-native corpora that exercise the
   system end-to-end: deep recursion and nesting, large rewriting workloads,
   package loading, resource-limit trips (recursion/iteration/parse-depth/
   match-depth `TerminatedEvaluation` tags), and cancellation under load.
   This is the current growth area; see `TODO.md`.

## Coverage Axes

Do not restrict suites to happy paths. For each documented function or
mechanism, choose cases across the axes that matter for it:

- canonical examples from the reference documentation
- atom, empty, singleton, nested, and large-enough structural boundaries
- symbolic, held, unevaluated, and delayed inputs
- option defaults, explicit overrides, invalid option names, invalid values
- invalid arity, invalid specs, message tags, and inert residual expression
  shape
- stateful behavior: definitions, attributes, contexts, messages, options,
  removed-symbol identity, package loading, and session-local caches
- interactions with core mechanisms such as `Part`, `Heads`, pattern matching,
  replacement, `Quiet`, `Check`, `Failure`, and suspension/resume
- documented limits and deliberate design boundaries, asserted as such rather
  than hidden in generic failures

A small, well-tagged set that spans these axes is more useful than a large
pile of similar happy-path examples.

## Documentation Is the Contract

Behavior worth keeping is documented; documentation is what tests cite.

- The builtin reference is **generated** from the `rum-stdlib` catalog
  (`BuiltinSpec`), and a drift test regenerates it and fails if the committed
  reference disagrees — so any contract change is visible as a reviewable
  documentation diff.
- Do not treat accidental behavior as supported just because one test passes.
  If a behavior matters, promote it into the guide/reference (or an explicit
  limits note) before depending on it.
- When an intentional behavior change lands, the same change updates the
  documentation and the affected tests.

## Performance Gates

- `scripts/core-language-microbench.sh` is the timing baseline: a labeled
  suite of small core-language statements producing per-label `timings.tsv`.
  An A/B is two runs — base build and candidate build — compared per label.
- `scripts/core-memory-microbench.sh` is the memory panel (retained / peak /
  transient / post-cleanup, per workload, fresh process each).
- Any change touching an evaluation hot path requires an A/B (see
  `AGENTS.md`); a regression on the target label's same-tier neighbors
  rejects the change even if the target improves.
- Timing rows must come from the suite's in-kernel `AbsoluteTiming`, which uses
  Rust's monotonic `Instant`. Never measure elapsed work by differencing
  `AbsoluteTime` or another civil/wall clock: clock correction can slew or step
  inside a sample.
- Run timing gates on an otherwise idle machine. A competing CPU-heavy process
  invalidates accept/reject decisions even though the monotonic clock itself is
  sound. Allocation and operation counters may still prove that a mechanism
  fired, but they do not replace a valid timing pass.
- Keep profiling/diagnostic env vars OFF for timing numbers; change one variable
  per A/B; persist raw output to files before parsing. Run the benchmark scripts
  in the normal workspace sandbox; timing does not require elevated execution.

## Coverage Measurement

Use `cargo-llvm-cov` for line/branch coverage when auditing test depth.
Coverage buildout is prioritized by surface, not by count: host lifecycle
first, then error/resource-limit paths, then breadth across the builtin
catalog. Do not target a test count; target documented-surface coverage.

## Fixture Admission Checklist

Before adding a regression fixture, identify:

- the documented contract it proves (guide section, reference entry, or
  rustdoc item);
- the test layer (unit, runtime regression, host/adapter contract,
  integration/stress);
- the coverage axis (canonical, boundary, negative, stateful, interaction,
  limit);
- the comparison mode (value, held shape, messages, side effects,
  request/suspension, resource-limit outcome, performance).

If the contract is not yet documented, document it first or record it as
explicitly unspecified — do not encode accidental behavior as a fixture.

The `rum-stdlib` `retention_closure` tests are required in the ordinary
crate and workspace gates, including with default features disabled. They
reconcile the whole workspace source tree, cross-check TSV obligations against
live memory-growth declarations and catalog routes, and check compiled Rust
fixtures with canonical call identities and a restricted ownership/effect
grammar. Unknown syntax, dispatch, macros and conditional bindings reject;
negative fixtures pin the diagnostic kind, target, source span and witness.
The gate also prevents promotion removal or redirection and checks exact,
exercised trusted contracts and their dependency fingerprints.

This gate currently certifies only the accepted fixture protocol: no production
region is covered and the production promotion inventory is empty. Pending
catalog rejections, option-frontier edges and proposed boundaries are explicit
unresolved obligations, never exemptions or certificates. The accepted fixture
demonstrates a leased accumulator across a loop, callback-result adoption and
outgoing ownership transfer; it does not establish production lease bounds,
callback-private allocation limits, destructor stack safety or whole-program
Rust soundness. Those require their own contracts and validation.

The rejection census adds two diagnostic populations without changing those
admission rules. `EDGE_ALL` enumerates every indexed production definition
once, including conditional alternatives; `EDGE_N_FRONTIER` starts at every
live N attachment and follows only the checker's exactly resolved edges.
Unresolved attachments retain their catalog spans. Traversal follows proved inherent methods and stops at unresolved dispatch,
closures and unexpanded macros; a source index is not a semantic call graph.
Rows retain the full reason, canonical definition (or an explicit unresolved
attachment), span, target, root set and witness. Counts deduplicate on
`(definition, site, kind, target, reason)`; raw root incidence is separate.
The reason tables group variable path/identity suffixes, while the full exact
reasons remain in separate tables. `effects::check` first-failure results are
reported separately: they are certification blockers, not edge frequencies.
An independent Python TSV parser recomputes the counts and reads every source
span. It does not supply a second Rust resolver or a coverage certificate.

The required compiler oracle checks a sample of those Rust resolution claims
and concrete scalar type claims against stable Rust 1.85.0. This is the
2026-09-23 ruling's distinction: checking Rust facts about our own repository
with the compiler that builds it is not the banned comparison of rum behavior
against other vendors' WL engines. Every compiler run uses a scratch source
copy under `target/brief-535/`, offline, with unchanged manifests and lockfile.
Adding an argument to a claimed target must produce an `E0061` primary error
at the claimed call's exact file and line. Mutating a different same-name
definition must produce no error there, and must produce an error at that
other definition's own control call. An unrelated build failure proves neither.

Type probes use generic unification and reject both wrong scalar types and
coercible mismatches. The direct `assert_type::<T>(expr)` form itself permits
argument coercion, so an inferred invariant nominal carrier freezes the type
before the expected type is introduced; a compiled array-reference-to-slice
control demonstrates why this guard is necessary. The harness also challenges
swapped resolution claims and missing negative controls. Instrument failures
fail the gate. All sampled agreements and disagreements are saved and printed;
a checker disagreement is a finding to investigate, never a pruned sample or
an automatic change to admission. Neither the sample nor the census establishes
whole-program Rust soundness or production retention coverage.

The A1 constructor stage separates exact Rust identity (nominal definition and
concrete type arguments, tuple/array/slice shapes, and reference layers) from
retention state. Checked parameters, resolved return signatures, and named
field/pattern projections supply provenance. An explicit local annotation is
checked against its initializer; it is not evidence of an ownership receipt.
The diagnostic layer can know `Result<Expr, EvalError>` at `Ok(expr.clone())`
while the effect checker still rejects the method call. Borrowing a runtime
owner likewise establishes no charge or mutation permission.

Only the standard `Some`, `None`, `Ok`, and `Err` constructors are newly
admitted. Namespace selection precedes their rules, including aliases and
prelude paths. Unit variants and variant patterns have separate obligations.
Named fields and variant payloads preserve nested raw, callback, and retained
states; a field move leaves siblings live. Exhaustive matches, returning
`let`-else branches, and explicit returns still require cleanup and compatible
ownership states. Guards, shadowing, unknown cleanup, method syntax, `?`,
closures, macros, and additional operator semantics remain outside this stage.
The existing primitive operator rules are unchanged. Concrete type substitution
and elided reference layers are deliberately specialized; this is not trait,
lifetime, or general Rust inference.

Trusted-use keys include the exact type instantiation. Two compiled scalar
instances of the fixture's `keep` function challenge stale, wildcard, missing,
and unused records independently. This adds no production exemption or
promotion; the five persistent leaf inventory entries remain fixture-exercised.
The A1 compiler oracle checks every recorded fixture type fact with an
invariant carrier, and enumerates every new constructor resolution in the
production census. Stable MIR distinguishes constructor items from ordinary
same-named functions, while independently called same-name negative definitions
are mutated in source copies. Test-only sites within indexed bodies are
explicitly observed by compiling the relevant tests without executing them.
Every agreement, disagreement, compiler diagnostic, and mutation is retained;
an instrumentation failure fails the gate. These checks establish Rust facts,
not retention certificates. Production coverage remains zero.


The A2 method stage proves a restricted inherent selection from independently
checked nominal receiver provenance: parameters, named fields and pattern
projections, consistent local annotations, and resolved return signatures.
Candidate receiver order is the original type followed by shared and mutable
autoref at each built-in reference dereference. Reference matches record a
reborrow. The canonical impl owner, visibility, unconditional definition and
`Self` substitution must agree. Earlier trait candidates are excluded using
lexical scopes and the indexed trait definitions and impls; an inherent method
wins only within its own receiver slot. Opaque imports, unknown trait bounds,
generated candidates, user `Deref`/`DerefMut`, unsizing, arbitrary self types and
unproved mutable or borrowed moves reject. No trait dispatch is certified.

Generic instances are deliberately narrower than Rust: only reached, closed,
unbounded scalar type substitutions are installed. Impl parameters must be
positional; method parameters require explicit arguments. Each body is keyed
by all substitutions, and recursion closes only on that exact key. Nominal,
unknown and growing substitutions reject instead of using a depth limit. In
particular the public generic `OptionReentry` wrapper remains unproved. A
signature supplies a type fact, never a retention summary: newly selected
method bodies and their helpers are checked automatically with full witnesses.

The required A2 oracle enumerates every newly admitted method claim, including
closed instances, without sampling. Mutated target methods carry unique
deprecation markers and caller functions deny that lint, so stable rustc must
identify the selected definition at the exact method token. This keeps
upstream dependencies compilable when observing a downstream call. Direct
self recursion suppresses that lint; those sites additionally require `E0061`
from a signature mutation and a note at the claimed definition. Every claim
also has a distinct same-named negative definition with an independently
exercised control call; no error may occur at the original site in that run.
Compile-pass controls run first, and all disagreements and raw compiler output
are retained. Oracle fingerprints include source, harnesses, inventories,
manifests, toolchain, feature set and claim files. Reusing an unchanged run
replays its evidence. These are Rust selection facts only: production retention
coverage remains zero and no production region is promoted.
