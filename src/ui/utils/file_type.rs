use std::path::Path;

/// Represents the type of file for preview/display purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Image files (png, jpg, gif, etc.)
    Image,
    /// Text files (txt, md, rs, json, etc.)
    Text,
    /// Other/unknown file types
    Other,
}

/// Determine the file type based on the file extension
pub fn classify_file(path: &Path) -> FileType {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    if is_image_ext(extension) {
        FileType::Image
    } else if is_text_ext(extension) {
        FileType::Text
    } else {
        FileType::Other
    }
}

/// Check if an extension is an image format
pub fn is_image_ext(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico"
    )
}

/// Check if an extension is a text format
pub fn is_text_ext(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "txt" | "md" | "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "json"
            | "yaml" | "yml" | "toml" | "html" | "css" | "sh" | "bash"
            | "c" | "cpp" | "h" | "hpp" | "go" | "java" | "kt" | "swift"
            | "xml" | "ini" | "conf" | "log"
    )
}

/// Check if a file should be previewed as an image
pub fn should_preview_as_image(path: &Path) -> bool {
    classify_file(path) == FileType::Image
}

/// Check if a file should be previewed as text
pub fn should_preview_as_text(path: &Path) -> bool {
    classify_file(path) == FileType::Text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_classify_image() {
        assert_eq!(classify_file(&PathBuf::from("test.png")), FileType::Image);
        assert_eq!(classify_file(&PathBuf::from("test.JPG")), FileType::Image);
        assert_eq!(classify_file(&PathBuf::from("test.svg")), FileType::Image);
    }

    #[test]
    fn test_classify_text() {
        assert_eq!(classify_file(&PathBuf::from("test.txt")), FileType::Text);
        assert_eq!(classify_file(&PathBuf::from("test.md")), FileType::Text);
        assert_eq!(classify_file(&PathBuf::from("test.rs")), FileType::Text);
    }

    #[test]
    fn test_classify_other() {
        assert_eq!(classify_file(&PathBuf::from("test.pdf")), FileType::Other);
        assert_eq!(classify_file(&PathBuf::from("test")), FileType::Other);
    }
}
