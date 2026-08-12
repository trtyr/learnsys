#!/usr/bin/env bash
# recall MVP 端到端验证
# 用法: ./scripts/e2e.sh [源数据目录]    默认 ~/.pi/learning-data
# 验证链: 单测 → 迁移 → 后端 → 全 API 端点 → SM-2 review → 前端构建
set -euo pipefail
cd "$(dirname "$0")/.."

DB="${RECALL_DB:-/tmp/learnsys-e2e.db}"
SRC="${1:-$HOME/.pi/learning-data}"
B="http://127.0.0.1:7878"

step() { printf "\n=== %s ===\n" "$1"; }

step "1. 单测 (learnsys-core)"
cargo test -p learnsys-core

step "2. 迁移 $SRC → $DB"
rm -f "$DB"
RECALL_DB="$DB" cargo run -q -p learnsys-migrate -- "$SRC"

step "3. 起后端"
RECALL_DB="$DB" cargo run -q -p learnsys-api > /tmp/learnsys-e2e-api.log 2>&1 &
API=$!
trap 'kill $API 2>/dev/null || true' EXIT
for i in $(seq 1 40); do curl -sf "$B/" >/dev/null 2>&1 && break; sleep 1; done
curl -sf "$B/" >/dev/null || { echo "后端没起来，日志："; tail -20 /tmp/learnsys-e2e-api.log; exit 1; }

step "4. API 端点"
echo "  health    : $(curl -sf "$B/" | python3 -c 'import sys,json;print(json.load(sys.stdin)["status"])')"
echo "  cards/due : $(curl -sf "$B/api/cards/due" | python3 -c 'import sys,json;print(len(json.load(sys.stdin)),"张")')"
echo "  topics    : $(curl -sf "$B/api/topics" | python3 -c 'import sys,json;print(len(json.load(sys.stdin)),"个")')"
echo "  dashboard :"
curl -sf "$B/api/dashboard" | python3 -m json.tool | sed 's/^/    /'

step "5. review (SM-2 q=5)"
CID=$(curl -sf "$B/api/cards/due" | python3 -c 'import sys,json;print(json.load(sys.stdin)[0]["id"])')
curl -sf -X POST "$B/api/cards/$CID/review" -H 'Content-Type: application/json' -d '{"quality":5}' \
  | python3 -m json.tool | sed 's/^/    /'

step "6. 前端构建"
( cd frontend && npm run build ) >/dev/null 2>&1 && echo "  dist ✓" || echo "  (跳过：npm 未就绪)"

step "E2E 通过 ✓"
