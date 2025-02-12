//! # Rootdns Pallet
//!
//! The Rootdns pallet allows users to register top-level domain (TLD) networks on the blockchain.
//! The pallet stores the chain specification of the TLD network.
//!
//! ## Overview
//! - Register TLDs with their chain specifications.
//! - Prevent duplicate registrations.
//! - Optionally restrict TLD registration to authorized accounts.
//! - Allow administrators to remove invalid TLD entries.
//!
//! ## Extrinsics
//! - `register_tld`: Registers a new TLD.
//! - `remove_tld`: Removes an existing TLD (admin-only).
//!
//! ## Error Handling
//! - Handles duplicate TLD registrations.
//! - Ensures input lengths are within allowed limits.
//!
//! ## Storage
//! - `TLDMap`: Maps TLD names to their associated chain specifications.
//!
//! ## Usage
//! 1. Call `register_tld` with a unique TLD name and valid chain specification.
//! 2. Call `remove_tld` through network governance to manage invalid or outdated entries.
//!
//! Run `cargo doc --package pallet-rootdns --open` to view this pallet's documentation.

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
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + TypeInfo {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        /// Origin that is allowed to manage TLDs (e.g., remove invalid ones).
        /// Can be agreed upon by the chain's governance.
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        /// Maximum length allowed for TLD names.
        #[pallet::constant]
        type MaxTLDNameLength: Get<u32>;
        /// Maximum size allowed for the chain specification.
        #[pallet::constant]
        type MaxChainSpecSize: Get<u32>;
    }

    // Type aliases
    type TLDName<T> = BoundedVec<u8, <T as Config>::MaxTLDNameLength>;
    type ChainSpec<T> = BoundedVec<u8, <T as Config>::MaxChainSpecSize>;

    #[derive(Encode, Decode, Clone, PartialEq, Default, TypeInfo)]
    pub struct TLDInfo<T: Config> {
        // The chain specification of the TLD network
        pub chain_spec: ChainSpec<T>,
    }

    /// TLDMap stores the chain specification of the TLD network.
    #[pallet::storage]
    #[pallet::getter(fn tld_map)]
    pub(super) type TLDMap<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        TLDName<T>, // The TLD name
        TLDInfo<T>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new TLD has been registered.
        TLDRegistered {
            tld_name: TLDName<T>,
            creator: T::AccountId,
        },
        /// A TLD has been removed.
        TLDRemoved { tld_name: TLDName<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The TLD has already been registered.
        TLDAlreadyRegistered,
        /// The TLD name exceeds the maximum allowed length.
        TLDNameTooLong,
        /// The chain specification exceeds the maximum allowed size.
        ChainSpecTooLarge,
        /// The specified TLD does not exist.
        TLDNotFound,
    }

    impl<T: Config> Pallet<T> {
        /// Get the chain specification of the TLD network.
        pub fn get_chainspec_for_tld(tld: &[u8]) -> Option<TLDInfo<T>> {
            let bounded_tld: TLDName<T> = BoundedVec::try_from(tld.to_vec())
                .map_err(|_| Error::<T>::TLDNameTooLong)
                .ok()?;
            TLDMap::<T>::get(bounded_tld)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a top-level domain (TLD) network on the blockchain.
        #[pallet::call_index(0)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_register_tld())]
        pub fn register_tld(
            origin: OriginFor<T>,
            tld_name: TLDName<T>,
            chain_spec: ChainSpec<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Validate input sizes
            ensure!(
                tld_name.len() <= T::MaxTLDNameLength::get() as usize,
                Error::<T>::TLDNameTooLong
            );
            ensure!(
                chain_spec.len() <= T::MaxChainSpecSize::get() as usize,
                Error::<T>::ChainSpecTooLarge
            );

            // Check if the TLD already exists
            ensure!(
                !TLDMap::<T>::contains_key(&tld_name),
                Error::<T>::TLDAlreadyRegistered
            );

            // Insert into storage
            let tld_info = TLDInfo { chain_spec };
            TLDMap::<T>::insert(&tld_name, tld_info);

            // Emit an event
            Self::deposit_event(Event::TLDRegistered {
                tld_name,
                creator: who,
            });

            Ok(())
        }

        /// Remove an existing TLD (admin-only).
        #[pallet::call_index(1)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_remove_tld())]
        pub fn remove_tld(origin: OriginFor<T>, tld_name: TLDName<T>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            ensure!(
                TLDMap::<T>::contains_key(&tld_name),
                Error::<T>::TLDNotFound
            );

            TLDMap::<T>::remove(&tld_name);

            // Emit an event
            Self::deposit_event(Event::TLDRemoved { tld_name });

            Ok(())
        }
    }
}
