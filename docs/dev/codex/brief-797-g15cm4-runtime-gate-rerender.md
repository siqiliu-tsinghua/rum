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

## Amendment 3 (gate c163) — the real promotion STOP is correct; the zero-overlay control carries the baseline promotion inventory and Stage B resumes

The Amendment 2 resume (unit `brief-797-cdx-20261125-112946-2653243`, fork of
`01a21099-0601-7271-8f24-952473a7c14a`, `gpt-6.2-astra` max, 11:29:46–11:53, ≈1,400 s dispatch
wall; Stage A2 704.873 s of 1,500; Stage B native 420.759 s, report closeout 691.644 s of 2,800;
rc 0, peak 2,847,981,568 B, oom 0; `report.py check 797 --stage amendment2` OK with 351
citations; 12 checks = 10 PASS + 1 FAIL + 1 NA; 3 deviations; 1 question `real-promotion-role`;
verdict STOP) is accepted through Stage B.2's real-role STOP. Verified and credited: the four 793
parent trees and 101 pins at entry and exit; the single 970-byte ordinary block (`dc859d7c…`)
exactly once in each arm; the cold owner's guard `_4` dropping in `bb2`/`bb4` after each branch
with cleanup in `bb8` and zero guard locals in the hot `resume`; the itemized 122-row projection
(97 diffs, 25 extensions, 26 shapes, 14 owners) identical on raw and canonical sources; 303
ordinary sites (280 external / 23 internal; 43 measured / 260 bounded); two byte-identical
source emissions (`SOURCE IDEMPOTENCE PASS roles=4 fresh_emissions=2 byte_identical=true`); the
514/54-of-568 frame split and census `405:58–405:68`; and F13 on armed, disabled and real (68
records, first 18 cells preserved, 68 fresh fingerprints, 62 falsifiers, F55 adoption, reread).
The first two reports stayed pinned (`0b4b3bfb…`, `dafd90f3…`); scope was git-clean with all
writes under `.temp/brief-797/`.

**The finding is correct.** Real's three changed additions alter the F13 closure (every `crates`
`.rs`/`.py`/`.toml` is hashed), so all 68 real fingerprints moved, and the promotion inventory's
direct-contract marks moved with them: `REAL PROMOTION INPUT PASS role=real bodies=2
feature_modes=2 direct_marks_per_mode=11 changed_real_F13_marks=11 carried_cells=4
common_armed_routing=false`. Re-earning the cells needs the `local_production_acceptance` review
in both configurations, and real's `--no-default-features` lib test cannot build: its manifest is
the preimage `6dcfaa9f…` and `sha2` is optional behind the default `hash` feature, so the build
exits 101 with `error[E0432]: unresolved import sha2` at
`crates/rum-stdlib/src/tests/ownership_graph/reconcile.rs:10:5` (zero tests; `tree == post_tree`
= `d0d956cf…`). Stage B.2's carry rule admits exact unchanged inputs or armed-authored
common-table routing; neither holds, so the STOP is the honest outcome.

**The ruling.** Real is the zero-overlay receipt control. Its role is defined by the 793 handoff
("Baseline and real keep the three Rust preimages and the manifest preimage `6dcfaa9f…`; real is
the zero-overlay receipt control"), whose `ARMS = ("armed", "disabled")` never ran the F13 or
promotion producers on real, and by 790's scene membership, where `promoted.tsv` is common at
`4337950c…` while `local-contracts.tsv` is role-specific (`79e44283…` at 793 — already its own
value). Real has never been a promotion producer, and the delivered 793 R4 accepted exactly that
state. Therefore:

1. Real's `promoted.tsv` is the baseline promotion inventory
   (`4337950c5b5e5f2a26f94d4aee66aacc0263a62b732d34b74e3021d1182dbbc7`), byte-identical to
   baseline's, carried as the control's role bytes. This is the third carry basis: **the
   non-producing control role**, whose generated tables are its declared preimage, exactly as
   baseline's F54 preimage role already is. The exception covers the promotion inventory only;
   real's F13 is a producer output and stays.
2. The replay must print `PROMOTION CARRY PASS role=real sha256=4337950c… baseline_sha256=4337950c…
   equal=true basis=zero-overlay-receipt-control`, with a one-byte-mutant falsifier proving the
   check can fail (`PROMOTION CARRY FALSIFIER PASS rejected_mutated=true`), and record the role
   routing `f13=armed,disabled,real promotions=armed,disabled real_promotion=baseline-carry`.
   The seal's real role pin and the R4 snapshot must record the real `promoted.tsv` as carried
   (input == final), and the report's deviations must carry this ruling. No claim that real
   certifies promotions may appear anywhere.
3. The repair option is rejected: the only on-record way to build real's no-default lib test is
   the 791 manifest postimage (`67be9610…`), which is overlay row 124 (preimage `6dcfaa9f…`).
   Installing it would give the zero-overlay control one overlay row and delete the only
   no-overlay shape of the source-auth gate, invalidating 792's pinned receipt specification
   (`9c7d156d…`), the six-case real leg and the 793 R4. The `--features sha2/compress` fallback
   stays unauthorized (793 §1). The promotion producer therefore runs on armed and disabled
   only, in both configurations, as briefed; real's unbuildable no-default lib test is a role
   property, not a defect to repair in this brief.

**Resume.** Continue in session `01a21099-0601-7271-8f24-952473a7c14a` from the accepted state
(the two byte-identical source emissions and the three F13 adoptions: armed `9dd31a24…`,
disabled `ce7f40ed…`, real `d0d956cf…`, each 5,851 files, debris 0) at the remaining Stage B.2
replay, then Stage B.3's ownership producers and seven-table reconciliation on the two arms, the
disabled diagnostic's two declared `cfg!` differences, B.4 (class discoveries, the 20,625-row
class table, negatives, LP01, the 875 bindings, the typed-slot source receipts), B.5 (chain and
additions), B.6 (the promotion reviews/checks on armed and disabled in both configurations with
adoption and recheck; zero rejections, two bodies, three production marks, four probe marks, the
two rendered anchors), B.7 (the 165-row overlay seal, 151 mutants, nine additions, four role
pins, `overlay_digest.py`, idempotence), then Stages C and D unchanged, including the 122-row
scene table and `PROBE CLOSURE facades=4 outside=3 attributed=3 unexplained=0`. The resumed
dispatch writes `report-amendment3.json` and passes `python3 scripts/report.py check 797 --stage
amendment3`; the earlier three reports stay pinned.

**Budget.** Charged: 838.952 + 549.586 + 704.873 + 691.644 = **2,785.055 s**. Amendment 3 tops up
Stage B **2,800 → 3,800 s** (+1,000 s, funded by an explicit 1,000 s top-up under the 2026-11-19
ruling): carve **19,900 s** = focused 13,900 (A 1,400 / A2 1,500 / B 3,800 / C 4,800; worker D
400; director R4 leg 2,000) + workspace 6,000 + epoch 0; remaining focused **11,114.945 s**. The
793 same-shape remainder (F13 start to seal end, 01:50:08–02:25:49) measured 2,141 s. Unspent
returns; a projected overrun stops for a ruling, never a scope cut.

## Amendment 4 (gate c164) — the Stage B.3 historical-identity STOP is correct; the class getter's historical closure is restored and every source-dependent proof replays

The Amendment 3 resume (unit `brief-797-cdx-20261125-120908-2676003`, fork of
`01a21099-0601-7271-8f24-952473a7c14a`, `gpt-6.2-astra` max, 12:09:08–12:24:48, 940.062 s
charged to report; rc 0, peak 1,245,569,024 B, oom 0; `report.py check 797 --stage amendment3`
OK with 572 citations; 18 checks = 16 PASS + 1 FAIL + 1 NA; 2 measurements PASS; 6 deviations;
1 question `class-lease-capture`; verdict STOP) is accepted through the Stage B.3 STOP.
Verified and credited: the four 793 parent trees and 101 pins at entry and exit (baseline
`fa01b3d1…`/5,842; real `d703b165…`→`d0d956cf…`; disabled `60ad65ed…`→`ce7f40ed…`; armed
`c04bbf82…`→`9dd31a24…`; each 5,851 files, debris 0; second emissions unchanged at `99f032a8…`,
`eb2fd226…`, `c7618f0e…`); the three pinned prior reports (`0b4b3bfb…`, `dafd90f3…`,
`cadeed35…`); the real promotion carry `4337950c…` equal to baseline with the one-byte
falsifier reproduced (offset 0 → `2b564378…`) and the routing line; the four exact ignored
ownership reviews green 4/0 on both arms after the one stale adapter witness position (141:1
recorded, 162:1 verified) was refreshed from native output through the successor 776 extractor,
with the historical input `33651b33…` retained and the successor input `62544b6b…` differing
by that single line; the seven whole tables per arm (armed 10,382 rows: 9,587 exact, 790
position by mapped line, 2 position by source owner and occurrence order, 3 change records;
`Local=2`; `new_UnknownHostPayload=0`; `unowned=0`; the disabled diagnostic's two declared
`cfg!` rows at `lib.rs:27:32` and `lib.rs:27:51`); and the two explained new `UnknownConstruct`
rows (the `Debug` derive at `lib.rs:38:5`, the `panic!` at `lib.rs:1443:9`).

**The finding is correct.** The parent's `class_lease.rs:42:33` is
`    super::ENABLED && FLAG.with(|flag| flag.get()) != 0`, a closure occurrence recorded in the
parent's `unknown_captures.tsv` line 8. The authored source supplies
`FLAG.with(super::PublishedFlag::get)`, a function item, so the row is gone: historical
occurrences 3, current 2 (the setter closures at 154:15 and 159:15 survive). The producer is
purely syntactic — `visit_expr_closure` records every closure from its `move`-ness and
parameter spelling — so the row's existence depends only on the source form, and Stage B.3's
bar fires correctly.

**The ruling.** The brief mandates the prototype's `PublishedFlag` lifecycle for the class
`FLAG`, not the getter's argument spelling. The lifecycle is untouched either way: `FLAG` stays
`super::PublishedFlag` with `off()`, `active()`, `get`, `set` and `Drop` preserving the six
balanced owner lifetimes, and the two setter closures keep their form. The parent's closure is
semantically identical (a zero-capture closure and a `fn(&PublishedFlag) -> bool` both inline to
the same read), so the historical form is restored: line 42 becomes
`    super::ENABLED && FLAG.with(|flag| flag.get())` (no `!= 0`; `get` returns `bool`),
preserving line 42, column 33 and the closure text. Nothing else in the five authored files
changes, and the source certificate records the one-expression declared delta from the
prototype, as it already records the three formatter reflows. Accepting the removal instead
would be an exception to the identity contract the 775/776 reconciliation exists to enforce,
bought for an incidental prototype spelling; it is rejected.

**Resume.** Continue in session `01a21099-0601-7271-8f24-952473a7c14a` from the accepted state
(the entry checkpoint `9dd31a24…`/`ce7f40ed…`/`d0d956cf…`, the carry verdicts, the refreshed
witness input and the reconciled seven tables) by re-authoring the class getter as above and
replaying everything that depends on the source bytes before Stage B.4: the two independent
byte-identical source emissions in all four roles (baseline stays `fa01b3d1…`); the source
certificate with the declared prototype delta, the 122-row classifier projection, the 303-site
surface, the MIR/guard verdicts (the adapter is untouched), the four 791 postimages, the
514/54-of-568 frame split and the census `405:58–405:68`; F13 on armed, disabled and real with
F55 adoption and the promotion carry re-attested verbatim; and the four ownership reviews on
both arms with the seven-table reconciliation required to show
`lost_historical_UnknownConstruct=0`, the same two disclosed additions,
`new_UnknownHostPayload=0`, `unowned=0`, `changed_records=794` (792 positions + 2 additions),
every row bound, both `Local` dispositions and the historical exclusions preserved, and the
disabled diagnostic's two declared `cfg!` rows. The witness input stays at the refreshed
`162:1` form; a repeat witness mismatch is a STOP, not another refresh. Then Stage B.4–B.7
unchanged (class discoveries, the 20,625-row class table, negatives, LP01, the 875 bindings,
the typed-slot receipts; chain and additions; the promotion reviews on armed and disabled in
both configurations; the 165-row overlay seal, 151 mutants, nine additions, four role pins,
`overlay_digest.py`, idempotence), then Stages C and D unchanged, including the 122-row scene
table and `PROBE CLOSURE facades=4 outside=3 attributed=3 unexplained=0`. The resumed dispatch
writes `report-amendment4.json` and passes `python3 scripts/report.py check 797 --stage
amendment4`; the four earlier reports stay pinned.

**Budget.** Charged: 838.952 + 549.586 + 704.873 + 691.644 + 940.062 = **3,725.117 s**. Stage B
has spent 1,631.706 s of 3,800 s. Amendment 4 tops up Stage B **3,800 → 5,000 s** (+1,200 s,
funded by an explicit top-up under the 2026-11-19 ruling; the rest-of-B.3 replay measures
≈1,200 s from this run's components and B.4–B.7 ≈1,740 s from 793's same-shape remainder less
this brief's measured F13+ownership share): carve **21,100 s** = focused 15,100 (A 1,400 /
A2 1,500 / B 5,000 / C 4,800; worker D 400; director R4 leg 2,000) + workspace 6,000 +
epoch 0; remaining focused **11,374.883 s**, of which Stage B has 3,368.294 s left. Unspent
returns; a projected overrun stops for a ruling, never a scope cut.

## Amendment 5 (gate c165) — the Stage B.4 STOP on the class-target destructor digests is correct; the class table is adopted with the certified source-driven destructor delta

The Amendment 4 resume (unit `brief-797-cdx-20261125-123600-2688027`, fork of
`01a21099-0601-7271-8f24-952473a7c14a`, `gpt-6.2-astra` max, 12:36:00–12:57, 1,309.842 s
charged to report; rc 0, peak 2,887,794,688 B, oom 0; `report.py check 797 --stage amendment4`
OK with 332 citations; 18 checks = 16 PASS + 1 FAIL + 1 NA; 1 measurement PASS + 1 NA; 4
deviations; 1 question `class-target-destructor-seals`; verdict STOP) is accepted through the
Stage B.4 STOP. Verified and credited: the c164 getter (`getter_delta` prototype `70d6bcc5…`,
c162 `24ef0748…`, c164 `e2f1ca42…`; `FLAG.with(|flag| flag.get())` at 42:33); two byte-identical
source emissions in all four roles (baseline `fa01b3d1…`/5,842, armed `ef48d0e2…`/5,851,
disabled `e1cf3b42…`/5,851, real `6ce4a8cd…`/5,851); the 122-row classifier projection
(116 unchanged, 3 refreshes, 1 rebind, 2 new; 97/25/26/14), the 303-site surface and
`PROBE CLOSURE facades=4 outside=3 attributed=3 unexplained=0`; unchanged MIR/guard verdicts and
exact postimages; F13 on armed (`4543670c…`), disabled (`05969f4e…`) and real (`626149bc…`)
with F55 adoption, reread and 62 falsifiers; the real promotion carry `4337950c…` re-attested
with its mutants; the ownership reviews green 4/0 on both arms with
`OWNERSHIP IDENTITY PASS roles=armed,disabled class_lease.rs:42:33=preserved
historical_exclusions=preserved Local=2 positions=792 additions=2 all_rows_bound=true`,
`lost_historical_UnknownConstruct=0`, `changed_records=794`, the restored `unknown_captures.tsv`
(3,949 rows) and the seven common tables installed in all four roles with only the two declared
disabled `cfg!` rows differing; and the two exact native class discoveries (29,907 twenty-field
records per arm, `raw_exit 0`, trees stable), the capture map (`stable_keys=29,754 added_keys=153
removed_keys=75 required_targets_added=0 required_targets_removed=0`) and both independent
20,625-row reviews at `d183c38e…` with 18,203 source rows unchanged and the eight declared
absences preserved.

**The finding is correct.** All 2,422 class targets' conservative destructor field and the
semantic digest that embeds it moved (`old_semantic_digests_matched=0/2422` on both arms), so
Stage B.4's "Preserve stable identities and semantic digest fields" bar fired and the table was
not installed.

**The ruling.** The `destructors` field is a conservative fingerprint over the whole crate's ADT
and `Drop` bodies (`resolve.rs`: "A conservative dependency fingerprint includes every ADT and
Drop body. It may over-invalidate; it cannot bless an unseen destructor change."), not a
per-target fact. The gate deliberately adds `PublishedFlag` with its balancing `Drop` and
replaces four `Cell<bool>` declarations with it; the witness (802 files both scenes; parent
`3615ec32…`/2,246 terms, candidate `552860c6…`/2,248 terms, 2,422/2,422 targets matched)
enumerates the entire delta — four removed terms (the old `q25_ownership::State { watch: bool }`,
`RunAuthority { current: Cell<bool> }`, `Installing { … bound: Cell<bool> }`, `Bound { …
emitted: Cell<bool> }`) and six added (the same four with `PublishedFlag`, plus `struct
PublishedFlag` and `impl Drop for PublishedFlag`), every term in the two certified files
(`lib.rs`, `q25_ownership.rs`). All other target fields are equal (`other_field_changes=0`), the
source rows are unchanged, the eight absences hold and no new absence appears, and both arms
agree byte-for-byte. So the regenerated table is adopted exactly as produced:
`d183c38eb2dfc47ed3d3332a16dec0dfdd77a22a06e9e30a2c26f10c6ea6505b`, installed as the common
armed-authored addition in armed, disabled and real (baseline keeps it absent per 793's
routing), then reread (20,625 rows, eight absences, unchanged source rows, the file digest).
The adoption check is rebuilt as a fail-closed attribution check — identical target keys,
tokens, tails and every other field, the same eight absences and zero new, unchanged source rows,
arms byte-equal, and only the destructor field and the row's semantic digest differing, only to
values reproduced by the native producers and certified by the per-term witness — and ships two
falsifiers: a mutated token is refused, and a destructor digest that does not match the certified
native value is refused. The ruling is recorded as a report deviation; both tables
(`626eb3d4…` historical, `d183c38e…` adopted) and the complete capture deltas (153/75 keys, 480
line, 8 order, 8 tail, per-source attributed) stay as evidence; nothing is hand-edited.

**Resume.** Continue in session `01a21099-0601-7271-8f24-952473a7c14a` from the accepted state
(the checkpoint trees armed `78d14631…`, disabled `329ac494…`, real `297f79a6…`, baseline
`65f37cbf…`; the two class reviews at `d183c38e…`; the destructor witness) by installing and
rereading the class table, rebuilding the adoption check with its two falsifiers, then Stage
B.4's remaining producers (class negatives, `logical_counter_port`, the eight LP01 producer
paths, the 875 bindings, the typed-slot source receipts with the recorded splits), Stage B.5
(chain and additions, with the class table's new SHA/length among the nine additions), B.6 (the
promotion reviews on armed and disabled in both configurations with adoption and recheck; zero
rejections, two bodies, three production marks, four probe marks, the two 793 rendered anchors),
B.7 (the 165-row overlay seal, 151 mutants, nine additions, four role pins, `overlay_digest.py`,
clean rerender idempotence), then Stages C and D unchanged, including the 122-row scene table,
`PROBE CLOSURE facades=4 outside=3 attributed=3 unexplained=0`, the 793 §6 source-auth gate, the
workspace, flag-on and fmt gates and the R4 handoff. The resumed dispatch writes
`report-amendment5.json` and passes `python3 scripts/report.py check 797 --stage amendment5`;
the five earlier reports stay pinned.

**Budget.** Charged: 3,725.117 + 1,309.842 = **5,034.959 s**. Stage B has spent 2,941.548 s of
5,000 s; remaining **2,058.452 s**; focused remaining **10,065.041 s**. No top-up now: the
remaining B.4 tail plus B.5–B.7 measures ≈1,300–1,500 s from this run's components (source, F13
and ownership ≈270 s; B.4 discovery and projection ≈680 s measured) and 793's same-shape
remainder. Unspent returns; a projected overrun stops for a ruling, never a scope cut.

## Amendment 6 (gate c166)

The Amendment 5 run (unit `brief-797-cdx-20261125-130657-2704631`, 13:06:57–15:27, 8,406.765 s
charged; `report-amendment5.json` `b0c84042…`) completed Stages B and C and correctly STOPped at
Stage D: the workspace returned 4,539/26/71 against the 4,555/10/71 bar, and the seven
unrecognized reds are six compiler-driver setup failures and the stdout hygiene test. Ruling 818
accepts the stop and amends the render and the driver.

**Repair 1 — the balance-test markers move to stderr.** The two marker emissions that
`renderer/test_packaging.py` appends to the carrier `crates/rum-runtime/src/lib.rs` change from
`println!` to `eprintln!`; the marker strings, the test names, the module, the order and the
whole child-isolation protocol stay byte-identical, and the expected-marker extraction reads the
transformed text. The 796 prototype under `.temp/brief-796/scratch/amendment1/balance.rs` stays
read-only: pin its SHA, carry the two-token transformation in the 797 renderer, and declare the
delta in the report. The choice is mandatory, not a preference: `eprintln!` is captured by
libtest through the same output capture and replayed on the child's stdout under
`--show-output` (verified on the pinned 1.85.0), so `publication_test_child`'s stdout assertions
do not change; the carrier `lib.rs` already writes its probe diagnostics with `eprintln!`
(lines 1015, 1016, 1290). Do not move the tests to a new `/tests/` file (a tenth armed-only
addition breaks B.5/B.7's nine-row contract, and a module prefix renames the two tests the Stage
D reconciliation binds) and do not touch the hygiene test. Rebuild the stdout witness: zero
unprefixed hits in all four roles' `lib.rs`, the red witness (`logs/a5-stdout-witness-1.log`
and the workspace raw log) retained as the proof that the check fires, and the workspace hygiene
test green.

**Repair 2 — the workspace driver's nested selector path.** Keep
`compiler_driver.py::proxy_environment` (PATH `/usr/bin:/bin` before the offline toolchain, the
brief-local `RUSTUP_HOME` with the symlinked installed toolchain, `RUSTUP_TOOLCHAIN=1.85.0`,
`RUSTUP_AUTO_INSTALL=0`, offline dist URLs) and `workspace.py::main`'s installation of it.
Re-run the rustup dispatch verification (`binding` and `version` for cargo and rustc) in the
same dispatch, and re-run the complete workspace under that environment: the six nested tests
(`compiler_oracle_agrees_with_resolution_claims`,
`census_constructor_resolutions_are_compiler_observed`, `inherent_method_claims_match_rustc`,
`exact_types_and_constructor_claims_match_rustc`, `local_probe_claims_match_stable_rustc`,
`local_option_read_claims_match_stable_rustc`) must be green by name, with fresh child receipts
showing `+1.85.0` still in argv and the six now passing. The reconcile must again require
exactly the two added passing names (the balance tests), no removed names, the ten accepted reds
(0 missing) and the nine inherited loopback names re-attested from the 793 host receipt.

**Replay.** The emitted bytes change for armed and disabled (two tokens in the carrier
`lib.rs`), and baseline and real must reproduce their Amendment 5 seals exactly — baseline
`31b1d674…` (5,842 files), real `9f2d0bfc…` (5,851 files) — as the blast-radius falsifier.
Every source-dependent proof replays: entry and exit trees and two byte-identical emissions per
role; the source certificate, the 122-row projection, the 303-site surface, the MIR/guard
witnesses and the four 791 postimages; F13 on all three roles with F55 adoption and the carry
with its one-byte mutant; the ownership reviews on both arms (`lost_historical_UnknownConstruct=0`,
`Local=2`, the exclusions, the seven common tables); the two native class discoveries and the
adoption check with its two falsifiers, fail-closed — the only new allowance is the two named
test rows' semantic digests if the discovery covers them (target keys, tokens, tails and all
other fields equal; arms byte-equal; any other row change is a STOP), with the fresh table SHA
pinned as addition row 7; B.5's chain and nine additions (the set stays nine — no new file);
B.6's promotions on armed and disabled in both configurations with adoption and recheck; B.7's
165-overlay seal, 151 mutants, nine additions, four role pins, `overlay_digest.py`, independent
rerender and idempotence; Stage C's instruction bars on the new tree, the 26-line/1,741-byte/
`a2c48c90…` ledger, the 122-row scene and its closure line, the six source-auth cases in both
configurations, F48 2/0, the fourteen selectors 14/0 and fmt; then Stage D's final flag-on
LP 3+1+1/0 and class 197/0, final fmt and the R4 handoff (92 snapshot rows / 90 files, the
190-path mapping).

**Report.** Continue in session `01a21099-0601-7271-8f24-952473a7c14a` from the Amendment 5
state; write `report-amendment6.json`, pin the six earlier reports (`0b4b3bfb…`, `dafd90f3…`,
`cadeed35…`, `1add9c95…`, `0692606f…`, `b0c84042…`), and pass
`python3 scripts/report.py check 797 --stage amendment6`. All writes stay under
`.temp/brief-797/`; no git, no tag, no network; one Cargo dev-dependency line at most; no new
`unsafe` in `rum-core`/`rum-runtime`.

**Budget.** Top-up **8,650 s**: Stage B 5,000→6,900, Stage C 4,800→9,900, D worker focused
400→550 and workspace 2,800→4,300. Carve **29,750 s** = focused 22,250 (A 1,400, A2 1,500,
B 6,900, C 9,900, worker D 550, director R4 2,000) + workspace 7,500 (worker D 4,300, director
3,200). Charged so far 13,441.724; remaining after the top-up: B 2,357.884, C 5,231.719,
D worker 389.635 + 2,322.449, director 5,200. Unspent returns; a projected overrun stops for a
ruling, never a scope cut.

## Amendment 7 (gate c167)

The Amendment 6 run (unit `brief-797-cdx-20261125-153842-2804321`, 15:38:42–16:08, 1,769.754 s
charged; `report-amendment6.json` `ce148d5e…`) completed Stage A and B.1 on the repaired render
and correctly STOPped at Stage B.2: the mandatory row-5 refresh of the shared overlay table makes
the c165 whole-tree control pins unsatisfiable, the real F13 run reused a stale cached test
executable and its output was refused and contained, and the projected batch already exceeded the
Stage B remainder. Ruling 819 accepts the stop and replaces the exact-pin bar with a file-level,
fail-closed one.

**Seal bar — file-level, fail-closed.** The seal's declared carrier refresh is unchanged (the four
rows `lib.rs`, `adapter.rs`, `local-contracts.tsv`, `promoted.tsv`), and row 5 must equal the
recomputed required row (`0b161f07…`; the old row is `6c535856…`; both serializers agree); any
other overlay row change is a STOP. Against the c165 seals (`31b1d674…`/5,842, `9f2d0bfc…`/5,851,
`b91b4d76…`/5,851, `df7ad0ef…`/5,851):

- the control source emissions are byte-identical to c165 (`fa01b3d1…`, `6ce4a8cd…`), and the only
  arm source delta is the two one-byte `e` insertions in `lib.rs`;
- `delta(role)`, the changed paths of the final release versus the c165 seal, is
  `{overlays.tsv, mutants.tsv}` for baseline;
  `{overlays.tsv, mutants.tsv, emission_chain.tsv, additions.tsv} ⊆ delta(real) ⊆
  {overlays.tsv, mutants.tsv, emission_chain.tsv, additions.tsv, class-provenance.tsv}`; and
  `delta(armed) == delta(disabled) ⊆ {lib.rs, local-contracts.tsv, promoted.tsv,
  emission_chain.tsv, additions.tsv, class-provenance.tsv, overlays.tsv, mutants.tsv}`;
  every changed path's bytes equal the independent reconstruction (`overlay_digest.py`), and no
  other path changes;
- `predicted(role) = tree(c165_role, overrides={p: new bytes})` over that role's `delta` must
  equal the final seal for all four roles; any mismatch is a STOP;
- fresh complete release pins are recorded, and every shared table carries the same SHA in every
  role that holds it;
- the class table's fresh SHA is addition row 7 under the c166 allowance: the two named test rows
  (`publication_balances_each_independent_owner`, `publication_preserves_control_scope`) may
  refresh their semantic digest only; every target key, every other field and every other row must
  equal `d183c38e…`; any other change is a STOP;
- the chain (152) and the additions (nine — the set stays nine, no new file) are rebuilt from the
  installed bytes before the seal, and the sealed tables' SHAs are pinned.

**Real F13 recovery.** Run the real F13 in a scene-rooted fresh target (the isolated plan's
`amendment6/target-real` or an equivalent under the amendment6 scene) with the candidate
`amendment6/images/real`; adoption requires the native provenance (the embedded manifest is the
candidate and every output lands in the candidate). The stale artifact stays archived and excluded;
the work-level target reuse is retired for F13 in all roles (cargo's source-hash fingerprint reuse
hides across candidate paths on the source-identical role); reauthenticate the four seals after the
run. The expected output is `626149bc…`, but equality must be observed. Adoption is F55 with the
control verdicts (68 records, 18 identity cells, 62 falsifiers).

**Order and replay.** Real F13 recovery → the two native class discoveries and the adoption check
with its two falsifiers (fail-closed per the class allowance) → B.4's standing producers (class
negatives, LP01, typed source, the retained mutation verdicts, 818.086 s measured — replay them;
the changed carriers and mutant table deny hash-identity reuse) → B.5's chain and nine additions →
B.6's promotions on both arms in both configurations with adoption and recheck → B.7's seal,
mutants, role pins, `overlay_digest.py`, independent rerender and idempotence → Stage C's bars, the
26-line/1,741-byte `a2c48c90…` ledger, the 122-row scene and its closure line, the source-auth gate,
F48 2/0, the fourteen selectors 14/0 and fmt → Stage D's workspace through the proxy driver (the six
compiler tests green by name, the hygiene test green, exactly the two added passing names,
4,555/10/71 after the authoritative loopback reconciliation), final flag-on LP 3+1+1/0 and class
197/0, final fmt and the R4 handoff. Every standing receipt carries its prior pin and falsifier; a
red in any of them stops.

**Report.** Continue in session `01a21099-0601-7271-8f24-952473a7c14a` from the
`checkpoint-B.2-stop` state; write `report-amendment7.json`, pin the seven earlier reports
(`0b4b3bfb…`, `dafd90f3…`, `cadeed35…`, `1add9c95…`, `0692606f…`, `b0c84042…`, `ce148d5e…`), and
pass `python3 scripts/report.py check 797 --stage amendment7`. All writes stay under
`.temp/brief-797/`; no git, no tag, no network; one Cargo dev-dependency line at most; no new
`unsafe` in `rum-core`/`rum-runtime`.

**Budget.** Top-up **3,600 s**: Stage B 6,900→10,500. Carve **33,350 s** = focused 25,850
(A 1,400, A2 1,500, B 10,500, C 9,900; worker D 550; director R4 2,000) + workspace 7,500 (worker
D 4,300, director 3,200). Charged so far 15,211.478; remaining after the top-up: B 4,188.130,
C 5,231.719, D worker 389.635 + 2,322.449, director 5,200. Unspent returns; a projected overrun
stops for a ruling, never a scope cut.

## Amendment 8 (gate c168)

The Amendment 7 run (unit `brief-797-cdx-20261125-162219-2825623`, 16:22:19–16:48, 1,499.405 s
charged; `report-amendment7.json` `6a5eadc2…`) recovered the real F13, reproduced the class
tables, and finished the ownership and B.4 producers, then correctly STOPped on the retained
promotion selector: its positive check consumes the B.6 promotion inventory, which is still the
793 parent's, and the shared additions manifest's chain row changes in every role. Ruling 820
accepts the stop, moves the retained block after the seal, and corrects the baseline delta.

**Order — the Amendment 5 proven order.** Entry from the `checkpoint-B.4-stop` state
(`checkpoint-B.4-stop.json` `5044a801…`). B.5's chain (152 members, one changed member, `lib.rs`)
and nine additions (row 4 = the chain, `ed54d95e…`; row 7 = `d183c38e…`; nine stays nine; one SHA
in every role that holds them; the two independent projections agree) → checkpoint B.5 → B.6's
promotions on both arms in both configurations (the four native runs per arm, the two F55
adoptions, the R3 rechecks and cell deltas, the two rejection censuses, the four stale
certificates archived) → checkpoint B.6 → B.7's seal, mutants, role pins, `overlay_digest.py`,
the independent rerender and idempotence → the retained block against the sealed images, which is
where the 818.086 s replay belongs: the seven selectors per arm, the no-default variants and the
typed-full (10/0/1, 8,172 outputs, 15 identical raw reports), each with its prior pin and
falsifiers, the mutation groups restored 15/15 and 6/6; their archive stays in the excluded
`.temp/brief-546/` → Stage C's bars, the 26-line/1,741-byte `a2c48c90…` ledger, the 122-row scene
and its closure line, the source-auth gate, F48 2/0, the fourteen selectors 14/0 and fmt → Stage
D's workspace through the proxy driver (the six compiler tests green by name, the hygiene test
green, exactly the two added passing names, 4,555/10/71 after the authoritative loopback
reconciliation), the final flag-on LP 3+1+1/0 and class 197/0, final fmt and the R4 handoff. The
seal precedes the retained block because the selectors consume the final overlay and mutant
tables; the retained writes never touch a role tree.

**Seal bar — corrected baseline clause.** The declared carrier refresh is unchanged (the four
rows `lib.rs`, `adapter.rs`, `local-contracts.tsv`, `promoted.tsv`; row 5 must equal the
recomputed `0b161f07…`; any other overlay row change is a STOP). Against the c165 seals
(`31b1d674…`/5,842, `9f2d0bfc…`/5,851, `b91b4d76…`/5,851, `df7ad0ef…`/5,851):

- `delta(baseline)` is exactly `{overlays.tsv, mutants.tsv, additions.tsv}` — the shared additions
  manifest follows the chain's row 4; the row4-only baseline counterfactual is `b6a27d01…`/5,842;
- `delta(real)` is exactly `{overlays.tsv, mutants.tsv, emission_chain.tsv, additions.tsv}`;
- `delta(armed) == delta(disabled)` stays within `{lib.rs, local-contracts.tsv, promoted.tsv,
  emission_chain.tsv, additions.tsv, class-provenance.tsv, overlays.tsv, mutants.tsv}` and must
  contain `{lib.rs, local-contracts.tsv, promoted.tsv, emission_chain.tsv, additions.tsv,
  overlays.tsv, mutants.tsv}` (the seven observed at the stop);
- the class table is byte-identical to `d183c38e…` (the c166 allowance is unused this round); any
  movement is the allowed two named test rows only, otherwise a STOP;
- every changed path's bytes equal the independent reconstruction (`overlay_digest.py`), no other
  path changes, and `predicted(role) = tree(c165_role, overrides={p: new bytes})` over that role's
  delta must equal the final seal for all four roles; a mismatch is a STOP;
- fresh complete release pins are recorded, and every shared table carries the same SHA in every
  role that holds it.

**Checkpoint discipline.** Checkpoints run only after all native producers end. The c168 first
ownership checkpoint refused on `candidate debris at images/armed: {'.temp'}` while the LP01
native job was live; the retry was green. Record both attempts and the reason.

**Report.** Continue in session `01a21099-0601-7271-8f24-952473a7c14a` from the
`checkpoint-B.4-stop` state; write `report-amendment8.json`, pin the eight earlier reports
(`0b4b3bfb…`, `dafd90f3…`, `cadeed35…`, `1add9c95…`, `0692606f…`, `b0c84042…`, `ce148d5e…`,
`6a5eadc2…`), and pass `python3 scripts/report.py check 797 --stage amendment8`. All writes stay
under `.temp/brief-797/`; no git, no tag, no network; one Cargo dev-dependency line at most; no
new `unsafe` in `rum-core`/`rum-runtime`.

**Budget.** Charged so far 16,710.883; Stage B remaining **2,688.725**; the remaining B plan is
B.5 ~1 + B.6 617 + B.7 58 + the retained block 818.086 + checkpoints ~20 = 1,514.1, a margin of
1,174.6; no top-up. The carve stays **33,350 s** = focused 25,850 (A 1,400, A2 1,500, B 10,500,
C 9,900; worker D 550; director R4 2,000) + workspace 7,500 (worker D 4,300, director 3,200).
Unspent returns; a projected overrun stops for a ruling, never a scope cut.

## Amendment 9 (gate c169)

The Amendment 8 run (unit `brief-797-cdx-20261125-165751-2846846`, 16:57:51–18:54, 6,913.691 s
charged; `report-amendment8.json` `50e54cca…`) completed Stage B and Stage C and stopped honestly
at the Stage D workspace bound: `cargo test --offline --workspace --no-fail-fast` under a 2,290 s
command limit expired at `typed_slot_compile_controls` after 58 of 70 result lines (4,317 passed,
19 failed, 71 ignored, raw_exit 124, the tree unchanged). Ruling 821 accepts the stop and
authorizes the complete rerun.

**The rerun.** `cargo test --offline --workspace --no-fail-fast` under a **3,400 s** bound, with
the exact bar: all 70 result lines; 4,555 passed / 10 failed / 71 ignored after the authoritative
loopback reconciliation; the 19 raw failures exactly the ten accepted plus the nine loopback reds
(the 793 receipt `c0b96f27…`; I re-parsed both logs, the name sets are identical); the six
compiler children green by name with their offline `+1.85.0` argv receipts; the stdout hygiene
test green; the two added passing names (the two publication balance tests, absent from both 793
logs) and zero removed names; then the workspace checkpoint of all four roles. A red outside that
set, a missing line, or a name difference is a STOP.

**The closing gates.** After the workspace checkpoint: the final flag-on LP 3+1+1/0, the final
flag-on class-lease 197/0 (the Stage C `class-flag 197/0/0` is credited; the closing re-run stays
only if the order requires it), the final fmt, and the R4 handoff package (the exact handoff
receipts the director's install leg will verify).

**Closeout.** Fix the two wrapper bugs so the closeout runs clean: the summary-check membership
must accept the filename suffix on the verdict line, and `checked8.py` must not subscript a
rebound `ROOT`. Write `report-amendment9.json`, pin the nine earlier reports (`0b4b3bfb…`,
`dafd90f3…`, `cadeed35…`, `1add9c95…`, `0692606f…`, `b0c84042…`, `ce148d5e…`, `6a5eadc2…`,
`50e54cca…`), and pass `python3 scripts/report.py check 797 --stage amendment9`. All writes stay
under `.temp/brief-797/`; no git, no tag, no network; one Cargo dev-dependency line at most; no new
`unsafe` in `rum-core`/`rum-runtime`.

**Budget.** Top-up **4,200 s**: worker workspace 2,322.449→5,922.449 (+3,600) and worker focused
389.635→989.635 (+600). Carve **33,350→37,550 s** = focused 26,450 (A 1,400, A2 1,500, B 10,500,
C 9,900; worker D 1,150; director R4 2,000) + workspace 11,100 (worker D 7,900, director 3,200).
Charged so far 23,624.574; remaining after the top-up: B 922.685, C 2,571.517, D workspace
3,632.317, D focused 792.318, director R4 5,200. Unspent returns; a projected overrun stops for a
ruling, never a scope cut.
