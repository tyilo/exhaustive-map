#![doc = include_str!("../README.md")]
//! ## Features
//!
//! - `serde` - Enables serialization and deserialization of [`ExhaustiveMap`]. Example:
//! ```
//! # #[cfg(feature = "serde")]
//! # {
//! use exhaustive_map::{ExhaustiveMap, Finite};
//! use serde::Serialize;
//!
//! #[derive(Finite, Serialize)]
//! enum Color {
//!     Red,
//!     Green,
//!     Blue,
//! }
//!
//! let map = ExhaustiveMap::<Color, _>::from_usize_fn(|i| i);
//! let json = serde_json::to_string(&map).unwrap();
//! assert_eq!(json, r#"{"Red":0,"Green":1,"Blue":2}"#);
//! # }
//! ```
#![warn(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]

mod finite;
mod map;
mod range;

pub use finite::{Finite, FiniteExt, FitsInUsize, IterAll};
pub use generic_array::{self, typenum};
pub use map::{ExhaustiveMap, IntoIter, IntoValues, Iter, IterMut, Values, ValuesMut};
pub use range::{InRange, InRangeBounds, InRangeInclusive};

extern crate self as exhaustive_map;
