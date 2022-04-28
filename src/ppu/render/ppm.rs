use std::hash::Hash;
// Portable PixMap binary (P6 not P3) file format.
#[derive(PartialEq, Eq, Hash)]
pub struct Ppm {
    data: Vec<u8>,
}

impl Ppm {
    const METADATA: &'static [u8] = b"P6\n256 240\n255\n";
    const DATA_SIZE: usize = 3 * 256 * 240;

    pub fn new(data: Vec<u8>) -> Ppm {
        Ppm { data }
    }

    pub fn from_bytes(raw: &[u8]) -> Result<Ppm, String> {
        if !raw.starts_with(Ppm::METADATA) {
            return Err(format!(
                "Bad PPM metadata: {:?}",
                &raw[0..Ppm::METADATA.len()],
            ));
        }

        let data = raw[Ppm::METADATA.len()..].to_vec();
        if data.len() != Ppm::DATA_SIZE {
            return Err(format!(
                "Expected PPM data length to be {} but was {}.",
                Ppm::DATA_SIZE,
                data.len(),
            ));
        }

        Ok(Ppm { data })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Ppm::METADATA.len() + Ppm::DATA_SIZE);
        bytes.extend_from_slice(Ppm::METADATA);
        bytes.extend_from_slice(&self.data);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let mut data = Vec::with_capacity(Ppm::DATA_SIZE);
        for i in 0..Ppm::DATA_SIZE {
            data.push((i % 256) as u8);
        }

        let ppm = Ppm::new(data.clone());
        let bytes = &ppm.to_bytes();
        assert_eq!(&bytes[Ppm::METADATA.len()..], &data);
        let ppm = Ppm::from_bytes(bytes).unwrap();
        let bytes = &ppm.to_bytes();
        assert_eq!(&bytes[Ppm::METADATA.len()..], &data);
    }
}
