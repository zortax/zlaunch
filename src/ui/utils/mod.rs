pub mod color;
mod file_type;

pub use color::{parse_color, Color};
pub use file_type::{
    classify_file, is_image_ext, is_text_ext, should_preview_as_image, should_preview_as_text,
    FileType,
};
