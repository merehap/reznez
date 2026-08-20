use crate::circuit::component_type::ComponentType;
use crate::circuit::label::{LABEL_COUNT, Label};
use crate::circuit::signal_state::SignalState;

const INPUT_LABELS: &'static [Label] = &[Label::new('A'), Label::new('B')];
const OUTPUT_LABELS: &'static [Label] = &[Label::new('Q')];

#[derive(Clone, Debug)]
pub struct Nor;

impl ComponentType for Nor {
    fn calculate_output_signal_states(&mut self, inputs: &[SignalState; LABEL_COUNT], outputs: &mut [SignalState; LABEL_COUNT]) {
        use SignalState::*;
        let q = match (&inputs[Label::A as usize], &inputs[Label::B as usize]) {
            (Resolved(false), Resolved(false)) => Resolved(true),
            (Resolved(true) , _              ) => Resolved(false),
            (_              , Resolved(true) ) => Resolved(false),
            (Unknown        , _              ) => Unknown,
            (_              , Unknown        ) => Unknown,
        };


        outputs[Label::Q as usize] = q;
    }

    fn sink_labels(&self) -> &[Label] {
        INPUT_LABELS
    }

    fn source_labels(&self) -> &[Label] {
        OUTPUT_LABELS
    }
}
