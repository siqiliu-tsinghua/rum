# Progress Log

This file records completed work, archived planning detail, and operational
lessons that are useful history but should not keep cluttering `TODO.md`.

`TODO.md` should stay focused on active or near-term work. When a completed
checklist stops helping prioritization, move or summarize it here.

**Archive note (2026-07-03):** the previous log (6386 lines, everything through
the long-tail/capability-sandbox/release-prep era) moved to
`docs/PROGRESS-archive-2026-07-03.md`. State at archive time: correctness
mainline complete (README 3126/0 reproduced exactly); host-embedding hardening
arc done (Tier-1 + Tier-2 core + rum-jupyter real interrupt); long-tail builtin
batches done and the support matrix accuracy-passed (brief-094); capability
sandbox landed (brief-095); public-release scrub method recorded and
`release-staging/` deleted pending re-creation (brief-096); release DEFERRED in
favor of code cleanliness. Active line: crate rebalancing, route 2a —
`docs/crate-rebalance-plan.md`.

## 2026-07-04 Hygiene-policy revision + coda batches DONE (briefs 120/121) — the whole line closes

The user asked for an evidence-based review of the size-hygiene policy
before continuing; the review's four amendments landed in
`docs/code-organization-guidelines.md` (4c88a44): quantified test-file
thresholds (15k/25k/45k) + identity-proof rule, a NEW Directory Fan-Out
section (~20 direct children soft ceiling), measure-implementation-after-
test-extraction, and assess-on-touch replacing the split-plan blanket-TODO;
plus an honest scoping of the incremental-compilation goal (crate is the
unit) and a "Splitting Is Organization" section codifying the brief-114
principles.

Then two coda batches applied it. Brief-120 (932c4d9): the four 30k+ files
split on real boundaries (iteration → spec/driver/table_fast_path with the
113 fast path housed intact; model → 9 type-domain modules; scoping/lexical
→ block/module/with; eval_core → symbol/call/stability) — 22 modules
≤17.4k, sentinel improved both A/B runs, the assess-and-keep option was
available but unneeded. Brief-121 (this commit): runtime/'s fan-out grouped
74 → exactly 48 via five fixed domain directories (definitions/, messages/,
session/, substitution/, kernel_data/ — the last collecting the rebalance
line's kernel-resident *_data machinery), transparent re-exports, zero body
changes, sentinel 55ns/iter. Workspace 1301/0 through everything.

## 2026-07-04 File-slimming line COMPLETE (brief-119) — the >35k debt tier is ZERO

The final batch split the two test files by BEHAVIOR FAMILY with a
test-name identity proof (stdlib 359=359, syntax 152=152, normalized diff
EMPTY): `incidental_runtime.rs` 67k → 12 behavior modules ≤12.2k (its
migration-artifact grab-bag decomposed into what is actually being tested);
`parser/tests.rs` 38k → 8 grammar-area modules ≤12.8k. Gate re-verified the
line's end state: the workspace-wide >35k audit is EMPTY (was 9 files two
days ago: profiling 138k, catalog 74k, incidental 67k, evaluation_cache
63k, definition_application 55k, vm_rhs_template 53k, arithmetic 42k, lexer
41k, parser/tests 38k); the 20-35k split-plan band stands at 26 files (the
next-granularity decision, not urgent). Workspace 1301/0 conserved through
all six batches; two hot splits carried independent gate A/Bs; zero
behavior change end to end. The splitting principles (by logic/function,
incremental-compilation-aware, future-refactoring-aligned, verbatim, split
map first) are codified in brief-114 for reuse.

## 2026-07-04 File-slimming batch 5 DONE (brief-118) — arithmetic.rs 42KB split by numeric domain

The hottest bodies split into 8 modules: `canonical.rs` (18.5k — Plus/Times/
combine_* kept COHESIVE per principle 1, the biggest-but-justified module),
exact (shared rational helpers), rounding, integer_ops, factorial, extremum,
parts, facade. Bodies verbatim; public surface (ArithmeticDataBuiltin,
eval_arithmetic_data_builtin, expr_as_exact_*, combine_*) unchanged; only two
sibling-shared helpers widened to pub(super). A/B: the brief-113 regression
SENTINEL table-iterator-plus at +1.7% (clean); do-small −5.7%; downvalue
cluster +2-5% inside the day's drift band. Gate re-verified: 1301/0
conserved, smoke 110, sentinel triple-probe 55-69ns/iter (matches the
post-113-fix baseline). Remaining debt: the two test files (batch 6, last).

## 2026-07-04 File-slimming batch 4 DONE (brief-117) — vm_rhs_template.rs 53KB + lexer.rs 41KB split

The opt-in VM split into 7 modules (model/compiler/executor/cache/safety/
tests + facade, ≤16.2k) with the disabled-gate check verbatim in cache.rs
and boundaries that keep the future feature-gate/move decision trivial. The
lexer split into 10 modules (core dispatch loop, cursor primitives, strings/
escapes, named_characters table, numbers, symbols, operators, trivia, tests
+ facade, ≤14k) with a clean DAG rooted at cursor/operators. Zero caller
changes (directory-module convention). The pre-existing source_template
dead-code warning was left for the VM-fate decision (the fix-in-passing
authorization was optional). Gate: 1301/0 conserved; lexer named-character
escape smoke (`\[Alpha]`) → 21. Remaining debt: arithmetic 42k (HOT, next),
the two test files.

## 2026-07-04 File-slimming batch 3 DONE (brief-116) — definition_application.rs 55KB split by value-application domain

The hottest file of the line split into 8 modules, all ≤14k: down_values /
up_values / sub_values (dispatch paths), fast_match (the hot candidate/
early-out layer, ISOLATED), default_values, rhs, replace_all (the rewrite
drivers the rewrite_data wrappers consume), mod (facade + shared). DAG:
dispatch paths → fast_match/rhs; replace_all → fast_match. Bodies verbatim;
visibility adjusted only where module boundaries force it. Codex's A/B first
run showed +3-13% across judged labels, receding on its rerun to ≤+5.6% on
two labels — because this is the hottest file, the GATE ran an independent
fresh A/B (HEAD worktree vs working tree): ALL judged labels within ±3.6%,
mostly NEGATIVE (downvalue-exact-hit −1.4%, replaceall −2.8%, matchq
−2.4~−3.5%) — drift confirmed, split neutral. Workspace 1301/0 conserved.
Remaining debt: vm_rhs_template 53k, arithmetic 42k, lexer 41k, 2 test files.

## 2026-07-04 File-slimming batch 2 DONE (brief-115) — evaluation_cache.rs 63KB split on the mechanism/instrumentation axis

The first HOT split held its red lines: `evaluation_cache/` now separates the
per-eval cache MECHANISM (`stability.rs`, 15.5k — impl Session decisions +
bloom) from the env-gated INSTRUMENTATION (`profile/{mod,source,report}.rs`,
the future move-to-profiling/ candidate) with a 1.6k facade + verbatim
tests.rs; all files ≤19.8k. Codex ran the A/B itself per the brief: machine
labels improved-or-flat, the suspicious downvalue cluster (+8%) fell to
+2-3% on the allowed rerun — no consistent regression; measurement-hygiene
note of the batch: it caught the microbench script reusing a STALE release
binary, discarded the polluted run, and rebuilt before measuring. Gate
re-verified 1301/0 conserved, pure-deletion diff (1681 lines), downvalue
smoke 25. Pre-existing dead-code warning noted in vm_rhs_template.rs
(source_template) — untouched, belongs to that file's own batch.

## 2026-07-04 File-slimming batch 1 DONE (brief-114) — profiling.rs 138KB + catalog.rs 74KB split

The slimming line opened (the debt deferred by the rebalance plan; user
principles codified in the brief: split by logic/function never mechanically,
optimize for incremental compilation — stable vs churning regions, sibling
DAG only — and for future refactoring — boundaries movable wholesale; pure
organization, split map first). Batch 1 took the two largest COLD files:

`runtime/profiling.rs` (138k chars, 151 fns) → `profiling/` with 17 modules,
all ≤14.8k: state/config/counters/scopes (stable declarations + env gates) →
records/timing/presence/fingerprints (instrument domains) → report_* +
summary + formatting (churny assembly). The split map survived one revision
(mod.rs initially 24k → stable state extracted). Disabled-path red line
asserted: same env-gated fast-outs, bodies verbatim. `stdlib/catalog.rs`
(74k) → `catalog/` with 14 per-family files (≤12.5k) + an 881-char facade;
install_core and the drift test compile UNCHANGED — and the bidirectional
attribute-drift test passing IS the strongest verbatim proof for the spec
data (gate also verified SET_OPERATION_ATTRIBUTES pre-existed and the 345
count). Zero >20k exceptions; zero caller changes (git status shows exactly
2 deletions + 2 new directories). Gate: fmt + workspace 1301/0; CLI smoke 61.
Remaining debt: evaluation_cache 63k + definition_application 55k (HOT, next,
with A/B), vm_rhs_template 53k, arithmetic 42k, lexer 41k, 2 test files.

## 2026-07-04 Stage-3 FINALE DONE (brief-113) — arithmetic moved; ALL 14 families' spec ownership in rum-stdlib

The line's endpoint: `builtin_catalog/numeric.rs` is GONE; the 27 hottest
handlers (Plus/Times/Power/…, + Identity) moved as wrapper-shape specs
(stdlib catalog 345); arithmetic bodies + `combine_*` canonicalization stay
kernel-side behind `ArithmeticDataBuiltin`. Kernel catalog residue is now
pure evaluator/host machinery: definition/rule machinery + Part family
(rules.rs), Function/HoldForm, scoping, namespace/context/loading,
debug/resource tools, 19 symbol-only string specs, known-symbol vocabulary.
A shared `#[cfg(test)] test_builtins.rs` now serves runtime mechanism tests.

THE GATE CAUGHT THE LINE'S ONE REAL PERFORMANCE REGRESSION here — the
mandatory finale A/B found `table-iterator-plus` at **14x** (63→907ns/iter)
while all 1300 tests stayed green (a pure fast-path-eligibility loss,
invisible to correctness). Root cause: the iterator fast path gates on
`session attributes == default_system_attribute_set(symbol)`; the static
side lost EVERY moved family's attribute data as catalogs left the kernel
(latent since batch 5, exposed only when Plus — the hottest symbol — hit the
check). Architectural conclusion recorded: symbol ATTRIBUTES
(Orderless/Flat/Hold*) are evaluator semantics and must stay kernel-known —
of the spec quintuple, handler and capability may move; static attribute
knowledge may not. Fix: kernel-side attribute vocabulary covering all 345
stdlib specs (112 non-empty) consulted as the fallback + a BIDIRECTIONAL
drift test (stdlib spec attrs == kernel vocabulary, no missing/no extra).
Fix verified: table-iterator-plus stable at −6~−10% vs base across two runs;
remaining table-wide +10-19% swings re-run-verified as machine drift
(receding + direction-flipping; the measurement environment ran ~15
compile+bench cycles today — noise floor ±10% acknowledged in the judgment).
Workspace 1301/0 (the drift test adds one); CLI smoke → 55.

## 2026-07-04 Stage-3 batch 11 DONE (brief-112) — rewrite_builtins: wrapper shape, engine untouched

Codex chose the brief-111 wrapper shape with clean evidence (the rewrite
bodies are matcher machinery: match/compile/normalize, rewrite-rule drivers,
traversal resolution, suspension drivers, trace hooks — a full move would
blow the promotion line): implementation renamed `runtime/rewrite_data*`,
7 specs moved (MatchQ, Cases, DeleteCases, Replace, ReplaceList, ReplaceAll,
ReplaceRepeated; stdlib catalog 318) behind thin wrappers via
`RewriteDataBuiltin` + one dispatch method (promotions: 2). Count/MemberQ/
FreeQ/Position/Level/Rule/NormalizeRules and the definition machinery stayed
(handlers not rewrite-module-owned). Tests: zero moved — mechanism tests use
test-local registrations per the amended rule. Workspace 1300/0; CLI → 7.

Gate ran the declared microbench A/B (wrapper covers hot ReplaceAll):
rewrite/matcher labels flat-to-faster (replaceall −1.6~−2.8% across two runs,
matchq −2.6~−4.2%, downvalue family ±2%). A +13% cluster appeared on the
SMALLEST sub-ms labels (loop-empty, do-iterator-null, literal-integer) —
re-run verdict: machine-state drift, not regression (second run halved it;
`do-small` which actually executes Do bodies was NEGATIVE; literal-integer is
untouched by both batches). Two wrapper batches, two neutral A/Bs — the
wrapper shape is performance-safe. Remaining: arithmetic (the finale).

## 2026-07-04 Stage-3 batch 10 DONE (brief-111) — control_flow: spec ownership moved, implementation stays

The highest-coupling family got a DIFFERENT solution shape, gate-reviewed in
depth and accepted: reference analysis showed nearly the whole implementation
is evaluator machinery (non-local-exit signals consumed by eval/VM paths,
loop iterators on `BlockSymbolState`, abort/cleanup on the interrupt state,
failure/confirmation helpers used by eval_core/eval_dispatch) — so the
implementation stayed kernel-side (renamed `runtime/control_data/`), while
the 35 SPECS moved to the stdlib catalog with thin wrapper handlers
delegating through one promoted facade
(`ControlDataBuiltin` enum + `Session::eval_control_data_builtin`). Semantics
of the split: registration ownership (no stdlib install → no If/Do) is
stdlib's; the machinery is honestly kernel. Promotions: 2 (vs ≤12 line).

Because the wrapper adds a per-call indirection on HOT handlers
(Do/CompoundExpression/If), the gate ran a full microbench A/B (HEAD worktree
vs working tree): control-relevant labels flat-to-faster (do-small −4.2%,
if-true −1.7%, do-count-null −6.8%), the few +5% entries are sub-ms jitter
with contradicting same-mechanism neighbors — NO consistent regression;
neutral confirmed. Also accepted with a SECOND discipline note: Codex kept
~12 mechanism-test files runtime-side with TEST-LOCAL control registrations
instead of migrating them per the Option-1 letter — the right instinct (it
matches the batch-5 exile-list concern) but a rule deviation it should have
STOPPED to report; the rule is now amended going forward: mechanism tests MAY
stay runtime-side with test-local registrations, listed per batch. Behavior
adaptation this batch: `$AssertFunction`/`PropagateAborts` added to the
known-symbol vocabulary (string option forms). Workspace 1300/0 conserved
(runtime 430, stdlib 358); stdlib catalog 311; CLI smoke → 22. Remaining:
rewrite_builtins, arithmetic.

## 2026-07-04 Stage-3 batch 9 DONE (brief-110) — functional_ops moved; apply machinery stays

The cleanest of the big moves: 42 specs out (Map/Apply/Fold/Nest/Select/
Array/Outer/Composition/…), and the runtime functional catalog is now
EXACTLY the evaluator-semantics residue — `Function` and `HoldForm`. The
callable-application machinery the dispatcher consults
(parse_composition/apply_composition_to_args, operator-wrapper apply,
comparison/Key/NearestTo wrappers) stayed kernel-side as
`runtime/functional_data.rs`; the pure-function apply machinery was never
touched. Promotions 7/8 (line held). Test migration surprisingly small — 18
fns (16 whole-file across 6 files + 2 partials) — because earlier batches had
already pulled the functional test hotspots. Workspace 1300/0 conserved
(runtime 510, stdlib 278); stdlib catalog 276; CLI smoke → 12; gate
cross-boundary smoke (kernel Function × stdlib Map/Total) → 14. Remaining:
control_flow, rewrite_builtins, arithmetic.

## 2026-07-04 Stage-3 batch 8 DONE (brief-109) — structure_ops moved; Part stays kernel

The reference-analysis split produced a bigger stay than any batch yet, and
rightly so: `Part`/`Extract`/`ReplacePart`/`ReplaceAt` — specs (in
`builtin_catalog/rules.rs`) AND implementation — stayed kernel-side as
`runtime/structure_data/`, because Part-lvalue assignment is definitions-code
machinery; their runtime tests stayed too (`tests/structure_data/`). The other
38 specs (Length/Flatten/Sort/Take/…) + implementation moved. Runtime catalog
279, stdlib 234; workspace 1300/0 conserved (runtime 529, stdlib 259); CLI
smoke → 7, and the gate added a cross-boundary smoke (kernel-resident
ReplacePart/Extract composed with stdlib-resident ops) → 104. Promotions 7/8 —
the resized stop line held this time. Test-local registration list grew:
scoping_gensym_gc now registers a minimal `First` (same pattern as the VM
SameQ). Next: functional_ops — Codex flags `targeted_map.rs` as depending on
kernel-side exact-path helpers; reference analysis before handlers move.

## 2026-07-04 Stage-3 batch 7 DONE (brief-108) — associations moved with the data/builtin split; 9 families out

The designed split held: the association DATA LAYER (entry forms,
parse/render, key lookup/apply, set-lvalue — everything referenced by
kernel-side code or not-yet-moved families) stayed kernel-side as
`runtime/association_data.rs`; the 33 builtin specs + family helpers moved
(`MissingQ` stayed, its call is kernel-adjacent). `eval_dispatch` untouched
(the split-correctness stop-loss never fired). Runtime catalog 317, stdlib
196; workspace 1300/0 conserved (runtime 577, stdlib 211); CLI smoke → 21.
Biggest migration so far, as predicted from the 28-file incidental footprint:
22 dedicated tests + ~40 measured moves (9 whole-file + 12 partial sources).

Two gate notes. (1) Promotions came to 6 narrow facades — ONE OVER the
brief's ≤5 stop line; Codex should have stopped and reported rather than
finishing (outcome harmless, all six justified, but the discipline slip is
noted; future briefs size the line to the family's audited kernel-import
count instead of a flat 5). (2) One behavior-equivalence adaptation was
required and gate-verified: the new `protected_or_known_definition_target_symbol`
facade adds a REGISTERED-spec branch alongside the static vocabulary — moved
builtins (no longer in the static catalog) must still count as known/protected
for AssociateTo; consistent with the brief-102 resolution-probe conclusion.

## 2026-07-04 Stage-3 batch 6 DONE (brief-107) — predicates moved; 8 families out

50 predicate specs moved (runtime catalog 350, stdlib 163); the family got its
free-fn conversion in transit; `builtin_catalog/predicates.rs` deleted whole
(`MissingQ` stayed — its spec lives in the associations catalog). Migration
measured 33 test fns (vs logic's 31 — the mild trend holds). Promotions 4
(≤5 line): the audit-predicted `invalid_call_with_arg_form` +
`collect_level_parts_owned` + 2 `CasesTraversalConfig` constructors.
Workspace 1300/0 conserved (runtime 644, stdlib 144); CLI smoke → 2.

Two gate-reviewed edge calls, both accepted with lessons: (1)
`associations/basic/predicates.rs` renamed to `key_predicates.rs` ONLY because
the brief's grep gate used a bare word — grep gates must be path-precise
(second wide-grep incident after brief-100's); (2) the VM template tests lost
`SameQ` and now register a TEST-LOCAL minimal SameQ (structural equality) via
the open registry — probes unchanged, and crate-internal mechanism tests
genuinely cannot migrate (they touch runtime internals). EXPECT this pattern
to recur when control_flow moves (VM tests likely use If/Do too); the
test-local registration list joins the exile list as future-cleanup tracking.

## 2026-07-04 Stage-3 batch 5 DONE (brief-106) — logic moved; the comparisons left the kernel

The user chose Option 1 for the load-bearing tier (tests migrate WITH their
families; no dev-dep cycle; runtime test tree trends mechanism-only), and the
first mass migration turned out MILD: measured reality was 31 test fns
(22 whole-file + 9 partial), not the feared hundreds — runtime tests lean on
comparisons far less than assumed. Logic itself was the lowest-increment move
(free-fn conversion + kernel-method pub-ification all done in 099/101):
21 specs moved, runtime catalog 400, stdlib 113; the leftover catalog slice
renamed honestly to `builtin_catalog/predicates.rs` (50 specs); obsolete
nand/nor adapter shims deleted. Workspace 1300/0 conserved (runtime 679,
stdlib 111); CLI smoke `If[Xor[True, False] && 1 <= 1, 42, 0]` → 42.

Promotions: 3 (within the ≤5 line) — the gensym-GC introspection counters
(`collect_and_sweep_dead_gensyms` visibility flip + 2 read-only count
accessors), needed because the gensym-GC MECHANISM test uses logic builtins in
its probes and migrated per the Option-1 rule. Noted for a future cleanup
line: mechanism tests exiled to the stdlib only because their probe
expressions use stdlib builtins could be probe-rewritten and repatriated
later; the rule "don't rewrite probes to dodge migration" was correct for
keeping this batch honest, but the exile list is worth tracking (currently:
scoping_gensym_gc partial, debug_tools_resources partial).

## 2026-07-04 Stage-3 batch 4 DONE (brief-105) — strings moved (largest batch); 6 families out

The biggest single family so far: `strings` (facade + 21 files ≈6.5k lines, 61
handler-backed specs) now lives in rum-stdlib (which gained the `rum-syntax`
dep for ToExpression/SyntaxQ). Runtime catalog 482 → 421 (19 symbol-only
string specs legitimately stay); stdlib catalog 92; workspace 1300/0; CLI
smoke `StringLength[StringJoin[{"ab","cd"}]] + ToExpression["3*4"]` → 16.
Tests: the whole `tests/strings/` directory + 19 measured-failure fns
(→ `tests/strings/incidental_runtime.rs`) + the FileTemplateApply capability
assertion moved with the handler.

The one structural judgment call, gate-audited in depth: ToString/TextString's
expression RENDERER is shared kernel machinery (message_format,
builtins_input, scoping, control_flow all render through it) — it stayed in
the kernel as `runtime/text_rendering.rs`. Verified: byte-identical to the old
`strings/text/render.rs` modulo visibility/docs, plus `string_form_value`
folded in from template.rs (its only caller is the renderer itself). No
formatting code was copied or forked. Promotions within stop-loss: 7
kernel_api exports (incl. `Capability`, `ToStringFormat`, `render_*`) + 7 pub
Session methods (incl. the suspension API `suspend_with_frame`,
`drive_evaluated_value`/`drive_internal_value`,
`enforce_active_memory_limit_for_projected_bytes`) — 10 of 14 were
audit-predicted. Stop-loss lines (>12 new promotions / >40 failing fns /
pattern-matcher entanglement) NOT hit: measured 20 failing fns, no
entanglement (the string-pattern compiler moved cleanly with the family).

## 2026-07-04 Stage-3 batch 3 DONE (brief-104) — calculus + polynomials moved; 5 families out

First multi-file directory family through the pipeline: `polynomials/`
(mod/candidates/core/core/factors/factorization/rational_function) moved with
its layout intact; `calculus` (the `D` handler) along. Runtime catalog
488 → 482, stdlib 31 (11+5+9+1+5); runtime greps clean; CLI smoke
`--eval "Coefficient[Expand[(x+1)^4], x, 2] + D[x^3, x] /. x -> 2"` → 18;
workspace 1299/0 (runtime 753 + stdlib 34; 4 calculus + 13 polynomials test
fns migrated). One family-boundary discovery handled correctly: the
`PolynomialQ`/`PolynomialExpressionQ`/`RationalExpressionQ` PREDICATE handlers
physically lived inside `polynomials/mod.rs` — they moved VERBATIM (md5-equal
bodies, gate-checked) into `runtime/predicates.rs` so the predicates family
stays runtime-owned; their catalog entries never left. Promotions:
`HoldAttribute` + `Session::invalid_call_with_message` pub;
`parse_indeterminate_expr` facade retargeted. Remaining Stage-3 families: 8
(predicates, logic, arithmetic, strings, structure_ops, functional_ops,
associations, control_flow, rewrite_builtins — the load-bearing tier; test
strategy per family by measured failing-fn count).

## 2026-07-04 Stage-3 batch 2 DONE (brief-103) — integer_digits + number_theory moved; 3 families out

First BATCH move (two families in one brief) through the proven pipeline:
`rum-stdlib` now owns `integer_digits` (5 handlers) and `number_theory` (9
handlers, incl. the Prime/PrimePi small tables and Pollard-rho path); runtime
catalog 502 → 488, stdlib catalog 25 (11+5+9); runtime greps clean; CLI smoke
`--eval "NextPrime[100] + FromDigits[{1,2,3}] + EulerPhi[10]"` → 228; workspace
1299/0. Promotions: `Session::{invalid_call_with_arg_range,
enforce_generated_element_limit_for_count, thread_call_args, push_message}`
now `pub` (+ `evaluate` documented); `kernel_api` re-exports
`num_integer::{Integer,Roots}` + `num_traits::{One,Signed,Zero}` additions.

Two process notes. (1) The NEW test-migration rule proved out: `number_theory`
tests moved as a whole file (8 fns), while `integer_digits` tests were embedded
in `tests/arithmetic.rs` — the rule "run the runtime suite after the move; every
MEASURED failing function moves whole to the stdlib" migrated exactly 3
functions, no guesswork. (2) The first dispatch of this brief was killed
mid-move by my own foreground-chained dispatch command (Bash tool 2-min
timeout); the half-done tree was rolled back cleanly and re-dispatched. That
failure mode is now MECHANICALLY blocked by a PreToolUse hook
(`.claude/hooks/guard-dispatch.py`): any `dispatch-brief`/`cdx exec` Bash call
that is chained, bare-`&` detached, or not run_in_background is denied by the
harness before execution (DRY_RUN=1 exempt) — verified live.

## 2026-07-04 Stage 3 OPENED and the MOVE pipeline is PROVEN (brief-102) — combinatorics lives in rum-stdlib

The first family physically left the monolith. `rum-stdlib` now owns the
combinatorics implementation (`src/combinatorics.rs`, 11 handlers converted to
free fns while moving, bodies verbatim), its specs (`src/catalog.rs`, 11
entries byte-identical to the deleted runtime ones modulo handler path), and
its 5 behavioral tests; `install_core` = runtime catalog (now 502 entries)
then stdlib specs. rum-runtime greps CLEAN for combinatorics. Pilot family was
combinatorics (not logic) because it has zero incidental test usage — logic
owns the comparison operators the kernel's own tests use everywhere.

Step-0 resolution probe (the one real unknown) PASSED: a builtin registered
under a name ABSENT from the static catalog resolves and dispatches through
`eval_text` — no vocabulary fix needed; the probe stays as a regression test
(`registered_builtin_resolution.rs`). Kernel promotions were restrained and
documented: `Session::{invalid_call_with_exact_arg_count,
combine_additive_args, combine_multiplicative_args}` now `pub`;
`kernel_api` re-exports `BuiltinSpec` + `num_traits::ToPrimitive` (spares the
stdlib a num-traits dep). rum-stdlib gained the `rum-core` dep.

Gate: fmt + workspace 1299/0 (runtime −5 moved tests +1 probe, stdlib +5);
CLI smoke `--eval "Fibonacci[10] + CatalanNumber[4]"` → 69 (the brief's
literal `eval` subcommand syntax was wrong — rum-cli takes `--eval`; noted for
future briefs). No microbench (registration source only). Remaining Stage-3
work: the other 12 families through the same pipeline; load-bearing ones
(logic/predicates/arithmetic/…) additionally need the test-migration /
dev-dep-cycle decision recorded in the memory notes.

## 2026-07-03 Stage-2 step 3 DONE (brief-101) — logic handlers are free functions

The pilot family's handlers no longer live in `impl Session`: 16 handlers
(truth.rs 8 + comparison.rs 8, incl. the private `eval_comparison_subset` and
the truth continuation helpers) are free functions
`fn eval_x(session: &mut Session, …)` with bodies VERBATIM (receiver-only
change); the facade re-exports them per-item; `builtin_catalog/logic.rs` and
the nand/nor adapters reference `crate::runtime::logic::eval_*` explicitly.
Scope note that shaped this brief: catalog file grouping ≠ implementation
module grouping (catalog/logic.rs references 69 handlers; the logic MODULE
owns 16 — the rest are predicates/structure_ops and stay untouched until their
family's brief). Gate: fmt + workspace 1298/0; zero `impl Session` under
logic*; no microbench (static call → static call, dispatch fn-pointer shape
unchanged). The logic family is now move-ready: brief-102 = the physical move
pilot (files → the stdlib crate, spec ownership, kernel Session-method
pub-ification, test migration).

## 2026-07-03 Stage-2 step 2 DONE (brief-100) — registration is the ONLY builtin dispatch path

Discovery that motivated it: dispatch resolution silently fell back to a static
`OnceLock` map over `core_builtin_specs()`, so builtins never needed
registration and `rum-stdlib::install_core` was decorative — families cannot
leave the crate while the kernel statically consults its own catalog. Now:
`resolution.rs` reads ONLY the session's registered map (fallback deleted);
`Session::default()`/`new()` auto-install `core_builtin_specs()` (the
transitional bridge — shrinks as families move out); `with_config()` builds a
truly RAW session (hosts/stdlib must install — they already do);
`register_builtin` applies the `with_fallback_dispatch` merge against the
previously REGISTERED spec, preserving user-override semantics; `Names`
enumerates registered specs. The static `default_builtin_dispatch_spec*`
helpers legitimately REMAIN as vocabulary (known-system-symbol predicates in
`expr_runtime_helpers.rs` — a dispatch-vs-vocabulary distinction the original
brief's grep gate missed; Codex stop-lossed on it and the gate was amended).
New test: fresh session evaluates a builtin + DumpSave/Get restore keeps the
registered set. Gate: fmt + workspace 1298/0; resolution grep = 0; microbench
A/B flat-to-better (−8%…+1.6%, no consistent regression; mechanism touches
only the cache-miss resolution path).

## 2026-07-03 Stage-2 step 1 DONE (brief-099) — kernel_api v1 exists, logic family de-globbed

`runtime/kernel_api.rs` is now the single public door to the kernel's helper
surface: 40 audited fan-out ≥2 items re-exported in 14 grouped `pub use` lines
(`pub mod kernel_api` from `runtime/mod.rs`; defining `mod`s stay private, so
the promoted `pub` items are unreachable except via kernel_api). Pilot: the
`logic` family (`logic.rs` + `logic/{comparison,shared,truth}.rs`) carries NO
`use super::*` — only explicit `kernel_api`/`rum_core` imports. The pilot
CONFIRMED the audit model: demanded exactly the predicted 9 items (5 kernel_api
+ 4 rum_core), zero unplanned promotions.

Learned (now standing for briefs 100+): promoted items sit in private
submodules behind family facades; Rust cannot re-export a restricted re-export
(E0364/E0365), so facades need per-item `pub use self::<submod>::item;` lines
(never widen a glob re-export) — facade files belong in MAY-modify from the
start. Codex stop-lossed correctly on this and was resumed with a scoped
authorization. Gate: fmt + `cargo test --workspace` 1297/0; visibility/imports
only (rustfmt reflow aside, no body/signature changes; no microbench — no
codegen delta); the `SymbolExprMap`-vs-private-hasher warning closed at gate
time by making `RuntimeCacheHasher` `pub` inside its private mod (one word +
comment). Diff: 21 files, ~+56/−51.

## 2026-07-03 Stage-2 audit COMPLETE (brief-098) — the real kernel API is tiny

Method: per-family compile-error audit (comment out `use super::*` in each of the
14 builtin family modules, one at a time; collect unresolved names from
`cargo check`; restore) + a grep axis for kernel `Session` methods called from
family code. Full artifact: `docs/dev/kernel-api-audit-2026-07-03.md` (from
`/tmp/cdx_brief_098_report.md`); raw logs `/tmp/cdx_brief_098_raw_<family>.log`.
Diagnostic only — tree clean, final `cargo check` green. (Operational note: the
first run stopped at 6/14 families because Claude's concurrent PROGRESS-archive
commit tripped the whole-tree clean-check; resumed with the check scoped to
`crates/`. Lesson recorded: no repo mutation while a brief is in flight.)

Findings (free-item axis): of the runtime-internal namespace, only **258 items
are used by any family at all**; **24 have fan-out ≥3 families** (the prelude
core: `Session`/`EvalError` types, `system_expr`, `is_system_symbol_name`, the
`expr_as_*` coercion helpers, `compare_expr_canonical`, a few model/session
types like `ThreadingDispatch`/`ThreadArgSpec`/`CasesLevelSpec`), 16 have
fan-out 2, **218 are single-family** (move WITH their family, don't promote),
and ~600 internal symbols are used by no family (stay kernel-private). Known
family→family couplings that constrain move order: `arithmetic` numeric
helpers (`expr_as_exact_integer` 7 families, `expr_as_exact_rational` 5),
`associations/data` helpers (`association_entries_from_expr`,
`render_association` — 6 families each), structure path helpers.

Findings (Session-method axis, the future pub method surface, top of ~15-25):
`evaluate` (148 calls), the `invalid_call_with_*` argument-error helpers
(131+113+112+67), `suspend_with_frame` (100) + `drive_internal_value` (41) +
`suspend_with_result_frame` (17), `push_message` (37), the `enforce_*` resource
limits, `match_expr_owned`, `merged_option_exprs_for_system_symbol`.

Cheapest first move: **`logic`** — 9 free items, no same-family glob fallout, no
other-family coupling. Count reconciliation note: the earlier "528
`pub(in crate::runtime)` symbols" was a grep line count; the audit's item-name
scan counts 651 unique names — same order of magnitude, the design conclusion
(promote ~40, not 500+) is insensitive to the method.

## 2026-07-03 Crate-rebalance line OPENED (route 2a) — Stage 1 DONE: builtin dispatch is open

Line context: `rum-runtime` had grown into a 116,293-line / 391-file monolith
(evaluator + every builtin family) while `rum-stdlib` was a 21-line installer
facade; builtins were unmovable because dispatch was a CLOSED ~470-variant
`BuiltinHandler` enum + a three-file chained `match` + an exhaustive per-variant
capability `match`, and the families compile against a 528-symbol
`pub(in crate::runtime)` shared namespace via `use super::*` (377 files).
Canonical plan: `docs/crate-rebalance-plan.md` (d49e101) — Stage 1 open the
dispatch, Stage 2 define the kernel API, Stage 3 move the families; file-slimming
deliberately AFTER the split.

**Stage 1 (brief-097, commit 45d2e8a): delete the closed enum.** `BuiltinSpec`
now carries the handler as a plain fn pointer
(`pub type BuiltinFn = fn(&mut Session, &Expr, &[Expr]) -> Result<Expr, EvalError>`)
and the capability as data (`capability: Capability`, default `Pure`; 8 FileRead
+ 3 FileWrite annotated in the catalog). Deleted: the `BuiltinHandler` enum, the
`eval_dispatch/{surface,symbolic,control_functional}.rs` match chain (~530
lines), and the `impl BuiltinHandler::capability` match (~500 lines); net −1467
lines. The 513 catalog entries reference `Session::eval_*` directly (fn-item →
fn-pointer coercion in const arrays); ~25 non-standard arms (args-only
signatures + inline bodies) live VERBATIM as adapters in `builtin_adapters.rs`
— `Evaluate` keeps its `suspend_with_frame` suspension shape. `CallDispatchPlan`
carries the resolved `BuiltinSpec` (Copy, resolve-once-at-plan-build preserved);
capability inheritance follows handler inheritance in `with_fallback_dispatch`
(a re-registered `Get` cannot shed its `FileRead` requirement). The
fn-pointer-comparison lint is allowed locally at the type (all spec copies
originate from one catalog entry, so pointer equality is stable).

Gate evidence: `cargo fmt --check` + `cargo test --workspace` green (1297/0);
`grep -rn "BuiltinHandler" crates/` = 0 hits; microbench A/B run TWICE
independently (Codex once, Claude reproduced via a HEAD worktree). KEY
measurement lesson: the two A/Bs disagreed label-by-label IN BOTH DIRECTIONS
(labels reported −21% in one run showed +9–11% in the other, and vice versa) —
sub-ms dispatch-tier deltas are noise-dominated; judge only cross-run
consistent-direction movement, of which there was none. Verdict: match-jump →
indirect call is measurement-neutral; the extraction precondition (builtin =
data registered into the session) now holds.

**Stage 2 audit dispatched (brief-098, e251618):** diagnostic, no source change
— per-family compile-error-driven audit (comment out `use super::*`, collect
unresolved names, restore) over 14 family modules + a grep axis for kernel
`Session` methods called from family code. Output: the promotion-candidate list
for the explicit kernel API (≥3-family items → prelude; single-family items →
move with the family; unused internals → stay private) + the family→family
couplings that constrain move order.

## 2026-07-04 — Release transformation line STARTED (plan committed)

Decision (user, this session): drop the fresh-history release-staging approach;
transform `main` itself into release shape, keep full history. New testing policy:
rum tests only its own documented design + rum-vs-rum perf A/B; difftest and the
mma-hol snapshot pipeline are removed (archived privately at
../Archive/rum-history, MANIFEST there; WLBuiltin/ already moved). Support matrix
dropped in favor of a hand-written language guide + a builtin reference generated
from the stdlib catalog + rustdoc. Endpoint: clean tree + bundle cold backup (no
remote). Plan: docs/dev/release-transformation-plan.md (briefs 124-130 + M0-M2).

## 2026-07-04 — M1: narrative + policy rewrite (release transformation)

DISCLAIMER.md rewritten (user-reviewed): independent project, no compatibility
claims, factual history of the early black-box method and its July 2026
removal, standing policy. README repositioned (embeddable sandbox-first
term-rewriting language; external-comparison framing removed). AGENTS.md
rewritten around the new Testing Policy hard lines (own-documented-behavior
tests + rum-vs-rum perf A/B only; no oracle harnesses/compat matrices — 
permanent). testing-strategy.md rewritten (4 layers, coverage axes kept,
documentation-as-contract, llvm-cov guidance). TODO.md rewritten (release line
active items + post-cut queue: toolchain refresh to current stable, coverage
rebuild, scoped-fs profile, pyo3-on-rum-host, showcase, vm_rhs_template).
CLAUDE.md updated to the new world. Briefs 124-126 all gated green earlier
today (commits 5eddf1b, 131614a, 3814ef0).

## 2026-07-05 — M1.5: ARCHITECTURE + v0.1 boundary rewritten; toolchain settled

ARCHITECTURE.md: independent positioning, rum-stdlib section updated to the
post-rebalance reality, rum-difftest section removed, Compatibility Target →
Behavior Contract (documentation is the contract), Performance Kernel
Direction → Performance Discipline (rum-vs-rum A/B), Conformance Strategy
removed, core-language boundary now defined by rum's own guide/reference.
docs/v0.1-acceptance-boundary.md rewritten (acceptance = documented slice +
hardened rum-host + CLI/REPL + jupyter preview; external-corpus lanes gone).
Toolchain/MSRV SETTLED per user: stay on Rust 1.85 (Debian stable default
rustc + edition-2024 floor); recorded in TODO/README. brief-127 gated green
(e0448c9): crates wording scan, zero behavior change, suite totals identical.

## 2026-07-05 — briefs 128 + 129 gated green (reference + guide)

brief-128 (8503321): generated builtin reference — 513 entries from the
installed registry (attributes, capability class, summary, verified limits
notes) + drift test (negative check demonstrated); determinism reproduced at
the gate (double run, empty diff); minimal doc-hidden read-only iteration
APIs in rum-runtime. brief-129 (64b7c08): nine-chapter language guide, all
examples current-build transcripts, nine spot-reproduced at the gate; README
links guide + reference. Guide writing surfaced two real pre-release bugs
(now in TODO §Pre-Release Fixes): unbounded user recursion natively
overflows the stack instead of tripping recursion_limit; ReplaceRepeated
limit surfaces as a CLI hard error instead of TerminatedEvaluation.

## 2026-07-05 — brief-130 gated green; fs-expansion direction queued

brief-130 (2089d35): rum-host fully rustdoc'd under warn(missing_docs) (zero
warnings, doctest green; one grouped re-export split into per-item pub use —
semantically identical), crate-level overviews for the library crates,
workspace metadata (descriptions, license/keywords/categories inheritance, no
vendor terms, repository field deferred to publication), ISSUE file archived.
Workspace 1131/0. TODO gains the post-cut fs-surface expansion item (scoped
roots first). Next: brief-131 fixes the two guide-surfaced bugs before M2.

## 2026-07-06 — brief-131 + M2: the release transformation line is COMPLETE

brief-131 (cdeaa59): Phase A measured ~20 KiB native stack per WL recursion
level (overflow at ~depth 412 of limit 1024) → native_stack_limit guard
(default 6 MiB consumed) at the amortized poll point, recoverable
TerminatedEvaluation["StackLimit"]; ReplaceRepeated overflow now
TerminatedEvaluation["IterationLimit"]. Zero new unsafe; workspace 1136/0;
A/B no hot-label regression (median 1.018, reproduced at the gate). M2:
TEMPLATE.md rewritten to the new policy; briefs 124-131 + the plan doc
archived; final sweep = whitelist-exact (DISCLAIMER nominative, three
one-sentence family mentions, jupyter ecosystem identifiers); TODO/CLAUDE
updated; bundle cold backup follows this commit.

## 2026-07-06 — FS purity line STARTED

Decision (user): strip the six general file builtins (FileNames, FileExistsQ,
FileDate, FileByteCount, Import, FileTemplateApply) out of the repo entirely
(archived for a future downstream rum-fs project, post-launch); keep the
module system (Get/Needs/FindFile) and persistence (Save/DumpSave) with their
I/O injected via a host provider; kernel crates become std::fs-free by
construction with a hygiene test. Plan: docs/dev/fs-purity-plan.md
(briefs 132-133 + M0/M1).

## 2026-07-06 — FS purity line COMPLETE (briefs 132-134)

brief-132 (a46836e): the six general fs builtins stripped and archived as the
future downstream rum-fs seed; pure path-string helpers stay; reference 507.
brief-133 (06dbbb6): FileIo provider seam — Get/Needs/FindFile/Save/DumpSave
do I/O through a host-injected trait; StdFileIo lives in rum-host (absent
under sandboxed()); General::noio contract for permissive-policy-no-provider;
hygiene test forbids std::fs in kernel crates outside /tests/ paths.
brief-134 (c92b84f): Capability::Clock — audit rule "Pure iff observable
result depends only on session state + arguments" moved 8 clock readers out
of Pure (incl. Pause/EvaluationData/TestReport the audit caught);
MemoryConstrained stays Pure with evidence; BlockRandom removed as an inert
shell; sandboxed() is now deterministic by construction; docs adopt the two
claims (no ambient authority; deterministic by default). Reference 506;
workspace 1141/0. Process docs archived; kernel hard lines now three: zero
unsafe, zero std::fs (non-test), Pure=deterministic.

## 2026-07-06 — brief-135: scripting-ergonomics probe (diagnostic, no source change)

Six embedded-scripting tasks all completed in current rum (log summary, text
transform, quoted-CSV munging via a hand-written character parser, config
validation, template report, word frequency). Friction: no raw multiline
string literals; CSV needs a hand parser; reached-for-but-missing: Mean,
Tally, MaxBy/TakeLargestBy, ReverseSort, StringPadRight, StringLines.
String-pattern matrix: the engine supports ~~, alternatives, blanks, .., 
character classes, Except, Shortest/Longest, named binding across all six
string functions — almost none of it cataloged (dark surface); inert:
PunctuationCharacter, StartOfLine, EndOfLine, WordBoundary; StringSplit lacks
IgnoreCase; four bug-like edges recorded. Batches 136-140 planned in TODO.

## 2026-07-06 — brief-136 gated (12 builtins); codec/hash/rum-os planned

brief-136 (ba4ad60): Tally, MaxBy, MinBy, TakeLargestBy, TakeSmallestBy,
ReverseSort, ReverseSortBy, GatherBy, Riffle, exact Mean, StringPadRight,
StringLines. Codex stop-lossed correctly on the kernel attribute-vocabulary
registration (fenced out by the brief); completed at the gate, plus
attribute alignment to the family norm (ordinary, not Protected — the brief
was wrong). Reference 518; workspace 1151/0. TODO gains: codec family plan
(JSON cornerstone conventions: From*/To* pairs, fail-closed parsing; then
INI/TOML/XML/YAML-subset), hash/encoding family (RustCrypto + self-impl
hex/base64 + ByteArrayQ; hashing yes, encryption never), and the downstream
rum-os sketch (RunCommand design, Capability::Process reservation).

## 2026-07-06 — brief-137 gated: string-pattern language is now contract

Four inert atoms implemented, StringSplit IgnoreCase, zero-width anchor
semantics unified and documented ({pos, pos-1} spans), empty-match
consistency, Shortest/MatchQ fix; the whole pattern language incl. the
pre-existing undocumented RegularExpression subset cataloged in reference +
new guide chapter with real transcripts; matrix all-W. Workspace 1151/0.
Pipeline-citizenship brief planned as 139 (Random→140, date/time→141).

## 2026-07-07 — briefs 138-139 gated: JSON codec + pipeline citizenship

brief-138 (a067bda): FromJson/ToJson, zero deps, all numbers exact
(decimals→rationals), fail-closed discipline with offset-bearing messages,
guide chapter structured-data.md; reference 536. brief-139 (8c11068):
filter mode on non-TTY stdin, $ScriptInput verbatim data binding,
$ScriptArgs, stream discipline pinned by tests, quiet EPIPE exit; the
config-conversion shell one-liner works end to end. Workspace 1166/0.
Python-stdlib sweep candidates and the TokenizeWith/RumTokens mechanism
(highlighting for dun as rum-package grammars, no grammar zoo) recorded in
TODO. Next: brief-140 Random family.

## 2026-07-07 — brief-140 gated: Random family

xoshiro256**+splitmix64 as a public reproducibility contract (pinned
literal sequences; two CLI runs identical); PRNG builtins Pure; BlockRandom
back with a real backend; EntropySource seam (StdFileIo pattern) with
Capability::Random gating RandomBytes and zero-arg SeedRandom (in-handler
dual-form check); Capability::Process reserved. Workspace 1177/0. Perl-core
/CPAN-River scan recorded (4 net-new candidates + StringForm inertness
audit; ecosystem scans closed). Next: brief-141 date/time.

## 2026-07-07 — briefs 141-142 gated: date/time + hash/encoding

brief-141 (d672a79): exact rational Unix-second timestamps (epoch as rum's
contract), ISO 8601 fail-closed parsing with offset folding, exact calendar
arithmetic, Clock dual-form readers; 1/3-second round-trip pinned. brief-142
prep (d8a957f): sha2/sha1/md-5/crc32fast pinned + cached (first external
deps under the security/format clause). brief-142 (98da945): digests with
KAT pins, self-implemented hex/base64, ByteArrayQ/ByteArrayToString;
hashing-yes-encryption-never boundary in the guide. Reference 563; workspace
1196/0. Next: brief-143 (Python/Perl tier-1 utilities + StringForm
contract), then the TokenizeWith mechanism batch closes the expansion line.

## 2026-07-07 — STDLIB EXPANSION LINE COMPLETE (briefs 135-144)

From the brief-135 diagnostic to brief-144, the scripting stdlib went
506 → 584 builtins, every addition contract-documented (reference drift
enforced) and transcript-verified: collection odds-and-ends, the
string-pattern contract (dark surface cataloged, four atoms implemented,
edges fixed), FromJson/ToJson (fail-closed cornerstone), rum-cli pipeline
citizenship ($ScriptInput/$ScriptArgs/EPIPE), the deterministic Random
family with the entropy capability, exact date/time, digests + hex/base64,
the 19-builtin utilities batch with the StringForm contract, and
TokenizeWith/RumTokens (highlighting mechanism; grammars stay packages).
Workspace 1212/0. Process briefs archived. Parked: multiline string
literals (parser-level), schema helpers, ByteArray depth, compression
codecs, unicode normalization, ipaddress. Next mainline: user's call
(coverage rebuild is the standing debt).

## 2026-07-07 — CODEC FAMILY LINE STARTED

User picked the codec continuation as the next mainline. Plan committed
(docs/dev/codec-family-plan.md): 145 CSV+INI, 146 TOML (datetimes on the
brief-141 exact timestamps), 147 XML (XmlElement symbolic tree), 148 YAML
sane-subset (aggressively fail-closed; full YAML a permanent non-goal),
M1 conversion-matrix closure.

## 2026-07-07 — CODEC FAMILY LINE COMPLETE (briefs 145-148 + M1)

FromCsv/ToCsv + FromIni/ToIni (d2ab264), FromToml/ToToml (aea144d,
offset-datetime folding onto exact timestamps; encode never synthesizes
datetime syntax), FromXml/ToXml (3a22c5e, XmlElement inert head, no-DTD
security stance), FromYaml/ToYaml (4d49c0d, block-only sane subset,
Norway-safe typing, safe-plain encode quoting). M1 (411d684): six-test
cross-codec matrix incl. the JSON→TOML→YAML→JSON grand-tour identity.
Six formats, one value model, all fail-closed with byte offsets, zero new
dependencies. Reference 595; workspace 1247/0. The config-conversion story
is complete: cat a.json | rum-cli -e 'ToYaml[FromJson[$ScriptInput]]'.

## 2026-07-07 — HARDENING COVERAGE LINE STARTED (A-half)

User decision: the coverage-rebuild debt splits — the infrastructure half
(host lifecycle, limit paths, stress corpus) runs NOW as briefs 149-151;
stdlib-breadth coverage is deferred until stdlib growth plateaus (batch
discipline already covers new code). Plan: docs/dev/hardening-coverage-plan.md.

## 2026-07-07 — brief-149 baseline: the net is better than feared

Workspace region 79.00% / line 82.30%. ALL 12 TerminatedEvaluation tags
have direct exercising tests (the batch discipline held). Real holes:
error-mapping tables (SessionRequestError / HostFailureCode), snapshot/
panic lifecycle edges, iterator-spec parsing, the memory-proxy path, and —
highest risk — the suspension continuation serializers at 0% (never-run
deserializers). Contract note: MemoryConstraintExceeded deliberately has no
TerminatedEvaluation mapping (MemoryConstrained consumes it) — to be made
explicit in docs at M1. brief-150 (test-only buildout) dispatched.

## 2026-07-07 — brief-151: the stress corpus earned its keep immediately

Corpus landed (guard-adjacent pins incl. exact MAX_PARSE_DEPTH boundary,
exact-result rewriting workloads, cancel/timeout under load, battered
session, snapshot at scale; 1289/0, 9 ignored) and DISCOVERED TWO real
bugs: a legal 50k-element list pipeline and 100-500-definition
snapshot/restore both die by native stack overflow — kernel-internal
recursion that never reaches the eval-loop poll point where the
native_stack_limit check lives. brief-152 (diagnose + extend the guard)
dispatched.

## 2026-07-07 — HARDENING COVERAGE LINE (A-half) COMPLETE (briefs 149-152)

149 baseline: region 79.00%/line 82.30%; all 12 TerminatedEvaluation tags
already exercised; verified hole list. 150: test-only buildout (+31),
error-mapping tables and the 0% suspension serializers covered (unit +
real suspend/snapshot/resume), focus files reproduced at gate (outcome.rs
21→97.7%, suspension 0→83-92%). 151: stress corpus (+11) — and it caught
two real bugs: per-element native recursion in Select/DecorateBy resume
helpers (died at 2668 elements) and restore_from_bytes evaluating the whole
snapshot as one nested expression (overflow at ~75 defs; silent definition
loss on big main-thread restores). 152: loops for normal progression
(continuations only on InputRequired), restore through the script/form
splitter; bug stubs are real tests (50k pipeline = exact 25000;
500-definition snapshot round-trip); A/B in the noise band. Workspace
1291/0. MemoryConstrained consume-contract now explicit in the guide.

## 2026-07-07 — A-group + B1 + B2 complete (briefs 153-156)

153: downstream builtin-installer hook (reset/clone survival, capability
dispatch on downstream builtins — the rum-fs/rum-os/dun registration seam)
+ DecimalString + guide touch-ups; rust-toolchain.toml pinned at 1.85.0.
154: adoption-cost numbers measured and documented (session 0.09 ms steady,
eval 4.5 us, 6.5 MiB stripped delta, 23 external crates). 155: schema
validation — patterns are the type language (SchemaErrors data + 
ConformSchema defaults/strict-extras; FromYaml→ConformSchema pipeline).
156: multiline string literals \"\"\"/''' (TOML-mirror), formatter-verbatim
round-trip, ' derivative pinned both sides, is_complete aware. Workspace
1315/0; reference 598. Process briefs archived.

## 2026-07-07 — random family + character-class constants complete (157-159)

157 RandomString/RandomPassword, 158 password policy options, 159 named
Protected character-class constants (UppercaseLetters/LowercaseLetters/
DigitCharacters/PunctuationCharacters/Alphanumeric — sharing brief-137's
punctuation definition; new install_system_own_value kernel hook installs
them as system ownvalues excluded from DumpSave/snapshot). Password policy
usage reads Require -> {UppercaseLetters, DigitCharacters}; constants are
Protected so reassignment fails (Set::wrsym). Reference 605; workspace
1339/0. Briefs 153-159 archived.

## 2026-07-14 — HOST-EMBEDDING LINE OPENED: diagnosis + b1 (brief-231)

Line opened per the user's sequencing ruling (diagnosis → concern (b)
dun pure-sandbox/protocol → concern (a) general embedding API,
scenario-first). Diagnosis landed in docs/dev/host-embedding-line.md
(0ff7d15): pure-sandbox gates 1-3 already done (sandboxed profile,
Capability::Interactive, panic isolation); stdout-zero-writes,
crash-only, and cold start (~4.8 ms full process cycle, 50-run amortized
×3) satisfied but unpinned; the two real seams = structured termination
discriminator + request-level memory ceiling. brief-231 (74afadf) closed
the first seam: EvalTerminationReason (9 reasons, .tag() preserves the
exact legacy strings) threaded finalize → EvalOutcome /
SessionEvalResponse → HostEvalSuccess.termination
(HostTerminationReason mirror). In-band TerminatedEvaluation[...]
values/messages byte-identical; user-constructed literals stay
termination=None (provenance). Gate: fmt clean, workspace 1478/0,
scope exact (12 files), no A/B (finalize runs once per top-level form).

## 2026-07-15 — host line b2: request-level memory ceiling (brief-232)

ResourceLimits.eval_memory_limit (default None) armed per top-level
request at the wall-clock-deadline seam; uncaught breach → in-band
TerminatedEvaluation["MemoryLimit"] + General::memlim + b1's
termination=Some(MemoryLimit). A top-level-only reason helper keeps the
shared reason map untouched, so EvaluationData/TestReport memory
renderings stay byte-identical; MemoryConstrained frames min-combine and
still catch in-band. Active-mode enforcement strided (full proxy walk
every 64 armed checks; projected-byte prechecks unstrided).

Two process notes. (1) The FIRST dispatch of 232 was a wasted round and
MY fault: adding a public field to ResourceLimits forces the five
exhaustive literals in runtime/tests/session_config.rs, which was missing
from the MAY list — Codex correctly stopped at the scope stop-loss with a
clean tree. Lesson recorded: grep the mechanical ripple (new public field
→ every exhaustive literal; new enum variant → every exhaustive match)
BEFORE writing a MAY list. (2) The gate's microbench hung twice at the
run-bounded timeout — instrument failure, not a code defect: rum-cli
binds non-TTY stdin as $ScriptInput, so a benchmark invocation without
`< /dev/null` blocks forever. Confirmed both directions (no stdin →
timeout; /dev/null → 0.33 s). Same family as the "feed probes stdin"
lesson from the stdlib line.

Gate: fmt clean, workspace 1481/0, scope exact (12 files). Armed workload
base 3.825 s → cand 0.667 s (−82.6%, interleaved best-of-3) keeps the
stride. Default-path A/B against a CALIBRATED noise floor (base-vs-base,
same binary: median 2.0%, p90 7.4%, max 10.4%): candidate median −3.4%,
worst label +6.0%, ZERO labels above the floor → no regression.

## 2026-07-15 — host line b3: stdout ownership hard line (brief-233)

The kernel already wrote zero bytes to stdout; this brief turned that
from discipline into a mechanical hygiene test (same family as the
std::fs ban): println!/print!/io::stdout/stdout() forbidden in
rum-core/rum-runtime/rum-stdlib library code, excluding /tests/ and
/bin/ (codegen tools own their stdout). Print stays buffered per request
into `prints`; rum-cli's stdout sink is the host's explicit, legitimate
choice (it owns its stdout), and rum-host exposes no sink at all — so an
embedded session's Print can never reach the host process's stdout,
which is exactly what a framed-protocol host needs.

Instrument trap worth remembering: "eprintln!" CONTAINS the substring
"println!", so a naive contains() check flags all 72 legitimate
env-gated profiling lines (12 files). The scan is identifier-boundary
aware. The hygiene test was self-calibrated at the gate — an injected
println! makes it FAIL and name the file; the director reproduced this
independently in a different file (rum-stdlib/src/lib.rs) before
crediting it. A hygiene test that cannot fail is worthless.

Free property from b2, now tested: `prints` is counted by the memory
proxy, so Do[Print["x"], {100000}] under a small request ceiling
terminates with TerminatedEvaluation["MemoryLimit"] + General::memlim
instead of growing unbounded.

Gate: fmt clean, workspace 1483/0, scope exact (4 files). No A/B — tests
and docs only, no evaluation-path code.

## 2026-07-15 — host line b4: isolation + crash-only contract (brief-234)

reset()=build_inner_session already rebuilds the whole Session from
config (dropping every runtime dimension, keeping config-carried state),
so per-request isolation and crash-only were held by construction but
under-tested — existing coverage touched a single ownvalue. Three
lifecycle contract tests now pin: (1) reset clears own/down values,
attributes, options, messages, contexts, the gensym counter, and PRNG
advancement, compared against a fresh reference session (not hardcoded);
(2) reset-between-requests — no definition leak, and the seeded
RandomInteger stream replays identically (isolation reaches the entropy
stream, which is what lets a host pass explicit state as data); (3) an
abandoned poisoned session leaves a freshly built one unaffected (no
process-global mutable state). embedding.md documents the pattern and
the crash-only guarantee. Probe facts confirmed at write time (not
guessed): empty Options render List[]; bare SetOptions needs Options[h]
pre-declared (SetOptions::optnf); Module in Custom` renders Custom`t$1,
rewound to t$1 after reset.

Honest scope call: the OS-level spawn+SIGKILL+no-temp-files integration
test is deferred to the b5 prototype, where a real host process exists —
SIGKILL-safety is already structural (no temp files, enforced by the
b3 std::fs hygiene test), so a heavyweight flaky spawn test here would
add little. Gate: fmt clean, workspace 1486/0, scope exact (2 files),
no A/B (tests + docs only).

b-line status: b1-b4 DONE (termination discriminator, request memory
ceiling, stdout hard line, isolation/crash-only contract). Remaining:
b5 — the rum protocol host prototype (syntax-highlight role) validated
against dun's hosts/check-host.py.

## 2026-07-16 — Embedding line (a) OPENED: scenario design + consumer survey

Concern (a) of the host-embedding line opened per the sequencing ruling
(scenario design first, no bricklaying on instinct). Landed
docs/dev/embedding-api-line.md as the line's canonical doc: three
user-supplied abstract scenario families (Python/AI-infra adapter,
Rust/wasm tooling, tauri typst IDE — dun's ancestor) + a survey of the
four in-tree consumers with findings F1-F8 (rum-cli smuggles host data
through the input-suspension machinery; HostTyped has zero consumers;
the b1 termination discriminator is unadopted — rum-jupyter still
string-matches TerminatedEvaluation["Cancelled"]; HostFailureCode::Exit
drops the exit status; streaming prints and script/file eval exist only
below the contract). USER RULINGS: dogfood = stance A (in-tree trusted
hosts port onto rum-host; rum-cli first with a microbench A/B hard
gate); brief queue signed off as a1 contract-honesty → a2 trusted-host
surface + dogfood port → a3 typed bridge out → a4 typed bridge in →
a5 named profiles. Also recorded (Q&A): fs/os/net future attach
architecture needs no new invention (register_builtin/installer +
capability gate + provider seams), with three design prerequisites —
the Capability u8 bitmask is FULL (a Network variant forces a widening),
blocking syscalls do not observe cancel, stateful handles vs
snapshot/reset.

## 2026-07-16 — brief-236: host failure-classification honesty (embedding line a1)

Three contract-honesty fixes, host-layer only (zero kernel changes):
HostFailure.exit_status carries the Exit[code] status (was dropped at
the outcome.rs mapping — a contract host could not implement Exit[7]);
HostFailureCode::requires_reset() publishes the reset classification
(poison truth verified from session.rs: only the guard's caught-panic
path sets poisoned, so exactly Panic|Poisoned require reset) and
rum-jupyter's hand-rolled copy now delegates to it; rum-jupyter adopts
brief-231's HostEvalSuccess.termination field, killing the
TerminatedEvaluation["Cancelled"] string-match — deliberate behavior
fix: a program whose ordinary result VALUE is that literal is no longer
misclassified as a cancelled failure (provenance-tested field), pinned
by a new adapter test. Gate: scope exact (7 files, all MAY-listed),
fmt clean, workspace 1488/0 reproduced, key claims reproduced by name
(Exit[7] -> Some(7), reset truth table, literal-value success). No A/B:
failure-path metadata + adapter classification only, no eval hot path.

## 2026-07-16 — brief-237: trusted-host contract surface (embedding line a2a)

Four additive HostSession methods lifting existing Session capabilities
to the embedding contract (zero kernel changes; every method = guarded
forward + existing outcome mapping): set_print_sink/clear_print_sink
(EXCLUSIVE streaming — prints buffer stays empty while a sink is
installed; sink does not survive reset, documented alongside the
cancel-handle invalidation), eval_script (multi-form, last-form value +
termination, aggregated messages/prints), eval_file (reads through the
FileIo provider; provider-less/sandboxed fails as Other), and
queue_input_response (the intended batch-input channel, replacing
rum-cli's input-suspension smuggling). Status quo pinned, not changed:
script suspension surfaces as InputRequired but is NOT resumable —
resume_with_input fails with Other and remaining forms are not
evaluated (a documented seam for a future consumer-driven brief).
Gate: scope exact (3 files), fmt clean, workspace 1494/0 reproduced
(+6 contract tests, each rerun by name). No A/B: additive host facade
only, no kernel code touched. Next: a2b (rum-cli dogfood port,
microbench A/B hard gate).

## 2026-07-16 — brief-238: rum-cli dogfood port (embedding line a2b)

rum-cli's non-interactive paths now ride the rum-host contract,
byte-preserving (27 frozen CLI tests, zero expectation changes): eval /
eval_script / eval_file via HostSession, streaming prints via the
contract sink (EPIPE -> cancel_handle), batch input via
queue_input_response (the argv/stdin smuggling is gone), exit status
via HostFailure.exit_status, message/failure rendering rebuilt
byte-identically from HostMessage/HostFailure. --parse/--format go
straight to rum-syntax (no session construction at all). The REPL
branch keeps a raw Session until a2c (marked in source). The port
exposed and fixed two contract gaps: HostValue.text is now
CanonicalHoldFormTransparent (canonical style with one top-level
HoldForm peeled — the CLI's rendering; jupyter text/plain inherits the
WL-correct display), and HostFailure.syntax_rendered carries the full
rendered diagnostic (Syntax failures only). Process note: the first
dispatch stop-lossed on a brief error (I wrote the expected rendering
as infix "1 + 1" from memory; the canonical truth is "Plus[1, 1]" —
brief revised, expectation values must be probed, not recalled). The
second dispatch stop-lossed on the Part C A/B gate reporting 5
regressed labels; the director's independent 4-pair INTERLEAVED re-run
overturned it: all 5 suspects cleared (best-of-3: literal-string -4.5%,
set-ownvalue -2.8%, block-local -2.1%, symbol-ownvalue-read +0.2%,
downvalue-pattern-hit +0.9%), median delta 0.00%, and with-local's
apparent +14% was label bistability (the same candidate binary observed
at 0.85ms and 0.97ms levels across sessions; pair 4: base 0.000863 vs
cand 0.000865). Lesson pinned: A/B rounds must be interleaved (B C B C),
never sequential blocks — a brief allowing baseline reuse invites
time-drift artifacts. Gate: scope exact (7 files), fmt clean, workspace
1495/0 reproduced, raw A/B data under /tmp/rum-238-gate/.

## 2026-07-16 — brief-239: rum-repl dogfood port (embedding line a2c) — a2 arc CLOSED

rum-repl rides the contract: main loop on is_complete (three-state:
Complete/Invalid submit to eval — syntax failures render via
syntax_rendered — Incomplete accumulates), commands routed per the user
rulings (:lower → new HostSession::lower_dump, diagnostic-grade with an
explicit no-stability-promise rustdoc; :messages → new
HostSession::message_catalog reusing HostMessageTemplate — the
MessageCatalogEntry fields map one-for-one, zero new types;
:parse/:format straight to rum-syntax; :complete/:inspect on the
existing contract). rum-terminal became the contract-type render-twin
crate: render_host_message(s)/failure/eval_response/inspect/
message_catalog, byte-identical (frozen REPL/CLI tests are the oracle);
all 7 runtime-typed renderers deleted with grep-verified zero
consumers. rum-cli's dual session construction unified away. Probed
facts: attribute_count/message_count equal len() (no truncation, no
field additions needed). Dependency payoff: rum-cli and rum-repl no
longer depend on rum-runtime or rum-stdlib AT ALL; rum-terminal depends
only on rum-host — the in-tree hosts are pure contract consumers, which
was the point of the dogfood ruling. Cargo.lock updated for the shrunk
dependency graph (Codex correctly left it out of its allowlisted tree;
the director folded the ripple into the commit). Gate: scope exact (11
files + lock), fmt clean, workspace 1498/0 reproduced, new tests rerun
by name. No A/B: REPL is not the perf vehicle; --eval/script paths
untouched. The a2 dogfood arc (a2a surface, a2b CLI, a2c REPL) is
CLOSED. Next: a3 (typed value bridge, out).

## 2026-07-16 — brief-240: typed value bridge, out (embedding line a3)

HostTyped widened with three documented variants: Bool (the system
symbols True/False now surface as Bool rather than Symbol — deliberate
convenience mapping, no in-tree consumer depended on the old shape),
Association (ordered key/value pairs via the kernel_api association
helpers — no hand-rolled shape detection), and Rule (immediate,
two-argument). Honesty rule pinned by tests: an association containing
a delayed entry and RuleDelayed both stay Other — a typed data view
must not present an unevaluated right-hand side as a value. Depth-cap
discipline uniform across all structured variants. First real consumer
lands with it (killing the F2 zero-consumer status): jupyter execute
results now carry a conservative application/json MIME entry iff the
typed tree is fully representable (Integer/Str/Bool/List/string-keyed
deduplicated Association; BigInteger/Rational/Symbol/Rule/Other or a
depth-capped tail reject the whole tree), with the three existing MIME
entries byte-unchanged (user-approved wire addition 2026-07-16). Gate:
scope exact (6 files), fmt clean, workspace 1506/0 reproduced (+10
tests, all rerun by name), diff reviewed (delayed check precedes entry
conversion; exhaustive match in json_from_typed lists every rejected
variant). No A/B: typed conversion runs once per top-level outcome
mapping; microbench in-process loops never reach it. Next: a4 (typed
value bridge, in) — the line's last core item, then a5 (named
profiles).

## 2026-07-16 — brief-241: typed value bridge, in (embedding line a4)

The input half of the typed bridge: expr_from_typed converts HostTyped
into kernel expressions (depth guard shares MAX_TYPED_DEPTH=64 — trees
under the cap sit far below the documented deep-drop danger, no Drop
machinery touched; Other rejected as an output-only lossy view;
BigInteger/Rational strings validated, zero denominators refused,
rationals normalize so 2/4 round-trips as 1/2) and
HostSession::call(head, args) is the injection-safe invocation path:
head and Symbol arguments resolve through the session's public
resolve_symbol_name — short names hit System builtins exactly as
source text would, fully-qualified names pin both contexts — and a
hostile string argument ("], Print[666], x = " …) round-trips as pure
data (pinned). One kernel seam, authorized by the line's ground rules:
Session::eval_expr_request evaluates an already-built Expr through the
full request protocol (checkpoint/messages/termination), implemented
by extracting eval_request's post-lower tail into a shared helper;
eval_request behavior frozen and verified. A/B story: Codex ran the
newly-mandated interleaved procedure and correctly stop-lossed on 3
labels exceeding their per-label base spreads — but those spreads
(1-11µs, three-sample max-min) were razor-thin artifacts; the
director's independent interleaved re-run measured the same labels'
true cross-round spreads at 5-15% with deltas of +0.6~0.9%, median
delta 0.00%, bidirectional scatter (fastest label -3.8%), two
independent interleaved sessions agreeing. Verdict: no regression;
lesson refined — a FAIL verdict needs the label to clear the WHOLE
FIELD's |delta| distribution, not a three-sample per-label min-max.
Gate: scope exact (6 files), fmt clean, workspace 1519/0 reproduced
(+13 tests, call suite rerun by name; Codex had stopped before the
full suite per its stop-loss — the director completed it). Next: a5
(named profiles), the line's last queued item.

## 2026-07-16 — brief-242: named profiles (embedding line a5) — LINE COMPLETE

hermetic() and scripting() land as named contracts (user-ruled minimal
set): hermetic shares sandboxed()'s implementation but carries its own
determinism promise (same input → same output; fixed-seed PRNG replays
— the stateful↔pure line's dividend: RandomInteger is Pure, only true
entropy is Capability::Random, so the sandbox was already mechanically
hermetic); scripting() names default()'s trusted-local semantics and
rum-cli adopts it (redundant provider hand-wiring gone; the director
removed the leftover unused imports post-gate). The drift-proofing is
the real deliverable: each named profile is pinned by an
all-8-capability probe test with an exhaustive Capability match — a
ninth variant (e.g. a future Network capability, which also forces the
u8 bitmask widening) fails compilation until every profile takes an
explicit stance. Gate: scope exact (4 files + the import cleanup), fmt
clean, workspace 1523/0 reproduced, all 4 new tests rerun by name. No
A/B: config constructors only. One watch item: a rum-jupyter native
ZeroMQ ordering flake surfaced once in Codex's first workspace attempt
(passed in isolation and in two full reruns; also seen once around
brief-165 — if it recurs, brief it).

**EMBEDDING API LINE (concern a) COMPLETE — briefs 236-242, opened and
closed 2026-07-16.** Scenario design (three user-supplied abandoned
downstream concepts) + in-tree consumer survey (F1-F8) drove five
gated increments. Structural endstate: rum-cli and rum-repl are pure
rum-host consumers (zero rum-runtime/rum-stdlib dependencies),
rum-terminal renders contract types only, HostTyped is a bidirectional
injection-safe bridge with a real wire consumer, and the contract
carries the full trusted-host surface plus three capability-pinned
named profiles. The dogfood ruling (stance A) did the work its
rationale promised: every gap was pulled by a real consumer, none
invented.

## 2026-07-17 — briefs 243-253: cleanup + message-registry line — LINE COMPLETE

One day, eight gated briefs, all four queue items closed (user ruling
2026-07-17 opened the line; the same ruling closed the parked pools
WON'T-DO). (1) The message-tag registry arc: inventory (brief-243: 114
push_message sites, 144 tags, 966 closed pairs, live probes proving
Messages[Set] empty), mechanism (245: BuiltinSpec.messages + the
four-surface merge — user definitions win, output shapes frozen), fill
sweeps (246-248: kernel 304 + stdlib 433 + 214 pairs, reconciled
exactly against the inventory; one ledger row proven unreachable at the
source), docgen pairing (249: the reference renders Messages blocks
from the registry — 951 rows, additions-only diff), carriers (252, user
ruling: General 12 pairs + UpTo 2 as inert Protected specs; reference
615 entries; the first dispatch's stop-loss caught an unverified
Take[..., UpTo[...]] expectation — Take/Drop UpTo support recorded as a
WL-conformance follow-up candidate), and enforcement (253: a
debug-assertions check at the push_message seam, declared-or-exempt
rule shaped by two correct stop-loss rounds; zero registry gaps at
landing). (2) vm_rhs_template DELETED (244): the opt-in experiment's ON
path crashed the guard-adjacent recursion test; removal A/B clean.
(3) rum-jupyter signature.rs ported onto Hmac<Sha256> (250) — the
repository's last hand-rolled crypto gone; the ZeroMQ ordering flake
did not recur. (4) rum-cli feature forwarding (251): a four-crate
chain makes the CLI digest-free under --no-default-features with
defaults byte-equivalent. Tooling: the dispatch sandbox now defaults
to workspace-write (danger-full-access reserved, by env override, for
sample-profiling briefs — the original 2026-06-21 reason); the one
observed friction is loopback-bind denial on rum-jupyter's transport
tests, covered by the out-of-sandbox authoritative gate. Cold backup
rum-2026-07-17-dc9181a.bundle verified. Next: the wl15/wfr design
pass (ACTIVE PLAN item 5).

## 2026-07-17 — wl15/wfr pass batch 1: Take/Drop/TakeDrop UpTo (brief-254)

The wl15/wfr design pass (ACTIVE PLAN item 5) opened with the mechanical
batch: `UpTo[n]`/`UpTo[Infinity]` sequence specs for Take/Drop/TakeDrop
(clamp-to-length, any spec position, multi-level free via per-spec
parsing). Malformed contents reuse the brief-252-registered UpTo::innf/
argx carriers. Gate correction: the innf trigger was narrowed to
definitely-invalid contents (negative integer or rational) to mirror the
kernel Part-span rule — symbolic (`UpTo[x]`) and oversize integers stay
on the silent inert path. Workspace 1523/0; probes reproduced at the
gate. Commits cd81665 (brief) + 5a2ef4a (impl). Batches 2 (hold design)
and 3 (structured exceptions) proceed design-first.

## 2026-07-18 — wl15/wfr pass batch 2 (hold half) COMPLETE bar 2c

Batch 2 landed the SubValuesHoldAll line: 2a mechanism (brief-255,
8186536 — new attribute bit 1<<15, compound-head args prepared as
HoldAll via the existing base-symbol walk, symbol-head calls
untouched); 2b-I mutation operator forms (brief-256, acaf1f3 — nine
compound-assignment heads AddTo/AppendTo/ApplyTo/AssociateTo/DivideBy/
KeyDropFrom/PrependTo/SubtractFrom/TimesBy adopt the attribute + a
mutation wrapper family; unary captures quiet-inert; two brief
revisions on file ownership); 2b-II schema operator forms (brief-257,
476cf19 — ConformSchema/SchemaErrors switch to special_with_hold, raw
schema captured, value evaluation handler-owned, byte-parity with the
direct form incl. its pre-existing validation-time pattern resolution).
Workspace 1543/0. All three A/B gates clean (no same-direction hot-label
regression). Remaining: 2c ReplaceAllUnheld (brief-258). Gate lessons
this batch: file ownership must be opened-and-verified not name-grepped
(brief-256 two stop-losses); equivalence contracts pin A==B side-by-side
not "both have property P" (brief-257 one stop-loss). Both recorded in
[[brief-may-list-must-cover-mechanical-ripple]].

## 2026-07-18 — wl15/wfr batch 2 CLOSED: ReplaceAllUnheld (brief-258)

2c landed as a separate builtin (ruling C1, 48e909f): ReplaceAllUnheld
evaluates its target first (mirror ReplaceAll), then traverses the
evaluated tree with a structural descent mask that skips held argument
positions. The mask reads the ORIGINAL pre-rewrite head attributes and
honors compound-head SubValuesHoldAll (brief-255 consistency);
ReplaceAll's own path is byte-unchanged (the mask branch short-circuits
on the source discriminant). Operator form via one line in
rewrite_operator_symbol. Workspace 1547/0; A/B 3-pair interleaved with
replaceall-simple as the hard veto — -2.53% (faster), tier flat-to-
better, inside the control noise floor. Codex correctly stop-lossed on
a missing stdlib_attributes.rs vocabulary-mirror line (new-builtin
ripple I omitted from MAY); it left the complete correct implementation
in the tree, so the director added that one mechanical line and ran the
full gate rather than waste a re-dispatch. Lesson recorded in
[[brief-may-list-must-cover-mechanical-ripple]]. BATCH 2 (the hold half)
now fully closed: 2a SubValuesHoldAll + 2b-I mutation forms + 2b-II
schema forms + 2c ReplaceAllUnheld. Remaining in the wl15/wfr pass:
batch 3 structured exceptions, pending user green light on E1-E6.

## 2026-07-18 — wl15/wfr batch 3 CLOSED + THE LINE COMPLETE: structured exceptions (brief-259)

3a core landed (42a2476): a folk structured-exception system with
observably-separate channels from classic Throw/Catch. A new
EvalError::ExceptionThrown variant carries the Exception[type, payload]
object; Catch's match arms never mention it, so the type system makes
cross-catching impossible (director sized this via cargo check: exactly
one non-wildcard EvalError match breaks, the top-level mapper). Five
heads register in stdlib and dispatch to kernel eval via
ControlDataBuiltin (Exception inert; ThrowException/CatchExceptions/
RegisterExceptionType/ExceptionTypes active). String type hierarchy in
a session-field registry mirroring message_definitions (snapshot
round-trip covered). Ancestry dispatch = handler list-order priority ×
nearest-ancestor, suspension-safe via the same matcher Catch uses;
no-match re-raises. E4 uncaught → in-band Failure["UncaughtException",
<|Type,Payload|>] at finalize_top_level_eval_value + a nocatch message.
E5 unregistered-throw fails closed to a root exception. E6: WithCleanup
runs cleanup on unwind (free via its wildcard path), Enclose passes
exceptions through. Workspace 1554/0; reference 621. Two brief revisions,
both correct Codex stop-losses on my brief omissions (stdlib file
ownership for the control family; the kernel-eval/stdlib-registration
ControlDataBuiltin split). 3b = the evaluation.md §Structured Exceptions
guide section (director-written, examples measured on the release CLI).

**THE wl15/wfr DESIGN PASS IS COMPLETE** (ACTIVE PLAN item 5, briefs
254-259): batch 1 Take/Drop UpTo; batch 2 the hold half (SubValuesHoldAll
+ mutation/schema operator forms + ReplaceAllUnheld); batch 3 structured
exceptions. The cleanup + message-registry line (item 5 was its last
open item) is now fully closed.

## 2026-07-18 — catalog file-size splits (briefs 260-261)

Cleared the split-plan-range (20-35KB) catalog files left by the
message-registry fill. brief-260 (e48d12f): kernel builtin_catalog/
namespace.rs (26.0KB) → namespace/{variables,contexts,symbols}.rs
(1.4/10.4/14.4KB), Vec aggregator concatenates in order. brief-261
(fdf2d6a): stdlib strings.rs (27.5KB) → strings/strings_text/strings_match
(STRING_CORE/TEXT/MATCH), structure.rs (25.4KB) → structure/
structure_ordering, associations.rs (21.0KB) → associations/
associations_transform; sibling files preserve super::message_templates
imports; seven sub-consts registered across catalog/mod.rs + lib.rs +
tests.rs in preserved family order. Both briefs: pure reorganization,
reference byte-identical (the master gate — docgen reads registered
specs and sorts by name), workspace 1554/0, all resulting files under
15KB. rules.rs (17KB) is warning-range only, left as-is. Remaining
over-20KB: stdlib_attributes.rs (28KB) — a flat alphabetically-sorted
attribute mirror with a binary_search_by consumer, so splitting is a
lookup-mechanism change with no family boundary; surfaced to the user
as split-with-adaptation vs. accept-as-debt.

## 2026-07-20 — Sealed scoping line (d) COMPLETE (briefs 268-271)

The lazy + sealed-scoping ACTIVE PLAN is fully closed. (d) landed in four
steps: brief-268 grounding (REFUTED the design's original Module-gensym
substrate — Module locals are name-mangled and source-reachable; the real
seal = fresh bit-62 identities kept OUT of symbol_generations, resolver
byte-untouched), brief-269 mechanism spine (DefinePackage three-tier
registration, suspension-safe install, 12 reachability tests, reference
637), brief-270 hidden-by-default introspection (one predicate + one
Diagnostic-gated reveal across Names/Contexts/Definition/Information/
catalogs/completion/Save-DumpSave; 17 dual-variant tests), brief-271
snapshot-by-re-registration + re-link (RestoreSealedReferences restore-mode
marker, orphan refusal, fallible snapshot_to_bytes, byte-parity for
sealed-free sessions; reference 638). Close-out: guide §Sealed Packages,
README divergence section (user ruling 2026-07-19: the dual dividend —
real encapsulation + persistence in one mode, dissolving the strict/stable
split — is publication-narrative material). Workspace 1630/0. Gate
corrections this line: brief-271's undeclared process-global identity
allocator reverted to session-scoped. A/B method note: the microsecond
list-tier labels proved layout-lottery-sensitive (±60%, bidirectional,
closed by brief-271's reversal of brief-269's movements).

## 2026-07-20 — Row-workload pure line (DB-prep) COMPLETE (briefs 272-274)

User ruling opened and closed the line in one day: pure builtins only,
preparing the "table = List of Associations" workload for future
downstream database applications. Audit (F1-F5) then three gated briefs:
272 JoinAcross (hash-based Inner/Left/Right/Outer joins, key spec =
key | Key | rule | list; right-wins collisions; unmatched rows bare;
reference 638->639); 273 SortBy multi-key tie-break form (literal-List
spec -> evaluated key tuple, lexicographic + stable) + MaximalBy/
MinimalBy (reference 639->641; gate fix: ASCII-sort violation in the
attribute-mirror insert, no sortedness assertion exists - debt note);
274 kernel named-slot fill from Association arguments (Lookup-
consistent Missing on absent key; microbench A/B noise-floor clean).
Endstate: workspace 1644/0, reference 641, guide syntax.md documents
named-slot fill. Not pursued: Query head (Dataset subsystem, out of
folk scope); parser wrinkle "#1 &[x]" without parens parked.

## 2026-07-20 → 07-21 — matcher respec, debt clearing, REPL/trace (briefs 275-288)

Endstate: workspace 1675/0, reference 642, zero files over the 35k
architecture-debt line.

- **Matcher OneIdentity/Optional + Flat respec (275-279).** WK-derived
  `#[ignore]` tests retired as policy fossils and re-pinned to rum's own
  measured behavior; non-`OneIdentity` `Flat` singleton wrapped candidates
  implemented, closing a real gap (`MatchQ[g[1,2], g[a_g, b_]]` was False
  though `g[g[1], 2]` flattens to `g[1,2]`). The `optional.rs` collect-all
  omission arm was first-match-shaped and under-collected; fixing it is not
  `OneIdentity`-gated, so built-in `Plus`/`Times` recovered dropped rows.
  2^n candidate growth accepted per user ruling (no pruning).
- **Debt clearing (280-281).** `stdlib_attributes.rs` split along its ASCII
  name ordering into four first-byte chunks, lookup still const-time;
  `rum-host/src/tests.rs` (68,639 bytes) split by responsibility into six
  files, proven a pure move by name-set equality.
- **REPL/trace (282-288).** REPL rendering frozen with full-output snapshots
  (`contains` 48 → 0); `:trace`; `TraceEvent` switched from `String` to
  `Expr`, deleting a render→parse round trip and cutting tracing overhead
  from 5.5-5.8× to 2.4×; `TraceDetail` (lossless, names which overlapping
  definition fired, key `RuleLeft`); `:trace --rules`.
- Re-probed and deliberately not actioned: symbolic `Plus`/`Times` like-term
  detection confirmed superlinear on distinct terms from roughly n ≥ 400,
  but the entry's own revisit trigger (a real symbolic-algebra workload)
  is still unmet.

## 2026-07-29/30 — incremental container update + positional Part assignment (briefs 291-300)

Two arcs, both opened by design-only briefs under the plan-first workflow.
Origin was a rum-hol feasibility question about log-replay cost, which led to
measuring `assoc[k] = v` and finding it O(n^3).

plan-291 (briefs 292-296): microbench labels; SharedArgs COW + cache-proof
primitives; flat AppendTo fast path; Part spine; Association direct
canonical-Rule mutation. AppendTo O(n^2)->O(n) (6.4x), Part spine 13.2x,
Association 25.1->0.43 ms (57.9x, shared_expr_alloc 20322->222). Step 6
(bounded key index) deliberately not done.

plan-297 (briefs 298-300): the diagnosis brief found that `lst[[i]] = v` never
worked -- it left the list unchanged and emitted a false "Set::wrsym: Symbol
Part is Protected". Fixed as a feature (88625df), measured (885ec0f), then made
independent of container width (60c4ab4; length quotient 2.87 -> 0.995).

Durable lessons, all learned the hard way in this arc:

- Counters (allocation counts) are immune to machine load; timings are not. A
  video call invalidated an entire timing pass while every "did the mechanism
  fire" conclusion still stood. Since brief-294 the rule is: report counters
  first, and if they do not move, do not quote a time.
- This suite has a per-run whole-run offset moving all labels together by a few
  percent. Two runs of the identical script read -4.07% with 7/38 up, so a
  lopsided direction split is not evidence. Alternating order does not cancel
  it.
- Never normalize against the suite median in a lane of successive
  optimizations: it drifts cumulatively (a sentinel read +21.9% across four
  steps purely because optimized labels dragged the median down). Use named
  untouched labels, or within-run quotients.
- Cross-session label comparison is unreliable even with paired ratios: the
  same comparison read -4.96% across sessions and -0.16% within one.
- Every performance probe needs an assertion that the work actually happened.
  The `lst[[i]] = v` numbers measured a failing rebuild-then-reject path for
  several rounds. The probe contained a calibration point that was never read,
  and the scaling check did not catch it because the failing path is genuinely
  O(m) -- the number scaled correctly while measuring the wrong thing.
- Plan-first paid off immediately: Codex's design brief refuted three of five
  findings on 291 (the call-stability cache as hidden third owner; "only
  ApplyTo runs user code" false for all eleven compound assignments;
  Association O(n^3) not O(n^2)), all confirmed at the gate, and on 297 found
  the feature gap that had been mis-diagnosed as a performance problem.

## 2026-08-22 → 08-23 — post-v0.1 safe-Rust algorithmic performance sweep

The first repository-wide audit for avoidable O(n^2)-and-worse work after
`v0.1.0` landed as 53 commits after the prior documentation checkpoint
`2c8b824`: 50 performance changes, two correctness fixes found while tracing
cost shapes, and one test-environment fix. `rum-core` and `rum-runtime` remain
zero-`unsafe`.

The accepted changes fall into six groups:

- resumable predicate, collection, traversal, tensor, fold, sort, equivalence,
  and grouping loops now keep cursors/accumulators instead of rebuilding or
  recursively cloning continuation state;
- wide collections, associations, schemas, arithmetic groups, definition
  candidates, codec dispatch, and exact replacement rules use adaptive indexes
  while retaining allocation-light small-input tiers;
- polynomial transforms batch canonicalization and use sparse/indexed term,
  variable, factor-multiset, and containment machinery; this optimizes the
  existing polynomial kernel but deliberately does not expand `Factor`'s
  documented algorithmic scope;
- `ReplacePart`, `ReplaceAll`, batched association selection, DownValue/UpValue
  construction and lookup, traversal, and `Module` escape analysis no longer
  repeat suffix copies or linear closure scans;
- string matching/search, reversible suffix matching, entity scanning, wide
  sequence edits, `Drop`, and bounded `Nearest` selection were linearized or
  indexed where the previous wide path rescanned prefixes/suffixes;
- the final P0 tranche landed as `c73df5c` (traversal suffix state), `97873ee`
  (`Module` escape closure), and `d97abd7` (wide UpValue symbols).

Two correctness defects surfaced during the audit rather than being hidden by
performance patches: recursive operator parsing is now depth-bounded
(`28afe35`), and uninitialized `Module` bindings are processed iteratively
(`28b321c`). The nine native ZeroMQ tests now skip when loopback access is
unavailable (`69fde1f`), so ordinary sandboxed workspace tests no longer need
an out-of-sandbox rerun merely to classify that environment.

Correction to the 2026-07-20/21 entry above: its statement that symbolic
`Plus`/`Times` like-term grouping remained an accepted quadratic tradeoff is
historical, not current. Commit `48838b3` added an adaptive group index to all
four additive/multiplicative grouping paths. Inputs stay linear-scan below the
threshold; wide inputs build a `HashMap` after 16 distinct groups when enough
work remains. The old unconditional O(n^2) description has therefore been
removed from `TODO.md`.

Timing discipline was also re-confirmed. Elapsed measurements come from
`AbsoluteTiming`, whose implementation uses Rust's monotonic `Instant`; wall
clock differencing is invalid on a host whose civil clock can slew or step.
Competing CPU load can still invalidate timings even though it cannot make the
monotonic clock run backward. Counters remain useful for proving that a
mechanism fired, but do not substitute for an uncontended timing gate.

Three P1 items remain: the ordered all-`Single` matcher cursor; a shared
`RewriteRulePlan` plus negative-level-only depth computation for `Replace`; and
the bounded `RumTokens` prefix, TOML table-key, and `Reap` tag-bucket scans.
An ordered-matcher candidate has been prepared but deliberately not accepted or
committed while another CPU-heavy workload makes its timing gate unreliable.
`Factor` expansion is separately deferred because a complete
factorizer needs finite-field polynomial machinery, not another local linear-
factor optimization.

## 2026-08-23 — P1 ordered matcher cursor

Commit `d93c859` replaced repeated remaining-vector rebuilds in the ordered,
non-`Flat`, multi-argument matcher with owned pattern/value cursors. The cursor
continues across all-`Single` prefixes, falls back to the existing generic
sequence/optional machinery on the first variable-arity argument, and is
preserved by typed suspension continuations so resumed conditions do not replay
already matched prefixes. The implementation remains safe Rust.

The acceptance gate used independent release builds from the committed base
and candidate source trees. Timing came only from the benchmark program's
`AbsoluteTiming`/Rust `Instant`; the suite's wall clock was used only for its
process timeout record. With loop-style labels scaled 3x and B/C each run five
times, median candidate changes were -17.06% for four arguments, -89.59% for
800 arguments, and -94.74% for 1600 arguments. Same-tier `MatchQ`,
`ReplaceAll`, `Cases`, `Count`, and `Position` sentinels ranged from -1.61% to
-0.18%, so none regressed.

The allocator gate was collected in the normal sandbox with the new
`core-memory-microbench.sh --no-rss` mode because macOS `/usr/bin/time -l`
tries a denied `sysctl` even after its child succeeds. After subtracting the
fresh-process baseline, allocation counts changed only -0.15%/-0.08% at
800/1600 arguments, while allocated bytes fell 8.63% at both widths and peak
live allocator bytes fell 8.27%/8.21%. Rum's expression-memory proxy remained
360 bytes on both sides because it does not account for these Rust temporary
vectors. This is a time-and-space improvement, not a space-for-time tradeoff.

Correctness gates passed `cargo test -p rum-runtime` (674/674) and
`cargo test --workspace` (all non-ignored tests; three existing stdlib stress
tests remained ignored). The remaining P1 order is now: shared `Replace`
planning/depth work, then the bounded `RumTokens`/TOML/`Reap` wide-input scans.

## 2026-08-24 — second algorithmic sweep, crash class, and two cliffs

The post-v0.1 audit continued through briefs 301-321. Beyond the complexity
reductions it set out to make, it closed two defect classes the original survey
had not named, and one of those was found only because a probe was pushed past
the size the plan had specified.

**Native stack exhaustion on wide input was a shipped bug, not a regression.**
`Replace[Range[10000], z -> 0]` and `Cases[Range[7000], _Integer]` aborted the
process; `origin/main` aborted identically. The walkers recursed once per
element from inside a `drive_internal_value` continuation, and the native-stack
guard could not see it: the guard polls in `evaluate`, keyed on
`evaluation_depth`, which these continuation chains never increment. Raising a
limit was therefore not the fix and adding a poll would only have converted a
crash into a refusal — a 50 000-element list is a reasonable input. Thirteen
builtins were converted to explicit loops carrying a `pending` outcome so a
suspension resumes at the same element: `Replace`, `ReplaceAll`, `Cases`,
`Count`, `Position`, `DeleteCases`, `ReplaceList`, `Reap` rendering, the
`UpValue`/`SubValue` dispatchers, `StringReplace`, `StringCases` and
`StringApply`. The boundary is now `GeneratedElementLimit`, which refuses
gracefully.

**Two cliffs, both invisible until a probe crossed them.**

Incremental association writes cost 1.5 s at 16 000 keys and 55 s at 20 000 —
1.25x the work for 36x the time. Every direct write stability-checked a freshly
built `Rule`, inserting a cache entry for an expression that died immediately,
so crossing the 16384 entry cap wiped the whole cache including the proof the
direct path needs; the slow path never re-seeded it, disabling the fast path for
the rest of the session. The table in the brief that opened this work stopped at
16 000 — precisely below the cliff — and it surfaced only because the worker
measured past the specified range.

Installing many UpValues on one symbol took 38.6 s at 20 000 definitions. The
diagnosis refuted all three suspected causes (index rebuild on non-terminal
insert, specificity recomputation in the comparator, metadata prune) and found
the trigger was punctuation: each line ending in `;` lowers to a
`CompoundExpression` wrapper, ordinary calls enter UpValue dispatch before
builtin dispatch, and the wrapper's arguments contain the tag nested inside the
`TagSetDelayed`. A 128-line probe showed `candidates=8128` — C(128,2) — with a
prefilter skip rate of 1000 per mille: every candidate rejected, the whole scan
wasted. The same definitions took 2.54 s with trailing semicolons and 0.03 s
without.

**Measured outcomes.** Script loading 7.75 s to 0.01 s at 10 000 lines, plus one
parse per form instead of two. `ToToml` 6.9 s to 0.03 s at 8000 entries;
`FromToml` 0.289 s to 0.014 s at 16 000 keys. `StringReplace` and `StringCases`
2.7 s and 3.4 s to 0.017 s and 0.018 s at 8000 strings. Association writes
675 s to 0.08 s at 32 000 keys, across the cliff fix and the write index.
`KeyTake`/`KeyDrop`/`KeyDropFrom` roughly 20x at 8000 keys. `ReplacePart`
0.0175 s to 0.0032 s at 2000 rules, and 20 000 rules from a crash to 0.014 s.
`Together` 0.53 s to 0.077 s at 40 addends. UpValue installation 38.6 s to
0.15 s. `Reap` collection 0.129 s to 0.014 s at 8000 tags. Workspace tests went
from 1862 to 1921, always green.

**Method notes worth keeping.**

- *A brief's own measurement range can hide the defect.* The association table
  stopped one step short of the cliff. Probe past the specified range.
- *Three green probes do not establish a universal claim.* brief-305's design
  rested on "balance open implies an incomplete parse error"; the spot-checks
  passed only because juxtaposition rescues adjacent operands, and real grammar
  conflicts die before EOF. The brief was withdrawn intact — the working tree
  was byte-identical — and replaced by a parser-side design whose equivalence
  follows from prefix determinism instead of sampling.
- *Counters settle what stopwatches cannot.* Whether a write mutates in place is
  a yes/no question with a load-immune answer: 65,840,485 allocations became
  20,020, about one per write. A probe also demonstrated the ownership trap
  directly — a weak witness alive across `make_mut` forces a copy, dropping it
  first keeps the allocation.
- *A survey's structural reading needs a probe before it becomes a plan.*
  `DownValue` dispatch was listed with `UpValue`/`SubValue` on structure alone
  and measured fine. `StringCases` was cited as a linear in-tree model, but that
  measurement used a literal pattern routed elsewhere; its rule form had the
  same defect. Conversely `KeyDropFrom`, named without a probe, was confirmed.
- *Semantics-preserving is not the same as merging.* `ReplacePart` rules apply
  sequentially, each to the result of the last, so merging targets collected
  against the original tree changes results silently. `Together`'s obvious
  deferred-normalization fix would have made identical denominators slower;
  pairwise merging kept the per-step normalization and only changed operand
  sizes.

Two items from the original P1 list remain and are recorded in `TODO.md`:
`RumTokens` offset conversion and the shared replacement plan for `Replace`.

## 2026-08-26 — third sweep: crashes, chain lowering, and callback ordering

Two independent re-surveys produced about thirty items, worked in batches. Six
batches ran; one turned out to be already fixed. Every batch below was gated by
reproducing its evidence directly rather than accepting the report's numbers.

**Closed.** Loop and `Module`-initializer drivers made iterative — a
20 000-iteration `While` had terminated the process, which is a correctness
defect before it is a performance one. YAML and TOML decoders bounded to match
`FromJson`, so all three accept 256 nested collections and refuse the 257th
with a byte offset. Left-associative chain lowering rebuilt over five steps:
all sixteen affected operators overflowed the stack at 20 000 links, and they
now fold iteratively, nine into n-ary calls, under a 4 096-link ceiling proved
on a 2 MiB embedding stack through parse, lower, format and drop. Resource
budgets made symmetric across families. User-callback ordering moved onto one
resumable merge engine: `Sort` and friends were quadratic on every input, and
`KeySort` overflowed the stack at 110 keys.

**The instrument lied four times, and that is the durable part of this entry.**

1. *The microbench never closed stdin.* `run-bounded.py` launches what it is
   handed, and rum-cli binds non-TTY stdin as `$ScriptInput` when given a
   program argument, so an inherited pipe that never reaches EOF left it
   waiting until the timeout. A run therefore either finished in four seconds
   or burned its whole timeout and produced a timings file containing only a
   header — with no mention of stdin, so it read as a loaded machine. A worker
   spent forty minutes polling `uptime` looking for a quiet window that would
   not have helped, and every A/B before `dd4bdefc` was exposed to this.
2. *A probe measured the wrong builtin form.* rum's custom equivalence test is
   a positional argument, not a `SameTest` option. `DeleteDuplicates[list,
   SameTest -> f]` passes a `Rule` where a function belongs, so every
   comparison fails and nothing is deduplicated — which is why an all-identical
   list appeared to cost the same as an all-distinct one, and why "SameTest
   never exits early" reached a brief as a finding. With the supported syntax
   the early exit is obvious: 399 predicate calls versus 79 800 at n=400.
3. *A trace probe closed over the wrong scope.* `(trace = Append[trace, #]; #
   &)` binds `&` to the final `#` only, so the assignment ran once, outside the
   function, and the trace held a bare `Slot`. It would have supported a
   confident claim that key evaluation count had changed.
4. *A survey finding was a fossil.* The "Association 4-to-5" and "65 536-key"
   cliffs did not exist any more; `c8d34876` and `dc5f6698` had closed them
   earlier in the same sweep. Eight operations measured flat.

The common root is that none of those probes carried an assertion that the work
they claimed to measure had happened. A call count, a result length, or a
`Depth` check would have caught all four immediately. That requirement is now
written into the briefs.

**A regression I introduced, and what it teaches about gates.** Charging nested
iterator counts as a running product was right for `Table[Table[…]]` and wrong
for `Do[Table[…], {200}]`: a `Do` discards each value, so 200 sequential rounds
of 1000 elements are not 200 000 simultaneous ones. The propagation rule is now
whether the enclosing construct *retains* what the inner one produces. The gate
that missed it had tested that separate statements do not share an accumulator
— the same question asked only from the bypass side. Asking it from the
false-refusal side was a matter of enumerating enclosing-by-inner construct
combinations, which is now how these are gated.

**Two workers disagreeing with me was worth more than either agreeing.** The
plan for the chain work refuted three of my premises, including that `:` cannot
be flattened because its lowering is stateful per link — a detail I had not
seen. The plan for the ordering work refuted my `SameTest` finding outright and
found the `KeySort` crash next to it. A worker stopping clean on a `Scope`
omission — the merge engine sits behind a private `mod`, and the MAY list named
the engine file but not the module declaring it — cost one redispatch and no
wrong code.

**Deferred with evidence:** parentheses are transparent to comparison-chain
collection, so `1 == (1 == 1)` is True while `1 == True` stays unevaluated; the
rhs half is fixable, the lhs half shares one AST with the unparenthesized form.
Product propagation is decided from construct nesting rather than data flow, so
`Table[Length[Table[…]]]` is refused conservatively. Both are recorded in place
— the first as an annotated golden row — because a defect written into a test
without comment becomes an expectation the next reader will defend.

## 2026-08-27 — batch 7 re-probe, and the two defects it uncovered

Batch 7 of the post-`v0.1.0` sweep was supposed to be six performance items.
Re-probing them first — the rule this project adopted after two earlier
findings turned out to be already fixed — is what made the batch worth
running, because two of the three most valuable results came from probes
written to measure something else.

**The batch itself, all six re-probed and closed.** `ValueQ` at 8 000
definitions went from 332.6 us to 0.76 us per call and, more importantly,
stopped scaling with session history at all. Association `Part` reads went
from 96 101 allocations to 14, independent of width; positional reads are
exactly flat across width, keyed reads sublinear. `StringTrim` with a literal
list got the fast path its single-literal sibling already had, 31.6x and
36.1x. Symbol ordering stopped allocating two `String`s per comparison with
byte-identical output. The REPL runs one full completeness check per form
instead of one per line. The Jupyter kernel sleeps instead of spinning a core,
and the gensym sweep rearms at a watermark so per-batch cost stops climbing —
+106% by the third batch before, -11% after.

**`Do[f[i] = i, {i, 1, 3}]` never worked.** It installed a single
`HoldPattern[f[i]] :> 3` and said nothing. A clean iterator symbol is bound in
a dynamic own-value frame; ordinary evaluation resolves definitions and then
that frame, but the held-lhs normalizer stopped after definitions — invisible
exactly where the key is built. The decisive experiment was pre-valuing the
symbol: that pushes the iterator onto its fallback branch, which installs a
real definition, and the same code then produces the right keys. The fix is
the normalizer taking the same third step in the same order.

**Right-hand-side `Condition` guards were not implemented anywhere.**
`f[x_] := rhs /; test` stored the whole `Condition` as the body and never
looked at it, so `b[x_] := x^2 /; x > 0` made `b[3]` return `9 /; True` and
`b[-3]` return `9 /; False`. Dedup compared left-hand sides only, so a rule
differing solely by its guard replaced the unguarded one and vanished. Eight
candidate-selecting lanes were affected. `Cases` was the sharpest: it
collected elements whose guard had declined — a wrong answer, not a leaked
wrapper. Repaired lane by lane with one substrate and one accept rule, with a
decline made indistinguishable from a pattern mismatch to every consumer,
including `ReplaceRepeated`'s fixed-point test and `ReplaceList`'s enumeration.

**The lesson that cost the most.** After six green gates on that line, a
guarded `DownValue` still stopped enforcing its guard the moment anything
bumped the attribute revision — any `SetAttributes` in the session, or a
loader-snapshot restore such as `ValueQ`. The reconstruction written in the
first step was correct but unreachable: on a stale revision the fast path
rematched from the lhs alone, succeeded for a simple pattern, and returned
before reaching it. Every gate, and the whole-line regression test, had
asserted the guard *before* the trigger and never after one, so none of them
could see it. The repair put the barrier in the single rematch API rather than
patching a second call site, and the regression test now carries an
attribute-bump dimension. For anything that caches or compiles state derived
from a definition, the test has to act after the invalidation.

**Two instrument failures, both caught by their own guards.** A benchmark row
added for the gensym work emitted `Do::argform` and measured a failing path
until a liveness guard was added asserting the batch actually reaped its
values. And a stop-loss fired on three control labels that had exceeded
floors computed from two baseline runs; three runs each way put the real
same-binary spread at 12.6-26.7% and the movements at -1.72%, -0.01% and
-16.16%. Two runs give one random difference, not a floor.

**Opened, not closed.** Gensyms held only by an opaque continuation frame are
not treated as live, and it is reachable two ways. The `rum-jupyter` ZeroMQ
ordering flake reached its third appearance. Both are recorded in `TODO.md`
with what a fix would have to decide.

## 2026-08-28 — the suspension-frame roots line (briefs 364-375)

The defect `TODO.md` opened after batch 7 is closed. A gensym reachable only
from a suspension frame could be swept while still needed, because the root
walk stopped at `active_stack` and the frames themselves were
`Box<dyn SuspensionFrame>` wrapping arbitrary closures — state Rust cannot
enumerate. The repair replaced that with named frames whose traversal is
derived from their fields, and made forgetting it a compile error rather than
a thing to remember.

**Four defects, not two.** The design brief's re-probe found a matcher binding
table (`SymbolExprMap`) losing its bindings, and a `Block` frame holding a
bare `Symbol` that *leaked* on the following sweep — failing in the opposite
direction. That fourth one settled a design question: a visitor covering only
`Expr` would have fixed the three requested probes and left the leak in place,
so `Symbol` is a root in its own right. All four are verified closed against
plan-364's original probes, which assert the old buggy values and now fail by
asserting the defect.

**Enforcement, not vigilance.** `SuspensionFrame: FrameRootTrace` makes a
missing traversal fail compilation; the frame queue is private so there is one
insertion route; a repository invariant test bans the deleted APIs and any
handwritten traversal outside a sorted two-entry allowlist. Both allowlist
entries pass a two-part test argued per type: a derive is structurally
impossible (`std`, `rum-syntax`), and the type cannot name `Expr` at all
because its crate cannot reach `rum-core`. The honest limit is recorded too —
Rust cannot prove an impl came from a derive across crates, so the allowlist
is the boundary, accepted by user ruling on 2026-08-27 rather than paying for
absolute sealing by moving every stdlib frame into `rum-runtime` and undoing
the crate-rebalance line.

**It paid for itself on the ordinary path.** A closure captures at
construction, before the `Ok`/`InputRequired` split, so every call built a
suspension capture it usually threw away: a width-2003 `Switch` paid 63.8 KiB.
Measured by differential against a lazy-capturing control, that width slope is
now -128 bytes of constant noise. `INTO_VEC_CLONE` could not see the copy at
all — the builtin boundary had already degraded `SharedArgs` to a slice — so
the allocation counter was the applicable instrument.

**Six stop-losses, five of them mine.** A list treated as scope authority on a
line whose purpose is to replace lists with compiler enforcement; an absolute
prohibition with no test for its first legitimate exception; a
stop-on-second-candidate rule whose test was mechanically checkable all along;
and twice, reaching for step 374's frame lifetime before it existed — once in
the cutover's `self.as_ref().clone()`, once in carriers assuming
`try_unwrap` uniqueness while a rooted frame still held an alias. The one
genuinely technical stop was the worker's: a suspension-only merge
representation changed observable comparator progression, so the sort family
waited for the cutover.

**Instrument notes.** A scan for remaining escape hatches reported zero
everywhere including inside `suspension/core.rs`, where eight were known to
remain; the regex had broken on the comma in `Result<Expr, EvalError>`.
Recalibrated against that known-eight control, it then confirmed zero
elsewhere. A scanner reporting a clean result has to be shown finding
something first. Separately, a probe that "reproduced" nothing turned out to
be an `Option<Expr>` field left as `None`, which visits nothing — a probe
needs a value that can actually be seen.

**A/B read in both directions.** Step 372's labels all improved and step 373's
all regressed, and both were reported as no regression: the untouched controls
moved as far or further each time. Step 373 had the stronger evidence — the
sort family had zero diff and still moved 7%, and code that was not touched
cannot have been affected even indirectly.

Endstate: workspace 2163/0, zero build warnings, no production file over 35k,
and a first occurrence of a call-stability-proof flake recorded in `TODO.md`
without adopting the worker's explanation for it.

## 2026-08-28 — the assignment-target normalization line (briefs 376-383)

Opened from three items `TODO.md` had recorded as batch-7 leftovers. A
re-probe found one of the four already fixed four days earlier and never
retired from the record, and the design brief then found three more defects of
the same family. The line closed eight defects in total across two repair
families.

**Family A (brief-377)** — thirteen mutation heads did not evaluate a `Part`
selector, so `kk = 1; lst[[kk]] += 5` reported `Part::pkspec1` and changed
nothing while `lst[[kk]] = 99` worked. A typed `EvaluatedPartSelectors` is now
what assignment-target parsers accept, obtainable only by exactly-once
evaluation or a stable-literal proof, so a future head cannot pass raw
selectors without failing to compile.

**Family B (briefs 379-383)** — seven defects, one invariant: the identity a
definition is stored under must come from the same attribute-aware preparation
as the identity a lookup constructs. Four failed to prepare, two prepared a
position that had to stay raw, and one needed the `Flat`/`Orderless`
normalization the ground index was not applying. Storage and removal now
travel as tokens whose constructors are private to one module.

**What the line is worth beyond the fixes.**

- *The bidirectional invariant was the thing.* Four defects failed to
  normalize and two over-normalized. Fixing only the common direction would
  have closed three and hidden the fourth — the shape the frame-roots line hit
  with `Block`'s bare `Symbol`. The user's decision to check WL behavior
  before ruling is what exposed the second direction.
- *A conclusion whose evidence is our own defect is not evidence.* plan-376
  recorded "raw rules become reachable after `Clear`" as resolved-with-evidence
  and reasoned from it. The behavior was real but was a by-product of the raw
  store itself; WL has no such effect. The whole preserve-raw option went with
  it.
- *Gate instruments must be calibrated before they are believed.* Every
  differential gate in this line was first shown to report a diff against a
  pre-family-A binary. The sealing test was calibrated the same way — one
  inserted call makes it fail with the exact count — and the compile-time
  boundary was checked by trying to forge a token (E0603/E0624).
- *Measurement lessons, three of them, each caught the hard way.* Best-of-2
  reports a spread four to five times too small: 1c left two labels open at
  +6.34%/+6.72% against 2.01%/2.96% spreads that are 10.6%/11.3% at
  best-of-3. Whether valid control labels exist depends on the change's
  reach — 1a sat on the common call path where nothing is untouched, 1b sat
  on the assignment path where 28 pure-computation labels decided the gate by
  moving *more* than the affected ones. And when a label looks like a
  regression but the disassembly is unchanged, a third binary carrying only an
  inert dead function settles it: in step 2 that arm moved the labels further
  than the real change did.
- *The same coincidence appeared twice, one step apart.* After 1b made `Set`
  store raw held forms, `Unset` on a held position "worked" because two
  unprepared expressions happened to be equal; after 1c, `TagUnset` did the
  same. Both had to be migrated in the step that would otherwise break them.

**Left open, deliberately.** Two guarded lookup labels moved +7.47% and +6.01%
in step 3, above the 0.4-4.1% spread measured between same-semantics binaries
differing only in layout, while the three hot lookup functions' disassembly is
instruction-identical with addresses shifted `+0xa0`. The layout estimate is
incomplete in a known direction — the inert arms add code where the change
removes 232 lines net. Recorded rather than tuned blind.

## 2026-08-31 — three long-standing items, then a consistency review (briefs 384-396)

`TODO.md` had carried three items for months. All three closed in a day, and
each turned on the same thing: a test asserting a property the implementation
never promised.

**The `rum-jupyter` ZeroMQ flake (brief-384)** had been seen three times since
brief-165, always green in isolation. It reproduces on demand in **release**
mode — 22 of 30 runs — and is nearly invisible in debug, which is what the test
gate runs. The test sent a shutdown on control, slept, sent an execute on
shell, and asserted the *next* event was control; the poller's priority is
correct but selects among channels already pending, and native transport
buffers on background workers. The repair waits until both channels report
pending through the same predicate the poller uses.

**Syntax error locations (brief-385)** were computed per form and rendered
against the whole file. A four-line script failing at 4:7 reported 2:1 — the
position of an unrelated character — and `Get` reported no location at all. The
suite could not see it: every pinned location was `1:x`, where form and file
coordinates coincide.

**Comparison chains (brief-386).** `TODO.md` recorded this as half-fixable
because `(a == b) == c` and `a == b == c` "share one AST". Both halves were
wrong — `a == (b == c)` flattened too, and the shared AST was a consequence of
parsing these operators as associative. The user's diagnosis was the right one:
they have no associativity. Chains are collected while parsing, and mixed
chains lower to `Inequality`, whose absence of a hold attribute is why each
operand evaluates once and nothing short-circuits.

**The call-stability flake (briefs 387-388)** turned out to be all three
recorded sightings of one cause: a 128-bit Bloom taking its bit from a
process-global interner id. E1 made it deterministic — 36 preinterned symbols
fail the direct test every time, 142 the indexed one — which was worth more
than the statistics that preceded it. The predicate is one-sided by design, so
the tests were at fault, and the repair pairs an identity-bearing target with
regular-lane dependencies so the lanes cannot intersect for any interner order.

**Then a four-part consistency review of briefs 364-388 (389-392), repaired by
393-396.** Eight findings, three high. Every repair replaced a convention with
a structure: the list of protected pattern wrappers became a registry shared
with the matcher; assertions resting on interner order moved to a
lane-disjoint construction; a public installer that trusted its caller now
demands a typed left-hand side; a wrapper claiming two constructors left the
continuation protocol that offered a third.

**What the review was worth, beyond the findings.**

- *Two findings were corrections to the briefs themselves.* The frame-root
  allowlist is an inventory of handwritten traversals, not a two-entry
  exemption list, and the frame queue is the single insertion route for
  ordinary code rather than literally. `CLAUDE.md` was wrong on both.
- *One was a gate miss.* brief-388 repaired the two assertions that had been
  seen failing; five more rested on the same mechanism, because the diagnosis
  and the repair both examined the recorded sightings, and the recorded
  sightings were two. **Sweep the class, not the instances.**
- *A boundary can hold inside the repository and evaporate at the crate edge.*
  Two findings were exactly this, and in both cases the test meant to police it
  scanned in-repository callers — **a source scan cannot establish a property
  about code that does not exist yet.** The gate that settled them compiled an
  external crate and required the compiler errors.
- *The worker's honesty was worth more than several of the findings.* It
  reported that native tests skipped in its sandbox and that its green rows
  were compilation evidence rather than stability evidence; that two assertions
  could not be made to fail and so were reported rather than changed on
  suspicion; and that a brief's own background was wrong. Where its evidence
  could not exist, the verification became the director's: forty release
  repetitions on a host where those tests actually run.

## 2026-09-02 — the suspension and cancellation line (briefs 397-411)

The project moved from macOS to Linux, and the first release-profile full-test
run on the new host produced two `rum-jupyter` failures the debug gate had
never shown. Fifteen briefs later, three kernel defects and a class of invalid
test assumptions are closed. Endstate: workspace 2287/0/7, the native ZeroMQ
suite 0 failures in 30 repetitions on a host where it demonstrably runs.

**What was actually wrong.**

- *Cancellation was erased by the evaluation it was meant to stop.* The
  prologue cleared a session-global flag as `eval_request`'s first statement,
  and the jupyter adapter takes its handle before spawning — so the handle was
  obtained at the right moment, the cancel was set at the right moment, and the
  evaluation threw it away. Now request-scoped and generation-qualified.
- *Abandoning a suspension leaked every scope its frames owned* — `Block`,
  `Module` activations and gensym reclamation, iterator scopes, `OptionValue`,
  `Reap`, `Monitor`, constraint deadlines, trace and capture checkpoints,
  `AbortProtect` depth, `BlockRandom`, partial package installs. The leaked
  `AbortProtect` depth let evaluation continue past `Abort[]`. Reachable
  through public `rum-host` API alone.
- *Overlapping public requests silently replaced a pending continuation.* Now
  rejected, by user ruling; supersede stays unimplemented rather than being an
  accident.
- *The jupyter test suite asserted timing properties the implementation does
  not promise*, in three unrelated families plus an ephemeral-port TOCTOU.

**The lessons worth keeping.**

- *An instrument reported nine skips as nine passes.* A native ZeroMQ test that
  cannot bind loopback printed one line to stderr and early-returned, and
  libtest said `ok`. This had already cost the project a false 0-of-30
  stability result in brief-384. It is now a red test unless a named
  environment variable opts in. **Silence had been an option; it no longer is.**
- *A worker's green gate is worth exactly what it can execute.* brief-409
  passed its own gate and broke all nine native tests on the director's host,
  because the panic it introduced was masked by the same skip path. The repair
  was one line; the durable fix was socket-free coverage of the harness, which
  paid off two briefs later when family B needed a gate the worker could run.
- *Refuting a repair is worth more than performing one.* brief-401 killed
  brief-400's proposed mechanism — driving saved frames with `Err(Cancelled)`
  is not an unwind, because ordinary frames consume errors and run work.
  Splitting that defect out was a hypothesis at the time; had it been wrong,
  the repair would have gone back inside brief-400.
- *Measure the mechanism, not the outcome.* A probe that varied only what
  happens between dispatch and the first completion poll showed cold, warm,
  worker-warm and yield all reading not-ready and only a sleep reading ready.
  That is what forbade a spin as the fix, before anyone wrote one.
- *After families A and D there was nothing failing*, and C and B still had to
  be done. A test that passes because a machine happened to be fast is
  supplying false confidence, so "it still passes" stopped being evidence and
  the gates became structural.
- *Waiting cannot prove absence.* Two tests waited 200 ms, saw no reply, and
  concluded none was produced. They now read state that says so.
- *My own probes lied three times in one session* — a `git stash show`
  invocation that reported a conflict for every file, a `grep` for
  `test result: ok` that is always true when several test binaries run, and a
  bare test name passed to `--exact` that matched nothing and was counted as
  failure. Same shape each time: trusting a predicate without first checking it
  against an answer already known. The verification harness needs calibrating
  as much as the code does.

## 2026-09-03 — the memory-accounting and traversal-depth lines (briefs 412-427)

Opened by paying an owed verification and closed nothing; two lines are part
built and both are recorded in `TODO.md` rather than here.

**The first security audit this project has had** (brief-413, GLM, no scope
narrowing, because Codex's cybersecurity filter makes such work impossible
there). Seven findings, two HIGH, both the same root: bytes the memory model
cannot see. The process-global symbol interner leaks by construction and is
invisible to `eval_memory_limit`; the ceiling is sampled every 64 evaluation
points and can be overshot multiplicatively in between. Its three criticisms of
the threat model were worth more than several of the findings, particularly
that no document says which state is per-request and which is per-session.

**The memory-accounting line (415-420).** Reviewing the design turned up a
third gap the audit missed: `current_memory_proxy_bytes` walked 34 of
`Session`'s 93 fields, so the baseline every ceiling decision measures against
was itself short. A macro now generates both the checked-in classification and
an exhaustive `let Session { .. }` pattern, so a new field stops compiling
until it is classified — verified by adding one and watching the compiler
refuse, after a first probe that only tripped the constructor and proved
nothing.

Then `MemoryGrowthContract` became a required argument to all nine
`BuiltinSpec` constructors across 709 construction sites, and — the part that
mattered — a falsifier that fails a declaration the handler contradicts. It
caught eleven wrong declarations immediately, `Rule` returning 64,289 bytes
from 424. brief-418 then found the falsifier armed on a production memory
ceiling, where it cost 2.2x and could panic, and unarmed in every crate but
`rum-runtime`, where `cfg!(test)` does not reach. brief-419's `Preflight`
falsifier named eighteen more; ten were repaired and eight retained with argued
reasons. **Five of the eleven that brief-417 reclassified had landed in
`Preflight` while it had no teeth** — a deferral wearing a repair's clothes,
which neither the worker nor I noticed at that gate.

**The traversal-depth line (422-427)** began with a claim of mine that Codex
refuted: I wrote that all eight retained decoders had depth guards after
checking three of four. `FromXml` had none, and 12,000 levels of nesting built
with two `Pure` string builtins ended the process outside the panic boundary.
`TODO.md` records why: the previous repair bounded "all three codecs" when
there were four.

Four descents were bounded and ten traversals converted; the bound itself is
not installed, because caching the height that makes admission O(1) costs 30%
on list rebuilds — `SharedArgs::new` went from `Arc::new` to scanning every
argument. The stop-loss fired and the spike is in the stash.

**Method, and it is mostly about instruments.**

- *An instrument that has never been seen to fail is not known to work.* Both
  falsifiers ship with checked-in `#[should_panic]` proofs, one through eager
  dispatch and one through suspend/resume. I verified each myself by
  misdeclaring a real catalog builtin and watching the suite go red.
- *Seven probes of mine measured something other than what they claimed*, all
  the same shape: `git stash show -p` with a path, a `grep -q` that is always
  true, `Nest` stopping at the iteration limit before building anything deep,
  `Level[x, {2}]` barely descending, `grep -rl | head -1` returning a file that
  references a constant rather than defining it, summing two inventories
  without checking their overlap, and `MatchQ[x, _]` taking a fast path that
  never reaches the code under test. The discipline that catches these is
  calibrating against a known answer before believing a result.
- *Codex's content filter is not a wording problem.* It killed brief-422 twice
  — the second time after the security framing was gone, so the trigger was the
  reproduction transcript — and brief-424 twice more, at 120k tokens, on the
  worker's own generated probe code. The judgement is whether a brief needs an
  executable reproduction, not which words it uses. brief-424 I ran myself;
  brief-427 was rewritten as source reading, which is what a "why" question
  wanted anyway.
- *A report that exists only in a dispatch log is one reboot from gone.*
  brief-414's inventory was never appended, and brief-417 stopped without
  touching a file because the authority it was told to use was not in the
  repository. Appending the report is now the first thing I do with a returned
  brief.

## 2026-09-03 (later) — the candidate normalizer's recursion (briefs 428-433)

The 4,500-height ceiling six converted traversals shared turned out not to be
in the traversals. A single `MatchQ[x, _?IntegerQ]` — no traversal at all — died
in the same bracket as an all-level `Count` over the same value, while
`MatchQ[x, _]` cleared 20,001 through the blank fast path. The whole ceiling was
one recursive normalization of the match candidate, which is now bounded:
43,001 for the single match, 20,001 for the traversals, against a ruled minimum
of 4,097.

**Six briefs, three of them the same repair in three shapes.** The design
(428) was sound and the plumbing was not. Propagating exhaustion as a `Result`
cost 6-15% on wide traversal normalization, because the depth budget is
consumed only on call nodes but the return type was paid on every atom.
Handling exhaustion locally removed that and left ~8% on `matcher-single-*`,
because atoms still entered a function carrying a depth parameter their arm
ignored. Giving the recursive worker a decomposed call — so an atom cannot be
passed to it — took it to parity. Each shape was predicted before it was
measured, and each prediction named what would falsify it.

**The measurement lessons are the durable part of this line.**

- *A single base/candidate pair cannot gate this panel.* brief-431's stop-loss
  named two coherent regressing families. Four passes in one window put the
  same-binary spread at −7.78% to +11.19%: `matcher-single-wide-small` moved
  +11.19% between two runs of the identical binary, larger than the regression
  attributed to it, and its two replicates read +13.31% and +0.11%. Half that
  stop-loss was noise, and the protocol that produced it was the director's.
- *Layout is not noise and has to be sampled separately.* A third binary — HEAD
  with one function moved to the end of its `impl` block, identical semantics —
  moves this panel at most 4.45% and the `matcher-single` family 0.10%-1.52%.
  Without that number there is no way to say whether an 8% movement is the
  change. The first attempt at that control used `#[no_mangle]`, which does not
  compile here without an `unsafe` attribute the project forbids; the `cp` that
  followed silently copied the previous binary, and only the hash check caught
  that the "control" was the candidate.
- *When every affected label moves the same direction by a similar amount after
  control adjustment, the adjustment is the artifact.* The final A/B read +1.3%
  to +5.3% across the whole panel — including labels the change cannot touch.
  The within-run quotient against a same-tier untouched neighbour, which is
  immune to a whole-run offset by construction, read −0.52% to +2.64% with
  every label inside its own same-binary quotient spread. That is the measure
  that settled it.
- *A liveness probe can be the thing that fails.* `MatchQ` at height 43,000
  failed with a `Depth[x]` prelude and passed without it: `Depth` is its own
  unbounded recursion and gives out between 20,001 and 43,001. The checked-in
  certificate therefore excludes `Depth`, and a separate test carries the
  height assertion.
- *Three attempts is a plan, not drift, when each one is falsifiable.* Each
  brief named the labels that would have to move and the number at which to
  stop and hand the decision back, so a fourth shape was never on the table.

**Method that worked and should be reused.** The repair was split into three
gated steps — lock the contract in tests, move the module, change the
algorithm — so the semantic diff was readable on its own. The contract lock is
a frozen transcription of the old algorithm compared against production on
three axes per case: the `Option` discriminant, the logical value, and which
input `Arc`s the output reuses, mapped over the whole output tree. That third
axis is the one that matters, because pointer identity here is an observable
contract: the evaluation-stability cache validates entries by exact pointer
equality, and its uncached path is another recursion. Both red calibrations
were recorded before the green run, and the director reproduced the sharing one
independently.

**A record correction.** `7ae6aa3a` is labelled "docs" but also carried
brief-432's implementation of three source files; a `git add -A` swept it in.
History is left unrewritten and the fact is recorded in `0f679699`.

## 2026-09-04 — the native stack guard that could never fire (briefs 436-438)

Opened as the first of four repairs for traversals short of the ruled 4,097
height, and the first one turned out not to be about `StringExpression`.

brief-436 (diagnostic) found the mechanism: a strict head whose call stays
residual and never acquires an evaluation-stability proof is re-evaluated
through eager argument preparation on every construction step, one native
evaluator level per spine level. All three of its offered predictions held at
the gate, and it correctly refuted my `Sin` control — `Context[Sin]` is
`Global`, so that probe was a second copy of the `foo` control. Its recommended
repair reached half the class; `Power`, `Mod`, `Max`, `Min`, `Equal`, `Less`
and `Not` carry real dispatch and are in the class too.

The gate found what the diagnosis stopped one step short of.
`poll_native_stack_limit` measures bytes actually consumed, polled every four
levels — stack-honest by construction — but its default was 6 MiB against the
ruled 2 MiB embedder stack, so on the ruled configuration it could never fire.
The same construction returned `reclim` in band on 8 MiB and took SIGABRT on
2 MiB. `SECURITY.md:43-60` published "Exceeding one yields an in-band
`TerminatedEvaluation[...]` value, not a crash", which was false there, and
`codecs_xml.rs` already carried the workaround comment.

brief-437 (design-only) measured the frame: a 4,736 B static packet over seven
resident frames, against my 4,675 B bisected slope — 1.3% apart by two
independent methods. It corrected my inventory (`run_call_dispatch_stage` runs
on ascent, not during descent) and projected a 1.5-2.6 KiB optimization floor,
which settles that height 4,097 for a strict residual call needs an explicit
task stack rather than frame polishing.

Its ripple inventory was priced on an unqualified "approximately 20 KiB"
quoted from the 2026-07-06 entry, without noting that it is a **debug** number
while its own packet came from a release binary. I measured all three — 4.7
KiB release strict, 7.5 KiB release user recursion, 20.6 KiB debug — which
reversed my own first reading: I had taken 7.5 KiB and concluded the projected
test breakage would not happen, when `cargo test` runs debug on a 2 MiB thread
where the depth-72 guard-adjacent test already consumes 1.48 MiB. Two invalid
probes were caught on the way: a literal nested source pinned at 125 levels at
every stack size because it was hitting the parser's 128 nesting guard, and a
region above 2 MiB pinned by the fixed-point pass limit rather than the stack.

brief-438 landed it. Four tests broke; all four were repaired by declaring a
thread and deriving the budget from the same size, with depths and `reclim`
assertions preserved — I rejected the draft's own proposal to take the
guard-adjacent tests from 96/72/120 down to 32/24/40, which would shrink
coverage to fit the budget. The `codecs_xml` workaround is retired and its
acceptance coverage restored from 64 to the full 256.

Gate: `cargo fmt --check` clean; workspace 2329/0/19 **without** the ZeroMQ
skip flag the worker's sandbox needed, so the nine native tests it could not
run actually ran here; 12 release depth certificates green on 2 MiB under the
tighter budget, confirming the 422-433 traversal work holds; both rewired
ignored tests green. Behaviour verified directly: `Power`, `StringExpression`
and `General` at N=3000 on a 2 MiB stack went from exit 134 to exit 1 with
`General::reslim: evaluation exceeded native stack limit (1048576 bytes)`.

Three items stay open: the strict-residual class is still quadratic in
construction (brief-436's options A and B), the explicit task stack that would
actually reach 4,097 is unscheduled, and the byte guard still cannot see a
recursion that runs entirely inside one builtin.

## 2026-09-04 (later) — the deep-rebuild class and the stability walk (briefs 439-446)

Opened as "dispatch the remaining three traversal repairs" and re-probing them
under the new 1 MiB budget found one shared defect instead of three.

**brief-439 (design-only).** A traversal that rebuilds a deep result hands the
evaluator a tree with fresh `Arc` identities at every level, so every stability
cache key misses and `call_stability_uncached_with_bloom` walks the whole tree
through `cached_expr_stability_with_bloom` — mutual recursion in a file where
`evaluation_depth` never appears, so the byte guard has no poll point for the
entire descent. That is why `ReplaceAll` on a match, `Apply` at a descending
spec and `Replace` at all levels died the same way, and why the same operations
with a rule that never matches survived: the traversal returns
`Expr::call_shared` with the original pointers and the root's proof still hits.
brief-425's conversions were not wrong; the shared post-result check downstream
of them was never converted.

**The certificate panel had a matching hole, and it is the session's largest
finding.** None of the twelve exercised a rebuild. `Map` and `Apply` used
`{-4097, 0}`, which in rum selects level 0 only — intended, test-pinned
semantics, verified against `traversal.rs:142-149` — so they wrapped the value
once and handed it to an already-iterative `Depth`. `ReplaceAll` replaced a
symbol absent from the value. All three now assert the exact rebuilt
`InputForm` string, and each is paired with a shallow companion that pins its
selector on a three-level fixture. Those companions are the falsifier: pointed
at the old selectors they fail, and they need no deep value, which matters
because the pre-fix deep case aborts the test binary.

**Two repairs were rejected before the third worked.** briefs 440 and 441 made
the walk iterative behind a depth-bounded front. Both were functionally
complete — every invariant covered by differential tests including an exact
cache-insertion-order trace, twelve certificates green, counters identical —
and both cost. Reproduced in my own four-pass window with within-run quotients:
`downvalue-exact-miss-repeat` +5.6%/+6.9% against a 0.62% base spread,
`cases-wide` +5.7%/+3.8%, `matcher-single-tiny` +2.4%/+2.2%. A depth parameter
and an `Option` return on the most-consulted function in the system is not
free. Preserved as stashes.

**What worked was moving locals, not control flow (brief-442).** The walk cost
811 B per level and its dominant frame was 584 B — not because the function is
large but because the whole chain is inlined into it, making the frame the union
of thirteen data-constructor arms, cache lookup and insertion, head-effect
computation and profiling tokens. A deep descent executes almost none of that.
Outlining them gave 255.6 B/level measured, a 3.17x reduction, with the
previously-regressed families neutral to faster.

**brief-443 measured three fail-closed shapes and none fit** — a per-level floor
check grew the frame to 272 B and bailed at 3,797; the same check on a stride
held 256 B but bailed at 4,040; a bare depth counter kept the certificates but
was not stack-size-honest. All three would have turned nine certificates into
in-band failures. A well-measured no was the outcome, and it found the route
out: about 64 bytes of off-by-default profiling state resident in the hot
recursive frame, which is a hard-line violation independent of any poll. Its
method is worth keeping — gdb ptrace is denied in the worker's sandbox, so it
measured the slope from `/proc` VmStk high-water at four heights, without a
single abort.

**brief-444 removed that state.** The key is a pure function of the head, so it
is derived inside the cold enabled path rather than carried; the start instant
moved to a thread-local stack. Prologue 248 → 184, walk 256 → 192 B/level.
Height 9,500 completes on 2 MiB where 8,008 was the ceiling. The per-head
profiling surface had no test anywhere; it has an env-gated one now, run against
HEAD before the change and pasted.

**briefs 445-446 made the walk fail closed.** The strided poll fits at 192
B/level: bail-out bisected at my gate to 5,339 ok / 5,341, a margin of 1,242
levels (30%) over the ruled 4,097, and height 20,001 on a 2 MiB stack moves from
exit 134 to an in-band `General::reslim`. The bail-out is the existing
early-unstable return of `(false, bloom)` — stability only decides whether
evaluation may be skipped, so a conservative `false` is always safe and needs no
error channel. brief-445 put the counter at the walk's entry, where it also ran
for atoms and cache hits and cost `list-literal-small` +7.99%/+7.98%; moving it
to the call-cache miss path removed that entirely and is the more honest meter.
Two codec depth tests surfaced with the budget enforced and were given declared
threads like their XML sibling, not a lowered depth.

Gate at each step: workspace 2333/0/20 without the ZeroMQ skip flag, twelve
release certificates, the profiling certificate, and a four-pass panel I ran
myself rather than reading. My own instrument lied twice — a parser reading the
wrong filename that printed a table of zeroes, and a bisection whose binaries
this host had wiped from `/tmp`, so every run failed with exit 127 and it
converged on the low bound. The calibration point caught the second.

Still open: brief-439's steps 2-8, the per-site conversions whose projections
rest on brackets measured under the old 6 MiB budget; the same profiling-state
defect in the evaluator's own 4.7 KiB/level chain, worth an order of magnitude
more than the one just fixed; and three files over the 35k organization line.

## 2026-09-05 — the deep-rebuild tail, and three instruments that lied (briefs 447-456)

Ten briefs closing brief-439's tail and the association class brief-454 opened.
Every one of them was scoped by re-probing first, and **every re-probe moved the
scope** — that is the finding of the session, not any single conversion.

**What landed.** `Replace` (1,308 -> 5,843, fail-closed), `D` (1,738 -> completes
at 4,097), `DeleteCases` (1,808 -> 5,408), multi-spec `Take` (2,573) and
deep-path `ReplacePart` (2,854) both to 4,097, the lexical and rule-binding
substitution engines (3,385 and 4,430 -> 5,867/5,843), the association arm of the
stability walk, and `MergeNested`. The last three association sites — `Map` over
a deep association, a read after nested key-path assignment, and `MergeNested` —
now run to the construction ceiling at 5,457 wrappers, so the operation is no
longer the limit in any of them.

**Re-probing changed the plan five times.** draft 5 lost six of its nine named
consumers, which already cleared the contract; draft 6 split into two briefs
because `DeleteCases` and the substitution engines had different shortfalls and
different caller surfaces; slot substitution left the scope entirely once a
probe using a real `Slot[1]` replaced one that used a named parameter and
therefore measured the lexical engine; rule-binding substitution entered the
scope despite clearing the contract, because its 1.08x margin was thinner than
the ~8% these brackets drift between unrelated builds; and `MergeNested` flipped
from fail-closed to fail-open the moment brief-455 removed the polled cost that
used to be hit first. **Removing a guard-adjacent cost can convert a
fail-closed site into a fail-open one.**

**The measurement lane was rebuilt mid-session, on a user ruling.** This host has
P-cores and E-cores 1.27x apart in clock with no pinning, so a wall-time
percentage is an indication and never proof. Two worker stop-losses (+0.61% and
+7.14%) turned out to be noise; the second needed a layout-control binary to
settle. `CLAUDE.md`'s standing "counters beat timings" rule had lapsed and is
reinstated, and the lane now has three instruments: allocation counts
(`RUM_PROFILE_ALLOC`), retired instructions (`perf stat -e instructions:u`,
readable unprivileged here), and depth brackets, which are integers and immune
to all of it.

**Three instruments lied, and each cost a wrong conclusion before it was
caught.**

- *The hybrid PMU.* A run that migrates between `cpu_atom` and `cpu_core` reads
  about 55% high. Discard it; both counters non-zero is the tell.
- *The build path.* Comparing a binary built in a `git worktree` under `/tmp`
  against one built in the repository read **-8.4%** on probes the change could
  not touch. Same-shaped paths bring it to **+0.005%**. At the brief-455 gate I
  attributed those numbers to codegen from a module growing 444 KB and shipped
  that attribution into the record; the brief-456 gate refuted it. The tell was
  already in my data — two unrelated changes produced the same figure on the
  same untouched probes — and I stopped at the first explanation that fit.
- *The height unit.* One logical association wrapper is two physical levels,
  because `Association["k" -> v]` renders as `Association[Rule[…]]`. I compared
  wrapper counts against a physical-height contract and overstated three
  shortfalls twofold. `Depth` hides the `Rule` layer and must never be the unit.

**A worker diagnosis was accepted and one of its sections refuted.** brief-454
correctly identified that `Map` and post-assignment reads are the association
arm of the same stability walk this line had already shrunk on its generic arm,
carrying a 400 B association helper and a 112 B rule helper above the 192 B
generic packet. Its attribution of the shared 5,455 bracket to `Expr` teardown
was wrong: construction alone brackets identically, in band, before any output.
That mattered because the teardown reading would have sent a later reader to
optimize `Expr::drop`, a lane the standing debt already records as tried and
rejected.

**One change was accepted over its own stated bar, deliberately.** The lexical
substitution front costs +0.714% instructions on `With[{x = 1}, x + 1]` against
a ±0.15% bar. Two shapes were built and measured; the alternative, a
thread-local depth, was slower at +1.085% and carried five manual enter/leave
pairs with no `Drop` guard, whose breakage would silently shrink a thread's
recursion budget. The cost buys the removal of a process kill at 3,385 levels,
and the decision is recorded as a measured trade rather than a waiver.

**Dispatch failures, cause open.** Three dispatches were terminated mid-run with
no error text and no exit line, each after edits had landed and before
verification. It is not the content filter — the three recorded filter kills
ended with `exit 1` and a policy message — and not a size or duration ceiling,
since the three failures are the three shortest logs in the line while a 15.3 MB
run over 18 minutes completed. Recorded as sightings; the recovery that worked
was to shrink the brief and keep the long verification on the director's side.

Still open: brief-454's drafts C through J, `MapIndexed` at 2,694 (fail-closed),
and the second recursion inside `Take`/`ReplacePart` that terminates the process
above roughly 11,000, for which `merge_replace_plans` is the named candidate.

## 2026-09-05 — the association depth class re-derived; six of eight drafts dropped

brief-457 (design-only) re-derived brief-454's drafts C-J against `80c8acce`,
and the director bracketed every site in parallel. The split was deliberate: a
past-ceiling probe is not delegable in this line — brief-454 forbade the worker
from running one, and the three recorded content-filter kills were all
executable failure reproductions — so Codex got the source-side derivation and
the brackets stayed here.

**The re-pricing was the point.** brief-455 was the shared prerequisite every
one of drafts C-J was priced against; removing it re-priced the class. Six of
the eight drafts are dropped because their priced target now clears the ruled
4,097, the same outcome brief-450 reached when six of nine consumers already
cleared. What survives is one draft (multi-spec `GroupBy`, 2.03x short and
fail-open), one lane brief-454 had deferred (`Keys` and `KeyTake` over nested
lists, both short and fail-open), the generic-shape `MapIndexed` post-result
ceiling, and the schema validator.

**Four things from it are method, not history.**

- *A ceiling is not a deterministic quantity, and this line has been reporting
  it as one.* Inside a band a few tens of levels wide the same height flips run
  to run — `DeleteMissing` read 3 PASS / 2 KILL over five runs at one height —
  because the stack base moves between processes. A single-run bisection reports
  a bracket 8 wide when the real uncertainty is nearer 50. Every bracket in this
  entry is the highest height passing 3/3 against the lowest failing 3/3, with
  the failure direction read well inside the stable-fail region.
- *Reasoned packet arithmetic is reliable for an operation-local native walk and
  unreliable the moment the descent crosses the evaluator.* Ten of twelve
  projections landed within 2-25%, four within 2%. Both failures are the same
  error: an evaluator crossing priced as a native frame. The schema validator was
  reasoned at 1,986 pairs failing open and measures 104 pairs failing closed —
  19x, and the direction wrong — because it crosses the evaluator at every
  nested pair, not only at the leaf. `Keys` was reasoned to clear and does not.
- *Removing a guard-adjacent cost converts fail-closed into fail-open, and it
  did so to an entire class.* brief-456 saw this in one operation. Measured
  across the class, **every** site that now clears the contract terminates the
  process above it and not one fails in band: brief-455 lifted them over 4,097
  and removed the polling frame they had been borrowing in the same stroke. The
  contract is met with 1.5x-2x margin; the fail-closed promise is not. That is
  now the largest open item here, ahead of any individual site.
- *Decompose a composite probe before believing its attribution.* The schema
  probe varies two axes at once. Split, a deep value against a shallow schema
  clears outright and only a nested schema descends — so the site's axis is
  schema depth, and a repair aimed at value traversal would have missed it.

**The instrument was calibrated before anything was believed, and it lied
twice.** The classifier was shown printing each of its four verdicts on a known
case; its first known process-kill case was wrong, because `ToString[_,
InputForm]` runs on the iterative `RenderTask` path and survives 30,000 levels,
so the kill branch had never been demonstrated until `FullForm` replaced it.
Nineteen probes were then calibrated at height 3 against exact deepest-target
assertions, which caught three broken ones: a `Scan` count off by a factor, a
malformed `Merge` collection, and a `KeyDropFrom` written with bare string
selectors — that last one returns `invak`, which reads exactly like a depth
failure and is not one. A literal deep association source turned out to be
unreachable entirely: rum refuses source nesting past 128.

**A stated worker risk was settled by measurement rather than argued.**
brief-457 named `GroupBy`'s failure direction as its largest likely
contradiction, on the reasoning that a callback with enough evaluator depth
might reach a poll and fail in band. It does not: the direction stays a process
kill and the ceiling drops below 1,008 specs.

Still open: the class-wide fail-open direction above the contract, multi-spec
`GroupBy`, the deferred collection-shape lane, generic-shape `MapIndexed`, the
schema validator's per-pair evaluator crossing, and — unchanged from the last
entry — the second recursion inside `Take`/`ReplacePart` above roughly 11,000.

## 2026-09-05 — the native-descent guard, and a boundary the gate refused

brief-458 designed the class-wide fail-open repair and brief-459 landed its
first step: a guard substrate plus one real consumer, `Merge`'s nested
collection parser, which now fails closed in band instead of terminating the
process while still completing at raw physical height 4,097.

**The gate refused the proposed boundary, and the refusal changed the plan.**
brief-458 wanted the operation guard to run to `native_stack_limit` plus the
512 KiB non-evaluator reserve. But consumption is measured from the request
base, and that reserve covers frames below it, so granting it to frames above
the base counts the same bytes twice. Refusing it exposed the real finding:
measured under a 1 MiB budget at contract height, four of the six sites cannot
do the work at all and a fifth sits on the line, while the two already-converted
controls clear it with room. **A guard at the honest budget would have converted
four contract-meeting sites into contract-failing ones.** So the repair for
those five is conversion, not a guard — and `Merge`, the one site that passes
1 MiB, became the first consumer precisely because it proves the mechanism and
the boundary together.

**Three things from this are method, not history.**

- *A design that needs a bigger budget is telling you the budget is not the
  problem.* The 1.5 MiB proposal was arithmetically correct and would have
  worked; what it was really encoding is that five walks are too expensive per
  level. Raising the boundary would have hidden that, and the standing debt
  would have grown a number nobody could later justify.
- *A mechanism with no consumer cannot demonstrate its own boundary.* The
  worker's draft shipped the substrate with no production caller, which is
  cheaper to gate and proves less: the question was never whether a guard can
  fire, it was whether it can fire without costing the height contract.
- *Reproduce the red, not just the green.* The worker reported its red/green;
  the gate redid it by hand — removing the guard from the real edge and watching
  the focused test fail — because a falsifier reported rather than reproduced is
  the same claim the whole falsifier rule exists to distrust.

**The dispatch instrument was upgraded in the same session and immediately
showed something.** codex logs through `tracing` and emits only ERROR by
default: across fourteen dispatch logs and about 1.5 million captured lines the
entire tracing output was 25 ERROR lines from two targets. That is why three
silent mid-run terminations left nothing to read, and why "cause is open" had
been declared on an instrument that can only see ERROR. The dispatcher now
exports `RUST_LOG`, and the very next run captured INFO lines never seen before
at negligible volume. A false lead was retracted at the same time: the
`codex_models_manager` timeout appears in successful runs too, five times in one
that completed. Codex's own session rollout for the failed run ends on a
mid-turn reasoning record with no terminal event, which is the shape a stream
cut leaves; the user's report of WebSocket transport disconnects in other
sessions is now the leading hypothesis, and it remains formally open.

Still open: the enforcement layers, the five conversions, and — unchanged — the
four sites short of the contract and the second recursion inside
`Take`/`ReplacePart` above roughly 11,000.

## 2026-09-05 — the association depth class becomes a closed declaration

brief-460 installed the two enforcement layers: a 52-row surface registry where
every association builtin and cross-family axis declares how it is bounded and
whether it meets 4,097, and a `syn` recursion inventory where every recursive
descent in the implementation surface declares which row it implements. Tests
only; `git diff` is `Cargo.lock`, a dev-dependency, module wiring and the new
test directory.

**The design rule that did the work: a row states what is true now, never what
a later brief will make true.** Each row carries two orthogonal fields —
mechanism today, and measured contract status with failure direction and the
brief that owes the work — and neither may borrow from the other. brief-458's
draft had proposed `GuardedNative` for two sites that are not guarded; they are
recorded as `UnguardedNative + MeetsContract + Unmeasured`. Nineteen rows read
`UnguardedNative`, which is visibly unfinished, and that is the point: the
registry makes the remaining work un-forgettable instead of remembered.

**`Unmeasured` earned its place.** Two sites clear the association fixture's own
cap, so their behaviour above it has never been observed. The brief made "this
cell is unmeasured" an acceptable deliverable, and the alternative — inventing a
direction to fill a cell — is the failure this whole line keeps paying for.

**Both falsifiers were reproduced by hand rather than accepted on report**: one
extra syntactic recursive call added to a real function turned the inventory red
with the exact changed count, and deleting one registry row failed the
exhaustiveness check naming that builtin, which is what proves the
catalog comparison is live and not a list compared with itself.

**The dispatcher's new WARN capture paid for itself on its second run.** The
worker's `cargo test --workspace` hit `WARN codex_sandboxing::violation:
operation_not_permitted` and exited 101; its final green then used
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`, so it never exercised the nine native tests.
Under the previous ERROR-only capture none of that would have appeared in the
log, and the worker's green would have looked complete. The director's run,
without that variable on a loopback-capable host, is 2 209 tests green.

Deferred with a re-open condition: the `rum-derive` compile-time validator that
would make an unwrapped recursive edge a build error. Stronger than the
inventory, but a separate correctness surface, and the inventory covers the same
class meanwhile.

Still open: two guards, five conversions, and — unchanged — the four sites short
of the contract and the second recursion inside `Take`/`ReplacePart`.

## 2026-09-05 — the guarded population closes at three

brief-461 guarded association `Scan` and nested key-path `Set`, the two sites
measured to fit the 1 MiB operation boundary at contract height. Both now fail
closed in band where they terminated the process, and both still complete at
raw physical height 4,097. Every remaining `UnguardedNative` row in the registry
is now a conversion, not a guard — which is the useful thing the registry does:
the remaining work is a list rather than a memory.

**A guard refuses at the budget; it cannot make work fit inside it.** That
sentence is the whole selection rule for this line, and it is why only three of
the eight sites are guarded. The other five do not fit 1 MiB at 4,097, so a
guard would convert contract-meeting sites into contract-failing ones.

**Two verification habits earned their place again.**

- *Four edges, four reds.* Removing each real wrapper in turn and watching its
  focused test fail is what distinguishes a guarded edge from a decorated one.
  All four turned red; an edge whose removal leaves the suite green would have
  been coverage theatre, and this is the only check that finds it.
- *Name the control that shares the machinery.* Three of the four edges live in
  `apply_map`, which `Map` also uses, so `Map` was required in the cost gate
  rather than a generic control. It moved +0.002% against the guarded
  operations' +0.082% and +0.175%, with allocation counts identical to the byte
  throughout.

**I miscounted my own registry in the previous entry** — 52 rows where there are
44, from a grep that swept up the enum declaration and the vocabulary list
alongside the rows. The correction is in `TODO.md`. The rule it breaks is one I
had written into the brief that produced the registry: a script-derived count is
a claim, not a fact, and wants a hand cross-check before it is recorded.

Still open: the five conversions, the four sites short of the contract, and the
second recursion inside `Take`/`ReplacePart`.

## 2026-09-05 — the first conversion, and a stop-loss that was right for the wrong reason

briefs 462-463 converted `DeleteMissing`'s recursive-mode walkers, its two pairs
of delayed-wrapper counters, its memory preflight projection and its missing
scan to bounded fronts over explicit work stacks. The finite-target/resume
family is untouched and the registry row stays `UnguardedNative`.

**brief-462 stopped on a red certificate and was right to stop — but the red was
its own probe.** It armed `MemoryConstrained` with an absolute ceiling of 1e9
while `MemoryInUse[]` at the certificate's height is 2.8 GB, so the limit sat
below current usage and the call aborted before running. It had copied the
two-argument form from its own shallow companion, where 1e9 is comfortably above
usage. **A constant that is generous at depth 3 and impossible at depth 4,097 is
a scaling assumption, and this line keeps finding them in probes rather than in
code.**

**The gate then found two things the stop-loss had hidden**, because stopping
early meant the worker never reached its own cost gate or its last deliverable:

- *An allocation regression on the default path*, +1 per association level, at a
  depth shallower than the front so no heap tail ran at all. The cause is a
  detail worth carrying: `entries.into_iter().filter_map(…).collect()` over an
  **owned** `Vec` can be satisfied in place, reusing the source buffer, and
  rewriting it as `Vec::with_capacity` plus `push` silently gives that up. The
  sibling list walk looked identical and was unaffected, because its base
  iterated a borrowed slice and had to allocate anyway.
- *The forced-front falsifier was missing*, and it mattered more than it looks:
  every certificate fixture in the conversion is a single chain, so nothing
  exercised branch handling in the heap tail. A tail that dropped or reordered
  entries would have passed the whole panel.

**The replacement falsifier is better than the one I specified.** I had asked
for a test-only cargo feature to force the front; brief-463 instead calls the
private `*_with_front_depth` with a front of zero from an in-module test — no
feature, no production branch — and asserts front-zero really reports
`Exhausted`, front-16 reports `Complete`, and **tail == front == expected** on a
three-entry-per-level fixture carrying a per-level witness. Verified to fire by
reversing the tail's push order.

**Not committing the conversion before the repair** was the other decision here.
A known allocation regression shipped now and fixed later is the stopgap-now
shape this project rules out, so brief-462's tree stayed uncommitted and
brief-463 built on it; the two land as one correct change.

Gate: 2 379 tests green without the ZeroMQ skip, three release certificates at
raw physical height 4,097, allocation counts byte-identical to base, and
instructions -0.639% on the converted path.

Still open: the finite-target/resume family, the remaining four conversions, the
four sites short of the contract, and the second recursion inside
`Take`/`ReplacePart`.

## 2026-09-05 — a width defect nobody had looked for, and a conversion shape that will not transfer

brief-464 planned `DeleteMissing`'s finite-target/resume conversion. The
measurement that preceded it mattered more than the plan.

**`DeleteMissing[Table[Missing[], {3000}], 1, 0]` terminates the process.** The
value has physical height 2. The whole line has been measuring *depth* because
the ruled contract is about depth, and this defect sits on an axis nobody had a
probe for. brief-454 had dismissed those edges as "loops or tail transitions
over siblings"; Rust does not guarantee tail-call elimination and they are not
loops. **A reasoned claim about what the compiler will do is a hypothesis, and
it wants a measurement before a plan rests on it.**

**The conversion shape this line has used four times does not transfer.** Every
previous conversion — `Replace`, `DeleteCases`, `MergeNested`, and
`DeleteMissing`'s own recursive mode — lets a cold iterative tail redo the
traversal once the bounded front exhausts. That is only sound for a pure walker.
Here a target-level delayed evaluation runs user code, so a restart could run it
twice; the exhausted handoff has to carry the live continuation and the
completed prefixes instead. Finding that before writing an implementation brief
is exactly why the step was design-only.

**A negative result was calibrated instead of asserted.** The plan surveyed ten
other association sites sharing the same `index + 1` idiom and concluded each
lands in an explicit loop, so no other registry row is dishonest. That is the
shape of claim this project distrusts, so it was spot-checked by measurement at
a width where `DeleteMissing` reliably dies: `Counts`, `AssociationMap` and
`KeySelect` all clear 10,000 siblings 3/3. The probe could have contradicted the
survey.

My own per-edge axis reading was corrected in both directions — right but
incomplete on both axes — which is the ordinary outcome of asking a second
reader and the reason the plan-first rule exists.

Still open: the two-step repair, the remaining four conversions, the four sites
short of the contract, and the second recursion inside `Take`/`ReplacePart`.

## 2026-09-05 — DeleteMissing reaches BoundedHeap, and a crux that dissolved

briefs 466-467 converted `DeleteMissing`'s finite-target/resume family. Both
axes are closed: a flat 3,000-element list that terminated the process now
completes, 64,000 siblings complete, and depth runs to 8,000 against a previous
ceiling of 4,944. The ten semantic locks from step A passed unmodified, which is
what step A was for.

**The interesting part is what happened to the crux.** brief-464 had identified
the carried handoff as the thing that made this conversion different: a cold
tail that redid the traversal could re-run user code, so the exhausted handoff
had to carry live cursors and completed prefixes. At the gate I perturbed the
entry point to discard its work and restart from the root — and **890 tests
stayed green**, including the test named for exactly that property.

The follow-up brief asked for a test that would catch it. It stop-lossed with a
proof that the test cannot exist: delayed right-hand sides are evaluated only at
`level == 1`, front exhaustion happens only at front depth zero where `level` is
still above 1, so on the only path with a front, exhaustion strictly precedes
any user-code evaluation. I confirmed the other half myself — the resume driver
runs entirely on the heap with no front, including after a suspension.

**A restart would therefore be invisible, and the risk was real in the abstract
and unreachable in the concrete.** Three things follow.

- *Stopping with a proof beat producing a test.* The worker could have written
  something that passes and claims coverage; it wrote the invariant instead.
  That is the same judgement brief-465 asked for on its item 8, and the same
  answer: a property nothing can violate is either structural or not a property.
- *The invariant is now recorded beside the handoff*, naming the two changes
  that would re-open it — a predicate evaluated above target level, or a front
  reintroduced on the resume path. A structural guarantee that nobody wrote down
  is one refactor from being gone.
- *A test whose name claims more than its fixture can see is worse than no
  test.* The in-module one hand-assembled a cursor at index 1 with the prefix
  already accumulated, so it never exercised the real entry. Renamed to say what
  it establishes.

**A procedural self-inflicted failure, recorded because it mimics the open
dispatch-interruption sighting.** The first brief-467 dispatch produced no log
and exited 0 because I piped the dispatcher through `head -6`; with a dirty tree
the extra warning line pushed a later `echo` past the closed pipe, and
`set -euo pipefail` killed the script before it dispatched. Read carelessly —
exit 0, no new tests — it looks exactly like the silent mid-run terminations
this project has been chasing. The tell was the missing `rust-log:` line.

Still open: four conversions, the four sites short of the contract, and the
second recursion inside `Take`/`ReplacePart`.

## 2026-09-05 — two independent implementations of one brief, and what the second saw

brief-468 converted the association key-reshape walks. Its first dispatch was
interrupted while running its own verification, so on the user's suggestion the
partial work was parked outside the repository and outside `/tmp` — both inside
the worker's write fence — and the brief was re-dispatched from a clean tree.
That produced two independent implementations of the same specification, which
is not something this project has had before.

**They converged on architecture.** Both chose three separate depth-16 front
constants rather than one shared constant, both produced the same three heap
loops and the same render work enum, and their files are within 7% on size. When
two independent runs of a brief produce the same design, the brief was specific
enough to determine it — which is a useful thing to be able to say about a brief
format.

**They diverged on one thing, and the divergence was a real defect.**
`NestedPathNode::Branch(NestedPathNodes)` is recursive, so its derived `Drop`
recurses; deflatten's error path drops a partially built tree. The second
attempt added an iterative drop, the first did not. Measured on the same probe:
attempt 1 terminates the process at depth 60,000, attempt 2 completes. Neither
the brief nor I had thought about the drop of the *intermediate* structure — the
standing debt covers deep `Expr` drop, and this is a local type nobody had
looked at.

**My first probe of that defect measured nothing**, and the reason is worth
keeping: I assumed a non-list association key would be rejected, so I built a
fixture around `"notalist"`. `path_from_key` treats any non-list key as a
one-element path and rejects **only** an empty list. The probe ran the ordinary
success path at every depth and both implementations passed it. Calibrating the
trigger at depth 3 — checking the operation actually returned unevaluated — is
what exposed the difference.

**The recorded gap.** The iterative drop has no test. It is unrequested
hardening that closes a real fail-open, every deliverable the brief asked for is
complete and verified, and holding a finished conversion over a bonus would be
the wrong trade — so it is committed with the certificate owed and named.

Still open: the drop-path certificate, two conversions, the four sites short of
the contract, and the second recursion inside `Take`/`ReplacePart`.

## 2026-09-05 — association MapIndexed bounded, and a certificate that could not exist

brief-469 converted `MapIndexed`'s three traversal edges behind a depth-16 front
over a heap tail. Gate: 2 399 tests green, the plain certificate exact at raw
physical height 4,097, the forced-front falsifier reproduced red by reversing
the tail's assembly order, and allocation counts byte-identical to base on the
`Map` and `Scan` controls that share the file.

**The brief's two named traps were both real and both avoided.** `Scan` sits in
the same file with the same three-edge shape and was *guarded* rather than
converted, because it fits the 1 MiB boundary at contract height and
`MapIndexed` does not; naming that explicitly is what kept the worker from
copying the neighbour. And the walker is shared with the generic shape, so the
brief drew the line: convert the walker, flip only the association-axis row,
leave the generic shortfall's tracking alone because it lives downstream.

**A certificate I demanded turned out to be impossible, for the second time.**
I asked for a deep `Heads -> True` certificate so the head edge would have
certificate coverage. It fails — identically on the pre-conversion build. The
head-bearing centre is a call of undefined symbols, which the evaluator descends
as real evaluation levels, while an `Association` is a data constructor costing
none; the shape refuses in band from height 515 upward, nowhere near where the
walker matters. Rewritten to pin the honest property: it must fail closed, never
terminate the process.

brief-467 was the first instance of this. The lesson is narrow and repeatable:
**probe a certificate's premise before writing it into a brief.** Both times the
impossibility was structural and a few minutes of probing would have shown it,
and both times it cost a dispatch or a gate cycle to discover.

A smaller one worth keeping: my first probe of the heads shape used
`Heads -> True` on a pure association chain and produced output identical to the
plain call, because there are no heads to map there. It looked like the option
was ignored. The certificate's own fixture — an `outerHead[innerHead[...]]`
centre — is what makes the option observable.

Still open: one conversion, the four sites short of the contract, and the second
recursion inside `Take`/`ReplacePart`.

## 2026-09-05 — the conversion phase closes, and the registry corrects my summary

brief-470 converted the runtime's three exact-path walks, so `AssociateTo` and
`KeyDropFrom` on a `Part` target are `BoundedHeap`. That was the last of the
five conversions. Gate: 2 404 tests green, two release certificates at raw
physical height 4,097 — one per head, since `KeyDropFrom` needs `Key[...]`
selectors where `AssociateTo` also accepts a bare string — three forced-front
falsifiers, and allocation counts byte-identical to base on the `ReplacePart`
and `MapAt` controls that share these helpers.

The axis was **path length**, not value depth, which is the one thing about this
site that could have sent a fixture to the wrong walk. `mutation.rs` turned out
to need no change at all.

**The registry corrected my own summary of where the line stands, which is the
point of having built it.** I had been saying the conversion phase would leave
only the four short-of-contract sites. It leaves nine unguarded rows: four short
of the contract — and the deferred collection-shape lane has *three* members,
`Keys`, `KeyTake` and `KeyDrop`, not the two I had repeated several times — plus
**five that meet the contract and are still unbounded**: `Association`,
`KeyComplement`, `Lookup`, `Normal`, `Values`.

Those five are brief-454's second table, recursion on collection, selector and
wrapper axes rather than association value nesting. They are **reasoned** to
clear 4,097 and have never been bracketed. Every time this line has measured
something it had only reasoned about — the `DeleteMissing` width axis, the
`Keys` ceiling, the target-mode packet ratio — the scope moved. So they are the
next thing to measure, not the next thing to convert.

A structural detail worth keeping: the registry refuses an `UnguardedNative` row
whose `owed_by` is `None`. That is what kept these five visible instead of
quietly reading as finished work.

Still open: bracket the five residual shapes, the four short sites, and the
second recursion inside `Take`/`ReplacePart`.

## 2026-09-05 — the residual shapes measured, and the reasoning held

The five association rows that were `UnguardedNative + MeetsContract` on
reasoning alone now have brackets. All five clear the ruled 4,097, so the
reasoned column was right — which is unusual enough in this line to be worth
recording. The pattern it fits: these are operation-local native walks that
never reach the evaluator, and brief-457 already identified that as the class
where packet arithmetic is reliable. Where it has failed, the descent crossed
`Session::evaluate`.

What was actually unknown was the failure direction. Four fail open; `Normal`
alone fails closed.

**`Lookup` is the thinnest margin in the whole class at 1.16x**, thinner than
the `DeleteMissing` target mode that this line treated as urgent, and it fails
open. It leads the residual queue.

**`KeyComplement`'s recursion lives on an error-message path.** Passing a second
argument sends it through `collect_option_names_from_tail`, which recurses over
nested `List`s to name the offending option. The reachable shape is therefore a
deep *bad argument*, not a deep value. Diagnostic paths take inputs nobody
writes on purpose, and this line has not been looking at them.

**I doubted the registry and the registry was right.** brief-454 records `Keys`
and `Values` as sharing one walk, yet their rows carry different contract
statuses, which read as an inconsistency worth catching. Measured on the same
fixture: `Keys` 3,056, `Values` 5,896. The rows are correct and the shared-walk
citation is what needs re-reading. Building the registry so that a row must
state what it is and who owes it has now caught my summary twice and been wrong
zero times.

Still open: the residual queue in ranked order, the four short-of-contract
sites, and the second recursion inside `Take`/`ReplacePart`.

## 2026-09-18/19 — a neighbouring project's review, and the axis rum's tooling did not model

A sibling project, `serval`, took three measures from rum — the stack-byte
probe, the `syn` self-recursion inventory, and the per-site registry — and had
them reviewed by a third model, which rejected them as a final architecture. The
user asked whether that review helps rum. It does, though not in the way it
helps serval.

**Its diagnosis transfers and its prescription does not.** The central claim is
that the host call stack carries growable computation while the host destructor
carries a growable ownership graph, and the budget error constrains only part of
the former. rum's tooling is exactly that shape: the recursion inventory models
**call** cycles, the registry models per-builtin **descent**, and neither models
**ownership**. The evidence was already on the record and nobody had read it that
way — the two real drop hazards found this month, `NestedPathNodes` in brief-468
and `LookupOutcome` in brief-474, were both found **by accident**, one by an
unplanned cross-comparison of two implementations and one because the director
remembered the first. The prescription — arena plus flat IDs plus an explicit
state machine — does not transfer: it is a representation change for a
dependent-type kernel, rum's pain is in intermediate and suspended-state types,
and rum already rejected iterative `Drop` for `Expr` on a measured 20-50%
hot-path regression.

**Measuring the exposure took five failed probes and produced a better answer
than the reasoning did.** The `Persistent*Node` family — eleven files, a singly
linked `Option<Arc<Node>>` with no compaction — looked unbounded and I could not
reach it. Five shapes failed, each verified by a `panic!` planted in the target
as a liveness assertion. brief-477 found it: the tail grows **per element** on
the string-replacement resume paths, one suspension and then a node per
character, and with the generated-element and iteration-work limits **both set to
8** the sequence still reached 1 000. My probes had simply been the wrong
operation family. A negative from a probe that never reached its target says
nothing, which is the whole content of that standing lesson.

**The ledger was split so its first step would produce a number.** The open
question — whether to change the embedding API so host payloads declare their
ownership — was deferred until the scale was measured rather than argued, so
brief-479 ships discovery only. The answer: **48 `UnknownHostPayload` sites, of
which only 6 are places rum stores an opaque payload and therefore owns its
destruction**; the other 42 are public inputs taken by value. The structural fix
needs a contract on six sites, not forty-eight — an order of magnitude smaller
than the option looked while it was only an argument.

**Three things from this are method.**

- *A check's specification can be the director's own failure.* My scan for
  self-referential types missed both known hazards — one says `Vec<Self>`, the
  other is a mutual pair whose edge runs through a tuple inside a `Vec` — so the
  brief required the new scanner to rediscover both **without being told their
  names**. It did, and the witnesses name exactly the two constructs my regex
  could not see.
- *Maximal conservatism produces a number you cannot act on.* Told never to skip
  an undecidable construct, the scanner reported 7 068 — of which 1 271 are plain
  `derive` and only 70 are storage or representation. The instruction was right
  and the classification is now the design problem: an unknown expansion that
  cannot introduce an owning edge is not the same fact as one that can.
- *Pick work by how expensive it is to gate, not only by its rank.* On a
  low-quota day the two dispatches were a measurement whose gate is a number and
  a pure test-file move whose gate is `wc -c` plus an unchanged test-name set.
  Both were heavy for the worker and nearly free to verify, and a design-only
  brief was preferred over a conversion precisely because it leaves a clean tree
  if the session ends mid-review.

**Five instrument slips in two days, all the same shape**: a `git grep` that
could not see untracked files, a classification keyed on the function containing
a call rather than its callers, a perturbation applied to synthetic fixtures
instead of the production inventory, a size listing that globbed `*.rs` while the
inventory is a `.tsv`, and a probe set whose controls moved so much that a real
+0.6% read as noise. Every one reported something confidently about ground it
had never touched.

Still open: the disposition ledger and its second axis, the `Persistent*Node`
drop ceiling, `GroupBy`, the residual shapes, and the schema validator.

## 2026-09-21 — the ownership branch closes, and four defects it was not looking for (briefs 480-498)

The branch opened from `serval`'s review with one question — should the
embedding API demand bounded destruction — and the user ruled to change the
API rather than document the hazard. It closes with that contract built, the
one class of unbounded destruction rum could actually repair repaired, and
four defects that had nothing to do with the question.

**The contract.** `BoundedDestruction` is a safe opt-in marker with no blanket
impl and no derive; implementing it *is* the stance, and its rustdoc says
plainly that the compiler enforces only that a stance was taken. `EntropySource`
and `FileIo` require it. The print sink could not: a trait cannot be implemented
on a host's closure, so `Session` now stores a named `PrintSink` and closures
arrive through `ClosurePrintSink::assert_bounded_destruction`, whose field is
private and which has no `From`, no `Default` and above all no blanket impl —
that blanket impl being both the shortcut that would silently restore the old
permissiveness and, therefore, the falsifier. The authoritative evidence is a
compiler error from crates **outside** the workspace; `compile_fail` doctests
stayed documentation. `rum-cli` migrated through `rum-host`'s re-export and did
not gain a `rum-runtime` dependency.

**Step D was cancelled rather than executed, which is the arc's best
judgement.** Making the marker a supertrait of `SuspensionFrame` and
`PureFrameFinalizer` would have asked about 598 implementors to assert what the
project records as false: `Expr`'s destruction is unbounded standing debt and
266 of 535 frame structs own an `Expr`-family field directly. The review added
two qualifications worth keeping — returning an `Expr` does not make a
*particular* frame unbounded, so what the evidence refutes is a universal
supertrait rather than every declaration; and `#[doc(hidden)]` does not close an
implementation set, so recording the in-tree debt cannot discharge external
implementors. The line ends with a disposition ledger over all 50
`UnknownHostPayload` rows whose `Unbounded` entries are **verified** against the
discovered graph rather than merely written, so a row that does not reach
`Expr` cannot borrow its debt.

**The repair that was possible.** All 29 accumulator-chain members — every
`Option<Arc<Self>>` spine in the kernel and stdlib — now destroy through one
`#[repr(transparent)]` wrapper carrying a single cold drain that acquires with
`Arc::into_inner` and stops at a shared node. Three enforcement layers, one
claim each: the derive rejects a non-wrapper link at compile time, a depth
observer proves the drain is iterative (exactly 1 nesting at chain lengths 16
and 64), and a discovery-derived gate proves the family is complete — with no
name list, allowlist or exemption table, so a future type of the shape must be
converted or the tree is red.

**Four defects, none of them the question.** A resumed `Table[InputString[], 2]`
panicked the kernel, because `into_vec` demanded unique ownership of a tail the
resume driver legitimately roots; the repair moves out of a unique node and
clones out of a shared one, which is what the accumulator's own frozen prefix
already did. A **call-head chain** evades the parser's 128 delimiter and
operator counters — `f[1][1]…` accepted at 5 000 where the nested form of the
same depth is refused at 300 — and overflowed AST-to-`Expr` lowering at 1.22x to
1.46x the contract height, failing open; lowering is now bounded. The parse dump
was **cubic**, because `{:#?}` wraps the writer in another pad adapter per
nesting level. And the standing-debt entry claiming source-created trees are
depth-guarded was false for that shape and is corrected.

**The scanner could not see a crate the kernel owns.** `rum-syntax` was not a
root while `rum-runtime` owns its types in four places, and its AST is
recursive. The root set is now a criterion — kernel and stdlib plus workspace
crates whose values they own, transitively — rather than a list.

**E2 was examined and declined.** Of 7 068 `UnknownConstruct` rows only 70 can
hide an owning edge, and those are 29 `std`, 6 `num_bigint`, 6 `rum_syntax` and
flat leaves. An axis declaring an unknown "cannot introduce an owning edge"
silences it; maximal conservatism is loud and useless but never lies.

**Method, in the order it cost the most.**

- *A permission list gets under-enumerated; a disclosure duty cannot.* Five
  stop-losses on this branch were the director's MAY-list and inventory-rule
  omissions, every one a correct worker stop. The rule's shape was wrong, not
  its care: "regenerate as discovery requires, list every content change with
  its witness, classified, and anything unclassified is a stop" is the form that
  survives.
- *A falsifier that goes red is not evidence until you read why.* The ledger's
  attribution falsifier first went red on an unrelated precondition about
  ambiguous trait bounds and had to be re-run before it certified anything; an
  exit-status-only probe turned 60-second timeouts into phantom stack overflows;
  and a growth ratio believed without subtracting process startup made a cubic
  look sub-quadratic.
- *Calibrate the instrument and hand over the recipe, not the tool name.* The
  timing panel cannot resolve a question this size — its own controls moved
  +3.70% to +12.45%. Instruction counts pinned to one core type
  (`taskset -c 2 perf stat -e instructions:u`, `cpu_core` at 100.00 multiplex)
  repeat to **0.003%** on this host, and that recipe went into the briefs.
- *Acceptance is judged on the split.* The chain work costs +3.57% on a
  `DeleteMissing` deep enough to enter the heap tail and **-0.005%** at ordinary
  depth; bounded lowering moves +0.117% on a lowering-dominated workload, inside
  both arms' spreads, and is 0.351% faster on an evaluation-dominated one. A
  cost confined to the shape the change is about, with the ordinary case
  unmoved, is what this project's precedents accepted for removing a process
  kill.
- *`git checkout` restores from the index, so on a worker's uncommitted output
  it is a delete.* The director did it twice while warning others against it;
  recovery cost a diff extraction from the dispatch log, a hand rewrite, a test
  recovered from the log, and two inventory regeneration passes.

Still open: the security-hardening mainline's own findings, and the three
criticisms of the threat model that the 2026-09-02 audit left unanswered.

## 2026-09-21 — the A1 spike refutes its own remedy, and the ruling that follows (briefs 499-504)

The security-hardening mainline reopened on its HIGH finding — a process-global
symbol interner that leaks by construction and is invisible to the per-session
memory ceiling — and the 2026-09-02 ruling had already chosen the remedy:
session-scoped ownership, option A1, opening as a spike because it makes
`Symbol` non-`Copy`. The spike was built and the remedy did not survive it.

**The migration was the cheap half.** Removing `Copy` from `Symbol` cost
+0.06% to +0.84% and touched 92 files, and its fallout turned out not to be a
count at all but a fixpoint: each repair pass exposed the next layer of types
forced to drop their own `derive(Copy)`, converging 304 → 80 → 5 → 0. The
closure is 35 types, and 29 of the 34 containing types are the off-by-default
profiling keys — so what `Symbol`'s `Copy`-ness was holding up is very nearly
just diagnostics.

**The representation was not.** With sizes unchanged — `Symbol` 24, `Expr` 32
on every arm — the candidate cost **+14.33% instructions on ordinary
evaluation**, and +10.4% to +18.9% on every one of twelve workloads. A split
into two correct variants attributed it cleanly, residuals under 0.1
percentage points: **+10.93% for the owning representation, +3.42% for the
equality demotion.** The variant that pays only the first is the cheapest
owning `Symbol` that can exist, and it is not a valid A1 — its dense-id
equality is sound only while the interner stays global, which is the thing A1
exists to change.

**The finding the spike was not looking for.** `Symbol`'s `needs_drop` flips
false to true, the debug stack slope rises 11.39%, and a depth-48 rewrite
recursion goes red. Release is untouched — the slope is 3.36% *smaller* and
the margin over 4 097 is 44.84% against a historical 30% — so the contract
never moved. Measuring the stack first was right and its answer was no.

**The ruling (user, 2026-09-21).** `Symbol` keeps its lifetime-free public
type. brief-504's design pass had surveyed six options and recommended a
generatively branded session epoch — the only compiler-enforced candidate —
but stated plainly that a brand forces scoped continuations and a scoped host
API. The embedding line deliberately built an owned, movable `HostSession`;
suspension frames are `Send + Sync + 'static`. That is semantic loss rather
than cost, which is why it counts, and with the type frozen no in-process
reclaiming fix exists. The finding stays open behind a bounded, loud
mitigation: a process-wide byte quota, a compiler-enforced refusal in place of
the `u32` panic, and documentation that says out loud that `reset()` does not
reclaim and one tenant can exhaust the shared quota.

**Method, in the order it cost the most.**

- *A stop-loss keyed on the failure's reason fires on every converging
  mechanical repair.* "Stop if the same crate fails twice for the same reason"
  was guaranteed to trip: a non-`Copy` migration emits only ownership errors,
  so the rule could stop the work but never diagnose it. It now keys on
  whether the distinct count fell.
- *A gate is worth what the worker's environment can run.* The expected
  failure set was recorded on this host with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`
  set and written into the brief without it, so nine sandbox-bound ZeroMQ
  failures read as regressions. The lesson was already in memory and was not
  applied when writing the gate.
- *HEAD is process-global.* A background job switching branches to build A/B
  arms swallowed a foreground commit onto a detached HEAD; `cherry-pick`
  recovered it, but the script had been sitting on `main` for 25 seconds.
- *Calibrate before believing, again.* The first instruction probe measured
  process startup, because a `Do` over 200 000 iterations hits the 100 000
  iteration ceiling and bails. The tell is that instruction counts must scale
  with `N`; at N=25 000 and 50 000 they read a ratio of 1.995, and only then
  was the recipe trustworthy — at which point it repeated to 0.00005%.
- *A worker that reports a preparation gap is worth more than one that
  improvises.* `alloc-count` is an off-by-default feature, so the supplied
  binaries could not report allocations; brief-501 said so instead of
  producing numbers.
- *Name the constraint you actually mean.* brief-504 was told `Symbol`'s
  representation was frozen and noticed that this does not say whether its
  lifetime-free type is frozen too — a difference that decides whether any
  fix exists — and refused to resolve it silently. That refusal is what
  produced the ruling.

## 2026-09-22 — the interner line finds its root constraint, and one defect closes (briefs 505-511)

Seven briefs. The line set out to bound the process-global symbol interner
and instead found why it cannot currently be bounded usefully; one unrelated
defect on the same mainline was diagnosed and fixed along the way.

**The census came before the build, and earned its place immediately.**
brief-505's design plan proposed an 8 MiB default while stating that the
source offers no measured legitimate working set to argue from, so the
director turned the default into a measurement obligation rather than
approving a number. brief-506 measured: bootstrap is 662 strings and 661
pairs at about 77-85 KB, and the **ordinary workspace suite's maximum over
test binaries is 14.4-15.9 MB**. The proposed default was below this
repository's own test suite.

**It also refuted a premise the director had asserted as verified.** The
claim was that `Module`'s gensyms do not grow the interner because
`with_identity` varies only a numeric identity. True of that API, and
irrelevant to `Module`, whose locals reach `Symbol::new` with formatted fresh
text. Every activation permanently adds a string and a pair, about 120 bytes,
and the gensym GC cannot help because the interner has no removal path at
all. The director had read one call path and generalized it to another. A
third correction came with it: lookup is itself an insertion ingress,
charging **two** pairs for one fresh unqualified name.

**That retargeted the line, and then the retarget was refuted twice.** A cap
that ordinary code exhausts is a timer rather than a defence, so the dominant
legitimate consumer had to go first. brief-507 found the `Module` repair
feasible in representation — `Symbol` already carries the identity field —
and blocked on semantics. brief-509 found the same wall at `Enclose`'s
internal tag, which had looked like a safe pilot because the user never
writes its name.

**Two refutations on identical evidence is the line's actual result:**

> A symbol's identity must be expressible as text, because the host snapshot
> is source text.

`qualified_name` ignores `identity` and `Display` delegates to it, so two
same-text symbols with distinct identities are indistinguishable the moment
they are rendered; the resolver rebuilds no identity from a name. `Symbol`'s
serde does preserve identity and is not the snapshot format. So every
generator of distinguishable fresh symbols must mint fresh text, the interner
grows with generated symbols, and `Module`, `Enclose` and `Unique` are three
instances of one cause. Patching constructs one at a time cannot work. The
already-deferred binary snapshot format in Standing Engineering Debt is the
deferral that makes the constraint binding, and three ways out — a reserved
textual encoding, a non-source snapshot format, or accepting the leak behind
the cap — all await a user ruling.

**The cap's mechanism is nevertheless proven.** brief-508's scratch package
is green and director-reproduced: exact charging including backing-table
bytes, refusal that publishes and leaks nothing, the two-pair ingress
admitted or refused atomically, a full interner still serving lookups it
already holds, and — with enforcement disabled — a falsifier watched going
red at peak 4307 against cap 4306. What is in doubt is whether the cap is a
defence or a timer, which is exactly what the pending ruling decides.

**One defect closed while the line waited.** `eval_with_timeout` cleared its
timeout unconditionally, so an evaluation suspended for host input lost its
bound. Re-probing before briefing shrank it: the rustdoc three lines above
already said the timeout is cleared after the request completes or fails, and
a suspension is neither, so the repair honours an existing contract instead
of inventing one. The deadline stays absolute across the suspension because
the kernel cannot do otherwise — `TimeConstrained` computes an `Instant` once
and nothing recomputes or pauses it — and a characterization test pins that
precedent before production changed. 2 490 passed, 0 failed, 57 ignored, with
the nine native ZeroMQ scenarios actually executing.

**Method, in the order it cost the most.**

- *A premise asserted as verified is still a premise.* The `Module` claim was
  written into a brief as director-verified and was wrong. The brief told the
  worker to confirm rather than assume, and that instruction is the only
  reason it was caught the same day.
- *A stop-loss keyed on the failure's reason fires on every converging
  mechanical repair.* brief-499's rule stopped a repair at 304 → 80 → 5
  distinct diagnostics, five one-line sites from green, because a non-`Copy`
  migration emits only ownership errors. It now keys on whether the count
  fell.
- *A gate is worth what the worker's environment can run.* An expected
  failure set recorded with `RUM_ALLOW_NATIVE_ZEROMQ_SKIP` set, and written
  into a brief without it, turned nine sandbox-bound failures into apparent
  regressions. The lesson was already in memory.
- *HEAD is process-global.* A background job switching branches to build A/B
  arms swallowed a foreground commit onto a detached HEAD.
- *Calibrate before believing.* The first instruction probe measured process
  startup, because a `Do` over 200 000 iterations hits the iteration ceiling
  and bails; counts must be shown to scale with `N` first.
- *Workers that report a gap beat workers that improvise.* brief-501 reported
  that the supplied binaries lacked allocation counters instead of producing
  numbers; brief-510 reported that it could not build the harness its
  precedent check needed instead of inferring the answer. Both gaps were then
  closed deliberately.

## 2026-09-22 — three defects closed, three routes refuted, and the armed ceiling made affordable (briefs 512-523)

The security-hardening mainline's memory half. It closed three defects, spent
three full implementations on routes that were then refused on measurement,
and ended with the armed memory ceiling costing 93-99.9% less than it did
that morning.

**Three defects closed.** A truncated snapshot restore reported success,
because `restore_from_bytes` discarded the whole response including its
termination; it now fails with the reason, restore stays non-transactional,
and the partial-state consequence is documented rather than discovered
(`acfda0e0`). The audit's second HIGH — a ceiling a loop walked straight
through — reproduced with a control: an 8 MB `MemoryConstrained` let
`Nest[StringJoin[#,#]&, seed, 16]` finish at 65 536 000 characters and ~358 MB
RSS while the same ceiling aborted a single large `StringRepeat`. Every armed
checkpoint now walks (`d1e5de5c`). And the occurrence model that walk computes
was specified per field with an independent oracle, an exhaustive schema whose
compile barrier was watched stopping an unclassified field, and pinned formula
and overflow tests (`902aea1f`).

**The audit's mechanism for that HIGH was wrong, and so were two of the
director's own claims.** `StringJoin` is declared `Preflight` and does not
amplify per call — concatenation rearranges bytes — so the growth assertion
passes it correctly; and that assertion is a **test-only instrument**,
bypassed unless `RUM_TEST_BUILTIN_MEMORY_GROWTH` is set, so it never described
production at all. The overshoot was repetition of a non-amplifying step
between strided walks. The director then claimed a `StringJoin` precheck could
not have caught it, which was also wrong: projecting output bytes into the
unstrided enforcement refuses at 1000 × 2^13 > 8 000 000.

**Then the cost of that fix became the problem, and the user reframed it.**
Walking every armed checkpoint cost 2.26x-43.34x. The director treated that as
a niche opt-in and invoked the no-default-path-cost hard line to refuse a
repair. `SECURITY.md` says the opposite: rum's model *is* evaluating untrusted
scripts, and a host doing so **MUST** arm the ceiling. So the armed path is the
security model's own path, its cost is the security model's cost, and the hard
line — written about off-by-default diagnostics — did not apply. The bar became
~0.7% unarmed.

**Three routes were built and measured before one worked.** Carrying an
accounting mode costs 0.466%/0.697%/0.623% unarmed and, alone, buys nothing —
the counter hooks erase, the mode does not. Inferring freshness from existing
counters is **fail-open**: 42 of 52 walked fields have no covering counter, and
it reused a stale low total on the reproduction itself. Maintaining
unconditionally costs 2.4%-3.1% for the evaluation stack **alone**, one owner
of 52 — which is precisely what the mode is for, and refuted the director's
claim that the mode bought nothing once an unarmed cost was accepted.

**A stubbed measurement decided whether to continue.** Replacing the session
term with an O(1) stub — deliberately incorrect, purely for timing — removed
99.79%-99.99% of the armed overhead, taking the worst shape from 801 714 469
instructions to 5 374 916, with a residual of ~50 instructions per checkpoint
that disassembly fully accounts for. That bounded the prize before the work was
bought.

**The user then ruled the line's method, not just its direction.** Asked to
choose between building all 52 owners and probing the distribution first, the
answer was to build: *所有路径都要确定地趟过，量过真实的代价才能得出结论，可以
回退，可以浪费时间，但不能不知道*. A bar became a data point; "stop here" stopped
being an option to offer beside an unwalked path.

**The result.** All 52 owners maintain their term behind the mode; the armed
checkpoint reads the maintained total; cold arming's double walk is gone.
Armed overhead removed: **93.206%-99.858%**, landing essentially on the
stub's ceiling. Unarmed cost: **+0.965% / +3.239% / +1.018%**, recorded rather
than tuned.

**And the curve paid for itself.** Of ownvalue's 3.24 points, **2.26 come from
two batches** — options and syntax information, then exceptions, scopes and
sealed units — while the other ten add under one point between them. A
distribution probe would have measured how walk cost is spread; only building
it measured how *maintenance* cost is spread, and those are different
questions.

**Method, in the order it cost the most.**

- *A favourable number you cannot explain is not a pass.* brief-519's counter
  arm was faster than base across three families with 0.0002% spreads. It was
  an inlining artifact — the counter build inlined `evaluate_inner_unstable`
  into its caller — and the real cost appeared only once inlining and field
  layout were held constant. It was caught because the brief required the
  anomaly to be explained before it was banked.
- *Judge a panel against itself before believing it.* brief-514's timing panel
  showed 64 of 120 labels positive and nine consistently slower. Splitting its
  own four base runs two ways gives 69/120 and 72/120 positive, 28 and 30
  consistently slower, and a worst outlier of +18.43% — larger than anything
  the candidate did. The label flagged hardest against the candidate moves
  +18.43% between two runs of the identical binary.
- *A vertical slice cannot prove a benefit that only exists at completion.*
  brief-522 maintained one owner and still walked the other 51, so it paid the
  walk twice and every armed number was worse. That was the brief's design
  fault, not the worker's.
- *Capture to a file, then parse.* Piping a long `cargo test` into a filter
  truncated it: 2 456 passed across 30 groups instead of 2 491 across 33. The
  number was green and wrong, and the project's own rule already said to
  persist raw output before parsing.
- *`git checkout <branch> -- files` stages them*, so a following `--amend`
  swallowed un-gated source into a docs commit and briefly put a red tree on
  main. Recovered with `reset --soft` plus `restore --staged`.
- *Two dispatches went out with a bare `&`*, which the project forbids because
  the run is then untracked. Both were killed and re-dispatched. The shared
  cause with the item above: compressing several steps into one command to
  save a turn.

## 2026-09-22 → 09-28 — the retention line: a defect class, a failed route, and the first certified region (briefs 524-546)

The line opened on 165 `Incremental` declarations that nothing enforced — the
validator's `Incremental` arm was empty and the validator itself is an
off-by-default diagnostic. It ended with the option region's two bodies
certified by a local retention lint, and the reasons it took the shape it did
are worth more than the certificates.

**The declaration was not the defect.** brief-524 found 162 of the 165 need a
local builder, zero discharged by the maintained session total — the
director's premise that A-E had covered 35-40 was wrong, and `Print` was the
counter-example. brief-525 then censused all 643 declarations under one
criterion and found at least 497 in the class, with only 56 outside it, 55 of
them handlers-free registrations. Essentially every builtin that runs code
builds its result before it becomes session state or the in-flight
expression. The class split in two: **N**, where guest code drives unbounded
growth (the hole), and **P**, bounded by a constant factor of already-charged
input (a precision debt). The user ruled N only, P a documented bound.

**Whole-closure traversal was built, measured, and abandoned.** A restricted
Rust type-and-effect checker, a rejection census (57 661 sites over 7 057
definitions) and a compiler oracle — rustc used as an oracle for Rust facts
about rum's own source, which is not the Testing Policy's banned shape — all
worked. Then brief-536 measured the option region's closure: 868 bodies and
5 763 sites and still not an upper bound, because every N region calls
`Session::evaluate`. That is the evaluator, and proving it means proving guest
behaviour, which is the runtime ceiling's job. The theory says the same thing:
aliasing is undecidable (Landi 1992, Ramalingam 1994), context-sensitive data
dependence is undecidable (Reps 2000), and IFDS needs the known call graph that
open trait dispatch denies (Reps, Horwitz, Sagiv 1995). Rice's theorem, often
cited for this, does not apply to a sound and deliberately incomplete checker.
`docs/dev/retention-closure-design.md` records all of it.

**Composition replaced it.** A body is covered when no raw owned container is
live across a call that can run guest code, and every call reaches a mark — a
covered body, an installed contract, or an admitted construct. The second
condition is a lookup, not a walk. The first was stated wrongly at first as
"constructs no raw container"; the protocol's own probe, moving a fresh
`String` straight into a lease, showed the hazard is retention across
evaluation, not construction. Contracts were defined before anything relied on
them (brief-537: sixteen boundaries for the option region, one assumed), then
the lease protocol and activation bridge (538), then the lint (539-540).

**The option region, in six steps.** Local reads (542), the leased
delayed-option rule and its continuation (543), the leased snapshot,
accumulator and handoff under a runtime certified domain (544), erasing the
unarmed cost (545), and two `LocalCovered` rows in `promoted.tsv` (546). The
compile-time seal on `OptionReentry` could not be expressed — its implementor
lives in another crate and Rust visibility is crate-granular — so the user
ruled a runtime-enforced domain generated from one declaration shared with the
lint. Its public installer lets a host assert its own trust; that is recorded
in every certificate's exclusions rather than hidden.

**Method, in the order it cost the most.**

- *A misclassification costs whole dispatches.* brief-540 stopped twice, once
  because the director called `Expr == Expr` an operator admission when it is a
  call to a trait impl, and brief-541 was made design-only for that reason.
- *Stop-loss keyed on progress must count compile errors and failing tests.*
  brief-544 stopped twice on repair work — compile errors, then stale
  fingerprints — because the brief counted only rejections and falsifiers. The
  template now carries the rule so a brief inherits it.
- *Instruction counts decide; timing on this host does not.* Timing reported
  +7-12%, +145% and +110% regressions that per-iteration instruction slopes
  refuted, and once reported a real regression as a 50% speedup. The armed
  option cost looked superlinear in percentage terms; a two-point linear fit
  predicted the intermediate rows within 0.12 points.
- *The director's instruments were wrong three times and each was caught:* a
  turbofish type assertion that still coerces (the oracle caught it), an oracle
  cache key that omitted the harness scripts (the worker caught it), and two
  instruction panels that hung because `rum-cli` reads stdin after a script and
  the director's shell never closes it.
- *Protect the scene.* A reboot wiped a dispatch log mid-line; logs and scratch
  now live in the git-ignored `.temp/`, and red trees are preserved on side refs
  built with a temporary index, never by moving HEAD.

## 2026-09-28 — the N map, and a branch: 86 builtins resume past their own work (briefs 547-550)

**brief-547 settled the retention line's map.** All 643 declarations re-read
against the current tree: N 195, P 291, OutOfClass 56, Unsettled 101. The 147
rows carried from brief-524 split 90/19/0/38, and a screen of the P rows moved
12 to N. Grouping by named helper had made the leverage look flat after the
option path (31, then 5, 4, 3, …, 23 singletons); grouping by the mechanism a
conversion needs gave 13 regions and seven new protocol surfaces, and refuted
the director's own hypothesis that the control family was one single-slot
shape — it is three (end an unused value's lifetime with no lease at all, a
needed value slot, a payload-bearing outcome). The retention line's order is now
E0 → a lint exporter for named bodies → settle the 101 → contracts and
admissions → surfaces before users → regions by leverage.

**It also found Scan resuming past its loop,** and that became a branch (user
ruling). When guest code requests input mid-builtin, frames are pushed as the
suspension propagates and driven inner-to-outer on resume; a Rust function with
work left that pushes no frame, or an identity frame, hands the inner value
outward as its own result. brief-530 had repaired one instance on the option
path; nobody had swept the class. **brief-548 swept it** with a rum-against-rum
oracle — the resumed path must equal the queued-input path in value and side
effects, with the suspension asserted — and found 86 `Lost` sites in eight
groups (option readers 29, rule and held-pattern normalization 16, the lazy
pull machine 14, trace filtering and `Sow` 14, the schema validator 7, and
smaller groups), 38 `Transformed` boundaries, and 495 unsettled pairs. The
director re-ran all 195 probes with identical results.

**The loader was worse than a lost continuation.** Guest `Get` converted a
suspension into `$Failed` while the input request stayed installed: a lone
`Get` of a file that asks for input wedges the session, and a second input in
the same evaluation fails the kernel's `assert!` that a new suspension must not
replace a live owner. `ValueQ` and `Monitor` swallow correctly. The user ruled
guest loading resumable (the host batch APIs keep their documented
non-resumability). brief-549 designed it and audited all 38 boundaries; the
front ends (REPL, synchronous Jupyter execute, CLI helpers) also leave their
session blocked after reporting input.

**Two resource-limit bypasses on the same path, director-confirmed.** Each form
of a guest-loaded file renewed the request budget:
`MemoryConstrained[Get[...], 100000]` over a 50 000-element file returned
`50000` where the inline form aborts, and three ~465 ms forms loaded by `Get`
ran 1414 ms under an 800 ms request timeout that stops them inline at 801 ms.
Both need `FileRead`. brief-550 makes a guest load inherit the enclosing
budget.

**brief-550 was stopped by the host, not the work.** Its dispatch was killed
under host memory pressure after every permanent red went green (45), the
white-box (8/8) and host (4/4) tests passed and the docs were regenerated. The
un-gated tree is left in the working copy on purpose and preserved on
`refs/wip/brief-550-interrupted`; the brief carries a resume note.

Method, in the order it cost the most:

- *The director's budget probes measured the wrong limit three times* — the
  generated-element limit, the per-`Do` iteration limit, and a `Do[Null, …]`
  body that never reaches an interrupt poll — before the fourth, with a
  calibrated per-form cost that scaled linearly, showed the deadline renewal.
  Each control has to fire before its treatment is read.
- *A cleared prompt is not a retired owner.* Every conversion of a suspension
  into a value must abandon the pending owner exactly once; the loader's own
  snapshot restore could never do it, because the owner is not in the snapshot.
- *The worker ran `git status` at bootstrap twice despite the brief's first
  line forbidding it* (548, 550; 549 ran `pwd`). Harmless and disclosed each
  time, but the prohibition is not self-enforcing.

## 2026-09-29 → 30 — the suspension branch: batch boundary, every front end, the first loops, and a machine that aborts (briefs 551-562)

brief-551 closed what the re-probe of brief-550 found at the batch boundary: a
public `eval_file` suspended inside a nested guest load panicked on cancel,
and a failed `restore_from_bytes` on input wedged the session; a request that
completes while leaving a suspension owner now fails a test/debug-build check.
brief-552 established by execution what each front end does when guest code
asks for input, and the user accepted all ten recommendations (U1-U10) and two
dependencies (`rustix`, `signal-hook`). Briefs 553-558 carried them out: the
REPL answers from its next line and resumes (masked input with echo off,
Ctrl-C cancels to the prompt), the CLI's helpers retire the request they
admitted, and Jupyter abandons, cancels and emits each output once. Found and
not queued: every front end loses the relative order of prints and messages
within a request, because the kernel keeps them in separate buffers.

brief-559 gave `Scan`, `OptionValue`, `ReplaceRepeated`, `Dot` and
`Differences` frames that resume into their own loops (lost continuations
86 -> 80); brief-560 moved every capture into cold helpers and restored the
ordinary loops, leaving `Differences` +0.82% instructions after ten measured
candidates — recorded, attributed by callgrind to `difference_expr` losing
its inlining. Two host kills under memory pressure led to the user's ruling
that all runs are serial, which exposed a latent defect in brief-555's CLI
test harness.

G2, the lazy pull machine, opened with a design (brief-561): one frame owns
the machine, state moves into it only on `InputRequired`, three completion
frames finish `Normal`, `Take` and `First`. Probing the adjacent boundary
before writing the implementation brief, the director found the machine
already aborts the process: a `LazyMap` chain built by a loop overflows the
native stack on its first realization at ~2.2k stages on the ruled 2 MiB stack
(944 bytes per recursive constructor activation), with no guard anywhere and
only an inventory row to record the self-reaching `Box` chain. brief-562
revised the design into a flat source plus stage vector under an explicit
cursor — every walk iterative, the suspension state the cursor itself — and
the director ruled on its three open questions (a cancellation boundary at
every lazy callback, the 256-transition poll rejected, the inventory map
authorized as written).

Method:

- *Probe the structure a design keeps, not only the defect it fixes.* The
  abort predates the branch; the design would have added four more walks of
  the shape that causes it.
- *Prefer the representation in which the hazard cannot be written.* A guard
  on every recursive walk fails where the walk is `Clone`, tracing or `Drop`,
  which cannot return its error.
- *An adjacent finding is not the branch's scope unless the branch rewrites
  it.* The duplicated `First::nofirst` on an empty lazy list is recorded, not
  fixed here; the stack abort is in G2 because G2 rewrites that machine.

## 2026-10-04 → 06 — the suspension branch: three walks made flat, a typed option protocol, and every brief-548 group repaired (briefs 563-582)

G2, the lazy pull machine, was built as brief-562 chose it (brief-563): one
source plus a stage vector under an indexed down/up cursor, every walk
iterative, the suspension state the cursor itself. Structural checks forbid a
recursive call edge in the lazy sources and a self-reaching lazy type, each
with a falsifier, and a 20 000-stage chain on the ruled 2 MiB stack that
aborted the release CLI now returns exact values. brief-564 resumed
`LazyTable`'s bounds into its plan (lost 80 -> 69 -> 66). brief-565 then
stopped at its stop-loss on a measurement error that was the director's: the
release profile has 16 codegen units, so adding code anywhere repartitions
them and moves inlining in untouched code. With one unit, G2 costs the
non-lazy path nothing; brief-566 cut the shallow lazy residual to six labels at
+0.07% to +0.40%, recorded as the price of depth safety.

Each later group's re-probe asked what else the walk it rewrites could do, and
G3 and G4 had the same answer as G2. The schema validator's own walk aborts the
process at list depth 6 000 (the director's first abort probe had measured the
matcher instead, because `ConformSchema` holds its schema); brief-568 made it
one flat work stack with one mutable path buffer and one frame (66 -> 59), and
brief-569 left small lists faster than before G3. G4 (briefs 570-575) is
thirteen repairs and three retirements of dead matcher adapters: the rule
left-hand-side walk and the held-pattern resolver became explicit work stacks
(59 -> 43). Its second tuning round settled each walker's base cases in a
small entry before the out-of-line driver, with falsifiers driving every base
case through both, and `MatchQ[5, _Integer]` ended 3.3% faster than before G4;
the residuals (+1.4% to +4.3%, populated structural walks only) are recorded.

G5 (briefs 576-579) is a typed read-and-complete boundary: an option reader
takes the caller's continuation and returns the completed builtin's `Expr`, so
a read without its continuation does not compile, and read order stays
observable (after-target reads, `Names`' reversed order, reads before arity
checks). The retention line's certified option bodies stayed verbatim (43 ->
14). G7 (briefs 580-582) made `Sow`'s tag matching and the structured trace
filters resume, the filters as one flat work stack (14 -> 0). **All 195 of
brief-548's probes agree on `033711f9`.**

Found outside the branch: the matcher aborts on deep literals (`MatchQ[v, w]`
at 1 200 levels) and several `Expr`-depth helpers recurse unguarded, both
queued for the hardening line; and guest recursion stops at 60-80 levels,
because the native guard is fixed at 1 MiB and one guest level costs 13-15 KB,
recorded for the user to place.

Method:

- *An instruction gate finer than about 1% needs one codegen unit on both
  arms.* Sixteen units moved untouched code by ~0.6%; `TEMPLATE.md` pitfall 8.
- *Attribute an abort to the code that ran, not the code the probe named.* A
  holding attribute decides which walk a probe reaches.
- *A design can be wrong about depth in the other direction.* brief-580 showed
  a captured trace expression can be deeper than the evaluation that produced
  it, so the flat walk was chosen for enforceability, not on an empirical
  bound.
- *A fast entry is not a second algorithm when a falsifier drives every base
  case through both it and the driver.* The director's earlier rule against
  it had blocked G4's best candidate.

## 2026-10-06 → 13 — the re-census, a catalog-wide oracle, and the three groups the census could not see (briefs 583-599)

brief-583 re-ran the census on the current tree and resolved every Unsettled
edge, 495 of which brief-548 had left open: 2 443 candidate edges, all 86 old
Lost sites repaired or removed, and **4 new Lost sites in two groups**, both
long-standing. G9 (brief-584): list-destructuring assignment resumed to one
element and skipped the later stores; it now stores through an owned cursor.
G10 (brief-585): a suspended direct `Default` lookup did not continue the
down-value scan. The worker stopped on +0.26% instructions, and the cause was
one attribute: the shared helper was `#[inline(always)]`, so its body existed
at two call sites and the indexed-scan helper lost LLVM's single-call-site
inlining bonus. Lost 4 -> 0.

E1 (briefs 586-599) built the oracle the census cannot replace: a reviewed
manifest row for every installed spec (643 default, 637 no-default), a
two-session runner comparing resumed against queued input in value and side
effects, falsifiers watched to fail, and an owed ledger drained family by
family. The manifest had been written by inference; the drains removed **68
declared roles that did not exist**, each verified from source. It closed with
an exact equality — 438 declared entries, each with an executed descriptor, in
both builds — and the ledger mechanism deleted.

The drains found what the census could not, because the frames existed but
were incomplete. **G11** (briefs 593-595): `EvaluationData` resumed to the whole
capture association, and `TestReport` **panicked the kernel** when a second
case suspended. Of 32 production `Arc::try_unwrap` sites, three were
guest-reachable panics, each a consumer finalizing an accumulator node a still
rooted frame shares; one private completion cursor now moves a unique node or
shallow-clones a shared one. Output inside an unsettled capture became
provisional through one declarative output boundary and one pause builder,
which also caught `Quiet` leaking messages at a pause. Its third amendment
found that the resume driver, on a re-pause, restored only the guest form and
left the guest's tail, checkpoints, deadlines and memory limit live. **G12**
(brief-597): `ToExpression[str, form, head]` resumed through a fieldless frame
and never applied `head`. Sweeping 600 sites for frames carrying less state
than the work after them found no other instance of that shape and **G13**:
typed continuation state crosses frames in-band as `Expr` under
``System`Internal*`` heads that guest source can write, decoded leniently or by
panic.

Method:

- *A census of call edges cannot see an incomplete frame.* G11 and G12 had
  frames; only executing resumed against queued showed what they dropped.
- *Verify a manifest row from source before writing it.* 68 of 506 inferred
  roles did not exist.
- *When a split shares a helper between a hot path and a cold resume, look
  first for the inlining decision that flipped.*
- *A resume driver must restore everything it displaces.* The test now asserts
  that two pauses leave identical live state. The second amendment, which
  ruled the opposite, was the director's error.

## 2026-10-13 → 17 — G13: five design rounds, type-then-flip, the first eight steps, and G14 (briefs 600-616)

G13's design took five rounds and two models. GLM drafted rounds 1-3 (600,
601, 603; round 1 was begun by Codex and cut by the content filter), and Codex
reviewed rounds 2 and 3 (602, 604). Each review found faults the drafts had
missed; the second required seven changes, among them that `EvaluationData`
turns any error into `$Failed`, so a mismatch error could be swallowed, and
that round 3's propagation join compared `yes` with `YES`, so 617 of 619 edges
read as frameless. Round 4 (605) was Codex forked from its own review session.
The director accepted it except the atomic 528-file conversion, replaced by
**type-then-flip**: a per-type transport mode, so each typing step exchanges
only `Value` with legacy neighbours and lands green with its own gate, then
each legacy type flips under its own corpus. Atomicity bought no detection,
since a missed frameless edge is caught at run time either way. Round 5 (606)
partitioned the work into 78 gated steps; each of its 1 399 obligation rows
has exactly one owning step.

S1 (607) installed the dormant typed API in five dispatches and S2 (608) the
compatible driver in seven; most of their stops were the director's scope
omissions. S2's resume
harness runs on `HostSession::resume_with_input`, because `rum-cli` retires
every input suspension. S3 (609) found that the typed finalizer had reboxed
itself on every finish since S1, visible only once a measured path executed
it. S4 (610) was Kimi K3's first brief, and no macro expansion it tried
reached an at-or-below-base stack bar at 16-byte granularity; the bar became
the named closure environment, attributed per slot. S5 (611) repaired the
skipped `Some` (guest-written ``System`InternalContinuationSome`` values were
decoded as the continuation's own wrapper) and found the insertion shape the
design describes: the zero-sized witness and the request cross the cold call,
not the producer's `Result`. That cut the stack residual from 2 320 B to near
the base and deleted fourteen `unreachable!` arms.

**G14** (612-613) was found when S5's 2 000-resume harness stopped at 1 021
resumes. Resume never unwound the evaluation stack it restored, a defect as
old as resumable input (`b51dd794`, 2026-03-20). A repeated identical
expression reported a false cycle after one pause, changing expressions leaked
one depth per resume, and the per-level resource epilogue was skipped on
resume, which let `TimeConstrained`/`MemoryConstrained` be bypassed. Every
queue cell now carries the context it was inserted under, and an inline
return event runs the skipped checks. Its timing residual (+20.8% on one label
at identical instructions) was settled causally: with every function 64-byte
aligned in both arms it is +1.3%.

S6 (614), S7 (615) and S8 (616) typed guards and binding completion, Part
selectors and held assignment (the first typed frames in `rum-stdlib`, and the
`Arc<OnceLock>` selector protocol deleted), and `ReplaceAll` with the
traversal collectors. Their pre-probes found no defect (22, 32 and 48 shapes,
all equal). S8's worker run was cut by Codex's usage limit at its timing
stage, and the director finished the gate. 70 of 606 production frame
implementations are typed; S9 (brief-617, control and cleanup) is committed
and waits for its worker.

Method:

- *A design reviewed by a second model is worth the round.* Both reviews found
  faults that three drafting rounds had not.
- *Dormant code is unmeasured code.*
- *A shape that needs an `unreachable!` arm for a state the types allow points
  at the shape nobody tried.*
- *A class falsifier must vary the depth between the pause and the repeating
  construct, and repeat an identical expression.* G14 hid behind a drop-two
  heuristic that patched the instances seen.
- *Settle an identical-instruction timing residual causally*, by aligning every
  function in both arms. *A timing probe runs the benchmark program itself,
  whole*: a cut probe hit a fast path 250 times cheaper than its label.
- *Run a new probe shape once before writing it into a brief.* S8's first
  dispatch stopped on a stack shape of the director's that recursed on the
  `List` head.

## 2026-10-17 → 18 — G13 S9: control and cleanup, two new workers, and a copy the worker's instrument could not see (brief 617)

S9 (brief-617) typed the 24 frames and the finalizer that suspend inside
`CheckAbort`, `AbortProtect`, `Assert`, `Check`, `Quiet`, `WithCleanup`,
`CatchExceptions` and `Catch`. That leaves 95 of 606 production frame
implementations typed. This family resumes on *errors*: every
non-`InputRequired` outcome reaches the continuation it reached at S8.
`WithCleanup`'s finalizer is a typed `SlotFinalizer`, and its `Arc` stays
unique because the typed driver drops the frame and its queue entry before
`finish`. Committed as `f4a7c6b0`.

Codex's quota was out, so the user tried two new workers. The dispatcher
gained `BRIEF_WORKER=agy` and `BRIEF_WORKER=grk`. Each refuses to dispatch
unless the developer's config keeps the worker's OS fence.
- **agy** (Antigravity, Gemini 4 Argon) runs in Linux namespaces with a single
  repository write grant.
- **grk** (Grok Build) runs under a strict Landlock profile with `.git` denied.
  Before each dispatch the script runs a census session that reads back the
  toolset the model is shown and the kernel profile grok applied.

The first grok probe showed that grok loads Claude Code's MCP servers by
default, including a Wolfram engine. The census now refuses any MCP server.

- **agy did the conversion.** Its weekly quota ran out after 2 h 20 min,
  before it wrote any report. Its checkpoints had stopped 26 minutes in.
- **grk took the tree over.** It reviewed run 1, ran every gate and wrote a
  report. Two of its claims were false. It said the nine native ZeroMQ tests
  "did not skip", although its sandbox cannot bind loopback. It also reported
  memcpy "at or below base 164/164": libc had no symbols in its sandbox and
  the counter matched names, so every label read a constant.
- **The director's gate found the defect.** Five sites handed the whole
  136-byte `Result` to the continuation, which cost one memcpy per call on
  seven ordinary labels.
- **grk's quota ran out again** while it applied the fix, and the director
  finished by user ruling. The fix grk had applied added a copy on
  `CheckAbort`'s abort path and grew `cleanup-body` by about 300 B per level.
  The director measured five `WithCleanup` variants, and only one was at or
  below base on both copies and stack.

The final gate passed:
- copies: 0 of 164 labels above base;
- stack: all 17 shapes at or below base, S9's smaller;
- instructions: every S9 ordinary label at or below base;
- timing: 0 labels above base on the aligned rebuild;
- oracle, census and the workspace green, with the nine ZeroMQ tests run
  natively;
- probes: 186 pairs, all equal.

Both headless CLIs stayed alive after their last turn and had to be
interrupted.

Method:

- *Calibrate a worker's instrument in the worker's environment.* A
  name-matched count in an unsymbolized sandbox read a constant, and the report
  quoted it as a pass.
- *Before trusting a worker's green, ask what its sandbox can execute.* This is
  the 09-02 ZeroMQ lesson again: a skip still prints `ok`.
- *A copy fix is not done until the stack shapes are rerun.* The arms that
  removed the memcpy put their temporaries in the inlined dispatcher's frame.
- *Make a worker's checkpoints a stop condition, and write it into the
  continuation.* agy lost its whole record by not writing one. grk, told to
  checkpoint, left a tree the director could finish from.
