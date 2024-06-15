extern crate splitbits;

use splitbits::{splitbits, splitbits_tuple, splitbits_tuple_into, onefield,
                splithex, splithex_tuple, splithex_tuple_into, onehexfield};

#[test]
fn u8() {
    let fields = splitbits!(0b11011101, "aaabbccc");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);
}

// Decimal constants should work.
#[test]
fn decimal() {
    let fields = splitbits!(221, "aaabbccc");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);
}

// Passing in a variable is the most common use case for the macro.
#[test]
fn variable() {
    let value: u8 = 221;
    let fields = splitbits!(value, "aaabbccc");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);
}

// The caller shouldn't have to specify the type of the value passed in.
#[test]
fn variable_type_inferred() {
    let value = 221;
    let fields = splitbits!(value, "aaabbccc");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);
}

// Single bit fields should result in bools, not u8s.
#[test]
fn bools() {
    let fields = splitbits!(0b11010101, "abbbcdee");
    assert_eq!(fields.a, true);
    assert_eq!(fields.b, 0b101u8);
    assert_eq!(fields.c, false);
    assert_eq!(fields.d, true);
    assert_eq!(fields.e, 0b01u8);
}

// Periods hold their place, but the bits they correspond to are ignored.
#[test]
fn periods() {
    let fields = splitbits!(0b11011101, ".aa.bb..");
    assert_eq!(fields.a, 0b10u8);
    assert_eq!(fields.b, 0b11u8);
}

// Spaces are stripped out before processing, whatever place they are in.
#[test]
fn underscores() {
    let fields = splitbits!(0b110_11101, " a aa   b bccc  ");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);
}

#[test]
fn noncontinguous() {
    let fields = splitbits!(0b1101_1101, "abadadda");
    assert_eq!(fields.a, 0b1011u8);
    assert_eq!(fields.b, true);
    assert_eq!(fields.d, 0b110u8);
}

#[test]
fn some_of_everything() {
    let fields = splitbits!(0b1111_1101, ".ad. cdd.");
    assert_eq!(fields.a, true);
    assert_eq!(fields.c, true);
    assert_eq!(fields.d, 0b110u8);
}

// Using the same template twice in the same scope should work (i.e. no struct name conflicts)
#[test]
fn duplicate() {
    let fields = splitbits!(0b11011101, "aaabbccc");
    assert_eq!(fields.a, 0b110u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b101u8);

    let fields2 = splitbits!(0b01001100, "aaabbccc");
    assert_eq!(fields2.a, 0b010u8);
    assert_eq!(fields2.b, 0b01u8);
    assert_eq!(fields2.c, 0b100u8);
}

// LARGE FIELD TESTS

#[test]
fn u16() {
    let fields = splitbits!(
        0b1101110111110001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(fields.a, 0b110111011u16);

    assert_eq!(fields.d, 0b111u8);
    assert_eq!(fields.e, false);
    assert_eq!(fields.f, 0b001u8);
}

#[test]
fn u32() {
    let fields = splitbits!(
        0b1101_1101_1000_0100_0000_0000_1111_1001,
         "aaaa bbbb bbbb bbbb bbbb bbbi jjjj klll",
    );

    assert_eq!(fields.a, 0b1101u8);
    assert_eq!(fields.b, 0b110_1100_0010_0000_0000u32);
    assert_eq!(fields.i, false);
    assert_eq!(fields.j, 0b1111u8);
    assert_eq!(fields.k, true);
    assert_eq!(fields.l, 0b001u8);
}

#[test]
fn u64() {
    let fields = splitbits!(
        0b1101_1101_1000_0000_0000_0000_1111_0001_1101_1101_1000_0000_0000_0000_1101_0001,
         "aaaa bbbb bbbb bbbb bbbb bbbi jjjk klll mmmm nnoo pppq qrrr ssst tuuu uuvw xxxx",
    );
    assert_eq!(fields.a, 0b1101u8);
    assert_eq!(fields.b, 0b110_1100_0000_0000_0000u32);
    assert_eq!(fields.i, false);
    assert_eq!(fields.j, 0b111u8);
    assert_eq!(fields.k, 0b10u8);
    assert_eq!(fields.l, 0b001u8);
    assert_eq!(fields.m, 0b1101u8);
    assert_eq!(fields.n, 0b11u8);
    assert_eq!(fields.o, 0b01u8);
    assert_eq!(fields.p, 0b100u8);
    assert_eq!(fields.q, 0b00u8);
    assert_eq!(fields.r, 0b000u8);
    assert_eq!(fields.s, 0b000u8);
    assert_eq!(fields.t, 0b00u8);
    assert_eq!(fields.u, 0b00011u8);
    assert_eq!(fields.v, false);
    assert_eq!(fields.w, true);
    assert_eq!(fields.x, 0b0001u8);
}

#[test]
fn u128() {
    let fields = splitbits!(
        0b1101_1101_1000_0100_0001_0000_1111_0001_1101_1101_1000_0000_0100_0110_1101_0001_1101_1101_1000_0000_1001_0000_1111_0001_1101_1101_1000_0000_0000_0000_1101_0001,
         "aaaa bbcc cdde efff ffff ff.. .... gg.. ...h hiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iiii iii. pppq qrrr ssst tuuu vvvw wxxx",
    );
    assert_eq!(fields.a, 0b1101u8);
    assert_eq!(fields.b, 0b11u8);
    assert_eq!(fields.c, 0b011u8);
    assert_eq!(fields.d, 0b00u8);
    assert_eq!(fields.e, 0b00u8);
    assert_eq!(fields.f, 0b1_0000_0100u16);
    assert_eq!(fields.g, 0b00u8);
    assert_eq!(fields.h, 0b11u8);
    assert_eq!(fields.i, 0b10_1100_0000_0010_0011_0110_1000_1110_1110_1100_0000_0100_1000_0111_1000_1110_1110u128);
    assert_eq!(fields.p, 0b100u8);
    assert_eq!(fields.q, 0b00u8);
    assert_eq!(fields.r, 0b000u8);
    assert_eq!(fields.s, 0b000u8);
    assert_eq!(fields.t, 0b00u8);
    assert_eq!(fields.u, 0b000u8);
    assert_eq!(fields.v, 0b110u8);
    assert_eq!(fields.w, 0b10u8);
    assert_eq!(fields.x, 0b0001u8);
}

// ALTERNATE VARIABLE SETTING METHODS

#[test]
fn onefield() {
    let field = onefield!(
        0b1101_1101_1000_0100_0000_0000_1111_1001,
         ".... bbbb bbbb bbbb bbbb bbb. .... ....",
    );

    assert_eq!(field, 0b110_1100_0010_0000_0000u32);
}

#[test]
fn tuple() {
    let (greatest, of, all, time) = splitbits_tuple!(
        0b1101110111110001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(greatest, 0b110111011u16);
    assert_eq!(of, 0b111u8);
    assert_eq!(all, false);
    assert_eq!(time, 0b001u8);
}

#[test]
fn tuple_existing_variables() {
    let greatest; let of; let all; let time;
    (greatest, of, all, time) = splitbits_tuple!(
        0b1101110111110001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(greatest, 0b110111011u16);
    assert_eq!(of, 0b111u8);
    assert_eq!(all, false);
    assert_eq!(time, 0b001u8);
}

#[test]
fn tuple_existing_types() {
    let greatest: u128; let of: u8; let all: u32; let time: u16;
    (greatest, of, all, time) = splitbits_tuple_into!(
        0b1101110111110001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(greatest, 0b110111011u128);
    assert_eq!(of, 0b111u8);
    assert_eq!(all, 0u32);
    assert_eq!(time, 0b001u16);

    let (greatest, of, all, time): (u128, u8, u32, u16) = splitbits_tuple_into!(
        0b1101110111110001,
         "aaaaaaaaadddefff",
    );
    assert_eq!(greatest, 0b110111011u128);
    assert_eq!(of, 0b111u8);
    assert_eq!(all, 0u32);
    assert_eq!(time, 0b001u16);
}

// HEXADECIMAL

#[test]
fn hex() {
    // IPV6
    let fields = splithex!(
        0x2001_0db8_85a3_0000_0000_8a2e_0370_7334,
         "aaaa bbbb cccc dddd eeee ffff gggg hhhh",
    );
    assert_eq!(fields.a, 0x2001u16);
    assert_eq!(fields.b, 0x0db8u16);
    assert_eq!(fields.c, 0x85a3u16);
    assert_eq!(fields.d, 0x0000u16);
    assert_eq!(fields.e, 0x0000u16);
    assert_eq!(fields.f, 0x8a2eu16);
    assert_eq!(fields.g, 0x0370u16);
    assert_eq!(fields.h, 0x7334u16);
}

#[test]
fn hex_tuple() {
    // IPV6
    let (everybody, wants, to, rule, the, world) = splithex_tuple!(
        0x2001_0db8_85a3_0000_0000_8a2e_0370_7334,
         "aaaa bbbb cccc .... eeee ffff hhhh hhhh",
    );
    assert_eq!(everybody, 0x2001u16);
    assert_eq!(wants, 0x0db8u16);
    assert_eq!(to, 0x85a3u16);
    assert_eq!(rule, 0x0000u16);
    assert_eq!(the, 0x8a2eu16);
    assert_eq!(world, 0x0370_7334u32);
}

#[test]
fn hex_tuple_into() {
    // IPV6
    let (everybody, wants, to, rule, the, world): (u16, u128, u8, u32, u128, u64) =
        splithex_tuple_into!(
            0x2101_0db8_85a3_0000_0000_8a2e_0370_7334,
             "abbb bbbb bbbb bbbb bbbe ffff ffff hhii",
        );
    assert_eq!(everybody, 0x2u16);
    assert_eq!(wants, 0x10_10db_885a_3000_0000u128);
    assert_eq!(to, 0x0u8);
    assert_eq!(rule, 0x8a2e_0370u32);
    assert_eq!(the, 0x73u128);
    assert_eq!(world, 0x34u64);
}

#[test]
fn onehex() {
    // IPV6
    let value = onehexfield!(
        0x2001_0db8_85a3_0000_0000_8a2e_0370_7334,
         ".... bbbb .... .... .... .... .... ....",
    );
    assert_eq!(value, 0x0db8u16);
}
