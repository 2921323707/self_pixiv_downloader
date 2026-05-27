# 文档地图

这是项目文档的总索引。以后上下文重置、阶段切换、或者你想知道某个决策在哪里，都先从这里进入。

## 第一入口

| 你想了解 | 推荐阅读 |
| --- | --- |
| 当前项目进度 | `docs/progress.md` |
| 如何快速恢复上下文 | `docs/CONTEXT_HANDOFF.md` |
| 桌面端交付状态（v1.2.0 Windows 当前锚点 + macOS 历史锚点） | `tauri-app/docs/progress.md` |
| 桌面端测试标准 | `tauri-app/docs/testing.md` |
| 桌面安装包分发说明（Windows `.exe` / macOS `.dmg`） | `tauri-app/docs/distribution.md` |
| 原始 PRD | `docs/product_requirements.md` |
| spec-coding 文档索引 | `docs/specs/README.md` |
| 需求到代码/测试的追踪 | `docs/specs/traceability.md` |
| 实现顺序 | `docs/specs/implementation-plan.md` |
| 测试命令与策略 | `tests/README.md`, `docs/specs/testing-strategy.md` |
| 清理与维护策略 | `docs/maintenance.md` |

## 核心产品规格

| 文档 | 作用 |
| --- | --- |
| `docs/specs/requirements.md` | 需求编号与验收标准 |
| `docs/specs/domain-model.md` | 领域实体、枚举、状态机 |
| `docs/specs/database-schema.md` | SQLite 表、索引、默认值、迁移策略 |
| `docs/specs/api-contract.md` | 后续 Axum API 契约 |
| `docs/specs/task-flow.md` | 异步任务生命周期、日志、轮询、失败模型 |

## Downloader-First 规格

| 文档 | 作用 |
| --- | --- |
| `docs/specs/architecture.md` | 前后端模块边界与依赖方向 |
| `docs/specs/pixiv-client.md` | Pixiv 认证、metadata、图片下载、限速、live smoke 规则 |
| `docs/specs/file-storage.md` | 本地文件布局与安全写入协议 |
| `docs/specs/error-catalog.md` | 日志、API、测试、任务共用的稳定错误码 |
| `docs/specs/phase5-batch-download-plan.md` | Phase 5/6 批量下载拆分；5A Author、5B Bookmarks、6A/6B Smart Retrieval 已完成 |

## 前端与视觉规格

| 文档 | 作用 |
| --- | --- |
| `docs/specs/frontend-spec.md` | 页面/路由计划与前端状态模型 |
| `docs/specs/visual-theme.md` | 主题规则与已选中的 `cyan-studio` 方向 |
| `docs/design/theme-reference/demo_B.html` | 当前视觉参考页面 |
| `docs/design/theme-reference/sakura-light.html` | 备选主题视觉参考页面 |

## 当前实现地图

| 模块 | 路径 |
| --- | --- |
| 后端 crate | `src/backend` |
| Axum API wrapper | `src/backend/src/api/` |
| API route registration | `src/backend/src/api/routes.rs` |
| API DTO / response mapping | `src/backend/src/api/dto.rs` |
| API error envelope mapping | `src/backend/src/api/error.rs` |
| API runtime settings / path resolution | `src/backend/src/api/runtime.rs` |
| API worker queue glue | `src/backend/src/api/worker.rs` |
| API handlers | `src/backend/src/api/handlers/` |
| SQLite 初始迁移 | `src/backend/migrations/0001_init.sql` |
| DB migration runner | `src/backend/src/db/mod.rs` |
| 图片仓储 | `src/backend/src/images/mod.rs` |
| 配置仓储 | `src/backend/src/settings/mod.rs` |
| Pixiv client | `src/backend/src/pixiv/mod.rs` |
| 下载编排 / DB-aware downloader | `src/backend/src/downloads/mod.rs` |
| 文件存储规划 | `src/backend/src/storage/mod.rs` |
| 任务仓储 / 任务包装 | `src/backend/src/tasks/mod.rs` |
| 真实单作品下载入口 | `src/backend/src/bin/live_single.rs` |
| 后端 server 入口 | `src/backend/src/bin/server.rs` |
| 前端 Next.js scaffold | `src/frontend` |
| Tauri 桌面壳源码 | `tauri-app/src-tauri` |
| Windows 本地 Web 启动脚本 | `tools/dev_backend_windows.ps1`, `tools/dev_frontend_windows.ps1` |
| Windows Tauri 前端导出脚本 | `tauri-app/scripts/build-frontend-windows.cmd` |

## 测试地图

| 命令 | 作用 |
| --- | --- |
| `./tests/run_local.sh` | 默认本地质量门 |
| `./tests/unit/backend_unit.sh` | 后端单测 |
| `./tests/stage/phase2a_repository.sh` | Phase 2A 迁移/仓储专项测试 |
| `./tests/stage/phase2c_tasks.sh` | Phase 2C 任务仓储/生命周期专项测试 |
| `./tests/stage/phase3b_queue.sh` | Phase 3B 后台队列专项测试 |
| `./tests/stage/phase4b_data_api.sh` | Phase 4B gallery/settings/task-list 数据 API 专项测试 |
| `./tests/stage/phase4c_configured_download.sh` | Phase 4C 前端配置单作品下载专项测试 |
| `./tests/stage/phase4d_gallery_file_api.sh` | Phase 4D Gallery 文件预览专项测试 |
| `./tests/stage/phase5a_author_batch.sh` | Phase 5A Author 批量下载专项测试 |
| `./tests/stage/phase5b_bookmark_batch.sh` | Phase 5B Bookmarks 批量下载专项测试 |
| `./tests/stage/phase6a_smart_parse.sh` | Phase 6A Smart Retrieval 解析专项测试 |
| `./tests/stage/phase6b_smart_download.sh` | Phase 6B Smart Retrieval 下载专项测试 |
| `./tests/stage/frontend_scaffold.sh` | 前端 scaffold 类型检查与构建 |
| `./tests/integration/backend_sqlite.sh` | 后端 SQLite 确定性集成测试 |
| `./tests/smoke/backend_api.sh` | Axum API wrapper 冒烟测试 |
| `PIXIV_PHPSESSID=... ./tests/e2e/live_single_download.sh` | 真实 Pixiv 单作品 E2E，手动 opt-in |

## 决策快照

| 决策 | 当前方向 |
| --- | --- |
| 实现优先级 | downloader-first |
| 默认主题 | Demo B / `cyan-studio` |
| 文件布局 | `{download_root}/originals/{pixiv_id}/{pixiv_id}_p{page}.{ext}` |
| Secret 处理 | 本地存储，对外 public 读取必须 mask |
| Live 测试 | 手动 opt-in，凭证运行时提供 |
| 当前阶段 | v1.2.0 Windows Desktop Release：Windows NSIS 是当前默认构建目标，macOS 源码分支保留 |
| 当前手动验证 | 前端单作品、Author Batch、Bookmarks Batch、Smart Retrieval、Home Dashboard、macOS Tauri、Pixiv Login/Refresh、Windows Web、Windows Tauri App 均已手动确认 |
| 桌面源码关系 | Windows `.exe` / NSIS 和 macOS `.app` / `.dmg` 共用 `src/frontend`、`src/backend`、`tauri-app/src-tauri`，安装包是平台构建产物 |
| 后续策略 | 以 v1.2.0 作为 Windows 桌面发布锚点；Top10/Random、缩略图缓存、任务 cancel/retry、双平台 build config 拆分、正式签名/公证等归入后续演进 |
