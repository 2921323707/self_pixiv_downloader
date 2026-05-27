# Tauri Desktop Testing Standard

## 当前目标

`v1.1.1` 已作为成熟交付第一版本 release。当前测试标准不再追踪每一次重复 build 流水，而是保留：

- 交付验收标准。
- 最小命令行质量门。
- 对后续 debug 有价值的失败根因和排查入口。
- live 测试的安全边界。

最新验证锚点：2026-05-27 在 API 模块拆分后重新运行 `./tests/unit/backend_unit.sh`、`./tests/run_local.sh`、`cd tauri-app && npm run build`、`codesign --verify --deep --strict`、`hdiutil verify` 均通过。Live Pixiv E2E 未运行，因为当前 shell 未设置 `PIXIV_PHPSESSID`。

## 分层验证

按成本从低到高验证：

1. 文档一致性：版本锚点、交付边界、分发说明、测试标准同步。
2. 前端静态检查：`cd src/frontend && npm run lint`。
3. 前端构建检查：`cd src/frontend && npm run build`。
4. 静态导出检查：`cd src/frontend && NEXT_OUTPUT_EXPORT=1 npm run build`。
5. 后端检查：`cd src/backend && cargo check`，必要时运行相关 `cargo test`。
6. Tauri 检查：`cd tauri-app/src-tauri && cargo check`。
7. Tauri release build：`cd tauri-app && npm run build`。
8. macOS bundle 验证：`codesign`、`hdiutil verify`、挂载结构检查。
9. 用户手动验证：打开 `.app` 或 `.dmg` 安装后确认核心页面与下载流程。

## 交付验收点

桌面端：

- `.app` 双击后无需手动启动 Next dev server 或 Rust server。
- Tauri 进程内 Axum 后端监听 `127.0.0.1:<random-port>`。
- 主窗口创建前 `GET /api/health` 返回 `200 OK`。
- Home / Download / Gallery / Tasks / Settings 可访问。
- 前端通过 Tauri 注入的运行时 API base URL 访问后端，不写死 `127.0.0.1:3000`。
- 退出 `.app` 后不残留独立后端进程。
- 后端启动或健康检查失败时，桌面窗口显示错误原因和日志路径。
- 日志写入 `~/Library/Logs/Pixiv Platform/desktop.log`。

数据与设置：

- Web / 后端独立运行和桌面端默认下载根目录均为 `~/Downloads/Pixiv Platform/`。
- 默认 SQLite 路径为 `~/Downloads/Pixiv Platform/pixiv_platform.sqlite3`。
- 旧项目 `output/` 不再自动恢复；如需旧数据，应通过备份或显式路径配置处理。
- Settings 下载目录在 Tauri 桌面端通过系统文件夹选择器配置。
- Settings 保存 secret 后仍显示 masked secret，不向 UI 或日志暴露完整值。

Pixiv 登录态刷新：

- Settings 中 Pixiv Login/Refresh 可打开官方 Pixiv 登录窗口。
- 用户登录后自动获取 Pixiv 域下的 `PHPSESSID` 并保存为 `pixiv_cookie`。
- 成功后自动执行 Pixiv connection test，关闭登录窗口，并显示不含 secret 的成功提示。
- Web 端不自动读取 Pixiv cookie；网页环境继续由用户手动填写 `pixiv_cookie` 并使用 Test 验证。

macOS 分发：

- `.dmg` 可产出并通过 `hdiutil verify`。
- `.dmg` 可挂载，卷内存在 `Pixiv Platform.app` 和 `Applications -> /Applications`。
- `.app` 通过 `codesign --verify --deep --strict`。
- 当前包是 ad-hoc signed、未 Developer ID 签名、未公证；Gatekeeper 仍可能要求用户手动允许。

## 当前验证命令

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

挂载验证：

```text
hdiutil attach -nobrowse -readonly "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg"
ls -la "/Volumes/Pixiv Platform"
codesign --verify --deep --strict --verbose=2 "/Volumes/Pixiv Platform/Pixiv Platform.app"
hdiutil detach "/Volumes/Pixiv Platform"
```

完整本地质量门：

```text
./tests/run_local.sh
```

Live E2E 仍为 opt-in：

```text
PIXIV_PHPSESSID=... ./tests/e2e/live_single_download.sh
```

## 已知 Debug 锚点

- `.next/lock` 竞争：不要并行运行独立 Next build 和 Tauri build；Tauri build 会触发前端静态导出。
- Tauri 自定义 command 权限：新增 command 后需要对应 permission manifest，否则 build 可能失败。
- 旧数据排查：如果用户反馈图库或 cookie 消失，优先检查共享 SQLite 路径和用户是否保留了旧数据备份。
- DMG 挂载：沙箱内 `hdiutil attach` 可能失败，需要在授权环境下挂载验证。
- 旧包损坏提示：优先跑 `codesign --verify --deep --strict`。曾见旧 `.app` 缺少完整 sealed resources；当前通过 `bundle.macOS.signingIdentity = "-"` 修复。
- Pixiv Refresh：只记录 cookie 是否存在、长度和非敏感元信息；不要打印完整 `PHPSESSID`。

## Live 测试规则

- Pixiv cookie 和 DeepSeek key 只允许运行时输入。
- live Pixiv / live LLM 测试保持 opt-in。
- 不在文档、代码、fixture、终端输出中写入完整 secret。
- Pixiv 登录态刷新 live 验证需要用户在 Tauri Pixiv 登录窗口手动登录。
- 自动化测试只验证 WebView cookie store 读取能力和非敏感日志输出。

## 失败处理

- 依赖下载失败时，先判断是否是网络或沙箱限制。
- 需要联网安装依赖时，必须请求用户批准。
- Tauri 启动失败时，先检查 `~/Library/Logs/Pixiv Platform/desktop.log`、`GET /api/health`、本地端口、SQLite 路径和 codesign 状态。
