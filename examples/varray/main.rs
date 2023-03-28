mod varray;
mod varray_error;
mod varray_item;

use serde::{Deserialize, Serialize};
use varray::VArray;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct Record {
    value: f64,
    timestamp: i128,
}

impl varray_item::Item<'_> for [Record; 3] {}

fn main() {
    let swap_file = tempfile::tempfile().unwrap();

    // array: VArray<[Record; 3], File>
    let mut array = VArray::new(swap_file, 4).unwrap();

    let r = Record {
        value: 913.08491,
        timestamp: 109247203947,
    };

    for i in 0..100 {
        array.write(i, [r, r, r]).unwrap();
    }

    dbg! {array.read(99)};
}
