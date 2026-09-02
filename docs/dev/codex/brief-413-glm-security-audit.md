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

---

brief-413 执行完毕。`Scope: NONE` 遵守：仓库零文件改动（`git status` 干净），所有探针在 `/tmp` 一次性 crate 与 release `rum-cli` 上完成，scratch 已清除。以下为报告。

---

# brief-413 报告 — rum 安全审计

## 1. 环境能力（逐字）

- 工具链：`cargo 1.85.0 (d73d2caf9 2024-12-31)` / `rustc 1.85.0 (4e91de4e4 2025-02-17)`，Linux（CachyOS），`ulimit -s` = 8192。
- `cargo test --offline --workspace`：30 条 `test result:` 行全部为 ok，无 FAILED，进程 `exit=0`。逐字样例（每 crate 的行）：
  ```
  test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s   (rum-jupyter)
  test result: ok. 930 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 3.12s  (rum-runtime)
  test result: ok. 828 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.55s  (rum-stdlib)
  test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
  exit=0
  ```
  （跨 crate 合计 2287 passed 为我用 awk 计算的派生数，非逐字行。）
- **九个 native ZeroMQ 测试在本机真实跑过且通过**（loopback 可绑定），逐字：
  ```
  test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... ok
  test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... ok
  test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... ok
  test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... ok
  ...（共 9 条 native，全部 ok；rum-jupyter 合计 106 passed, 0 failed）
  ```
- 探针载体：`cargo build --offline --release` 的 `rum-cli`（target/release/rum-cli，13 MB），以及 `/tmp/b413_probe`（依赖 `rum-host`、`rum-jupyter` 的 path 依赖一次性 crate，release 构建，已删除）。RSS 取自 `/proc/self/status` 的 `VmRSS`/`VmHWM`。
- 从源码推理（非观测）之处，在各条中明确标注。

## 2. 发现（按严重度排序）

### F1（高）进程全局 symbol interner 只增不减：跨请求、跨会话、无视内存上限、reset() 不可回收

**攻击者得到什么**：任何会话里的任何脚本（`sandboxed()` 的 `{Pure}` 也一样，`Symbol`/`ToExpression` 都是 Pure）可以让宿主进程的内存**永久**增长，速率由脚本选定：每个请求可稳定泄漏约 8 MB（实测），或用循环以约 375 B/符号 × 每请求最多 ~10 万符号泄漏；无内存上限时单请求可先倍增字符串再 `Symbol[...]`，泄漏量实际上不受任何上限约束。`HostSession::reset()` 与每请求的 `eval_memory_limit` 都**不能回收或阻止**它。对一个长驻进程（jupyter kernel、编辑器宿主、每请求一个 session 的多租户服务）这是无法通过任何文档列出的内核机制关闭的耗尽向量，且所有租户共享同一个 interner。

**复现**（`/tmp/b413_probe interner-bignames`，沙箱 + 16 MB 内存上限已武装，10 个请求，每请求 `Symbol[StringJoin[Table["a×8192", {1000}]]]`）逐字：
```
request 0 rss=47396  ... request 9 rss=119856   (每请求净增 ~8 MB)
after reset rss=103964 kb hwm=119140 kb          (reset 后 100 MB 仍在)
```
`interner-count`（12 请求 × 20000 个唯一符号）：RSS 5.8 MB → 96.6 MB，reset 后仍 75.6 MB。

**机制**（`path:line`）：`crates/rum-core/src/expr.rs:47-96` — interner 是进程级 `OnceLock<Mutex<SymbolInterner>>`，`intern_text` 对每个新名字 `Box::leak(value.to_owned().into_boxed_str())`（expr.rs:66），`pairs` HashMap 只插不删，**没有任何清除 API**；`Session` reset 重建会话也不触碰它。内存模型 `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:50-261` 的 `current_memory_proxy_bytes()` 只遍历会话自有结构（definitions、prints、messages、suspension…），**不包含 interner**，所以上限检查对泄漏字节不可见（`constraints.rs:139-162` 的 stride 检查调用的是这个函数）。

**打破的书面承诺**：SECURITY.md「Every session runs under a host-chosen policy」与内存上限契约（resource-limits-and-sandbox.md：`eval_memory_limit` 「installs a modeled-memory ceiling for each request. A breach returns TerminatedEvaluation["MemoryLimit"]」）。没有任何文档披露存在进程级、不可回收、不受上限约束的状态。文档确实要求宿主自建 RSS 看门狗（memory.rs 之外的 `docs/guide/resource-limits-and-sandbox.md`「a native host that requires a hard process-wide memory bound must enforce one outside the kernel」），但该缓解是文档性的，且“永久、reset 无效”这一性质未被任何文档陈述。

### F2（高）模建内存上限在 stride-64 采样间隔内可被乘法增长击穿：实测 256× 瞬时超限，原理上无界，可致进程 abort/OOM

**攻击者得到什么**：在内存上限已武装的沙箱会话里，脚本用字符串自拼接或大整数自乘，让**实际进程 RSS** 在两次全量内存 walk 之间按乘法放大。实测：8 MB 上限 → 峰值 RSS 2.04 GB（`hwm=2142420 kb`），耗时 383 ms，最终以 `TerminatedEvaluation["MemoryLimit"]` 在带内结束——但“带内结束”只在进程活到下一次采样时成立。每步增长倍率（`s <> s <> s <> …` 的元数）与步数不受限制，原理上可让分配在检查到来之前失败：Rust 分配失败是 `handle_alloc_error` **abort**，panic 边界（crash-only 之外）不捕获。64 MB 上限下字符串倍增实测峰值 348 MB（5.5×），大整数自乘实测峰值 1.19 GB（18×）。

**复现**（`/tmp/b413_probe triplegrow`，8 MB 上限，`s = StringRepeat["b",100000]; Do[s = s <> s <> s, {20}]`）逐字：
```
start rss=5752 kb
elapsed=383.238102ms rss=648192 hwm=2142420
  ok term=MemoryLimit ... val=TerminatedEvaluation["MemoryLimit"]
```
`squaring2`（64 MB 上限，`x = 2^2000000; Do[x = x*x, {40}]`）：`elapsed=1.192876634s rss=257508 hwm=1194412`，`term=MemoryLimit`。

**机制**：检查只在每第 64 个表达式评估点做全量 walk（`constraints.rs:9` `ACTIVE_MEMORY_LIMIT_CHECK_STRIDE: u32 = 64`；`constraints.rs:145-159`），间隔期只递减计数器；`constraints.rs:5-8` 的注释宣称间隔期的超额“由既有 per-step element/integer/iteration 预算兜底”——但这些预算限制的是**个数**，不是单步乘法放大的**字节数**。关键缺口：`crates/rum-stdlib/src/strings/text/builtins.rs:13-42` 的 `eval_string_join` **无任何 projected-bytes/element 预检**（直接 `pieces.concat()`）；大整数 `Times` 同样没有（对照：`StringRepeat` 有双预检 `builtins.rs:664-668`，`Power` 有位长预检 `canonical.rs:99-136`，`Table` 有 `iteration/spec.rs:338`）。

**打破的书面承诺**：SECURITY.md「With them, such a request is terminated in-band」——终止确实在带内，但发生在瞬时超限（实测 256×）之后；“not an allocator meter or process RSS measurement” 已声明，但文档未说明超限可以乘法无界，也未说明可能以进程 abort（不可捕获）而非带内终止收场。

### F3（中高）rum-jupyter 手写 JSON 解析器无深度守卫：深度嵌套 wire JSON 使 kernel 进程栈溢出 abort（panic 边界外）

**攻击者得到什么**：kernel 进程死亡（SIGABRT/core dump，`fatal runtime error: stack overflow, aborting`），不可被 `catch_unwind` 捕获。到达条件：能向 kernel 端口发帧。签名策略为 `Required` 时 JSON 解析发生在 `verify_signature` **之后**（`codec.rs:307-308` 先验签再 parse），未验签攻击者到不了解析器；但 (a) 连接文件 `key` 为空或 `signature_scheme` 非 `hmac-sha256` 时策略为 `Disabled`（见 F7），此时**未认证**的本地对端即可触发；(b) 受信任客户端（前端）发深嵌套帧同样触发。

**复现**（`/tmp/b413_probe jsondepth 50000`，`KernelWireConfig::default()` 即 Disabled 签名，`metadata_json` 嵌套 50000 层，调用公开 API `KernelWireCodec::decode_request_message`）逐字：
```
--- depth 100:   decoded ok (no crash)
--- depth 50000:
thread 'main' (439921) has overflowed its stack
fatal runtime error: stack overflow, aborting
timeout: 被监视的命令已核心转储
exit=134
```

**机制**：`crates/rum-jupyter/src/json_support.rs:46-160` 递归下降 `parse_value → parse_object/parse_array → parse_value`，无任何深度计数（grep 全文件无 depth/limit）；对照内核自身解析器的四层深度守卫（resource-limits-and-sandbox.md §Parse Depth）。

**打破的书面承诺**：无文档对 jupyter 传输层做安全承诺（SECURITY.md 的模型只覆盖 kernel/embedding 库），所以这一条记为“无文档承诺”——本身值得知道：rum 为自己 parser 定下的纪律没有传导到 adapter 的 parser。

### F4（中）`eval_with_timeout` 的截止时间在 `Input` 挂起-恢复链路上被静默丢弃：恢复的 continuation 无任何时间界

**攻击者得到什么**：宿主明确用 `eval_with_timeout(source, 500ms)` 设了每请求截止；脚本以 `Input[]`（需 `Interactive` 能力——jupyter stdin 流正是此形状）挂起，宿主 `resume_with_input` 后，**续体完全不受该截止约束**，可运行任意久（受限于其他限制，或无上限配置时无界）。实测恢复段运行 712 ms > 500 ms 且正常完成、无 `TimeConstraint`。宿主无法补设：`HostSession` 没有暴露 `set_eval_timeout`，`resume_with_input` 没有 timeout 变体；唯一等效手段是另起线程的 `cancel_handle()` 看门狗。

**复现**（`/tmp/b413_probe timeout-resume`，策略 `{Pure, Interactive}`，载荷 `Input[]; s = StringRepeat["a",100000]; Do[s = s <> s, {13}]; StringLength[s]`）逐字：
```
initial elapsed=66.674µs
  input-required: ?
resume elapsed=712.233485ms
  ok term=- ... val=819200000        <- 无 TimeConstraint，越过 500ms 截止正常返回
```

**机制**：`crates/rum-host/src/session.rs:518-524` — guard 体内的请求以 `InputRequired` 返回后，`set_eval_timeout(None)` 无条件执行；`resume_eval_request_with_input`（`session_requests.rs:265`）不调用 `begin_top_level_interrupt_budget`（`session_helpers.rs:30-37`），`eval_timeout` 已为 `None`，deadline 不再武装。rustdoc 自己写的是「The timeout is cleared after the request completes or fails」（session.rs:506）——挂起既非 complete 也非 fail，却已被清除。`docs/guide/embedding.md:447`「Use `eval_with_timeout(source, duration)` for a per-evaluation elapsed-time deadline」未提及此缺口。

### F5（低）被终止的请求把 `$Context` 留在脚本打开的 context 里，后续请求继承

397-411 线修复的那一族“控制态泄漏”的存活同族（语义上是会话用户态，但无文档界定）。实测（沙箱）：
```
Begin["evil`"]; Do[i++, 100000000]; End[]   => TerminatedEvaluation["IterationWorkLimit"]
$Context                                     => "evil`"
x                                            => evil`x
```
后续宿主请求的定义落入 `evil`。对照：`Block` 的动态绑定在终止后**正确恢复**（`ValueQ[y]` = False），`TimeConstrained` 的 deadline 也无跨请求泄漏（带 `Clock` 复测通过）。authority 上无提升，记为隔离卫生问题；**没有任何文档说明 context 是 per-request 还是 per-session 状态**（reset 文档只说 reset 清除它）。

### F6（低，威胁模型）`restore_from_bytes` 按设计就是“执行任意源码”，文档未按安全后果陈述

实测：往 `HostSession::new()`（trusted/scripting 全权）restore 字节 `x = Get["/etc/hostname"]; y = AbsoluteTime[]; z = RandomBytes[8];` → `x = "zzz"`（真读了我机器的 hostname）、`y` = 新时间戳、`z` = 熵字节。往沙箱会话 restore 同样字节 → `x = $Failed`（dispatch 拒绝，**这条是阴性结果**）。快照无任何完整性/来源封装，就是明文源码。rustdoc 如实写了机制（"evaluates the snapshot source"），但未写“restore 不可信字节 = 在该会话策略下执行任意代码”；若下游出现快照迁移/缓存/网络收发，这就是设计内的 RCE。修复方向属导演，我只记录。

### F7（低，加固）jupyter 连接文件签名配置失败方向为开放 + 非常数时间比较

- `crates/rum-jupyter/src/connection.rs:123-131`：`signature_scheme` 为不认识的值（如 `hmac-sha384` 或拼写错）时 → `KernelSignaturePolicy::Disabled`，**静默**降级为不验签；key 为空同样 Disabled。fail-open 方向对安全相关配置是错的（应拒绝启动）。
- `crates/rum-jupyter/src/transport/codec/validation.rs:45,50`：HMAC 比较用 `found == &expected`（非常数时间）。在此传输上可利用性几乎为零（本地 IPC 噪声 + 前缀泄漏不助于伪造 HMAC），仅记录为与参考实现（`hmac.compare_digest`）的差距。

## 3. 检查过并判定为完好的部分（含证据）

1. **能力 dispatch 单检查点**：`eval_dispatch/mod.rs:150` 对每次 builtin 调用检查；loader 缝（`builtins_context_loader.rs:64-79,183,249,299`）与 stub 自动加载（`eval_core/symbol.rs:62-68`）双重设防。运行期构造路径实测全部拒绝（逐字见 caps 探针）：`Symbol["Get"]["/etc/passwd"]`、`ToExpression["Get[...]"]` 均 `$Failed`；`Get/Needs/FindFile/FileNames/Save/DumpSave/Pause/AbsoluteTime/RandomBytes/MemoryInUse/Input` 在沙箱全拒。
2. **全目录 643 个 builtin 的能力列**一致（脚本抽取：Pure 618 / Clock 8 / Interactive 3 / FileRead 4 / FileWrite 2 / Random 3 / Diagnostic 4 / Process 1=Exit）；`Run/Install/ReadList/WriteString/OpenWrite/Directory/CopyFile/DeleteFile/RenameFile/Put/Export/Import/URLRead/RunProcess/…` 等 ambient-authority 内建**不存在**。`DateList[]`/`DateString[]` 无参形式在 Pure 下保持未求值（惰性，无时钟读取）；`Exit[3]` 沙箱下 `$Failed`、进程存活；`SeedRandom[7]` 两次同结果（确定性，符合文档）。
3. **单次巨型分配的预检**：`StringRepeat["a",10^12]` → `GeneratedElementLimit`（0.00s）；32 MB 上限下 400 MB 请求 → `MemoryLimit` 且 `hwm` 仅 7.1 MB（未发生分配，`builtins.rs:664-668` 双预检有效）。`Table[1,{1000},{1000}]`、`ConstantArray[1,{1000,1000}]`、`Tuples[{1,2,3,4},10]`、`Subsets[Range[20]]` 全部 `GeneratedElementLimit`（笛卡尔积计入）。
4. **Print 缓冲计入模型**：32 MB 上限下 `Do[Print[StringRepeat["a",20000]],{100000}]` → `MemoryLimit`，缓冲 1671 条/33.4 MB 时被截停。
5. **panic 边界与健壮性**：152 条恶意边界表达式（`Mod[0,0]`、`0^0`、`Prime[0]`、`FromDigits["",2]`、`Partition[{1,2,3},0]`、`Block[{$RecursionLimit=10},...]` 等）在 release 下**零 panic**、会话保持可用；4 条合法带内失败（uncaught throw、incomplete input）。
6. **挂起弃置机制（397-411 修复面）**：deadline/memory-limit/abort-protect 的 `FrameAbandon` 恢复在源码（`constraints.rs:356-364,423-431`）与 12+ 专项测试（`tests/suspension_abandonment.rs`，含 `abandonment_restores_constraint_nesting_without_fallbacks`、memory-limit 恢复断言）双层落实；重叠请求拒绝（brief-411）在 `session_requests.rs:157,185,369`。
7. **深树 drop**：`Nest[g, x, 99999]` 构建+丢弃 10 万层链无栈溢出（0.01s）；`NestList` 超 256 被 pass limit 截停。
8. **`HostSession::call`、script/file 请求路径**：逐 form 武装预算（`session_eval.rs:215`）；`eval_file` 加载快照在错误路径恢复（`session_eval.rs:375-383`）。
9. **kernel crates 零 `unsafe`**：`grep -rEn 'unsafe (fn|\{|\(|impl|trait)'` 于 rum-core/rum-runtime/rum-stdlib/rum-host = 0 条，硬线成立。

## 4. 未能确立的部分

- **rum-syntax 解析器与 jupyter JSON 解析器未做专门模糊测试**；我只验证了文档列出的守卫与定向探针。JSON 深度崩溃（F3）是定向发现，不是 fuzzing 产物；解析器里可能还有别的形状。
- **一次未复现的异常**：本次会话第一次运行 `NestList[g, x, 99999]; Null`（release rum-cli）在 60s 超时被杀；其后三次完全相同的运行均在 0.01s 内以 `passlim` 结束。无法复现、无法归因（当时机身无其他负载）。记录在案。
- **签名比较的时序行为**：只读了源码（`==`），未做时序测量；结论（可利用性可忽略）是推理而非观测。
- **debug 构建的 panic 面**：探针全在 release；若有人以 debug 或 overflow-checks 构建嵌入，算术溢出 panic 类未覆盖（但会被 panic 边界捕获为 Poisoned，非进程死亡）。
- **是否存在“恢复不可信快照”的真实下游宿主**：in-tree 无此消费者（jupyter/repl/cli 均不 restore 外部字节）；F6 的风险是前瞻性的。
- **interner 单符号精确成本**：~375 B/符号为 240k 符号/90 MB 的聚合推算，未逐项分解。

## 5. 对既定威胁模型的批评

1. **模型把“宿主”当成单一可信方，但 shipped adapter 自己就是宿主，且选择了全权。** `rum-jupyter` 的 kernel 用 `HostSession::new()`（= scripting 全权 + 文件 + 熵，`launcher.rs:75`），每个 cell 用裸 `session.eval`（`adapter/mod.rs:636`），无 per-cell 超时、无内存上限。这符合 Jupyter 本地 kernel 的行业常态（python kernel 同样全权），但与"Sandbox-first"的项目第一印象相悖——sandbox-first 止步于库边界，而没有任何文档陈述 shipped kernel 自身的立场。建议在 SECURITY.md 或 jupyter 文档里写明“kernel 信任连接文件持有者，cell 即全权代码”。
2. **两段式内存防线之间有条缝。** 文档的立场是：内核给“模建上限”（带内终止），硬 RSS 界交给宿主外部看门狗。F1/F2 表明：模建上限对 (a) 进程级 interner（根本不在模型里）、(b) 采样间隔内的乘法放大（模型迟到）都不设防，而外部看门狗在文档里只是"one pattern"建议。合起来的现实是：**不设外部看门狗的宿主对 Pure 沙箱脚本没有内存防线**，这一点文档的表述（"With them, such a request is terminated in-band"）偏乐观。
3. **哪些状态是 per-request、哪些是 per-session，没有文档清单。** 397-411 修的、F5 里存活的、F4 里的 deadline 生命周期，全是同一类问题的不同实例。`HostSession::reset()` 的 rustdoc 列了它清除的东西，但没有任何文档从“请求边界应恢复什么”的角度正面陈述契约——下一次同族缺陷大概率还是从这里来。
4. **快照是程序不是数据**（F6）：机制诚实、后果未声明。威胁模型应决定 restore 的信任前提并在 rustdoc/SECURITY.md 写明。
5. 值得肯定的部分：`Protected` 非边界的裁定在代码里贯彻得很干净；能力分类与 dispatch 检查点是真正单一的 choke point；无 `unsafe` 硬线成立；kernel 自身解析器的四层深度守卫比大多数同类项目纪律好——问题只在于这套纪律没有传导到 jupyter 的 JSON parser（F3）。

---

**给导演的话（不属于报告正文）**：未按 brief 要求起草任何修复 brief；F1/F2 是同一根因族（“模型看不见的字节”）的两个实例，F3/F7 同属 jupyter 传输层，F4/F5 同属请求边界契约，分组处置可能比逐条便宜。所有探针脚本已按 Hard rules 删除；若需要复跑，报告中的 mode 名与表达式足以重建（`/tmp` crate 依赖仅 `rum-host`、`rum-jupyter` path 依赖）。
