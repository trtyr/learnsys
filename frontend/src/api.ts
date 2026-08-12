import type { Card, Goal, LearnerProfile, Module, ModuleMastery, Pathway, PathwayModule, Session, Topic } from './types'

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

export const api = {
  cards: {
    due: (topic?: string) => get<Card[]>(`/cards/due${topic ? `?topic=${encodeURIComponent(topic)}` : ''}`),
    list: (topic?: string) => get<Card[]>(`/cards${topic ? `?topic=${encodeURIComponent(topic)}` : ''}`),
  },
  topics: { list: () => get<Topic[]>('/topics') },
  stats: () => get<import('./types').Stats>('/stats'),
  dashboard: () => get<import('./types').Dashboard>('/dashboard'),

  goals: {
    list: () => get<Goal[]>('/goals'),
    create: (body: { title: string; description?: string; success_criteria?: string; topic?: string }) =>
      post<Goal>('/goals', body),
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
  },
  modules: {
    list: (topic?: string) => get<Module[]>(`/modules${topic ? `?topic=${encodeURIComponent(topic)}` : ''}`),
    create: (body: { title: string; topic?: string; description?: string }) => post<Module>('/modules', body),
    mastery: (id: string) => get<ModuleMastery>(`/modules/${id}/mastery`),
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
