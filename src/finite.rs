use std::num::{NonZeroI16, NonZeroI32, NonZeroI8, NonZeroU16, NonZeroU32, NonZeroU8};

pub use exhaustive_map_macros::Finite;
use exhaustive_map_macros::{__impl_enum, __impl_tuples};

/// Represents a type that has a finite number of inhabitants.
///
/// If the number of inhabitants is more than `usize::MAX`,
/// such as `usize`, `isize`, `u64`, `i64` and `f64`,
/// then `Finite` should not be implemented for the type.
///
/// Example:
/// ```
/// use exhaustive_map::{Finite, FiniteExt};
///
/// #[derive(Finite, Debug, PartialEq)]
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
///
/// assert_eq!(Color::INHABITANTS, 3);
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
    const INHABITANTS: usize;

    /// Should return a number in the range `0..INHABITANTS`.
    fn to_usize(&self) -> usize;

    /// Should be the inverse function of `to_usize`.
    ///
    /// This should return `Some` if and only if `i < T::INHABITANTS`.
    fn from_usize(i: usize) -> Option<Self>;
}

/// An extension for [`Finite`] providing the [`iter_all`](FiniteExt::iter_all) method.
pub trait FiniteExt: Finite {
    /// An iterator over all inhabitants of the type, ordered by the order provided by [`Finite`].
    fn iter_all() -> IterAll<Self> {
        IterAll((0..Self::INHABITANTS).map(|i| {
            Self::from_usize(i).expect("unexpected None returned from Finite::from_usize in range")
        }))
    }
}

impl<T: Finite> FiniteExt for T {}

/// An owned iterator over all inhabitants of a type implementing [`Finite`].
///
/// This `struct` is created by the [`FiniteExt::iter_all`] method.
pub struct IterAll<T>(std::iter::Map<std::ops::Range<usize>, fn(usize) -> T>);

impl<T> Iterator for IterAll<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl Finite for std::convert::Infallible {
    const INHABITANTS: usize = 0;

    fn to_usize(&self) -> usize {
        match *self {}
    }

    fn from_usize(_: usize) -> Option<Self> {
        None
    }
}

macro_rules! impl_singleton {
    ($type:path) => {
        impl_singleton!($type, $type);
    };
    ($type:tt, $value:expr) => {
        impl Finite for $type {
            const INHABITANTS: usize = 1;

            fn to_usize(&self) -> usize {
                0
            }

            fn from_usize(i: usize) -> Option<Self> {
                match i {
                    0 => Some($value),
                    _ => None,
                }
            }
        }
    };
}

impl_singleton!((), ());
impl_singleton!(std::alloc::System);
impl_singleton!(std::marker::PhantomPinned);

impl<T: ?Sized> Finite for std::marker::PhantomData<T> {
    const INHABITANTS: usize = 1;

    fn to_usize(&self) -> usize {
        0
    }

    fn from_usize(i: usize) -> Option<Self> {
        match i {
            0 => Some(Self),
            _ => None,
        }
    }
}

impl Finite for bool {
    const INHABITANTS: usize = 2;

    fn to_usize(&self) -> usize {
        *self as usize
    }

    fn from_usize(i: usize) -> Option<Self> {
        match i {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}

macro_rules! impl_uprim {
    ($type:path) => {
        impl Finite for $type {
            const INHABITANTS: usize = <$type>::MAX as usize + 1;

            fn to_usize(&self) -> usize {
                *self as usize
            }

            fn from_usize(i: usize) -> Option<Self> {
                i.try_into().ok()
            }
        }
    };
}

impl_uprim!(u8);
impl_uprim!(u16);
impl_uprim!(u32);

macro_rules! impl_iprim {
    ($itype:path, $utype:path) => {
        impl Finite for $itype {
            const INHABITANTS: usize = <$utype as Finite>::INHABITANTS;

            fn to_usize(&self) -> usize {
                (*self as $utype).to_usize()
            }

            fn from_usize(i: usize) -> Option<Self> {
                <$utype as Finite>::from_usize(i).map(|v| v as Self)
            }
        }
    };
}

impl_iprim!(i8, u8);
impl_iprim!(i16, u16);
impl_iprim!(i32, u32);

macro_rules! impl_unonzero {
    ($type:path) => {
        impl Finite for $type {
            const INHABITANTS: usize = 2usize.pow(std::mem::size_of::<$type>() as u32 * 8) - 1;

            fn to_usize(&self) -> usize {
                self.get() as usize - 1
            }

            fn from_usize(i: usize) -> Option<Self> {
                <$type>::new((i + 1).try_into().ok()?)
            }
        }
    };
}

impl_unonzero!(NonZeroU8);
impl_unonzero!(NonZeroU16);
impl_unonzero!(NonZeroU32);

macro_rules! impl_inonzero {
    ($nonzero_type:path, $itype:path) => {
        impl Finite for $nonzero_type {
            const INHABITANTS: usize = <$itype as Finite>::INHABITANTS - 1;

            fn to_usize(&self) -> usize {
                (self.get() as $itype).to_usize() - 1
            }

            fn from_usize(i: usize) -> Option<Self> {
                <$nonzero_type>::new(<$itype>::from_usize(i + 1)?)
            }
        }
    };
}

impl_inonzero!(NonZeroI8, i8);
impl_inonzero!(NonZeroI16, i16);
impl_inonzero!(NonZeroI32, i32);

const CHAR_GAP_START: usize = 0xD800;
const CHAR_GAP_END: usize = 0xDFFF;
const CHAR_GAP_SIZE: usize = CHAR_GAP_END - CHAR_GAP_START + 1;
impl Finite for char {
    const INHABITANTS: usize = char::MAX as usize + 1 - CHAR_GAP_SIZE;

    fn to_usize(&self) -> usize {
        let mut v = *self as usize;
        if v > CHAR_GAP_END {
            v -= CHAR_GAP_SIZE;
        }
        v
    }

    fn from_usize(mut i: usize) -> Option<Self> {
        if i >= CHAR_GAP_START {
            i += CHAR_GAP_SIZE;
        }
        char::from_u32(i.try_into().ok()?)
    }
}

impl Finite for f32 {
    const INHABITANTS: usize = u32::INHABITANTS;

    fn to_usize(&self) -> usize {
        self.to_bits().to_usize()
    }

    fn from_usize(i: usize) -> Option<Self> {
        u32::from_usize(i).map(Self::from_bits)
    }
}

__impl_enum!(std::cmp::Ordering, [Less, Equal, Greater]);
__impl_enum!(std::net::Shutdown, [Read, Write, Both]);
__impl_enum!(
    std::num::FpCategory,
    [Nan, Infinite, Zero, Subnormal, Normal]
);
__impl_enum!(std::sync::mpsc::RecvTimeoutError, [Timeout, Disconnected]);
__impl_enum!(std::sync::mpsc::TryRecvError, [Empty, Disconnected]);
__impl_enum!(std::fmt::Alignment, [Left, Right, Center]);

impl<T: Finite> Finite for Option<T> {
    const INHABITANTS: usize = T::INHABITANTS + 1;

    fn to_usize(&self) -> usize {
        match self {
            Some(v) => v.to_usize(),
            None => T::INHABITANTS,
        }
    }

    fn from_usize(i: usize) -> Option<Self> {
        match i.cmp(&T::INHABITANTS) {
            std::cmp::Ordering::Less => Some(Some(T::from_usize(i)?)),
            std::cmp::Ordering::Equal => Some(None),
            std::cmp::Ordering::Greater => None,
        }
    }
}

macro_rules! impl_from {
    ($type:path, $from:path) => {
        impl Finite for $type {
            const INHABITANTS: usize = <$from as Finite>::INHABITANTS - 1;

            fn to_usize(&self) -> usize {
                <$from>::from(*self).to_usize()
            }

            fn from_usize(i: usize) -> Option<Self> {
                Self::try_from(<$from>::from_usize(i)?).ok()
            }
        }
    };
}

impl_from!(std::net::Ipv4Addr, u32);

impl<const N: usize, T: Finite> Finite for [T; N] {
    const INHABITANTS: usize = T::INHABITANTS.pow(N as u32);

    fn to_usize(&self) -> usize {
        let mut res = 0;
        for v in self.iter().rev() {
            res *= T::INHABITANTS;
            res += v.to_usize();
        }
        res
    }

    fn from_usize(mut i: usize) -> Option<Self> {
        if i >= Self::INHABITANTS {
            None
        } else {
            let arr = std::array::from_fn(|_| {
                let v = T::from_usize(i % T::INHABITANTS).unwrap();
                i /= T::INHABITANTS;
                v
            });
            Some(arr)
        }
    }
}

__impl_tuples!(16);

#[cfg(test)]
mod test {
    use std::{
        fmt::Debug,
        marker::PhantomData,
        num::{NonZeroI16, NonZeroI8, NonZeroU16, NonZeroU8},
    };

    use super::*;

    fn test_all<T: Finite + Debug + PartialEq>(expected_elements: usize) {
        assert_eq!(T::INHABITANTS, expected_elements);

        for i in 0..T::INHABITANTS {
            let v = T::from_usize(i).unwrap();
            let i2 = v.to_usize();
            assert_eq!(i2, i, "{i}usize -> {v:?} -> {i2}usize");
        }

        for k in [8, 16, 32, 64] {
            for k in [k - 1, k, k + 1] {
                let Some(n) = 2usize.checked_pow(k) else {
                    continue;
                };
                for i in [n - 1, n, n + 1] {
                    if i >= T::INHABITANTS {
                        assert_eq!(
                            T::from_usize(i),
                            None,
                            "expected None from T::from_usize({i})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_infallible() {
        test_all::<std::convert::Infallible>(0);
    }

    #[test]
    fn test_unit() {
        test_all::<()>(1);
    }

    #[test]
    fn test_bool() {
        test_all::<bool>(2);
    }

    #[test]
    fn test_u8() {
        test_all::<u8>(256);
    }

    #[test]
    fn test_u16() {
        test_all::<u16>(256 * 256);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_u32() {
        test_all::<u32>(256 * 256 * 256 * 256);
    }

    #[test]
    fn test_i8() {
        test_all::<i8>(256);
    }

    #[test]
    fn test_i16() {
        test_all::<i16>(256 * 256);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_i32() {
        test_all::<i32>(256 * 256 * 256 * 256);
    }

    #[test]
    fn test_nonzero_u8() {
        test_all::<NonZeroU8>(256 - 1);
    }

    #[test]
    fn test_nonzero_u16() {
        test_all::<NonZeroU16>(256 * 256 - 1);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_nonzero_u32() {
        test_all::<NonZeroU32>(256 * 256 * 256 * 256 - 1);
    }

    #[test]
    fn test_nonzero_i8() {
        test_all::<NonZeroI8>(256 - 1);
    }

    #[test]
    fn test_nonzero_i16() {
        test_all::<NonZeroI16>(256 * 256 - 1);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_nonzero_i32() {
        test_all::<NonZeroI32>(256 * 256 * 256 * 256 - 1);
    }

    #[test]
    fn test_char() {
        test_all::<char>(0x110000 - CHAR_GAP_SIZE);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_f32() {
        test_all::<f32>(256usize.pow(4));
    }

    #[test]
    fn test_u8_arr_0() {
        test_all::<[u8; 0]>(1);
    }

    #[test]
    fn test_u8_arr_1() {
        test_all::<[u8; 1]>(256);
    }

    #[test]
    fn test_u8_arr_2() {
        test_all::<[u8; 2]>(256 * 256);
    }

    #[test]
    fn test_unit_arr() {
        test_all::<[(); 100]>(1);
    }

    #[test]
    fn test_tuple_u8_bool() {
        test_all::<(u8, bool)>(512);
    }

    #[test]
    fn test_tuple_bool_u8() {
        test_all::<(bool, u8)>(512);
    }

    #[test]
    fn test_tuple_and_arr_same_encoding() {
        let i1 = [1u8, 2u8].to_usize();
        let i2 = (1u8, 2u8).to_usize();
        assert_eq!(i1, i2);
    }

    #[test]
    fn test_derive_unit_struct() {
        #[derive(Finite, Debug, PartialEq)]
        struct UnitStruct;
        test_all::<UnitStruct>(1);
    }

    #[test]
    fn test_derive_empty_tuple_struct() {
        #[derive(Finite, Debug, PartialEq)]
        struct EmptyTupleStruct();
        test_all::<EmptyTupleStruct>(1);
    }

    #[test]
    fn test_derive_tuple_struct() {
        #[allow(dead_code)]
        #[derive(Finite, Debug, PartialEq)]
        struct TupleStruct(u8, bool);
        test_all::<TupleStruct>(256 * 2);
    }

    #[test]
    fn test_derive_empty_named_struct() {
        #[derive(Finite, Debug, PartialEq)]
        struct EmptyNamedStruct {}
        test_all::<EmptyNamedStruct>(1);
    }

    #[test]
    fn test_derive_named_struct() {
        #[derive(Finite, Debug, PartialEq)]
        struct Struct {
            _a: bool,
            _b: u8,
            _c: Option<bool>,
        }
        test_all::<Struct>(2 * 256 * 3);
    }

    #[test]
    fn test_derive_empty_enum() {
        #[derive(Finite, Debug, PartialEq)]
        enum EmptyEnum {}
        test_all::<EmptyEnum>(0);
    }

    #[test]
    fn test_derive_simple_enum() {
        #[derive(Finite, Debug, PartialEq)]
        enum SimpleEnum {
            _A,
            _B,
            _C,
        }
        test_all::<SimpleEnum>(3);
    }

    #[test]
    fn test_tuple_enum() {
        #[derive(Finite, Debug, PartialEq)]
        enum TupleEnum {
            _A(u8, bool),
            _B(()),
            _C(),
        }
        test_all::<TupleEnum>(256 * 2 + 1 + 1);
    }

    #[test]
    fn test_derive_struct_enum() {
        #[derive(Finite, Debug, PartialEq)]
        enum StructEnum {
            _A { _a: u8, _b: bool },
            _B { _c: () },
            _C {},
        }
        test_all::<StructEnum>(256 * 2 + 1 + 1);
    }

    #[test]
    fn test_derive_mixed_enum() {
        #[derive(Finite, Debug, PartialEq)]
        enum MixedEnum {
            _A,
            _B(u8),
            _C { _a: Option<bool>, _b: u8 },
        }
        test_all::<MixedEnum>(1 + 256 + 3 * 256);
    }

    #[test]
    fn test_derive_generic() {
        #[derive(Finite, Debug, PartialEq)]
        struct Generic<T> {
            _a: Option<T>,
        }
        test_all::<Generic<u8>>(257);
    }

    #[test]
    fn test_derive_generic_lifetime() {
        #[derive(Finite, Debug, PartialEq)]
        struct Lifetime<'a> {
            _a: PhantomData<&'a ()>,
        }
        test_all::<Lifetime>(1);
    }
}
