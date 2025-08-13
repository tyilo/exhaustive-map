use std::{
    marker::PhantomData,
    ops::{Add, Sub},
};

use generic_array::ArrayLength;

use crate::{
    typenum::{Unsigned, B1},
    Finite, FitsInUsize,
};

/// A `usize` value that is guaranteed to be in the range `A..B`.
///
/// Common methods are in the [`InRangeBounds`] trait implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InRange<A: Unsigned, B: Unsigned>
where
    B: Sub<A>,
    <B as Sub<A>>::Output: Unsigned,
{
    value: usize,
    _phantom: PhantomData<(A, B)>,
}

/// A `usize` value that is guaranteed to be in the range `A..=B`.
///
/// Common methods are in the [`InRangeBounds`] trait implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InRangeInclusive<A: Unsigned, B: Unsigned>
where
    B: Sub<A>,
    <B as Sub<A>>::Output: Add<B1>,
    <<B as Sub<A>>::Output as Add<B1>>::Output: ArrayLength,
{
    value: usize,
    _phantom: PhantomData<(A, B)>,
}

pub trait InRangeBounds: Copy + Sized {
    /// The smallest value representable (if `INHABITANTS` is non-zero).
    type MIN: Unsigned;

    /// The number of values representable.
    type INHABITANTS: ArrayLength + FitsInUsize;

    /// Creates a value without checking whether the value is in range. This results in undefined behavior if the value is not in range.
    ///
    /// # Safety
    /// `i` must satisfy `Self::MIN <= i` and `i < Self::MIN + Self::INHABITANTS`.
    #[must_use]
    unsafe fn new_unchecked(i: usize) -> Self;

    /// Returns the value as a `usize`.
    #[must_use]
    fn get(self) -> usize;

    /// Same as `InRangeBounds::new(Self::MIN + i)`.
    #[must_use]
    fn new_from_start_offset(offset: usize) -> Option<Self> {
        Self::new(Self::MIN::USIZE + offset)
    }

    /// Returns the offset from `Self::MIN` if `i` is in range.
    #[must_use]
    fn offset_from_start(i: usize) -> Option<usize> {
        let offset = i.checked_sub(Self::MIN::USIZE)?;
        if offset < Self::INHABITANTS::USIZE {
            Some(offset)
        } else {
            None
        }
    }

    /// Returns whether `i` is in range.
    #[must_use]
    fn in_bounds(i: usize) -> bool {
        Self::offset_from_start(i).is_some()
    }

    /// Creates a value if the given value is in range.
    #[must_use]
    fn new(i: usize) -> Option<Self> {
        if Self::in_bounds(i) {
            // SAFETY: `i` is in bounds.
            Some(unsafe { Self::new_unchecked(i) })
        } else {
            None
        }
    }
}

impl<A: Unsigned, B: Unsigned> InRangeBounds for InRange<A, B>
where
    B: Sub<A>,
    <B as Sub<A>>::Output: ArrayLength + FitsInUsize,
{
    type MIN = A;
    type INHABITANTS = <B as Sub<A>>::Output;

    unsafe fn new_unchecked(i: usize) -> Self {
        Self {
            value: i,
            _phantom: PhantomData,
        }
    }

    fn get(self) -> usize {
        self.value
    }
}

impl<A: Unsigned, B: Unsigned> InRangeBounds for InRangeInclusive<A, B>
where
    B: Sub<A>,
    <B as Sub<A>>::Output: Add<B1>,
    <<B as Sub<A>>::Output as Add<B1>>::Output: ArrayLength + FitsInUsize,
{
    type MIN = A;
    type INHABITANTS = <<B as Sub<A>>::Output as Add<B1>>::Output;

    unsafe fn new_unchecked(i: usize) -> Self {
        Self {
            value: i,
            _phantom: PhantomData,
        }
    }

    fn get(self) -> usize {
        self.value
    }
}

impl<A: Unsigned, B: Unsigned> Finite for InRange<A, B>
where
    B: Sub<A>,
    <B as Sub<A>>::Output: ArrayLength + FitsInUsize,
{
    type INHABITANTS = <Self as InRangeBounds>::INHABITANTS;

    fn to_usize(&self) -> usize {
        self.get() - <Self as InRangeBounds>::MIN::USIZE
    }

    fn from_usize(i: usize) -> Option<Self> {
        Self::new_from_start_offset(i)
    }
}

impl<A: Unsigned, B: Unsigned> Finite for InRangeInclusive<A, B>
where
    B: Sub<A>,
    <B as Sub<A>>::Output: Add<B1>,
    <<B as Sub<A>>::Output as Add<B1>>::Output: ArrayLength + FitsInUsize,
{
    type INHABITANTS = <Self as InRangeBounds>::INHABITANTS;

    fn to_usize(&self) -> usize {
        self.get() - <Self as InRangeBounds>::MIN::USIZE
    }

    fn from_usize(i: usize) -> Option<Self> {
        Self::new_from_start_offset(i)
    }
}

#[cfg(test)]
mod test {
    use std::{fmt::Debug, ops::RangeBounds};

    use super::*;
    use crate::typenum::{Pow, Sub1, U, U0, U1, U256, U3};

    type UsizeMax = Sub1<<U256 as Pow<U<{ std::mem::size_of::<usize>() }>>>::Output>;

    fn test_range<T: InRangeBounds + Debug + PartialEq, R: RangeBounds<usize>>(expected_range: R) {
        for i in (0..10).chain(usize::MAX - 10..=usize::MAX) {
            let v = T::new(i);
            if expected_range.contains(&i) {
                assert_eq!(v.map(InRangeBounds::get), Some(i));
            } else {
                assert_eq!(v, None);
            }
        }
    }

    #[test]
    fn test_in_range_full() {
        test_range::<InRange<U0, UsizeMax>, _>(0..usize::MAX);
    }

    #[test]
    fn test_in_range_inclusive_almost_full() {
        test_range::<InRangeInclusive<U1, UsizeMax>, _>(1..=usize::MAX);
    }

    #[test]
    fn test_in_range() {
        test_range::<InRange<U1, U3>, _>(1..3);
    }

    #[test]
    fn test_in_range_inclusive() {
        test_range::<InRangeInclusive<U1, U3>, _>(1..=3);
    }
}
