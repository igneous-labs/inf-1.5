#[inline]
pub const fn bool_to_u8(b: bool) -> u8 {
    match b {
        true => 1,
        false => 0,
    }
}

#[inline]
pub const fn u8_to_bool(u: u8) -> bool {
    !matches!(u, 0)
}
