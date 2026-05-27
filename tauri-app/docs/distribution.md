# Tauri Desktop Distribution

## 当前发布锚点

GitHub `v1.1.1` 已 release，作为成熟 macOS 交付锚点。当前 Windows 发布锚点为 `v1.2.0`。

当前本地 checkout 已适配 Windows，默认 Tauri 构建目标为 NSIS `.exe` 安装包。macOS 包历史锚点仍是 ad-hoc signed、未 Developer ID 签名、未公证的 `.dmg`，适合小范围分发和手动允许启动的测试/交付场景；正式公开分发前仍需要评估 Apple Developer ID 签名、公证和自动更新。

## 源码与产物关系

Windows `.exe` / NSIS installer 与 macOS `.app` / `.dmg` 共用同一套源码：

```text
src/frontend/                Next.js 前端源码
src/backend/                 Rust Axum 后端源码
tauri-app/src-tauri/src/     Tauri 桌面壳源码
tauri-app/src-tauri/target/  构建产物目录，不是源码
```

安装包不是两套源码仓库。Tauri 桌面壳通过 Cargo path dependency 复用 `src/backend`，通过静态导出复用 `src/frontend`。平台差异主要是：

- Windows：NSIS installer，日志在 `%LOCALAPPDATA%\Pixiv Platform\desktop.log`，文件夹选择器走 WinForms。
- macOS：`.app` / `.dmg`，日志在 `~/Library/Logs/Pixiv Platform/desktop.log`，文件夹选择器走 AppleScript。

## 构建产物

当前 Windows 默认构建执行：

```powershell
cd tauri-app
npm.cmd install
npm.cmd run build
```

产物：

```text
tauri-app/src-tauri/target/release/pixiv_platform_tauri_app.exe
tauri-app/src-tauri/target/release/bundle/nsis/Pixiv Platform_1.2.0_x64-setup.exe
```

说明：当前 Windows Tauri 版本号已同步为 `1.2.0`，生成的 NSIS 文件名与 release tag 对齐。macOS 历史公开 release 锚点仍为 `v1.1.1`。

最新 Windows 本机构建锚点（2026-05-27）：`cd tauri-app && npm.cmd run build` 成功，NSIS 路径为 `tauri-app/src-tauri/target/release/bundle/nsis/Pixiv Platform_1.2.0_x64-setup.exe`。用户手动确认 Web 正常、Windows App 正常，Pixiv Refresh 弹窗正常。

macOS 历史构建锚点（2026-05-27）：`cd tauri-app && npm run build` 曾成功产出 `.app` 和 `.dmg`，`.dmg` 路径为 `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`，本地大小约 8.2M。`.app` 通过 `codesign --verify --deep --strict`，`.dmg` 通过 `hdiutil verify`。

注意：当前 `tauri.conf.json` 的默认 `beforeBuildCommand` 和 `bundle.targets` 已切到 Windows。macOS 源码分支仍保留，但要重新构建 `.app` / `.dmg`，需要在 macOS 上把前端导出命令切回 macOS shell 写法，并把 bundle targets 切回 `["app", "dmg"]`，或新增 macOS 专用 Tauri config。

## 签名边界

Tauri macOS bundle 显式使用：

```json
"macOS": {
  "signingIdentity": "-"
}
```

这会对 `.app` bundle 执行 ad-hoc signing，并生成完整 sealed resources，避免旧构建中 `.app` 缺少完整 `_CodeSignature/CodeResources` 导致的损坏提示风险。

这不等同于 Apple Developer ID 签名，也不等同于 notarization。

## 本机验证点

```text
codesign --verify --deep --strict --verbose=2 "tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app"
hdiutil verify "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg"
hdiutil attach -nobrowse -readonly "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg"
ls -la "/Volumes/Pixiv Platform"
codesign --verify --deep --strict --verbose=2 "/Volumes/Pixiv Platform/Pixiv Platform.app"
hdiutil detach "/Volumes/Pixiv Platform"
```

验证标准：

- `.app` codesign 校验通过。
- `.dmg` checksum 校验通过。
- 挂载卷中存在 `Pixiv Platform.app`。
- 挂载卷中存在 `Applications -> /Applications` 拖拽链接。
- 验证后卸载 `.dmg` 卷。

## GitHub Release 说明

Release notes 需要明确：

- 运行 `.dmg` 内已打包应用不需要安装 Rust、Cargo、Node、npm 或 TypeScript。
- 当前包未 Developer ID 签名、未公证，macOS Gatekeeper 可能要求用户在系统安全设置中手动允许。
- 不要把 Apple 账号、证书、notary profile、API key、secret 或本地私有路径写入仓库或文档。
