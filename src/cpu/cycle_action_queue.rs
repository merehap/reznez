use std::collections::VecDeque;

use crate::cpu::step::*;
use crate::cpu::instruction::Instruction;

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
        self.append(READ_AND_INTERPRET_OP_CODE_STEPS);
    }

    pub fn enqueue_instruction(&mut self, instruction: Instruction) {
        let code_point = usize::try_from(instruction.template.code_point).unwrap();
        self.prepend(INSTRUCTIONS[code_point].steps());
    }

    pub fn enqueue_nmi(&mut self) {
        self.append(NMI_STEPS);
    }

    // Note: the values of the address bus might not be correct for some cycles.
    pub fn enqueue_dma_transfer(&mut self, current_cycle: u64) {
        // Unclear this is the correct timing. Might not matter even if it's wrong.
        let is_odd_cycle = current_cycle % 2 == 1;
        if is_odd_cycle {
            self.queue.push_back(NOP_STEP);
        }

        self.append(&*OAM_DMA_TRANSFER_STEPS);
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
