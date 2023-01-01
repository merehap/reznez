use std::collections::VecDeque;

use crate::cpu::cycle_action::{CycleAction, Location};
use crate::cpu::instruction::{Instruction, AccessMode, OpCode};
use crate::memory::mapper::CpuAddress;

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
        use CycleAction::*;
        self.queue.push_back(
            (
                Copy { from: Location::ProgramCounter, to: Location::Instruction},
                IncrementProgramCounter,
            )
        );
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
                    (Copy { from: ProgramCounter, to: DataBus                }, Nop                    ),
                    (Copy { from: TopOfStack    , to: DataBus                }, IncrementStackPointer  ),
                    (Copy { from: TopOfStack    , to: DataBus                }, IncrementStackPointer  ),
                    (Copy { from: TopOfStack    , to: ProgramCounterHighByte }, Nop                    ),
                    // TODO: Make sure this dummy read is correct.
                    (Copy { from: ProgramCounter, to: DataBus                }, IncrementProgramCounter),
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
                    (Copy { from: ProgramCounter, to: DataBus                }, IncrementProgramCounter),
                    (Copy { from: ProgramCounter, to: PendingAddressHighByte }, IncrementProgramCounter),
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

    // Note: the values of the address bus might not be correct for some cycles.
    pub fn enqueue_dma_transfer(&mut self, port: u8, current_cycle: u64) {
        use CycleAction::*;
        use Location::*;

        let transfer_start_address = CpuAddress::from_low_high(0x00, port);
        // Unclear this is the correct timing. Might not matter even if it's wrong.
        self.queue.push_back((Nop, SetAddressBus(transfer_start_address)));

        let is_odd_cycle = current_cycle % 2 == 1;
        if is_odd_cycle {
            self.queue.push_back((Nop, Nop));
        }

        for _ in 0..256 {
            self.queue.push_back((Copy { from: AddressBus, to: DataBus }, Nop                ));
            self.queue.push_back((Copy { from: DataBus   , to: OamData }, IncrementAddressBus));
        }
    }

    fn prepend(&mut self, actions: &[(CycleAction, CycleAction)]) {
        for &action in actions.iter().rev() {
            self.queue.push_front(action);
        }
    }
}
