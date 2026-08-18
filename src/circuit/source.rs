use std::fmt;

use crate::circuit::component::ComponentName;
use crate::circuit::label::Label;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Source {
    Primary(PrimarySource),
    Internal(SourceTerminal),
}

impl fmt::Display for Source {
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
pub struct SourceTerminal {
    component_name: ComponentName,
    label: Label,
}

impl SourceTerminal {
    pub fn new(component_name: ComponentName, label: Label) -> Self {
        Self { component_name, label }
    }

    pub fn from_raw(component_name: &'static str, label: char) -> Self {
        Self {
            component_name: ComponentName::new(component_name),
            label: Label::new(label),
        }
    }
}

impl fmt::Display for SourceTerminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{:?}", self.component_name, self.label)
    }
}