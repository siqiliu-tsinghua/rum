# brief-495 — the ownership scanner cannot see a recursive AST it owns

A gap in the scanner's own reach, found while deciding whether to build the
`UnknownConstruct` second axis. It is **not** that axis, and this brief does
not build it.

## The finding

`ownership_graph/source.rs:18` scans three crates:

```rust
pub(super) const ROOTS: &[&str] = &["rum-core", "rum-runtime", "rum-stdlib"];
```

`rum-syntax` is not among them, and the scanned crates **own its types in four
places** — `SessionParseResponse.parsed`, `SessionRequestError::Syntax.0`
(`runtime/model/requests.rs`), `TopLevelEvalInput::Parsed.0`
(`runtime/session/session_eval.rs`), `EvalError::Syntax.0`
(`runtime/session/session_state.rs`) — where they appear only as
`foreign representation rum_syntax::…` rows in `unknown_storage.tsv`.

**Those types are recursive.** `SyntaxExprKind` (`rum-syntax/src/ast.rs:150`)
has `List(Vec<SyntaxExpr>)`, `Association(Vec<SyntaxExpr>)`, and `Box<SyntaxExpr>`
in many variants. So the ownership graph — built precisely to find recursive
owning edges — is blind to one that the runtime holds.

The director checked whether anything else is missing: of the 70
storage/representation unknowns, the originating crates are **29 `std`, 6
`num_bigint`, 6 `rum_syntax`**, and the rest are flat leaves (hashers,
`Instant`, `PathBuf`, `BigInt`, `NonZeroU64`, `Duration`, `AtomicU64`).
`num_bigint` is a third-party dependency and `BigInt` is a flat leaf. **So
`rum-syntax` is the only in-workspace crate the root set misses in this
direction.** Confirm that independently; if the root set is wrong in some other
way, that is the more valuable finding.

## Goal

The scanner sees `rum-syntax`. Whatever that surfaces is classified honestly
using the machinery this line already built — the exact inventories, the
accumulator coverage gate, and the host-payload disposition ledger
(brief-494) — and nothing is silently absorbed.

## What to do

1. **Settle the root set from first principles** before changing it. Which
   workspace crates hold types that the scanned crates own by value? Answer
   with evidence, not with the director's list. Say explicitly whether
   `rum-host`, `rum-jupyter`, `rum-repl`, `rum-cli` and `rum-terminal` belong —
   and if they do not, say why the criterion excludes them, so the next person
   inherits a rule rather than a list.
2. **Add what belongs**, and report the churn plainly: how many rows each
   inventory partition gains, and how many new self-reaching types appear.
   Churn is expected here; it is not a reason to narrow the change.
3. **Classify `SyntaxExpr` and every other newly self-reaching type.** For each,
   say whether its destruction is bounded, and by what. Two facts to weigh
   rather than assume:
   - source-created trees are bounded by the parser's nesting limit
     (`rum-syntax/src/error.rs:25`, `parse nesting exceeded limit {max}`) —
     find the actual limit and say what it is;
   - `rum-syntax`'s AST types are `pub`, so a host can construct a tree of any
     depth through the public API and hand it to the runtime.

   That is the same shape as `Expr`'s recorded debt — source-guarded, host-
   reachable — so if the honest answer is a debt entry, say so. **Do not
   propose bounding it by adding a `Drop`**: `Expr`'s three iterative-drop
   designs were rejected on a measured 20-50% hot-path regression, and this
   brief is not the place to reopen that.
4. **Use the existing machinery rather than inventing more.** If a new row is a
   host payload, it belongs in brief-494's ledger with a disposition; if it is
   an accumulator chain, the coverage gate already applies and will say so; if
   it is a new class, stop and report before creating anything.

## What this brief is not

- It is **not** the `UnknownConstruct` second axis. Do not classify macros,
  closure captures or derives, and do not propose a scheme for them.
- It does **not** add `BoundedDestruction` to anything.
- It does **not** change production behavior. If you believe a production
  change is needed, STOP and report.

## The falsifier

The scanner's new reach must be demonstrated, not assumed: show that with
`rum-syntax` scanned, a **deliberately added** recursive type in that crate is
discovered (added, seen, removed), and that before the root change the same
type is invisible. That is the whole claim — that the graph can now see into
this crate.

Restore by hand from a scratchpad copy; `git checkout` restores from the index
and deletes uncommitted work.

## Scope

Files you MAY modify: `crates/rum-stdlib/src/tests/ownership_graph/` (the root
set, and any check that must learn about the new crate) and its `inventory/`
files, including brief-494's ledger if a row's disposition genuinely belongs
there.

MUST NOT touch: any production source, `rum-syntax` included — its recursion is
the subject, not the target; `accumulator_chain.rs`, `host_payload.rs`,
`crates/rum-derive/**`, the suspension traits; `SECURITY.md`, `AGENTS.md`,
`PROGRESS.md`, `TODO.md`, other `docs/**`, `docs/dev/codex/**` except your own
report; `.git`, git config, `Cargo.lock`.

## The inventory rule — disclosure, not permission

You MAY regenerate any `.tsv` as discovery requires — this change is *expected*
to move many rows. Report the churn per partition as counts, and then list
individually only: every new **self-reaching** type, every new
**`UnknownHostPayload`** row, and every row that was **removed**. Those three
classes are the ones a reader must see one by one; bulk macro, capture and
derive rows may be reported as counts. Never hand-edit a row to make a check
pass — regenerate, then explain.

## Verification (MANDATORY)

```
python3 scripts/run-bounded.py --timeout 180  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

No A/B: test-only. Confirm that from your own diff. If your sandbox cannot bind
loopback, say so and name the skip you used.

## Hard rules

- Do NOT `git commit`, branch, push, touch git config, or run `git stash` /
  `git checkout` on a modified file.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **No crash, overflow, past-ceiling, or fatal-process probe.** In particular
  do not build a deep AST to see what happens — that bracket is the director's.
- **No new `unsafe`.**
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Cost is not an argument** (user ruling 2026-08-27), and neither is churn: a
  scanner that cannot see a crate the kernel owns is a gap whether or not
  closing it is noisy.
- **A silenced unknown is worse than a loud one.** If something cannot be
  classified, leave it unknown and say so; do not invent a disposition to make
  a check green.
- Minimal diff: no drive-by reformatting or renames.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. The root-set criterion you derived, the crates it includes and excludes, and
   why — with evidence.
2. The churn, per inventory partition, as counts.
3. Every new self-reaching type, new `UnknownHostPayload` row, and removed row,
   individually.
4. `SyntaxExpr`'s classification and that of anything else newly recursive,
   with the parser's actual nesting limit and the host-construction path.
5. The falsifier: the added type seen, and invisible before the root change.
6. `cargo fmt --check` and the workspace total.
7. Stop-loss / open questions.
