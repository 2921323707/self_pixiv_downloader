# 下一会话提示词

你在当前 Pixiv Platform 项目继续工作。请节省 token，不要全仓库浏览。

请先读取：

1. `README.md`
2. `docs/CONTEXT_HANDOFF.md`
3. `docs/DOCUMENT_MAP.md`
4. `docs/progress.md`
5. `tauri-app/docs/progress.md`
6. `tauri-app/docs/testing.md`
7. `tauri-app/docs/distribution.md`

当前锚点：

- GitHub `v1.1.1` 已 release，作为成熟 macOS 交付锚点；当前 Windows 桌面发布锚点为 `v1.2.0`。
- 后端仍是 downloader-first：Pixiv single / Author / Bookmarks / Smart 下载、SQLite 索引、任务队列、Gallery、Settings、Tasks、Home 均可用。
- 2026-05-27 已完成 API 层拆分：原 `src/backend/src/api.rs` 现在拆为 `src/backend/src/api/`，包含 `routes.rs`、`dto.rs`、`error.rs`、`runtime.rs`、`worker.rs`、`handlers/*` 和 `tests.rs`。外部入口 `pixiv_platform_backend::api::{AppState, router, serve, serve_listener}` 保持兼容。
- macOS 发布验证已完成：`./tests/unit/backend_unit.sh` 通过 86 个后端测试；`./tests/run_local.sh` 通过全部本地确定性门禁；`cd tauri-app && npm run build` 成功生成 `.app` 和 `.dmg`。`.app` 通过 `codesign --verify --deep --strict`，`.dmg` 通过 `hdiutil verify`。
- Windows v1.2.0 发布验证已完成：用户手动确认 Web 正常、Windows Tauri App 正常、Settings Pixiv Refresh 弹窗正常；`cd src/backend && cargo test` 通过 86 个后端测试；`cd src/frontend && npm.cmd run lint` 通过；`cd tauri-app && npm.cmd run build` 生成 `tauri-app/src-tauri/target/release/bundle/nsis/Pixiv Platform_1.2.0_x64-setup.exe`。
- Live Pixiv E2E 未运行，因为当前 shell 未设置 `PIXIV_PHPSESSID`。如要运行，使用 `PIXIV_PHPSESSID=... ./tests/e2e/live_single_download.sh`，不要把 cookie 写入仓库。
- Tauri 桌面壳复用 `src/frontend` 和 `src/backend`，不复制业务代码；桌面端使用随机端口、`GET /api/health`、运行时 API base URL 注入和启动失败窗口。Windows `.exe` / NSIS 与 macOS `.app` / `.dmg` 共用同一套源码，差异在构建配置和少量 `cfg(target_os)` 分支。
- 当前 `tauri.conf.json` 默认构建目标为 Windows NSIS；macOS 源码分支仍保留，但要重新构建 `.app` / `.dmg` 需要切回 macOS build command 和 `["app", "dmg"]` targets，或新增 macOS 专用配置。
- Web / 后端独立运行和桌面端默认共享 `~/Downloads/Pixiv Platform/`；旧 `output/` 不再自动迁移。
- 当前包仍未 Developer ID 签名、未公证；Gatekeeper 可能要求用户手动允许。

Debug 时优先关注：

- 后端 API 模块入口：`src/backend/src/api/mod.rs` 和 `src/backend/src/api/routes.rs`。
- API handlers：`src/backend/src/api/handlers/`。
- API runtime/worker：`src/backend/src/api/runtime.rs`、`src/backend/src/api/worker.rs`。
- 任务执行核心：`src/backend/src/tasks/mod.rs`，特别是 `execute_queued_task`。
- 启动日志：macOS `~/Library/Logs/Pixiv Platform/desktop.log`；Windows `%LOCALAPPDATA%\Pixiv Platform\desktop.log`。
- DMG/签名：`codesign --verify --deep --strict`、`hdiutil verify`、挂载卷内 app 与 Applications 链接。
- Pixiv secret：不要打印完整 `PHPSESSID`。

建议下一步：

1. 以 `v1.2.0` 为 Windows 桌面发布基线，优先处理缺陷修复、安装体验和小范围分发反馈。
2. 若继续做功能，优先 Gallery Thumbnail Cache，降低大图库滚动和 WebView 压力。
3. 若要长期双平台打包，优先拆分 Windows/macOS Tauri build config。
4. 正式公开分发前，再评估 Apple Developer ID 签名、公证和自动更新。
5. 文档更新采用“顶部锚点覆写 + 关键 debug 信息保留 + 阶段结束压缩”，避免继续堆流水。
