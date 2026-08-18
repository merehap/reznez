use std::fmt;

use crate::circuit::component::ComponentName;
use crate::circuit::label::Label;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Sink {
    Primary(PrimarySink),
    Internal(SinkTerminal),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum PrimarySink {
    Ale,
    Read,
    Write,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct SinkTerminal {
    component_name: ComponentName,
    label: Label,
}

impl SinkTerminal {
    pub fn from_raw(component_name: &'static str, label: char) -> Self {
        Self {
            component_name: ComponentName::new(component_name),
            label: Label::new(label),
        }
    }

    pub fn component_name(&self) -> ComponentName {
        self.component_name
    }

    pub fn label(&self) -> Label {
        self.label
    }
}

impl fmt::Display for SinkTerminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sink: {}-{:?}", self.component_name, self.label)
    }
}