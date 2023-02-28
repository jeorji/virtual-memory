use std::mem;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub enum DataLocation<T, const N: usize>
where
    T: Default + Copy,
{
    Inline(usize, [T; N]),
    Heap(Vec<T>),
}

impl<T, const N: usize> DataLocation<T, N>
where
    T: Default + Copy,
{
    pub fn new(size: usize) -> Self {
        // amount of bytes needed to hold `size` elements of type `T`
        let in_bytes = size * mem::size_of::<T>();
        if in_bytes > N {
            Self::Heap(vec![T::default(); size])
        } else {
            Self::Inline(size, [T::default(); N])
        }
    }
}

impl<T, const N: usize> Index<usize> for DataLocation<T, N>
where
    T: Default + Copy,
{
    type Output = T;

    fn index(&self, byte_index: usize) -> &Self::Output {
        match self {
            Self::Inline(len, bytes) => {
                assert!(
                    byte_index < *len,
                    "index out of bounds: the len is {} but the index is {}",
                    len,
                    byte_index
                );
                &bytes[byte_index]
            }
            Self::Heap(bytes) => &bytes[byte_index],
        }
    }
}

impl<T, const N: usize> IndexMut<usize> for DataLocation<T, N>
where
    T: Default + Copy,
{
    fn index_mut(&mut self, byte_index: usize) -> &mut Self::Output {
        match self {
            Self::Inline(len, bytes) => {
                assert!(
                    byte_index < *len,
                    "index out of bounds: the len is {} but the index is {}",
                    len,
                    byte_index
                );
                &mut bytes[byte_index]
            }
            Self::Heap(bytes) => &mut bytes[byte_index],
        }
    }
}

impl<T, const N: usize> From<&[T]> for DataLocation<T, N>
where
    T: Default + Copy,
{
    fn from(value: &[T]) -> Self {
        let in_bytes = value.len() * mem::size_of::<T>();
        if in_bytes > N {
            Self::Heap(Vec::from(value))
        } else {
            let ptr = value.as_ptr() as *const [T; N];
            // deref is safe because the size of the `value` is checked
            // and it will fit the stack-allocated array
            Self::Inline(value.len(), unsafe { *ptr })
        }
    }
}

impl<T, const N: usize> AsRef<[T]> for DataLocation<T, N>
where
    T: Default + Copy,
{
    fn as_ref(&self) -> &[T] {
        match &self {
            DataLocation::Inline(len, v) => &v[..*len],
            DataLocation::Heap(v) => v,
        }
    }
}

#[cfg(test)]
mod test {
    use super::DataLocation;

    #[test]
    fn index_operator() {
        // on stack
        let mut stack_region = DataLocation::<u8, 8>::new(8);
        assert_eq!(stack_region[0], <u8>::default());
        stack_region[0] = 42;
        assert_eq!(stack_region[0], 42);

        // on heap
        let mut heap_region = DataLocation::<u8, 8>::new(100);
        assert_eq!(heap_region[0], <u8>::default());
        heap_region[0] = 42;
        assert_eq!(heap_region[0], 42);
    }

    #[test]
    fn new_heap() {
        let mr = DataLocation::<u8, 64>::new(65);
        match mr {
            DataLocation::Heap(v) => assert_eq!(v.len(), 65),
            _ => panic!("Expected DataLocation::Heap"),
        }
    }

    #[test]
    fn new_stack() {
        let mr = DataLocation::<u8, 64>::new(64);
        match mr {
            DataLocation::Inline(len, _) => assert_eq!(len, 64),
            _ => panic!("Expected DataLocation::Inline"),
        }
    }

    #[test]
    #[should_panic]
    fn stack_out_of_bounds() {
        let mut mr = DataLocation::<u8, 8>::new(2);
        mr[0] = 1;
        mr[1] = 2;
        // this should panic
        mr[2];
    }

    #[test]
    #[should_panic]
    fn heap_out_of_bounds() {
        let mut mr = DataLocation::<u8, 64>::new(65);
        mr[64] = 1;
        mr[65] = 2;
        // this should panic
        mr[67];
    }

    #[test]
    fn from_slice_heap() {
        let mr = DataLocation::<u8, 64>::from(vec![0; 65].as_ref());
        match mr {
            DataLocation::Heap(v) => assert_eq!(v, vec![0; 65]),
            _ => panic!("Expected DataLocation::Heap"),
        }
    }

    #[test]
    fn from_slice_stack() {
        let mr = DataLocation::<u8, 64>::from([1, 2, 3].as_ref());
        match mr {
            DataLocation::Inline(len, v) => {
                assert_eq!(len, 3);
                assert_eq!(&v[..len], [1, 2, 3].as_ref());
            }
            _ => panic!("Expected DataLocation::Inline"),
        }
    }
}
