/// Cast bytes array to u64 array
pub fn bytes_to_u64(bytes: [u8; 32]) -> [u64; 4] {
    let mut result = [0; 4];

    for (i, item) in result.iter_mut().enumerate() {
        let bytes_8 = bytes.split_at(i * 8).1.split_at(8).0;
        let bytes_array = <&[u8; 8]>::try_from(bytes_8).unwrap();
        *item = u64::from_be_bytes(*bytes_array);
    }

    result
}

/// Cast u64 array to bytes array
#[allow(dead_code)]
pub fn u64_to_bytes(array: [u64; 4]) -> [u8; 32] {
    let mut result = [0; 32];

    for i in 0..array.len() {
        let bytes_array = array[i].to_be_bytes();
        for j in 0..bytes_array.len() {
            result[i * 8 + j] = bytes_array[j];
        }
    }

    result
}

/// Array that implement Default trait for creating default circuit
#[derive(Debug)]
pub(crate) struct Array<T: Default + Clone + Copy, const N: usize>(pub [T; N]);

impl<T: Default + Clone + Copy, const N: usize> Default for Array<T, N> {
    fn default() -> Self {
        Self([Default::default(); N])
    }
}

/// Generate path from index and tree`s depth
pub(crate) fn index_to_path<const DEPTH: usize>(index: usize) -> [u8; DEPTH] {
    let mut result = [0; DEPTH];

    let mut current_index = index;
    for path in result.iter_mut().take(DEPTH) {
        *path = (current_index % 2) as u8;

        current_index >>= 1;
    }

    result
}
