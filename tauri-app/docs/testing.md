# Tauri Desktop Testing Standard

## 分层验证

MVP 阶段按成本从低到高验证：

1. 文档完整性检查：需求、架构、计划、进度、清单同步。
2. 前端静态检查：`npm run lint`。
3. 前端构建检查：`npm run build`。
4. 后端 Rust 检查：优先运行相关 `cargo test` 或 `cargo check`。
5. Tauri 配置检查：`npm run tauri -- info` 或等价命令。
6. Tauri dev 手动验证：窗口打开，页面加载，API 可访问。

## MVP 验收页

桌面窗口中至少确认以下页面能打开：

- Home
- Download
- Gallery
- Tasks
- Settings

当前自动/命令行验证已确认 `GET /` 返回 `200 OK`。页面级人工巡检仍建议由用户
在 Tauri 窗口中点开五个页面复核。

2026-05-26：用户手动测试 P0 收尾版本 `.app`，确认通过。

2026-05-26：主页面左上角 logo 替换为图标后，`cd src/frontend && npm run lint`
通过。

2026-05-26：因旧 `.app` 仍显示默认 logo，重新执行 `cd tauri-app && npm run build`；
新的静态导出主页已包含 `/app-icon.png`。

2026-05-26：Gallery 删除问题排查：
`cd src/backend && cargo test req_img_007` 通过，确认后端删除 API 正常；
前端删除确认改为应用内 modal 后，`cd src/frontend && npm run lint`、
`cd src/frontend && NEXT_OUTPUT_EXPORT=1 npm run build`、`cd tauri-app && npm run build`
均通过。静态产物已包含 `delete-confirm-modal`。

2026-05-26：用户手动打开最新 `.app` 直接运行并测试，确认没有问题。

2026-05-26：P2 启动诊断补强：
Tauri 启动流程新增 `~/Library/Logs/Pixiv Platform/desktop.log` 追加日志；
后端启动或健康检查失败时会创建启动失败窗口，显示错误原因和日志路径；
`cd tauri-app/src-tauri && cargo check`、`cd src/frontend && npm run lint`、
`cd src/frontend && NEXT_OUTPUT_EXPORT=1 npm run build`、`cd tauri-app && npm run build`
均通过。曾并行触发一次 Tauri build 与独立 Next build，因 `.next/lock` 竞争失败；
随后单独重跑 `cd tauri-app && npm run build` 通过。

2026-05-26：热缓存测速：
`cd tauri-app/src-tauri && /usr/bin/time -p cargo check` real 0.78s；
`cd src/frontend && /usr/bin/time -p npm run lint` real 1.07s；
`cd src/frontend && /usr/bin/time -p env NEXT_OUTPUT_EXPORT=1 npm run build` real 4.92s；
`cd tauri-app && /usr/bin/time -p npm run build` real 26.58s。

2026-05-26：P1 桌面默认下载目录迁移：
Tauri 桌面端未设置 `PIXIV_DOWNLOAD_ROOT` 时默认使用
`~/Downloads/Pixiv Platform/`；未设置 `PIXIV_PLATFORM_DB_PATH` 时默认使用
`~/Downloads/Pixiv Platform/pixiv_platform.sqlite3`；Settings 页面下载目录改为 Tauri
文件夹选择器选择并自动保存。验证通过：
`cd tauri-app/src-tauri && cargo check`、`cd src/frontend && npm run lint`、
`cd src/frontend && env NEXT_OUTPUT_EXPORT=1 npm run build`、`cd tauri-app && npm run build`。
曾因自定义 command 缺少 Tauri app permission manifest 导致一次 build 失败；补充
`permissions/select-download-directory.toml` 后重跑通过。

2026-05-26：P1 旧库自动恢复修复：
用户反馈打开迁移后的 `.app` 后旧图库和旧配置消失，并提示
`Pixiv cookie is required in settings or PIXIV_PHPSESSID`。确认旧
`output/pixiv_platform.sqlite3` 仍包含 `pixiv_cookie` 与 22 条图库记录，而新桌面库为空图库。
修复为：新桌面 SQLite 不存在，或新桌面库没有 `pixiv_cookie` 且没有图库记录时，若旧
`output/pixiv_platform.sqlite3` 有用户数据，则备份新空库、复制旧库、复制旧下载文件到
`~/Downloads/Pixiv Platform/`，并把图片绝对路径改写到新目录。验证通过：
`cd tauri-app/src-tauri && cargo check`、`cd tauri-app && npm run build`。

2026-05-26：用户手动测试最新 `.app`，确认旧库自动恢复后的当前阶段功能正常。

2026-05-26：P2 基础菜单：
新增 Tauri 原生菜单 About / Quit / Reload，Developer Tools 仅 debug 构建显示，并保留
File / Edit / Window 常见菜单项。验证通过：
`cd tauri-app/src-tauri && cargo check`、`cd tauri-app/src-tauri && cargo check --release`、
`cd tauri-app && npm run build`。新的 `.app` 产物位于
`tauri-app/src-tauri/target/release/bundle/macos/Pixiv Platform.app`。用户手动测试最新
`.app`，确认 P2 基础菜单后当前阶段功能正常。

2026-05-26：P3a 未签名 `.dmg` 最小闭环：
Tauri bundle targets 扩展为 `app` 和 `dmg`，验证通过：
`cd tauri-app/src-tauri && cargo check`、`cd tauri-app && npm run build`。首次在沙箱内执行
`npm run build` 时，`.app` 已产出，但 `bundle_dmg.sh` 无法完成 macOS 磁盘映像挂载/打包；
随后按权限规则在沙箱外重跑同一命令通过。新的 `.dmg` 产物位于
`tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`，大小约
6.1M。本机挂载验证通过：`.dmg` CRC 校验通过，挂载卷中存在 `Pixiv Platform.app` 和
`Applications -> /Applications` 拖拽链接；验证后已卸载。

2026-05-26：残留清理：
已删除全部 `.DS_Store`，以及可再生成的 `src/frontend/.next`、`src/frontend/out`、
`src/frontend/tsconfig.tsbuildinfo`、`tauri-app/src-tauri/target`、`tauri-app/node_modules`。
清理后 `find . -name .DS_Store -print` 无输出，相关构建目录已不存在。由于
`tauri-app/src-tauri/target` 已清理，本地 `.app` / `.dmg` 产物也已移除；发布或人工分发复核前
需重新执行 `cd tauri-app && npm run build`。

2026-05-26：`v1.1.0` 提交前最终验证：
推荐下一版本号为 `v1.1.0`，并同步 Tauri 桌面壳版本到 `1.1.0`。验证通过：
`cd tauri-app/src-tauri && cargo check`、`cd tauri-app && npm install`、
`cd tauri-app && npm run build`。新的 `.dmg` 产物位于
`tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`，大小约
6.1M。最终复核 `find . -name .DS_Store -print` 无输出；重新生成的
`src/frontend/.next`、`src/frontend/out`、`tauri-app/node_modules`、`tauri-app/src-tauri/target`
均为 `.gitignore` 覆盖的本地构建/依赖产物。

## `.app` 打包验收

首轮 `.app` MVP 打包验收按最小闭环执行：

1. `cd src/frontend && npm run build` 可成功生成生产产物。
2. `cd tauri-app && npm run build` 可成功生成 macOS `.app`。
3. 双击 `.app` 后无需手动启动 Next dev server。
4. Tauri 进程内 Axum 后端监听 `127.0.0.1:<random-port>`。
5. Tauri 创建主窗口前 `GET /api/health` 返回 `200 OK`。
6. Home / Download / Gallery / Tasks / Settings 可访问。
7. `GET /api/settings`、`GET /api/tasks`、`GET /api/images` 可用或返回清晰错误。
8. 退出 `.app` 后不残留独立后端进程。
9. `127.0.0.1:3000` 被旧实例占用时，新 `.app` 仍可通过随机端口启动。
10. 静态 `.app` 直连本地 API 时，后端返回 `Access-Control-Allow-Origin`。
11. 静态导出产物不写死 `127.0.0.1:3000`，前端通过 Tauri 注入的运行时 API base URL 访问后端。
12. 后端启动或健康检查失败时，桌面窗口显示错误原因和日志文件路径。
13. 桌面端未设置 `PIXIV_DOWNLOAD_ROOT` 时，默认下载根目录为
    `~/Downloads/Pixiv Platform/`。
14. 桌面端未设置 `PIXIV_PLATFORM_DB_PATH` 时，默认 SQLite 路径为
    `~/Downloads/Pixiv Platform/pixiv_platform.sqlite3`。
15. Settings 页面下载目录配置在 Tauri 桌面端通过系统文件夹选择器选择。
16. 从旧项目 `output/` 升级到桌面默认目录时，旧 SQLite 中的 Pixiv cookie、图库记录和
    已下载图片路径应自动迁移，避免出现空图库或缺失 cookie。
17. 未签名、未公证 `.dmg` 可产出，并可在本机挂载后看到 `.app` 与 Applications 拖拽链接。

## API 验收点

- `GET /api/settings` 可返回数据或清晰错误。
- `GET /api/health` 可返回 `{"data":{"status":"ok"}}`。
- `GET /api/tasks` 可返回任务列表。
- `GET /api/images` 可返回图库列表。
- Settings 页面保存配置不能暴露 secret。
- 带 Tauri WebView origin 的 CORS 请求可访问本地 API。

## Live 测试规则

- Pixiv cookie 和 DeepSeek key 只允许用户运行时输入。
- live Pixiv / live LLM 测试保持 opt-in。
- 不在文档、代码、fixture、终端输出中写入完整 secret。

## 失败处理

- 依赖下载失败时，先判断是否是网络或沙箱限制。
- 需要联网安装依赖时，必须请求用户批准。
- Tauri dev 未能启动时，先记录失败命令、错误摘要和下一步建议到进度文档。

## 当前 MVP 验证命令

```text
cd src/backend && cargo check
cd src/backend && cargo test api_health_returns_ok
cd src/frontend && npm run lint
cd src/frontend && npm run build
cd src/frontend && NEXT_OUTPUT_EXPORT=1 npm run build
cd tauri-app/src-tauri && cargo check
cd tauri-app && npm run build
cd tauri-app && npm run dev
hdiutil attach -nobrowse -readonly "tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg"
hdiutil detach "/Volumes/Pixiv Platform"
```

测速命令：

```text
cd tauri-app/src-tauri && /usr/bin/time -p cargo check
cd src/frontend && /usr/bin/time -p npm run lint
cd src/frontend && /usr/bin/time -p env NEXT_OUTPUT_EXPORT=1 npm run build
cd tauri-app && /usr/bin/time -p npm run build
```
