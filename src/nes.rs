use std::ops::Add;
use std::time::{Duration, SystemTime};

use log::{info, warn};

use crate::cartridge::INes;
use crate::config::Config;
use crate::controller::joypad::Joypad;
use crate::cpu::cpu::{Cpu, StepResult};
use crate::cpu::instruction::Instruction;
use crate::cpu::memory::Memory as CpuMem;
use crate::cpu::memory::*;
use crate::cpu::port_access::{PortAccess, AccessMode};
use crate::gui::gui::Gui;
use crate::mapper::mapper0::Mapper0;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::ppu::Ppu;
use crate::ppu::memory::Memory as PpuMem;
use crate::ppu::register::ctrl::Ctrl;
use crate::ppu::register::mask::Mask;
use crate::ppu::render::frame::Frame;
use crate::ppu::render::frame_rate::TargetFrameRate;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    pub joypad_1: Joypad,
    pub joypad_2: Joypad,
    nmi_was_enabled: bool,
    cycle: u64,

    target_frame_rate: TargetFrameRate,
    stop_frame: Option<u64>,
}

impl Nes {
    pub fn new(config: &Config) -> Nes {
        let (cpu_mem, ppu_mem) = Nes::initialize_memory(
            &config.ines,
            config.system_palette.clone(),
        );

        Nes {
            cpu: Cpu::new(cpu_mem, config.program_counter_source),
            ppu: Ppu::new(ppu_mem),
            joypad_1: Joypad::new(),
            joypad_2: Joypad::new(),
            nmi_was_enabled: false,
            cycle: 0,

            target_frame_rate: config.target_frame_rate,
            stop_frame: config.stop_frame,
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

    pub fn step_frame(&mut self, gui: &mut dyn Gui) {
        let frame_index = self.ppu().clock().frame();
        let start_time = SystemTime::now();
        let intended_frame_end_time = start_time.add(self.frame_duration());

        let events = gui.events();

        for (button, status) in events.joypad_1_button_statuses {
            self.joypad_1.set_button_status(button, status);
        }

        for (button, status) in events.joypad_2_button_statuses {
            self.joypad_2.set_button_status(button, status);
        }

        loop {
            let is_last_cycle = self.ppu().clock().is_last_cycle_of_frame();
            self.step(gui.frame_mut());

            if is_last_cycle {
                break;
            }
        }

        gui.display_frame(frame_index);
        info!("Frame: {}", frame_index);

        let end_time = SystemTime::now();
        if let Ok(duration) = intended_frame_end_time.duration_since(end_time) {
            std::thread::sleep(duration);
        }

        let end_time = SystemTime::now();
        if let Ok(duration) = end_time.duration_since(start_time) {
            info!("Framerate: {}", 1_000_000_000.0 / duration.as_nanos() as f64);
        } else {
            warn!("Unknown framerate. System clock went backwards.");
        }

        if events.should_quit || Some(frame_index) == self.stop_frame {
            std::process::exit(0);
        }
    }

    pub fn step(&mut self, frame: &mut Frame) -> Option<Instruction> {
        let mut instruction = None;
        if self.cycle % 3 == 0 {
            match self.cpu.step() {
                StepResult::Nop => {},
                StepResult::InstructionComplete(inst) => instruction = Some(inst),
                StepResult::DmaWrite(oamaddr, value) =>
                    self.ppu.write_oam(oamaddr, value),
            }

            if let Some(port_access) = self.cpu.memory.latch() {
                self.execute_port_action(port_access);
            }
        }

        self.ppu.step(self.ppu_ctrl(), self.ppu_mask(), frame);
        // TODO: Why do we need to access PPUSTATUS before Ppu.step()?
        *self.cpu.memory.bus_access_mut(PPUSTATUS) = self.ppu.status().to_u8();
        *self.cpu.memory.bus_access_mut(OAMDATA) = self.ppu.read_oam(self.oam_address());
        *self.cpu.memory.bus_access_mut(PPUDATA) = self.ppu.vram_data();

        if self.ppu.vblank_just_started() {
            self.schedule_nmi_if_allowed();
        }

        let status = self.joypad_1.selected_button_status() as u8;
        self.cpu.memory.write(JOYSTICK_1_PORT, status);
        let status = self.joypad_2.selected_button_status() as u8;
        self.cpu.memory.write(JOYSTICK_2_PORT, status);

        self.cycle += 1;

        instruction
    }

    fn initialize_memory(ines: &INes, system_palette: SystemPalette) -> (CpuMem, PpuMem) {
        if ines.mapper_number() != 0 {
            unimplemented!("Only mapper 0 is currently supported.");
        }

        let mut cpu_mem = CpuMem::new();
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

        use AccessMode::{Read, Write};
        match (port_access.address, port_access.access_mode) {
            (PPUMASK | PPUSTATUS | OAMADDR, Write) => {},
            (OAMDATA, Read) => {},

            (PPUCTRL, Write) => {
                // A second NMI can only be scheduled if VBlankNmi was
                // disabled last cycle.
                if !self.nmi_was_enabled {
                    self.schedule_nmi_if_allowed();
                }

                self.nmi_was_enabled = self.ppu_ctrl().nmi_enabled;
            },

            // TODO: Reading the status register will clear bit 7 mentioned
            // above and also the address latch used by PPUSCROLL and PPUADDR.
            (PPUSTATUS, Read) => {
                self.ppu.stop_vblank();
                self.ppu.reset_address_latch();
            },
            (OAMDATA, Write) => self.write_oam(value),
            (OAM_DMA, Write) =>
                self.cpu.initiate_dma_transfer(value, self.oam_address()),
            (PPUADDR, Write) => self.ppu.write_partial_vram_address(value),
            (PPUDATA, Read) => self.ppu.update_vram_data(self.ppu_ctrl()),
            (PPUDATA, Write) => self.ppu.write_vram(self.ppu_ctrl(), value),
            (PPUSCROLL, Write) => self.ppu.write_scroll_dimension(value),

            // Now that the ROM has read a button status, advance to the next status.
            (JOYSTICK_1_PORT, Read) => self.joypad_1.select_next_button(),
            (JOYSTICK_2_PORT, Read) => self.joypad_2.select_next_button(),
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

    fn schedule_nmi_if_allowed(&mut self) {
        if self.ppu.can_generate_nmi(self.ppu_ctrl()) {
            info!("Scheduling NMI.");
            self.cpu.schedule_nmi();
        }
    }

    fn ppu_ctrl(&self) -> Ctrl {
        Ctrl::from_u8(*self.cpu.memory.bus_access(PPUCTRL))
    }

    fn ppu_mask(&self) -> Mask {
        Mask::from_u8(*self.cpu.memory.bus_access(PPUMASK))
    }

    fn oam_address(&self) -> u8 {
        *self.cpu.memory.bus_access(OAMADDR)
    }

    fn write_oam(&mut self, value: u8) {
        let oamaddr = *self.cpu.memory.bus_access(OAMADDR);
        self.ppu.write_oam(oamaddr, value);
        // Advance to next sprite byte to write.
        // TODO: Verify that wrapping is the correct behavior.
        *self.cpu.memory.bus_access_mut(OAMADDR) = oamaddr.wrapping_add(1);
    }

    fn frame_duration(&self) -> Duration {
        match self.target_frame_rate {
            TargetFrameRate::Value(frame_rate) => frame_rate.to_frame_duration(),
            TargetFrameRate::Unbounded => Duration::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::address::Address;
    use crate::cpu::cpu::ProgramCounterSource;
    use crate::ppu::palette::system_palette::SystemPalette;
    use crate::ppu::register::ctrl::Ctrl;
    use crate::ppu::render::frame::Frame;
    use crate::ppu::render::frame_rate::TargetFrameRate;

    use crate::cartridge::tests::sample_ines;

    use super::*;

    #[test]
    fn nmi_enabled_upon_vblank() {
        let mut nes = sample_nes();
        step_until_vblank_nmi_enabled(&mut nes);
        assert!(nes.cpu.nmi_pending());
    }

    #[test]
    fn second_nmi_fails_without_ctrl_toggle() {
        let mut nes = sample_nes();
        step_until_vblank_nmi_enabled(&mut nes);
        assert!(nes.cpu.nmi_pending());

        let mut frame = Frame::new();
        while nes.step(&mut frame).is_none() {}
        nes.step(&mut frame);

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
        step_until_vblank_nmi_enabled(&mut nes);
        assert!(nes.cpu.nmi_pending());

        let mut frame = Frame::new();
        while nes.step(&mut frame).is_none() {}
        nes.step(&mut frame);

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

        let (cpu_mem, ppu_mem) = Nes::initialize_memory(&ines, system_palette);
        Nes {
            cpu: Cpu::new(
                cpu_mem,
                ProgramCounterSource::Override(Address::new(0x0)),
                ),
            ppu: Ppu::new(ppu_mem),
            joypad_1: Joypad::new(),
            joypad_2: Joypad::new(),
            nmi_was_enabled: false,
            cycle: 0,

            target_frame_rate: TargetFrameRate::Unbounded,
            stop_frame: None,
        }
    }

    fn step_until_vblank_nmi_enabled(nes: &mut Nes) {
        let mut ctrl = Ctrl::from_u8(*nes.cpu.memory.bus_access(PPUCTRL));
        ctrl.nmi_enabled = true;
        *nes.cpu.memory.bus_access_mut(PPUCTRL) = ctrl.to_u8();

        let mut frame = Frame::new();
        while !nes.ppu.can_generate_nmi(nes.ppu_ctrl()) {
            assert!(!nes.cpu.nmi_pending(), "NMI must not be pending before one is scheduled.");
            nes.step(&mut frame);
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
        let mut frame = Frame::new();
        while nes.step(&mut frame).is_none() {}
        while nes.step(&mut frame).is_none() {}
    }
}
