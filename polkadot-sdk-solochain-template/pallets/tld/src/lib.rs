//! # TLD Pallet
//!
//! The TLD pallet enables the registration and management of top-level domain (TLD) networks on the blockchain.
//! It facilitates the storage of TLD chain specifications and provides functionality for their modification and removal.
//!
//! ## Features
//! - **Register TLDs**: Allows users to register unique TLDs with their associated chain specifications.
//! - **Amend Chain Specifications**: Enables TLD owners to update their chain specifications.
//! - **Revoke TLDs**: Provides TLD owners the ability to remove their TLD registrations.
//! - **Expiration Management**: Ensures TLD entries expire at a specified block number unless renewed.
//! - **Access Control**: Restricts actions based on ownership and account verification.
//!
//! ## Overview
//! The pallet maintains mappings between TLD names and their metadata, including chain specifications, creators, and expiration blocks. Administrators or authorized users can perform the following operations:
//! - Register a TLD with a unique name, chain specification, maintainer details, and an expiration block.
//! - Update the chain specification for an existing TLD.
//! - Remove expired or invalid TLDs.
//!
//! ## Extrinsics
//! - **`register_domain`**: Registers a new TLD with specified details.
//! - **`amend_chainspec`**: Updates the chain specification for an existing TLD.
//! - **`revoke_domain`**: Removes a TLD from the registry.
//!
//! ## Storage
//! - **`DomainMap`**: Maps TLD names to their metadata, including creator, chain specification, maintainer, and availability.
//! - **`DomainExpiry`**: Stores the block number at which a TLD registration expires.
//!
//! ## Events
//! - **`DomainRegistered`**: Triggered when a TLD is successfully registered.
//! - **`DomainAmended`**: Triggered when a TLD's chain specification is updated.
//! - **`DomainRevoked`**: Triggered when a TLD is removed from the registry.
//!
//! ## Errors
//! - **`DomainNameTooLong`**: The provided TLD name exceeds the maximum allowed length.
//! - **`ChainSpecTooLarge`**: The chain specification exceeds the maximum allowed size.
//! - **`MaintainerTooLarge`**: The maintainer details exceed the maximum allowed size.
//! - **`DomainAlreadyExists`**: The specified TLD is already registered.
//! - **`DomainNotFound`**: The specified TLD does not exist.
//! - **`DomainExpired`**: The TLD registration has expired.
//! - **`InvalidOwnerId`**: The caller does not own the specified TLD.
//!
//! ## Usage
//! 1. **Register a TLD**: Call `register_domain` with a unique TLD name, valid chain specification, maintainer details, and an expiration block.
//! 2. **Amend Chain Specification**: Call `amend_chainspec` to update the chain specification of an existing TLD.
//! 3. **Revoke a TLD**: Use `revoke_domain` to remove an existing TLD from the registry.
//!
//! ## Note
//! Run `cargo doc --package pallet-tld --open` to view the complete documentation for this pallet.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use crate::{SubstrateWeight, WeightInfo};
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use scale_info::prelude::vec::Vec;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        #[pallet::constant]
        type MaxDomainLength: Get<u32>;
        #[pallet::constant]
        type MaxChainSpecSize: Get<u32>;
        #[pallet::constant]
        type MaxMaintainerSize: Get<u32>;
    }

    #[derive(Debug, Encode, Decode, Clone, PartialEq, Default, TypeInfo)]
    pub struct DomainInfo<AccountId> {
        pub creator: AccountId,
        pub chain_spec: Vec<u8>,
        pub maintainer: Vec<u8>,
        pub available: bool,
    }

    impl<AccountId> DomainInfo<AccountId> {
        pub fn new(
            creator: AccountId,
            chain_spec: Vec<u8>,
            maintainer: Vec<u8>,
            available: bool,
        ) -> Self {
            Self {
                creator,
                chain_spec,
                maintainer,
                available,
            }
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn domain_map)]
    pub(super) type DomainMap<T: Config> =
        StorageMap<_, Blake2_128Concat, Vec<u8>, DomainInfo<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn domain_expiry)]
    pub(super) type DomainExpiry<T: Config> =
        StorageMap<_, Blake2_128Concat, Vec<u8>, BlockNumberFor<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_transfers)]
    pub(super) type PendingTransfers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Vec<u8>,      // Domain name
        T::AccountId, // New owner
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        DomainRegistered {
            domain_name: Vec<u8>,
            creator: T::AccountId,
        },
        DomainAmended {
            domain_name: Vec<u8>,
            editor: T::AccountId,
        },
        DomainRevoked {
            domain_name: Vec<u8>,
            revoker: T::AccountId,
        },
        TransferInitiated {
            domain_name: Vec<u8>,
            from: T::AccountId,
            to: T::AccountId,
        },
        TransferAccepted {
            domain_name: Vec<u8>,
            new_owner: T::AccountId,
        },
        TransferRevoked {
            domain_name: Vec<u8>,
            owner: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        DomainNameTooLong,
        ChainSpecTooLarge,
        MaintainerTooLarge,
        DomainAlreadyExists,
        DomainNotFound,
        DomainExpired,
        InvalidOwnerId,
        TransferAlreadyPending,
        NoPendingTransfer,
        NotDomainOwner,
        NotTransferRecipient,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_register_domain())]
        pub fn register_domain(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            chain_spec: Vec<u8>,
            maintainer: Vec<u8>,
            expiry: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !(DomainMap::<T>::contains_key(&domain_name)
                    && Pallet::<T>::ensure_not_expired(&domain_name).is_ok()),
                Error::<T>::DomainAlreadyExists
            );

            let domain_info = DomainInfo::new(who.clone(), chain_spec, maintainer, false);
            DomainMap::<T>::insert(&domain_name, &domain_info);
            DomainExpiry::<T>::insert(&domain_name, expiry);

            Self::deposit_event(Event::DomainRegistered {
                domain_name,
                creator: who,
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_amend_chainspec())]
        pub fn amend_chainspec(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            chain_spec: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut domain_info =
                DomainMap::<T>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;
            ensure!(who == domain_info.creator, Error::<T>::InvalidOwnerId);

            domain_info.chain_spec = chain_spec;
            DomainMap::<T>::insert(&domain_name, domain_info);

            Self::deposit_event(Event::DomainAmended {
                domain_name,
                editor: who,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_revoke_domain())]
        pub fn revoke_domain(origin: OriginFor<T>, domain_name: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let domain_info =
                DomainMap::<T>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;
            ensure!(who == domain_info.creator, Error::<T>::InvalidOwnerId);

            DomainMap::<T>::remove(&domain_name);
            DomainExpiry::<T>::remove(&domain_name);

            Self::deposit_event(Event::DomainRevoked {
                domain_name,
                revoker: who,
            });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_initiate_transfer())]
        pub fn initiate_transfer(
            origin: OriginFor<T>,
            domain_name: Vec<u8>,
            new_owner: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let domain_info =
                DomainMap::<T>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;
            ensure!(who == domain_info.creator, Error::<T>::NotDomainOwner);
            ensure!(
                !PendingTransfers::<T>::contains_key(&domain_name),
                Error::<T>::TransferAlreadyPending
            );

            PendingTransfers::<T>::insert(&domain_name, &new_owner);

            Self::deposit_event(Event::TransferInitiated {
                domain_name,
                from: who,
                to: new_owner,
            });

            Ok(())
        }
        #[pallet::call_index(4)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_accept_transfer())]
        pub fn accept_transfer(origin: OriginFor<T>, domain_name: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let new_owner =
                PendingTransfers::<T>::get(&domain_name).ok_or(Error::<T>::NoPendingTransfer)?;
            ensure!(who == new_owner, Error::<T>::NotTransferRecipient);

            let mut domain_info =
                DomainMap::<T>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;
            domain_info.creator = who.clone();

            DomainMap::<T>::insert(&domain_name, domain_info);
            PendingTransfers::<T>::remove(&domain_name);

            Self::deposit_event(Event::TransferAccepted {
                domain_name,
                new_owner: who,
            });

            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_revoke_transfer())]
        pub fn revoke_transfer(origin: OriginFor<T>, domain_name: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let domain_info =
                DomainMap::<T>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;
            ensure!(who == domain_info.creator, Error::<T>::NotDomainOwner);

            ensure!(
                PendingTransfers::<T>::contains_key(&domain_name),
                Error::<T>::NoPendingTransfer
            );

            PendingTransfers::<T>::remove(&domain_name);

            Self::deposit_event(Event::TransferRevoked {
                domain_name,
                owner: who,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn ensure_not_expired(domain_name: &Vec<u8>) -> DispatchResult {
            let expiry = DomainExpiry::<T>::get(domain_name).ok_or(Error::<T>::DomainNotFound)?;
            ensure!(
                frame_system::Pallet::<T>::block_number() < expiry,
                Error::<T>::DomainExpired
            );
            Ok(())
        }
    }
}
