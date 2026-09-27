# 同名文件与目录整改

## 规则与计划

仓库统一使用：独立 Rust 模块使用 `<name>.rs`；模块拥有 `<name>/` 子目录时，入口放在 `<name>/mod.rs`，不同时保留父目录中的 `<name>.rs`。这是一致性选择，不宣称 Rust 允许的另一种写法本身错误。无子模块的文件不人为创建空目录。

扫描全部仓库自有源文件的 `.rs/.ts/.tsx/.vue/.js/.mjs` 同名目录组合，排除依赖、构建产物、运行状态与临时目录。发现三处，均属于 Rust 模块入口与测试子目录并存。先记录计划，再移动入口；不加入路径别名或转发文件，不改变模块逻辑路径、可见性或测试行为。只做静态检查，不编译、运行测试或打包。

## 全部发现与调整

| 原文件                                                    | 新入口                                                        | 已有子模块          |
| --------------------------------------------------------- | ------------------------------------------------------------- | ------------------- |
| `src-tauri/src/local_resources/tools/deno_archive.rs`     | `src-tauri/src/local_resources/tools/deno_archive/mod.rs`     | 原 tests 子模块保留 |
| `src-tauri/src/local_resources/tools/install_requests.rs` | `src-tauri/src/local_resources/tools/install_requests/mod.rs` | 原 tests 子模块保留 |
| `src-tauri/src/spaces/assets/transcription/registry.rs`   | `src-tauri/src/spaces/assets/transcription/registry/mod.rs`   | 原 tests 子模块保留 |

## 静态验证

扫描 2194 个仓库自有源文件，同名目录与源文件并存为零；核对 928 个 Rust 文件模块声明，均能定位入口。三个移动文件与 HEAD 原文件逐字节一致，原测试模块路径保持有效。rustfmt --check 通过。规范已同步更新。未编译、运行测试或打包。
