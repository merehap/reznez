use crate::cartridge::INes;
use crate::cpu::address::Address;
use crate::cpu::cpu::Cpu;
use crate::cpu::instruction::Instruction;
use crate::cpu::memory::Memory;
use crate::ppu::ppu::Ppu;
use crate::ppu::ppu_registers::PpuRegisters;
use crate::mapper::mapper0::Mapper0;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    cycle: u64,
}

impl Nes {
    pub fn startup(ines: INes) -> Nes {
        Nes {
            cpu: Cpu::startup(Nes::initialize_memory(ines)),
            ppu: Ppu::startup(),
            cycle: 0,
        }
    }

    pub fn with_program_counter(ines: INes, program_counter: Address) -> Nes {
        let cpu = Cpu::with_program_counter(
            Nes::initialize_memory(ines),
            program_counter,
        );

        Nes {
            cpu,
            ppu: Ppu::startup(),
            cycle: 0,
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
            .slice(Address::new(0x2000), 8)
            .try_into()
            .unwrap();
        let ppu_registers = PpuRegisters::from_mem(
            regs,
            &self.cpu.memory[Address::new(0x4014)],
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
