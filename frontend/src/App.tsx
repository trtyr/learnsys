import { useCallback, useEffect, useState, type ReactNode } from 'react'
import { api } from './api'
import type { Card, Dashboard, Goal, LearnerProfile, Module, Pathway, PathwayModule, Session } from './types'

type Tab = 'plan' | 'review' | 'progress' | 'profile'

export default function App() {
  const [tab, setTab] = useState<Tab>('plan')
  const [dash, setDash] = useState<Dashboard | null>(null)
  const [profile, setProfile] = useState<LearnerProfile | null>(null)
  const [loading, setLoading] = useState(true)
  const [err, setErr] = useState<string | null>(null)

  const reload = useCallback(() => {
    setLoading(true)
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
          {dash.due_today > 0 && <span className="warn">◆ {dash.due_today} 待出发</span>}
          {dash.due_soon > 0 && <span className="hot">◈ {dash.due_soon} 延误</span>}
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
        {tab === 'review' && <ReviewView dash={dash} />}
        {tab === 'progress' && <ProgressView dash={dash} />}
        {tab === 'profile' && <ProfileView profile={profile} onRefresh={reload} />}
      </main>

      <footer className="footer">
        知识出发板 · AI 调 API 操作，平台负责记录与调度 · today {today}
      </footer>
    </Shell>
  )
}

function Shell({ children }: { children: ReactNode }) {
  return <div className="wrap">{children}</div>
}

// ═════════════════ PlanView ═════════════════

function PlanView({ onRefresh }: { onRefresh: () => void }) {
  const [goals, setGoals] = useState<Goal[]>([])
  const [expand, setExpand] = useState<string | null>(null)
  const [newTitle, setNewTitle] = useState('')
  const [method, setMethod] = useState('基础优先')
  const load = useCallback(() => { api.goals.list().then(setGoals) }, [])
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
      {goals.map((g) => (
        <div key={g.id} className="panel">
          <div className="goal-row" onClick={() => setExpand(expand === g.id ? null : g.id)}>
            <span className="tag" style={{ background: g.status === 'active' ? 'rgba(245,166,35,.15)' : g.status === 'achieved' ? 'rgba(90,158,111,.15)' : 'rgba(102,102,102,.15)', color: g.status === 'active' ? 'var(--amber)' : g.status === 'achieved' ? 'var(--green)' : 'var(--muted)' }}>{g.status}</span>
            <b>{g.title}</b>
            {g.success_criteria && <span className="muted">— {g.success_criteria}</span>}
            <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-flap)', color: 'var(--muted)' }}>{expand === g.id ? '▾' : '▸'}</span>
          </div>
          {expand === g.id && <GoalDetail gid={g.id} onRefresh={onRefresh} />}
        </div>
      ))}
    </div>
  )
}

function GoalDetail({ gid, onRefresh }: { gid: string; onRefresh: () => void }) {
  const [pws, setPws] = useState<Pathway[]>([])
  const [sel, setSel] = useState<string | null>(null)
  const [allMods, setAllMods] = useState<Module[]>([])
  const [mods, setMods] = useState<Record<string, { pms: PathwayModule[]; next: { done?: boolean; module?: Module; position?: number; total?: number } | null }>>({})
  const [title, setTitle] = useState('')
  const [order, setOrder] = useState(1)

  useEffect(() => {
    api.pathways.listByGoal(gid).then(setPws)
    api.modules.list().then(setAllMods)
  }, [gid])

  return (
    <div className="panel-body" style={{ borderTop: '1px solid var(--border)' }}>
      {pws.map((pw) => {
        const d = mods[pw.id] || { pms: [], next: null }
        return (
          <div key={pw.id}>
            <div className="pathway-row" onClick={() => {
              if (sel === pw.id) { setSel(null); return }
              setSel(pw.id)
              if (!mods[pw.id]) {
                api.pathways.modules(pw.id).then((pms) => {
                  api.pathways.next(pw.id).then((n) => setMods((m) => ({ ...m, [pw.id]: { pms, next: n } }))).catch(() => {})
                })
              }
            }}>
              <span className="tag" style={{ background: 'rgba(192,192,192,.12)', color: 'var(--accent)' }}>{pw.methodology || '路径'}</span>
              <b>{pw.name}</b>
              <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-flap)', color: 'var(--muted)' }}>{sel === pw.id ? '▾' : '▸'}</span>
            </div>
            {sel === pw.id && (
              <div>
                {d.pms.length === 0 && <div className="muted" style={{ padding: '8px 0', fontSize: 12 }}>还没模块。</div>}
                {d.pms.map((pm, i) => {
                  const mod = allMods.find((m) => m.id === pm.module_id)
                  return (
                    <div key={pm.module_id} className="module-row">
                      <span className="idx">{i + 1}</span>
                      <span className="title">{mod?.title || pm.module_id.slice(0, 12)}</span>
                      <span className="tag" style={{
                        background: mod?.status === 'mastered' ? 'rgba(90,158,111,.15)' : mod?.status === 'learning' ? 'rgba(245,166,35,.15)' : 'rgba(102,102,102,.1)',
                        color: mod?.status === 'mastered' ? 'var(--green)' : mod?.status === 'learning' ? 'var(--amber)' : 'var(--muted)'
                      }}>{mod?.status || '?'}</span>
                      {pm.depends_on.length > 0 && (
                        <span className="depends">◂ {pm.depends_on.map((did) => allMods.find((m) => m.id === did)?.title || did.slice(0, 8)).join(' · ')}</span>
                      )}
                    </div>
                  )
                })}
                {d.next?.done && <div style={{ padding: '8px 0 8px 24px', color: 'var(--green)', fontFamily: 'var(--font-flap)', fontSize: 12 }}>◆ 全线完成</div>}
                {d.next?.module && !d.next.done && (
                  <div style={{ padding: '8px 0 8px 24px', color: 'var(--amber)', fontFamily: 'var(--font-flap)', fontSize: 12 }}>▶ 下一站: {d.next.module.title} ({d.next.position}/{d.next.total})</div>
                )}
                <div className="form-row" style={{ marginTop: 8, marginLeft: 24 }}>
                  <input placeholder="模块名" value={title} onChange={(e) => setTitle(e.target.value)} style={{ flex: 2 }} />
                  <input placeholder="#" value={order} type="number" style={{ width: 56 }} onChange={(e) => setOrder(Number(e.target.value))} />
                  <button onClick={async () => {
                    if (!title) return
                    const m = await api.modules.create({ title })
                    await api.pathways.addModule(pw.id, { module_id: m.id, sort_order: order })
                    setTitle(''); setOrder(order + 1); onRefresh()
                    api.pathways.modules(pw.id).then(async (pms) => {
                      const n = await api.pathways.next(pw.id).catch(() => null)
                      setMods((x) => ({ ...x, [pw.id]: { pms, next: n } }))
                    })
                    api.modules.list().then(setAllMods)
                  }}>+ 模块</button>
                </div>
              </div>
            )}
          </div>
        )
      })}
      <button onClick={async () => {
        await api.pathways.create({ name: '新路径', goal_id: gid })
        api.pathways.listByGoal(gid).then(setPws); onRefresh()
      }} style={{
        marginTop: 8, padding: '6px 14px', background: 'var(--border)', border: '1px solid var(--border-light)',
        color: 'var(--text)', fontFamily: 'var(--font-flap)', fontSize: 12, letterSpacing: '.05em', textTransform: 'uppercase', cursor: 'pointer'
      }}>+ 新路径</button>
    </div>
  )
}

// ═══════════════ ReviewView — THE departure board ═══════════════

function ReviewView({ dash }: { dash: Dashboard }) {
  const [due, setDue] = useState<Card[]>([])
  useEffect(() => { api.cards.due().then(setDue) }, [])
  const today = new Date().toISOString().slice(0, 10)

  return (
    <div>
      <div className="stats-strip">
        <Stat label="待出发" value={dash.due_today} accent={dash.due_today > 0 ? 'warn' : ''} />
        <Stat label="延误" value={dash.due_soon} accent={dash.due_soon > 0 ? 'hot' : ''} />
        <Stat label="总计" value={dash.stats.total_cards} />
        <Stat label="平均 EF" value={dash.stats.avg_ef.toFixed(2)} />
      </div>

      <div className="panel">
        <div className="panel-header">出发板 · 今日待复习 {due.length} 班</div>
        <div className="panel-body" style={{ padding: 0 }}>
          {due.length === 0 ? (
            <div className="loading">◆ 今日无出发——全线清空。</div>
          ) : (
            <table className="board">
              <thead>
                <tr>
                  <th>到期</th><th>班次 / 标题</th><th>状态</th><th>领域</th><th>EF</th>
                </tr>
              </thead>
              <tbody>
                {due.map((c) => {
                  const days = Math.floor((Date.parse(today) - Date.parse(c.due)) / 86400000)
                  const cls = days > 1 ? 'extreme' : days > 0 ? 'overdue' : 'on-time'
                  const status = days > 1 ? `延误 ${days}天` : days > 0 ? '已延误' : '准时'
                  const tagCls = days > 1 ? 'red' : days > 0 ? 'amber' : 'green'
                  return (
                    <tr key={c.id} className={cls}>
                      <td className="cell-due">{c.due.slice(5)}</td>
                      <td className="cell-module">{c.front}</td>
                      <td className="cell-status"><span className={`tag ${tagCls}`}>{status}</span></td>
                      <td className="cell-topic"><span className="tag muted">{c.topic}</span></td>
                      <td className="cell-topic muted" style={{ fontSize: 12 }}>{c.ef.toFixed(1)}</td>
                    </tr>
                  )
                })}
              </tbody>
            </table>
          )}
        </div>
        <div className="status-bar" style={{ padding: '8px 16px' }}>
          <span>已出发 <b>{dash.stats.total_cards - dash.due_today}</b></span>
          <span>待出发 <b className="warn">{dash.due_today}</b></span>
          {dash.due_soon > 0 && <span>延误预警 <b className="hot">{dash.due_soon}</b></span>}
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

// ═══════════════ ProgressView ═══════════════

function ProgressView({ dash }: { dash: Dashboard }) {
  const [sessions, setSessions] = useState<Session[]>([])
  useEffect(() => { api.sessions.list(10).then(setSessions) }, [])
  const maxCount = Math.max(...dash.stats.by_topic.map((x) => x.count), 1)

  return (
    <div>
      <div className="panel">
        <div className="panel-header">领域分布</div>
        <div className="panel-body">
          <div style={{ display: 'grid', gap: 6 }}>
            {dash.stats.by_topic.map((b) => (
              <div key={b.topic} style={{ display: 'flex', alignItems: 'center', gap: 10, fontFamily: 'var(--font-flap)', fontSize: 12 }}>
                <span style={{ width: 70, color: 'var(--text)' }}>{b.topic}</span>
                <div style={{ flex: 1, background: '#1a1a1a', height: 14, borderRadius: 2, overflow: 'hidden' }}>
                  <div style={{ width: `${(b.count / maxCount) * 100}%`, height: '100%', background: 'var(--amber)', borderRadius: 2, transition: 'width .4s' }} />
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
          {sessions.length === 0 ? (
            <div className="loading">暂无运行记录。</div>
          ) : (
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
          )}
        </div>
      </div>
    </div>
  )
}

// ═══════════════ ProfileView ═══════════════

function ProfileView({ profile, onRefresh }: { profile: LearnerProfile | null; onRefresh: () => void }) {
  const [form, setForm] = useState<LearnerProfile>(
    profile || { id: 1, level: '', style: '', weak_points: [], preferences: {}, notes: '', updated: '' }
  )
  useEffect(() => { if (profile) setForm(profile) }, [profile])

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
