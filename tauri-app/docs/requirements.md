# Tauri Desktop Requirements

## 当前阶段

阶段：`v1.1.1` 成熟交付第一版已 release；Desktop MVP、P3a `.dmg` 最小分发闭环、Pixiv 登录态刷新和 macOS ad-hoc signing 均已完成。

目标是在 macOS 上跑通一个可开发验证的 Tauri 桌面壳。该壳引用现有
`src/frontend` 前端与 `src/backend` Rust 后端，不复制业务代码。

## 范围

- 新建并维护 `tauri-app` 作为桌面应用项目源码目录。
- Tauri 开发模式加载现有 Next.js 前端。
- Tauri 生产打包优先使用 Next 静态导出产物，由 `.app` 内嵌加载。
- Tauri 后端通过 Rust path dependency 引用现有 `src/backend`。
- 桌面端每次启动绑定 `127.0.0.1` 随机可用端口，避免固定端口冲突。
- Tauri 创建 WebView 时注入运行时 API base URL，前端优先使用该值访问本地 Rust API。
- 前端 API 层保留环境变量切换后端 base URL，作为 Web 开发兼容路径。
- Web / 后端独立运行和桌面端默认共享 `~/Downloads/Pixiv Platform/`。
- 桌面端默认 SQLite 放在默认下载目录下的 `pixiv_platform.sqlite3`，除非运行时显式配置
  `PIXIV_PLATFORM_DB_PATH`。
- 旧项目 `output/` 不再作为默认下载目录，也不再自动迁移。
- 平台 Settings 中下载目录配置应通过系统文件夹选择器选择，不要求用户手动输入目录路径。
- 在用户没有 Apple Developer Program / Apple 开发者认证背景的前提下，先产出 ad-hoc signed、
  未公证的 macOS `.dmg`，用于 GitHub Release 小范围分发验证。
- 分发文档需说明：未 Developer ID 签名、未公证 `.dmg` 上传 GitHub Release 后，其他 macOS 用户可能遇到
  Gatekeeper 拦截；但运行已打包应用不需要安装 Rust、Cargo、Node、npm 或 TypeScript。
- Settings 中 Pixiv 连接应支持桌面端内置 Pixiv 登录窗口刷新 `PHPSESSID`，降低用户手动从浏览器
  开发者工具复制 cookie 的成本。
- Pixiv 登录态刷新必须在 Pixiv 官方页面完成登录，不采集、不保存、不代理用户 Pixiv 密码。
- 自动获取到的 `PHPSESSID` 继续写入现有 `pixiv_cookie` setting，沿用后端 secret masking、
  runtime settings 和 `POST /api/settings/test/pixiv` 验证流程。
- Pixiv 登录态刷新成功后应自动关闭登录窗口，并在 Settings 主窗口显示不含 secret 的成功提示。

## 非目标

- 不做 Windows 桌面端。
- 不做手机端。
- 不做 Apple Developer ID 签名、Apple notarization、公证或自动更新；允许 Tauri bundle 使用 ad-hoc signing 修复 bundle 完整性。
- 不迁移到 macOS Application Support 数据目录；用户已确认共享默认目录使用
  `~/Downloads/Pixiv Platform/`。
- 不复制前端或后端源码到 `tauri-app`。
- 不重写现有 HTTP API 为 Tauri commands。
- 不实现后端模拟 Pixiv 账号密码登录。
- 不在 Web 端用 bookmarklet 或页面脚本读取 Pixiv cookie；桌面端走 Tauri WebView cookie store。

## 约束

- 遵循 spec-coding：先更新文档，再实现，再验证，再同步进度。
- 用户未确认前，不做破坏性清理。
- 不提交 secret、cookie、API key 或本地私有路径。
- 不在日志、文档、测试输出中打印完整 `PHPSESSID`；只允许输出是否存在、长度和非敏感元信息。
- 现有 Web 开发体验应保持可用。
- Git 操作默认只改文件不提交，除非用户明确要求。

## 待办优化

- 后续如需更正式的数据分层，再评估是否把 SQLite 迁移到
  `~/Library/Application Support/Pixiv Platform/`。
- 为随机端口启动继续补充更完整的崩溃诊断。
- 后续如需公开正式分发，再评估 Apple 账号、证书、签名、公证和自动更新流程。
- Pixiv 登录态刷新已完成正式实现与用户 live 验证；后续只做体验微调或错误提示增强。
