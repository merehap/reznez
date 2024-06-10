extern crate proc_macro;

use std::collections::BTreeMap;
use std::fmt;

use proc_macro2::TokenStream;

use quote::{quote, format_ident};
use syn::{Token, Expr, Lit};
use syn::parse::Parser;
use syn::punctuated::Punctuated;

// TODO:
// * Allow more int types as input.
// * Allow more int types as output.
// * Allow parsing a single field.
// * Enable setting pre-defined variables as an alternative to making a new struct.
// * Enable emitting precise-sized ux crate types.
// * Support no_std.
#[proc_macro]
pub fn splitbits(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
    assert_eq!(template.len(), 8);

    let mut fields = template.clone();
    fields.dedup();
    let fields: BTreeMap<char, Field> = fields.iter()
        // Periods aren't names, they are placeholders for ignored bits.
        .filter(|&&c| c != '.')
        .map(|&name| {
            assert!(name.is_ascii_lowercase());

            let mut mask: u8 = 0;
            for (index, &n) in template.iter().enumerate() {
                if n == name {
                    mask |= 1 << (7 - index);
                }
            }

            let t = if mask.count_ones() == 1 {
                Type::Bool
            } else {
                Type::U8
            };

            let field = Field {
                name,
                mask,
                t,
            };

            (name, field)
        })
        .collect();
    let fields: Vec<_> = fields.values().collect();

    let template: String = template.iter()
        // Underscores work in struct names, periods do not.
        .map(|&c| if c == '.' { '_' } else { c })
        .collect();

    let struct_name = format!("Fields·{}", template);

    let struct_ident = format_ident!("{}", struct_name);
    let names = fields.iter().map(|field| format_ident!("{}", field.name));
    let names2 = names.clone();
    let types: Vec<_> = fields.iter()
        .map(|field| format_ident!("{}", format!("{}", field.t)))
        .collect();

    let values: Vec<TokenStream> = fields.iter()
        .map(|field| {
            let mask = field.mask;
            let shift = mask.trailing_zeros();
            match field.t {
                Type::Bool => quote! {
                    {
                        #value & #mask != 0
                    }
                },
                Type::U8 => quote! {
                    {
                        (#value & #mask) >> #shift
                    }
                },
            }
        })
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

#[derive(Clone, Copy)]
struct Field {
    name: char,
    mask: u8,
    t: Type,
}

#[derive(Clone, Copy, Debug)]
enum Type {
    Bool,
    U8,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = format!("{:?}", *self).to_lowercase();
        write!(f, "{}", result)
    }
}
