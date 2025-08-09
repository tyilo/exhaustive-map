//@normalize-stderr-test: "/[^ ]*/src/finite.rs" -> "../src/finite.rs"

use exhaustive_map::Finite;

pub const SHOULD_OVERFLOW: usize = {
    const LEN: usize = u32::INHABITANTS;
    const _: () = {
        if LEN as u32 != 0 {
            panic!();
        }
    };

    <[bool; LEN]>::INHABITANTS
};
