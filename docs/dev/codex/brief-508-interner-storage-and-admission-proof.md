# brief-508 — prove interner storage can be charged, and refused before it is published

## Goal

A safe, **scratch-only** proof of the two mechanisms the ruled quota rests
on, neither of which exists yet:

1. **Exact charging.** The interner's storage can be accounted in the ruled
   proxy units, including the backing-table bytes that an opaque `HashSet`/
   `HashMap` will not tell you.
2. **Refusal before publication.** An insertion that would exceed the budget
   is refused with **nothing leaked and nothing published** — the permit is
   consumed only by a step that cannot fail.

This is brief-505's Draft A, rewritten under the director's D1-D7 rulings and
brief-506's census. **It is not a production interner, not a performance
claim, and nothing lands.** The scratch package is evidence.

## Why this is unblocked

brief-507 found the `Module` repair feasible in representation but blocked on
a user ruling about display labels versus source-resolvable names
(`TODO.md` §The Module repair is feasible but BLOCKED ON A USER RULING). The
quota is required under **every** outcome of that ruling, so this proceeds
now. Its *sizing* waits; its *mechanism* does not.

## Settled, and not to be reopened

- **`Symbol` does not change**: size, `Copy`-ness, id-based fast-accept
  equality, and its lifetime-free `'static`-compatible public type are all
  frozen (user ruling 2026-09-21).
- **No reclamation, no per-session attribution.** The quota is process-wide.
- **D2: the charging model is a proxy, and only a proxy.** Exact Rust
  allocation-layout bytes plus a conservative fixed reserve, explicitly
  excluding allocator metadata, rounding and RSS. The portable-physical /
  RSS reading is rejected; do not reintroduce it under the word
  "conservative".
- **The direction of error is not free.** An undercharge is a fail-open. Where
  a byte count cannot be taken exactly in safe Rust, the bound must err
  **high**, and you must say which bound you used and why it is an upper one.
- **You do not choose the default.** D1 reserves that for the director. Report
  what the reserved vocabulary costs and what headroom a given cap leaves;
  do not select one.

## What the census already established — inputs, not things to re-derive

From brief-506 (verify anything you lean on, but do not re-run the suite):

- Bootstrap: **662 distinct strings, 661 pairs, ~77 004-85 033 proxy bytes**.
  661 of those pairs are `System\``, and 580 are installed non-canonical
  catalog names rather than `CANONICAL_CORE_HEADS`.
- Ordinary workspace suite, **maximum over test binaries**:
  **14 413 346-15 855 999 bytes**. 8 MiB is below this repository's own test
  suite; 32 MiB is the first envelope clearing bootstrap, suite and panel.
- **Lookup is itself an insertion ingress.** Resolving a fresh unqualified
  name constructs candidates per probed context and charges **two** pairs,
  sharing one text payload
  (`crates/rum-runtime/src/runtime/resolution.rs:101-138`). An admission
  model that assumes one pair per textual symbol is wrong, and this is the
  single most important thing for your seam to get right.
- Draft A's original 8 MiB subtraction is **not** calibration, and the
  present container's bytes-per-call average is **not** evidence about a
  replacement index's budget.

## Deliverables

Write each section of your report before starting the next.

### 1. The storage substrate, explicitly sized

Safe Rust, no `unsafe`, in the scratch package: an explicitly sized index
whose slot, control-byte and padding storage you can state exactly, standing
in for today's `HashSet<&'static str>` and `HashMap<(usize,usize), &'static SymbolText>`
(`crates/rum-core/src/expr.rs:41-44`). Report its layout per capacity class
and the growth transitions.

State the accounting unit and enumerate **every** allocation: the text
itself, the pair record, the index backing storage, any staging or
conversion allocation, the old/new table overlap during a growth, and the
fixed reserve. Use checked arithmetic. **No live-index growth before
admission.**

### 2. The staged insertion transaction

Reserve → stage → admit → publish, where **publish cannot fail**. Show, with
drop evidence, that a refused insertion frees everything it staged. Prove
the decision ordering through a private seam: **an existing-pair hit is
returned before any permit is requested**, so a full interner never refuses
a lookup of a name it already holds.

Model the two-pair lookup ingress explicitly: a single fresh unqualified name
arriving through resolution must be admissible or refusable **as a unit**, or
you must state what happens when the first pair is admitted and the second
refused. That partial state is the interesting case; do not leave it
undefined.

### 3. The independently checked ledger

The ledger's expected value must **not** be produced by the same projection
it is checking — a test that calls the projection for its own expected answer
certifies nothing about storage the projection forgets. Derive the expected
bytes a second way and say how.

Boundary cases, each with verbatim output: same string used as both context
and name; a new pair over existing strings; long text; every growth
transition; arithmetic overflow; id exhaustion; exact fit; one byte short.

### 4. The falsifier

Per this project's standing rule, a check nobody has watched fail is not
known to work. Ship checked-in proofs that: refusal fires at the cap;
refusal does **not** fire one byte below it; and enforcement removed from the
admission path is **detected** by the tests rather than passing silently.
Demonstrate that third one by actually disabling enforcement and showing the
red.

Note the trap brief-505 named: a process-global, never-reclaimed quota makes
tests order-dependent. Your seam must be injectable per test without
manufacturing real `Symbol`s in a second domain.

### 5. The reserved vocabulary, sized

Draft the finite reserved set — the names that must be interned before the
quota can bind so that a refusal can always construct its own message.
Enumerate its origins: canonical heads (`crates/rum-core/src/heads.rs:112`),
fixed runtime and stdlib catalog names, option defaults, error and
continuation tags, the `Global`/`Information` helpers. Measure its declared
layout. Report the headroom it leaves at 8, 16 and 32 MiB — as arithmetic for
the director, **not as a recommendation**.

## Scope

- You MAY create and write a standalone offline scratch Rust package and all
  evidence under `target/brief-508/**`, and append your report to this file.
- **MUST NOT** edit any file under `crates/`, `scripts/`, `docs/` other than
  this one, `Cargo.toml`, `Cargo.lock`, `AGENTS.md`, `TODO.md`,
  `PROGRESS.md`, `.git`, or the ownership-graph inventory fixtures.
- No new external dependency. Keep every build artifact inside the scratch
  subtree (set `CARGO_TARGET_DIR` and `TMPDIR` there).

## rum / Rust pitfalls (read twice)

1. `capacity * entry_size` is **not** evidence about an opaque hash table's
   backing bytes. If you use a real `HashMap` anywhere in the ledger's path,
   you cannot state its bytes exactly — that is why §1 asks for an explicit
   index.
2. `Box::new` does not promise recoverable OOM. Refusal must be a budget
   decision you make, never an allocation failure you catch.
3. A lifetime-free `Copy` handle cannot lease a local arena, and the scratch
   package must not export a real `rum-core` `Symbol` backed by its own id
   sequence.
4. A `&'static str` set is not a closed vocabulary.
5. Do not use `unsafe` to obtain a measurement. Bound it and state the
   direction instead.

## Verification (MANDATORY — you run it, paste it verbatim)

```
CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout <derived> -- \
  cargo test --offline --manifest-path target/brief-508/Cargo.toml
```

Derive the timeout from a recorded run of the same shape and say which; a
fresh small package builds in well under a minute on this host, so a 300 s
bound has ample headroom. Use the workspace-declared toolchain (1.85.0).

Do **not** run the production suite or the timing panel: production source is
unchanged, so there is nothing for them to say. Failure to bound an
allocation is an explicit failed proof, not permission to substitute an
estimate.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout` or
  `git restore`, or touch git config or the index.
- **No new `unsafe`**, anywhere, for any reason.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no comparisons against other vendors'
  engines.
- No network. Touch nothing outside this repo.
- **Stop-loss, keyed on progress.** If the same proof step fails twice and
  the second attempt produced no more information than the first, stop and
  report what you have. If the ledger undercharges, or the storage needs a
  forbidden representation, that is a **failed proof to report**, not a
  problem to engineer around.
- Paste real verbatim output, including failures. No cost claims from a toy
  prototype.

## Report format

1. The storage graph: every allocation, the layout per capacity class, and
   the independent charge calculation.
2. Transaction evidence: staging, drop-on-refusal, ordering, and the
   partial-admission answer for the two-pair ingress.
3. Boundary and falsifier evidence, verbatim, including the deliberately
   disabled-enforcement red run.
4. The reserved vocabulary, its origins and its measured layout, with
   headroom arithmetic at 8/16/32 MiB.
5. Proof verdict, what remains unproven, and the director decisions you hit.


---

## Execution report — brief-508 (2026-09-22)

### 1. Storage graph and accounting unit

This section was written before implementing the transaction. Evidence and
standalone dependency-free package: `target/brief-508/`. No production
source, manifest, lockfile, inventory, or Git state is changed. The original
brief is preserved verbatim as a prefix. The declared toolchain is 1.85.0.
The census is input only; its production suite and timing panel were not rerun.

The unit is one Rust allocation-layout byte, plus a declared 4096-byte fixed
reserve. Allocator bookkeeping, size-class rounding, RSS, executable code,
caller-owned input, test-harness/logging allocations, and compiler-selected
stack frames are outside this proxy. The reserve is an upper allowance for
the explicitly enumerated inline control objects, checked later by a size
sum; it is **not** a bound on those excluded physical costs. No default cap
is selected.

`src/storage.rs` uses `repr(C) Backing<T, N>`: `[Option<T>; N]` followed by
`[u8; N]`, inside one `Box`. Text slots hold `&str`; pair slots hold
`&Record`. Empty tables allocate nothing. The finite classes are 4 through
4096, doubling, with an explicit Capacity error beyond the last class. This
is a dense, linear-search insertion-only index, not a production hash-table
proposal. Controls record occupied slots; there are no tombstones or extra
probe groups. Each growth happens in a private replacement table.

Every owned allocation in the proposed transaction is enumerated here:

- One final `Box<str>` per distinct new text, constructed directly from the
  borrowed input (`Box::<str>::from`), including contexts. Length bytes,
  alignment 1; the empty string requires no payload allocation. No String,
  Vec-to-box conversion, or separate text header allocation is introduced.
- One `Box<Record>` per new pair: u32 ID and two static string references.
  The measured allocation is 40 bytes, alignment 8, including 4 padding
  bytes. Its box/header and observer stamp live inline in the fixed staging
  array. This is a private scratch record, never a real rum Symbol.
- Two replacement index backing boxes on each miss, including misses that
  do not grow a capacity. Existing entries are copied as references; old
  payloads/records are not duplicated. Thus the old **and** new text and
  pair tables coexist during staging. This deliberately simple proof pays
  that overlap even without growth; it makes no cost claim.
- Staging arrays (at most four new strings and two pairs), plans, reservation,
  permit, index-enum headers, and the borrowed stack-owned drop observer
  introduce no heap allocations. Their declared object sizes will be checked
  against the fixed reserve. There is no allocated observer, hidden map,
  closure environment, conversion buffer, or dynamically sized plan.
- Successful payloads and records will be leaked only during publication,
  modeling process lifetime. Old index boxes are freed after replacement;
  refused staged boxes remain owned and drop normally. No real public Symbol
  is constructed in a second ID domain.

For lengths L, pair count P, and capacities Cs/Cp on this measured 64-bit
host, retained charge is:

```text
4096 + sum(distinct allocated L) + 40*P
     + round_up_8(17*Cs) + round_up_8(9*Cp)
```

Zero-capacity terms are zero. For each miss, the reservation peak is current
retained charge plus every new text/pair allocation plus **both entire new
index boxes**. Final charge subtracts both old index boxes from that peak.
All projection additions/multiplications/alignment roundups are checked.
No opaque HashMap/HashSet capacity is used anywhere in this ledger.

The second calculation reads `Layout::for_value` on the actual boxed array
objects, plus actual payload/record objects, rather than invoking the
projection. The layout test also uses independent explicit 64-bit field
arithmetic and asserts the Record ABI. Table padding is counted, not guessed;
there is no uncertain dynamic allocation bound to round down.

The first same-shape offline `cargo test --manifest-path` run below used the
brief's explicit 300-second initial allowance. It recorded 0.582268 seconds
monotonically (`logs/baseline.json`). Subsequent bounds are 60 seconds:
`max(60, ceil(20 * 0.582268 + 30))`. Every Cargo run is delegated to the
unchanged `scripts/run-bounded.py`. `run.py` records exact argv/environment,
exit, monotonic elapsed time, and unmodified merged output. Environment:

```text
CARGO_NET_OFFLINE=true
RUSTUP_TOOLCHAIN=1.85.0
CARGO_TARGET_DIR=/home/remi/dev/rum/target/brief-508/build
TMPDIR=/home/remi/dev/rum/target/brief-508/tmp
```

Initial same-shape run, verbatim (`logs/baseline.log`, exit 0):

```text
   Compiling brief-508-proof v0.0.0 (/home/remi/dev/rum/target/brief-508)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.48s
     Running unittests src/lib.rs (target/brief-508/build/debug/deps/brief_508_proof-47dd105277137b08)

running 1 test
test storage::tests::storage_layout_classes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Layout evidence, verbatim (`logs/storage.log`, exit 0):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/lib.rs (target/brief-508/build/debug/deps/brief_508_proof-47dd105277137b08)

running 1 test
test storage::tests::storage_layout_classes ... Record bytes=40 align=8 fields=4+16+16 padding=4
capacity=4 text_slots=64 controls=4 padding=4 total=72 pair_slots=32 controls=4 padding=4 total=40
capacity=8 text_slots=128 controls=8 padding=0 total=136 pair_slots=64 controls=8 padding=0 total=72
capacity=16 text_slots=256 controls=16 padding=0 total=272 pair_slots=128 controls=16 padding=0 total=144
capacity=32 text_slots=512 controls=32 padding=0 total=544 pair_slots=256 controls=32 padding=0 total=288
capacity=64 text_slots=1024 controls=64 padding=0 total=1088 pair_slots=512 controls=64 padding=0 total=576
capacity=128 text_slots=2048 controls=128 padding=0 total=2176 pair_slots=1024 controls=128 padding=0 total=1152
capacity=256 text_slots=4096 controls=256 padding=0 total=4352 pair_slots=2048 controls=256 padding=0 total=2304
capacity=512 text_slots=8192 controls=512 padding=0 total=8704 pair_slots=4096 controls=512 padding=0 total=4608
capacity=1024 text_slots=16384 controls=1024 padding=0 total=17408 pair_slots=8192 controls=1024 padding=0 total=9216
capacity=2048 text_slots=32768 controls=2048 padding=0 total=34816 pair_slots=16384 controls=2048 padding=0 total=18432
capacity=4096 text_slots=65536 controls=4096 padding=0 total=69632 pair_slots=32768 controls=4096 padding=0 total=36864
growth=0->4->8->16->32->64->128->256->512->1024->2048->4096; above 4096=Capacity refusal
index allocation/drop counts=([0, 0, 22], [0, 0, 22], [0, 0, 0])
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

### 2. Transaction evidence

This section was written before starting the boundary-ledger tests.
`planning.rs`, `admission.rs`, and `transaction.rs` implement the private
seam. The standalone crate has no rum dependency and exports no Symbol.
Each test injects its own gate/observer; only private u32 markers identify
its scratch records. Accepted text/records are intentionally leaked for
process lifetime; this is not reclamation or a local-arena public handle.

The sequence is:

1. Look up existing pairs, deduplicate up to two missing pairs/four texts,
   check IDs and arithmetic, and calculate capacities without allocating.
   An all-hit request returns here, before `Gate::reserve`.
2. Reserve the entire projected peak under exclusive `&mut Interner` access.
   Budget refusal here allocates nothing. The cap includes old/new index
   overlap, not merely the retained increment. There is no live-index growth.
3. Stage the final text/record boxes and both replacement tables. Copy
   existing references and validate capacity/control invariants. The live
   tables, charge, and ID cursor are still unchanged. Box OOM may abort;
   there is no claim of recoverable allocator OOM.
4. Independently walk actual allocations, compare both peak and final charge
   against the reservation, and recheck admission. A private injected veto
   at this point exercises the actual caller's `?`/RAII cleanup path.
5. Consume the private, non-Copy permit in publication: leak the owned
   text/records, fill already-sized tables, replace table owners, and assign
   precomputed scalars. No allocation, checked conversion, capacity choice,
   user callback, fallible return, or data-dependent indexing occurs here.
   Text selectors are a closed four-variant enum; array destructuring is
   total. Table iteration is bounded by capacities proved before admission.
   Old box destructors only deallocate and update non-panicking observers.

This is an exclusive-access proof, not a concurrency integration proof. A
production process lock must enclose this sequence. The fixed control-state
size sum deliberately counts two plans/stages and a second Gate: 1528 bytes
against the 4096-byte allowance, leaving 2568 bytes. It does not purport to
measure compiler stack temporaries or allocator bookkeeping.

The two-context ingress is **atomic**: `resolve_fresh` submits both
`(System`, name)` and `(Global`, name)` in one plan/reservation/permit.
At 4306 bytes it refuses the complete 4307-byte transaction, before either
pair or the shared name is allocated; at exact fit both publish. A veto
after staging frees all seven staged boxes: three texts, two records, two
indexes. ID cursor, tables, retained charge, permit count and publication
count stay unchanged. Existing pairs never need a new permit even when the
cap is full, IDs are exhausted, and the injected admission veto is armed.
This models the verified two-pair ingress in resolution.rs:101-138; wiring
that grouping through production's arbitrary context path remains future
integration work.

Verbatim transaction evidence (`logs/transaction.log`, exit 0):

```text
   Compiling brief-508-proof v0.0.0 (/home/remi/dev/rum/target/brief-508)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.51s
     Running unittests src/lib.rs (target/brief-508/build/debug/deps/brief_508_proof-47dd105277137b08)

running 4 tests
test transaction_tests::transaction_drop_after_stage ... staged veto: made/freed/leaked=([3, 2, 2], [3, 2, 2], [0, 0, 0]); charge=4096 next_id=0 tables=0/0 permits=0 publications=0
retry: charge=4307 pairs=2 distinct_texts=3 permits=1 publications=1
ok
test transaction_tests::transaction_fixed_reserve_covers_declared_inline_state ... inline-size terms=[136, 72, 464, 704, 48, 16, 8, 64, 16] sum=1528 fixed-reserve=4096 upper slack=2568
ok
test transaction_tests::transaction_hit_before_permit_at_full_cap_and_id_limit ... full cap=4253 and exhausted IDs: existing id=0 returned; new permit requests=0 new allocations=0
ok
test transaction_tests::transaction_two_pairs_are_atomic ... two-pair one-byte-short: cap=4306 peak=4307; neither pair nor shared text allocated/published
two-pair exact-fit: cap=4307 charge=4307; ids=0,1 shared name payloads=1; made/freed/leaked=([3, 2, 2], [0, 0, 0], [3, 2, 0])
ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

```

### 3. Boundary and falsifier evidence

#### 3a. Independently checked ledger (completed before the falsifier)

`boundary_tests.rs` derives expected answers from independent, explicit
64-bit field arithmetic: a separate list of capacity thresholds, 17/9-byte
slot-plus-control strides rounded to 8, literal 40-byte record size, and
known input byte lengths/counts. It does **not** call `projected_table`,
`capacity_for`, or `plan` to produce expected byte counts. It compares those
answers with both the planner and a walk of actual owned/leaked objects
using `Layout::for_value`. The full transaction also compares projected
peak/final values with that graph walk before producing its permit.

Every index class is reached, including separate string and pair growth.
At each transition the independently calculated peak minus one refuses
without allocating or changing live table addresses/IDs; the exact peak
then succeeds. Old and new tables are both included. This verifies the
retained-vs-peak distinction rather than subtracting an opaque container
estimate. No ledger undercharge or unmatched allocation was observed.

Additional boundaries cover shared context/name payload, a new pair using
existing strings, a 65536-byte text, empty zero-layout text, duplicate pairs,
checked sum/product/layout overflow, last valid u32 ID, a two-pair batch
with only one ID left, an existing/new candidate mixture, and drop of staged
4-to-8 growth while the old tables remain live. The zero-layout observer
counts only nonzero Box allocations, so the empty text contributes zero.

Verbatim boundary run (`logs/boundaries.log`, exit 0):

```text
   Compiling brief-508-proof v0.0.0 (/home/remi/dev/rum/target/brief-508)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.58s
     Running unittests src/lib.rs (target/brief-508/build/debug/deps/brief_508_proof-47dd105277137b08)

running 6 tests
test boundary_tests::boundary_arithmetic_overflow_and_id_exhaustion ... overflow: add, multiply, both table layouts, transaction sum -> Overflow; allocations=0 permit_requests=0
IDs: last=4294967295 admitted, next=4294967296 refused, existing last ID still hits; no rejected allocation
two-pair ID exhaustion: one available ID cannot admit two; neither pair published, cursor unchanged
ok
test boundary_tests::boundary_duplicate_pair_is_one_admission ... duplicate pair in batch: one text, one record, one permit; both results id=0 retained=4249
ok
test boundary_tests::boundary_every_growth_transition_exact_fit_and_short ... growth pairs=1 text=0->4 pair=0->4 old_tables=0 new_tables=112 peak=4253 retained=4253; peak-1 refused, exact peak admitted
growth pairs=4 text=4->8 pair=4->4 old_tables=112 new_tables=176 peak=4561 retained=4449; peak-1 refused, exact peak admitted
growth pairs=5 text=8->8 pair=4->8 old_tables=176 new_tables=208 peak=4701 retained=4525; peak-1 refused, exact peak admitted
growth pairs=8 text=8->16 pair=8->8 old_tables=208 new_tables=344 peak=5001 retained=4793; peak-1 refused, exact peak admitted
growth pairs=9 text=16->16 pair=8->16 old_tables=344 new_tables=416 peak=5253 retained=4909; peak-1 refused, exact peak admitted
growth pairs=16 text=16->32 pair=16->16 old_tables=416 new_tables=688 peak=5905 retained=5489; peak-1 refused, exact peak admitted
growth pairs=17 text=32->32 pair=16->32 old_tables=688 new_tables=832 peak=6365 retained=5677; peak-1 refused, exact peak admitted
growth pairs=32 text=32->64 pair=32->32 old_tables=832 new_tables=1376 peak=7713 retained=6881; peak-1 refused, exact peak admitted
growth pairs=33 text=64->64 pair=32->64 old_tables=1376 new_tables=1664 peak=8589 retained=7213; peak-1 refused, exact peak admitted
growth pairs=64 text=64->128 pair=64->64 old_tables=1664 new_tables=2752 peak=11329 retained=9665; peak-1 refused, exact peak admitted
growth pairs=65 text=128->128 pair=64->128 old_tables=2752 new_tables=3328 peak=13037 retained=10285; peak-1 refused, exact peak admitted
growth pairs=128 text=128->256 pair=128->128 old_tables=3328 new_tables=5504 peak=18561 retained=15233; peak-1 refused, exact peak admitted
growth pairs=129 text=256->256 pair=128->256 old_tables=5504 new_tables=6656 peak=21933 retained=16429; peak-1 refused, exact peak admitted
growth pairs=256 text=256->512 pair=256->256 old_tables=6656 new_tables=11008 peak=33025 retained=26369; peak-1 refused, exact peak admitted
growth pairs=257 text=512->512 pair=256->512 old_tables=11008 new_tables=13312 peak=39725 retained=28717; peak-1 refused, exact peak admitted
growth pairs=512 text=512->1024 pair=512->512 old_tables=13312 new_tables=22016 peak=61953 retained=48641; peak-1 refused, exact peak admitted
growth pairs=513 text=1024->1024 pair=512->1024 old_tables=22016 new_tables=26624 peak=75309 retained=53293; peak-1 refused, exact peak admitted
growth pairs=1024 text=1024->2048 pair=1024->1024 old_tables=26624 new_tables=44032 peak=119809 retained=93185; peak-1 refused, exact peak admitted
growth pairs=1025 text=2048->2048 pair=1024->2048 old_tables=44032 new_tables=53248 peak=146477 retained=102445; peak-1 refused, exact peak admitted
growth pairs=2048 text=2048->4096 pair=2048->2048 old_tables=53248 new_tables=88064 peak=235521 retained=182273; peak-1 refused, exact peak admitted
growth pairs=2049 text=4096->4096 pair=2048->4096 old_tables=88064 new_tables=106496 peak=288813 retained=200749; peak-1 refused, exact peak admitted
all 11 capacity classes exercised for both indexes; capacity 4097 refused before allocation
ok
test boundary_tests::boundary_growth_drop_and_mixed_existing_pair ... 4->8 staged growth veto: made/freed delta=[1,1,2], leaked delta=[0,0,0]; live table addresses/charge/IDs unchanged
mixed hit/miss at full cap: preexisting System pair retained; no Global pair or additional text published
ok
test boundary_tests::boundary_long_and_empty_text ... long text: bytes=65536 context_bytes=1 records=40 indexes=112 retained=69785
empty context/name: one zero-layout text, payload allocations=0 retained=4248
ok
test boundary_tests::boundary_same_string_and_new_pair_existing_strings ... same text as context/name: texts=1 pairs=1 text_bytes=4 charge=4252
new pair over existing strings: new_text=0 new_records=1 old/new tables=112+112 peak=4402 retained=4290
ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.76s

```

#### 3b. The falsifier (completed before reserved-vocabulary work)

Source proofs are shipped in the scratch package's `falsifier_tests.rs`;
no Git index/commit operation is permitted by this brief. The normal tests
require refusal when used equals cap and new storage is requested, permit a
one-byte request when used is cap minus one, and permit a whole factory
transaction whose required peak is one byte below cap. Exact fit is allowed;
"full cap refuses" never means rejecting an existing-pair hit.

The `disable-enforcement` feature **actually removes both cap comparisons**
from `Gate::check_cap`, used by reserve and admit. Tests and their expected
answers do not inspect that feature. This is a bounded mutation: at most a
few tiny allocations, with no unbounded loop or OOM reliance. Running the
same four source tests with the feature produced exit 101: both negative
controls admitted forbidden storage and failed; both positive controls
continued to pass. Turning the feature off returned all four to green.
This expected mutation is not an accidental ledger undercharge; the ledger
still accounts exactly for the deliberately over-cap publication.

Commands, with the previously recorded scratch environment and 60-second
process-group bound, were:

```sh
python3 target/brief-508/run.py falsifier-green 60 falsifier_ -- --nocapture --test-threads=1
python3 target/brief-508/run.py falsifier-red 60 --features disable-enforcement falsifier_ -- --nocapture --test-threads=1
python3 target/brief-508/run.py falsifier-restored 60 falsifier_ -- --nocapture --test-threads=1
```

Verbatim `logs/falsifier-green.log` (exit 0):

```text
   Compiling brief-508-proof v0.0.0 (/home/remi/dev/rum/target/brief-508)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.42s
     Running unittests src/lib.rs (target/brief-508/build/debug/deps/brief_508_proof-47dd105277137b08)

running 4 tests
test falsifier_tests::falsifier_at_cap_refuses_new_pair ... full-cap control: used=cap=4096, fresh peak=4250 refused three times; no allocation/publication
ok
test falsifier_tests::falsifier_one_byte_below_cap_admits_one_byte ... below-cap positive control: used=4096 cap=4097 +1 byte admitted, used=4097
ok
test falsifier_tests::falsifier_one_byte_short_refuses_entire_two_pair_ingress ... two-pair negative control: cap=4306 required=4307 refused as a unit
ok
test falsifier_tests::falsifier_required_peak_one_below_cap_is_accepted ... whole-factory positive control: required=4307 cap=4308 admitted, one byte remains
ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

```

Verbatim `logs/falsifier-red.log` (exit 101):

```text
   Compiling brief-508-proof v0.0.0 (/home/remi/dev/rum/target/brief-508)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.62s
     Running unittests src/lib.rs (target/brief-508/build/debug/deps/brief_508_proof-4580141b4ce374dc)

running 4 tests
test falsifier_tests::falsifier_at_cap_refuses_new_pair ... 
thread 'falsifier_tests::falsifier_at_cap_refuses_new_pair' panicked at src/falsifier_tests.rs:11:9:
assertion `left == right` failed: enforcement absent: a full interner admitted fresh storage
  left: Ok(0)
 right: Err(Budget { peak: 4250, cap: 4096 })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test falsifier_tests::falsifier_one_byte_below_cap_admits_one_byte ... below-cap positive control: used=4096 cap=4097 +1 byte admitted, used=4097
ok
test falsifier_tests::falsifier_one_byte_short_refuses_entire_two_pair_ingress ... 
thread 'falsifier_tests::falsifier_one_byte_short_refuses_entire_two_pair_ingress' panicked at src/falsifier_tests.rs:36:5:
assertion `left == right` failed: enforcement absent: two-pair ingress crossed its cap
  left: Ok([Some(0), Some(1)])
 right: Err(Budget { peak: 4307, cap: 4306 })
FAILED
test falsifier_tests::falsifier_required_peak_one_below_cap_is_accepted ... whole-factory positive control: required=4307 cap=4308 admitted, one byte remains
ok

failures:

failures:
    falsifier_tests::falsifier_at_cap_refuses_new_pair
    falsifier_tests::falsifier_one_byte_short_refuses_entire_two_pair_ingress

test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib`
```

Verbatim `logs/falsifier-restored.log` (exit 0):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running unittests src/lib.rs (target/brief-508/build/debug/deps/brief_508_proof-47dd105277137b08)

running 4 tests
test falsifier_tests::falsifier_at_cap_refuses_new_pair ... full-cap control: used=cap=4096, fresh peak=4250 refused three times; no allocation/publication
ok
test falsifier_tests::falsifier_one_byte_below_cap_admits_one_byte ... below-cap positive control: used=4096 cap=4097 +1 byte admitted, used=4097
ok
test falsifier_tests::falsifier_one_byte_short_refuses_entire_two_pair_ingress ... two-pair negative control: cap=4306 required=4307 refused as a unit
ok
test falsifier_tests::falsifier_required_peak_one_below_cap_is_accepted ... whole-factory positive control: required=4307 cap=4308 admitted, one byte remains
ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

```

### 4. Reserved vocabulary, origins, and measured layout

This section was written before final verification/verdict. The finite draft
is `target/brief-508/reserved-pairs.tsv`; every entry's source/category/line
is enumerated in `reserved-origins.tsv`. `vocabulary.py` regenerates both
without compiling or executing production. `reserved-source-sha256.json`
records the read source snapshots. Inline test modules and test source files
are excluded. An initial provenance review found four names from
`exact_paths_tests.rs`; the generator's test-file exclusion was corrected
before measuring the final draft. The initial and corrected extraction
outputs are retained in `logs/vocabulary-{extraction,final}.log`.

The result is **940 pairs, 941 distinct strings, 15078 payload bytes**:
932 System pairs, seven Global pairs, and one Information pair. Categories
overlap; their counts must not be summed:

| Origin | Distinct pairs in category |
| --- | ---: |
| attribute_and_option_names | 31 |
| calendar_defaults | 7 |
| canonical | 81 |
| error_continuation | 206 |
| explicit_context | 1 |
| fixed_attribute_catalog | 509 |
| fixed_construction | 161 |
| fixed_installers | 6 |
| non_system_helpers | 5 |
| option_default_values | 8 |
| option_defaults | 40 |
| proposed_refusal_continuation | 1 |
| raw_message_helper | 2 |
| runtime_catalog | 185 |
| stdlib_catalog | 476 |
| yaml_fixed_values | 3 |

The 185 runtime and 476 stdlib catalog names have 18 overlaps, giving 643
catalog names. Their union with all 81 canonical heads is 661 pairs, exactly
matching the existing brief-506 bootstrap breakdown (63 canonical/catalog
overlaps, 580 noncanonical catalog names). This checks the census input; it
does not rerun or reinterpret its HashMap/HashSet byte measurements.

Source origins are:

- `crates/rum-core/src/heads.rs:112`: canonical head registry and its spelling
  constants. Runtime `builtin_catalog/**` and stdlib `catalog/**` provide the
  fixed BuiltinSpec names; runtime `stdlib_attributes/**` supplies its fixed
  name registry too. The default-feature hash names are included.
- `runtime/options/defaults.rs`: 40 option names and eight directly
  constructed default values; True/False also belong to the canonical set.
  `runtime/namespace_tools/names.rs` supplies the finite extra attribute and
  option-name list. `rum-stdlib/src/character_classes.rs` supplies six fixed
  installer names; `datetime/calendar.rs` supplies the seven weekday symbols;
  YAML's literal `symbol_value` calls supply Null/True/False.
- Fixed construction calls and internal continuation spellings across
  runtime/stdlib/host sources, especially `runtime/suspension/error.rs` and
  matcher/rewrite resume frames. The 206 existing internal spellings plus
  proposed `InternalContinuationEvalErrorInternerLimit` are included. The
  latter spelling is a draft entry, not an existing production error kind.
- `System` General, TerminatedEvaluation, MessageName, HoldForm, List,
  $Failed, $Aborted, and the proposed continuation head cover the refusal
  construction path. **InternerLimit and internerlim stay string values**;
  the draft explicitly rejects them as reserved symbols.
- The five Global parameters are rum$keyedArg, rum$extremalByArg, rum$sortArg,
  rum$replaceListArg, rum$replaceAtArg, from associations, collection_stats,
  ordering_sets, replace/list, and replace_at. The other two Global pairs are
  General and ThrowException, reconstructed from raw message names through
  `message_format.rs:3-8,23-30` and `session_helpers.rs:311-342`.
  `Information` InformationValueForm comes from
  `builtins_messages_inspect.rs:489-492`.

`ReservedToken` has private fields and only a validating membership lookup;
a caller's arbitrary `&'static str` is not a proof. Its finite declaration
contains only spellings; enumeration order supplies the private bootstrap
IDs. There are no attributes, specs, callbacks, or builtin-status fields.
All 940 token lookups unify with the corresponding ordinary interner pair
at a full cap and exhausted IDs, requesting no permit. A stack-backed scalar
message also succeeds then. This proves those declared names/refusal
spellings are available; **it is not a compiler-forced closure proof of every
production constructor**. Migration of variable-argument construction and
catalog consistency into typed production APIs remains unproven here.

The measured declared representation deliberately keeps a static TSV blob
in addition to the interned copies, so both are charged. The blob has 23538
bytes, its reference object 16, error-word reference array 32, and the two
error-word literals total 24. Charging those literal lengths separately is
an **upper bound** if linker substring/constant sharing occurs; it never
subtracts unobservable sharing. The 23610-byte static upper charge is
separate from the exact 83398-byte heap-layout-plus-fixed-reserve charge.
There is no separate canonical-symbol cache allocation in this substrate.
The published draft baseline is therefore **107008 proxy bytes**; sequential
bootstrap staging peaks at **133632**, including all old/new index overlap.
An implementation must cover that initialization peak or bootstrap using a
separately proved preallocated path; the retained baseline alone is not an
initialization admission proof.

| Candidate cap | Bytes | Minus reserved 107008 | Remaining proxy bytes |
| --- | ---: | ---: | ---: |
| 8 MiB | 8388608 | 107008 | 8281600 |
| 16 MiB | 16777216 | 107008 | 16670208 |
| 32 MiB | 33554432 | 107008 | 33447424 |

This is subtraction for the director, **not a recommendation or workload
calibration**. It does not revise the census's ordinary-suite maximum or
infer replacement-index Unique-call capacity. The scratch index has an
explicit 4096-slot limit and copies both backing tables per miss; neither
is a production sizing/performance claim.

Verbatim reserved-layout run (`logs/reserved.log`, exit 0):

```text
   Compiling brief-508-proof v0.0.0 (/home/remi/dev/rum/target/brief-508)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.59s
     Running unittests src/lib.rs (target/brief-508/build/debug/deps/brief_508_proof-47dd105277137b08)

running 1 test
test reserved::tests::reserved_declared_layout_and_full_cap_refusal_message ... reserved pairs=940 strings=941 contexts=System:932,Global:7,Information:1
reserved heap: text=15078 records=37600 text_index=17408 pair_index=9216 fixed=4096 retained=83398
reserved static declaration: blob=23538 header=16 error_headers=32 error_text=24 upper=23610
reserved total=107008 bootstrap_peak_heap=110022 bootstrap_peak_total=133632
cap=8MiB bytes=8388608 minus_reserved=107008 headroom=8281600
cap=16MiB bytes=16777216 minus_reserved=107008 headroom=16670208
cap=32MiB bytes=33554432 minus_reserved=107008 headroom=33447424
all 940 reserved hits at full cap + exhausted IDs: new permits=0 new allocations=0
scalar refusal message at full cap: System`General::internerlim: InternerLimit charged=107008 cap=107008
InternerLimit and internerlim are strings, not new symbol reservations; unknown static spelling rejected
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.26s

```

### 5. Proof verdict, remaining work, and final verification

**PASS for this explicitly scoped storage/admission mechanism.** The finite
safe-Rust substrate has exact dynamic allocation layouts, independently
checked peak and retained ledgers, preallocation budget reservation,
all-or-nothing two-pair admission, drop-on-refusal evidence, hit-before-permit
ordering, checked byte/ID boundaries, and a publication step with no
fallible work left. The enforcement-removal mutation was actually observed
red, and normal enforcement is restored by default. No ledger undercharge,
forbidden representation, or repeated no-progress proof failure occurred.

The reserved-vocabulary deliverable is a measured **draft**, with a closed
scratch token API and full-cap evidence for all declared names and scalar
refusal reporting. It does not certify complete production construction
closure. Its total includes the declared static input blob as well as the
interned copies; it is not substituted for the census's existing-container
budget or bytes-per-call measurements.

Still unproven/out of scope:

- Production integration, error propagation through all constructors,
  registration/serde/restore, infallible typed installers, and a
  compiler-enforced complete reserved vocabulary. Actual runtime error
  expression/continuation construction is not executed by this independent
  package; only its required spellings and scalar reporting seam are proved.
- An efficient production index, arbitrary context-path batching, and the
  process-global synchronization/configuration integration. The scratch
  tables have a finite 4096-slot ceiling and linear lookup. No performance,
  instruction-count, supported-workload, or Unique-call-capacity claim is
  made from them.
- Recoverable allocator OOM, allocator overhead, portable physical memory
  bounds, or RSS. Staging can still abort on allocator OOM even after budget
  reservation. The success/failure claims concern budget and checked
  representation decisions, with normal Rust allocation/drop semantics.
- The separate Module naming ruling and D7 state-lifetime audit. Production
  Symbol size/Copy/equality/public lifetime remain untouched; no new
  reclamation or per-session attribution scheme is introduced.

Director decisions respected: D1 retains the default-cap decision; the three
headroom rows are arithmetic only. D2 remains the ruled allocation-layout
proxy. D4's draft contains spellings/IDs only, with no behavioral fields.
The new interner continuation spelling is explicitly a draft for the future
D3/D5 migration, not a change to production semantics. D6's production
performance gate remains future work; no production suite or timing panel
was run for this scratch-only change. No additional user ruling was needed
to complete the mechanism proof.

Mandatory command actually executed via `run.py` (exact argv and environment
in `logs/verification.json`):

```sh
CARGO_NET_OFFLINE=true \
CARGO_TARGET_DIR=/home/remi/dev/rum/target/brief-508/build \
TMPDIR=/home/remi/dev/rum/target/brief-508/tmp \
RUSTUP_TOOLCHAIN=1.85.0 \
scripts/run-bounded.py --timeout 60 -- \
  cargo test --offline --manifest-path target/brief-508/Cargo.toml
```

The same-shape fresh baseline was 0.582268 seconds, as recorded in section 1;
the derived bound is 60 seconds. The final recorded elapsed time was
1.378697 seconds (timeout evidence only). Toolchain output, verbatim:

```text
rustc 1.85.0 (4d91de4e4 2025-02-17)
```

Final mandatory test output, verbatim (`logs/verification.log`, exit 0;
default parallel test scheduling also exercises independent per-test gates):

```text
   Compiling brief-508-proof v0.0.0 (/home/remi/dev/rum/target/brief-508)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.63s
     Running unittests src/lib.rs (target/brief-508/build/debug/deps/brief_508_proof-47dd105277137b08)

running 16 tests
test boundary_tests::boundary_duplicate_pair_is_one_admission ... ok
test boundary_tests::boundary_growth_drop_and_mixed_existing_pair ... ok
test boundary_tests::boundary_arithmetic_overflow_and_id_exhaustion ... ok
test falsifier_tests::falsifier_at_cap_refuses_new_pair ... ok
test boundary_tests::boundary_same_string_and_new_pair_existing_strings ... ok
test boundary_tests::boundary_long_and_empty_text ... ok
test falsifier_tests::falsifier_one_byte_below_cap_admits_one_byte ... ok
test falsifier_tests::falsifier_one_byte_short_refuses_entire_two_pair_ingress ... ok
test transaction_tests::transaction_fixed_reserve_covers_declared_inline_state ... ok
test transaction_tests::transaction_drop_after_stage ... ok
test storage::tests::storage_layout_classes ... ok
test falsifier_tests::falsifier_required_peak_one_below_cap_is_accepted ... ok
test transaction_tests::transaction_hit_before_permit_at_full_cap_and_id_limit ... ok
test transaction_tests::transaction_two_pairs_are_atomic ... ok
test reserved::tests::reserved_declared_layout_and_full_cap_refusal_message ... ok
test boundary_tests::boundary_every_growth_transition_exact_fit_and_short ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s

```

Scratch `cargo fmt --manifest-path target/brief-508/Cargo.toml -- --check`
also exited 0 with empty output (`logs/format-check.{log,json}`). Every
scratch Rust source file is below 10k bytes; the largest is 8949 bytes.
This appended evidence report exceeds the document-size threshold: the
brief explicitly requires one append-only report and verbatim output, and
forbids moving it to another document. That is the documented exception;
implementation responsibilities are split into storage, planning,
admission, transaction, vocabulary, and behavioral test modules.

Other command failures are preserved rather than hidden. An initial
formatting command mistakenly supplied Cargo's `--offline` flag to
`cargo fmt`, which does not accept it. The corrected command retained
`CARGO_NET_OFFLINE=true`, removed that flag, and both formatting and its
check then succeeded. Original diagnostic, verbatim (`logs/fmt.log`; the
exploratory shell's final `cat` did not preserve cargo-fmt's own exit code):

```text
error: unexpected argument '--offline' found

  tip: to pass '--offline' as a value, use '-- --offline'

Usage: cargo fmt [OPTIONS] [-- <rustfmt_options>...]

For more information, try '--help'.
```

One read-only source search also included a nonexistent directory. Its
error line was:

```text
rg: crates/rum-runtime/src/model: No such file or directory (os error 2)
```

The actual attribute definitions were subsequently read from
`runtime/metadata.rs`. This was not a test or storage-proof failure.

The first scope audit correctly detected trailing whitespace copied from
the mandatory raw red-run output. Its original failure, verbatim
(`logs/scope.log`):

```text
Traceback (most recent call last):
  File "/home/remi/dev/rum/target/brief-508/audit_scope.py", line 24, in <module>
    assert result.returncode==0,result.stdout.decode()
           ^^^^^^^^^^^^^^^^^^^^
AssertionError: docs/dev/codex/brief-508-interner-storage-and-admission-proof.md:538: trailing whitespace.
+test falsifier_tests::falsifier_at_cap_refuses_new_pair ... 
docs/dev/codex/brief-508-interner-storage-and-admission-proof.md:547: trailing whitespace.
+test falsifier_tests::falsifier_one_byte_short_refuses_entire_two_pair_ingress ... 

```

Those spaces are preserved to honor the verbatim-evidence requirement.
`git diff --check` therefore remains exit 2 for copied evidence lines;
it is **not** reported as a passing check. The final scope audit checks that
all such lines originate in saved raw logs, then separately verifies file
scope. Before this closing appendix, its output was (`logs/scope-final.log`):

```text
{
  "tracked_files_checked": 1483,
  "changed_tracked_files": [
    "docs/dev/codex/brief-508-interner-storage-and-admission-proof.md"
  ],
  "original_brief_prefix_preserved": true,
  "git_diff_check_exit": 2,
  "git_diff_check_output": "docs/dev/codex/brief-508-interner-storage-and-admission-proof.md:538: trailing whitespace.\n+test falsifier_tests::falsifier_at_cap_refuses_new_pair ... \ndocs/dev/codex/brief-508-interner-storage-and-admission-proof.md:547: trailing whitespace.\n+test falsifier_tests::falsifier_one_byte_short_refuses_entire_two_pair_ingress ... \n",
  "only_verbatim_evidence_whitespace": true,
  "scratch_rust_files": 9,
  "largest_scratch_rust_file_bytes": 8949
}
```

`audit_scope.py` is rerun after this appendix as well; the resulting final
scope data is in `scope-audit.json` and `logs/scope-handoff.log`. It verifies
all 1483 initially tracked file hashes, allows only this brief to differ,
and verifies the entire original brief prefix is intact. Source/data
artifact hashes are in `artifact-sha256.json`. All package sources, generated
vocabulary data, temporary files, build products, and logs remain under
`target/brief-508/`. Nothing was committed or staged, and no production
source/manifest/lockfile/script/inventory was modified.
