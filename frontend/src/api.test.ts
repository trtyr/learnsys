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

  it('cards.search(q, topic) 命中 /api/cards/search', async () => {
    vi.mocked(fetch).mockResolvedValue(ok([]))
    await api.cards.search('borrow', 'rust')
    expect(fetch).toHaveBeenCalledWith('/api/cards/search?q=borrow&topic=rust')
  })

  it('cards.update 走 PUT 且带补丁字段', async () => {
    vi.mocked(fetch).mockResolvedValue(ok({}))
    await api.cards.update('c1', { front: 'q', tags: ['rust'], code_block: 'fn', image_urls: ['u'] })
    expect(fetch).toHaveBeenCalledWith('/api/cards/c1', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ front: 'q', tags: ['rust'], code_block: 'fn', image_urls: ['u'] }),
    })
  })

  it('cards.new / cards.leeches / quiz 命中各自端点', async () => {
    vi.mocked(fetch).mockResolvedValue(ok([]))
    await api.cards.new()
    expect(fetch).toHaveBeenCalledWith('/api/cards/new')
    await api.cards.leeches()
    expect(fetch).toHaveBeenCalledWith('/api/cards/leeches')
    await api.quiz(5, 'rust')
    expect(fetch).toHaveBeenCalledWith('/api/quiz?n=5&topic=rust')
  })

  it('settings.get / settings.put 读写 new_per_day', async () => {
    vi.mocked(fetch).mockResolvedValue(ok({ new_per_day: 5 }))
    await api.settings.get()
    expect(fetch).toHaveBeenCalledWith('/api/settings')
    await api.settings.put({ new_per_day: 3 })
    expect(fetch).toHaveBeenCalledWith('/api/settings', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ new_per_day: 3 }),
    })
  })
})
