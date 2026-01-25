//! Launcher state management.
//!
//! Contains mode state tracking and view mode definitions.

use crate::config::LauncherMode;

/// Tracks the active modes list and current mode index.
#[derive(Clone, Debug)]
pub struct ModeState {
    /// Ordered list of enabled modes
    pub modes: Vec<LauncherMode>,
    /// Index of the currently active mode
    pub current_index: usize,
}

impl ModeState {
    /// Create a new mode state from a list of modes.
    pub fn new(modes: Vec<LauncherMode>) -> Self {
        Self {
            modes,
            current_index: 0,
        }
    }

    /// Get the currently active mode.
    pub fn current_mode(&self) -> &LauncherMode {
        &self.modes[self.current_index]
    }

    /// Switch to the next mode (wraps around).
    pub fn next_mode(&mut self) {
        self.current_index = (self.current_index + 1) % self.modes.len();
    }

    /// Switch to the previous mode (wraps around).
    pub fn prev_mode(&mut self) {
        self.current_index = if self.current_index == 0 {
            self.modes.len() - 1
        } else {
            self.current_index - 1
        };
    }

    /// Check if there are multiple modes (mode switching enabled).
    pub fn has_multiple_modes(&self) -> bool {
        self.modes.len() > 1
    }
}

impl Default for ModeState {
    fn default() -> Self {
        Self::new(vec![LauncherMode::Combined])
    }
}

/// The current view mode of the launcher.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    /// Main launcher view showing apps, windows, commands.
    #[default]
    Main,
    /// Emoji picker grid view.
    EmojiPicker,
    /// Clipboard history view.
    ClipboardHistory,
    /// AI response streaming view.
    AiResponse,
    /// Theme picker view.
    ThemePicker,
}
