mod api_client;
mod config;
mod state;
mod ui;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use state::AppState;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = Config::load()?;
    let mut state = AppState::new(config);

    // Initial refresh
    state.refresh_devices().await;

    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
    terminal.hide_cursor()?;

    terminal.draw(|f| ui::render(f, &state))?;

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                            break;
                        }
                        _ => {
                            state.handle_key(key).await;
                        }
                    }
                }
            }
        }
        terminal.draw(|f| ui::render(f, &state))?;
    }

    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}