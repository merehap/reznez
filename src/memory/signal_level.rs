use std::marker::ConstParamTy;
use std::ops::{Not, BitAnd, BitOr, BitOrAssign};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, ConstParamTy)]
pub enum SignalLevel {
    #[default]
    High,
    Low,
}

impl SignalLevel {
    pub fn is_high(self) -> bool {
        self == Self::High
    }

    pub fn is_low(self) -> bool {
        self == Self::Low
    }
}

pub fn nor(a: SignalLevel, b: SignalLevel) -> SignalLevel {
    !(a | b)
}

pub fn nor3(a: SignalLevel, b: SignalLevel, c: SignalLevel) -> SignalLevel {
    !(a | b | c)
}

pub fn nand(a: SignalLevel, b: SignalLevel) -> SignalLevel {
    !(a & b)
}

impl Not for SignalLevel {
    type Output = SignalLevel;

    fn not(self) -> Self::Output {
        match self {
            Self::High => Self::Low,
            Self::Low => Self::High,
        }
    }
}

impl BitAnd for SignalLevel {
    type Output = SignalLevel;

    fn bitand(self, rhs: Self) -> Self::Output {
        if self.is_high() && rhs.is_high() {
            Self::High
        } else {
            Self::Low
        }
    }
}

impl BitOr for SignalLevel {
    type Output = SignalLevel;

    fn bitor(self, rhs: Self) -> Self::Output {
        if self.is_high() || rhs.is_high() {
            Self::High
        } else {
            Self::Low
        }
    }
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