use std::marker::ConstParamTy_;

use crate::circuit::sink::{Sink, SinkTerminal};
use crate::circuit::source::Source;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Signal<Name: ConstParamTy_> {
    pub source: Source<Name>,
    pub sinks: Vec<Sink<Name>>,
}

impl <Name: ConstParamTy_ + PartialEq + Eq + PartialOrd + Ord + Clone> Signal<Name> {
    pub fn sink_terminals(&self) -> Vec<SinkTerminal<Name>> {
        self.sinks.iter()
            .filter_map(|sink| {
                match sink {
                    Sink::Primary(_) => None,
                    Sink::Internal(terminal) => Some(terminal.clone()),
                }
            })
            .collect()
    }
}
