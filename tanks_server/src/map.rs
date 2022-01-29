use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use tanks_core::{MAP_HEIGHT, MAP_WIDTH};

const MAP_INDICATOR: char = '@';

#[derive(Clone, Default, Debug)]
pub struct MapData {
    pub map: Vec<Vec<usize>>,
    pub name: String,
}

pub fn parse_maps(filepath: &str) -> Vec<MapData> {
    log::info!("loading maps..");

    let file_lines = BufReader::new(File::open(filepath).expect("failed to open mapdata file"))
        .lines()
        .filter_map(|line_result| match line_result {
            Ok(line) => Some(line),
            _ => None,
        });

    let mut maps = Vec::new();
    let mut reading = false;
    let mut current_data = MapData::default();
    for (row, line) in file_lines.enumerate() {
        if line.starts_with(MAP_INDICATOR) {
            match reading {
                true => {
                    log::info!("map :: {:#?}", current_data);
                    maps.push(current_data.clone());
                }
                false => {
                    let name = String::from(&line[1..]);
                    let map = vec![vec![0; MAP_WIDTH]; MAP_HEIGHT];
                    current_data = MapData { name, map };
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
