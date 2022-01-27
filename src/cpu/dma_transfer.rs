use crate::memory::cpu_address::CpuAddress;

pub struct DmaTransfer {
    current_cpu_address: CpuAddress,
    current_oam_address: u8,
    remaining_byte_count: u16,
    should_fix_cycle_alignment: bool,
    next_state: DmaTransferState,
}

impl DmaTransfer {
    pub fn new(
        page: u8,
        oam_start_address: u8,
        current_cycle: u64,
    ) -> DmaTransfer {
        DmaTransfer {
            current_cpu_address: CpuAddress::from_low_high(0, page),
            current_oam_address: oam_start_address,
            remaining_byte_count: 256,
            should_fix_cycle_alignment: current_cycle % 2 == 1,
            next_state: DmaTransferState::WaitOnPreviousWrite,
        }
    }

    pub fn inactive() -> DmaTransfer {
        DmaTransfer {
            current_cpu_address: CpuAddress::new(0),
            current_oam_address: 0,
            remaining_byte_count: 0,
            should_fix_cycle_alignment: false,
            next_state: DmaTransferState::Finished,
        }
    }

    pub fn step(&mut self) -> DmaTransferState {
        let current_state = self.next_state;

        use DmaTransferState::*;
        self.next_state = match current_state {
            WaitOnPreviousWrite if self.should_fix_cycle_alignment =>
                AlignToEven,
            WaitOnPreviousWrite | AlignToEven =>
                Read,
            Read =>
                Write(self.current_cpu_address, self.current_oam_address),
            Write(_, _) if self.remaining_byte_count <= 1 =>
                Finished,
            Write(_, _) => {
                self.current_cpu_address.inc();
                self.current_oam_address = self.current_oam_address.wrapping_add(1);
                self.remaining_byte_count -= 1;
                Read
            },
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
    Write(CpuAddress, u8),
    Finished,
}
