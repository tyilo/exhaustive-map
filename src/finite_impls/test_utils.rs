use std::{fmt::Debug, prelude::rust_2021::*};

use crate::{typenum::Unsigned, Finite};

pub(crate) fn test_some<T: Finite + Debug + PartialEq>(expected_elements: usize) {
    assert_eq!(T::INHABITANTS::USIZE, expected_elements);

    let mut values: Vec<_> = (0..1000).collect();
    for base in [usize::MAX, T::INHABITANTS::USIZE] {
        for offset in -10..=10 {
            if let Some(i) = base.checked_add_signed(offset) {
                values.push(i);
            }
        }
    }

    for i in values {
        match T::from_usize(i) {
            None => {
                assert!(
                    i >= T::INHABITANTS::USIZE,
                    "Got {i}usize -> None, but INHABITANTS={}",
                    T::INHABITANTS::USIZE
                );
            }
            Some(v) => {
                assert!(
                    i < T::INHABITANTS::USIZE,
                    "Got {i}usize -> {v:?}, but INHABITANTS={}",
                    T::INHABITANTS::USIZE
                );
                let i2 = v.to_usize();
                assert_eq!(i2, i, "{i}usize -> {v:?} -> {i2}usize");
            }
        }
    }
}

pub(crate) fn test_all<T: Finite + Debug + PartialEq>(expected_elements: usize) {
    test_some::<T>(expected_elements);

    for i in 0..T::INHABITANTS::USIZE {
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
                if i >= T::INHABITANTS::USIZE {
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
