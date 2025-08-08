use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

use crate::{
    finite::{Finite, FiniteExt},
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
#[repr(transparent)]
pub struct ExhaustiveMap<K: Finite, V> {
    // Replace with [V; { K::INHABITANTS }] when Rust supports it
    array: Box<[V]>,
    _phantom: PhantomData<K>,
}

impl<K: Finite, V> ExhaustiveMap<K, V> {
    /// Creates a map by providing a mapping function from `K` to `V`.
    ///
    /// Similar to [`array::from_fn`](std::array::from_fn).
    #[must_use]
    pub fn from_fn(f: impl FnMut(K) -> V) -> Self {
        Self {
            array: K::iter_all().map(f).collect(),
            _phantom: PhantomData,
        }
    }

    /// Tries to create a map by providing a mapping function from `K` to `Result<V, E>`.
    ///
    /// # Errors
    ///
    /// Returns the first error if any of the mappings fails.
    pub fn try_from_fn<E>(f: impl FnMut(K) -> Result<V, E>) -> Result<Self, E> {
        Ok(Self {
            array: K::iter_all().map(f).collect::<Result<_, E>>()?,
            _phantom: PhantomData,
        })
    }

    /// Creates a map by providing a mapping function from `usize` to `V`.
    /// The map is filled according to the [`Finite`] implementation of `K`.
    ///
    /// ```
    /// use exhaustive_map::{ExhaustiveMap, Finite};
    ///
    /// #[derive(Finite, Debug)]
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     Blue,
    /// }
    ///
    /// let map = ExhaustiveMap::from_usize_fn(|i| i);
    /// assert_eq!(map[Color::Red], 0);
    /// assert_eq!(map[Color::Green], 1);
    /// assert_eq!(map[Color::Blue], 2);
    /// ```
    #[must_use]
    pub fn from_usize_fn(f: impl FnMut(usize) -> V) -> Self {
        Self {
            array: (0..K::INHABITANTS).map(f).collect(),
            _phantom: PhantomData,
        }
    }

    /// Returns the number of elements in the map.
    ///
    /// Always equal to `K::INHABITANTS`.
    #[must_use]
    pub const fn len(&self) -> usize {
        K::INHABITANTS
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// The map can only be empty if `K::INHABITANTS` is zero,
    /// meaning the type `K` is uninhabitable.
    #[must_use]
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
            .swap(k1.borrow().to_usize(), k2.borrow().to_usize());
    }

    /// Replace the value stored for `k` with the default value of `V`, returning the previous stored value.
    pub fn take<Q: Borrow<K>>(&mut self, k: Q) -> V
    where
        V: Default,
    {
        std::mem::take(&mut self[k])
    }

    /// Change the values of the stored values via a mapping function.
    ///
    /// ```
    /// use exhaustive_map::ExhaustiveMap;
    ///
    /// let bool_to_int = ExhaustiveMap::from_fn(|k| if k { 1 } else { 0 });
    /// let bool_to_int_string = bool_to_int.map_values(|v| v.to_string());
    ///
    /// assert_eq!(bool_to_int_string[false], "0");
    /// assert_eq!(bool_to_int_string[true], "1");
    /// ```
    #[must_use]
    pub fn map_values<U>(self, f: impl FnMut(V) -> U) -> ExhaustiveMap<K, U> {
        ExhaustiveMap {
            array: self.into_values().map(f).collect(),
            _phantom: PhantomData,
        }
    }

    /// An iterator visiting all keys in the order provided by [`Finite`].
    ///
    /// This creates new keys by calling [`K::from_usize`](Finite::from_usize) for each key.
    pub fn keys() -> IterAll<K> {
        K::iter_all()
    }

    /// An iterator visiting all values stored in the map, ordered by the keys order provided by [`Finite`].
    pub fn values(&self) -> Values<'_, V> {
        Values(self.array.iter())
    }

    /// A mutable iterator visiting all values stored in the map, ordered by the keys order provided by [`Finite`].
    pub fn values_mut(&mut self) -> ValuesMut<'_, V> {
        ValuesMut(self.array.iter_mut())
    }

    /// Creates a consuming iterator visiting all the values, ordered by the keys order provided by [`Finite`].
    /// The map cannot be used after calling this.
    pub fn into_values(self) -> IntoValues<V> {
        IntoValues(self.array.into_vec().into_iter())
    }

    /// An iterator visiting all entries stored in the map, ordered by the keys order provided by [`Finite`].
    ///
    /// This creates new keys by calling [`K::from_usize`](Finite::from_usize) for each key.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter(Self::keys().zip(self.values()))
    }

    /// A mutable iterator visiting all entries stored in the map, ordered by the keys order provided by [`Finite`].
    ///
    /// This creates new keys by calling [`K::from_usize`](Finite::from_usize) for each key.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut(Self::keys().zip(self.values_mut()))
    }

    /// Creates a map with [`MaybeUninit`] values.
    ///
    /// After every value have been initialized [`assume_init`](ExhaustiveMap::assume_init) can be
    /// called to obtain a map with values of type `V`.
    #[must_use]
    pub fn new_uninit() -> ExhaustiveMap<K, MaybeUninit<V>> {
        ExhaustiveMap::from_usize_fn(|_| MaybeUninit::uninit())
    }
}

impl<K: Finite, V> ExhaustiveMap<K, Option<V>> {
    /// Tries to convert an `ExhaustiveMap<K, Option<V>>` to an `ExhaustiveMap<K, V>`.
    ///
    /// # Errors
    ///
    /// If any of the values are `None`, this returns `Err` containing the input map.
    pub fn try_unwrap_values(self) -> Result<ExhaustiveMap<K, V>, ExhaustiveMap<K, Option<V>>> {
        if !self.array.iter().all(Option::is_some) {
            return Err(self);
        }
        #[allow(clippy::missing_panics_doc)]
        let values: Box<[V]> = self
            .array
            .into_vec()
            .into_iter()
            .map(|v| v.unwrap())
            .collect();
        // SAFETY: `values` has the correct length as we used `map`.
        Ok(unsafe { values.try_into().unwrap_unchecked() })
    }
}

impl<K: Finite, V> ExhaustiveMap<K, MaybeUninit<V>> {
    /// # Safety
    ///
    /// All elements must have been initialized.
    #[must_use]
    pub unsafe fn assume_init(self) -> ExhaustiveMap<K, V> {
        ExhaustiveMap {
            array: std::mem::transmute::<Box<[MaybeUninit<V>]>, Box<[V]>>(self.array),
            _phantom: PhantomData,
        }
    }
}

impl<K: Finite, V> TryFrom<Box<[V]>> for ExhaustiveMap<K, V> {
    type Error = Box<[V]>;

    fn try_from(value: Box<[V]>) -> Result<Self, Self::Error> {
        if value.len() != K::INHABITANTS {
            return Err(value);
        }
        Ok(Self {
            array: value,
            _phantom: PhantomData,
        })
    }
}

impl<K: Finite, V> From<ExhaustiveMap<K, V>> for Box<[V]> {
    fn from(value: ExhaustiveMap<K, V>) -> Self {
        value.array
    }
}

impl<K: Finite, V> TryFrom<Vec<V>> for ExhaustiveMap<K, V> {
    type Error = Vec<V>;

    fn try_from(value: Vec<V>) -> Result<Self, Self::Error> {
        if value.len() != K::INHABITANTS {
            return Err(value);
        }
        Ok(Self {
            array: value.into(),
            _phantom: PhantomData,
        })
    }
}

impl<const N: usize, K: Finite, V> TryFrom<[V; N]> for ExhaustiveMap<K, V> {
    type Error = [V; N];

    fn try_from(value: [V; N]) -> Result<Self, Self::Error> {
        if N != K::INHABITANTS {
            return Err(value);
        }
        Ok(Self {
            array: value.into(),
            _phantom: PhantomData,
        })
    }
}

impl<K: Finite + Eq + Hash, V> TryFrom<HashMap<K, V>> for ExhaustiveMap<K, V> {
    type Error = K;

    fn try_from(mut value: HashMap<K, V>) -> Result<Self, Self::Error> {
        Self::try_from_fn(|k| value.remove(&k).ok_or(k))
    }
}

impl<K: Finite + Eq + Hash, V, S: BuildHasher + Default> From<ExhaustiveMap<K, V>>
    for HashMap<K, V, S>
{
    fn from(value: ExhaustiveMap<K, V>) -> Self {
        Self::from_iter(value)
    }
}

impl<K: Finite + Ord, V> From<ExhaustiveMap<K, V>> for BTreeMap<K, V> {
    fn from(value: ExhaustiveMap<K, V>) -> Self {
        Self::from_iter(value)
    }
}

/// An iterator over the values of an [`ExhaustiveMap`].
///
/// This `struct` is created by the [`ExhaustiveMap::values`] method.
#[must_use = "iterators are lazy and do nothing unless consumed"]
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
#[must_use = "iterators are lazy and do nothing unless consumed"]
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
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IntoValues<V>(std::vec::IntoIter<V>);

impl<V> Iterator for IntoValues<V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<K: Finite, V: Default> Default for ExhaustiveMap<K, V> {
    fn default() -> Self {
        Self::from_fn(|_| V::default())
    }
}

/// An iterator over the entries of an [`ExhaustiveMap`].
///
/// This `struct` is created by the [`ExhaustiveMap::iter`] method.
#[must_use = "iterators are lazy and do nothing unless consumed"]
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
#[must_use = "iterators are lazy and do nothing unless consumed"]
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
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IntoIter<K: Finite, V>(std::iter::Zip<IterAll<K>, IntoValues<V>>);

impl<K: Finite, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<K: Finite, V> IntoIterator for ExhaustiveMap<K, V> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(Self::keys().zip(self.into_values()))
    }
}

impl<'a, K: Finite, V> IntoIterator for &'a ExhaustiveMap<K, V> {
    type Item = (K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: Finite, V> IntoIterator for &'a mut ExhaustiveMap<K, V> {
    type Item = (K, &'a mut V);

    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K: Finite + Debug, V: Debug> Debug for ExhaustiveMap<K, V> {
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

// The following traits could have been implemented using a derive macro,
// however that would put an unnecessary trait bound on the key.

impl<K: Finite, V: Clone> Clone for ExhaustiveMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            array: self.array.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<K: Finite, V: PartialEq> PartialEq for ExhaustiveMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.array.eq(&other.array)
    }
}

impl<K: Finite, V: Eq> Eq for ExhaustiveMap<K, V> {}

impl<K: Finite, V: PartialOrd> PartialOrd for ExhaustiveMap<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.array.partial_cmp(&other.array)
    }
}

impl<K: Finite, V: Ord> Ord for ExhaustiveMap<K, V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.array.cmp(&other.array)
    }
}

impl<K: Finite, V: Hash> Hash for ExhaustiveMap<K, V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.array.hash(state);
    }
}

// SAFETY: `ExhaustiveMap<K, V>` is just a transparent wrapper around `Box<[V]>`.
unsafe impl<K: Finite, V> Send for ExhaustiveMap<K, V> where Box<[V]>: Send {}
// SAFETY: `ExhaustiveMap<K, V>` is just a transparent wrapper around `Box<[V]>`.
unsafe impl<K: Finite, V> Sync for ExhaustiveMap<K, V> where Box<[V]>: Sync {}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Finite)]
    struct Key(PhantomData<*mut u8>);

    #[allow(unused)]
    const fn assert_implements_traits<
        T: Send + Sync + Default + Clone + PartialEq + Eq + PartialOrd + Ord + Hash,
    >() {
    }

    const _: () = assert_implements_traits::<ExhaustiveMap<Key, bool>>();

    #[test]
    fn test_uninit() {
        let mut m = ExhaustiveMap::<bool, u8>::new_uninit();
        m[true].write(123);
        m[false].write(45);
        // SAFETY: All elements has been initialized.
        let m = unsafe { m.assume_init() };
        println!("{m:?}");
    }

    #[test]
    fn test_conversion() {
        let m: ExhaustiveMap<bool, u8> = [2, 3].try_into().unwrap();
        assert_eq!(m[false], 2);
        assert_eq!(m[true], 3);
    }

    #[test]
    fn test_try_unrwap_values() {
        let m: ExhaustiveMap<bool, Option<u8>> = ExhaustiveMap::from_fn(|_| None);
        let mut m = m.try_unwrap_values().unwrap_err();
        m[false] = Some(2);
        let mut m = m.try_unwrap_values().unwrap_err();
        m[true] = Some(3);
        let m = m.try_unwrap_values().unwrap();
        let expected: ExhaustiveMap<bool, u8> = [2, 3].try_into().unwrap();
        assert_eq!(m, expected);
    }
}
