use std::fmt;
use std::marker::ConstParamTy_;

use crate::circuit::label::Label;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Source<Name: ConstParamTy_> {
    Primary(PrimarySource),
    Internal(SourceTerminal<Name>),
}

impl <Name: ConstParamTy_ + fmt::Display> fmt::Display for Source<Name> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Primary(primary_source) => primary_source.to_string(),
            Self::Internal(terminal) => terminal.to_string(),
        };
        write!(f, "{text}")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum PrimarySource {
    PpuClockFirstHalf,
    PpuClockSecondHalf,
    Read2007,
    Write2007,
}

impl fmt::Display for PrimarySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct SourceTerminal<Name: ConstParamTy_> {
    component_name: Name,
    label: Label,
}

impl <Name: ConstParamTy_ + fmt::Debug + fmt::Display> SourceTerminal<Name> {
    pub fn new(component_name: Name, label: Label) -> Self {
        Self { component_name, label }
    }

    pub fn from_raw(component_name: Name, label: char) -> Self {
        Self {
            component_name,
            label: Label::new(label),
        }
    }
}

impl <Name: ConstParamTy_ + fmt::Display> fmt::Display for SourceTerminal<Name> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{:?}", self.component_name, self.label)
    }
}