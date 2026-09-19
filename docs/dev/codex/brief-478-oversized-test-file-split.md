# brief-478 — split the four test files over the architecture-debt line

## Goal

Four test files are over the 35 000-byte architecture-debt threshold in
`docs/code-organization-guidelines.md`. Split each by **responsibility** so that
every resulting file is under the line, **without changing a single test**.

| file | bytes | `#[test]` count |
|---|---:|---:|
| `crates/rum-runtime/src/runtime/tests/context_and_loader.rs` | 38 873 | 36 |
| `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` | 38 327 | 26 |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | 37 971 | 13 |
| `crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs` | 36 121 | 37 |

**Excluded deliberately**:
`crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
at 163 597 bytes. It is a **generated** flat const array of type names
(`DERIVED_FRAME_ROOT_TRACE_TYPES`) with no responsibility boundary to cut on;
splitting it would be an arbitrary cut, which the guidelines explicitly
discourage. Leave it alone.

## This is a move, and the accept-vs-stop criterion is that word

**No test body may change. No test may be added, removed, renamed, merged or
split.** Not to fix a lint, not to reduce duplication, not to "improve" an
assertion. If a test does not compile after a move because a helper is now out
of scope, move or re-export the helper — do not edit the test.

Any of the following is a **stop**, to be reported rather than shipped:

- a `#[test]` function body differs by even one token from its original;
- the set of test names changes in any way;
- a production file appears in `git diff --stat`;
- a helper's behaviour changes rather than its location.

**The mechanical proof I will check**: the sorted list of `#[test]` function
names across the workspace is identical before and after, and the workspace test
count stays **2 417**.

## How to split

Follow `docs/code-organization-guidelines.md`: split by responsibility, not by
size. Prefer a real boundary the file already has — a `mod` block, a contiguous
run of tests about one subject, a distinct fixture family — over cutting at a
byte offset. State the boundary you chose for each file in one line.

Where a file's tests share helpers, put the helpers in the parent module or a
sibling `…/common.rs` that the new modules import, rather than duplicating them.
**Duplicated helpers are a stop**: they are how two copies of a fixture drift
apart later.

Aim comfortably under the line rather than just under it — the last split in
this line came in at 35 131 bytes, 131 over, and needed a second pass.

## Scope

- Files you MAY modify: the four files above; new sibling modules under
  `crates/rum-runtime/src/runtime/tests/` and
  `crates/rum-runtime/src/runtime/evaluation_cache/`; the module roots that
  declare them (`tests/mod.rs`, `evaluation_cache/mod.rs`); and a
  `…/common.rs`-style helper module if one is needed.
- Files you MUST NOT touch: **all production source**, `derived_inventory.rs`,
  every other test file, `metadata.rs`, `Expr`, `SECURITY.md`, `TODO.md`,
  `PROGRESS.md`, other `docs/**`, `Cargo.toml`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **`rum-runtime` tests are in-crate `mod` files**, not a `tests/` integration
   directory; visibility is `pub(super)`/`pub(in crate::runtime)` and a moved
   test can lose access to a private item. Fix by adjusting the module wiring,
   never by changing what the test asserts.
2. `frame_root_trace.rs` owns an invariant test with an allowlist and an exact
   count assertion against the generated inventory. Moving it must not change
   which file the inventory is read from or what the count is compared against.
3. `evaluation_cache/tests.rs` contains assertions about cache residency and
   pointer identity that were deliberately rebuilt on lane-disjoint targets
   (briefs 387-394). Move them intact; do not "simplify" a fixture that looks
   redundant — its redundancy is the repair.
4. Wall-clock timing is not evidence on this host, and nothing here needs it.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`; if so, say
plainly that the nine native tests did not run in your gate.

Then paste:

- `wc -c` for every file you created or modified, showing all under 35 000;
- the workspace test count, showing **2 417**;
- `git diff --stat`, showing **no production file**;
- for each of the four originals, its destination files and the `#[test]` count
  in each, summing to 36, 26, 13 and 37 respectively.

**No performance gate**: no production code changes. Say that explicitly rather
than running the panel.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. **No new `unsafe`.** No network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Stop-loss.** Stop and report if: a test cannot be moved without editing it;
  the test count or name set changes; a responsibility boundary is not visible
  and the only available cut would be arbitrary; or the same step fails twice
  for the same reason.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. Per original file: the responsibility boundary you cut on, the destination
   files, and the `#[test]` count in each.
2. Any helper you moved or re-exported, and where it now lives.
3. Verification: every command with verbatim output, the ZeroMQ statement, the
   `wc -c` table, the test count, and `git diff --stat`.
4. Confirmation that no test body changed, and how you established it.
5. Stop-loss and open questions.
