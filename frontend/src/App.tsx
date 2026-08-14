import { useCallback, useEffect, useState, type ReactNode } from 'react'
import { api } from './api'
import type { Card, Dashboard, Goal, GoalProgress, HeatmapDay, LearnerProfile, Module, ModuleMastery, Pathway, PathwayModule, Resource, Session } from './types'

type Tab = 'plan' | 'review' | 'progress' | 'profile'

export default function App() {
  const [tab, setTab] = useState<Tab>('review')
  const [dash, setDash] = useState<Dashboard | null>(null)
  const [profile, setProfile] = useState<LearnerProfile | null>(null)
  const [loading, setLoading] = useState(true)
  const [err, setErr] = useState<string | null>(null)

  const reload = useCallback(() => {
    Promise.all([api.dashboard(), api.profile.get()])
      .then(([d, p]) => { setDash(d); setProfile(p) })
      .catch((e) => setErr(String(e)))
      .finally(() => setLoading(false))
  }, [])
  useEffect(reload, [reload])

  if (err) return <Shell><div className="error">加载失败: {err}</div></Shell>
  if (loading || !dash) return <Shell><div className="loading">知识出发板启动中…</div></Shell>

  const today = new Date().toISOString().slice(0, 10)

  return (
    <Shell>
      <header className="header">
        <h1>学习系统<span className="muted"> / 知识出发板</span></h1>
        <div className="header-stats">
          <span>{today}</span>
          <ReminderBadges dash={dash} />
        </div>
      </header>

      <nav className="tabs">
        {(['plan', 'review', 'progress', 'profile'] as Tab[]).map((t) => (
          <button key={t} className={`tab ${tab === t ? 'active' : ''}`} onClick={() => setTab(t)}>
            {{ plan: '计划 / Plan', review: '出发板 / Board', progress: '进度 / Log', profile: '站务 / Master' }[t]}
          </button>
        ))}
      </nav>

      <main>
        {tab === 'plan' && <PlanView onRefresh={reload} />}
        {tab === 'review' && <ReviewView dash={dash} onRefresh={reload} />}
        {tab === 'progress' && <ProgressView dash={dash} />}
        {tab === 'profile' && <ProfileView profile={profile} onRefresh={reload} />}
      </main>

      <footer className="footer">知识出发板 · AI 调 API 操作 · today {today}</footer>
    </Shell>
  )
}

function Shell({ children }: { children: ReactNode }) {
  return <div className="wrap">{children}</div>
}

// 提醒红点：待出发/延误/顽固卡/streak，按需亮起的徽标。
export function ReminderBadges({ dash }: { dash: Dashboard }) {
  return (
    <>
      {dash.due_today > 0 && <span className="warn">◆ {dash.due_today} 待出发</span>}
      {dash.due_soon > 0 && <span className="hot">◈ {dash.due_soon} 延误</span>}
      {dash.leech_count > 0 && <span className="hot">⚠ {dash.leech_count} 顽固卡</span>}
      {dash.streak > 0 && <span className="green">🔥 {dash.streak} 天连续</span>}
    </>
  )
}

// ═════════════════ PlanView — 目标进度 + 路径模块 ═════════════════

function PlanView({ onRefresh }: { onRefresh: () => void }) {
  const [goals, setGoals] = useState<Goal[]>([])
  const [progress, setProgress] = useState<Record<string, GoalProgress>>({})
  const [newTitle, setNewTitle] = useState('')
  const [method, setMethod] = useState('基础优先')

  const load = useCallback(() => {
    api.goals.list().then((gs) => {
      setGoals(gs)
      gs.forEach((g) => api.goals.progress(g.id).then((p) => setProgress((x) => ({ ...x, [g.id]: p }))).catch(() => {}))
    })
  }, [])
  useEffect(load, [load])

  return (
    <div>
      <div className="form-row">
        <input placeholder="新目标…" value={newTitle} onChange={(e) => setNewTitle(e.target.value)} style={{ flex: 2 }} />
        <select value={method} onChange={(e) => setMethod(e.target.value)}>
          <option>基础优先</option><option>项目驱动</option><option>源码驱动</option><option>问题驱动</option>
        </select>
        <button onClick={async () => {
          if (!newTitle) return
          const g = await api.goals.create({ title: newTitle })
          await api.pathways.create({ name: newTitle + ' · ' + method, goal_id: g.id, methodology: method })
          setNewTitle(''); load(); onRefresh()
        }}>+ 目标</button>
      </div>

      {goals.length === 0 && <div className="loading">还没有学习目标——建一个吧。</div>}
      {goals.map((g) => {
        const p = progress[g.id]
        return (
          <div key={g.id} className="panel">
            <GoalRow goal={g} progress={p} onRefresh={() => { load(); onRefresh() }} />
          </div>
        )
      })}
    </div>
  )
}

function GoalRow({ goal, progress, onRefresh }: { goal: Goal; progress?: GoalProgress; onRefresh: () => void }) {
  const [expand, setExpand] = useState(false)
  const [pws, setPws] = useState<Pathway[]>([])

  useEffect(() => {
    if (expand) api.pathways.listByGoal(goal.id).then(setPws)
  }, [expand, goal.id])

  const pct = progress?.percent ?? 0

  return (
    <div>
      <div className="goal-row" onClick={() => setExpand(!expand)}>
        <span className={`tag ${goal.status === 'achieved' ? 'green' : goal.status === 'abandoned' ? 'muted' : 'amber'}`}>{goal.status}</span>
        <b>{goal.title}</b>
        {goal.success_criteria && <span className="muted">— {goal.success_criteria}</span>}
        <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-flap)', color: 'var(--muted)' }}>{expand ? '▾' : '▸'}</span>
      </div>
      {progress && progress.total_modules > 0 && (
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '0 0 8px', fontFamily: 'var(--font-flap)', fontSize: 11, color: 'var(--muted)' }}>
          <div style={{ flex: 1, background: '#1a1a1a', height: 6, borderRadius: 3, overflow: 'hidden' }}>
            <div style={{ width: `${pct}%`, height: '100%', background: 'var(--amber)', borderRadius: 3, transition: 'width .4s' }} />
          </div>
          <span>{progress.mastered}/{progress.total_modules} · {pct.toFixed(0)}%</span>
        </div>
      )}
      {expand && (
        <div className="panel-body" style={{ borderTop: '1px solid var(--border)' }}>
          {pws.length === 0 && <div className="muted" style={{ fontSize: 12 }}>还没路径。</div>}
          {pws.map((pw) => <PathwayRow key={pw.id} pw={pw} onRefresh={onRefresh} />)}
          <button className="ghost-btn" onClick={async () => {
            await api.pathways.create({ name: '新路径', goal_id: goal.id })
            api.pathways.listByGoal(goal.id).then(setPws); onRefresh()
          }}>+ 新路径</button>
        </div>
      )}
    </div>
  )
}

function PathwayRow({ pw, onRefresh }: { pw: Pathway; onRefresh: () => void }) {
  const [expand, setExpand] = useState(false)
  const [pms, setPms] = useState<PathwayModule[]>([])
  const [allMods, setAllMods] = useState<Module[]>([])
  const [next, setNext] = useState<{ done?: boolean; module?: Module; position?: number; total?: number } | null>(null)
  const [title, setTitle] = useState('')
  const [order, setOrder] = useState(1)

  const loadDetail = useCallback(() => {
    api.pathways.modules(pw.id).then(setPms)
    api.pathways.next(pw.id).then(setNext).catch(() => {})
    api.modules.list().then(setAllMods)
  }, [pw.id])

  useEffect(() => { if (expand) loadDetail() }, [expand, loadDetail])

  return (
    <div>
      <div className="pathway-row" onClick={() => setExpand(!expand)}>
        <span className="tag" style={{ background: 'rgba(192,192,192,.12)', color: 'var(--accent)' }}>{pw.methodology || '路径'}</span>
        <b>{pw.name}</b>
        <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-flap)', color: 'var(--muted)' }}>{expand ? '▾' : '▸'}</span>
      </div>
      {expand && (
        <div>
          {pms.map((pm, i) => <ModuleRow key={pm.module_id} pm={pm} idx={i} modules={allMods} onRefresh={() => { loadDetail(); onRefresh() }} />)}
          {next?.done && <div style={{ padding: '6px 0 6px 24px', color: 'var(--green)', fontFamily: 'var(--font-flap)', fontSize: 12 }}>◆ 全线完成</div>}
          {next?.module && !next.done && (
            <div style={{ padding: '6px 0 6px 24px', color: 'var(--amber)', fontFamily: 'var(--font-flap)', fontSize: 12 }}>▶ 下一站: {next.module.title} ({next.position}/{next.total})</div>
          )}
          <div className="form-row" style={{ marginTop: 8, marginLeft: 24 }}>
            <input placeholder="模块名" value={title} onChange={(e) => setTitle(e.target.value)} style={{ flex: 2 }} />
            <input placeholder="#" value={order} type="number" style={{ width: 56 }} onChange={(e) => setOrder(Number(e.target.value))} />
            <button onClick={async () => {
              if (!title) return
              const m = await api.modules.create({ title })
              await api.pathways.addModule(pw.id, { module_id: m.id, sort_order: order })
              setTitle(''); setOrder(order + 1); loadDetail(); onRefresh()
            }}>+ 模块</button>
          </div>
        </div>
      )}
    </div>
  )
}

function ModuleRow({ pm, idx, modules, onRefresh }: { pm: PathwayModule; idx: number; modules: Module[]; onRefresh: () => void }) {
  const [expand, setExpand] = useState(false)
  const mod = modules.find((m) => m.id === pm.module_id)
  const modId = mod?.id
  const [mastery, setMastery] = useState<ModuleMastery | null>(null)
  const [resources, setResources] = useState<Resource[]>([])
  const [resTitle, setResTitle] = useState('')
  const [resUrl, setResUrl] = useState('')

  useEffect(() => {
    if (expand && modId) {
      api.modules.mastery(modId).then(setMastery)
      api.resources.list(modId).then(setResources)
    }
  }, [expand, modId])

  const cycleStatus = async () => {
    if (!mod) return
    const next = mod.status === 'mastered' ? 'learning' : mod.status === 'learning' ? 'not_started' : 'mastered'
    await api.modules.updateStatus(mod.id, next)
    onRefresh()
  }

  return (
    <div>
      <div className="module-row">
        <span className="idx">{idx + 1}</span>
        <span className="title" style={{ cursor: 'pointer' }} onClick={() => setExpand(!expand)}>
          {mod?.title || pm.module_id.slice(0, 12)} {expand ? '▾' : '▸'}
        </span>
        {mod && (
          <span className="tag" onClick={cycleStatus} style={{
            cursor: 'pointer',
            background: mod.status === 'mastered' ? 'rgba(90,158,111,.15)' : mod.status === 'learning' ? 'rgba(245,166,35,.15)' : 'rgba(102,102,102,.1)',
            color: mod.status === 'mastered' ? 'var(--green)' : mod.status === 'learning' ? 'var(--amber)' : 'var(--muted)'
          }}>{mod.status}</span>
        )}
        {pm.depends_on.length > 0 && (
          <span className="depends">◂ {pm.depends_on.map((did) => modules.find((m) => m.id === did)?.title || did.slice(0, 8)).join(' · ')}</span>
        )}
      </div>
      {expand && mod && (
        <div style={{ padding: '4px 0 12px 48px', borderLeft: '1px solid var(--border)', marginLeft: 12 }}>
          {mastery && (
            <div style={{ fontFamily: 'var(--font-flap)', fontSize: 11, color: 'var(--muted)', marginBottom: 6 }}>
              卡片 {mastery.total_cards} · 已学 {mastery.learned} · 平均 EF {mastery.avg_ef.toFixed(2)} · 待复习 {mastery.due_count}
            </div>
          )}
          {resources.map((r) => (
            <div key={r.id} style={{ fontFamily: 'var(--font-flap)', fontSize: 12, padding: '2px 0' }}>
              {r.url ? <a href={r.url} target="_blank" rel="noreferrer" style={{ color: 'var(--accent)' }}>{r.title}</a> : <span>{r.title}</span>}
              {r.notes && <span className="muted"> — {r.notes}</span>}
            </div>
          ))}
          <div className="form-row" style={{ marginTop: 6 }}>
            <input placeholder="资源名" value={resTitle} onChange={(e) => setResTitle(e.target.value)} />
            <input placeholder="URL" value={resUrl} onChange={(e) => setResUrl(e.target.value)} />
            <button onClick={async () => {
              if (!resTitle) return
              await api.resources.create({ title: resTitle, url: resUrl, module_id: mod.id })
              setResTitle(''); setResUrl('')
              api.resources.list(mod.id).then(setResources)
            }}>+ 资源</button>
          </div>
        </div>
      )}
    </div>
  )
}

// ═══════════════ ReviewView — 翻卡复习 + 搜索 + 编辑 ═══════════════

function ReviewView({ dash, onRefresh }: { dash: Dashboard; onRefresh: () => void }) {
  const [due, setDue] = useState<Card[]>([])
  const [flipped, setFlipped] = useState<string | null>(null)
  const [q, setQ] = useState('')
  const [results, setResults] = useState<Card[] | null>(null)
  const [editing, setEditing] = useState<Card | null>(null)
  const [newCards, setNewCards] = useState<Card[] | null>(null)
  const [quiz, setQuiz] = useState<Card[] | null>(null)
  const [budget, setBudget] = useState(5)
  const load = useCallback(() => { api.cards.due().then(setDue) }, [])
  useEffect(load, [load])
  useEffect(() => { api.settings.get().then((s) => setBudget(s.new_per_day)).catch(() => {}) }, [])
  const today = new Date().toISOString().slice(0, 10)

  const doSearch = (v: string) => {
    setQ(v)
    const s = v.trim()
    if (!s) { setResults(null); return }
    api.cards.search(s).then(setResults).catch(() => setResults([]))
  }

  const review = (c: Card, qq: number) => {
    api.cards.review(c.id, qq).then(() => { setFlipped(null); load(); onRefresh() })
  }

  return (
    <div>
      <div className="stats-strip">
        <Stat label="待出发" value={dash.due_today} accent={dash.due_today > 0 ? 'warn' : ''} />
        <Stat label="延误" value={dash.due_soon} accent={dash.due_soon > 0 ? 'hot' : ''} />
        <Stat label="总计" value={dash.stats.total_cards} />
        <Stat label="平均 EF" value={dash.stats.avg_ef.toFixed(2)} />
      </div>

      <div className="form-row" style={{ margin: '10px 0' }}>
        <input placeholder="搜卡片（正面/背面/标签）…" value={q} onChange={(e) => doSearch(e.target.value)} />
      </div>

      <div className="form-row" style={{ margin: '0 0 10px', gap: 8 }}>
        <button className="ghost-btn" onClick={() => {
          if (newCards) setNewCards(null)
          else api.cards.new().then(setNewCards).catch(() => setNewCards([]))
        }}>{newCards ? '收起新卡' : `学新卡 (${dash.stats.new_cards})`}</button>
        <button className="ghost-btn" onClick={() => {
          if (quiz) setQuiz(null)
          else api.quiz(5).then(setQuiz).catch(() => setQuiz([]))
        }}>{quiz ? '收起测验' : '测验 5 题'}</button>
        <input type="number" style={{ width: 64 }} value={budget}
          onChange={(e) => setBudget(Number(e.target.value))}
          onBlur={() => api.settings.put({ new_per_day: budget }).then(onRefresh)}
          title="每日新卡预算" />
      </div>

      {newCards !== null && (
        <div className="panel" style={{ marginBottom: 10 }}>
          <div className="panel-header">新卡 · 今日 {newCards.length} 张</div>
          <div className="panel-body" style={{ padding: 0 }}>
            {newCards.length === 0 ? (
              <div className="loading">◆ 今日新卡已学完。</div>
            ) : (
              <div className="board-list">
                {newCards.map((c) => (
                  <CardRow key={c.id} c={c} today={today} flipped={flipped === c.id}
                    onFlip={() => setFlipped(flipped === c.id ? null : c.id)}
                    onEdit={setEditing} onReview={(qq) => review(c, qq)} showStatus={false} />
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {quiz !== null && (
        <div className="panel" style={{ marginBottom: 10 }}>
          <div className="panel-header">测验 · {quiz.length} 题（随机抽自到期复习卡）</div>
          <div className="panel-body" style={{ padding: 0 }}>
            {quiz.length === 0 ? (
              <div className="loading">◆ 暂无到期复习卡。</div>
            ) : (
              <div className="board-list">
                {quiz.map((c) => (
                  <CardRow key={c.id} c={c} today={today} flipped={flipped === c.id}
                    onFlip={() => setFlipped(flipped === c.id ? null : c.id)}
                    onEdit={setEditing} onReview={(qq) => review(c, qq)} showStatus={false} />
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {results !== null && (
        <div className="panel" style={{ marginBottom: 10 }}>
          <div className="panel-header">搜索结果 · {results.length} 张</div>
          <div className="panel-body" style={{ padding: 0 }}>
            {results.length === 0 ? (
              <div className="loading">◆ 无匹配。</div>
            ) : (
              <div className="board-list">
                {results.map((c) => (
                  <CardRow key={c.id} c={c} today={today} flipped={flipped === c.id}
                    onFlip={() => setFlipped(flipped === c.id ? null : c.id)}
                    onEdit={setEditing} onReview={(qq) => review(c, qq)} showStatus={false} />
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {editing && (
        <CardEditor card={editing} onClose={() => setEditing(null)}
          onSaved={() => { setEditing(null); setResults(null); load(); onRefresh() }} />
      )}

      <div className="panel">
        <div className="panel-header">出发板 · 今日待复习 {due.length} 班</div>
        <div className="panel-body" style={{ padding: 0 }}>
          {due.length === 0 ? (
            <div className="loading">◆ 今日无出发——全线清空。</div>
          ) : (
            <div className="board-list">
              {due.map((c) => (
                <CardRow key={c.id} c={c} today={today} flipped={flipped === c.id}
                  onFlip={() => setFlipped(flipped === c.id ? null : c.id)}
                  onEdit={setEditing} onReview={(qq) => review(c, qq)} showStatus />
              ))}
            </div>
          )}
        </div>
        <div className="status-bar" style={{ padding: '8px 16px' }}>
          <span>已出发 <b>{dash.stats.total_cards - dash.due_today}</b></span>
          <span>待出发 <b className="warn">{dash.due_today}</b></span>
          <span className="muted">点卡片翻面 · 打分后自动调度</span>
        </div>
      </div>
    </div>
  )
}

export function CardRow({ c, today, flipped, onFlip, onEdit, onReview, showStatus }: {
  c: Card; today: string; flipped: boolean; onFlip: () => void;
  onEdit: (c: Card) => void; onReview: (q: number) => void; showStatus: boolean;
}) {
  const days = Math.floor((Date.parse(today) - Date.parse(c.due)) / 86400000)
  const cls = days > 1 ? 'extreme' : days > 0 ? 'overdue' : 'on-time'
  const status = days > 1 ? `延误 ${days}天` : days > 0 ? '已延误' : '准时'
  const tagCls = days > 1 ? 'red' : days > 0 ? 'amber' : 'green'
  return (
    <div className={`flap-row ${cls}`} onClick={onFlip}>
      <div className="flap-due">{c.due.slice(5)}</div>
      <div className="flap-main">
        <div className="flap-front">{flipped ? c.back : c.front}</div>
        {c.tags.length > 0 && (
          <div style={{ display: 'flex', gap: 4, marginTop: 4, flexWrap: 'wrap' }}>
            {c.tags.map((t) => <span key={t} className="tag muted" style={{ fontSize: 10 }}>#{t}</span>)}
          </div>
        )}
        {c.code_block && (
          <pre style={{ marginTop: 6, padding: 6, background: '#111', borderRadius: 3, fontSize: 11, overflowX: 'auto', fontFamily: 'var(--font-flap)' }}>{c.code_block}</pre>
        )}
        {c.image_urls.length > 0 && (
          <div style={{ display: 'flex', gap: 6, marginTop: 6, flexWrap: 'wrap' }}>
            {c.image_urls.map((u) => (
              <img key={u} src={u} alt="" style={{ maxHeight: 80, borderRadius: 3, border: '1px solid var(--border)' }} />
            ))}
          </div>
        )}
        {flipped && (
          <div className="flap-answer-actions" onClick={(e) => e.stopPropagation()}>
            <span style={{ fontSize: 11, color: 'var(--muted)' }}>翻面 · 自评质量分：</span>
            {[0, 1, 2, 3, 4, 5].map((qq) => (
              <button key={qq} className="rate-btn" onClick={() => onReview(qq)}>{qq}</button>
            ))}
          </div>
        )}
      </div>
      {showStatus && <div className="flap-status"><span className={`tag ${tagCls}`}>{status}</span></div>}
      <div className="flap-meta" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
        <span className="tag muted">{c.topic}</span>
        <button className="ghost-btn" style={{ padding: '2px 6px', fontSize: 11 }}
          onClick={(e) => { e.stopPropagation(); onEdit(c) }}>✎ 编辑</button>
      </div>
    </div>
  )
}

export function CardEditor({ card, onClose, onSaved }: { card: Card; onClose: () => void; onSaved: () => void }) {
  const [front, setFront] = useState(card.front)
  const [back, setBack] = useState(card.back)
  const [tags, setTags] = useState(card.tags.join(', '))
  const [codeBlock, setCodeBlock] = useState(card.code_block ?? '')
  const [imageUrls, setImageUrls] = useState(card.image_urls.join(', '))
  return (
    <div className="panel" style={{ marginBottom: 10, borderColor: 'var(--amber)' }}>
      <div className="panel-header">编辑卡片 · {card.id.slice(-6)}</div>
      <div className="panel-body form-col">
        <label>正面</label>
        <textarea rows={2} value={front} onChange={(e) => setFront(e.target.value)} />
        <label>背面</label>
        <textarea rows={2} value={back} onChange={(e) => setBack(e.target.value)} />
        <label>标签（逗号分隔）</label>
        <input value={tags} onChange={(e) => setTags(e.target.value)} placeholder="rust, 基础" />
        <label>代码块（可选）</label>
        <textarea rows={3} value={codeBlock} onChange={(e) => setCodeBlock(e.target.value)} placeholder="fn main() {}" style={{ fontFamily: 'var(--font-flap)', fontSize: 12 }} />
        <label>图片 URL（逗号分隔）</label>
        <input value={imageUrls} onChange={(e) => setImageUrls(e.target.value)} placeholder="https://…" />
        <div className="form-row">
          <button onClick={async () => {
            await api.cards.update(card.id, {
              front, back,
              tags: tags.split(',').map((s) => s.trim()).filter(Boolean),
              code_block: codeBlock,
              image_urls: imageUrls.split(',').map((s) => s.trim()).filter(Boolean),
            })
            onSaved()
          }}>保存</button>
          <button className="ghost-btn" onClick={onClose}>取消</button>
        </div>
      </div>
    </div>
  )
}

function Stat({ label, value, accent }: { label: string; value: ReactNode; accent?: string }) {
  return (
    <div className={`stat-item ${accent || ''}`}>
      <div className="stat-value">{value}</div>
      <div className="stat-label">{label}</div>
    </div>
  )
}

// ═══════════════ ProgressView — 热力图 + 分布 + 会话 ═══════════════

function ProgressView({ dash }: { dash: Dashboard }) {
  const [sessions, setSessions] = useState<Session[]>([])
  const [heat, setHeat] = useState<HeatmapDay[]>([])
  useEffect(() => { api.sessions.list(10).then(setSessions); api.stats.heatmap(84).then(setHeat) }, [])
  const maxCount = Math.max(...dash.stats.by_topic.map((x) => x.count), 1)
  const heatMap = new Map(heat.map((h) => [h.date, h.count]))

  // 生成最近 12 周的日历格子
  const cells: { date: string; count: number; inFuture: boolean }[] = []
  const today = new Date()
  for (let i = 83; i >= 0; i--) {
    const d = new Date(today); d.setDate(d.getDate() - i)
    const iso = d.toISOString().slice(0, 10)
    cells.push({ date: iso, count: heatMap.get(iso) || 0, inFuture: i < 0 })
  }

  const level = (c: number) => c === 0 ? 0 : c === 1 ? 1 : c <= 3 ? 2 : c <= 6 ? 3 : 4

  return (
    <div>
      <div className="panel">
        <div className="panel-header">复习热力 · 最近 12 周</div>
        <div className="panel-body">
          <div className="heatmap">
            {cells.map((c) => (
              <div key={c.date} className={`heat-cell lvl-${level(c.count)}`} title={`${c.date} · ${c.count} 次复习`} />
            ))}
          </div>
          <div style={{ fontFamily: 'var(--font-flap)', fontSize: 11, color: 'var(--muted)', marginTop: 8, display: 'flex', gap: 4, alignItems: 'center' }}>
            少 <span className="heat-cell lvl-0" style={{ display: 'inline-block' }} /><span className="heat-cell lvl-1" style={{ display: 'inline-block' }} /><span className="heat-cell lvl-2" style={{ display: 'inline-block' }} /><span className="heat-cell lvl-3" style={{ display: 'inline-block' }} /><span className="heat-cell lvl-4" style={{ display: 'inline-block' }} /> 多
          </div>
        </div>
      </div>

      <div className="panel">
        <div className="panel-header">领域分布</div>
        <div className="panel-body">
          <div style={{ display: 'grid', gap: 6 }}>
            {dash.stats.by_topic.map((b) => (
              <div key={b.topic} style={{ display: 'flex', alignItems: 'center', gap: 10, fontFamily: 'var(--font-flap)', fontSize: 12 }}>
                <span style={{ width: 70 }}>{b.topic}</span>
                <div style={{ flex: 1, background: '#1a1a1a', height: 14, borderRadius: 2, overflow: 'hidden' }}>
                  <div style={{ width: `${(b.count / maxCount) * 100}%`, height: '100%', background: 'var(--amber)', borderRadius: 2 }} />
                </div>
                <span className="muted">{b.count}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="panel">
        <div className="panel-header">最近班次日志</div>
        <div className="panel-body" style={{ padding: 0 }}>
          <SessionTimeline sessions={sessions} />
        </div>
      </div>
    </div>
  )
}

// 会话时间轴：起止时间 + summary + 新建/复习计数。
export function SessionTimeline({ sessions }: { sessions: Session[] }) {
  if (sessions.length === 0) {
    return <div className="loading">暂无运行记录。</div>
  }
  return (
    <ul className="card-list" style={{ padding: '0 16px' }}>
      {sessions.map((s) => (
        <li key={s.id} className="card-item" style={{ flexDirection: 'column', alignItems: 'stretch', gap: 2 }}>
          <div style={{ display: 'flex', gap: 12, fontFamily: 'var(--font-flap)', fontSize: 11, color: 'var(--muted)' }}>
            <span>{new Date(s.started_at).toLocaleString('zh-CN')}</span>
            {s.ended_at && <span>▸ {new Date(s.ended_at).toLocaleTimeString('zh-CN')}</span>}
          </div>
          <div>{s.summary || '（无记录）'}</div>
          <div style={{ display: 'flex', gap: 16, fontFamily: 'var(--font-flap)', fontSize: 11, color: 'var(--muted)' }}>
            <span>新建 {s.new_cards}</span><span>复习 {s.reviewed}</span>
          </div>
        </li>
      ))}
    </ul>
  )
}

// ═══════════════ ProfileView ═══════════════

function ProfileView({ profile, onRefresh }: { profile: LearnerProfile | null; onRefresh: () => void }) {
  const [form, setForm] = useState<LearnerProfile>(
    profile || { id: 1, level: '', style: '', weak_points: [], preferences: {}, notes: '', updated: '' }
  )
  const [prevProfile, setPrevProfile] = useState(profile)
  if (profile !== prevProfile) {
    setPrevProfile(profile)
    if (profile) setForm(profile)
  }

  return (
    <div className="panel">
      <div className="panel-header">站务日志 / 学习画像</div>
      <div className="panel-body">
        <p className="muted" style={{ fontSize: 12, marginBottom: 12 }}>AI 积累的对你的认知（温和记忆）。帮助跨会话更懂你。</p>
        <div className="form-col">
          <label>水平定位</label>
          <input value={form.level} onChange={(e) => setForm({ ...form, level: e.target.value })} placeholder="例: Rust 入门，已掌握所有权" />
          <label>学习风格</label>
          <select value={form.style} onChange={(e) => setForm({ ...form, style: e.target.value })}>
            <option value="">未设</option><option>项目驱动</option><option>教材式</option><option>源码驱动</option>
          </select>
          <label>盲点（逗号分隔）</label>
          <input value={form.weak_points.join(', ')} onChange={(e) => setForm({ ...form, weak_points: e.target.value.split(',').map((s) => s.trim()).filter(Boolean) })} placeholder="例: 生命周期标注, async Pin" />
          <label>自由笔记</label>
          <textarea rows={4} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} />
          <button onClick={async () => { await api.profile.upsert(form); onRefresh() }}>保存日志</button>
        </div>
      </div>
    </div>
  )
}
