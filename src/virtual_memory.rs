use crate::page::Page;
use crate::BITS_IN_BYTE;
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Debug)]
pub struct VirtualMemory<RWS>
where
    RWS: Read + Write + Seek,
{
    swap_source: RWS,
    buffer: Vec<Page>,
    page_size: usize,
    max_index: usize,
}

impl<RWS> VirtualMemory<RWS>
where
    RWS: Read + Write + Seek,
{
    const SIGNATURE: &[u8; 2] = b"VM";
    pub fn new(mut swap_source: RWS, page_size: usize, buffer_size: usize) -> Self {
        assert!(
            buffer_size > 2,
            "Virtual memory should have buffer size > 2"
        );
        assert!(page_size > 1, "Virtual memory should have page size > 1");

        swap_source
            .write(Self::SIGNATURE)
            .expect("Failed to write signature to swap file");

        let buffer: Vec<Page> = Vec::with_capacity(buffer_size);

        VirtualMemory {
            swap_source,
            buffer,
            page_size,
            max_index: 0,
        }
    }

    pub fn write(&mut self, index: usize, element: u8) {
        self.max_index = self.max_index.max(index);

        let page_index = index / self.data_size();
        let value_offset = index % self.data_size();
        self.page_mut(page_index).set_value(value_offset, element);
    }

    // mut because access_time of value mb changed
    pub fn read(&mut self, index: usize) -> Option<u8> {
        if index > self.max_index {
            return None;
        }

        let page_index = index / self.data_size();
        let value_offset = index % self.data_size();
        self.page_mut(page_index).get_value(value_offset)
    }

    pub fn remove(&mut self, index: usize) -> Option<u8> {
        let page_index = index / self.data_size();
        let value_offset = index % self.data_size();
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

    fn data_size(&self) -> usize {
        // The data section size is 8/9 of the byte page size
        // 1/9 is bitmap
        self.page_size * BITS_IN_BYTE / 9
    }

    fn page_offset(&self, page_index: usize) -> u64 {
        (page_index * self.page_size + Self::SIGNATURE.len()) as u64
    }

    fn is_buffer_full(&self) -> bool {
        // buffer capacity defined at init,
        // stays constant during object's lifetime
        self.buffer.len() == self.buffer.capacity()
    }

    fn drop_oldest_page(&mut self) {
        self.buffer
            .sort_by_key(|e| std::cmp::Reverse(e.last_access));
        if let Some(last_page) = self.buffer.last() {
            self.unload_page(last_page.index);
        }
    }

    // load page from file to vec buffer
    fn load_page(&mut self, page_index: usize) {
        if self.is_buffer_full() {
            self.drop_oldest_page();
        }

        // set cursor to the start of the page in the file
        let offset = SeekFrom::Start(self.page_offset(page_index));
        self.swap_source.seek(offset).unwrap();

        let mut bytes = vec![0u8; self.page_size];
        self.swap_source
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
            self.swap_source.seek(offset).unwrap();

            self.swap_source
                .write(page.bitmap.as_ref())
                .expect("Failed to write bitmap to swap file");
            self.swap_source
                .write(page.values.as_ref())
                .expect("Failed to write bitmap to swap file");
        }

        self.buffer.retain(|e| e.index != page_index);
    }
}

impl<RWS> Drop for VirtualMemory<RWS>
where
    RWS: Read + Write + Seek,
{
    fn drop(&mut self) {
        while let Some(last_page) = self.buffer.last() {
            self.unload_page(last_page.index);
        }
    }
}

#[cfg(test)]
mod test {
    use super::VirtualMemory;
    use tempfile::tempfile;

    #[test]
    fn write_read_remove() {
        let swap_file = tempfile().unwrap();
        let mut vm = VirtualMemory::new(swap_file, 4, 3);
        vm.write(0, 1);
        vm.write(1, 2);
        vm.write(2, 3);
        vm.write(3, 4);

        assert_eq!(vm.read(0), Some(1));
        assert_eq!(vm.read(1), Some(2));
        assert_eq!(vm.read(2), Some(3));
        assert_eq!(vm.read(3), Some(4));

        assert_eq!(vm.remove(0), Some(1));
        assert_eq!(vm.read(0), None);
    }

    #[test]
    fn data_size() {
        let swap_file = tempfile().unwrap();
        let vm = VirtualMemory::new(swap_file, 16, 3);
        // page size = 16, bitmap size = 2, 16 - 2 = 14 data size
        assert_eq!(vm.data_size(), 14);
    }

    #[test]
    fn page_offset() {
        let swap_file = tempfile().unwrap();
        let vm = VirtualMemory::new(swap_file, 16, 3);
        // page size (16) = bitmap size (2) + values size (14)
        assert_eq!(vm.page_offset(0), 2);
        assert_eq!(vm.page_offset(1), 18);
        assert_eq!(vm.page_offset(2), 34);
    }

    #[test]
    fn is_buffer_full() {
        let swap_file = tempfile().unwrap();
        let mut vm = VirtualMemory::new(swap_file, 16, 3);
        assert!(!vm.is_buffer_full());
        vm.write(0, 0);
        vm.write(16, 0);
        vm.write(32, 0);
        assert!(vm.is_buffer_full());
    }

    #[test]
    fn drop_oldest_page() {
        let swap_file = tempfile().unwrap();
        let mut vm = VirtualMemory::new(swap_file, 16, 3);
        vm.write(0, 1);
        vm.write(16, 2);
        vm.write(32, 3);
        vm.drop_oldest_page();
        assert_eq!(vm.buffer.len(), 2);
        assert_eq!(vm.buffer[0].index, 2);
        assert_eq!(vm.buffer[1].index, 1);
    }

    #[test]
    fn load_page() {
        let swap_file = tempfile().unwrap();
        let mut vm = VirtualMemory::new(swap_file, 16, 3);
        vm.load_page(0);
        assert_eq!(vm.buffer.len(), 1);
    }

    #[test]
    fn unload_page() {
        let swap_file = tempfile().unwrap();
        let mut vm = VirtualMemory::new(swap_file, 8, 3);
        vm.write(0, 1);
        vm.unload_page(0);
        assert_eq!(vm.buffer.len(), 0);
    }
}
