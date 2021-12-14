use crate::cpu::address::Address;

pub struct DmaTransfer {
    current_address: Address,
    remaining_byte_count: u16,
    should_fix_cycle_alignment: bool,
    next_state: DmaTransferState,
}

impl DmaTransfer {
    pub fn new(
        page: u8,
        size: u16,
        current_cycle: u64,
        ) -> DmaTransfer {

        DmaTransfer {
            current_address: Address::from_low_high(0, page),
            remaining_byte_count: size,
            should_fix_cycle_alignment: current_cycle % 2 == 1,
            next_state: DmaTransferState::WaitOnPreviousWrite,
        }
    }

    pub fn inactive() -> DmaTransfer {
        DmaTransfer {
            current_address: Address::new(0),
            remaining_byte_count: 0,
            should_fix_cycle_alignment: false,
            next_state: DmaTransferState::Finished,
        }
    }

    // TODO: Determine if the full 513/514 cycles must occur even if we
    // aren't transfering the maximum amount of OAM (256 bytes).
    pub fn next(&mut self) -> DmaTransferState {
        let current_state = self.next_state;

        use DmaTransferState::*;
        self.next_state = match current_state {
            WaitOnPreviousWrite if self.should_fix_cycle_alignment =>
                AlignToEven,
            WaitOnPreviousWrite | AlignToEven =>
                Read,
            Read =>
                Write(self.current_address),
            Write(_) if self.remaining_byte_count == 0 =>
                Finished,
            Write(_) => {
                self.current_address.inc();
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
    Write(Address),
    Finished,
}
