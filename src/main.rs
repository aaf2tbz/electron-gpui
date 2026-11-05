//! Entry point: the "main process" of an Electron app. It registers fonts, key
//! bindings and the menu bar, installs app-wide services, then opens the first
//! window. The UI itself lives in `app.rs`.
mod app;
mod assets;
mod components;
mod platform;
mod theme;

use anyhow::Result;
use gpui::{
    App, AppContext, Application, Bounds, KeyBinding, Menu, MenuItem, PromptLevel, SystemMenuType,
    TitlebarOptions, WindowBounds, WindowOptions, actions, point, px, size,
};
use std::borrow::Cow;
use std::time::Duration;

use crate::components::{text_input, toast};
use crate::platform::{hotkeys, lifecycle};
use crate::theme::Theme;

/// Shown in the menu bar, window titles and the About box.
pub const APP_NAME: &str = "Electron GPUI";

actions!(
    electron_gpui,
    [About, Quit, Hide, HideOthers, ShowAll, NewWindow, ToggleTheme]
);

/// Id passed to `hotkeys::register` / `hotkeys::take` for the example global shortcut.
const SHOW_APP_HOTKEY: u32 = 1;

fn main() -> Result<()> {
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
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-h", Hide, None),
        KeyBinding::new("alt-cmd-h", HideOthers, None),
        KeyBinding::new("cmd-n", NewWindow, None),
        KeyBinding::new("cmd-shift-t", ToggleTheme, None),
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

    // On macOS the first menu is always the application menu.
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

/// App-wide background work: a system-wide shortcut and graceful quit on
/// SIGTERM/SIGINT. Both are flags set off the main thread and polled here.
fn install_global_services(cx: &mut App) {
    let hotkey = hotkeys::register(
        hotkeys::keys::K,
        hotkeys::modifiers::CMD | hotkeys::modifiers::SHIFT,
        SHOW_APP_HOTKEY,
    );
    if let Err(error) = hotkey {
        eprintln!("{error}");
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
        ..Default::default()
    };
    cx.open_window(options, |window, cx| cx.new(|cx| app::Shell::new(window, cx)))
        .expect("failed to open the main window");
}
