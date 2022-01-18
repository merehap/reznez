use std::fmt;
use std::rc::Rc;

pub struct MemoryMappings {
    mappings: Vec<Mapping>,
}

impl MemoryMappings {
    pub fn new() -> MemoryMappings {
        MemoryMappings {mappings: Vec::new()}
    }

    pub fn add_mapping(
        &mut self,
        slice: Rc<[u8]>,
        start_index: usize,
    ) -> Result<(), String> {

        let mapping = Mapping {slice, start_index};
        for existing_mapping in &self.mappings {
            if mapping.overlaps(existing_mapping) {
                return Err(format!(
                    "Can't add mapping {} since it overlaps existing mapping {}.",
                    mapping,
                    existing_mapping,
                ));
            }
        }

        self.mappings.push(mapping);

        Ok(())
    }

    #[inline]
    pub fn get_byte(&self, index: usize) -> Option<u8> {
        for mapping in &self.mappings {
            let byte = mapping.get_byte(index);
            if byte.is_some() {
                return byte;
            }
        }

        None
    }
}

struct Mapping {
    slice: Rc<[u8]>,
    start_index: usize,
}

impl Mapping {
    #[inline]
    fn is_in_range(&self, index: usize) -> bool {
        self.start_index <= index && index < self.end_index()
    }

    fn overlaps(&self, other: &Mapping) -> bool {
        self.is_in_range(other.start_index) ||
            self.is_in_range(other.end_index())
    }

    #[inline]
    fn get_byte(&self, index: usize) -> Option<u8> {
        if self.is_in_range(index) {
            Some(self.slice[index - self.start_index])
        } else {
            None
        }
    }

    fn end_index(&self) -> usize {
        self.start_index + self.slice.len()
    }
}

impl fmt::Display for Mapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.start_index, self.end_index())
    }
}
