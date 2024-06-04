extern crate proc_macro;

use std::collections::BTreeMap;
use std::fmt;

use proc_macro::TokenStream;

// TODO:
// * Allow more int types as input.
// * Allow more int types as output.
// * Allow parsing a single field.
// * Enable setting pre-defined variables as an alternative to making a new struct.
// * Enable emitting precise-sized ux crate types.
// * Support no_std.
#[proc_macro]
pub fn splitbits(item: TokenStream) -> TokenStream {
    let input = item.to_string();
    let parts: Vec<String> = input.split(',').map(str::to_string).collect();
    assert_eq!(parts.len(), 2);

    let value = parts[0].trim();
    let template = parts[1].trim();

    let mut template: Vec<char> = template.chars()
        // Underscores are only for human-readability.
        .filter(|&c| c != '_')
        .collect();
    assert_eq!(template.len(), 10);
    assert_eq!(template[0], '"');
    assert_eq!(template[9], '"');
    template.remove(9);
    template.remove(0);

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

    let template: String = template.iter()
        // Underscores work in struct names, periods do not.
        .map(|&c| if c == '.' { '_' } else { c })
        .collect();

    let struct_name = format!("Fields_{}", template);

    let mut result = "{".to_string();
    result.push_str(&format!("struct {struct_name} {{\n"));
    for (name, field) in &fields {
        result.push_str(&format!("    {name}: {},\n", field.t));
    }

    result.push('}');

    result.push_str(&format!("    {struct_name} {{\n"));
    for field in fields.values() {
        let name = field.name;
        let mask = field.mask;
        match field.t {
            Type::Bool => {
                result.push_str(&format!("        {name}: {value} & {mask} != 0,\n"));
            }
            Type::U8 => {
                let shift = mask.trailing_zeros();
                result.push_str(&format!("        {name}: ({value} & {mask}) >> {shift},\n"));
            }
        }
    }

    result.push_str("    }\n");

    result.push('}');

    result.parse().unwrap()
}

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
