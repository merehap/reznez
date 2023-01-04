use std::collections::VecDeque;

use crate::cpu::step::*;
use crate::cpu::instruction::{Instruction, AccessMode, OpCode};

// More than enough space for a DMA transfer (513 cycles) plus an instruction.
const CAPACITY: usize = 1000;

#[derive(Debug)]
pub struct CycleActionQueue {
    queue: VecDeque<Step>,
}

impl CycleActionQueue {
    pub fn new() -> CycleActionQueue {
        CycleActionQueue { queue: VecDeque::with_capacity(CAPACITY) }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn dequeue(&mut self) -> Option<Step> {
        self.queue.pop_front()
    }

    pub fn skip_to_front(&mut self, step: Step) {
        self.queue.push_front(step);
    }

    pub fn enqueue_instruction_fetch(&mut self) {
        self.queue.push_back(READ_INSTRUCTION_STEP);
    }

    pub fn enqueue_instruction(&mut self, instruction: Instruction) {
        use AccessMode::*;
        use OpCode::*;

        let mut fallback = false;
        match (instruction.template.access_mode, instruction.template.op_code) {
            (Imp, BRK) => self.prepend(BRK_STEPS),
            (Imp, RTI) => self.prepend(RTI_STEPS),
            (Imp, RTS) => self.prepend(RTS_STEPS),
            (Imp, PHA) => self.prepend(PHA_STEPS),
            (Imp, PHP) => self.prepend(PHP_STEPS),
            (Imp, PLA) => self.prepend(PLA_STEPS),
            (Imp, PLP) => self.prepend(PLP_STEPS),
            (Abs, JSR) => self.prepend(JSR_STEPS),
            (Abs, JMP) => self.prepend(JMP_ABS_STEPS),
            _ => fallback = true,
        }

        if !fallback {
            return;
        }

        self.queue.push_front(FULL_INSTRUCTION_STEP);

        // Cycle 0 was the instruction fetch, cycle n - 1 is the instruction execution.
        match (instruction.template.access_mode, instruction.template.op_code) {
            (Abs, JMP) => self.queue.push_front(NOP_STEP),
            (Abs, _code) => {
                self.prepend(&vec![NOP_STEP; instruction.template.cycle_count as usize - 4]);
                // TODO: Make exceptions for JSR and potentially others.
                self.prepend(&[
                    PENDING_ADDRESS_LOW_BYTE_STEP,
                    PENDING_ADDRESS_HIGH_BYTE_STEP,
                ]);
            }
            _ => {
                self.prepend(&vec![NOP_STEP; instruction.template.cycle_count as usize - 2]);
            }
        }
    }

    pub fn enqueue_nmi(&mut self) {
        self.append(NMI_STEPS);
    }

    // Note: the values of the address bus might not be correct for some cycles.
    pub fn enqueue_dma_transfer(&mut self, current_cycle: u64) {
        // Unclear this is the correct timing. Might not matter even if it's wrong.
        self.queue.push_back(OAM_DMA_START_TRANSFER_STEP);

        let is_odd_cycle = current_cycle % 2 == 1;
        if is_odd_cycle {
            self.queue.push_back(NOP_STEP);
        }

        for _ in 0..256 {
            self.queue.push_back(OAM_DMA_READ_STEP);
            self.queue.push_back(OAM_DMA_WRITE_STEP);
        }
    }

    fn append(&mut self, steps: &[Step]) {
        for step in steps.iter() {
            self.queue.push_back(step.clone());
        }
    }

    fn prepend(&mut self, steps: &[Step]) {
        for step in steps.iter().rev() {
            self.queue.push_front(step.clone());
        }
    }
}
