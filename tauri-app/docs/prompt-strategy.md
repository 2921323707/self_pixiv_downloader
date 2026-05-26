# Prompt Strategy

## 基本策略

对 Tauri 桌面化任务，提示词应优先说明：

- 当前阶段目标。
- 明确非目标。
- 允许修改的目录。
- 禁止触碰的内容。
- 需要运行的验证命令。
- 是否允许联网安装依赖。

## 推荐短提示

```text
继续 tauri-app 桌面化。Desktop MVP dev 模式已跑通，用户已手动验证体验与网页端一致。
先读 tauri-app/docs/progress.md 和 tauri-app/docs/implementation-plan.md，
只处理当前阶段，不做应用数据目录迁移、签名、公证、自动更新。
遵循文档先行，完成后更新 progress/checklist。
```

## 防扩散提示

```text
不要重构现有业务逻辑，不复制前后端源码，不把 HTTP API 改成 Tauri commands。
只做本阶段最小改动。
```

## 验证提示

```text
完成后运行相关最小验证。若依赖安装或网络访问失败，记录错误并请求批准，
不要绕过沙箱或手工写入外部依赖。
```

## 下一阶段规划提示

```text
本轮只做 .app 打包前置规划，不直接大改代码。请先基于 tauri-app/docs 说明：
1. Next 前端生产模式应该采用静态导出、standalone/server，还是其它方式。
2. 生产模式下 Tauri 如何访问本地 Rust API。
3. 固定 127.0.0.1:3000 在打包阶段的风险和替代方案。
4. 最小打包验收清单。
5. 需要用户确认的事项。

请保持渐进式披露检索：只有需要判断具体配置时，再打开 next.config、package.json、
tauri.conf、src-tauri/main.rs 或前端 api client。
```
