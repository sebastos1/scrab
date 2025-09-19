use crate::{
    Direction, Game, Pos, Tile,
    engine::moves::{Move, PlayedTile},
};
use csv::Reader;
use memmap2::{Mmap, MmapOptions};
use smallvec::SmallVec;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

#[derive(Clone, Debug)]
pub struct GameRecord {
    pub game_id: String,
    pub moves: Vec<GameMove>,
}

#[derive(Clone, Debug)]
pub struct GameMove {
    pub player: usize,
    pub action: Action,
    pub rack: Vec<Tile>,
    pub equity: f32,
}

#[derive(Clone, Debug)]
pub enum Action {
    Move(Move),
    Swap(Vec<Tile>),
    Pass,
}

// Macondo self-play format
#[derive(serde::Deserialize)]
pub struct CsvRow {
    #[serde(rename = "playerID")]
    pub player_id: String,
    #[serde(rename = "gameID")]
    pub game_id: String,
    pub turn: u32,
    pub rack: String,
    pub play: String,
    pub score: u16,
    pub equity: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TrainingPosition {
    pub board: [[u8; 15]; 15], // 225 bytes, board tiles (0=empty, 1-27=tiles)
    pub rack_counts: [u8; 27], // player rack counts
    pub bag_counts: [u8; 27],  // unseen tiles
    pub my_score: u16,
    pub opp_score: u16,
    pub scoreless_turns: u8,
    pub target_equity: f32,
    _padding: [u8; 3], // padding to align
}

impl GameRecord {
    pub fn from_csv(csv_content: &str) -> Result<Vec<GameRecord>, Box<dyn std::error::Error>> {
        let mut reader = Reader::from_reader(csv_content.as_bytes());
        let mut games: HashMap<String, Vec<CsvRow>> = HashMap::new();

        for result in reader.deserialize() {
            let row: CsvRow = result?;
            games.entry(row.game_id.clone()).or_default().push(row);
        }

        let mut records = Vec::new();
        for (game_id, mut rows) in games {
            rows.sort_by_key(|r| r.turn);

            let moves = rows
                .into_iter()
                .map(|row| {
                    let player = if row.player_id == "p1" { 0 } else { 1 };
                    let rack = row
                        .rack
                        .chars()
                        .map(|c| match c {
                            '?' => Tile::blank(None),
                            c => Tile::letter(c as u8),
                        })
                        .collect();

                    let action = match row.play.trim() {
                        "(Pass)" => Action::Pass,
                        s if s.starts_with("(exch ") => {
                            let tiles_str = &s[6..s.len() - 1];
                            let tiles = tiles_str
                                .chars()
                                .map(|c| match c {
                                    '?' => Tile::blank(None),
                                    c => Tile::letter(c as u8),
                                })
                                .collect();
                            Action::Swap(tiles)
                        }
                        _ => Action::Move(parse_move(&row.play, row.score).unwrap()),
                    };

                    GameMove {
                        player,
                        action,
                        rack,
                        equity: row.equity,
                    }
                })
                .collect();

            records.push(GameRecord { game_id, moves });
        }

        Ok(records)
    }

    pub fn csv_to_positions(csv_path: &str, output_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let csv_content = std::fs::read_to_string(csv_path)?;
        let records = Self::from_csv(&csv_content)?;

        let mut writer = BufWriter::new(File::create(output_path)?);
        let mut position_count = 0;

        for record in records {
            let mut game = Game::init();

            for game_move in &record.moves {
                let mut pos = TrainingPosition {
                    board: [[0; 15]; 15],
                    rack_counts: [0; 27],
                    bag_counts: [0; 27],
                    my_score: game.scores[game.current_player],
                    opp_score: game.scores[1 - game.current_player],
                    scoreless_turns: game.zeroed_turns,
                    _padding: [0; 3],
                    target_equity: game_move.equity,
                };

                for row in 0..15 {
                    for col in 0..15 {
                        if let Some(tile) = game.board.get_board_tile(Pos::new(row, col)) {
                            pos.board[row][col] = tile.to_index() + 1; // 0=empty, 1-27=tiles
                        }
                    }
                }

                for tile in game.racks[game.current_player].tiles() {
                    pos.rack_counts[tile.to_index() as usize] += 1;
                }

                for i in 0..27 {
                    pos.bag_counts[i] = game.bag.count(i);
                }
                for tile in game.racks[1 - game.current_player].tiles() {
                    pos.bag_counts[tile.to_index() as usize] += 1;
                }

                let bytes = unsafe { std::slice::from_raw_parts(&pos as *const _ as *const u8, std::mem::size_of::<TrainingPosition>()) };
                writer.write_all(bytes)?;

                match &game_move.action {
                    Action::Move(mv) => game.play_move(mv),
                    Action::Pass => game.pass_turn(),
                    Action::Swap(tiles) => game.exchange(tiles.clone()),
                }

                position_count += 1;
            }
        }

        writer.flush()?;
        println!("Wrote {} positions to {}", position_count, output_path);
        Ok(position_count)
    }
}

pub fn parse_move(play_str: &str, score: u16) -> Result<Move, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = play_str.trim().split_whitespace().collect();
    let pos_str = parts[0];
    let word = parts[1];

    let chars: Vec<char> = pos_str.chars().collect();
    let (pos, direction) = if chars[0].is_ascii_digit() {
        let digits: String = chars.iter().take_while(|c| c.is_ascii_digit()).collect();
        let row = digits.parse::<usize>()? - 1;
        let col = (chars[digits.len()] as u8 - b'A') as usize;
        (Pos::new(row, col), Direction::Horizontal)
    } else {
        let col = (chars[0] as u8 - b'A') as usize;
        let row_str: String = chars[1..].iter().collect();
        let row = row_str.parse::<usize>()? - 1;
        (Pos::new(row, col), Direction::Vertical)
    };

    let tiles_data: SmallVec<[PlayedTile; 7]> = word
        .chars()
        .map(|c| match c {
            '.' => PlayedTile::Board(Tile::letter(b'.')),
            c if c.is_lowercase() => PlayedTile::Rack(Tile::blank(Some(c.to_ascii_uppercase() as u8))),
            c => PlayedTile::Rack(Tile::letter(c as u8)),
        })
        .collect();

    Ok(Move {
        tiles_data,
        pos,
        direction,
        score,
    })
}

pub struct PositionsReader {
    mmap: Mmap,
    count: usize,
}

impl PositionsReader {
    pub fn open(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let file_len = file.metadata()?.len() as usize;
        let position_size = std::mem::size_of::<TrainingPosition>();
        let count = file_len / position_size;

        let mmap = unsafe { MmapOptions::new().map(&file)? };

        Ok(PositionsReader { mmap, count })
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn get(&self, index: usize) -> Option<&TrainingPosition> {
        if index >= self.count {
            return None;
        }

        unsafe {
            let positions = std::slice::from_raw_parts(self.mmap.as_ptr() as *const TrainingPosition, self.count);
            Some(&positions[index])
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &TrainingPosition> {
        let positions = unsafe { std::slice::from_raw_parts(self.mmap.as_ptr() as *const TrainingPosition, self.count) };
        positions.iter()
    }
}
