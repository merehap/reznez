use std::marker::ConstParamTy;
use std::ops::BitOrAssign;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, ConstParamTy)]
pub enum SignalLevel {
    #[default]
    High,
    Low,
}

impl BitOrAssign for SignalLevel {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = if *self == Self::Low || rhs == Self::Low {
            Self::Low
        } else {
            Self::High
        }
    }
}