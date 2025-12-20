use crate::calculator::evaluate_expression;
use crate::config::{ConfigModule, config};
use crate::items::{ActionItem, AiItem, CalculatorItem, ListItem, SearchItem, SubmenuItem};
use crate::search::{SearchDetection, detect_search, get_providers};
use crate::ui::delegates::BaseDelegate;
use crate::ui::theme::theme;
use crate::ui::views::render_item;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use gpui::{App, Context, SharedString, Task, Window, div, prelude::*};
use gpui_component::IndexPath;
use gpui_component::list::{ListDelegate, ListItem as GpuiListItem, ListState};
use std::sync::Arc;

/// Section information for tracking item counts by type
#[derive(Clone, Debug, Default)]
struct SectionInfo {
    search_count: usize,
    window_count: usize,
    command_count: usize,
    app_count: usize,
}

/// Type alias for confirm callback
type ConfirmCallback = Arc<dyn Fn(&ListItem) + Send + Sync>;

/// Enhanced delegate for the main item list.
///
/// This delegate composes with BaseDelegate<ListItem> and adds:
/// - Dynamic calculator results
/// - AI query detection
/// - Web search suggestions
/// - Section management
pub struct ItemListDelegate {
    /// Base delegate handling common behavior
    base: BaseDelegate<ListItem>,
    /// Section counts for organizing items
    section_info: SectionInfo,
    /// Calculator result (shown at top when query is math expression)
    calculator_item: Option<CalculatorItem>,
    /// AI item (shown when query starts with !ai)
    ai_item: Option<AiItem>,
    /// Search items (shown when query triggers search providers)
    search_items: Vec<SearchItem>,
    /// Confirm callback (stored here to handle dynamic items)
    on_confirm: Option<ConfirmCallback>,
}

impl ItemListDelegate {
    /// Create a new item list delegate
    pub fn new(mut items: Vec<ListItem>) -> Self {
        let disabled_modules = config().disabled_modules.unwrap_or_default();

        // Add built-in submenu items
        if !disabled_modules.contains(&ConfigModule::Emojis) {
            items.push(ListItem::Submenu(
                SubmenuItem::grid("submenu-emojis", "Emojis", 8)
                    .with_description("Search and copy emojis")
                    .with_icon("smiley"),
            ));
        }
        if !disabled_modules.contains(&ConfigModule::Clipboard) {
            items.push(ListItem::Submenu(
                SubmenuItem::list("submenu-clipboard", "Clipboard History")
                    .with_description("View and paste clipboard history")
                    .with_icon("clipboard"),
            ));
        }
        if !disabled_modules.contains(&ConfigModule::Themes) {
            items.push(ListItem::Submenu(
                SubmenuItem::list("submenu-themes", "Themes")
                    .with_description("Browse and apply themes")
                    .with_icon("palette"),
            ));
        }

        // Add built-in action items
        for action in ActionItem::builtins() {
            items.push(ListItem::Action(action));
        }

        // Sort items by priority to ensure correct section order
        // (Windows=2, Commands=3, Applications=4)
        items.sort_by_key(|item| item.sort_priority());

        let section_info =
            Self::compute_section_info(&items, &(0..items.len()).collect::<Vec<_>>());

        Self {
            base: BaseDelegate::new(items),
            section_info,
            calculator_item: None,
            ai_item: None,
            search_items: Vec::new(),
            on_confirm: None,
        }
    }

    /// Set the confirm callback
    pub fn set_on_confirm(&mut self, callback: impl Fn(&ListItem) + Send + Sync + 'static) {
        self.on_confirm = Some(Arc::new(callback));
    }

    /// Set the cancel callback
    pub fn set_on_cancel(&mut self, callback: impl Fn() + Send + Sync + 'static) {
        self.base.set_on_cancel(callback);
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> Option<usize> {
        self.base.selected_index()
    }

    /// Set the selected index (override to handle dynamic items)
    pub fn set_selected(&mut self, index: usize) {
        // BaseDelegate's set_selected checks against base filtered_count
        // But we have dynamic items (Calculator, AI, Search), so check against total count
        if index < self.filtered_count() {
            // Use unchecked method to bypass base's count validation
            self.base.set_selected_unchecked(index);
        }
    }

    /// Get the total count of filtered items (including dynamic items)
    pub fn filtered_count(&self) -> usize {
        let calc_count = if self.calculator_item.is_some() { 1 } else { 0 };
        let ai_count = if self.ai_item.is_some() { 1 } else { 0 };
        let search_count = self.search_items.len();
        self.base.filtered_count() + calc_count + ai_count + search_count
    }

    /// Get the current query
    pub fn query(&self) -> &str {
        self.base.query()
    }

    /// Clear the query and reset all dynamic items
    pub fn clear_query(&mut self) {
        self.calculator_item = None;
        self.ai_item = None;
        self.search_items.clear();
        self.base.clear_query();
        self.update_section_info();
    }

    /// Set the query and trigger filtering
    pub fn set_query(&mut self, query: String) {
        self.base.set_query(query.clone());
        self.process_query(&query);
    }

    /// Process the query to detect special items (calculator, AI, search)
    fn process_query(&mut self, query: &str) {
        // Get the config disabled modules
        let disabled_modules = config().disabled_modules.unwrap_or_default();

        // Check for calculator expression
        if !disabled_modules.contains(&ConfigModule::Calculator)
            && query.chars().any(|c| c.is_numeric())
            && let Ok(result) = evaluate_expression(query)
        {
            self.calculator_item = Some(result);
            self.update_section_info();
        } else {
            self.calculator_item = None;
        }

        // Filter the base items first
        self.filter_items();
        let trimmed = query.trim();

        // Check for trigger phrases
        let has_ai_trigger = trimmed.starts_with("!ai");
        let search_detection = detect_search(query);
        let has_search_trigger = matches!(search_detection, SearchDetection::Triggered { .. });

        // Clear previous dynamic items
        self.ai_item = None;
        self.search_items.clear();

        // Logic:
        // 1. If !ai trigger → only show AI item
        // 2. Else if search trigger (!g, !ddg, etc.) → only show that search provider
        // 3. Else if query not empty → always show AI item + all search providers at bottom

        if !disabled_modules.contains(&ConfigModule::Ai) && has_ai_trigger {
            // Only show AI item when !ai trigger is used
            let ai_query = trimmed.strip_prefix("!ai").unwrap().trim();
            if !ai_query.is_empty() {
                self.ai_item = Some(AiItem::new(ai_query.to_string()));
            }
        } else if !disabled_modules.contains(&ConfigModule::Search) && has_search_trigger {
            // Only show the triggered search provider
            if let SearchDetection::Triggered { provider, query } = search_detection {
                self.search_items.push(SearchItem::new(provider, query));
            }
        } else if !trimmed.is_empty() {
            // Always show AI item and all search providers when query is not empty
            // These appear at the bottom in "Search and AI" section
            if !disabled_modules.contains(&ConfigModule::Ai) {
                self.ai_item = Some(AiItem::new(trimmed.to_string()));
            }
            if !disabled_modules.contains(&ConfigModule::Search)
                && let SearchDetection::Fallback { query } = search_detection
            {
                for provider in get_providers() {
                    self.search_items
                        .push(SearchItem::new(provider, query.clone()));
                }
            }
        }

        // Update section info after adding search items
        self.update_section_info();

        // Ensure selection is initialized when we have items (base or dynamic)
        if self.base.selected_index().is_none() && self.filtered_count() > 0 {
            self.base.set_selected_unchecked(0);
        }
    }

    /// Filter items based on the current query
    fn filter_items(&mut self) {
        let query = self.base.query();
        let items = self.base.items();

        if query.is_empty() {
            // Sort by priority even when showing all items
            // This ensures sections (Windows, Commands, Applications) appear in correct order
            let mut sorted_indices: Vec<usize> = (0..items.len()).collect();
            sorted_indices.sort_by_key(|&idx| items[idx].sort_priority());
            self.base.apply_filtered_indices(sorted_indices);
        } else {
            let filtered_indices = Self::filter_items_sync(items, query);
            self.base.apply_filtered_indices(filtered_indices);
        }
        self.update_section_info();

        // Ensure selection is initialized when we have dynamic items but no base matches
        if self.base.selected_index().is_none() && self.filtered_count() > 0 {
            self.base.set_selected_unchecked(0);
        }
    }

    /// Filter items synchronously using fuzzy matching
    fn filter_items_sync(items: &[ListItem], query: &str) -> Vec<usize> {
        if query.is_empty() {
            return (0..items.len()).collect();
        }

        let matcher = SkimMatcherV2::default();
        let mut scored: Vec<(usize, i64)> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                matcher
                    .fuzzy_match(item.name(), query)
                    .map(|score| (idx, score))
            })
            .collect();

        // Sort by priority first, then by score
        scored.sort_by(|a, b| {
            let priority_a = items[a.0].sort_priority();
            let priority_b = items[b.0].sort_priority();
            priority_a.cmp(&priority_b).then_with(|| b.1.cmp(&a.1))
        });

        scored.into_iter().map(|(idx, _)| idx).collect()
    }

    /// Compute section counts from filtered indices
    fn compute_section_info(items: &[ListItem], filtered_indices: &[usize]) -> SectionInfo {
        let mut info = SectionInfo::default();

        for &idx in filtered_indices {
            if let Some(item) = items.get(idx) {
                if item.is_window() {
                    info.window_count += 1;
                } else if item.is_submenu() || item.is_action() {
                    info.command_count += 1;
                } else if item.is_application() {
                    info.app_count += 1;
                }
            }
        }

        info.search_count = 0; // Will be set by search_items.len()
        info
    }

    /// Update section info after filtering
    fn update_section_info(&mut self) {
        self.section_info =
            Self::compute_section_info(self.base.items(), self.base.filtered_indices());
        self.section_info.search_count = self.search_items.len();
    }

    /// Get an item at a global index (including dynamic items)
    /// Order: Calculator, Regular items (Windows/Commands/Apps), AI, Search
    pub fn get_item_at(&self, global_index: usize) -> Option<ListItem> {
        let calc_offset = if self.calculator_item.is_some() { 1 } else { 0 };
        let regular_count = self.base.filtered_count();
        let ai_offset = if self.ai_item.is_some() { 1 } else { 0 };

        // Calculator item (always first if present)
        if global_index == 0 && self.calculator_item.is_some() {
            return self.calculator_item.clone().map(ListItem::Calculator);
        }

        // Regular filtered items (Windows, Commands, Applications)
        let regular_start = calc_offset;
        let regular_end = regular_start + regular_count;
        if global_index >= regular_start && global_index < regular_end {
            let filtered_idx = global_index - regular_start;
            return self.base.get_filtered_item(filtered_idx).cloned();
        }

        // AI item
        let ai_start = regular_end;
        if global_index == ai_start && self.ai_item.is_some() {
            return self.ai_item.clone().map(ListItem::Ai);
        }

        // Search items
        let search_start = ai_start + ai_offset;
        let search_end = search_start + self.search_items.len();
        if global_index >= search_start && global_index < search_end {
            let search_idx = global_index - search_start;
            return self
                .search_items
                .get(search_idx)
                .cloned()
                .map(ListItem::Search);
        }

        None
    }

    /// Execute confirm callback for the selected item
    pub fn do_confirm(&self) {
        if let Some(idx) = self.selected_index()
            && let Some(item) = self.get_item_at(idx)
            && let Some(ref callback) = self.on_confirm
        {
            callback(&item);
        }
    }

    /// Execute cancel callback
    pub fn do_cancel(&self) {
        self.base.do_cancel();
    }

    /// Move selection down
    pub fn select_down(&mut self) {
        let count = self.filtered_count();
        if count == 0 {
            return;
        }

        let current = self.selected_index().unwrap_or(0);
        let next = if current + 1 >= count { 0 } else { current + 1 };
        self.set_selected(next);
    }

    /// Move selection up
    pub fn select_up(&mut self) {
        let count = self.filtered_count();
        if count == 0 {
            return;
        }

        let current = self.selected_index().unwrap_or(0);
        let prev = if current == 0 { count - 1 } else { current - 1 };
        self.set_selected(prev);
    }

    /// Get all items for external access
    pub fn items(&self) -> Arc<Vec<ListItem>> {
        Arc::new(self.base.items().to_vec())
    }

    /// Determine what type of section is at the given section index.
    /// Order: Calculator, Windows, Commands, Applications, SearchAndAi
    fn section_type_at(&self, section: usize) -> SectionType {
        let has_calc = self.calculator_item.is_some();
        let has_windows = self.section_info.window_count > 0;
        let has_commands = self.section_info.command_count > 0;
        let has_apps = self.section_info.app_count > 0;
        let has_search_and_ai = self.ai_item.is_some() || !self.search_items.is_empty();

        let mut current_section = 0;

        // Calculator always first
        if has_calc {
            if section == current_section {
                return SectionType::Calculator;
            }
            current_section += 1;
        }

        // Regular items in the middle
        if has_windows {
            if section == current_section {
                return SectionType::Windows;
            }
            current_section += 1;
        }

        if has_commands {
            if section == current_section {
                return SectionType::Commands;
            }
            current_section += 1;
        }

        if has_apps {
            if section == current_section {
                return SectionType::Applications;
            }
            current_section += 1;
        }

        // SearchAndAi section at the end (combined, no gap)
        if has_search_and_ai && section == current_section {
            return SectionType::SearchAndAi;
        }

        // Default (shouldn't reach here)
        SectionType::Applications
    }

    /// Get the starting global index for a given section type.
    /// Order: Calculator, Windows, Commands, Applications, SearchAndAi
    fn section_start_index(&self, section_type: SectionType) -> usize {
        let calc_offset = if self.calculator_item.is_some() { 1 } else { 0 };

        match section_type {
            SectionType::Calculator => 0,
            SectionType::Windows => calc_offset,
            SectionType::Commands => calc_offset + self.section_info.window_count,
            SectionType::Applications => {
                calc_offset + self.section_info.window_count + self.section_info.command_count
            }
            SectionType::SearchAndAi => {
                calc_offset
                    + self.section_info.window_count
                    + self.section_info.command_count
                    + self.section_info.app_count
            }
        }
    }

    /// Convert section+row to global index.
    fn section_row_to_global(&self, section: usize, row: usize) -> usize {
        let section_type = self.section_type_at(section);
        self.section_start_index(section_type) + row
    }

    /// Convert global index to section+row IndexPath.
    /// Order: Calculator, Windows, Commands, Applications, SearchAndAi
    pub fn global_to_index_path(&self, global_idx: usize) -> Option<IndexPath> {
        let calc_offset = if self.calculator_item.is_some() { 1 } else { 0 };
        let regular_count = self.base.filtered_count();

        let mut current_section = 0;

        // Calculator section
        if self.calculator_item.is_some() {
            if global_idx < calc_offset {
                return Some(IndexPath::new(global_idx).section(current_section));
            }
            current_section += 1;
        }

        // Regular filtered items (Windows, Commands, Applications)
        let regular_start = calc_offset;
        let regular_end = regular_start + regular_count;

        if global_idx >= regular_start && global_idx < regular_end {
            let regular_idx = global_idx - regular_start;

            // Windows section
            if self.section_info.window_count > 0 {
                if regular_idx < self.section_info.window_count {
                    return Some(IndexPath::new(regular_idx).section(current_section));
                }
                current_section += 1;
            }

            // Commands section
            if self.section_info.command_count > 0 {
                let cmd_start = self.section_info.window_count;
                let cmd_end = cmd_start + self.section_info.command_count;
                if regular_idx >= cmd_start && regular_idx < cmd_end {
                    return Some(IndexPath::new(regular_idx - cmd_start).section(current_section));
                }
                current_section += 1;
            }

            // Applications section
            if self.section_info.app_count > 0 {
                let app_start = self.section_info.window_count + self.section_info.command_count;
                if regular_idx >= app_start {
                    return Some(IndexPath::new(regular_idx - app_start).section(current_section));
                }
                current_section += 1;
            }
        } else {
            // Skip past regular sections in section counter
            if self.section_info.window_count > 0 {
                current_section += 1;
            }
            if self.section_info.command_count > 0 {
                current_section += 1;
            }
            if self.section_info.app_count > 0 {
                current_section += 1;
            }
        }

        // SearchAndAi section (combined AI + Search, no gap)
        if self.ai_item.is_some() || !self.search_items.is_empty() {
            let search_and_ai_start = regular_end;
            let ai_count = if self.ai_item.is_some() { 1 } else { 0 };
            let search_and_ai_end = search_and_ai_start + ai_count + self.search_items.len();

            if global_idx >= search_and_ai_start && global_idx < search_and_ai_end {
                return Some(
                    IndexPath::new(global_idx - search_and_ai_start).section(current_section),
                );
            }
        }

        None
    }
}

/// Section types for organizing items in the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SectionType {
    Calculator,
    Windows,
    Commands,
    Applications,
    SearchAndAi, // Combined AI + Search section (no gap between them)
}

/// Implement ListDelegate trait for GPUI integration.
impl ListDelegate for ItemListDelegate {
    type Item = GpuiListItem;

    fn sections_count(&self, _cx: &App) -> usize {
        let has_calc = self.calculator_item.is_some();
        let has_search_and_ai = self.ai_item.is_some() || !self.search_items.is_empty();
        let has_windows = self.section_info.window_count > 0;
        let has_commands = self.section_info.command_count > 0;
        let has_apps = self.section_info.app_count > 0;

        let mut count = 0;
        if has_calc {
            count += 1;
        }
        if has_windows {
            count += 1;
        }
        if has_commands {
            count += 1;
        }
        if has_apps {
            count += 1;
        }
        if has_search_and_ai {
            count += 1; // Combined AI + Search section
        }

        count
    }

    fn items_count(&self, section: usize, _cx: &App) -> usize {
        let section_type = self.section_type_at(section);
        match section_type {
            SectionType::Calculator => 1,
            SectionType::Windows => self.section_info.window_count,
            SectionType::Commands => self.section_info.command_count,
            SectionType::Applications => self.section_info.app_count,
            SectionType::SearchAndAi => {
                let ai_count = if self.ai_item.is_some() { 1 } else { 0 };
                ai_count + self.search_items.len()
            }
        }
    }

    fn render_section_header(
        &mut self,
        section: usize,
        _window: &mut Window,
        _cx: &mut Context<'_, ListState<Self>>,
    ) -> Option<impl IntoElement> {
        let section_type = self.section_type_at(section);

        // Show "Search and AI" header when we have regular items above
        let has_regular_items = self.section_info.window_count > 0
            || self.section_info.command_count > 0
            || self.section_info.app_count > 0
            || self.calculator_item.is_some();

        if section_type == SectionType::SearchAndAi && has_regular_items {
            let theme = theme();
            return Some(
                div()
                    .w_full()
                    .px(theme.item_margin_x + theme.item_padding_x)
                    .pt(theme.section_header.margin_top)
                    .pb(theme.section_header.margin_bottom)
                    .text_xs()
                    .font_weight(gpui::FontWeight::EXTRA_BOLD)
                    .text_color(theme.section_header.color)
                    .child(SharedString::from("Search and AI")),
            );
        }

        // SearchAndAi (without regular items) has no header
        if section_type == SectionType::SearchAndAi {
            return None;
        }

        // Count how many non-special sections we have
        let has_windows = self.section_info.window_count > 0;
        let has_commands = self.section_info.command_count > 0;
        let has_apps = self.section_info.app_count > 0;
        let non_special_section_count =
            has_windows as usize + has_commands as usize + has_apps as usize;
        let has_search_and_ai = self.ai_item.is_some() || !self.search_items.is_empty();

        // Show headers if we have multiple non-special sections
        // OR if we have SearchAndAi section (to visually separate them from regular items)
        if non_special_section_count <= 1 && !has_search_and_ai {
            return None;
        }

        let theme = theme();
        let title = match section_type {
            SectionType::Calculator => "Calculator",
            SectionType::SearchAndAi => return None,
            SectionType::Windows => "Windows",
            SectionType::Commands => "Commands",
            SectionType::Applications => "Applications",
        };

        Some(
            div()
                .w_full()
                .px(theme.item_margin_x + theme.item_padding_x)
                .pt(theme.section_header.margin_top)
                .pb(theme.section_header.margin_bottom)
                .text_xs()
                .font_weight(gpui::FontWeight::EXTRA_BOLD)
                .text_color(theme.section_header.color)
                .child(SharedString::from(title)),
        )
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        _cx: &mut Context<'_, ListState<Self>>,
    ) -> Option<Self::Item> {
        let global_idx = self.section_row_to_global(ix.section, ix.row);
        let selected = self.base.selected_index() == Some(global_idx);

        let item = self.get_item_at(global_idx)?;
        let item_content = render_item(&item, selected, global_idx);

        // Reset ListItem default padding - we handle all styling ourselves
        Some(
            GpuiListItem::new(("list-item", global_idx))
                .py_0()
                .px_0()
                .child(item_content),
        )
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        let global_idx = ix
            .map(|i| self.section_row_to_global(i.section, i.row))
            .unwrap_or(0);

        // Use unchecked method to allow selection of dynamic items (AI, Search)
        // that are beyond the base filtered count
        self.base.set_selected_unchecked(global_idx);
    }

    fn perform_search(
        &mut self,
        query: &str,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        self.set_query(query.to_string());
        Task::ready(())
    }

    fn confirm(
        &mut self,
        _secondary: bool,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        self.do_confirm();
    }

    fn cancel(&mut self, _window: &mut Window, _cx: &mut Context<ListState<Self>>) {
        self.do_cancel();
    }

    fn render_empty(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<'_, ListState<Self>>,
    ) -> impl IntoElement {
        let theme = theme();
        div()
            .w_full()
            .h(theme.empty_state_height)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .text_color(theme.empty_state_color)
                    .child(SharedString::from("No items found")),
            )
    }
}
