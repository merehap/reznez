use std::fmt;
use std::marker::ConstParamTy_;

use crate::circuit::component::{Component, ToIndex};
use crate::circuit::signal::Signal;
use crate::circuit::source::{PrimarySource, Source};

pub enum ComponentTypeName {
    Nor,
    DLatch(bool),
    SRLatch(bool),
}

#[derive(Clone, Debug)]
pub enum Factor<ComponentName: ConstParamTy_> {
    PrimarySource(PrimarySource),
    Nor { a: Signal<ComponentName>, b: Signal<ComponentName> },
    DLatchQ { e: Signal<ComponentName>, d: Signal<ComponentName>, default_state: bool },
    DLatchN { e: Signal<ComponentName>, d: Signal<ComponentName>, default_state: bool },
    SRLatchQ { s: Signal<ComponentName>, r: Signal<ComponentName>, default_state: bool },
    SRLatchN { s: Signal<ComponentName>, r: Signal<ComponentName>, default_state: bool },
}

pub struct FactorGraph<ComponentName: ConstParamTy_> {
    factors_with_source: Vec<(Factor<ComponentName>, Source<ComponentName>)>,
    signals: Vec<Signal<ComponentName>>,
}

impl <ComponentName: ConstParamTy_ + Clone + Copy + fmt::Debug + ToIndex> FactorGraph<ComponentName> {
    pub fn new(signals: Vec<Signal<ComponentName>>, primary_sources: &[PrimarySource]) -> Self {
        Self {
            factors_with_source: primary_sources.iter()
                .map(|primary_source| (Factor::PrimarySource(*primary_source), Source::Primary(*primary_source)))
                .collect(),
            signals,
        }
    }

    pub fn push_component(&mut self, component: &Component<ComponentName>) {
        for factor in component.factors(&self.signals) {
            self.factors_with_source.insert(0, factor);
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Node {
    Constant(bool),
    State(Index),
    NotState(Index),

    PrimarySource(PrimarySource),

    Identity(Index),
    Not(Index),

    StateOrState { state_a: Index, state_b: Index },
    StateAndNotState { state_a: Index, state_b: Index },

    StateOr { state: Index, other: Index },
    StateAndNot { state: Index, other: Index },
    StateOrNot { state: Index, other: Index },

    Nor(Index, Index),
    DLatchQ { e: Index, d: Index },
    DLatchN { e: Index, d: Index },
    SRLatchQ { s: Index, r: Index },
    SRLatchN { s: Index , r: Index },
}

pub type Index = usize;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct NodeGraph<ComponentName: ConstParamTy_ + Clone + Copy + fmt::Debug + ToIndex> {
    states: Vec<bool>,
    cofactors: Vec<Cofactor<ComponentName>>,
}

impl <ComponentName: ConstParamTy_ + Clone + Copy + fmt::Debug + fmt::Display + ToIndex> NodeGraph<ComponentName> {
    pub fn new(factor_graph: FactorGraph<ComponentName>) -> Self {
        let mut factors_with_source = factor_graph.factors_with_source.clone();
        for i in (0..factors_with_source.len()).rev() {
            let source = &factors_with_source[i].1;
            if matches!(source, Source::Internal(_)) {
                let found = factor_graph.signals.iter()
                    .find(|signal| signal.source == *source)
                    .is_some();
                if !found {
                    // Nothing is hooked up to this source.
                    println!("Removing source {source:?}");
                    //factors_with_source.remove(i);
                }
            }
        }

        let signal_to_source_index = |signal: Signal<ComponentName>| -> Index {
            for (i, (_factor, source)) in factors_with_source.iter().enumerate() {
                if signal.source == source.clone() {
                    return i;
                }
            }

            panic!();
        };

        let factor_to_node = |factor: Factor<ComponentName>| -> (Node, bool) {
            match factor {
                Factor::PrimarySource(primary_source) => (Node::PrimarySource(primary_source), false),
                Factor::Nor { a, b } => (Node::Nor(signal_to_source_index(a), signal_to_source_index(b)), false),
                Factor::DLatchQ { e, d, default_state } => (Node::DLatchQ { e: signal_to_source_index(e), d: signal_to_source_index(d) }, default_state),
                Factor::DLatchN { e, d, default_state } => (Node::DLatchN { e: signal_to_source_index(e), d: signal_to_source_index(d) }, default_state),
                Factor::SRLatchQ { s, r, default_state } => (Node::SRLatchQ { s: signal_to_source_index(s), r: signal_to_source_index(r) }, default_state),
                Factor::SRLatchN { s, r, default_state } => (Node::SRLatchN { s: signal_to_source_index(s), r: signal_to_source_index(r) }, default_state),
            }
        };

        //let mut states = vec![false; primary_sources.len() + factor_graph.factors_with_source.len()];
        let mut nodes = Vec::new();
        let mut states = Vec::new();
        for (i, (factor, source)) in factor_graph.factors_with_source.into_iter().enumerate() {
            let (node, default_state) = factor_to_node(factor.clone());
            nodes.push((source.clone(), node, i));
            states.push(default_state);
        }

        let primary_sources = [PrimarySource::PpuClockFirstHalf, PrimarySource::PpuClockSecondHalf, PrimarySource::Read2007, PrimarySource::Write2007];

        let state_count = states.len();
        let mut cofactors = Vec::new();
        for mut i in 0..2u8.pow(primary_sources.len() as u32) {
            for j in 0..primary_sources.len() {
                let constant = i & 1 == 1;
                print!("{}={constant} ", primary_sources[j]);
                let node_index = state_count - primary_sources.len() + j;
                nodes[node_index] = (Source::Primary(primary_sources[j]), Node::Constant(constant), node_index);
                i >>= 1;
            }

            println!();

            let mut cofactor = Cofactor { nodes: nodes.clone() };
            cofactor.propagate_constants();
            //cofactor.nodes.truncate(cofactor.nodes.len() - primary_sources.len());
            cofactor.sort();
            //println!("SORTED {:?}", cofactor.nodes);
            println!();
            println!();

            cofactors.push(cofactor);
        }

        //states.truncate(states.len() - primary_sources.len());

        Self { states, cofactors }
    }

    pub fn evaluate(&mut self, primary_sources: [bool; PrimarySource::COUNT]) {
        let cofactor_index: usize = primary_sources.iter().rev().enumerate()
            .map(|(i, value)| (*value as usize) << (PrimarySource::COUNT - i - 1))
            .sum();
        println!("Cofactor index: {cofactor_index:04b}");
        self.cofactors[cofactor_index].evaluate(&mut self.states);
        self.cofactors[cofactor_index].print_states(&self.states);
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Cofactor<ComponentName: ConstParamTy_ + Clone + Copy + fmt::Debug + ToIndex> {
    nodes: Vec<(Source<ComponentName>, Node, usize)>,
}

impl <ComponentName: ConstParamTy_ + Clone + Copy + fmt::Debug + fmt::Display + ToIndex> Cofactor<ComponentName> {
    fn print_states(&self, states: &[bool]) {
        assert_eq!(states.len(), self.nodes.len());
        for i in 0..states.len() {
            for (source, _node, state_index) in &self.nodes {
                if *state_index == i {
                    print!("{}:{} ", source, states[i] as u8);
                }
            }
        }

        println!();
        println!();
    }

    fn evaluate(&self, states: &mut [bool]) {
        for (_source, node, target_index) in &self.nodes {
            let value = match node {
                Node::PrimarySource(_) => panic!("PrimarySources should have been removed by now."),
                Node::Constant(value) => *value,
                Node::Identity(index) | Node::State(index) => states[*index],
                Node::Not(index) | Node::NotState(index) => !states[*index],
                Node::StateOrState { state_a, state_b } => states[*state_a] || states[*state_b],
                Node::StateOr { state, other } => states[*state] || states[*other],
                Node::StateOrNot { state, other } => states[*state] || !states[*other],
                Node::StateAndNotState { state_a, state_b } => states[*state_a] && !states[*state_b],
                Node::StateAndNot { state, other } => states[*state] && !states[*other],
                Node::Nor(a, b) => !states[*a] && !states[*b],
                Node::DLatchQ {..} | Node::DLatchN {..} | Node::SRLatchQ {..} | Node::SRLatchN {..} => todo!("How was this not reduced?"),
            };

            states[*target_index] = value;
        }
    }

    fn sort(&mut self) {
        let mut sorted_nodes = Vec::new();

        let mut unsorted_nodes: Vec<_> = self.nodes.clone().into_iter()
            .map(Some)
            .collect();

        loop {
            let mut node_pushed = false;
            for i in 0..unsorted_nodes.len() {
                if let Some(source_node) = unsorted_nodes[i].clone() {
                    use Node::*;
                    match source_node.1 {
                        PrimarySource(_) => {
                            panic!("PrimarySources should have already been eliminated.");
                        }
                        Constant(..) => {
                            sorted_nodes.push(source_node);
                            unsorted_nodes[i] = None;
                        }
                        State(_) | NotState(_) | StateOrState {..} | StateAndNotState {..} => {
                            sorted_nodes.push(source_node);
                            unsorted_nodes[i] = None;
                        }
                        Identity(index) | Not(index) | StateOr { other: index, .. } | StateOrNot { other: index, .. } | StateAndNot { other: index, .. } => {
                            if unsorted_nodes[index].is_none() {
                                sorted_nodes.push(source_node);
                                unsorted_nodes[i] = None;
                            }
                        }
                        Nor(a, b) | DLatchQ { e: a, d: b } | DLatchN { e: a, d: b } | SRLatchQ { s: a, r: b } | SRLatchN { s: a, r: b } => {
                            if unsorted_nodes[a].is_none() && unsorted_nodes[b].is_none() {
                                sorted_nodes.push(source_node);
                                unsorted_nodes[i] = None;
                            }
                        }
                    }

                    if unsorted_nodes[i].is_none() {
                        node_pushed = true;
                    }
                }
            }

            if !node_pushed {
                break;
            }
        }

        //assert_eq!(self.nodes.len(), sorted_nodes.len());
        self.nodes = sorted_nodes;
    }

    fn propagate_constants(&mut self) {
        loop {
            let reduced = self.reduce();
            if reduced {
                //println!("REDUCED {:?}", self.nodes);
            } else {
                return;
            }
        }
    }

    fn reduce(&mut self) -> bool {
        let original = self.clone();

        use Node::*;
        for i in 0..self.nodes.len() {
            let (source_terminal, node, _i) = self.nodes[i].clone();
            let new_node = match node {
                Identity(index) if let Constant(constant) = self.nodes[index].1 => Some(Constant(constant)),
                Identity(index) if let State(target) = self.nodes[index].1 => Some(State(target)),
                Not(index) if let Constant(constant) = self.nodes[index].1 => Some(Constant(!constant)),
                Not(index) if let State(index) = self.nodes[index].1 => Some(NotState(index)),
                StateOr { state, other } if let State(state_b) = self.nodes[other].1 => Some(StateOrState { state_a: state, state_b }),
                StateOr { other, .. } if let Constant(true) = self.nodes[other].1 => Some(Constant(true)),
                StateOr { state, other, .. } if let Constant(false) = self.nodes[other].1 => Some(State(state)),
                StateOrNot { state, other } if let Constant(true) = self.nodes[other].1 => Some(State(state)),
                StateOrNot { other, .. } if let Constant(false) = self.nodes[other].1 => Some(Constant(true)),
                StateAndNot { state, other } if let State(state_b) = self.nodes[other].1 => Some(StateAndNotState { state_a: state, state_b }),
                StateAndNot { state, other } if let Constant(true) = self.nodes[other].1 => Some(State(state)),
                StateAndNot { other, .. } if let Constant(false) = self.nodes[other].1 => Some(Constant(false)),
                Identity(_) | Not(_) | StateOrState {..} | StateAndNotState {..} | StateOr {..} | StateOrNot {..} | StateAndNot {..} => None,
                Constant(..) | PrimarySource(_) | State(_) | NotState(_) => None,
                Nor(a, b) => match (self.nodes[a].1, self.nodes[b].1) {
                    (Constant(constant_a), Constant(constant_b)) => Some(Constant(!constant_a && !constant_b)),
                    (Constant(true), _) => Some(Constant(false)),
                    (_, Constant(true)) => Some(Constant(false)),
                    (Constant(false), _) => Some(Not(b)),
                    (_, Constant(false)) => Some(Not(a)),
                    (_, _) => None,
                }
                DLatchQ { e, d } => match (self.nodes[e].1, self.nodes[d].1) {
                    (Constant(false), _) => Some(State(i)),
                    (Constant(true), Constant(constant_d)) => Some(Constant(constant_d)),
                    (Constant(true), _) => Some(Identity(d)),
                    (_, Constant(true)) => Some(StateOr { state: i, other: e }),
                    (_, Constant(false)) => Some(StateAndNot { state: i, other: e }),
                    (_, _) => None,
                }
                DLatchN { e, d } => match (self.nodes[e].1, self.nodes[d].1) {
                    (Constant(false), _) => Some(State(i)),
                    (Constant(true), Constant(constant_d)) => Some(Constant(!constant_d)),
                    (Constant(true), _) => Some(Not(d)),
                    (_, Constant(true)) => Some(Nor(e, i)),
                    (_, Constant(false)) => Some(StateOr { state: i, other: e }),
                    (_, _) => None,
                }
                SRLatchQ { s, r } => match (self.nodes[s].1, self.nodes[r].1) {
                    (Constant(false), Constant(false)) => Some(State(i)),
                    (Constant(true) , Constant(false)) => Some(Constant(true)),
                    (Constant(_)    , Constant(true) ) => Some(Constant(false)),
                    (Constant(true), _) => Some(Not(r)),
                    (Constant(false), _) => Some(StateAndNot { state: i, other: r }),
                    (_, Constant(true)) => Some(Constant(false)),
                    (_, Constant(false)) => Some(StateOr { state: i, other: s }),
                    (_, _) => None,
                }
                SRLatchN { s, r } => match (self.nodes[s].1, self.nodes[r].1) {
                    (Constant(false), Constant(false)) => Some(State(i)),
                    (Constant(false), Constant(true) ) => Some(Constant(true)),
                    (Constant(true) , Constant(_)    ) => Some(Constant(false)),
                    (Constant(true), _) => Some(Identity(r)),
                    (Constant(false), _) => Some(StateOr { state: i, other: r }),
                    (_, Constant(true)) => Some(Not(s)),
                    (_, Constant(false)) => Some(StateOrNot { state: i, other: s }),
                    (_, _) => None,
                }
            };

            if let Some(new_node) = new_node {
                self.nodes[i] = (source_terminal, new_node, i);
            }
        }

        let reduced = *self != original;
        reduced
    }
}