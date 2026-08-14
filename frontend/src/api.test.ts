import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { api } from './api'

const ok = (data: unknown) =>
  ({ ok: true, status: 200, json: async () => data }) as Response

beforeEach(() => {
  vi.stubGlobal('fetch', vi.fn())
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('api client 契约', () => {
  it('cards.due(topic) 命中 /api/cards/due?topic=<encoded>', async () => {
    vi.mocked(fetch).mockResolvedValue(ok([]))
    await api.cards.due('rust 基础')
    expect(fetch).toHaveBeenCalledWith('/api/cards/due?topic=rust%20%E5%9F%BA%E7%A1%80')
  })

  it('cards.review 走 POST 且 body 带 quality', async () => {
    vi.mocked(fetch).mockResolvedValue(ok({}))
    await api.cards.review('abc', 5)
    expect(fetch).toHaveBeenCalledWith('/api/cards/abc/review', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ quality: 5 }),
    })
  })
})
