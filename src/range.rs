use crate::Finite;

/// A `usize` value that is guaranteed to be in the range `A..B`.
///
/// Common methods are in the [`InRangeBounds`] trait implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InRange<const A: usize, const B: usize>(usize);

/// A `usize` value that is guaranteed to be in the range `A..=B`.
///
/// Common methods are in the [`InRangeBounds`] trait implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InRangeInclusive<const A: usize, const B: usize>(usize);

pub trait InRangeBounds: Copy + Sized {
    /// The smallest value representable (if `INHABITANTS` is non-zero).
    const MIN: usize;

    /// The number of values representable.
    const INHABITANTS: usize;

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
        Self::new(Self::MIN + offset)
    }

    /// Returns the offset from `Self::MIN` if `i` is in range.
    #[must_use]
    fn offset_from_start(i: usize) -> Option<usize> {
        let offset = i.checked_sub(Self::MIN)?;
        if offset < Self::INHABITANTS {
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

impl<const A: usize, const B: usize> InRangeBounds for InRange<A, B> {
    const MIN: usize = A;
    const INHABITANTS: usize = B - A;

    unsafe fn new_unchecked(i: usize) -> Self {
        Self(i)
    }

    fn get(self) -> usize {
        self.0
    }
}

impl<const A: usize, const B: usize> InRangeBounds for InRangeInclusive<A, B> {
    const MIN: usize = A;
    const INHABITANTS: usize = B - A + 1;

    unsafe fn new_unchecked(i: usize) -> Self {
        Self(i)
    }

    fn get(self) -> usize {
        self.0
    }
}

impl<const A: usize, const B: usize> Finite for InRange<A, B> {
    const INHABITANTS: usize = <Self as InRangeBounds>::INHABITANTS;

    fn to_usize(&self) -> usize {
        self.get() - Self::MIN
    }

    fn from_usize(i: usize) -> Option<Self> {
        Self::new_from_start_offset(i)
    }
}

impl<const A: usize, const B: usize> Finite for InRangeInclusive<A, B> {
    const INHABITANTS: usize = <Self as InRangeBounds>::INHABITANTS;

    fn to_usize(&self) -> usize {
        self.get() - Self::MIN
    }

    fn from_usize(i: usize) -> Option<Self> {
        Self::new_from_start_offset(i)
    }
}

#[cfg(test)]
mod test {
    use std::{fmt::Debug, ops::RangeBounds};

    use super::*;

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
        test_range::<InRange<0, { usize::MAX }>, _>(0..usize::MAX);
    }

    #[test]
    fn test_in_range_inclusive_almost_full() {
        test_range::<InRangeInclusive<1, { usize::MAX }>, _>(1..=usize::MAX);
    }

    #[test]
    fn test_in_range() {
        test_range::<InRange<1, 3>, _>(1..3);
    }

    #[test]
    fn test_in_range_inclusive() {
        test_range::<InRangeInclusive<1, 3>, _>(1..=3);
    }
}
