use crate::memory::cpu::cpu_address::CpuAddress;

pub struct DmaTransfer {
    current_cpu_address: CpuAddress,
    should_fix_cycle_alignment: bool,
    next_state: DmaTransferState,
}

impl DmaTransfer {
    pub fn new(page: u8, current_cycle: u64) -> DmaTransfer {
        DmaTransfer {
            current_cpu_address: CpuAddress::from_low_high(0, page),
            should_fix_cycle_alignment: current_cycle % 2 == 1,
            next_state: DmaTransferState::WaitOnPreviousWrite,
        }
    }

    pub fn inactive() -> DmaTransfer {
        DmaTransfer {
            current_cpu_address: CpuAddress::new(0),
            should_fix_cycle_alignment: false,
            next_state: DmaTransferState::Finished,
        }
    }

    pub fn step(&mut self) -> DmaTransferState {
        let current_state = self.next_state;

        use DmaTransferState::*;
        self.next_state = match current_state {
            WaitOnPreviousWrite if self.should_fix_cycle_alignment => AlignToEven,
            WaitOnPreviousWrite | AlignToEven => Read,
            Read => Write(self.current_cpu_address),
            Write(_) if self.current_cpu_address.is_end_of_page() => Finished,
            Write(_) => {self.current_cpu_address.inc(); Read},
            Finished => Finished,
        };

        current_state
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DmaTransferState {
    WaitOnPreviousWrite,
    AlignToEven,
    Read,
    Write(CpuAddress),
    Finished,
}
