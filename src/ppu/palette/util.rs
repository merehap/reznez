const LEVELS: [u16; 9] = [3, 6, 12, 24, 48, 96, 192, 384, 768];

pub fn spectrum(count: u16) -> Vec<u8> {
    let mut granularity = None;
    for i in 0..LEVELS.len() {
        if count < LEVELS[i] {
            granularity = Some(i);
            break;
        }
    }

    let granularity = granularity.unwrap_or(8);

    //let alt_granularity = 
    unreachable!()
}

fn spectrum_granularity(count: u16) -> u8 {
    let mut granularity = None;
    for i in 0..LEVELS.len() {
        if count <= LEVELS[i] {
            granularity = Some(i as u8);
            break;
        }
    }

    let old_value = granularity.unwrap_or(8);

    //let new_value = (1.5 * f64::from(count)).log2().trunc() as u8 - 1;
    //assert_eq!(old_value, new_value);

    old_value
}

#[cfg(test)]
pub mod test_data {
    use super::*;

    #[test]
    fn spectrum_lengths() {
        assert_eq!(spectrum_granularity(0), 0);
        assert_eq!(spectrum_granularity(1), 0);
        assert_eq!(spectrum_granularity(2), 0);
        assert_eq!(spectrum_granularity(3), 0);
        assert_eq!(spectrum_granularity(4), 1);
        assert_eq!(spectrum_granularity(5), 1);
        assert_eq!(spectrum_granularity(6), 1);
        assert_eq!(spectrum_granularity(7), 2);

        assert_eq!(spectrum_granularity(12), 2);
        assert_eq!(spectrum_granularity(13), 3);

        assert_eq!(spectrum_granularity(384), 7);
        assert_eq!(spectrum_granularity(385), 8);

        assert_eq!(spectrum_granularity(768), 8);
        assert_eq!(spectrum_granularity(769), 8);

        assert_eq!(spectrum_granularity(1024), 8);
        assert_eq!(spectrum_granularity(1025), 8);
    }
}