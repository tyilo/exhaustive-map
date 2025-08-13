use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    num::Wrapping,
    ops::Index,
};

use divan::{black_box, counter::ItemsCount, Bencher};
use exhaustive_map::{typenum::Unsigned, ExhaustiveMap, Finite, FiniteExt};

fn main() {
    divan::main();
}

trait Map<K: Finite, V>: From<ExhaustiveMap<K, V>> + for<'a> Index<&'a K, Output = V> {
    fn values<'a>(&'a self) -> impl Iterator<Item = &'a V>
    where
        V: 'a;
}

impl<K: Finite, V> Map<K, V> for ExhaustiveMap<K, V> {
    fn values<'a>(&'a self) -> impl Iterator<Item = &'a V>
    where
        V: 'a,
    {
        self.values()
    }
}

impl<K: Finite + Ord, V> Map<K, V> for BTreeMap<K, V> {
    fn values<'a>(&'a self) -> impl Iterator<Item = &'a V>
    where
        V: 'a,
    {
        self.values()
    }
}

impl<K: Finite + Eq + Hash, V> Map<K, V> for HashMap<K, V> {
    fn values<'a>(&'a self) -> impl Iterator<Item = &'a V>
    where
        V: 'a,
    {
        self.values()
    }
}

macro_rules! bench {
    ($name:ident, $key:ty, $value:ty, $from_fn:expr) => {
        mod $name {
            use super::*;

            #[divan::bench(
                                                        types = [
                                                            ExhaustiveMap<$key, $value>,
                                                            BTreeMap<$key, $value>,
                                                            HashMap<$key, $value>,
                                                        ],
                                                        counters = [
                                                            ItemsCount::new(<$key as Finite>::INHABITANTS::USIZE),
                                                        ],
                                                    )]
            fn sum_index<T: Map<$key, $value>>(bencher: Bencher) {
                let map = ExhaustiveMap::from_fn($from_fn);
                let map = &T::from(map);

                bencher.bench_local(|| {
                    let map = black_box(map);
                    let mut sum = Wrapping(<$value>::default());
                    for k in <$key>::iter_all() {
                        sum += map[&k];
                    }
                    sum
                });
            }

            #[divan::bench(
                                                        types = [
                                                            ExhaustiveMap<$key, $value>,
                                                            BTreeMap<$key, $value>,
                                                            HashMap<$key, $value>,
                                                        ],
                                                        counters = [
                                                            ItemsCount::new(<$key as Finite>::INHABITANTS::USIZE),
                                                        ],
                                                    )]
            fn sum_values_iter<T: Map<$key, $value>>(bencher: Bencher) {
                let map = ExhaustiveMap::from_fn($from_fn);
                let map = &T::from(map);

                bencher.bench_local(|| {
                    let map = black_box(map);
                    let mut sum = Wrapping(<$value>::default());
                    for v in map.values() {
                        sum += v;
                    }
                    sum
                });
            }
        }
    };
}

bench!(u8_identity, u8, u8, |i| i);
bench!(u8_to_u64, u8, u64, |i| u64::from(i));
bench!(u16_identity, u16, u16, |i| i);
bench!(u16_to_u64, u16, u64, |i| u64::from(i));

#[derive(Finite, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Color {
    Red,
    Green,
    Blue,
}

bench!(custom_color, Color, usize, |c| match c {
    Color::Red => 123,
    Color::Green => 456,
    Color::Blue => 789,
});
