use dusk_bls12_381::BlsScalar;

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

pub fn bytes_to_scalar(bytes: [u8; 32]) -> BlsScalar {
    BlsScalar(bytes_to_u64(bytes))
}

pub fn scalar_to_bytes(scalar: BlsScalar) -> [u8; 32] {
    u64_to_bytes(*scalar.internal_repr())
}
