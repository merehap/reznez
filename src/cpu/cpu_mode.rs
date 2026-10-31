use std::fmt;

use crate::cpu::step::*;
use crate::cpu::instruction::Instruction;
use crate::memory::cpu::cpu_address::CpuAddress;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SubState {
    Normal,
    Oops,
    BranchTaken,
    BranchOops,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CpuMode {
    WaitingForRst,
    FirstStep,
    Instruction { sub_state: SubState, pending_sub_state: SubState },
    InterruptSequence,
    Jammed,
}

impl fmt::Display for CpuMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::WaitingForRst => write!(f, "Wait"),
            Self::FirstStep => write!(f, "First"),
            Self::Instruction{..} => write!(f, "Instruction"),
            Self::InterruptSequence => write!(f, "INT"),
            Self::Jammed => write!(f, "Jammed"),
        }
    }
}

#[derive(Debug)]
pub struct CpuModeState {
    steps: &'static [Step],
    step_index: usize,

    mode: CpuMode,

    instruction: Instruction,
    next_sequence: NextSequence,
    is_reset: bool,
    code_point_address: CpuAddress,
}

impl CpuModeState {
    pub fn startup() -> Self {
        Self {
            steps: &[FIRST_STEP],
            step_index: 0,

            mode: CpuMode::WaitingForRst,

            instruction: Instruction::from_code_point(0), // Initial value ignored.
            next_sequence: NextSequence::NotReady,
            is_reset: false,
            code_point_address: CpuAddress::ZERO,
        }
    }

    pub fn state_label(&self) -> String {
        match self.mode {
            CpuMode::WaitingForRst => "WAIT".to_owned(),
            CpuMode::FirstStep => "FIRST".to_owned(),
            CpuMode::Instruction { sub_state, .. } => match sub_state {
                SubState::Normal => format!("{:?}", self.instruction.op_code()),
                SubState::Oops => "OOPS".to_owned(),
                SubState::BranchTaken => "BTAKEN".to_owned(),
                SubState::BranchOops => "BOOPS".to_owned(),
            }
            CpuMode::InterruptSequence => "INT".to_owned(),
            CpuMode::Jammed => "JAM".to_owned(),
        }
    }

    pub fn mode(&self) -> CpuMode {
        self.mode
    }

    pub fn is_jammed(&self) -> bool {
        self.mode == CpuMode::Jammed
    }

    pub fn should_suppress_next_instruction_start(&self) -> bool {
        matches!(self.mode, CpuMode::Instruction { pending_sub_state: SubState::BranchTaken | SubState::BranchOops, ..})
    }

    pub fn is_interrupt_sequence_active(&self) -> bool {
        self.next_sequence.is_interrupt()
            || (self.mode == CpuMode::InterruptSequence && self.step_index < self.steps.len() - 1)
    }

    pub fn current_step(&self) -> Step {
        match self.mode {
            CpuMode::Instruction { sub_state: SubState::Oops, .. } => OOPS_STEP,
            CpuMode::Instruction { sub_state: SubState::BranchTaken, .. } => BRANCH_TAKEN_STEP,
            CpuMode::Instruction { sub_state: SubState::BranchOops, .. } => READ_OP_CODE_STEP,
            _ => self.steps[self.step_index],
        }
    }

    pub fn current_instruction(&self) -> Option<Instruction> {
        if matches!(self.mode, CpuMode::Instruction {..}) {
            Some(self.instruction)
        } else {
            None
        }
    }

    pub fn new_instruction_with_address(&self) -> Option<(Instruction, CpuAddress)> {
        if let NextSequence::Instruction(next_instruction) = self.next_sequence {
            Some((next_instruction, self.code_point_address))
        } else {
            None
        }
    }

    pub fn instruction(&mut self, instruction_code_point: u8, code_point_address: CpuAddress) {
        self.code_point_address = code_point_address;
        self.next_sequence = NextSequence::Instruction(Instruction::from_code_point(instruction_code_point));
    }

    pub fn interrupt_sequence(&mut self) {
        self.is_reset = self.mode == CpuMode::FirstStep;
        self.next_sequence = NextSequence::Interrupt;
    }

    pub fn jammed(&mut self) {
        self.mode = CpuMode::Jammed;
    }

    pub fn branch_taken(&mut self) {
        self.mode = CpuMode::Instruction {
            sub_state: SubState::Normal,
            pending_sub_state: SubState::BranchTaken
        };
    }

    pub fn branch_oops(&mut self) {
        self.mode = CpuMode::Instruction {
            sub_state: SubState::BranchTaken,
            pending_sub_state: SubState::BranchOops,
        };
    }

    pub fn oops(&mut self) {
        self.mode = CpuMode::Instruction {
            sub_state: SubState::Normal,
            pending_sub_state: SubState::Oops,
        };
    }

    pub fn step(&mut self) {
        match self.mode {
            CpuMode::WaitingForRst => {
                self.mode = CpuMode::FirstStep;
            }
            CpuMode::Jammed => { /* Stay jammed. */ }
            CpuMode::Instruction { pending_sub_state, .. } if pending_sub_state != SubState::Normal => {
                self.mode = CpuMode::Instruction { sub_state: pending_sub_state, pending_sub_state: SubState::Normal };
            }
            _ if self.step_index == self.steps.len() - 1 => {
                self.step_index = 0;
                self.mode = match self.next_sequence {
                    NextSequence::NotReady => panic!("Next sequence should have been ready!"),
                    NextSequence::Instruction(instruction) => {
                        self.instruction = instruction;
                        self.steps = instruction.steps();
                        CpuMode::Instruction { sub_state: SubState::Normal, pending_sub_state: SubState::Normal }
                    }
                    NextSequence::Interrupt => {
                        self.instruction = Instruction::from_code_point(0x00);
                        // TODO: Unify these branches.
                        self.steps = if self.mode == CpuMode::FirstStep {
                            RESET_STEPS
                        } else {
                            self.instruction.steps()
                        };

                        CpuMode::InterruptSequence
                    }
                };
            }
            CpuMode::FirstStep => {
                panic!();
            }
            CpuMode::Instruction {..} => {
                self.step_index += 1;
                self.mode = CpuMode::Instruction { sub_state: SubState::Normal, pending_sub_state: SubState::Normal };
            }
            CpuMode::InterruptSequence => {
                self.step_index += 1;
            }
        }
    }

    pub fn step_name(&self) -> String {
        let name: String = match self.mode {
            CpuMode::WaitingForRst => "WAIT".to_owned(),
            CpuMode::FirstStep => "FIRST".to_owned(),
            CpuMode::Instruction { sub_state: SubState::Normal, ..} => format!("{:?}{}", self.instruction.op_code(), self.step_index + 1),
            CpuMode::Instruction { sub_state: SubState::Oops, ..} => "OOPS".to_owned(),
            CpuMode::Instruction { sub_state: SubState::BranchTaken, ..} => "BTAKEN".to_owned(),
            CpuMode::Instruction { sub_state: SubState::BranchOops, ..} => "BOOPS".to_owned(),
            CpuMode::InterruptSequence => format!("INT{}", self.step_index + 1),
            CpuMode::Jammed => "JAMMED".to_owned(),
        };
        format!("{name:<6}")
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum InterruptType {
    Nmi,
    Reset,
    Irq,
}

#[derive(Clone, Copy, Debug)]
enum NextSequence {
    NotReady,
    Instruction(Instruction),
    Interrupt,
}

impl NextSequence {
    fn is_interrupt(self) -> bool {
        matches!(self, Self::Interrupt)
    }
}