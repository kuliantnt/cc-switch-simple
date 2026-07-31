# 更新日志 (Changelog)

本文件记录 `cc-switch` / `cx-switch` 的每个发布版本。版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

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

### 修复

- 修复 Codex Plus 切换后需要重新登录的问题：认证变化默认必须保存，不再允许按回车丢弃刚登录好的 Plus token（此修复曾以 v1.1.5 标记发布）

### 其他

- release workflow 将三平台构建产物打包为归档（zip / tar.gz）上传

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
