use std::fmt;
use std::marker::ConstParamTy_;

use crate::circuit::label::{LABEL_COUNT, Label};
use crate::circuit::node::{ComponentTypeName, Factor};
use crate::circuit::signal::Signal;
use crate::circuit::signal_state::SignalState;
use crate::circuit::component_type::ComponentType;
use crate::circuit::sink::{Sink, SinkTerminal};
use crate::circuit::source::{Source, SourceTerminal};

#[derive(Clone)]
pub struct Component<Name: ConstParamTy_> {
    name: Name,
    component_type: Box<dyn ComponentType + Send + Sync>,
    input_signal_states: [SignalState; LABEL_COUNT],
    output_signal_states: [SignalState; LABEL_COUNT],
}

impl <Name: ConstParamTy_ + Clone + Copy + fmt::Debug + ToIndex> Component<Name> {
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

    pub fn factors(&self, signals: &[Signal<Name>]) -> Vec<(Factor<Name>, Source<Name>)> {
        let find_signal = |sink_terminal: SinkTerminal<Name>| -> Signal<Name> {
            for signal in signals {
                for sink in &signal.sinks {
                    if *sink == Sink::Internal(sink_terminal) {
                        return signal.clone();
                    }
                }
            }

            panic!();
        };

        match self.component_type.component_type_name() {
            ComponentTypeName::Nor => {
                let a = find_signal(SinkTerminal::new(self.name, Label::A));
                let b = find_signal(SinkTerminal::new(self.name, Label::B));
                vec![
                    (
                        Factor::Nor { a, b },
                        Source::Internal(SourceTerminal { component_name: self.name, label: Label::Q })
                    ),
                ]
            }
            ComponentTypeName::DLatch(default_state) => {
                let e = find_signal(SinkTerminal::new(self.name, Label::E));
                let d = find_signal(SinkTerminal::new(self.name, Label::D));
                vec![
                    (
                        Factor::DLatchQ { e: e.clone(), d: d.clone(), default_state },
                        Source::Internal(SourceTerminal { component_name: self.name, label: Label::Q }),
                    ),
                    (
                        Factor::DLatchN { e, d, default_state: !default_state },
                        Source::Internal(SourceTerminal { component_name: self.name, label: Label::N }),
                    ),
                ]
            }
            ComponentTypeName::SRLatch(default_state) => {
                let s = find_signal(SinkTerminal::new(self.name, Label::S));
                let r = find_signal(SinkTerminal::new(self.name, Label::R));
                vec![
                    (
                        Factor::SRLatchQ { s: s.clone(), r: r.clone(), default_state },
                        Source::Internal(SourceTerminal { component_name: self.name, label: Label::Q }),
                    ),
                    (
                        Factor::SRLatchN { s, r, default_state: !default_state },
                        Source::Internal(SourceTerminal { component_name: self.name, label: Label::N }),
                    ),
                ]
            }
        }
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
