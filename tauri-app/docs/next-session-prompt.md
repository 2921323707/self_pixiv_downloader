# 新会话首轮提示词

请在 `/Users/Admin/Downloads/pixiv_platform` 继续 Tauri 桌面化工作。

请节省 token，采用渐进式披露检索，不要全局浏览仓库。先只读：

1. `tauri-app/docs/progress.md`
2. `tauri-app/docs/requirements.md`
3. `tauri-app/docs/architecture.md`
4. `tauri-app/docs/implementation-plan.md`
5. `tauri-app/docs/checklist.md`
6. `tauri-app/docs/testing.md`
7. `tauri-app/docs/next-session-prompt.md`

当前锚点：

- `tauri-app` 是桌面壳，不复制 `src/frontend` 或 `src/backend`。
- 前端继续来自 `src/frontend`。
- 后端通过 Cargo path dependency 引用 `src/backend`。
- Desktop MVP dev 模式已跑通。
- `.app` 打包 MVP 已跑通。
- 最新 `.app` 可直接打开运行，用户手动测试通过。
- 生产前端采用 Next 静态导出。
- Tauri 启动时在后台 Tokio runtime 中启动现有 Axum 后端。
- 后端每次启动绑定 `127.0.0.1:0` 随机端口。
- 后端已有 `GET /api/health`。
- Tauri 启动后先轮询 `/api/health`，健康后再创建主窗口。
- Tauri 程序化创建主窗口，并注入 `window.__PIXIV_PLATFORM_BACKEND_URL__`。
- 前端 `apiUrl()` 优先读取运行时注入的 API base URL，再回退 `NEXT_PUBLIC_PIXIV_BACKEND_URL`，最后回退 `/api`。
- 静态导出产物不再写死 `127.0.0.1:3000`。
- 后端已加 CORS layer，支持 Tauri 静态页面直连本地 API。
- App 图标和主页面左上角 logo 已替换为用户提供图片。
- Gallery 删除交互已修复：不再依赖 `window.confirm()`，改为应用内确认弹窗后调用 `POST /api/images/delete-batch`。
- Tauri 启动日志追加写入 `~/Library/Logs/Pixiv Platform/desktop.log`。
- 后端启动失败或健康检查超时时，Tauri 创建启动失败窗口，显示错误原因和日志文件路径。
- Tauri 桌面端默认下载目录已迁移到 `~/Downloads/Pixiv Platform/`。
- Tauri 桌面端默认 SQLite 路径已迁移到
  `~/Downloads/Pixiv Platform/pixiv_platform.sqlite3`。
- P1 迁移回归已修复：如果新桌面库不存在，或新库为空且旧
  `output/pixiv_platform.sqlite3` 仍有用户配置/图库，Tauri 会自动恢复旧库、复制旧下载文件，
  并改写图片路径到 `~/Downloads/Pixiv Platform/`。
- P2 基础菜单已完成：About / Quit / Reload；Developer Tools 仅 debug 构建显示；
  File / Edit / Window 保留常见系统菜单项。
- 用户手动测试 P2 基础菜单后的最新 `.app`，确认当前阶段功能正常。
- P3a 未签名 `.dmg` 最小闭环已完成：Tauri bundle targets 已包含 `app` 与 `dmg`；
  `cd tauri-app && npm run build` 可产出未签名、未公证 `.dmg`。
- 推荐下一版本号为 `v1.1.0`，Tauri 桌面壳版本已同步为 `1.1.0`。
- 当前最终 build 验证后的 `.dmg` 产物路径：
  `tauri-app/src-tauri/target/release/bundle/dmg/Pixiv Platform_1.1.0_aarch64.dmg`。
- `.dmg` 本机挂载验证通过：卷内存在 `Pixiv Platform.app` 和
  `Applications -> /Applications` 拖拽链接；验证后已卸载。
- 已新增 `tauri-app/docs/distribution.md`，记录 GitHub Release 小范围分发说明、Gatekeeper
  风险和测试用户无需安装 Rust、Cargo、Node、npm、TypeScript。
- 已清理残留：全部 `.DS_Store`、`src/frontend/.next`、`src/frontend/out`、
  `src/frontend/tsconfig.tsbuildinfo`、`tauri-app/src-tauri/target`、`tauri-app/node_modules`。
  随后为 `v1.1.0` 提交前最终验证重新执行了 `npm install` 和 `npm run build`，当前本地构建产物
  已重新生成且均被 `.gitignore` 覆盖。
- `tauri-app/src-tauri/gen/` 是 Tauri 自动生成 schema 目录，已加入 `.gitignore`，不要提交。
- Settings 中 `download_base_path` 已改为桌面文件夹选择器选择并自动保存，不再要求手动输入。
- 热缓存测速结果：Tauri `cargo check` 约 0.78s；前端 lint 约 1.07s；Next 静态导出约 4.92s；完整 Tauri build 约 26.58s。
- Web / 后端独立运行默认仍沿用项目 `output/`；Tauri 桌面端默认目录使用
  `~/Downloads/Pixiv Platform/`。
- 默认不要 git commit，除非用户明确要求。
- 用户当前没有 Apple Developer Program / Apple 开发者认证背景；下一阶段默认以未签名 `.dmg`
  和 GitHub Release 小范围分发为前提，不把签名、公证、自动更新作为立即实现项。
- 当前工作区已有用户既有改动，尤其 `src/frontend/app/gallery/page.tsx`、`src/frontend/app/page.tsx`、`src/frontend/lib/api.ts` 等，需谨慎，不要回退用户改动。
- `tauri-app` 目录目前可能仍显示为 untracked，按现状工作，不要擅自提交或清理。

渐进式披露规则：

- 只有当前任务需要时，才打开具体代码文件。
- 用 `rg` 精确搜索符号或路径。
- 不要重复读取大文件，不要重新总结整个仓库。
- 先输出本阶段最小计划和需要用户确认的事项，再动实现。
- 遵循 spec-coding：先文档，后实现，再验证，再同步 `progress` / `checklist` / `testing`。

下一阶段候选：

1. GitHub commit：如用户明确确认，可把当前桌面化阶段作为一个提交；建议 commit message 为
   `feat: add macOS Tauri desktop packaging`。
2. P3a 人工分发复核：重新 build 后，用户可手动打开 `.dmg`，拖入 Applications 后启动，确认小范围分发体验。
3. P3 后续评估：签名、公证、自动更新仅作为未来正式分发路径评估；不要把账号、证书或
   secret 写入仓库或文档。
4. P1 后续：如需更正式的数据分层，再评估 SQLite 是否迁移到 macOS Application Support。

清理候选，未确认前不要删除：

- 多处 `.DS_Store`。
- `src/frontend/.next`。
- `src/frontend/out`。
- `tauri-app/src-tauri/target`。
- `tauri-app/node_modules`。
- `src/frontend/.next/dev/lock`。
- `src/frontend/tsconfig.tsbuildinfo`。
