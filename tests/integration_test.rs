use vmem::virtual_memory::VirtualMemory;

#[test]
fn swap_pages_in_buffer() {
    let mut vm = VirtualMemory::new("testfile".to_string(), 9, 3);
    // page size (9) = bitmap size (1) + data size (8)

    // writing to 1 page
    vm.insert(0, 1);
    vm.insert(2, 2);
    vm.insert(4, 3);
    // bit map should be = 00010101 (0x15)

    // writing to 2 page
    vm.insert(8, 4);
    // writing to 3 page
    vm.insert(16, 5);

    // now buffer is full

    // writing to 4 page
    vm.insert(24, 6);

    // 1 page is unloaded from buffer

    // reading from 1 page
    assert_eq!(vm.get(0), Some(1));
    assert_eq!(vm.get(2), Some(2));
    assert_eq!(vm.get(4), Some(3));

    std::fs::remove_file("testfile").unwrap();
}
