use crate::differ::LogDifferences;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_vec_from;
use crate::utils::write_utils::WriteTo;
use std::io;
use std::io::Read;
use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;

/// A heap-allocated fixed-size array of `N` elements of `T`.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxedArray<T, const N: usize>(pub(crate) Box<[T; N]>);

impl<T: Default, const N: usize> BoxedArray<T, N> {
    /// Builds a `BoxedArray` from a `Vec`, padding with `T::default()` if it has fewer than `N`
    /// elements, or discarding the excess if it has more.
    pub(crate) fn from_vec(mut vec: Vec<T>) -> Self {
        vec.resize_with(N, T::default);

        // The vec now always has exactly N elements, so this conversion cannot fail.
        return vec.into_boxed_slice().try_into().map(BoxedArray).unwrap_or_else(|_| unreachable!());
    }
}

impl<T, const N: usize> Deref for BoxedArray<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl<T, const N: usize> DerefMut for BoxedArray<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.0;
    }
}

impl<T: Default, const N: usize> Default for BoxedArray<T, N> {
    fn default() -> Self {
        return BoxedArray::from_vec(Vec::new());
    }
}

impl<T: ReadFrom + Default, const N: usize> ReadFrom for BoxedArray<T, N> {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let vec: Vec<T> = read_vec_from(reader, N)?;
        return Ok(BoxedArray::from_vec(vec));
    }
}

impl<T: WriteTo, const N: usize> WriteTo for BoxedArray<T, N> {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        return WriteTo::write_to(self.0.as_ref(), writer);
    }
}

impl<T: LogDifferences, const N: usize> LogDifferences for BoxedArray<T, N> {
    fn log_differences(a: &Self, b: &Self, context: String) {
        LogDifferences::log_differences(a.0.as_ref(), b.0.as_ref(), context);
    }
}
