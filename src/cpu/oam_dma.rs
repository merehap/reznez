use crate::apu::apu_registers::CycleParity;
use crate::memory::cpu::cpu_address::CpuAddress;

pub struct OamDma {
    state: OamDmaState,
    address: CpuAddress,
}

impl OamDma {
    pub const IDLE: Self = Self {
        state: OamDmaState::Idle,
        address: CpuAddress::ZERO,
    };

    pub fn dma_pending(&self) -> bool {
        self.state == OamDmaState::TryHalt
    }

    pub fn address(&self) -> CpuAddress {
        self.address
    }

    pub fn increment_address(&mut self) {
        self.address.inc();
    }

    pub fn prepare_to_start(&mut self, page: u8) {
        self.state = OamDmaState::TryHalt;
        self.address = CpuAddress::from_low_high(0, page);
    }

    pub fn step(&mut self, is_read_step: bool, parity: CycleParity, block_memory_access: bool) -> OamDmaAction {
        self.state.step(is_read_step, parity, block_memory_access)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum OamDmaState {
    Idle,
    TryHalt,
    TryRead(u8),
    Write(u8),
}

impl OamDmaState {
    fn step(&mut self, is_read_step: bool, parity: CycleParity, block_memory_access: bool) -> OamDmaAction {
        // DMA can't halt until the CPU is reading.
        if *self == Self::TryHalt && !is_read_step {
            return OamDmaAction::DoNothing;
        }

        // DMA can't read on PUT steps.
        // TODO: This should fail on PUTs, not GETS, but somehow the parity tracking is off.
        if matches!(*self, Self::TryRead(_)) && parity == CycleParity::Get {
            return OamDmaAction::Align;
        }

        if block_memory_access && matches!(*self, Self::TryRead(_) | Self::Write(_)) {
            return OamDmaAction::DoNothing;
        }

        let (step_result, next_state) = match *self {
            Self::Idle =>
                (OamDmaAction::DoNothing, Self::Idle),
            Self::TryHalt =>
                (OamDmaAction::Halt, Self::TryRead(0)),
            Self::TryRead(n) =>
                (OamDmaAction::Read, Self::Write(n)),
            Self::Write(n@0..=254) =>
                (OamDmaAction::Write, Self::TryRead(n + 1)),
            Self::Write(255) =>
                (OamDmaAction::Write, Self::Idle),
        };

        *self = next_state;
        step_result
    }
}


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum OamDmaAction {
    DoNothing,
    Halt,
    Align,
    Read,
    Write,
}