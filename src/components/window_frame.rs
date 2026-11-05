//! Window chrome for when the app has to draw its own (client-side decorations).
//!
//! macOS, X11 and Wayland compositors with server-side decorations (KDE, Sway)
//! draw the title bar themselves, and `frame` just returns your content. GNOME
//! on Wayland draws nothing, so there `frame` adds a title bar you can drag,
//! double-click to maximize and right-click for the window menu, buttons for
//! the controls the compositor supports, and resize edges.
use gpui::{
    AnyElement, CursorStyle, Decorations, FontWeight, HitboxBehavior, MouseButton, Pixels, Point,
    ResizeEdge, Size, Window, canvas, div, prelude::*, px, rgb, rgba,
};

use crate::theme::Theme;

const TITLEBAR_HEIGHT: f32 = 36.0;
/// Width of the invisible band along the window edges that resizes it.
const RESIZE_BAND: f32 = 6.0;

/// `content` inside the window chrome this platform needs, if any.
pub fn frame(
    window: &mut Window,
    theme: Theme,
    title: &'static str,
    content: impl IntoElement,
) -> AnyElement {
    let Decorations::Client { tiling } = window.window_decorations() else {
        return div().size_full().child(content).into_any_element();
    };
    let controls = window.window_controls();
    let band = px(RESIZE_BAND);

    let titlebar = div()
        .id("titlebar")
        .flex_none()
        .flex()
        .items_center()
        .justify_between()
        .h(px(TITLEBAR_HEIGHT))
        .pl(px(14.0))
        .pr(px(6.0))
        .bg(rgb(theme.sidebar))
        .border_b_1()
        .border_color(rgba(theme.border))
        .on_mouse_down(MouseButton::Left, |event, window, _| {
            if event.click_count == 2 {
                window.zoom_window();
            } else {
                window.start_window_move();
            }
        })
        .on_mouse_down(MouseButton::Right, |event, window, _| {
            window.show_window_menu(event.position);
        })
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(theme.muted))
                .child(title),
        )
        .child(
            div()
                .flex()
                .gap(px(2.0))
                .when(controls.minimize, |row| {
                    row.child(control(theme, "minimize", "–", |window| {
                        window.minimize_window()
                    }))
                })
                .when(controls.maximize, |row| {
                    row.child(control(theme, "maximize", "□", |window| window.zoom_window()))
                })
                .child(control(theme, "close", "✕", |window| window.remove_window())),
        );

    div()
        .id("window-frame")
        .relative()
        .size_full()
        .flex()
        .flex_col()
        .overflow_hidden()
        .when(!tiling.is_tiled(), |frame| {
            frame.rounded(px(10.0)).border_1().border_color(rgba(theme.border))
        })
        .child(titlebar)
        .child(div().flex_1().min_h_0().child(content))
        // Resize edges: a hitbox over the whole window that shows the resize
        // cursor near an edge, and starts the resize on mouse down there.
        .child(
            canvas(
                |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
                move |_, hitbox, window, _| {
                    let size = window.window_bounds().get_bounds().size;
                    if let Some(edge) = resize_edge(window.mouse_position(), band, size, tiling) {
                        window.set_cursor_style(cursor_for(edge), &hitbox);
                    }
                },
            )
            .absolute()
            .size_full(),
        )
        .on_mouse_move(|_, window, _| window.refresh())
        .on_mouse_down(MouseButton::Left, move |event, window, _| {
            let size = window.window_bounds().get_bounds().size;
            if let Some(edge) = resize_edge(event.position, band, size, tiling) {
                window.start_window_resize(edge);
            }
        })
        .into_any_element()
}

fn control(
    theme: Theme,
    id: &'static str,
    glyph: &'static str,
    action: fn(&mut Window),
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .size(px(28.0))
        .rounded(px(6.0))
        .text_size(px(13.0))
        .text_color(rgb(theme.muted))
        .cursor_pointer()
        .hover(move |style| style.bg(rgb(theme.hover)).text_color(rgb(theme.text)))
        // Keep the press from reaching the title bar, which would start a drag.
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(move |_, window, _| action(window))
        .child(glyph)
}

fn resize_edge(
    pos: Point<Pixels>,
    band: Pixels,
    size: Size<Pixels>,
    tiling: gpui::Tiling,
) -> Option<ResizeEdge> {
    let top = pos.y < band && !tiling.top;
    let bottom = pos.y > size.height - band && !tiling.bottom;
    let left = pos.x < band && !tiling.left;
    let right = pos.x > size.width - band && !tiling.right;
    Some(match (top, bottom, left, right) {
        (true, _, true, _) => ResizeEdge::TopLeft,
        (true, _, _, true) => ResizeEdge::TopRight,
        (_, true, true, _) => ResizeEdge::BottomLeft,
        (_, true, _, true) => ResizeEdge::BottomRight,
        (true, ..) => ResizeEdge::Top,
        (_, true, ..) => ResizeEdge::Bottom,
        (_, _, true, _) => ResizeEdge::Left,
        (_, _, _, true) => ResizeEdge::Right,
        _ => return None,
    })
}

fn cursor_for(edge: ResizeEdge) -> CursorStyle {
    match edge {
        ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
        ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
        ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
        ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Tiling, point, size};

    #[test]
    fn edges_and_tiling() {
        let window = size(px(800.0), px(600.0));
        let band = px(6.0);
        let free = Tiling::default();
        let edge = |x: f32, y: f32, tiling| resize_edge(point(px(x), px(y)), band, window, tiling);
        assert_eq!(edge(2.0, 2.0, free), Some(ResizeEdge::TopLeft));
        assert_eq!(edge(798.0, 598.0, free), Some(ResizeEdge::BottomRight));
        assert_eq!(edge(400.0, 2.0, free), Some(ResizeEdge::Top));
        assert_eq!(edge(2.0, 300.0, free), Some(ResizeEdge::Left));
        assert_eq!(edge(400.0, 300.0, free), None);
        // A side tiled against the screen edge or another window can't be dragged.
        let left_tiled = Tiling { left: true, ..free };
        assert_eq!(edge(2.0, 300.0, left_tiled), None);
        assert_eq!(edge(2.0, 2.0, left_tiled), Some(ResizeEdge::Top));
    }
}
