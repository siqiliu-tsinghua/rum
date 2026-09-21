# Embedding

`rum-host` is the narrow Rust embedding facade.

Use it instead of depending directly on runtime internals.

The facade owns session construction, request entrypoints, structured outcomes,
input suspension, cancellation, timeout, reset, snapshots, and capability
policy.

The crate-level rustdoc is the full API reference. This chapter is the quick
start.

## Minimal Example

The repository includes a compile-checked example at
`crates/rum-host/examples/basic_embedding.rs`.

```rust
use rum_host::{HostEvalOutcome, HostSession};

fn main() {
    let mut session = HostSession::new();

    match session.eval("rule[x_Integer] := x + 1; rule[41]") {
        HostEvalOutcome::Success(success) => {
            println!("value: {}", success.value.text);
        }
        HostEvalOutcome::InputRequired(suspension) => {
            println!("input required: {}", suspension.request.prompt);
        }
        HostEvalOutcome::Failure(failure) => {
            eprintln!("error: {}", failure.message);
        }
    }
}
```

Run it from the workspace:

```text
$ cargo run -q -p rum-host --example basic_embedding
value: 42
```

`HostSession::new()` installs the core builtins with the default unrestricted
trusted-host configuration.

## Building A Session

Use `HostSession::new()` for the default configuration.

Use `HostSession::builder()` when you need to change configuration:

- tracing
- capability policy
- whether core builtins are installed
- downstream builtin-family installers
- the full `HostConfig`

`HostConfig` carries:

- `resource_limits`
- `capability_policy`
- `current_context`
- `context_path`
- `package_search_paths`
- `file_io`
- `entropy_source`
- `tracing_enabled`
- `asserts_enabled`
- `install_core_builtins`
- `builtin_installers`

The default trusted configuration installs `StdFileIo` and
`StdEntropySource`; sandboxed configuration omits both providers.

The default `HostResourceLimits::native_stack_limit` is a 1 MiB evaluator
budget calibrated for ordinary shallow entry on a caller-declared thread stack
of at least `MIN_SUPPORTED_NATIVE_STACK_BYTES` (2 MiB). The host cannot discover
or verify the thread's actual stack bounds through portable safe Rust. A host
using a smaller stack, or entering evaluation beneath substantial caller
frames, must lower the budget.

When provisioning a larger worker stack, use
`native_stack_limit_for_declared_stack` with the same byte count and install the
returned budget in `HostResourceLimits`. The helper reserves four 128 KiB
unpolled levels plus 512 KiB for non-evaluator frames, and returns `None` for a
declared stack below 2 MiB.

Downstream builtin families install through
`HostSession::builder().with_builtin_installer(...)`. Installers receive the
runtime kernel session, run in builder order after the optional core builtin
install, are retained by cloned `HostConfig` values, and are re-applied by
`reset()`. Capability policy still gates these builtins at dispatch, so a
sandboxed session can run a downstream `Pure` builtin while denying a
downstream `FileRead` or `FileWrite` builtin.

## Profiles

`HostConfig` provides three named presets:

- `sandboxed()` is the untrusted-code profile. It permits only `Pure`
  builtins and installs neither a file I/O provider nor a host entropy
  provider.
- `hermetic()` is the deterministic build-step and compile-time profile. The
  same input produces the same output across sessions and resets; fixed-seed
  deterministic random streams remain available, while wall-clock, host
  entropy, file I/O, process, interactive, and diagnostic access are denied.
  It has the same surface as `sandboxed()` today, but the two named contracts
  may diverge in the future.
- `scripting()` is the trusted local-scripting profile used by command-line
  hosts. It installs the core builtins, standard file I/O and host entropy
  providers, and an unrestricted capability policy, so it carries full
  ambient authority and must never receive untrusted code.

For untrusted code, choose `sandboxed()`; for deterministic build steps,
choose `hermetic()`. For trusted local scripting, choose `scripting()`.

## Adoption Cost

Measured on Intel(R) Core(TM) i9-9980HK CPU @ 2.40GHz, rustc 1.85.0,
release, 2026-07-06.

The normal `rum-host` dependency graph is 257 `cargo tree --no-dedupe` lines:
three direct `rum` crates (`rum-core`, `rum-runtime`, `rum-stdlib`) and 23
unique external crates.

A scratch `println!("hi")` binary was 379 KiB unstripped and 317 KiB stripped.
Adding `HostSession::new()`, `eval("1 + 1")`, and result printing made it
8.2 MiB unstripped and 6.9 MiB stripped, a delta of 7.8 MiB and 6.5 MiB.

In one release process with ten fresh sessions, `HostSession::new()` took
0.38 ms for the first iteration, then 0.09 ms min and 0.09 ms median. The
first `eval("1 + 1")` took 0.13 ms for the first iteration, then 4.4 us min
and 4.5 us median.

Immediately after install, `MemoryInUse[]` reported 46,607 bytes (~45.5 KiB).
This is the cooperative session memory-proxy byte count, not RSS.

A host that links `rum-host` but does not construct a session pays only the
link cost. Refresh these numbers when the workspace changes materially.

## Evaluating

`HostSession::eval(source)` returns `HostEvalOutcome`.

The variants are:

- `Success(HostEvalSuccess)`
- `InputRequired(HostInputSuspension)`
- `Failure(HostFailure)`

Do not treat input suspension as a failure.

It is a resumable state for `Input` and `InputString`.

## Typed Invocation

Use `HostSession::call(head, args)` when host values should enter the language
without constructing source text:

```rust
use rum_host::{HostEvalOutcome, HostSession, HostTyped};

let mut session = HostSession::new();
let args = [HostTyped::Str("a".into()), HostTyped::Str("b".into())];

match session.call("StringJoin", &args) {
    HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "\"ab\""),
    other => panic!("unexpected outcome: {other:?}"),
}
```

The head and every `HostTyped::Symbol` argument use the session's normal symbol
resolution rules. Short names therefore search the active context path, while
fully qualified names select their stated context. Arguments are constructed as
expression data and are never parsed, so quotes, brackets, newlines, and other
source-like text inside `HostTyped::Str` cannot inject evaluation. Once built,
the call uses ordinary evaluation semantics, including argument evaluation and
head attributes.

Typed input accepts exact integers, decimal big integers, decimal
numerator/denominator rationals, booleans, symbols, strings, lists,
associations, and immediate rules. Decimal fields are validated, rational
denominators must be nonzero, and exact rationals are normalized: input `2/4`
returns as `1/2`. `HostTyped::Other` is an output-only lossy view and is always
rejected as input because accepting its text would reintroduce parsing. Nested
typed containers are capped at 64 levels.

## Success Values

`HostEvalSuccess` includes:

- `value`
- `messages`
- `prints`
- `trace`
- `outputs`

`value.text` is the canonically rendered expression with one top-level
`HoldForm` peeled, matching the CLI front end. For example,
`HoldForm[1 + 1]` is rendered as `Plus[1, 1]`. The `value.typed` field keeps the
transport-friendly structural view of the evaluated expression.

`value.typed` is a transport-friendly structured value:

- small integers
- big integers as decimal strings
- rationals as numerator and denominator strings
- booleans, with the system symbols `True` and `False` mapped to `Bool` rather
  than `Symbol`
- other symbols
- strings
- lists up to the typed depth cap
- associations as ordered key/value pairs up to the typed depth cap
- immediate rules with typed left- and right-hand sides up to the typed depth cap
- other expressions as rendered text with a head name

Associations containing any delayed entry and all `RuleDelayed` expressions
remain `Other`: the typed data view never presents an unevaluated right-hand
side as a value. Structured values at the typed depth cap also remain `Other`.

There is no floating-point typed variant because the language boundary is exact
rationals only.

## Messages, Prints, And Outputs

Messages are exposed as `HostMessage` values with:

- `symbol`
- `tag`
- `text`

Print output is exposed as strings.

The kernel never writes to stdout or stderr on behalf of evaluated code.
By default, `Print` output is buffered per request and returned through `prints`
and `outputs`, so the embedder owns its process's stdout completely and can
reserve it for a framed protocol. `HostSession::set_print_sink` switches to
exclusive streaming when a host needs output during evaluation. Buffered print
volume is bounded by the request memory ceiling configured through
`HostResourceLimits::eval_memory_limit`.

Structured outputs are a flattened adapter-friendly stream with kinds:

- result
- print
- message
- trace

Adapters can map these into their own transport without depending on runtime
internal structs.

## Failures

`HostFailure` includes:

- a coarse `HostFailureCode`
- an optional `exit_status`, populated for language-level `Exit[code]` failures
- a user-facing message
- an optional `syntax_rendered`, populated with the full rendered source
  diagnostic exactly for syntax failures
- messages, prints, trace, and structured outputs collected before failure

Failure codes include syntax failures, load failures, evaluation cycles,
recursion limits, match recursion limits, cancellation, timeout, resource
limits, panic, poisoned session, exit, and other.

Recoverable resource-limit outcomes often appear as successful values containing
`TerminatedEvaluation[...]` rather than `HostFailure`.

### Failure Classification

Use `failure.code.requires_reset()` to decide whether a session must be reset
after a failure. It returns true for caught panics and poisoned-session failures;
other failure codes leave the session usable. For `HostFailureCode::Exit`, read
`failure.exit_status` to obtain the requested process status.

For successful evaluations, compare `success.termination` with
`Some(HostTerminationReason::Cancelled)` to detect runtime-synthesized
cancellation. Do not infer cancellation from the rendered value: an ordinary
program result can itself be `TerminatedEvaluation["Cancelled"]`.

## Resumable Input

`Input` and `InputString` can return `HostEvalOutcome::InputRequired`.

The suspension contains:

- `request.kind`
- `request.prompt`
- `request.password`
- outputs collected before the suspension

Resume with `HostSession::resume_with_input(input)`.

For single-source `eval`, the session keeps the continuation. The host should
not replay the original source to answer input.

`pending_input_request()` lets a host inspect whether the session is currently
waiting for input.

While an evaluation is suspended, every new evaluation entrypoint (`eval`,
`call`, `eval_script`, `eval_file`, and `eval_with_timeout`) returns
`HostEvalOutcome::Failure` with `HostFailureCode::Other`. The rejection does
not poison the session, consume queued input, or replace or abandon the pending
continuation. The original single-source continuation remains resumable through
`resume_with_input`. `queue_input_response` remains available for future input,
and reset, snapshot inspection, symbol/editor inspection, and other read-only
operations remain available; queuing a value is not an implicit resume or
supersede operation. The one exception is an accepted cancellation for that
same suspended request: the next resume or guest-code evaluation entrypoint
reports the structured cancellation, abandons the continuation without
converting or delivering host input, and leaves the session reusable.

The host crate has a public contract test for this flow:

```text
$ cargo test -q -p rum-host public_contract_preserves_input_suspension_and_resume_boundaries -- --exact

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 27 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
```

## Streaming prints and batch input

`HostSession::set_print_sink` accepts a `PrintSink`, whose `BoundedDestruction`
declaration covers its owned state. For a closure, use
`ClosurePrintSink::assert_bounded_destruction(closure)` to assert the destruction
bound for everything it captures; the compiler does not verify that assertion.
All three types are available from `rum_host`. Each `Print` line is sent to the
sink as it is produced on the evaluating thread. Routing is exclusive: while the
sink is installed, the `prints` field on both successful and failed outcomes is
empty. `clear_print_sink()` restores per-request buffering. A sink must not
panic, because its panic is caught at the host boundary and poisons the session.
The sink does not survive `reset()`; reinstall it after reset.

For batch input, queue raw host strings in consumption order before evaluating
the script:

```rust
let host_argument = String::from("value supplied by the host");
session.queue_input_response(host_argument);
let outcome = session.eval_script("scriptArgument = InputString[]\nuse[scriptArgument]");
```

This queue-then-eval pattern is the intended alternative to interpolating host
data into a source string and inventing an escaping scheme. Use `InputString`
when the supplied value must remain literal text; `Input` parses its response as
rum source. `reset()` clears any queued responses.

Pre-queue every response a batch script needs. If `eval_script` reaches `Input`
or `InputString` with an empty queue, it returns `InputRequired`. The current
script-shaped request is not resumable through `resume_with_input`: that call
returns `Failure` with `HostFailureCode::Other`, and forms after the suspended
form are not evaluated. New evaluation requests are also rejected while that
script suspension remains installed. Because the script has no resumable host
checkpoint, the ordinary escape routes are `reset()`, `restore_from_bytes()`
(which resets after decoding; see [Snapshot And Restore](#snapshot-and-restore)
for termination failures and partial state), or replacing the whole
`HostSession`. Request-scoped cancellation is a second escape: cancel the
handle accepted for the script, then re-enter through `resume_with_input` or a
guest-code evaluation entrypoint.
That re-entry reports cancellation for the old script without converting its
argument or evaluating new source, structurally abandons the script
continuation, and makes the following request usable. Read-only inspection
remains safe and available. Queuing a response after suspension does not turn
the script into a resumable request. Single-source `eval` retains the resumable
flow described above.

`eval_script` and `eval_file` are the script-shaped entrypoints. Both evaluate
forms in order and report the final form's value while aggregating messages and
buffered prints. `eval_file` reads through the configured file I/O provider; a
sandboxed session has no provider and therefore fails the file request.

## Reset

`HostSession::reset()` rebuilds the session from its original configuration.

It drops user definitions, attributes, contexts, loaded packages, messages, and
gensyms.

It also clears queued input responses and an installed print sink, clears a
poisoned session, and invalidates old cancel handles.

Obtain a new `cancel_handle()` and reinstall any print sink after reset.

## Per-Request Isolation

`reset()` rebuilds the session from its configuration. It drops all user
definitions, attributes, options, messages, non-configured contexts, loaded
packages, gensym-counter advancement, and PRNG advancement. Configured resource
limits, capability policy, contexts, file I/O and entropy providers, and core
and downstream builtin installers remain in effect.

A host serving independent requests should reset between requests. Pass
explicit state as request data rather than relying on implicit session state to
carry over. When explicit carry-over is wanted, use snapshot and restore
instead.

A text snapshot captures user definitions, but not the gensym counter or PRNG
state. Reset-between-requests therefore provides the stronger determinism
guarantee.

The lifecycle is crash-only: a host process may be terminated at any instant.
The kernel keeps no temporary files, as enforced by the `std::fs` hygiene test,
and depends on no destructors for correctness. A killed host leaves no residue,
and a relaunched host starts clean.

## Snapshot And Restore

`snapshot_to_bytes()` serializes user definitions and attributes for defined
symbols to a UTF-8 source image.

`restore_from_bytes(bytes)` decodes the image, resets the session, and evaluates
the source. If evaluation terminates early, restore returns a `HostFailure`
with `HostFailureCode::Other` and a message naming the termination reason.
Restore is not transactional: a failure after reset leaves any snapshot state
installed before the failure in place, so the session may be partially
populated. The prior state is not recovered and partial state is not cleared.
Invalid UTF-8 is rejected before reset, leaving the session unchanged.

This is an in-memory text image, not a binary snapshot.

It does not capture attribute-only symbols, `$` variables, options, messages,
the gensym counter, or unrelated context state.

Use this when the host wants persistence without filesystem capability.

## Sandbox Configuration

Use `HostConfig::sandboxed()` for untrusted source.

It installs a `{Pure}` capability policy.

Use `{Pure, Clock}` when untrusted scripts need time builtins such as
`AbsoluteTime`, `Timing`, or `TimeConstrained`.

Construct the session through `HostSession::builder().with_config(...)` and
pass `HostConfig::sandboxed()`.

That API shape is the same one used by the host tests. The sandboxed policy
blocks every non-`Pure` capability at dispatch, including runtime-constructed
calls, and omits the file and host-entropy providers.

## Cancel And Timeout

Call `cancel_handle()` before starting cancellable work, then move or clone the
handle to the controlling thread. The handle is qualified to that one request:
cancellation is effective immediately, survives the request prologue, and
remains attached through input suspension and resume. Completion is the
boundary; later calls on the same handle cannot affect another request. A
handle held across `reset()` is likewise inert.

If a single-source request is waiting for input, call the handle and then
`resume_with_input`. Cancellation wins before the supplied value is converted
or delivered, the continuation is structurally abandoned, and the returned
success carries `HostTerminationReason::Cancelled`.

Use `eval_with_timeout(source, duration)` for a per-evaluation elapsed-time
deadline backed by Rust's monotonic `Instant`. The original absolute deadline
stays armed across `Input` and `InputString` suspensions and every
`resume_with_input` call. Time spent waiting for the host's input counts toward
the budget; resuming does not pause the clock or grant a fresh duration. A host
that needs more time for input should choose a longer timeout or use `eval`
without a timeout.

The timeout is cleared when the evaluation completes or fails, including after
resume or cancellation, and by `reset()`. An overlapping evaluation rejected
while input is pending leaves the suspended request's deadline unchanged,
including when the rejected call supplies a different timeout.

An unhandled timeout during resumed evaluation has the ordinary timeout shape:
`HostEvalOutcome::Success` with value `TerminatedEvaluation["TimeConstraint"]`,
`termination: Some(HostTerminationReason::TimeConstraint)`, and a
`General::timelim` message. Check the structured termination reason to distinguish
it from a program that returns the same expression as an ordinary value.

Both are cooperative checks. The session remains usable after normal
recoverable cancellation and timeout outcomes.

## Threading Contract

`HostSession` is `Send`.

Move one session to one worker thread.

It is intentionally not a shared mutable object. Do not share one session
concurrently across threads.

Use one session per worker when you need parallel evaluation.

If a worker is created with `std::thread::Builder::stack_size`, derive its
`native_stack_limit` by passing that exact declared size to
`native_stack_limit_for_declared_stack`. Raising the evaluator budget without
provisioning the matching worker stack breaks the recoverable-failure contract;
lower the budget when caller frames consume a material part of the stack before
evaluation begins.

`HostCancelHandle` is cloneable and can cross threads.

## Editor Helpers

The facade also exposes parser, completion, inspection, and completeness
requests:

- `parse`
- `lower_dump`
- `inspect`
- `inspect_symbol`
- `message_catalog`
- `complete`
- `is_complete`

Those helpers are for adapters and editors.

`lower_dump(source)` is a diagnostic-grade developer-tooling surface. It
lowers against the session's current context state and returns a canonical core
expression rendering, but that rendered format is not a stability promise.

`message_catalog(query)` returns the global catalog for `None`, or narrows by a
symbol or `symbol::tag` selector. An invalid selector returns `None`. Symbol
inspection also retains its per-symbol message view in
`HostInspectResponse.message_catalog`.

They do not expose matcher IR, planner internals, or mutable evaluator state.
