//! Pod boolean type derived from a single byte where:
//! false = all zeros
//! true = anything other bit pattern

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct U8Bool<'a>(pub &'a u8);

impl U8Bool<'_> {
    #[inline]
    pub const fn to_bool(&self) -> bool {
        *self.0 != 0
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct U8BoolMut<'a>(pub &'a mut u8);

impl U8BoolMut<'_> {
    #[inline]
    pub fn set_true(&mut self) {
        *self.0 = 1;
    }

    #[inline]
    pub fn set_false(&mut self) {
        *self.0 = 0;
    }
}
