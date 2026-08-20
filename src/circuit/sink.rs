use std::fmt;
use std::marker::ConstParamTy_;

use crate::circuit::label::Label;

pub const PRIMARY_SINK_COUNT: usize = std::mem::variant_count::<PrimarySink>();

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Sink<Name: ConstParamTy_> {
    Primary(PrimarySink),
    Internal(SinkTerminal<Name>),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum PrimarySink {
    Ale,
    Read,
    Write,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct SinkTerminal<Name: ConstParamTy_> {
    component_name: Name,
    label: Label,
}

impl <Name: ConstParamTy_ + Clone + Copy> SinkTerminal<Name> {
    pub fn from_raw(component_name: Name, label: char) -> Self {
        Self {
            component_name,
            label: Label::new(label),
        }
    }

    pub fn component_name(&self) -> Name {
        self.component_name
    }

    pub fn label(&self) -> Label {
        self.label
    }
}

impl <Name: ConstParamTy_ + fmt::Debug + fmt::Display> fmt::Display for SinkTerminal<Name> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sink: {}-{:?}", self.component_name, self.label)
    }
}