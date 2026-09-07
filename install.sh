#!/usr/bin/env bash
# ============================================================
# xterminium — Instalador / Atualizador  (Linux .deb)
# Uso:  bash <(curl -fsSL https://raw.githubusercontent.com/Josemarcio15/xterminium/main/install.sh)
# ============================================================
set -euo pipefail

REPO="Josemarcio15/xterminium"
API="https://api.github.com/repos/${REPO}/releases/latest"
TMP_DEB="/tmp/xterminium_latest.deb"

# ── Cores ───────────────────────────────────────────────────
RESET="\033[0m"
BOLD="\033[1m"
GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RED="\033[31m"

info()    { printf "${CYAN}${BOLD}[xterminium]${RESET} %s\n" "$*"; }
success() { printf "${GREEN}${BOLD}✓${RESET} %s\n" "$*"; }
warn()    { printf "${YELLOW}${BOLD}⚠${RESET}  %s\n" "$*"; }
error()   { printf "${RED}${BOLD}✗ Erro:${RESET} %s\n" "$*" >&2; exit 1; }

# ── Verificações de dependências ────────────────────────────
check_deps() {
    for cmd in curl dpkg sudo; do
        command -v "$cmd" &>/dev/null || error "Dependência não encontrada: '$cmd'. Instale e tente novamente."
    done
}

# ── Busca a versão mais recente no GitHub ───────────────────
fetch_release() {
    info "Buscando versão mais recente em github.com/${REPO} ..."

    local response
    response=$(curl -fsSL -H "User-Agent: xterminium-installer" "$API") \
        || error "Falha ao consultar a API do GitHub. Verifique sua conexão."

    TAG=$(printf '%s' "$response" | grep -o '"tag_name": *"[^"]*"' | head -1 | cut -d'"' -f4)
    [[ -z "$TAG" ]] && error "Não foi possível determinar a versão mais recente."

    # Remove 'v' do início para montar o nome do asset
    local version="${TAG#v}"

    # Extrai a URL do .deb
    DEB_URL=$(printf '%s' "$response" \
        | grep -o '"browser_download_url": *"[^"]*\.deb"' \
        | head -1 \
        | cut -d'"' -f4)

    [[ -z "$DEB_URL" ]] && error "Nenhum asset .deb encontrado no release '${TAG}'."

    success "Última versão: ${TAG}"
}

# ── Verifica versão já instalada ────────────────────────────
check_installed_version() {
    if dpkg -s xterminium &>/dev/null 2>&1; then
        INSTALLED_VERSION=$(dpkg -s xterminium 2>/dev/null | grep '^Version:' | awk '{print $2}')
        local latest_clean="${TAG#v}"
        if [[ "$INSTALLED_VERSION" == "$latest_clean" ]]; then
            success "xterminium ${INSTALLED_VERSION} já está instalado e atualizado."
            exit 0
        fi
        info "Versão instalada: ${INSTALLED_VERSION}  →  Nova versão: ${latest_clean}"
    else
        info "xterminium não encontrado. Realizando instalação inicial..."
    fi
}

# ── Download ────────────────────────────────────────────────
download_deb() {
    info "Baixando ${DEB_URL##*/} ..."
    curl -fL --progress-bar "$DEB_URL" -o "$TMP_DEB" \
        || error "Falha no download. Tente novamente."
    success "Download concluído."
}

# ── Instalação ───────────────────────────────────────────────
install_deb() {
    info "Instalando (sudo dpkg -i) — você pode precisar digitar sua senha:"
    echo
    sudo dpkg -i "$TMP_DEB" || {
        warn "dpkg reportou dependências faltantes. Tentando corrigir com apt..."
        sudo apt-get install -f -y
    }
    rm -f "$TMP_DEB"
    echo
    success "xterminium ${TAG} instalado com sucesso! 🚀"
    info "Execute 'xterminium' no terminal ou procure no lançador de apps."
}

# ── Main ─────────────────────────────────────────────────────
main() {
    echo
    printf "${BOLD}${CYAN}"
    echo "  ██╗  ██╗████████╗███████╗██████╗ ███╗   ███╗██╗███╗   ██╗██╗██╗   ██╗███╗   ███╗"
    echo "  ╚██╗██╔╝╚══██╔══╝██╔════╝██╔══██╗████╗ ████║██║████╗  ██║██║██║   ██║████╗ ████║"
    echo "   ╚███╔╝    ██║   █████╗  ██████╔╝██╔████╔██║██║██╔██╗ ██║██║██║   ██║██╔████╔██║"
    echo "   ██╔██╗    ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║██║╚██╗██║██║██║   ██║██║╚██╔╝██║"
    echo "  ██╔╝ ██╗   ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║██║ ╚████║██║╚██████╔╝██║ ╚═╝ ██║"
    echo "  ╚═╝  ╚═╝   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝ ╚═════╝ ╚═╝     ╚═╝"
    printf "${RESET}"
    echo
    info "Instalador / Atualizador — Linux (.deb)"
    echo

    check_deps
    fetch_release
    check_installed_version
    download_deb
    install_deb
}

main "$@"
