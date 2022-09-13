#![cfg(feature = "proof_generator")]

use crate::hasher::MerkleTreeHasher;
use crate::utils::index_to_path;

use alloc::vec::Vec;
pub struct MerkleTree<const DEPTH: usize, Hash: MerkleTreeHasher> {
    /// Merkle tree nodes
    pub layers: Vec<Vec<Hash::Output>>,
}

impl<const DEPTH: usize, Hash: MerkleTreeHasher> MerkleTree<DEPTH, Hash> {
    pub fn get_opening(&self, leaf_index: usize) -> Result<[Hash::Output; DEPTH], MerkleTreeError> {
        let mut result = [Default::default(); DEPTH];

        let mut current_index = leaf_index;

        for (i, elem) in result.iter_mut().enumerate().take(DEPTH) {
            if current_index % 2 == 0 {
                *elem = *self.layers[i]
                    .get(current_index + 1)
                    .ok_or(MerkleTreeError::WrongLeafIndex)?;
            } else {
                *elem = *self.layers[i]
                    .get(current_index - 1)
                    .ok_or(MerkleTreeError::WrongLeafIndex)?;
            }

            current_index >>= 1;
        }

        Ok(result)
    }

    pub fn get_path(&self, leaf_index: usize) -> Result<[u8; DEPTH], MerkleTreeError> {
        index_to_path(leaf_index).map_err(|_| MerkleTreeError::WrongLeafIndex)
    }
}

/// Creation merkle tree from array
impl<const DEPTH: usize, Hash: MerkleTreeHasher> TryFrom<&[Hash::Output]>
    for MerkleTree<DEPTH, Hash>
{
    type Error = MerkleTreeError;

    fn try_from(source: &[Hash::Output]) -> Result<MerkleTree<DEPTH, Hash>, MerkleTreeError> {
        if source.len() >= 1 << DEPTH {
            return Err(MerkleTreeError::VecTooLong);
        }

        let mut leaves = vec![Hash::ZEROS[0]; 1 << DEPTH];

        // Fill leaves using source vec
        for (i, elem) in source.iter().enumerate() {
            leaves[i] = *elem;
        }
        let mut layers = Vec::with_capacity(DEPTH);
        layers.push(leaves);

        // Compute all node hashes
        for i in 1..DEPTH {
            layers.push(vec![Hash::ZEROS[i]; 1 << (DEPTH - i)]);

            for j in 0..source.len() / (1 << i) {
                layers[i][j] =
                    Hash::hash_left_right(layers[i - 1][2 * j], layers[i - 1][2 * j + 1]);
            }
        }

        Ok(Self { layers })
    }
}

#[derive(Debug)]
pub enum MerkleTreeError {
    VecTooLong,
    WrongLeafIndex,
}
