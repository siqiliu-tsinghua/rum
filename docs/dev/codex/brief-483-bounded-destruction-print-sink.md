# brief-483 — the destruction declaration on the print sink

Step C of the host-payload destruction contract (plan: brief-480). Steps A
(brief-481, `b3de526d`) and B (brief-482, `ce084977`) applied the marker to the
two providers, whose payloads are trait objects. **This one is different in
kind**: the stored payload is a bare `Box<dyn FnMut(&str) + Send>`, and a trait
cannot be implemented on the host's closure. So the interface itself changes.

**This is an intentional API break**, ruled acceptable by the user on
2026-09-19. It is **not** a release: `v0.2.0` onward is the security-hardening
line and no tag lands mid-line.

## Goal

`Session` stores a declared sink rather than an anonymous closure. A named
`PrintSink` trait requires `BoundedDestruction`; a closure reaches the setter
only through an adapter whose constructor names the assertion it is making. An
undeclared sink — a raw closure, a raw `Box<dyn FnMut>`, or a named sink type
without the marker — **fails to compile**, certified from crates outside this
workspace. Print routing behavior is unchanged in every respect the guide
documents.

## Context pointers

- The committed mechanism from steps A and B: the marker at
  `crates/rum-runtime/src/runtime/host_payload.rs`, the runner at
  `scripts/check-host-payload-destruction.py` (ten cases, a `Case` table with a
  per-case expected type, trait and impl line), fixtures at
  `crates/rum-host/tests/fixtures/host_payload_destruction_contract/`, the gate
  test at `crates/rum-host/tests/host_payload_destruction_contract.rs`.
  **Follow that shape; extend it, do not redesign it.**
- `crates/rum-runtime/src/runtime/model/session.rs:95` — the stored field;
  `:131-142` — `set_print_sink` / `clear_print_sink` and their rustdoc.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:302-307` —
  `push_print`, the **only** invocation site: `sink(&text)` when a sink is
  installed, `self.prints.push(text)` when it is not.
- `crates/rum-host/src/session.rs:245-263` — the host setter, currently
  `impl FnMut(&str) + Send + 'static`, with the contract rustdoc.
- `crates/rum-host/src/lib.rs:51-74` — the re-export surface, including the
  existing `pub use rum_runtime::…` precedent.
- `docs/guide/embedding.md` (the print-sink section) — the documented contract.
- The twelve installation sites: `crates/rum-stdlib/src/tests/lazy.rs:109`,
  `.../tests/incidental_print_sink.rs:17`,
  `.../tests/valueq_effect_suppression.rs:123`, `crates/rum-cli/src/main.rs:332`,
  `crates/rum-host/tests/lifecycle_contract.rs:176`,
  `crates/rum-host/tests/suspension_abandonment_contract.rs:22`,
  `crates/rum-host/src/tests/session_lifecycle.rs:48`,
  `crates/rum-host/src/tests/editor_surface.rs:47`,
  `crates/rum-host/src/tests/eval_contract.rs:222`,
  `crates/rum-host/tests/stress.rs:61`,
  `crates/rum-runtime/src/runtime/tests/session_requests/api.rs:37` and `:821`.
  Four `clear_print_sink` calls also exist and should need no change. Confirm
  the set yourself; if it is not twelve, report that.

## Scope

Files you MAY modify:

- **new** `crates/rum-runtime/src/runtime/print_sink.rs` — the trait and adapter.
- `crates/rum-runtime/src/runtime/mod.rs`, `.../kernel_api.rs` — wiring and
  re-export.
- `crates/rum-runtime/src/runtime/model/session.rs` — the field type and the
  two setters.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs` — the invocation
  in `push_print`, and nothing else in that file.
- `crates/rum-host/src/session.rs` and `crates/rum-host/src/lib.rs` — the host
  setter and the re-exports.
- The twelve installation sites listed above.
- `scripts/check-host-payload-destruction.py`,
  `crates/rum-host/tests/host_payload_destruction_contract.rs`, and new fixture
  files in the existing fixture directory — extension only.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv` — under the
  inventory rule below, which is **different this time**.
- `docs/guide/embedding.md` — the print-sink contract paragraph, updated to the
  new API. Minimal; do not restructure the chapter.

MUST NOT touch: `crates/rum-runtime/src/runtime/host_payload.rs` (the marker is
finished), `.../random.rs`, `.../file_io.rs`, `.../suspension/**`, the ownership
scanner's `.rs` code, `SECURITY.md`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, any
other `docs/**`, `docs/dev/codex/**` except your own report,
`crates/rum-core/src/expr.rs` representation, `.git`, git config, `Cargo.lock`.

## The mechanism, as ruled

1. **`PrintSink: BoundedDestruction + Send`**, one method taking `&mut self`
   and the line as `&str`. It must be usable as `dyn PrintSink`, since that is
   what the session stores. Preserve `Send`; the sink runs on the evaluating
   thread and the session must stay `Send` exactly as it is today.
2. **`ClosurePrintSink<F>`** — a public generic adapter with a **private**
   closure field, a single constructor named `assert_bounded_destruction`, and
   a `PrintSink` implementation. **No** `From<F>`, no `Default`, no public
   field, and above all **no blanket `impl PrintSink for F where F: FnMut`** —
   a blanket impl would let every existing caller keep making no declaration,
   which is the entire thing this step removes. The adapter's own
   `BoundedDestruction` declaration covers the captured state, and its rustdoc
   must say that the assertion is about what the closure captures.
3. **The kernel setter takes `Box<dyn PrintSink>`; the host setter takes
   `impl PrintSink + 'static`** and boxes it, preserving today's division of
   responsibility.
4. **`rum-host` must re-export `PrintSink`, `ClosurePrintSink` and
   `BoundedDestruction`** (`lib.rs`, alongside the existing
   `pub use rum_runtime::…` lines). This is load-bearing, not tidiness:
   `rum-cli` depends on `rum-host` only — it has **no** `rum-runtime`
   dependency, by the 2026-07-16 dogfood ruling — so `main.rs:332` cannot name
   a `rum_runtime` path. If a host-side call site cannot be migrated through
   `rum-host`'s surface alone, that is a defect in the re-export, not a reason
   to add a dependency.
5. **The documented contract does not change**: streaming is exclusive (prints
   are not also buffered while a sink is installed), the sink runs on the
   evaluating thread, a panicking sink poisons the session, and a sink does not
   survive `reset`. Update the wording in `docs/guide/embedding.md` to the new
   type, not the behavior.

## No microbench A/B — and you must verify the premise, not assume it

`push_print` is the only invocation site; the no-sink branch
(`self.prints.push(text)`) is untouched; and a boxed `FnMut` call and a boxed
trait-method call are the same indirect call. Nothing on an evaluation path
runs differently when no sink is installed, so a panel A/B would measure noise,
and performing the ceremony anyway is worse than saying why it was skipped.

**Confirm all three of those before relying on this.** If the invocation site
is not unique, or your diff changes anything reached when no sink is installed,
STOP and report — do not proceed and do not silently run an A/B instead.

## The inventory rule — different from steps A and B

This step **legitimately changes one row's content**. The pinned
`host_payloads.tsv` row for `Session.print_sink` names the payload
`dyn FnMut (& str) + Send`; after this change the payload is `dyn PrintSink`.

- **Exactly one row may change content**: the `Session.print_sink` storage row,
  and only in its payload type. Its ownership path
  (`Option[Own]/Box[Own]`) must still be `Option[Own]/Box[Own]`.
- Every other row is **position drift only** — line and column columns, nothing
  else.
- New rows may appear only if the adapter introduces a genuinely new opaque
  storage edge. If one appears, **STOP and report it with its witness** rather
  than pinning it; whether the adapter's `F` is an opaque payload of its own is
  a director decision, not a snapshot update.
- Anything else — a removed row, a changed ownership path, a reclassification —
  is a stop.

## Extending the certificate, not loosening it

The runner's `Case` table currently models one failure shape: `E0277` whose
message is `` the trait bound `T: BoundedDestruction` is not satisfied `` with
a note `` required by a bound in `Trait` ``. The sink cases need **two** shapes,
so generalize the case model to carry an expected diagnostic per case — its
error code and its exact message — and keep every check exact. **Do not
introduce an "any error is fine" path**, and do not relax the existing ten
cases: a runner that accepts an unrelated compile failure passes for the wrong
reason, and this project has been burned by exactly that instrument shape.

Six new cases, sixteen total:

| case | expectation |
|---|---|
| `legacy_raw_closure_sink` | rejected: a bare closure passed to the host setter does not implement `PrintSink` |
| `legacy_boxed_fnmut_sink` | rejected: `Box<dyn FnMut(&str) + Send>` passed to the kernel setter no longer type-checks |
| `named_sink_without_marker` | rejected: `E0277`, missing `BoundedDestruction` on a named `PrintSink` implementor |
| `adapter_private_field_bypass` | rejected: `ClosurePrintSink` cannot be built by struct literal from outside |
| `declared_named_sink` | compiles |
| `declared_closure_adapter` | compiles, through `assert_bounded_destruction` |

The four negative fixtures must be **unchanged legacy source** where such
source exists — `legacy_raw_closure_sink` and `legacy_boxed_fnmut_sink` are
written exactly the way today's call sites are written, and must reference
`PrintSink`, `ClosurePrintSink` and `BoundedDestruction` nowhere. That is what
makes them baseline-sensitive. `named_sink_without_marker` and
`adapter_private_field_bypass` name new API and therefore **cannot** have a
pre-change baseline; say so explicitly in the report rather than implying they
do, and certify them by their post-change perturbation instead.

Keep the runner's existing discipline: consumers generated under
repository-local `target/` with their own `[workspace]` and target directory,
`cargo check --offline`, never executing a consumer, diagnostics read rather
than exit codes trusted. Re-derive the gate timeout from a recorded run of the
same shape and name the run.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

Then all sixteen certificate cases:

```
CARGO_NET_OFFLINE=true python3 scripts/check-host-payload-destruction.py
```

Paste every case line. Any of the existing ten regressing is a stop.

## Falsifier (MANDATORY)

1. Show the two legacy sink fixtures compiling **before** the interface change
   (or with the change temporarily reverted). Paste it.
2. With the change in, perturb the enforcement — add a blanket
   `impl<F: FnMut(&str) + Send> PrintSink for F` — and show the raw-closure
   negative now compiling and the runner **red**. Remove it and rerun to 16/16.
   This is the perturbation that matters: the blanket impl is exactly the
   shortcut that would silently restore the old permissiveness.
3. Separately, remove `BoundedDestruction` from `PrintSink`'s supertraits and
   show `named_sink_without_marker` going green-compiling and the runner red.
   Restore and rerun.
4. Removing a fixture instead of the enforcement proves nothing.

**Restore by hand, never with `git checkout`** — it restores from the index,
which holds the pre-brief version, and would silently revert your own work.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave everything in
  the working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **No default-path cost.** The no-sink branch of `push_print` must stay what
  it is.
- Do not run any workload that reproduces a crash, an overflow or a process
  termination.
- **Cost is not an argument** (user ruling 2026-08-27). Twelve call sites
  describe the work; they do not argue for a blanket impl or a compatibility
  shim. No stopgap to be fixed later.
- Minimal diff: no drive-by reformatting or renames.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — every command, verbatim, with the workspace total and all
   sixteen certificate cases.
3. The falsifier — all three steps, verbatim, plus which new cases have no
   pre-change baseline and why.
4. The inventory diff — the one content change, and everything else position
   only.
5. Confirmation of the A/B premise: the unique invocation site, the untouched
   no-sink branch.
6. Stop-loss / open questions.
