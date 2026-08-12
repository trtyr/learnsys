import { useCallback, useEffect, useState, type ReactNode } from 'react'
import { api } from './api'
import type { Dashboard, Goal, LearnerProfile, Module, ModuleMastery, Pathway, PathwayModule, Session } from './types'

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

  if (err) return <Shell>加载失败: {err}</Shell>
  if (loading || !dash) return <Shell>加载中…</Shell>

  return (
    <Shell>
      <header className="header">
        <h1>📚 学习系统 <span className="muted">学习管理</span></h1>
        <div className="tagline">
          {profile?.level && <span>{profile.level} · </span>}
          {dash.due_today > 0
            ? <>🔔 {dash.due_today} 张待复习</>
            : <>🎉 今日无到期</>}
        </div>
      </header>

      <nav className="tabs">
        {(['plan', 'review', 'progress', 'profile'] as Tab[]).map((t) => (
          <button key={t} className={`tab ${tab === t ? 'active' : ''}`} onClick={() => setTab(t)}>
            {{ plan: '📋 计划', review: '🔔 复习', progress: '📊 进度', profile: '🧠 画像' }[t]}
          </button>
        ))}
      </nav>

      <main className="main">
        {tab === 'plan' && <PlanView onRefresh={reload} />}
        {tab === 'review' && <ReviewView dash={dash} />}
        {tab === 'progress' && <ProgressView dash={dash} />}
        {tab === 'profile' && <ProfileView profile={profile} onRefresh={reload} />}
      </main>

      <footer className="footer muted">
        学习系统 · AI 调 API 操作，平台负责记录与调度
      </footer>
    </Shell>
  )
}

function Shell({ children }: { children: ReactNode }) {
  return <div className="wrap">{children}</div>
}

// ═══════════════════ PlanView ═══════════════════

function PlanView({ onRefresh }: { onRefresh: () => void }) {
  const [goals, setGoals] = useState<Goal[]>([])
  const [pathways, setPathways] = useState<Record<string, Pathway[]>>({})
  const [expand, setExpand] = useState<string | null>(null)
  const [newTitle, setNewTitle] = useState('')
  const [newMethodology, setNewMethodology] = useState('基础优先')

  const load = useCallback(() => {
    api.goals.list().then(setGoals)
  }, [])
  useEffect(load, [load])

  const toggle = (gid: string) => {
    if (expand === gid) { setExpand(null); return }
    setExpand(gid)
    if (!pathways[gid]) {
      api.pathways.listByGoal(gid).then((ps) => setPathways((p) => ({ ...p, [gid]: ps })))
    }
  }

  return (
    <div>
      <div className="form-row">
        <input placeholder="新建学习目标…" value={newTitle} onChange={(e) => setNewTitle(e.target.value)} />
        <select value={newMethodology} onChange={(e) => setNewMethodology(e.target.value)}>
          <option>基础优先</option><option>项目驱动</option><option>源码驱动</option><option>问题驱动</option>
        </select>
        <button onClick={async () => {
          if (!newTitle) return
          const g = await api.goals.create({ title: newTitle, success_criteria: '' })
          await api.pathways.create({ name: newTitle + ' · ' + newMethodology, goal_id: g.id, methodology: newMethodology })
          setNewTitle('')
          load()
          onRefresh()
        }}>+ 目标</button>
      </div>

      {goals.length === 0 && <p className="muted">还没学习目标，先建一个。</p>}
      {goals.map((g) => (
        <div key={g.id} className="panel">
          <div className="goal-head" onClick={() => toggle(g.id)}>
            <span className={`status-dot ${g.status}`} />
            <strong>{g.title}</strong>
            <span className="muted" style={{ marginLeft: 8 }}>{g.status}</span>
            {g.success_criteria && <span className="tag">{g.success_criteria}</span>}
            <span style={{ marginLeft: 'auto' }}>{expand === g.id ? '▾' : '▸'}</span>
          </div>
          {expand === g.id && <GoalDetail gid={g.id} pws={pathways[g.id] || []} onRefresh={onRefresh} />}
        </div>
      ))}
    </div>
  )
}

function GoalDetail({ gid, pws, onRefresh }: { gid: string; pws: Pathway[]; onRefresh: () => void }) {
  const [sel, setSel] = useState<string | null>(null)
  const [mods, setMods] = useState<Record<string, { pms: PathwayModule[]; modules: Module[]; next: { done?: boolean; module?: Module; position?: number; total?: number } | null }>>({})
  const [title, setTitle] = useState('')
  const [order, setOrder] = useState(1)

  return (
    <div style={{ marginTop: 10, paddingLeft: 12, borderLeft: '2px solid var(--border)' }}>
      {pws.length === 0 && <p className="muted">还没建路径。</p>}
      {pws.map((pw) => {
        const d = mods[pw.id] || { pms: [], modules: [], next: null }
        return (
          <div key={pw.id} style={{ marginBottom: 10 }}>
            <div className="pathway-head" onClick={() => {
              if (sel === pw.id) { setSel(null); return }
              setSel(pw.id)
              api.pathways.modules(pw.id).then(async (pms) => {
                const ms = await Promise.all(pms.map((pm) => api.modules.list().then((all) => all.find((m) => m.id === pm.module_id)!)))
                const n = await api.pathways.next(pw.id).catch(() => null)
                setMods((m) => ({ ...m, [pw.id]: { pms, modules: ms, next: n } }))
              })
            }}>
              <b>{pw.name}</b> <span className="tag">{pw.methodology}</span>
              <span style={{ marginLeft: 'auto' }}>{sel === pw.id ? '▾' : '▸'}</span>
            </div>
            {sel === pw.id && (
              <div style={{ paddingLeft: 16 }}>
                {d.pms.map((pm, i) => (
                  <div key={pm.module_id} className="module-row">
                    <span>{i + 1}.</span>
                    <span>{d.modules[i]?.title || pm.module_id.slice(0, 10)}</span>
                    <span className={`status-tag ${d.modules[i]?.status || 'not_started'}`}>{d.modules[i]?.status || '?'}</span>
                    {pm.depends_on.length > 0 && <span className="muted" style={{ fontSize: 11 }}>前置: {pm.depends_on.map((did) => d.modules.find((m) => m?.id === did)?.title || did.slice(0, 8)).join(', ')}</span>}
                  </div>
                ))}
                {d.next?.done && <p className="muted">🎉 路径完成！</p>}
                {d.next?.module && !d.next.done && <p style={{ color: 'var(--accent)' }}>▶ 下一个: {d.next.module.title}（{d.next.position}/{d.next.total}）</p>}

                <div className="form-row" style={{ marginTop: 8 }}>
                  <input placeholder="模块名" value={title} onChange={(e) => setTitle(e.target.value)} />
                  <input placeholder="序号" value={order} type="number" style={{ width: 60 }} onChange={(e) => setOrder(Number(e.target.value))} />
                  <button onClick={async () => {
                    if (!title) return
                    const m = await api.modules.create({ title, topic: 'rust' })
                    await api.pathways.addModule(pw.id, { module_id: m.id, sort_order: order })
                    setTitle(''); setOrder(order + 1)
                    onRefresh()
                    // refresh detail
                    api.pathways.modules(pw.id).then(async (pms) => { /* reload */ })
                  }}>+ 模块</button>
                </div>
              </div>
            )}
          </div>
        )
      })}
      <button onClick={async () => {
        await api.pathways.create({ name: '新路径', goal_id: gid })
        onRefresh()
      }} style={{ fontSize: 12 }}>+ 新路径</button>
    </div>
  )
}

// ═══════════════════ ReviewView ═══════════════════

function ReviewView({ dash }: { dash: Dashboard }) {
  const [due, setDue] = useState<Card[]>([])
  useEffect(() => { api.cards.due().then(setDue) }, [])

  const today = new Date().toISOString().slice(0, 10)
  return (
    <div>
      <section className="stats-grid">
        <Stat label="今日待复习" value={dash.due_today} accent={dash.due_today > 0 ? 'hot' : ''} />
        <Stat label="明后天到期" value={dash.due_soon} accent={dash.due_soon > 0 ? 'warn' : ''} />
        <Stat label="总卡片" value={dash.stats.total_cards} />
        <Stat label="平均 EF" value={dash.stats.avg_ef.toFixed(2)} />
      </section>

      <section className="panel">
        <h2>🔔 今日待复习 <span className="count">{due.length}</span></h2>
        {due.length === 0 ? <p className="muted">🎉 今天没有到期卡片</p> : (
          <ul className="card-list">
            {due.map((c) => {
              const overdue = c.due < today
              return (
                <li key={c.id} className="card-item">
                  <span className="dot">{overdue ? '🔴' : '🟢'}</span>
                  <div className="card-body">
                    <div className="card-front">{c.front}</div>
                    <div className="card-meta">
                      <span className="tag">{c.topic}</span>
                      <span>到期 {c.due}{overdue ? '（逾期）' : ''}</span>
                      <span>EF {c.ef}</span>
                    </div>
                  </div>
                </li>
              )
            })}
          </ul>
        )}
      </section>
    </div>
  )
}

function Stat({ label, value, accent }: { label: string; value: ReactNode; accent?: string }) {
  return (
    <div className={`stat-card ${accent || ''}`}>
      <div className="stat-value">{value}</div>
      <div className="stat-label">{label}</div>
    </div>
  )
}

// ═══════════════════ ProgressView ═══════════════════

function ProgressView({ dash }: { dash: Dashboard }) {
  const maxTopic = Math.max(...dash.stats.by_topic.map((x) => x.count), 1)
  const [sessions, setSessions] = useState<Session[]>([])
  useEffect(() => { api.sessions.list(10).then(setSessions) }, [])

  return (
    <div>
      <section className="panel">
        <h2>📊 主题分布</h2>
        <div className="bars">
          {dash.stats.by_topic.map((b) => (
            <div key={b.topic} className="bar-row">
              <span className="bar-label">{b.topic}</span>
              <div className="bar-track">
                <div className="bar-fill" style={{ width: `${(b.count / maxTopic) * 100}%` }} />
              </div>
              <span className="bar-count">{b.count}</span>
            </div>
          ))}
        </div>
      </section>
      <section className="panel">
        <h2>📝 最近学习会话</h2>
        {sessions.length === 0 ? <p className="muted">暂无会话记录</p> : (
          <ul className="card-list">
            {sessions.map((s) => (
              <li key={s.id} className="card-item" style={{ flexDirection: 'column', alignItems: 'stretch' }}>
                <div className="card-meta">
                  <span>{new Date(s.started_at).toLocaleString('zh-CN')}</span>
                  {s.ended_at && <span>→ {new Date(s.ended_at).toLocaleTimeString('zh-CN')}</span>}
                </div>
                <div>{s.summary || '（无记录）'}</div>
                <div className="card-meta">
                  <span>新建 {s.new_cards}</span><span>复习 {s.reviewed}</span>
                </div>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  )
}

// ═══════════════════ ProfileView ═══════════════════

function ProfileView({ profile, onRefresh }: { profile: LearnerProfile | null; onRefresh: () => void }) {
  const [form, setForm] = useState<LearnerProfile>(profile || { id: 1, level: '', style: '', weak_points: [], preferences: {}, notes: '', updated: '' })

  useEffect(() => { if (profile) setForm(profile) }, [profile])

  return (
    <div>
      <section className="panel">
        <h2>🧠 学习者画像（温和 AI 记忆）</h2>
        <p className="muted">这些数据帮助 AI 跨会话更懂你。</p>
        <div className="form-col">
          <label>水平定位</label>
          <input value={form.level} onChange={(e) => setForm({ ...form, level: e.target.value })} placeholder="例：Rust 入门，已掌握所有权" />
          <label>学习风格</label>
          <select value={form.style} onChange={(e) => setForm({ ...form, style: e.target.value })}>
            <option value="">未设</option><option>项目驱动</option><option>教材式</option><option>源码驱动</option>
          </select>
          <label>盲点（逗号分隔）</label>
          <input value={form.weak_points.join(', ')} onChange={(e) => setForm({ ...form, weak_points: e.target.value.split(',').map((s) => s.trim()).filter(Boolean) })} placeholder="例：生命周期标注、async Pin" />
          <label>自由笔记（AI 记录的关键认知）</label>
          <textarea rows={4} value={form.notes} onChange={(e) => setForm({ ...form, notes: e.target.value })} />
          <button onClick={async () => {
            await api.profile.upsert(form)
            onRefresh()
          }}>💾 保存画像</button>
        </div>
      </section>
    </div>
  )
}
