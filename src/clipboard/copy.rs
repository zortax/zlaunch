//! Clipboard copy utilities.

use wl_clipboard_rs::copy::{MimeType, Options, Source};

use super::data;
use crate::error::ClipboardError;

/// Copy text to the system clipboard.
///
/// Returns `Ok(())` on success, or a `ClipboardError` on failure.
pub fn copy_to_clipboard(text: &str) -> Result<(), ClipboardError> {
    Options::new()
        .copy(Source::Bytes(text.as_bytes().into()), MimeType::Text)
        .map_err(|e| ClipboardError::CopyFailed(e.to_string()))
}

/// Copy an RGBA image to the system clipboard.
///
/// Returns `Ok(())` on success, or a `ClipboardError` on failure.
pub fn copy_image_to_clipboard(
    width: usize,
    height: usize,
    rgba_bytes: &[u8],
) -> Result<(), ClipboardError> {
    let png = data::encode_png(width, height, rgba_bytes)
        .map_err(|e| ClipboardError::CopyFailed(e.to_string()))?;
    Options::new()
        .copy(
            Source::Bytes(png.into()),
            MimeType::Specific("image/png".into()),
        )
        .map_err(|e| ClipboardError::CopyFailed(e.to_string()))
}
