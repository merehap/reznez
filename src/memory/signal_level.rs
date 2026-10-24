use std::marker::ConstParamTy;
use std::ops::{BitAnd, BitOr, BitOrAssign, Deref, Not};

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, ConstParamTy)]
pub enum SignalLevel {
    Low = 0,
    #[default]
    High = 1,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, ConstParamTy)]
pub enum ActiveHigh {
    #[default]
    Inactive = 0,
    Active = 1,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, ConstParamTy)]
pub enum ActiveLow {
    Active = 0,
    #[default]
    Inactive = 1,
}

impl From<ActiveHigh> for SignalLevel {
    fn from(value: ActiveHigh) -> Self {
        match value {
            ActiveHigh::Inactive => SignalLevel::Low,
            ActiveHigh::Active => SignalLevel::High,
        }
    }
}

impl From<ActiveLow> for SignalLevel {
    fn from(value: ActiveLow) -> Self {
        match value {
            ActiveLow::Inactive => SignalLevel::High,
            ActiveLow::Active => SignalLevel::Low,
        }
    }
}

impl Deref for ActiveHigh {
    type Target = SignalLevel;

    fn deref(&self) -> &Self::Target {
        match *self {
            ActiveHigh::Inactive => &SignalLevel::Low,
            ActiveHigh::Active => &SignalLevel::High,
        }
    }
}

impl Deref for ActiveLow {
    type Target = SignalLevel;

    fn deref(&self) -> &Self::Target {
        match *self {
            ActiveLow::Inactive => &SignalLevel::High,
            ActiveLow::Active => &SignalLevel::Low,
        }
    }
}

impl From<SignalLevel> for ActiveHigh {
    fn from(value: SignalLevel) -> Self {
        match value {
            SignalLevel::Low => ActiveHigh::Inactive,
            SignalLevel::High => ActiveHigh::Active,
        }
    }
}

impl From<ActiveLow> for ActiveHigh {
    fn from(value: ActiveLow) -> Self {
        (*value).into()
    }
}

impl From<SignalLevel> for ActiveLow {
    fn from(value: SignalLevel) -> Self {
        match value {
            SignalLevel::Low => ActiveLow::Active,
            SignalLevel::High => ActiveLow::Inactive,
        }
    }
}

impl From<ActiveHigh> for ActiveLow {
    fn from(value: ActiveHigh) -> Self {
        (*value).into()
    }
}

impl SignalLevel {
    pub fn is_high(self) -> bool {
        self == Self::High
    }

    pub fn is_low(self) -> bool {
        self == Self::Low
    }

    pub fn negate(self) -> SignalLevel {
        !self
    }

    pub fn nor(self, other: impl Into<Self>) -> SignalLevel {
        !(self | other.into())
    }

    pub fn nand(self, other: impl Into<Self>) -> SignalLevel {
        !(self & other.into())
    }
}

impl ActiveHigh {
    pub fn is_inactive(self) -> bool {
        self == Self::Inactive
    }

    pub fn is_active(self) -> bool {
        self == Self::Active
    }

    pub fn and(self, other: impl Into<Self>) -> Self {
        (*self & *other.into()).into()
    }

    pub fn or(self, other: impl Into<Self>) -> Self {
        (*self | *other.into()).into()
    }
}

impl ActiveLow {
    pub fn is_inactive(self) -> bool {
        self == Self::Inactive
    }

    pub fn is_active(self) -> bool {
        self == Self::Active
    }

    pub fn and(self, other: impl Into<Self>) -> Self {
        (*self & *other.into()).into()
    }
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

impl Not for ActiveHigh {
    type Output = ActiveHigh;

    fn not(self) -> Self::Output {
        match self {
            Self::Inactive => Self::Active,
            Self::Active => Self::Inactive,
        }
    }
}

impl Not for ActiveLow {
    type Output = ActiveLow;

    fn not(self) -> Self::Output {
        match self {
            Self::Inactive => Self::Active,
            Self::Active => Self::Inactive,
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