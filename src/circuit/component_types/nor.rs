use std::collections::{BTreeMap, BTreeSet};

use crate::circuit::component_type::ComponentType;
use crate::circuit::label::{LABEL_COUNT, Label};
use crate::circuit::signal_state::SignalState;

#[derive(Clone, Debug)]
pub struct Nor;

impl ComponentType for Nor {
    fn calculate_output_signal_states(&mut self, inputs: &[SignalState; LABEL_COUNT]) -> BTreeMap<Label, SignalState> {
        use SignalState::*;
        let q = match (&inputs[Label::A as usize], &inputs[Label::B as usize]) {
            (Resolved(false), Resolved(false)) => Resolved(true),
            (Resolved(true) , _              ) => Resolved(false),
            (_              , Resolved(true) ) => Resolved(false),
            (Unknown        , _              ) => Unknown,
            (_              , Unknown        ) => Unknown,
        };

        BTreeMap::from([(Label::Q, q)])
    }

    fn input_labels(&self) -> BTreeSet<Label> {
        ['A', 'B'].into_iter().map(Label::new).collect()
    }

    fn output_labels(&self) -> BTreeSet<Label> {
        ['Q'].into_iter().map(Label::new).collect()
    }
}
