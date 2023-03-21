use std::collections::VecDeque;

use crate::cpu::instruction::INSTRUCTIONS;
use crate::cpu::step::*;

// More than enough space for a DMA transfer (513 cycles) plus an instruction.
const CAPACITY: usize = 1000;

#[derive(Debug)]
pub struct StepQueue {
    queue: VecDeque<Step>,
}

impl StepQueue {
    pub fn new() -> StepQueue {
        let mut queue = StepQueue { queue: VecDeque::with_capacity(CAPACITY) };
        queue.append(START_STEPS);
        queue
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn peek(&self) -> Option<Step> {
        self.queue.front().copied()
    }

    pub fn dequeue(&mut self) -> Option<Step> {
        self.queue.pop_front()
    }

    pub fn skip_to_front(&mut self, step: Step) {
        self.queue.push_front(step);
    }

    pub fn enqueue_op_code_read(&mut self) {
        self.queue.push_back(READ_OP_CODE_STEP);
    }

    pub fn enqueue_op_code_interpret(&mut self) {
        self.queue.push_back(INTERPRET_OP_CODE_STEP);
    }

    pub fn enqueue_instruction(&mut self, code_point: u8) {
        let code_point = usize::from(code_point);
        self.prepend(INSTRUCTIONS[code_point].steps());
    }

    pub fn enqueue_nmi(&mut self) {
        self.append(NMI_STEPS);
    }

    pub fn enqueue_irq(&mut self) {
        self.append(IRQ_STEPS);
    }

    // Note: the values of the address bus might not be correct for some cycles.
    pub fn enqueue_dma_transfer(&mut self, current_cycle: i64) {
        // TODO: Improve accuracy by following this: https://www.nesdev.org/wiki/DMA#OAM_DMA
        let is_odd_cycle = current_cycle % 2 == 1;
        if is_odd_cycle {
            self.queue.push_back(ADDRESS_BUS_READ_STEP);
        }

        self.append(&*OAM_DMA_TRANSFER_STEPS);
    }

    fn append(&mut self, steps: &[Step]) {
        for &step in steps.iter() {
            self.queue.push_back(step);
        }
    }

    fn prepend(&mut self, steps: &[Step]) {
        for &step in steps.iter().rev() {
            self.queue.push_front(step);
        }
    }
}
