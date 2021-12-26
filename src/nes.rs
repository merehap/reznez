use std::collections::BTreeSet;

use crate::cartridge::INes;
use crate::config::Config;
use crate::controller::joypad::Joypad;
use crate::cpu::address::Address;
use crate::cpu::cpu::{Cpu, StepResult};
use crate::cpu::instruction::Instruction;
use crate::cpu::memory::Memory as CpuMem;
use crate::cpu::port_access::{PortAccess, AccessMode};
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::ppu::{Ppu, VBlankEvent};
use crate::ppu::memory::Memory as PpuMem;
use crate::ppu::register::ctrl::{Ctrl, VBlankNmi};
use crate::ppu::register::mask::Mask;
use crate::ppu::screen::Screen;
use crate::mapper::mapper0::Mapper0;

const PPUCTRL:    Address = Address::new(0x2000);
const PPUMASK:    Address = Address::new(0x2001);
const PPUSTATUS:  Address = Address::new(0x2002);
const OAMADDR:    Address = Address::new(0x2003);
const OAMDATA:    Address = Address::new(0x2004);
const PPUSCROLL:  Address = Address::new(0x2005);
const PPUADDR:    Address = Address::new(0x2006);
const PPUDATA:    Address = Address::new(0x2007);
const OAM_DMA:    Address = Address::new(0x4014);

const JOYSTICK_1_PORT: Address = Address::new(0x4016);
const JOYSTICK_2_PORT: Address = Address::new(0x4017);

const CPU_READ_PORTS: [Address; 5] = [
    PPUSTATUS,
    OAMDATA,
    PPUDATA,

    JOYSTICK_1_PORT,
    JOYSTICK_2_PORT,
];

// All ports are write ports, even the "read-only" PPUSTATUS.
const CPU_WRITE_PORTS: [Address; 10] =
    [
        PPUCTRL,
        PPUMASK,
        PPUSTATUS,
        OAMADDR,
        OAMDATA,
        PPUSCROLL,
        PPUADDR,
        PPUDATA,
        OAM_DMA,

        JOYSTICK_1_PORT,
    ];

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    pub joypad_1: Joypad,
    pub joypad_2: Joypad,
    cycle: u64,
}

impl Nes {
    pub fn new(config: Config) -> Nes {
        let (cpu_mem, ppu_mem) = Nes::initialize_memory(
            config.ines().clone(),
            config.system_palette().clone(),
            );

        Nes {
            cpu: Cpu::new(cpu_mem, config.program_counter_source()),
            ppu: Ppu::new(ppu_mem),
            joypad_1: Joypad::new(),
            joypad_2: Joypad::new(),
            cycle: 0,
        }
    }

    pub fn cpu(&self) -> &Cpu {
       &self.cpu
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn step(&mut self, screen: &mut Screen) -> Option<Instruction> {
        let mut instruction = None;
        if self.cycle % 3 == 2 {
            match self.cpu.step() {
                StepResult::Nop => {},
                StepResult::InstructionComplete(inst) => instruction = Some(inst),
                StepResult::DmaWrite(value) => self.ppu.write_oam(value),
            }

            if let Some(port_access) = self.cpu.memory.latch() {
                self.execute_port_action(port_access);
            }
        }

        let step_events = self.ppu.step(self.ppu_mask(), screen);
        match step_events.vblank_event() {
            VBlankEvent::Started => self.set_vblank(),
            VBlankEvent::Stopped => self.clear_vblank(),
            VBlankEvent::None => {},
        }

        if step_events.nmi_trigger() {
            self.schedule_nmi_if_enabled();
        }

        let status = self.joypad_1.selected_button_status() as u8;
        self.cpu.memory.write(JOYSTICK_1_PORT, status);
        let status = self.joypad_2.selected_button_status() as u8;
        self.cpu.memory.write(JOYSTICK_2_PORT, status);

        self.cycle += 1;

        instruction
    }

    fn initialize_memory(ines: INes, system_palette: SystemPalette) -> (CpuMem, PpuMem) {
        if ines.mapper_number() != 0 {
            unimplemented!("Only mapper 0 is currently supported.");
        }

        let mut cpu_mem = CpuMem::new(
            BTreeSet::from(CPU_READ_PORTS),
            BTreeSet::from(CPU_WRITE_PORTS),
            );

        let mut ppu_mem = PpuMem::new(ines.name_table_mirroring(), system_palette);

        let mapper = Mapper0::new();
        mapper.map(ines, &mut cpu_mem, &mut ppu_mem)
            .expect("Failed to copy cartridge ROM into CPU memory.");

        (cpu_mem, ppu_mem)
    }

    // TODO: Reading PPUSTATUS within two cycles of the start of vertical
    // blank will return 0 in bit 7 but clear the latch anyway, causing NMI
    // to not occur that frame.
    fn execute_port_action(&mut self, port_access: PortAccess) {
        let value = port_access.value;

        use AccessMode::*;
        match (port_access.address, port_access.access_mode) {
            (PPUCTRL, Write) => {
                let old_vblank_nmi = self.ppu.ctrl().vblank_nmi();
                let ctrl = Ctrl::from_u8(value);
                self.ppu.set_ctrl(ctrl);
                // A second NMI can only be scheduled if VBlankNmi was toggled.
                if old_vblank_nmi == VBlankNmi::Off {
                    self.schedule_nmi_if_enabled();
                }
            },
            (PPUMASK, Write) => {},

            // TODO: Reading the status register will clear bit 7 mentioned
            // above and also the address latch used by PPUSCROLL and PPUADDR.
            (PPUSTATUS, Read) => self.clear_vblank(),
            (PPUSTATUS, Write) => {/* PPUSTATUS is read-only. */},

            (OAMADDR, Write) => self.ppu.set_oam_address(value),
            (OAMDATA, Read) => unimplemented!(),
            (OAMDATA, Write) => unimplemented!(),
            (OAM_DMA, Write) =>
                self.cpu.initiate_dma_transfer(
                    value,
                    256 - self.ppu.oam_address() as u16,
                    ),

            (PPUADDR, Write) => self.ppu.write_partial_vram_address(value),
            (PPUDATA, Read) => unimplemented!(),
            (PPUDATA, Write) => self.ppu.write_vram(value),

            (PPUSCROLL, Write) => println!("PPUSCROLL was written to (not supported)."),

            (JOYSTICK_1_PORT, Read) => {
                // Now that the ROM has read a button status, advance to the next one.
                self.joypad_1.select_next_button();
            },
            (JOYSTICK_2_PORT, Read) => {
                // Now that the ROM has read a button status, advance to the next one.
                self.joypad_2.select_next_button();
            },
            (JOYSTICK_1_PORT, Write) => {
                if value & 1 == 1 {
                    self.joypad_1.strobe_on();
                    self.joypad_2.strobe_on();
                } else {
                    self.joypad_1.strobe_off();
                    self.joypad_2.strobe_off();
                }
            },

            (_, _) => unreachable!(),
        }
    }

    fn set_vblank(&mut self) {
        println!("VBlank set.");
        let status = self.cpu.memory.read(PPUSTATUS);
        self.cpu.memory.write(PPUSTATUS, status | 0b1000_0000);
    }

    fn clear_vblank(&mut self) {
        println!("VBlank cleared.");
        let status = self.cpu.memory.read(PPUSTATUS);
        self.cpu.memory.write(PPUSTATUS, status & 0b0111_1111);
    }

    fn schedule_nmi_if_enabled(&mut self) {
        if self.ppu.nmi_enabled() {
            println!("Scheduling NMI.");
            // Execute an extra NMI beyond the vblank-start NMI.
            self.cpu.schedule_nmi();
        }
    }

    fn ppu_mask(&self) -> Mask {
        Mask::from_u8(*self.cpu.memory.bus_access(PPUMASK))
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::cpu::ProgramCounterSource;
    use crate::ppu::palette::system_palette::SystemPalette;
    use crate::ppu::screen::Screen;

    use crate::cartridge::tests::sample_ines;

    use super::*;

    #[test]
    fn nmi_enabled_upon_vblank() {
        let mut nes = sample_nes();
        nes.ppu.set_ctrl(Ctrl::from_u8(0b1000_0000));
        step_until_vblank_nmi_enabled(&mut nes);
        assert!(nes.cpu.nmi_pending());
    }

    #[test]
    fn second_nmi_fails_without_ctrl_toggle() {
        let mut nes = sample_nes();
        nes.ppu.set_ctrl(Ctrl::from_u8(0b1000_0000));
        step_until_vblank_nmi_enabled(&mut nes);
        assert!(nes.cpu.nmi_pending());

        let mut screen = Screen::new();
        while nes.step(&mut screen).is_none() {}
        nes.step(&mut screen);

        assert!(
            !nes.cpu.nmi_pending(),
            "nmi_pending should have been cleared after one instruction."
            );

        // Disable vblank_nmi.
        write_ppuctrl_through_opcode_injection(&mut nes, 0b0000_0000);

        assert!(
            !nes.cpu.nmi_pending(),
            "A second NMI should not have been allowed without toggling CTRL.0 .",
            );
    }

    #[test]
    fn second_nmi_succeeds_after_ctrl_toggle() {
        let mut nes = sample_nes();
        nes.ppu.set_ctrl(Ctrl::from_u8(0b1000_0000));
        step_until_vblank_nmi_enabled(&mut nes);
        assert!(nes.cpu.nmi_pending());

        let mut screen = Screen::new();
        while nes.step(&mut screen).is_none() {}
        nes.step(&mut screen);

        assert!(
            !nes.cpu.nmi_pending(),
            "nmi_pending should have been cleared after one instruction."
            );

        // Disable vblank_nmi.
        write_ppuctrl_through_opcode_injection(&mut nes, 0b0000_0000);
        // Enable vblank_nmi.
        write_ppuctrl_through_opcode_injection(&mut nes, 0b1000_0000);

        assert!(
            nes.cpu.nmi_pending(),
            "A second NMI should have been allowed after toggling CTRL.0 .",
            );
    }

    fn sample_nes() -> Nes {
        let ines = sample_ines();
        let system_palette =
            SystemPalette::parse(include_str!("../palettes/2C02.pal")).unwrap();

        let (cpu_mem, ppu_mem) = Nes::initialize_memory(ines, system_palette);
        Nes {
            cpu: Cpu::new(
                cpu_mem,
                ProgramCounterSource::Override(Address::new(0x2000)),
                ),
            ppu: Ppu::new(ppu_mem),
            joypad_1: Joypad::new(),
            joypad_2: Joypad::new(),
            cycle: 0,
        }
    }

    fn step_until_vblank_nmi_enabled(nes: &mut Nes) {
        let mut screen = Screen::new();
        while !nes.ppu.nmi_enabled() {
            assert!(!nes.cpu.nmi_pending(), "NMI must not be pending before one is scheduled.");
            nes.step(&mut screen);
            if nes.ppu.clock().total_cycles() > 200_000 {
                panic!("It took too long for the PPU to enable NMI.");
            }
        }
    }

    fn write_ppuctrl_through_opcode_injection(nes: &mut Nes, ctrl: u8) {
        // STA: Store to the accumulator.
        nes.cpu.memory.write(nes.cpu.program_counter().advance(0), 0xA9);
        // Store VBLANK_NMI DISABLED to the accumulator.
        nes.cpu.memory.write(nes.cpu.program_counter().advance(1), ctrl);

        // LDA: Load the accumulator into a memory location.
        nes.cpu.memory.write(nes.cpu.program_counter().advance(2), 0x8D);
        // Low byte of PPUCTRL, the address to be set.
        nes.cpu.memory.write(nes.cpu.program_counter().advance(3), 0x00);
        // High byte of PPUCTRL, the address to be set.
        nes.cpu.memory.write(nes.cpu.program_counter().advance(4), 0x20);

        // Execute the two op codes we just injected.
        let mut screen = Screen::new();
        while nes.step(&mut screen).is_none() {}
        while nes.step(&mut screen).is_none() {}
    }
}
