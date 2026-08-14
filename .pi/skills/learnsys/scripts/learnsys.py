#!/usr/bin/env python3
"""learnsys CLI —— 用命令行操作学习系统的所有功能。

只依赖 Python 标准库，无需安装任何包。
API 地址用环境变量 `LEARNSYS_URL` 指定（默认 http://127.0.0.1:7878）。

示例：
  python3 learnsys.py card create --topic rust --front "所有权是什么" --back "独占"
  python3 learnsys.py card review <id> 5
  python3 learnsys.py dashboard
"""

import argparse
import json
import os
import sys
from urllib import error, request

BASE = os.environ.get("LEARNSYS_URL", "http://127.0.0.1:7878").rstrip("/")


def call(method, path, body=None):
    """发请求，返回解析后的 JSON（204 或空体返回 None）。"""
    url = BASE + path
    data = None
    headers = {}
    if body is not None:
        data = json.dumps(body).encode("utf-8")
        headers["Content-Type"] = "application/json"
    req = request.Request(url, data=data, method=method, headers=headers)
    try:
        with request.urlopen(req) as resp:
            raw = resp.read()
            return None if not raw else json.loads(raw)
    except error.HTTPError as e:
        detail = e.read().decode("utf-8", errors="replace")
        try:
            detail = json.loads(detail)
        except Exception:
            pass
        print(f"✗ HTTP {e.code}: {detail}", file=sys.stderr)
        sys.exit(1)
    except error.URLError as e:
        print(f"✗ 无法连接 {BASE}: {e.reason}", file=sys.stderr)
        sys.exit(1)


def call_text(path):
    """发 GET 请求，返回纯文本（用于 markdown 导出）。"""
    url = BASE + path
    try:
        with request.urlopen(url) as resp:
            return resp.read().decode("utf-8")
    except error.HTTPError as e:
        print(f"✗ HTTP {e.code}: {e.read().decode(errors='replace')}", file=sys.stderr)
        sys.exit(1)


def out(data):
    if data is None:
        return
    print(json.dumps(data, ensure_ascii=False, indent=2))


def split_csv(s):
    """逗号分隔字符串 → 去空白的列表；None/空返回 None。"""
    if not s:
        return None
    return [x.strip() for x in s.split(",") if x.strip()]


def add_id(p):
    p.add_argument("id")


def add_topic_opt(p):
    p.add_argument("--topic", help="主题名")


def main():
    p = argparse.ArgumentParser(prog="learnsys", description="操作学习系统的 CLI")
    sub = p.add_subparsers(dest="cmd", required=True)

    # ── card ──
    c = sub.add_parser("card", help="卡片")
    cs = c.add_subparsers(dest="sub")

    p_cc = cs.add_parser("create", help="建卡")
    p_cc.add_argument("--topic", required=True)
    p_cc.add_argument("--front", required=True)
    p_cc.add_argument("--back", required=True)
    p_cc.add_argument("--tags")

    p_cl = cs.add_parser("list", help="列卡")
    add_topic_opt(p_cl)

    p_cg = cs.add_parser("get", help="取一张")
    add_id(p_cg)

    p_cs = cs.add_parser("search", help="搜索")
    p_cs.add_argument("q")
    add_topic_opt(p_cs)

    p_ce = cs.add_parser("edit", help="编辑卡片（不改 SM-2）")
    add_id(p_ce)
    p_ce.add_argument("--front")
    p_ce.add_argument("--back")
    p_ce.add_argument("--topic")
    p_ce.add_argument("--tags")
    p_ce.add_argument("--code-block")
    p_ce.add_argument("--image-urls")
    p_ce.add_argument("--module-id", help="挂到模块（空串=脱离）")

    p_cd = cs.add_parser("delete", help="删卡")
    add_id(p_cd)

    p_cr = cs.add_parser("review", help="复习（SM-2 唯一入口）")
    add_id(p_cr)
    p_cr.add_argument("quality", type=int, help="0-5")

    cs.add_parser("new", help="今日新卡（每日预算）")
    cs.add_parser("leeches", help="顽固卡")

    # ── topic ──
    t = sub.add_parser("topic", help="主题")
    ts = t.add_subparsers(dest="sub")
    p_tc = ts.add_parser("create", help="建主题")
    p_tc.add_argument("name")
    ts.add_parser("list", help="列主题")
    p_tg = ts.add_parser("get", help="取一个")
    add_id(p_tg)
    p_tu = ts.add_parser("update", help="更新主题")
    add_id(p_tu)
    p_tu.add_argument("--stage")
    p_tu.add_argument("--status", choices=["active", "completed", "paused"])
    p_tu.add_argument("--next-plan")
    p_tu.add_argument("--last-studied")

    # ── goal ──
    g = sub.add_parser("goal", help="目标")
    gs = g.add_subparsers(dest="sub")
    p_gc = gs.add_parser("create", help="建目标")
    p_gc.add_argument("title")
    p_gc.add_argument("--description")
    p_gc.add_argument("--criteria")
    p_gc.add_argument("--topic")
    gs.add_parser("list", help="列目标")
    p_gg = gs.add_parser("get", help="取目标")
    add_id(p_gg)
    p_gu = gs.add_parser("update", help="重命名/改目标")
    add_id(p_gu)
    p_gu.add_argument("--title")
    p_gu.add_argument("--description")
    p_gu.add_argument("--criteria")
    p_gd = gs.add_parser("delete", help="删目标（级联删路径）")
    add_id(p_gd)
    p_gs = gs.add_parser("status", help="更新状态")
    add_id(p_gs)
    p_gs.add_argument("status", choices=["active", "achieved", "abandoned"])
    p_gs.add_argument("--achieved-at")
    p_gp = gs.add_parser("progress", help="目标进度")
    add_id(p_gp)

    # ── pathway ──
    pw = sub.add_parser("pathway", help="路径")
    pws = pw.add_subparsers(dest="sub")
    p_pc = pws.add_parser("create", help="建路径")
    p_pc.add_argument("name")
    p_pc.add_argument("--goal", required=True, help="目标 id")
    p_pc.add_argument("--methodology")
    p_pc.add_argument("--description")
    p_pl = pws.add_parser("list", help="列路径")
    p_pl.add_argument("--goal", required=True, help="目标 id")
    p_pg = pws.add_parser("get", help="取路径")
    add_id(p_pg)
    p_pu = pws.add_parser("update", help="重命名/改路径")
    add_id(p_pu)
    p_pu.add_argument("--name")
    p_pu.add_argument("--methodology")
    p_pu.add_argument("--description")
    p_pd = pws.add_parser("delete", help="删路径")
    add_id(p_pd)
    p_pm = pws.add_parser("modules", help="列路径模块")
    add_id(p_pm)
    p_pa = pws.add_parser("add-module", help="挂模块到路径")
    add_id(p_pa)
    p_pa.add_argument("--module", required=True, help="模块 id")
    p_pa.add_argument("--order", type=int, required=True)
    p_pa.add_argument("--depends", help="逗号分隔的前置模块 id")
    p_pn = pws.add_parser("next", help="下一个可学模块")
    add_id(p_pn)

    # ── module ──
    m = sub.add_parser("module", help="模块")
    ms = m.add_subparsers(dest="sub")
    p_mc = ms.add_parser("create", help="建模块")
    p_mc.add_argument("title")
    p_mc.add_argument("--topic")
    p_mc.add_argument("--description")
    p_ml = ms.add_parser("list", help="列模块")
    add_topic_opt(p_ml)
    p_mu = ms.add_parser("update", help="重命名模块")
    add_id(p_mu)
    p_mu.add_argument("--title")
    p_mu.add_argument("--description")
    p_md = ms.add_parser("delete", help="删模块（卡片降为散卡）")
    add_id(p_md)
    p_mm = ms.add_parser("mastery", help="模块掌握度")
    add_id(p_mm)
    p_mk = ms.add_parser("cards", help="模块下的卡片")
    add_id(p_mk)
    p_mst = ms.add_parser("status", help="更新模块状态")
    add_id(p_mst)
    p_mst.add_argument("status", choices=["not_started", "learning", "mastered"])

    # ── session ──
    s = sub.add_parser("session", help="学习会话")
    ss = s.add_subparsers(dest="sub")
    p_ss = ss.add_parser("start", help="开会话")
    p_ss.add_argument("--goal")
    p_ss.add_argument("--pathway")
    p_se = ss.add_parser("end", help="结会话")
    p_se.add_argument("id", type=int)
    p_se.add_argument("--summary")
    p_se.add_argument("--new-cards", type=int)
    p_se.add_argument("--reviewed", type=int)
    p_sl = ss.add_parser("list", help="列会话")
    p_sl.add_argument("--limit", type=int)

    # ── resource / profile / settings / quiz / stats / export ──
    r = sub.add_parser("resource", help="学习资源/笔记")
    rs = r.add_subparsers(dest="sub")
    p_rc = rs.add_parser("create", help="建资源")
    p_rc.add_argument("title")
    p_rc.add_argument("--url")
    p_rc.add_argument("--notes")
    p_rc.add_argument("--module-id")
    p_rc.add_argument("--card-id")
    p_rl = rs.add_parser("list", help="列资源")
    p_rl.add_argument("--module-id")

    pr = sub.add_parser("profile", help="学习者画像")
    prs = pr.add_subparsers(dest="sub")
    prs.add_parser("get", help="读画像")
    p_pu = prs.add_parser("update", help="写画像")
    p_pu.add_argument("--level")
    p_pu.add_argument("--style")
    p_pu.add_argument("--weak-points", help="逗号分隔")
    p_pu.add_argument("--notes")

    st = sub.add_parser("settings", help="设置")
    sts = st.add_subparsers(dest="sub")
    sts.add_parser("get", help="读设置")
    p_sst = sts.add_parser("set", help="写设置")
    p_sst.add_argument("--new-per-day", type=int)

    q = sub.add_parser("quiz", help="测验抽取")
    q.add_argument("--n", type=int, help="题数（默认 5）")
    add_topic_opt(q)

    sub.add_parser("stats", help="统计")
    sub.add_parser("dashboard", help="看板聚合")
    h = sub.add_parser("heatmap", help="复习热力")
    h.add_argument("--days", type=int, default=90)
    sub.add_parser("export", help="全量 JSON 导出")
    sub.add_parser("export-markdown", help="markdown 导出")
    sub.add_parser("backup", help="SQLite 快照备份")
    sub.add_parser("timeline", help="今日活动时间线")

    a = p.parse_args()
    c = a.cmd

    if c == "card":
        s2 = a.sub
        if s2 == "create":
            out(call("POST", "/api/cards", {"topic": a.topic, "front": a.front, "back": a.back, "tags": split_csv(a.tags)}))
        elif s2 == "list":
            out(call("GET", f"/api/cards?topic={a.topic}" if a.topic else "/api/cards"))
        elif s2 == "get":
            out(call("GET", f"/api/cards/{a.id}"))
        elif s2 == "search":
            q = f"/api/cards/search?q={request.quote(a.q)}"
            out(call("GET", q + (f"&topic={a.topic}" if a.topic else "")))
        elif s2 == "edit":
            out(call("PUT", f"/api/cards/{a.id}", {
                "front": a.front, "back": a.back, "topic": a.topic,
                "tags": split_csv(a.tags), "code_block": a.code_block,
                "image_urls": split_csv(a.image_urls), "module_id": a.module_id,
            }))
        elif s2 == "delete":
            call("DELETE", f"/api/cards/{a.id}")
        elif s2 == "review":
            out(call("POST", f"/api/cards/{a.id}/review", {"quality": a.quality}))
        elif s2 == "new":
            out(call("GET", "/api/cards/new"))
        elif s2 == "leeches":
            out(call("GET", "/api/cards/leeches"))

    elif c == "topic":
        s2 = a.sub
        if s2 == "create":
            out(call("POST", "/api/topics", {"name": a.name}))
        elif s2 == "list":
            out(call("GET", "/api/topics"))
        elif s2 == "get":
            out(call("GET", f"/api/topics/{a.id}"))
        elif s2 == "update":
            out(call("PUT", f"/api/topics/{a.id}", {"stage": a.stage, "status": a.status, "next_plan": a.next_plan, "last_studied": a.last_studied}))

    elif c == "goal":
        s2 = a.sub
        if s2 == "create":
            out(call("POST", "/api/goals", {"title": a.title, "description": a.description, "success_criteria": a.criteria, "topic": a.topic}))
        elif s2 == "list":
            out(call("GET", "/api/goals"))
        elif s2 == "get":
            out(call("GET", f"/api/goals/{a.id}"))
        elif s2 == "update":
            out(call("PUT", f"/api/goals/{a.id}", {"title": a.title, "description": a.description, "success_criteria": a.criteria}))
        elif s2 == "delete":
            call("DELETE", f"/api/goals/{a.id}")
        elif s2 == "status":
            out(call("PUT", f"/api/goals/{a.id}/status", {"status": a.status, "achieved_at": a.achieved_at}))
        elif s2 == "progress":
            out(call("GET", f"/api/goals/{a.id}/progress"))

    elif c == "pathway":
        s2 = a.sub
        if s2 == "create":
            out(call("POST", "/api/pathways", {"name": a.name, "goal_id": a.goal, "methodology": a.methodology, "description": a.description}))
        elif s2 == "list":
            out(call("GET", f"/api/pathways?goal={a.goal}"))
        elif s2 == "get":
            out(call("GET", f"/api/pathways/{a.id}"))
        elif s2 == "update":
            out(call("PUT", f"/api/pathways/{a.id}", {"name": a.name, "methodology": a.methodology, "description": a.description}))
        elif s2 == "delete":
            call("DELETE", f"/api/pathways/{a.id}")
        elif s2 == "modules":
            out(call("GET", f"/api/pathways/{a.id}/modules"))
        elif s2 == "add-module":
            out(call("POST", f"/api/pathways/{a.id}/modules", {"module_id": a.module, "sort_order": a.order, "depends_on": split_csv(a.depends) or []}))
        elif s2 == "next":
            out(call("GET", f"/api/pathways/{a.id}/next"))

    elif c == "module":
        s2 = a.sub
        if s2 == "create":
            out(call("POST", "/api/modules", {"title": a.title, "topic": a.topic, "description": a.description}))
        elif s2 == "list":
            out(call("GET", f"/api/modules?topic={a.topic}" if a.topic else "/api/modules"))
        elif s2 == "update":
            out(call("PUT", f"/api/modules/{a.id}", {"title": a.title, "description": a.description}))
        elif s2 == "delete":
            call("DELETE", f"/api/modules/{a.id}")
        elif s2 == "mastery":
            out(call("GET", f"/api/modules/{a.id}/mastery"))
        elif s2 == "cards":
            out(call("GET", f"/api/modules/{a.id}/cards"))
        elif s2 == "status":
            out(call("PUT", f"/api/modules/{a.id}/status", {"status": a.status}))

    elif c == "session":
        s2 = a.sub
        if s2 == "start":
            out(call("POST", "/api/sessions/start", {"goal_id": a.goal, "pathway_id": a.pathway}))
        elif s2 == "end":
            call("POST", f"/api/sessions/{a.id}/end", {"summary": a.summary, "new_cards": a.new_cards, "reviewed": a.reviewed})
        elif s2 == "list":
            out(call("GET", f"/api/sessions?limit={a.limit}" if a.limit else "/api/sessions"))

    elif c == "resource":
        s2 = a.sub
        if s2 == "create":
            out(call("POST", "/api/resources", {"title": a.title, "url": a.url, "notes": a.notes, "module_id": a.module_id, "card_id": a.card_id}))
        elif s2 == "list":
            out(call("GET", f"/api/resources?module_id={a.module_id}" if a.module_id else "/api/resources"))

    elif c == "profile":
        s2 = a.sub
        if s2 == "get":
            out(call("GET", "/api/profile"))
        elif s2 == "update":
            out(call("PUT", "/api/profile", {"level": a.level, "style": a.style, "weak_points": split_csv(a.weak_points), "notes": a.notes}))

    elif c == "settings":
        s2 = a.sub
        if s2 == "get":
            out(call("GET", "/api/settings"))
        elif s2 == "set":
            out(call("PUT", "/api/settings", {"new_per_day": a.new_per_day}))

    elif c == "quiz":
        q = f"/api/quiz?n={a.n or 5}"
        if a.topic:
            q += f"&topic={a.topic}"
        out(call("GET", q))

    elif c == "stats":
        out(call("GET", "/api/stats"))
    elif c == "dashboard":
        out(call("GET", "/api/dashboard"))
    elif c == "heatmap":
        out(call("GET", f"/api/stats/heatmap?days={a.days}"))
    elif c == "export":
        out(call("GET", "/api/export"))
    elif c == "export-markdown":
        print(call_text("/api/export/markdown"))
    elif c == "backup":
        out(call("POST", "/api/backup"))
    elif c == "timeline":
        out(call("GET", "/api/timeline"))


if __name__ == "__main__":
    main()
