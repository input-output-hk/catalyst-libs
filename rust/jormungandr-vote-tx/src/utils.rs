//! Utility functions.

use std::io::Read;

/// Read a single byte from the reader.
#[inline]
pub(crate) fn read_be_u8<R: Read>(reader: &mut R) -> anyhow::Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(u8::from_be_bytes(buf))
}

/// Read a big-endian u32 from the reader.
#[inline]
pub(crate) fn read_be_u32<R: Read>(reader: &mut R) -> anyhow::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

/// Read a big-endian u64 from the reader.
#[inline]
pub fn read_be_u64<R: Read>(reader: &mut R) -> anyhow::Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

/// Read a N-byte array from the reader.
#[inline]
pub(crate) fn read_array<R: Read, const N: usize>(reader: &mut R) -> anyhow::Result<[u8; N]> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}
