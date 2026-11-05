//! App-wide colors as a GPUI global. Read with `cx.global::<Theme>()` in any
//! `render`; switch with `Theme::set(cx, ...)`, which redraws every window.
use crate::components::markdown::MarkdownColors;
use gpui::{App, FontWeight, Global, SharedString, div, prelude::*, px, rgb, rgba};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub dark: bool,
    pub background: u32,
    pub sidebar: u32,
    pub surface: u32,
    pub hover: u32,
    /// RGBA, so it works on any background.
    pub border: u32,
    pub text: u32,
    pub muted: u32,
    pub accent: u32,
    /// Text drawn on top of `accent`.
    pub on_accent: u32,
    pub link: u32,
    pub code: u32,
}

impl Global for Theme {}

impl Theme {
    pub fn dark() -> Self {
        Self {
            dark: true,
            background: 0x101214,
            sidebar: 0x0a0c0e,
            surface: 0x181b1e,
            hover: 0x22262a,
            border: 0xffffff1f,
            text: 0xf2efe6,
            muted: 0x9aa09e,
            accent: 0xe8d6b7,
            on_accent: 0x101416,
            link: 0x5fb7e8,
            code: 0xf0d9a8,
        }
    }

    pub fn light() -> Self {
        Self {
            dark: false,
            background: 0xf7f5f0,
            sidebar: 0xece9e2,
            surface: 0xffffff,
            hover: 0xe6e2d9,
            border: 0x0000001f,
            text: 0x1a1c1e,
            muted: 0x6a6f72,
            accent: 0x2f6f62,
            on_accent: 0xffffff,
            link: 0x1f6fb2,
            code: 0x8a4b12,
        }
    }

    pub fn toggled(self) -> Self {
        if self.dark { Self::light() } else { Self::dark() }
    }

    pub fn set(cx: &mut App, theme: Theme) {
        cx.set_global(theme);
        cx.refresh_windows();
    }

    pub fn markdown(&self) -> MarkdownColors {
        MarkdownColors {
            text: self.text,
            muted: self.muted,
            link: self.link,
            code: self.code,
            overlay: if self.dark { 0xffffff1a } else { 0x0000000f },
        }
    }

    /// A clickable button; chain `.on_click(...)` onto it.
    pub fn button(
        &self,
        id: impl Into<gpui::ElementId>,
        label: impl Into<SharedString>,
        primary: bool,
    ) -> gpui::Stateful<gpui::Div> {
        let theme = *self;
        div()
            .id(id)
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(7.0))
            .h(px(34.0))
            .px(px(14.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(rgba(self.border))
            .bg(rgb(if primary { self.accent } else { self.surface }))
            .text_color(rgb(if primary { self.on_accent } else { self.text }))
            .text_size(px(13.0))
            .font_weight(FontWeight::SEMIBOLD)
            .cursor_pointer()
            .when(primary, |button| button.hover(|style| style.opacity(0.88)))
            .when(!primary, |button| {
                button.hover(move |style| style.bg(rgb(theme.hover)))
            })
            .child(label.into())
    }

    /// A bordered panel with a title, for grouping content.
    pub fn card(&self, title: impl Into<SharedString>) -> gpui::Div {
        div()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .p(px(18.0))
            .rounded(px(10.0))
            .border_1()
            .border_color(rgba(self.border))
            .bg(rgb(self.surface))
            .child(
                div()
                    .text_size(px(14.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(self.text))
                    .child(title.into()),
            )
    }
}
