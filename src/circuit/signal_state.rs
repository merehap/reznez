use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SignalState {
    Unknown,
    Resolved(bool),
}

impl SignalState {
    pub fn negate(self) -> Self {
        use SignalState::*;
        match self {
            Unknown => Unknown,
            Resolved(value) => Resolved(!value),
        }
    }
}

impl fmt::Display for SignalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match *self {
            Self::Unknown => 'X',
            Self::Resolved(false) => 'F',
            Self::Resolved(true) => 'T',
        };

        write!(f, "{c}")
    }
}
