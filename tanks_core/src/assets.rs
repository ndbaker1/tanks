use std::{collections::HashMap, str::Utf8Error};

use crate::common::environment::{Environment, Tile};

/// Character to denote the start and end of a map in a MapData file
///
/// # Example
///
/// file contents:
/// ```plaintext
/// @MAP_NAME
/// ..........
/// ..x11.x1..
/// ..........
/// ..........
/// @
/// ```
///
/// you can have multiple definitions of a map in a single map file
pub fn parse_environments_file(
    map_bytes: &[u8],
) -> Result<HashMap<String, Environment>, Utf8Error> {
    const MAP_DELIMITER: char = '@';

    let mut maps = HashMap::new();

    let mut reading = false;
    let mut relative_row = 0;
    let mut current_environment_entry = (String::default(), Environment::default());

    for (row, line) in std::str::from_utf8(map_bytes)?.lines().enumerate() {
        if line.starts_with(MAP_DELIMITER) {
            if reading {
                maps.insert(current_environment_entry.0, current_environment_entry.1);
                current_environment_entry = (String::default(), Environment::default());
            } else {
                current_environment_entry.0 = String::from(&line[1..]);
                relative_row = row;
            }

            // toggle reading state
            reading = !reading;
        } else {
            for (col, sym) in line.chars().enumerate() {
                match sym {
                    // indestructable
                    '1'..='5' => {
                        let elevation = sym.to_digit(10).unwrap() as usize;
                        current_environment_entry.1.tiles.insert(
                            (row - relative_row, col),
                            Tile::IndestructableWall(elevation),
                        );
                    }
                    // destructable
                    '6'..='9' => {
                        let elevation = sym.to_digit(10).unwrap() as usize - 5;
                        current_environment_entry.1.tiles.insert(
                            (row - relative_row, col),
                            Tile::DesructableWall((3, elevation)),
                        );
                    }
                    // empty
                    'x' => {
                        current_environment_entry
                            .1
                            .tiles
                            .insert((row - relative_row, col), Tile::Empty);
                    }
                    _ => (),
                };
            }
        }
    }

    Ok(maps)
}
