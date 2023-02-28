use vmem::VirtualMemory;

#[test]
fn swap_pages_in_buffer() {
    let swap_file = tempfile::tempfile().unwrap();

    let mut vm = VirtualMemory::new(swap_file, 9, 3);
    // page size (9) = bitmap size (1) + data size (8)

    // writing to 1 page
    vm.write(0, 1);
    vm.write(2, 2);
    vm.write(4, 3);
    // bit map should be = 00010101 (0x15)

    // writing to 2 page
    vm.write(8, 4);
    // writing to 3 page
    vm.write(16, 5);

    // now buffer is full

    // writing to 4 page
    vm.write(24, 6);

    // 1 page is unloaded from buffer

    // reading from 1 page
    assert_eq!(vm.read(0), Some(1));
    assert_eq!(vm.read(2), Some(2));
    assert_eq!(vm.read(4), Some(3));
}
