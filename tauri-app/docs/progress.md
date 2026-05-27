# Tauri Desktop Progress

## 当前锚点

日期：2026-05-27

公开发布锚点：GitHub `v1.1.1` 已 release。用户已确认当前 `v1.1.1` 可作为成熟的交付第一版本。

当前桌面端状态：

- Tauri macOS 桌面壳已完成成熟交付闭环，复用现有 `src/frontend` 与 `src/backend`，不复制业务代码。
- 桌面端启动时在进程内启动 Axum 后端，使用随机本地端口，并向静态前端注入运行时 API base URL。
- 主窗口创建前会轮询 `GET /api/health`；后端启动失败或健康检查失败时显示启动失败窗口。
- 桌面启动日志写入 `~/Library/Logs/Pixiv Platform/desktop.log`。
- Web / 后端独立运行和桌面端默认共享 `~/Downloads/Pixiv Platform/`；旧项目 `output/` 不再自动恢复。
- Settings 支持桌面端文件夹选择器配置下载目录。
- Gallery 删除、Gallery 预览稳定性、基础菜单、正式图标、启动诊断均已完成并通过手动或命令行验证。
- Pixiv 内置登录窗口刷新 `PHPSESSID` 已完成，用户 live 验证成功；网页端继续保留手动填写 `pixiv_cookie` fallback。
- macOS bundle 显式启用 ad-hoc signing：`bundle.macOS.signingIdentity = "-"`。
- `.app` 已通过 `codesign --verify --deep --strict`；`.dmg` 已通过 `hdiutil verify` 和挂载结构检查。
- 2026-05-27 API 模块拆分后重新构建发布产物：`cd tauri-app && npm run build` 成功，产出 `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`。

## 成熟交付边界

`v1.1.1` 的交付边界是：稳定的本地 Pixiv 下载、索引、任务、图库、设置、智能检索工作台，加上可小范围分发的 macOS Tauri 桌面端。

已完成能力：

- Single / Author / Bookmarks / Smart 下载入口。
- SQLite 图片、来源、标签、任务、任务项、任务日志持久化。
- DB-aware dedupe、缺失文件修复和共享默认存储路径。
- Gallery 真实本地文件预览、详情、批量删除。
- Tasks 任务列表、详情、进度、日志。
- Settings Pixiv / DeepSeek / Storage / Appearance 配置与测试。
- DeepSeek 智能解析、Pixiv tag search、smart batch task。
- macOS `.app` / `.dmg` 打包。
- 桌面端 Pixiv 登录窗口刷新 `PHPSESSID`。

仍属于后续演进，不阻塞 `v1.1.1` 成熟交付：

- Gallery Thumbnail Cache / 大图库性能优化。
- Top10 / Random discovery modes。
- 任务 cancel / retry。
- 更完整的图片编辑、地图视图、语义检索和相似图聚类。
- Apple Developer ID 签名、公证、自动更新。

## 调试保留信息

以下信息对后续 debug 有实际价值，文档中继续保留：

- 日志路径：`~/Library/Logs/Pixiv Platform/desktop.log`。
- 健康检查：`GET /api/health`，主窗口创建前必须健康。
- 旧项目 `output/` 已退出默认路径，不再作为自动恢复来源。
- 旧 `.dmg` 损坏提示复查根因：旧 `.app` bundle 只有 linker ad-hoc 签名，缺少完整 sealed resources；`codesign --verify --deep --strict` 曾报 `code has no resources but signature indicates they must be present`。
- 修复方式：Tauri 配置显式使用 `bundle.macOS.signingIdentity = "-"`，重新 build 后 bundle 内存在 `_CodeSignature/CodeResources`。
- DMG 验证点：`hdiutil verify` 通过；挂载卷内存在 `Pixiv Platform.app` 和 `Applications -> /Applications`。
- Pixiv cookie 安全边界：不采集 Pixiv 账号密码；不在日志、文档、测试输出中打印完整 `PHPSESSID`；只允许记录存在性、长度和非敏感元信息。
- 分发边界：当前是 ad-hoc signed、未 Developer ID 签名、未公证；Gatekeeper 仍可能要求用户手动允许。

## 当前验证基线

最新验证结果（2026-05-27）：

```text
./tests/unit/backend_unit.sh
86 backend tests passed

./tests/run_local.sh
backend unit/stage/integration/smoke checks passed; frontend typecheck/build passed; 0 failed

cd tauri-app && npm run build
Finished 2 bundles:
tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app
tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg

codesign --verify --deep --strict --verbose=2 "tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app"
valid on disk; satisfies its Designated Requirement

hdiutil verify "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg"
checksum is VALID
```

Live Pixiv E2E was not run in this shell because `PIXIV_PHPSESSID` was not set.

建议交付前或关键改动后运行：

```text
cd src/backend && cargo check
cd src/backend && cargo test api_health_returns_ok
cd src/frontend && npm run lint
cd src/frontend && npm run build
cd src/frontend && NEXT_OUTPUT_EXPORT=1 npm run build
cd tauri-app/src-tauri && cargo check
cd tauri-app && npm run build
codesign --verify --deep --strict --verbose=2 "tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app"
hdiutil verify "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg"
```

DMG 挂载验证：

```text
hdiutil attach -nobrowse -readonly "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg"
ls -la "/Volumes/Pixiv Platform"
codesign --verify --deep --strict --verbose=2 "/Volumes/Pixiv Platform/Pixiv Platform.app"
hdiutil detach "/Volumes/Pixiv Platform"
```

## 下一步建议

1. 以 `v1.1.1` 作为成熟交付第一版维护基线，优先接受缺陷修复、文档、安装体验和小范围分发反馈。
2. 如继续做功能，优先考虑 Gallery Thumbnail Cache，降低大图库滚动时的 WebView 压力。
3. 正式公开分发前再评估 Apple Developer ID 签名、公证和自动更新。
4. 定期压缩 `progress.md`，避免把重复 build 输出和临时流水长期堆进文档。
