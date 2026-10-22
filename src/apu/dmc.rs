use splitbits::{combinebits, splitbits, splitbits_named_ux};
use ux::u7;

use crate::bus::Bus;
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
    pub fn write_control_byte(bus: &mut Bus) {
        let fields = splitbits!(bus.cpu_pinout.data_bus, "il..pppp");
        bus.apu_regs.dmc.irq_enabled = fields.i;
        bus.apu_regs.dmc.should_loop = fields.l;
        bus.apu_regs.dmc.period = NTSC_PERIODS[fields.p as usize] - 1;
        if !bus.apu_regs.dmc.irq_enabled {
            CpuPinout::acknowledge_dmc_irq(bus);
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
    pub(super) fn set_enabled(bus: &mut Bus, enable: bool) {
        CpuPinout::acknowledge_dmc_irq(bus);

        let Bus { dmc_dma, apu_regs, master_clock, ..} = bus;
        let dmc = &mut apu_regs.dmc;
        if !enable {
            dmc_dma.disable_soon();
        } else if !dmc_dma.enabled() {
            dmc_dma.enable();
            dmc.sample_address = dmc.sample_start_address;

            if dmc.sample_buffer.is_none() {
                dmc_dma.start_load(master_clock.apu_clock().cycle_parity());
            }
        }
    }

    pub(super) fn tick(&mut self, dmc_dma: &mut DmcDma) {
        // If we don't early return here, then we must be on a PUT cycle since all NTSC_PERIODs are even.
        if self.cycles_remaining >= 1 {
            self.cycles_remaining -= 1;
            return;
        }

        if !self.output_unit.silenced {
            if self.output_unit.sample_shifter & 1 == 1 {
                if self.output_unit.volume < u7::new(126) {
                    self.output_unit.volume = self.output_unit.volume + u7::new(2);
                }
            } else {
                if self.output_unit.volume > u7::new(1) {
                    self.output_unit.volume = self.output_unit.volume - u7::new(2);
                }
            }

            self.output_unit.sample_shifter >>= 1;
        }

        self.cycles_remaining = self.period;
        self.output_unit.bits_remaining = self.output_unit.bits_remaining.saturating_sub(1);
        if self.output_unit.bits_remaining > 0 {
            return;
        }

        // Output cycle just ended, so start a new one.
        self.output_unit.bits_remaining = 8;
        self.output_unit.silenced = self.sample_buffer.is_none();
        if let Some(sample) = self.sample_buffer.take() {
            self.output_unit.sample_shifter = sample;
            if dmc_dma.enabled() {
                dmc_dma.start_reload();
            }
        }
    }

    // Called upon the completion of a DMC DMA (Load OR Reload).
    pub fn set_sample_buffer(bus: &mut Bus, value: u8) {
        if bus.dmc_dma.enabled() {
            bus.apu_regs.dmc.sample_buffer = Some(value);
            bus.apu_regs.dmc.sample_address.inc();
            if bus.apu_regs.dmc.sample_address == CpuAddress::ZERO {
                bus.apu_regs.dmc.sample_address = CpuAddress::new(0x8000);
            }

            bus.dmc_dma.decrement_sample_bytes_remaining();
            if !bus.dmc_dma.enabled() {
                if bus.apu_regs.dmc.should_loop {
                    bus.dmc_dma.enable();
                    bus.apu_regs.dmc.sample_address = bus.apu_regs.dmc.sample_start_address;
                } else if bus.apu_regs.dmc.irq_enabled {
                    CpuPinout::assert_dmc_irq(bus);
                }
            }
        }
    }

    pub(super) fn sample_volume(&self) -> u8 {
        u8::from(self.output_unit.volume)
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