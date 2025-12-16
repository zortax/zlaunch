//! Clipboard history management and copy utilities.

mod copy;
pub mod data;
pub mod item;
pub mod monitor;

pub use copy::{copy_image_to_clipboard, copy_to_clipboard};
pub use item::{ClipboardContent, ClipboardItem};
