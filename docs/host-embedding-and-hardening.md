# Host Embedding And Hardening

Status: refreshed 2026-08-23 against the post-`v0.1.0` codebase.

This document is a compact map of the current embedding and hardening
boundaries. The normative API is the `rum-host` crate rustdoc; the tutorial and
examples live in [the embedding guide](./guide/embedding.md), and the threat
model lives in [SECURITY.md](../SECURITY.md).

## Embedding Boundary

Rust applications should embed `rum` through `rum-host`, not by depending on
evaluator internals. `HostSession` owns:

- source, script, file, and typed-call evaluation;
- structured values, messages, prints, traces, termination reasons, and
  failures;
- input suspension/resume, cancellation, elapsed-time deadlines, reset, and
  snapshot/restore;
- parser, lowering, completion, inspection, and completeness requests used by
  adapters;
- downstream builtin-family installation through stored `BuiltinInstaller`
  callbacks.

`rum-cli`, `rum-repl`, and `rum-jupyter` all consume this same facade. Protocol,
terminal, and transport types stay outside `rum-runtime`.

## Configuration Profiles

`HostConfig` exposes three named profiles:

- `scripting()` (also `default()`) is the trusted local profile. It has an
  unrestricted capability policy plus standard file-I/O and host-entropy
  providers. Never use it for untrusted source.
- `sandboxed()` permits only `Capability::Pure` and removes the file-I/O and
  entropy providers. Runtime-constructed calls are checked at dispatch, so
  constructing a denied head through `Symbol` or `ToExpression` does not bypass
  the policy.
- `hermetic()` is the deterministic build-step profile. It currently has the
  same implementation as `sandboxed()` but is a separate named contract.

Fine-grained allowlists use `CapabilityPolicy`. The current capability classes
are `Pure`, `FileRead`, `FileWrite`, `Clock`, `Random`, `Diagnostic`, `Process`,
and `Interactive`; `Process` is reserved for downstream integrations. Role,
identity, and per-plugin policy selection belong to the embedding application.

The in-tree filesystem surface is provider-mediated. Current file operations
are package loading and lookup (`Get`, `Needs`, `FindFile`, `FileNames`) plus
textual persistence (`Save`, `DumpSave`). There is no general network or process
API in the kernel.

## Resource And Failure Boundaries

`HostResourceLimits` configures evaluator recursion, native-stack use,
`ReplaceRepeated` iterations, fixed-point passes, generated elements, exact
integer work, iterative builtin work, and an optional per-request modeled-memory
ceiling (`eval_memory_limit`). The memory ceiling is a deterministic structural
proxy, not process RSS; a host that requires a hard process-wide RSS limit must
still enforce that outside the kernel.

The default native-stack evaluator budget is 1 MiB, derived from
`MIN_SUPPORTED_NATIVE_STACK_BYTES` (2 MiB) minus 512 KiB of four-poll headroom
and 512 KiB of non-evaluator reserve. It assumes ordinary shallow evaluator
entry on a caller-declared thread stack of at least 2 MiB. Safe Rust provides no
portable stack-bound discovery, so the kernel cannot verify this precondition.
Embedders provisioning a larger worker stack should pass the same byte count to
`native_stack_limit_for_declared_stack` and install the returned budget; hosts
on smaller stacks or beneath substantial caller frames must lower the budget.

`HostSession::eval_with_timeout` installs a per-request elapsed-time deadline.
The evaluator checks cancellation and deadlines cooperatively at amortized poll
points using Rust's monotonic `Instant`. A `HostCancelHandle` is cloneable and
may be triggered from another thread. Normal cancellation, timeout, and
resource-limit outcomes leave the session usable.

Every public guest-code entrypoint catches Rust panics at the host boundary. A
caught panic becomes `HostFailureCode::Panic` and poisons the session; later
requests return `Poisoned` until `reset()`. Use
`HostFailureCode::requires_reset()` instead of matching error text. This boundary
does not catch process-aborting failures such as a native stack overflow.
Meeting the declared-stack precondition is what lets the byte guard return an
in-band stack-limit outcome before that boundary is reached.

## Lifecycle And Isolation

`HostSession::reset()` rebuilds the runtime from the stored `HostConfig`. It:

- clears user definitions, attributes, options, messages, loaded-package state,
  queued input, gensym and PRNG advancement, and an installed print sink;
- clears poison state;
- re-installs the selected core and downstream builtin families;
- preserves configured limits, contexts, providers, and capability policy;
- invalidates cancellation handles obtained before the reset.

`snapshot_to_bytes()` emits a UTF-8 rum source image of user definitions and
the attributes attached to defined symbols. `restore_from_bytes()` resets the
target session and evaluates that image. The format is textual, not binary, and
does not capture the PRNG position, gensym counter, messages, options,
attribute-only symbols, or arbitrary session internals. Sealed packages are
restored by re-registration and leaked sealed references are re-linked or
rejected rather than serialized as forgeable names.

`HostSession` is `Send` and intentionally not a concurrently shared mutable
object. Move one session to one worker thread and use one session per worker for
parallel evaluation. `HostCancelHandle` is the cross-thread control object.

## Structured I/O

`HostEvalOutcome` distinguishes `Success`, `InputRequired`, and `Failure`.
Successful values carry canonical text plus `HostTyped` data for exact
integers/rationals, booleans, symbols, strings, lists, ordered associations, and
immediate rules. Typed trees are depth-capped; delayed rules and unsupported
shapes remain `Other`. `HostSession::call` constructs typed arguments directly
and does not parse their text.

The kernel does not write guest output to process stdout/stderr. Prints are
buffered in structured outcomes unless the host installs an exclusive print
sink. `Input` and `InputString` suspend a single-source evaluation and resume
through `resume_with_input`. Script/file evaluation supports pre-queued input;
if a multi-form script suspends with an empty queue, that script-shaped request
is not currently resumable.

## Deliberate Limits And Deferred Work

- Source-created trees are protected by parser, evaluator, matcher, and native-
  stack limits. A host can still construct a pathologically deep `Expr` directly
  through the Rust API and overflow the stack during recursive drop. Three safe
  iterative-drop designs regressed the normal hot path by 20-50% and were
  rejected; hosts must depth-limit directly constructed trees.
- The modeled-memory ceiling is not an allocator or RSS meter. Treat a process
  boundary, OS quota, or external watchdog as the hard limit when hostile work
  must not threaten other tenants.
- Binary snapshots remain deferred until session-image size justifies a new
  compatibility format.
- Broader filesystem, process, compression, and language-binding integrations
  belong in downstream crates built through `rum-host`; they do not expand the
  kernel's ambient-authority surface.
