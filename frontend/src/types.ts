// 与后端 recall-core::entity / repo 对齐的类型。
// 日期字段为 ISO 字符串（YYYY-MM-DD 或 RFC3339）。

export interface Card {
  id: string
  topic: string // API 对外用 topic 名（见 name_topic）
  front: string
  back: string
  ef: number
  interval: number
  reps: number
  due: string
  created: string
  updated: string
}

export type TopicStatus = 'active' | 'completed' | 'paused'

export interface Topic {
  id: string
  name: string
  stage: string
  status: TopicStatus
  last_studied: string | null
  next_plan: string
  created: string
}

export interface TopicCount {
  topic: string
  count: number
}

export interface Stats {
  total_cards: number
  due_today: number
  due_soon: number
  avg_ef: number
  by_topic: TopicCount[]
}

export interface Dashboard {
  due_today: number
  due_soon: number
  active_topics: Topic[]
  stats: Stats
}
