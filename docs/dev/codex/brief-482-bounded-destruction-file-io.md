# brief-482 — the destruction declaration on file-I/O providers

Step B of the host-payload destruction contract (plan: brief-480). Step A
(brief-481, `b3de526d`) built the mechanism and proved it on entropy. This step
extends it to the second provider, which has the same shape, and extends the
existing crate-edge certificate rather than building a new one.

**This is an intentional API break**, ruled acceptable by the user on
2026-09-19. It is **not** a release: `v0.2.0` onward is the security-hardening
line and no tag lands mid-line.

## Goal

`FileIo` requires `BoundedDestruction`. An implementor that declares no stance
fails to compile, certified from crates outside this workspace through both
`FileIoProvider::new` and the erased `FileIoProvider::from_arc`. All five
in-tree implementors declare it with a written rationale. File-I/O behavior,
provider identity, threading bounds and the `list_dir` default method are
unchanged.

## Context pointers

- `docs/dev/codex/brief-481-bounded-destruction-entropy.md` and the committed
  step A: the marker at `crates/rum-runtime/src/runtime/host_payload.rs`, the
  runner at `scripts/check-host-payload-destruction.py`, the fixtures at
  `crates/rum-host/tests/fixtures/host_payload_destruction_contract/`, and the
  gate test at `crates/rum-host/tests/host_payload_destruction_contract.rs`.
  **Follow step A's shape; do not redesign it.**
- `crates/rum-runtime/src/runtime/file_io.rs:70` — `trait FileIo: Send + Sync`;
  `:94-113` — the provider, `new`, `from_arc`, `as_file_io`.
- The five implementors, counted with a newline-tolerant scan because the
  narrow `impl FileIo for` spelling undercounts this tree by 38x on the
  suspension traits:
  - `crates/rum-host/src/file_io.rs:31` — `StdFileIo`, the production one;
  - `crates/rum-host/tests/lifecycle_buildout.rs:61` — `InMemoryFileIo`;
  - `crates/rum-stdlib/src/tests/file_io_support.rs:11` — `TestFileIo`;
  - `crates/rum-runtime/src/runtime/tests/mod.rs:52` — `TestFileIo`;
  - `crates/rum-runtime/src/runtime/tests/file_names.rs:138` — `LoopingFileIo`.
  Confirm the set yourself before editing; if it is not five, report that.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/file_io.rs` — the supertrait bound only.
- The five implementor files — the declaration plus its rationale.
- `scripts/check-host-payload-destruction.py`,
  `crates/rum-host/tests/host_payload_destruction_contract.rs`, and new fixture
  files under the existing fixture directory — **extension only**, see below.
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv` — **position
  drift only**, see the inventory rule.
- `docs/guide/embedding.md` — only if a sentence becomes false (`:75` names
  `StdFileIo`); a minimal correction, not a new section.

MUST NOT touch: `crates/rum-runtime/src/runtime/model/session.rs`,
`crates/rum-runtime/src/runtime/suspension/**`, `crates/rum-runtime/src/runtime/random.rs`,
`crates/rum-runtime/src/runtime/host_payload.rs` (the marker is finished),
the ownership scanner's `.rs` code, `SECURITY.md`, `AGENTS.md`, `PROGRESS.md`,
`TODO.md`, any other `docs/**`, `docs/dev/codex/**` except your own report,
`crates/rum-core/src/expr.rs` representation, `.git`, git config, `Cargo.lock`.

This step adds a trait bound and no runtime work. If you find yourself adding a
branch, an allocation or a counter to any evaluation path, STOP and report.

## The mechanism, as ruled

1. `FileIo: BoundedDestruction + Send + Sync`. Preserve `Send + Sync` and the
   `'static` admission in `FileIoProvider::new` exactly; the marker adds no
   threading requirement. Keep `FileIo` usable as a trait object — `dyn FileIo`
   is stored in the provider and returned by `as_file_io`.
2. Do not touch `BoundedDestruction` itself: no methods, no blanket impl, no
   derive, no automatic implementation from anything. It is finished.
3. **Each of the five implementations carries a one-line rationale** in a doc
   comment on the impl, naming what it owns and why that destruction is
   bounded. `StdFileIo` is the one that matters most — it is the production
   implementor. A declaration with no rationale is a rubber stamp and is
   rejected at the gate.

## Extending the certificate, not loosening it

The runner currently hardcodes one case family: the expected diagnostic is
`E0277` whose message names `LegacyEntropy: BoundedDestruction` and whose
primary span sits on `impl EntropySource for LegacyEntropy`. Generalize the
case table so each case carries **its own** expected type name and expected
impl line, and keep the per-case specificity. **Do not weaken the check to
"any `E0277`"** or to a substring of the trait name — a runner that accepts any
missing-bound error would pass for the wrong reason, and this project has been
burned by exactly that shape of instrument.

Add three negative cases and two positive controls, mirroring entropy:

| case | expectation |
|---|---|
| `legacy_file_io_impl` | rejected, `E0277`, missing bound on this consumer's `impl FileIo` |
| `legacy_file_io_new` | same, through `FileIoProvider::new` |
| `legacy_file_io_from_arc` | same, through the erased `FileIoProvider::from_arc` |
| `declared_file_io_new` | compiles |
| `declared_file_io_from_arc` | compiles |

The three negative fixtures must be **unchanged legacy source** — a `FileIo`
implementor written the way one is written today — and must reference
`BoundedDestruction` nowhere. That property is what makes them
baseline-sensitive; a fixture that imports the marker fails on the pre-change
tree only because the name does not exist, which proves nothing.

Keep the runner's existing discipline: consumer crates generated under
repository-local `target/` with their own `[workspace]` and target directory,
`cargo check --offline`, never executing a consumer, and diagnostics read
rather than exit codes trusted. Re-derive the timeout from a recorded run of
the same shape and say which run.

## The inventory rule (read twice)

Editing `file_io.rs` moves line numbers, and the ownership inventory pins exact
positions — `host_payloads.tsv` has a row at `file_io.rs:95`. You MAY update the
`.tsv` files for that drift under one condition the gate checks: **every row's
content is unchanged except its line and column columns.** No row added,
removed, reclassified, or with an altered witness path. If the diff shows
anything else, STOP and report it. A drifting snapshot must never become a way
to accept a new unknown.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

Then the certificate, all ten cases (entropy's five must stay green):

```
CARGO_NET_OFFLINE=true python3 scripts/check-host-payload-destruction.py
```

Paste every case line. The entropy cases regressing is a stop, not a detail.

## Falsifier (MANDATORY)

1. Show the three new legacy fixtures compiling **before** the `FileIo`
   supertrait lands (or, equivalently, with the supertrait temporarily removed
   from the post-change tree). Paste it.
2. With the change in, remove the `FileIo` supertrait, rerun the runner, and
   show the three file-I/O negatives now compiling and the runner **red**.
   Restore and rerun to 10/10 green. Paste both.
3. Removing a fixture instead of the enforcement proves nothing.

**Restore with `git`-free means** — re-edit the line by hand. `git checkout` on
a file restores it from the index, which holds the pre-brief version, and would
silently revert your own work.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave everything in
  the working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No differential testing against other engines.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`, and this
  marker must never become the reason for the first.
- Do not run any workload that reproduces a crash, an overflow or a process
  termination. Nothing here needs one.
- **Cost is not an argument** (user ruling 2026-08-27). No documentation-only
  contract, no stopgap to be fixed later.
- Minimal diff: no drive-by reformatting or renames.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — every command, verbatim, including the workspace test total
   and all ten certificate cases.
3. The falsifier — the three steps above, verbatim.
4. The inventory diff — show that only line/column columns moved, or that the
   files are byte-identical.
5. Stop-loss / open questions.
