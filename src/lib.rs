mod bitmap;
mod page;
mod data_location;
pub mod virtual_memory;

pub(crate) const BYTE_SIZE: usize = 8;

// round up `dividend` to the nearest usize value of `divisor`
pub(crate) fn div_ceil(dividend: usize, divisor: usize) -> usize {
    (dividend + divisor - 1) / divisor
}
