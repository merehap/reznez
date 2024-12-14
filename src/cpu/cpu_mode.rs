use crate::apu::apu_registers::CycleParity;
use crate::cpu::step::*;
use crate::cpu::instruction::Instruction;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum CpuMode {
    Reset,
    Instruction { oam_dma_pending: bool },
    InterruptSequence,
    OamDma,
    /*
    OamDma {
        suspended_mode: Box<CpuMode>,
        suspended_steps: &'static [Step],
        suspended_step_index: usize,
    },
    */
    DmcDma,

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
    current_instruction: Option<Instruction>,
}

impl CpuModeState {
    pub fn startup() -> Self {
        Self {
            steps: RESET_STEPS,
            step_index: 0,
            mode: CpuMode::Reset,
            next_mode: None,
            current_instruction: None,
        }
    }

    pub fn jammed(&self) -> bool {
        self.mode == CpuMode::Jammed
    }

    pub fn current_step(&self) -> Step {
        self.steps[self.step_index]
    }

    pub fn current_instruction(&self) -> Option<Instruction> {
        self.current_instruction
    }

    pub fn clear_current_instruction(&mut self) {
        self.current_instruction = None;
    }

    pub fn set_next_mode(&mut self, next_mode: CpuMode) {
        assert_eq!(self.next_mode, None, "next_mode should not already be set when setting it to {next_mode:?}");
        self.next_mode = Some(next_mode);
    }

    pub fn oam_dma_pending(&mut self) {
        match self.mode {
            CpuMode::StartNext { oam_dma_pending: false } => self.set_next_mode(CpuMode::OamDma),
            // CpuMode::StartNext { oam_dma_pending: false } => self.mode = CpuMode::StartNext { oam_dma_pending: true },
            CpuMode::Instruction { oam_dma_pending: false } => self.set_next_mode(CpuMode::Instruction { oam_dma_pending: true }),
            _ => todo!(),
        }
    }

    /*
    fn oam_dma(&mut self) {
        self.mode = CpuMode::OamDma {
            suspended_mode: Box::new(self.mode.clone()),
            suspended_steps: self.steps,
            suspended_step_index: self.step_index,
        };
        self.steps = &*OAM_DMA_TRANSFER_STEPS;
        self.step_index = 0;
    }
    */

    pub fn instruction(&mut self, instruction: Instruction) {
        let oam_dma_pending = self.mode == CpuMode::Instruction { oam_dma_pending: true };
        assert_eq!(oam_dma_pending, false);

        self.current_instruction = Some(instruction);
        self.steps = instruction.steps();
        self.set_next_mode(CpuMode::Instruction { oam_dma_pending });
    }

    pub fn oops(&mut self) {
        assert_eq!(self.mode, CpuMode::Instruction { oam_dma_pending: false });
        self.set_next_mode(CpuMode::Oops {
            suspended_steps: self.steps,
            suspended_step_index: self.step_index + 1,
        });
    }

    pub fn step(&mut self, cycle_parity: CycleParity) {
        if let Some(next_mode) = self.next_mode.take() {
            match next_mode {
                CpuMode::StartNext {..} => unreachable!(),
                CpuMode::DmcDma => todo!(),
                CpuMode::Jammed => self.steps = &[],

                CpuMode::OamDma => {
                    self.steps = match cycle_parity {
                        CycleParity::Get => &*OAM_DMA_TRANSFER_STEPS,
                        CycleParity::Put => &*ALIGNED_OAM_DMA_TRANSFER_STEPS,
                    };
                }
                CpuMode::Reset => self.steps = RESET_STEPS,
                CpuMode::InterruptSequence => self.steps = BRK_STEPS,
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
            CpuMode::Reset | CpuMode::Instruction { oam_dma_pending: false } | CpuMode::InterruptSequence | CpuMode::OamDma | CpuMode::DmcDma => {
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
