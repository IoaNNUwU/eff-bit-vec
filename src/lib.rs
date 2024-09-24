#![cfg(target_endian = "little")]

use std::{
    fmt::Debug, marker::PhantomData, ops::{BitAnd, BitOr, Shl, Shr}, u8
};

/// Сколько бит занимает структура в памяти
/// T - числовое представление Self, которое имеет тот же размер
pub unsafe trait EfficientSize: Sized {
    const EFF_SIZE_BITS: usize;

    type Repr: Shr<usize> + Shl<usize> + BitOr + BitAnd + Debug + Copy;
}

// std::Vec<Trio>
// [1000] -> 1000 u8

// bit_vec::Vec<bool>
// [1000] -> 1000 / 4 u8

// Option<Trio> -> 2 bit

// std::Vec<Option<Trio>>
// [1000] -> (1 + 1) * 1000 = 2000

// bit_vec::Vec<Option<Trio>>
// [1000] -> 1 * 1000 / 8 !! 16

unsafe impl EfficientSize for u8 {
    const EFF_SIZE_BITS: usize = size_of::<Self>() * 8;
    type Repr = Self;
}

unsafe impl EfficientSize for u16 {
    const EFF_SIZE_BITS: usize = size_of::<Self>() * 8;
    type Repr = Self;
}

unsafe impl EfficientSize for u32 {
    const EFF_SIZE_BITS: usize = size_of::<Self>() * 8;
    type Repr = Self;
}

unsafe impl EfficientSize for u64 {
    const EFF_SIZE_BITS: usize = size_of::<Self>() * 8;
    type Repr = Self;
}

unsafe impl EfficientSize for bool {
    const EFF_SIZE_BITS: usize = 1;

    type Repr = u8;
}

pub struct EffVec<T: EfficientSize> {
    bits_in_use_in_last_byte: usize,
    pub vec: Vec<u8>,
    _phantom: PhantomData<T>,
}

impl<T: EfficientSize> EffVec<T> {
    pub fn new() -> Self {
        assert!(T::EFF_SIZE_BITS < 8);
        Self {
            bits_in_use_in_last_byte: 0,
            vec: vec![],
            _phantom: PhantomData::default(),
        }
    }

    fn free_bits_in_last_byte(&self) -> usize {
        8 - self.bits_in_use_in_last_byte
    }

    fn bits_in_use(&self) -> usize {
        self.vec.len() * 8 - self.free_bits_in_last_byte()
    }

    fn bytes_in_use(&self) -> usize {
        self.vec.len()
    }

    pub fn push(&mut self, value: T) {

        if self.vec.is_empty() {
            self.vec.push(0);
        }

        let t_repr = unsafe { std::mem::transmute_copy::<T, T::Repr>(&value) };
        let t_size = size_of_val(&t_repr);
        let eff_size = T::EFF_SIZE_BITS;

        // [11111111, 11111111] -> [11111111, 11111111, 00000001]
        if self.free_bits_in_last_byte() < eff_size {
            let len = self.vec.len();

            for _ in 0..t_size {
                // 1
                self.vec.push(0u8);
            }

            let t_ptr = &mut self.vec[len] as *mut u8 as *mut T::Repr;

            unsafe { std::ptr::write(t_ptr, t_repr) };

            self.bits_in_use_in_last_byte = eff_size;
        } else {

            let len = self.vec.len();

            let bits_in_use = self.bits_in_use_in_last_byte;

            let current_last_byte: u8 = self.vec[self.vec.len() - 1];

            let mask = t_repr << bits_in_use;

            let mask = unsafe { std::mem::transmute_copy::<_, u8>(&mask) };

            println!("REPR: {:?}, IN_USE: {:?}, MASK: {:?}", &t_repr, bits_in_use, mask);

            let new_last_byte: u8 = mask | current_last_byte;

            self.vec[len - 1] = new_last_byte;

            self.bits_in_use_in_last_byte += eff_size;
        }
    }

    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        for value in iter {
            self.push(value);
        }
    }

    pub fn remove(&mut self, ind: usize) -> T {
        todo!();
    }

    pub fn len(&self) -> usize {
        self.bits_in_use() / T::EFF_SIZE_BITS
    }

    /// panics if `ind` < `self.len`
    pub fn set(&mut self, ind: usize, value: T) {
        let t_repr = unsafe { std::mem::transmute_copy::<T, T::Repr>(&value) };
        let t_size = size_of_val(&t_repr);
        let eff_size = T::EFF_SIZE_BITS;

        if !(self.bytes_in_use() >= (eff_size * ind).div_ceil(8)) {
            let starting_bit = eff_size * ind;
            let ending_bit = starting_bit + eff_size;

            let starting_byte = starting_bit / 8;
            let ending_byte = ending_bit / 8;

            if starting_byte == ending_byte {
                let mut num: u8 = self.vec[starting_byte];

                // |  |
                // 101!!101 << 3
                // !!101000 >> 6
                // 000000!!
                let left_shift = starting_bit - starting_byte * 8;
                let right_shift = 8 - (left_shift + eff_size) - 1;

                num = num << left_shift;
                num = num >> right_shift;

                unsafe { std::mem::transmute_copy(&num) }

            } else {
            }
        } else {
            panic!()
        }
    }
}

impl<T: EfficientSize + Copy> EffVec<T> {
    pub fn get(&self, ind: usize) -> T {
        let eff_size = T::EFF_SIZE_BITS; // 1

        let starting_bit = eff_size * ind; // 0
        let ending_bit = starting_bit + eff_size; // 0 + 1 = 1

        let starting_byte = starting_bit / 8; // 0
        let ending_byte = ending_bit / 8; // 0

        if starting_byte == ending_byte {
            let mut num: u8 = self.vec[starting_byte];

            // 0b00000001
            // 0b10000000

            // |  |
            // 101!!101 << 3
            // !!101000 >> 6
            // 000000!!
            let left_shift = 8 - (ending_bit % 8);
            let right_shift = starting_bit % 8;

            println!("start: {}, end: {}", starting_bit, ending_bit);

            num = num << left_shift;
            num = num >> right_shift;

            unsafe { std::mem::transmute_copy(&num) }
        } else {
            unreachable!();
        }
    }
}

impl<'a, T: EfficientSize + Copy> IntoIterator for &'a EffVec<T> {
    type Item = T;

    type IntoIter = EffVecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        EffVecIter {
            eff_vec: self,
            current_element: 0,
        }
    }
}

pub struct EffVecIter<'a, T: EfficientSize> {
    eff_vec: &'a EffVec<T>,
    current_element: usize,
}

impl<'a, T: EfficientSize + Copy> Iterator for EffVecIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {

        if self.eff_vec.len() > self.current_element {

            let opt = Some(self.eff_vec.get(self.current_element));
            self.current_element += 1;
            opt
        }
        else {
            None
        }
    }
}

impl<T: EfficientSize + Debug + Copy> Debug for EffVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        write!(f, "[")?;
        for value in self.into_iter() {
            write!(f, "{:?}, ", value)?;
        };
        write!(f, "]")?;

        std::fmt::Result::Ok(())
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn push_works_easy() {

        let mut vec = EffVec::<bool>::new();

        vec.push(true);
        vec.push(false);
        vec.push(true);
        vec.push(false);

        assert_eq!(&vec.vec, &[0b_0000_0101])
    }

    #[test]
    fn push_works_realloc() {

        let mut vec = EffVec::<bool>::new();

        vec.extend([
            true, true, true, true,
            true, true, true, true,
        ]);

        assert_eq!(&vec.vec, &[0b_1111_1111]);

        vec.extend([
            true, true, true, true,
            true, true, true, true,
        ]);

        assert_eq!(&vec.vec, &[0b_1111_1111, 0b_1111_1111]);

        vec.push(true);

        assert_eq!(&vec.vec, &[0b_1111_1111, 0b_1111_1111, 0b_0000_0001]);
    }

    enum Trio {
        One,
        Two,
        Three,
    }

    unsafe impl EfficientSize for Trio {
        const EFF_SIZE_BITS: usize = 2;
        type Repr = u8;
    }

    #[test]
    fn push_works_eff_size_2() {

        use Trio::*;

        let mut vec = EffVec::<Trio>::new();

        vec.extend([
            Two, Three, Two, Three,
        ]);

        assert_eq!(&vec.vec, &[0b_10_01_10_01]);

        vec.push(Two);

        assert_eq!(&vec.vec, &[0b_10_01_10_01, 0b_00_00_00_01]);
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

    #[test]
    fn push_works_eff_size_3() {

        use Pento::*;

        let mut vec = EffVec::<Pento>::new();

        vec.push(Two);
        vec.push(Three);

        assert_eq!(&vec.vec, &[0b_00_010_001]);

        vec.push(Two);

        //                        ||                      |
        assert_eq!(&vec.vec, &[0b_00_010_001, 0b0_000_000_1]);

        vec.push(Two);

        //                                            |||
        assert_eq!(&vec.vec, &[0b_00_010_001, 0b0_000_010_1]);
    }


    #[test]
    fn get_works_super_easy() {

        use Pento::*;

        let mut vec = EffVec::<bool>::new();

        vec.push(true);

        assert_eq!(vec.vec, &[0b00000001]);

        let value = vec.get(0);

        assert_eq!(value, true);
    }

    #[test]
    fn get_works_easy() {

        use Pento::*;

        let mut vec = EffVec::<bool>::new();

        vec.push(true);
        vec.push(true);

        assert_eq!(vec.vec, &[0b0000_0011]);

        assert!(vec.get(0));

        assert!(vec.get(1));
    }
}
