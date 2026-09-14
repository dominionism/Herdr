#!/usr/bin/env bash
# Build the carbon-sidebar herdr; pass --install to copy it to ~/.local/bin/herdr.
#
# herdr vendors libghostty-vt, which needs Zig 0.15.x. Upstream Zig 0.15.2 cannot
# link against the macOS 26 SDK, so carbon/xcrun-shim hands it a macOS 15 SDK.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLCHAIN="$REPO/.toolchain"
ZIG_VERSION=0.15.2
ZIG_DIR="$TOOLCHAIN/zig-aarch64-macos-$ZIG_VERSION"
SDK="${HERDR_MACOS_SDK:-$(ls -d /Library/Developer/CommandLineTools/SDKs/MacOSX15*.sdk 2>/dev/null | tail -1)}"

if [ ! -x "$ZIG_DIR/zig" ]; then
  mkdir -p "$TOOLCHAIN"
  read -r tarball shasum < <(curl -fsSL https://ziglang.org/download/index.json |
    python3 -c "import json, sys; d = json.load(sys.stdin)['$ZIG_VERSION']['aarch64-macos']; print(d['tarball'], d['shasum'])")
  curl -fsSL -o "$TOOLCHAIN/zig.tar.xz" "$tarball"
  echo "$shasum  $TOOLCHAIN/zig.tar.xz" | shasum -a 256 -c -
  tar -xJf "$TOOLCHAIN/zig.tar.xz" -C "$TOOLCHAIN"
  rm "$TOOLCHAIN/zig.tar.xz"
fi

if [ ! -d "$SDK" ]; then
  echo "no macOS 15 SDK found; install Command Line Tools or set HERDR_MACOS_SDK" >&2
  exit 1
fi

cd "$REPO"
PATH="$REPO/carbon/xcrun-shim:$PATH" \
  HERDR_XCRUN_SDK="$SDK" \
  RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}" \
  ZIG="$ZIG_DIR/zig" \
  cargo build --release

if [ "${1:-}" = "--install" ]; then
  install -m 0755 target/release/herdr "$HOME/.local/bin/herdr"
  echo "installed $("$HOME/.local/bin/herdr" --version) to ~/.local/bin/herdr"
fi
