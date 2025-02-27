use crate::apu::apu_registers::CycleParity;
use crate::cpu::step::Step;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum OamDmaStage {
    Wait,
    TryHalt,
    TryRead(u8),
    Write(u8),
}

impl OamDmaStage {
    pub const WAIT: Self = Self::Wait;

    pub fn try_halt(&mut self) {
        *self = OamDmaStage::TryHalt;
    }

    pub fn step(&mut self, cpu_step: Step, parity: CycleParity) -> OamDmaAction {
        // DMA can't halt until the CPU is reading.
        if *self == Self::TryHalt && !cpu_step.is_read() {
            return OamDmaAction::DoNothing;
        }

        // DMA can't read on PUT steps.
        // TODO: This should fail on PUTs, not GETS, but somehow the parity tracking is off.
        if matches!(*self, Self::TryRead(_)) && parity == CycleParity::Get {
            return OamDmaAction::Align;
        }

        let (step_result, next_stage) = match *self {
            Self::Wait =>
                (OamDmaAction::DoNothing, Self::Wait),
            Self::TryHalt =>
                (OamDmaAction::Halt, Self::TryRead(0)),
            Self::TryRead(n) =>
                (OamDmaAction::Read, Self::Write(n)),
            Self::Write(n@0..=254) =>
                (OamDmaAction::Write, Self::TryRead(n + 1)),
            Self::Write(255) =>
                (OamDmaAction::Write, Self::Wait),
        };

        *self = next_stage;
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