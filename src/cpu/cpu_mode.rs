use crate::cpu::step::*;
use crate::cpu::instruction::{Instruction, OpCode};
use crate::memory::cpu::cpu_address::CpuAddress;

#[derive(PartialEq, Eq, Clone, Debug)]
enum CpuMode {
    StartNext,
    Instruction(OpCode, InstructionMode),
    InterruptSequence(InterruptType),
    Jammed,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum InstructionMode {
    Normal,
    Oops,
    BranchTaken,
    BranchOops,
}


#[derive(Debug)]
pub struct CpuModeState {
    steps: &'static [Step],
    step_index: usize,
    mode: CpuMode,
    next_mode: Option<CpuMode>,

    current_instruction: Option<Instruction>,
    new_instruction_with_address: Option<(Instruction, CpuAddress)>,
}

impl CpuModeState {
    pub fn startup() -> Self {
        Self {
            steps: RESET_STEPS,
            step_index: 0,
            mode: CpuMode::InterruptSequence(InterruptType::Reset),
            next_mode: None,
            current_instruction: None,

            new_instruction_with_address: None,
        }
    }

    pub fn state_label(&self) -> String {
        match self.mode {
            CpuMode::Instruction(op_code, instruction_mode) => {
                match instruction_mode {
                    InstructionMode::Normal => format!("{op_code:?}"),
                    InstructionMode::Oops => "OOPS".to_owned(),
                    InstructionMode::BranchTaken => "BTAKEN".to_owned(),
                    InstructionMode::BranchOops => "BOOPS".to_owned(),
                }
            }
            CpuMode::InterruptSequence(InterruptType::Reset) => "RESET".to_owned(),
            CpuMode::InterruptSequence(InterruptType::Irq) => "IRQ".to_owned(),
            CpuMode::InterruptSequence(InterruptType::Nmi) => "NMI".to_owned(),
            CpuMode::Jammed => "JAM".to_owned(),
            CpuMode::StartNext => "STARTNEXT".to_owned(),
        }
    }

    pub fn is_jammed(&self) -> bool {
        self.mode == CpuMode::Jammed
    }

    pub fn should_suppress_next_instruction_start(&self) -> bool {
        matches!(self.next_mode, Some(CpuMode::Instruction(_, InstructionMode::BranchTaken | InstructionMode::BranchOops)))
    }

    pub fn is_interrupt_sequence_active(&self) -> bool {
        matches!(self.next_mode, Some(CpuMode::InterruptSequence {..}))
            || matches!(self.mode, CpuMode::InterruptSequence {..})
    }

    pub fn current_step(&self) -> Step {
        match self.mode {
            CpuMode::Instruction(_, InstructionMode::Oops) => OOPS_STEP,
            _ => self.steps[self.step_index],
        }
    }

    pub fn current_instruction(&self) -> Option<Instruction> {
        self.current_instruction
    }

    pub fn new_instruction_with_address(&self) -> Option<(Instruction, CpuAddress)> {
        if matches!(self.mode, CpuMode::StartNext | CpuMode::Instruction(_, InstructionMode::Normal)) {
            self.new_instruction_with_address
        } else {
            None
        }

    }

    pub fn clear_new_instruction(&mut self) {
        self.new_instruction_with_address = None;
    }

    pub fn set_current_instruction_with_address(&mut self, instruction: Instruction, address: CpuAddress) {
        self.current_instruction = Some(instruction);
        self.new_instruction_with_address = Some((instruction, address));
    }

    pub fn instruction(&mut self, instruction: Instruction) {
        self.steps = instruction.steps();
        self.next_mode = Some(CpuMode::Instruction(instruction.op_code(), InstructionMode::Normal));
    }

    pub fn interrupt_sequence(&mut self, interrupt_type: InterruptType) {
        if self.next_mode == Some(CpuMode::Jammed) {
            return;
        }

        assert_eq!(self.next_mode, None, "next_mode should not already be set");
        self.next_mode = Some(CpuMode::InterruptSequence(interrupt_type));
        self.current_instruction = None;
    }

    pub fn jammed(&mut self) {
        assert_eq!(self.next_mode, None, "next_mode should not already be set");
        self.next_mode = Some(CpuMode::Jammed);
    }

    pub fn branch_taken(&mut self) {
        assert_eq!(self.next_mode, None, "next_mode should not already be set");

        let CpuMode::Instruction(op_code, InstructionMode::Normal) = self.mode else {
            panic!("Current mode must be Instruction with no branching mode.");
        };
        self.next_mode = Some(CpuMode::Instruction(op_code, InstructionMode::BranchTaken));
    }

    pub fn branch_oops(&mut self) {
        assert_eq!(self.next_mode, None, "next_mode should not already be set");

        let CpuMode::Instruction(op_code, InstructionMode::BranchTaken) = self.mode else {
            panic!("Current mode must be Instruction (BranchTaken) with no branching mode.");
        };
        self.next_mode = Some(CpuMode::Instruction(op_code, InstructionMode::BranchOops));
    }

    pub fn oops(&mut self) {
        assert_eq!(self.next_mode, None, "next_mode should not already be set");
        let CpuMode::Instruction(op_code, InstructionMode::Normal) = self.mode else {
            unreachable!("Oops steps can only occur during Instructions");
        };

        // Don't repeat the last instruction step when we resume.
        self.step_index += 1;
        self.next_mode = Some(CpuMode::Instruction(op_code, InstructionMode::Oops));
    }

    pub fn interrupt_vector_set(&mut self, interrupt_vector: Option<InterruptType>) {
        // FIXME: This doesn't currently let RESET hijack properly since RESET has an extra step.
        if let Some(new_interrupt_type) = interrupt_vector {
            self.mode = CpuMode::InterruptSequence(new_interrupt_type);
        }
    }

    pub fn step(&mut self) {
        if let Some(next_mode) = self.next_mode.take() {
            match next_mode {
                CpuMode::StartNext {..} => unreachable!(),
                CpuMode::Jammed => self.steps = &[],

                CpuMode::InterruptSequence(InterruptType::Reset) => self.steps = RESET_STEPS,
                CpuMode::InterruptSequence(_) => self.steps = BRK_STEPS,
                CpuMode::Instruction(_, InstructionMode::Normal) => { /* steps will be set by the caller in this case. */ }
                CpuMode::Instruction(_, InstructionMode::Oops) => assert!(matches!(self.mode, CpuMode::Instruction {..})),
                CpuMode::Instruction(_, InstructionMode::BranchTaken) => self.steps = &[BRANCH_TAKEN_STEP],
                CpuMode::Instruction(_, InstructionMode::BranchOops) => self.steps = &[READ_OP_CODE_STEP],
            }

            self.mode = next_mode;
            if !matches!(self.mode, CpuMode::Instruction(_, InstructionMode::Oops)) {
                self.step_index = 0;
            }

            return;
        }

        if !self.is_last_step() {
            self.increment_step_index();
            return;
        }

        // Transition to a new mode since we're at the last index of the current one.
        self.mode = match self.mode.clone() {
            CpuMode::StartNext | CpuMode::Instruction(_, InstructionMode::BranchTaken | InstructionMode::BranchOops) => panic!(),
            CpuMode::Instruction(op_code, InstructionMode::Oops) => {
                CpuMode::Instruction(op_code, InstructionMode::Normal)
            }

            CpuMode::Instruction(_, InstructionMode::Normal) | CpuMode::InterruptSequence {..} => {
                self.steps = &[READ_OP_CODE_STEP];
                self.step_index = 0;
                CpuMode::StartNext
            }

            CpuMode::Jammed => CpuMode::Jammed,
        };
    }

    pub fn step_name(&self) -> String {
        let name: String = match (&self.mode, &self.next_mode) {
            (CpuMode::Jammed, _) =>
                "JAMMED".into(),
            (CpuMode::StartNext, Some(CpuMode::Instruction(op_code, InstructionMode::Normal))) =>
                format!("{op_code:?}0"),
            (CpuMode::StartNext, Some(CpuMode::InterruptSequence(InterruptType::Irq))) => "IRQ0".into(),
            (CpuMode::StartNext, Some(CpuMode::InterruptSequence(InterruptType::Nmi))) => "NMI0".into(),
            (CpuMode::StartNext, Some(CpuMode::InterruptSequence(InterruptType::Reset))) => "RESET0".into(),
            (CpuMode::StartNext, _) => unreachable!(),
            (CpuMode::Instruction(op_code, InstructionMode::Normal), _) =>
                format!("{:?}{}", op_code, self.step_index + 1),
            (CpuMode::Instruction(_, InstructionMode::Oops), _) => "OOPS".into(),
            (CpuMode::Instruction(_, InstructionMode::BranchTaken), _) => "BTAKEN".to_owned(),
            (CpuMode::Instruction(_, InstructionMode::BranchOops), _) => "BOOPS".to_owned(),
            (CpuMode::InterruptSequence(InterruptType::Irq), _) =>
                format!("IRQ{}", self.step_index + 1),
            (CpuMode::InterruptSequence(InterruptType::Nmi), _) =>
                format!("NMI{}", self.step_index + 1),
            (CpuMode::InterruptSequence(InterruptType::Reset), _) =>
                format!("RESET{}", self.step_index),
        };

        format!("{name:<6}")
    }

    fn is_last_step(&self) -> bool {
        self.step_index == self.steps.len() - 1
    }

    fn increment_step_index(&mut self) {
        self.step_index += 1;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum InterruptType {
    Nmi,
    Reset,
    Irq,
}