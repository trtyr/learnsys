// 与后端 learnsys-core::entity / repo 对齐的类型。

export interface Card {
  id: string; topic: string; front: string; back: string;
  ef: number; interval: number; reps: number;
  due: string; created: string; updated: string;
  module_id: string | null;
  tags: string[];
  code_block: string | null;
  image_urls: string[];
  source: string | null;
}

export type TopicStatus = 'active' | 'completed' | 'paused'
export interface Topic {
  id: string; name: string; stage: string; status: TopicStatus;
  last_studied: string | null; next_plan: string; created: string;
}

export interface TopicCount { topic: string; count: number }
export interface Stats {
  total_cards: number; due_today: number; due_soon: number;
  new_cards: number;
  avg_ef: number; by_topic: TopicCount[];
}
export interface Dashboard {
  due_today: number; due_soon: number;
  leech_count: number;
  streak: number;
  studied_today: boolean;
  active_topics: Topic[]; stats: Stats;
}

// ─────────────── LMS ───────────────
export interface Goal {
  id: string; title: string; description: string;
  success_criteria: string; topic: string | null;
  status: 'active' | 'achieved' | 'abandoned';
  created: string; achieved_at: string | null;
}
export interface Pathway {
  id: string; name: string; methodology: string;
  description: string; goal_id: string; is_active: boolean;
  created: string;
}
export interface Module {
  id: string; title: string; topic: string | null;
  description: string; status: 'not_started' | 'learning' | 'mastered';
}
export interface PathwayModule {
  pathway_id: string; module_id: string;
  sort_order: number; depends_on: string[];
}
export interface Session {
  id: number; started_at: string; ended_at: string | null;
  goal_id: string | null; pathway_id: string | null;
  summary: string; new_cards: number; reviewed: number;
}
export interface LearnerProfile {
  id: number; level: string; style: string;
  weak_points: string[]; preferences: Record<string, unknown>;
  notes: string; updated: string;
}
export interface ModuleMastery {
  module_id: string; total_cards: number; learned: number;
  avg_ef: number; due_count: number;
}
export interface Resource {
  id: string; title: string; url: string; notes: string;
  module_id: string | null; card_id: string | null; created: string;
}
export interface HeatmapDay { date: string; count: number }
export interface GoalProgress {
  goal_id: string; total_modules: number; mastered: number; percent: number;
}
export interface TimelineEvent {
  at: string; kind: 'card' | 'review' | 'session'; summary: string;
}
