// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { CaptureModal, CardEditor, CardLibrary, CardRow, GoalRow, LibraryView, QuickCapture, RelatedPicker, ReminderBadges, SessionTimeline, TimelineView } from './App'
import { api } from './api'
import type { Card, Dashboard, Goal, Resource, Session, TimelineEvent } from './types'

const baseCard: Card = {
  id: 'c1', topic: 'rust', front: 'q', back: 'a',
  ef: 2.5, interval: 0, reps: 0,
  due: '2026-08-14', created: '2026-08-14', updated: '2026-08-14T00:00:00Z',
  module_id: null, tags: ['rust'], code_block: 'fn main() {}', image_urls: ['https://e.com/a.png'],
  source: null,
  related: [],
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

describe('TimelineView', () => {
  it('渲染事件与时间', () => {
    const events: TimelineEvent[] = [
      { at: '2026-08-14T10:00:00Z', kind: 'card', summary: '记了卡「所有权」' },
      { at: '2026-08-14T09:00:00Z', kind: 'review', summary: '复习「trait」 q=4' },
    ]
    const { container } = render(<TimelineView events={events} />)
    expect(container.textContent).toContain('记了卡「所有权」')
    expect(container.textContent).toContain('复习「trait」 q=4')
  })

  it('空事件显示提示', () => {
    const { container } = render(<TimelineView events={[]} />)
    expect(container.textContent).toContain('还没有记录')
  })
})

describe('QuickCapture', () => {
  it('点击按钮触发对应 onOpen', () => {
    const onOpen = vi.fn()
    render(<QuickCapture onOpen={onOpen} />)
    fireEvent.click(screen.getByText('＋ 记卡'))
    expect(onOpen).toHaveBeenCalledWith('card')
    fireEvent.click(screen.getByText('＋ 目标'))
    expect(onOpen).toHaveBeenCalledWith('goal')
    fireEvent.click(screen.getByText('＋ 笔记'))
    expect(onOpen).toHaveBeenCalledWith('note')
  })
})

describe('CaptureModal', () => {
  it('记卡提交调用 cards.create（含 tags/code_block/source）', async () => {
    const create = vi.spyOn(api.cards, 'create').mockResolvedValue({} as Card)
    const { container } = render(<CaptureModal kind="card" onClose={() => {}} onSaved={() => {}} />)
    const tas = container.querySelectorAll('textarea')
    fireEvent.change(tas[0], { target: { value: 'q' } })
    fireEvent.change(tas[1], { target: { value: 'a' } })
    fireEvent.change(screen.getByPlaceholderText('rust, 基础'), { target: { value: 'rust, 基础' } })
    fireEvent.change(tas[2], { target: { value: 'fn main() {}' } })
    fireEvent.change(screen.getByPlaceholderText('https://…'), { target: { value: 'https://e.com/a.png' } })
    fireEvent.change(screen.getByPlaceholderText('《Rust 编程之道》第 3 章'), { target: { value: '《Rust 编程之道》第 3 章' } })
    fireEvent.click(screen.getByText('保存'))
    await vi.waitFor(() => expect(create).toHaveBeenCalledWith(expect.objectContaining({
      topic: 'rust', front: 'q', back: 'a',
      tags: ['rust', '基础'],
      code_block: 'fn main() {}',
      image_urls: ['https://e.com/a.png'],
      source: '《Rust 编程之道》第 3 章',
    })))
  })

  it('开目标提交调用 goals.create', async () => {
    const create = vi.spyOn(api.goals, 'create').mockResolvedValue({} as Goal)
    render(<CaptureModal kind="goal" onClose={() => {}} onSaved={() => {}} />)
    fireEvent.change(screen.getByPlaceholderText('学会 Rust'), { target: { value: '学会 Rust' } })
    fireEvent.click(screen.getByText('保存'))
    await vi.waitFor(() => expect(create).toHaveBeenCalledWith({ title: '学会 Rust' }))
  })

  it('记笔记提交调用 resources.create', async () => {
    const create = vi.spyOn(api.resources, 'create').mockResolvedValue({} as Resource)
    const { container } = render(<CaptureModal kind="note" onClose={() => {}} onSaved={() => {}} />)
    fireEvent.change(screen.getByPlaceholderText('所有权要点'), { target: { value: '所有权要点' } })
    fireEvent.change(container.querySelectorAll('textarea')[0], { target: { value: '一些笔记' } })
    fireEvent.click(screen.getByText('保存'))
    await vi.waitFor(() => expect(create).toHaveBeenCalledWith(expect.objectContaining({ title: '所有权要点', notes: '一些笔记' })))
  })
})

describe('GoalRow（学习库树节点）', () => {
  const goal: Goal = { id: 'g1', title: 'Rust 入门', description: '', success_criteria: '', topic: null, status: 'active', created: '2026-08-14', achieved_at: null }

  it('渲染目标标题与状态标签', () => {
    const { container } = render(<GoalRow goal={goal} onRefresh={() => {}} />)
    expect(container.textContent).toContain('Rust 入门')
    expect(container.textContent).toContain('active')
  })

  it('内联编辑重命名调用 goals.update', async () => {
    const update = vi.spyOn(api.goals, 'update').mockResolvedValue({} as Goal)
    const { container } = render(<GoalRow goal={goal} onRefresh={() => {}} />)
    fireEvent.click(screen.getByText('✎'))
    fireEvent.change(container.querySelector('input') as Element, { target: { value: 'Rust 进阶' } })
    fireEvent.click(screen.getByText('保存'))
    await vi.waitFor(() => expect(update).toHaveBeenCalledWith('g1', { title: 'Rust 进阶' }))
  })

  it('删除调用 goals.delete', async () => {
    vi.spyOn(window, 'confirm').mockReturnValue(true)
    const del = vi.spyOn(api.goals, 'delete').mockResolvedValue(undefined)
    render(<GoalRow goal={goal} onRefresh={() => {}} />)
    fireEvent.click(screen.getByText('🗑'))
    await vi.waitFor(() => expect(del).toHaveBeenCalledWith('g1'))
  })
})

describe('LibraryView（笔记闭环）', () => {
  it('展示全局笔记（快捷记笔记后可在此读回）', async () => {
    vi.spyOn(api.goals, 'list').mockResolvedValue([])
    vi.spyOn(api.resources, 'list').mockResolvedValue([
      { id: 'r1', title: '所有权要点', url: '', notes: '独占与借用', module_id: null, card_id: null, created: '2026-08-14' },
    ])
    render(<LibraryView onRefresh={() => {}} />)
    await screen.findByText('所有权要点')
    expect(screen.getByText('独占与借用')).toBeTruthy()
  })
})

describe('CardLibrary（标签筛选）', () => {
  it('按标签筛选卡片', async () => {
    vi.spyOn(api.cards, 'list').mockResolvedValue([
      { ...baseCard, id: 'c1', front: '所有权', tags: ['rust'] },
      { ...baseCard, id: 'c2', front: '连接复用', tags: ['async'] },
    ])
    const { container } = render(<CardLibrary />)
    await screen.findByText('所有权')
    expect(screen.getByText('连接复用')).toBeTruthy()

    const selects = container.querySelectorAll('select')
    fireEvent.change(selects[2], { target: { value: 'rust' } })
    expect(screen.getByText('所有权')).toBeTruthy()
    expect(screen.queryByText('连接复用')).toBeNull()
  })

  it('显示关联卡片（关联 line 含对方 front）', async () => {
    vi.spyOn(api.cards, 'list').mockResolvedValue([
      { ...baseCard, id: 'c1', front: '所有权', related: ['c2'] },
      { ...baseCard, id: 'c2', front: '连接复用', related: ['c1'] },
    ])
    const { container } = render(<CardLibrary />)
    await vi.waitFor(() => expect(container.textContent).toContain('所有权'))
    expect(container.textContent).toContain('关联：')
    expect(container.textContent).toContain('连接复用')
  })

  it('点击关联卡片跳转到编辑', async () => {
    vi.spyOn(api.cards, 'list').mockResolvedValue([
      { ...baseCard, id: 'c1', front: '所有权', related: ['c2'] },
      { ...baseCard, id: 'c2', front: '连接复用', related: ['c1'] },
    ])
    const { container } = render(<CardLibrary />)
    await vi.waitFor(() => expect(container.textContent).toContain('所有权'))
    const link = container.querySelector('a')
    fireEvent.click(link as Element)
    await vi.waitFor(() => expect(container.textContent).toContain('编辑卡片'))
  })
})

describe('RelatedPicker', () => {
  it('搜索关键词 → 点选卡片 → 回调携带 id（不填 id）', async () => {
    vi.spyOn(api.cards, 'list').mockResolvedValue([])
    const search = vi.spyOn(api.cards, 'search').mockResolvedValue([
      { ...baseCard, id: 'c2', front: '连接复用', back: 'RAII' },
    ])
    const onChange = vi.fn()
    render(<RelatedPicker value={[]} onChange={onChange} excludeId="c1" />)
    fireEvent.change(screen.getByPlaceholderText(/搜索要关联的卡/), { target: { value: '连接' } })
    await vi.waitFor(() => expect(search).toHaveBeenCalledWith('连接'))
    fireEvent.click(await screen.findByText('连接复用'))
    expect(onChange).toHaveBeenCalledWith(['c2'])
  })

  it('显示已选 chip，点击 × 移除', async () => {
    vi.spyOn(api.cards, 'list').mockResolvedValue([
      { ...baseCard, id: 'c2', front: '连接复用' },
    ])
    const onChange = vi.fn()
    render(<RelatedPicker value={['c2']} onChange={onChange} />)
    await vi.waitFor(() => expect(screen.getByText('连接复用')).toBeTruthy())
    fireEvent.click(screen.getByLabelText('移除'))
    expect(onChange).toHaveBeenCalledWith([])
  })
})
