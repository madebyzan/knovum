# 直接依赖能力清单

本清单按根目录 `package.json` 和 `src-tauri/Cargo.toml` 的**直接依赖**编写。能力描述只列该库对本项目有用的主要职责，并不表示项目已经用到它的全部 API。版本、可选特性和平台限制以两个清单及锁文件为准；审查实现时先核对已启用的 API，再决定是否删去自写逻辑。

## 前端运行时依赖（51）

| 库                                   | 主要能力                                     |
| ------------------------------------ | -------------------------------------------- |
| `@internationalized/date`            | 与时区、日历和地区设置有关的日期对象及运算。 |
| `@lucide/vue`                        | Vue 图标组件。                               |
| `@mozilla/readability`               | 从网页文档提取正文、标题等可读内容。         |
| `@tanstack/vue-query`                | 异步数据查询、缓存、失效、重试和状态管理。   |
| `@tanstack/vue-virtual`              | 大列表/网格的虚拟滚动。                      |
| `@tauri-apps/api`                    | 前端调用 Tauri 核心 IPC、窗口、事件等 API。  |
| `@tauri-apps/plugin-dialog`          | 系统原生文件选择和消息对话框。               |
| `@tauri-apps/plugin-fs`              | 按授权范围读写文件和目录。                   |
| `@tauri-apps/plugin-log`             | 前端向 Tauri 日志系统写入日志。              |
| `@tauri-apps/plugin-opener`          | 用系统默认程序打开文件或 URL。               |
| `@tauri-apps/plugin-os`              | 查询当前操作系统信息。                       |
| `@tauri-apps/plugin-store`           | 小规模持久化键值存储。                       |
| `@tiptap/core`                       | 无界面的富文本编辑器核心和扩展机制。         |
| `@tiptap/extension-file-handler`     | 编辑器内的文件拖入、粘贴处理。               |
| `@tiptap/extension-find-and-replace` | 编辑器内容查找与替换。                       |
| `@tiptap/extension-highlight`        | 文本高亮标记。                               |
| `@tiptap/extension-image`            | 图片节点。                                   |
| `@tiptap/extension-list`             | 列表节点和命令。                             |
| `@tiptap/extension-subscript`        | 下标文本。                                   |
| `@tiptap/extension-superscript`      | 上标文本。                                   |
| `@tiptap/extension-table`            | 表格节点、行列操作。                         |
| `@tiptap/extension-text-align`       | 文本对齐。                                   |
| `@tiptap/extension-text-style`       | 行内文本样式。                               |
| `@tiptap/extension-typography`       | 排版替换规则。                               |
| `@tiptap/extensions`                 | Tiptap 通用扩展集合。                        |
| `@tiptap/markdown`                   | 编辑器内容与 Markdown 的解析/序列化。        |
| `@tiptap/pm`                         | Tiptap 使用的 ProseMirror 模块入口。         |
| `@tiptap/starter-kit`                | 常用编辑功能的组合配置。                     |
| `@tiptap/vue-3`                      | Tiptap 的 Vue 组件和组合式接口。             |
| `@vueuse/core`                       | Vue 组合式浏览器、定时器、事件和响应式工具。 |
| `clsx`                               | 条件式组合 CSS 类名。                        |
| `dompurify`                          | 净化将要渲染的 HTML。                        |
| `es-toolkit`                         | 常用集合、对象、函数工具。                   |
| `fast-fuzzy`                         | 模糊文本匹配和搜索。                         |
| `konva`                              | Canvas 场景图、绘制和交互。                  |
| `markdown-it`                        | Markdown 解析与 HTML 渲染。                  |
| `p-limit`                            | 限制异步任务并发数。                         |
| `pathe`                              | 跨平台路径字符串处理。                       |
| `pinia`                              | Vue 应用共享状态与 store。                   |
| `rbush`                              | 二维矩形空间索引和范围检索。                 |
| `reka-ui`                            | 无样式、可访问的 Vue 交互组件原语。          |
| `tailwind-merge`                     | 合并互相冲突的 Tailwind 工具类。             |
| `tauri-plugin-clipboard-api`         | Tauri 剪贴板插件的前端接口。                 |
| `tinykeys`                           | 键盘快捷键匹配。                             |
| `ufo`                                | URL 解析、拼接和规范化。                     |
| `vue`                                | 组件、响应式系统和生命周期。                 |
| `vue-i18n`                           | 文案翻译、地区设置和本地化格式化。           |
| `vue-konva`                          | Konva 场景节点的 Vue 绑定。                  |
| `vue-router`                         | 页面路由和导航。                             |
| `vue-sonner`                         | Toast 通知 UI。                              |
| `zod`                                | 运行时数据验证和 TypeScript 类型推导。       |

## 前端开发依赖（25）

| 库                      | 主要能力                                 |
| ----------------------- | ---------------------------------------- |
| `@tailwindcss/vite`     | Tailwind CSS 的 Vite 集成。              |
| `@tauri-apps/cli`       | Tauri 开发、构建与打包命令。             |
| `@types/node`           | Node.js 的 TypeScript 类型。             |
| `@types/rbush`          | RBush 的 TypeScript 类型。               |
| `@vitejs/plugin-vue`    | Vite 的 Vue 单文件组件编译。             |
| `@vue/test-utils`       | Vue 组件测试的挂载与交互工具。           |
| `@wdio/cli`             | WebdriverIO 命令行测试入口。             |
| `@wdio/globals`         | WebdriverIO 全局测试 API。               |
| `@wdio/local-runner`    | WebdriverIO 本地测试执行器。             |
| `@wdio/mocha-framework` | WebdriverIO 的 Mocha 测试框架。          |
| `@wdio/spec-reporter`   | WebdriverIO 文本测试报告。               |
| `@wdio/tauri-plugin`    | Tauri 应用的 WebdriverIO 插件侧集成。    |
| `@wdio/tauri-service`   | WebdriverIO 启动/管理 Tauri 应用的服务。 |
| `dependency-cruiser`    | 依赖图和模块边界规则检查。               |
| `happy-dom`             | 测试使用的 DOM 环境。                    |
| `jscpd`                 | 重复代码检测。                           |
| `knip`                  | 未使用文件、导出及依赖检测。             |
| `oxfmt`                 | 代码格式化与格式检查。                   |
| `oxlint`                | JavaScript/TypeScript 静态 lint。        |
| `tailwindcss`           | 工具类 CSS 生成。                        |
| `tw-animate-css`        | Tailwind 动画工具类。                    |
| `typescript`            | TypeScript 编译器与类型检查。            |
| `vite`                  | 前端开发服务器和构建。                   |
| `vitest`                | 单元/组件测试运行器。                    |
| `vue-tsc`               | Vue 单文件组件的 TypeScript 类型检查。   |

## Rust 构建依赖（1）

| 库            | 主要能力                                     |
| ------------- | -------------------------------------------- |
| `tauri-build` | 在 Cargo 构建阶段生成 Tauri 所需资源和配置。 |

## Rust 通用依赖（57，含 2 个仅用于 e2e 的可选依赖）

| 库                             | 主要能力                                                      |
| ------------------------------ | ------------------------------------------------------------- |
| `tauri`                        | 桌面应用运行时、命令、事件、窗口、托盘和资源协议。            |
| `tauri-plugin-clipboard`       | 系统剪贴板读写和变更监听。                                    |
| `clipboard-rs`                 | 跨平台剪贴板数据操作。                                        |
| `tauri-plugin-dialog`          | 原生对话框。                                                  |
| `tauri-plugin-fs`              | 受权限控制的文件系统接口。                                    |
| `tauri-plugin-log`             | Tauri 日志输出和目标配置。                                    |
| `tauri-plugin-updater`         | 应用更新检查和安装。                                          |
| `tauri-plugin-opener`          | 交给系统打开文件或 URL。                                      |
| `tauri-plugin-os`              | 操作系统信息。                                                |
| `tauri-plugin-single-instance` | 应用单实例和再次启动事件。                                    |
| `tauri-plugin-store`           | 本地键值持久化。                                              |
| `tauri-plugin-window-state`    | 窗口位置/尺寸状态保存与恢复。                                 |
| `serde`                        | Rust 数据结构序列化和反序列化。                               |
| `log`                          | 统一日志门面。                                                |
| `base64`                       | Base64 编解码。                                               |
| `ring`                         | 加密、签名与摘要原语。                                        |
| `infer`                        | 根据文件内容识别类型。                                        |
| `mime`                         | MIME 类型解析和表示。                                         |
| `mime_guess`                   | 根据文件扩展名推测 MIME 类型。                                |
| `tauri-plugin-wdio`            | 可选 e2e 模式的 Tauri WebdriverIO 接入。                      |
| `tauri-plugin-wdio-webdriver`  | 可选 e2e 模式的 WebDriver 服务。                              |
| `image`                        | PNG/JPEG/BMP/TIFF/WebP 图片解码、编码与处理（按已启用特性）。 |
| `reqwest`                      | HTTP 客户端、JSON、查询参数、流和 multipart 请求。            |
| `zip`                          | ZIP 压缩包读写。                                              |
| `tar`                          | TAR 归档读写。                                                |
| `bzip2`                        | bzip2 压缩流读写。                                            |
| `flate2`                       | DEFLATE/gzip 等压缩流。                                       |
| `encoding_rs`                  | 常见文本字符编码转换。                                        |
| `genai`                        | 多提供商生成式 AI 调用接口。                                  |
| `rig-agent`                    | AI agent、模型和工具调用构件。                                |
| `docx-rs`                      | DOCX 文档生成和内容结构处理。                                 |
| `pulldown-cmark`               | Markdown 事件流解析。                                         |
| `typst-as-lib`                 | 作为 Rust 库运行 Typst 排版。                                 |
| `typst-layout`                 | Typst 布局数据结构。                                          |
| `typst-pdf`                    | 将 Typst 文档导出为 PDF。                                     |
| `fontdb`                       | 字体发现、加载和匹配。                                        |
| `futures`                      | Future/Stream 组合工具。                                      |
| `office_oxide`                 | Office 文档读取与解析。                                       |
| `pdf-inspector`                | PDF 内容和结构检查。                                          |
| `serde_json`                   | JSON 值及 Serde JSON 编解码。                                 |
| `sqlx`                         | SQLite 异步查询、宏和迁移。                                   |
| `sha2`                         | SHA-2 摘要。                                                  |
| `tokio`                        | 异步运行时、任务、计时器、进程、网络和文件 I/O。              |
| `uuid`                         | UUID v4 生成。                                                |
| `fnv`                          | FNV 哈希实现。                                                |
| `lancedb`                      | LanceDB 向量/列式数据存储与检索。                             |
| `arrow-array`                  | Apache Arrow 列式数组。                                       |
| `arrow-schema`                 | Apache Arrow 字段和模式。                                     |
| `rmcp`                         | MCP 协议服务端与传输。                                        |
| `axum`                         | HTTP 路由、中间件和服务端处理。                               |
| `tokio-util`                   | Tokio 取消令牌等异步辅助工具。                                |
| `walkdir`                      | 递归遍历目录树。                                              |
| `agent-client-protocol`        | Agent Client Protocol 数据结构与通信。                        |
| `chrono`                       | 日期时间与时钟。                                              |
| `schemars`                     | Rust 类型生成 JSON Schema。                                   |
| `ts-rs`                        | 从 Rust 类型导出 TypeScript 类型。                            |
| `jsonschema`                   | JSON Schema 编译与实例验证。                                  |

## macOS 专用 Rust 依赖（8）

| 库                              | 主要能力                                  |
| ------------------------------- | ----------------------------------------- |
| `block2`                        | Objective-C block 互操作。                |
| `objc2`                         | Objective-C 运行时绑定。                  |
| `objc2-core-graphics`           | Core Graphics 绘制、位图与 PDF 接口绑定。 |
| `objc2-core-foundation`         | Core Foundation 字符串、URL 等类型绑定。  |
| `objc2-foundation`              | Foundation 字符串、URL、错误等类型绑定。  |
| `objc2-image-io`                | Image I/O 图片目标和编码接口绑定。        |
| `objc2-quick-look-thumbnailing` | Quick Look 缩略图生成接口绑定。           |
| `quicklook`                     | macOS Quick Look 预览集成。               |

## Windows 专用 Rust 依赖（1）

| 库        | 主要能力                                                                   |
| --------- | -------------------------------------------------------------------------- |
| `windows` | WinRT/Win32 API 绑定；当前启用 PDF、存储、图像、Shell、窗口和 COM 等接口。 |

能力参考入口：[Vue 官方文档](https://vuejs.org/guide/reusability/composables.html)、[Tauri 官方插件目录](https://v2.tauri.app/plugin/)、[Tiptap 官方扩展文档](https://tiptap.dev/docs/editor/extensions/overview)、[npm 包页面](https://www.npmjs.com/)、[docs.rs](https://docs.rs/)。项目实际可用能力还受已安装版本、Cargo features 和 Tauri permissions 限制。

## macOS Apple Silicon 专用 Rust 依赖（2）

| 库              | 主要能力                                 |
| --------------- | ---------------------------------------- |
| `core-graphics` | macOS Core Graphics 原生绘图和图像接口。 |
| `objc2-app-kit` | AppKit 应用和工作区 API 绑定。           |

## macOS Apple Silicon 与 Windows x64 专用 Rust 依赖（3）

| 库                             | 主要能力                      |
| ------------------------------ | ----------------------------- |
| `cpal`                         | 跨平台音频输入/输出设备与流。 |
| `hound`                        | WAV 音频文件读写。            |
| `tauri-plugin-global-shortcut` | 注册和管理系统全局快捷键。    |
