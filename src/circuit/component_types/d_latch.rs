use std::collections::{BTreeMap, BTreeSet};

use crate::circuit::component_type::ComponentType;
use crate::circuit::label::Label;
use crate::circuit::signal_state::SignalState;

#[derive(Clone, Debug)]
pub struct DLatch {
    state: bool,
}

impl DLatch {
    pub fn new(initial_state: bool) -> Self {
        Self { state: initial_state }
    }
}

impl ComponentType for DLatch {
    fn calculate_output_signal_states(&mut self, inputs: BTreeMap<Label, SignalState>) -> BTreeMap<Label, SignalState> {
        use SignalState::*;
        let new_state = match (&inputs[&Label::E], &inputs[&Label::D]) {
            (Resolved(true) , d      ) => *d,
            (Resolved(false), _      ) => Resolved(self.state),
            (Unknown        , _      ) => panic!("Don't know what to do if DLatch is neither enabled nor disabled."),
        };

        if let SignalState::Resolved(state) = new_state {
            self.state = state;
        }

        BTreeMap::from([
            (Label::Q, new_state),
            (Label::N, new_state.negate()),
        ])
    }

    fn input_labels(&self) -> BTreeSet<Label> {
        ['E', 'D'].into_iter().map(Label::new).collect()
    }

    fn output_labels(&self) -> BTreeSet<Label> {
        ['Q', 'N'].into_iter().map(Label::new).collect()
    }
}