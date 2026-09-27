# brief-544 — lease the option snapshot, accumulator and consumer handoff: C01 to 0

Implementation brief. **Do not run `git` in any form.** Everything you need
about the tree's state is in this brief.

## Goal

Third of brief-541's four steps. The draft is §7 "Draft brief-544" of
`docs/dev/codex/brief-541-option-region-conversion-plan.md` (report from line
165) — **read it in full; its Deliverable, pitfalls, required falsifiers and
hot-path gate all apply**, with the director decisions below inserted.
`0a16d146` left C01 at 17 and C02 at 0.

Convert the default snapshot (B01), the merge loop (B02), the frozen and
resumed prefix, and the outgoing handoff to the consumer onto continuous
checked ownership. **Predicted end state: C01 0, C02 0, within the certified
domain**, `LeaseProbe538` clean, production promotion still 0 — recording
certificates is step 545. The four `UnresolvedConsumer` escaping copies stay
open and are not repaired by retaining the source collection.

## Director decisions inserted into the draft

### U1 — a runtime certified-registration domain (user ruling, 2026-09-27)

Coverage of C01 must hold for every execution admitted into it, so the
runtime itself enforces a closed certified domain; the lint does not merely
believe in one.

- **One declaration source** — for example a declaration in `rum-stdlib` —
  generates the installer's registrations **and** the domain the lint checks.
  Independently handwritten lists that agree by coincidence are not
  acceptable. A required test reconciles the declaration, the installer's
  actual registrations, the impl headers, feature configurations and
  certificate keys.
- At **entry and on resume**, the runtime checks the concrete `TypeId`
  belongs to the certified domain, in addition to plain registration. A frame
  restored after the domain changed must fail.
- **The limit you must state, not paper over.** The compile-time seal was not
  expressible because Rust visibility is crate-granular, and any entry point
  `rum-runtime` gives `rum-stdlib` to certify a type is reachable by an
  external crate too. Establish exactly what a host using only the public API
  can and cannot do: can it register an uncertified type (yes — and that type
  must be refused on the certified path)? Can it cause an uncertified type to
  be treated as certified? If some path lets a host mint certification, the
  guarantee must be stated as bounded to builds of this repository's
  declaration, with a falsifier showing the boundary — **or stop and report**.
  Do not claim more than the mechanism enforces.
- `rum-runtime` must not name `rum-stdlib` types (no crate cycle) and must not
  gain a builtin enum; builtins stay registered data.

### U3 — the retained consumer-handoff representation

Choose the concrete owner / view representation that keeps defaults, merged
prefix and the outgoing collection visible across the consumer's nested guest
work and a late arm, with no lease dropped before a raw `Vec` handoff and no
unguarded extraction. Any `OptionReentry` or consumer-signature ripple across
the 33 `OptionConsumer` variants is authorized; rum's API is deliberately
unlocked. If shared ownership costs a real allocation, measure and report it —
it is a data point under the standing ruling, not a stop.

### Erase the unarmed cost brief-543 introduced

`0a16d146` left a real unarmed cost on delayed-option paths: instructions
**+1.31%** (`option-explicit-delayed`), **+2.61%**
(`option-default-plus-delayed`), **+3.03%** (`option-prefix-n`), plus a
one-time 128-byte activation route. This brief reworks the same activation
bridge. Make unarmed accounting erase on these paths — the protocol's own
design says unarmed receipts are `()` — and report those three rows before and
after, against the pre-543 base as well as your own. If full erasure turns
out impossible, report the measured floor and why.

## Scope

**An objective and a fence, with a disclosure duty.** The objective, fence and
disclosure duty are the draft's (§7, "Scope"), with one addition: the
certified-domain declaration may live in `rum-stdlib`, and the plumbing that
lets the runtime check membership without naming stdlib types is inside the
objective. Every touched path appears in your report with its role.

## Verification (MANDATORY)

Director baseline: 43 checker tests green, C01 17, C02 0, `LeaseProbe538`
clean; workspace 2 654 passed, 0 failed.

```
python3 scripts/run-bounded.py --timeout 600  -- cargo fmt --check
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-runtime
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib options_suspension
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib retention_closure -- --nocapture
python3 scripts/run-bounded.py --timeout 1800 -- cargo test -p rum-stdlib --no-default-features retention_closure
python3 scripts/run-bounded.py --timeout 2400 -- cargo test --workspace
```

The draft's required falsifiers apply in full, plus: register an outside type
through the public API and show it cannot gain a certificate or run on the
certified path; alter each side of the declaration / runtime / lint
reconciliation independently and watch each fail.

**Hot-path measurement**, as in brief-543: allocation counts first; then
per-iteration instructions with `taskset -c 2 perf stat -e instructions:u`,
both binaries copied to the same path in turn, `cpu_core` row only, `rum-cli`
always given `< /dev/null`; timing secondary, and where timing disagrees with
instructions report both. Include the three delayed rows above, option-count,
late-arm and resume shapes, and every label brief-541 §4 lists.

Nine native ZeroMQ tests cannot bind loopback in your sandbox; report them
honestly as skipped-by-environment. Do not set
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP`.

## Checkpointing — protect the scene

All scratch and raw output under `.temp/brief-544/`. Update
`.temp/brief-544/PROGRESS.md` after the domain seam, the B01 snapshot, the B02
loop, the frame transfer and resume, the handoff, and the final gate.

## Hard rules

The draft's hard rules, in full: no `git`, no new `unsafe`, dependency or
nightly, no `Pending` record as a mark, no wildcard domain, no hidden raw
extraction, no callee traversal, no `rum-core` / interner / gensym / snapshot /
`S` change, no manifests or lockfile, no `AGENTS.md` / `PROGRESS.md` /
`TODO.md`. If the approved boundary cannot enforce the claimed domain or
ownership without crossing the fence, **stop and report** rather than silently
excluding the accumulator or widening an assumption. Stop-loss keyed on
progress: two consecutive attempts with neither a lower count nor a new
watched falsifier trigger a located report.

## Report format

The draft's, plus: exactly what the certified domain enforces against a host
using only the public API, and the three delayed-option rows before and after.
Append to this file and leave the tree uncommitted.
