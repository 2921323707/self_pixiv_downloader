# Tauri Desktop Distribution

## 当前发布锚点

GitHub `v1.1.1` 已 release。用户确认当前 `v1.1.1` 可作为成熟的交付第一版本。

当前 macOS 包是 ad-hoc signed、未 Developer ID 签名、未公证。它适合小范围分发和手动允许启动的测试/交付场景；正式公开分发前仍需要评估 Apple Developer ID 签名、公证和自动更新。

## 构建产物

执行：

```text
cd tauri-app
npm install
npm run build
```

产物：

```text
tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app
tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg
```

说明：当前本地 Tauri 配置和产物文件名仍可能保留 `1.1.0` 字样；GitHub 当前公开 release 锚点为 `v1.1.1`。后续如需要统一包体文件名，再同步 Tauri 版本号与 release asset 命名。

最新本机构建锚点（2026-05-27）：`cd tauri-app && npm run build` 成功，`.dmg` 路径为 `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`，本地大小约 8.2M。`.app` 通过 `codesign --verify --deep --strict`，`.dmg` 通过 `hdiutil verify`。

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
