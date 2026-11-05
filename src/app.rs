//! The example app: a sidebar shell with a few pages showing the bundled
//! components. This is the "renderer" of an Electron app. Replace the pages
//! with your own; keep or drop the shell.
use gpui::{
    AnyElement, Context, Entity, FontWeight, IntoElement, Render, SharedString, Window, div,
    prelude::*, px, rgb, rgba, svg,
};
use std::time::Duration;

use crate::components::{markdown, text_input::TextInput, toast};
use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Home,
    Components,
    Markdown,
    Settings,
}

impl Page {
    const ALL: [Page; 4] = [Page::Home, Page::Components, Page::Markdown, Page::Settings];

    fn label(self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Components => "Components",
            Page::Markdown => "Markdown",
            Page::Settings => "Settings",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Page::Home => "icons/home.svg",
            Page::Components => "icons/blocks.svg",
            Page::Markdown => "icons/text.svg",
            Page::Settings => "icons/settings.svg",
        }
    }
}

const SAMPLE_MARKDOWN: &str = "\
# Markdown, natively

GPUI has no HTML engine, so this template renders **markdown** straight to GPUI elements: \
*emphasis*, `inline code`, ~~strikethrough~~ and [links](https://www.gpui.rs) that open in \
your browser.

## Lists

- Bullet items
  - nested ones
- and more

1. Numbered
2. items

> Quotes render with a bar on the left.

```
fn main() {
    println!(\"code blocks keep their spacing\");
}
```

| Feature | Electron | GPUI |
|---|---|---|
| Rendering | Chromium | Metal / native GPU |
| Language | JavaScript | Rust |
";

pub struct Shell {
    page: Page,
    clicks: u32,
    name: Entity<TextInput>,
    password: Entity<TextInput>,
}

impl Shell {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| {
            let mut input = TextInput::new(cx);
            input.placeholder = "Your name".into();
            input
        });
        let password = cx.new(|cx| {
            let mut input = TextInput::new(cx);
            input.placeholder = "A secret".into();
            input.secret = true;
            input
        });
        Self {
            page: Page::Home,
            clicks: 0,
            name,
            password,
        }
    }

    fn sidebar(&self, theme: Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let mut nav = div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .w(px(200.0))
            .flex_none()
            .h_full()
            .p(px(12.0))
            .bg(rgb(theme.sidebar))
            .border_r_1()
            .border_color(rgba(theme.border))
            .child(
                div()
                    .px(px(10.0))
                    .pt(px(8.0))
                    .pb(px(14.0))
                    .text_size(px(15.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(theme.text))
                    .child(crate::APP_NAME),
            );
        for page in Page::ALL {
            let active = page == self.page;
            nav = nav.child(
                div()
                    .id(page.label())
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .h(px(34.0))
                    .px(px(10.0))
                    .rounded(px(7.0))
                    .cursor_pointer()
                    .text_size(px(13.0))
                    .text_color(rgb(if active { theme.text } else { theme.muted }))
                    .when(active, |item| item.bg(rgb(theme.hover)))
                    .hover(move |style| style.bg(rgb(theme.hover)))
                    .on_click(cx.listener(move |shell, _, _, cx| {
                        shell.page = page;
                        cx.notify();
                    }))
                    .child(
                        svg()
                            .path(page.icon())
                            .size(px(16.0))
                            .text_color(rgb(if active { theme.accent } else { theme.muted })),
                    )
                    .child(page.label()),
            );
        }
        nav
    }

    fn header(theme: Theme, title: &str, subtitle: &str) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_size(px(24.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(theme.text))
                    .child(title.to_owned()),
            )
            .child(
                div()
                    .text_size(px(13.5))
                    .text_color(rgb(theme.muted))
                    .child(subtitle.to_owned()),
            )
    }

    fn home(&self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(Self::header(
                theme,
                "Welcome",
                "A native window, a menu bar, and GPU-rendered UI. No browser engine inside.",
            ))
            .child(
                theme
                    .card("State and events")
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(rgb(theme.muted))
                            .child(format!("The button has been clicked {} times.", self.clicks)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.0))
                            .child(theme.button("click", "Click me", true).on_click(cx.listener(
                                |shell, _, _, cx| {
                                    shell.clicks += 1;
                                    cx.notify();
                                },
                            )))
                            .child(theme.button("reset", "Reset", false).on_click(cx.listener(
                                |shell, _, _, cx| {
                                    shell.clicks = 0;
                                    cx.notify();
                                },
                            ))),
                    ),
            )
            .child(
                theme
                    .card("Things to try")
                    .child(bullet(theme, "⌘N opens another window; ⌘⇧T switches light and dark."))
                    .child(bullet(theme, "⌘⇧K brings the app to the front from anywhere (a global shortcut)."))
                    .child(bullet(theme, "Pick \"About\" in the app menu for a native dialog.")),
            )
            .into_any_element()
    }

    fn components(&self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(Self::header(
                theme,
                "Components",
                "Building blocks in src/components, styled from src/theme.rs.",
            ))
            .child(
                theme
                    .card("Text input")
                    .child(field(theme, self.name.clone()))
                    .child(field(theme, self.password.clone()))
                    .child(
                        div().flex().child(theme.button("greet", "Say hello", true).on_click(
                            cx.listener(|shell, _, _, cx| {
                                let name = shell.name.read(cx).content.to_string();
                                let name = name.trim();
                                if name.is_empty() {
                                    toast::error(cx, "Type a name first");
                                } else {
                                    toast::success(cx, format!("Hello, {name}!"));
                                }
                            }),
                        )),
                    ),
            )
            .child(
                theme
                    .card("Toasts")
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(8.0))
                            .child(theme.button("t-ok", "Success", false).on_click(
                                |_, _, cx| toast::success(cx, "Saved"),
                            ))
                            .child(theme.button("t-err", "Error", false).on_click(
                                |_, _, cx| toast::error(cx, "Something went wrong"),
                            ))
                            .child(theme.button("t-info", "Info", false).on_click(
                                |_, _, cx| toast::info(cx, "Just so you know"),
                            ))
                            .child(theme.button("t-dl", "Progress", false).on_click(
                                |_, _, cx| start_fake_download(cx),
                            )),
                    ),
            )
            .into_any_element()
    }

    fn markdown(&self, theme: Theme) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(Self::header(
                theme,
                "Markdown",
                "components/markdown.rs turns markdown into GPUI elements.",
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .p(px(18.0))
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(rgba(theme.border))
                    .bg(rgb(theme.surface))
                    .children(markdown::render("sample", SAMPLE_MARKDOWN, theme.markdown())),
            )
            .into_any_element()
    }

    fn settings(&self, theme: Theme) -> AnyElement {
        let (icon, label) = if theme.dark {
            ("icons/sun.svg", "Switch to light")
        } else {
            ("icons/moon.svg", "Switch to dark")
        };
        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(Self::header(theme, "Settings", "App-wide state kept in a GPUI global."))
            .child(
                theme.card("Appearance").child(
                    div().flex().child(
                        theme
                            .button("theme", label, true)
                            .child(svg().path(icon).size(px(15.0)).text_color(rgb(theme.on_accent)))
                            .on_click(|_, _, cx| {
                                let next = cx.global::<Theme>().toggled();
                                Theme::set(cx, next);
                            }),
                    ),
                ),
            )
            .child(
                theme
                    .card("About")
                    .child(bullet(theme, &format!("Version {}", env!("CARGO_PKG_VERSION"))))
                    .child(bullet(theme, "Built with GPUI, the UI framework from the Zed editor.")),
            )
            .into_any_element()
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        for input in [&self.name, &self.password] {
            input.update(cx, |input, _| {
                input.text_color = theme.text;
                input.placeholder_color = theme.muted;
            });
        }
        let page = match self.page {
            Page::Home => self.home(theme, cx),
            Page::Components => self.components(theme, cx),
            Page::Markdown => self.markdown(theme),
            Page::Settings => self.settings(theme),
        };
        div()
            .relative()
            .size_full()
            .flex()
            .font_family("Rethink Sans")
            .bg(rgb(theme.background))
            .text_color(rgb(theme.text))
            .child(self.sidebar(theme, cx))
            .child(
                div()
                    .id("content")
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_y_scroll()
                    .child(div().max_w(px(760.0)).p(px(32.0)).child(page)),
            )
            // Last child, so toasts overlay everything else.
            .children(toast::hub(cx))
    }
}

fn bullet(theme: Theme, text: &str) -> impl IntoElement {
    div()
        .flex()
        .gap(px(8.0))
        .text_size(px(13.0))
        .text_color(rgb(theme.muted))
        .child(div().text_color(rgb(theme.accent)).child("•"))
        .child(SharedString::from(text.to_owned()))
}

fn field(theme: Theme, input: Entity<TextInput>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .h(px(36.0))
        .px(px(12.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(rgba(theme.border))
        .bg(rgb(theme.background))
        .child(input)
}

/// Drive a progress toast from a background timer, as a real download would.
fn start_fake_download(cx: &mut gpui::App) {
    let Some(id) = toast::show_download(cx, "Downloading sample.zip", 0.0) else {
        return;
    };
    cx.spawn(async move |cx| {
        for step in 1..=20 {
            cx.background_executor()
                .timer(Duration::from_millis(120))
                .await;
            let progress = step as f32 / 20.0;
            let _ = cx.update(|cx| {
                toast::update_download(
                    cx,
                    id,
                    format!("Downloading sample.zip, {}%", (progress * 100.0) as u32),
                    progress,
                )
            });
        }
        let _ = cx.update(|cx| toast::finish_download(cx, id, "Downloaded sample.zip", true));
    })
    .detach();
}
