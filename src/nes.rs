use std::collections::BTreeSet;

use crate::cartridge::INes;
use crate::config::Config;
use crate::cpu::address::Address;
use crate::cpu::cpu::Cpu;
use crate::cpu::instruction::Instruction;
use crate::cpu::memory::Memory;
use crate::ppu::ppu::Ppu;
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
            ppu: Ppu::new(config.system_palette().clone()),
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
        }

        self.ppu.step();

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
}
