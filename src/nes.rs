use crate::cartridge::INes;
use crate::cpu::cpu::Cpu;
use crate::cpu::memory::Memory;
use crate::mapper::mapper0::Mapper0;

pub struct Nes {
    cpu: Cpu,
}

impl Nes {
    pub fn startup(ines: INes) -> Nes {
        if ines.mapper_number() != 0 {
            panic!("Only mapper 0 is currently supported.");
        }

        let mut memory = Memory::startup();

        let mapper = Mapper0::new();
        mapper.map(ines, &mut memory)
            .expect("Failed to copy cartridge ROM into CPU memory.");

        Nes {
            cpu: Cpu::startup(memory),
        }
    }

    pub fn step(&mut self) {
        self.cpu.step();
    }
}
