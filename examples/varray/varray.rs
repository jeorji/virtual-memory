use super::{varray_error, varray_item};
use bincode;
use std::io::{Read, Seek, Write};
use std::marker::PhantomData;
use vmem::VirtualMemory;

type Result<T> = std::result::Result<T, varray_error::Error>;

pub struct VArray<T, RWS>
where
    RWS: Read + Write + Seek,
    for<'a> T: varray_item::Item<'a>,
{
    vm: VirtualMemory<RWS>,
    buffer: Vec<u8>,
    element_size: Option<usize>,
    array_type: PhantomData<T>,
}

impl<T, RWS> VArray<T, RWS>
where
    RWS: Read + Write + Seek,
    for<'a> T: varray_item::Item<'a>,
{
    const PAGE_SIZE: usize = 4096;

    pub fn new(stream: RWS, buffer_size: usize) -> Result<Self> {
        let vm = VirtualMemory::new(stream, Self::PAGE_SIZE, buffer_size);

        Ok(VArray {
            vm,
            buffer: Vec::new(),
            element_size: None,
            array_type: PhantomData,
        })
    }

    pub fn read(&mut self, index: usize) -> Option<T> {
        let size = self.element_size?;
        let start = index * size;

        for i in start..(start + size) {
            if let Some(byte) = self.vm.read(i) {
                self.buffer[i - start] = byte;
            } else {
                return None;
            }
        }

        bincode::deserialize::<T>(&self.buffer).ok()
    }

    pub fn write(&mut self, index: usize, element: T) -> Result<()> {
        if self.element_size.is_none() {
            self.element_size = Some(bincode::serialized_size(&element)? as usize);
        }
        self.buffer = bincode::serialize(&element)?;
        
        let size = self.element_size.unwrap();
        let start = index * size;

        for i in start..(start + size) {
            self.vm.write(i, self.buffer[i - start]);
        }

        Ok(())
    }
}
