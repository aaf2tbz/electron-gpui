//! Entry point: the "main process" of an Electron app. It registers fonts, key
//! bindings and the menu bar, installs app-wide services, then opens the first
//! window. The UI itself lives in `app.rs`.
//!
//! Runs on macOS and Linux (X11 and Wayland). Shortcuts use GPUI's `secondary`
//! modifier: Command on macOS, Ctrl on Linux.
mod app;
mod assets;
mod components;
mod platform;
mod theme;

use anyhow::Result;
use gpui::{
    App, AppContext, Application, Bounds, KeyBinding, Menu, MenuItem, PromptLevel, SystemMenuType,
    TitlebarOptions, WindowBounds, WindowDecorations, WindowOptions, actions, point, px, size,
};
use std::borrow::Cow;
use std::time::Duration;

use crate::components::{text_input, toast};
use crate::platform::{hotkeys, lifecycle};
use crate::theme::Theme;

/// Shown in the menu bar, window titles and the About box.
pub const APP_NAME: &str = "Electron GPUI";
/// Reverse-DNS id. On Linux it is the Wayland app id / X11 WM_CLASS, and must
/// match the `.desktop` file name for the desktop to show the right icon.
pub const APP_ID: &str = "dev.example.electron-gpui";

actions!(
    electron_gpui,
    [About, Quit, Hide, HideOthers, ShowAll, NewWindow, ToggleTheme]
);

/// Id passed to `hotkeys::register` / `hotkeys::take` for the example global shortcut.
const SHOW_APP_HOTKEY: u32 = 1;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    lifecycle::install_signal_handlers()?;

    Application::new().with_assets(assets::Assets).run(|cx: &mut App| {
        cx.text_system()
            .add_fonts(assets::FONTS.iter().map(|font| Cow::Borrowed(*font)).collect())
            .expect("failed to register bundled fonts");

        cx.set_global(Theme::dark());
        toast::install(cx);
        text_input::register_keys(cx);

        install_menus(cx);
        install_global_services(cx);

        open_main_window(cx);
        cx.activate(true);
    });
    Ok(())
}

fn install_menus(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("secondary-q", Quit, None),
        KeyBinding::new("secondary-n", NewWindow, None),
        KeyBinding::new("secondary-shift-t", ToggleTheme, None),
    ]);
    #[cfg(target_os = "macos")]
    cx.bind_keys([
        KeyBinding::new("cmd-h", Hide, None),
        KeyBinding::new("alt-cmd-h", HideOthers, None),
    ]);

    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &Hide, cx| cx.hide());
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
    cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
    cx.on_action(|_: &NewWindow, cx| open_main_window(cx));
    cx.on_action(|_: &ToggleTheme, cx| {
        let next = cx.global::<Theme>().toggled();
        Theme::set(cx, next);
    });
    cx.on_action(|_: &About, cx| {
        let Some(window) = cx.active_window() else {
            return;
        };
        let detail = format!(
            "Version {}\nA native desktop app built on GPUI.",
            env!("CARGO_PKG_VERSION")
        );
        let _ = window.update(cx, |_, window, cx| {
            // Dropping the receiver is fine: we don't need the button index.
            let _ = window.prompt(PromptLevel::Info, APP_NAME, Some(&detail), &["OK"], cx);
        });
    });

    // On macOS the first menu is always the application menu. Linux has no
    // global menu bar, so GPUI keeps these but doesn't show them: the key
    // bindings above still work, and the example app puts About on its
    // Settings page.
    cx.set_menus(vec![
        Menu {
            name: APP_NAME.into(),
            items: vec![
                MenuItem::action(format!("About {APP_NAME}"), About),
                MenuItem::separator(),
                MenuItem::os_submenu("Services", SystemMenuType::Services),
                MenuItem::separator(),
                MenuItem::action(format!("Hide {APP_NAME}"), Hide),
                MenuItem::action("Hide Others", HideOthers),
                MenuItem::action("Show All", ShowAll),
                MenuItem::separator(),
                MenuItem::action(format!("Quit {APP_NAME}"), Quit),
            ],
        },
        Menu {
            name: "File".into(),
            items: vec![MenuItem::action("New Window", NewWindow)],
        },
        Menu {
            name: "View".into(),
            items: vec![MenuItem::action("Toggle Light/Dark", ToggleTheme)],
        },
    ]);
}

/// App-wide background work: a system-wide shortcut (macOS only, see
/// `platform/hotkeys.rs`) and graceful quit on SIGTERM/SIGINT. Both are flags
/// set off the main thread and polled here.
fn install_global_services(cx: &mut App) {
    if hotkeys::SUPPORTED {
        let hotkey = hotkeys::register(
            hotkeys::keys::K,
            hotkeys::modifiers::CMD | hotkeys::modifiers::SHIFT,
            SHOW_APP_HOTKEY,
        );
        if let Err(error) = hotkey {
            log::warn!("{error}");
        }
    }

    cx.spawn(async move |cx| {
        loop {
            cx.background_executor()
                .timer(Duration::from_millis(100))
                .await;
            let result = cx.update(|cx| {
                if lifecycle::quit_requested() {
                    cx.quit();
                }
                if hotkeys::take(SHOW_APP_HOTKEY) {
                    cx.activate(true);
                    toast::info(cx, "Brought to front by the global shortcut ⌘⇧K");
                }
            });
            if result.is_err() {
                break; // the app is shutting down
            }
        }
    })
    .detach();

    // Electron's default: quit once the last window is closed. Remove this for
    // the macOS convention of staying in the Dock with no windows.
    cx.on_window_closed(|cx| {
        if cx.windows().is_empty() {
            cx.quit();
        }
    })
    .detach();
}

pub fn open_main_window(cx: &mut App) {
    // Cascade extra windows a little so they don't stack exactly.
    let offset = px(28.0 * cx.windows().len() as f32);
    let mut bounds = Bounds::centered(None, size(px(1100.0), px(720.0)), cx);
    bounds.origin = point(bounds.origin.x + offset, bounds.origin.y + offset);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(640.0), px(460.0))),
        titlebar: Some(TitlebarOptions {
            title: Some(APP_NAME.into()),
            ..Default::default()
        }),
        app_id: Some(APP_ID.into()),
        // Ask Linux compositors for a title bar. Those that can't provide one
        // (GNOME on Wayland) fall back to client-side decorations, and
        // `components/window_frame.rs` then draws the title bar itself.
        window_decorations: Some(WindowDecorations::Server),
        ..Default::default()
    };
    cx.open_window(options, |window, cx| cx.new(|cx| app::Shell::new(window, cx)))
        .expect("failed to open the main window");
}
