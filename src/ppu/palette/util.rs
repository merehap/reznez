use std::num::NonZeroU8;

use super::rgb::Rgb;

pub fn spectrum(step_count: u16) -> Vec<Rgb> {
    RawColor::spectrum(step_count).iter()
        .map(|color| color.to_rgb())
        .collect()
}

pub fn greyscale(step_count: u16) -> Vec<Rgb> {
    let step_count: u32 = step_count.into();

    let mut scale = Vec::new();
    for step in 0..step_count {
        let value: u8 = (256 * step / step_count).try_into().unwrap();
        scale.push(Rgb::new(value, value, value));
    }

    scale
}

#[derive(Clone, Copy)]
struct RawColor {
    range: Range,
    index: NonZeroU8,
}

impl RawColor {
    const FULL_RANGE: u16 = 6 * 254;

    fn spectrum(step_count: u16) -> Vec<RawColor> {
        let step_count: u32 = step_count.into();

        let mut spectrum = Vec::new();
        for step in 0..step_count {
            let full_index = u32::from(RawColor::FULL_RANGE) * step / step_count;
            let full_index: u16 = full_index.try_into().unwrap();
            spectrum.push(RawColor::from_full_index(full_index));
        }

        spectrum
    }

    fn from_full_index(full_index: u16) -> RawColor {
        assert!(full_index < RawColor::FULL_RANGE);
        let range = match full_index / 254 {
            0 => Range::RedYellow,
            1 => Range::YellowGreen,
            2 => Range::GreenCyan,
            3 => Range::CyanBlue,
            4 => Range::BlueMagenta,
            5 => Range::MagentaRed,
            _ => unreachable!(),
        };

        RawColor {
            range,
            index: NonZeroU8::new((full_index % 254 + 1).try_into().unwrap()).unwrap(),
        }
    }

    fn to_rgb(self) -> Rgb {
        let index = self.index.get();
        match self.range {
            Range::RedYellow => Rgb::new(0xFF, index, 0x00),
            Range::YellowGreen => Rgb::new(index, 0xFF, 0x00),
            Range::GreenCyan => Rgb::new(0x00, 0xFF, index),
            Range::CyanBlue => Rgb::new(0x00, index, 0xFF),
            Range::BlueMagenta => Rgb::new(index, 0x00, 0xFF),
            Range::MagentaRed => Rgb::new(0xFF, 0x00, index),
        }
    }
}

#[derive(Clone, Copy)]
enum Range {
    RedYellow,
    YellowGreen,
    GreenCyan,
    CyanBlue,
    BlueMagenta,
    MagentaRed,
}