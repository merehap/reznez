// Portable PixMap binary file format.
pub struct Ppm {
    data: [u8; Ppm::DATA_SIZE],
}

impl Ppm {
    const METADATA: &'static [u8] = b"P6\n256 240\n255\n";
    const DATA_SIZE: usize = 3 * 256 * 240;

    pub fn new(data: [u8; Ppm::DATA_SIZE]) -> Ppm {
        Ppm {data}
    }

    pub fn from_bytes(raw: &[u8]) -> Result<Ppm, String> {
        if !raw.starts_with(Ppm::METADATA) {
            return Err(format!(
                "Bad PPM metadata: {:?}",
                &raw[0..Ppm::METADATA.len()],
            ));
        }

        let data = &raw[Ppm::METADATA.len()..];
        if let Ok(data) = data.try_into() {
            Ok(Ppm {data})
        } else {
            Err(format!(
                "Expected PPM data length to be {} but was {}.",
                Ppm::DATA_SIZE,
                data.len(),
            ))
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Ppm::METADATA.len() + Ppm::DATA_SIZE);
        bytes.extend_from_slice(&Ppm::METADATA);
        bytes.extend_from_slice(&self.data);
        bytes
    }
}
