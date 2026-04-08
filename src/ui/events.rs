/// Manejo de eventos
use super::app::App;

pub fn handle_events(_app: &mut App) -> std::io::Result<()> {
    // use crossterm::event::{self, Event, KeyCode, KeyEvent};
    /*
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            // Procesamiento de teclas
        }
    }
    */
    Ok(())
}
