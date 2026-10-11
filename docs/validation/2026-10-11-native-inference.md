# Python 业务迁移 Rust：交付验证

验证日期：2026-10-11。实现分支：`codex/native-inference`。执行方式：单 agent，自审和实际测试，没有并行 agent。

本记录保存本次改动和测量证据，不另行定义架构契约。稳定契约分别由 [系统架构](../system-architecture.md)、[模型目录](../local-models.md)、[依赖政策](../dependency-policy.md)、[打包](../packaging.md) 和 [发布](../release.md) 拥有。

## 实现范围

保留 Deep Agent/LangGraph 的独立 Python worker、锁文件、受控 CPython 供应和 Agent 契约生成。其他自维护 Python 推理、模型下载和验收 CLI 已退出执行链。非 Agent 桌面开发、下载、图片、向量、语音及媒体准备使用 Rust 和现有 Node 工具；第三方独立 yt-dlp 的内部解释器不属于自维护业务。

全部 22 个模型条目保留：9 个图片、1 个向量、3 个转写、2 个对齐、2 个说话人分离、2 个音源分离和3个语音生成。移除 resources、light-inference、torch-inference、voice-generation 的 Python 包、schema、锁文件、tensor Conda 供应、生产配置与 CI 矩阵，不保留旧运行时协议或失败回退。

`local_resources` 继续拥有固定来源、续传、哈希、安装、共享任务和租约。独立 Rust worker 通过共享 Rust DTO 接入原 supervisor，隔离原生崩溃、持有内存预留至进程退出并执行取消。图片资产、知识索引和语音发布保持原业务所有权。新 embedding/native execution 身份拒绝旧数据，使用既有显式重建流程。MOSS voice profile 的 SQLite 基线约束使用 native-v2 身份；旧预发布数据库基线校验不符时显式拒绝打开，遵守仓库的重置政策，没有兼容读取、自动迁移或改写本机用户数据库。

## 依赖与供应决定

- ONNX：精确锁定 ort 2.0.0-rc.13、API21 和 Microsoft ONNX Runtime1.22.1。真实测试证明该供应下 API22 的自动设备机制不可用；使用公开的稳定 API21 和显式 CPU EP，未修改供应商源码。
- 向量：官方 Candle0.11 Qwen3 模型与 tokenizers；fastembed 的公开接口及供应成本不能满足冻结的本地模型和执行约束。KV 状态按文本独立，角色提示、2048 token 上限及归一化保持明确。
- 语音：冻结 audio.cpp/VoxCPM2 ABI、Sherpa 原版 C API 和 whisper.cpp 原版 C API。Whisper 许可候选不满足 allowlist，采用窄 RAII 接入；没有扩充许可白名单或复制解码器。
- MOSS：现成0.1.1库固定375帧且要求 ORT API27，不能满足1200帧和现有供应要求。保留最小模型编排，SentencePiece、重采样、随机和张量计算分别使用成熟库；十音色、参考克隆、固定 seed 与八帧输出得到实测。
- 高精度对齐：原 HF 快照缺少上游公开原生加载器要求的 generation_config.json。高精度选项使用同上游正式 F16 GGUF，1,840,097,696字节，SHA256 `d0a96fb96de17686766688b622394877ee5e89b193be7f53de8715d99151a000`。保留 Q8 独立选项；F16 功能测试不宣称与旧 HF 浮点实现精度等价。
- 构建：固定14项配方输入，最终配方 SHA256 `b3a92443843a343b563b881c3194f45a4658d4957ec5f2980e4d07c7b0a783db`。未修改供应源码。CPU 分派使用上游公开 GGML 模块，Whisper 的 GGML 置于私有目录，Mach-O 依赖重定位后封闭清单、签名和哈希绑定最终字节。
- 下载：扩展既有 reqwest 下载所有者，仅适配固定 HF/ModelScope 文件端点，不实现仓库客户端。Range/Content-Range、取消、长度、SHA256 和原子安装均有边界测试。
- Windows 安装包：CMake 官方 InstallRequiredSystemLibraries 选择 x64 MSVC 可分发 DLL，验证并保留 Microsoft 原始签名。Vulkan loader 使用 LunarG 1.4.363.0 官方 components ZIP，18,211,191字节，SHA256 `a25a927aa8b9f0371048f1861cf88ac3b9bc9b1fb332c42d897c8ab32695769a`，本机实际下载验证与归档提取；打包检查 PE imports 只解析到导入者所在目录、受控 bin 或系统 allowlist。纯策略回归通过不代表 Windows 实机执行通过；开发机需要工具链安装的 MSVC/Vulkan runtime，干净开发机和安装包均须实机资格。
- 离线评分：既有 Node 工具使用 npyjs 和 fastest-levenshtein，保持严格 corpus/capture/provenance；不自造 NPY parser、编辑距离或另一个验收运行时。

Windows 最低版本更新为 Windows10 2004/build19041，与实际 ONNX Runtime 供应要求一致，早于该版本显式拒绝。DLL 搜索限于受控目录与系统目录。

## 本机模型实测

硬件：macOS15.6.1/build24G90、Intel Core i7-9750H x86_64、64GiB RAM、12逻辑 CPU，Xcode 原生工具链。下表是实际权重执行与边界资格，不能替代正式长音频、多说话人、噪声和人工听评语料。对照 Python 环境只用于隔离 QA，不在产品、正常开发或 CI 执行链中。

| 能力              | 实际证据                                                                             | 结论                                                                                                                                 |
| ----------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ |
| 九个图片模型      | U2netp/U2Net、ESRGAN general/plus、MI-GAN、SAM Ti/S、Depth UINT8/FP32 的原图输出对照 | 九项均与原 Python ORT 输出逐字节一致；SAM Ti/S 的 A→B→A 缓存实测通过，且 A/B 输出不同                                                |
| Qwen3 向量        | 12份 document/note/code/transcript 文本及4条英中查询；16对向量                       | 最大绝对误差4.814937710762024e-7，最小余弦0.9999999999951212；全部 top10 排序一致；双方 Recall@10=1、平均 nDCG@10=0.9625862263836886 |
| Whisper 三尺寸    | Tiny/Small/Large-v3 实际4秒语音，12词，原始时间线边界                                | 最终包三尺寸全部通过；耗时5.502/21.750/145.701秒                                                                                     |
| F16/Q8 对齐       | 同一实际语音及12词                                                                   | 均覆盖12词，起止0–3920ms，未超过3954ms输入；RSS约2.04GB/1.34GB                                                                       |
| Sherpa 说话人分离 | INT8/FP32 模型，实际4秒语音                                                          | 有效时间段与边界；复杂重叠说话人质量尚需正式语料                                                                                     |
| Spleeter 两精度   | FP16/FP32，44100Hz双声道                                                             | 两 stem 均精确110251帧、441048字节；RSS约357MB/550MB                                                                                 |
| MOSS 十音色       | 十项实际执行，preset6重复；7秒参考克隆                                               | 全部生成，preset6同seed产物 SHA256 `cfedf37121f37edb2ed0da01f74949c9cfd550220588924284c2b461b1a330d7` 重复一致；参考克隆成功         |
| VoxCPM2 原精度/Q8 | 原精度7块/Q8设计21块；Q8参考克隆17块                                                 | 实际生成并完成；Q8首块后取消通过；RSS约11.81GB/7.79GB                                                                                |
| 流式背压与尾块    | MOSS实际38块长文本；另行 delayed-tail、stale ACK、EOF                                | 8块未ACK时停止发布，释放后38块完整完成；延迟尾块不提前完成；错误epoch及EOF拒绝结果                                                   |

原始捕获、日志与本地模型存放于忽略目录 `.runtime-development/qualification/`，不把权重或大体积产物提交进仓库。最终签名包的22项模型及克隆、背压、尾块和 EOF 资格全部通过。manifest SHA256 `386abee309ce417620486eeaf8fb43675423f05121bbd21458ef8981c93bd247`；worker SHA256 `999f6c8153acd840f7b9976f2413aa944174cc5dd79cfb4db021a30e28afc956`。实际 U2Net 推理取消耗时191.24ms，进程随后恢复 idle。

## 本地检查与打包记录

最终 `pnpm check:rust` 退出0：Rust layout、capabilities、workspace fmt、严格 Clippy 全部通过；849项主应用、3项共享契约、28项 worker 测试通过，8项依赖实机/外部资源的现有测试保留 ignored；Rust 重复行比例0.95%，符合1%门禁。权威 Rust 契约生成及 `contracts:check` 通过，Agent 生成字节保持不变。172项 Node 测试、79项保留 Agent/Python 测试通过。最终 `pnpm check:web` 退出0：四套类型检查、Oxc、layout、dependency-cruiser、337文件/1252项测试、Knip、0.19%重复行比例和1951文件格式检查全部通过。普通非 Agent 开发准备 `node scripts/runtime/native-worker.mjs` 退出0，不调用 Python 准备。

桌面打包按用户最新指示停止：`pnpm package --bundles app` 在最终应用模块编译期间收到 SIGTERM，退出143；此前日志没有编译错误，但没有完成 `.app` 产物。用户明确不需要打包操作，本次不再执行打包或依赖该产物的桌面 E2E；最终 `.app` 内运行时、应用签名和桌面启动测试均不计通过。独立原生 worker 的正常 release 构建及上列真实模型验证已完成，不受停止桌面打包影响。

最终签名包的 Whisper Small 六项回归全部通过：60秒前置静音、79.080秒长音频、10秒固定语言全静音、中文、60秒前置静音自动识别中文、自动识别全静音。中文输出25个有效时间单元；全静音输出为空。前置静音的片段起止59970–63930ms，保留原始时间位置及 Silero 的30ms预留。Whisper 的末尾补零窗口经过边界校验后限制到原音频；词时间段保持正长度并限制在所属片段内。自动识别中文输出 zh，片段60000–65734ms；自动识别全静音输出 und 和空结果。

OS 合成语音 smoke 的离线评分：Tiny/Small/Large 英文 WER 均为1/12=0.0833333333333，唯一差异为产品专名；中文 CER=0/23。此项验证评分链和实际转写，不作为独立正式质量语料。

Agent QA：实际84,525,179字节归档，SHA256 `d919b2f3f6b19d5327ebaa57d25d3899aae903829462ab4959ca90d5771df4b3`；21962项 TAR inventory 完成安全预检、提取和封闭清单校验。FFmpeg QA：33,957,215字节，SHA256 `e4e4169414951c1f54a6a75133d09c9ec1a2b0e896ac52fba543eafe400910c2`，发布/提取/哈希校验通过。这些归档使用本地 QA URL，没有对外发布。

## 发布门禁与限制

- 本机不能提供 Windows x64/macOS arm64 实机、GPU 路径及对应安装包的通过证据。CI 已按原生目标接入构建和校验，未执行的 job 不计通过。
- ASR 英中长音频/噪声/重叠语料、复杂说话人质量、F16 对齐对旧 HF 的精度对照、TTS 每样本至少三名独立盲听及模型/供应合法性人工复核，仍属于发布门禁。功能 smoke 不替代这些证据。
- 实际 cargo-deny：bans、licenses、sources 通过；advisories 未通过，仍有 bincode RUSTSEC-2025-0141、paste RUSTSEC-2024-0436、rustybuzz RUSTSEC-2026-0206、ttf-parser RUSTSEC-2026-0192、yaml-rust RUSTSEC-2024-0320 的未维护依赖提示。paste 同时由新增 Candle gemm/tokenizers 引入，其他路径包含现有 Typst/PDF/Lance/genai。没有添加忽略、补丁或放宽政策，也不宣称安全门禁全绿。
- 本地 macOS QA 使用 ad-hoc 签名；生产签名、notarization、不可变正式发布及硬件资格不能由本地 QA 替代。
