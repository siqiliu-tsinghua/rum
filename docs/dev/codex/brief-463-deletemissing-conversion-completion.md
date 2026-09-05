# brief-463 — complete brief-462's `DeleteMissing` conversion

## Situation — read this first

**The working tree already contains brief-462's uncommitted conversion. Build on
it; do not restart it and do not revert it.** That brief stop-lossed correctly
on a failing certificate and left three things undone. Its conversion itself is
sound — the director has verified it — and this brief closes the gaps so it can
be committed as one correct change rather than a regression now and a repair
later.

**What the director already established at the gate, so you do not repeat it:**

- **The failing armed certificate was a bad probe, not a defect.** It used an
  *absolute* ceiling, `MemoryConstrained[…, 1000000000]`, while `MemoryInUse[]`
  at physical height 4,097 is **2,810,469,079 bytes**. The limit sat below
  current usage, so the call aborted before running and returned `$Aborted` — a
  Symbol, hence the "non-list". **The director has already changed both armed
  certificates to `MemoryInUse[] + 1000000000`, and all three release
  certificates now pass. Leave that fix in place.**
- `cargo fmt --check` passes and `cargo test --workspace` is **2 377 tests
  green** on a loopback-capable host with no ZeroMQ skip.
- The heap tail handles branching correctly. Director probe at height 4,097 with
  two entries per level plus a per-level `"keep" -> i` witness: every level keeps
  `{"k", "keep"}` in order with the right `i`, and the deepest keeps only
  `{"y"}`. **That property is verified but not checked in**, which is deliverable
  2.

## Deliverable 1 — restore the default-path allocation profile

Measured at the gate, base against candidate, both built at the same filesystem
path, `RUM_PROFILE_ALLOC=1` on a `--features alloc-count` build, 300 calls over
a 9-level fixture:

| probe | base | candidate | delta |
|---|---:|---:|---|
| `DeleteMissing[a, Infinity]` | 37 403 | 40 103 | **+2 700 (+7.2%)** |
| the same under an armed `MemoryConstrained` | 862 006 | 865 906 | +3 900 |
| untouched control | 4 649 | 4 649 | 0 |

**+2 700 over 300 calls on a 9-level fixture is exactly +1 allocation per
association level**, and the fixture is shallower than the 16-level front, so no
heap tail runs at all. The cause is in
`delete_missing_from_association_entries_recursive`
(`crates/rum-stdlib/src/associations/missing/traversal.rs`): the base consumed an
owned `Vec<AssociationEntryForm>` with
`entries.into_iter().filter_map(…).collect()`, which Rust can satisfy **in
place, reusing the source buffer**; the candidate rebuilds into a fresh
`Vec::with_capacity`, so the reuse is lost on every level.

Restore an allocation-neutral shape while keeping the bounded front. `retain_mut`
over the owned `entries` with a captured exhaustion flag is the obvious
candidate — it mutates in place and needs no second buffer — but any shape that
measures neutral is acceptable. **The gate is the number, not the idiom.**

The sibling `delete_missing_from_list_recursive` is **not** affected: its base
iterated a borrowed slice and always allocated, so the candidate's
`Vec::with_capacity(items.len())` is equivalent. Do not change it.

**Acceptance: allocation counts on all three probes above must return to
base**, byte-identical, with the counter's sensitivity calibrated and the
same-binary spread stated.

## Deliverable 2 — the forced-front falsifier brief-462 owed and did not deliver

A conversion has no boundary override to force, so its proof is that the
**bounded front actually hands off and the tail agrees with it**. Ship a
checked-in test that forces the front constant to zero or one behind a test-only
path and asserts that a **shallow branched** fixture produces the exact same
value through the heap tail as through the front.

The fixture must have **more than one entry per level and a distinguishing
witness per branch**, so that a tail which drops, reorders, or mis-nests a
branch fails. The director's probe shape is the model: two entries per level
plus `"keep" -> i`, asserting keys, order, and the witness value at more than
one depth.

Without this, every certificate in brief-462 runs a single chain, and branch
handling in the tail is untested. That is the gap this deliverable exists to
close — the whole certificate panel of this line was once green while exercising
nothing, for exactly this kind of reason.

## Deliverable 3 — bring the new file under the organization line

`crates/rum-stdlib/src/associations/missing/traversal.rs` is **35 131 bytes**,
over the 35 000 architecture-debt threshold in
`docs/code-organization-guidelines.md`. `missing.rs` itself came down from
33 236 to 26 353, so the split worked; it just landed the new file over the
line. Split it by responsibility — the counting walks, the projection walk, and
the deletion walks are three distinct jobs — rather than by size.

## Deliverable 4 — registry bookkeeping

`DeleteMissing`'s registry row must **stay `UnguardedNative`**: the
finite-target/resume family is still natively recursive. Its `owed_by` currently
reads `"brief-463 DeleteMissing target/resume conversion"`; that number is now
this brief, so change it to name **brief-464**. Update inventory rows for any
edge whose shape you change.

## Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/associations/missing/traversal.rs` and new sibling
    modules under `crates/rum-stdlib/src/associations/missing/`;
  - `crates/rum-stdlib/src/associations/missing.rs` (module declarations and
    re-exports only);
  - `crates/rum-stdlib/src/tests/delete_missing_depth.rs` and its wiring;
  - `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs` and
    `recursion_inventory.rs` (row updates only);
  - `crates/rum-stdlib/Cargo.toml` or `crates/rum-runtime/Cargo.toml`, only for a
    test-only feature if the forced front needs one.
- Files you MUST NOT touch: the guard substrate, `kernel_api.rs`, the other four
  conversion sites, `delete_missing_at_target_level` and the `resume_*` family,
  `metadata.rs`, `Expr`, `SECURITY.md`, `TODO.md`, `PROGRESS.md`, other
  `docs/**`, `Cargo.lock`.
- **Accept-vs-stop, general so a file I failed to list still cannot admit a
  regression:** the only intended changes are an allocation-neutral rewrite of
  one function, a new test, a file split, and registry rows. Any change to what
  `DeleteMissing` removes, to result order, to diagnostics, or to the preflight's
  projected byte counts is a **stop**.

## rum / Rust pitfalls (read twice)

1. **Do not re-run the director's measurements above.** They are given.
2. **Counters beat timings and are the acceptance here.** A shape that "looks
   allocation-free" is not evidence; the number is.
3. Do not collapse a `suspend_with_frame*` or touch any frame type. If the split
   would require it, STOP and report.
4. The preflight's projected byte counts are in-band behavior. Assert them; do
   not eyeball them.
5. Wall-clock timing is not evidence on this host.

## Verification (MANDATORY — you run it, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-stdlib
cargo test --workspace
```

State whether your sandbox required `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`.

Then the three release certificates on their declared 2 MiB threads, and the new
forced-front falsifier.

Then the allocation gate: the three probes above, base against candidate, both
arms built at the same filesystem path, with sensitivity calibration and the
same-binary spread. Follow with `perf stat -e instructions:u`, minimum of the
tight clean cluster, migrated runs discarded.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- **No new `unsafe`.**
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **Do not run any probe that drives a recursion to or past its ceiling.**
- **Stop-loss.** Stop and report if: allocation counts do not return to base
  after two attempts; the forced-front falsifier cannot be made to exercise the
  tail; a release certificate regresses; or the split would require touching a
  suspension frame.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed, per file, with line ranges and a one-line why.
2. The allocation-neutral shape you chose, and the before/after counts for all
   three probes.
3. The forced-front falsifier: its fixture, and what it would catch.
4. The file sizes after the split.
5. Verification: every command with verbatim output, and the ZeroMQ statement.
6. Registry and inventory row changes.
7. Stop-loss and open questions.
