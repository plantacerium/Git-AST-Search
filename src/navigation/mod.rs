pub mod modes;

pub use modes::NavMode;

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
    }
}

impl Default for NavigationState {
    fn default() -> Self {
        Self::new()
    }
}
