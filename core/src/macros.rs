#[macro_export]
macro_rules! concat_u16 {
    ($hi:expr,$lo:expr) => {
        (($hi as u16) << 8) | ($lo as u16)
    };
}
