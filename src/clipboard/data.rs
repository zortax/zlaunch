//! Clipboard history data storage and search.

use super::item::{ClipboardContent, ClipboardItem};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use lazy_static::lazy_static;
use rusqlite::Connection;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::{Duration, UNIX_EPOCH};

const MAX_HISTORY: usize = 500;

/// Global clipboard history storage.
static CLIPBOARD_HISTORY: RwLock<Option<VecDeque<ClipboardItem>>> = RwLock::new(None);

lazy_static! {
    static ref DB: Mutex<Connection> = {
        let cache_dir = dirs::cache_dir().expect("No cache directory found");
        let path = cache_dir.join("zlaunch").join("clipboard.db");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(&path).expect("Failed to open clipboard database");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clipboard_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type TEXT NOT NULL,
                text_value TEXT,
                image_width INTEGER,
                image_height INTEGER,
                image_data BLOB,
                file_paths TEXT,
                rich_plain TEXT,
                rich_html TEXT,
                timestamp INTEGER NOT NULL
            );",
        )
        .expect("Failed to create clipboard history table");
        Mutex::new(conn)
    };
}


/// Initialize the clipboard history storage.
pub fn init() {
    let mut history = CLIPBOARD_HISTORY.write().unwrap();
    if history.is_none() {
        let db = DB.lock().unwrap();
        let mut stmt = db
            .prepare(
                "SELECT content_type, text_value, image_width, image_height,
                        image_data, file_paths, rich_plain, rich_html, timestamp
                 FROM clipboard_history ORDER BY id DESC",
            )
            .expect("Failed to prepare query");
        let items: Vec<ClipboardItem> = stmt
            .query_map([], |row| {
                let ts =
                    UNIX_EPOCH + Duration::from_secs(row.get::<_, i64>("timestamp")?.max(0) as u64);
                let content = match row.get::<_, String>("content_type")?.as_str() {
                    "text" => ClipboardContent::Text(row.get("text_value")?),
                    "image" => ClipboardContent::Image {
                        width: row.get::<_, i64>("image_width")? as usize,
                        height: row.get::<_, i64>("image_height")? as usize,
                        rgba_bytes: row.get("image_data")?,
                    },
                    "file_paths" => ClipboardContent::FilePaths(
                        serde_json::from_str(&row.get::<_, String>("file_paths")?)
                            .unwrap_or_default(),
                    ),
                    "rich_text" => ClipboardContent::RichText {
                        plain: row.get("rich_plain")?,
                        html: row.get("rich_html")?,
                    },
                    _ => {
                        return Err(rusqlite::Error::InvalidColumnName(
                            "unknown content_type".to_string(),
                        ));
                    }
                };
                Ok(ClipboardItem {
                    content,
                    timestamp: ts,
                })
            })
            .expect("Failed to get clipboard history")
            .filter_map(|r| r.ok())
            .collect();
        *history = Some(VecDeque::from(items));
    }
}

/// Add a new item to clipboard history.
/// If the item is identical to the most recent one, it won't be added.
pub fn add_item(content: ClipboardContent) {
    let mut history = CLIPBOARD_HISTORY.write().unwrap();
    let history = history.as_mut().expect("Clipboard history not initialized");

    // Don't add duplicate consecutive items
    if let Some(last) = history.front()
        && is_same_content(&last.content, &content)
    {
        return;
    }

    let item = ClipboardItem::new(content);
    let ts = item
        .timestamp
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs() as i64;

    let db = DB.lock().unwrap();
    match &item.content {
        ClipboardContent::Text(text) => {
            db.execute(
                "INSERT INTO clipboard_history (content_type,text_value,timestamp)
                 VALUES ('text',?1,?2)",
                rusqlite::params![text, ts],
            )
            .expect("Failed to insert item");
        }
        ClipboardContent::Image {
            width,
            height,
            rgba_bytes,
        } => {
            db.execute(
                "INSERT INTO clipboard_history (content_type,image_width,image_height,image_data,timestamp)
                 VALUES ('image',?1,?2,?3,?4)",
                rusqlite::params![*width as i64, *height as i64, rgba_bytes, ts],
            )
            .expect("Failed to insert clipboard item");
        }
        ClipboardContent::FilePaths(paths) => {
            let json = serde_json::to_string(paths).expect("Failed to serialize paths");
            db.execute(
                "INSERT INTO clipboard_history (content_type,file_paths,timestamp)
                 VALUES ('file_paths',?1,?2)",
                rusqlite::params![json, ts],
            )
            .expect("Failed to insert clipboard item");
        }
        ClipboardContent::RichText { plain, html } => {
            db.execute(
                "INSERT INTO clipboard_history (content_type,rich_plain,rich_html,timestamp)
                 VALUES ('rich_text',?1,?2,?3)",
                rusqlite::params![plain, html, ts],
            )
            .expect("Failed to insert clipboard item");
        }
    }
    db.execute(
        "DELETE FROM clipboard_history WHERE id <= (
            SELECT id FROM clipboard_history ORDER BY id DESC LIMIT 1 OFFSET ?
        )",
        rusqlite::params![MAX_HISTORY as i64],
    )
    .ok();
    drop(db);

    history.push_front(item);
}

/// Check if two clipboard contents are the same.
fn is_same_content(a: &ClipboardContent, b: &ClipboardContent) -> bool {
    match (a, b) {
        (ClipboardContent::Text(a), ClipboardContent::Text(b)) => a == b,
        (
            ClipboardContent::Image {
                width: w1,
                height: h1,
                rgba_bytes: b1,
            },
            ClipboardContent::Image {
                width: w2,
                height: h2,
                rgba_bytes: b2,
            },
        ) => w1 == w2 && h1 == h2 && b1 == b2,
        (ClipboardContent::FilePaths(a), ClipboardContent::FilePaths(b)) => a == b,
        (
            ClipboardContent::RichText {
                plain: p1,
                html: h1,
            },
            ClipboardContent::RichText {
                plain: p2,
                html: h2,
            },
        ) => p1 == p2 && h1 == h2,
        _ => false,
    }
}

/// Get all clipboard items, optionally filtered by a search query.
pub fn search_items(query: &str) -> Vec<ClipboardItem> {
    let history = CLIPBOARD_HISTORY.read().unwrap();
    let history = history.as_ref().expect("Clipboard history not initialized");

    if query.is_empty() {
        return history.iter().cloned().collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<(ClipboardItem, i64)> = history
        .iter()
        .filter_map(|item| {
            let search_text = match &item.content {
                ClipboardContent::Text(text) => text.clone(),
                ClipboardContent::Image { .. } => "image".to_string(),
                ClipboardContent::FilePaths(paths) => paths
                    .iter()
                    .filter_map(|p| p.to_str())
                    .collect::<Vec<_>>()
                    .join(" "),
                ClipboardContent::RichText { plain, .. } => plain.clone(),
            };

            matcher
                .fuzzy_match(&search_text, query)
                .map(|score| (item.clone(), score))
        })
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().map(|(item, _)| item).collect()
}

/// Get the total number of items in history.
pub fn item_count() -> usize {
    let history = CLIPBOARD_HISTORY.read().unwrap();
    history.as_ref().map(|h| h.len()).unwrap_or(0)
}

/// Clear all clipboard history.
#[allow(dead_code)]
pub fn clear_history() {
    let mut history = CLIPBOARD_HISTORY.write().unwrap();
    if let Some(h) = history.as_mut() {
        h.clear();
    }
    let db = DB.lock().unwrap();
    db.execute("DELETE FROM clipboard_history", [])
        .expect("Failed to clear clipboard history");
}
