use std::collections::VecDeque;

use crate::cpu::cycle_action::{CycleAction, From, To};
use crate::cpu::instruction::{Instruction, AccessMode, OpCode};
use crate::memory::mapper::CpuAddress;

// More than enough space for a DMA transfer (513 cycles) plus an instruction.
const CAPACITY: usize = 1000;

#[derive(Debug)]
pub struct CycleActionQueue {
    queue: VecDeque<(From, To, CycleAction)>,
}

impl CycleActionQueue {
    pub fn new() -> CycleActionQueue {
        CycleActionQueue { queue: VecDeque::with_capacity(CAPACITY) }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn dequeue(&mut self) -> Option<(From, To, CycleAction)> {
        self.queue.pop_front()
    }

    pub fn skip_to_front(&mut self, copy_from: From, to: To, action: CycleAction) {
        self.queue.push_front((copy_from, to, action));
    }

    pub fn enqueue_instruction_fetch(&mut self) {
        use CycleAction::*;
        self.queue.push_back((From::ProgramCounter, To::Instruction, IncrementProgramCounter));
    }

    pub fn enqueue_instruction(&mut self, instruction: Instruction) {
        use AccessMode::*;
        use OpCode::*;
        use CycleAction::*;

        let mut fallback = false;
        match (instruction.template.access_mode, instruction.template.op_code) {
            (Imp, BRK) => {
                self.prepend(&[
                    (From::ProgramCounter        , To::DataBus               , IncrementProgramCounter),
                    (From::ProgramCounterHighByte, To::TopOfStack            , DecrementStackPointer  ),
                    (From::ProgramCounterLowByte , To::TopOfStack            , DecrementStackPointer  ),
                    (From::StatusForInstruction  , To::TopOfStack            , DecrementStackPointer  ),
                    // Copy the new ProgramCounterLowByte to the data bus.
                    (From::IRQ_VECTOR_LOW        , To::DataBus               , DisableInterrupts      ),
                    (From::IRQ_VECTOR_HIGH       , To::ProgramCounterHighByte, Nop                    ),
                ]);
            }
            (Imp, RTI) => {
                self.prepend(&[
                    (From::ProgramCounter, To::DataBus               , Nop                  ),
                    (From::TopOfStack    , To::DataBus               , IncrementStackPointer),
                    (From::TopOfStack    , To::Status                , IncrementStackPointer),
                    (From::TopOfStack    , To::DataBus               , IncrementStackPointer),
                    (From::TopOfStack    , To::ProgramCounterHighByte, Nop                  ),
                ]);
            }
            (Imp, RTS) => {
                self.prepend(&[
                    (From::ProgramCounter, To::DataBus               , Nop                    ),
                    (From::TopOfStack    , To::DataBus               , IncrementStackPointer  ),
                    (From::TopOfStack    , To::DataBus               , IncrementStackPointer  ),
                    (From::TopOfStack    , To::ProgramCounterHighByte, Nop                    ),
                    // TODO: Make sure this dummy read is correct.
                    (From::ProgramCounter, To::DataBus               , IncrementProgramCounter),
                ]);
            }
            (Imp, PHA) => {
                self.prepend(&[
                    (From::ProgramCounter, To::DataBus   , Nop                  ),
                    (From::Accumulator   , To::TopOfStack, DecrementStackPointer),
                ]);
            }
            (Imp, PHP) => {
                self.prepend(&[
                    (From::ProgramCounter      , To::DataBus   , Nop                  ),
                    (From::StatusForInstruction, To::TopOfStack, DecrementStackPointer),
                ]);
            }
            _ => fallback = true,
        }

        if !fallback {
            return;
        }

        self.queue.push_front((From::DataBus, To::DataBus, Instruction));

        // Cycle 0 was the instruction fetch, cycle n - 1 is the instruction execution.
        match (instruction.template.access_mode, instruction.template.op_code) {
            (Abs, JMP) => self.queue.push_front((From::DataBus, To::DataBus, Nop)),
            (Abs, _code) => {
                self.prepend(&vec![
                    (From::DataBus, To::DataBus, Nop);
                    instruction.template.cycle_count as usize - 4
                ]);
                // TODO: Make exceptions for JSR and potentially others.
                self.prepend(&[
                    (From::ProgramCounter, To::DataBus               , IncrementProgramCounter),
                    (From::ProgramCounter, To::PendingAddressHighByte, IncrementProgramCounter),
                ]);
            }
            _ => {
                self.prepend(&vec![
                    (From::DataBus, To::DataBus, Nop);
                    instruction.template.cycle_count as usize - 2
                ]);
            }
        }
    }

    pub fn enqueue_nmi(&mut self) {
        use CycleAction::*;
        self.append(&[
            // Not sure what NMI does during the first cycle, so just put NOPs here.
            (From::DataBus               , To::DataBus               , Nop                  ),
            (From::ProgramCounter        , To::DataBus               , Nop                  ),
            (From::ProgramCounterHighByte, To::TopOfStack            , DecrementStackPointer),
            (From::ProgramCounterLowByte , To::TopOfStack            , DecrementStackPointer),
            (From::StatusForInterrupt    , To::TopOfStack            , DecrementStackPointer),
            // Copy the new ProgramCounterLowByte to the data bus.
            (From::NMI_VECTOR_LOW        , To::DataBus               , Nop                  ),
            (From::NMI_VECTOR_HIGH       , To::ProgramCounterHighByte, Nop                  ),
        ]);
    }

    // Note: the values of the address bus might not be correct for some cycles.
    pub fn enqueue_dma_transfer(&mut self, port: u8, current_cycle: u64) {
        use CycleAction::*;

        let transfer_start_address = CpuAddress::from_low_high(0x00, port);
        // Unclear this is the correct timing. Might not matter even if it's wrong.
        self.queue.push_back((From::DataBus, To::DataBus, SetAddressBus(transfer_start_address)));

        let is_odd_cycle = current_cycle % 2 == 1;
        if is_odd_cycle {
            self.queue.push_back((From::DataBus, To::DataBus, Nop));
        }

        for _ in 0..256 {
            self.queue.push_back((From::AddressBus, To::DataBus, Nop                ));
            self.queue.push_back((From::DataBus   , To::OamData, IncrementAddressBus));
        }
    }

    fn append(&mut self, actions: &[(From, To, CycleAction)]) {
        for &action in actions.iter() {
            self.queue.push_back(action);
        }
    }

    fn prepend(&mut self, actions: &[(From, To, CycleAction)]) {
        for &action in actions.iter().rev() {
            self.queue.push_front(action);
        }
    }
}
