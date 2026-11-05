//! KDE KWin compositor implementation using D-Bus WindowsRunner API.
//!
//! Uses KWin's krunner interface via D-Bus to enumerate and focus windows.
//! This approach uses the /WindowsRunner D-Bus path which provides direct
//! window listing without needing to capture script print() signals.

use super::base::CompositorCapabilities;
use super::{Compositor, WindowInfo};
use anyhow::{Context, Result};
use image::{ImageBuffer, ImageFormat, Rgba};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::process::Command;
use zbus::zvariant::{OwnedValue, Structure, Value};
use zbus::{Connection, Proxy, block_on};

/// Type alias for KRunner match results from WindowsRunner.Match D-Bus call.
/// Tuple: (match_id, text, subtext, type, relevance, properties)
type KRunnerMatch = (
    String,
    String,
    String,
    i32,
    f64,
    HashMap<String, OwnedValue>,
);

/// KWin compositor client using D-Bus WindowsRunner API.
pub struct KwinCompositor {
    connection: Connection,
}

/// Parse icon-data from KRunner and return as PNG bytes.
///
/// The icon-data format is a D-Bus structure: (iiibiiay)
/// - i32: width
/// - i32: height
/// - i32: rowstride (bytes per row)
/// - bool: has_alpha
/// - i32: bits_per_sample (usually 8)
/// - i32: channels (3 for RGB, 4 for RGBA)
/// - Vec<u8>: pixel data
fn parse_icon_data(icon_data: &OwnedValue) -> Option<Vec<u8>> {
    // Try to extract the structure
    let structure: &Structure = icon_data.downcast_ref().ok()?;
    let fields = structure.fields();

    if fields.len() < 7 {
        return None;
    }

    let width: i32 = fields[0].downcast_ref::<i32>().ok()?;
    let height: i32 = fields[1].downcast_ref::<i32>().ok()?;
    let rowstride: i32 = fields[2].downcast_ref::<i32>().ok()?;
    let has_alpha: bool = fields[3].downcast_ref::<bool>().ok()?;
    let _bits_per_sample: i32 = fields[4].downcast_ref::<i32>().ok()?;
    let channels: i32 = fields[5].downcast_ref::<i32>().ok()?;

    // Validate dimensions and channels before any arithmetic
    if width <= 0 || height <= 0 || width > 256 || height > 256 {
        return None;
    }
    if rowstride <= 0 || channels < 3 || channels > 4 {
        return None;
    }

    // Get the pixel data array
    let pixel_data: Vec<u8> = match &fields[6] {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.downcast_ref::<u8>().ok())
            .collect(),
        _ => return None,
    };

    let width = width as u32;
    let height = height as u32;
    let rowstride = rowstride as usize;
    let channels = channels as usize;

    // Create RGBA image buffer
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            // Use checked arithmetic to avoid overflow
            let row_offset = (y as usize).checked_mul(rowstride)?;
            let col_offset = (x as usize).checked_mul(channels)?;
            let src_offset = row_offset.checked_add(col_offset)?;

            // Use safe array access with .get()
            let pixel = if channels == 4 && has_alpha {
                Rgba([
                    *pixel_data.get(src_offset)?,
                    *pixel_data.get(src_offset + 1)?,
                    *pixel_data.get(src_offset + 2)?,
                    *pixel_data.get(src_offset + 3)?,
                ])
            } else if channels == 3 {
                Rgba([
                    *pixel_data.get(src_offset)?,
                    *pixel_data.get(src_offset + 1)?,
                    *pixel_data.get(src_offset + 2)?,
                    255,
                ])
            } else {
                return None;
            };

            img.put_pixel(x, y, pixel);
        }
    }

    // Encode as PNG in memory
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    img.write_to(&mut cursor, ImageFormat::Png).ok()?;

    Some(png_bytes)
}

impl KwinCompositor {
    /// Create a new KWin compositor client.
    ///
    /// Returns None if KDE session is not detected or KWin is not available.
    pub fn new() -> Option<Self> {
        // Check if we're in a KDE session
        if std::env::var("KDE_SESSION_VERSION").is_err() {
            return None;
        }

        // Connect to session D-Bus
        let connection = block_on(Connection::session()).ok()?;

        // Verify KWin is available by calling supportInformation
        let kwin_proxy = block_on(Proxy::new(
            &connection,
            "org.kde.KWin",
            "/KWin",
            "org.kde.KWin",
        ))
        .ok()?;

        let _: String = block_on(kwin_proxy.call("supportInformation", &())).ok()?;

        Some(Self { connection })
    }

    /// List windows using the WindowsRunner krunner interface.
    /// Returns tuples of (match_id, title, subtext, type, relevance, properties)
    fn list_windows_via_runner(&self) -> Result<Vec<WindowInfo>> {
        // Create proxy for WindowsRunner
        let runner_proxy = block_on(Proxy::new(
            &self.connection,
            "org.kde.KWin",
            "/WindowsRunner",
            "org.kde.krunner1",
        ))
        .context("Failed to create WindowsRunner proxy")?;

        // Call Match with empty query to get all windows
        // Returns: a(sssida{sv}) - array of tuples
        let result: Vec<KRunnerMatch> = block_on(runner_proxy.call("Match", &("",)))
            .context("Failed to call WindowsRunner.Match")?;

        // Track seen window IDs to deduplicate (KRunner returns multiple actions per window)
        let mut seen_ids: HashSet<String> = HashSet::new();

        let windows: Vec<WindowInfo> = result
            .into_iter()
            .filter_map(|(match_id, title, _subtext, _type_id, _relevance, props)| {
                // match_id format: "{action_index}_{uuid}" - extract the window ID
                // Action indices: 0 = activate, 1 = close, 8 = switch to desktop, etc.
                // Only keep action 0 (activate) entries to avoid duplicates
                let window_id = match_id.strip_prefix("0_")?;

                // Skip if we've already seen this window
                if !seen_ids.insert(window_id.to_string()) {
                    return None;
                }

                // Try to parse icon-data as PNG bytes if available
                let icon_data = props
                    .get("icon-data")
                    .and_then(|data| parse_icon_data(data));

                // Use icon name for class if available, otherwise extract from title
                let icon_name = props
                    .get("icon")
                    .and_then(|v| TryInto::<String>::try_into(v.clone()).ok());

                let class = icon_name
                    .unwrap_or_else(|| title.rsplit(" - ").next().unwrap_or(&title).to_string());

                Some(WindowInfo {
                    address: window_id.to_string(),
                    title: title.clone(),
                    class,
                    workspace: 1,   // WindowsRunner doesn't expose workspace info
                    focused: false, // We can't easily determine this from krunner
                    icon_data,
                })
            })
            .collect();

        Ok(windows)
    }

    /// Focus a window using the WindowsRunner Run method.
    fn focus_window_via_runner(&self, window_id: &str) -> Result<()> {
        let runner_proxy = block_on(Proxy::new(
            &self.connection,
            "org.kde.KWin",
            "/WindowsRunner",
            "org.kde.krunner1",
        ))
        .context("Failed to create WindowsRunner proxy")?;

        // match_id needs the "0_" prefix for the activate action
        let match_id = format!("0_{}", window_id);

        // Run with empty action_id (default action = activate)
        let _: () = block_on(runner_proxy.call("Run", &(&match_id, "")))
            .context("Failed to call WindowsRunner.Run")?;

        Ok(())
    }
}

impl Compositor for KwinCompositor {
    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        self.list_windows_via_runner()
    }

    fn focus_window(&self, window_id: &str) -> Result<()> {
        // First try the krunner approach
        if let Ok(()) = self.focus_window_via_runner(window_id) {
            return Ok(());
        }

        // Fallback: use qdbus to activate window
        let status = Command::new("qdbus")
            .args([
                "org.kde.KWin",
                "/WindowsRunner",
                "org.kde.krunner1.Run",
                &format!("0_{}", window_id),
                "",
            ])
            .status()
            .context("Failed to run qdbus")?;

        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("qdbus command failed with status: {}", status)
        }
    }

    fn name(&self) -> &'static str {
        "KWin"
    }

    fn capabilities(&self) -> CompositorCapabilities {
        CompositorCapabilities::limited()
    }
}
