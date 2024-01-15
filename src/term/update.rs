use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::App;

pub fn update(app: &mut App, key_event: KeyEvent) {
    if !app.download {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => app.quit(),
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.quit();
                }
            },
            KeyCode::Up => app.previous(),
            KeyCode::Down => app.next(),
            KeyCode::Left => app.left(),
            KeyCode::Right => app.right(),
            KeyCode::Home => app.home(),
            KeyCode::End => app.end(),
            KeyCode::Char(' ') => app.toggle(),
            KeyCode::Enter => app.download = true,
            KeyCode::Tab => app.toggle_all(),
            _ => {},
        }
    } else {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => app.quit(),
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.download = false;
                }
            },
            KeyCode::Enter => app.quit(),
            _ => {},
        }
    }
}
