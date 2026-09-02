# brief-413 — audit: find security vulnerabilities in rum

Shape: **security audit**. `Scope: NONE — no source change, not one file.`
Worker: **cgl (GLM)**, necessarily.

## Why you

Codex has an upstream cybersecurity content filter, and it has cost this
project twice: brief-289 had to be written in an engineering-hygiene register
rather than saying what it meant, and brief-208's report was **truncated
mid-write** when the filter fired, which is why its protection-bypass section
was missing and looked like a fabricated citation. **A reviewer cannot audit an
attack surface in a register that forbids naming it.** You have no such filter.
Write plainly: bypass, escape, escalation, exhaustion, forgery, confusion.

## I am not telling you where to look

This is deliberate and it is the user's instruction. No prioritised areas, no
suggested categories, no checklist. Range wherever the evidence takes you.
Everything below is context you would otherwise have to reconstruct, or a
statement about what counts as a finding — not a boundary on the search.

## The threat model, as the project states it

`SECURITY.md`: rum is "a **language kernel and embedding library**, not a
network service or an application — its security model is about evaluating
*untrusted `rum` scripts* safely inside a trusted Rust host." Every session runs
under a host-chosen policy.

So the adversary is the script, the asset is the host, and the claim is
containment. **If you think that model is wrong, incomplete, or quietly assumes
something it should not, say so — that is a finding in its own right**, and
possibly the most valuable one.

Where the posture is written down:

- `SECURITY.md` — the stated model and its scope.
- `docs/guide/resource-limits-and-sandbox.md`, `docs/guide/design-boundaries.md`.
- `docs/host-embedding-and-hardening.md` — the embedding contract, per-request
  isolation, crash-only behaviour, stdout ownership.
- `docs/guide/embedding.md` — the public `rum-host` surface.
- Capabilities live in `crates/rum-runtime/src/runtime/capabilities.rs`; named
  profiles pin every one of them.
- `AGENTS.md` and `ARCHITECTURE.md` for the crate layout and hard lines
  (zero `unsafe` in `rum-core` and `rum-runtime`, no default-path cost).

## One distinction that is not negotiable, so you do not waste a finding on it

**`Protected` is a mistake-guard and WL conformance, not a sandbox boundary.**
That is a standing project ruling. A script defeating `Protected` is a
correctness bug at most, not an isolation failure. The real containment is the
capability gates, the resource ceilings, and per-request isolation. Judge
against those.

Everything else about the model is open to challenge, including that ruling's
consequences.

## Why now

The line that just closed (briefs 397-411) was not a security audit, and it
found two isolation failures by accident:

- Abandoning a suspended evaluation leaked the `AbortProtect` depth its frames
  owned, so **evaluation continued past `Abort[]`** in a later request.
- The same discard leaked a time-constraint deadline, so **an unrelated later
  evaluation was terminated** by a limit it never set.

Both were reachable through public `rum-host` API alone. They are fixed. Nobody
has asked how many more of that shape exist, or of any other shape, because
this codebase has never had an audit that was allowed to say "attack".

## What I want back

Findings, ranked by severity, each with:

1. **What an attacker gets** — stated concretely. "Evaluation continues past
   `Abort[]` in a subsequent request on the same session" is a finding;
   "insufficient validation" is not.
2. **A reproduction**, or an explicit statement that you could not produce one
   and why. A script through the release `rum-cli`, or a small `/tmp` crate
   against `rum-host`, is ideal. Speculative findings are welcome **so long as
   they are labelled speculative** and carry the reasoning that would let
   someone confirm or kill them.
3. **`path:line` evidence** for the mechanism.
4. **Which stated guarantee it breaks**, quoted from the doc that states it —
   or a note that no document makes the guarantee, which is itself worth
   knowing.

Also tell me what you looked at and found sound. A negative result with
evidence is worth more than silence, and it tells the next audit where not to
spend its time.

**Do not draft repair briefs.** What to do about a finding is the director's.

## Method requirements

These are about honesty, not scope.

- **State first, verbatim, what your environment can actually run.** In
  particular the nine native ZeroMQ tests in `rum-jupyter` skip where loopback
  binding is not permitted, and until 2026-09-02 libtest reported such a skip
  as `ok` — which is how an earlier brief recorded a stability result that
  measured nothing. There is now a gate: without `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`
  a host that cannot bind fails all nine loudly.
- **Never report a rate, count, or stability claim for a test you did not
  observe running.** A total you computed is not a line you quoted; do not put
  a derived number under a heading marked verbatim.
- Say plainly where you reason from source rather than from observation.
- **A confident wrong answer is worse than "I could not establish this."**

## Hard rules

- **`Scope: NONE`. Change no file** — not source, not tests, not docs. You may
  read anything, run tests, build throwaway crates under `/tmp`, and run the
  release `rum-cli`. If you believe a file must change, that is a finding.
- Do NOT `git commit`, branch, push, checkout, stash, or touch git config. Do
  not touch `stash@{0}`. Reading history with `git log`, `git show`, `git diff`
  is expected.
- Touch nothing outside this repo except `/tmp` scratch you remove afterwards.
  No network. Pass `--offline` to every `cargo` command.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only. No vendor engine, oracle harness, or compatibility matrix.
- **Nothing destructive.** Do not attempt anything that would damage this
  machine, its files outside the repo, or anything beyond it. The target is
  rum's own containment, evaluated locally, and nothing else.
- Paste genuine verbatim output.

## Report format

1. Environment capability, verbatim.
2. Findings, ranked by severity, in the shape described above.
3. What you examined and found sound, with evidence.
4. What you could not establish, and why.
5. Anything about the stated threat model you think is wrong.
