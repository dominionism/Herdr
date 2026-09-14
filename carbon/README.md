# Carbon sidebar

A personal patch on herdr that restyles the left sidebar:

- A toolbar row with rounded buttons: sidebar toggle top-left, menu top-right.
- A `SPACES` header with a `+` new-space button, replacing the old `new … menu` footer.
- An `AGENTS` header with a `grouped` / `priority` sort pill.
- A steel accent bar on the focused space and agent rows.
- The collapsed rail keeps its expand button in the same top-left spot.

Colors come from `[theme.custom]` in `~/.config/herdr/config.toml`: buttons use
`surface1`, the sort pill `surface0`, and the accent bar `accent`. Icons are Nerd Font
glyphs, which Ghostty renders without a patched font.

## Build and install

    carbon/build.sh --install

This builds `target/release/herdr` and copies it to `~/.local/bin/herdr`. Detach with
`ctrl+b q` and run `herdr` again to load the new client; running agents are untouched.

On first run the script downloads Zig 0.15.2 into `.toolchain/`. Upstream Zig 0.15.2
cannot link against the macOS 26 SDK, so `carbon/xcrun-shim` points it at the newest
installed macOS 15 SDK (override with `HERDR_MACOS_SDK`).

## Moving to a new herdr release

    git fetch --tags
    git rebase --onto vX.Y.Z v0.9.0 carbon-sidebar
    carbon/build.sh --install

Check `vendor/libghostty-vt/build.zig.zon` for `minimum_zig_version` first and update
`ZIG_VERSION` in `carbon/build.sh` if it changed. Homebrew's herdr is unlinked and
pinned so it cannot shadow or replace this build.

## Back to stock herdr

    rm ~/.local/bin/herdr
    brew unpin herdr && brew link herdr
