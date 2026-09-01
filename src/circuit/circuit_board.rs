use std::collections::{BTreeSet, BTreeMap};
use std::fmt;
use std::marker::{ConstParamTy, ConstParamTy_};
use std::sync::LazyLock;

use crate::circuit::component::{Component, ToIndex};
use crate::circuit::component_types::d_latch::DLatch;
use crate::circuit::component_types::nor::Nor;
use crate::circuit::component_types::sr_latch::SRLatch;
use crate::circuit::label::{LABEL_COUNT, Label};
use crate::circuit::node::{FactorGraph, NodeGraph};
use crate::circuit::signal::Signal;
use crate::circuit::signal_state::SignalState;
use crate::circuit::sink::{PRIMARY_SINK_COUNT, PrimarySink, Sink, SinkTerminal};
use crate::circuit::source::{PrimarySource, Source, SourceTerminal};
use crate::memory::ppu::ppu_pinout::PpuPinout;

pub static READ_2007_COMPONENTS: LazyLock<ComponentGraph<Read2007ComponentName, {READ_2007_COMPONENT_NAME_COUNT}, {LABEL_COUNT}>> = LazyLock::new(|| {
    use Read2007ComponentName::*;
    let mut graph = ComponentGraphBuilder::new()
        .component(Component::new(DLatch0, Box::new(DLatch::new(false))))
        .component(Component::new(DLatch1, Box::new(DLatch::new(true))))
        .component(Component::new(DLatch2, Box::new(DLatch::new(false))))
        .component(Component::new(DLatch3, Box::new(DLatch::new(true))))
        .component(Component::new(DLatch4, Box::new(DLatch::new(false))))
        // This latch storing "true" at startup ensures that calculation doesn't start until a Read2007 occurs.
        .component(Component::new(SRLatch0, Box::new(SRLatch::new(true))))
        .component(Component::new(Nor0, Box::new(Nor)))
        .component(Component::new(NorAle, Box::new(Nor)))
        .component(Component::new(NorRead, Box::new(Nor)))
        .signal(Signal {
            source: Source::Primary(PrimarySource::PpuClockFirstHalf),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch0, 'E')),
                Sink::Internal(SinkTerminal::from_raw(DLatch2, 'E')),
                Sink::Internal(SinkTerminal::from_raw(DLatch4, 'E')),
            ],
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::PpuClockSecondHalf),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch1, 'E')),
                Sink::Internal(SinkTerminal::from_raw(DLatch3, 'E')),
            ],
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::Read2007),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(Nor0, 'A'))
            ],
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::Read2007),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(SRLatch0, 'S'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(SRLatch0, 'Q')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(Nor0, 'B'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(Nor0, 'Q')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch0, 'D'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch0, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch1, 'D'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch1, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch2, 'D'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch2, 'Q')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(NorRead, 'B')),
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch2, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch3, 'D')),
                Sink::Internal(SinkTerminal::from_raw(NorAle, 'A')),
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch3, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch4, 'D'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch3, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(SRLatch0, 'R'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch4, 'Q')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(NorAle, 'B')),
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch4, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(NorRead, 'A')),
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(NorAle, 'Q')),
            sinks: vec![
                Sink::Primary(PrimarySink::Ale)
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(NorRead, 'Q')),
            sinks: vec![
                Sink::Primary(PrimarySink::Read)
            ],
        })
        .build();

    graph.compute_primary_sinks(&STARTUP_PRIMARY_SOURCE_STATES);

    graph
});

const READ_2007_COMPONENT_NAME_COUNT: usize = std::mem::variant_count::<Read2007ComponentName>();

#[derive(ConstParamTy, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Read2007ComponentName {
    DLatch0,
    DLatch1,
    DLatch2,
    DLatch3,
    DLatch4,
    SRLatch0,
    Nor0,
    NorAle,
    NorRead,
}

impl fmt::Display for Read2007ComponentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ToIndex for Read2007ComponentName {
    fn to_index(&self) -> usize {
        *self as usize
    }
}

static WRITE_2007_COMPONENTS: LazyLock<ComponentGraph<Write2007ComponentName, {WRITE_2007_COMPONENT_NAME_COUNT}, {LABEL_COUNT}>> = LazyLock::new(|| {
    use Write2007ComponentName::*;
    let mut graph = ComponentGraphBuilder::new()
        .component(Component::new(DLatch0, Box::new(DLatch::new(false))))
        .component(Component::new(DLatch1, Box::new(DLatch::new(true))))
        .component(Component::new(DLatch2, Box::new(DLatch::new(false))))
        .component(Component::new(DLatch3, Box::new(DLatch::new(true))))
        .component(Component::new(DLatch4, Box::new(DLatch::new(false))))
        // This latch storing "true" at startup ensures that calculation doesn't start until a Write2007 occurs.
        .component(Component::new(SRLatch0, Box::new(SRLatch::new(true))))
        .component(Component::new(Nor0, Box::new(Nor)))
        .component(Component::new(NorAle, Box::new(Nor)))
        .component(Component::new(NorWrite, Box::new(Nor)))
        .signal(Signal {
            source: Source::Primary(PrimarySource::PpuClockFirstHalf),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch0, 'E')),
                Sink::Internal(SinkTerminal::from_raw(DLatch2, 'E')),
                Sink::Internal(SinkTerminal::from_raw(DLatch4, 'E')),
            ],
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::PpuClockSecondHalf),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch1, 'E')),
                Sink::Internal(SinkTerminal::from_raw(DLatch3, 'E')),
            ],
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::Write2007),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(Nor0, 'A'))
            ],
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::Write2007),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(SRLatch0, 'S'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(SRLatch0, 'Q')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(Nor0, 'B'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(Nor0, 'Q')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch0, 'D'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch0, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch1, 'D'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch1, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch2, 'D'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch1, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(NorWrite, 'B')),
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch2, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch3, 'D')),
                Sink::Internal(SinkTerminal::from_raw(NorAle, 'A')),
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch3, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(DLatch4, 'D'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch3, 'N')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(SRLatch0, 'R'))
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch4, 'Q')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(NorAle, 'B')),
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(DLatch3, 'Q')),
            sinks: vec![
                Sink::Internal(SinkTerminal::from_raw(NorWrite, 'A')),
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(NorAle, 'Q')),
            sinks: vec![
                Sink::Primary(PrimarySink::Ale)
            ],
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw(NorWrite, 'Q')),
            sinks: vec![
                Sink::Primary(PrimarySink::Write)
            ],
        })
        .build();

    graph.compute_primary_sinks(&STARTUP_PRIMARY_SOURCE_STATES);
    graph
});

const WRITE_2007_COMPONENT_NAME_COUNT: usize = std::mem::variant_count::<Write2007ComponentName>();

#[derive(ConstParamTy, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum Write2007ComponentName {
    DLatch0,
    DLatch1,
    DLatch2,
    DLatch3,
    DLatch4,
    SRLatch0,
    Nor0,
    NorAle,
    NorWrite,
}

impl fmt::Display for Write2007ComponentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ToIndex for Write2007ComponentName {
    fn to_index(&self) -> usize {
        *self as usize
    }
}

const STARTUP_PRIMARY_SOURCE_STATES: [bool; std::mem::variant_count::<PrimarySource>()] = [
    false, // PpuClockFirstHalf
    true,  // PpuClockSecondHalf
    false, // Read2007
    false, // Write2007
];

pub struct CircuitBoard {
    primary_source_states: [bool; std::mem::variant_count::<PrimarySource>()],
    read_2007_components: ComponentGraph<Read2007ComponentName, {std::mem::variant_count::<Read2007ComponentName>()}, {std::mem::variant_count::<Label>()}>,
    write_2007_components: ComponentGraph<Write2007ComponentName, {std::mem::variant_count::<Write2007ComponentName>()}, {std::mem::variant_count::<Label>()}>,
}

impl CircuitBoard {
    pub fn new() -> Self {
        let mut circuit_board = Self {
            primary_source_states: STARTUP_PRIMARY_SOURCE_STATES,
            read_2007_components: READ_2007_COMPONENTS.clone(),
            write_2007_components: WRITE_2007_COMPONENTS.clone(),
        };
        // This way the first PPU tick will start the first half of the PPU dot.
        circuit_board.set_source(PrimarySource::PpuClockSecondHalf, true);

        circuit_board
    }

    pub fn source(&self, input: PrimarySource) -> bool {
        self.primary_source_states[input as usize]
    }

    pub fn set_source(&mut self, primary_source: PrimarySource, value: bool) {
        self.primary_source_states[primary_source as usize] = value;
    }

    pub fn calculate_read_2007_outputs(&mut self, ppu_pinout: &mut PpuPinout) {
        self.read_2007_components.compute_primary_sinks(&self.primary_source_states);

        ppu_pinout.ppu_data_ale = self.read_2007_components.primary_sinks[PrimarySink::Ale as usize];
        ppu_pinout.ppu_data_read = self.read_2007_components.primary_sinks[PrimarySink::Read as usize];
    }

    pub fn is_read_2007_circuit_in_base_state(&self) -> bool {
        !self.source(PrimarySource::Read2007) && self.read_2007_components.signal_states_equal(&READ_2007_COMPONENTS)
    }

    pub fn calculate_write_2007_outputs(&mut self, ppu_pinout: &mut PpuPinout) {
        self.write_2007_components.compute_primary_sinks(&self.primary_source_states);

        ppu_pinout.ppu_data_ale = self.write_2007_components.primary_sinks[PrimarySink::Ale as usize];
        ppu_pinout.ppu_data_write = self.write_2007_components.primary_sinks[PrimarySink::Write as usize];
    }

    pub fn is_write_2007_circuit_in_base_state(&self) -> bool {
        !self.source(PrimarySource::Write2007) && self.write_2007_components.signal_states_equal(&WRITE_2007_COMPONENTS)
    }
}

#[derive(Clone)]
pub struct ComponentGraph<Name: ConstParamTy_, const NAME_COUNT: usize, const LABEL_COUNT: usize> {
    primary_sinks: [bool; PRIMARY_SINK_COUNT],
    component_sink_info: Box<[(Component<Name>, Vec<(Label, Signal<Name>)>)]>,
    signals_by_source_terminal: Box<[[Vec<Signal<Name>>; LABEL_COUNT]; NAME_COUNT]>,
    primary_signal_states: [bool; std::mem::variant_count::<PrimarySource>()],
    internal_signal_states: Box<[[SignalState; LABEL_COUNT]; NAME_COUNT]>,

    components: Vec<Component<Name>>,
    signals: Vec<Signal<Name>>,
}

impl <Name: ConstParamTy_ + PartialEq + Eq + PartialOrd + Ord + Clone + Copy + fmt::Debug + fmt::Display + ToIndex, const NAME_COUNT: usize, const LABEL_COUNT: usize> ComponentGraph<Name, NAME_COUNT, LABEL_COUNT> {
    pub fn to_node_graph(&self, primary_sources: &[PrimarySource]) -> NodeGraph<Name> {
        let mut factor_graph: FactorGraph<Name> = FactorGraph::new(self.signals.clone(), primary_sources);
        for component in &self.components {
            factor_graph.push_component(component);
        }

        NodeGraph::new(factor_graph)
    }
    /*
    pub fn to_netlist(&self, primary_signal_states: &[bool]) -> Netlist {
        let mut components_by_source_terminal: Vec<(SourceTerminal<Name>, Component<Name>, Option<IndexedGate>)> = self.components.iter()
            .flat_map(|component| {
                component.source_labels().iter().flat_map(|source_label| {
                    self.signals_by_source_terminal[component.name().to_index()][source_label.to_index()].iter()
                        .filter_map(|signal| {
                            match &signal.source {
                                Source::Primary(_) => None,
                                Source::Internal(terminal) => Some((terminal.clone(), component.clone(), None)),
                            }
                        })
                })
            })
            .collect();


        for i in 0..components_by_source_terminal.len() {
            let source_terminal = components_by_source_terminal[i].0.clone();
            let component = components_by_source_terminal[i].1.clone();

            let labeled_signals = self.component_sink_info.iter()
                .find(|(comp, _)| comp.name() == component.name())
                .unwrap()
                .1
                .clone();

            for (sink_label, signal) in labeled_signals {
                match signal.source {
                    Source::Primary(primary_source) => {
                        let signal_state = primary_signal_states[primary_source as usize];
                        let cofactor_id = CofactorId::new((sink_label, signal_state), source_terminal.label);
                        let sink_labeled_gate = component.cofactors()[&cofactor_id].clone();
                        components_by_source_terminal[i].2 = Some(sink_labeled_gate.to_indexed(i, |label| {
                            let sink_terminal = SinkTerminal::new(component.name(), label);
                            let Source::Internal(remaining_source_terminal) = self.signals_by_sink_terminal.get(&sink_terminal)
                                    .expect("terminal, not primary source").source.clone() else {
                                panic!("Expected terminal, not primary source");
                            };

                            for j in 0..components_by_source_terminal.len() {
                                let (s_term, _, _) = &components_by_source_terminal[j];
                                if remaining_source_terminal == *s_term {
                                    return j;
                                }
                            }

                            panic!();
                        }));
                    }
                    Source::Internal(_) => {}
                }
            }
        }

        for i in 0..components_by_source_terminal.len() {
            let source_terminal = components_by_source_terminal[i].0.clone();
            let component = components_by_source_terminal[i].1.clone();

            let labeled_signals = self.component_sink_info.iter()
                .find(|(comp, _)| comp.name() == component.name())
                .unwrap()
                .1
                .clone();

            for (sink_label, signal) in labeled_signals {
                match signal.source {
                    Source::Primary(_) => panic!("Primaries should have already been processed."),
                    Source::Internal(upstream_source_terminal) => {

                    }
                }
            }
        }

        todo!()
    }
    */

    pub fn signal_states_equal(&self, other: &ComponentGraph<Name, NAME_COUNT, LABEL_COUNT>) -> bool {
        self.internal_signal_states == other.internal_signal_states
    }

    pub fn compute_primary_sinks(&mut self, primary_sources: &[bool]) {
        for (_component, labeled_signals) in &self.component_sink_info {
            for (_, signal) in labeled_signals {
                if let Source::Primary(primary_source) = &signal.source {
                    let primary_source_value = primary_sources[*primary_source as usize];
                    self.primary_signal_states[*primary_source as usize] = primary_source_value;
                }
            }
        }

        // TODO: Find a proper way to terminate this rather than just iterating 10 times and hoping for the best.
        for _ in 0..2 {
            for (component, labeled_signals) in &mut self.component_sink_info {
                for (label, signal) in labeled_signals.iter() {
                    let signal_state = match &signal.source {
                        Source::Primary(primary_source) => SignalState::Resolved(self.primary_signal_states[*primary_source as usize]),
                        Source::Internal(terminal) => self.internal_signal_states[terminal.component_name.to_index()][terminal.label.to_index()],
                    };
                    component.set_input(*label, signal_state);
                }

                component.calculate_output_signal_states();
                for source_label in component.source_labels() {
                    let signal_state = component.output_signal_state(*source_label);
                    for signal in &self.signals_by_source_terminal[component.name().to_index()][source_label.to_index()] {
                        for sink in &signal.sinks {
                            match sink {
                                Sink::Primary(primary_sink) => {
                                    if let SignalState::Resolved(resolved_state) = signal_state {
                                        self.primary_sinks[*primary_sink as usize] = resolved_state;
                                    }
                                }
                                Sink::Internal(_) => {
                                    match &signal.source {
                                        Source::Primary(_) => panic!(),
                                        Source::Internal(terminal) => self.internal_signal_states[terminal.component_name.to_index()][terminal.label.to_index()] = signal_state,
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct ComponentGraphBuilder<Name: ConstParamTy_, const NAME_COUNT: usize, const LABEL_COUNT: usize> {
    components: BTreeMap<Name, Component<Name>>,
    signals: BTreeSet<Signal<Name>>,
}

impl <Name: ConstParamTy_ + PartialEq + Eq + PartialOrd + Ord + Clone + Copy + fmt::Debug + ToIndex, const NAME_COUNT: usize, const LABEL_COUNT: usize> ComponentGraphBuilder<Name, NAME_COUNT, LABEL_COUNT> {
    pub fn new() -> Self {
        Self {
            components: BTreeMap::new(),
            signals: BTreeSet::new(),
        }
    }

    pub fn component(mut self, component: Component<Name>) -> Self {
        self.components.insert(component.name(), component);
        self
    }

    pub fn signal(mut self, signal: Signal<Name>) -> Self {
        self.signals.insert(signal);
        self
    }

    pub fn build(mut self) -> ComponentGraph<Name, {NAME_COUNT}, {LABEL_COUNT}> {
        assert_eq!(self.components.len(), std::mem::variant_count::<Name>());
        let components = self.components.clone().into_iter()
            .map(|(_, component)| component)
            .collect();

        let mut sink_label_and_signal_by_component_name: Box<[(Component<Name>, Vec<(Label, Signal<Name>)>)]> =
            vec![(self.components.values().next().unwrap().clone(), Vec::new()); std::mem::variant_count::<Name>()].into_boxed_slice();
        for (signal, sink_terminal) in self.signals.iter().flat_map(|signal| signal.clone().sink_terminals().into_iter().map(|sink_terminal| (signal.clone(), sink_terminal))) {
            let (component, mut signals) = sink_label_and_signal_by_component_name[sink_terminal.component_name().to_index()].clone();
            if signals.is_empty() {
                let component = self.components.remove(&sink_terminal.component_name())
                    .expect(&format!("Signal refers to a component ({:?}) that doesn't exist.", sink_terminal.component_name()));
                let signals = vec![(sink_terminal.label(), signal)];
                sink_label_and_signal_by_component_name[component.name().to_index()] = (component.clone(), signals);
            } else {
                signals.push((sink_terminal.label(), signal));
                sink_label_and_signal_by_component_name[sink_terminal.component_name().to_index()] = (component, signals);
            }
        }

        let mut signals_by_source_terminal: Vec<[Vec<Signal<Name>>; LABEL_COUNT]> = Vec::new();
        for _ in 0..NAME_COUNT {
            let labels: Vec<_> = (0..LABEL_COUNT).map(|_| Vec::new()).collect();
            signals_by_source_terminal.push(labels.try_into().unwrap());
        }

        let mut signals_by_source_terminal: Box<[[Vec<Signal<Name>>; LABEL_COUNT]; NAME_COUNT]> = Box::new(signals_by_source_terminal.try_into().unwrap());
        for signal in &self.signals {
            match signal.source {
                Source::Primary(_) => {}
                Source::Internal(ref terminal) => {
                    signals_by_source_terminal[terminal.component_name.to_index()][terminal.label.to_index()].push(signal.clone());
                }
            }
        }

        let primary_sinks = [false; PRIMARY_SINK_COUNT];
        let primary_signal_states: [bool; std::mem::variant_count::<PrimarySource>()] =
            [false; std::mem::variant_count::<PrimarySource>()];
        let internal_signal_states = Box::new([[SignalState::Unknown; LABEL_COUNT]; NAME_COUNT]);
        ComponentGraph {
            primary_sinks,
            component_sink_info: sink_label_and_signal_by_component_name,
            signals_by_source_terminal,
            primary_signal_states,
            internal_signal_states,
            components,
            signals: self.signals.into_iter().collect(),
        }
    }
}