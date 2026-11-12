<div align="center">
  <a href="https://github.com/madebyzan/knovum/releases/latest">
    <img src="docs/assets/readme/banner.png" width="100%" alt="Knovum — 本地优先的 AI 知识工作空间" />
  </a>

  <p>
    <strong>把分散的资料，变成可以探索、验证和使用的知识。</strong>
    <br />
    面向研究、学习与创作的本地优先 AI 工作空间。
  </p>

  <p>
    <a href="https://github.com/madebyzan/knovum/releases/latest"><img src="https://img.shields.io/github/v/release/madebyzan/knovum?label=下载&color=2563eb" alt="最新版本" /></a>
    <a href="https://github.com/madebyzan/knovum/actions/workflows/quality.yml"><img src="https://github.com/madebyzan/knovum/actions/workflows/quality.yml/badge.svg" alt="质量检查 CI" /></a>
    <a href="https://github.com/madebyzan/knovum/stargazers"><img src="https://img.shields.io/github/stars/madebyzan/knovum?style=flat&color=f5b301" alt="GitHub Stars" /></a>
    <img src="https://img.shields.io/badge/macOS-14%2B-111111?logo=apple&logoColor=white" alt="macOS 14 及以上，支持 Apple 芯片和 Intel" />
    <img src="https://img.shields.io/badge/Windows-10%202004%2B-0078D6?logo=windows&logoColor=white" alt="Windows 10 2004 及以上，x64" />
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-16a34a" alt="MIT 许可证" /></a>
  </p>

  <p>
    <a href="https://github.com/madebyzan/knovum/releases/latest"><strong>下载 Knovum</strong></a> ·
    <a href="#你可以用-knovum-做什么">你可以做什么</a> ·
    <a href="#本地优先的设计">隐私与本地运行</a> ·
    <a href="#常见问题">常见问题</a> ·
    <a href="#从源码构建">从源码构建</a> ·
    <a href="README.md">English</a>
  </p>
</div>

## Knovum 是什么？

Knovum 是一个**本地优先的桌面知识工作空间**，帮助你收集、整理、理解并使用自己的资料。

把笔记、收藏的网页与视频、会议录音、PDF、Office 文档、图片和代码放进同一个可视化空间。你可以在无限画布上组织资料、发现内容之间的联系、搜索整个资料库，也可以向 AI 提问，并从回答中的引用直接回到原文段落、PDF 页码或录音时间点。

你还可以把收集到的素材转化为文档和媒体，同时保留每个结果与原始资料之间的关联。

你的资料库存放在自己的设备上，无需 Knovum 账号，也没有 Knovum 云服务。搜索、转写、OCR 等受支持的 AI 能力可以在本地运行；当你选择云端模型来回答问题或生成内容时，Knovum 只会把该操作所需的上下文发送给你选择的服务商。

<img src="docs/assets/readme/screenshot.png" width="100%" alt="Knovum 桌面应用：无限画布中组织着资料、主张、证据和简报，侧边展示带来源引用的 AI 回答。" />

## 你可以用 Knovum 做什么

- **把资料集中到一个地方。** 将笔记、网页、视频、录音、PDF、Office 文件、图片和代码收集到同一个 Space。
- **看见想法之间的联系。** 在无限画布上整理资料，把主张与证据关联起来，让问题、决策和来源保持在一起。
- **向资料提问，并核对回答。** 按关键词或语义搜索，向已连接的 AI 模型提问，并沿着引用回到原始资料的位置。
- **基于已有知识继续创作。** 将选中的内容转化为文档、图片、视频、字幕或语音，并记录使用过的输入和模型。
- **自己掌控数据与 AI。** 资料保存在本地，支持的 AI 能力可在设备上运行；每项任务都可以选择模型服务商或兼容的桌面 Agent。

## 工作方式

<div align="center">
  <img src="docs/assets/readme/source-to-work.png" width="100%" alt="知识工作流：收集资料、连接想法、提出问题，并创作可追溯到原始资料的成果。" />
  <sub>从分散的资料，到相互连接的知识，再到来源可追溯的成果。</sub>
</div>

1. **收集。** 粘贴链接即可保存网页，也可以保存 YouTube、Bilibili、抖音、Vimeo 等平台的视频。添加录音、PDF、Office 文件、图片、代码和笔记。录音可以在本地转写，并保留时间戳和说话人信息。
2. **整理。** 使用分区和分组组织资料，把主张、证据、问题和决策作为知识对象加入画布，再用有意义的关系将它们连接起来。
3. **探索。** 在单个 Space 或整个资料库中按关键词或语义搜索。针对自己的资料提问，并沿着每条引用回到原始位置。
4. **创作。** 选中所需素材，生成文档、图片、视频或语音内容。导出成果的同时，保留所用资料及其版本记录。

## 功能一览

### Space：容纳各种资料的无限画布

富文本笔记、Markdown、网页摘录、PDF、Office 文档、图片、音频、视频、代码和普通文件都可以放在同一张无限画布上。使用分区、分组、标签和保存的视图整理内容，通过大纲导航，并可撤销最近 100 步编辑。系统**收集箱**可以暂存尚未归类的资料。

Knovum 支持开放的 **JSON Canvas** 格式（`.canvas`），可与兼容工具交换画布。整个 Space —— 包括内容、资源、关系、派生记录和引用 —— 也可以打包为便于迁移的 `.knovumspace` 文件。

### 针对资料提问，并追溯来源

使用已连接的模型，针对自己的资料提问。回答会引用所依据的段落，每条引用都会保留原文中的位置：PDF 页码、录音时间段或笔记标题。

引用会绑定到它所依据的来源版本。来源发生变化时，引用可能变为“已脱离”，而不是悄悄指向已经变化的内容。点击 PDF 引用，会在内置阅读器中打开原文对应页面并高亮引用段落。

### 搜索整个资料库

关键词与向量混合搜索完全在本机运行，使用 Qwen3 向量模型和 LanceDB。既可以在单个 Space 内查找，也可以搜索全部 Space；除了精确匹配关键词，还能按语义发现相关内容。

### 把想法连接成结构化知识

使用内置的六种知识对象 —— **概念、问题、主张、证据、实验和决策** —— 也可以创建自定义类型。通过“支持”“反驳”“回答”“依赖”“取代”“派生自”等关系将它们连接起来。

知识检查可以发现缺乏支持或存在争议的主张、缺少依据的证据、已被回答的问题，以及来源发生变化的笔记。你可以根据实际工作需要决定使用多少结构，不必一开始就搭建复杂体系。

### 基于选中内容创作，并保留来源记录

选中任意内容，都可以在不丢失上下文的情况下创建新资源：生成文档、生成图片或视频、去除背景、拆分图层、添加字幕。每次操作都会记录所用引擎、模型、输入及输入版本，方便追溯每个结果是如何产生的。

### 文档、媒体与语音工具

- **编辑器：** 富文本笔记与 Markdown，支持表格、图片、查找替换和行内引用。
- **导出：** 将笔记和网页摘录导出为 DOCX、PDF、PPTX、HTML、Markdown 和 TXT；支持图片与音频格式转换。
- **视频工作台：** 绝对时间轴、代理播放和烧录字幕。
- **语音工作室：** 多角色有声书与发音词典、旁白、配音、人声与伴奏分离、声音克隆和声音设计。
- **全局语音听写：** 按住全局快捷键说话，松开即可。Whisper 在本地完成转写，并将文字输入当前位于前台的应用。

### 连接兼容的 AI Agent

Knovum 内置 Agent，可以用待办规划任务，只读地研究你的 Space，并通过经过校验的原子变更集修改内容。你也可以通过 Agent Client Protocol（ACP）使用本机已安装的桌面 Agent，沿用它们自己的登录和工具。

Knovum 通过仅限本机回环的 MCP 服务器，向兼容的桌面 Agent 提供 Space 内容；同时也可以连接外部 MCP 服务器作为 Agent 扩展。技能、插件、连接器和应用可以集中管理。

## 本地优先的设计

你的知识库保存在自己的设备上：Space、文件、提取的文本、搜索索引和备份都存放在本地应用数据目录中。Knovum 不需要账号，也没有 Knovum 云服务。

本地 AI 与云端 AI 的边界是明确的：

- **受支持的能力优先在本地运行。** 编辑、关键词与语义搜索、转写、说话人识别、OCR、语音合成和下方列出的图像工具，在模型下载后都可以在本机运行。
- **问答和生成由你选择。** 连接你已有的模型服务，也可以使用兼容的本地接口，或把任务交给桌面 Agent。
- **云端模型只接收任务所需的上下文。** 选择云端模型执行操作时，Knovum 会将该操作所需的选中内容和上下文发送给你选择的服务商。服务商如何处理这些数据，仍以该服务商的条款和设置为准。

某项功能首次需要模型时才会下载对应模型。模型文件会按固定的 SHA-256 摘要校验，也可以在设置中删除；下载进度和取消操作都可以在任务中心查看。

## 使用自己的 AI 服务

画布编辑、关键词与语义搜索、转写，以及下面列出的端侧能力，**无需 AI 账号**即可使用。

需要 AI 问答和生成时，可以连接你已经在使用的服务：

**OpenAI · Anthropic · Google Gemini · DeepSeek · 通义千问 · 智谱 · 月之暗面 · 豆包 · MiniMax · 阶跃星辰 · 硅基流动 · Mistral · Groq · Grok · Together · OpenRouter · 可灵 · Ollama · 任意 OpenAI 兼容接口**

也可以把任务交给自带凭据的桌面 Agent。密钥保存在独立的本地密钥数据库中，从不写入日志。

## 下载与开始使用

| 平台    | 系统要求                    | 安装包                        |
| ------- | --------------------------- | ----------------------------- |
| macOS   | macOS 14 及以上，Apple 芯片 | `Knovum_<版本>_aarch64.dmg`   |
| macOS   | macOS 14 及以上，Intel      | `Knovum_<版本>_x64.dmg`       |
| Windows | Windows 10 2004 及以上，x64 | `Knovum_<版本>_x64-setup.exe` |

**[下载最新版本](https://github.com/madebyzan/knovum/releases/latest)**

目前安装包尚未经过 Apple 公证或 Authenticode 签名，因此首次启动时 macOS 和 Windows 会请你确认。之后的更新在应用内完成，并按发布签名校验。暂不支持 Linux 和 Windows on ARM。

**推荐的第一次尝试：** 新建一个 Space，加入两三份你正在处理的资料，再写下一条想要回答的问题。需要 AI 生成回答或内容时，再到设置中添加 AI 连接。

## 常见问题

<details>
<summary><strong>Knovum 可以离线使用吗？</strong></summary>

可以。模型下载完成后，编辑、整理、关键词与语义搜索、转写、说话人识别、OCR、语音合成和列出的图像工具都在本机运行。只有交给云端模型执行的操作才需要联网。

</details>

<details>
<summary><strong>数据存在哪里？会上传吗？</strong></summary>

你的资料库存放在本地应用数据目录中：SQLite 保存结构，LanceDB 保存语义索引，托管文件保存资源，独立的本地数据库保存密钥。内容不会上传到 Knovum 云端。当你让云端模型执行某项操作时，只有该操作所需的上下文会发送给你选择的服务商。

</details>

<details>
<summary><strong>使用 Knovum 需要 AI 账号吗？</strong></summary>

不需要账号即可使用 Knovum，以及本地编辑、整理、搜索、转写、OCR 和其他列出的端侧能力。AI 问答和生成需要连接模型，可以是云端服务，也可以是兼容的本地接口。

</details>

<details>
<summary><strong>它和 NotebookLM、Obsidian Canvas、Heptabase 有什么不同？</strong></summary>

Knovum 将基于资料的 AI 回答、用于思考的空间画布和结构化知识放进同一个本地优先的桌面工作空间。它还支持音视频处理、端侧 AI，并记录生成结果背后的来源和版本。

</details>

<details>
<summary><strong>可以通过 Ollama 使用本地大模型吗？</strong></summary>

可以。Ollama 和任意 OpenAI 兼容接口，都可以作为问答和内置 Agent 的模型。下面列出的端侧 AI 能力不需要 Ollama 或其他模型服务器。

</details>

<details>
<summary><strong>本地模型下载失败或连接超时怎么办？</strong></summary>

OCR、Whisper 转写和图像处理等端侧能力会在首次使用时下载模型权重。如果网络访问 `huggingface.co` 受限，可以配置标准环境变量 `HF_ENDPOINT` 使用镜像源，例如 `https://hf-mirror.com`。

- **Windows：** 配置永久用户环境变量。按 `Win + R`，输入 `sysdm.cpl`，打开**高级 → 环境变量**，添加变量 `HF_ENDPOINT`，值为 `https://hf-mirror.com`。也可以在命令提示符中运行 `setx HF_ENDPOINT "https://hf-mirror.com"`，然后重启 Knovum。
- **macOS / Linux：** 在 Shell 配置文件（如 `~/.zshrc` 或 `~/.bashrc`）中添加：

```bash
echo 'export HF_ENDPOINT="https://hf-mirror.com"' >> ~/.zshrc
source ~/.zshrc
```

下载后的模型文件会按固定的 SHA-256 摘要校验。

</details>

<details>
<summary><strong>可以导入已有笔记和画布吗？</strong></summary>

可以直接导入 Markdown、文档和媒体文件；画布支持通过开放的 JSON Canvas 格式（`.canvas`）导入和导出。整个 Space 也可以导出为便于迁移的 `.knovumspace` 文件。

</details>

<details>
<summary><strong>Knovum 开源吗？免费吗？</strong></summary>

是的。Knovum 是基于 [MIT 许可证](LICENSE) 发布的免费开源软件，你可以在许可证条款下自由使用、阅读、构建、修改和分发。

</details>

## 本地 AI 能力清单

以下能力都在本地运行。首次使用时下载所需模型，按固定的 SHA-256 摘要校验，也可以在设置中删除。

| 能力                     | 模型                                                         | 运行时                  |
| ------------------------ | ------------------------------------------------------------ | ----------------------- |
| 语音转写                 | Whisper（默认）/ Qwen3-ASR 1.7B + Silero VAD                 | whisper.cpp / audio.cpp |
| 说话人分离               | Pyannote 3.0 + ERes2Net                                      | sherpa-onnx             |
| 人声 / 伴奏分离          | HTDemucs                                                     | audio.cpp               |
| 语音合成、声音克隆与设计 | MOSS-TTS-Nano、VoxCPM2、IndexTTS2、Qwen3-TTS、Fun-CosyVoice3 | audio.cpp               |
| 逐词字幕对齐             | Qwen3 Forced Aligner                                         | audio.cpp               |
| 语义搜索向量             | Qwen3-Embedding-0.6B                                         | Candle                  |
| 去除背景                 | BiRefNet、BEN2、IS-Net Anime                                 | ONNX Runtime            |
| 图像超分辨率             | Real-ESRGAN                                                  | ONNX Runtime            |
| 局部修复                 | Big LaMa                                                     | ONNX Runtime            |
| 智能选区                 | MobileSAM、SAM 2.1 Hiera-Tiny                                | ONNX Runtime            |
| 相对深度估计             | Depth Anything V2                                            | ONNX Runtime            |
| 文字识别（扫描件、截图） | PP-OCRv6 small                                               | ONNX Runtime            |

在 Apple 芯片上，向量化与转写使用 Metal 加速。以上能力都不需要 Ollama 或其他模型服务器。

## 从源码构建

需要 **Node.js 22.13+** 与 **Corepack**、`rust-toolchain.toml` 中固定的 Rust 工具链，以及桌面构建使用的本地工具：**uv**、**CMake**、**Protobuf** 和 **LLVM/libclang**。具体版本见 [docs/installation.md](docs/installation.md)。

```bash
git clone https://github.com/madebyzan/knovum.git
cd knovum
corepack enable
pnpm install --frozen-lockfile
pnpm desktop:dev
```

首次构建会编译本地推理组件，耗时较长；之后启动会更快。

| 命令                | 用途                                                               |
| ------------------- | ------------------------------------------------------------------ |
| `pnpm desktop:dev`  | 启动桌面应用，并准备好 Agent 环境                                  |
| `pnpm check`        | 完整质量门禁：Web、Rust 和 Python                                  |
| `pnpm check:web`    | 类型检查、oxlint、dependency-cruiser、Vitest、knip、jscpd 和 oxfmt |
| `pnpm check:rust`   | cargo fmt、clippy（`-D warnings`）和工作区测试                     |
| `pnpm check:python` | Agent 运行时的 ruff、mypy（strict）与 pytest                       |
| `pnpm e2e`          | 基于调试构建运行 WebdriverIO 端到端测试                            |
| `pnpm package`      | 为当前平台构建桌面安装包                                           |

CI 会在 macOS（Apple 芯片与 Intel）和 Windows 上运行这些检查。推送 `v*` 标签会构建 macOS 与 Windows 安装包，并生成一个草稿 GitHub Release。

### 技术架构

Rust 核心负责数据、推理和系统集成，Vue 3 渲染层负责交互，两者之间通过生成的契约连接。本地模型运行在独立的推理进程中，内置 Agent 运行在随应用打包的 Python 运行时里。

**Tauri 2 · Rust · Vue 3 · TypeScript · SQLite (sqlx) · LanceDB · Candle · ONNX Runtime · whisper.cpp · Konva · Tailwind CSS v4**

更多内容见 [docs/system-architecture.md](docs/system-architecture.md)、[文档地图](docs/README.md) 和产品设计约定 [DESIGN.md](DESIGN.md)。

## 参与贡献

欢迎提交问题报告、聚焦的功能建议、翻译和文档改进。请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md) 和[行为准则](CODE_OF_CONDUCT.md)。

[提交 issue](https://github.com/madebyzan/knovum/issues) 时，请附上使用平台、Knovum 版本和复现步骤，并删除日志中的凭据。安全问题请按 [SECURITY.md](SECURITY.md) 私下报告。

如果 Knovum 对你有帮助，点一个 ⭐ 能让更多人发现它。

## 许可证

Knovum 基于 [MIT 许可证](LICENSE) 发布。第三方组件保留各自的许可证。

## Star 历史

<a href="https://star-history.com/#madebyzan/knovum&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=madebyzan/knovum&type=Date&theme=dark" />
    <img alt="madebyzan/knovum 的 Star 历史" src="https://api.star-history.com/svg?repos=madebyzan/knovum&type=Date" />
  </picture>
</a>
