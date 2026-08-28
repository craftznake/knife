#!/usr/bin/env bash
#
# install.sh — build and install the `knife` CLI.
#
# Usage:
#   scripts/install.sh            # cargo install --path . (uses your system Rust toolchain)
#   scripts/install.sh --nix      # build with `nix build` and install into ~/.cargo/bin
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_BIN_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
USE_NIX=0

for arg in "$@"; do
  case "$arg" in
    --nix) USE_NIX=1 ;;
    -h|--help)
      sed -n '2,8p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown option: $arg" >&2
      exit 1
      ;;
  esac
done

cd "$REPO_ROOT"

if [ "$USE_NIX" -eq 1 ]; then
  if ! command -v nix >/dev/null 2>&1; then
    echo "error: --nix requested but 'nix' is not installed. See https://nixos.org/download.html" >&2
    exit 1
  fi
  echo "==> Building knife with 'nix build'..."
  nix build .#default
  mkdir -p "$CARGO_BIN_DIR"
  install -m 755 "$(readlink -f result)/bin/knife" "$CARGO_BIN_DIR/knife"
  rm -f result
else
  if ! command -v cargo >/dev/null 2>&1; then
    if command -v nix >/dev/null 2>&1; then
      echo "==> 'cargo' not found, using 'nix develop' to get a Rust toolchain..."
      nix develop --command cargo install --path .
    else
      cat >&2 <<'EOF'
error: 'cargo' not found and 'nix' is not available either.

Install Rust first, e.g.:
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

or install Nix and re-run this script:
  https://nixos.org/download.html
EOF
      exit 1
    fi
  else
    echo "==> Building and installing knife with 'cargo install --path .'..."
    cargo install --path .
  fi
fi

echo
echo "==> knife installed to: $CARGO_BIN_DIR/knife"

case ":$PATH:" in
  *":$CARGO_BIN_DIR:"*) ;;
  *)
    echo
    echo "warning: $CARGO_BIN_DIR is not on your PATH."
    echo "Add this to your shell config:"
    echo "  export PATH=\"\$PATH:$CARGO_BIN_DIR\""
    ;;
esac

SHELL_NAME="$(basename "${SHELL:-sh}")"
echo
echo "==> To enable shell completion, add this to your shell config:"
echo "  if type knife >/dev/null 2>&1; then"
echo "    eval \"\$(knife completion $SHELL_NAME 2>/dev/null)\""
echo "  fi"
