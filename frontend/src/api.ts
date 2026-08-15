import type { Card, Goal, GoalProgress, HeatmapDay, LearnerProfile, Module, ModuleMastery, Pathway, PathwayModule, Resource, Session, TimelineEvent, Topic, UpcomingDay, WeakTopic } from './types'

const BASE = '/api'

async function get<T>(path: string): Promise<T> {
  const r = await fetch(`${BASE}${path}`)
  if (!r.ok) throw new Error(`${r.status}`)
  return r.json()
}
async function post<T>(path: string, body: unknown): Promise<T> {
  const r = await fetch(`${BASE}${path}`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!r.ok) throw new Error(`${r.status}`)
  if (r.status === 204) return undefined as T
  return r.json()
}
async function put<T>(path: string, body: unknown): Promise<T> {
  const r = await fetch(`${BASE}${path}`, {
    method: 'PUT', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!r.ok) throw new Error(`${r.status}`)
  if (r.status === 204) return undefined as T
  return r.json()
}
async function del(path: string): Promise<void> {
  const r = await fetch(`${BASE}${path}`, { method: 'DELETE' })
  if (!r.ok) throw new Error(`${r.status}`)
}

export const api = {
  cards: {
    due: (topic?: string) => get<Card[]>(`/cards/due${topic ? `?topic=${encodeURIComponent(topic)}` : ''}`),
    list: (topic?: string) => get<Card[]>(`/cards${topic ? `?topic=${encodeURIComponent(topic)}` : ''}`),
    get: (id: string) => get<Card>(`/cards/${id}`),
    review: (id: string, quality: number) => post<Card>(`/cards/${id}/review`, { quality }),
    create: (body: { topic: string; front: string; back: string; tags?: string[]; code_block?: string; image_urls?: string[]; source?: string; related?: string[] }) => post<Card>('/cards', body),
    update: (id: string, patch: { front?: string; back?: string; topic?: string; tags?: string[]; code_block?: string; image_urls?: string[]; module_id?: string; source?: string; related?: string[] }) =>
      put<Card>(`/cards/${id}`, patch),
    search: (q: string, topic?: string) =>
      get<Card[]>(`/cards/search?q=${encodeURIComponent(q)}${topic ? `&topic=${encodeURIComponent(topic)}` : ''}`),
    new: (limit?: number) => get<Card[]>(`/cards/new${limit ? `?limit=${limit}` : ''}`),
    leeches: () => get<Card[]>('/cards/leeches'),
  },
  settings: {
    get: () => get<{ new_per_day: number }>('/settings'),
    put: (body: { new_per_day?: number }) => put<void>('/settings', body),
  },
  quiz: (n?: number, topic?: string) =>
    get<Card[]>(`/quiz?n=${n ?? 5}${topic ? `&topic=${encodeURIComponent(topic)}` : ''}`),
  timeline: () => get<TimelineEvent[]>('/timeline'),
  topics: { list: () => get<Topic[]>('/topics') },
  dashboard: () => get<import('./types').Dashboard>('/dashboard'),

  goals: {
    list: () => get<Goal[]>('/goals'),
    create: (body: { title: string; description?: string; success_criteria?: string; topic?: string }) =>
      post<Goal>('/goals', body),
    progress: (id: string) => get<GoalProgress>(`/goals/${id}/progress`),
    update: (id: string, patch: { title?: string; description?: string; success_criteria?: string }) =>
      put<Goal>(`/goals/${id}`, patch),
    delete: (id: string) => del(`/goals/${id}`),
  },
  pathways: {
    listByGoal: (goalId: string) => get<Pathway[]>(`/pathways?goal=${encodeURIComponent(goalId)}`),
    create: (body: { name: string; goal_id: string; methodology?: string; description?: string }) =>
      post<Pathway>('/pathways', body),
    modules: (pathwayId: string) => get<PathwayModule[]>(`/pathways/${pathwayId}/modules`),
    addModule: (pathwayId: string, body: { module_id: string; sort_order: number; depends_on?: string[] }) =>
      post<PathwayModule>(`/pathways/${pathwayId}/modules`, body),
    next: (pathwayId: string) =>
      get<{ module?: Module; position?: number; total?: number; done?: boolean }>(`/pathways/${pathwayId}/next`),
    update: (id: string, patch: { name?: string; methodology?: string; description?: string }) =>
      put<Pathway>(`/pathways/${id}`, patch),
    delete: (id: string) => del(`/pathways/${id}`),
  },
  modules: {
    list: (topic?: string) => get<Module[]>(`/modules${topic ? `?topic=${encodeURIComponent(topic)}` : ''}`),
    create: (body: { title: string; topic?: string; description?: string }) => post<Module>('/modules', body),
    mastery: (id: string) => get<ModuleMastery>(`/modules/${id}/mastery`),
    updateStatus: (id: string, status: string) => put<void>(`/modules/${id}/status`, { status }),
    update: (id: string, patch: { title?: string; description?: string }) => put<Module>(`/modules/${id}`, patch),
    delete: (id: string) => del(`/modules/${id}`),
    cards: (id: string) => get<Card[]>(`/modules/${id}/cards`),
  },
  resources: {
    list: (moduleId?: string) => get<Resource[]>(`/resources${moduleId ? `?module_id=${encodeURIComponent(moduleId)}` : ''}`),
    create: (body: { title: string; url?: string; notes?: string; module_id?: string }) => post<Resource>('/resources', body),
  },
  stats: {
    heatmap: (days?: number) => get<HeatmapDay[]>(`/stats/heatmap${days ? `?days=${days}` : ''}`),
    upcoming: (days?: number) => get<UpcomingDay[]>(`/stats/upcoming?days=${days ?? 7}`),
    weakTopics: () => get<WeakTopic[]>('/stats/weak-topics'),
  },
  sessions: {
    start: (body?: { goal_id?: string; pathway_id?: string }) => post<Session>('/sessions/start', body || {}),
    end: (id: number, body: { summary?: string; new_cards?: number; reviewed?: number }) =>
      post<void>(`/sessions/${id}/end`, body),
    list: (limit?: number) => get<Session[]>(`/sessions${limit ? `?limit=${limit}` : ''}`),
  },
  profile: {
    get: () => get<LearnerProfile>('/profile').catch(() => null),
    upsert: (body: LearnerProfile) => put<void>('/profile', body),
  },
}
