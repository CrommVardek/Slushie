//! # Slushie
//!
//! This is a tornado.cash-like mixer alternative on `pallet-contracts`-compatible chains
//!
//! ## Warning
//!
//! This is in the early stage of development. Use with caution and at your own risk. : )
//!
//! ## Overview
//!
//! Users `deposit` a fixed amount of tokens to a smart contract, wait some time, and then
//! can withdraw it back from another account. Or someone else can do it, who knows
//! the proper information.
//!
//! ## Error Handling
//!
//! Any function that modifies the state returns a `Result` type and does not changes the state
//! if the `Error` occurs. The errors are defined as an `enum` type.
//!
//! ### Deposit
//!
//! Tokens can only be deposited in a constant `deposit_size` amount.
//! Returns a MerkleTree root hash after the insertion of the nullifier.
//!
//! ### Withdraw
//!
//! Tokens can be withdrawn at any time, but for security reasons, it's better to wait some period say, 24 hours
//! after deposit and before withdrawal to make it harder to track the token transfer.
//! Tokens can be withdrawn only in a constant `deposit_size` amount by anyone who knows the nullifier and the root hash.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

mod tree;

#[allow(clippy::let_unit_value)]
#[allow(clippy::large_enum_variant)]
#[ink::contract]
mod slushie {
    use super::*;
    use crate::tree::hasher::Poseidon;
    use crate::tree::merkle_tree::{MerkleTree, MerkleTreeError, DEFAULT_ROOT_HISTORY_SIZE};
    use shared::constants::DEFAULT_DEPTH;

    type PoseidonHash = [u8; 32];

    const SERIALIZED_PP: &[u8] = include_bytes!("./pp-test");

    #[ink(storage)]
    #[derive(ink_storage::traits::SpreadAllocate)]
    pub struct Slushie {
        merkle_tree: MerkleTree<DEFAULT_DEPTH, DEFAULT_ROOT_HISTORY_SIZE, Poseidon>,
        deposit_size: Balance,
        used_nullifiers: ink_storage::Mapping<PoseidonHash, bool>,
    }

    /// Deposit event when the tokens deposited successfully
    #[ink(event)]
    pub struct Deposited {
        #[ink(topic)]
        hash: PoseidonHash,

        timestamp: Timestamp,
    }

    /// Withdraw event when the tokens withdrawn successfully
    #[ink(event)]
    pub struct Withdrawn {
        #[ink(topic)]
        hash: PoseidonHash,

        timestamp: Timestamp,
    }

    /// Errors which my be returned from the smart contract
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        DepositFailure,
        MerkleTreeIsFull,
        MerkleTreeInvalidDepth,
        InvalidTransferredAmount,
        InvalidDepositSize,
        InsufficientFunds,
        NullifierAlreadyUsed,
        UnknownRoot,
        VerificationProofFailed,
        TransferFailed,
    }

    impl From<MerkleTreeError> for Error {
        fn from(err: MerkleTreeError) -> Self {
            match err {
                MerkleTreeError::MerkleTreeIsFull => Error::MerkleTreeIsFull,
                MerkleTreeError::DepthTooLong => Error::MerkleTreeInvalidDepth,
                MerkleTreeError::DepthIsZero => Error::MerkleTreeInvalidDepth,
            }
        }
    }

    /// Struct for public inputs for withdraw method
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PublicInputs {
        nullifier_hash: PoseidonHash,
        root: PoseidonHash,
        proof: [u8; 1040],
        fee: u64,
        recipient: AccountId,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Slushie {
        /// create a new Slushie contract
        ///
        /// Takes the deposit_size Balance amount
        /// so the users can deposit and withdraw
        /// only in a fixed amount of tokens.
        /// Can be set only when the smart contract
        /// instantiated.
        #[ink(constructor)]
        pub fn new(deposit_size: Balance) -> Self {
            ink::utils::initialize_contract(|me: &mut Self| {
                *me = Self {
                    merkle_tree:
                        MerkleTree::<DEFAULT_DEPTH, DEFAULT_ROOT_HISTORY_SIZE, Poseidon>::new()
                            .unwrap(),
                    deposit_size,
                    used_nullifiers: Default::default(),
                };
            })
        }

        /// Deposit a fixed amount of tokens into mixer
        ///
        /// Returns the merkle_tree root hash after insertion
        #[ink(message, payable)]
        pub fn deposit(&mut self, commitment: PoseidonHash) -> Result<PoseidonHash> {
            // Check that transferred value equal to deposit size
            if self.env().transferred_value() != self.deposit_size {
                return Err(Error::InvalidTransferredAmount);
            }

            // Save commitment to the merkle tree
            self.merkle_tree.insert(commitment)?;

            // Emit Deposited Event
            self.env().emit_event(Deposited {
                hash: commitment,
                timestamp: self.env().block_timestamp(),
            });

            Ok(self.merkle_tree.get_last_root() as PoseidonHash)
        }

        /// Withdraw a fixed amount of tokens from the mixer
        ///
        /// Can be withdrawn by anyone who knows the nullifier and the correct root hash
        #[ink(message)]
        pub fn withdraw(&mut self, public_inputs: PublicInputs) -> Result<()> {
            // Check that provided root is known
            if !self.merkle_tree.is_known_root(public_inputs.root) {
                return Err(Error::UnknownRoot);
            }

            // Check that contract has enough balance
            if self.env().balance() < self.deposit_size {
                return Err(Error::InsufficientFunds);
            }

            // Check that provided nullifier hash is not used
            if self
                .used_nullifiers
                .get(public_inputs.nullifier_hash)
                .is_some()
            {
                return Err(Error::NullifierAlreadyUsed);
            }

            // Check provided proof
            if !Self::check_proof(SERIALIZED_PP, &public_inputs, self.env().caller()) {
                return Err(Error::VerificationProofFailed);
            }

            // Transfer to recipient
            self.env()
                .transfer(public_inputs.recipient, self.deposit_size)
                .map_err(|_| Error::TransferFailed)?;

            // Save used nullifier hash
            self.used_nullifiers
                .insert(public_inputs.nullifier_hash, &true);

            // Emit Withdrawn Event
            self.env().emit_event(Withdrawn {
                hash: public_inputs.nullifier_hash,
                timestamp: self.env().block_timestamp(),
            });

            Ok(())
        }

        /// Returns the merkle_tree root hash
        #[ink(message)]
        pub fn get_root_hash(&self) -> PoseidonHash {
            self.merkle_tree.get_last_root() as PoseidonHash
        }

        //TODO MP-34
        fn check_proof(_pp: &[u8], _public_inputs: &PublicInputs, _relayer: AccountId) -> bool {
            true
        }
    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        use super::*;
        use hex_literal::hex;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        #[ink::test]
        fn test_constructor() {
            let slushie: Slushie = Slushie::new(13);

            assert_eq!(slushie.deposit_size, 13 as Balance);
            assert_eq!(
                slushie.merkle_tree,
                MerkleTree::<DEFAULT_DEPTH, DEFAULT_ROOT_HISTORY_SIZE, Poseidon>::new().unwrap()
            );
        }

        /// can deposit funds with a proper `deposit_size`
        #[ink::test]
        fn deposit_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            let mut slushie: Slushie = Slushie::new(13);
            let commitment: PoseidonHash =
                hex!("0001020304050607 08090a0b0c0d0e0f 0001020304050607 08090a0b0c0d0e0f");

            let initial_root_hash = slushie.get_root_hash();

            ink_env::test::set_caller::<Environment>(accounts.bob);
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(13);
            let res = slushie.deposit(commitment);
            assert!(res.is_ok());

            let resulting_root_hash = slushie.get_root_hash();
            assert_ne!(initial_root_hash, resulting_root_hash);
        }

        /// can't deposit funds with an invalid `deposit_size`
        #[ink::test]
        fn deposit_invalid_amount_fails() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            let deposit_size = 13;
            let invalid_deposit_size = 55;
            let mut slushie: Slushie = Slushie::new(deposit_size);
            let commitment: PoseidonHash =
                hex!("0001020304050607 08090a0b0c0d0e0f 0001020304050607 08090a0b0c0d0e0f");

            let initial_root_hash = slushie.get_root_hash();

            ink_env::test::set_caller::<Environment>(accounts.bob);
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(
                invalid_deposit_size,
            );
            let res = slushie.deposit(commitment);
            assert_eq!(res.unwrap_err(), Error::InvalidTransferredAmount);

            let resulting_root_hash = slushie.get_root_hash();
            assert_eq!(initial_root_hash, resulting_root_hash);
        }

        /// can't deposit funds if account doesn't have enough money
        ///
        /// this case shouldn't be tested cause is a pallete, which
        /// checks the sufficient amount of funds

        /// - can withdraw funds with a proper deposit_size and hash
        #[ink::test]
        fn withdraw_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            let deposit_size: Balance = 13;
            let mut slushie: Slushie = Slushie::new(deposit_size);
            let hash: PoseidonHash =
                hex!("0001020304050607 08090a0b0c0d0e0f 0001020304050607 08090a0b0c0d0e0f");

            ink_env::test::set_caller::<Environment>(accounts.alice);
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(deposit_size);
            let res = slushie.deposit(hash);
            assert!(res.is_ok());

            let resulting_root_hash = slushie.get_root_hash();

            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(deposit_size);
            let res = slushie.withdraw(PublicInputs {
                nullifier_hash: hash,
                root: resulting_root_hash,
                proof: [0; 1040],
                fee: 0,
                recipient: accounts.bob,
            });
            assert!(res.is_ok());
        }

        /// - can withdraw funds with a proper deposit_size and hash by different account
        #[ink::test]
        fn withdraw_from_different_account_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            let deposit_size = 13;
            let mut slushie: Slushie = Slushie::new(deposit_size);
            let hash: PoseidonHash =
                hex!("0001020304050607 08090a0b0c0d0e0f 0001020304050607 08090a0b0c0d0e0f");

            ink_env::test::set_caller::<Environment>(accounts.alice);
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(deposit_size);
            let res = slushie.deposit(hash);
            assert!(res.is_ok());

            let resulting_root_hash = slushie.get_root_hash();

            ink_env::test::set_caller::<Environment>(accounts.eve);
            let res = slushie.withdraw(PublicInputs {
                nullifier_hash: hash,
                root: resulting_root_hash,
                proof: [0; 1040],
                fee: 0,
                recipient: accounts.bob,
            });
            assert!(res.is_ok());
        }

        /// - can't withdraw funds with invalid root hash
        #[ink::test]
        fn withdraw_with_invalid_root_fails() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            let deposit_size = 13;
            let mut slushie: Slushie = Slushie::new(deposit_size);
            let hash: PoseidonHash =
                hex!("0001020304050607 08090a0b0c0d0e0f 0001020304050607 08090a0b0c0d0e0f");

            ink_env::test::set_caller::<Environment>(accounts.alice);
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(deposit_size);
            let res = slushie.deposit(hash);
            assert!(res.is_ok());

            let invalid_root_hash: PoseidonHash =
                hex!("0000000000000000 0000000000000000 0001020304050607 08090a0b0c0d0e0f");

            let res = slushie.withdraw(PublicInputs {
                nullifier_hash: hash,
                root: invalid_root_hash,
                proof: [0; 1040],
                fee: 0,
                recipient: accounts.bob,
            });
            assert_eq!(res.unwrap_err(), Error::UnknownRoot);
        }

        /// - can't double withdraw funds with a proper deposit_size and a valid hash
        #[ink::test]
        fn withdraw_with_used_nullifier_fails() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();
            let deposit_size = 13;
            let mut slushie: Slushie = Slushie::new(deposit_size);
            let hash: PoseidonHash =
                hex!("0001020304050607 08090a0b0c0d0e0f 0001020304050607 08090a0b0c0d0e0f");

            ink_env::test::set_caller::<Environment>(accounts.alice);
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(deposit_size);
            let res = slushie.deposit(hash);
            assert!(res.is_ok());
            let resulting_root_hash = slushie.get_root_hash();

            let res = slushie.withdraw(PublicInputs {
                nullifier_hash: hash,
                root: resulting_root_hash,
                proof: [0; 1040],
                fee: 0,
                recipient: accounts.bob,
            });
            assert!(res.is_ok());

            let res = slushie.withdraw(PublicInputs {
                nullifier_hash: hash,
                root: resulting_root_hash,
                proof: [0; 1040],
                fee: 0,
                recipient: accounts.bob,
            });
            assert_eq!(res.unwrap_err(), Error::NullifierAlreadyUsed);
        }
    }
}
