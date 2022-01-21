use std::fmt;
use std::ops::Index;
use std::rc::Rc;

#[derive(Debug)]
pub struct MappedArray<const SIZE: usize> {
    array: Box<[u8; SIZE]>,
    mappings: MemoryMappings,
}

impl <const SIZE: usize> MappedArray<SIZE> {
    pub fn new() -> MappedArray<SIZE> {
        MappedArray {
            array: Box::new([0; SIZE]),
            mappings: MemoryMappings::new(),
        }
    }

    pub fn set_mappings(&mut self, mappings: MemoryMappings) {
        self.mappings = mappings;
    }

    #[inline]
    pub fn read_byte(&self, index: usize) -> u8 {
        for mapping in &self.mappings.mappings {
            if let Some(value) = mapping.get_byte(index) {
                return *value;
            }
        }

        self.array[index]
    }

    #[inline]
    pub fn write_byte(&mut self, index: usize, value: u8) {
        for mapping in &self.mappings.mappings {
            if mapping.is_in_range(index) {
                println!(
                    "Attempted to write 0x{:02X} to read-only memory at 0x{:04X}. {}",
                    value,
                    index,
                    mapping
                );
                return;
            }
        }

        self.array[index] = value;
    }

    pub fn slice<const NEW_SIZE: usize>(&self, start_index: usize) -> MappedSlice<'_, NEW_SIZE> {
        MappedSlice {
            slice: (&self.array[start_index..start_index + NEW_SIZE]).try_into().unwrap(),
            mappings: self.mappings.shift_left(start_index),
        }
    }

    pub fn unmapped_slice(
        &self,
        start_index: usize,
        end_index: usize,
    ) -> Result<&[u8], String> {

        if self.mappings.is_in_range(start_index) ||
            self.mappings.is_in_range(end_index - 1) {

            return Err(format!(
                "Cannot create unmapped slice ({}, {}) due to overlap.",
                start_index,
                end_index,
            ));
        }

        Ok(&self.array[start_index..end_index])
    }
}

#[derive(Clone, Debug)]
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
        self.add_mapping_internal(mapping)
    }

    #[inline]
    pub fn is_in_range(&self, index: usize) -> bool {
        self.mappings
            .iter()
            .any(|mapping| mapping.is_in_range(index))
    }

    #[inline]
    pub fn get_byte(&self, index: usize) -> Option<u8> {
        for mapping in &self.mappings {
            let byte = mapping.get_byte(index);
            if byte.is_some() {
                return byte.map(|b| *b);
            }
        }

        None
    }

    pub fn end_index(&self) -> usize {
        if let Some(last_mapping) = self.mappings.last() {
            last_mapping.end_index()
        } else {
            0
        }
    }

    pub fn shift_left(&self, count: usize) -> MemoryMappings {
        let mut result = MemoryMappings::new();
        for mapping in &self.mappings {
            if let Some(mapping) = mapping.shift_left(count) {
                result.add_mapping_internal(mapping).unwrap();
            }
        }

        result
    }

    fn add_mapping_internal(&mut self, mapping: Mapping) -> Result<(), String> {
        for i in 0..self.mappings.len() {
            let existing_mapping = &self.mappings[i];
            if mapping.end_index() < existing_mapping.start_index {
                self.mappings.insert(i, mapping);
                return Ok(());
            } else if mapping.start_index <= existing_mapping.end_index() {
                return Err(format!(
                    "Can't add mapping {} since it overlaps existing mapping {}.",
                    mapping,
                    existing_mapping,
                ));
            }
        }

        // Add it to the end if it is higher indexed than all previous mappings.
        self.mappings.push(mapping);

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Mapping {
    slice: Rc<[u8]>,
    start_index: usize,
}

impl Mapping {
    #[inline]
    fn is_in_range(&self, index: usize) -> bool {
        self.start_index <= index && index <= self.end_index()
    }

    fn overlaps(&self, other: &Mapping) -> bool {
        self.is_in_range(other.start_index) ||
            self.is_in_range(other.end_index())
    }

    #[inline]
    fn get_byte(&self, index: usize) -> Option<&u8> {
        if self.is_in_range(index) {
            Some(&self.slice[index - self.start_index])
        } else {
            None
        }
    }

    fn end_index(&self) -> usize {
        self.start_index + self.slice.len() - 1
    }

    fn shift_left(&self, count: usize) -> Option<Mapping> {
        if self.end_index() < count {
            return None;
        }

        let start_index;
        let slice;
        if self.start_index < count {
            // Truncate the front of the slice.
            start_index = 0;
            slice = self.slice[count - self.start_index - 1..].into();
        } else {
            // Shift the slice without truncation.
            start_index = self.start_index - count;
            slice = self.slice.clone();
        }

        Some(Mapping {slice, start_index})
    }
}

impl fmt::Display for Mapping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:04X},{:04X})", self.start_index, self.end_index())
    }
}

#[derive(Clone, Debug)]
pub struct MappedSlice<'a, const SIZE: usize> {
    slice: &'a [u8; SIZE],
    mappings: MemoryMappings,
}

impl <'a, const SIZE: usize> MappedSlice<'a, SIZE> {
    pub fn slice<const NEW_SIZE: usize>(&self, start_index: usize) -> MappedSlice<'a, NEW_SIZE> {
        MappedSlice {
            slice: (&self.slice[start_index..start_index + NEW_SIZE]).try_into().unwrap(),
            mappings: self.mappings.shift_left(start_index),
        }
    }
}

impl <const SIZE: usize> Index<usize> for MappedSlice<'_, SIZE> {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        for mapping in &self.mappings.mappings {
            if let Some(value) = mapping.get_byte(idx) {
                return value;
            }
        }

        &self.slice[idx]
    }
}
