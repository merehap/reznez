use splitbits::{combinebits, splitbits, splitbits_named_ux};
use ux::u7;

use crate::apu::apu_registers::CycleParity;
use crate::cpu::dmc_dma::DmcDma;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::cpu_pinout::CpuPinout;

const NTSC_PERIODS: [u16; 16] =
    [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106,  84,  72,  54];

pub struct Dmc {
    // TODO: The wiki claims there is an irq_status flag independent of frame_irq_asserted in CpuPinout.
    // But there seem to be no tests to verify the behavior of these two flags in relationship to each other,
    // so currently no separate flag is stored here.
    irq_enabled: bool,

    should_loop: bool,

    period: u16,
    cycles_remaining: u16,

    sample_start_address: CpuAddress,
    sample_address: CpuAddress,
    sample_buffer: Option<u8>,

    output_unit: OutputUnit,
}

impl Dmc {
    // 0x4010
    pub fn write_control_byte(&mut self, cpu_pinout: &mut CpuPinout) {
        let fields = splitbits!(cpu_pinout.data_bus, "il..pppp");
        self.irq_enabled = fields.i;
        self.should_loop = fields.l;
        self.period = NTSC_PERIODS[fields.p as usize] - 1;
        if !self.irq_enabled {
            cpu_pinout.acknowledge_dmc_irq();
        }
    }

    // 0x4011
    pub fn write_volume(&mut self, value: u8) {
        self.output_unit.volume = splitbits_named_ux!(value, ".vvvvvvv");
    }

    // 0x4012
    pub fn write_sample_start_address(&mut self, addr: u8) {
        // Minimum: 0xC000, Maximum: 0xFFC0
        self.sample_start_address = CpuAddress::new(combinebits!(addr, "11aa aaaa aa00 0000"));
    }

    // 0x4015
    pub(super) fn set_enabled(&mut self, cpu_pinout: &mut CpuPinout, dma: &mut DmcDma, cycle_parity: CycleParity, enabled: bool) {
        cpu_pinout.acknowledge_dmc_irq();

        if !enabled {
            dma.clear_sample_bytes_remaining();
        } else if !dma.sample_bytes_remain() {
            //println!("Reloading sample bytes remaining from 0 to {}", self.sample_length);
            dma.reload_sample_bytes_remaining();
            self.sample_address = self.sample_start_address;

            if self.sample_buffer.is_none() {
                //println!("Attempting to load sample buffer soon.");
                dma.start_load(cycle_parity);
            }
        }
    }

    pub(super) fn tick(&mut self, dmc_dma: &mut DmcDma) {
        // If we don't early return here, then we must be on a PUT cycle since all NTSC_PERIODs are even.
        if self.cycles_remaining >= 1 {
            self.cycles_remaining -= 1;
            return;
        }

        self.cycles_remaining = self.period;
        self.output_unit.bits_remaining = self.output_unit.bits_remaining.saturating_sub(1);
        if self.output_unit.bits_remaining > 0 {
            return;
        }

        self.output_unit.bits_remaining = 8;
        self.output_unit.silenced = self.sample_buffer.is_none();
        if let Some(sample) = self.sample_buffer.take() {
            //println!("Taking sample buffer.");
            self.output_unit.sample_shifter = sample;
            if dmc_dma.sample_bytes_remain() {
                //println!("Attempting to RELOAD sample buffer soon.");
                dmc_dma.start_reload();
            }
        }
    }

    // Called upon the completion of a DMC DMA (Load OR Reload).
    pub fn set_sample_buffer(&mut self, cpu_pinout: &mut CpuPinout, dma: &mut DmcDma, value: u8) {
        //println!("Checking if sample buffer should be loaded.");
        if dma.sample_bytes_remain() {
            //println!("Loading sample buffer.");
            self.sample_buffer = Some(value);
            self.sample_address.inc();
            if self.sample_address == CpuAddress::ZERO {
                self.sample_address = CpuAddress::new(0x8000);
            }

            dma.decrement_sample_bytes_remaining();
            if !dma.sample_bytes_remain() {
                //println!("No sample bytes remaining. Should loop? {}", self.should_loop);
                if self.should_loop {
                    dma.reload_sample_bytes_remaining();
                    self.sample_address = self.sample_start_address;
                } else if self.irq_enabled {
                    cpu_pinout.assert_dmc_irq();
                }
            }
        }
    }

    pub(super) fn sample_volume(&self) -> f32 {
        if self.output_unit.silenced {
            0.0
        } else {
            f32::from(u8::from(self.output_unit.volume))
        }
    }

    pub fn dma_sample_address(&self) -> CpuAddress {
        self.sample_address
    }
}

impl Default for Dmc {
    fn default() -> Self {
        Dmc {
            irq_enabled: false,
            should_loop: false,
            period: NTSC_PERIODS[0] - 1,
            cycles_remaining: NTSC_PERIODS[0] - 1,
            sample_start_address: CpuAddress::new(0xC000),
            sample_address: CpuAddress::new(0xC000),
            sample_buffer: None,
            output_unit: OutputUnit::default(),
        }
    }
}

struct OutputUnit {
    // Values from 0 to 8.
    bits_remaining: u8,
    sample_shifter: u8,
    volume: u7,
    silenced: bool,
}

impl Default for OutputUnit {
    fn default() -> Self {
        Self {
            silenced: true,
            volume: u7::default(),
            sample_shifter: 0,
            bits_remaining: 8,
        }
    }
}