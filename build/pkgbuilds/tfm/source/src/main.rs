mod app;
mod ui;
mod fs_ops;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use app::App;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()? {
            match (key.modifiers, key.code) {
                (_, KeyCode::Char('q')) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => break,
                (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.move_up(),
                (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.move_down(),
                (_, KeyCode::Enter) => app.enter()?,
                (_, KeyCode::Backspace) | (_, KeyCode::Left) => app.go_parent()?,
                (_, KeyCode::Tab) => app.switch_panel(),
                (_, KeyCode::Char('d')) => app.delete()?,
                (_, KeyCode::Char('r')) => app.rename_start(),
                (_, KeyCode::Char('n')) => app.new_dir_start(),
                (_, KeyCode::Esc) => app.cancel_input(),
                (_, KeyCode::Char(c)) => app.handle_char(c),
                _ => {}
            }
        }
    }
    Ok(())
}
