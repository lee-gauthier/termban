/*!
 * Author: Lee Gauthier
 *
 * Description:
 * This module is responsible for loading Sokoban game levels designed by David W. Skinner.
 * It provides functionality to parse and load level data from files into the game structure.
 * Each level is parsed and constructed into a game-ready format to be utilized by the game engine.
 *
 * This loader supports the standard Sokoban level format as specified by David W. Skinner,
 * ensuring compatibility and correctness across different Sokoban level sets.
 *
 * Usage:
 * The module functions are intended to be used by the game engine to initialize and start
 * new games with pre-defined levels. Errors in file format or content are handled gracefully
 * and reported to the caller.
 *
 * Notes:
 * The levels are expected to be stored in plain text format with each level separated by
 * a newline and starting with a level identifier.
 */

use crate::types::{Coordinate, Entity, Player, SokoBox, Tile, World};
use ndarray::Array2;

#[derive(Debug, Clone)]
enum Token {
    Text(String),
    Wall,
    Player,
    SokoBox,
    Goal,
    SokoBoxAndGoal,
    PlayerAndGoal,
    Empty,
    NewLine,
}

type Tokens = Vec<Token>;

fn tokenize(contents: &str) -> Option<Tokens> {
    let mut tokens: Tokens = Tokens::new();
    for line in contents.lines() {
        if line.starts_with(';') {
            // TODO: Do this less dumb
            let mut line_chars = line.chars();
            line_chars.next();
            line_chars.next();
            tokens.push(Token::Text(line_chars.as_str().to_string()));
            continue;
        }
        if line.is_empty() {
            continue;
        }
        for ch in line.chars() {
            match ch {
                '#' => tokens.push(Token::Wall),
                '@' => tokens.push(Token::Player),
                '$' => tokens.push(Token::SokoBox),
                '.' => tokens.push(Token::Goal),
                '*' => tokens.push(Token::SokoBoxAndGoal),
                '+' => tokens.push(Token::PlayerAndGoal),
                ' ' => tokens.push(Token::Empty),
                _ => return None,
            }
        }
        tokens.push(Token::NewLine);
    }
    Some(tokens)
}

fn get_board_dimensions(tokens: &[Token]) -> (usize, usize) {
    let mut x = 0;
    let y = tokens
        .iter()
        .filter(|tok| matches!(tok, Token::NewLine))
        .count();

    let mut count: usize = 0;
    for token in tokens {
        match token {
            Token::NewLine => {
                x = if count > x { count } else { x };
                count = 0;
            }
            _ => {
                count += 1;
            }
        }
    }
    (x, y)
}

// Return a Vector of each sokoban level (as a Vector of Tokens)
fn group_sokoban_tokens(tokens: Vec<Token>) -> Vec<Vec<Token>> {
    // Collect all indexes where the element is alphabetic
    let mut text_indexes: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| matches!(token, Token::Text(_title)))
        .map(|(index, _)| index)
        .collect();

    // Add the length of the vector as the last index to handle the final group
    text_indexes.push(tokens.len());

    // Use the indexes to create slices of the vector with map
    text_indexes
        .windows(2)
        .map(|window| {
            let start = window[0];
            let end = window[1];
            tokens[start..end].to_vec()
        })
        .collect()
}

// Parses a Single level in the form of a Vec of tokens
fn parse_sokoban_level(tokens: Vec<Token>) -> Result<World, String> {
    match tokens.as_slice() {
        [Token::Text(title), level_toks @ ..] => {
            // Dimensions for the board
            let (x, y) = get_board_dimensions(level_toks);

            // Create an initial board with default values (e.g., all `Wall`)
            let mut board = Array2::from_elem((y, x), Tile::Empty);
            let mut entities = Vec::new();

            let (mut x, mut y): (usize, usize) = (0, 0);
            for tok in level_toks.iter() {
                match tok {
                    Token::Wall => {
                        board[[y, x]] = Tile::Wall;
                    }
                    Token::Player => {
                        entities.push(Entity::Player(Player {
                            position: Coordinate { x, y },
                        }));
                    }
                    Token::Goal => {
                        board[[y, x]] = Tile::Goal;
                    }
                    Token::SokoBox => {
                        entities.push(Entity::SokoBox(SokoBox {
                            position: Coordinate { x, y },
                        }));
                    }
                    Token::SokoBoxAndGoal => {
                        board[[y, x]] = Tile::Goal;
                        entities.push(Entity::SokoBox(SokoBox {
                            position: Coordinate { x, y },
                        }));
                    }
                    Token::PlayerAndGoal => {
                        board[[y, x]] = Tile::Goal;
                        entities.push(Entity::Player(Player {
                            position: Coordinate { x, y },
                        }));
                    }
                    Token::NewLine => {
                        y += 1;
                        x = 0;
                        continue;
                    }
                    _ => {}
                }
                x += 1;
            }

            // Create an instance of Level
            let level = World {
                name: title.to_string(),
                board,
                entities,
                camera_position: Coordinate { x: 0, y: 0 },
            };
            Ok(level)
        }
        _ => Err("Level must start with a title".to_string()),
    }
}

pub fn parse_sokoban_worlds(sokoban_text: &str) -> Result<Vec<World>, String> {
    if let Some(tokens) = tokenize(sokoban_text) {
        let sokoban_token_groups = group_sokoban_tokens(tokens);
        let mut worlds = Vec::new();
        for level in sokoban_token_groups {
            if let Ok(world) = parse_sokoban_level(level.clone()) {
                worlds.push(world);
            }
        }
        Ok(worlds)
    } else {
        Err("Level failed to load".to_string())
    }
}
