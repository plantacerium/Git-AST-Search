use ratatui::Frame;
use super::app::App;
use super::layout::{calculate_main_layout, calculate_sidebar_layout};
use super::components::{sidebar, results_grid, search_bar, status_bar, help_overlay};
use crate::navigation::modes::NavMode;

/// Orchestrates the different UI components and passes them the appropriate area
pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let chunks = calculate_main_layout(f.area());

    let main_area = calculate_sidebar_layout(chunks[0], app.nav_state.sidebar_visible);

    if app.nav_state.sidebar_visible {
        sidebar::render(f, main_area[0], app);
    }

    if app.nav_state.mode == NavMode::Help {
        help_overlay::render_help(f, main_area[1]);
    } else if app.results.is_empty() && !app.is_searching {
        help_overlay::render_welcome(f, main_area[1], app);
    } else {
        results_grid::render(f, main_area[1], app);
    }

    status_bar::render(f, chunks[1], app);
    search_bar::render(f, chunks[2], app);
}
