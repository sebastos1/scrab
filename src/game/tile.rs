#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tile(pub u8);

impl Tile {
    const EMPTY: u8 = 0;
    const BLANK_BIT: u8 = 0x80; // 10000000
    const LETTER_MASK: u8 = 0x1F; // 00011111

    pub fn empty() -> Self {
        Self(Self::EMPTY)
    }

    pub fn letter(letter: u8) -> Self {
        Self(letter - b'A' + 1)
    }

    pub fn blank(letter: Option<u8>) -> Self {
        match letter {
            Some(l) => Self((l - b'A' + 1) | Self::BLANK_BIT),
            None => Self(Self::BLANK_BIT),
        }
    }

    pub fn is_empty(self) -> bool {
        self.0 == Self::EMPTY
    }

    pub fn is_blank(self) -> bool {
        (self.0 & Self::BLANK_BIT) != 0
    }

    pub fn is_some(self) -> bool {
        !self.is_empty()
    }

    pub fn byte(self) -> u8 {
        if self.is_empty() {
            0
        } else if self.is_blank() {
            let letter_bits = self.0 & Self::LETTER_MASK;
            if letter_bits == 0 { b'*' } else { letter_bits + b'A' - 1 }
        } else {
            (self.0 & Self::LETTER_MASK) + b'A' - 1
        }
    }

    pub fn to_char(self) -> char {
        self.byte() as char
    }

    pub fn points(self) -> u8 {
        if self.is_empty() || self.is_blank() {
            0
        } else {
            let letter = self.byte();
            match letter {
                b'A' | b'E' | b'I' | b'L' | b'N' | b'O' | b'R' | b'S' | b'T' | b'U' => 1,
                b'D' | b'G' => 2,
                b'B' | b'C' | b'M' | b'P' => 3,
                b'F' | b'H' | b'V' | b'W' | b'Y' => 4,
                b'K' => 5,
                b'J' | b'X' => 8,
                b'Q' | b'Z' => 10,
                _ => 0,
            }
        }
    }

    pub fn to_index(&self) -> u8 {
        if self.is_empty() {
            return 0;
        }
        if self.is_blank() { 26 } else { self.0 & Self::LETTER_MASK }
    }
}
