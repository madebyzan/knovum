# 架构边界补充整改

前一轮不能据质量门通过认定所有职责和命名均已完成。用户指出PPTX重复实现后重新核查实际调用链。

## 实施计划

1. 确认前端生成入口的真实消费者，区分产品调用与测试独占。
2. 用现有Rust PPTX引擎的输出回归锁定当前工作流行为。
3. 删除没有产品消费者的前端演示文稿生成、worker及专属IPC，移除PptxGenJS与专属override。
4. 将纯PPTX生成迁入documents/generation/pptx，Spaces只负责授权、来源修订校验及结果持久化；纠正引擎元数据。
5. 检查相关目录和方法命名，核对生产可达性及跨层依赖，更新架构守卫和权威文档。
6. 执行针对性测试、类型/lint/架构/死代码检查；不执行产品build或打包。

## 已确认事实

- 真实UI通过presentation-production工作流在Rust执行render-presentation。
- createPresentationFromDocument/presentationInputs没有调用者；worker仅被这段旧代码和自己的测试引用。之前所谓“交互与后台两种上下文”不是当前产品需要，R34保留判断撤回。
- Rust生成器位于Spaces内，违背documents拥有纯文档生成的既有边界。
- Rust持久化硬编码pptxgenjs:4.0.1，与实际引擎不符。

实际工作流目前是文本演示文稿；删除不可达图片生成代码不改变产品现有功能，不虚构图片功能已经迁移。

## 可达性补查

从src/main.ts追踪依赖，排除测试、声明、生成合同，并补计new URL的worker边。发现旧caption、资源refresh/action/path扫描、第二套canvas-document-sync、静态未消费detail-strategy、structured-input、model-name hook、旧Knowledge菜单/状态组件、旧SceneCue/FilterChip和UiTagsInput均无产品消费者。删除其专属测试，保留实际画布投影测试。不是按“短文件”删除，是按真实调用关系删除。新增生产可达性检查，防止测试或Vue插件自动入口把死产品代码伪装成可达。

## 命名与合同第二批计划

- `json-canvas-client`实际是编解码IPC操作，改为json-canvas-codec；`space-edit-client`是编辑命令/查询操作集合，改为space-edit-operations。保留实际TanStack QueryClient和reqwest Client名称。
- 扩大现有ts-rs导出范围：结构编辑/JSON Canvas、捕获工作流、AI/Agent、本地运行资源及其余实际IPC DTO。Rust是跨进程类型来源；运行时Zod约束与UI派生模型不作为独立wire类型重复维护。
- 生成契约按业务归属：Spaces DTO 和 JSON Schema 直接输出到 features/spaces/contracts/generated；通用 AI、资源状态、错误契约输出到 shared/contracts/generated。Rust 类型的 ts-rs export_to 决定唯一输出路径。Settings 使用应用层注入的动作选项投影，不引用 Spaces 契约。
- 已确认foundation/ai/agents同时调用Space产物命令，拆回Space能力owner；目录/方法按实际职责命名，不通过“换一个泛化后缀”解决职责问题。
- 保留Knip自动发现的实际入口（显式main.ts会重复），另用生产可达性守卫补查仅被测试消费的导出；同名Rust/TS符号先区分类型别名、UI模型与真实重复，逐项处理。

## 已实施的职责与命名调整

| 原位置/名称                                              | 目标位置/名称                                  | 原因                                                                      |
| -------------------------------------------------------- | ---------------------------------------------- | ------------------------------------------------------------------------- |
| foundation/ai/agents 的 Space 产物命令                   | features/spaces/agent-creation/agent-artifacts | 业务授权与产物生命周期属于 Spaces；foundation 只保留 Agent 发现           |
| foundation/ai/space-action-catalog                       | features/spaces/workflows/action-catalog       | 动作定义是 Space 业务能力；Settings 通过 app 路由的显式 prop 获取加载函数 |
| foundation/system-preview/system-preview                 | features/spaces/assets/asset-preview           | 此桥接调用需要 Space 身份、节点与私有资产权限，不能伪装成通用 OS 预览     |
| json-canvas-client                                       | json-canvas-codec                              | 实际是解析/序列化接口，不管理连接或客户端生命周期                         |
| space-edit-client                                        | space-edit-operations                          | 实际是编辑命令、收据和快照操作                                            |
| Web scene/space/node/relation/chunk/knowledge repository | 对应 *-operations                              | 实际调用 Rust IPC，没有访问数据库的职责                                   |
| space-projection-repository                              | space-projection                               | 会话加载及 UI 投影解码                                                    |
| embedding-repository                                     | retrieval-index                                | 索引状态、构建和检索操作                                                  |
| credential-repository                                    | credential-storage                             | 密钥标识校验、序列化及专用存储调用                                        |
| canvas-view-state/clipboard/transfer/import repository   | 对应 canvas-view-state 或 *-operations         | 按实际保存/导入/转移操作命名                                              |
| global-search-repository                                 | global-search                                  | 查询和摘要投影，不是持久化仓库                                            |
| Web NodeDataChange                                       | NodePayloadChange                              | 历史 before/after，区别于 Rust IPC expectedData/data                      |
| Web TranslationRequest                                   | TranslationInput                               | UI 文本和目标语言输入，区别于 Rust 已解析的执行请求                       |

`query-client.ts` 和 Rust `ai/transport/client.rs` 保留，分别实际拥有 QueryClient 与 reqwest Client。没有把所有短文件机械合并，也没有新增 manager/service 层。

## 协议收敛方式

结构编辑、JSON Canvas、捕获工作流、AI/Agent、资源安装、用量统计、助手请求、供应商和动作定义，以及更新状态、派生失败、图片操作、Space 包进度的 wire 类型均从 Rust 导出。前端导入生成类型；需要外部输入校验时用 Zod，并以生成类型约束校验结果。

供应商、执行能力、路由用途、Agent、AI 错误码、用量用途的 runtime 值与知识分段字节上限由 Rust 一并生成。删除只由测试消费的 Web 知识分段算法及其专属测试，真实分段仍由 Rust 负责。

捕获工作流绑定的枚举字段原来输出 input_slot_id/step_id，前端读取 inputSlotId/stepId；改为明确 camelCase，并增加三个来源分支的序列化/反序列化回归。旧开发期数据若采用原错误字段，不添加兼容读取或自动改写用户数据。

外观的 UI 模型允许 undefined，而 JSON wire 值不允许。复用 serializeCanvasAppearance 显式去掉缺省字段，移除调用点强转；附件在 IPC 边界复制为普通数组，保留 UI 的只读所有权。

## 风险边界

- 文件移动、命名、生成类型替换属于结构整理，IPC 命令名和合法数据表达保持既有值。
- 捕获绑定 camelCase 属于协议 bug 修复；应与纯文件整理区分审查。
- PPTX 引擎路径和来源元数据纠正、Markdown 解析统一、超 100 页显式拒绝属于行为整改，已有输出/负例回归。
- native asset preview 的 Windows 实现已存在于 Rust host/handler；本轮移动业务桥接，不把静态检查描述成 Windows 实机验证。
- 生成代码保留生成器原输出，排除 authored lint/formatter；继续接受 TypeScript、依赖边界和生成回归检查。

## 生成契约的能力边界修正

此前把所有 wire DTO 集中到 shared 的处理范围过大，现已修正：

| 内容                                                                      | 唯一生成位置                                        | Rust 定义归属 |
| ------------------------------------------------------------------------- | --------------------------------------------------- | ------------- |
| 节点、结构编辑、JSON Canvas、知识、工作流、助手、Space 产物任务           | `src/features/spaces/contracts/generated`           | `spaces`      |
| AI 提供商、模型、Agent 发现、通用 AI 请求及错误、资源安装、用量、更新状态 | `src/shared/contracts/generated`                    | 对应基础能力  |
| Space 工作流与动作参数 JSON Schema、知识分段上限                          | `src/features/spaces/contracts/generated`           | `spaces`      |
| 通用 AI enum runtime 值                                                   | `src/shared/contracts/generated/contract-values.ts` | `ai`          |

Rust 类型通过 ts-rs 的 `export_to` 声明输出归属；递归依赖由生成器生成相对 import，不能在生成文件中手工搬运或修补。中立 JSON 值定义仍只有一份。产物任务请求、事件、权限选项和恢复记录的 Rust 定义移到 `spaces/agent_artifacts/contracts.rs`，通用 Agent 发现和产物分类类型保留在 AI 能力。

Settings 不读取 SpaceActionDefinition：app/router 把动作目录投影成 Settings 自己需要的 `GenerationActionOption`（id、routingPurpose），经 props 注入。这是 UI 选择项，不是重复 IPC 协议。现有 feature isolation、foundation independence、shared leaf 规则覆盖生成文件，不能通过放进 shared 来规避能力隔离。

删除无源定义、无引用的 BatchItemState、MisfirePolicy、OverlapPolicy 旧生成文件；修正依赖守卫中的旧 repository 路径，避免命名整改后规则漏检。新增 Rust 输出归属回归，明确 Space 编辑/动作/产物请求与通用 AgentKind 的输出目录。

本次验证：官方契约生成测试通过；Web 136 个测试文件、461 项测试通过，TypeScript、Oxlint、依赖图（1241 模块、3940 条依赖）、生产源码可达性、Knip、重复率门禁通过；格式修复后全仓 formatter 检查通过。Rust 架构与 rustfmt 检查通过。未执行产品打包或桌面平台实机验证。

Rust 最终复验：Clippy all-targets 在 `-D warnings` 下通过；全量 Rust 测试通过，包含契约归属回归、文档导出和内置抠图模型推理。产品打包与桌面平台实机验证仍未执行。
