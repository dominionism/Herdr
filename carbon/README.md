# Carbon sidebar

A personal patch on herdr that restyles the left sidebar:

- A top bar with outlined buttons: sidebar toggle top-left, menu top-right.
- A `SPACES` header with a `+` new-space button, replacing the old `new … menu` footer.
- An `AGENTS` header with a `grouped` / `priority` sort button.
- Nerd Font state icons instead of dots, with two spaces before each label.
- A rounded accent border around the focused space and agent, drawn in the gap rows so
  the list never shifts. Entries packed against a neighbour keep a background fill.
- Collapsing the sidebar leaves only the expand button, at the same size and position.

Buttons and the focus border are drawn by one rounded-box helper, so every corner has
the same radius. Colors come from `[theme.custom]` in `~/.config/herdr/config.toml`:
button outlines use `overlay0` like unfocused pane frames, and the focus border uses
`accent`. Icons are Nerd Font glyphs, which Ghostty renders without a patched font. Each button
puts a no-break space after its icon: Ghostty spreads an icon followed by a plain space
over two cells and left-aligns it there, which pushes it off the button's center.

## Build and install

    carbon/build.sh --install

This builds `target/release/herdr` and copies it to `~/.local/bin/herdr`. Detach with
`ctrl+b q` and run `herdr` again to load the new client; running agents are untouched.

On first run the script downloads Zig 0.15.2 into `.toolchain/`. Upstream Zig 0.15.2
cannot link against the macOS 26 SDK, so `carbon/xcrun-shim` points it at the newest
installed macOS 15 SDK (override with `HERDR_MACOS_SDK`).

## Moving to a new herdr release

    git fetch upstream --tags
    git rebase --onto vX.Y.Z v0.9.0 carbon-sidebar
    carbon/build.sh --install
    git push --force origin carbon-sidebar:main

`upstream` is herdr's own repository and `origin` is this fork.

Check `vendor/libghostty-vt/build.zig.zon` for `minimum_zig_version` first and update
`ZIG_VERSION` in `carbon/build.sh` if it changed. Homebrew's herdr is unlinked and
pinned so it cannot shadow or replace this build.

## Back to stock herdr

    rm ~/.local/bin/herdr
    brew unpin herdr && brew link herdr
