# exhaustive-map [![Latest Version]][crates.io] [![API Docs]][docs.rs]

[Latest Version]: https://img.shields.io/crates/v/exhaustive-map.svg
[crates.io]: https://crates.io/crates/exhaustive-map
[API Docs]: https://img.shields.io/docsrs/exhaustive-map.svg
[docs.rs]: https://docs.rs/exhaustive-map

An exhaustive map for types with finite inhabitants.

Example usage:
```rust
use exhaustive_map::ExhaustiveMap;

let mut map = ExhaustiveMap::<u8, u16>::from_fn(|i| i as u16 + 100);
assert_eq!(map.len(), 256);

assert_eq!(map[3], 103);

map[7] = 9999;
assert_eq!(map[7], 9999);

map.swap(7, 3);
assert_eq!(map[3], 9999);
assert_eq!(map[7], 103);
```

The key type must implement the `Finite` trait.
You can implement this for your own types using derive:
```rust
use exhaustive_map::{Finite, FiniteExt};

#[derive(Finite, Debug, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

let all: Vec<_> = Color::iter_all().collect();
assert_eq!(all, vec![Color::Red, Color::Green, Color::Blue]);
```

The `Finite` trait can also be implemented manually:
or manually:
```rust
use exhaustive_map::Finite;

#[derive(Debug, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

impl Finite for Color {
    const INHABITANTS: usize = 3;

    fn to_usize(&self) -> usize {
        match self {
            Self::Red => 0,
            Self::Green => 1,
            Self::Blue => 2,
        }
    }

    fn from_usize(i: usize) -> Option<Self> {
        Some(match i {
            0 => Self::Red,
            1 => Self::Green,
            2 => Self::Blue,
            _ => return None,
        })
    }
}
```
