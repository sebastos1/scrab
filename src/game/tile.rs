#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Letter(u8),
    Blank(Option<u8>),
}

impl Tile {
    pub fn to_char(self) -> char {
        match self {
            Tile::Letter(letter) => letter as char,
            Tile::Blank(Some(letter)) => letter as char,
            Tile::Blank(None) => '*',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        let c = c.to_ascii_uppercase();
        if c == '*' {
            Some(Tile::Blank(None))
        } else if ('A'..='Z').contains(&c) {
            Some(Tile::Letter(c as u8))
        } else {
            None
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            Tile::Letter(b) => b,
            Tile::Blank(Some(b)) => b,
            Tile::Blank(None) => b'*',
        }
    }

    pub fn points(self) -> u8 {
        match self {
            Tile::Letter(letter) => match letter {
                b'A' | b'E' | b'I' | b'L' | b'N' | b'O' | b'R' | b'S' | b'T' | b'U' => 1,
                b'D' | b'G' => 2,
                b'B' | b'C' | b'M' | b'P' => 3,
                b'F' | b'H' | b'V' | b'W' | b'Y' => 4,
                b'K' => 5,
                b'J' | b'X' => 8,
                b'Q' | b'Z' => 10,
                _ => 0,
            },
            Tile::Blank(_) => 0,
        }
    }
}
