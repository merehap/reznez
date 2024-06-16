extern crate proc_macro;

use std::cmp::Ordering;
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

fn splitbits_base(input: proc_macro::TokenStream, base: Base, precision: Precision) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into(), base, precision);
    let fields = template.extract_fields(&value);

    let struct_name = template.to_struct_name();
    let names: Vec<_> = fields.iter().map(|field| field.name()).collect();
    let types: Vec<_> = fields.iter().map(|field| field.t()).collect();
    let values: Vec<TokenStream> = fields.iter().map(|field| field.value.clone()).collect();
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
    let values: Vec<TokenStream> = fields.iter().map(|field| field.value.clone()).collect();

    let result = quote! {
        (#(#values,)*)
    };

    result.into()
}

fn splitbits_tuple_into_base(input: proc_macro::TokenStream, base: Base, precision: Precision) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into(), base, precision);
    let fields = template.extract_fields(&value);
    let values: Vec<TokenStream> = fields.iter().map(|field| field.value.clone()).collect();

    let result = quote! {
        (#((#values).into(),)*)
    };

    result.into()
}

fn onefield_base(input: proc_macro::TokenStream, base: Base, precision: Precision) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into(), base, precision);
    let fields = template.extract_fields(&value);
    assert_eq!(fields.len(), 1);
    fields[0].value.clone().into()
}

fn parse_input(item: TokenStream, base: Base, precision: Precision) -> (Expr, Template) {
    let parts: Punctuated::<Expr, Token![,]> = Parser::parse2(
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

        let input_type = match template.len() {
            8 => Type(8),
            16 => Type(16),
            32 => Type(32),
            64 => Type(64),
            128 => Type(128),
            len => panic!("Template length must be 8, 16, 32, 64, or 128, but was {len}."),
        };

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
                    (p.len() as u32, p[0].0 as u32)
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

    fn to_struct_name(&self) -> Ident {
        let struct_name_suffix: String = self.names().iter()
            // Underscores work in struct names, periods do not.
            .map(|&c| if c == '.' { '_' } else { c })
            .collect();
        format_ident!("{}", format!("Fields·{}", struct_name_suffix))
    }
}

type Name = char;
type Location = (u32, u32);

#[derive(Clone)]
struct Field {
    name: char,
    value: TokenStream,
    t: Type,
}

impl Field {
    fn new(name: char, input_type: Type, input: &Expr, precision: Precision, locations: &[Location]) -> Field {
        let bit_count = locations.iter().map(|(length, _)| length).sum();
        let t = match bit_count {
            0 => panic!(),
            1..=128 if precision == Precision::Ux => Type(bit_count.try_into().unwrap()),
            1        => Type(1),
            2..=8    => Type(8),
            9..=16   => Type(16),
            17..=32  => Type(32),
            33..=64  => Type(64),
            65..=128 => Type(128),
            129..=u32::MAX => panic!("Integers larger than u128 are not supported."),
        }.into();

        let input_type = format_ident!("{}", input_type.to_string());

        let value = if t == Type::BOOL {
            let (length, mask_offset) = locations[0];
            let mut mask: u128 = 2u128.pow(length as u32) - 1;
            mask <<= mask_offset;
            quote! { #input as #input_type & #mask as #input_type != 0 }
        } else {
            let mut segment_offset = 0;
            let mut segments = Vec::new();
            for &(length, mask_offset) in locations {
                let mut mask: u128 = 2u128.pow(length as u32) - 1;
                mask <<= mask_offset;

                let shifter = match mask_offset.cmp(&segment_offset) {
                    // There's no need to shift if the shift is 0.
                    Ordering::Equal => quote! { },
                    Ordering::Greater => {
                        let shift = mask_offset - segment_offset;
                        quote! { >> #shift }
                    }
                    Ordering::Less => {
                        let shift = segment_offset - mask_offset;
                        quote! { << #shift }
                    }
                };

                let segment = quote! { ((#input as #input_type & #mask as #input_type) #shifter) };
                segments.push(segment);
                segment_offset += length;
            }

            let t = format_ident!("{}", format!("{}", t));
            quote! { #t::try_from(#(#segments)|*).unwrap() }
        };

        Field { name, value, t }
    }

    fn name(&self) -> Ident {
        format_ident!("{}", self.name)
    }

    fn t(&self) -> Ident {
        format_ident!("{}", format!("{}", self.t))
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Type(u8);

impl Type {
    const BOOL: Type = Type(1);
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
