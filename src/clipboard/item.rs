//! Clipboard item data structures.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

/// Represents a single clipboard history entry.
#[derive(Clone, Debug)]
pub struct ClipboardItem {
    pub id: i64,
    pub content: ClipboardContent,
    pub timestamp: SystemTime,
}

/// The content type of a clipboard item.
#[derive(Clone, Debug)]
pub enum ClipboardContent {
    /// Plain text content
    Text(String),
    /// Image with dimensions
    Image {
        width: usize,
        height: usize,
        data: ImageData,
    },
    /// File path(s) copied from file manager
    FilePaths(Vec<PathBuf>),
    /// Rich text / HTML content
    RichText { plain: String, html: String },
}

/// PNG-encoded data of a clipboard image.
///
/// Images are only kept in memory until they are stored in the database,
/// afterwards they are loaded on demand (see `data::image_png`).
#[derive(Clone, Debug)]
pub enum ImageData {
    /// PNG data held in memory
    Png(Arc<[u8]>),
    /// PNG data stored in the clipboard database
    Stored,
}

impl ClipboardItem {
    /// Create a new clipboard item with the current timestamp.
    pub fn new(content: ClipboardContent) -> Self {
        Self {
            id: 0,
            content,
            timestamp: SystemTime::now(),
        }
    }

    /// Get a short preview string for display in the list.
    pub fn preview(&self) -> String {
        const MAX_LENGTH: usize = 30;

        match &self.content {
            ClipboardContent::Text(text) => {
                let first_line = text.trim_start().lines().next().unwrap_or("");
                truncate_preview_line(first_line, MAX_LENGTH)
            }
            ClipboardContent::Image { .. } => "[Image]".to_string(),
            ClipboardContent::FilePaths(paths) => {
                if paths.len() == 1 {
                    paths[0]
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("[File]")
                        .to_string()
                } else {
                    format!("[{} files]", paths.len())
                }
            }
            ClipboardContent::RichText { plain, .. } => {
                let first_line = plain.trim_start().lines().next().unwrap_or("");
                truncate_preview_line(first_line, MAX_LENGTH)
            }
        }
    }

    /// Get the full content as a string for preview panel.
    pub fn full_content(&self) -> String {
        match &self.content {
            ClipboardContent::Text(text) => text.clone(),
            ClipboardContent::Image { .. } => "[Image preview]".to_string(),
            ClipboardContent::FilePaths(paths) => paths
                .iter()
                .filter_map(|p| p.to_str())
                .collect::<Vec<_>>()
                .join("\n"),
            ClipboardContent::RichText { plain, .. } => plain.clone(),
        }
    }

    /// Check if this item is a text file that can be previewed.
    pub fn is_previewable_file(&self) -> bool {
        if let ClipboardContent::FilePaths(paths) = &self.content
            && paths.len() == 1
            && let Some(ext) = paths[0].extension().and_then(|e| e.to_str())
        {
            return matches!(
                ext,
                "txt"
                    | "md"
                    | "rs"
                    | "py"
                    | "js"
                    | "ts"
                    | "json"
                    | "yaml"
                    | "yml"
                    | "toml"
                    | "html"
                    | "css"
                    | "sh"
            );
        }
        false
    }
}

/// Truncate wihtout splitting emojis
fn truncate_preview_line(line: &str, max: usize) -> String {
    let truncated: String = line.chars().take(max).collect();

    if truncated.len() < line.len() {
        format!("{}...", truncated)
    } else {
        truncated
    }
}
