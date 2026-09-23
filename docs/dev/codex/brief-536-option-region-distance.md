# brief-536 — design: how far is the option region from zero unknowns?

`Scope: NONE — no source change.` Design-only. Survey, reason, return a
disposition per rejection and a distance estimate. Write no code. **Do not run
`git` in any form**, including while first loading this repository.

## Goal

A2 admitted proved inherent method selection and the option frontier did not
move: it is still **14** rejections over four bodies in three files. What
changed is the diagnosis. The eight method rejections were one blanket "method
syntax is outside admitted resolution"; they are now specific:

```
4  method syntax: no unique inherent definition; user Deref/unsizing/trait dispatch is not searched
2  method syntax: method receiver lacks checked nominal/place provenance
1  method syntax: opaque imported trait candidate rum_runtime::runtime::metadata::ALL_ATTRIBUTES
1  method syntax: method instance has unknown bounds or argument arity
2  primitive operand proof or exact operator/conversion impl required
1  implicit Try/FromResidual/error ownership not modeled
1  closure capture/callback effects unresolved; closure body is a rejected boundary
1  unexpanded macro; contents are not resolved Rust
1  not a resolved function definition: std::vec::Vec::new
```

**Answer one question: what would it actually take to make C01/C02 a
zero-unknown Covered region, and is that reachable?** The remaining plan
(A3 onward) assumes growing the surface is the way. Four of these fourteen
name things that are on the never-admit list, which is a reason to check the
assumption before spending three more stages on it.

## What I want back

### 1. A disposition per rejection, all fourteen

For each, with `path:line`: the site, what the checker needs, and **one** of:

- **`AdmitLater`** — a named future stage would clear it. Say which, and
  give the soundness argument in one or two sentences. "A3 will handle it" is
  not a disposition; *why it is soundly admissible* is.
- **`Rewrite`** — clearable now by changing the covered region into the
  current admitted surface. **Sketch the concrete rewrite** — the actual
  shape the code would take — and say whether it is idiomatic Rust a reviewer
  would accept, or a contortion. Name the file and whether it sits on a hot
  path.
- **`NeverAdmit`** — needs user `Deref`, trait dispatch, open generics,
  unsizing or a function-pointer call. Then say what the region must become
  instead, or that it stays uncovered.

The `RET001` on `Vec::new` is the line's actual defect and is D2b's lease
work — dispose of it as such, not as a resolution problem.

### 2. The layer behind the frontier — this is the real question

Fourteen is the count of rejections **at the frontier**. Clearing them
exposes the bodies behind them, which carry their own. The N frontier already
went 2 732 → 6 014 when A2 resolved methods.

So: **if all fourteen were cleared, what is the next layer for C01/C02
specifically?** Estimate it from the census data already in the repository —
the resolved and rejected edges reachable from those four bodies — and say:

- how many bodies a zero-unknown C01/C02 region would have to close;
- what the rejection distribution across that set looks like;
- whether it converges — does each cleared layer expose fewer than it clears —
  or whether the reachable set keeps growing.

**If it does not converge, say so plainly.** That is the most useful finding
this brief can produce, and it is not a failure.

### 3. The recommendation, with its cost

Given (1) and (2): grow the surface through A3-A5, rewrite the region, some
mix, or something else. State what you would do and what it costs in
implementation steps. **Cost is not an argument against a correct answer on
this project**, but an unconverging plan is not made correct by being
expensive — say which you think this is.

### 4. What a rewrite would cost the code, honestly

If you propose rewriting: would a reviewer who knows nothing about this
checker accept the result as reasonable Rust? The standing rule from the
traversal-depth line is **move locals, not control flow**; a covered region
sits on paths the evaluator uses, and a rewrite that reshapes control flow to
please an analyzer is a stop, not a solution. Say where your sketches cross
that line.

### 5. Open questions for the director

## Settled — do not reopen

- Closure is the destination; unknown constructs reject. No permissive mode.
- The analyzer is ours; read rustc and rust-analyzer freely, and the compiler
  oracle checks our claims.
- The line targets the N class. P is a documented bound.
- `rum-core`, the interner, gensym, the snapshot format, the `S` model,
  leases and admission policy are out of scope here.
- No promotion. Production coverage stays 0 until a region is actually
  proved.

## rum / Rust pitfalls (read twice)

1. **A frontier count is not a distance.** Rejections behind a rejected edge
   are invisible until it resolves; do not report the frontier as the work
   remaining.
2. **The census data is already in the repository** — use it rather than
   re-deriving, and say which tables you read.
3. **A count derived by regex is suspect here.** Cross-check against live
   source reads and say which rows you checked by hand.
4. **A rewrite sketch that has not been type-checked is a sketch.** Say so;
   do not present it as verified.
5. **`Vec::new` in C01 is the defect the line exists to close**, not an
   analysis gap to engineer around.

## Scope

- Files you MAY modify: **NONE**, with two exceptions:
  - append your report to **this** file,
    `docs/dev/codex/brief-536-option-region-distance.md`;
  - write scratch artifacts under `target/brief-536/`.
- Everything else is read-only. No source change, no test change, no doc
  change, no `Cargo.toml` change.

## Verification (MANDATORY — the evidence IS the deliverable)

- State every command you ran; persist raw output under `target/brief-536/`
  before parsing it.
- Paste the fourteen dispositions as a table, and the distance estimate with
  the numbers it rests on.
- State explicitly anything you could not settle.

Existing checker tests must still pass unchanged — you are changing nothing,
so run them once to confirm the tree is as described: twenty-two green, warm
in about 26 s, `cargo test --workspace` green in about 97 s.

## Checkpointing — protect the scene

After each milestone write `target/brief-536/PROGRESS.md` with what is done,
the exact next action, and anything the director needs to take over.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the two exceptions in Scope. If you believe
  you must, STOP and write that in the report instead.
- Touch nothing outside this repo. No network.
- **No code.** If you are editing a `.rs` file, you have left the brief.
- **Testing Policy is a hard line** (`AGENTS.md`).
- Do not resolve a hard disposition by widening the trusted set or by
  proposing a permissive mode.

## Report format (your final message, appended to this file)

1. What changed — "design only, no source change", plus scratch artifacts.
2. Verification — commands and the verbatim tables you derived.
3. The fourteen dispositions.
4. The distance estimate, and whether it converges.
5. The recommendation with its cost, and what a rewrite would cost the code.
6. Stop-loss / open questions.
