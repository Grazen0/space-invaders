mod cpu;
mod memory;
mod error;
mod macros;

pub use error::{Result, Error};
pub use cpu::CPU;
pub use memory::Memory;

pub fn even_parity(mut n: u8) -> bool {
    let mut parity = true;

    while n != 0 {
        parity = !parity;
        n &= n - 1;
    }

    parity
}

#[cfg(test)]
mod test {
    #[test]
    fn test_even_parity() {
        assert_eq!(super::even_parity(0b1101), false);
        assert_eq!(super::even_parity(0b0101_1101), false);
        assert_eq!(super::even_parity(0b1001), true);
        assert_eq!(super::even_parity(0b1100_1111), true);
    }

    #[test]
    fn test_concat_u16() {
        assert_eq!(super::concat_u16!(0xF6, 0x78), 0xF678);
        assert_eq!(super::concat_u16!(0xD1, 0x4A), 0xD14A);
        assert_eq!(super::concat_u16!(0x00, 0x20), 0x0020);
    }
}
