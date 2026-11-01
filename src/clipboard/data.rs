//! Clipboard history data storage and search.

use super::item::{ClipboardContent, ClipboardItem, ImageData};
use crate::config;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock};
use std::time::{Duration, UNIX_EPOCH};
use tracing::{debug, error, warn};

/// Global clipboard history storage.
static CLIPBOARD_HISTORY: RwLock<Option<VecDeque<ClipboardItem>>> = RwLock::new(None);

/// Clipboard history database, `None` if it could not be opened.
static DB: LazyLock<Option<Mutex<Connection>>> = LazyLock::new(|| match open_db() {
    Ok(conn) => Some(Mutex::new(conn)),
    Err(e) => {
        error!(
            "Failed to open clipboard database, history will not be persisted: {}",
            e
        );
        None
    }
});

/// Next id for items that could not be stored in the database.
/// Negative so it can never collide with a database row id.
static NEXT_MEMORY_ID: AtomicI64 = AtomicI64::new(-1);

/// Default max number of clipboard history entries.
const DEFAULT_MAX_HISTORY: usize = 500;

fn open_db() -> Result<Connection, Box<dyn std::error::Error>> {
    let dir = dirs::cache_dir()
        .ok_or("No cache directory found")?
        .join("zlaunch");
    std::fs::create_dir_all(&dir)?;
    let conn = Connection::open(dir.join("clipboard.db"))?;
    create_schema(&conn)?;
    Ok(conn)
}

fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
    // This feature makes sure that the clipboard.db shrinks when entries are deleted.
    conn.execute_batch("PRAGMA auto_vacuum = FULL;")?;

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
}

/// Run `f` with the database connection, logging any error.
/// Returns `None` if the database is unavailable or `f` failed.
fn with_db<T>(f: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Option<T> {
    let db = DB.as_ref()?.lock().unwrap();
    f(&db)
        .inspect_err(|e| error!("Clipboard database error: {}", e))
        .ok()
}

/// Max number of clipboard history entries, from config.
fn max_history() -> usize {
    config::config()
        .max_clipboard_history
        .filter(|&v| v > 0)
        .unwrap_or(DEFAULT_MAX_HISTORY)
}

/// Delete all but the newest `max` entries from the database.
fn prune_db(db: &Connection, max: usize) -> rusqlite::Result<()> {
    let deleted = db.execute(
        "DELETE FROM clipboard_history WHERE id <= (
            SELECT id FROM clipboard_history ORDER BY id DESC LIMIT 1 OFFSET ?1
        )",
        params![max as i64],
    )?;
    if deleted > 0 {
        debug!(
            "Pruned {} clipboard history entries (max history length is {})",
            deleted, max
        );
    }
    Ok(())
}

/// Load all clipboard history entries from the database, newest first.
fn load_items(db: &Connection) -> rusqlite::Result<Vec<ClipboardItem>> {
    let mut stmt = db.prepare(
        "SELECT id, content_type, text_value, image_width, image_height,
                file_paths, rich_plain, rich_html, timestamp
         FROM clipboard_history ORDER BY id DESC",
    )?;
    let items = stmt
        .query_map([], |row| {
            let id = row.get::<_, i64>("id")?;
            let ts =
                UNIX_EPOCH + Duration::from_secs(row.get::<_, i64>("timestamp")?.max(0) as u64);
            let content = match row.get::<_, String>("content_type")?.as_str() {
                "text" => ClipboardContent::Text(row.get("text_value")?),
                "image" => ClipboardContent::Image {
                    width: row.get::<_, i64>("image_width")? as usize,
                    height: row.get::<_, i64>("image_height")? as usize,
                    data: ImageData::Stored,
                },
                "file_paths" => ClipboardContent::FilePaths(
                    serde_json::from_str(&row.get::<_, String>("file_paths")?).unwrap_or_default(),
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
                id,
                content,
                timestamp: ts,
            })
        })?
        .filter_map(|r| {
            r.inspect_err(|e| warn!("Skipping clipboard entry: {}", e))
                .ok()
        })
        .collect();
    Ok(items)
}

/// Initialize the clipboard history storage.
pub fn init() {
    let mut history = CLIPBOARD_HISTORY.write().unwrap();
    if history.is_none() {
        let items = with_db(|db| {
            // Apply a possibly lowered limit from the config
            prune_db(db, max_history())?;
            migrate_raw_images(db)?;
            load_items(db)
        })
        .unwrap_or_default();
        *history = Some(VecDeque::from(items));
    }
}

/// Insert a clipboard item into the database, returning its row id.
fn insert_item(db: &Connection, item: &ClipboardItem) -> rusqlite::Result<i64> {
    let ts = item
        .timestamp
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs() as i64;

    match &item.content {
        ClipboardContent::Text(text) => db.execute(
            "INSERT INTO clipboard_history (content_type,text_value,timestamp)
             VALUES ('text',?1,?2)",
            params![text, ts],
        )?,
        ClipboardContent::Image {
            width,
            height,
            data,
        } => {
            let ImageData::Png(png) = data else {
                return Err(rusqlite::Error::ToSqlConversionFailure(
                    "image data is not loaded".into(),
                ));
            };
            db.execute(
                "INSERT INTO clipboard_history (content_type,image_width,image_height,image_data,timestamp)
                 VALUES ('image',?1,?2,?3,?4)",
                params![*width as i64, *height as i64, &png[..], ts],
            )?
        }
        ClipboardContent::FilePaths(paths) => {
            let json = serde_json::to_string(paths)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
            db.execute(
                "INSERT INTO clipboard_history (content_type,file_paths,timestamp)
                 VALUES ('file_paths',?1,?2)",
                params![json, ts],
            )?
        }
        ClipboardContent::RichText { plain, html } => db.execute(
            "INSERT INTO clipboard_history (content_type,rich_plain,rich_html,timestamp)
             VALUES ('rich_text',?1,?2,?3)",
            params![plain, html, ts],
        )?,
    };
    Ok(db.last_insert_rowid())
}

/// Add a new item to clipboard history.
/// If the item is identical to the most recent one, it won't be added.
pub fn add_item(content: ClipboardContent) {
    let mut history = CLIPBOARD_HISTORY.write().unwrap();
    let history = history.as_mut().expect("Clipboard history not initialized");

    // Don't add duplicate consecutive items
    if let Some(last) = history.front()
        && is_same_content(last, &content)
    {
        return;
    }

    let mut item = ClipboardItem::new(content);
    let max = max_history();

    let id = with_db(|db| {
        let id = insert_item(db, &item)?;
        prune_db(db, max)?;
        Ok(id)
    });
    // Stored images are loaded from the database on demand
    if id.is_some()
        && let ClipboardContent::Image { data, .. } = &mut item.content
    {
        *data = ImageData::Stored;
    }
    // Fall back to keeping the item in memory only if it can't be stored
    item.id = id.unwrap_or_else(|| NEXT_MEMORY_ID.fetch_sub(1, Ordering::Relaxed));

    history.push_front(item);
    history.truncate(max);
}

/// Delete a clipboard item from the history and the database.
pub fn delete_item(id: i64) {
    let mut history = CLIPBOARD_HISTORY.write().unwrap();
    let history = history.as_mut().expect("Clipboard history not initialized");

    history.retain(|item| item.id != id);

    with_db(|db| db.execute("DELETE FROM clipboard_history WHERE id = ?1", params![id]));
}

/// PNG file signature, used to tell PNG data apart from legacy raw RGBA data.
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// Encode raw RGBA pixels as PNG.
pub fn encode_png(width: usize, height: usize, rgba: &[u8]) -> image::ImageResult<Vec<u8>> {
    use image::ImageEncoder;
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};

    let mut png = Vec::new();
    PngEncoder::new_with_quality(&mut png, CompressionType::Fast, FilterType::Adaptive)
        .write_image(
            rgba,
            width as u32,
            height as u32,
            image::ExtendedColorType::Rgba8,
        )?;
    Ok(png)
}

/// Get the PNG data of a clipboard image, loading it from the database if needed.
pub fn image_png(item: &ClipboardItem) -> Option<Arc<[u8]>> {
    match &item.content {
        ClipboardContent::Image {
            data: ImageData::Png(png),
            ..
        } => Some(png.clone()),
        ClipboardContent::Image {
            data: ImageData::Stored,
            ..
        } => with_db(|db| {
            db.query_row(
                "SELECT image_data FROM clipboard_history WHERE id = ?1",
                params![item.id],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()
        })
        .flatten()
        .map(Into::into),
        _ => None,
    }
}

/// Get the raw RGBA pixels of a clipboard image, loading it from the database if needed.
pub fn image_rgba(item: &ClipboardItem) -> Option<Vec<u8>> {
    let png = image_png(item)?;
    image::load_from_memory_with_format(&png, image::ImageFormat::Png)
        .inspect_err(|e| error!("Failed to decode clipboard image: {}", e))
        .ok()
        .map(|image| image.into_rgba8().into_raw())
}

/// Convert images stored as raw RGBA data by earlier versions to PNG.
fn migrate_raw_images(db: &Connection) -> rusqlite::Result<()> {
    let mut stmt = db.prepare(
        "SELECT id, image_width, image_height, image_data FROM clipboard_history
         WHERE content_type = 'image' AND substr(image_data, 1, 8) != ?1",
    )?;
    let rows = stmt
        .query_map(params![PNG_SIGNATURE], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)? as usize,
                row.get::<_, i64>(2)? as usize,
                row.get::<_, Vec<u8>>(3)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    for (id, width, height, rgba) in rows {
        match encode_png(width, height, &rgba) {
            Ok(png) => db.execute(
                "UPDATE clipboard_history SET image_data = ?1 WHERE id = ?2",
                params![png, id],
            )?,
            Err(e) => {
                warn!("Dropping invalid clipboard image {}: {}", id, e);
                db.execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])?
            }
        };
    }
    Ok(())
}

/// Check if a clipboard item has the same content as `b`.
fn is_same_content(item: &ClipboardItem, b: &ClipboardContent) -> bool {
    match (&item.content, b) {
        (ClipboardContent::Text(a), ClipboardContent::Text(b)) => a == b,
        (
            ClipboardContent::Image {
                width: w1,
                height: h1,
                ..
            },
            ClipboardContent::Image {
                width: w2,
                height: h2,
                data: ImageData::Png(png),
            },
        ) => w1 == w2 && h1 == h2 && image_png(item).is_some_and(|p| p == *png),
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
    with_db(|db| db.execute("DELETE FROM clipboard_history", []));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        conn
    }

    fn text_item(text: &str) -> ClipboardItem {
        ClipboardItem::new(ClipboardContent::Text(text.to_string()))
    }

    #[test]
    fn test_prune_keeps_newest() {
        let db = test_db();
        for i in 0..10 {
            insert_item(&db, &text_item(&i.to_string())).unwrap();
        }
        prune_db(&db, 3).unwrap();
        let items = load_items(&db).unwrap();
        let texts: Vec<_> = items.iter().map(|i| i.full_content()).collect();
        assert_eq!(texts, ["9", "8", "7"]);
    }

    #[test]
    fn test_images_are_loaded_lazily() {
        let db = test_db();
        let png = encode_png(2, 1, &[255, 0, 0, 255, 0, 255, 0, 255]).unwrap();
        let item = ClipboardItem::new(ClipboardContent::Image {
            width: 2,
            height: 1,
            data: ImageData::Png(png.into()),
        });
        insert_item(&db, &item).unwrap();

        let items = load_items(&db).unwrap();
        assert!(matches!(
            items[0].content,
            ClipboardContent::Image {
                width: 2,
                height: 1,
                data: ImageData::Stored
            }
        ));
    }

    #[test]
    fn test_migrate_raw_images() {
        let db = test_db();
        let rgba = vec![1, 2, 3, 255, 4, 5, 6, 255];
        db.execute(
            "INSERT INTO clipboard_history (content_type,image_width,image_height,image_data,timestamp)
             VALUES ('image',2,1,?1,0)",
            params![rgba],
        )
        .unwrap();

        migrate_raw_images(&db).unwrap();

        let png: Vec<u8> = db
            .query_row("SELECT image_data FROM clipboard_history", [], |r| r.get(0))
            .unwrap();
        assert!(png.starts_with(PNG_SIGNATURE));
        let decoded = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
            .unwrap()
            .into_rgba8()
            .into_raw();
        assert_eq!(decoded, rgba);

        // Already migrated images are left alone
        migrate_raw_images(&db).unwrap();
        let png2: Vec<u8> = db
            .query_row("SELECT image_data FROM clipboard_history", [], |r| r.get(0))
            .unwrap();
        assert_eq!(png, png2);
    }
}
