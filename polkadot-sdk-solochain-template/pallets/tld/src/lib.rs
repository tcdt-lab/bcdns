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

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + TypeInfo {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        #[pallet::constant]
        type MaxDomainLength: Get<u32>;
        #[pallet::constant]
        type MaxChainSpecSize: Get<u32>;
        #[pallet::constant]
        type MaxMaintainerSize: Get<u32>;
    }

    // Type aliases
    type DomainName<T> = BoundedVec<u8, <T as Config>::MaxDomainLength>;
    type ChainSpec<T> = BoundedVec<u8, <T as Config>::MaxChainSpecSize>;
    type Maintainer<T> = BoundedVec<u8, <T as Config>::MaxMaintainerSize>;

    #[derive(Debug, Encode, Decode, Clone, PartialEq, Default, TypeInfo)]
    pub struct DomainInfo<T: Config> {
        pub creator: T::AccountId,     // Account that created the domain
        pub chain_spec: ChainSpec<T>,  // Blockchain chain specification
        pub maintainer: Maintainer<T>, // Maintainer's details
        pub available: bool,           // Indicates if the domain is active
    }

    impl<T: Config> DomainInfo<T> {
        pub fn new(
            creator: T::AccountId,
            chain_spec: ChainSpec<T>,
            maintainer: Maintainer<T>,
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

    // Maps domain names to domain metadata
    #[pallet::storage]
    #[pallet::getter(fn domain_map)]
    pub(super) type DomainMap<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainName<T>, DomainInfo<T>, OptionQuery>;

    // Tracks the expiration block of each domain
    #[pallet::storage]
    #[pallet::getter(fn domain_expiry)]
    pub(super) type DomainExpiry<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainName<T>, BlockNumberFor<T>, OptionQuery>;

    // Tracks pending domain ownership transfers
    #[pallet::storage]
    #[pallet::getter(fn pending_transfers)]
    pub(super) type PendingTransfers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        DomainName<T>, // Domain name
        T::AccountId,  // New owner
        OptionQuery,
    >;

    // Events emitted by the pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        DomainRegistered {
            domain_name: DomainName<T>, // Domain name being registered
            creator: T::AccountId,      // Creator's account
        },
        DomainAmended {
            domain_name: DomainName<T>, // Domain name being amended
            editor: T::AccountId,       // Editor's account
        },
        DomainRevoked {
            domain_name: DomainName<T>, // Domain name being revoked
            revoker: T::AccountId,      // Revoker's account
        },
        TransferInitiated {
            domain_name: DomainName<T>, // Domain name for transfer
            from: T::AccountId,         // Transferor account
            to: T::AccountId,           // Transferee account
        },
        TransferAccepted {
            domain_name: DomainName<T>, // Domain name for transfer acceptance
            new_owner: T::AccountId,    // New owner account
        },
        TransferRevoked {
            domain_name: DomainName<T>, // Domain name transfer revoked
            owner: T::AccountId,        // Owner's account
        },
    }

    // Errors emitted by the pallet
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

    // Pallet's extrinsics (functions callable from outside)
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // Registers a new domain with provided details
        #[pallet::call_index(0)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_register_domain())]
        pub fn register_domain(
            origin: OriginFor<T>,
            domain_name: DomainName<T>,
            chain_spec: ChainSpec<T>,
            maintainer: Maintainer<T>,
            expiry: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure domain does not already exist or expired
            ensure!(
                !(DomainMap::<T>::contains_key(&domain_name)
                    && Pallet::<T>::ensure_not_expired(&domain_name).is_ok()),
                Error::<T>::DomainAlreadyExists
            );

            let domain_info = DomainInfo::new(who.clone(), chain_spec, maintainer, false);
            DomainMap::<T>::insert(&domain_name, &domain_info);
            DomainExpiry::<T>::insert(&domain_name, expiry);

            // Emit domain registered event
            Self::deposit_event(Event::DomainRegistered {
                domain_name,
                creator: who,
            });

            Ok(())
        }

        // Amend the chain specification of an existing domain
        #[pallet::call_index(1)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_amend_chainspec())]
        pub fn amend_chainspec(
            origin: OriginFor<T>,
            domain_name: DomainName<T>,
            chain_spec: ChainSpec<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut domain_info =
                DomainMap::<T>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;
            ensure!(who == domain_info.creator, Error::<T>::InvalidOwnerId);

            domain_info.chain_spec = chain_spec;
            DomainMap::<T>::insert(&domain_name, domain_info);

            // Emit domain amended event
            Self::deposit_event(Event::DomainAmended {
                domain_name,
                editor: who,
            });

            Ok(())
        }

        // Revoke a domain from the registry
        #[pallet::call_index(2)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_revoke_domain())]
        pub fn revoke_domain(origin: OriginFor<T>, domain_name: DomainName<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let domain_info =
                DomainMap::<T>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;
            ensure!(who == domain_info.creator, Error::<T>::InvalidOwnerId);

            DomainMap::<T>::remove(&domain_name);
            DomainExpiry::<T>::remove(&domain_name);

            // Emit domain revoked event
            Self::deposit_event(Event::DomainRevoked {
                domain_name,
                revoker: who,
            });

            Ok(())
        }

        // Initiate a transfer of domain ownership
        #[pallet::call_index(3)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_initiate_transfer())]
        pub fn initiate_transfer(
            origin: OriginFor<T>,
            domain_name: DomainName<T>,
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

            // Emit transfer initiated event
            Self::deposit_event(Event::TransferInitiated {
                domain_name,
                from: who,
                to: new_owner,
            });

            Ok(())
        }

        // Accept a pending transfer of domain ownership
        #[pallet::call_index(4)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_accept_transfer())]
        pub fn accept_transfer(origin: OriginFor<T>, domain_name: DomainName<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let new_owner =
                PendingTransfers::<T>::get(&domain_name).ok_or(Error::<T>::NoPendingTransfer)?;
            ensure!(who == new_owner, Error::<T>::NotTransferRecipient);

            let mut domain_info =
                DomainMap::<T>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;
            domain_info.creator = who.clone();

            DomainMap::<T>::insert(&domain_name, domain_info);
            PendingTransfers::<T>::remove(&domain_name);

            // Emit transfer accepted event
            Self::deposit_event(Event::TransferAccepted {
                domain_name,
                new_owner: who,
            });

            Ok(())
        }

        // Revoke a pending domain transfer
        #[pallet::call_index(5)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_revoke_transfer())]
        pub fn revoke_transfer(origin: OriginFor<T>, domain_name: DomainName<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let domain_info =
                DomainMap::<T>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;
            ensure!(who == domain_info.creator, Error::<T>::NotDomainOwner);

            ensure!(
                PendingTransfers::<T>::contains_key(&domain_name),
                Error::<T>::NoPendingTransfer
            );

            PendingTransfers::<T>::remove(&domain_name);

            // Emit transfer revoked event
            Self::deposit_event(Event::TransferRevoked {
                domain_name,
                owner: who,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // Ensure that the domain has not expired
        fn ensure_not_expired(domain_name: &DomainName<T>) -> DispatchResult {
            let expiry = DomainExpiry::<T>::get(domain_name).ok_or(Error::<T>::DomainNotFound)?;
            ensure!(
                frame_system::Pallet::<T>::block_number() < expiry,
                Error::<T>::DomainExpired
            );
            Ok(())
        }
    }
}
