use std::collections::BTreeMap;

use enum_iterator::IntoEnumIterator;
use num_traits::FromPrimitive;

use crate::ppu::palette::color::{Brightness, Color, Hue};
use crate::ppu::palette::rgb::Rgb;

#[derive(Clone)]
pub struct SystemPalette(BTreeMap<Color, Rgb>);

impl SystemPalette {
    pub fn parse(raw: &str) -> Result<SystemPalette, String> {
        let mut result = BTreeMap::new();

        let lines: Vec<&str> = raw
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        if lines.len() != 4 {
            return Err(format!(
                "A system palette must have exactly 4 brightness lines, but found {}.",
                lines.len(),
            ));
        }

        for (i, line) in lines.iter().enumerate() {
            let brightness = FromPrimitive::from_usize(i).unwrap();
            SystemPalette::parse_line(&mut result, brightness, line)?;
        }

        Ok(SystemPalette(result))
    }

    pub fn lookup_rgb(&self, color: Color) -> Rgb {
        self.0[&color]
    }

    fn parse_line(
        palette: &mut BTreeMap<Color, Rgb>,
        brightness: Brightness,
        line: &str,
    ) -> Result<(), String> {
        let words: Vec<&str> = line.split(' ').filter(|s| !s.is_empty()).collect();

        let mut nums = Vec::new();
        for word in &words {
            if let Ok(number) = word.parse() {
                nums.push(number);
            } else {
                return Err(format!(
                    "A .pal file must consist of all numbers, but found '{}'.",
                    word,
                ));
            }
        }

        if words.len() != 48 {
            return Err(format!(
                "There must be exactly 48 values (16 RGB triples) on each line, but found {}.",
                words.len(),
            ));
        }

        for hue in Hue::into_enum_iter() {
            let i = hue as usize;
            let color = Color::new(hue, brightness);
            let rgb = Rgb::new(nums[3 * i], nums[3 * i + 1], nums[3 * i + 2]);
            palette.insert(color, rgb);
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod test_data {
    use super::*;

    pub fn system_palette() -> SystemPalette {
        SystemPalette::parse(include_str!("../../../palettes/2C02.pal")).unwrap()
    }
}
