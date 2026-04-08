use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Calculates the main layout regions for the application (Sidebar, Main Area, Status, Input)
pub fn calculate_main_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),      // Main Application Area
            Constraint::Length(1),   // Status Bar
            Constraint::Length(7),   // Input Area
        ])
        .split(area)
}

/// Splits the main area into a Sidebar and a Content Area based on whether the sidebar is visible
pub fn calculate_sidebar_layout(area: Rect, sidebar_visible: bool) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(if sidebar_visible { 25 } else { 0 }),
            Constraint::Percentage(if sidebar_visible { 75 } else { 100 }),
        ])
        .split(area)
}

/// Computes the vertical chunks for rows within the result grid
pub fn calculate_grid_rows(area: Rect, rows: u16) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100 / rows); rows as usize])
        .split(area)
}

/// Computes horizontal chunks for a specific row in the result grid
pub fn calculate_grid_cols(area: Rect, cols: u16) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(100 / cols); cols as usize])
        .split(area)
}
