use std::{
    borrow::Borrow,
    fmt::Debug,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

use crate::{
    finite::{Finite, FromUsize, FromUsizeExt},
    IterAll,
};

/// A map which is guaranteed to always contain a value for each possible key of type `K`.
/// ```
/// use exhaustive_map::ExhaustiveMap;
///
/// let mut map = ExhaustiveMap::<u8, u16>::from_fn(|i| i as u16 + 100);
/// assert_eq!(map.len(), 256);
///
/// assert_eq!(map[3], 103);
///
/// map[7] = 9999;
/// assert_eq!(map[7], 9999);
///
/// map.swap(7, 3);
/// assert_eq!(map[3], 9999);
/// assert_eq!(map[7], 103);
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExhaustiveMap<K: Finite, V> {
    // Replace with [V; { K::INHABITANTS }] when Rust supports it
    array: Box<[V]>,
    _phantom: PhantomData<K>,
}

impl<K: FromUsize, V> ExhaustiveMap<K, V> {
    /// Creates a map by providing a mapping function from `K` to `V`.
    ///
    /// Similar to [`array::from_fn`](std::array::from_fn).
    pub fn from_fn(f: impl FnMut(K) -> V) -> Self {
        Self {
            array: K::iter_all().map(f).collect(),
            _phantom: PhantomData,
        }
    }

    /// An iterator visiting all keys in the order provided by [`Finite`].
    ///
    /// This creates new keys by calling [`K::from_usize`](Finite::from_usize) for each key.
    pub fn keys() -> IterAll<K> {
        K::iter_all()
    }

    /// An iterator visiting all entries stored in the map, ordered by the keys order provided by [`Finite`].
    ///
    /// This creates new keys by calling [`K::from_usize`](Finite::from_usize) for each key.
    pub fn iter(&self) -> Iter<K, V> {
        Iter(Self::keys().zip(self.values()))
    }

    /// A mutable iterator visiting all entries stored in the map, ordered by the keys order provided by [`Finite`].
    ///
    /// This creates new keys by calling [`K::from_usize`](Finite::from_usize) for each key.
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut(Self::keys().zip(self.values_mut()))
    }
}

impl<K: Finite, V> ExhaustiveMap<K, V> {
    pub fn new_filled(mut f: impl FnMut() -> V) -> Self {
        Self {
            array: (0..K::INHABITANTS).map(|_| f()).collect(),
            _phantom: PhantomData,
        }
    }

    /// Returns the number of elements in the map.
    ///
    /// Always equal to `K::INHABITANTS`.
    pub const fn len(&self) -> usize {
        K::INHABITANTS
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// The map can only be empty if `K::INHABITANTS` is zero,
    /// meaning the type `K` is uninhabitable.
    pub const fn is_empty(&self) -> bool {
        K::INHABITANTS == 0
    }

    /// Replace the value stored for `k` with `v`, returning the previous stored value.
    pub fn replace<Q: Borrow<K>>(&mut self, k: Q, v: V) -> V {
        std::mem::replace(&mut self[k], v)
    }

    /// Swaps the values at stored at `k1` and `k2`.
    pub fn swap<Q1: Borrow<K>, Q2: Borrow<K>>(&mut self, k1: Q1, k2: Q2) {
        self.array
            .swap(k1.borrow().to_usize(), k2.borrow().to_usize())
    }

    /// Replace the value stored for `k` with the default value of `V`, returning the previous stored value.
    pub fn take<Q: Borrow<K>>(&mut self, k: Q) -> V
    where
        V: Default,
    {
        std::mem::take(&mut self[k])
    }

    /// An iterator visiting all values stored in the map, ordered by the keys order provided by [`Finite`].
    pub fn values(&self) -> Values<V> {
        Values(self.array.iter())
    }

    /// A mutable iterator visiting all values stored in the map, ordered by the keys order provided by [`Finite`].
    pub fn values_mut(&mut self) -> ValuesMut<V> {
        ValuesMut(self.array.iter_mut())
    }

    /// Creates a consuming iterator visiting all the values, ordered by the keys order provided by [`Finite`].
    /// The map cannot be used after calling this.
    pub fn into_values(self) -> IntoValues<V> {
        IntoValues(self.array.into_vec().into_iter())
    }

    pub fn new_uninit() -> ExhaustiveMap<K, MaybeUninit<V>> {
        ExhaustiveMap::new_filled(|| MaybeUninit::uninit())
    }
}

impl<K: Finite, V> ExhaustiveMap<K, MaybeUninit<V>> {
    /// # Safety
    ///
    /// All elements must have been initialized.
    pub unsafe fn assume_init(self) -> ExhaustiveMap<K, V> {
        ExhaustiveMap {
            array: std::mem::transmute::<_, Box<[V]>>(self.array),
            _phantom: PhantomData,
        }
    }
}

/// An iterator over the values of an [`ExhaustiveMap`].
///
/// This `struct` is created by the [`ExhaustiveMap::values`] method.
pub struct Values<'a, V>(std::slice::Iter<'a, V>);

impl<'a, V> Iterator for Values<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// A mutable iterator over the values of an [`ExhaustiveMap`].
///
/// This `struct` is created by the [`ExhaustiveMap::values_mut`] method.
pub struct ValuesMut<'a, V>(std::slice::IterMut<'a, V>);

impl<'a, V> Iterator for ValuesMut<'a, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// An owning iterator over the values of an [`ExhaustiveMap`].
///
/// This `struct` is created by the [`ExhaustiveMap::into_values`] method.
pub struct IntoValues<V>(std::vec::IntoIter<V>);

impl<V> Iterator for IntoValues<V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<K: Finite, V: Default> Default for ExhaustiveMap<K, V> {
    fn default() -> Self {
        Self::new_filled(|| V::default())
    }
}

/// An iterator over the entries of an [`ExhaustiveMap`].
///
/// This `struct` is created by the [`ExhaustiveMap::iter`] method.
pub struct Iter<'a, K: Finite, V>(std::iter::Zip<IterAll<K>, Values<'a, V>>);

impl<'a, K: Finite, V> Iterator for Iter<'a, K, V> {
    type Item = (K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// A mutable iterator over the entries of an [`ExhaustiveMap`].
///
/// This `struct` is created by the [`ExhaustiveMap::iter_mut`] method.
pub struct IterMut<'a, K: Finite, V>(std::iter::Zip<IterAll<K>, ValuesMut<'a, V>>);

impl<'a, K: Finite, V> Iterator for IterMut<'a, K, V> {
    type Item = (K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// An owning iterator over the entries of an [`ExhaustiveMap`].
///
/// This `struct` is created by the [`into_iter`](IntoIterator::into_iter) method on [`ExhaustiveMap`]
/// (provided by the [`IntoIterator`] trait).
pub struct IntoIter<K: Finite, V>(std::iter::Zip<IterAll<K>, IntoValues<V>>);

impl<K: Finite, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<K: FromUsize, V> IntoIterator for ExhaustiveMap<K, V> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(Self::keys().zip(self.into_values()))
    }
}

impl<'a, K: FromUsize, V> IntoIterator for &'a ExhaustiveMap<K, V> {
    type Item = (K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: FromUsize, V> IntoIterator for &'a mut ExhaustiveMap<K, V> {
    type Item = (K, &'a mut V);

    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K: FromUsize + Debug, V: Debug> Debug for ExhaustiveMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self).finish()
    }
}

impl<K: Finite, V, Q: Borrow<K>> Index<Q> for ExhaustiveMap<K, V> {
    type Output = V;

    fn index(&self, index: Q) -> &Self::Output {
        &self.array[K::to_usize(index.borrow())]
    }
}

impl<K: Finite, V, Q: Borrow<K>> IndexMut<Q> for ExhaustiveMap<K, V> {
    fn index_mut(&mut self, index: Q) -> &mut Self::Output {
        &mut self.array[K::to_usize(index.borrow())]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_uninit() {
        let mut m = ExhaustiveMap::<bool, u8>::new_uninit();
        m[true].write(123);
        m[false].write(45);
        let m = unsafe { m.assume_init() };
        println!("{m:?}");
    }
}
