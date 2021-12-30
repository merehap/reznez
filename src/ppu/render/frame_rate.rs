use std::str::FromStr;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct FrameRate(f64);

impl FrameRate {
    const NTSC:  FrameRate = FrameRate(60.0988);
    const PAL:   FrameRate = FrameRate(50.0070);
    const DENDY: FrameRate = FrameRate(50.0070);
    const RGB:   FrameRate = FrameRate(60.0985);

    pub fn new(value: f64) -> Result<FrameRate, String> {
        if value.is_normal() {
            Ok(FrameRate(value))
        } else {
            Err("Frame rate must be a normal, non-zero number.".to_string())
        }
    }

    pub fn to_frame_duration(self) -> Duration {
        Duration::from_nanos((1_000_000_000.0 / self.0) as u64)
    }
}

impl FromStr for FrameRate {
    type Err = String;

    fn from_str(value: &str) -> Result<FrameRate, String> {
        Ok(match value {
            "ntsc"  => FrameRate::NTSC,
            "pal"   => FrameRate::PAL,
            "dendy" => FrameRate::DENDY,
            "rgb"   => FrameRate::RGB,
            _ => {
                let value = f64::from_str(value)
                    .map_err(|err| err.to_string())?;
                FrameRate::new(value)?
            },
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TargetFrameRate {
    Value(FrameRate),
    Unbounded,
}

impl FromStr for TargetFrameRate {
    type Err = String;

    fn from_str(value: &str) -> Result<TargetFrameRate, String> {
        match value {
            "unbounded" => Ok(TargetFrameRate::Unbounded),
            _ => Ok(TargetFrameRate::Value(FrameRate::from_str(value)?)),
        }
    }
}
