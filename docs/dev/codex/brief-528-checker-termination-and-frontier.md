# brief-528 — D1 continued: make the production scan terminate, then finish D1

Implementation brief. **Resume the working tree as it stands** — do not
restart, do not rewrite what already passes. Production source stays
read-only.

## Where this starts, measured by the director

brief-527 left 2 883 lines across ten checker modules and seven fixtures, and
reported **1 of 8 required tests passing**. That report understates the tree.
The director added one type annotation — `regions.rs:86`, `let mut classes:
BTreeMap<String, String>` — and measured the real state on this host:

```
production_obligations_are_exact         TIMEOUT/NO-RESULT
restricted_surface_rejects_six_shapes    ok. 1 passed;  finished in 0.01s
extracted_helpers_enter_closure          ok. 1 passed;  finished in 0.02s
unresolved_edges_are_errors              ok. 1 passed;  finished in 0.04s
coverage_cannot_shrink                   ok. 1 passed;  finished in 0.02s
trusted_items_have_exact_bounds          ok. 1 passed;  finished in 0.01s
implicit_operations_are_classified       ok. 1 passed;  finished in 0.04s
accepted_fixture_exercises_d2_shapes     ok. 1 passed;  finished in 0.01s
  ACCEPTED bodies=2 leaves=5 expression_facts=29 loops=1 adoptions=1 transfers=1
```

**Seven of eight pass, every one in under 0.04 s.** The accepted fixture
expresses all three shapes D2 needs — an accumulator held across a loop under
a lease, an adopted callback result, and a transfer to an outgoing owner —
which is the result D2's feasibility rested on. That annotation is already in
the tree; it is the director's edit, not yours to undo.

The eighth test is the problem, and the director located it:

- `production_obligations_are_exact` ran **99.8 % CPU, 590 MB RSS and
  climbing, for over 12 minutes**, and was killed without returning.
- Run with `--nocapture` it prints **nothing at all** — not even its first
  `MODULES=…` line. So the non-termination is inside **`source::production()`**,
  the whole-workspace module reconciliation, before any region, census,
  route or frontier work begins.
- Every other test drives `source::fixture(...)` or `fixtures::program()` —
  small in-memory sources. The scanner works on toy inputs and does not
  terminate on the real tree.

## Goal

Four things, in this order:

1. `source::production()` terminates, with its cause diagnosed and stated
   before it is fixed.
2. The obligations inventory stops depending on a design document's
   typography.
3. The eighteen falsifier transcripts exist.
4. The option-frontier report exists, with the resolved closure size.

Then D1 is complete and the director gates it.

## 1. Make the production scan terminate — diagnose before repairing

**State the mechanism first.** Say what actually fails to terminate, with
evidence: an unbounded module recursion, a path cycle, an exponential `cfg`
expansion, a fixed point over a lattice that is not finite, a visited set
that is keyed on something non-canonical. The memory growth says something is
being re-expanded rather than revisited. Report the cause, then the fix.

**Two things you may not do**, both of which would make the symptom vanish
without fixing it:

- **Do not narrow the scanned input.** Restricting the scan to one crate, to
  the option module, or to a file allowlist makes the test fast and the
  obligations census meaningless. The census must cover the production tree.
- **Do not add a depth cap, an iteration cap or a time budget as the
  termination argument.** A cap that fires is an unresolved edge being
  silently dropped, which is precisely the permissive unknown mode this
  checker exists to refuse. If a bound is genuinely part of the analysis —
  a finite lattice, a canonical visited set, an SCC-wise fixed point — say
  which, and why it is finite.

**The termination argument is a deliverable, not a side note.** One
paragraph: what the analysis's state space is, why it is finite, and what
makes each item enter it at most once.

**Budget.** The director measured this host before dispatch, warm tree:
`cargo test -p rum-stdlib` green in **36 s**, `cargo test --workspace` green
in **79 s**. A required test that adds minutes to every run is itself a
defect. `production_obligations_are_exact` must complete in **≤ 30 s** on
this host, and `cargo test -p rum-stdlib` must stay **under 120 s**. Report
both measured times. If you cannot reach the budget, STOP and report the
measured time and the mechanism — do not buy it by scanning less.

## 2. The obligations must not come from a design document

`regions::census()` currently recovers the 78 N names by `include_str!`-ing
`docs/dev/codex/brief-525-local-retention-class-census.md` and splitting
markdown table rows on `|` (`regions.rs:83-97`).

That is wrong in a way this project has been burned by repeatedly: a required
test now depends on the typography of a narrative document, and a
`|`-splitting parser is exactly the class of script-derived count that has
under-reported here before.

- `inventory/obligations.tsv` is the source of truth for the obligation rows.
- Cross-check it against **live source**: the declarations in
  `crates/rum-stdlib/src/catalog/memory_growth_contract.rs` and the resolved
  catalog routes. The check is against code, not against a report.
- Nothing under `docs/` may be `include_str!`-ed by the checker. If a class
  assignment originates in brief-525, transcribe it into the TSV with its
  evidence column; the TSV is then the artifact under test.
- The counts the census asserts (78 N, 147 unsplit, 90 unsettled) stay, but
  they are asserted against the TSV and the live catalog.

## 3. The eighteen falsifier transcripts

For each of the six shapes: the run where it is **red** with its pinned
diagnostic kind, canonical target where one applies, file and span, and the
witness path; the run where the valid map is restored and it is **green**;
and the run where **that specific check is disabled in the checker** and its
permanent negative test fails. Save complete stdout/stderr to a file under
`target/brief-528/` before quoting any of it.

An `is_err()` assertion, or "it did not compile", is not a transcript. Each
fixture must compile as real Rust first, so an unrelated error cannot
impersonate the intended rejection.

## 4. The option frontier

The report `production_obligations_are_exact` already prints is the right
shape; it has simply never run. Produce it: every resolved body, every
rejected edge with its reason, the proposed boundary records, and the
resolved closure size **beside** brief-526's name-spelling candidate count of
2 633 functions across 349 files. Do the same for the public wrapper
`merged_option_exprs_for_system_symbol`.

This is D2's scope input and the first real size number this line has.

## Scope

- **Files you MAY modify:** everything under
  `crates/rum-stdlib/src/tests/retention_closure/`, including its
  `inventory/*.tsv` and `fixtures/*.rs`; and
  `crates/rum-stdlib/src/tests.rs` for module registration only.
- **Files you MAY create:** further modules under that directory, if the
  termination repair needs them.
- `docs/testing-strategy.md` — two paragraphs recording the new required gate
  and what it does and does not guarantee. No weakening of anything there.
- **Everything else is read-only**, explicitly including every production
  `.rs` file in every crate, all `Cargo.toml` files and `Cargo.lock`. A
  needed production edit is a **STOP**, not a scope extension — it would also
  void this brief's "no A/B required" basis.
- Scratch goes under `target/brief-528/`.

## rum / Rust pitfalls (read twice)

1. **The scanner already works on fixtures.** Whatever you change must keep
   the seven passing tests passing, at their current speed. Re-run them after
   every change, not only at the end.
2. **A visited set keyed on a non-canonical identity does not terminate.**
   Two spellings of the same module or item must collapse to one key.
3. **`syn` is syntax, not resolution.** The fix is not to guess harder;
   unresolved stays an error.
4. **An unknown `cfg` arm, macro, or local function value is an error**, not
   a skipped body — and not a reason to expand both arms combinatorially
   either. Say which you do and why it terminates.
5. **Do not reorganize production control flow.** Production is read-only.
6. **A required test that hangs is worse than one that fails**, because it
   burns every future gate run. Treat the time budget as part of
   correctness.

## Verification (MANDATORY — you run it; iterate to green)

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 900  -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 900  -- cargo test -p rum-stdlib
python3 scripts/run-bounded.py --timeout 900  -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

Paste verbatim: all eight test results; the measured wall time of
`production_obligations_are_exact` and of `cargo test -p rum-stdlib`; the
eighteen falsifier transcripts; and the full frontier output.

**Microbench A/B: not applicable** — no production code changes, so no hot
path moves. State that rather than performing the ceremony. If you find you
need a production edit, that statement stops being true and you must STOP.

## Hard rules

- **Do not run `git` in any form.** Not `status`, not `diff`, not `log`. The
  last four briefs each ran it after being told not to. Running it is a
  violation to report, not a bootstrap convenience.
- Do NOT commit, branch, push, or touch git config. The director gates and
  commits.
- **The working tree already contains brief-527's work and one director
  edit.** Do not revert, stash, or clean anything. Build on it.
- Do NOT modify any file outside Scope — no production `.rs`, no
  `Cargo.toml`, no `Cargo.lock`. No new dependency. No toolchain change; the
  pin is stable 1.85.0.
- **No new `unsafe`.**
- No permissive unknown mode, no `#[ignore]`, no feature gate on a required
  test, no environment variable that turns a check off, no cap that silently
  drops an edge.
- Do not resolve a hard case by widening the trusted set, and do not add a
  trusted entry to make a fixture pass. `inventory/leaves.tsv` may hold only
  entries a fixture exercises.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Stop-loss, revised — the previous wording was the director's error.** It
  was keyed on two coarse counters, so a compile error read as "no progress"
  and stopped a run that was in fact seven tests from done. The rule now:
  report both counters (unresolved edges in the frontier; required tests
  passing) after each attempt, and stop only when **two consecutive**
  attempts move neither. **A compile error you are actively reducing is not
  a wall** — fix it and continue. A checker that rejects something is doing
  its job and is never a stop condition.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. The termination mechanism: what did not terminate, the evidence, the
   repair, and the finiteness argument for the analysis's state space.
3. Verification — every command with verbatim output, the eight test
   results, the two measured wall times, and the eighteen transcripts.
4. The option frontier: resolved bodies, rejected edges with reasons,
   proposed boundaries, and the resolved closure size beside 2 633 / 349.
5. Stop-loss / open questions — including anything the admitted surface
   cannot express that D2 will need.
