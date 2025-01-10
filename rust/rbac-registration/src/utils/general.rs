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
