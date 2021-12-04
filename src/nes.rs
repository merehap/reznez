use crate::cartridge::INes;
use crate::config::Config;
use crate::cpu::address::Address;
use crate::cpu::cpu::Cpu;
use crate::cpu::instruction::Instruction;
use crate::cpu::memory::Memory;
use crate::ppu::ppu::Ppu;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::ppu_registers::PpuRegisters;
use crate::mapper::mapper0::Mapper0;

const PPU_REGISTERS_START_ADDRESS: Address = Address::new(0x2000);
const PPU_REGISTER_COUNT: u16 = 8;
const OAM_DMA_ADDRESS: Address = Address::new(0x4014);

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    cycle: u64,
    system_palette: SystemPalette,
}

impl Nes {
    pub fn new(config: Config) -> Nes {
        Nes {
            cpu: Cpu::new(
                Nes::initialize_memory(config.ines().clone()),
                config.program_counter_source(),
            ),
            ppu: Ppu::startup(),
            cycle: 0,
            system_palette: config.system_palette().clone(),
        }
    }

    pub fn cpu(&self) -> &Cpu {
       &self.cpu
    }

    pub fn step(&mut self) -> Option<Instruction> {
        let mut instruction = None;
        if self.cycle % 3 == 2 {
            instruction = self.cpu.step();
        }

        let regs = &self.cpu.memory
            .slice(PPU_REGISTERS_START_ADDRESS, PPU_REGISTER_COUNT)
            .try_into()
            .unwrap();
        let ppu_registers = PpuRegisters::from_mem(
            regs,
            &self.cpu.memory[OAM_DMA_ADDRESS],
        );
        self.ppu.step(ppu_registers);

        self.cycle += 1;

        instruction
    }

    fn initialize_memory(ines: INes) -> Memory {
        if ines.mapper_number() != 0 {
            unimplemented!("Only mapper 0 is currently supported.");
        }

        let mut memory = Memory::startup();

        let mapper = Mapper0::new();
        mapper.map(ines, &mut memory)
            .expect("Failed to copy cartridge ROM into CPU memory.");

        memory
    }
}
