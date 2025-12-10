//! Embedded assets for zlaunch.
//!
//! This module provides embedded Phosphor icons (bold style) for use in the launcher,
//! combined with gpui-component-assets for the UI component icons.

use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use std::borrow::Cow;

/// Embedded Phosphor icons for zlaunch.
#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/*.svg"]
struct PhosphorAssets;

/// Combined asset source that serves both zlaunch's Phosphor icons
/// and gpui-component's UI icons.
pub struct CombinedAssets;

impl AssetSource for CombinedAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        // First try our Phosphor icons
        if let Some(file) = PhosphorAssets::get(path) {
            return Ok(Some(file.data));
        }

        // Fall back to gpui-component-assets
        gpui_component_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut result: Vec<SharedString> = PhosphorAssets::iter()
            .filter_map(|p| {
                p.starts_with(path)
                    .then(|| SharedString::from(p.to_string()))
            })
            .collect();

        // Add gpui-component assets
        if let Ok(component_assets) = gpui_component_assets::Assets.list(path) {
            result.extend(component_assets);
        }

        Ok(result)
    }
}

/// Icon names for Phosphor bold icons.
/// These correspond to SVG files in assets/icons/.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhosphorIcon {
    Power,
    Reboot,
    Moon,
    Lock,
    SignOut,
    Smiley,
    Terminal,
    Clipboard,
    ClipboardText,
    File,
    FileText,
    FileImage,
    Image,
    MagnifyingGlass,
    Globe,
    BookOpen,
    YoutubeLogo,
    Brain,
    Palette,
}

impl PhosphorIcon {
    /// Get the asset path for this icon.
    pub fn path(self) -> &'static str {
        match self {
            Self::Power => "icons/power.svg",
            Self::Reboot => "icons/reboot.svg",
            Self::Moon => "icons/moon.svg",
            Self::Lock => "icons/lock.svg",
            Self::SignOut => "icons/sign-out.svg",
            Self::Smiley => "icons/smiley.svg",
            Self::Terminal => "icons/terminal.svg",
            Self::Clipboard => "icons/clipboard-bold.svg",
            Self::ClipboardText => "icons/clipboard-text-bold.svg",
            Self::File => "icons/file-bold.svg",
            Self::FileText => "icons/file-text-bold.svg",
            Self::FileImage => "icons/file-image-bold.svg",
            Self::Image => "icons/image-bold.svg",
            Self::MagnifyingGlass => "icons/magnifying-glass-bold.svg",
            Self::Globe => "icons/globe-bold.svg",
            Self::BookOpen => "icons/book-open-bold.svg",
            Self::YoutubeLogo => "icons/youtube-logo-bold.svg",
            Self::Brain => "icons/brain-bold.svg",
            Self::Palette => "icons/palette-bold.svg",
        }
    }

    /// Try to get a PhosphorIcon from an icon name string.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "power" => Some(Self::Power),
            "reboot" => Some(Self::Reboot),
            "moon" => Some(Self::Moon),
            "lock" => Some(Self::Lock),
            "sign-out" => Some(Self::SignOut),
            "smiley" => Some(Self::Smiley),
            "terminal" => Some(Self::Terminal),
            "clipboard" => Some(Self::Clipboard),
            "clipboard-text" => Some(Self::ClipboardText),
            "file" => Some(Self::File),
            "file-text" => Some(Self::FileText),
            "file-image" => Some(Self::FileImage),
            "image" => Some(Self::Image),
            "magnifying-glass" => Some(Self::MagnifyingGlass),
            "globe" => Some(Self::Globe),
            "book-open" => Some(Self::BookOpen),
            "youtube-logo" => Some(Self::YoutubeLogo),
            "brain" => Some(Self::Brain),
            "palette" => Some(Self::Palette),
            _ => None,
        }
    }
}
