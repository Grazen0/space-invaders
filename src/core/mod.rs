pub mod cpu;
pub mod emulator;
pub mod error;
pub mod macros;
pub mod memory;

#[derive(Debug, Clone)]
pub enum Button {
    P1Start,
    P2Start,
    P1Shoot,
    P2Shoot,
    P1Left,
    P2Left,
    P1Right,
    P2Right,
    Tilt,
    Coin,
}

impl Button {
    fn mask(&self) -> u8 {
        match self {
            Self::Coin => 0b0000_0001,
            Self::P2Start => 0b0000_0010,
            Self::P1Start => 0b0000_0100,
            Self::P1Shoot => 0b0001_0000,
            Self::P1Left => 0b0010_0000,
            Self::P1Right => 0b0100_0000,
            Self::Tilt => 0b0000_0100,
            Self::P2Shoot => 0b0001_0000,
            Self::P2Left => 0b0010_0000,
            Self::P2Right => 0b0100_0000,
        }
    }
}

pub fn even_parity(mut n: u8) -> bool {
    let mut parity = true;

    while n != 0 {
        parity = !parity;
        n &= n - 1;
    }

    parity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_even_parity() {
        assert_eq!(even_parity(0b1101), false);
        assert_eq!(even_parity(0b0101_1101), false);
        assert_eq!(even_parity(0b1001), true);
        assert_eq!(even_parity(0b1100_1111), true);
    }
}
