//! Assets compiled into the binary, so the app runs from `cargo run` and from a
//! packaged `.app` alike. Add a file under `assets/` and a line to `ASSETS`;
//! then use it by path, e.g. `svg().path("icons/home.svg")`.
use anyhow::Result;
use gpui::{AssetSource, SharedString};
use std::borrow::Cow;

macro_rules! embed {
    ($($path:literal),* $(,)?) => {
        &[$(($path, include_bytes!(concat!("../assets/", $path)) as &[u8])),*]
    };
}

const ASSETS: &[(&str, &[u8])] = embed![
    "icons/home.svg",
    "icons/blocks.svg",
    "icons/text.svg",
    "icons/settings.svg",
    "icons/sun.svg",
    "icons/moon.svg",
];

/// Font files registered with GPUI's text system at startup.
pub const FONTS: &[&[u8]] = &[
    include_bytes!("../assets/fonts/RethinkSans-Regular.ttf"),
    include_bytes!("../assets/fonts/RethinkSans-SemiBold.ttf"),
];

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(ASSETS
            .iter()
            .find(|(name, _)| *name == path)
            .map(|(_, bytes)| Cow::Borrowed(*bytes)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(ASSETS
            .iter()
            .filter(|(name, _)| name.starts_with(path))
            .map(|(name, _)| (*name).into())
            .collect())
    }
}
