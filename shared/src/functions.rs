use dusk_bls12_381::BlsScalar;
/// Cast bytes array to u64 array
pub fn bytes_to_u64(bytes: [u8; 32]) -> [u64; 4] {
    let mut result = [0; 4];

    for i in 0..4 {
        let bytes_array = <&[u8; 8]>::try_from(&bytes[i * 8..(i + 1) * 8]).unwrap();
        result[i] = u64::from_be_bytes(*bytes_array);
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

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn to_bytes_and_back() {
        assert_eq!(bytes_to_u64(u64_to_bytes([1, 2, 3, 4])), [1, 2, 3, 4]);
        assert_eq!(
            bytes_to_u64(u64_to_bytes([12314, 4423, 53, 4923])),
            [12314, 4423, 53, 4923]
        );
        assert_eq!(
            bytes_to_u64(u64_to_bytes([9483, 3212, 2324, 12432])),
            [9483, 3212, 2324, 12432]
        );
        assert_eq!(bytes_to_u64(u64_to_bytes([0, 3, 8, 120])), [0, 3, 8, 120]);
    }

    #[test]
    fn to_u64_and_back() {
        let bytes = hex!("ad344e9209368feb18d238773ee6fa09ea90180d99a4913f293e4fd4a659f329");
        assert_eq!(u64_to_bytes(bytes_to_u64(bytes)), bytes);
        let bytes = hex!("1f7ab5c71a2942319ad238773ee6fa09ea90180d99a4913f293e4fd4a659f329");
        assert_eq!(u64_to_bytes(bytes_to_u64(bytes)), bytes);
        let bytes = hex!("3f293e4fd4a659f3298773ee6fa09ea90180d99a4913f293e4fd4a6595c71a29");
        assert_eq!(u64_to_bytes(bytes_to_u64(bytes)), bytes);
        let bytes = hex!("ad344e9209368feb18db5c71a2942319ad29a4913f293e4913f293e4fd4af322");
        assert_eq!(u64_to_bytes(bytes_to_u64(bytes)), bytes);
    }
}
