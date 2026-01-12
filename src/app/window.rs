use crate::app::{DaemonEvent, DaemonEventSender, WindowEvent};
use crate::compositor::Compositor;
use crate::config::{ConfigModule, config};
use crate::items::{ApplicationItem, ListItem, WindowItem};
use crate::ui::LauncherView;
use gpui::{
    App, AppContext, Bounds, Entity, WindowBackgroundAppearance, WindowBounds, WindowDecorations,
    WindowHandle, WindowKind, WindowOptions,
    layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
    point, px, size,
};
use gpui_component::Root;
use std::sync::Arc;
use tracing::warn;

/// Handle to an open launcher window, containing both the window and view entity.
pub struct LauncherWindow {
    pub handle: WindowHandle<Root>,
    pub launcher_view: Entity<LauncherView>,
}

pub fn create_and_show_window(
    applications: Vec<ApplicationItem>,
    compositor: Arc<dyn Compositor>,
    event_tx: DaemonEventSender,
    cx: &mut App,
) -> anyhow::Result<LauncherWindow> {
    // Fetch open windows from compositor (if not disabled)
    let disabled_modules = config().disabled_modules.unwrap_or_default();
    let windows = if disabled_modules.contains(&ConfigModule::Windows) {
        Vec::new()
    } else {
        fetch_windows(compositor.as_ref())
    };
    create_and_show_window_impl(applications, compositor, windows, event_tx, cx)
}

fn create_and_show_window_impl(
    applications: Vec<ApplicationItem>,
    compositor: Arc<dyn Compositor>,
    windows: Vec<WindowItem>,
    event_tx: DaemonEventSender,
    cx: &mut App,
) -> anyhow::Result<LauncherWindow> {
    // Combine windows and applications into items list
    // Built-in actions and submenus are added by the delegate
    // Order doesn't matter here - sort_priority in delegate handles display order
    let mut items: Vec<ListItem> = Vec::with_capacity(windows.len() + applications.len());
    items.extend(windows.into_iter().map(ListItem::Window));
    items.extend(applications.into_iter().map(ListItem::Application));
    // Get display size based on compositor
    let display_size = if compositor.name() == "KWin" {
        // For KDE/KWin, use fixed 1920x1080
        size(px(1920.0), px(1080.0))
    } else {
        // For other compositors, 8K and hope for the best
        size(px(7680.0), px(4320.0))
    };

    let fullscreen_bounds = Bounds {
        origin: point(px(0.0), px(0.0)),
        size: display_size,
    };

    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(fullscreen_bounds)),
        titlebar: None,
        focus: true,
        show: true,
        app_id: Some("zlaunch".to_string()),
        window_background: WindowBackgroundAppearance::Transparent,
        window_decorations: Some(WindowDecorations::Server),
        kind: WindowKind::LayerShell(LayerShellOptions {
            namespace: "zlaunch".to_string(),
            layer: Layer::Overlay,
            // Anchor to all edges = fullscreen overlay
            anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
            // Exclusive keyboard so typing works immediately
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            ..Default::default()
        }),
        ..Default::default()
    };

    // We need to capture the launcher view entity from inside the closure
    let launcher_view_cell: std::cell::RefCell<Option<Entity<LauncherView>>> =
        std::cell::RefCell::new(None);

    let window_handle = cx.open_window(options, |window, cx| {
        let on_hide = move || {
            let _ = event_tx.send(DaemonEvent::Window(WindowEvent::RequestHide));
        };
        let view = cx.new(|cx| LauncherView::new(items, compositor.clone(), on_hide, window, cx));

        // Auto-focus the list/search input
        view.update(cx, |launcher: &mut LauncherView, cx| {
            launcher.focus(window, cx);
        });

        // Store the view entity for later access
        *launcher_view_cell.borrow_mut() = Some(view.clone());

        cx.new(|cx| Root::new(view, window, cx))
    })?;

    window_handle.update(cx, |_root, window, _cx| {
        window.activate_window();
    })?;

    let launcher_view = launcher_view_cell
        .into_inner()
        .expect("Launcher view should have been created");

    Ok(LauncherWindow {
        handle: window_handle,
        launcher_view,
    })
}

pub fn close_window(handle: &WindowHandle<Root>, cx: &mut App) {
    let _ = handle.update(cx, |_root, window, _cx| {
        window.remove_window();
    });
}

/// Fetch open windows from the compositor and convert to WindowItems.
fn fetch_windows(compositor: &dyn Compositor) -> Vec<WindowItem> {
    match compositor.list_windows() {
        Ok(windows) => {
            windows
                .into_iter()
                .map(|info| {
                    // Try to resolve icon from app class
                    let icon_path = resolve_window_icon(&info.class);
                    WindowItem::from_window_info(info, icon_path)
                })
                .collect()
        }
        Err(e) => {
            warn!(%e, "Failed to list windows");
            Vec::new()
        }
    }
}

/// Try to resolve an icon path for a window based on its app class.
fn resolve_window_icon(app_class: &str) -> Option<std::path::PathBuf> {
    use crate::ui::icon::resolve_icon_path;

    // Try the class name directly (most apps use this)
    if let Some(path) = resolve_icon_path(app_class) {
        return Some(path);
    }

    // Try lowercase version
    let lower = app_class.to_lowercase();
    if let Some(path) = resolve_icon_path(&lower) {
        return Some(path);
    }

    // For reverse-DNS style names (org.kde.dolphin), try the last segment
    if let Some(name) = app_class.rsplit('.').next() {
        if let Some(path) = resolve_icon_path(name) {
            return Some(path);
        }
        if let Some(path) = resolve_icon_path(&name.to_lowercase()) {
            return Some(path);
        }
    }

    None
}
