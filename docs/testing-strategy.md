# Testing Strategy

`rum`'s tests verify exactly two things:

1. **`rum`'s behavior matches its own documented design.** The language guide
   (`docs/guide/`), the generated builtin reference (`docs/reference/`), and
   the host crates' rustdoc are the normative contract.
2. **Performance does not regress between `rum` builds.** Optimization work is
   gated by rum-vs-rum A/B on the microbench suites.

There is no external oracle. The project does not run black-box differential
testing against other vendors' engines, does not maintain compatibility
matrices or conformance reports, and does not commit third-party engine
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

The eight `rum-stdlib` `retention_closure` tests are required in the ordinary
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
