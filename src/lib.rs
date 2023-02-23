mod bitmap;
mod data_location;
mod page;
mod virtual_memory;

pub use virtual_memory::VirtualMemory;

pub(crate) const BITS_IN_BYTE: usize = 8;

// round up `dividend` to the nearest usize value of `divisor`
pub(crate) fn div_ceil(dividend: usize, divisor: usize) -> usize {
    (dividend + divisor - 1) / divisor
}
