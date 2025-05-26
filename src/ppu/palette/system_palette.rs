use enum_iterator::all;
use num_traits::FromPrimitive;

use crate::ppu::palette::color::{Brightness, Color, Hue};
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::register::registers::mask::Mask;

// Good enough emphasis for now.
// Taken from https://forums.nesdev.org/viewtopic.php?p=4634
// RGB
const ALL_EMPHASIS_FACTORS: [[f32; 3]; 8] =
[
	[1.00, 1.00, 1.00],
	[1.00, 0.80, 0.81],
	[0.78, 0.94, 0.66],
	[0.79, 0.77, 0.63],
	[0.82, 0.83, 1.12],
	[0.81, 0.71, 0.87],
	[0.68, 0.79, 0.79],
	[0.70, 0.70, 0.70],
];

/*
 * [
 *     No emphasis,
 *     Red emphasis,
 *     Green emphasis,
 *     Blue emphasis,
 *     Red and green emphasis,
 *     Red and blue emphasis,
 *     Blue and green emphasis,
 *     All emphasis,
 * ]
 */
#[derive(Clone)]
pub struct SystemPalette([[Rgb; 64]; 8]);

impl SystemPalette {
    pub fn parse(raw: &str) -> Result<SystemPalette, String> {
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

        let mut unemphasized_palette = [Rgb::BLACK; 64];
        for (i, line) in lines.iter().enumerate() {
            let brightness = FromPrimitive::from_usize(i).unwrap();
            SystemPalette::parse_line(&mut unemphasized_palette, brightness, line)?;
        }

        let mut emphasized_palettes = Vec::new();
        for emphasis_factors in ALL_EMPHASIS_FACTORS {
            let emphasized_palette = unemphasized_palette.map(|rgb| {
                rgb.emphasized(emphasis_factors)
            });
            emphasized_palettes.push(emphasized_palette);
        }

        Ok(SystemPalette(emphasized_palettes.try_into().unwrap()))
    }

    pub fn lookup_rgb(&self, color: Color, mask: Mask) -> Rgb {
        self.0[mask.emphasis_index()][color.to_usize()]
    }

    fn parse_line(
        palette: &mut [Rgb; 64],
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
                    "A .pal file must consist of all numbers, but found '{word}'.",
                ));
            }
        }

        if words.len() != 48 {
            return Err(format!(
                "There must be exactly 48 values (16 RGB triples) on each line, but found {}.",
                words.len(),
            ));
        }

        for hue in all::<Hue>() {
            let i = hue as usize;
            let color = Color::new(hue, brightness);
            let rgb = Rgb::new(nums[3 * i], nums[3 * i + 1], nums[3 * i + 2]);
            palette[color.to_usize()] = rgb;
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
