//! Utility functions.

use std::io::Read;

/// Read a N-byte array from the reader.
#[inline]
pub(crate) fn read_array<R: Read, const N: usize>(reader: &mut R) -> anyhow::Result<[u8; N]> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}
