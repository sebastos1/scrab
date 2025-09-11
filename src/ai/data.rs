/*
woogles uses "gcg":
>HastyBot: AEEFHRU 8D FEUAR +24 24
>saviosas: ??AEKLN D8 .LANKErs +76 76
...
>saviosas: EIRS 2E REIS +21 326
>saviosas: (AU) +4 330

I save these in a sqlite db, already with some data from game parsing (fetch.py)
game_id | winner | gcg | player1 | player2
*/

use smallvec::SmallVec;

use crate::{
    Direction, Pos, Tile,
    engine::moves::{Move, PlayedTile},
};

pub fn test() -> GameRecord {
    let id = "SUKyN5ec";
    let winner = 0;
    let player1 = "HastyBot";
    let player2 = "marvin";
    let gcg = "#character-encoding UTF-8
        #description Created with Macondo
        #id io.woogles SUKyN5ec
        #lexicon CSW24
        #player1 HastyBot HastyBot
        #player2 marvin Amit Ch
        >HastyBot: DIOTTUV 8D DIVOT +22 22
        >marvin: ABHNNXY F8 .AX +29 29
        >HastyBot: JNOTUVY 7G JUNTO +40 62
        >marvin: BEHNNTY 8K HENNY +41 70
        >HastyBot: IILOUVY 9F .YU +30 92
        >marvin: BEFORTU 6K FORB +24 94
        >HastyBot: AAIILOV N8 .OVALIA +36 128
        >marvin: AEENSTU D1 UNSEATE. +70 164
        >HastyBot: EEIKLSU E10 EUK +23 151
        >marvin: AEGGIRS 13E SAGGIER +70 234
        >HastyBot: AEFILRS 4C S.RAFILE +62 213
        >marvin: ACENQSW 1C Q.ENAS +45 279
        >HastyBot: AEHIOOT D10 HOO +39 252
        >marvin: CEIPPRW 3J WIPER +25 304
        >HastyBot: AEGIIMT 2B MI.AE +35 287
        >marvin: CDIMNPT L1 DI.T +18 322
        >HastyBot: ?GIILLT 1L .ILL +15 302
        >marvin: BCEMNOP J10 MOB. +14 336
        >HastyBot: ??GITTZ O13 TIZ +40 342
        >marvin: CDDENPR K11 DE.P +24 360
        >HastyBot: ??EGRTW M12 EWT +29 371
        >marvin: ACCDNOR H12 A.ON +18 378
        >HastyBot: ??EGR 2I hEG.Ra +32 403
        >HastyBot: (CCDR) +18 421
        >marvin: CCDR (time) -10 368";

    let record = GameRecord::from_gcg(id.to_string(), Some(winner), gcg, player1.to_string(), player2.to_string()).unwrap();

    println!("moves: {:?}", record.moves);

    record
}

#[derive(Clone, Debug)]
pub struct GameRecord {
    pub game_id: String,
    pub winner: Option<usize>, // 0 or 1, None for draw
    pub moves: Vec<GameMove>,
    pub player1: String,
    pub player2: String,
}

#[derive(Clone, Debug)]
pub struct GameMove {
    pub player: usize,
    pub action: Action,
    pub rack: Vec<Tile>, // tiles BEFORE the move
}

#[derive(Clone, Debug)]
pub enum Action {
    Move(Move),
    Swap(Vec<Tile>),
    Pass,
}

impl GameRecord {
    pub fn from_gcg(
        game_id: String,
        winner: Option<usize>,
        gcg_content: &str,
        player1: String,
        player2: String,
    ) -> Result<GameRecord, Box<dyn std::error::Error>> {
        let mut moves = Vec::new();

        // only parse the move lines since we already have the metadata
        for line in gcg_content.lines() {
            let line = line.trim();
            // ignored:
            // phoney: >Oreoluwa: EKLLRRV -- -14 256 (must remove the last entry)
            // >HastyBot:  (challenge) +0 505
            // >marvin:  (time) -10 330
            if line.starts_with('>') && (!line.contains("(challenge)") && !line.contains("(time)")) {
                if line.contains("--") {
                    moves.pop(); // i'm getting bot games, so i dont think challenges ever miss
                    continue;
                }

                if let Some(game_move) = parse_line(line, &player1) {
                    moves.push(game_move);
                }
            }
        }

        Ok(GameRecord {
            game_id,
            winner,
            moves,
            player1,
            player2,
        })
    }
}

fn parse_line(line: &str, player1: &str) -> Option<GameMove> {
    if line.contains("(challenge)") || line.contains("(time)") {
        return None;
    }

    // >HastyBot: AEEFHRU 8D FEUAR +24 24
    // hastybot (0) plays FEAUR horizontally, starting at 8D which is row 8 and column D (4)
    // it's rack before the move was AEEFHRU which means that they will have E,H left after the move
    // for 24 points, which is all I care about

    // >saviosas: ??AEKLN D8 .LANKErs +76 76
    // saviosas (1) has two blanks (??) from their first draw (crazy biz)
    // they play LANKErs vertically (letter before number) from the F in FEUAR
    // for 76

    // in my internal format, this would be a move struct with:
    // pos at (number, letter)
    // direction is horizontal if letter after number, vertical if letter before number
    // tiles would be, for second example, a vec with:
    // PlayedTile::Board('F'), PlayedTile::Tile('L'), PT:T('A'), etc.

    let parts: Vec<&str> = line.split_whitespace().collect();
    let rack_str = parts[1];

    // end game
    // >BasicBot: (GLRR) +10 357
    // basic bot added the double value of GLRR (from opponent rack) to their score
    if rack_str.starts_with('(') {
        return None;
    }

    let player_name = &parts[0][1..parts[0].len() - 1];
    let player = if player_name == player1.split_whitespace().next().unwrap_or("") {
        0
    } else {
        1
    };
    let rack = parse_rack(rack_str)?;

    let move_str = parts[2];

    let action = match move_str {
        // pass
        // >LikeMike: EEEIRSZ - +0 96
        "-" => Action::Pass,

        // exchange
        // >Oreoluwa: EKLLRRV -LLRR +0 256
        // they exchanged LLRR. simple enough.
        desc if desc.starts_with('-') => Action::Swap(parse_exchange(&desc[1..])),

        // should be a move
        _ => Action::Move(parse_move(&format!("{} {}", parts[2], parts[3]), parts[4])),
    };

    Some(GameMove { player, action, rack })
}

fn parse_rack(rack_str: &str) -> Option<Vec<Tile>> {
    Some(
        rack_str
            .chars()
            .map(|c| match c {
                '?' => Tile::blank(None),
                c => Tile::letter(c as u8),
            })
            .collect(),
    )
}

fn parse_exchange(tiles_str: &str) -> Vec<Tile> {
    tiles_str
        .chars()
        .map(|c| match c {
            '?' => Tile::blank(None),
            c => Tile::letter(c as u8),
        })
        .collect()
}

fn parse_move(move_str: &str, score_str: &str) -> Move {
    println!("parsing move: {} {}", move_str, score_str);

    // "8D FEUAR +24" -> horizontal at (7,3), tiles=[F,E,U,A,R], score=24
    let parts: Vec<&str> = move_str.split_whitespace().collect();
    let pos_str = parts[0];
    let word = parts[1];
    let score = score_str.trim_start_matches('+').parse::<u16>().unwrap_or(0);

    // pos and direction
    let chars: Vec<char> = pos_str.chars().collect();
    let (pos, direction) = if chars[0].is_ascii_digit() {
        let digits: String = chars.iter().take_while(|c| c.is_ascii_digit()).collect();
        let row = digits.parse::<usize>().unwrap() - 1;
        let col = (chars[digits.len()] as u8 - b'A') as usize;
        (Pos::new(row, col), Direction::Horizontal)
    } else {
        let col = (chars[0] as u8 - b'A') as usize;
        let row_str: String = chars[1..].iter().collect();
        let row = row_str.parse::<usize>().unwrap() - 1;
        (Pos::new(row, col), Direction::Vertical)
    };

    let tiles_data: SmallVec<[PlayedTile; 7]> = word
        .chars()
        .map(|c| match c {
            '.' => PlayedTile::Board(Tile::letter(b'.')), // we dont know which one it is but we don't need it for training
            c if c.is_lowercase() => PlayedTile::Rack(Tile::blank(Some(c.to_ascii_uppercase() as u8))),
            c => PlayedTile::Rack(Tile::letter(c as u8)),
        })
        .collect();

    Move {
        tiles_data,
        pos,
        direction,
        score,
    }
}
