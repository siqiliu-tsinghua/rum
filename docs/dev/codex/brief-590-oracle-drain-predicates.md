# brief-590 — E1 drain 4 of 8: `predicates`, `structure`, `structure_ordering`, and six carried entries (78 owed entries)

Implementation brief, **test-only**. **Do not run `git` in any form**,
including before you have read this brief. Leave every change uncommitted for
the director's gate.

## Goal

brief-586 built the catalog-wide suspension oracle
(`crates/rum-stdlib/tests/suspension_oracle/`) and left **500** declared
guest-effect entries in an exact owed ledger (`inventory/owed.tsv`). Drains 1-3
(briefs 587-589) took `rules`, `rewrite`, `control`, `logic`, `arithmetic` and
`random`; eleven declared roles that do not exist were removed at the
director's gate (owed 500 -> 319). **This one drains 78 entries**: exactly the
IDs listed in
`.temp/brief-586/followups/draft-E1-{10-predicates,19-structure,20-structure_ordering}.md`
— the predicates, comparators, key functions, `SameTest` callbacks and option
roles of the containment, ordering and set builtins, `Depth`'s options,
`Differences`' arithmetic and the lazy consumers of `First` and `Take` — **plus
six carried entries** whose policy the director corrected to `Tail`:
`Part.lazyConsumer`, `If.branch`, `Which.branch`, `Switch.branch`,
`Catch.handler`, `Enclose.handler`.

Each entry leaves the ledger **only in the same change that attaches and runs
its complete descriptor**: every required variant (`Middle`, `Twice`, and the
`Decline`, `Error`, `AlreadyResumed`, `Nested` variants its effect row
records), lifecycle at the first and second pause for every distinct
continuation owner (completion, cancellation, reset, a pending request
rejecting a new one), and a fresh request after.

Those drafts are the raw material; **this brief governs** where they differ.

## A failing descriptor is a finding, not a test to fix

If a correct descriptor shows the resumed path differing from the queued one —
a lost continuation the census did not see — **do not weaken the descriptor,
do not repair production code, and do not commit a failing test.** Keep the
entry owed, write the witness (source, answers, both observations) to
`.temp/brief-590/defects/<entry>.md`, and report it. It becomes a repair group
of this branch. **A role that does not exist** (an `Ordinary` builtin's option or operand
evaluated by shared preparation, a positional argument declared as an option,
option syntax admitted only literally) is reported with source evidence and
kept owed; the director removes it at the gate — briefs 587-589 found eleven.
Check the registration (`Ordinary` or held) and the argument partition first.
**A wrong policy is not a reason to keep an entry owed**: the oracle's equality
does not depend on whether the entry is `Frame` or `Tail`, so attach the
descriptor, and report the correct policy for the director to set (brief-589
held five tails owed for this; that rule is withdrawn).

## Context pointers

- briefs 586-589 and their reports; the three drafts above; the descriptors in
  `fixtures/rules/`, `fixtures/rewrite_roles/` and `fixtures/control_roles/` as
  the model.
- `crates/rum-stdlib/tests/suspension_oracle/{manifest,descriptors,catalog}.rs`,
  `support/`, `fixtures/`, `inventory/{catalog,effects,shared-effects,owed,historical-labels}.tsv`.
- The owning functions named in the drafts; the guide and the generated
  reference are the contract.
- The existing `lazy_pull_continuations.rs` (`First`, `Take`, `Part` lazy
  consumers) and `loop_continuations.rs` (`Differences`) cases, and the
  historical labels, reusable as descriptors. A `SameTest` or comparator
  callback must pause inside the builtin's own comparison loop, after earlier
  comparisons have run.

## Scope

**Edit surface:** the `suspension_oracle` test tree; the existing
continuation suites only to share descriptors (test names and specialized
assertions stay).

**Standing duty — always in scope** (`TEMPLATE.md`; test edits under `crates/`
expire the fingerprints): fingerprints, both certificates, ownership
reconciliation, every row disclosed. **STOP** on any existing row change or
disappearance, any new `UnknownHostPayload` or `self_reaching` row, any new
row in untouched code.

**Fence:** no production `.rs`, manifest disposition, `lib.rs`, Cargo
manifest, lockfile or dependency change; no other family's entries; no edits
to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier briefs, `scripts/` or
docs.

**Disclosure duty:** every entry drained, every entry kept owed and why, every
defect witness, every changed test name, every inventory row.

## Deliverable

Role-specific descriptor factories with explicit expected values, logs and
state projections and exact input kind, prompt and answer plans. A callback,
guard, option or held-target phase is exercised by its own role — ordinary
argument preparation never stands in. Loops run at least five operations and
pause in the middle and at a second position. Shared factories across the
family are welcome where they demonstrably exercise each entry's role.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix or resumed-only value the expected answer; expected
   values come from the documented contract and the queued path.
2. An eager-only or single-pause fixture proves nothing.
3. Support modules under 10k characters, scenario modules under 15k.
4. Stop-loss counters: entries drained, variants passing, lifecycle
   obligations passing, defects recorded. Stop after two consecutive compiling
   attempts that move none; fewer compiler errors is progress.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-590/tmp …`. Bounds: the oracle target 900 s per
build; runtime, ownership and retention 300 s; `cargo test --workspace
--no-fail-fast` 3 000 s.

```
cargo fmt --check
scripts/check-suspension-oracle.sh 900
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

The oracle's `CATALOG` lines must show `owed` reduced by exactly the drained
count in both builds. Report native ZeroMQ skips as skips.

## Checkpointing — protect the scene

Everything under `.temp/brief-590/`; `PROGRESS.md` after each milestone with
the counters and the exact next action.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, `scripts/check-suspension-oracle.sh`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only. No
  new `unsafe`.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Files changed; the no-production-edit audit.
2. Entries drained and kept owed (with reasons); owed before and after, per
   build; variants and lifecycle obligations added.
3. Defect witnesses, if any; the broad suites verbatim; every inventory row
   change.
4. Stop-loss counters, open questions.

---
