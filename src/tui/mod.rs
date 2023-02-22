use std::io;
use tui::{backend::CrosstermBackend,Terminal};

fn tui_main() -> std::io::Result<()> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend);
    Ok(())
}
