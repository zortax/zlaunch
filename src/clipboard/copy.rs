//! Clipboard copy utilities.

use arboard::Clipboard;

/// Copy text to the system clipboard.
///
/// Returns `Ok(())` on success, or an error message on failure.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;

    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))
}

/// Copy an RGBA image to the system clipboard.
///
/// Returns `Ok(())` on success, or an error message on failure.
pub fn copy_image_to_clipboard(
    width: usize,
    height: usize,
    rgba_bytes: &[u8],
) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;

    let image_data = arboard::ImageData {
        width,
        height,
        bytes: std::borrow::Cow::Borrowed(rgba_bytes),
    };

    clipboard
        .set_image(image_data)
        .map_err(|e| format!("Failed to copy image to clipboard: {}", e))
}
