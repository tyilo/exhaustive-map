#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]

mod finite;
mod map;
mod range;

pub use finite::{Finite, FiniteExt, IterAll};
pub use map::{ExhaustiveMap, IntoIter, IntoValues, Iter, IterMut, Values, ValuesMut};
pub use range::{InRange, InRangeBounds, InRangeInclusive};

extern crate self as exhaustive_map;
