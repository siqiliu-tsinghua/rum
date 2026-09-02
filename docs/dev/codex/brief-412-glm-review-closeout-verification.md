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

---

# brief-412 验证报告

## 1. 环境能力声明(逐字)

`cargo test --workspace --offline` 完整跑了两次，均以退出码 0 结束(第一次 `| tail -80` 截掉了记录，故重跑一次存全量日志；两次结果一致)。合计 **2290 passed, 0 failed, 7 ignored**(7 个 ignored = `rum-host/tests/stress.rs` 2 个 + rum-runtime 2 个 + rum-stdlib 3 个，与既往 gate 一致)。关键行逐字：

```
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-33e27c27e8b88c34)
running 106 tests
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
```

九个 native ZeroMQ 测试在本机**真实运行**(非 skip、非 ignored;未设置 `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`),逐字：

```
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... ok
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... ok
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... ok
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... ok
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... ok
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... ok
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... ok
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... ok
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... ok
```

本机可绑定 loopback(`zeromq_native_test_support.rs:335-354` 的探测会在不可绑定时 panic)。每个 native 测试我只观察到一次通过，**不作任何稳定性/频次声明**。

## 2. 八项 finding 逐项裁定

**F1(HIGH)lhs 准备在模式区域内求值 — 393 修复：已关闭(类级)。**
- 注册表是唯一来源:`pattern_syntax.rs:52-72` 的 `matcher_pattern_heads!` 列出 19 个头；准备侧 `syntax_policy.rs:42` 经 `matcher_pattern_head_name` 查注册表，未注册头一律 `Ordinary`。
- 匹配器侧全部识别点都经注册表:`matcher_and_symbols.rs`(38 处 `MatcherPatternHead`/`matcher_pattern_head`,含 `pattern_named_pattern:162`、`pattern_condition:174`、`pattern_optional:212`、`pattern_blank_head:567`)、`options/patterns.rs:89-152`、`substitution/function_and_subst_helpers.rs:26`、`kernel_data/rewrite_data/cases/match_q.rs:167,196`。
- 类而非两例：测试 `syntax_policy.rs:151` `every_registered_matcher_head_has_an_explicit_lhs_policy` 对 `MatcherPatternHead::ALL` 逐一要求显式非 `Ordinary` 策略 — 新增头入注册表而不给策略会红。
- 我自己的活探针(经 `rum-cli`,逐字)：

```
{HoldPattern[fV[Verbatim[_typeAlias]]] :> 7}        (* Verbatim 内容保持原始 *)
{HoldPattern[fP[PatternSequence[sideCall[], b_]]] :> 8}
0                                                    (* sideCall 未运行,计数为 0 *)
{HoldPattern[fN[x_]] :> 9}                          (* 普通 binder 不变 *)
{HoldPattern[fTT[_Integer?IntegerQ]]] :> 11}        (* 过矫正检查:PatternTest 模式操作数仍准备,谓词延迟 *)
```

- 行为变更面：对照 `d423d2d3~1` 的旧策略，缺的正是那七个名字；`HoldPattern`/`OptionsPattern` 是注册表相对旧 matcher 文件十七名的两个“收编”项，旧准备策略本就保护 `HoldPattern`。指南逐形式记录在 `docs/guide/evaluation.md:123-131`。

**F2(HIGH)稳定性断言类 — 394 修复：已关闭(类级)，我独立复核了 sweep。**
- 通道机制从源码验证:`stability.rs:36-38`(有 identity → generated 通道)、`model/cache.rs:541-544`(`intersects` 按通道独立 AND,通道之间永不相交)。
- 各测试构造核对无误：identity 目标(`Symbol::with_identity`,如 `evaluation_cache/tests.rs:95`、`association_write_index.rs:34`、`association_write_recovery.rs:88`)写 generated 通道；其 Association/List 值只含 System 符号与整数(regular 通道、整数为空 bloom,`stability.rs:215`)。任何 interner 顺序下 `intersects` 化为 `(X & 0) || (0 & Y)`。
- 断言未删除：三个文件的 diff 全是搬移+新增(如 `evaluation_cache/tests.rs` 的 stale/dirty 断言从普通 session 移到 identity session,语义断言留在普通符号上)。
- 我重做了 sweep 搜索：其余 `index_builds`/`indexed_probes`/`ptr_key` 断言点(`association_lookup_cache.rs:31-69`、`association_part_cursor.rs:85-142`、`valueq_stable_shortcut.rs`、`part_assignment.rs:118`)均无“种子证明与消费之间发生普通符号写”的暴露形状(无写、或强制 fallback、或缓存清空重置 bloom、或断言的是新算判定而非缓存驻留)— 394 的“五个就是全部”在我这里的复推成立。**说明：这是从源码推的，非 sweep 复跑，见 §6。**

**F3(HIGH)`install_system_down_value` 存原始 lhs — 395 修复：已关闭，且在 crate 边界经我亲手编译验证。**
- 现签名 `session_api.rs:294` 取 `TrustedSystemDownValueLhs`;字段与 `new` 私有(`session_api.rs:25`),对外只有 `default_for_plus`/`default_for_times` 两个具名目录身份(`session_api.rs:43-50`);`prepared_store.rs:386-396` 另有 debug 断言 lhs 必须归类到声明 owner。
- 我在 `/tmp` 新建外部 crate(依赖 path 引用，`cargo check --offline`,已删除)，逐字错误：

```
error[E0061]: this method takes 2 arguments but 3 arguments were supplied
error[E0624]: associated function `from_continuation_expr` is private
error[E0624]: associated function `new` is private
```

- 合法路径同样编译通过(安装 `Default[Plus]`/`Default[Times]`、调 `evaluate_part_selectors_owned_for_builtin`)。

**F4 `EvaluatedPartSelectors` 可经续体解码器伪造 — 395 修复：已关闭。**
- `evaluated_part_selectors.rs:103` `from_continuation_expr` 已降为 `pub(in crate::runtime)`;`from_evaluated`(`:57`)私有；对外路径只有 `evaluate_part_selectors_owned_for_builtin`(`:133`)的真实恰好一次求值，悬挂时经 `EvaluatedPartSelectorsContinuation::finish`(`:33`)只能取回运行时自己的 frame 填入的 OnceLock,无法注入任意选择器。上面的 E0624 即我的外部复证。

**F5 选择器次序契约被无条件陈述 — 396 item 4 修复：已关闭。**
- `docs/guide/evaluation.md:172-179` 现为 "When a mutation reaches target selection, the `Part` selectors evaluate exactly once…" 并给 `AssociateTo`/`KeyDropFrom` 加了 "after their evaluated mutation specification passes validation"。
- 代码侧仍在 `associations/mutation.rs:170-177`(spec 非法 → `invlb`,先于 `normalize_part_mutation_target`)。邻句核对：七个二元头与 Increment 家族的陈述无条件但代码无对应分支，正确。(小瑕疵见 §5 F-new-2 下的附注。)

**F6 比较链契约两部无测试 — 396 item 3 修复：已关闭，实测绿。**
- `crates/rum-stdlib/src/tests/logic/inequality.rs:31-35`(源码面 `a < b < c` → `Less[a, b, c]`)与 `:82-98`(左/中/右三个计数器含最左操作数)。逐字运行：

```
test tests::logic::inequality::uniform_less_chain_keeps_its_nary_head ... ok
test tests::logic::inequality::uniform_less_chain_evaluates_every_operand_once ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 823 filtered out; finished in 0.02s
```

**F7 取消测试靠 sleep 同步 — 396 item 1(后被 404 重做)：已关闭，残余一项已如实记录。**
- 四个站点现均以 `Print["evaluation-started"]` 经 `mpsc` 通道建立 happens-before:`rum-host/tests/stress.rs:58-70`、`lifecycle_contract.rs:144-186`、`rum-host/src/tests/eval_contract.rs:220-246`、`runtime/tests/session_requests/api.rs:35-50`(file-request 变体 `:669-710`)。
- 唯一残余:`stress.rs:122-145` `cancel_rewrite_loop_stability_10_cycles` 仍 sleep 25 ms,但它 `#[ignore]`、带运行说明，且 `TODO.md:742-744` 明示"replacing that test synchronization remains separate cleanup" — 不是悄悄丢掉。

**F8 native IOPub 测试依赖 50 ms 传播延迟 — 396 item 2 当年并未修复，现由 411 关闭。**
- 396 的 `acknowledge_iopub_subscription` 走独立 Req/Rep 心跳，不构成 PUB/SUB 屏障 — 这是 brief-406 自己发现并记录在 `TODO.md:758-763` 的。
- 现行 `zeromq_native_test_support.rs:398-466`:kernel_info 哨兵经 Shell 发出、其 IOPub 状态消息走后端**自己的 PUB 路径**，循环直到 SUB 实际收到带哨兵 parent_header 的帧才返回，并断言哨兵未扰动 adapter 状态；配套 `collect_iopub_for_parent`(`:264-289`)按目标 parent 收集、以其终态 Idle 停止 — “下四帧”假设已消除。四个测试文件均已接入(各文件 grep 证实)，且九个 native 测试在本机跑过全绿。

## 3. 惯例 vs 结构，及 crate 边界裁定

- **F1(393)**:核心是结构 — 注册表把“准备策略集”与“匹配器识别集”焊在同一枚举上，测试强制策略穷尽。**一个未强制的外沿要如实说**：没有任何机制阻止未来某人在匹配器里新写一个绕过注册表的原始字符串识别点(我的 grep 不是门禁)；今天在 matching/preparation 路径里这样的点为零，但“新增识别必须入注册表”这一半仍是惯例。非 crate 边界问题(注册表 `pub(in crate::runtime)`,外部不可见)。
- **F2(394)**:构造级结构(通道不相交是机制性质，不是纪律)。它不能阻止未来再写出暴露的新测试 — 测试代码的固有属性。源扫描：无。
- **F3/F4(395)**:**结构，且属性在 crate 边界成立** — 我用全新外部 crate 的编译错误(E0061/E0624)亲自证实。`definition_store_sealing.rs:133-155` 仍是源扫描，但边界不依赖它(它只防仓库内回归)；这符合 brief-412 要的判别标准。
- **F5/F6/F7(396 的文档与测试项)**:测试与措辞，无强制机制可言；F7 的同步是真实信号(handshake),非时长。
- **F8(411)**:结构(真实 PUB 路径往返 + parent 作用域收集)，另有 skip-loudly 门(`zeromq_native_test_support.rs:335-354`)。

## 4. briefs 397-411 是否削弱了这些修复：**未发现**

核对过全部落在修复文件上的后续改动：
- `fc959898`(abandonment 协议)对 `evaluated_part_selectors.rs` 只是给两个 frame 加 `PureFrameAbandon` derive;对 `kernel_api.rs` 是加导出。
- `a81b703e`/`1373182f`(请求级取消)对 `session_api.rs`/`kernel_api.rs` 是取消机制本身;`evaluation_cache/tests.rs` 只是 `association_lvalue_direct_update_polls_interrupt_before_mutation` 迁到新请求 API,断言集未变。
- `match_q.rs`/`options/patterns.rs` 的改动同为 abandonment 迁移。
- 我的**外部探针是在 HEAD 上编译的**，所以边界在当前树成立，不是在修复点上成立。
- 397-411 新增测试里没有新的 proof/计数/指针断言形状(`git diff aa8644cf..HEAD -- 'crates/**/test*'` 中无 `index_builds|ptr_key|has_valid_call_stability_proof` 新增行)。

## 5. 新发现(按严重度)

1. **(低，文档)`TODO.md:782-785`(比较链覆盖)与 `TODO.md:836-843`(选择器次序契约)缺少关闭标注。** 八项 finding 中六项有 `FIXED/CLOSED` 行，恰是 brief-396 关的这两项没有；而 item 1/item 2 的条目被 404/406-411 改写并标 CLOSED。修复本身是真的(§2 F5/F6 证据)，但条目现状会被误读为仍开放。旁证:`65005e3f`(close the consistency review)只加了节首，`f3fa5215`/`00d24450`/`9e880960` 各标了各自的项，396 的两项无人标。独立确认方式:`git log -S "Comparison-chain coverage omits" -- TODO.md` 后读 `TODO.md:782`。
2. **(信息)`EvaluatedPartSelectorsContinuation::finish` 的失败模式是 panic**(`evaluated_part_selectors.rs:40-42` 的 `.expect`):`#[doc(hidden)]` 的完成句柄若在运行时 frame 填槽前被调用会 panic 而非报错。伪造不变量不受影响(只能取回运行时产物)，仅是误用的失败面。
3. **(信息，已判定非缺陷)残留的并行逐头字符串匹配**，不经注册表:`prepared_store.rs:626-658` `definition_specificity`(特异性启发式)、`definition_analysis_helpers.rs:382-383`(UpSet 目标收集)、`lexical_substitution.rs:396,408,513`(具名模式变换)、`function_and_subst_helpers.rs:42-53` `release_hold_once`(hold 家族表)、`expr_runtime_helpers.rs:375-418`(System 名字分类)。它们不决定“准备是否在模式区域内求值”，修复的不变量不压在它们上面；但若模式语言将来长出新头，这些是同一“清单”形状会率先过时的地方。

## 6. 无法确立的事项

- **0..256 preintern sweep 未复跑**：394 的受控 interner 是临时插桩、已还原，仓库内无可复用工具，而 Scope: NONE 不许我加。我的裁定基于结构论证(`stability.rs:18-38` + `model/cache.rs:541-544` + 各测试构造)，这是源码推理，不是 sweep 观测。
- **目录安装 -0.56% 与 brief-404 的 30 次采样稳定性**未复测 — 均非本次八项 finding 的正确性主张，且后者属 flake 领域，不是本次验证对象。
- **native 测试只观察到各一次通过**；不作频次主张。
- **393 的“无普通路径无条件开销”**：commit 信息未附测量；我从源码判断新代码是同数量级的字符串匹配(19 名 vs 旧较短清单)，但这是推理，不是测量。

git 状态未动(无 commit/branch/checkout/stash/config);/tmp 探针与日志已删除;未改任何仓库文件。
