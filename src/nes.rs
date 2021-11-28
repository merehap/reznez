use crate::cartridge::INes;
use crate::cpu::address::Address;
use crate::cpu::cpu::Cpu;
use crate::cpu::instruction::Instruction;
use crate::cpu::memory::Memory;
use crate::mapper::mapper0::Mapper0;

pub struct Nes {
    cpu: Cpu,
}

impl Nes {
    pub fn startup(ines: INes) -> Nes {
        Nes {
            cpu: Cpu::startup(Nes::initialize_memory(ines)),
        }
    }

    pub fn with_program_counter(ines: INes, program_counter: Address) -> Nes {
        Nes {
            cpu: Cpu::with_program_counter(Nes::initialize_memory(ines), program_counter),
        }
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

    pub fn step(&mut self) -> Instruction {
        self.cpu.step()
    }

    pub fn cpu(&self) -> &Cpu {
       &self.cpu
    }
}
