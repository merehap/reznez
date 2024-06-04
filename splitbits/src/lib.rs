extern crate proc_macro;

use std::collections::BTreeMap;

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
    let masks: BTreeMap<char, u8> = fields.iter()
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

            (name, mask)
        })
        .collect();

    let template: String = template.iter()
        // Underscores work in struct names, periods do not.
        .map(|&c| if c == '.' { '_' } else { c })
        .collect();

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

    result.push_str(&format!("    {struct_name} {{\n"));
    for (name, mask) in masks {
        if mask.count_ones() == 1 {
            result.push_str(&format!("        {name}: {value} & {mask} != 0,\n"));
        } else {
            let shift = mask.trailing_zeros();
            result.push_str(&format!("        {name}: ({value} & {mask}) >> {shift},\n"));
        }
    }

    result.push_str("    }\n");

    result.push('}');

    result.parse().unwrap()
}
