#!/usr/bin/env bash
# Instala o Glint (AppImage) para o seu usuário, sem precisar de root:
#   - copia o AppImage que está na MESMA pasta deste script para ~/Aplicativos/Glint.AppImage
#   - cria o atalho no menu de aplicativos e o início automático no login
#   - abre o Glint desligado do terminal
# Uso:  bash install-glint.sh      (rode de novo para atualizar/reparar a instalação)
set -euo pipefail

AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DESTINO_DIR="$HOME/Aplicativos"
DESTINO="$DESTINO_DIR/Glint.AppImage"

say() { printf '\033[1m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[33maviso:\033[0m %s\n' "$*" >&2; }

# 1) acha o AppImage ao lado do script (o mais novo, se houver mais de um)
ORIGEM="$(ls -t "$AQUI"/Glint*.AppImage 2>/dev/null | head -n 1 || true)"
if [ -z "$ORIGEM" ]; then
  echo "Não achei nenhum Glint*.AppImage em $AQUI" >&2
  echo "Coloque o arquivo (ex.: Glint_2.6.2_amd64.AppImage) na mesma pasta deste script e rode de novo." >&2
  exit 1
fi
say "Usando $(basename "$ORIGEM")"

# 2) AppImage precisa do FUSE 2
if ! ldconfig -p 2>/dev/null | grep -q 'libfuse.so.2'; then
  warn "libfuse2 não encontrada: o AppImage pode não abrir."
  if command -v apt-get >/dev/null 2>&1; then
    PKG="libfuse2"
    apt-cache show libfuse2t64 >/dev/null 2>&1 && PKG="libfuse2t64"
    if [ -t 0 ]; then
      read -r -p "Instalar $PKG agora (pede sua senha)? [S/n] " R
      case "${R:-S}" in
        [nN]*) warn "Seguindo sem instalar. Se não abrir: sudo apt install $PKG" ;;
        *) sudo apt-get install -y "$PKG" ;;
      esac
    else
      warn "Se não abrir, rode: sudo apt install $PKG"
    fi
  fi
fi

# 3) fecha o Glint que estiver rodando (para poder trocar o arquivo)
if pgrep -f "Glint.*AppImage" >/dev/null 2>&1; then
  say "Fechando o Glint que está aberto"
  pkill -f "Glint.*AppImage" || true
  sleep 1
fi

# 4) copia e marca como executável
mkdir -p "$DESTINO_DIR"
cp -f "$ORIGEM" "$DESTINO"
chmod +x "$DESTINO"
say "Instalado em $DESTINO"

# 5) ícone (extraído do próprio AppImage; se falhar, segue sem ícone)
ICONE_DIR="$HOME/.local/share/icons/hicolor/128x128/apps"
mkdir -p "$ICONE_DIR"
TMP="$(mktemp -d)"
( cd "$TMP" && "$DESTINO" --appimage-extract '*.png' >/dev/null 2>&1 || true )
ICONE="$(find "$TMP/squashfs-root" -maxdepth 6 -name '*.png' 2>/dev/null | head -n 1 || true)"
if [ -n "$ICONE" ]; then cp -f "$ICONE" "$ICONE_DIR/glint.png"; else warn "não consegui extrair o ícone (sem problema)"; fi
rm -rf "$TMP"

# 6) atalho no menu de aplicativos
mkdir -p "$HOME/.local/share/applications"
cat > "$HOME/.local/share/applications/glint.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Glint
Comment=Busca rápida de apps, arquivos e ações
Exec="$DESTINO"
Icon=glint
Terminal=false
Categories=Utility;
EOF

# 7) iniciar com o sistema (reescreve sempre, para corrigir caminhos antigos que apontavam para pasta temporária)
mkdir -p "$HOME/.config/autostart"
cat > "$HOME/.config/autostart/glint.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Glint
Exec="$DESTINO"
X-GNOME-Autostart-enabled=true
EOF
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1 || true

# 8) abre o Glint desligado do terminal
say "Abrindo o Glint"
setsid nohup "$DESTINO" >/dev/null 2>&1 &
sleep 1

echo
echo "Pronto. Aperte Ctrl+Espaço para abrir."
echo "  Sessão gráfica: ${XDG_SESSION_TYPE:-desconhecida}"
if [ "${XDG_SESSION_TYPE:-}" = "wayland" ]; then
  echo "  No Wayland o atalho global não funciona: crie um atalho do sistema que execute:"
  echo "    $DESTINO"
fi
echo "  Se o Ctrl+Espaço não responder, ele pode estar ocupado pelo seletor de idioma:"
echo "  troque o atalho em Configurações (Ctrl+,) ou pelo ícone da bandeja."
echo "  Para atualizar depois: Configurações > Verificar agora > Atualizar."
