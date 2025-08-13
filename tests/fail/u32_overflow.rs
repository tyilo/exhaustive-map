//@normalize-stderr-test: "/[^ ]*/src/finite.rs:\d+:\d+" -> "../src/finite.rs:...:..."
//@normalize-stderr-test: "'\S*+long-type-\d+.txt" -> "'long-type-n.txt"

use exhaustive_map::Finite;
use exhaustive_map::generic_array::GenericArray;
use exhaustive_map::typenum::Unsigned;

pub const SHOULD_OVERFLOW: usize = {
    type LEN = <u32 as Finite>::INHABITANTS;
    const _: () = {
        if LEN::USIZE as u32 != 0 {
            panic!();
        }
    };

    <GenericArray<bool, LEN> as Finite>::INHABITANTS::USIZE
};
