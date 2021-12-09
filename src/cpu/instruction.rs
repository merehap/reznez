use std::fmt;

use lazy_static::lazy_static;
use strum_macros::EnumString;

use crate::cpu::address::Address;
use crate::cpu::memory::Memory;

lazy_static! {
    static ref INSTRUCTION_TEMPLATES: [InstructionTemplate; 256] = instruction_templates();
}

fn instruction_templates() -> [InstructionTemplate; 256] {
    use OpCode::*;
    use AccessMode::*;
    use ExtraCycle::*;

    let jam = (JAM, Imp, 0, No);
    let codes: [[(OpCode, AccessMode, u8, ExtraCycle); 8]; 32] = [
        /*00*/           /*20*/          /*40*/          /*60*/          /*80*/          /*a0*/          /*c0*/          /*e0*/
/*+00*/ [(BRK,Imp,7,No), (JSR,Abs,6,No), (RTI,Imp,6,No), (RTS,Imp,6,No), (NOP,Imm,2,No), (LDY,Imm,2,No), (CPY,Imm,2,No), (CPX,Imm,2,No)],
/*+01*/ [(ORA,IzX,6,No), (AND,IzX,6,No), (EOR,IzX,6,No), (ADC,IzX,6,No), (STA,IzX,6,No), (LDA,IzX,6,No), (CMP,IzX,6,No), (SBC,IzX,6,No)],
/*+02*/ [jam           , jam           , jam           , jam           , jam           , (LDX,Imm,2,No), jam           , jam           ],
/*+03*/ [(SLO,IzX,8,No), (RLA,IzX,8,No), (SRE,IzX,8,No), (RRA,IzX,8,No), (SAX,IzX,6,No), (LAX,IzX,6,No), (DCP,IzX,8,No), (ISC,IzX,8,No)],
/*+04*/ [(NOP,ZP ,3,No), (BIT,ZP ,3,No), (NOP,ZP ,3,No), (NOP,ZP ,3,No), (STY,ZP ,3,No), (LDY,ZP ,3,No), (CPY,ZP ,3,No), (CPX,ZP ,3,No)],
/*+05*/ [(ORA,ZP ,3,No), (AND,ZP ,3,No), (EOR,ZP ,3,No), (ADC,ZP ,3,No), (STA,ZP ,3,No), (LDA,ZP ,3,No), (CMP,ZP ,3,No), (SBC,ZP ,3,No)],
/*+06*/ [(ASL,ZP ,5,No), (ROL,ZP ,5,No), (LSR,ZP ,5,No), (ROR,ZP ,5,No), (STX,ZP ,3,No), (LDX,ZP ,3,No), (DEC,ZP ,5,No), (INC,ZP ,5,No)],
/*+07*/ [(SLO,ZP ,5,No), (RLA,ZP ,5,No), (SRE,ZP ,5,No), (RRA,ZP ,5,No), (SAX,ZP ,3,No), (LAX,ZP ,3,No), (DCP,ZP ,5,No), (ISC,ZP ,5,No)],
/*+08*/ [(PHP,Imp,3,No), (PLP,Imp,4,No), (PHA,Imp,3,No), (PLA,Imp,4,No), (DEY,Imp,2,No), (TAY,Imp,2,No), (INY,Imp,2,No), (INX,Imp,2,No)],
/*+09*/ [(ORA,Imm,2,No), (AND,Imm,2,No), (EOR,Imm,2,No), (ADC,Imm,2,No), (NOP,Imm,2,No), (LDA,Imm,2,No), (CMP,Imm,2,No), (SBC,Imm,2,No)],
/*+0a*/ [(ASL,Imp,2,No), (ROL,Imp,2,No), (LSR,Imp,2,No), (ROR,Imp,2,No), (TXA,Imp,2,No), (TAX,Imp,2,No), (DEX,Imp,2,No), (NOP,Imp,2,No)],
/*+0b*/ [(ANC,Imm,2,No), (ANC,Imm,2,No), (ALR,Imm,2,No), (ARR,Imm,2,No), (XAA,Imm,2,No), (LAX,Imm,2,No), (AXS,Imm,2,No), (SBC,Imm,2,No)],
/*+0c*/ [(NOP,Abs,4,No), (BIT,Abs,4,No), (JMP,Abs,3,No), (JMP,Ind,5,No), (STY,Abs,4,No), (LDY,Abs,4,No), (CPY,Abs,4,No), (CPX,Abs,4,No)],
/*+0d*/ [(ORA,Abs,4,No), (AND,Abs,4,No), (EOR,Abs,4,No), (ADC,Abs,4,No), (STA,Abs,4,No), (LDA,Abs,4,No), (CMP,Abs,4,No), (SBC,Abs,4,No)],
/*+0e*/ [(ASL,Abs,6,No), (ROL,Abs,6,No), (LSR,Abs,6,No), (ROR,Abs,6,No), (STX,Abs,4,No), (LDX,Abs,4,No), (DEC,Abs,6,No), (INC,Abs,6,No)],
/*+0f*/ [(SLO,Abs,6,No), (RLA,Abs,6,No), (SRE,Abs,6,No), (RRA,Abs,6,No), (SAX,Abs,4,No), (LAX,Abs,4,No), (DCP,Abs,6,No), (ISC,Abs,6,No)],

/*+10*/ [(BPL,Rel,2,PB), (BMI,Rel,2,PB), (BVC,Rel,2,PB), (BVS,Rel,2,PB), (BCC,Rel,2,PB), (BCS,Rel,2,PB), (BNE,Rel,2,PB), (BEQ,Rel,2,PB)],
/*+11*/ [(ORA,IzY,5,PB), (AND,IzY,5,PB), (EOR,IzY,5,PB), (ADC,IzY,5,PB), (STA,IzY,6,No), (LDA,IzY,5,PB), (CMP,IzY,5,PB), (SBC,IzY,5,PB)],
/*+12*/ [jam           , jam           , jam           , jam           , jam           , jam           , jam           , jam           ],
/*+13*/ [(SLO,IzY,8,No), (RLA,IzY,8,No), (SRE,IzY,8,No), (RRA,IzY,8,No), (AHX,IzY,8,No), (LAX,IzY,5,PB), (DCP,IzY,8,No), (ISC,IzY,8,No)],
/*+14*/ [(NOP,ZPX,4,No), (NOP,ZPX,4,No), (NOP,ZPX,4,No), (NOP,ZPX,4,No), (STY,ZPX,4,No), (LDY,ZPX,4,No), (NOP,ZPX,4,No), (NOP,ZPX,4,No)],
/*+15*/ [(ORA,ZPX,4,No), (AND,ZPX,4,No), (EOR,ZPX,4,No), (ADC,ZPX,4,No), (STA,ZPX,4,No), (LDA,ZPX,4,No), (CMP,ZPX,4,No), (SBC,ZPX,4,No)],
/*+16*/ [(ASL,ZPX,6,No), (ROL,ZPX,6,No), (LSR,ZPX,6,No), (ROR,ZPX,6,No), (STX,ZPY,4,No), (LDX,ZPY,4,No), (DEC,ZPX,6,No), (INC,ZPX,6,No)],
/*+17*/ [(SLO,ZPX,6,No), (RLA,ZPX,6,No), (SRE,ZPX,6,No), (RRA,ZPX,6,No), (SAX,ZPY,4,No), (LAX,ZPY,4,No), (DCP,ZPX,6,No), (ISC,ZPX,6,No)],
/*+18*/ [(CLC,Imp,2,No), (SEC,Imp,2,No), (CLI,Imp,2,No), (SEI,Imp,2,No), (TYA,Imp,2,No), (CLV,Imp,2,No), (CLD,Imp,2,No), (SED,Imp,2,No)],
/*+19*/ [(ORA,AbY,4,PB), (AND,AbY,4,PB), (EOR,AbY,4,PB), (ADC,AbY,4,PB), (STA,AbY,5,No), (LDA,AbY,4,PB), (CMP,AbY,4,PB), (SBC,AbY,4,PB)],
/*+1a*/ [(NOP,Imp,2,No), (NOP,Imp,2,No), (NOP,Imp,2,No), (NOP,Imp,2,No), (TXS,Imp,2,No), (TSX,Imp,2,No), (NOP,Imp,2,No), (NOP,Imp,2,No)],
/*+1b*/ [(SLO,AbY,7,No), (RLA,AbY,7,No), (SRE,AbY,7,No), (RRA,AbY,7,No), (TAS,AbY,5,No), (LAS,AbY,4,PB), (DCP,AbY,7,No), (ISC,AbY,7,No)],
/*+1c*/ [(NOP,AbX,4,PB), (NOP,AbX,4,PB), (NOP,AbX,4,PB), (NOP,AbX,4,PB), (SHY,AbX,5,No), (LDY,AbX,4,PB), (NOP,AbX,4,PB), (NOP,AbX,4,PB)],
/*+1d*/ [(ORA,AbX,4,PB), (AND,AbX,4,PB), (EOR,AbX,4,PB), (ADC,AbX,4,PB), (STA,AbX,5,No), (LDA,AbX,4,PB), (CMP,AbX,4,PB), (SBC,AbX,4,PB)],
/*+1e*/ [(ASL,AbX,7,No), (ROL,AbX,7,No), (LSR,AbX,7,No), (ROR,AbX,7,No), (SHX,AbY,5,No), (LDX,AbY,4,PB), (DEC,AbX,7,No), (INC,AbX,7,No)],
/*+1f*/ [(SLO,AbX,7,No), (RLA,AbX,7,No), (SRE,AbX,7,No), (RRA,AbX,7,No), (AHX,AbY,5,No), (LAX,AbY,4,PB), (DCP,AbX,7,No), (ISC,AbX,7,No)],
    ];

    let mut result = [InstructionTemplate::from_tuple(0x2, jam); 256];
    for (index, template) in result.iter_mut().enumerate() {
        let i = index % 0x20;
        let j = index / 0x20;
        *template = InstructionTemplate::from_tuple(index as u8, codes[i][j]);
    }

    result
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub template: InstructionTemplate,
    pub argument: Argument,
    pub page_boundary_crossed: bool,
}

impl Instruction {
    pub fn from_memory(
        program_counter: Address,
        x_index: u8,
        y_index: u8,
        mem: &mut Memory,
    ) -> Instruction {

        let template = INSTRUCTION_TEMPLATES[mem.read(program_counter) as usize];
        let low = mem.read(program_counter.offset(1));
        let high = mem.read(program_counter.offset(2));

        let mut page_boundary_crossed = false;

        use AccessMode::*;
        let argument = match template.access_mode {
            Imp => Argument::Imp,
            Imm => Argument::Imm(low),
            ZP => {
                let address = Address::zero_page(low);
                Argument::Addr(address, mem.read(address))
            },
            ZPX => {
                let address = Address::zero_page(low.wrapping_add(x_index));
                Argument::Addr(address, mem.read(address))
            },
            ZPY => {
                let address = Address::zero_page(low.wrapping_add(y_index));
                Argument::Addr(address, mem.read(address))
            },
            Abs => {
                let address = Address::from_low_high(low, high);
                Argument::Addr(address, mem.read(address))
            },
            AbX => {
                let start_address = Address::from_low_high(low, high);
                let address = start_address.advance(x_index);
                page_boundary_crossed = start_address.page() != address.page();
                Argument::Addr(address, mem.read(address))
            },
            AbY => {
                let start_address = Address::from_low_high(low, high);
                let address = start_address.advance(y_index);
                page_boundary_crossed = start_address.page() != address.page();
                Argument::Addr(address, mem.read(address))
            },
            Rel => {
                let address = program_counter
                    .offset(low as i8)
                    .advance(template.access_mode.instruction_length());
                Argument::Addr(address, mem.read(address))
            },
            Ind => {
                let first = Address::from_low_high(low, high);
                let second = Address::from_low_high(low.wrapping_add(1), high);
                let address = Address::from_low_high(mem.read(first), mem.read(second));
                Argument::Addr(address, mem.read(address))
            },
            IzX => {
                let low = low.wrapping_add(x_index);
                let address = Address::from_low_high(
                    mem.read(Address::zero_page(low)),
                    mem.read(Address::zero_page(low.wrapping_add(1))),
                );
                Argument::Addr(address, mem.read(address))
            },
            IzY => {
                let start_address = Address::from_low_high(
                    mem.read(Address::zero_page(low)),
                    mem.read(Address::zero_page(low.wrapping_add(1))),
                );
                // TODO: Should this wrap around just the current page?
                let address = start_address.advance(y_index);
                page_boundary_crossed = start_address.page() != address.page();
                Argument::Addr(address, mem.read(address))
            },
        };

        Instruction {
            template,
            argument,
            page_boundary_crossed,
        }
    }

    pub fn should_add_oops_cycle(&self) -> bool {
        self.template.extra_cycle == ExtraCycle::PB && self.page_boundary_crossed
    }

    pub fn length(&self) -> u8 {
        self.template.access_mode.instruction_length()
    }
}

impl fmt::Display for Instruction {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> fmt::Result {
        let mut access_mode = format!("{:?}", self.template.access_mode);
        if access_mode.len() == 2 {
            access_mode.push(' ');
        }

        write!(f, "0x{:02X} ({:?} {} Cycles:{:?}+{:?}) Arg:{:5} PB:{}",
            self.template.code_point, self.template.op_code, access_mode,
            self.template.cycle_count as usize, self.template.extra_cycle,
            self.argument, self.page_boundary_crossed)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Argument {
    Imp,
    Imm(u8),
    Addr(Address, u8),
}

impl fmt::Display for Argument {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> fmt::Result {
        match self {
            Argument::Imp => write!(f, "No   "),
            Argument::Imm(value) => write!(f, "#{:02X}  ", value),
            Argument::Addr(address, _) => write!(f, "{}", address),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InstructionTemplate {
    pub code_point: u8,
    pub op_code: OpCode,
    pub access_mode: AccessMode,
    pub cycle_count: CycleCount,
    pub extra_cycle: ExtraCycle,
}

impl InstructionTemplate {
    fn from_tuple(code_point: u8, tuple: (OpCode, AccessMode, u8, ExtraCycle)) -> InstructionTemplate {
        InstructionTemplate {
            code_point,
            op_code: tuple.0,
            access_mode: tuple.1,
            cycle_count: CycleCount::new(tuple.2).unwrap(),
            extra_cycle: tuple.3,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, EnumString)]
pub enum OpCode {
    // Logical/Arithmetic
    ORA,
    AND,
    EOR,
    ADC,
    SBC,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    INC,
    INX,
    INY,
    ASL,
    ROL,
    LSR,
    ROR,

    /* Move */
    LDA,
    STA,
    LDX,
    STX,
    LDY,
    STY,
    TAX,
    TXA,
    TAY,
    TYA,
    TSX,
    TXS,
    // Pull accumulator from stack.
    PLA,
    // Push accumulator to stack.
    PHA,
    // Pull status from stack.
    PLP,
    // Push status from stack.
    PHP,

    /* Jump/Flag */
    BPL,
    BMI,
    BVC,
    BVS,
    BCC,
    BCS,
    BNE,
    BEQ,
    BRK,
    RTI,
    JSR,
    RTS,
    JMP,
    BIT,
    CLC,
    SEC,
    CLD,
    SED,
    CLI,
    SEI,
    CLV,
    NOP,

    // Illegal
    SLO,
    RLA,
    SRE,
    RRA,
    SAX,
    LAX,
    DCP,
    ISC,
    ANC,
    ALR,
    ARR,
    XAA,
    AXS,
    AHX,
    SHY,
    SHX,
    TAS,
    LAS,

    JAM,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug)]
pub enum AccessMode {
    Imp,
    Imm,
    ZP,
    ZPX,
    ZPY,
    Abs,
    AbX,
    AbY,
    Rel,
    Ind,
    IzX,
    IzY,
}

impl AccessMode {
    pub fn instruction_length(&self) -> u8 {
        use AccessMode::*;
        match *self {
            Imp => 1,
            Imm | ZP | ZPX | ZPY | Rel | IzX | IzY => 2,
            Abs | AbX | AbY | Ind => 3,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CycleCount {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl CycleCount {
    fn new(value: u8) -> Result<CycleCount, String> {
        use CycleCount::*;
        Ok(match value {
            0 => Zero,
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            6 => Six,
            7 => Seven,
            8 => Eight,
            _ => return Err(format!("CycleCount can't exceed 8 but was {}.", value)),
        })
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ExtraCycle {
    No,
    PB,
}
