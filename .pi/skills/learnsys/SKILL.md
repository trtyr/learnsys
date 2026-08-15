---
name: learnsys
description: 操作「学习系统 (learnsys)」个人学习平台的命令行工具。用 Python CLI 脚本对卡片/主题/目标/路径/模块/会话/资源/画像/统计/导出/备份做全部增删改查与复习调度。当你需要读写学习系统数据、建卡、复习、查进度、导出备份，或让 AI 通过脚本操作这个项目时使用。
---

# 学习系统 CLI

通过 Python 脚本操作 learnsys 的全部功能。脚本只依赖标准库，无需安装依赖。

## 前置条件

后端服务要跑着。二选一：

```bash
# 本地开发
cargo run -p learnsys-api            # http://127.0.0.1:7878

# Docker
docker compose up -d
```

若 API 不在默认地址，用环境变量指定：

```bash
export LEARNSYS_URL=http://127.0.0.1:7878
```

## 用法

```bash
python3 scripts/learnsys.py <资源> <子命令> [参数]
```

所有子命令把返回的 JSON 原样打印（`ensure_ascii=False`，中文可读）。`delete` / `end` 等 204 响应静默成功；出错打印 `✗ HTTP <code>: <detail>` 到 stderr。

## 快速上手

```bash
# 记一张卡（topic 不存在会自动建；可带标签/代码块/出处/关联）
python3 scripts/learnsys.py card create --topic rust --front "所有权是什么" --back "独占" \
    --tags rust,基础 --code-block "fn main() {}" --source "《Rust 编程之道》第 3 章"
python3 scripts/learnsys.py card create --topic rust --front "连接复用" --back "RAII" --related <卡id>

# 今天要复习的卡 + 复习一张
python3 scripts/learnsys.py card new
python3 scripts/learnsys.py card review <id> 5

# 看今日概览 + 时间线
python3 scripts/learnsys.py dashboard
python3 scripts/learnsys.py timeline

# 复习洞察：未来 7 天排期 + 薄弱点
python3 scripts/learnsys.py upcoming --days 7
python3 scripts/learnsys.py weak-topics

# 建一条学习链路：目标 → 路径 → 模块 → 挂卡
python3 scripts/learnsys.py goal create "学 Rust"
python3 scripts/learnsys.py pathway create "基础优先" --goal <goal_id>
python3 scripts/learnsys.py module create "所有权"
python3 scripts/learnsys.py pathway add-module <pathway_id> --module <module_id> --order 0

# 导出 / 备份
python3 scripts/learnsys.py export > backup.json
python3 scripts/learnsys.py backup
```

## 命令清单

| 资源 | 子命令 |
|------|--------|
| `card` | `create` `list` `get` `search` `edit` `delete` `review` `new` `leeches` |
| `topic` | `create` `list` `get` `update` |
| `goal` | `create` `list` `get` `update` `delete` `status` `progress` |
| `pathway` | `create` `list` `get` `update` `delete` `modules` `add-module` `next` |
| `module` | `create` `list` `update` `delete` `mastery` `cards` `status` |
| `session` | `start` `end` `list` |
| `resource` | `create` `list` |
| `profile` | `get` `update` |
| `settings` | `get` `set` |
| `quiz` | （无子命令，`--n` / `--topic`） |
| — | `stats` `dashboard` `heatmap` `upcoming` `weak-topics` `export` `export-markdown` `backup` `timeline` |

每个命令的具体参数和 payload 见 [references/api.md](references/api.md)；实体字段与关系见 [references/data-model.md](references/data-model.md)。

## 给 AI 的使用约定

- 用 `python3 scripts/learnsys.py ...` 操作，不要直接手写 HTTP 请求。
- `card review` 是 SM-2 调度的**唯一入口**——判断掌握度时用它喂 quality（0-5）。
- 建卡前先 `card search <关键词>` 查重，避免重复记录。
- `export` / `backup` 用于持久化；`timeline` 用于看"今天做了什么"。
- 涉及数据模型细节（字段、枚举值、级联关系）先读 references 再操作。
