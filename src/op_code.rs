pub fn op_codes() -> [(OpCode, AccessMode, u8); 256] {
    use OpCode::*;
    use AccessMode::*;

    /* 0x00        0x01         0x02         0x03         0x04         0x05         0x06         0x07     */
    [ (Brk,Imp,7), (Ora,IzX,6), (Jam,Imp,0), (Slo,IzX,8), (Nop, ZP,3), (Ora, ZP,3), (Asl, ZP,5), (Slo, ZP,5)
    /* 0x08        0x09         0x0A         0x0B         0x0C         0x0D         0x0E         0x0F     */
    , (Php,Imp,3), (Ora,Imm,2), (Asl,Imp,2), (Anc,Imm,2), (Nop,Abs,4), (Ora,Abs,4), (Asl,Abs,6), (Slo,Abs,6)
    /* 0x10        0x11         0x12         0x13         0x14         0x15         0x16         0x17     */
    , (Bpl,Rel,2), (Ora,IzY,5), (Jam,Imp,0), (Slo,IzY,8), (Nop,ZPX,4), (Ora,ZPX,4), (Asl,ZPX,6), (Slo,ZPX,6)
    /* 0x18        0x19         0x1A         0x1B         0x1C         0x1D         0x1E         0x1F     */
    , (Clc,Imp,2), (Ora,AbY,4), (Nop,Imp,2), (Slo,AbY,7), (Nop,AbX,4), (Ora,AbX,4), (Asl,AbX,7), (Slo,AbX,7)
    /* 0x20        0x21         0x22         0x23         0x24         0x25         0x26         0x27     */
    , (Jsr,Abs,6), (And,IzX,6), (Jam,Imp,0), (Rla,IzX,8), (Bit, ZP,3), (And, ZP,3), (Rol, ZP,5), (Rla, ZP,5)
    /* 0x28        0x29         0x2A         0x2B         0x2C         0x2D         0x2E         0x2F     */
    , (Plp,Imp,4), (And,Imm,2), (Rol,Imp,2), (Anc,Imm,2), (Bit,Abs,4), (And,Abs,4), (Rol,Abs,6), (Rla,Abs,6)
    /* 0x30        0x31         0x32         0x33         0x34         0x35         0x36         0x37     */
    , (Bmi,Rel,2), (And,IzY,5), (Jam,Imp,0), (Rla,IzY,8), (Nop,ZPX,4), (And,ZPX,4), (Rol,ZPX,6), (Rla,ZPX,6)
    /* 0x38        0x39         0x3A         0x3B         0x3C         0x3D         0x3E         0x3F     */
    , (Sec,Imp,2), (And,AbY,4), (Nop,Imp,2), (Rla,AbY,7), (Nop,AbX,4), (And,AbX,4), (Rol,AbX,7), (Rla,AbX,7)
    /* 0x40        0x41         0x42         0x43         0x44         0x45         0x46         0x47     */
    , (Rti,Imp,6), (Eor,IzX,6), (Jam,Imp,0), (Sre,IzX,8), (Nop, ZP,3), (Eor, ZP,3), (Lsr, ZP,5), (Sre, ZP,5)
    /* 0x48        0x49         0x4A         0x4B         0x4C         0x4D         0x4E         0x4F     */
    , (Pha,Imp,3), (Eor,Imm,2), (Lsr,Imp,2), (Alr,Imm,2), (Jmp,Abs,3), (Eor,Abs,4), (Lsr,Abs,6), (Sre,Abs,6)
    /* 0x50        0x51         0x52         0x53         0x54         0x55         0x56         0x57     */
    , (Bvc,Rel,2), (Eor,IzY,5), (Jam,Imp,0), (Sre,IzY,8), (Nop,ZPX,4), (Eor,ZPX,4), (Lsr,ZPX,6), (Sre,ZPX,6)
    /* 0x58        0x59         0x5A         0x5B         0x5C         0x5D         0x5E         0x5F     */
    , (Cli,Imp,2), (Eor,AbY,4), (Nop,Imp,2), (Sre,AbY,7), (Nop,AbX,4), (Eor,AbX,4), (Lsr,AbX,7), (Sre,AbX,7)
    /* 0x60        0x61         0x62         0x63         0x64         0x65         0x66         0x67     */
    , (Rts,Imp,6), (Adc,IzX,6), (Jam,Imp,0), (Rra,IzX,8), (Nop, ZP,3), (Adc, ZP,3), (Ror, ZP,5), (Rra, ZP,5)
    /* 0x68        0x69         0x6A         0x6B         0x6C         0x6D         0x6E         0x6F     */
    , (Pla,Imp,4), (Adc,Imm,2), (Ror,Imp,2), (Arr,Imm,2), (Jmp,Ind,5), (Adc,Abs,4), (Ror,Abs,6), (Rra,Abs,6)
    /* 0x70        0x71         0x72         0x73         0x74         0x75         0x76         0x77     */
    , (Bvs,Rel,2), (Adc,IzY,5), (Jam,Imp,0), (Rra,IzY,8), (Nop,ZPX,4), (Adc,ZPX,4), (Ror,ZPX,6), (Rra,ZPX,6)
    /* 0x78        0x79         0x7A         0x7B         0x7C         0x7D         0x7E         0x7F     */
    , (Sei,Imp,2), (Adc,AbY,4), (Nop,Imp,2), (Rra,AbY,7), (Nop,AbX,4), (Adc,AbX,4), (Ror,AbX,7), (Rra,AbX,7)
    /* 0x80        0x81         0x82         0x83         0x84         0x85         0x86         0x87     */
    , (Nop,Imm,2), (Sta,IzX,6), (Nop,Imm,2), (Sax,IzX,6), (Sty, ZP,3), (Sta, ZP,3), (Stx, ZP,3), (Sax, ZP,3)
    /* 0x88        0x89         0x8A         0x8B         0x8C         0x8D         0x8E         0x8F     */
    , (Dey,Imp,2), (Nop,Imm,2), (Txa,Imp,2), (Xaa,Imm,2), (Sty,Abs,4), (Sta,Abs,4), (Stx,Abs,4), (Sax,Abs,4)
    /* 0x90        0x91         0x92         0x93         0x94         0x95         0x96         0x97     */
    , (Bcc,Rel,2), (Sta,IzY,6), (Jam,Imp,0), (Ahx,IzY,6), (Sty,ZPX,4), (Sta,ZPX,4), (Stx,ZPY,4), (Sax,ZPY,4)
    /* 0x98        0x99         0x9A         0x9B         0x9C         0x9D         0x9E         0x9F     */
    , (Tya,Imp,2), (Sta,AbY,5), (Txs,Imp,2), (Tas,AbY,5), (Shy,AbX,5), (Sta,AbX,5), (Shx,AbY,5), (Ahx,AbY,5)
    /* 0xA0        0xA1         0xA2         0xA3         0xA4         0xA5         0xA6         0xA7     */
    , (Ldy,Imm,2), (Lda,IzX,6), (Ldx,Imm,2), (Lax,IzX,6), (Ldy, ZP,3), (Lda, ZP,3), (Ldx, ZP,3), (Lax, ZP,3)
    /* 0xA8        0xA9         0xAA         0xAB         0xAC         0xAD         0xAE         0xAF     */
    , (Tay,Imp,2), (Lda,Imm,2), (Tax,Imp,2), (Lax,Imm,2), (Ldy,Abs,4), (Lda,Abs,4), (Ldx,Abs,4), (Lax,Abs,4)
    /* 0xB0        0xB1         0xB2         0xB3         0xB4         0xB5         0xB6         0xB7     */
    , (Bcs,Rel,2), (Lda,IzY,5), (Jam,Imp,0), (Lax,IzY,5), (Ldy,ZPX,4), (Lda,ZPX,4), (Ldx,ZPY,4), (Lax,ZPY,4)
    /* 0xB8        0xB9         0xBA         0xBB         0xBC         0xBD         0xBE         0xBF     */
    , (Clv,Imp,2), (Lda,AbY,4), (Tsx,Imp,2), (Las,AbY,4), (Ldy,AbY,4), (Lda,AbX,4), (Ldx,AbY,4), (Lax,AbY,4)
    /* 0xC0        0xC1         0xC2         0xC3         0xC4         0xC5         0xC6         0xC7     */
    , (Cpy,Imm,2), (Cmp,IzX,6), (Nop,Imm,2), (Dcp,IzX,8), (Cpy, ZP,3), (Cmp, ZP,3), (Dec, ZP,5), (Dcp, ZP,5)
    /* 0xC8        0xC9         0xCA         0xCB         0xCC         0xCD         0xCE         0xCF     */
    , (Iny,Imp,2), (Cmp,Imm,2), (Dex,Imp,2), (Axs,Imm,2), (Cpy,Abs,4), (Cmp,Abs,4), (Dec,Abs,6), (Dcp,Abs,6)
    /* 0xD0        0xD1         0xD2         0xD3         0xD4         0xD5         0xD6         0xD7     */
    , (Bne,Rel,2), (Cmp,IzY,5), (Jam,Imp,0), (Dcp,IzY,8), (Nop,ZPX,4), (Cmp,ZPX,4), (Dec,ZPX,4), (Dcp,ZPX,6)
    /* 0xD8        0xD9         0xDA         0xDB         0xDC         0xDD         0xDE         0xDF     */
    , (Cld,Imp,2), (Cmp,AbY,4), (Nop,Imp,2), (Dcp,AbY,7), (Nop,AbX,4), (Cmp,AbX,4), (Dec,AbX,7), (Dcp,AbX,7)
    /* 0xE0        0xE1         0xE2         0xE3         0xE4         0xE5         0xE6         0xE7     */
    , (Cpx,Imm,2), (Sbc,IzX,6), (Nop,Imm,2), (Isc,IzX,8), (Cpx, ZP,3), (Sbc, ZP,3), (Inc, ZP,5), (Isc, ZP,5)
    /* 0xE8        0xE9         0xEA         0xEB         0xEC         0xED         0xEE         0xEF     */
    , (Inx,Imp,2), (Sbc,Imm,2), (Nop,Imp,2), (Sbc,Imm,2), (Cpx,Abs,4), (Sbc,Abs,4), (Inc,Abs,6), (Isc,Abs,6)
    /* 0xF0        0xF1         0xF2         0xF3         0xF4         0xF5         0xF6         0xF7     */
    , (Beq,Rel,2), (Sbc,IzY,5), (Jam,Imp,2), (Isc,IzY,8), (Nop,ZPX,4), (Sbc,ZPX,4), (Inc,ZPX,6), (Isc,ZPX,6)
    /* 0xF8        0xF9         0xFA         0xFB         0xFC         0xFD         0xFE         0xFF     */
    , (Sed,Imp,2), (Sbc,AbY,4), (Nop,Imp,2), (Isc,AbY,7), (Nop,AbX,4), (Sbc,AbX,4), (Inc,AbX,7), (Isc,AbX,7)
    ]
}

pub enum OpCode {
    // Logical/Arithmetic
    Ora,
    And,
    Eor,
    Adc,
    Sbc,
    Cmp,
    Cpx,
    Cpy,
    Dec,
    Dex,
    Dey,
    Inc,
    Inx,
    Iny,
    Asl,
    Rol,
    Lsr,
    Ror,

    // Move
    Lda,
    Sta,
    Ldx,
    Stx,
    Ldy,
    Sty,
    Tax,
    Txa,
    Tay,
    Tya,
    Tsx,
    Txs,
    Pla,
    Pha,
    Plp,
    Php,

    // Jump/Flag
    Bpl,
    Bmi,
    Bvc,
    Bvs,
    Bcc,
    Bcs,
    Bne,
    Beq,
    Brk,
    Rti,
    Jsr,
    Rts,
    Jmp,
    Bit,
    Clc,
    Sec,
    Cld,
    Sed,
    Cli,
    Sei,
    Clv,
    Nop,

    // Illegal
    Slo,
    Rla,
    Sre,
    Rra,
    Sax,
    Lax,
    Dcp,
    Isc,
    Anc,
    Alr,
    Arr,
    Xaa,
    Axs,
    Ahx,
    Shy,
    Shx,
    Tas,
    Las,
    Jam,
}

pub enum AccessMode {
    Imp,
    Acc,
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
