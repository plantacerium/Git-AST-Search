pub mod modes;

pub use modes::NavMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Results,
    History,
}

#[derive(Debug, Clone)]
pub struct NavigationState {
    pub mode: NavMode,
    pub page: usize,
    pub total_results: usize,
    pub results_per_page: usize,
    pub sidebar_visible: bool,
    pub command_buffer: String,
    /// Currently selected result index within current page
    pub selected_idx: usize,
    pub active_pane: ActivePane,
    pub history_selected_idx: usize,
    pub pending_g: bool,
    pub overlay_content: Option<(String, String)>,
    pub overlay_lines: Option<Vec<ratatui::text::Line<'static>>>,
    pub overlay_scroll: u16,
}

impl NavigationState {
    pub fn new() -> Self {
        Self {
            mode: NavMode::Normal,
            page: 0,
            total_results: 0,
            results_per_page: 6,
            sidebar_visible: true,
            command_buffer: String::new(),
            selected_idx: 0,
            active_pane: ActivePane::Results,
            history_selected_idx: 0,
            pending_g: false,
            overlay_content: None,
            overlay_lines: None,
            overlay_scroll: 0,
        }
    }

    pub fn total_pages(&self) -> usize {
        if self.total_results == 0 {
            1
        } else {
            (self.total_results.saturating_sub(1) / self.results_per_page) + 1
        }
    }

    pub fn update_results(&mut self, count: usize) {
        self.total_results = count;
        self.page = 0;
        self.selected_idx = 0;
    }

    pub fn next_page(&mut self) {
        if (self.page + 1) < self.total_pages() {
            self.page += 1;
            self.selected_idx = 0;
        }
    }

    pub fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
            self.selected_idx = 0;
        }
    }

    /// Items visible in the current page
    pub fn current_page_count(&self) -> usize {
        let start = self.page * self.results_per_page;
        if self.total_results <= start {
            0
        } else {
            (self.total_results - start).min(self.results_per_page)
        }
    }

    /// Move selection down (j / ↓)
    pub fn select_next(&mut self) {
        let count = self.current_page_count();
        if count == 0 {
            return;
        }
        if self.selected_idx + 1 < count {
            self.selected_idx += 1;
        } else if (self.page + 1) < self.total_pages() {
            self.page += 1;
            self.selected_idx = 0;
        }
    }

    /// Move selection up (k / ↑)
    pub fn select_prev(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        } else if self.page > 0 {
            self.page -= 1;
            self.selected_idx = self.current_page_count().saturating_sub(1);
        }
    }

    /// Jump to first result (gg / /first)
    pub fn select_first(&mut self) {
        self.page = 0;
        self.selected_idx = 0;
    }

    /// Jump to last result (G / /last)
    pub fn select_last(&mut self) {
        if self.total_results == 0 {
            self.page = 0;
            self.selected_idx = 0;
        } else {
            self.page = self.total_pages().saturating_sub(1);
            self.selected_idx = self.current_page_count().saturating_sub(1);
        }
    }

    /// Jump to a 1-based page number (/page <n>)
    pub fn goto_page(&mut self, page_1based: usize) -> bool {
        let total = self.total_pages();
        if page_1based >= 1 && page_1based <= total {
            self.page = page_1based - 1;
            self.selected_idx = 0;
            true
        } else {
            false
        }
    }

    pub fn show_overlay(&mut self, title: impl Into<String>, body: impl Into<String>) {
        self.overlay_content = Some((title.into(), body.into()));
        self.overlay_lines = None;
        self.overlay_scroll = 0;
    }

    pub fn show_styled_overlay(&mut self, title: impl Into<String>, lines: Vec<ratatui::text::Line<'static>>) {
        self.overlay_content = Some((title.into(), String::new()));
        self.overlay_lines = Some(lines);
        self.overlay_scroll = 0;
    }

    pub fn close_overlay(&mut self) {
        self.overlay_content = None;
        self.overlay_lines = None;
        self.overlay_scroll = 0;
    }

    /// Absolute index of selected result in `results` vec
    pub fn global_selected(&self) -> usize {
        self.page * self.results_per_page + self.selected_idx
    }

    pub fn set_mode(&mut self, mode: NavMode) {
        self.mode = mode;
        if mode != NavMode::Command {
            self.command_buffer.clear();
        }
    }

    pub fn enter_command_mode(&mut self) {
        self.set_mode(NavMode::Command);
        self.command_buffer = String::from("/");
    }

    pub fn exit_mode(&mut self) {
        self.set_mode(NavMode::Normal);
    }

    pub fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
        if !self.sidebar_visible && self.active_pane == ActivePane::History {
            self.active_pane = ActivePane::Results;
        }
    }

    pub fn toggle_pane(&mut self) {
        if !self.sidebar_visible {
            self.sidebar_visible = true;
            self.active_pane = ActivePane::History;
        } else {
            self.active_pane = match self.active_pane {
                ActivePane::Results => ActivePane::History,
                ActivePane::History => ActivePane::Results,
            };
        }
    }

    pub fn set_active_pane(&mut self, pane: ActivePane) {
        if pane == ActivePane::History {
            self.sidebar_visible = true;
        }
        self.active_pane = pane;
    }

    pub fn history_next(&mut self, total_history: usize) {
        if total_history == 0 {
            self.history_selected_idx = 0;
            return;
        }
        if self.history_selected_idx + 1 < total_history {
            self.history_selected_idx += 1;
        }
    }

    pub fn history_prev(&mut self) {
        if self.history_selected_idx > 0 {
            self.history_selected_idx -= 1;
        }
    }
}

impl Default for NavigationState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_toggle_and_history_nav() {
        let mut nav = NavigationState::new();
        assert_eq!(nav.active_pane, ActivePane::Results);
        assert_eq!(nav.history_selected_idx, 0);

        nav.toggle_pane();
        assert_eq!(nav.active_pane, ActivePane::History);

        nav.history_next(5);
        assert_eq!(nav.history_selected_idx, 1);

        nav.history_next(5);
        assert_eq!(nav.history_selected_idx, 2);

        nav.history_prev();
        assert_eq!(nav.history_selected_idx, 1);

        nav.toggle_pane();
        assert_eq!(nav.active_pane, ActivePane::Results);
    }
}

