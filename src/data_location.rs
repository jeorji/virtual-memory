use std::mem;
use std::ops::{Index, IndexMut};

const MAX_STACK_SIZE: usize = 64;

#[derive(Debug)]
pub enum DataLocation<T>
where
    T: Default + Copy,
{
    Stack(usize, [T; MAX_STACK_SIZE]),
    Heap(Vec<T>),
}

impl<T> DataLocation<T>
where
    T: Default + Copy,
{
    pub fn new(size: usize) -> Self {
        // amount of bytes needed to hold `size` elements of type `T`
        let in_bytes = size * mem::size_of::<T>();
        if in_bytes > MAX_STACK_SIZE {
            Self::Heap(vec![T::default(); size])
        } else {
            Self::Stack(size, [T::default(); MAX_STACK_SIZE])
        }
    }
}

impl<T> Index<usize> for DataLocation<T>
where
    T: Default + Copy,
{
    type Output = T;

    fn index(&self, byte_index: usize) -> &Self::Output {
        match self {
            Self::Stack(len, bytes) => {
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

impl<T> IndexMut<usize> for DataLocation<T>
where
    T: Default + Copy,
{
    fn index_mut(&mut self, byte_index: usize) -> &mut Self::Output {
        match self {
            Self::Stack(len, bytes) => {
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

impl<T> From<&[T]> for DataLocation<T>
where
    T: Default + Copy,
{
    fn from(value: &[T]) -> Self {
        let in_bytes = value.len() * mem::size_of::<T>();
        if in_bytes > MAX_STACK_SIZE {
            Self::Heap(Vec::from(value))
        } else {
            let ptr = value.as_ptr() as *const [T; MAX_STACK_SIZE];
            // deref is safe because the size of the `value` is checked
            // and it will fit the stack-allocated array
            Self::Stack(value.len(), unsafe { *ptr })
        }
    }
}

impl<T> AsRef<[T]> for DataLocation<T>
where
    T: Default + Copy,
{
    fn as_ref(&self) -> &[T] {
        match &self {
            DataLocation::Stack(len, v) => &v[..*len],
            DataLocation::Heap(v) => v,
        }
    }
}

#[cfg(test)]
mod test {
    use super::DataLocation;
    use super::MAX_STACK_SIZE;

    #[test]
    fn index_operator() {
        // on stack
        let mut stack_region = DataLocation::new(5);
        assert_eq!(stack_region[0], <i32>::default());
        stack_region[0] = 42;
        assert_eq!(stack_region[0], 42);

        // on heap
        let mut heap_region = DataLocation::new(100);
        assert_eq!(heap_region[0], <i32>::default());
        heap_region[0] = 42;
        assert_eq!(heap_region[0], 42);
    }

    #[test]
    fn new_heap() {
        let mr = DataLocation::<i32>::new(MAX_STACK_SIZE + 1);
        match mr {
            DataLocation::Heap(v) => assert_eq!(v.len(), MAX_STACK_SIZE + 1),
            _ => panic!("Expected DataLocation::Heap"),
        }
    }

    #[test]
    fn new_stack() {
        let mr = DataLocation::<u8>::new(MAX_STACK_SIZE - 1);
        match mr {
            DataLocation::Stack(len, _) => assert_eq!(len, MAX_STACK_SIZE - 1),
            _ => panic!("Expected DataLocation::Stack"),
        }
    }

    #[test]
    #[should_panic]
    fn stack_out_of_bounds() {
        let mut mr = DataLocation::<i32>::new(2);
        mr[0] = 1;
        mr[1] = 2;
        // this should panic
        mr[2];
    }

    #[test]
    #[should_panic]
    fn heap_out_of_bounds() {
        let mut mr = DataLocation::<i32>::new(MAX_STACK_SIZE + 1);
        mr[MAX_STACK_SIZE] = 1;
        mr[MAX_STACK_SIZE + 1] = 2;
        // this should panic
        mr[MAX_STACK_SIZE + 2];
    }

    #[test]
    fn from_slice_heap() {
        let mr = DataLocation::from(vec![0; MAX_STACK_SIZE + 1].as_ref());
        match mr {
            DataLocation::Heap(v) => assert_eq!(v, vec![0; MAX_STACK_SIZE + 1]),
            _ => panic!("Expected DataLocation::Heap"),
        }
    }

    #[test]
    fn from_slice_stack() {
        let mr = DataLocation::from([1, 2, 3].as_ref());
        match mr {
            DataLocation::Stack(len, v) => {
                assert_eq!(len, 3);
                assert_eq!(&v[..len], [1, 2, 3].as_ref());
            }
            _ => panic!("Expected DataLocation::Stack"),
        }
    }
}
