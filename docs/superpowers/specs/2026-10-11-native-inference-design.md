# 非 Agent Python 业务迁移 Rust：架构方案

日期：2026-10-11。代码基线：`7f516b80`。状态：已获批准并实施，最终整仓与目标构建验收进行中。

输入：`Knovum_Python_to_Rust_Analysis_2026-10-10.docx`、当前源码、权威文档及根目录已有的 `local-image-models-plan.md`。报告基线落后于当前图片能力实现，实际调用链与锁文件优先。本文记录整改决策与验收范围；实施后的稳定契约回写现有权威文档，本文不替代它们。

## 1. 目标和完成边界

由单 agent 顺序完成设计、实现、审查与验证，不启动子 agent。保留 Deep Agents/LangGraph Python 执行环境及其现有预算、工具权限、checkpoint、恢复协议；其余自维护 Python 业务、推理运行时与非 Agent 开发脚本退出生产和常规开发链路。

Rust 化包含 Rust 调用成熟 C/C++ 引擎。优先复用库，不自行实现模型网络、FFT、音频 codec、tokenizer 或完整模型 Hub SDK。不添加旧引擎回退、旧数据读取、格式转换兼容器、失败换模型/设备及补丁式依赖改写。

默认保留仓库现有 macOS arm64、macOS x64、Windows x64 支持范围和全部已接入能力。CPU 是三目标必测基线；现有承诺的加速路径也要验收，不能把 CPU smoke 当作 GPU 路径完成。取消某个平台、模型或精度选项需独立明确产品决定，不能作为迁移的隐含结果。

不兼容旧数据不等于自动删除用户内容。为变更的模型执行身份、向量索引或持久请求设置明确的新基线；遇到不兼容状态显式拒绝或走既有用户触发的重置流程。无关数据库、资产和保留的 Agent checkpoint 不为本次迁移新增破坏性操作。

第三方 yt-dlp 独立程序继续作为媒体导入供应商资源管理；它可能内嵌解释器。本次目标是 Knovum 自维护业务与开发工具退出非 Agent Python，不将其表述为所有第三方二进制内部绝无 Python。

## 2. 实际范围

当前模型目录有 22 个模型条目，迁移覆盖下表全部条目及调用者，不仅覆盖报告中的抠图。

| 能力             | 当前模型与执行                                                        | 目标路线及必须保留的行为                                                                                             |
| ---------------- | --------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| 资源下载         | HF/ModelScope SDK，`resources` worker；另有现成 Rust 直链下载         | Rust 库与现有安装器，固定 revision/文件清单、续传、校验、取消、原子发布；保留显式来源                                |
| 向量             | Qwen3-Embedding-0.6B，Sentence Transformers/Torch                     | 现成 Rust Qwen3 实现，优先 fastembed/Candle；1024 维 F32、last-token pooling、L2、角色提示词、2048 token 超限拒绝    |
| 转写             | Whisper Tiny/Small/Large-v3，MLX/faster-whisper                       | whisper.cpp 原生链路；同 family/size，新冻结模型格式，VAD、语言策略、原时间线、长音频与取消                          |
| 强制对齐         | Qwen3 Q8 audio.cpp / HF 高精度 Transformers                           | Q8 直接复用 audio.cpp ABI；高精度优先验证非量化原生资源，不能把 Q8 替代或 F16 自动宣称为原 HF 精度等价               |
| 说话人分离       | Sherpa Pyannote INT8/FP32 + ERes2Net                                  | 官方 Sherpa Rust API；自动人数与精确 1–32 人分别验收                                                                 |
| 音源分离         | Spleeter FP16/FP32，Sherpa CPU                                        | Sherpa 原生 API；44100 Hz 双声道、1024-hop 补齐、最少 4096-window、裁回精确原长度、10 分钟限制与试听接受             |
| 小型语音生成     | MOSS Nano ONNX + Python host 编排                                     | 先评估现成 Rust MOSS 库；原 ONNX、十音色、参考克隆、48000 Hz、八帧流式、1200 帧上限、冻结 seed、双声道取均值         |
| 高质量语音与设计 | VoxCPM2 原精度/Q8 GGUF + `knovum_voice` ABI                           | Rust 直接接已有桥；参考 conditioning、设计、delivery instruction、流式回调、尾块、取消及句柄生命周期                 |
| 去背景           | U²-Net Compact/Full                                                   | `ort` 复用原 ONNX；320×320 张量及既有蒙版、alpha 合成规则                                                            |
| 放大             | Real-ESRGAN General x4v3/x4plus                                       | `ort` 复用图及 external weights；128px tile、32px context、64px core、原生 4×，边缘补齐、舍入、alpha、目标尺寸与接缝 |
| 修补             | MI-GAN pipeline v2                                                    | `ort` 复用 uint8 图；选中蒙版极性翻转，未选区域 RGB 字节严格不变                                                     |
| 选区             | EfficientSAM Ti/S encoder + decoder                                   | `ort`；正负点/框、单会话编码缓存、源修订校验、候选分数选取与过期结果拒绝                                             |
| 深度             | Depth Anything V2 Small FP32/UINT8                                    | `ort`；冻结 processor、14 倍数尺寸、有限非恒定相对深度、16 位 PNG；不恢复已排除的 signed INT8 图                     |
| 开发与发布       | Python media producer、inventory、协议生成、acceptance、多 profile CI | 现有 Node 脚本负责生产编排及离线评分，Rust 负责实际计算；仅 Agent 所需 Python 工具保留且归其目录                     |

源码所有者：`local_resources/{models,download,preparation,runtimes}`、`ai/{capabilities,contracts,local_runtime}`、`spaces/{knowledge,speech,derivation}`；图片处理与资产发布继续由原所有者负责。Renderer 只消费生成契约，不新增推理服务或模型映射表。

## 3. 架构选择

推荐独立 Rust 推理可执行文件。复用现有一计算 worker、优先级公平准入、内存 reservation、健康观察及 60 秒 idle 回收；进程内仅初始化本次选择的能力。它以受控 stdio 协议接收 Rust 宿主已经授权和验证的输入，不开放网络监听端口。

比较另外两条路线：在 Tauri 主进程直接调用引擎可少一个进程，却无法隔离原生崩溃与无法及时中断的同步计算；Rust 调 LibTorch 可减少 Python 代码，却保留庞大的 Torch/GPU 供应闭包。二者不满足本次可靠执行与依赖收缩目标。

```text
Vue 功能入口
  → 既有 Tauri 命令 / Space Action
  → Rust 所属业务：授权、Run、冻结输入和目标、资源准备
  → ai/local_runtime：计算准入、worker 生命周期、取消、输出验证
  → 私有 Rust worker：能力专属适配 → ORT / Candle / Sherpa / audio.cpp / whisper.cpp
  → 请求目录内产物与流式事件
  → Rust 所属业务：重新校验 source revision / attempt epoch → 原子提交与发布

Deep Agent → 既有独立 Python worker → 原有 Rust 模型/工具 gateway
```

### 3.1 源码与依赖边界

在 `src-tauri` Cargo workspace 中增加两个有实际消费者的包：`inference-contracts` 和 `inference-worker`。前者只拥有宿主与 worker 共享的严格 DTO、模型执行枚举和协议常量；后者拥有能力适配与可执行入口。现有 Tauri 包仅依赖 contracts，不把推理引擎链接进 desktop 宿主。

现有 `ai/contracts` 中真正跨进程的类型移入 contracts；宿主专用授权/领域类型仍留在原处。静态模型清单继续属于 `local_resources`，worker 接收冻结的模型身份和经验证路径，不能反向依赖宿主模型目录。TypeScript 仍由 Rust 的 `ts-rs` 导出。

worker 中按 `embedding`、`images`、`transcription`、`alignment`、`diarization`、`separation`、`synthesis` 拆真实职责；transport、artifact、device 只承担自身边界。以封闭枚举分派已支持能力，不引入插件注册中心、全局 DatabaseService、通用 InferenceService 或多层仅改名适配。

新 workspace 路径必须纳入 source-layout、rustfmt、Clippy、测试、jscpd 和 Cargo 安全检查，使用同一个正常 target 目录并串行构建。不能以新增 crate 逃过现有门禁。

### 3.2 协议、隔离和取消

复用当前请求/事件内容中仍有效的产品语义，发布严格的新 native 协议身份；删除 Python manifest/包布局字段，不保留旧初始化格式。请求冻结 capability、模型 spec/完整 snapshot digest、设备与精度、线程数、owner、request/attempt/epoch、来源修订、deadline；流式事件必须绑定同一身份。

一个会话只允许一个活动计算，stdio reader 与计算线程分开，使同步原生调用期间仍能接收取消。使用有界队列与现有 chunk ACK 背压；stdout 仅发协议，诊断输出受限并脱敏。协议帧、张量、文件大小、shape、dtype、摘要与有限数值在输入和产物边界校验。

ORT 优先调用该版本提供的 RunOptions 终止 API；whisper.cpp/已有 voice bridge 使用原生取消回调；不支持中止的引擎在安全边界检查取消。沿用宿主五秒取消等待与进程树终止逻辑，确认子进程退出后才释放计算/内存/资源租约。取消、超时、崩溃及旧 epoch 都不能发布结果。Windows Job Object、Unix 进程组、应用退出清理沿用现有规则。

worker 只有受控模型读取和请求目录写入权限，不接入 SQLite、LanceDB、secrets、Run ledger 或业务资产目录。宿主验证产物后交原能力提交；缓存不跨模型、源修订或会话混用。尤其 SAM 缓存只绑定当前完整输入身份。

### 3.3 资源和设备

`local_resources` 继续拥有确定快照、字节/哈希、来源、共享安装、staging、租约、取消与原子激活；模型下载不需要先装 Python。优先使用库已有的离线加载入口，禁用模型自行下载、ambient token/cache 和 remote code。

模型 spec 描述权重、输入处理、输出规则；execution profile 描述引擎、精度和设备。生成新 Rust embedding 身份并建新索引，去掉当前 spec 文本中的 Python SDK 版本与执行实现不一致风险；不能保留旧 spec 名称冒充同一种执行。

三目标先以同权重 CPU 测试建立迁移证据，再验收原承诺加速路径。设备探测必须来自目标原生引擎，替换现有 torch/ctypes probe。启动前不足可依产品现有准入规则明确选择 CPU；启动后的注册失败、OOM 或计算失败直接报错。显式加速配置不能被 vendor 默认的静默回退改变。

ORT wrapper/API 与原生 ORT 的精确配对是供应身份；不能把最新 crate 直接指向当前 Python ORT 1.22.1。图片/MOSS 尽量共用通过验收的 ORT；Sherpa 的完整链接闭包也要核对。若无法共用，分开原生加载闭包并在独立会话进程使用，不在一个进程加载未经核验的冲突版本。

## 4. 库选型及资格规则

下列版本为 2026-10-11 registry/源码核验的候选。实施后的实际选择、拒绝原因和资格结果记录在权威依赖文档与 [验证记录](../../validation/2026-10-11-native-inference.md)，精确 pin 由 Cargo lock 和各供应配置拥有。

| 责任            | 候选与证据                                                                                                                                                                         | 选择规则 / 不自造的边界                                                                                                                                                                                                                      |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ONNX            | [ort](https://github.com/pykeio/ort)，registry `2.0.0-rc.13`，MIT/Apache；[终止 API](https://docs.rs/ort/latest/ort/session/struct.RunOptions.html)                                | 先选择与受控原生库配对的版本，验证九个图片条目与 MOSS 图；不重写 ONNX 执行器                                                                                                                                                                 |
| Qwen 向量       | [fastembed](https://github.com/Anush008/fastembed-rs) `7.1.1` 的 Qwen3/Candle；[Candle](https://github.com/huggingface/candle) `0.11.0`                                            | 优先使用可传本地 model/tokenizer 的 API；明确禁用截断，按当前角色提示词计算。仅在真实公开 API 缺口成立时直接使用 Candle 官方模型，不能复制 vendor 网络实现                                                                                   |
| Sherpa          | [官方 Rust API](https://github.com/k2-fsa/sherpa-onnx/tree/master/sherpa-onnx/rust) `1.13.8`，Apache                                                                               | 优先官方安全 API；其没有 source-separation 导出时才通过官方 sys/C API 加有资源所有权的窄 RAII 接入，FFT/STFT 仍属上游                                                                                                                        |
| VoxCPM2/Q8 对齐 | [audio.cpp](https://github.com/0xShug0/audio.cpp)，沿用本库冻结 revision 和 voice bridge                                                                                           | 直接接现有 ABI，异常与回调不能跨 FFI 展开；句柄严格释放。新增 ABI 包含实际安全/生命周期职责                                                                                                                                                  |
| ASR             | [whisper.cpp](https://github.com/ggml-org/whisper.cpp)；[whisper-rs](https://github.com/tazz4843/whisper-rs) `0.16.0`；[其他 binding](https://github.com/findit-studio/whispercpp) | whisper-rs registry 许可为 Unlicense，目前不在 deny allowlist，不能直接扩白名单。其他 binding 必须核对维护、上游改动和供应可控性；若都不满足，消费原版 C API 的有界接入，不自行实现解码算法                                                  |
| MOSS            | [moss-tts-nano](https://github.com/mzdk100/moss-tts-nano) `0.1.1`，Apache；[官方 ONNX 实现](https://github.com/OpenMOSS/MOSS-TTS-Nano)                                             | 先验证本地十音色、tokenizer、prompt、固定八帧、seed、结束判定、reference codec 与尾块。README 的单音色/自适应 chunk 声明不构成产品等价证据。实际缺口不能通过改 vendor 补丁掩盖；库失败才允许最小模型 host 编排，tokenizer/RNG/张量仍用成熟库 |
| HF 下载         | [官方 hf-hub](https://github.com/huggingface/hf-hub) 与既有 reqwest                                                                                                                | 重新检查当前公开 API 的固定 revision、文件白名单、取消、可观察字节进度、续传及应用管理路径。选满足要求且重复代码最少者，不能沿用报告对旧版 API 的否定结论                                                                                    |
| ModelScope 下载 | [官方 SDK 协议来源](https://github.com/modelscope/modelscope_hub)、[Rust modelscope](https://github.com/xgpxg/modelscope) `0.1.5`、既有 reqwest                                    | 保留 ModelScope 固定来源；检查 Rust 候选是否满足精确 revision/文件/续传/取消。候选不满足时只实现固定资源端点适配，不能重写仓库管理/上传/搜索客户端                                                                                           |

续传使用确定的 source revision/文件身份，验证 Range/Content-Range、实际长度和最终 SHA-256。服务端不支持 range 时显式从零重新下载该文件并报告进度，不拼接错误数据。缓存命中仍经完整校验；失败不会换来源。扩展既有下载所有者，避免两个独立下载/发布系统。

高精度对齐是独立资格项：验证同源参数、tokenization/processor、语言、词字边界和时间戳误差。原生非量化 GGUF 或受控 ONNX 若达不到原功能，整改保持未完成并报告具体缺口；不得只隐藏 HF 选项或保留 Python fallback。研究转换可使用隔离的上游工具，终端与普通开发不执行转换脚本。

每个最终依赖按 `oss-first.md` 的 evidence 字段记录维护、许可、安全、平台、传递成本、调用者和拒绝备选的具体原因。模型及 compiled libraries 分别审查；库许可证不代表权重可分发。

## 5. 开发、构建与旧实现退出

保留现有 Node producer 编排并将非 Agent `.py` 生产脚本迁出；不为本次迁移新建一套通用 xtask 框架。普通 Vue/Rust 开发消费固定且校验的 FFmpeg/原生资产，asset producer 独立承担 CMake/C++、target 签名、SBOM、notices 和完整依赖 inventory。

Deep Agent 的 CPython、锁文件、供应与 Python 契约生成器归 Agent 边界。Agent 首次使用仍可能准备自身 Python；非 Agent `desktop:dev`、推理、模型下载和媒体准备不得触发 uv/Conda。Agent 的生成器可继续用现成 datamodel-code-generator，只处理 Agent schema，不为了消灭扩展名自造 Python 代码生成器。

`scripts/contracts.mjs` 分开通用 Rust→TS 导出与 Agent Python 生成入口。原生 inference 直接共用 Rust DTO，移除 inference/resources Python schema、生成目录及生成映射。原生 worker 的实际计算完全属于 Rust；验收保留严格 corpus、provenance、指标与报告语义，不保留 Python 验收 CLI。

实施选型：离线验收评分随现有开发工具归 `scripts/inference`，采用锁定的 Node `npyjs` 和 `fastest-levenshtein` 库读取数值文件与计算编辑距离。此处只读取 hash-bound 的真实输出，不承担业务推理或运行时加载。这样无需为开发评分增加第二个 Rust 执行系统，也无需自行实现 NPY 或编辑距离。验收捕获与单元合成 fixture 严格区分；缺少真实捕获或人工盲评只能报告 unavailable，不能推导通过。

逐能力切换时同时删除原 Python caller、适配、过时模型格式和本能力旧测试；未切换能力短期仍走自身当前实现，不双跑、不失败回退、不增加过渡兼容协议。迁移后的功能每阶段都可独立审阅和验证。

所有 caller 迁完后整体删除 `resources`、`light-inference`、`torch-inference`、`voice-generation` profiles/uv locks/license reviews、tensor Conda 供应、knovum_inference/model_resources 包、producer 配置、检查例外、release descriptor、CI matrix 和旧状态字段。共享供应中 Agent 的真实依赖保留。

现有 `python-runtime.yml` 不再以 light-inference job 顺带生产 FFmpeg。分离 Agent、media 与 native inference 的实际资源任务，publication 由真实产物类型/target 和 manifest 验证；旧的五 profile 矩阵不能继续作为发布完整性规则。新 worker 为私有 executable，不授予 Renderer shell/process 权限。

## 6. 验收与实施顺序

首先冻结现有模型、输入、参数和输出基线，取得真实能力资格证据，再扩展生产代码。顺序由核心风险和依赖决定，全部由同一 agent 执行：

1. 最小 native 执行验证：Qwen CPU F32、现有 voice/Q8 ABI、图片 ONNX；同时验证 MOSS 和高精度对齐候选，尽早暴露会妨碍全量退出 Python 的缺口。
2. 完成资源下载和 native worker 共享边界，接准入、健康、取消、租约、Task Center；现有下载/进程设施经定向重构复用。
3. 全部图片能力切换，随后向量/知识索引切换，再完成 VoxCPM2、Q8、高精度对齐、Sherpa、MOSS 和三尺寸 ASR。每项覆盖业务入口、catalog、设备、协议、publication 和验收。
4. 工具链、Agent 隔离与 runtime/CI 清理；把稳定契约写回 `architecture.md`、`system-architecture.md`、`local-models.md`、`oss-first.md`、`dependency-policy.md`、`packaging.md`、`release.md` 和受影响平台/desktop 文档。
5. 全库门禁与三目标真实桌面包/模型验证；未取得的目标或人工听审证据明确记未验证，不能将编译成功称为整改全验收完成。

具体任务、文件修改和命令由后续单 agent 实施计划展开。本草案不声称已通过以下检查。

| 验收面     | 必须证明的结果                                                                                                                                                        |
| ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Embedding  | 1024 有限单位向量；prompt/token 规则一致；2048 超限拒绝；同权重 CPU cosine ≥0.9999 作为数值诊断；全新 query/document 索引 Recall@10、NDCG@10 绝对下降各 ≤0.01         |
| ASR        | Tiny/Small/Large-v3 独立验收；中/英/混合、静音、噪声、长录音、语言、VAD 原时间线；中文 CER/英文 WER 绝对回退 ≤0.01                                                    |
| Alignment  | Q8 与高精度分别评估时间戳/词字边界及语言；当前 1–8 个片段、每段最多 300 秒；禁止错误扩大成整请求总长 300 秒                                                           |
| TTS        | 十个 MOSS 音色、reference clone、VoxCPM2 原精度/Q8、设计与 conditioning；每样本至少三位独立盲听评审；48k 单声道、流式连续、首块、尾块、seed、取消与最大帧限制         |
| Sherpa     | 自动与精确人数分别量化；二 stem 精确原帧数、hop/window 与长输入边界，质量听审及接受流程                                                                               |
| Images     | 两去背景 + 七新增变体全覆盖；多类别真实图片、透明度/EXIF/细长图、最大尺寸、tile 接缝；MI-GAN 非选区严格不变；SAM 缓存失效；深度归一化/16 位发布；逐模型取消与峰值内存 |
| 可靠执行   | 活动计算期间取消、硬终止、crash、deadline、旧 epoch、破损帧/文件/shape/hash；无已取消发布、无容量提前释放、无遗留进程；TTS ACK 背压与重复序号                         |
| Supply     | 三目标完整 inventory/hash、ABI/动态库加载、中文/特殊路径、离线基础启动、下载续传、租约和损坏资源；原生签名与真实 Tauri build；GPU 各路径实机执行                      |
| Agent 隔离 | 无 inference SDK/Torch/MLX 混入 Agent 环境；现有 SDK contract、输入去重、工具副作用/费用回执、pause/resume、crash recovery 仍通过                                     |
| 代码门禁   | 受影响 focused checks 后，`pnpm check:scripts`、`pnpm check:web`、覆盖 workspace 的 `pnpm check:rust`、收缩后的 Agent Python 检查、契约重生成一致性与 Cargo 依赖检查  |

最终审计要同时检查 import、进程启动、resource selector、package scripts、producer source closure、lock/manifest、CI 和文档中的可执行旧路径。源码文件数量减少不能单独证明 Python 已退出。

## 7. 本轮设计证据与限制

已读取报告，核对当前 22 条模型目录、runtime selector、图片 adapter、向量与 MOSS 实际处理、audio.cpp 对齐 ABI、资源下载 caller、检查脚本及发布 matrix。已完成依赖选择、业务调用链切换与实际模型对照；本机测量和最终检查记录在 [验证记录](../../validation/2026-10-11-native-inference.md)。上游 README 性能数字不作为 Knovum 验收结果。

报告补充点包括九个图片模型条目、现成 Rust MOSS 候选、最新 HF 客户端需要重新评估、whisper binding 当前许可门禁、跨 crate 检查覆盖、旧 embedding spec 的 SDK 身份与实际执行同步，以及仅 Agent 保留 Python 的工具链定义。

本机证据不能替代 macOS arm64/Windows x64 与 GPU 实机证据，人工盲听也不能由自动 smoke 代替。任何资格项失败都保留明确的未完成结论，先解决具体缺口，不创建兼容或兜底层。
