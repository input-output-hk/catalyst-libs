//! General utility functions

/// Getting the index by decrementing by 1.
/// e.g. 1 should refers to index 0
pub(crate) fn decremented_index(int: i16) -> anyhow::Result<usize> {
    match usize::try_from(int) {
        Ok(value) => Ok(value - 1),
        Err(e) => {
            Err(anyhow::Error::msg(format!(
                "Failed to convert to usize: {e}"
            )))
        },
    }
}

/// Decode the given UTF-8 content.
pub(crate) fn decode_utf8(content: &[u8]) -> anyhow::Result<String> {
    // Decode the UTF-8 string
    std::str::from_utf8(content)
        .map(std::string::ToString::to_string)
        .map_err(|_| {
            anyhow::anyhow!(
                "Invalid UTF-8 string, expected valid UTF-8 string but got {:?}",
                content
            )
        })
}

/// Zero out the last n bytes
pub(crate) fn zero_out_last_n_bytes(vec: &mut [u8], n: usize) {
    if let Some(slice) = vec.get_mut(vec.len().saturating_sub(n)..) {
        slice.fill(0);
    }
}
