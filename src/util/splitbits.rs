#[macro_export]
macro_rules! splitbits {
    ($value:ident, $mask:expr) => {
        splitbits_proc!($mask)($value)
    }
}
