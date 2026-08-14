// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { CardEditor, CardRow, ReminderBadges, SessionTimeline } from './App'
import { api } from './api'
import type { Card, Dashboard, Session } from './types'

const baseCard: Card = {
  id: 'c1', topic: 'rust', front: 'q', back: 'a',
  ef: 2.5, interval: 0, reps: 0,
  due: '2026-08-14', created: '2026-08-14', updated: '2026-08-14T00:00:00Z',
  module_id: null, tags: ['rust'], code_block: 'fn main() {}', image_urls: ['https://e.com/a.png'],
}

beforeEach(() => {
  vi.restoreAllMocks()
})

afterEach(cleanup)

describe('CardEditor', () => {
  it('保存时提交全部内容字段（tags/code_block/image_urls）', async () => {
    const update = vi.spyOn(api.cards, 'update').mockResolvedValue({} as Card)
    render(<CardEditor card={baseCard} onClose={() => {}} onSaved={() => {}} />)
    expect(screen.getByPlaceholderText('https://…')).toBeTruthy()
    fireEvent.click(screen.getByText('保存'))
    await vi.waitFor(() => expect(update).toHaveBeenCalled())
    expect(update).toHaveBeenCalledWith('c1', expect.objectContaining({
      tags: ['rust'],
      code_block: 'fn main() {}',
      image_urls: ['https://e.com/a.png'],
    }))
  })
})

describe('CardRow', () => {
  it('渲染标签、代码块和图片', () => {
    const { container } = render(<CardRow c={baseCard} today="2026-08-14" flipped={false} onFlip={() => {}} onEdit={() => {}} onReview={() => {}} showStatus={false} />)
    expect(screen.getByText('#rust')).toBeTruthy()
    expect(screen.getByText('fn main() {}')).toBeTruthy()
    expect(container.querySelector('img')).toBeTruthy()
  })
})

describe('ReminderBadges', () => {
  const dash = (over: Partial<Dashboard>): Dashboard => ({
    due_today: 0, due_soon: 0, leech_count: 0, streak: 0, studied_today: false,
    active_topics: [],
    stats: { total_cards: 0, due_today: 0, due_soon: 0, new_cards: 0, avg_ef: 0, by_topic: [] },
    ...over,
  })

  it('按需渲染待出发/延误/顽固卡/streak 徽标', () => {
    const { container } = render(<ReminderBadges dash={dash({ due_today: 3, due_soon: 1, leech_count: 2, streak: 5 })} />)
    expect(container.textContent).toContain('3 待出发')
    expect(container.textContent).toContain('1 延误')
    expect(container.textContent).toContain('2 顽固卡')
    expect(container.textContent).toContain('5 天连续')
  })

  it('全部为 0 时不渲染任何徽标', () => {
    const { container } = render(<ReminderBadges dash={dash({})} />)
    expect(container.textContent).toBe('')
  })
})

describe('SessionTimeline', () => {
  it('渲染会话起止时间与计数', () => {
    const sessions: Session[] = [{
      id: 1, started_at: '2026-08-14T10:00:00Z', ended_at: '2026-08-14T10:30:00Z',
      goal_id: null, pathway_id: null, summary: '学了所有权', new_cards: 2, reviewed: 5,
    }]
    const { container } = render(<SessionTimeline sessions={sessions} />)
    expect(container.textContent).toContain('学了所有权')
    expect(container.textContent).toContain('新建 2')
    expect(container.textContent).toContain('复习 5')
  })

  it('空会话显示暂无记录', () => {
    const { container } = render(<SessionTimeline sessions={[]} />)
    expect(container.textContent).toContain('暂无运行记录')
  })
})
