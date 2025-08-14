pub use exhaustive_map_macros::Finite;
use generic_array::{ArrayLength, typenum::Unsigned};

/// Represents a type that has a finite number of inhabitants.
///
/// If the number of inhabitants is more than `usize::MAX`,
/// such as `usize`, `isize`, `u64`, `i64` and `f64`,
/// then `Finite` should not be implemented for the type.
///
/// Example:
/// ```
/// use exhaustive_map::{Finite, FiniteExt, typenum::Unsigned};
///
/// #[derive(Finite, Debug, PartialEq)]
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
///
/// assert_eq!(<Color as Finite>::INHABITANTS::USIZE, 3);
/// assert_eq!(Color::from_usize(0), Some(Color::Red));
/// assert_eq!(Color::from_usize(1), Some(Color::Green));
/// assert_eq!(Color::from_usize(2), Some(Color::Blue));
/// assert_eq!(Color::from_usize(3), None);
///
/// let all: Vec<_> = Color::iter_all().collect();
/// assert_eq!(all, vec![Color::Red, Color::Green, Color::Blue]);
/// ```
pub trait Finite: Sized {
    /// The total number of different inhabitants of the type.
    ///
    /// This is a [`typenum::Unsigned`](crate::typenum::Unsigned) type-level
    /// number.
    type INHABITANTS: ArrayLength + FitsInUsize;

    /// Should return a number in the range `0..INHABITANTS`.
    #[must_use]
    fn to_usize(&self) -> usize;

    /// Should be the inverse function of `to_usize`.
    ///
    /// This should return `Some` if and only if `i < T::INHABITANTS`.
    #[must_use]
    fn from_usize(i: usize) -> Option<Self>;
}

/// Implemented for [`typenum`](crate::typenum) numbers which fits in an
/// `usize`.
///
/// The number of inhabitants for a [`Finite`] type must implement this trait.
pub trait FitsInUsize: sealed::Sealed {}
impl<T: sealed::Sealed> FitsInUsize for T {}

mod sealed {
    use crate::typenum::{B1, IsLessOrEqual, Pow, Sub1, U, U256, Unsigned};

    type UsizeMax = Sub1<<U256 as Pow<U<{ size_of::<usize>() }>>>::Output>;

    pub trait Sealed {}
    impl<U: Unsigned> Sealed for U where U: IsLessOrEqual<UsizeMax, Output = B1> {}
}

/// An extension for [`Finite`] providing the [`iter_all`](FiniteExt::iter_all)
/// method.
pub trait FiniteExt: Finite {
    /// An iterator over all inhabitants of the type, ordered by the order
    /// provided by [`Finite`].
    fn iter_all() -> IterAll<Self> {
        IterAll((0..Self::INHABITANTS::USIZE).map(|i| {
            Self::from_usize(i).expect("unexpected None returned from Finite::from_usize in range")
        }))
    }
}

impl<T: Finite> FiniteExt for T {}

/// An owned iterator over all inhabitants of a type implementing [`Finite`].
///
/// This `struct` is created by the [`FiniteExt::iter_all`] method.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IterAll<T>(core::iter::Map<core::ops::Range<usize>, fn(usize) -> T>);

impl<T> Iterator for IterAll<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}

impl<T> ExactSizeIterator for IterAll<T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> DoubleEndedIterator for IterAll<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}
