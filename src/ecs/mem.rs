#![allow(dead_code)]

use std::{
    alloc::{alloc, realloc, Layout},
    ptr::NonNull,
};

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub(crate) enum Error {
    MismatchedLayout(Layout, Layout),
    NullAllocation,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::MismatchedLayout(l1, l2) => format!(
                    "Layout of element, {:?}, does not match layout of allocated memory, {:?}",
                    l1, l2
                ),
                Self::NullAllocation =>
                    "Attempted to allocate memory and received a null pointer".to_string(),
            }
        )
    }
}

pub(crate) struct AlignedVec {
    layout: Layout,
    ptr: NonNull<u8>,
    stride: usize,
    len: usize,
    cap: usize,
    drop_in_place: fn(NonNull<u8>, usize, usize),
}

impl Drop for AlignedVec {
    fn drop(&mut self) {
        (self.drop_in_place)(self.ptr, self.len, self.stride);

        unsafe {
            std::alloc::dealloc(self.ptr.as_ptr(), self.layout.repeat(self.cap).unwrap().0);
        }
    }
}

impl AlignedVec {
    pub(crate) fn new<T>() -> Self {
        let layout = Layout::new::<T>();
        let (array_layout, stride) = layout.repeat(4).unwrap();

        Self {
            layout,
            ptr: NonNull::new(unsafe { alloc(array_layout) })
                .expect("Received null pointer on alloc"),
            stride,
            len: 0,
            cap: 4,
            drop_in_place: |non_null_ptr, count, stride| {
                // TODO: change to raw fn
                for i in 0..count {
                    unsafe { non_null_ptr.add(i * stride).cast::<T>().drop_in_place() };
                }
            },
        }
    }

    pub(crate) fn try_new<T>() -> Result<Self> {
        let layout = Layout::new::<T>();
        let array_layout = layout.repeat(4).unwrap(); // TODO: change to array

        Ok(Self {
            layout,
            ptr: NonNull::new(unsafe { alloc(array_layout.0) }).ok_or(Error::NullAllocation)?,
            stride: array_layout.1,
            len: 0,
            cap: 4,
            drop_in_place: |non_null_ptr, count, stride| {
                for i in 0..count {
                    unsafe { non_null_ptr.add(i * stride).cast::<T>().drop_in_place() };
                }
            },
        })
    }

    pub(crate) fn try_push<T>(&mut self, e: T) -> Result<()> {
        if self.layout != Layout::new::<T>() {
            return Err(Error::MismatchedLayout(self.layout, Layout::new::<T>()));
        }

        if self.len >= self.cap {
            self.ptr = unsafe {
                NonNull::new(realloc(
                    self.ptr.as_ptr(),
                    self.layout.repeat(self.cap).unwrap().0,
                    self.layout.repeat(self.cap * 2).unwrap().0.size(),
                ))
                .ok_or(Error::NullAllocation)?
            };
            self.cap *= 2;
        }

        unsafe { self.ptr.add(self.len * self.stride).cast::<T>().write(e) };
        self.len += 1;

        Ok(())
    }

    pub(crate) fn push<T>(&mut self, e: T) {
        if self.len >= self.cap {
            self.ptr = unsafe {
                NonNull::new(realloc(
                    self.ptr.as_ptr(),
                    self.layout.repeat(self.cap).unwrap().0,
                    self.layout.repeat(self.cap * 2).unwrap().0.size(),
                ))
                .expect("Received null pointer on realloc")
            };
            self.cap *= 2;
        }

        unsafe { self.ptr.add(self.len * self.stride).cast::<T>().write(e) };
        self.len += 1;
    }

    pub(crate) fn as_slice<T>(&self) -> Result<&[T]> {
        if self.layout != Layout::new::<T>() {
            return Err(Error::MismatchedLayout(self.layout, Layout::new::<T>()));
        }

        Ok(unsafe { NonNull::slice_from_raw_parts(self.ptr.cast::<T>(), self.len).as_ref() })
    }

    pub(crate) fn as_slice_mut<T>(&mut self) -> Result<&mut [T]> {
        if self.layout != Layout::new::<T>() {
            return Err(Error::MismatchedLayout(self.layout, Layout::new::<T>()));
        }

        Ok(unsafe { NonNull::slice_from_raw_parts(self.ptr.cast::<T>(), self.len).as_mut() })
    }

    pub(crate) fn as_slice_unchecked<T>(&self) -> &[T] {
        unsafe { NonNull::slice_from_raw_parts(self.ptr.cast::<T>(), self.len).as_ref() }
    }

    pub(crate) fn as_slice_unchecked_mut<T>(&mut self) -> &mut [T] {
        unsafe { NonNull::slice_from_raw_parts(self.ptr.cast::<T>(), self.len).as_mut() }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    struct ArbitraryData {
        a: f32,
        b: f64,
        c: u8,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct HeapData {
        a: std::rc::Rc<u8>,
        b: String,
    }

    #[repr(C, align(32))]
    #[derive(Debug, PartialEq, Eq)]
    struct OddlyAlignedArbitraryData(u8);

    #[test]
    fn test_layout() {
        let vec_u32 = AlignedVec::new::<u32>();
        let vec_arbitrary = AlignedVec::new::<ArbitraryData>();

        assert_eq!(vec_u32.layout, Layout::new::<u32>());
        assert_eq!(vec_arbitrary.layout, Layout::new::<ArbitraryData>());
    }

    #[test]
    fn test_slice() {
        let mut vec = AlignedVec::new::<ArbitraryData>();

        vec.push(ArbitraryData {
            a: 32.0_f32,
            b: 47.5_f64,
            c: 45_u8,
        });
        vec.push(ArbitraryData {
            a: 53250.7_f32,
            b: 16.0_f64,
            c: 35_u8,
        });

        let slice = vec.as_slice_unchecked::<ArbitraryData>();

        assert_eq!(slice[0].a, 32.0_f32);
        assert_eq!(slice[0].b, 47.5_f64);
        assert_eq!(slice[0].c, 45_u8);

        assert_eq!(slice[1].a, 53250.7_f32);
        assert_eq!(slice[1].b, 16.0_f64);
        assert_eq!(slice[1].c, 35_u8);
    }

    #[test]
    fn test_slice_mut() {
        let mut vec = AlignedVec::new::<ArbitraryData>();

        vec.push(ArbitraryData {
            a: 32.0_f32,
            b: 47.5_f64,
            c: 45_u8,
        });
        vec.push(ArbitraryData {
            a: 53250.7_f32,
            b: 16.0_f64,
            c: 35_u8,
        });

        let mut_slice = vec.as_slice_unchecked_mut::<ArbitraryData>();

        mut_slice[0] = ArbitraryData {
            a: 33.5_f32,
            b: 46.4_f64,
            c: 16_u8,
        };

        let slice = vec.as_slice_unchecked::<ArbitraryData>();

        assert_eq!(slice[0].a, 33.5_f32);
        assert_eq!(slice[0].b, 46.4_f64);
        assert_eq!(slice[0].c, 16_u8);

        assert_eq!(slice[1].a, 53250.7_f32);
        assert_eq!(slice[1].b, 16.0_f64);
        assert_eq!(slice[1].c, 35_u8);
    }

    #[test]
    fn test_expansion() {
        let mut vec = AlignedVec::new::<u16>();

        for i in 0..20 {
            vec.push(i);
        }

        assert_eq!(
            vec.as_slice_unchecked::<u16>(),
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
        );
        assert_eq!(vec.len(), 20);
        assert_eq!(vec.cap, 32);
    }

    #[test]
    fn test_odd_alignment() {
        let mut vec = AlignedVec::new::<OddlyAlignedArbitraryData>();

        vec.push(OddlyAlignedArbitraryData(1));
        vec.push(OddlyAlignedArbitraryData(2));
        vec.push(OddlyAlignedArbitraryData(3));

        assert_eq!(
            vec.as_slice_unchecked::<OddlyAlignedArbitraryData>(),
            &[
                OddlyAlignedArbitraryData(1),
                OddlyAlignedArbitraryData(2),
                OddlyAlignedArbitraryData(3)
            ]
        );
        assert_matches!(vec.as_slice::<u8>(), Err(Error::MismatchedLayout(..)));
        assert_eq!(vec.stride, 32);
    }

    #[test]
    fn test_empty_drop() {
        let vec = AlignedVec::new::<HeapData>();

        assert_eq!(vec.as_slice_unchecked::<HeapData>(), &[]);

        drop(vec)
    }

    #[test]
    fn test_heap_drop() {
        let mut vec = AlignedVec::new::<HeapData>();

        let rc = std::rc::Rc::new(5_u8);

        vec.push(HeapData {
            a: rc.clone(),
            b: "Hello".to_string(),
        });
        vec.push(HeapData {
            a: rc.clone(),
            b: "Sample".to_string(),
        });

        assert_eq!(std::rc::Rc::strong_count(&rc), 3);

        drop(vec);

        assert_eq!(std::rc::Rc::strong_count(&rc), 1);
    }

    #[test]
    fn test_try_push_error_handling() {
        let mut vec = AlignedVec::new::<u16>();
        assert_matches!(vec.try_push(5_u8), Err(Error::MismatchedLayout(..)));

        let mut vec = AlignedVec::new::<OddlyAlignedArbitraryData>();
        assert_matches!(vec.try_push(5_u8), Err(Error::MismatchedLayout(..)));
    }

    // Overflow not tested because it would require 18446744073.709553 Gb of RAM for a u8 on a 64
    // bit computer, which means it should be safe to ignore for now.
}
