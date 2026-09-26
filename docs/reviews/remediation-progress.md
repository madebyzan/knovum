# 全项目整改执行记录

依据：[审查方案](project-audit-2026-09-26.md)。架构边界补查及R34判断纠正见[补充整改](boundary-followup.md)。本文件记录执行证据，不替代权威架构文档。

约束：单 agent；不运行 build/打包；不自动删除或改写现有用户数据库；无新增项目直接依赖、无新增通用架构层。

## 执行顺序

1. 质量基线：当前协议 fixtures、类型、质量门、格式与CI失效路径。
2. 授权/取消：先负例锁定审批身份、流终态、等待请求释放。
3. 资产：提交前回滚、提交后回收、预览发布保护。
4. 合同：当前DB baseline、DTO、错误码；旧库只显式拒绝、不静默重置。
5. 状态：session唯一来源、编辑身份、run订阅寿命。
6. 删除：空provider/转发/别名、未用直接依赖；保留真实边界fallback。
7. 结构：深层优先入口迁移、准确命名、守卫与文档同步。
8. 条件优化：只落实有实际测量或明确成本证据的优化；PPTX原保留判断已在补查中撤回，详见 boundary-followup。

## 验证原则

行为改变先补故障/负例；纯整理保留原用例；每个批次先实现后独立自审。
静态检查和Web tests执行；Rust行为改动执行定向cargo test（包含测试目标编译）；不执行产品build、桌面构建或打包。未执行的平台检查不写成通过。

## 第一轮执行状态（补查前证据）

- 质量基线、结构、生命周期、依赖与协议批次已实现；Rust全量、定向HTTP回归与Clippy均已通过。
- R01–R37：随批次记录实际解决/条件保留/验证缺口，不凭计划标记完成。

## 已完成批次与证据

- 质量基线：修复 ShallowRef、当前协议 fixtures、i18n keys、IPC扫描器完整字面量识别；第一轮完整 Web 测试 131文件/446用例通过，初始15失败已消除。
- 授权和流：审批消费绑定 grant/Space；text stream 异常EOF与取消明确失败，删除 detached/unbounded frame 转发；ModelTurn delta 改为32条 bounded queue，发送等待可取消。对应授权、流终态、背压与真实HTTP预算回归已通过。
- 资源：website绑定提交成功后的旧资源清理不再反向标记失败，3个回归通过；预览采用独立生成路径、发布后回收；批量资产清理由逐条IPC变为一次查询引用图。
- 会话：image editor/comment 随Space切换及node删除释放草稿与尺寸缓存，2个身份/删除回归通过；route leave失败记录日志并显示错误；conversation/workflow共享run订阅，2个回归通过。
- 依赖：删除3个未使用Cargo直接依赖；精确范围替换剪贴板Valibot并删除PptxGenJS未使用的image-size；真实插件payload回归通过，官方registry生产audit无已知漏洞。锁文件只变化这两个依赖子树。
- 结构：消除Rust文件/目录同stem冲突，workspace定义重命名，规则路径同步；删除3个空provider、3个export转发文件与无用别名。Rust架构门通过，最终Web依赖图1126模块/3795边无违规。
- 数据：Spaces合并为当前v1 baseline，旧历史显式拒绝且不自动重置；旧迁移版本与旧checksum两种拒绝都保留用户行，真实DB回归通过。session改强类型DTO；官方生成器重生成32份合同，经格式化后零变化。

完整Web质量门已通过；Rust全量单元测试177通过，收尾HTTP定向2通过，Clippy所有target以-D warnings通过。

## 整改项对照

| 审查编号 | 实际处理                                                                        | 验证/验收点                                                                                                                             |
| -------- | ------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| R01      | 审批消费绑定 grant、Space、action、digest、过期与单次消费                       | 真实 SQLite 身份/决策/过期/并发4测试已通过                                                                                              |
| R02      | 流由消费者直接驱动，取消和无终态 EOF 返回失败                                   | 终态/取消/EOF/预算4测试已通过                                                                                                           |
| R03–R05  | 类型、当前协议 fixtures、IPC 地址扫描器修复                                     | Web 136文件456用例通过；authority通过                                                                                                   |
| R06      | 两个精确依赖子树修复，未扩大版本升级范围                                        | 真实剪贴板/PPTX worker回归；生产audit无已知漏洞                                                                                         |
| R07      | 当前v1基线；obsolete version/checksum显式拒绝                                   | 旧历史拒绝且用户行保留测试已通过；不处理真实用户库                                                                                      |
| R08      | website发布成功后回收失败只记录，不回滚新绑定                                   | 新增故障回归，3测试通过                                                                                                                 |
| R09      | 一次批量IPC，去重候选并复用一次引用图                                           | 3个批量协议/空集合回归通过                                                                                                              |
| R10      | UUID预览，先发布后回收；失败只回收本次生成文件                                  | 共享/软删除预览引用真实DB回归已通过；不宣称文件与DB原子提交                                                                             |
| R11      | try_get替代panic读取；坏locator JSON返回错误                                    | 新增损坏坐标DB负例已通过                                                                                                                |
| R12      | 只读授权使用普通读事务                                                          | 架构/代码检查；审批写事务仍BEGIN IMMEDIATE                                                                                              |
| R13      | 183个原String错误命令改code/message；集中IPC规范化，删除AI重复包装              | IPC红/绿回归通过；Web类型/lint通过；Rust全量单元测试通过                                                                                |
| R14      | Permission RAII释放；poison显式错误；锁不跨await                                | 请求future丢弃释放pending测试已通过                                                                                                     |
| R15      | Rust强类型session记录，ts-rs实际生成，Web复用既有Zod schema                     | 合同生成测试已执行；Web类型通过                                                                                                         |
| R16      | 消息DTO、branch职责分离；Role闭合枚举                                           | Rust架构门通过；Rust全量单元测试通过                                                                                                    |
| R17      | 保留单一session facade；明确既有content/media/history/workbench所有权与私有边界 | 见docs/spaces.md；图片瞬时草稿已本地化；删除只写不读的thumbnailCaptureActive全局状态。没有制造第二份nodes缓存或机械拆成互相同步的stores |
| R18–R19  | 编辑身份包含spaceId；删除/切换释放缓存；导航失败可见                            | 生命周期2回归通过；session旧回归继续通过                                                                                                |
| R20      | 一run ID一Channel；领域任务映射保留在能力owner                                  | 共享/失败/重订阅2回归通过                                                                                                               |
| R21      | 条件保留现有事务snapshot写入                                                    | 无实际大Space性能测量，按方案保留；新增幂等/回滚/竞争retry真实事务测试已通过                                                            |
| R22–R24  | 删除3空provider、宽泛unused_imports allow、3格式转发文件                        | 静态门通过；Rust实际编译和单元测试通过                                                                                                  |
| R25–R27  | 模块入口迁移、物理同stem守卫、失效规则/SQLpreload/CI路径修复                    | Web/Rust架构门与IPC覆盖门通过；迁移表module-moves.json                                                                                  |
| R28      | 删除3个未用直接Rust依赖；reqwest显式query feature                               | Cargo.lock同步；第一次实际Rust编译已通过，当前错误协议轮次也已通过                                                                      |
| R29–R31  | test-support、日志、字段级日期错误、具名options、准确命名及provider wire ID     | Web lint、Knip通过；不保留rename-only旧名兼容入口                                                                                       |
| R32      | 32项delta队列、8MiB模型输出、64MiB图片下载、JSON上限、单次模型调用剩余时限      | 模型/真实HTTP边界测试已通过；不是全应用通用限流系统                                                                                     |
| R33      | 无效显式环境路径返回错误，未设置才允许bundled解析                               | 新增真实文件/目录/不存在路径回归已通过                                                                                                  |
| R34      | 补查撤回保留判断；删除无产品消费者的前端PPTX链路，统一Rust documents引擎        | 见boundary-followup.md，补充整改验证单独记录                                                                                            |
| R35      | 条件保留当前加载边界                                                            | 未做打包/profile，不臆测收益；后续仅有实测成本时调整                                                                                    |
| R36      | package/Rust/Tauri版本与产品描述一致，权威文档路径同步                          | manifest与架构门检查                                                                                                                    |
| R37      | 授权、流、权限、引用回收、损坏行、幂等/回滚/竞争retry补关键路径                 | Web全量通过；Rust177个单元测试通过                                                                                                      |

## 行为与数据边界

- 数据风险：当前Spaces baseline不接受旧开发库；应用只返回显式错误，不自动删除/转换用户数据。手动重置前由用户备份。没有操作真实用户数据库。
- IPC协议：错误统一为code/message；批量remove_assets取代remove_asset；前后端同一工作区一起调整，不提供没有产品承诺的旧协议兼容。
- 行为变化：取消/异常EOF不能成为成功，错误不能被默认值吞掉，提交后回收错误不改变已提交结果。
- 纯整理：Rust入口移动、名称、目录、重复包装与未用直接依赖删除。
- 条件保留：R17既有session组合边界、R21 snapshot写入、R35未测加载优化。理由来自实际所有权/执行约束；不等同于新增待实现抽象。
- 平台缺口：未执行产品build、Vite/Tauri打包、Windows原生验证或桌面E2E。跨平台行为不得据此宣称通过。

## 实施中发现并修复的回执问题（R13/R37）

- `organization/confirmed-mutation.ts` 原先对任何失败查询旧回执，可能把相同mutationId但不同请求的拒绝转成成功。
- Rust `structure/receipts.rs` 给这类拒绝稳定的 `mutation_id_conflict`；revision拒绝统一为 `space_edit_revision_conflict`。
- Web保留所有返回的命令/分派失败（包括operation_failed、参数/权限拒绝）；仅无返回失败载荷的传输错误可确认回执。回归先证明旧实现误取旧回执，再验证领域拒绝、未分类命令拒绝、Tauri参数拒绝与传输恢复，4用例通过。

## 最终验证

| 检查                                                                       | 结果                                                                |
| -------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| `pnpm check:web`                                                           | 通过，含authority、Web/E2E类型、lint、架构、tests、Knip、重复与格式 |
| Web全量测试                                                                | 138文件、463用例通过                                                |
| Web依赖图                                                                  | 1126模块、3795边，无违规                                            |
| jscpd                                                                      | 总重复行0.46%，低于现有1%门槛；未机械抽象SQL/DTO重复                |
| Rust全量 `cargo test --offline --manifest-path src-tauri/Cargo.toml --lib` | 177通过、1显式合同生成器ignored；生成器另行运行1通过                |
| 合同生成                                                                   | 重新生成并Oxfmt，32文件零差异                                       |
| Rust格式/架构                                                              | 通过                                                                |
| Rust所有target Clippy `-D warnings`                                        | 通过，修正unused_io_amount与needless_question_mark后实际复验        |
| 物理同stem冲突                                                             | src、src-tauri/src、tools为0                                        |
| String错误命令                                                             | Rust AST扫描为0；内部能力诊断仍由owner保留，无全局异常继承体系      |
| 生产npm audit                                                              | 官方registry无已知漏洞                                              |
| `git diff --check`                                                         | 通过                                                                |
| 产品build/打包、Windows/桌面E2E                                            | 按用户约束未执行，不作为已验证结果                                  |

最后一次Rust全量之后的收尾仅为：HTTP测试正确读取完整请求头、领域取消调用直接使用原registry操作、删除空plugins默认配置。HTTP测试定向2通过和Clippy所有target通过覆盖这些收尾；不再重复未改动的模型推理路径。
