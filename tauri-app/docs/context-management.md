# Context Management Standard

## 目的

桌面化工作会跨越前端、后端、Tauri、打包和系统权限。为节省上下文与降低误改风险，
每个阶段都必须维护轻量但可恢复的上下文锚点。

## 必读顺序

新会话或上下文压缩后，优先只读：

1. `tauri-app/docs/progress.md`
2. `tauri-app/docs/requirements.md`
3. `tauri-app/docs/architecture.md`
4. `tauri-app/docs/implementation-plan.md`
5. 本轮任务直接相关文件

除非需要全局审计，不要一开始全仓库浏览。

## 何时提醒用户新开窗口

当出现以下情况，应提醒用户考虑开启新会话或新窗口：

- 当前对话已经包含大量历史实现细节，开始影响响应速度或准确性。
- 即将从文档/方案阶段切到大规模实现阶段。
- 即将从 MVP dev 模式切到打包、签名、公证等新主题。
- 当前任务完成，下一任务属于独立方向，例如数据目录迁移或自动更新。

## 省 Token 原则

- 先读进度文档，再读目标文件。
- 用 `rg` 查找具体符号，避免打开无关大文件。
- 只摘录必要上下文，不重复粘贴长代码。
- 实现前写清楚本阶段范围，避免边做边扩散。
- 每次完成阶段后更新 `progress.md`，让下一轮不必重新推理历史。

## 新会话提示词

```text
你在 /Users/Admin/Downloads/pixiv_platform 继续 Tauri 桌面化工作。

请节省 token，采用渐进式披露检索，不要全局浏览仓库。先只读：
1. tauri-app/docs/progress.md
2. tauri-app/docs/requirements.md
3. tauri-app/docs/architecture.md
4. tauri-app/docs/implementation-plan.md
5. tauri-app/docs/checklist.md
6. tauri-app/docs/testing.md

当前策略：
- tauri-app 是桌面壳，不复制 src/frontend 或 src/backend。
- 前端继续来自 src/frontend。
- 后端通过 Cargo path dependency 引用 src/backend。
- Desktop MVP dev 模式已跑通，并已由用户手动验证体验与网页端一致。
- Desktop `.app` 打包 MVP 已跑通，并已由用户手动验证功能正常。
- 当前桌面端每次启动使用随机 `127.0.0.1:<port>` 后端端口。
- Tauri 创建 WebView 时注入 `window.__PIXIV_PLATFORM_BACKEND_URL__`，前端用它拼接 API base URL。
- MVP 沿用项目 output/，未得到用户确认前不迁移到 macOS 应用数据目录。
- 遵循 spec-coding：先更新文档，再实现，再验证，再同步 progress/checklist。
- 默认不要 git commit，除非用户明确要求。

渐进式披露规则：
- 只有当当前任务需要时，才打开具体代码文件。
- 用 rg 精确搜索符号或路径。
- 不要重复读取大文件，不要重新总结整个仓库。
- 先输出本阶段最小计划和需要用户确认的事项，再动实现。

本轮目标：<填写具体任务>
```
