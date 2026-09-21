use core::fmt;

/// Fixed-capacity, stack-only [`fmt::Write`] sink. Never allocates.
///
/// On overflow the message is silently truncated at a UTF-8 char boundary.
pub struct LogBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

#[allow(clippy::new_without_default)]
impl<const N: usize> LogBuf<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: `write_str` only ever copies whole `&str` contents and snaps
        // truncation to a char boundary, so the buffer is always valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }
}

impl<const N: usize> fmt::Write for LogBuf<N> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let space = N - self.len;
        let mut n = s.len().min(space);
        while !s.is_char_boundary(n) {
            n -= 1;
        }
        self.buf[self.len..self.len + n].copy_from_slice(&s.as_bytes()[..n]);
        self.len += n;
        Ok(())
    }
}
