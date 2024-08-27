use ratatui::prelude::*;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
            LeaveAlternateScreen,
        },
        ExecutableCommand,
    },
    widgets::{Block, Paragraph},
    Frame, Terminal,
};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use std::fs::File;
use std::io::{self, stdout, Read};

mod colors;
mod soko_loader;
mod types;

impl Widget for types::Level {
    #[allow(clippy::cast_precision_loss, clippy::similar_names)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut row_pixels;
        let mut row_iter = self.map.outer_iter();
        let mut yi: usize = 0;
        while let Some(row_top) = row_iter.next() {
            let maybe_row_bottom = row_iter.next();

            let top_colors;
            let bottom_colors;
            match maybe_row_bottom {
                Some(row_bottom) => {
                    top_colors = row_top.map(|ent| ent.color());
                    bottom_colors = row_bottom.map(|ent| ent.color());
                    row_pixels = top_colors.iter().zip(bottom_colors.iter());
                }
                None => {
                    top_colors = row_top.map(|ent| ent.color());
                    bottom_colors = row_top.map(|_| None);
                    row_pixels = top_colors.iter().zip(bottom_colors.iter());
                }
            }

            for (xi, (fg, bg)) in row_pixels.enumerate() {
                let curs = &mut buf[(xi as u16 + area.x, yi as u16 + area.y)];
                curs.set_char('▀');
                if let Some(fg) = fg {
                    curs.set_fg(*fg);
                }
                if let Some(bg) = bg {
                    curs.set_bg(*bg);
                }
            }

            yi += 1;
        }
        for entity in self.entities {
            match entity {
                // TODO: Do this with traits or something, These render the exact same
                types::Entity::Player(player) => {
                    let px_x = player.coords.x as u16 + area.x;
                    let px_y = (player.coords.y / 2) as u16 + area.y;
                    if area.contains(Position { x: px_x, y: px_y }) {
                        let curs = &mut buf[(px_x, px_y)];
                        if player.coords.y % 2 == 0 {
                            curs.set_fg(player.color);
                        } else {
                            curs.set_bg(player.color);
                        }
                    }
                }
                types::Entity::Chest(chest) => {
                    let px_x = chest.coords.x;
                    let px_y = chest.coords.y / 2;
                    let curs = &mut buf[(px_x as u16 + area.x, px_y as u16 + area.y)];
                    if chest.coords.y % 2 == 0 {
                        curs.set_fg(chest.color);
                    } else {
                        curs.set_bg(chest.color);
                    }
                }
            }
        }
    }
}

fn read_file(filename: &str) -> Result<String, io::Error> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let filename = "./resources/levels/micro.ban";
    // TODO: actually handle errors here
    let mut level = read_file(filename)
        .map(|contents| soko_loader::load_level(&contents).unwrap())
        .unwrap();

    let title = level.name.clone();

    loop {
        let mut debug: Vec<String> = Vec::new();

        for entity in level.entities.iter_mut() {
            if let types::Entity::Player(player) = entity {
                debug.push(format!("{:?}", player.coords.clone()));
            }
        }

        terminal.draw(|frame: &mut Frame| {
            let main_area = frame.area();

            let [left_area, right_area] = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .areas(main_area);

            let outer_left_block = Block::bordered().title(title.clone());
            let inner_left = outer_left_block.inner(left_area);

            frame.render_widget(outer_left_block, left_area);
            frame.render_widget(level.clone(), inner_left);

            let text = debug.join("\n");
            frame.render_widget(
                Paragraph::new(text).block(Block::bordered().title("debug")),
                right_area,
            );
        })?;

        match handle_events()? {
            types::Action::Quit => {
                break;
            }
            types::Action::Move(direction) => {
                // Iterate over mutable references to entities
                let mut player_move = None;
                for (index, entity) in level.entities.iter().enumerate() {
                    if let types::Entity::Player(player) = entity {
                        let new_chords =
                            get_new_coords(player.coords.clone(), &direction);

                        match level.map[[new_chords.y, new_chords.x]] {
                            types::Tile::Wall => player_move = None,
                            _ => player_move = Some((index, new_chords)),
                        }
                        break;
                    }
                }

                let mut chest_move = None;
                if let Some((_, player_coords)) = player_move.clone() {
                    for (index, entity) in level.entities.iter().enumerate() {
                        if let types::Entity::Chest(chest) = entity {
                            if let Some((_, ref chest_coords)) = chest_move {
                                // if the place we are trying to move has a chest we can't do it.
                                if chest.coords == *chest_coords {
                                    chest_move = None;
                                    player_move = None;
                                    break;
                                }
                            } else if chest.coords == player_coords.clone() {
                                let new_coords =
                                    get_new_coords(chest.coords.clone(), &direction);

                                match level.map[[new_coords.y, new_coords.x]] {
                                    types::Tile::Wall => {
                                        player_move = None;
                                        break;
                                    }
                                    _ => {
                                        chest_move = Some((index, new_coords.clone()));
                                        for ent in level.entities.iter() {
                                            if ent.get_coords() == new_coords {
                                                chest_move = None;
                                                player_move = None;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some((index, new_coords)) = player_move {
                    if let types::Entity::Player(ref mut player) =
                        &mut level.entities[index]
                    {
                        player.coords = new_coords.clone();
                    }
                }
                if let Some((index, new_coords)) = chest_move {
                    if let types::Entity::Chest(ref mut chest) =
                        &mut level.entities[index]
                    {
                        chest.coords = new_coords.clone();
                    }
                }
            }
            types::Action::None => {}
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn get_new_coords(
    coords: types::Coordinates,
    direction: &types::Direction,
) -> types::Coordinates {
    match direction {
        types::Direction::Up => types::Coordinates {
            x: coords.x,
            y: if coords.y > 0 { coords.y - 1 } else { 0 },
        },
        types::Direction::Down => types::Coordinates {
            x: coords.x,
            y: coords.y + 1,
        },
        types::Direction::Left => types::Coordinates {
            x: if coords.x > 0 { coords.x - 1 } else { 0 },
            y: coords.y,
        },
        types::Direction::Right => types::Coordinates {
            x: coords.x + 1,
            y: coords.y,
        },
    }
}

fn handle_events() -> io::Result<types::Action> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q')
            {
                return Ok(types::Action::Quit);
            }
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('w')
            {
                return Ok(types::Action::Move(types::Direction::Up));
            }
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('s')
            {
                return Ok(types::Action::Move(types::Direction::Down));
            }
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('a')
            {
                return Ok(types::Action::Move(types::Direction::Left));
            }
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('d')
            {
                return Ok(types::Action::Move(types::Direction::Right));
            }
        }
    }
    Ok(types::Action::None)
}
