use std::fmt;
use std::marker::ConstParamTy_;

use crate::circuit::label::{LABEL_COUNT, Label};
use crate::circuit::signal_state::SignalState;
use crate::circuit::component_type::ComponentType;

#[derive(Clone)]
pub struct Component<Name: ConstParamTy_> {
    name: Name,
    component_type: Box<dyn ComponentType + Send + Sync>,
    input_signal_states: [SignalState; LABEL_COUNT],
    output_signal_states: [SignalState; LABEL_COUNT],
}

impl <Name: ConstParamTy_ + Clone + Copy + ToIndex> Component<Name> {
    pub fn new(name: Name, component_type: Box<dyn ComponentType + Send + Sync>) -> Self {
        Self {
            name,
            component_type,
            input_signal_states: [SignalState::Unknown; LABEL_COUNT],
            output_signal_states: [SignalState::Unknown; LABEL_COUNT],
        }
    }

    pub fn name(&self) -> Name {
        self.name
    }

    pub fn source_labels(&self) -> &[Label] {
        self.component_type.source_labels()
    }

    #[allow(dead_code)]
    pub fn sink_labels(&self) -> &[Label] {
        self.component_type.sink_labels()
    }

    pub fn output_signal_state(&self, label: Label) -> SignalState {
        self.output_signal_states[label as usize]
    }

    pub fn set_input(&mut self, label: Label, signal_state: SignalState) {
        self.input_signal_states[label as usize] = signal_state;
    }

    pub fn calculate_output_signal_states(&mut self) {
        self.component_type.calculate_output_signal_states(&self.input_signal_states, &mut self.output_signal_states);
    }
}

pub trait ToIndex {
    fn to_index(&self) -> usize;
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ComponentName(&'static str);

#[allow(dead_code)]
impl ComponentName {
    pub const fn new(raw: &'static str) -> Self {
        assert!(!raw.is_empty());
        let bytes = raw.as_bytes();
        assert!(bytes[0].is_ascii_alphabetic(), "The first character must be a letter.");
        let mut i = 1;
        while i < bytes.len() {
            assert!(bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_', "All characters must be letters or numbers.");
            i += 1;
        }

        Self(raw)
    }
}

impl fmt::Display for ComponentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
