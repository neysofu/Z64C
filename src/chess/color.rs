use enum_map_derive::Enum;
use std::ops;
use strum_macros::EnumIter;

#[derive(Copy, Clone, Debug, Enum, EnumIter, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn set_ascii_case(self, c: char) -> char {
        match self {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c.to_ascii_lowercase(),
        }
    }

    pub fn from_char_case(c: char) -> Self {
        if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        }
    }
}

impl ops::Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl From<char> for Color {
    fn from(c: char) -> Self {
        match c.to_ascii_lowercase() {
            'w' => Color::White,
            'b' => Color::Black,
            _ => panic!(),
        }
    }
}

impl From<Color> for char {
    fn from(color: Color) -> Self {
        match color {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn color_from_char() {
        assert_eq!(Color::from('w'), Color::White);
        assert_eq!(Color::from('B'), Color::Black);
    }

    #[test]
    fn char_from_color() {
        assert_eq!(char::from(Color::White), 'w');
        assert_eq!(char::from(Color::Black), 'b');
    }

    #[test]
    fn not_black_is_white() {
        assert_eq!(!Color::Black, Color::White);
    }
}