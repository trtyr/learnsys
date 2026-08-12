import { useEffect, useState, type ReactNode } from 'react'
import { api } from './api'
import type { Card, Dashboard } from './types'

export default function App() {
  const [dash, setDash] = useState<Dashboard | null>(null)
  const [due, setDue] = useState<Card[]>([])
  const [err, setErr] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    Promise.all([api.dashboard(), api.cards.due()])
      .then(([d, c]) => {
        setDash(d)
        setDue(c)
      })
      .catch((e) => setErr(String(e)))
      .finally(() => setLoading(false))
  }, [])

  if (err) return <Shell><div className="error">加载失败：{err}</div></Shell>
  if (loading || !dash) return <Shell><div className="muted">加载中…</div></Shell>

  const today = new Date().toISOString().slice(0, 10)
  const maxTopic = Math.max(...dash.stats.by_topic.map((x) => x.count), 1)

  return (
    <Shell>
      <header className="header">
        <h1>📚 recall <span className="muted">学习舱</span></h1>
        <p className="tagline">headless 学习数据平台 · 今日 {dash.due_today} 张待复习</p>
      </header>

      <section className="stats-grid">
        <Stat label="今日待复习" value={dash.due_today} accent={dash.due_today > 0 ? 'hot' : ''} />
        <Stat label="明后天到期" value={dash.due_soon} accent={dash.due_soon > 0 ? 'warn' : ''} />
        <Stat label="总卡片" value={dash.stats.total_cards} />
        <Stat label="平均 EF" value={dash.stats.avg_ef.toFixed(2)} />
      </section>

      <section className="panel">
        <h2>🔵 进行中主题</h2>
        {dash.active_topics.length === 0 ? (
          <p className="muted">暂无进行中主题</p>
        ) : (
          <div className="topics">
            {dash.active_topics.map((t) => (
              <div key={t.id} className="topic-card">
                <div className="topic-name">{t.name}</div>
                <div className="topic-stage">{t.stage || '（未设阶段）'}</div>
                {t.next_plan && <div className="topic-next">下一步：{t.next_plan}</div>}
              </div>
            ))}
          </div>
        )}
      </section>

      <section className="panel">
        <h2>🔔 今日待复习 <span className="count">{due.length}</span></h2>
        {due.length === 0 ? (
          <p className="muted">🎉 今天没有到期卡片，学点新的？</p>
        ) : (
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

      <footer className="footer muted">
        recall · AI 调 API 操作，平台只负责记录与调度 · 复习热力待 review_logs 积累后上线
      </footer>
    </Shell>
  )
}

function Shell({ children }: { children: ReactNode }) {
  return <div className="wrap">{children}</div>
}

function Stat({
  label,
  value,
  accent,
}: {
  label: string
  value: ReactNode
  accent?: string
}) {
  return (
    <div className={`stat-card ${accent || ''}`}>
      <div className="stat-value">{value}</div>
      <div className="stat-label">{label}</div>
    </div>
  )
}
