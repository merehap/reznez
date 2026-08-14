use std::collections::BTreeMap;
use std::fmt;

use crate::circuit::label::Label;
use crate::circuit::signal_state::SignalState;
use crate::circuit::component_type::ComponentType;

#[derive(Clone)]
pub struct Component {
    name: ComponentName,
    component_type: Box<dyn ComponentType + Send + Sync>,
}

impl Component {
    pub fn new(name: &'static str, component_type: Box<dyn ComponentType + Send + Sync>) -> Self {
        Self {
            name: ComponentName::new(name),
            component_type,
        }
    }

    pub fn calculate_output_signal_states(&mut self, input_signal_states: BTreeMap<Label, SignalState>) -> BTreeMap<Label, SignalState> {
        self.component_type.calculate_output_signal_states(input_signal_states)
    }

    pub fn name(&self) -> ComponentName {
        self.name
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ComponentName(&'static str);

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
