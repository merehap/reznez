extern crate proc_macro;

use std::fmt;

use proc_macro2::TokenStream;

use quote::{quote, format_ident};
use syn::{Token, Expr, Lit};
use syn::parse::Parser;
use syn::punctuated::Punctuated;

// TODO:
// * Allow parsing a single field.
// * Enable setting pre-defined variables as an alternative to making a new struct.
// * Enable emitting precise-sized ux crate types.
// * Support no_std.
// * Implement combinebits.
// * combinebits_hexadecimal
#[proc_macro]
pub fn splitbits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into());

    let fields = fields(template.clone());
    let struct_name_suffix: String = template.iter()
        // Underscores work in struct names, periods do not.
        .map(|&c| if c == '.' { '_' } else { c })
        .collect();

    let struct_name = format!("Fields·{}", struct_name_suffix);

    let struct_ident = format_ident!("{}", struct_name);
    let names = fields.iter().map(|field| format_ident!("{}", field.name));
    let names2 = names.clone();
    let types: Vec<_> = fields.iter()
        .map(|field| format_ident!("{}", format!("{}", field.t)))
        .collect();

    let values: Vec<TokenStream> = fields.iter()
        .map(|&field| quote_field_value(field, &value))
        .collect();

    let result = quote! {
        {
            struct #struct_ident {
                #(#names: #types,)*
            }

            #struct_ident {
                #(#names2: #values,)*
            }
        }
    };

    result.into()
}

#[proc_macro]
pub fn onefield(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into());
    let fields = fields(template.clone());
    assert_eq!(fields.len(), 1);
    quote_field_value(fields[0], &value).into()
}

fn parse_input(item: TokenStream) -> (Expr, Vec<char>) {
    let parts: Punctuated::<Expr, Token![,]> = Parser::parse2(
        Punctuated::<Expr, Token![,]>::parse_terminated,
        item.clone().into(),
    ).unwrap();
    let parts: Vec<Expr> = parts.into_iter().collect();
    assert_eq!(parts.len(), 2);

    let value = parts[0].clone();
    let Expr::Lit(template) = parts[1].clone() else { panic!() };
    let Lit::Str(template) = template.lit else { panic!() };
    let template: String = template.value();

    let template: Vec<char> = template.chars()
        // Spaces are only for human-readability.
        .filter(|&c| c != ' ')
        .collect();

    (value, template)
}

fn fields(template: Vec<char>) -> Vec<Field> {
    let input_type = match template.len() {
        8 => Type::U8,
        16 => Type::U16,
        32 => Type::U32,
        64 => Type::U64,
        128 => Type::U128,
        len => panic!("Template length must be 8, 16, 32, 64, or 128, but was {len}."),
    };

    let mut fields = template.clone();
    fields.dedup();
    fields.iter()
        // Periods aren't names, they are placeholders for ignored bits.
        .filter(|&&c| c != '.')
        .map(|&name| {
            assert!(name.is_ascii_lowercase());

            let mut mask: u128 = 0;
            for (index, &n) in template.iter().enumerate() {
                if n == name {
                    mask |= 1 << (input_type as usize - index - 1);
                }
            }

            let t = match u128::BITS - mask.leading_zeros() - mask.trailing_zeros() {
                0 => panic!(),
                1        => Type::Bool,
                2..=8    => Type::U8,
                9..=16   => Type::U16,
                17..=32  => Type::U32,
                33..=64  => Type::U64,
                65..=128 => Type::U128,
                129..=u32::MAX => panic!("Integers larger than u128 are not supported."),
            };

            Field {
                name,
                mask,
                t,
            }
        })
        .collect()
}

fn quote_field_value(field: Field, value: &Expr) -> TokenStream {
    let mask = field.mask;
    let shift = mask.trailing_zeros() as u128;
    match field.t {
        Type::Bool => quote! { #value as u128 & #mask != 0 },
        Type::U8   => quote! { ((#value as u128 & #mask) >> #shift) as u8 },
        Type::U16  => quote! { ((#value as u128 & #mask) >> #shift) as u16 },
        Type::U32  => quote! { ((#value as u128 & #mask) >> #shift) as u32 },
        Type::U64  => quote! { ((#value as u128 & #mask) >> #shift) as u64 },
        Type::U128 => quote! { ((#value as u128 & #mask) >> #shift) as u128 },
    }
}

#[derive(Clone, Copy)]
struct Field {
    name: char,
    mask: u128,
    t: Type,
}

#[derive(Clone, Copy, Debug)]
enum Type {
    Bool =   1,
    U8   =   8,
    U16  =  16,
    U32  =  32,
    U64  =  64,
    U128 = 128,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = format!("{:?}", *self).to_lowercase();
        write!(f, "{}", result)
    }
}
