# electron-gpui

An Electron-style starter for native macOS desktop apps, built on
[GPUI](https://www.gpui.rs), the GPU-accelerated Rust UI framework behind the
Zed editor.

You get what an Electron starter gives you (a window, a menu bar, an app
lifecycle, some UI components and a packaging script), without a browser engine
inside. The app is one native binary, rendered with Metal, that starts in a
fraction of a second and uses tens of megabytes of memory, not hundreds.

The scaffolding comes from the desktop app of
[MetalSharp](https://github.com/metalsharp/MetalSharp), which moved from Electron
to GPUI, with everything MetalSharp-specific taken out.

## What's in it

| Electron | Here |
|---|---|
| `main.js` (main process) | `src/main.rs`: fonts, key bindings, menu bar, global services, first window |
| `BrowserWindow` | `open_main_window` in `src/main.rs` (`cx.open_window`) |
| Renderer / your UI | `src/app.rs`: a sidebar shell with example pages |
| `Menu.setApplicationMenu` | `install_menus` in `src/main.rs` (`cx.set_menus`) |
| `globalShortcut` | `src/platform/hotkeys.rs` (system-wide, needs no Accessibility permission) |
| `app.on('before-quit')` from signals | `src/platform/lifecycle.rs` (SIGTERM/SIGINT quit cleanly on the main thread) |
| `window-all-closed` → quit | `cx.on_window_closed` in `src/main.rs` |
| `dialog.showMessageBox` | `window.prompt(...)`, see the About action |
| `shell.openExternal` | `cx.open_url(...)`, used by markdown links |
| CSS variables / theme | `src/theme.rs`: a dark and a light theme as a GPUI global |
| electron-builder | `scripts/package.sh`: `.app` bundle, icon, code signing, `.dmg` |

Components in `src/components/`, each a standalone file:

- **`text_input.rs`**: single-line text field with selection, clipboard, IME
  (Chinese, Japanese, emoji picker) and a password mode.
- **`toast.rs`**: app-wide notifications (success, error, info) and progress
  toasts for long jobs.
- **`markdown.rs`**: renders markdown (headings, emphasis, code, links, lists,
  quotes, tables) as GPUI elements. Useful for release notes and help pages.

The example app has four pages that use all of the above. Try **⌘N** (new
window), **⌘⇧T** (light/dark), **⌘⇧K** from any app (global shortcut), and
**About** in the app menu.

## Requirements

- macOS 13 or later (GPUI's macOS backend; Linux and Windows support in GPUI is
  still early, and this template is only tested on macOS)
- Rust 1.88 or later (edition 2024): `rustup update stable`
- Xcode Command Line Tools (`xcode-select --install`) for the linker,
  `codesign`, `sips` and `iconutil`. Full Xcode isn't needed.

## Build and run

```bash
cargo run
```

The first build compiles GPUI and takes a few minutes; after that, rebuilds take
seconds. `cargo test` runs the component tests.

## Package

```bash
scripts/package.sh
```

That builds in release mode and writes `target/release/Electron GPUI.app`,
ad-hoc signed so it runs on your Mac. Add `--dmg` for a disk image as well:

```bash
scripts/package.sh --dmg
```

Settings are environment variables (the defaults are in the script's header):

```bash
APP_NAME="My App" APP_BUNDLE_ID=com.example.myapp scripts/package.sh --dmg
```

To ship to other Macs, sign with a Developer ID and notarize:

```bash
SIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)" scripts/package.sh --dmg
```

```bash
xcrun notarytool submit "target/release/My App-0.1.0.dmg" --keychain-profile <profile> --wait
```

```bash
xcrun stapler staple "target/release/My App-0.1.0.dmg"
```

## Start your own app

1. Copy the template: `git clone <this repo> my-app && cd my-app && rm -rf .git && git init`.
2. Rename it: `name` in `Cargo.toml` (that's the binary name), `APP_NAME` in
   `src/main.rs`, and the `APP_NAME` / `APP_BUNDLE_ID` defaults in
   `scripts/package.sh`.
3. Replace `assets/AppIcon.png` with your own 1024×1024 icon. (`swift
   scripts/generate-icon.swift` redraws the placeholder.)
4. Put your UI in `src/app.rs`. Each page is a method returning an element;
   state lives on the `Shell` struct, and `cx.notify()` redraws after you change
   it. Add pages to the `Page` enum.
5. Add assets (SVG icons, images) under `assets/` and list them in
   `src/assets.rs`; they're compiled into the binary. SVGs are drawn in the
   current text color, so use `stroke="currentColor"`.
6. Change colors in `src/theme.rs`. Fonts are registered from `assets::FONTS`;
   set the family with `.font_family(...)` on the root element in `app.rs`.
7. Delete what you don't need: the global shortcut, the second theme, any
   component.

Some GPUI pointers:

- Views are structs that implement `Render`; create them with `cx.new(...)` and
  keep the `Entity<T>` handle to read or update them later.
- Layout is flexbox, written Tailwind-style: `div().flex().gap(px(8.0)).p(px(12.0))`.
- Clicks: `.id("name").on_click(cx.listener(|this, event, window, cx| ...))`.
- Background work: `cx.spawn(async move |this, cx| ...)`, with
  `cx.background_executor().timer(...)` for delays.
- App-wide state: a struct that implements `Global`, set with `cx.set_global`;
  see `theme.rs`.
- GPUI's own examples are the best reference for its API:
  `~/.cargo/registry/src/*/gpui-0.2.2/examples/`.

GPUI is pinned to an exact version (`=0.2.2`) because its API still changes
between releases. Upgrade on purpose, and expect small fixes when you do.

## Notes

- `vendor/grid` patches a security advisory (GHSA-38c5-483c-4qqp) in a crate GPUI
  depends on; it builds with a few lifetime-syntax warnings. Remove the patch
  from `Cargo.toml` once GPUI depends on `grid` 1.0.1 or later.
- The app quits when its last window closes, like Electron's usual
  `window-all-closed` handler. For the Mac convention of staying open in the
  Dock, remove the `on_window_closed` block in `src/main.rs`.

## License

MIT, see [LICENSE](LICENSE). The text input (adapted from GPUI's example,
Apache-2.0), the Rethink Sans font (SIL OFL 1.1) and the vendored `grid` crate
(MIT) keep their own licenses; see [NOTICE](NOTICE).
