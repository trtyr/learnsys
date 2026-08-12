import type { Card, Topic, Stats, Dashboard } from './types'

const BASE = '/api'

async function get<T>(path: string): Promise<T> {
  const r = await fetch(`${BASE}${path}`)
  if (!r.ok) throw new Error(`${r.status} ${r.statusText}`)
  return r.json()
}

function qs(topic?: string): string {
  return topic ? `?topic=${encodeURIComponent(topic)}` : ''
}

export const api = {
  cards: {
    due: (topic?: string) => get<Card[]>(`/cards/due${qs(topic)}`),
    list: (topic?: string) => get<Card[]>(`/cards${qs(topic)}`),
  },
  topics: {
    list: () => get<Topic[]>('/topics'),
  },
  stats: () => get<Stats>('/stats'),
  dashboard: () => get<Dashboard>('/dashboard'),
}
