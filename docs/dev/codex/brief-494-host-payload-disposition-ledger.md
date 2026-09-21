# brief-494 — the disposition ledger for host payloads

Step E1 of the host-payload destruction contract, and the last step of this
line. brief-479 discovered the payload rows; briefs 481-492 closed what could
be closed; brief-493 settled what cannot be. This makes the result **recorded
and mechanically checked** instead of living in commit messages.

It is **test-only**. No production behavior changes.

**Step D is cancelled**, by brief-493's recommendation and the director's
ruling: `BoundedDestruction` is **not** added to `SuspensionFrame` or
`PureFrameFinalizer`, because the marker promises a bound on all reachable
destruction and those implementors own `Expr`, whose destruction is recorded
unbounded standing debt. Do not add it. Do not reopen that.

## What the ledger must record

One entry per discovered `UnknownHostPayload` row. Derive the set from
`graph::discover(source::production())`, not from the pinned `.tsv` — the
director's count is **50**: six owned storage edges and forty-four public
by-value inputs. Report any difference before building anything.

The dispositions, and nothing else:

| disposition | applies to | what it asserts |
|---|---|---|
| `Declared` | `FileIoProvider.inner`, `EntropyProvider.inner`, `Session.print_sink` | the payload's trait requires `BoundedDestruction`, so an implementor cannot decline to take a stance. Truth remains the implementor's. |
| `Unbounded` | `SuspendedEvaluation.frames`, `FrameAction::Evaluate.next_frame`, `FrameAction::Finalize.0` | destruction reaches `Expr`/`SharedExpr`/`SharedArgs`, whose debt is recorded in `TODO.md` §Standing engineering debt. **Plus**: external implementations of the two public suspension traits are **unresolved**, and the entry must say so rather than implying the edge is closed. |
| `PublicInput` | the forty-four | the payload is taken by value and not retained as an owning edge. |

Each entry carries its witness, and each `Unbounded` entry carries its
`owed_by` citation and an explicit `external_implementation_coverage:
unresolved`.

## The three checks, and the third is the point

1. **Completeness** — every discovered row has an entry; a new row without one
   is red, naming the row.
2. **No staleness** — every entry matches a discovered row; an entry whose row
   is gone is red.
3. **Narrow attribution** — an `Unbounded` entry's claim must be *verified*,
   not merely written: the check must confirm that the row's ownership actually
   reaches one of the three expression types. **A row that does not reach them
   cannot claim their debt.**

Check 3 is what keeps this from becoming an excuse table. brief-476 says it
directly: unknown ownership must not be disguised as recorded debt, and
brief-417 already cost this project five declarations that moved into an
unenforced contract and were caught the day it gained a falsifier.

Where a check cannot be made mechanical, say so plainly and say what it would
take — do not write prose into the ledger and call it checked.

## Falsifiers (MANDATORY — all three)

1. **A new payload row with no entry** → check 1 red, naming it. Remove it.
2. **An entry whose row no longer exists** → check 2 red. Restore it.
3. **The one that matters**: add a **new recursive field unrelated to `Expr`**
   to one of the suspension types — a self-referential chain of some other
   payload — and give its row an `Unbounded` entry attributing it to the
   `Expr` debt. **Check 3 must reject it**, because that ownership does not
   reach `Expr`. Show it red, then remove both the field and the entry and show
   green.

Paste each red and each restored green verbatim. Restore by hand: `git
checkout` restores from the index and deletes uncommitted work — the director
has made that mistake twice on this line, so use a scratchpad copy.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/tests/ownership_graph/` — a new module for the ledger
  check and its `mod.rs` wiring.
- A new pinned ledger file under
  `crates/rum-stdlib/src/tests/ownership_graph/inventory/`.
- The existing inventory `.tsv` files, only if discovery requires it, under the
  disclosure rule below.

MUST NOT touch: any production source — this step changes no behavior; if you
believe a production change is required, **STOP and report**;
`accumulator_chain.rs`, `host_payload.rs`, `crates/rum-derive/**`, the
suspension traits, `rum-core`; `SECURITY.md`, `AGENTS.md`, `PROGRESS.md`,
`TODO.md`, other `docs/**`, `docs/dev/codex/**` except your own report; `.git`,
git config, `Cargo.lock`.

## The inventory rule — disclosure, not permission

You MAY regenerate any `.tsv` as discovery requires. List **every** content
change (position drift excepted) with its witness, classified: (a) expected
from this brief's own additions; (b) anything else — and **(b) is a stop**, as
is any new `UnknownHostPayload`, any removed row, or any reclassification. This
step should move **no** inventory row at all; if one moves, that is worth
stopping for.

## Verification (MANDATORY)

```
python3 scripts/run-bounded.py --timeout 180  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace --offline --locked
```

No A/B: this brief adds test-only code and changes no production path. Confirm
that from your own diff rather than asserting it. If your sandbox cannot bind
loopback, say so and name the skip you used.

## Hard rules

- Do NOT `git commit`, branch, push, touch git config, or run `git stash` /
  `git checkout` on a modified file.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network (`--offline` everywhere).
- **Do not add `BoundedDestruction` to anything.** That decision is made.
- **No crash, overflow, past-ceiling, or fatal-process probe.**
- **No new `unsafe`.**
- **No `#[cfg(test)]` divergence in production code** (there should be no
  production change at all).
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Cost is not an argument** (user ruling 2026-08-27) — and neither is
  convenience: an entry that cannot be checked is not made acceptable by being
  written down.
- Minimal diff: no drive-by reformatting or renames.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. Your derived row count and its split, against the director's 50 / 6 / 44.
2. What the ledger file looks like — its schema and two or three real entries.
3. The three checks: what each verifies mechanically, and anything that could
   not be made mechanical, stated plainly.
4. The three falsifiers, each red then restored green, verbatim.
5. `cargo fmt --check` and the workspace total.
6. Confirmation that no production file and no inventory row changed.
7. Stop-loss / open questions — including what step E2, the `UnknownConstruct`
   second axis, still needs.
