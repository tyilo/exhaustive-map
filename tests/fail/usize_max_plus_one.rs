//@normalize-stderr-test: "'\S*+long-type-\d+.txt" -> "'long-type-n.txt"

use exhaustive_map::Finite;
use std::num::NonZeroUsize;

#[derive(Finite)]
enum TooBig {
    A(NonZeroUsize),
    B,
}
