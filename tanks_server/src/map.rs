use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};
use tanks_core::shared_types::Tile;

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
///
const MAP_DELIMITER: char = '@';

#[derive(Clone, Default, Debug)]
pub struct MapData {
    pub name: String,
    pub tile_data: HashMap<(usize, usize), Tile>,
}

pub fn parse_maps(filepath: &str) -> HashMap<String, MapData> {
    log::info!("loading maps..");

    let file_lines = BufReader::new(File::open(filepath).expect("failed to open mapdata file"))
        .lines()
        .filter_map(|line_result| match line_result {
            Ok(line) => Some(line),
            _ => None,
        });

    let mut maps = HashMap::new();
    let mut reading = false;
    let mut current_data = MapData::default();
    for (row, line) in file_lines.enumerate() {
        if line.starts_with(MAP_DELIMITER) {
            match reading {
                true => {
                    log::info!("map :: {:#?}", current_data);
                    maps.insert(current_data.name.clone(), current_data.clone());
                }
                false => {
                    current_data = MapData {
                        name: String::from(&line[1..]),
                        tile_data: HashMap::new(),
                    };
                }
            };

            reading = !reading;
        } else {
            for (col, sym) in line.chars().enumerate() {
                match sym {
                    '1'..='9' => {
                        let elevation = sym.to_digit(10).unwrap();
                    }
                    'x' => {}
                    _ => {}
                };
            }
        }
    }

    maps
}
