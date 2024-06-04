extern crate proc_macro;

use std::collections::BTreeMap;

use proc_macro::TokenStream;

// TODO:
// * Allow more int types as input.
// * Allow more int types as output.
// * Allow parsing a single field.
// * Enable setting pre-defined variables as an alternative to making a new struct.
// * Enable emitting precise-sized ux crate types.
#[proc_macro]
pub fn splitbits_proc(item: TokenStream) -> TokenStream {
    let template = item.to_string();

    let mut template: Vec<char> = template.chars()
        // Underscores are only for human-readability.
        .filter(|&c| c != '_')
        // Underscores work in struct names, periods do not.
        .map(|c| if c == '.' { '_' } else { c })
        .collect();
    assert_eq!(template.len(), 10);
    assert_eq!(template[0], '"');
    assert_eq!(template[9], '"');
    template.remove(9);
    template.remove(0);

    let mut fields = template.clone();
    fields.dedup();
    let masks: BTreeMap<char, u8> = fields.iter()
        // Periods aren't names, they are placeholders for ignored bits.
        .filter(|&&c| c != '_')
        .map(|&name| {
            assert!(name.is_ascii_lowercase());

            let mut mask: u8 = 0;
            for (index, &n) in template.iter().enumerate() {
                if n == name {
                    mask |= 1 << (7 - index);
                }
            }

            (name, mask)
        })
        .collect();

    let template: String = template.iter().collect();

    let struct_name = format!("Fields_{}", template);

    let mut result = "{".to_string();
    result.push_str(&format!("struct {struct_name} {{\n"));
    for (name, mask) in &masks {
        if mask.count_ones() == 1 {
            result.push_str(&format!("    {name}: bool,\n"));
        } else {
            result.push_str(&format!("    {name}: u8,\n"));
        }
    }

    result.push('}');

    let function_name = format!("make_fields_{template}\n");
    result.push_str(&format!("fn {function_name}(value: u8) -> {struct_name} {{\n"));
    result.push_str(&format!("    {struct_name} {{\n"));
    for (name, mask) in masks {
        if mask.count_ones() == 1 {
            result.push_str(&format!("        {name}: value & {mask} != 0,\n"));
        } else {
            let shift = mask.trailing_zeros();
            result.push_str(&format!("        {name}: (value & {mask}) >> {shift},\n"));
        }
    }

    result.push_str("    }\n");

    result.push_str("}\n");

    result.push_str(&function_name);

    result.push('}');

    result.parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
