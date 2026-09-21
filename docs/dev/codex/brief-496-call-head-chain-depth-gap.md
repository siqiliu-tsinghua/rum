# brief-496 — a source shape that evades the parser's depth guard

**Shape: DIAGNOSIS + DESIGN. `Scope: NONE — no source change.`** The
deliverable is the remaining attribution, a recommended repair layer, and
drafted implementation briefs. You write no production code.

Most of this is already measured. **Do not re-derive what is below**; the one
open question is named in its own section.

## What is established, by director measurement

All at `ulimit -s 2048`, the ruled 2 MiB embedder stack, three runs per point,
release CLI.

**The guard exists, works, and one shape walks around it.** Two programs of the
same AST spine depth, 300:

| source | result |
|---|---|
| `Hold[f[f[f[…1…]]]]`, nesting 300 | **refused in band** — the parser's depth guard fires |
| `Hold[f[1][1][1]…]`, 300 applications | **accepted** |

`MAX_DELIMITER_NESTING_DEPTH` and `MAX_OPERATOR_CHAIN_DEPTH` are both 128
(`rum-syntax/src/parser/front.rs:10,22`, tested at `:117-151`). A **call-head
chain** boxes the prior head — `SyntaxExpr::call(expr, args, span)` — without
touching either counter, so it is unguarded at any depth.

**Where it ends, and it fails open.** With the chain evaluated from a script
(`x = Hold[f[1]…];`):

| applications | result |
|---|---|
| 4 097 — the ruled contract height | 3/3 survive |
| 5 000 | 3/3 survive |
| 6 000 | **3/3 killed**, `thread 'main' has overflowed its stack` |

That is **1.22x-1.46x the contract height, failing open** — a process kill, not
an in-band error. It sits in the same band as the residual queue's thinnest
entries (`Lookup` 1.16x, `KeyComplement` 1.42x).

**Parsing is not the phase.** `--format`, which parses the same input and then
builds and renders the formatter's `Doc` tree, **survives 6 000 3/3**. So the
overflow is downstream of the parse: AST-to-`Expr` lowering, evaluation, or the
`Expr` drop.

**A correction this forces.** `TODO.md` §Standing engineering debt says deep
`Expr` trees overflow on drop but that **"source-created trees are
depth-guarded"**. For this shape that sentence is **false**: ordinary source
builds a 5 000-deep spine. Whatever the repair, that entry must stop claiming a
protection this shape does not have.

**A second, separate defect, not this brief's subject.** `rum --parse <expr>`
on the same chain **times out** (exit 124 at 60 s) from about 1 000
applications, while `--format` on the same input is fast. The parse path is
fine, so the syntax-tree *rendering* appears superlinear. Record it; do not fix
it here.

*(Method note, since it nearly cost a wrong conclusion: an exit-status-only
probe counted those timeouts as stack overflows. rum exits 1 on a syntax error
and 124 on timeout, so "nonzero" means nothing on its own. Read the failure,
not the status.)*

## The one open question

**Which phase overflows** — AST-to-`Expr` lowering, evaluation, or the `Expr`
drop? Settle it from source and with bounded probes that do not drive a process
to termination past what is already measured above.

It matters because the repair differs:

- If it is the **`Expr` drop**, this is the recorded standing debt reached by a
  source path the debt claims is guarded, and the repair is at the guard.
- If it is **lowering**, there is an unguarded recursive descent in the
  AST-to-`Expr` conversion, and it belongs with the traversal-depth line's
  other conversions.
- If it is **evaluation**, `Hold` should have kept it shallow and that
  expectation is wrong.

## The design question

Where does the repair belong, and what does it cost the language?

1. **Extend the parser's guard to call-head chains**, so the chain is refused in
   band exactly as the nested form already is. Smallest and most consistent —
   but it **changes the accepted source surface**: `f[1][1]…` beyond the limit
   becomes a syntax error where today it is valid. Say what limit, and say
   plainly that this is a user-visible language change; the director will put
   it to the user rather than decide it silently.
2. **Bound the failing phase itself** (a guarded front plus an iterative tail,
   as briefs 439-475 did ten times), leaving the source surface untouched.
   Larger, and it must say which walker and what the certificate is.
3. **Both**, if the guard alone leaves the host-constructed path open — the AST
   types are `pub`, so a host can build any depth without the parser.

For each: what fails closed afterwards, what the certificate is, and whether
the `Expr` standing-debt entry can then be corrected rather than merely
reworded.

## Scope

- **Files you MAY modify: NONE.** The report is the deliverable, with the
  drafted briefs inside it.
- **Do not reproduce the crash beyond what is measured above**, and do not
  probe past the ceiling — that bracket is the director's. Bounded probes below
  5 000 are fine.

## Hard rules

- Do NOT `git commit`, branch, push, touch git config, or run `git stash` /
  `git checkout`.
- Do NOT modify any file. If a change seems needed to answer a question,
  describe it.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`** in any drafted brief.
- **Do not propose reopening `Expr`'s iterative `Drop`** — three designs were
  rejected on a measured 20-50% hot-path regression.
- **Cost is not an argument** (user ruling 2026-08-27), but a **user-visible
  language change is not the director's to make silently**: flag it, do not
  bury it in a recommendation.
- Read the failure, not the exit status, for every probe you run.
- Paste verbatim output for anything you run; if you did not run something,
  say so.

## Report format

1. "Diagnosis and design, no source change", and the phase attribution with
   `path:line` evidence and the probes that settled it.
2. Every command run, verbatim.
3. The design options, the recommendation, and explicitly what it would change
   about the accepted source surface.
4. The drafted implementation briefs.
5. What the `Expr` standing-debt entry should say afterwards.
6. Stop-loss / open questions.
