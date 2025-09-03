//! Pod boolean type derived from a single byte where:
//! false = all zeros
//! true = anything other bit pattern

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct U8Bool<'a>(pub &'a u8);

impl U8Bool<'_> {
    pub const fn is_false(&self) -> bool {
        *self.0 == 0
    }

    pub const fn is_true(&self) -> bool {
        !self.is_false()
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct U8BoolMut<'a>(pub &'a mut u8);

impl U8BoolMut<'_> {
    pub fn set_true(&mut self) {
        *self.0 = 1;
    }

    pub fn set_false(&mut self) {
        *self.0 = 0;
    }
}
