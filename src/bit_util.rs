use std::ops::BitAnd;

pub trait BitUtil<T> {
    fn is_set(self, target: T) -> bool;
}

impl<T> BitUtil<T> for T 
    where T: BitAnd + Copy, 
          T::Output: PartialEq + Default {
    fn is_set(self, target: T) -> bool {
        (self & target) != T::Output::default()
    }
}
