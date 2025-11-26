use crate::compositor::Compositor;
use crate::desktop::launch_application;
use crate::items::ListItem;
use crate::ui::items::ItemListDelegate;
use crate::ui::theme::theme;
use gpui::{
    App, AsyncApp, Context, Entity, FocusHandle, Focusable, KeyBinding, ScrollStrategy, Task,
    WeakEntity, Window, actions, div, image_cache, prelude::*, retain_all,
};
use gpui_component::IndexPath;
use gpui_component::input::{Input, InputState};
use gpui_component::list::{List, ListState};
use gpui_component::{ActiveTheme, Icon, IconName};
use std::sync::Arc;

actions!(launcher, [SelectNext, SelectPrev, Confirm, Cancel]);

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", SelectPrev, Some("LauncherView")),
        KeyBinding::new("down", SelectNext, Some("LauncherView")),
        KeyBinding::new("enter", Confirm, Some("LauncherView")),
        KeyBinding::new("escape", Cancel, Some("LauncherView")),
    ]);
}

pub struct LauncherView {
    list_state: Entity<ListState<ItemListDelegate>>,
    input_state: Entity<InputState>,
    focus_handle: FocusHandle,
    #[allow(dead_code)] // Kept alive for blur handler
    on_hide: std::sync::Arc<dyn Fn() + Send + Sync>,
    _search_task: Task<()>,
}

impl LauncherView {
    pub fn new(
        items: Vec<ListItem>,
        compositor: Arc<dyn Compositor>,
        on_hide: impl Fn() + Send + Sync + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut delegate = ItemListDelegate::new(items);

        // Set up callbacks using Arc for sharing
        let on_hide = std::sync::Arc::new(on_hide);
        let on_hide_for_confirm = on_hide.clone();
        let on_hide_for_cancel = on_hide.clone();

        delegate.set_on_confirm(move |item| {
            match item {
                ListItem::Application(app) => {
                    // Convert back to DesktopEntry for launching
                    let entry = crate::desktop::DesktopEntry::new(
                        app.id.clone(),
                        app.name.clone(),
                        app.exec.clone(),
                        None,
                        app.icon_path.clone(),
                        app.description.clone(),
                        vec![],
                        app.terminal,
                        app.desktop_path.clone(),
                    );
                    let _ = launch_application(&entry);
                }
                ListItem::Window(win) => {
                    // Focus the window via compositor
                    if let Err(e) = compositor.focus_window(&win.address) {
                        tracing::warn!(%e, "Failed to focus window");
                    }
                }
                _ => {}
            }
            on_hide_for_confirm();
        });
        delegate.set_on_cancel(move || on_hide_for_cancel());

        let list_state = cx.new(|cx| ListState::new(delegate, window, cx));

        let input_state =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search applications..."));

        let list_state_for_subscribe = list_state.clone();
        cx.subscribe(&input_state, move |this, input, event, cx| {
            if let gpui_component::input::InputEvent::Change = event {
                let text = input.read(cx).value().to_string();
                this.async_search(text, list_state_for_subscribe.clone(), cx);
            }
        })
        .detach();

        let focus_handle = cx.focus_handle();

        // Hide when the view loses focus (user clicked outside the window)
        let on_hide_for_blur = on_hide.clone();
        cx.on_blur(&focus_handle, window, move |_this, _window, _cx| {
            on_hide_for_blur();
        })
        .detach();

        Self {
            list_state,
            input_state,
            focus_handle,
            on_hide,
            _search_task: Task::ready(()),
        }
    }

    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.input_state.update(cx, |input: &mut InputState, cx| {
            input.focus(window, cx);
        });
    }

    pub fn reset_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.update(cx, |list_state, _cx| {
            list_state.delegate_mut().clear_query();
        });
        self.input_state.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
    }

    fn async_search(
        &mut self,
        query: String,
        list_state: Entity<ListState<ItemListDelegate>>,
        cx: &mut Context<Self>,
    ) {
        // Get items Arc for background processing
        let items = list_state.read(cx).delegate().items();
        let query_clone = query.clone();

        // Update query immediately (without filtering)
        list_state.update(cx, |list_state, _cx| {
            list_state.delegate_mut().set_query_only(query.clone());
        });

        let background = cx.background_executor().clone();

        self._search_task = cx.spawn(async move |_this: WeakEntity<Self>, cx: &mut AsyncApp| {
            // Run filtering on background thread
            let filtered_indices = background
                .spawn(async move { ItemListDelegate::filter_items_sync(&items, &query_clone) })
                .await;

            // Apply results on main thread
            let _ = cx.update(|cx| {
                list_state.update(cx, |list_state, cx| {
                    list_state
                        .delegate_mut()
                        .apply_filter_results(query, filtered_indices);
                    cx.notify();
                });
            });
        });
    }

    fn select_next(&mut self, _: &SelectNext, window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.update(cx, |list_state, cx| {
            let delegate = list_state.delegate_mut();
            let count = delegate.filtered_count();
            if count == 0 {
                return;
            }
            let current = delegate.selected_index().unwrap_or(0);
            let next = if current + 1 >= count { 0 } else { current + 1 };
            delegate.set_selected(next);
            let (section, row) = delegate.global_to_section_row(next);
            list_state.scroll_to_item(
                IndexPath::new(row).section(section),
                ScrollStrategy::Top,
                window,
                cx,
            );
            cx.notify();
        });
    }

    fn select_prev(&mut self, _: &SelectPrev, window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.update(cx, |list_state, cx| {
            let delegate = list_state.delegate_mut();
            let count = delegate.filtered_count();
            if count == 0 {
                return;
            }
            let current = delegate.selected_index().unwrap_or(0);
            let prev = if current == 0 { count - 1 } else { current - 1 };
            delegate.set_selected(prev);
            let (section, row) = delegate.global_to_section_row(prev);
            list_state.scroll_to_item(
                IndexPath::new(row).section(section),
                ScrollStrategy::Top,
                window,
                cx,
            );
            cx.notify();
        });
    }

    fn confirm(&mut self, _: &Confirm, _window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.update(cx, |list_state, _cx| {
            list_state.delegate_mut().do_confirm();
        });
    }

    fn cancel(&mut self, _: &Cancel, _window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.update(cx, |list_state, _cx| {
            list_state.delegate_mut().do_cancel();
        });
    }
}

impl Focusable for LauncherView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui::Render for LauncherView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme();

        // Fullscreen backdrop - clicking it closes the launcher
        let on_hide = self.on_hide.clone();
        div()
            .id("launcher-backdrop")
            .key_context("LauncherView")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            // Click on backdrop to close
            .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, _cx| {
                on_hide();
            })
            // Centered launcher panel
            .child(
                div()
                    .id("launcher-panel")
                    .w(t.window_width)
                    .h(t.window_height)
                    .flex()
                    .flex_col()
                    .bg(t.window_background)
                    .rounded(t.window_border_radius)
                    .border_1()
                    .border_color(t.window_border)
                    .overflow_hidden()
                    // Stop click propagation to backdrop
                    .on_mouse_down(gpui::MouseButton::Left, |_event, _window, _cx| {
                        // Do nothing - just stop propagation
                    })
                    // Search input section
                    .child(
                        div()
                            .w_full()
                            .px_2()
                            .py_3()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                Input::new(&self.input_state)
                                    .appearance(false)
                                    .cleanable(true)
                                    .prefix(
                                        Icon::new(IconName::Search)
                                            .text_color(cx.theme().muted_foreground)
                                            .mr_2(),
                                    ),
                            ),
                    )
                    // List section with image caching
                    .child(
                        image_cache(retain_all("app-icons"))
                            .flex_1()
                            .overflow_hidden()
                            .py_2()
                            .child(List::new(&self.list_state)),
                    ),
            )
    }
}
