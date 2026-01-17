//! Clipboard copy utilities.

use arboard::Clipboard;

use crate::error::ClipboardError;

/// Copy text to the system clipboard.
///
/// Returns `Ok(())` on success, or a `ClipboardError` on failure.
pub fn copy_to_clipboard(text: &str) -> Result<(), ClipboardError> {
    let mut clipboard =
        Clipboard::new().map_err(|e| ClipboardError::AccessFailed(e.to_string()))?;

    clipboard
        .set_text(text.to_string())
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
    let mut clipboard =
        Clipboard::new().map_err(|e| ClipboardError::AccessFailed(e.to_string()))?;

    let image_data = arboard::ImageData {
        width,
        height,
        bytes: std::borrow::Cow::Borrowed(rgba_bytes),
    };

    clipboard
        .set_image(image_data)
        .map_err(|e| ClipboardError::CopyFailed(e.to_string()))
}
