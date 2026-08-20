use std::collections::{BTreeMap, BTreeSet};

use crate::circuit::component_type::ComponentType;
use crate::circuit::label::{LABEL_COUNT, Label};
use crate::circuit::signal_state::SignalState;

#[derive(Clone, Debug)]
pub struct SRLatch {
    state: bool,
}

impl SRLatch {
    pub fn new(initial_state: bool) -> Self {
        Self { state: initial_state }
    }
}

impl ComponentType for SRLatch {
    fn calculate_output_signal_states(&mut self, inputs: &[SignalState; LABEL_COUNT]) -> BTreeMap<Label, SignalState> {
        let mut illegal_state = false;

        use SignalState::*;
        let new_state = match (&inputs[Label::S as usize], &inputs[Label::R as usize], self.state) {
            (Resolved(true) , Resolved(true) , _              ) => {
                log::info!("Simultaneous set and reset of an SR latch! Both outputs set to false.");
                illegal_state = true;
                // Pick an arbitrary state since there's no deterministic one.
                Resolved(true)
            }

            // Normal, intended inputs.
            (Resolved(true) , _              , _              ) => Resolved(false), // Successful set.
            (_              , Resolved(true) , _              ) => Resolved(true),  // Successful reset.
            (Resolved(false), Resolved(false), _              ) => Resolved(self.state),

            // In these cases, the unknown input does not need to be known in order to decide to maintain the current state.
            (Resolved(false), Unknown        , false) => Resolved(self.state),
            (Unknown        , Resolved(false), true ) => Resolved(self.state),

            // In these cases, the unknown input needed to be known to properly determine the output.
            (Resolved(false), Unknown        , _              ) => Unknown,
            (Unknown        , Resolved(false), _              ) => Unknown,
            (Unknown        , Unknown        , _              ) => Unknown,
        };

        if let Resolved(state) = new_state {
            self.state = state;
        }

        if illegal_state {
            BTreeMap::from([
                (Label::Q, Resolved(false)),
                (Label::N, Resolved(false)),
            ])
        } else {
            BTreeMap::from([
                (Label::Q, new_state),
                (Label::N, new_state.negate()),
            ])
        }
    }

    fn input_labels(&self) -> BTreeSet<Label> {
        ['S', 'R'].into_iter().map(Label::new).collect()
    }

    fn output_labels(&self) -> BTreeSet<Label> {
        ['Q', 'N'].into_iter().map(Label::new).collect()
    }
}