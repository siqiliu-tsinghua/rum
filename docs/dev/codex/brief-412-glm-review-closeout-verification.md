# brief-412 — audit: independent verification that the 2026-08-31 review's findings are actually closed

Shape: **diagnostic / audit**. `Scope: NONE — no source change, not one file.`
Worker: **cgl (GLM)**, deliberately.

## Why you and not the other worker

The 2026-08-31 consistency review of briefs 364-388 was run in four
independent parts; three of them were yours. Its eight findings were repaired
by briefs 393-396. **The close-out verification was then done by Codex alone,
because your quota was exhausted at the time.** That verification is the debt
this brief pays.

The point is not to re-run what Codex already ran. A second worker is worth
having because it has a different blind spot, so **do not try to reproduce
Codex's conclusions, and do not assume a repair is sound because it was
accepted.** Judge the repairs against the findings yourself.

## What to verify

The four repair commits:

| Brief | Commit | Claim |
|---|---|---|
| 393 | `d423d2d3` | Preparation no longer evaluates inside pattern regions; the list of protected wrappers became a registry shared with the matcher |
| 394 | `3a7a4f9f` | The interner-order-dependent assertion class was swept, not just its two sightings; each group splits semantic coverage from optimization coverage on a lane-disjoint target |
| 395 | `090286a8` | Two boundaries at the published surface were closed |
| 396 | `aa8644cf` | The review's remaining four items were closed |

For **each of the eight findings**, answer three questions with `path:line`
evidence:

1. **Is it actually closed**, or closed only for the instances that were
   named? This project has already been caught once closing sightings rather
   than a class — brief-388 repaired two assertions when seven rested on the
   same mechanism, and brief-394 exists because of it. Look for the same shape
   again.
2. **Did the repair replace a convention with a structure, as claimed, or
   relocate the convention?** A registry that a new call site can forget to
   consult is a convention wearing a structure's name. A repository test that
   scans in-repository sources is a convention too — see below.
3. **Did the repair introduce anything?** A narrowed public surface, a changed
   documented behaviour, a new obligation on a caller, a test that now asserts
   less than it did.

## The specific thing to be suspicious of

Several of these repairs rest on repository invariant tests that scan the
repository's own sources. **A source scan cannot establish a property about
code that does not exist yet.** That lesson came out of this very review —
two of its findings were boundaries that held inside the repository and
evaporated at the crate edge, and the gate that settled them compiled an
external crate and required the compiler errors.

So for every repair whose enforcement is a source scan, say explicitly whether
the property survives at the crate edge, and if you are unsure, say that
rather than assuming. `rum-runtime` and `rum-host` are the crates whose public
surface matters here.

## What else is worth your attention

- `TODO.md` §"Opened by the 2026-08-31 consistency review (briefs 389-392)"
  records what the review left open and what has since closed. Check that
  those closures are real and that nothing was quietly dropped.
- The repairs landed before `v0.2.0` (`77029938`). Anything in the 308 commits
  from `v0.1.0` to `v0.2.0` is fair reading material.
- Since the review, briefs 397-411 have changed suspension, cancellation, and
  the request boundary substantially. **If one of those changes has since
  weakened or bypassed one of these repairs, that is the most valuable thing
  you could find**, and nobody has looked.

## First, state what your environment can do

Before anything else, run the test suite once and report **verbatim** what ran
and what did not. In particular the native ZeroMQ tests in `rum-jupyter` skip
where loopback binding is not permitted — and until 2026-09-02 a skip was
reported by libtest as `ok`, which is how an earlier brief recorded a
stability result that measured nothing. There is now an explicit gate: without
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP` a host that cannot bind fails all nine loudly.

**Never report a rate, count, or stability claim for a test you did not
observe running.** If your sandbox skips them, say so and set the variable.

## Deliverable

1. The environment statement, verbatim, first.
2. Per finding: closed / partially closed / not closed, with `path:line`
   evidence and a one-line justification.
3. For each repair, the convention-versus-structure judgement, and the
   crate-edge judgement where enforcement is a source scan.
4. Anything briefs 397-411 have since weakened.
5. New findings, if any, ranked by severity, each with the evidence that would
   let a director confirm it independently.
6. **Do not draft repair briefs.** This one is verification; if you find
   something, the director decides what to do about it.

## Hard rules

- **`Scope: NONE`. Change no file — not source, not tests, not docs, not
  `TODO.md`.** You may read anything and run tests. If you believe a file must
  change, that is a finding, not a licence.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`. Reading history with `git log`, `git show` and
  `git diff` is expected and encouraged.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. Pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No vendor engine, oracle harness, or compatibility matrix.
- Every claim carries `path:line` evidence. Say plainly where you are reasoning
  from source rather than from observation.
- Paste genuine verbatim output. **A confident wrong answer is worse than "I
  could not establish this."**

## Report format

1. Environment capability, verbatim.
2. Per-finding verdicts with evidence.
3. Convention-versus-structure and crate-edge judgements.
4. What briefs 397-411 may have weakened.
5. New findings, ranked.
6. What you could not establish, and why.
