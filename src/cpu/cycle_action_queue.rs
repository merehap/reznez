use std::collections::VecDeque;

use crate::cpu::cycle_action::{CycleAction, DmaTransferState, Location};
use crate::cpu::instruction::{Instruction, AccessMode, OpCode};
use crate::memory::cpu::cpu_address::CpuAddress;

// More than enough space for a DMA transfer (513 cycles) plus an instruction.
const CAPACITY: usize = 1000;

#[derive(Debug)]
pub struct CycleActionQueue {
    queue: VecDeque<(CycleAction, CycleAction)>,
}

impl CycleActionQueue {
    pub fn new() -> CycleActionQueue {
        CycleActionQueue { queue: VecDeque::with_capacity(CAPACITY) }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn dequeue(&mut self) -> Option<(CycleAction, CycleAction)> {
        self.queue.pop_front()
    }

    pub fn skip_to_front(&mut self, first: CycleAction, second: CycleAction) {
        self.queue.push_front((first, second));
    }

    pub fn enqueue_instruction_fetch(&mut self) {
        self.queue.push_back((CycleAction::FetchInstruction, CycleAction::IncrementProgramCounter));
    }

    pub fn enqueue_instruction(&mut self, instruction: Instruction) {
        use AccessMode::*;
        use OpCode::*;
        use CycleAction::*;
        use Location::*;

        let mut fallback = false;
        match (instruction.template.access_mode, instruction.template.op_code) {
            (Imp, BRK) => {
                self.prepend(&[
                    (Copy { from: ProgramCounter        , to: DataBus                }, IncrementProgramCounter),
                    (Copy { from: ProgramCounterHighByte, to: TopOfStack             }, DecrementStackPointer  ),
                    (Copy { from: ProgramCounterLowByte , to: TopOfStack             }, DecrementStackPointer  ),
                    (Copy { from: InstructionStatus     , to: TopOfStack             }, DecrementStackPointer  ),
                    (Copy { from: IrqVectorLow          , to: DataBus                }, DisableInterrupts      ),
                    (Copy { from: IrqVectorHigh         , to: ProgramCounterHighByte }, Nop                    ),
                ]);
            }
            (Imp, RTI) => {
                self.prepend(&[
                    (Copy { from: ProgramCounter, to: DataBus                }, Nop                  ),
                    (Copy { from: TopOfStack    , to: DataBus                }, IncrementStackPointer),
                    (Copy { from: TopOfStack    , to: Status                 }, IncrementStackPointer),
                    (Copy { from: TopOfStack    , to: DataBus                }, IncrementStackPointer),
                    (Copy { from: TopOfStack    , to: ProgramCounterHighByte }, Nop                  ),
                ]);
            }
            (Imp, RTS) => {
                self.prepend(&[
                    (Read                           , Nop                    ),
                    (Nop                            , IncrementStackPointer  ),
                    (ReadProgramCounterLowFromStack , IncrementStackPointer  ),
                    (ReadProgramCounterHighFromStack, Nop                    ),
                    (Nop                            , IncrementProgramCounter),
                ]);
            }
            _ => fallback = true,
        }

        if !fallback {
            return;
        }

        self.queue.push_front((CycleAction::Instruction, CycleAction::Nop));

        // Cycle 0 was the instruction fetch, cycle n - 1 is the instruction execution.
        match (instruction.template.access_mode, instruction.template.op_code) {
            (Abs, JMP) => self.queue.push_front((Nop, Nop)),
            (Abs, _code) => {
                self.prepend(&vec![
                    (Nop, Nop);
                    instruction.template.cycle_count as usize - 4
                ]);
                // TODO: Make exceptions for JSR and potentially others.
                self.prepend(&[
                    (FetchAddressLow, IncrementProgramCounter),
                    (FetchAddressHigh, IncrementProgramCounter),
                ]);
            }
            _ => {
                self.prepend(&vec![
                    (Nop, Nop);
                    instruction.template.cycle_count as usize - 2
                ]);
            }
        }
    }

    pub fn enqueue_nmi(&mut self) {
        for _ in 0..6 {
            self.queue.push_back((CycleAction::Nop, CycleAction::Nop));
        }

        self.queue.push_back((CycleAction::Nmi, CycleAction::Nop));
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
            self.enqueue_dma_transfer_state(Read(current_cpu_address));
            self.enqueue_dma_transfer_state(Write);
            current_cpu_address.inc();
        }
    }

    fn enqueue_dma_transfer_state(&mut self, state: DmaTransferState) {
        self.queue.push_back((CycleAction::DmaTransfer(state), CycleAction::Nop));
    }

    fn prepend(&mut self, actions: &[(CycleAction, CycleAction)]) {
        for &action in actions.iter().rev() {
            self.queue.push_front(action);
        }
    }
}
