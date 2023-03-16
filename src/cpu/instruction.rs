use lazy_static::lazy_static;
use strum_macros::EnumString;

use crate::cpu::cpu::Cpu;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::memory::CpuMemory;

lazy_static! {
    pub static ref INSTRUCTION_TEMPLATES: [InstructionTemplate; 256] = instruction_templates();
}

#[rustfmt::skip]
fn instruction_templates() -> [InstructionTemplate; 256] {
    use OpCode::*;
    use AccessMode::*;

    let jam = (JAM, Imp);
    let codes: [[(OpCode, AccessMode); 8]; 32] = [
        /*00*/      /*20*/     /*40*/     /*60*/     /*80*/     /*A0*/     /*C0*/     /*E0*/
/*+00*/ [(BRK,Imp), (JSR,Abs), (RTI,Imp), (RTS,Imp), (NOP,Imm), (LDY,Imm), (CPY,Imm), (CPX,Imm)],
/*+01*/ [(ORA,IzX), (AND,IzX), (EOR,IzX), (ADC,IzX), (STA,IzX), (LDA,IzX), (CMP,IzX), (SBC,IzX)],
/*+02*/ [jam      , jam      , jam      , jam      , (NOP,Imm), (LDX,Imm), (NOP,Imm), (NOP,Imm)],
/*+03*/ [(SLO,IzX), (RLA,IzX), (SRE,IzX), (RRA,IzX), (SAX,IzX), (LAX,IzX), (DCP,IzX), (ISC,IzX)],
/*+04*/ [(NOP,ZP ), (BIT,ZP ), (NOP,ZP ), (NOP,ZP ), (STY,ZP ), (LDY,ZP ), (CPY,ZP ), (CPX,ZP )],
/*+05*/ [(ORA,ZP ), (AND,ZP ), (EOR,ZP ), (ADC,ZP ), (STA,ZP ), (LDA,ZP ), (CMP,ZP ), (SBC,ZP )],
/*+06*/ [(ASL,ZP ), (ROL,ZP ), (LSR,ZP ), (ROR,ZP ), (STX,ZP ), (LDX,ZP ), (DEC,ZP ), (INC,ZP )],
/*+07*/ [(SLO,ZP ), (RLA,ZP ), (SRE,ZP ), (RRA,ZP ), (SAX,ZP ), (LAX,ZP ), (DCP,ZP ), (ISC,ZP )],
/*+08*/ [(PHP,Imp), (PLP,Imp), (PHA,Imp), (PLA,Imp), (DEY,Imp), (TAY,Imp), (INY,Imp), (INX,Imp)],
/*+09*/ [(ORA,Imm), (AND,Imm), (EOR,Imm), (ADC,Imm), (NOP,Imm), (LDA,Imm), (CMP,Imm), (SBC,Imm)],
/*+0A*/ [(ASL,Imp), (ROL,Imp), (LSR,Imp), (ROR,Imp), (TXA,Imp), (TAX,Imp), (DEX,Imp), (NOP,Imp)],
/*+0B*/ [(ANC,Imm), (ANC,Imm), (ALR,Imm), (ARR,Imm), (XAA,Imm), (LAX,Imm), (AXS,Imm), (SBC,Imm)],
/*+0C*/ [(NOP,Abs), (BIT,Abs), (JMP,Abs), (JMP,Ind), (STY,Abs), (LDY,Abs), (CPY,Abs), (CPX,Abs)],
/*+0D*/ [(ORA,Abs), (AND,Abs), (EOR,Abs), (ADC,Abs), (STA,Abs), (LDA,Abs), (CMP,Abs), (SBC,Abs)],
/*+0E*/ [(ASL,Abs), (ROL,Abs), (LSR,Abs), (ROR,Abs), (STX,Abs), (LDX,Abs), (DEC,Abs), (INC,Abs)],
/*+0F*/ [(SLO,Abs), (RLA,Abs), (SRE,Abs), (RRA,Abs), (SAX,Abs), (LAX,Abs), (DCP,Abs), (ISC,Abs)],

/*+10*/ [(BPL,Rel), (BMI,Rel), (BVC,Rel), (BVS,Rel), (BCC,Rel), (BCS,Rel), (BNE,Rel), (BEQ,Rel)],
/*+11*/ [(ORA,IzY), (AND,IzY), (EOR,IzY), (ADC,IzY), (STA,IzY), (LDA,IzY), (CMP,IzY), (SBC,IzY)],
/*+12*/ [jam      , jam      , jam      , jam      , jam      , jam      , jam      , jam      ],
/*+13*/ [(SLO,IzY), (RLA,IzY), (SRE,IzY), (RRA,IzY), (AHX,IzY), (LAX,IzY), (DCP,IzY), (ISC,IzY)],
/*+14*/ [(NOP,ZPX), (NOP,ZPX), (NOP,ZPX), (NOP,ZPX), (STY,ZPX), (LDY,ZPX), (NOP,ZPX), (NOP,ZPX)],
/*+15*/ [(ORA,ZPX), (AND,ZPX), (EOR,ZPX), (ADC,ZPX), (STA,ZPX), (LDA,ZPX), (CMP,ZPX), (SBC,ZPX)],
/*+16*/ [(ASL,ZPX), (ROL,ZPX), (LSR,ZPX), (ROR,ZPX), (STX,ZPY), (LDX,ZPY), (DEC,ZPX), (INC,ZPX)],
/*+17*/ [(SLO,ZPX), (RLA,ZPX), (SRE,ZPX), (RRA,ZPX), (SAX,ZPY), (LAX,ZPY), (DCP,ZPX), (ISC,ZPX)],
/*+18*/ [(CLC,Imp), (SEC,Imp), (CLI,Imp), (SEI,Imp), (TYA,Imp), (CLV,Imp), (CLD,Imp), (SED,Imp)],
/*+19*/ [(ORA,AbY), (AND,AbY), (EOR,AbY), (ADC,AbY), (STA,AbY), (LDA,AbY), (CMP,AbY), (SBC,AbY)],
/*+1A*/ [(NOP,Imp), (NOP,Imp), (NOP,Imp), (NOP,Imp), (TXS,Imp), (TSX,Imp), (NOP,Imp), (NOP,Imp)],
/*+1B*/ [(SLO,AbY), (RLA,AbY), (SRE,AbY), (RRA,AbY), (TAS,AbY), (LAS,AbY), (DCP,AbY), (ISC,AbY)],
/*+1C*/ [(NOP,AbX), (NOP,AbX), (NOP,AbX), (NOP,AbX), (SHY,AbX), (LDY,AbX), (NOP,AbX), (NOP,AbX)],
/*+1D*/ [(ORA,AbX), (AND,AbX), (EOR,AbX), (ADC,AbX), (STA,AbX), (LDA,AbX), (CMP,AbX), (SBC,AbX)],
/*+1E*/ [(ASL,AbX), (ROL,AbX), (LSR,AbX), (ROR,AbX), (SHX,AbY), (LDX,AbY), (DEC,AbX), (INC,AbX)],
/*+1F*/ [(SLO,AbX), (RLA,AbX), (SRE,AbX), (RRA,AbX), (AHX,AbY), (LAX,AbY), (DCP,AbX), (ISC,AbX)],
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
}

impl Instruction {
    pub fn at_address(cpu: &Cpu, start_address: CpuAddress, mem: &CpuMemory) -> String {
        let op_code = mem.peek(start_address).unwrap();
        let template = INSTRUCTION_TEMPLATES[op_code as usize];
        let low = mem.peek(start_address.offset(1)).expect("Read open bus.");
        let high = mem.peek(start_address.offset(2)).expect("Read open bus.");

        let mut argument_string = String::new();
        use AccessMode::*;
        match template.access_mode {
            Imp => {},
            Imm => {
                argument_string.push_str(&format!("#${low:02X}"));
            }
            ZP => {
                let address = CpuAddress::zero_page(low);
                let value = mem.peek(address).unwrap_or(0);
                argument_string.push_str(&format!("${low:02X} = {value:02X}"));
            }
            ZPX => {
                argument_string.push_str(&format!("${low:02X},X"));
                let _address = CpuAddress::zero_page(low.wrapping_add(cpu.x_index()));
            }
            ZPY => {
                argument_string.push_str(&format!("${low:02X},Y"));
                let _address = CpuAddress::zero_page(low.wrapping_add(cpu.y_index()));
            }
            Abs => {
                let address = CpuAddress::from_low_high(low, high);
                argument_string.push_str(&format!("${high:02X}{low:02X}"));
                if template.op_code != OpCode::JMP {
                    let value = mem.peek(address).unwrap_or(0);
                    argument_string.push_str(&format!(" = {value:02X}"));
                }
            }
            AbX => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.x_index());
                let value = mem.peek(address).unwrap_or(0);
                argument_string.push_str(&format!("${high:02X}{low:02X},X = {value:02X}"));
            }
            AbY => {
                let start_address = CpuAddress::from_low_high(low, high);
                let address = start_address.advance(cpu.y_index());
                let value = mem.peek(address).unwrap_or(0);
                argument_string.push_str(&format!("${high:02X}{low:02X},Y = {value:02X}"));
            }
            Rel => {
                let address = start_address
                    .offset(low as i8)
                    .advance(template.access_mode.instruction_length());
                argument_string.push_str(&format!("${:04X}", address.to_raw()));
            }
            Ind => {
                let first = CpuAddress::from_low_high(low, high);
                let second = CpuAddress::from_low_high(low.wrapping_add(1), high);
                let _address = CpuAddress::from_low_high(
                    mem.peek(first).unwrap_or(0),
                    mem.peek(second).unwrap_or(0),
                );
            }
            IzX => {
                argument_string.push_str(&format!("(${low:02X}),X ="));
                let low = low.wrapping_add(cpu.x_index());
                let _address = CpuAddress::from_low_high(
                    mem.peek(CpuAddress::zero_page(low)).expect("Read open bus."),
                    mem.peek(CpuAddress::zero_page(low.wrapping_add(1))).expect("Read open bus."),
                );
            }
            IzY => {
                argument_string.push_str(&format!("(${low:02X}),Y ="));
                let start_address = CpuAddress::from_low_high(
                    mem.peek(CpuAddress::zero_page(low)).expect("Read open bus."),
                    mem.peek(CpuAddress::zero_page(low.wrapping_add(1))).expect("Read open bus."),
                );
                // TODO: Should this wrap around just the current page?
                let _address = start_address.advance(cpu.y_index());
            }
        };

        let instr_bytes = match template.access_mode.instruction_length() {
            1 => format!("{:02X}      ", template.code_point),
            2 => format!("{:02X} {:02X}    ", template.code_point, low),
            3 => format!("{:02X} {:02X} {:02X} ", template.code_point, low, high),
            _ => unreachable!(),
        };

        let text = format!(
            "{:04X}  {:<9} {:?} {:28}",
            start_address.to_raw(),
            instr_bytes,
            template.op_code,
            argument_string,
        );

        text
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InstructionTemplate {
    pub code_point: u8,
    pub op_code: OpCode,
    pub access_mode: AccessMode,
}

impl InstructionTemplate {
    pub fn from_code_point(code_point: u8) -> InstructionTemplate {
        INSTRUCTION_TEMPLATES[code_point as usize]
    }

    fn from_tuple(
        code_point: u8,
        tuple: (OpCode, AccessMode),
    ) -> InstructionTemplate {
        InstructionTemplate {
            code_point,
            op_code: tuple.0,
            access_mode: tuple.1,
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

    // Undocumented.
    SLO,
    RLA,
    SRE,
    RRA,
    SAX,
    LAX,
    DCP,
    ISC,
    // a.k.a. AAC
    ANC,
    ALR,
    ARR,
    XAA,
    AXS,
    // a.k.a. SHA
    AHX,
    // a.k.a. SYA
    SHY,
    // a.k.a. SXA
    SHX,
    // a.k.a. ANE
    TAS,
    LAS,

    JAM,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AccessMode {
    Imp,
    Imm,
    ZP,
    ZPX,
    ZPY,
    // Absolute addressing.
    Abs,
    AbX,
    AbY,
    Rel,
    Ind,
    IzX,
    IzY,
}

impl AccessMode {
    pub fn instruction_length(self) -> u8 {
        use AccessMode::*;
        match self {
            Imp => 1,
            Imm | ZP | ZPX | ZPY | Rel | IzX | IzY => 2,
            Abs | AbX | AbY | Ind => 3,
        }
    }
}
