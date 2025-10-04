use crate::cpu::dmc_dma::DmcDma;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::cpu_pinout::CpuPinout;
use crate::util::integer::U7;

const NTSC_PERIODS: [u16; 16] =
    [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54];

pub struct Dmc {
    muted: bool,

    irq_enabled: bool,

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
    // 0x4010
    pub fn write_control_byte(&mut self, cpu_pinout: &mut CpuPinout, value: u8) {
        self.irq_enabled = (value & 0b1000_0000) != 0;
        self.should_loop = (value & 0b0100_0000) != 0;
        self.period = NTSC_PERIODS[(value & 0b0000_1111) as usize] - 1;
        if !self.irq_enabled {
            cpu_pinout.acknowledge_dmc_irq();
        }
    }

    // 0x4011
    pub fn write_volume(&mut self, value: u8) {
        self.volume = (value & 0b0111_1111).into();
    }

    // 0x4012
    pub fn write_sample_start_address(&mut self, value: u8) {
        self.sample_start_address = CpuAddress::new(0xC000 | (u16::from(value) << 6));
    }

    // 0x4013
    pub fn write_sample_length(&mut self, value: u8) {
        self.sample_length = (u16::from(value) << 4) | 1;
        //println!("Setting sample length to {}", self.sample_length);
    }

    // 0x4015
    pub(super) fn set_enabled(&mut self, cpu_pinout: &mut CpuPinout, dma: &mut DmcDma, enabled: bool) {
        cpu_pinout.acknowledge_dmc_irq();

        if !enabled {
            self.sample_bytes_remaining = 0;
        } else if self.sample_bytes_remaining == 0 {
            //println!("Reloading sample bytes remaining from 0 to {}", self.sample_length);
            self.sample_bytes_remaining = self.sample_length;
            self.sample_address = self.sample_start_address;

            if self.sample_buffer.is_none() {
                //println!("Attempting to load sample buffer soon.");
                dma.start_load();
            }
        }
    }

    // Upon RESET
    pub(super) fn disable(&mut self, cpu_pinout: &mut CpuPinout) {
        cpu_pinout.acknowledge_dmc_irq();
        self.sample_bytes_remaining = 0;
    }

    pub fn set_sample_buffer(&mut self, cpu_pinout: &mut CpuPinout, value: u8) {
        //println!("Checking if sample buffer should be loaded.");
        if self.sample_bytes_remaining > 0 {
            //println!("Loading sample buffer.");
            self.sample_buffer = Some(value);
            self.sample_address.inc();
            if self.sample_address == CpuAddress::ZERO {
                self.sample_address = CpuAddress::new(0x8000);
            }

            self.sample_bytes_remaining -= 1;
            if self.sample_bytes_remaining == 0 {
                //println!("No sample bytes remaining. Should loop? {}", self.should_loop);
                if self.should_loop {
                    self.sample_bytes_remaining = self.sample_length;
                    self.sample_address = self.sample_start_address;
                } else if self.irq_enabled {
                    cpu_pinout.assert_dmc_irq();
                }
            }
        }
    }

    pub(super) fn execute_put_cycle(&mut self, dmc_dma: &mut DmcDma) {
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
            //println!("Taking sample buffer.");
            self.sample_shifter = sample;
            if self.sample_bytes_remaining > 0 {
                //println!("Attempting to RELOAD sample buffer soon.");
                dmc_dma.start_reload();
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

    pub(super) fn active(&self) -> bool {
        self.sample_bytes_remaining > 0
    }

    pub fn dma_sample_address(&self) -> CpuAddress {
        self.sample_address
    }
}

impl Default for Dmc {
    fn default() -> Self {
        Dmc {
            muted: true,
            irq_enabled: false,
            should_loop: false,
            volume: U7::default(),
            period: NTSC_PERIODS[0] - 1,
            cycles_remaining: NTSC_PERIODS[0] - 1,
            sample_start_address: CpuAddress::new(0xC000),
            sample_address: CpuAddress::new(0xC000),
            sample_buffer: None,
            sample_shifter: 0,
            sample_length: 1,
            sample_bytes_remaining: 0,

            bits_remaining: 8,
        }
    }
}
