use std::u8;

use bit_vec::{EffVec, EfficientSize};

fn main() {

    let mut vec = bit_vec::EffVec::new();

    vec.push(true);
    vec.push(true);
    vec.push(true);
    vec.push(true);

    println!("{}", vec.get(0));
    println!("{}", vec.get(1));
    println!("{}", vec.get(2));
    println!("{}", vec.get(3));
}

enum Pento {
    One,        // 0b00000000
    Two,        // 0b00000001
    Three,      // 0b00000010
    Four,       // 0b00000011
    Five        // 0b00000100
}

unsafe impl EfficientSize for Pento {
    const EFF_SIZE_BITS: usize = 3;
    type Repr = u8;
}

fn push_works_eff_size_3() {

    use Pento::*;

    let mut vec = EffVec::<Pento>::new();

    vec.push(Two);
    vec.push(Two);

    assert_eq!(vec.vec, &[0b_00_001_001])
}