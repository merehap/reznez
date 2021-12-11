use std::collections::BTreeSet;

use crate::cartridge::INes;
use crate::config::Config;
use crate::cpu::address::Address;
use crate::cpu::cpu::Cpu;
use crate::cpu::instruction::Instruction;
use crate::cpu::memory::Memory;
use crate::cpu::port_access::{PortAccess, AccessMode};
use crate::ppu::ppu::{Ppu, VBlankStatus};
use crate::ppu::register::ctrl::Ctrl;
use crate::ppu::register::mask::Mask;
use crate::mapper::mapper0::Mapper0;

const PPUCTRL:   Address = Address::new(0x2000);
const PPUMASK:   Address = Address::new(0x2001);
const PPUSTATUS: Address = Address::new(0x2002);
const OAMADDR:   Address = Address::new(0x2003);
const OAMDATA:   Address = Address::new(0x2004);
const PPUSCROLL: Address = Address::new(0x2005);
const PPUADDR:   Address = Address::new(0x2006);
const PPUDATA:   Address = Address::new(0x2007);
const OAM_DMA:   Address = Address::new(0x4014);

const CPU_READ_PORTS: [Address; 3] = [PPUSTATUS, OAMDATA, PPUDATA];
// All ports are write ports, even the "read-only" PPUSTATUS.
const CPU_WRITE_PORTS: [Address; 9] =
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
    ];

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    cycle: u64,
}

impl Nes {
    pub fn new(config: Config) -> Nes {
        Nes {
            cpu: Cpu::new(
                Nes::initialize_memory(config.ines().clone()),
                config.program_counter_source(),
                ),
            ppu: Ppu::new(
                config.ines().name_table_mirroring(),
                config.system_palette().clone(),
                ),
            cycle: 0,
        }
    }

    pub fn cpu(&self) -> &Cpu {
       &self.cpu
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn step(&mut self) -> Option<Instruction> {
        let mut instruction = None;
        if self.cycle % 3 == 2 {
            instruction = self.cpu.step();
            if let Some(port_access) = self.cpu.memory.latch() {
                self.update_ppu_registers(port_access);
            }
        }

        let step_events = self.ppu.step();
        match step_events.vblank_status() {
            VBlankStatus::Started => self.set_vblank(),
            VBlankStatus::Stopped => self.clear_vblank(),
            _ => {},
        }

        self.cycle += 1;

        instruction
    }

    fn initialize_memory(ines: INes) -> Memory {
        if ines.mapper_number() != 0 {
            unimplemented!("Only mapper 0 is currently supported.");
        }

        let mut memory = Memory::new(
            BTreeSet::from(CPU_READ_PORTS),
            BTreeSet::from(CPU_WRITE_PORTS),
            );

        let mapper = Mapper0::new();
        mapper.map(ines, &mut memory)
            .expect("Failed to copy cartridge ROM into CPU memory.");

        memory
    }

    // TODO: Reading PPUSTATUS within two cycles of the start of vertical
    // blank will return 0 in bit 7 but clear the latch anyway, causing NMI
    // to not occur that frame.
    fn update_ppu_registers(&mut self, port_access: PortAccess) {
        let value = port_access.value;

        use AccessMode::*;
        match (port_access.address, port_access.access_mode) {
            (PPUCTRL, Write) => self.ppu.set_ctrl(Ctrl::from_u8(value)),
            (PPUMASK, Write) => self.ppu.set_mask(Mask::from_u8(value)),
            (OAMADDR, Write) => unimplemented!(),
            (PPUSCROLL, Write) => println!("PPUSCROLL was written to (not supported)."),
            (PPUADDR, Write) => self.ppu.write_partial_vram_address(value),
            (OAM_DMA, Write) => unimplemented!(),

            // TODO: Reading the status register will clear bit 7 mentioned
            // above and also the address latch used by PPUSCROLL and PPUADDR.
            (PPUSTATUS, Read) => self.clear_vblank(),
            // PPUSTATUS is read-only.
            (PPUSTATUS, Write) => {},
            (OAMDATA, Read) => unimplemented!(),
            (OAMDATA, Write) => unimplemented!(),
            (PPUDATA, Read) => unimplemented!(),
            (PPUDATA, Write) => self.ppu.write_vram(value),

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
}
