use crate::memory::mapper::CpuAddress;
use crate::util::integer::U7;

const NTSC_PERIODS: [u16; 16] =
    [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54];

pub struct Dmc {
    enabled: bool,
    muted: bool,

    irq_enabled: bool,
    pub(super) irq_pending: bool,
    dma_pending_address: Option<CpuAddress>,

    should_loop: bool,
    volume: U7,

    period: u16,
    cycles_remaining: u16,

    sample_start_address: CpuAddress,
    sample_address: CpuAddress,
    sample_buffer: Option<u8>,
    sample_shifter: u8,
    sample_length: u16,
    sample_bytes_remaining: u16,

    // Values from 0 to 8.
    bits_remaining: u8,
}

impl Dmc {
    pub fn write_control_byte(&mut self, value: u8) {
        self.irq_enabled = (value & 0b1000_0000) != 0;
        self.should_loop = (value & 0b0100_0000) != 0;
        self.period = NTSC_PERIODS[(value & 0b0000_1111) as usize];
        if !self.irq_enabled {
            self.irq_pending = false;
        }
    }

    pub fn write_volume(&mut self, value: u8) {
        self.volume = (value & 0b0111_1111).into();
    }

    pub fn write_sample_start_address(&mut self, value: u8) {
        self.sample_start_address = CpuAddress::new(0xC000 | (u16::from(value) << 6));
    }

    pub fn write_sample_length(&mut self, value: u8) {
        self.sample_length = (u16::from(value) << 4) | 1;
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) {
        self.irq_pending = false;

        self.enabled = enabled;
        if !self.enabled {
            self.sample_bytes_remaining = 0;
        } else {
            if self.sample_bytes_remaining == 0 {
                self.sample_bytes_remaining = self.sample_length;
                self.sample_address = self.sample_start_address;
            }

            if self.sample_buffer.is_none() && self.sample_bytes_remaining > 0 {
                self.dma_pending_address = Some(self.sample_address);
            }
        }
    }

    pub(super) fn active(&self) -> bool {
        self.sample_bytes_remaining > 0
    }

    pub fn dma_pending(&self) -> bool {
        self.dma_pending_address.is_some()
    }

    pub fn take_dma_pending_address(&mut self) -> Option<CpuAddress> {
        self.dma_pending_address.take()
    }

    pub fn set_sample_buffer(&mut self, value: u8) {
        if self.sample_bytes_remaining > 0 {
            self.sample_buffer = Some(value);
            self.sample_address.inc();
            if self.sample_address == CpuAddress::ZERO {
                self.sample_address = CpuAddress::new(0x8000);
            }

            self.sample_bytes_remaining -= 1;
            if self.sample_bytes_remaining == 0 {
                if self.should_loop {
                    self.sample_bytes_remaining = self.sample_length;
                    self.sample_address = self.sample_start_address;
                } else if self.irq_enabled {
                    self.irq_pending = true;
                }
            }
        }
    }

    pub(super) fn execute_put_cycle(&mut self) {
        if self.cycles_remaining >= 2 {
            self.cycles_remaining -= 2;
            return;
        }

        self.cycles_remaining = self.period;
        self.bits_remaining = self.bits_remaining.saturating_sub(1);
        if self.bits_remaining > 0 {
            return;
        }

        self.bits_remaining = 8;
        self.muted = self.sample_buffer.is_none();
        if let Some(sample) = self.sample_buffer.take() {
            self.sample_shifter = sample;
            if self.sample_bytes_remaining > 0 {
                self.dma_pending_address = Some(self.sample_address);
            }
        }
    }

    pub(super) fn sample_volume(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            f32::from(self.volume.to_u8())
        }
    }
}

impl Default for Dmc {
    fn default() -> Self {
        Dmc {
            enabled: Default::default(),
            muted: Default::default(),
            irq_enabled: Default::default(),
            irq_pending: Default::default(),
            dma_pending_address: None,
            should_loop: Default::default(),
            volume: U7::default(),
            period: Default::default(),
            cycles_remaining: 0,
            sample_start_address: CpuAddress::new(0xC000),
            sample_address: CpuAddress::new(0xC000),
            sample_buffer: None,
            sample_shifter: 0,
            sample_length: 0,
            sample_bytes_remaining: 0,

            bits_remaining: 0,
        }
    }
}
