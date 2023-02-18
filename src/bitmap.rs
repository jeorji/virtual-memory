use crate::data_location::DataLocation;
use crate::BYTE_SIZE;

#[derive(Debug)]
pub struct BitMap(usize, DataLocation<u8>);

impl BitMap {
    #[allow(dead_code)]
    pub fn new(capacity: usize) -> Self {
        //round up `capacity` to the nearest multiple of `BYTE_SIZE`
        let bytes_amount = (capacity + BYTE_SIZE - 1) / BYTE_SIZE;
        BitMap(capacity, DataLocation::<u8>::new(bytes_amount))
    }

    pub fn get(&self, index: usize) -> bool {
        assert!(
            index < self.0,
            "index out of bounds: the len is {} but the index is {}",
            self.0,
            index
        );

        let byte_index = index / BYTE_SIZE;
        let bit_offset = index % BYTE_SIZE;

        let state = self.1[byte_index] & (1 << bit_offset);
        state != 0
    }

    // set bit to 1
    pub fn set(&mut self, index: usize) {
        let byte_index = index / BYTE_SIZE;
        let bit_offset = index % BYTE_SIZE;

        self.1[byte_index] |= 1 << bit_offset;
    }

    // set bit to 0
    pub fn unset(&mut self, index: usize) {
        let byte_index = index / BYTE_SIZE;
        let bit_offset = index % BYTE_SIZE;

        self.1[byte_index] &= !(1 << bit_offset);
    }

    // inverse the bit
    #[allow(dead_code)]
    pub fn inverse(&mut self, index: usize) {
        let byte_index = index / BYTE_SIZE;
        let bit_offset = index % BYTE_SIZE;

        self.1[byte_index] ^= 1 << bit_offset;
    }
}

impl From<&[u8]> for BitMap {
    fn from(value: &[u8]) -> Self {
        let bytes_amount = (value.len() + BYTE_SIZE - 1) / BYTE_SIZE;
        BitMap(bytes_amount * BYTE_SIZE, DataLocation::from(value))
    }
}

impl AsRef<[u8]> for BitMap {
    fn as_ref(&self) -> &[u8] {
        self.1.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::BitMap;

    #[test]
    #[should_panic]
    fn bitmap_out_of_bounds() {
        let bm = BitMap::new(64);
        assert_eq!(bm.get(0), false);
        assert_eq!(bm.get(63), false);
        bm.get(64);
    }

    #[test]
    fn bitmap_from_u8() {
        let data = vec![1u8; 64];
        let bm = BitMap::from(data.as_ref());
        assert_eq!(bm.get(0), true);
        assert_eq!(bm.get(1), false);
        assert_eq!(bm.get(8), true);
        assert_eq!(bm.get(9), false);
    }

    #[test]
    fn get_bit() {
        let bm = BitMap::new(64);
        assert_eq!(bm.get(63), false);
    }

    #[test]
    fn set_bit() {
        let mut bm = BitMap::new(64);
        bm.set(8);
        assert_eq!(bm.get(8), true);
    }

    #[test]
    fn unset_bit() {
        let mut bm = BitMap::new(64);
        bm.set(8);
        bm.unset(8);
        assert_eq!(bm.get(8), false);
    }

    #[test]
    fn inverse_bit() {
        let mut bm = BitMap::new(64);
        bm.set(8);
        bm.inverse(8);
        assert_eq!(bm.get(8), false);
    }
}
