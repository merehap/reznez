use std::collections::VecDeque;

use crate::cpu::cycle_action::{CycleAction, DmaTransferState};
use crate::cpu::instruction::Instruction;
use crate::memory::cpu::cpu_address::CpuAddress;

// More than enough space for a DMA transfer (513 cycles) plus an instruction.
const CAPACITY: usize = 1000;

pub struct CycleActionQueue {
    queue: VecDeque<CycleAction>,
}

impl CycleActionQueue {
    pub fn new() -> CycleActionQueue {
        CycleActionQueue {
            queue: VecDeque::with_capacity(CAPACITY),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn dequeue(&mut self) -> Option<CycleAction> {
        self.queue.pop_front()
    }

    pub fn enqueue_nop(&mut self) {
        self.queue.push_back(CycleAction::Nop);
    }

    pub fn enqueue_instruction(&mut self, instruction: Instruction) {
        for _ in 0..instruction.template.cycle_count as u8 - 1 {
            self.queue.push_back(CycleAction::Nop);
        }

        self.queue.push_back(CycleAction::Instruction(instruction));
    }

    pub fn enqueue_instruction_return(&mut self, instruction: Instruction) {
        self.queue.push_back(CycleAction::InstructionReturn(instruction));
    }

    pub fn enqueue_dma_transfer(&mut self, page: u8, current_cycle: u64) {
        let is_odd_cycle = current_cycle % 2 == 1;
        let mut current_cpu_address = CpuAddress::from_low_high(0, page);

        use DmaTransferState::*;
        self.enqueue_dma_transfer_state(WaitOnPreviousWrite);
        if is_odd_cycle {
            self.enqueue_dma_transfer_state(AlignToEven);
        }

        for _ in 0..256 {
            self.enqueue_dma_transfer_state(Read);
            self.enqueue_dma_transfer_state(Write(current_cpu_address));
            current_cpu_address.inc();
        }
    }

    pub fn enqueue_nmi(&mut self) {
        self.queue.push_back(CycleAction::Nop);
        self.queue.push_back(CycleAction::Nop);
        self.queue.push_back(CycleAction::Nmi);
    }

    fn enqueue_dma_transfer_state(&mut self, state: DmaTransferState) {
        self.queue.push_back(CycleAction::DmaTransfer(state));
    }
}
