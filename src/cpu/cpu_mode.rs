use crate::apu::apu_registers::CycleParity;
use crate::cpu::step::*;
use crate::cpu::instruction::Instruction;
use crate::memory::cpu::cpu_address::CpuAddress;

#[derive(PartialEq, Eq, Clone, Debug)]
enum CpuMode {
    Instruction { oam_dma_pending: bool },
    InterruptSequence { reset: bool },
    OamDma,
    /*
    OamDma {
        suspended_mode: Box<CpuMode>,
        suspended_steps: &'static [Step],
        suspended_step_index: usize,
    },
    */

    Jammed,
    StartNext { oam_dma_pending: bool },
    BranchTaken,
    BranchOops,
    // FIXME: If pending, OAM DMA should be triggered on Oops steps.
    Oops {
        suspended_steps: &'static [Step],
        suspended_step_index: usize,
    },
}

#[derive(Debug)]
pub struct CpuModeState {
    steps: &'static [Step],
    step_index: usize,
    mode: CpuMode,
    next_mode: Option<CpuMode>,
    current_instruction_with_address: Option<(Instruction, CpuAddress)>,
}

impl CpuModeState {
    pub fn startup() -> Self {
        Self {
            steps: RESET_STEPS,
            step_index: 0,
            mode: CpuMode::InterruptSequence { reset: true },
            next_mode: None,
            current_instruction_with_address: None,
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
        self.steps[self.step_index]
    }

    pub fn current_instruction(&self) -> Option<Instruction> {
        self.current_instruction_with_address
            .map(|(instruction, _)| instruction)
    }

    pub fn current_instruction_with_address(&self) -> Option<(Instruction, CpuAddress)> {
        self.current_instruction_with_address
    }

    pub fn is_instruction_starting(&self) -> bool {
        matches!(self.mode, CpuMode::Instruction {..}) && self.step_index == 0
    }

    pub fn reset(&mut self) {
        assert_eq!(self.next_mode, None, "next_mode should not already be set");
        self.next_mode = Some(CpuMode::InterruptSequence { reset: true });
        self.current_instruction_with_address = None;
    }

    pub fn instruction(&mut self, instruction: Instruction, address: CpuAddress) {
        let oam_dma_pending = self.mode == CpuMode::Instruction { oam_dma_pending: true };
        assert_eq!(oam_dma_pending, false);

        self.current_instruction_with_address = Some((instruction, address));
        self.steps = instruction.steps();
        self.next_mode = Some(CpuMode::Instruction { oam_dma_pending });
    }

    pub fn interrupt_sequence(&mut self) {
        assert_eq!(self.next_mode, None, "next_mode should not already be set");
        self.next_mode = Some(CpuMode::InterruptSequence { reset: false });
        self.current_instruction_with_address = None;
    }

    pub fn oam_dma_pending(&mut self) {
        match self.mode {
            CpuMode::StartNext { oam_dma_pending: false } => self.next_mode = Some(CpuMode::OamDma),
            // CpuMode::StartNext { oam_dma_pending: false } => self.mode = CpuMode::StartNext { oam_dma_pending: true },
            CpuMode::Instruction { oam_dma_pending: false } => self.next_mode = Some(CpuMode::Instruction { oam_dma_pending: true }),
            _ => todo!(),
        }
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
        assert_eq!(self.mode, CpuMode::Instruction { oam_dma_pending: false });
        self.next_mode = Some(CpuMode::Oops {
            suspended_steps: self.steps,
            suspended_step_index: self.step_index + 1,
        });
    }

    pub fn step(&mut self, cycle_parity: CycleParity) {
        if let Some(next_mode) = self.next_mode.take() {
            match next_mode {
                CpuMode::StartNext {..} => unreachable!(),
                CpuMode::Jammed => self.steps = &[],

                CpuMode::OamDma => {
                    self.steps = match cycle_parity {
                        CycleParity::Get => &*OAM_DMA_TRANSFER_STEPS,
                        CycleParity::Put => &*ALIGNED_OAM_DMA_TRANSFER_STEPS,
                    };
                }
                CpuMode::InterruptSequence { reset: false } => self.steps = BRK_STEPS,
                CpuMode::InterruptSequence { reset: true } => self.steps = RESET_STEPS,
                CpuMode::Instruction {..} => { /* steps will be set by the caller in this case. */ }
                CpuMode::BranchTaken => self.steps = &[BRANCH_TAKEN_STEP],
                CpuMode::Oops {..} => {
                    assert_eq!(self.mode, CpuMode::Instruction { oam_dma_pending: false });
                    self.steps = &[ADDRESS_BUS_READ_STEP];
                }
                CpuMode::BranchOops => self.steps = &[READ_OP_CODE_STEP],
            }

            self.mode = next_mode;
            self.step_index = 0;
            return;
        }

        if self.step_index < self.steps.len() - 1 {
            self.step_index += 1;
            return;
        }

        // Transition to a new mode since we're at the last index of the current one.
        self.mode = match self.mode.clone() {
            CpuMode::Instruction { oam_dma_pending: true } => {
                self.steps = &*OAM_DMA_TRANSFER_STEPS;
                self.step_index = 0;
                CpuMode::OamDma
            }
            CpuMode::Instruction { oam_dma_pending: false } | CpuMode::InterruptSequence {..} | CpuMode::OamDma => {
                self.steps = &[READ_OP_CODE_STEP];
                self.step_index = 0;
                CpuMode::StartNext { oam_dma_pending: false }
            }
            /*
            CpuMode::OamDma { suspended_mode, suspended_steps, suspended_step_index } => {
                self.steps = suspended_steps;
                self.step_index = suspended_step_index;
                *suspended_mode
            }
            */

            CpuMode::Jammed => CpuMode::Jammed,
            CpuMode::StartNext {..} => panic!(),
            CpuMode::BranchTaken => todo!(),
            CpuMode::BranchOops => todo!(),
            CpuMode::Oops { suspended_steps, suspended_step_index } => {
                self.steps = suspended_steps;
                self.step_index = suspended_step_index;
                CpuMode::Instruction { oam_dma_pending: false }
            }
        };
    }
}
