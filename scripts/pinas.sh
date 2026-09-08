#!/bin/bash
# PiNAS command center: lists every project script by category and runs them.
#
#   ./scripts/pinas.sh                 # interactive menu
#   ./scripts/pinas.sh list            # print the catalogue
#   ./scripts/pinas.sh <name> [args]   # run one entry directly, e.g. ./scripts/pinas.sh dev -d
#   ./scripts/pinas.sh help
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
cd "$ROOT"

if [ -t 1 ]; then
    B=$'\033[1m'; DIM=$'\033[2m'; CYAN=$'\033[0;36m'; GREEN=$'\033[0;32m'; YELLOW=$'\033[1;33m'; RED=$'\033[0;31m'; NC=$'\033[0m'
else
    B=""; DIM=""; CYAN=""; GREEN=""; YELLOW=""; RED=""; NC=""
fi

# name | category | command (relative to repo root) | description
# Categories are displayed in the order they first appear.
ENTRIES=(
  "dev|Développement|scripts/start.dev.sh|Stack de dev Docker Compose : backend dev mode :3388 + frontend hot reload :5173 (args : -d, stop, logs, reset, down, rebuild, shell)"
  "reset-data|Développement|scripts/reset-backend-data.sh|Efface backend/data (base, homes, fichiers) pour rejouer l'onboarding en dev natif"
  "check|Développement|scripts/pinas.sh _check|Vérifications locales : cargo test + -D warnings, svelte-check, vitest, build frontend, cohérence de version"
  "version|Développement|scripts/sync-version.sh|Propage VERSION vers Cargo/npm/package.mk (args : --check, --set X.Y.Z)"
  "build-pi|Build image (VM Linux)|scripts/build-arm64.sh|Image LibreELEC Raspberry Pi 5 : backend musl aarch64 + frontend + .img.gz (args : --backend-only, --frontend-only, --skip-libreelec, --clean)"
  "build-x86|Build image (VM Linux)|scripts/build-x86.sh|Image LibreELEC Generic x86_64 : .img.gz + appliance .ova (VMware, VirtualBox, Proxmox, Synology VMM)"
  "build-arm64-vm|Build image (VM Linux)|scripts/build-arm64-vm.sh|[EXPÉRIMENTAL, ne boote pas] image ARM64 pour QEMU/UTM via le projet Virtual"
  "run-vm|Build image (VM Linux)|scripts/run-vm-qemu.sh|[EXPÉRIMENTAL] démarre l'image ARM64-VM avec QEMU (direct kernel boot)"
  "remote-build|Distant (VM SSH .vm-config)|scripts/remote-build.sh|Lance le build d'image sur la VM Linux et rapatrie image/OVA (args : --arch arm64 / x86 / arm64-vm, --new)"
  "release|Distant (VM SSH .vm-config)|scripts/build-release.sh|Archive de mise à jour pinas-update-vX-<arch>.tar.gz + .sha256 (args : --arch aarch64 / x86_64 / all, --full, --frontend-only, --tag)"
  "deploy|Raspberry Pi|scripts/deploy-pi.sh|Hot deploy sur le Pi via la VM : build + copie backend/frontend/scripts (args : --backend, --frontend, --scripts, --restart, --revert)"
  "connect|Raspberry Pi|scripts/connect-pi.sh|Découvre le Pi sur le LAN et ouvre une session SSH"
  "catalog-tags|Catalogue d'apps|app-catalog/scripts/check-image-tags.py|Compare les tags d'images des manifests au registre (mise à jour disponible → '->')"
  "umbrel|Catalogue d'apps|scripts/convert-umbrel.py|Convertit un docker-compose Umbrel en manifest PiNAS"
  "umbrel-batch|Catalogue d'apps|scripts/convert-umbrel-batch.sh|Conversion en lot du dépôt Umbrel (args : --clone, --umbrel-dir)"
)

field() { echo "$1" | cut -d'|' -f"$2"; }

categories() {
    local seen=" " e c
    for e in "${ENTRIES[@]}"; do
        c=$(field "$e" 2)
        case "$seen" in *" $c "*) ;; *) seen="$seen$c "; echo "$c" ;; esac
    done
}

list() {
    echo ""
    echo "${B}PiNAS — scripts du projet${NC}  ${DIM}(VERSION $(tr -d '[:space:]' < VERSION))${NC}"
    local c e name desc mark
    while IFS= read -r c; do
        echo ""
        echo "  ${CYAN}${c}${NC}"
        for e in "${ENTRIES[@]}"; do
            [ "$(field "$e" 2)" = "$c" ] || continue
            name=$(field "$e" 1); desc=$(field "$e" 4)
            mark=""; case "$desc" in "[EXPÉRIMENTAL"*) mark="$YELLOW" ;; esac
            printf "    ${GREEN}%-15s${NC} ${mark}%s${NC}\n" "$name" "$desc"
        done
    done < <(categories)
    echo ""
    echo "  ${DIM}Sur le Pi lui-même (embarqués dans l'image) : pinas-init.sh, pinas-resize-storage.sh, pinas-kodi-config.sh, pinas-splash.sh, pinas-debug.sh${NC}"
    echo "  ${DIM}Usage : $0 <nom> [args]   ou   $0 sans argument pour le menu${NC}"
    echo ""
}

run_entry() {
    local name="$1"; shift
    local e cmd
    for e in "${ENTRIES[@]}"; do
        if [ "$(field "$e" 1)" = "$name" ]; then
            cmd=$(field "$e" 3)
            echo "${DIM}▶ $cmd $*${NC}"
            case "$cmd" in
                *.py) exec python3 "$cmd" "$@" ;;
                "scripts/pinas.sh _check") exec "$0" _check "$@" ;;
                *) exec "$cmd" "$@" ;;
            esac
        fi
    done
    echo "${RED}Commande inconnue : $name${NC}" >&2
    list >&2
    exit 2
}

# Local verification pipeline (what CI runs), tolerant to missing toolchains
do_check() {
    local rc=0
    export PATH="$HOME/.cargo/bin:$PATH"
    echo "${CYAN}>>> Version consistency${NC}"; scripts/sync-version.sh --check || rc=1
    if command -v cargo >/dev/null; then
        echo "${CYAN}>>> cargo test${NC}";  (cd backend && cargo test --quiet) || rc=1
        echo "${CYAN}>>> cargo check -D warnings${NC}"; (cd backend && RUSTFLAGS="-D warnings" cargo check --quiet) || rc=1
    else
        echo "${YELLOW}cargo absent : backend non vérifié (utiliser la stack Docker : $0 dev)${NC}"
    fi
    if command -v npm >/dev/null; then
        echo "${CYAN}>>> svelte-check${NC}"; (cd frontend && npm run -s check 2>&1 | tail -1) || rc=1
        echo "${CYAN}>>> vitest${NC}";       (cd frontend && npx vitest run 2>&1 | grep -E "Tests |FAIL") || rc=1
        echo "${CYAN}>>> build${NC}";        (cd frontend && npm run -s build >/dev/null 2>&1 && echo "build OK") || rc=1
    else
        echo "${YELLOW}npm absent : frontend non vérifié${NC}"
    fi
    [ $rc -eq 0 ] && echo "${GREEN}Tout est vert.${NC}" || echo "${RED}Des vérifications ont échoué.${NC}"
    return $rc
}

menu() {
    list
    local names=() e i choice
    for e in "${ENTRIES[@]}"; do names+=("$(field "$e" 1)"); done
    for i in "${!names[@]}"; do printf "  %2d) %s\n" "$((i+1))" "${names[$i]}"; done
    echo "   q) quitter"
    echo ""
    read -r -p "Choix : " choice
    case "$choice" in
        q|Q|"") exit 0 ;;
        ''|*[!0-9]*) run_entry "$choice" ;;
        *)
            [ "$choice" -ge 1 ] && [ "$choice" -le "${#names[@]}" ] || { echo "${RED}Choix invalide${NC}"; exit 2; }
            read -r -p "Arguments (optionnel) : " extra
            # shellcheck disable=SC2086
            run_entry "${names[$((choice-1))]}" $extra
            ;;
    esac
}

case "${1:-}" in
    "") menu ;;
    list|ls|-l) list ;;
    help|-h|--help) sed -n '2,7p' "$0" | sed 's/^# \{0,1\}//'; list ;;
    _check) shift; do_check "$@" ;;
    *) run_entry "$@" ;;
esac
