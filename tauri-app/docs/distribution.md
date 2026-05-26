# Tauri Desktop Distribution

## P3a 未签名 `.dmg` 小范围分发

当前用户没有 Apple Developer Program / Apple 开发者认证背景，因此 P3a 只做未签名、
未公证 `.dmg` 的最小闭环。

当前产物：

```text
tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg
```

同次 build 也会产出 `.app`：

```text
tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app
```

## GitHub Release 上传说明

1. 执行 `cd tauri-app && npm run build`。
2. 确认 `.dmg` 产物存在。
3. 在 GitHub Release 中上传 `.dmg` 文件作为 release asset。
4. Release notes 中明确写明该产物未签名、未公证，首次打开可能被 macOS Gatekeeper 拦截。
5. 告知测试用户：运行 `.dmg` 内已打包应用不需要安装 Rust、Cargo、Node、npm 或 TypeScript。

## 本机验证点

- `.dmg` 可以挂载。
- 挂载卷中存在 `Pixiv Platform.app`。
- 挂载卷中存在指向 `/Applications` 的拖拽链接。
- 验证后卸载 `.dmg` 卷，避免留下挂载状态。

## 未来正式分发

签名、公证和自动更新只作为未来正式分发路径评估。不要把 Apple 账号、证书、notary
profile、API key、secret 或本地私有路径写入仓库或文档。
