use crate::page::Page;
use crate::BYTE_SIZE;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Debug)]
pub struct VirtualMemory {
    swap_file: File,
    buffer: Vec<Page>,
    page_size: usize,
}

impl VirtualMemory {
    const SIGNATURE: &[u8; 2] = b"VM";
    pub fn new(file_name: String, page_size: usize, buffer_size: usize) -> Self {
        assert!(
            buffer_size > 2,
            "Vritual memory should have buffer size >= 3"
        );

        let mut swap_file = File::options()
            .write(true)
            .read(true)
            .create(true)
            .truncate(true)
            .open(file_name)
            .unwrap();

        swap_file
            .write(Self::SIGNATURE)
            .expect("Failed to write signature to swap file");

        let buffer: Vec<Page> = Vec::with_capacity(buffer_size);

        VirtualMemory {
            swap_file,
            buffer,
            page_size,
        }
    }

    pub fn insert(&mut self, index: usize, element: u8) {
        let page_index = index / self.page_size;
        let value_offset = index % self.page_size;
        self.page_mut(page_index).set_value(value_offset, element);
    }

    // mut because access_time of value mb changed
    pub fn get(&mut self, index: usize) -> Option<u8> {
        let page_index = index / self.page_size;
        let value_offset = index % self.page_size;
        self.page_mut(page_index).get_value(value_offset)
    }

    pub fn remove(&mut self, index: usize) -> Option<u8> {
        let page_index = index / self.page_size;
        let value_offset = index % self.page_size;
        let page = self.page_mut(page_index);
        let value = page.get_value(value_offset);
        page.remove_value(value_offset);
        value
    }

    fn page_mut(&mut self, index: usize) -> &mut Page {
        let page = self.buffer.iter().find(|e| e.index == index);
        if page.is_none() {
            self.load_page(index);
        }

        self.buffer
            .iter_mut()
            .find(|e| e.index == index)
            .expect("Failed to find page in buffer")
    }

    fn page_byte_size(&self) -> usize {
        let bitmap_size = (self.page_size + BYTE_SIZE - 1) / BYTE_SIZE;
        self.page_size + bitmap_size
    }

    fn page_offset(&self, page_index: usize) -> u64 {
        (page_index * self.page_byte_size() + Self::SIGNATURE.len()) as u64
    }

    fn is_buffer_full(&self) -> bool {
        // buffer capacity defined at init,
        // stays constant during object's lifetime
        self.buffer.len() == self.buffer.capacity()
    }

    fn drop_oldest_page(&mut self) {
        self.buffer.sort_by_key(|e| e.last_access);
        self.unload_page(self.buffer[0].index);
    }

    // load page from file to vec buffer
    fn load_page(&mut self, page_index: usize) {
        if self.is_buffer_full() {
            self.drop_oldest_page();
        }

        // set cursor to the start of the page in the file
        let offset = SeekFrom::Start(self.page_offset(page_index));
        self.swap_file.seek(offset).unwrap();

        let mut bytes = vec![0u8; self.page_byte_size()];
        self.swap_file
            .read(&mut bytes)
            .expect("Failed to read page");

        let page = Page::new(page_index, self.page_size, bytes);
        self.buffer.push(page);
    }

    fn unload_page(&mut self, page_index: usize) {
        let page = self
            .buffer
            .iter()
            .find(|e| e.index == page_index)
            .expect("Failed to find page in buffer");

        if page.is_modified {
            let offset = SeekFrom::Start(self.page_offset(page_index));
            self.swap_file.seek(offset).unwrap();

            self.swap_file
                .write(page.bitmap.as_ref())
                .expect("Failed to write bitmap ti swap file");
            self.swap_file
                .write(page.values.as_ref())
                .expect("Failed to write bitmap ti swap file");
        }

        self.buffer.retain(|e| e.index != page_index);
    }
}

impl Drop for VirtualMemory {
    fn drop(&mut self) {
        while !self.buffer.is_empty() {
            let page = &self.buffer[0];
            self.unload_page(page.index);
        }
    }
}

#[cfg(test)]
mod test {

    use super::VirtualMemory;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn insert_get_remove() {
        let mut virtual_memory = VirtualMemory::new("testfile_igr".to_string(), 4, 3);
        virtual_memory.insert(0, 1);
        virtual_memory.insert(1, 2);
        virtual_memory.insert(2, 3);
        virtual_memory.insert(3, 4);

        assert_eq!(virtual_memory.get(0), Some(1));
        assert_eq!(virtual_memory.get(1), Some(2));
        assert_eq!(virtual_memory.get(2), Some(3));
        assert_eq!(virtual_memory.get(3), Some(4));

        assert_eq!(virtual_memory.remove(0), Some(1));
        assert_eq!(virtual_memory.get(0), None);

        std::fs::remove_file("testfile_igr").unwrap();
    }

    #[test]
    fn page_byte_size() {
        let vm = VirtualMemory::new("testfile_page_bsize".to_string(), 16, 3);
        // page size = 16, bitmap size = 2
        assert_eq!(vm.page_byte_size(), 2 + 16);

        std::fs::remove_file("testfile_page_bsize").unwrap();
    }

    #[test]
    fn page_offset() {
        let vm = VirtualMemory::new("testfile_poffset".to_string(), 16, 3);
        // page size = 16, bitmap size = 2
        assert_eq!(vm.page_offset(0), 2);
        assert_eq!(vm.page_offset(1), 20);
        assert_eq!(vm.page_offset(2), 38);

        std::fs::remove_file("testfile_poffset").unwrap();
    }

    #[test]
    fn is_buffer_full() {
        let mut vm = VirtualMemory::new("testfile_buffer_full".to_string(), 16, 3);
        assert!(!vm.is_buffer_full());
        vm.insert(0, 0);
        vm.insert(16, 0);
        vm.insert(32, 0);
        assert!(vm.is_buffer_full());

        std::fs::remove_file("testfile_buffer_full").unwrap();
    }

    #[test]
    fn drop_oldest_page() {
        let mut vm = VirtualMemory::new("testfile_drop_oldest".to_string(), 16, 3);
        vm.insert(0, 1);
        vm.insert(16, 2);
        vm.insert(32, 3);
        vm.drop_oldest_page();
        assert_eq!(vm.buffer.len(), 2);
        assert_eq!(vm.buffer[0].index, 1);
        assert_eq!(vm.buffer[1].index, 2);

        std::fs::remove_file("testfile_drop_oldest").unwrap();
    }

    #[test]
    fn load_page() {
        let mut vm = VirtualMemory::new("testfile_load".to_string(), 16, 3);
        vm.load_page(0);
        assert_eq!(vm.buffer.len(), 1);

        std::fs::remove_file("testfile_load").unwrap();
    }

    #[test]
    fn unload_page() {
        let mut vm = VirtualMemory::new("testfile_unload".to_string(), 8, 3);
        vm.insert(0, 1);
        vm.unload_page(0);
        assert_eq!(vm.buffer.len(), 0);

        let mut file = File::open("testfile_unload").unwrap();
        // sign = 2, bitmap = 1, page size = 8
        let mut buffer = [0u8; 2 + 1 + 8];
        file.read(&mut buffer).unwrap();
        assert_eq!(buffer, [b'V', b'M', 1, 1, 0, 0, 0, 0, 0, 0, 0]);

        std::fs::remove_file("testfile_unload").unwrap();
    }
}
