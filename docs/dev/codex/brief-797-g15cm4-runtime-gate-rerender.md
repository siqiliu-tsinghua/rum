# Brief 797 — G15-CM4 the runtime publication gate: author it into the render, re-emit the release, prove the bar, and deliver R4

**Implementation brief (write only under `.temp/brief-797/`; the live tree and the sealed roles
stay the director's).** One gate. It continues ruling 812 (gate c160) and adopts brief 796's
Stage D decision (`.temp/brief-796/draft/decision-and-open-questions.md`; the design record is
`.temp/brief-796/evidence/amendment1/gate-design.md`). The design is measured and closed: the
process-wide `PUBLISHED` gate executes 4 instructions per StringReplace site and 4+3 per typed
adapter visit, ordinary cold self Ir is zero, and the driven path still records with a
byte-identical 26-line / 1,741-byte ledger. This brief turns that design into the rendered
release roles and an authenticated R4 package. Brief 798 (the repaired whole-panel A/B) is held
for dispatch after this brief's R4 is authenticated.

## Goal

When this brief is done, in `.temp/brief-797/` only:

- **A re-runnable successor renderer.** The 768/769 emission chain re-rooted on the verified five
  carriers, with two clean byte-identical emissions and a complete source change manifest.
- **Four new sealed roles** (`images/{baseline,real,disabled,armed}`) with pinned trees, the
  regenerated 165-row overlay table, 9-row additions table, 152-member chain, F13 (68 records),
  the four postimages, the class table (2,422 / 20,625) and the 92-row / 90-file snapshot.
- **The proven bar on the final armed release.** Both guarded StringReplace sites and all reached
  typed adapters within the c159 limits on the fresh binary, with the falsifiers and the driven
  ledger.
- **The 794 structural attribution on the new scene**: the accepted 121-row projection
  (116 unchanged, 3 refresh, 1 rebind, 1 new residual; 96 diffs, 25 extensions, 26 shapes,
  14 owners) and `PROBE CLOSURE facades=4 outside=3 attributed=3 unexplained=0`.
- **A director R4 package**: the 190-path crates mapping, the live install receipt, the standing
  replays on the integrated checkout and the workspace/flag-on/fmt gate verdicts.

## Context pointers

- Ruling 812, ruling 811, brief 796 Amendment 2; `.temp/brief-796/draft/` (the adopted decision
  and both drafts), `evidence/amendment1/{surface.tsv, internal-call-sites.tsv, lifecycle.tsv,
  gate-design.md, prototype-inputs.json, cascade.md, attribution-dispositions.tsv}`,
  `evidence/amendment2/{direct-carriers.json, revalidated-costs.json, budget-basis.json}`.
- The executable source transforms: `.temp/brief-796/scratch/amendment1/{prototype.py,
  publication.rs, balance.rs}` and the five files under its `source/`. These are inputs; do not
  edit them.
- Brief 793 with its three amendments and ruling 808's R4 install; brief 790's emission plan;
  briefs 791/792/794/795/796's evidence.
- `AGENTS.md`, `docs/testing-strategy.md`, `docs/dev/codex/TEMPLATE.md`.

## Scope

The worker owns new renderer sources, the four successor role copies, build outputs, evidence and
logs under `.temp/brief-797/`. Separate directories for inputs, authored transforms, releases and
independent second emissions. The existing 768/769 generators, 790/791 evidence, 792 tools, 793
roles, 794 classifier and 795/796 evidence are immutable inputs. Carry the lockfile, public
language and embedding contracts unchanged. Safe Rust and existing dependencies only.

| Authoring target in the successor renderer | Sealed carrier (one-based data row) | Required change |
| --- | --- | --- |
| `crates/rum-runtime/src/lib.rs` | `overlays.tsv` row 5 | Author publication, the six independent ownership transitions, cold predicates, recorder entry guards and the balance tests. Change the helper template as well as the carried module. |
| `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` | `overlays.tsv` row 109 | Test publication before the original three pre-decode probe operations; re-read after decode at the original control observation; the ordinary body keeps one `#[inline(always)]` definition and the resumed-frame guard is owned by the cold gated path through both branches (the shape below). |
| `crates/rum-runtime/src/cm740_probe/class_lease.rs` | `additions.tsv` row 1 | Count the independent class flag, preserving its enabled check and inactive return values. |
| `crates/rum-runtime/src/cm740_probe/q25_ownership.rs` | `additions.tsv` row 3 | Count ACTIVE and the independent watch flag separately; preserve explicit capability consumption and teardown. |
| `crates/rum-runtime/src/cm740_probe/tests/cm743_route_a_falsifiers.rs` | `additions.tsv` row 9 | Construct the test authority through the same publication owner. |

Five source files are directly authored; `direct-carriers.json` carries their `old`/`prototype`
digests. The 793 handoff's five rendered production paths stay the R4 witnesses. All
source-dependent tables and receipts are generated outputs.

## The shape (a departure is a STOP)

```rust
static PUBLISHED: AtomicUsize = AtomicUsize::new(0);

#[inline(always)]
fn possibly_active() -> bool {
    PUBLISHED.load(Ordering::Relaxed) != 0
}

#[inline(always)]
fn run_is_driven() -> bool {
    ENABLED && possibly_active() && run_is_driven_slow()
}
```

`run_is_driven_slow` keeps the original thread-local predicate behind `#[cold]
#[inline(never)]`. Inline `driven`, the fast arms of `enter` and `around*`, and the inactive
returns of the independently reachable `control`, class, Q25 and recorder hooks. Preserve the
per-family return values, the invalid direct-hit precondition, all owned guard drops and the
disabled role's semantic `Some` behavior. Apply the 303-site inventory, including the four host
facades and the nested control callers.

**Must not change.** The original thread-local walk, its five lexical equalities, the actual run
authority, queue/episode and pair-stage conditions on the cold path; the order of
`control::resume`, `note_typed_consumer` and the driven decision at the typed adapter entry; the
source positions of the two guarded StringReplace expressions. Forbidden: duplicated bodies, a
flattened representation, silent-drop helpers, lazy initialization, inline attributes keyed on
`debug_assertions`, and any cached predicate value that crosses a callback boundary.

**Ownership lifetimes.** Use the prototype's `PublishedFlag` lifecycle for `RunAuthority.current`,
class `FLAG`, Q25 `ACTIVE`, Q25 watch, `Installing` and `Bound`. A `false→true` transition
increments the relaxed count before the owner becomes visible; a `true→false` revokes the owner
before decrementing. Repeated transitions are idempotent. RAII balances thread exit, parked
frames, replacement and scope restore. The calling thread's increment precedes its later load;
another thread's publication may add slow work but can never remove an outstanding unit.

## The typed adapter shape (a departure is a STOP)

The guard returned by `control::resume` restores the previous `RESUMED` frame when it drops. The
parent holds that guard to the end of `resume`, so the driven branch's
`control::active("Some")` and the ordinary body's post-decode observation both run with the
resumed frame in place. The gated shape keeps exactly that ownership while the undriven entry
carries no probe state: the guard is owned by the cold gated function, and the ordinary block
keeps one definition that both callers inline.

```rust
impl<F: TypedFrame> ErasedFrame for TypedAdapter<F> {
    fn resume(&self, session: &mut Session, input: &mut Result<Slot, EvalError>) -> Action {
        if crate::cm740_probe::possibly_active() {
            return Self::resume_with_probe_state(self, session, input);
        }
        Self::resume_ordinary_body(self, session, input)
    }
    // output_boundary, inner_frame unchanged.
}

impl<F: TypedFrame> TypedAdapter<F> {
    // The parent's 970-byte block, byte-identical (sha dc859d7c…): decode …, the sole token
    // mint, control::active("Some"), resume, option-slot print, action. One definition; inlined
    // at both call sites so the undriven entry carries no call and no probe state.
    #[inline(always)]
    fn resume_ordinary_body(&self, session: &mut Session, input: &mut Result<Slot, EvalError>) -> Action {
        // <the block, unchanged>
    }

    // The gated path owns the resumed-frame guard from the first observation to the end of both
    // branches, exactly as the parent's local did.
    #[cold]
    #[inline(never)]
    fn resume_with_probe_state(&self, session: &mut Session, input: &mut Result<Slot, EvalError>) -> Action {
        let _resumed = crate::cm740_probe::control::resume::<F, _>(self);
        crate::cm740_probe::class_lease::note_typed_consumer(
            self as *const Self as *const () as usize,
            std::any::type_name::<F>(),
        );
        if crate::cm740_probe::driven() {
            return Self::resume_cm740_driven_620(self, session, input);
        }
        Self::resume_ordinary_body(self, session, input)
    }

    #[cold]
    #[inline(never)]
    #[allow(dead_code)]
    fn resume_cm740_driven_620(&self, session: &mut Session, input: &mut Result<Slot, EvalError>) -> Action {
        // Unchanged from the parent, including the observe conjunction and the option-route scope.
    }
}
```

**Guard ownership (a departure is a STOP).** The `ResumeScope` returned by `control::resume`
must be a live local of `resume_with_probe_state` across both the driven call and the ordinary
body, and drop on every return path after the return value is computed. `resume`'s undriven
path must construct, hold and drop no probe guard and no probe state: no `Option<ResumeScope>`
local, no drop region, no call in the hot frame. The MIR witness records the guard's drop blocks
in `resume_with_probe_state`, the absence of any guard local in `resume`, and the parent's drop
sites for comparison. The owner-written control contract must now pass: the disabled role's
`InternalContinuationSome[5]` requirement and the armed route's `5` are restored.

**Single block.** No second copy of the ordinary block may exist: it has exactly one definition,
`resume_ordinary_body`, and the Stage C source witness records its single occurrence and its
sha. The undriven entry is the load, its branch and the inlined body with no call in between;
the open branch calls the cold function, which runs the three observations in the parent's
order and then either the driven call or the same body. `resume_cm740_driven_620` stays exactly
as the parent has it.

## Stage A — source authority and publication (1,400 s; Amendment 2 adds a 1,500 s resume allowance)

1. Authenticate the 793 handoff (`sealed-trees.tsv`): baseline `fa01b3d1…`/5,842, armed
   `c04bbf82…`/5,851, disabled `60ad65ed…`/5,851, real `d703b165…`/5,851, each debris-free.
   Authenticate the overlay seal `57e9acbd…`, the four 791 postimages, the 794 tools and the c159
   report; check each direct preimage against `direct-carriers.json`. A mismatch stops before
   rendering.
2. Root a successor of the 768/769 `chain_file`, `phase_certificate`, `phase_ordinary` and
   `phase_tables` in these authenticated role inputs; preserve the 763/784 repair lineage through
   its carried 793 bytes. The old 768/769 tool prefixes are `7e3edddd` and `4328588a`. Fork the
   794 `render_tools/fn_split.py` helper emitter near lines 2205–2220 for the inline `driven()`
   wrapper; authenticate its inverse passes `ablate.py`, `canonical.py` and
   `recorder_removal.py`. Apply the certified transformations to the carrier and the four other
   files. Verify original and successor spans independently of the serializer. Keep the 16-row
   historical ordinary allowance and the four Q25 inserted regions separately certified.
3. Author the shape above into the carried module and the typed adapter shape of the next section.
   Close the 303-site inventory in the successor sources, record the adapter's single occurrence
   of the 970-byte ordinary block with its sha (`dc859d7c…`) and its function, and re-derive the
   frozen 794 classification on the amended eight-path sources in paired scratch views before the
   render: write the amended disposition table (122 rows as itemized in Stage C) and record the
   classifier's own emitted labels verbatim. A row outside that itemization or a missing row is a
   STOP before the render.
4. Keep the two publication tests runnable in the default parallel workspace. The scratch balance
   cases require a zero global count; the control case requires `RUM_CM740_SEMANTIC=Some` and
   `RUM_CM740_OMIT=Some`. Run each exact body in a fresh test process: a test-only parent wrapper
   may use `current_exe()`, its exact selector, `--test-threads=1`, an environment sentinel for
   the child and those two scoped values for the control case; check the child's exit and named
   output. Keep the original two test names, normal registration and assertions. Disclose this
   test-only packaging delta from the scratch bytes and bind it in the source witness before
   regeneration. For the `Bound` case check a real successful bind and balanced release through
   FRAMES and RESUMED, including a nonzero intermediate publication and a final zero. Keep
   ordinary production code identical to the amended production shape. Defer acceptance
   tests until the standing replays in Stage B.

## Stage B — re-emission, standing inventories and seal (2,800 s)

Use the tool/input/output/STOP table in `.temp/brief-796/evidence/amendment1/cascade.md`. Each
step records complete input SHA-256s, actual argv, role, feature mode, exit and independently
regenerated bytes.

1. Render `lib.rs` from its 243,786-byte, 6,173-line armed parent (the disabled role is 243,761
   bytes; the 25-byte delta is the sole `ENABLED` declaration substitution). Apply the established three
   disabled-role substitutions, carry baseline preimages, place additions in armed/disabled/real,
   and keep all nine additions absent in baseline. Verify the four 791 postimages by exact byte
   lengths and hashes (`frame_root_trace/derived_inventory.rs`, `frame_root_trace.rs`,
   `retention_closure/census/tests.rs`, `rum-stdlib/Cargo.toml`). Rederive the frame/census
   witnesses where their producing inputs changed; recheck the 514/54 split of 568 named frames
   and the census location at `options/core.rs` line 405 on the rendered arms.
2. At the end of every covered edit, replay the standing producers **before the first acceptance
   suite**. Use successor copies of the 792 `run_tree.py`/`adopt.py` and 793 `standing.py`,
   `resume_ownership.py`, `promotions.py`, with each role's inputs bound. Real receives changed
   source additions: check its actual input closure for every applicable role-specific producer.
   A carry requires exact unchanged producer inputs or the established armed-authored
   common-table routing. Preserve baseline's F54 preimage role. Stop for a role ruling if
   required changed inputs cannot produce valid role bytes.
3. On both edited arms run the exact ignored
   `retention_closure::local::contract_tests::regenerate_local_contract_fingerprints` producer,
   F55-adopt with current mtime, reread, and run `local_concrete_contract_controls`. Require
   F13's 68 records, first 18 semantic cells and all 68 freshly generated fingerprints (old
   armed F13 starts `5d27e91b`). Regenerate the four ownership reviews per 793 §3, reconcile the
   seven whole armed-authored tables and report every changed row and position. Preserve existing
   identities, exclusions and `Local` dispositions. Disclose explained new `UnknownConstruct`
   rows; a changed historical identity, a new `UnknownHostPayload` or an unowned row stops. The
   disabled diagnostic keeps the two declared `cfg!` differences. Carry common ownership bytes
   per 793's role routing.
4. Capture the two approved role-local class discoveries with the exact ignored
   `obligations_discovery_campaign` selector; verify each 20-field record with 792 `closure.py
   classes()`. Use the 769 class source-row computation plus its 2,422 target rows to emit the
   complete 20,625 rows, binding the eight declared absences out of 2,430. Preserve stable
   identities and semantic digest fields. Run class negatives, `logical_counter_port`, the eight
   LP01 producer paths and 875 bindings, and the typed-slot source receipts with the recorded
   splits. Only role-authenticated source digests may fill changed cells.
5. Emit the sorted headerless 152-member chain from the successor per-member certificates with
   the 776 `derive_chain_table.py` algorithm and the 774 exception lineage; reread all 152 final
   armed members. Four directly changed chain members and addition row 9 (the Route A fixture).
   Generate `additions.tsv` from nine actual SHA/length pairs, including the class and chain
   products. Verify each role's bytes and required absences.
6. Run `local_production_acceptance` from the same role in default and `--no-default-features`
   builds; archive each complete review, its direct F13 evidence and probes. Merge the observed
   active cells per 792 `adopt.py`, check the inactive `UNRECORDED` cells and seven metadata
   columns, F55-stamp and reread. Require zero rejections, two bodies, three production marks,
   four probe marks and the two 793 rendered anchors. Rerun both selectors and require
   byte-identical second emissions. Replay the rejection census by identity and reason,
   preserving the 1,229 keyed rows and 14/1,177/38 split or stopping with the exact discrepancy.
7. Serialize 165 sorted overlays with parent preimage, both role digests and the exact
   disabled/armed hex payloads. Rebind every one of the 151 mutant `source_schema` cells to the
   emitted whole-table SHA. Root successor 792 `seal.py` and independent `overlay_digest.py` in
   the candidate and its fresh captures. Require independent full-byte equality, the nine-addition
   bijection, all role differences explained and four new complete role pins. Verify clean
   rerender idempotence. An old checksum, position, class capture or stale certificate is never
   manually transplanted; a red solely from a stale derived cell is an ordering artifact — save
   the red, replay the producer and repeat the same check. Quote each replay verdict verbatim.

## Stage C — focused proof, structural attribution and source authentication (4,800 s)

- Build the final armed `rum-cli` with release `codegen-units=1` at a stable source/target path.
  Reuse 796's two paired N=50/100 inputs for both guarded-hit sites and its positive
  Count/InputString resume workload. Count all executed address, load, compare and branch
  instructions. Require **≤4 per independent check, ≤3 with a reused address, ≤8 per guarded-hit
  iteration, ≤8 per typed adapter visit**; compile-time elimination costs zero. The prototype
  measured 4/4 and 4+3 in four reached typed bodies (350/700 entries); verify all 97 adapter
  entry sequences and record whether the post-decode check reuses the entry address (3) or
  reloads it (4) — both are within the bar; a count above 4 or a visit above 8 is a STOP.
  Require zero ordinary slow-predicate and lexical self Ir. Preserve the positive workload,
  decode order, suspended frame identity and ordered ledger.
- Exercise all four falsifiers. Require the real P0256 hit, all 12 Route A falsifiers and the same
  26 ordered lines / 1,741 bytes (SHA prefix `a2c48c90`). Run the mid-request Print callback and
  retain both reached StringReplace edges. In a disposable forced arm publish the independent Q25
  watch at `install_core` with identical workload and build path; require both sites to exceed
  the fast bar through the slow callee (the prototype added 111 inclusive Ir/visit). Restore the
  candidate digest. Run both balance tests, the 25 typed tests and the flag-on 197 class
  fixtures. Record the precise driven boundary, all exits and the ledger comparison.
- Re-run the 794 classifier and its same-name multiset and mutation controls on new armed versus
  new real. Require `ws=0`, all 21 hot matches, zero missing/unmatched, the four session facades
  and exactly three attributable `Local` witnesses. The direct projection is **122 rows**: 116
  unchanged, three typed control digest refreshes (`enter`, `drop`, `is_active`), and the typed
  adapter's three rows — `resume` (diff: the entry publication gate and the two dispatches;
  rebound span and detail), `resume_ordinary_body` (diff: the single post-decode
  `control::active("Some")` observation, same `typed resume observations` shape and owner as the
  baseline `resume` row), and `resume_with_probe_state` (diff: the original ordered observations,
  owning the resumed-frame guard through both branches; the baseline cold-helper name and
  description) — 97 diffs, 25 extensions, 26 shapes, 14 owners. The scene table must equal the
  Stage A projection; a row outside this itemization, a missing row or any unexplained difference
  is a STOP before acceptance. Rebind all postimage-dependent witnesses to the final scene. The
  required line is `PROBE CLOSURE facades=4 outside=3 attributed=3 unexplained=0`. Check
  `host_payload_ledger.rs/check_entry` and `reconcile.rs/is_local_construct` and
  `reconcile_falsifier_disposition` explicitly.
- Follow 793 §6 against the final role pins: the six historic red-to-green selectors, full
  typed-slot 10/0/1 per arm, the P0543 lane with nine changed manifests, two configurations ×
  three roles, six cold primary builds, two disabled restores, twelve source-auth receipts, six
  cases and the 23 prefix minima. Use the successor of 792 `gate/{lane_build, probe_build,
  receipts, run_f48_tests, run_14_tests}`. Require F48 2/0 plus chain, fourteen selectors 14/0,
  and `cargo fmt --all -- --check`. Retain exact negative reasons; stop if any mutant passes the
  wrong arm or receives the wrong rejection.

## Stage D — final-role workspace, manifest and R4

Worker leg 3,200 s (400 focused + 2,800 workspace); director R4 leg 5,200 s (2,000 focused +
3,200 workspace), separately accountable inside this carve.

1. Run `cargo test --offline --workspace --no-fail-fast` on final armed with the final baseline,
   fresh lane environment, flag-off settings and eight test threads. Compare the entire result
   set to 790 `stage-d-reds.tsv` and `ignored-set.tsv` by name. The 793 reference was 4,553
   passes, ten exact accepted reds, 71 ignores, 70 result lines, raw exit 101; with exactly the
   two additional passing balance tests the predicted count is 4,555. Identify both added names,
   all six green remnants and every discrepancy. For inherited sandbox-only loopback failures use
   793 Amendment 3's authoritative host replay with its exact names and raw receipts. Run flag-on
   LP 3+1+1/0 and class 197/0, then final fmt. Require complete output and zero OOM.
2. Re-root 793 `handoff.py` and `verify_r4.py`. Bind four input/output tree pairs, the actual
   path deltas, 92 snapshot rows, 90 physical files and the two baseline absences. Verify sealed
   byte counts, the five inherited rendered production witnesses and the four postimages.
   Deliver the 190-path R4 mapping (165 overlays, nine additions, `additions.tsv` and 15
   carried/regenerated/adopted paths).
3. The director preserves inherited work, checks the whole live crates delta, installs the exact
   release bytes, and repeats every source-covering standing replay on the integrated checkout
   before its first suite. Independently reconstruct the seal. Verify the live crates digest and
   count against sealed armed, then repeat the workspace/flag-on/fmt gate. Quote each live replay
   and test verdict. Pin post-suite bytes and deliver the R4 receipts to 798. Campaign re-freeze,
   fresh epoch and the 744 reentry follow their own authorizations after the repaired A/B gate.

## Verification and checkpointing

After each stage append proofs, raw receipt paths, complete input pins, the exact next bounded
command and the director handoff to `.temp/brief-797/PROGRESS.md`. A change to previously
certified source invalidates its dependent replays. Stop on an unexplained surface site,
unbalanced ownership, a missed drive, a changed ledger, an uncovered native route, a cost above
any bar, an unclassified structural or semantic delta, a missing producer, an independent
emission mismatch, live/role contamination, a projected overrun, or an attempt to alter a
generated inventory manually.

## rum / Rust pitfalls and hard rules

Keep the fast branch to a relaxed static load and its address/branch instructions. Keep
allocation, TLS, per-frame state and locks behind the existing cold ownership or lifetime
operations. Do not cache the loaded predicate across a callback. Use the unchanged Rust 1.85
toolchain/link profile, one cargo command at a time, eight jobs and eight test threads, and the
inherited 22 GiB dispatch scope with at least 20 GiB for 793 lanes. Bound every long or
child-spawning operation with `scripts/run-bounded.py` using measured same-shape timeouts and the
remaining stage allowance. Keep performance captures serial and CPU-idle. Offline builds, `.rum`
scratch sources, brief-local temporary paths. No version-control operations by the worker, no
network, no dependencies, no unsafe, no vendor comparisons, no generator/inventory hand edits.

Q25 stops remain P0214, P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411,
P2416 and P2418. Briefs 744/745/746 remain held. CM5 has no authorization.

## Budget

Focused **12,900 s** (A 1,400 — the 1,300 s carve plus the 88.538 s closeout overrun of the two
dispatches; A2 1,500 / B 2,800 / C 4,800; worker D 400; director R4 leg 2,000) + workspace
**6,000 s** (worker 2,800; director 3,200) + epoch 0 = **18,900 s**. Funded by the **1,231.45 s
returned by brief 796** plus explicit top-ups of **16,068.55 s** (the original carve) and
**1,600.00 s** (Amendment 2) under the standing 2026-11-19 ruling → reserve 0 at the carve. The
two dispatches charged 838.952 s and 549.586 s = **1,388.538 s** of Stage A. The measured basis
is in ruling 812 (6,652.395 s of same-shape inputs plus successor authoring, independent
emissions, repeated live work and attribution). Unspent returns; a projected overrun stops for a
director ruling, never a scope cut.

## Report format

Write `.temp/brief-797/report.json` in `rum-report/1`. Include every attempt, actual argv,
exits, test counts, the full carrier and replay deltas, all measured instructions and slopes,
scope, peak/OOM, historical accepted exclusions, and verbatim verdict lines. The director
supplies separate R4 receipt paths. Run `python3 scripts/report.py check 797` and return the
validated summary. A missing integration receipt leaves its gate open.

## Amendment 1 (gate c161) — the Stage A STOP is correct; the armed parent is 243,786 bytes and the adapter keeps one ordinary body

The first dispatch (unit `brief-797-cdx-20261125-104633-2634808`, `gpt-6.2-astra` max,
10:46:33–11:00:32, ≈840 s dispatch wall; worker Stage A 838.952 s of 1,000 s; rc 0, peak
305,225,728 B, oom 0; `check 797` OK 108 citations; 8 checks = 5 PASS + 2 FAIL + 1 NA; verdict
STOP) stopped at Stage A on two named questions. Both are correct and are accepted, together
with the delivered Stage A work: the four role pins matched at entry and exit; the five carrier
transforms reproduced the measured digests in memory; the helper emitter fork agrees with the
carried module; the 16-row allowance and the four Q25 regions re-authenticate; the test-only
packaging has its witness. The first report's two FAIL rows (`armed-parent-length`,
`typed-adapter-body`) are the accepted STOP, and this amendment resolves them.

**D1 — the armed parent is 243,786 bytes.** My brief carried 243,761 from brief 796's cascade
note, where it is the disabled role's length. The director's own check: the armed `lib.rs` is
243,786 B, sha `0c67b9de…` (the carrier's `old`), and the disabled file 243,761 B, sha
`6325e17a…`; the two differ on exactly one line, `pub const ENABLED: bool = !cfg!(disabled) &&
!cfg!(live);` (57 B) vs `pub const ENABLED: bool = false;` (32 B) — the 25-byte delta. Stage
B.1 now reads 243,786 bytes; every other row of the brief is unchanged.

**D2 — one ordinary body; the observations move to a cold helper.** The 796 prototype placed a
second byte-identical copy of the 970-byte decode-through-return block in
`resume_with_probe_state`; a duplicated body is forbidden here and is avoidable. Author the
"typed adapter shape" section above exactly: `resume` keeps the parent's single ordinary body
byte-identical (`dc859d7c…`) behind the gate; `run_cm740_probe_observations` is the
`#[cold] #[inline(never)]` helper holding `control::resume`, `note_typed_consumer` and `driven()`
in the parent's order and returning the driven decision; `resume_cm740_driven_620` stays exactly
as the parent has it. No `resume_ordinary`, no second copy of the ordinary block, no new inline
attribute on the body. The driven order is unchanged: observations → `driven()` →
`resume_cm740_driven_620`.

**D3 — fresh witnesses under this shape.**
- Stage C re-measures the final binary: the bars are unchanged (≤4 per independent check, ≤3
  with a reused address, ≤8 per guarded-hit iteration, ≤8 per typed adapter visit; the
  prototype's 4+3 is the reference). Record for each reached adapter whether the post-decode
  check reuses the entry address (3) or reloads it (4) — both pass; above 4 or above 8 is a STOP.
- The driven ledger stays 26 lines / 1,741 bytes / `a2c48c90…`, and the mid-request Print
  callback witness stays false, true, true.
- The 794 classifier re-derives on the emitted scene. The 121-row projection is the baseline;
  the expected delta is exactly: the residual row is named `run_cm740_probe_observations` (same
  `typed resume observations` shape, same owner, detail = the original ordered observations), the
  `resume` row keeps the `active("Some")` post-decode hook and gains the entry gate, and no row
  is added for a body copy. Any other row count, a differently named residual or an unexplained
  difference is a STOP before acceptance; `PROBE CLOSURE facades=4 outside=3 attributed=3
  unexplained=0` still decides.
- The source witness binds the three-method diff, the ordinary block's single occurrence and sha,
  the helper's hook order, and the already-witnessed test-packaging delta.

**D4 — resume and budget.** Resume in session `01a21099-0601-7271-8f24-952473a7c14a` at the
corrected Stage B.1 and the amended adapter shape, on the pinned 793 inputs; the Stage A receipts
stay as evidence. Stage A gains a **+300 s** top-up (carve 1,300 s; total **17,300 s** funded
1,231.45 returned + 16,068.55 top-up; B 2,800 / C 4,800 / worker D 400 / director R4 leg 2,000;
workspace 6,000; epoch 0). The first dispatch's ≈840 s is charged to the original Stage A carve.
The resumed dispatch writes `report-amendment1.json` (the first `report.json` stays untouched and
pinned) and passes `python3 scripts/report.py check 797 --stage amendment1`. Unspent returns;
a projected overrun stops for a ruling, never a scope cut.

## Amendment 2 (gate c162) — the guard-lifetime STOP is correct; the guard is owned by the cold gated path and the ordinary body keeps one inline definition

The Amendment 1 resume (unit `brief-797-cdx-20261125-110822-2643829`, fork of
`01a21099-0601-7271-8f24-952473a7c14a`, `gpt-6.2-astra` max, 11:08:22–11:17:32, ≈550 s dispatch
wall; rc 0, peak 2,055,196,672 B, oom 0; `report.py check 797 --stage amendment1` OK with 101
citations; 4 checks = 2 PASS + 1 FAIL + 1 NA; 4 deviations; 1 question; verdict STOP) stopped on
`resume-ownership` before any render, suite or inventory. The finding is correct and is accepted.

**The finding.** Amendment 1's helper returns only the driven decision, so its local
`Option<ResumeScope>` drops at the helper's return and restores the previous `RESUMED` frame
before either consumer. The recorded MIR line is `AMENDED MIR STOP guard=_2 driven=bb14
drop=bb13 return=bb1 caller_guard=absent postdecode_Some=after_drop driven_Some=after_drop`, and
the owner's control contract records `CONTROL CONTRACT STOP outermost_previous=None
restored_before=Some scoped_frame=lost disabled_required=InternalContinuationSome[5]
armed_required=5` against `PARENT SCOPE PASS owner=TypedAdapter::resume
both_branches=covered route=StatusFunctionProjectionFrame row=Some`. My own check agrees: the
parent's MIR keeps `_4: Option<ResumeScope>` and drops it on the return blocks after `driven()`
(bb paths 331/340/357), while the amended helper drops `_2` inside itself on both return paths.
The lost observation is the driven route's `observe` conjunct and the ordinary body's post-decode
`control::active("Some")`, including the disabled role's semantic `Some` behavior. The parent's
`ResumeScope::drop` restores `RESUMED` to `previous` and forgets the bound frame; nothing else
re-establishes it.

**The shape.** The guard must be returned to its owner, but the owner is not `resume`: holding an
`Option<ResumeScope>` in `resume` would put always-present probe state and a drop-tested guard
region in the undriven hot frame — off-by-default state and per-visit cost, which the hard line
forbids and which only LLVM's discretion could remove, not the structure. So the ordinary block
keeps one definition, `resume_ordinary_body`, marked `#[inline(always)]` and inlined at both call
sites, and the cold gated function `resume_with_probe_state` owns the guard from the first
observation through both branches. The undriven entry is then the load, its branch and the
inlined body with no call and no probe state; the open branch calls the cold function. The body
block is byte-identical (970 B, sha `dc859d7c…`) and the observation order
(`control::resume` → `note_typed_consumer` → `driven()` → driven call or body) is the parent's.
This supersedes Amendment 1's D2 helper shape, its `run_cm740_probe_observations` name and its
`no resume_ordinary` clause, and ruling 813's rejection of a shared ordinary body: with the guard
lifetime proved, the extraction is the structural necessity, and it is the shape the 796
prototype measured (4 at the entry and 3 at the post-decode check) with the duplication removed.

**Consequences bound into the amendment.**
- The MIR witness records the guard's drop blocks in `resume_with_probe_state`, no guard local in
  `resume`, and the parent's drop sites; the owner's control contract must pass both the disabled
  `InternalContinuationSome[5]` and the armed `5` requirements.
- The 794 classifier re-derives on the amended sources in Stage A and on the emitted scene in
  Stage C; the projection is **122 rows** (97 diffs, 25 extensions, 26 shapes, 14 owners): the
  116 unchanged, the three typed control digest refreshes, `resume` (entry gate, rebound span and
  detail), `resume_ordinary_body` (the single post-decode hook; the baseline `resume` row's shape
  and owner) and `resume_with_probe_state` (the original ordered observations; the baseline
  cold-helper name and description). A row outside the itemization, a missing row or any
  unexplained difference is a STOP; `PROBE CLOSURE facades=4 outside=3 attributed=3
  unexplained=0` still decides.
- Stage C re-measures the final binary under the unchanged bars (≤4 per independent check, ≤3
  with a reused address, ≤8 per guarded-hit iteration, ≤8 per typed adapter visit; the
  post-decode check may reuse the address (3) or reload it (4); above 4 or above 8 is a STOP).
  The undriven adapter entry must show no call to `resume_ordinary_body` and no guard local; the
  driven ledger stays 26 lines / 1,741 bytes / `a2c48c90…` and the mid-request Print callback
  stays false, true, true.

**Resume and budget.** Resume in session `01a21099-0601-7271-8f24-952473a7c14a` on the pinned 793
inputs; the first two reports stay pinned (`report.json` = `0b4b3bfb…`; `report-amendment1.json`
written) and the resumed dispatch writes `report-amendment2.json` and passes
`python3 scripts/report.py check 797 --stage amendment2`. Amendment 2 adds a **1,500 s** Stage A
allowance and absorbs the 88.538 s closeout overrun in Stage A's allocation: the carve is now
focused **12,900 s** (A 1,400 / A2 1,500 / B 2,800 / C 4,800; worker D 400; director R4 leg
2,000) + workspace **6,000 s** + epoch 0 = **18,900 s**, funded by 1,231.45 returned plus
**17,668.55 s** of explicit top-ups. The two dispatches charged 1,388.538 s. Unspent returns; a
projected overrun stops for a ruling, never a scope cut.

Co-Authored-By: Claude Code <noreply@anthropic.com>
