extern crate proc_macro;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

use itertools::Itertools;
use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use syn::{Token, Expr, Lit};
use syn::parse::Parser;
use syn::punctuated::Punctuated;

// TODO:
// * Implement combinebits.
// * Implement splitbits_then_combine
// * combinebits_hexadecimal
// * Allow const variable templates.
// * Allow passing minimum variable size.
// * Allow non-const variable templates (as a separate macro?).
// * Better error messages.
// * Remove itertools dependency.
// * Allow non-standard template lengths.
#[proc_macro]
pub fn splitbits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Binary, Precision::Standard)
}

#[proc_macro]
pub fn splitbits_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Binary, Precision::Ux)
}

#[proc_macro]
pub fn splithex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Hexadecimal, Precision::Standard)
}

#[proc_macro]
pub fn splithex_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Hexadecimal, Precision::Ux)
}

#[proc_macro]
pub fn splitbits_tuple(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_base(input, Base::Binary, Precision::Standard)
}

#[proc_macro]
pub fn splitbits_tuple_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_base(input, Base::Binary, Precision::Ux)
}

#[proc_macro]
pub fn splithex_tuple(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_base(input, Base::Hexadecimal, Precision::Standard)
}

#[proc_macro]
pub fn splithex_tuple_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_base(input, Base::Hexadecimal, Precision::Ux)
}

#[proc_macro]
pub fn splitbits_tuple_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_into_base(input, Base::Binary, Precision::Standard)
}

#[proc_macro]
pub fn splitbits_tuple_into_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_into_base(input, Base::Binary, Precision::Ux)
}

#[proc_macro]
pub fn splithex_tuple_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_into_base(input, Base::Hexadecimal, Precision::Standard)
}

#[proc_macro]
pub fn splithex_tuple_into_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_tuple_into_base(input, Base::Hexadecimal, Precision::Ux)
}

#[proc_macro]
pub fn onefield(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    onefield_base(input, Base::Binary, Precision::Standard)
}

#[proc_macro]
pub fn onefield_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    onefield_base(input, Base::Binary, Precision::Ux)
}

#[proc_macro]
pub fn onehexfield(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    onefield_base(input, Base::Hexadecimal, Precision::Standard)
}

#[proc_macro]
pub fn onehexfield_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    onefield_base(input, Base::Hexadecimal, Precision::Ux)
}

// TODO:
// * Upsize output.
// * Repeated output fields.
#[proc_macro]
pub fn splitbits_then_combine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let base = Base::Binary;
    let precision = Precision::Standard;

    let parts = Parser::parse2(
        Punctuated::<Expr, Token![,]>::parse_terminated,
        input.clone().into(),
    ).unwrap();
    let parts: Vec<Expr> = parts.into_iter().collect();
    assert!(parts.len() >= 3);
    assert!(parts.len() % 2 == 1);

    let mut fields = Vec::new();
    for i in 0..parts.len() / 2 {
        let value = parts[2 * i].clone();
        let template = Template::from_expr(&parts[2 * i + 1], base, precision);
        fields = Field::merge(fields, template.extract_fields(&value));
    }

    let target = Template::from_expr(&parts[parts.len() - 1], base, precision);
    let result = target.substitute_fields(fields);
    result.into()
}

fn splitbits_base(input: proc_macro::TokenStream, base: Base, precision: Precision) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into(), base, precision);
    let fields = template.extract_fields(&value);

    let struct_name = template.to_struct_name();
    let names: Vec<_> = fields.iter().map(|field| field.name()).collect();
    let types: Vec<_> = fields.iter().map(|field| field.t()).collect();
    let values: Vec<TokenStream> = fields.iter().map(|field| field.to_token_stream()).collect();
    let result = quote! {
        {
            struct #struct_name {
                #(#names: #types,)*
            }

            #struct_name {
                #(#names: #values,)*
            }
        }
    };

    result.into()
}

fn splitbits_tuple_base(input: proc_macro::TokenStream, base: Base, precision: Precision) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into(), base, precision);
    let fields = template.extract_fields(&value);
    let values: Vec<TokenStream> = fields.iter().map(|field| field.to_token_stream()).collect();

    let result = quote! {
        (#(#values,)*)
    };

    result.into()
}

fn splitbits_tuple_into_base(input: proc_macro::TokenStream, base: Base, precision: Precision) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into(), base, precision);
    let fields = template.extract_fields(&value);
    let values: Vec<TokenStream> = fields.iter().map(|field| field.to_token_stream()).collect();

    let result = quote! {
        (#((#values).into(),)*)
    };

    result.into()
}

fn onefield_base(input: proc_macro::TokenStream, base: Base, precision: Precision) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into(), base, precision);
    let fields = template.extract_fields(&value);
    assert_eq!(fields.len(), 1);
    fields[0].to_token_stream().into()
}

fn parse_input(item: TokenStream, base: Base, precision: Precision) -> (Expr, Template) {
    let parts = Parser::parse2(
        Punctuated::<Expr, Token![,]>::parse_terminated,
        item.clone().into(),
    ).unwrap();
    let parts: Vec<Expr> = parts.into_iter().collect();
    assert_eq!(parts.len(), 2);

    let value = parts[0].clone();
    let template = Template::from_expr(&parts[1], base, precision);
    (value, template)
}

struct Template {
    input_type: Type,
    precision: Precision,
    names: Vec<Name>,
    locations_by_name: Vec<(Name, Vec<Location>)>,
}

impl Template {
    fn from_expr(template: &Expr, base: Base, precision: Precision) -> Template {
        let Expr::Lit(template) = template.clone() else { panic!() };
        let Lit::Str(template) = template.lit else { panic!() };
        let template: Vec<char> = template.value().chars()
            // Spaces are only for human-readability.
            .filter(|&c| c != ' ')
            .collect();

        assert!(template.len() <= 128);

        let names: Vec<Name> = template.clone()
            .iter()
            // Periods aren't names, they are placeholders for ignored bits.
            .filter(|&&n| n != '.')
            .map(|&name| { assert!(name.is_ascii_lowercase()); name })
            .unique()
            .collect();

        // Each template char needs to be repeated if we aren't working in base 2.
        let template: Vec<_> = template.iter()
            .cloned()
            .flat_map(|n| std::iter::repeat(n).take(base.bits_per_digit()))
            .collect();

        let input_type = Type::for_template(template.len() as u8);
        let mut locations_by_name: Vec<(Name, Vec<Location>)> = Vec::new();
        for &name in &names {
            let locations: Vec<Location> = template.iter()
                .rev()
                .enumerate()
                .chunk_by(|(_, &n)| n)
                .into_iter()
                .filter_map(|(n, p)| if n == name { Some(p) } else { None })
                .map(|p| {
                    let p: Vec<_> = p.collect();
                    Location {
                        len: p.len() as u8,
                        mask_offset: p[0].0 as u8,
                    }
                })
                .collect();
            locations_by_name.push((name, locations));
        }

        Template { input_type, precision, names, locations_by_name }
    }

    fn names(&self) -> &[char] {
        &self.names
    }

    fn extract_fields(&self, input: &Expr) -> Vec<Field> {
        self.locations_by_name.iter()
            .map(|(name, locations)| Field::new(*name, self.input_type, input, self.precision, &locations))
            .collect()
    }

    // WRONG ASSUMPTIONS:
    // * Each name only has a single segment.
    // * Each argument field isn't duplicated.
    // * Lengths match between inputs and outputs.
    fn substitute_fields(&self, fields: Vec<Field>) -> TokenStream {
        let fields: BTreeMap<_, _> = fields.iter()
            .map(|field| (field.name, field))
            .collect();
        let mut field_streams = Vec::new();
        for (name, locations) in &self.locations_by_name {
            assert_eq!(locations.len(), 1);
            let Location {len, mask_offset} = locations[0];
            let mut field = fields[name].clone();
            assert!(len <= field.t.size());
            field.shift_left(mask_offset);
            field.widen(self.input_type);
            field_streams.push(field.to_token_stream());
        }

        quote! { #(#field_streams)|* }
    }

    fn to_struct_name(&self) -> Ident {
        let struct_name_suffix: String = self.names().iter()
            // Underscores work in struct names, periods do not.
            .map(|&c| if c == '.' { '_' } else { c })
            .collect();
        format_ident!("{}", format!("Fields·{}", struct_name_suffix))
    }
}

type Name = char;

struct Location {
    len: u8,
    mask_offset: u8,
}

#[derive(Clone)]
struct Field {
    name: char,
    segments: Vec<Segment>,
    t: Type,
}

impl Field {
    fn new(name: char, input_type: Type, input: &Expr, precision: Precision, locations: &[Location]) -> Field {
        let mut segment_offset = 0;
        let mut segments = Vec::new();
        for &Location { len, mask_offset } in locations {
            let mut mask: u128 = 2u128.pow(len as u32) - 1;
            mask <<= mask_offset;

            let mut segment = Segment::new(input.clone(), input_type, mask);
            segment.shift_right(mask_offset);
            segment.shift_left(segment_offset);
            segment_offset += len;
            segments.push(segment);
        }

        let bit_count = locations.iter().map(|location| location.len).sum();
        let t = Type::for_field(bit_count, precision);
        Field { name, segments, t }
    }

    fn to_token_stream(&self) -> TokenStream {
        let t = self.t();
        let mut segments = self.segments.iter().map(Segment::to_value);
        if self.t == Type::BOOL {
            let segment = segments.next().unwrap();
            quote! { (#segment) != 0 }
        } else {
            quote! { #t::try_from(#(#segments)|*).unwrap() }
        }
    }

    fn merge(upper: Vec<Field>, lower: Vec<Field>) -> Vec<Field> {
        let lower_map: BTreeMap<_, _> = lower.iter()
            .map(|field| (field.name, field))
            .collect();
        let mut result = Vec::new();
        for u in &upper {
            if let Some(l) = lower_map.get(&u.name) {
                result.push(u.concat(&l));
            } else {
                result.push(u.clone());
            }
        }

        let upper_map: BTreeMap<_, _> = upper.iter()
            .map(|field| (field.name, field))
            .collect();
        for l in lower {
            if !upper_map.contains_key(&l.name) {
                result.push(l);
            }
        }

        result
    }

    fn concat(&self, lower: &Field) -> Field {
        assert_eq!(self.name, lower.name);

        let mut new_segments = Vec::new();
        for segment in &self.segments {
            let mut new_segment = segment.clone();
            new_segment.shift_left(lower.t.0);
            new_segments.push(new_segment);
        }

        for segment in &lower.segments {
            new_segments.push(segment.clone());
        }

        Field {
            name: self.name,
            segments: new_segments,
            t: self.t.concat(lower.t),
        }
    }

    // TODO: Fail on overflow.
    fn shift_left(&mut self, shift: u8) {
        for segment in &mut self.segments {
            segment.shift_left(shift);
        }
    }

    fn widen(&mut self, new_type: Type) {
        self.t = new_type;
        for segment in &mut self.segments {
            segment.widen(new_type);
        }
    }

    fn name(&self) -> Ident {
        format_ident!("{}", self.name)
    }

    fn t(&self) -> Ident {
        format_ident!("{}", format!("{}", self.t))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Type(u8);

impl Type {
    const BOOL: Type = Type(1);

    fn for_template(len: u8) -> Type {
        match len {
            8 => Type(8),
            16 => Type(16),
            32 => Type(32),
            64 => Type(64),
            128 => Type(128),
            len => panic!("Template length must be 8, 16, 32, 64, or 128, but was {len}."),
        }
    }

    fn for_field(len: u8, precision: Precision) -> Type {
        match len {
            0 => panic!(),
            1..=128 if precision == Precision::Ux => Type(len.try_into().unwrap()),
            1        => Type(1),
            2..=8    => Type(8),
            9..=16   => Type(16),
            17..=32  => Type(32),
            33..=64  => Type(64),
            65..=128 => Type(128),
            129..=u8::MAX => panic!("Integers larger than u128 are not supported."),
        }
    }

    fn concat(self, other: Type) -> Type {
        Type::for_field(self.0 + other.0, Precision::Standard)
    }

    fn size(self) -> u8 {
        self.0
    }
}



impl From<u8> for Type {
    fn from(value: u8) -> Type {
        Type(value)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 1 {
            write!(f, "bool")
        } else {
            write!(f, "u{}", self.0)
        }
    }
}

#[derive(Clone, Copy)]
enum Base {
    Binary,
    Hexadecimal,
}

impl Base {
    fn bits_per_digit(self) -> usize {
        match self {
            Base::Binary => 1,
            Base::Hexadecimal => 4,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Precision {
    Standard,
    Ux,
}

#[derive(Clone)]
struct Segment {
    input: Expr,
    t: Type,
    mask: u128,
    shift: i16,
}

impl Segment {
    fn new(input: Expr, t: Type, mask: u128) -> Self {
        Self { input, t, mask, shift: 0 }
    }

    fn shift_left(&mut self, shift: u8) {
        self.shift -= i16::from(shift);
    }

    fn shift_right(&mut self, shift: u8) {
        self.shift += i16::from(shift);
    }

    fn widen(&mut self, new_type: Type) {
        if new_type > self.t {
            self.t = new_type;
        }
    }

    fn to_value(&self) -> TokenStream {
        let input = &self.input;
        let ordering = self.shift.cmp(&0);
        let shift = self.shift.abs();
        let shifter = match ordering {
            // There's no need to shift if the shift is 0.
            Ordering::Equal => quote! { },
            Ordering::Greater => quote! { >> #shift },
            Ordering::Less => quote! { << #shift },
        };

        let t = format_ident!("{}", self.t.to_string());
        let mask = self.mask;
        quote! { (#input as #t & #mask as #t) #shifter }
    }
}
