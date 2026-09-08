#!/bin/bash
# Local development stack (Docker Compose): backend in dev mode (:3388, cargo watch)
# + frontend with hot reload (:5173). No Rust or Node needed on the host.
#
# Usage:
#   ./scripts/start.dev.sh              # build if needed, start, follow logs (Ctrl+C stops)
#   ./scripts/start.dev.sh -d           # start detached
#   ./scripts/start.dev.sh logs [svc]   # follow logs (backend | frontend)
#   ./scripts/start.dev.sh stop         # stop containers
#   ./scripts/start.dev.sh restart      # restart containers
#   ./scripts/start.dev.sh down         # stop and remove containers + volumes (cargo/node caches)
#   ./scripts/start.dev.sh reset        # stop, wipe backend/data (DB, homes, files), start again
#   ./scripts/start.dev.sh rebuild      # rebuild images from scratch (Dockerfiles or deps changed)
#   ./scripts/start.dev.sh shell [svc]  # shell in a running container (default: backend)
#   ./scripts/start.dev.sh status
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPOSE=(docker compose -f "$PROJECT_ROOT/docker/docker-compose/docker-compose.yml")

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; RED='\033[0;31m'; NC='\033[0m'

command -v docker >/dev/null || { echo -e "${RED}docker not found${NC}"; exit 1; }
docker compose version >/dev/null 2>&1 || { echo -e "${RED}docker compose (v2) not available${NC}"; exit 1; }

banner() {
    echo ""
    echo -e "  ${GREEN}Frontend${NC}  http://localhost:5173   (hot reload, proxy /api → backend)"
    echo -e "  ${GREEN}Backend${NC}   http://localhost:3388   (PINAS_DEV_MODE=true, cargo watch)"
    echo -e "  ${YELLOW}First run compiles the backend in the container: several minutes.${NC}"
    echo -e "  Data: backend/data (reset with: $0 reset)"
    echo ""
}

case "${1:-up}" in
    up|-d|--detach)
        banner
        if [ "${1:-}" = "-d" ] || [ "${1:-}" = "--detach" ]; then
            "${COMPOSE[@]}" up --build -d
            echo -e "${CYAN}Started. Logs: $0 logs${NC}"
        else
            "${COMPOSE[@]}" up --build
        fi
        ;;
    logs)     "${COMPOSE[@]}" logs -f --tail=100 "${2:-}" ;;
    stop)     "${COMPOSE[@]}" stop ;;
    restart)  "${COMPOSE[@]}" restart "${2:-}" ;;
    down)     "${COMPOSE[@]}" down -v ;;
    rebuild)  "${COMPOSE[@]}" build --no-cache && echo -e "${CYAN}Images rebuilt. Start with: $0${NC}" ;;
    status|ps) "${COMPOSE[@]}" ps ;;
    shell)
        svc="${2:-backend}"
        "${COMPOSE[@]}" exec "$svc" bash 2>/dev/null || "${COMPOSE[@]}" exec "$svc" sh
        ;;
    reset)
        "${COMPOSE[@]}" stop backend
        echo -e "${YELLOW}Wiping backend/data (database, homes, files, secrets)...${NC}"
        rm -rf "$PROJECT_ROOT/backend/data"
        "${COMPOSE[@]}" up -d backend
        echo -e "${CYAN}Backend restarted with an empty database: onboarding will run again.${NC}"
        ;;
    -h|--help|help) sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//' ;;
    *) echo -e "${RED}unknown command: $1${NC}"; sed -n '5,15p' "$0" | sed 's/^# \{0,1\}//'; exit 2 ;;
esac
