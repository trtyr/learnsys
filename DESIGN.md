# Design · 学习系统 (learnsys)

<!-- impeccable:design -->

## Visual World: Split-Flap Departure Board

学习管理系统的 UI 是一块**知识出发板**——就像火车站/机场的拆分翻页时刻表。
到期卡片是即将出发的航班，模块是站台，目标是目的地。

## Palette

| Token | Hex | Use |
|---|---|---|
| `--bg` | `#0d0d0d` | 底色（matte black flap board back） |
| `--surface` | `#1a1a1a` | 卡片/面板面（flap face） |
| `--border` | `#333333` | 分隔线（steel frame） |
| `--text` | `#e8e8e8` | 正文（white paint letters） |
| `--muted` | `#666666` | 次要信息 |
| `--amber` | `#f5a623` | 逾期/延误（row lamp） |
| `--red` | `#d64545` | 取消/严重逾期 |
| `--green` | `#5a9e6f` | 正常/已出发 |
| `--accent` | `#c0c0c0` | 强调（steel highlight） |

## Typography

- **Flap data**: 'JetBrains Mono', monospace — 出发板的核心字体，固定字宽，字符像翻页单元
- **Headings**: 'Inter', sans-serif — 站台标识/标题
- **Body/UI**: 'Inter', sans-serif — 辅助信息

rules: tracking floor -0.02em, body measure max 75ch, headings balanced

## Composition

- 顶部：站台钟式 header（产品名 + 延误计数 + 日期）
- 标签：站台标识（计划/复习/进度/画像），当前激活亮白、其它灰
- 内容区：规则行出发列表（列头 + 数据行 + 行间分隔线）
- 底部：concourse 状态栏（已出发/待出发/延误 汇总）

## Topology

- 行是活实体，按到期日排列
- 列固定：到期日 | 模块 | 状态 | 领域
- 逾期行亮琥珀灯（行左侧竖线）
- 切换标签 = 切换出发板视图

## Controls & States

- hover: 行背景微亮（ flap 被选中）
- active tab: 亮白文字 + 底部钢线
- overdue: 琥珀左竖线 + 琥珀日期
- extreme overdue: 红色左竖线
- on-time: 绿色小点

## Motion

- 页面加载：行从上到下依次淡入（40ms stagger）
- 标签切换：内容区从右滑入（150ms ease-out）
- hover 行：背景 100ms transition

## Responsive

- 窄屏（<640px）：列精简为 到期日 | 标题 | 状态
