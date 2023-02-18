use crate::bitmap::BitMap;
use crate::BYTE_SIZE;
use std::time::SystemTime;

#[derive(Debug)]
pub(crate) struct Page {
    pub index: usize,
    pub is_modified: bool,
    pub last_access: SystemTime,
    pub bitmap: BitMap,
    pub values: Vec<u8>,
}

impl Page {
    pub fn new(index: usize, size: usize, data: Vec<u8>) -> Self {
        let bitmap_size = (size + BYTE_SIZE - 1) / BYTE_SIZE;
        let (bitmap, values) = data.split_at(bitmap_size);
        Page {
            index,
            is_modified: false,
            last_access: SystemTime::now(),
            bitmap: BitMap::from(bitmap),
            values: Vec::from(values),
        }
    }

    pub fn set_value(&mut self, index: usize, value: u8) {
        self.is_modified = true;
        self.last_access = SystemTime::now();
        self.bitmap.set(index);
        self.values[index] = value;
    }

    pub fn get_value(&mut self, index: usize) -> Option<u8> {
        match self.bitmap.get(index) {
            true => {
                self.last_access = SystemTime::now();
                Some(self.values[index])
            }
            false => None,
        }
    }

    pub fn remove_value(&mut self, index: usize) {
        self.is_modified = true;
        self.last_access = SystemTime::now();
        self.bitmap.unset(index);
        self.values.remove(index);
    }
}

#[cfg(test)]
mod test {
    use super::Page;

    #[test]
    fn set_value() {
        let mut page = Page::new(0, 8, vec![0; 1 + 8]);
        page.set_value(3, 1);
        assert_eq!(page.is_modified, true);
        assert_eq!(page.bitmap.get(3), true);
        assert_eq!(page.values, vec![0, 0, 0, 1, 0, 0, 0, 0]);
    }

    #[test]
    fn get_value() {
        let mut page = Page::new(0, 8, vec![0; 1 + 8]);
        page.set_value(3, 1);
        assert_eq!(page.get_value(3), Some(1));
        assert_eq!(page.get_value(2), None);
    }

    #[test]
    fn remove_value() {
        let mut page = Page::new(0, 8, vec![0; 1 + 8]);
        page.set_value(3, 1);
        page.remove_value(3);
        assert_eq!(page.is_modified, true);
        assert_eq!(page.bitmap.get(3), false);
        assert_eq!(page.values, vec![0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn access_time_update() {
        let mut page = Page::new(0, 8, vec![0; 1 + 8]);
        let old_modification_time = page.last_access;

        std::thread::sleep(std::time::Duration::from_millis(1));
        page.set_value(2, 42);

        assert!(page.last_access > old_modification_time);
    }
}
