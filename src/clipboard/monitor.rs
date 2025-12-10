//! Clipboard monitoring using Wayland data-control protocol.

use super::data;
use super::item::ClipboardContent;
use arboard::Clipboard;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info};
use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::data_control::v1::client::{
    zwlr_data_control_device_v1, zwlr_data_control_manager_v1, zwlr_data_control_offer_v1,
    zwlr_data_control_source_v1,
};

/// State for the Wayland clipboard monitor.
struct ClipboardMonitorState {
    manager: Option<zwlr_data_control_manager_v1::ZwlrDataControlManagerV1>,
    seat: Option<wl_seat::WlSeat>,
    device: Option<zwlr_data_control_device_v1::ZwlrDataControlDeviceV1>,
    running: Arc<AtomicBool>,
}

/// Start monitoring clipboard changes in a background thread.
pub fn start_monitor() -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    thread::spawn(move || {
        info!("Starting clipboard monitor");

        if let Err(e) = run_monitor(running_clone) {
            error!("Clipboard monitor error: {}", e);
        }
    });

    running
}

fn run_monitor(running: Arc<AtomicBool>) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Wayland
    let conn = Connection::connect_to_env()?;
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());

    let mut state = ClipboardMonitorState {
        manager: None,
        seat: None,
        device: None,
        running,
    };

    // Initial roundtrip to get globals
    event_queue.roundtrip(&mut state)?;

    if state.manager.is_none() {
        error!("wlr-data-control protocol not available");
        return Err("wlr-data-control protocol not available".into());
    }

    if state.seat.is_none() {
        error!("No Wayland seat available");
        return Err("No Wayland seat available".into());
    }

    // Create data control device for the seat
    if let (Some(manager), Some(seat)) = (&state.manager, &state.seat) {
        let device = manager.get_data_device(seat, &qh, ());
        state.device = Some(device);
        debug!("Created data control device");
    }

    // Another roundtrip to initialize the device
    event_queue.roundtrip(&mut state)?;

    info!("Clipboard monitor initialized successfully");

    // Event loop
    while state.running.load(Ordering::Relaxed) {
        event_queue.blocking_dispatch(&mut state)?;
    }

    info!("Clipboard monitor stopped");
    Ok(())
}

impl Dispatch<wl_registry::WlRegistry, ()> for ClipboardMonitorState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == "zwlr_data_control_manager_v1" {
                let manager = registry
                    .bind::<zwlr_data_control_manager_v1::ZwlrDataControlManagerV1, _, _>(
                        name,
                        version.min(2),
                        qh,
                        (),
                    );
                state.manager = Some(manager);
                debug!("Bound to wlr-data-control-manager");
            } else if interface == "wl_seat" && state.seat.is_none() {
                let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, version.min(1), qh, ());
                state.seat = Some(seat);
                debug!("Bound to wl_seat");
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for ClipboardMonitorState {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_data_control_manager_v1::ZwlrDataControlManagerV1, ()>
    for ClipboardMonitorState
{
    fn event(
        _: &mut Self,
        _: &zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
        _: zwlr_data_control_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_data_control_device_v1::ZwlrDataControlDeviceV1, ()> for ClipboardMonitorState {
    fn event(
        _: &mut Self,
        _: &zwlr_data_control_device_v1::ZwlrDataControlDeviceV1,
        event: zwlr_data_control_device_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_data_control_device_v1::Event::Selection { id } => {
                if id.is_some() {
                    debug!("Clipboard selection changed");
                    // Clipboard changed, read the new content
                    if let Err(e) = read_clipboard_content() {
                        error!("Failed to read clipboard: {}", e);
                    }
                }
            }
            zwlr_data_control_device_v1::Event::PrimarySelection { .. } => {
                // Ignore primary selection for now
            }
            _ => {}
        }
    }

    fn event_created_child(
        opcode: u16,
        qhandle: &QueueHandle<Self>,
    ) -> std::sync::Arc<dyn wayland_client::backend::ObjectData> {
        match opcode {
            zwlr_data_control_device_v1::EVT_DATA_OFFER_OPCODE => {
                qhandle.make_data::<zwlr_data_control_offer_v1::ZwlrDataControlOfferV1, _>(())
            }
            _ => panic!("Unknown opcode {} for zwlr_data_control_device_v1", opcode),
        }
    }
}

impl Dispatch<zwlr_data_control_offer_v1::ZwlrDataControlOfferV1, ()> for ClipboardMonitorState {
    fn event(
        _: &mut Self,
        _: &zwlr_data_control_offer_v1::ZwlrDataControlOfferV1,
        _: zwlr_data_control_offer_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_data_control_source_v1::ZwlrDataControlSourceV1, ()> for ClipboardMonitorState {
    fn event(
        _: &mut Self,
        _: &zwlr_data_control_source_v1::ZwlrDataControlSourceV1,
        _: zwlr_data_control_source_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

/// Read the current clipboard content and add it to history.
fn read_clipboard_content() -> Result<(), Box<dyn std::error::Error>> {
    // Small delay to let clipboard settle
    thread::sleep(Duration::from_millis(50));

    let mut clipboard = Clipboard::new()?;

    // Try to get image first - browsers often put both image data and HTML markup
    // on the clipboard, and we prefer the actual image over the HTML representation
    if let Ok(image) = clipboard.get_image()
        && !image.bytes.is_empty()
    {
        debug!(
            "Adding image to clipboard history: {}×{} ({} bytes)",
            image.width,
            image.height,
            image.bytes.len()
        );
        data::add_item(ClipboardContent::Image {
            width: image.width,
            height: image.height,
            rgba_bytes: image.bytes.to_vec(),
        });
        return Ok(());
    }

    // Try to get text
    if let Ok(text) = clipboard.get_text()
        && !text.is_empty()
    {
        debug!("Adding text to clipboard history: {} chars", text.len());
        data::add_item(ClipboardContent::Text(text));
        return Ok(());
    }

    Ok(())
}
