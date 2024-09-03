use crate::types::{MenuAction, Model, RunningState};
use std::io;
use std::time::Duration;

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    widgets::Paragraph,
    Frame,
};

pub fn view(_model: &mut Model, frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new("Press space to start\nPress esc to Quit".to_string()),
        frame.area(),
    );
}

pub fn update(model: &mut Model, msg: MenuAction) -> Option<MenuAction> {
    match msg {
        MenuAction::StartGame => {
            model.running_state = RunningState::Game;
        }
        MenuAction::Quit => {
            // You can handle cleanup and exit here
            model.running_state = RunningState::Done;
        }
    };
    None
}

/// Convert Event to Message
///
/// We don't need to pass in a `model` to this function in this example
/// but you might need it as your project evolves
pub fn handle_event(_: &Model) -> io::Result<Option<MenuAction>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    Ok(None)
}

pub fn handle_key(key: event::KeyEvent) -> Option<MenuAction> {
    match key.code {
        KeyCode::Char(' ') => Some(MenuAction::StartGame),
        KeyCode::Esc => Some(MenuAction::Quit),
        _ => None,
    }
}
