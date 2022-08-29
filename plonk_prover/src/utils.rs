/// Array that implement Default trait for creating default circuit
#[derive(Debug)]
pub(crate) struct Array<T: Default + Clone + Copy, const N: usize>(pub [T; N]);

impl<T: Default + Clone + Copy, const N: usize> Default for Array<T, N> {
    fn default() -> Self {
        Self([Default::default(); N])
    }
}

/// Generate path from index and tree`s depth
#[cfg(feature = "proof_generator")]
pub fn index_to_path<const DEPTH: usize>(index: usize) -> Result<[u8; DEPTH], IndexToPathError> {
    let mut result = [0; DEPTH];

    if index > (1 << DEPTH) - 1 {
        return Err(IndexToPathError::WrongIndex);
    }

    let mut current_index = index;
    for path in result.iter_mut().take(DEPTH) {
        *path = (current_index % 2) as u8;

        current_index >>= 1;
    }

    Ok(result)
}

#[cfg(feature = "proof_generator")]
#[derive(Debug)]
pub enum IndexToPathError {
    WrongIndex,
}
