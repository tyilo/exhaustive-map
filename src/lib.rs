mod finite;
mod map;
mod range;

/// Implements [`Finite`] for a simple enum.
///
/// Example:
/// ```
/// use exhaustive_map::{Finite, FiniteExt, impl_enum};
///
/// #[derive(Debug, PartialEq)]
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
///
/// impl_enum!(Color, [Red, Green, Blue]);
///
/// let all: Vec<_> = Color::iter_all().collect();
/// assert_eq!(all, vec![Color::Red, Color::Green, Color::Blue]);
/// ```
///
/// Every enum variant must be specified:
/// ```compile_fail
/// use exhaustive_map::impl_enum;
///
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
///
/// impl_enum!(Color, [Red, Blue]);
/// ```
pub use exhaustive_map_macros::impl_enum;
pub use finite::{Finite, FiniteExt, IterAll};
pub use map::{ExhaustiveMap, IntoIter, IntoValues, Iter, IterMut, Values, ValuesMut};
pub use range::{InRange, InRangeBounds, InRangeInclusive};

extern crate self as exhaustive_map;
