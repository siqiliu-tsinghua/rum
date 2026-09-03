# Security Policy

## Reporting a vulnerability

`rum` is an independent, personal open-source project (see [DISCLAIMER.md](./DISCLAIMER.md)).
If you find a security issue, please report it privately by email to
**liusq@tsinghua.edu.cn** rather than opening a public issue.

Please include enough to reproduce it: the input script, the configured
`HostConfig` / capability policy and resource limits, the observed behavior, and
what you expected. You will get a good-faith, best-effort response. This is a
small project with no formal embargo, bounty, or CVE process; fixes land in the
public git history.

## Scope and supported versions

`rum` is pre-1.0 and source-available; the supported version is the current
`main`. `rum` is a **language kernel and embedding library**, not a network
service or an application — its security model is about evaluating *untrusted
`rum` scripts* safely inside a trusted Rust host.

## Security model

`rum` is sandbox-first. Every session runs under a host-chosen policy.

### Capabilities

Each builtin declares a capability (`Pure`, `FileRead`, `FileWrite`, `Clock`,
`Random`, `Diagnostic`, `Process`, `Interactive`). A session's
`CapabilityPolicy` is a default-deny allowlist checked at a single dispatch
checkpoint before any builtin runs. `HostConfig::sandboxed()` allows only
`Pure`, installs no file provider, and installs no entropy source — so a
sandboxed session is deterministic and cannot read the clock or host entropy,
touch the filesystem, request host input, or terminate the host. The internal
package-loading paths (`Get`/`Needs`, and `BeginPackage`/`DeclarePackage`
dependencies) are additionally gated at the effect, so they fail closed when
`FileRead` is not allowed.

`Protected` is **not** a security boundary — it is a redefinition mistake-guard
and a Wolfram-Language-conformance feature. Isolation comes from the capability
policy plus the resource limits below, not from `Protected`.

### Always-on resource limits

Every session enforces these by default; they bound the *shape* of a
computation:

| Limit | Default |
|---|---|
| Recursion depth | 1024 |
| Native stack | 1 MiB evaluator budget |
| Iteration work | 100 000 |
| Generated elements | 100 000 |
| Integer work (`Factorial`, `Power`) | 100 000 |
| Parse nesting depth | 128 |
| Pattern-match depth | 512 |
| `ReplaceRepeated` | 256 |
| Evaluation passes | 256 |

Exceeding one yields an in-band `TerminatedEvaluation[...]` value, not a crash,
provided the native-stack contract is met. The default native-stack budget is
calibrated for ordinary shallow entry on a thread whose caller declares at
least `MIN_SUPPORTED_NATIVE_STACK_BYTES` (2 MiB). A host using a smaller stack,
or entering evaluation beneath substantial caller frames, must lower
`native_stack_limit`. Stack capacity is caller-declared because safe Rust has
no portable API for discovering the current thread's actual stack bounds.

### Evaluating untrusted scripts — required host settings

The always-on limits bound the *shape* of a computation (depth, count,
iterations), **not** the elapsed time or memory of a single expensive
operation. To evaluate untrusted input safely you MUST also set the two opt-in
bounds, which are **off by default**:

- a **request memory ceiling** (`eval_memory_limit`, or `MemoryConstrained` in
  the language), and
- an **elapsed-time timeout** (`Session::set_eval_timeout`), enforced against
  Rust's monotonic `Instant` rather than the civil wall clock.

Without them, a single script can consume unbounded CPU time or memory (for
example, by producing a very large exact integer). With them, such a request is
terminated in-band.

### Isolation

Evaluation is crash-only and per-request isolated: a panic during evaluation is
caught and converted into a structured failure, a session can be reset, and a
poisoned session is reported as requiring reset rather than silently continuing.

## Out of scope

- The default catalog includes a narrow, provider-mediated filesystem surface
  for package loading/lookup and textual persistence. It is capability-gated,
  and `HostConfig::sandboxed()` removes its provider. General filesystem, OS,
  process, and network APIs remain outside the kernel and belong in downstream
  integrations.
- `rum` makes no compatibility claims about any other language or product
  (see [DISCLAIMER.md](./DISCLAIMER.md)).
