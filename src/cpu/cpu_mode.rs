use crate::apu::apu_registers::CycleParity;
use crate::cpu::step::*;
use crate::cpu::instruction::{Instruction, OpCode};
use crate::memory::cpu::cpu_address::CpuAddress;

#[derive(PartialEq, Eq, Clone, Debug)]
enum CpuMode {
    Instruction(OpCode),
    InterruptSequence(InterruptType),
    DmcDma {
        suspended_mode: Box<CpuMode>,
        suspended_steps: &'static [Step],
        suspended_step_index: usize,
    },

    Jammed,
    StartNext,
    BranchTaken,
    BranchOops,
    // FIXME: If pending, OAM DMA should be triggered on Oops steps.
    Oops {
        suspended_op_code: OpCode,
    },
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
            CpuMode::Instruction(op_code) => format!("{op_code:?}"),
            CpuMode::InterruptSequence(InterruptType::Reset) => "RESET".to_owned(),
            CpuMode::InterruptSequence(InterruptType::Irq) => "IRQ".to_owned(),
            CpuMode::InterruptSequence(InterruptType::Nmi) => "NMI".to_owned(),
            CpuMode::DmcDma {..} => "DMCDMA".to_owned(),
            CpuMode::Jammed => "JAM".to_owned(),
            CpuMode::StartNext => "STARTNEXT".to_owned(),
            CpuMode::BranchTaken => "BTAKEN".to_owned(),
            CpuMode::BranchOops => "BOOPS".to_owned(),
            CpuMode::Oops {..} => "OOPS".to_owned(),
        }
    }

    pub fn is_jammed(&self) -> bool {
        self.mode == CpuMode::Jammed
    }

    pub fn should_suppress_next_instruction_start(&self) -> bool {
        matches!(self.next_mode, Some(CpuMode::BranchTaken | CpuMode::BranchOops))
    }

    pub fn is_interrupt_sequence_active(&self) -> bool {
        matches!(self.next_mode, Some(CpuMode::InterruptSequence {..}))
            || matches!(self.mode, CpuMode::InterruptSequence {..})
    }

    pub fn current_step(&self) -> Step {
        match self.mode {
            CpuMode::Oops {..} => OOPS_STEP,
            _ => self.steps[self.step_index],
        }
    }

    pub fn current_instruction(&self) -> Option<Instruction> {
        self.current_instruction
    }

    pub fn new_instruction_with_address(&self) -> Option<(Instruction, CpuAddress)> {
        if !matches!(self.mode, CpuMode::StartNext | CpuMode::Instruction {..}) {
            return None;
        }

        self.new_instruction_with_address
    }

    pub fn clear_new_instruction(&mut self) {
        self.new_instruction_with_address = None;
    }

    pub fn set_current_instruction_with_address(&mut self, instruction: Instruction, address: CpuAddress) {
        if !matches!(self.next_mode, Some(CpuMode::DmcDma {..})) {
            self.current_instruction = Some(instruction);
        }

        self.new_instruction_with_address = Some((instruction, address));
    }

    pub fn instruction(&mut self, instruction: Instruction) {
        self.steps = instruction.steps();
        self.next_mode = Some(CpuMode::Instruction(instruction.op_code()));
    }

    pub fn interrupt_sequence(&mut self, interrupt_type: InterruptType) {
        if self.next_mode == Some(CpuMode::Jammed) {
            return;
        }

        assert_eq!(self.next_mode, None, "next_mode should not already be set");
        self.next_mode = Some(CpuMode::InterruptSequence(interrupt_type));
        self.current_instruction = None;
    }

    pub fn dmc_dma(&mut self) {
        assert!(self.mode != CpuMode::InterruptSequence(InterruptType::Reset));

        self.next_mode = Some(CpuMode::DmcDma {
            suspended_mode: Box::new(self.mode.clone()),
            suspended_steps: self.steps,
            suspended_step_index: self.step_index,
        });
    }

    pub fn jammed(&mut self) {
        assert_eq!(self.next_mode, None, "next_mode should not already be set");
        self.next_mode = Some(CpuMode::Jammed);
    }

    pub fn branch_taken(&mut self) {
        assert_eq!(self.next_mode, None, "next_mode should not already be set");
        self.next_mode = Some(CpuMode::BranchTaken);
    }

    pub fn branch_oops(&mut self) {
        assert_eq!(self.next_mode, None, "next_mode should not already be set");
        self.next_mode = Some(CpuMode::BranchOops);
    }

    // FIXME: If pending, OAM DMA should be triggered on Oops steps.
    pub fn oops(&mut self) {
        assert_eq!(self.next_mode, None, "next_mode should not already be set");
        let CpuMode::Instruction(op_code) = self.mode else {
            unreachable!("Oops steps can only occur during Instructions");
        };

        // Don't repeat the last instruction step when we resume.
        self.step_index += 1;
        self.next_mode = Some(CpuMode::Oops {
            suspended_op_code: op_code,
        });
    }

    pub fn interrupt_vector_set(&mut self, interrupt_vector: Option<InterruptType>) {
        // FIXME: This doesn't currently let RESET hijack properly since RESET has an extra step.
        if let Some(new_interrupt_type) = interrupt_vector {
            self.mode = CpuMode::InterruptSequence(new_interrupt_type);
        }
    }

    pub fn step(&mut self, cycle_parity: CycleParity) {
        if let Some(next_mode) = self.next_mode.take() {
            match next_mode {
                CpuMode::StartNext {..} => unreachable!(),
                CpuMode::Jammed => self.steps = &[],

                CpuMode::DmcDma {..} => {
                    self.steps = match cycle_parity {
                        CycleParity::Get => ALIGNED_DMC_DMA_TRANSFER_STEPS,
                        CycleParity::Put => DMC_DMA_TRANSFER_STEPS,
                    };
                }
                CpuMode::InterruptSequence(InterruptType::Reset) => self.steps = RESET_STEPS,
                CpuMode::InterruptSequence(_) => self.steps = BRK_STEPS,
                CpuMode::Instruction {..} => { /* steps will be set by the caller in this case. */ }
                CpuMode::BranchTaken => self.steps = &[BRANCH_TAKEN_STEP],
                CpuMode::Oops {..} => {
                    assert!(matches!(self.mode, CpuMode::Instruction {..}));
                }
                CpuMode::BranchOops => self.steps = &[READ_OP_CODE_STEP],
            }

            self.mode = next_mode;
            if !matches!(self.mode, CpuMode::Oops {..}) {
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
            CpuMode::Instruction {..} | CpuMode::InterruptSequence {..} => {
                self.steps = &[READ_OP_CODE_STEP];
                self.step_index = 0;
                CpuMode::StartNext
            }
            CpuMode::DmcDma { suspended_mode, suspended_steps, suspended_step_index } => {
                self.steps = suspended_steps;
                self.step_index = suspended_step_index;
                *suspended_mode
            }

            CpuMode::Jammed => CpuMode::Jammed,
            CpuMode::StartNext {..} => panic!(),
            CpuMode::BranchTaken => panic!(),
            CpuMode::BranchOops => panic!(),
            CpuMode::Oops { suspended_op_code } => {
                CpuMode::Instruction(suspended_op_code)
            }
        };
    }

    pub fn step_name(&self) -> String {
        let name: String = match (&self.mode, &self.next_mode) {
            (_, Some(CpuMode::DmcDma {..})) =>
                "DMC0".into(),
            (CpuMode::Oops {..}, _) =>
                "OOPS".into(),
            (CpuMode::BranchTaken, _) =>
                "BTAKEN".into(),
            (CpuMode::BranchOops, _) =>
                "BOOPS".into(),
            (CpuMode::Jammed, _) =>
                "JAMMED".into(),
            (CpuMode::StartNext, Some(CpuMode::Instruction(op_code))) =>
                format!("{op_code:?}0"),
            (CpuMode::StartNext, Some(CpuMode::InterruptSequence(InterruptType::Irq))) => "IRQ0".into(),
            (CpuMode::StartNext, Some(CpuMode::InterruptSequence(InterruptType::Nmi))) => "NMI0".into(),
            (CpuMode::StartNext, Some(CpuMode::InterruptSequence(InterruptType::Reset))) => "RESET0".into(),
            (CpuMode::StartNext, _) => unreachable!(),
            (CpuMode::Instruction(op_code), _) =>
                format!("{:?}{}", op_code, self.step_index + 1),
            (CpuMode::InterruptSequence(InterruptType::Irq), _) =>
                format!("IRQ{}", self.step_index + 1),
            (CpuMode::InterruptSequence(InterruptType::Nmi), _) =>
                format!("NMI{}", self.step_index + 1),
            (CpuMode::InterruptSequence(InterruptType::Reset), _) =>
                format!("RESET{}", self.step_index),
            (CpuMode::DmcDma {..} , _) =>
                format!("DMC{}", self.step_index + 1),
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