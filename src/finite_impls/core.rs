#[cfg(target_pointer_width = "64")]
use core::num::{NonZeroI64, NonZeroU64};
use core::{
    marker::PhantomData,
    mem::size_of,
    num::{
        NonZeroI16, NonZeroI32, NonZeroI8, NonZeroIsize, NonZeroU16, NonZeroU32, NonZeroU8,
        NonZeroUsize,
    },
};

use exhaustive_map_macros::{__impl_tuples, uint};
use generic_array::{
    typenum::{generic_const_mappings::U, Const, Pow, Sub1, ToUInt, Unsigned, U1, U2},
    ArrayLength, GenericArray,
};

use crate::{Finite, FitsInUsize};

impl<T: ?Sized> Finite for PhantomData<T> {
    type INHABITANTS = U1;

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
    type INHABITANTS = U2;

    fn to_usize(&self) -> usize {
        usize::from(*self)
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
            type INHABITANTS = U<{ <$type>::MAX as usize + 1 }>;

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
#[cfg(target_pointer_width = "64")]
impl_uprim!(u32);

macro_rules! impl_iprim {
    ($itype:path, $utype:path) => {
        impl Finite for $itype {
            type INHABITANTS = <$utype as Finite>::INHABITANTS;

            fn to_usize(&self) -> usize {
                #[allow(clippy::cast_sign_loss)]
                (*self as $utype).to_usize()
            }

            fn from_usize(i: usize) -> Option<Self> {
                #[allow(clippy::cast_possible_wrap)]
                <$utype as Finite>::from_usize(i).map(|v| v as Self)
            }
        }
    };
}

impl_iprim!(i8, u8);
impl_iprim!(i16, u16);
#[cfg(target_pointer_width = "64")]
impl_iprim!(i32, u32);

macro_rules! impl_unonzero {
    ($type:path) => {
        impl Finite for $type {
            type INHABITANTS = Sub1<<U2 as Pow<U<{ size_of::<$type>() * 8 }>>>::Output>;

            fn to_usize(&self) -> usize {
                usize::try_from(self.get()).unwrap() - 1
            }

            fn from_usize(i: usize) -> Option<Self> {
                <$type>::new((i.checked_add(1)?).try_into().ok()?)
            }
        }
    };
}

impl_unonzero!(NonZeroU8);
impl_unonzero!(NonZeroU16);
impl_unonzero!(NonZeroU32);
#[cfg(target_pointer_width = "64")]
impl_unonzero!(NonZeroU64);
impl_unonzero!(NonZeroUsize);

macro_rules! impl_inonzero {
    ($nonzero_type:path, $itype:path, $utype:path) => {
        impl Finite for $nonzero_type {
            type INHABITANTS = Sub1<<U2 as Pow<U<{ size_of::<$nonzero_type>() * 8 }>>>::Output>;

            #[allow(clippy::cast_sign_loss)]
            fn to_usize(&self) -> usize {
                usize::try_from(self.get() as $utype).unwrap() - 1
            }

            #[allow(clippy::cast_possible_wrap)]
            fn from_usize(i: usize) -> Option<Self> {
                <$nonzero_type>::new(
                    <$utype>::try_from(i.checked_add(1)?)
                        .map(|v| v as $itype)
                        .ok()?,
                )
            }
        }
    };
}

impl_inonzero!(NonZeroI8, i8, u8);
impl_inonzero!(NonZeroI16, i16, u16);
impl_inonzero!(NonZeroI32, i32, u32);
#[cfg(target_pointer_width = "64")]
impl_inonzero!(NonZeroI64, i64, u64);
impl_inonzero!(NonZeroIsize, isize, usize);

const CHAR_GAP_START: usize = 0xD800;
const CHAR_GAP_END: usize = 0xDFFF;
const CHAR_GAP_SIZE: usize = CHAR_GAP_END - CHAR_GAP_START + 1;
impl Finite for char {
    type INHABITANTS = uint!(1112064); //U<{char::MAX as usize + 1 - CHAR_GAP_SIZE}>;

    fn to_usize(&self) -> usize {
        const _: () = {
            assert!(
                <char as Finite>::INHABITANTS::USIZE == char::MAX as usize + 1 - CHAR_GAP_SIZE,
                "Wrong INHABITANTS for char"
            );
        };

        let mut v = *self as usize;
        if v > CHAR_GAP_END {
            v -= CHAR_GAP_SIZE;
        }
        v
    }

    fn from_usize(mut i: usize) -> Option<Self> {
        if i >= CHAR_GAP_START {
            i = i.checked_add(CHAR_GAP_SIZE)?;
        }
        char::from_u32(i.try_into().ok()?)
    }
}

#[cfg(target_pointer_width = "64")]
impl Finite for f32 {
    type INHABITANTS = <u32 as Finite>::INHABITANTS;

    fn to_usize(&self) -> usize {
        self.to_bits().to_usize()
    }

    fn from_usize(i: usize) -> Option<Self> {
        u32::from_usize(i).map(Self::from_bits)
    }
}

#[cfg(target_pointer_width = "64")]
macro_rules! impl_from {
    ($type:path, $from:path) => {
        impl Finite for $type {
            type INHABITANTS = <$from as Finite>::INHABITANTS;

            fn to_usize(&self) -> usize {
                <$from>::from(*self).to_usize()
            }

            fn from_usize(i: usize) -> Option<Self> {
                Self::try_from(<$from>::from_usize(i)?).ok()
            }
        }
    };
}

#[cfg(target_pointer_width = "64")]
impl_from!(core::net::Ipv4Addr, u32);

impl<const N: usize, T: Finite> Finite for [T; N]
where
    Const<N>: ToUInt,
    T::INHABITANTS: Pow<U<N>>,
    <T::INHABITANTS as Pow<U<N>>>::Output: ArrayLength + FitsInUsize,
{
    type INHABITANTS = <T::INHABITANTS as Pow<U<N>>>::Output;

    fn to_usize(&self) -> usize {
        let mut res = 0;
        for v in self.iter().rev() {
            res *= T::INHABITANTS::USIZE;
            res += v.to_usize();
        }
        res
    }

    fn from_usize(mut i: usize) -> Option<Self> {
        if i >= Self::INHABITANTS::USIZE {
            None
        } else {
            let arr = core::array::from_fn(|_| {
                let v = T::from_usize(i % T::INHABITANTS::USIZE).unwrap();
                i /= T::INHABITANTS::USIZE;
                v
            });
            Some(arr)
        }
    }
}

impl<T: Finite, N: ArrayLength> Finite for GenericArray<T, N>
where
    <T as Finite>::INHABITANTS: Pow<N>,
    <T::INHABITANTS as Pow<N>>::Output: ArrayLength + FitsInUsize,
{
    type INHABITANTS = <T::INHABITANTS as Pow<N>>::Output;

    fn to_usize(&self) -> usize {
        let mut res = 0;
        for v in self.iter().rev() {
            res *= T::INHABITANTS::USIZE;
            res += v.to_usize();
        }
        res
    }

    fn from_usize(mut i: usize) -> Option<Self> {
        if i >= Self::INHABITANTS::USIZE {
            None
        } else {
            Some(
                (0..N::USIZE)
                    .map(|_| {
                        let v = T::from_usize(i % T::INHABITANTS::USIZE).unwrap();
                        i /= T::INHABITANTS::USIZE;
                        v
                    })
                    .collect(),
            )
        }
    }
}

__impl_tuples!(16);

#[derive(Finite)]
#[__finite_foreign(core::convert::Infallible)]
enum _Infallible {}

#[derive(Finite)]
#[__finite_foreign(core::marker::PhantomPinned)]
struct _PhantomPinned;

#[derive(Finite)]
#[__finite_foreign(core::cmp::Ordering)]
enum _Ordering {
    Less,
    Equal,
    Greater,
}

#[derive(Finite)]
#[__finite_foreign(core::num::FpCategory)]
enum _FpCategory {
    Nan,
    Infinite,
    Zero,
    Subnormal,
    Normal,
}

#[derive(Finite)]
#[__finite_foreign(core::fmt::Alignment)]
enum _Alignment {
    Left,
    Right,
    Center,
}

#[derive(Finite)]
#[__finite_foreign(Option)]
enum _Option<T> {
    None,
    Some(T),
}

#[derive(Finite)]
#[__finite_foreign(Result)]
enum _Result<T, E> {
    Ok(T),
    Err(E),
}

#[derive(Finite)]
#[__finite_foreign(core::task::Poll)]
enum _Poll<T> {
    Ready(T),
    Pending,
}

#[derive(Finite)]
#[__finite_foreign(core::ops::Bound)]
enum _Bound<T> {
    Included(T),
    Excluded(T),
    Unbounded,
}

#[derive(Finite)]
#[__finite_foreign(core::ops::ControlFlow)]
enum _ControlFlow<B, C> {
    Continue(C),
    Break(B),
}

#[derive(Finite)]
#[__finite_foreign(core::ops::Range)]
struct _Range<Idx> {
    start: Idx,
    end: Idx,
}

#[derive(Finite)]
#[__finite_foreign(core::ops::RangeFrom)]
struct _RangeFrom<Idx> {
    start: Idx,
}

#[derive(Finite)]
#[__finite_foreign(core::ops::RangeTo)]
struct _RangeTo<Idx> {
    end: Idx,
}

#[derive(Finite)]
#[__finite_foreign(core::ops::RangeToInclusive)]
struct _RangeToInclusive<Idx> {
    end: Idx,
}

#[derive(Finite)]
#[__finite_foreign(core::ops::RangeFull)]
struct _RangeFull;

#[cfg(all(test, feature = "std"))]
mod test {
    use super::{
        super::test_utils::{test_all, test_some},
        *,
    };

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
    #[cfg_attr(debug_assertions, ignore = "too slow in debug build")]
    #[cfg(target_pointer_width = "64")]
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
    #[cfg_attr(debug_assertions, ignore = "too slow in debug build")]
    #[cfg(target_pointer_width = "64")]
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
    #[cfg_attr(debug_assertions, ignore = "too slow in debug build")]
    fn test_nonzero_u32() {
        test_all::<NonZeroU32>(usize::try_from(256u64 * 256 * 256 * 256 - 1).unwrap());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn test_nonzero_u64() {
        test_some::<NonZeroU64>(usize::try_from(2u128.pow(64) - 1).unwrap());
    }

    #[test]
    fn test_nonzero_usize() {
        test_some::<NonZeroUsize>(usize::try_from(2u128.pow(isize::BITS) - 1).unwrap());
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
    #[cfg_attr(debug_assertions, ignore = "too slow in debug build")]
    fn test_nonzero_i32() {
        test_all::<NonZeroI32>(usize::try_from(256u64 * 256 * 256 * 256 - 1).unwrap());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn test_nonzero_i64() {
        test_some::<NonZeroI64>(usize::try_from(2u128.pow(64) - 1).unwrap());
    }

    #[test]
    fn test_nonzero_isize() {
        test_some::<NonZeroIsize>(usize::try_from(2u128.pow(isize::BITS) - 1).unwrap());
    }

    #[test]
    fn test_char() {
        test_all::<char>(0x11_0000 - CHAR_GAP_SIZE);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "too slow in debug build")]
    #[cfg(target_pointer_width = "64")]
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
    #[cfg_attr(debug_assertions, ignore = "too slow in debug build")]
    #[cfg(target_pointer_width = "64")]
    fn test_ipv4_address() {
        test_all::<std::net::Ipv4Addr>(256usize.pow(4));
    }

    #[test]
    fn test_std_cmp_ordering() {
        test_all::<std::cmp::Ordering>(3);
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
    fn test_derive_struct_with_non_clone_field() {
        #[derive(Finite, Debug, PartialEq)]
        struct NonCopy(u8);

        #[derive(Finite, Debug, PartialEq)]
        struct Outer {
            inner: NonCopy,
        }

        test_all::<Outer>(256);
    }

    #[test]
    fn test_derive_enum_with_non_clone_field() {
        #[derive(Finite, Debug, PartialEq)]
        struct NonCopy(u8);

        #[derive(Finite, Debug, PartialEq)]
        enum Outer {
            A(NonCopy),
            B { inner: NonCopy },
        }

        test_all::<Outer>(2 * 256);
    }

    #[test]
    fn test_derive_struct_with_names_from_implementation() {
        #[allow(clippy::struct_excessive_bools)]
        #[derive(Finite, Debug, PartialEq)]
        struct Struct {
            v: bool,
            i: bool,
            res: bool,
            r#type: bool,
        }

        test_all::<Struct>(2usize.pow(4));
    }

    #[test]
    fn test_derive_enum_with_names_from_implementation() {
        #[derive(Finite, Debug, PartialEq)]
        enum Enum {
            Variant {
                v: bool,
                i: bool,
                res: bool,
                r#type: bool,
            },
        }

        test_all::<Enum>(2usize.pow(4));
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
    fn test_derive_generic_complex() {
        #[derive(Finite, Debug, PartialEq)]
        struct Generic<const N: usize, A, B, C, D> {
            _a: Option<A>,
            _b: Result<B, C>,
            _c: [D; N],
        }
        test_all::<Generic<10, (), (), (), ()>>(2 * 2);
    }

    #[test]
    fn test_derive_generic_complex_enum() {
        #[derive(Finite, Debug, PartialEq)]
        enum Generic<const N: usize, A, B, C, D> {
            A(Option<A>),
            B { b: B, c: C },
            C,
            D([D; N]),
        }
        test_all::<Generic<10, (), (), (), ()>>(2 + 1 + 1 + 1);
    }

    #[test]
    fn test_derive_generic_lifetime() {
        #[derive(Finite, Debug, PartialEq)]
        struct Lifetime<'a> {
            _a: PhantomData<&'a ()>,
        }
        test_all::<Lifetime>(1);
    }

    #[test]
    fn test_derive_max_inhabitants() {
        #[derive(Finite, Debug, PartialEq)]
        struct Big {
            a: NonZeroUsize,
        }
        test_some::<Big>(usize::MAX);
    }
}
