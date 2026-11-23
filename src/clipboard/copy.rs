//! Clipboard copy utilities.

use wl_clipboard_rs::copy::{MimeType, Options, Source};

use crate::error::ClipboardError;

/// Copy text to the system clipboard.
///
/// Returns `Ok(())` on success, or a `ClipboardError` on failure.
pub fn copy_to_clipboard(text: &str) -> Result<(), ClipboardError> {
    Options::new()
        .copy(Source::Bytes(text.as_bytes().into()), MimeType::Text)
        .map_err(|e| ClipboardError::CopyFailed(e.to_string()))
}

/// Copy PNG-encoded image data to the system clipboard.
///
/// Returns `Ok(())` on success, or a `ClipboardError` on failure.
pub fn copy_image_to_clipboard(png: &[u8]) -> Result<(), ClipboardError> {
    Options::new()
        .copy(
            Source::Bytes(png.into()),
            MimeType::Specific("image/png".into()),
        )
        .map_err(|e| ClipboardError::CopyFailed(e.to_string()))
}
