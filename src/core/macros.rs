#[macro_export]
macro_rules! concat_u16 {
    ($hi:expr,$lo:expr) => {
        (($hi as u16) << 8) | ($lo as u16)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_concat_u16() {
        assert_eq!(concat_u16!(0xF6, 0x78), 0xF678);
        assert_eq!(concat_u16!(0xD1, 0x4A), 0xD14A);
        assert_eq!(concat_u16!(0x00, 0x20), 0x0020);
    }
}
