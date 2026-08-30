use std::collections::BTreeMap;

use crate::circuit::component_type::{CofactorId, ComponentType};
use crate::circuit::gate::LabeledGate;
use crate::circuit::gate::UnaryGateType::*;
use crate::circuit::label::{LABEL_COUNT, Label};
use crate::circuit::label::Label::*;
use crate::circuit::node::ComponentTypeName;
use crate::circuit::signal_state::SignalState;

const SINK_LABELS: &'static [Label] = &[Label::S, Label::R];
const SOURCE_LABELS: &'static [Label] = &[Label::Q, Label::N];

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
    fn calculate_output_signal_states(&mut self, inputs: &[SignalState; LABEL_COUNT], outputs: &mut [SignalState; LABEL_COUNT]) {
        let mut illegal_state = false;

        use SignalState::*;
        let new_state = match (&inputs[S as usize], &inputs[R as usize], self.state) {
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
            outputs[Q as usize] = Resolved(false);
            outputs[N as usize] = Resolved(false);
        } else {
            outputs[Q as usize] = new_state;
            outputs[N as usize] = new_state.negate();
        }
    }

    fn cofactors(&self) -> BTreeMap<CofactorId, LabeledGate> {
        BTreeMap::from([
            (CofactorId::new((S, false), Q), LabeledGate::Unary(StateAndNot, R)),
            (CofactorId::new((S, false), N), LabeledGate::Unary(NotStateOr, R)),
            (CofactorId::new((S, true ), Q), LabeledGate::Unary(Not, R)),
            (CofactorId::new((S, true ), N), LabeledGate::Constant(false)),
            (CofactorId::new((R, false), Q), LabeledGate::Unary(StateOr, S)),
            (CofactorId::new((R, false), N), LabeledGate::Unary(StateNor, S)),
            (CofactorId::new((R, true ), Q), LabeledGate::Constant(false)),
            (CofactorId::new((R, true ), N), LabeledGate::Unary(Not, S)),
        ])
    }

    fn component_type_name(&self) -> ComponentTypeName {
        ComponentTypeName::SRLatch
    }

    fn sink_labels(&self) -> &[Label] {
        SINK_LABELS
    }

    fn source_labels(&self) -> &[Label] {
        SOURCE_LABELS
    }
}