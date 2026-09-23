# brief-533 — A0: the rejection census, and the oracle that will check our claims

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including while first loading this
repository.** Everything you need about the tree's state is in this brief.

## Goal

Two deliverables, and the second is the one that matters later.

1. **The census.** A repeatable rejection distribution over every production
   definition the checker already discovers, and over the N-root frontier,
   with exact reasons and spans, and with unresolved attachments kept visible
   rather than collapsed into an empty success. **Change no admission rule.**
   This is the measured frequency order the rest of the line is planned by.

2. **The differential harness** that uses the Rust compiler as an oracle for
   the checker's own claims. Build it now, before a single resolution or type
   claim is trusted.

## Why the oracle comes before the claims

The user ruled on 2026-09-23 that this analyzer is **ours**: it does not need
to be general, only to understand the subset of Rust that rum's covered
regions use, and owning the API removes the version and churn dependency that
`rustc_private` or the `ra_ap_*` crates would bring.

The cost of that ruling is that the checker's soundness is our own. Every
later stage claims things like *"this call resolves to that definition"* and
*"this expression has that exact type"*. Those claims are load-bearing, and
this project's standing rule is that **an instrument nobody has watched fail
is not known to work**. So the oracle is built first, not after.

Two further consequences of the ruling, which apply to this brief and every
one after it:

- **Read rustc's and rust-analyzer's source freely** for how they solve name
  resolution, method resolution and candidate ordering, places, moves and
  borrows. Do not re-derive a solved problem by guessing. Both are
  MIT/Apache-2.0 and rum is MIT: **code copied carries its copyright notice**;
  something adapted in spirit carries nothing. Say in your report what you
  adapted and from where.
- **Using rustc as an oracle is not the banned differential testing.** The
  Testing Policy hard line forbids black-box comparison against *other WL
  engines* and oracle harnesses for rum's own language behavior. Checking a
  Rust-language fact about rum's own source, with the toolchain that already
  builds the project, is a different thing. State this in the code comment
  where the harness lives so nobody later mistakes it.

## The harness — three techniques, all stable-only

The compiler is the oracle. Each technique compiles a scratch copy of real
repository source under `target/brief-533/`; none of them modifies production.

1. **A type claim.** Pin it by *unification*, not annotation:

   ```rust
   fn assert_type<T>(_: T) {}
   assert_type::<ExactType>(expr);
   ```

   A generic parameter unifies exactly. `let _: ExactType = expr;` does
   **not** test the same thing, because coercion can make a wrong claim
   compile.

2. **A resolution claim.** For a call the checker says targets definition
   `T`, mutate `T`'s signature in the scratch copy — add a parameter — and
   require a compiler error **at that exact call site**, matched on file and
   line, not merely "the build failed".

3. **The negative control, which is what makes (2) mean anything.** Mutate a
   *different* definition with the same name, and require **no** error at
   that site. Without it a resolution claim passes whenever everything
   errors, which is the shape that has fooled this project's probes before.

Run the harness over a sample of the census's currently resolved edges, and
report agreements and disagreements. **A disagreement is a finding to report,
not a failure to hide** — it means the checker is already wrong somewhere, and
I would rather learn that now than after four more stages are built on it.

## The census

- Keep the existing `OPTION_FRONTIER` and `PUBLIC_WRAPPER` outputs
  unchanged, and add separately labelled `EDGE_ALL` and `EDGE_N_FRONTIER`
  tables.
- `EDGE_ALL` visits **every indexed production definition once**, not only
  the catalog-reachable ones. A conditional, ambiguous or unresolved root
  produces a located rejection row; it must not panic and must not silently
  vanish.
- `EDGE_N_FRONTIER` starts from every N attachment in the live inventory,
  follows exactly resolved edges, and emits the boundary where traversal
  stops. The **nine unresolved attachments stay visible**. If C01/C02 are
  added as pilot seeds, label that population separately rather than quietly
  redefining N.
- Rows carry population, originating root set, canonical definition, kind,
  span, target, witness and the full reason. Count unique
  `(definition, site, kind, target, reason)`; report per-root incidence
  separately so a shared helper is not multiplied. Show raw and deduplicated
  totals with deterministic ordering.
- Report `effects::check` first-failure results **separately** from
  edge-site frequency. The first is what blocks certification; it is not a
  count of every rejection site. Neither is a coverage certificate.
- Cross-check both tables with an independent parser and with source-span
  reads per reason class. A regex-derived count in this repository has been
  wrong more than once; say which rows you checked by hand.

**Acceptance baseline, unchanged by this brief:** `OPTION_FRONTIER` 17
(RET001 1, RET002 15, RET003 1), `PUBLIC_WRAPPER` 9, production coverage 0.
If any of those move, you changed an admission rule, which this brief
forbids — stop and report.

## Scope

**An objective and a fence, not an enumeration.** A MAY list has been short
six times on this project; the guidance below is guidance.

**The objective.** Inside `crates/rum-stdlib/src/tests/retention_closure/`
and its registration, build the census and the harness — including files the
guidance does not name.

**The disclosure duty, which replaces asking permission.** Every file you
touch that the guidance does not name appears in your report's first section
with `path:line` ranges and one line on why. **An undisclosed edit is the
violation; a disclosed necessary edit is not.**

**Guidance**: the checker's own modules; `crates/rum-stdlib/src/tests.rs` for
registration; `docs/testing-strategy.md` for what the new gate does and does
not guarantee.

**The fence — STOP conditions:**

- **All production Rust is read-only**, in every crate. This brief measures
  and validates; it changes no behavior. A needed production edit is a stop.
- **No admission rule changes.** No new idiom is admitted here. The baseline
  numbers above must not move.
- `crates/rum-core/`, all `Cargo.toml` files and `Cargo.lock`. No new
  dependency, no toolchain change; the pin is stable 1.85.0 and this brief
  does not need nightly for anything, the harness included.
- The interner, gensym, snapshot format, the `S` model, leases and admission.
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`.

## rum / Rust pitfalls (read twice)

1. **A source index is not a semantic call graph.** Traversal blocked at a
   method does not include its unproved callee; keep the two populations
   distinct.
2. **`let _: T = expr` coerces.** Use unification, as above, or the type
   claim proves less than it appears to.
3. **"The build failed" is not a resolution proof.** Match the error's file
   and line, and keep the negative control.
4. **A scratch mutation must not touch the real tree.** Copy the source into
   `target/brief-533/` and compile there.
5. **Reporting under a rejected parent is enumeration, not acceptance.** No
   census output may read as a coverage certificate.
6. **Conditional roots must neither crash the census nor disappear from it.**
7. **A count you derived with a regex is suspect.** Cross-check by hand and
   say which rows.

## Verification (MANDATORY — you run it; iterate to green)

Director measurements on this host, warm tree, immediately before dispatch:
eight checker tests green in **8.2 s**, `cargo test --workspace` green in
**80 s** with no failures. The tree is green at your starting point.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 900  -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Proposed required tests: `production_rejection_census_is_complete`,
`n_frontier_keeps_unresolved_roots`, and the harness's own
`compiler_oracle_agrees_with_resolution_claims` including its negative
control.

**Falsifiers, watched red then green:** a census that omits one production
definition or one root; a resolution claim checked against a mutated target
that the harness must catch; and the negative control disabled, which must
make the harness's own test fail. Save complete output under
`target/brief-533/` before quoting.

**Microbench A/B: not applicable** — production is read-only, so no hot path
moves. Say that rather than performing the ceremony; if you find you need a
production edit, that statement stops being true and you must STOP.

Nine native ZeroMQ tests cannot bind loopback in your sandbox. Report them as
skipped-by-environment; the director settles them. Do not set
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Hard rules

- **No `git`, in any form.**
- Do NOT commit, branch, push, or touch git config.
- Respect the fence; outside it, the disclosure duty applies.
- **No new `unsafe`.** No new dependency. No nightly.
- No `#[ignore]`, no feature gate on a required test, no permissive mode.
- **Testing Policy is a hard line** (`AGENTS.md`) — and the compiler-oracle
  distinction above is a director ruling, stated so it is not mistaken for a
  violation.
- Scratch goes under `target/brief-533/`.
- **Stop-loss, keyed on progress.** Report two numbers per attempt: census
  populations produced, and harness techniques passing their own controls.
  Stop only when **two consecutive** attempts move neither. A compile error
  you are actively reducing is not a wall.

## Report format (your final message)

1. What changed — per file with line ranges, including every file the
   guidance did not name, with why; and what you adapted from rustc or
   rust-analyzer, with the source.
2. The census: verbatim `EDGE_ALL` and `EDGE_N_FRONTIER` reason tables, raw
   and deduplicated, and the resulting frequency order.
3. The harness: the three techniques, their controls, and the sample of
   resolution claims checked — with every agreement and **every
   disagreement**.
4. Verification — every command with verbatim output, and the falsifiers
   red-then-green.
5. Confirmation that the acceptance baseline did not move.
6. Stop-loss / open questions.
