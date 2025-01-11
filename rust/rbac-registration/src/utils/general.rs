//! General utility functions

use anyhow::Context;

/// Getting the index by decrementing by 1.
/// e.g. 1 should refers to index 0
pub(crate) fn decremented_index(int: i16) -> anyhow::Result<usize> {
    int.checked_sub(1)
        .and_then(|v| usize::try_from(v).ok())
        .context("Failed to convert '{int}' to usize: {e:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decremented_index() {
        assert_eq!(0, decremented_index(1).unwrap());
        decremented_index(0).unwrap_err();
        decremented_index(-1).unwrap_err();
    }
}
