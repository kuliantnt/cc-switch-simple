# 更新日志 (Changelog)

本文件记录 `cc-switch` / `cx-switch` 的每个发布版本。版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [0.1.10] - 2026-09-09

### 新增

- 新增首次收纳功能：切换离开当前 Codex 预设时，若活动目录中存在尚未保存到当前预设的 `models.json` 或 `models_catalog.json`（当前预设没有该文件但活动目录中有），会提示保存回当前预设
- 首次收纳的文件与常规变更一样经过 JSON 校验，拒绝非法 JSON 时中止切换且不产生部分写入状态

### 其他

- 补充 README 中 `models.json` 的处理说明

## [0.1.9] - 2026-09-09

### 新增

- Codex 预设支持可选的 `models.json`：切换时一并写入目标目录；预设不含该文件时先备份并删除活动目录中的旧文件
- 活动 `models.json` 回写前进行 JSON 校验，非法 JSON 阻止切换且不产生部分写入状态
- README（中英文）补充 `models.json` 的运行时目录、自动备份和切换行为说明

## [0.1.8] - 2026-07-31

### 变更

- DeepSeek 预设改为直连官方 API：`base_url` 指向 `https://api.deepseek.com/v1`，不再依赖本地 moonbridge 代理
- DeepSeek 预设改用 OpenAI Responses API（`wire_api = "responses"`），兼容 Codex CLI 对 `chat/completions` 的移除
- DeepSeek 预设默认模型改为 `deepseek-v4-flash`（DeepSeek V4 Flash）

## [0.1.7] - 2026-07-31

### 新增

- 内置 DeepSeek 预设：`codex/deepseek/`，包含 `config.toml`、`auth.json` 和完整的 `models_catalog.json`（DeepSeek V4 Pro 模型目录，支持 high / xhigh 推理强度）
- 独立 Codex 切换命令 `cx-switch`，可脱离 `cc-switch` 单独使用
- Codex 预设支持可选的 `models_catalog.json`，切换时一并写入目标目录
- `cx` 子命令新增独立参数（详见 `cc-switch cx --help`）

### 修复

- 修复 Codex 发布（release）与目录（catalog）切换时配置写入不完整的问题
- 修正 Codex 预设切换的测试覆盖，补齐成功与跳过场景

### 其他

- 更新 README 社区链接

## [0.1.6] - 2026-07-21

### 其他

- release workflow 将三平台构建产物打包为归档（zip / tar.gz）上传

## [0.1.5] - 2026-06-17

### 修复

- 修复 Codex Plus 切换后需要重新登录的问题：认证变化默认必须保存，不再允许按回车丢弃刚登录好的 Plus token

## [0.1.3] - 2026-05-31

### 修复

- 修复 Windows 下 Codex 路径相关测试失败

## [0.1.2] - 2026-05-31

### 新增

- 新增 Codex 预设切换：支持 `config.toml` / `auth.json` 预设的 `list` / `current` / `use` / `next` / `before` 命令
- 切换 Codex 预设前自动备份现有配置，`before` 可回退到最近一次切换前的预设
- 通过 `~/.cc-switch-simple/codex/current` 记录当前 Codex 预设

## [0.1.1] - 2026-05-31

### 其他

- 添加 CI release workflow：tag 触发三平台（Windows / macOS / Linux）构建并上传 artifact

## [0.1.0] - 2026-05-31

### 新增

- 首个版本：Claude Code JSON profile 切换工具
- 支持 `list` / `current` / `use` / `next` / `before` / `doctor` 命令
- 每次覆盖目标配置前自动备份
- 不输出敏感值，单文件可执行程序，跨平台运行
