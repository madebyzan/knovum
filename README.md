<p align="center">
  <img src="public/app-icon.png" width="80" alt="Knovum" />
</p>

<h1 align="center">Knovum</h1>

<p align="center">
  <strong>A canvas for ideas and the sources behind them.</strong>
</p>

<p align="center">
  A local-first desktop knowledge space for research, learning, and creation.<br />
  macOS · Windows · Early development
</p>

<p align="center">
  <a href="https://github.com/zandko/knovum/actions/workflows/quality.yml"><img src="https://github.com/zandko/knovum/actions/workflows/quality.yml/badge.svg" alt="Quality CI" /></a>
  <a href="https://github.com/zandko/knovum/releases"><img src="https://img.shields.io/github/v/release/zandko/knovum?color=blue" alt="Latest Release" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/zandko/knovum" alt="License" /></a>
</p>

<p align="center">
  <a href="#from-a-question-to-a-research-brief">Explore the workflow</a> ·
  <a href="#get-started">Get started</a> ·
  <a href="docs/README.md">Documentation</a> ·
  <a href="README.zh-CN.md">简体中文</a>
</p>

Research rarely stays in one format. A paper sits beside a saved article, a recording, a question,
and a half-written draft. The useful part is how they connect.

Knovum brings that work into a **Space**: an infinite canvas where sources, ideas, and the things you
create live together. Build an argument, inspect its evidence, and turn selected material into new
work with a recorded trail back to its inputs.

![An example knowledge flow: saved sources support a working claim, inform a question, and become a research brief with recorded inputs.](docs/assets/readme/source-to-work.svg)

<p align="center"><sub>An example knowledge flow, shown schematically. Not an application screenshot.</sub></p>

## From a question to a research brief

Imagine a personal research project: you have a few papers, articles, and a lecture recording.
You want to understand the topic and write a brief you can revisit later.

### 1. Bring the context into view

Save the articles, import the papers and recording, and add your own notes. Keep their relationships
visible on the canvas, with groups, sections, and saved views for different parts of the project.
Website captures retain their URLs; extracted PDF passages retain page locations. The original files
remain accessible through your desktop applications.

### 2. Make the reasoning explicit

Give your project a question. Add **claims, evidence, concepts, experiments, and decisions** as typed
knowledge objects, then connect them with meaningful relations such as **supports** and **contradicts**.
Knowledge checks flag unsupported claims, accepted claims with active contradictions, and evidence
whose sources have changed. Use them to review gaps in the argument. A generated draft and an
accepted claim remain separate things.

### 3. Work from the material you selected

Find passages with keyword or local semantic search. Ask a question against your sources and follow
the answer's citations back to their locations. Select relevant material to draft a document or
create a visualization using a configured model or desktop Agent.

Your result becomes another resource in the Space: something you can edit, search, connect, and use
as input to the next step. Local transcription can bring the recording into the same research flow.

### 4. Keep the trail, then reuse the process

Inspect a result's derivation to see **which inputs and revisions it used, which action ran, and which
engine or model produced it**. If a source changes, the recorded inputs still identify the version used;
the earlier result is never silently rewritten to match new material.

When a sequence works, capture its completed derivations as a **reusable workflow**. Bind it to fresh
sources for the next project and keep the intermediate results alongside the final output.

**The connection stays useful after the output is finished.** You can return to the evidence behind
an idea, inspect how a resource was created, or repeat the process with new inputs.

## Beyond the research brief

| Capability                       | What you can do                                                                                                                                                                                                              |
| -------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **A spatial workspace**          | Mix Markdown, notes, websites, documents, images, audio, video, and code. Organize with groups, sections, tags, and saved views.                                                                                             |
| **Search across your knowledge** | Retrieve material within a Space or across Spaces, by keyword or meaning. Local semantic search uses on-device embeddings.                                                                                                   |
| **Media and document creation**  | Transcribe locally, remove image backgrounds, create subtitled videos, and convert supported documents into formats such as DOCX, PDF, HTML, and PPTX. Generative media actions depend on the selected model's capabilities. |
| **Portable Spaces**              | Export `.knovumspace` packages with content, managed assets, relations, and derivation history. Use JSON Canvas for canvas exchange.                                                                                         |

Supported formats, model requirements, and conversion limits are described in the [Spaces guide](docs/spaces.md),
[local models guide](docs/local-models.md), and [resource export guide](docs/resource-export.md).

## Local by default, AI by choice

Your Spaces, managed files, extracted text, and search indexes are stored on your machine. Canvas
editing and keyword search work without a generative AI connection. Semantic indexing and transcription
run locally once you download their models.

For generative actions, choose a model connection or a desktop Agent. Connections include OpenAI,
Anthropic, Google Gemini, OpenAI-compatible endpoints, and Ollama. Desktop Agents use their own
authentication and available tools; Skills, plugins, and MCP connections can extend their capabilities.

Selected text or media may be sent to the executor you choose. What runs locally and what uses an
external service depends on the action and connection. See [AI and local model behavior](docs/local-models.md)
for the details.

## Get started

**Knovum is in early development.** The setup below runs the desktop app from source. Pre-release data
formats may change; export important Spaces before updating a development checkout.

| Platform | Target                                              |
| -------- | --------------------------------------------------- |
| macOS    | macOS 14 or later, Apple Silicon and Intel          |
| Windows  | Windows 10 version 2004 / build 19041 or later, x64 |

Install **Node.js 22.13+**, **Corepack**, the repository's **Rust toolchain**, and the native prerequisites
in the [installation guide](docs/installation.md). These include uv, CMake, Protobuf, LLVM/libclang,
and the platform tooling required by Tauri. The pnpm version is pinned in `package.json`.

```bash
git clone https://github.com/zandko/knovum.git
cd knovum
corepack enable
pnpm install --frozen-lockfile
pnpm desktop:dev
```

The development command prepares the Agent environment and native components before opening the app.
The first build downloads dependencies and takes longer than subsequent starts.

Start with one question and two or three sources. Create a Space, add the material, and write a note
about what you want to find out. Configure an AI connection when you want to ask source-grounded
questions or create a generated result. Local models download when you request a capability that
needs them, with progress and cancellation.

## Documentation and contributing

- [Installation](docs/installation.md) — prerequisites and desktop setup.
- [Spaces](docs/spaces.md) — capture, canvas organization, knowledge, workflows, and export.
- [Local models](docs/local-models.md) — on-device capabilities, downloads, and hardware requirements.
- [Troubleshooting](docs/troubleshooting.md) — setup, build, and runtime problems.
- [Contributing](CONTRIBUTING.md) — how to report issues and propose changes.
- [Documentation index](docs/README.md) — all product and engineering guides.

Bug reports, focused feature proposals, translations, and documentation improvements are welcome.
For [issue reports](https://github.com/zandko/knovum/issues), include your platform, version or commit,
and reproduction steps. Remove credentials and private material from any attached logs.

Built with **Tauri 2, Vue 3, TypeScript, and Rust**, with **SQLite** for structured knowledge and
**LanceDB** for vector search. Run `pnpm check` for the repository's web, Rust, and Python checks;
see [packaging](docs/packaging.md) for desktop builds.

## License

Knovum is **source-available for personal noncommercial use** under the
[Knovum Personal Noncommercial License 1.0](LICENSE). Individuals may use, modify, and share it
noncommercially under its terms. Organizational use, professional/client work, and commercialization
require separate prior written authorization. Third-party components retain their own licenses.

[中文许可说明](docs/licensing.md)
