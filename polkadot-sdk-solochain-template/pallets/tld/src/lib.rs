//! # TLD Pallet
//!
//! The TLD pallet enables the registration and management of top-level domain (TLD) networks on the blockchain.
//! It facilitates the storage of domain chain specifications and provides functionality for their modification and removal.
//!
//! ## Features
//! - **Register domains**: Allows users to register unique domains with their associated chain specifications.
//! - **Amend Chain Specifications**: Enables domain owners to update their chain specifications.
//! - **Revoke domain**: Provides domain owners the ability to remove their domain registrations.
//! - **Expiration Management**: Ensures domain entries expire at a specified block number unless renewed.
//! - **Access Control**: Restricts actions based on ownership and account verification.
//!
//! ## Overview
//! The pallet maintains mappings between domain names and their metadata, including chain specifications, creators, and expiration blocks. Administrators or authorized users can perform the following operations:
//! - Register a unique domain name with a chain specification, maintainer details, and an expiration block.
//! - Update the chain specification for an existing domain.
//! - Remove expired or invalid domains.
//! - Send heartbeats to prove domain maintainer is online.
//! - Report missed heartbeats to trigger revocation.
//!
//! ## Extrinsics
//! - **`register_domain`**: Registers a new domain with specified details.
//! - **`amend_chainspec`**: Updates the chain specification for an existing domain.
//! - **`revoke_domain`**: Removes a domain from the registry.
//! - **`send_heartbeat`**: Sends a heartbeat to prove domain maintainer is online.
//! - **`report_missed_heartbeat`**: Reports a domain with missed heartbeat.
//!
//! ## Storage
//! - **`DomainMap`**: Maps domain names to their metadata, including creator, chain specification, maintainer, and availability.
//! - **`DomainExpiry`**: Stores the block number at which a domain registration expires.
//! - **`ActiveDomains`**: Tracks domains that need heartbeat monitoring.
//!
//! ## Events
//! - **`DomainRegistered`**: Triggered when a domain is successfully registered.
//! - **`DomainAmended`**: Triggered when a domain's chain specification is updated.
//! - **`DomainRevoked`**: Triggered when a domain is removed from the registry.
//! - **`DomainHeartbeat`**: Triggered when a domain's maintainer sends a heartbeat.
//! - **`DomainExpiredHeartbeat`**: Triggered when a domain's maintainer fails to send a heartbeat.
//! - **`TransferInitiated`**: Triggered when a domain transfer is initiated.
//! - **`TransferAccepted`**: Triggered when a domain transfer is accepted.
//! - **`TransferRevoked`**: Triggered when a domain transfer is revoked.
//!
//! ## Errors
//! - **`DomainNameTooLong`**: The provided domain name exceeds the maximum allowed length.
//! - **`ChainSpecTooLarge`**: The chain specification exceeds the maximum allowed size.
//! - **`MaintainerTooLarge`**: The maintainer details exceed the maximum allowed size.
//! - **`DomainAlreadyExists`**: The specified domain is already registered.
//! - **`DomainNotFound`**: The specified domain does not exist.
//! - **`DomainExpired`**: The domain registration has expired.
//! - **`InvalidOwnerId`**: The caller does not own the specified domain.
//! - **`DuplicatePeerObservation`**: The peer observation already exists.
//! - **`InvalidUnsignedTransaction`**: The unsigned transaction is invalid.
//! - **`NotDomainMaintainer`**: The caller is not the maintainer of the domain.
//! - **`HeartbeatTooSoon`**: The heartbeat is sent too soon.
//! - **`DuplicateHeartbeatObservation`**: The heartbeat observation already exists.
//!
//! ## Usage
//! 1. **Register a Domain**: Call `register_domain` with a unique domain name, valid chain specification, maintainer details, and an expiration block.
//! 2. **Amend Chain Specification**: Call `amend_chainspec` to update the chain specification of an existing domain.
//! 3. **Revoke a domain**: Use `revoke_domain` to remove an existing domain from the registry.
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
	use frame_support::{
        sp_runtime::Saturating,
		pallet_prelude::*,
	};
	use frame_system::{
		offchain::{AppCrypto, CreateSignedTransaction, Signer, SendSignedTransaction, SigningTypes},
		pallet_prelude::*,
	};

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config + TypeInfo {
        /// The identifier type for an offchain worker.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        #[pallet::constant]
        type MaxDomainLength: Get<u32>;
        #[pallet::constant]
        type MaxChainSpecSize: Get<u32>;
        #[pallet::constant]
        type MaxMaintainerSize: Get<u32>;
        #[pallet::constant]
        type ExpiryBlocks: Get<u32>;
        #[pallet::constant]
        type RevocationThreshold: Get<u32>;
        /// Maximum number of blocks allowed between heartbeats before a domain is considered inactive
        #[pallet::constant]
        type HeartbeatInterval: Get<u32>;
        /// The type of crypto to use for signing transactions from the offchain worker
        type AuthorityId: AppCrypto<<Self as SigningTypes>::Public, <Self as SigningTypes>::Signature>;
    }

    // Type aliases
    type DomainName<T> = BoundedVec<u8, <T as Config>::MaxDomainLength>;
    type ChainSpec<T> = BoundedVec<u8, <T as Config>::MaxChainSpecSize>;
    type Maintainer<T> = BoundedVec<u8, <T as Config>::MaxMaintainerSize>;

    #[derive(Debug, Encode, Decode, Clone, PartialEq, Default, TypeInfo)]
    pub struct DomainInfo<T: Config> {
        pub creator: T::AccountId,     // Account that created the domain
        pub chain_spec: ChainSpec<T>,  // Blockchain chain specification
        pub maintainer: Maintainer<T>, // Maintainer's details (can be a human-readable identifier)
        pub available: bool,           // Indicates if the domain is active
        pub last_heartbeat: BlockNumberFor<T>, // Last block when maintainer sent a heartbeat
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
                last_heartbeat: frame_system::Pallet::<T>::block_number(),
            }
        }
    }

    // Maps domain names to domain metadata
    #[pallet::storage]
    #[pallet::getter(fn domain_map)]
    pub(super) type DomainMap<T: Config> =
        StorageMap<_, Blake2_128Concat, DomainName<T>, DomainInfo<T>, OptionQuery>;

    // Maps maintainers to domains
    #[pallet::storage]
    #[pallet::getter(fn maintainer_map)]
    pub(super) type MaintainerMap<T: Config> = 
        StorageMap<_, Blake2_128Concat, Maintainer<T>, DomainName<T>, OptionQuery>;

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

    // Tracks missed heartbeat observations by different nodes
    #[pallet::storage]
    #[pallet::getter(fn heartbeat_observations)]
    pub(super) type HeartbeatObservations<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        DomainName<T>,                      // Domain name with missed heartbeat
        Blake2_128Concat,
        T::AccountId,                       // Observer account
        BlockNumberFor<T>,                  // Block when observation was made
        OptionQuery,
    >;

    // Counts total missed heartbeat observations for each domain
    #[pallet::storage]
    #[pallet::getter(fn heartbeat_observation_count)]
    pub(super) type HeartbeatObservationCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        DomainName<T>,                      // Domain name
        u32,                                // Number of unique observers
        ValueQuery,
    >;

    // Storage for tracking domains that need heartbeat monitoring
    #[pallet::storage]
    #[pallet::getter(fn active_domains)]
    pub type ActiveDomains<T: Config> = StorageMap<_, Blake2_128Concat, DomainName<T>, BlockNumberFor<T>, OptionQuery>;

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
        // Event for domain heartbeat
        DomainHeartbeat {
            domain_name: DomainName<T>, // Domain name with heartbeat
            maintainer: T::AccountId,   // Maintainer account
            block_number: BlockNumberFor<T>, // Block when heartbeat was received
        },
        // Event for domain expiry due to missed heartbeats
        DomainExpiredHeartbeat {
            domain_name: DomainName<T>, // Domain name that expired
            maintainer: T::AccountId,   // Maintainer account
            last_heartbeat: BlockNumberFor<T>, // Last block when heartbeat was received
        },
        // Event for missed heartbeat observation
        HeartbeatMissedObserved {
            domain_name: DomainName<T>, // Domain name with missed heartbeat
            observer: T::AccountId,     // Observer account
            count: u32,                 // Current observation count
        },
        // Event for domain revocation due to threshold of missed heartbeat observations
        DomainRevokedByConsensus {
            domain_name: DomainName<T>, // Domain name being revoked
            observation_count: u32,     // Number of observations that triggered revocation
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
        DuplicatePeerObservation,
        InvalidUnsignedTransaction,
        NotDomainMaintainer,
        HeartbeatTooSoon,
        DuplicateHeartbeatObservation,
    }

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			// The offchain worker checks for missed heartbeats and submits observations
			log::info!("Running offchain worker at block: {:?}", block_number);
			
			// Check for domains with missed heartbeats every 10 blocks to avoid excessive processing
			if block_number % 10u32.into() != 0u32.into() {
				return;
			}
			
			// Get current block number
			let current_block = frame_system::Pallet::<T>::block_number();
			
			// Iterate through active domains only
			for (domain_name, last_active_block) in <ActiveDomains<T>>::iter() {
				let domain_info = match <DomainMap<T>>::get(&domain_name) {
					Some(info) if info.available => info,
					_ => continue, // Skip if domain doesn't exist or is unavailable
				};
				
				// Check if heartbeat interval has passed
				let heartbeat_deadline = last_active_block.saturating_add(T::HeartbeatInterval::get().into());
				if current_block > heartbeat_deadline {
					// Domain has missed heartbeats, submit an observation
					log::info!(
						"Domain {:?} has missed heartbeat. Last heartbeat at block: {:?}",
						domain_name,
						domain_info.last_heartbeat
					);
					
					// Submit a transaction to report the missed heartbeat
					let call = Call::report_missed_heartbeat { domain_name: domain_name.clone() };
					
					// Use a signed transaction for accountability
					let signer = Signer::<T, T::AuthorityId>::any_account();
					
					if let Some((acc, res)) = signer.send_signed_transaction(|_account| call.clone()) {
						match res {
							Ok(()) => log::info!(
								"[{:?}]: Submitted missed heartbeat observation successfully", 
								acc.id
							),
							Err(e) => log::error!(
								"[{:?}]: Failed to submit missed heartbeat observation: {:?}", 
								acc.id, 
								e
							),
						}
					} else {
						log::error!("No local account available to submit observation");
					}
				}
			}
		}
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
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure domain doesn't already exist
            ensure!(
                !<DomainMap<T>>::contains_key(&domain_name),
                Error::<T>::DomainAlreadyExists
            );

            // Calculate expiry block
            let current_block = frame_system::Pallet::<T>::block_number();
            let expiry_block = current_block.saturating_add(T::ExpiryBlocks::get().into());

            // Create domain info
            let domain_info = DomainInfo::new(who.clone(), chain_spec, maintainer.clone(), true);

            // Store domain info and expiry
            <DomainMap<T>>::insert(&domain_name, domain_info);
            <DomainExpiry<T>>::insert(&domain_name, expiry_block);
            <MaintainerMap<T>>::insert(&maintainer, domain_name.clone());
            <ActiveDomains<T>>::insert(&domain_name, current_block);

            // Emit event
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

            // Ensure domain exists and is not expired
            Self::ensure_not_expired(&domain_name)?;

            // Get domain info
            let mut domain_info = <DomainMap<T>>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;

            // Ensure caller is domain owner
            ensure!(domain_info.creator == who, Error::<T>::InvalidOwnerId);

            // Update chain spec
            domain_info.chain_spec = chain_spec;

            // Update domain info
            <DomainMap<T>>::insert(&domain_name, domain_info);

            // Emit event
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

            // Ensure domain exists
            let domain_info = <DomainMap<T>>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;

            // Ensure caller is domain owner
            ensure!(domain_info.creator == who, Error::<T>::InvalidOwnerId);

            // Remove domain mappings
            <DomainMap<T>>::remove(&domain_name);
            <DomainExpiry<T>>::remove(&domain_name);
            <MaintainerMap<T>>::remove(&domain_info.maintainer);
            <ActiveDomains<T>>::remove(&domain_name);

            // Emit event
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

        // Send a heartbeat to prove domain maintainer is online
        #[pallet::call_index(6)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_send_heartbeat())]
        pub fn send_heartbeat(
            origin: OriginFor<T>,
            domain_name: DomainName<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure domain exists
            let mut domain_info = <DomainMap<T>>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;

            // Ensure caller is domain owner
            ensure!(domain_info.creator == who, Error::<T>::NotDomainMaintainer);

            // Get current block number
            let current_block = frame_system::Pallet::<T>::block_number();
            
            // Ensure heartbeat isn't sent too frequently
            let min_interval = T::HeartbeatInterval::get() / 2; 
            ensure!(
                current_block > domain_info.last_heartbeat.saturating_add(min_interval.into()),
                Error::<T>::HeartbeatTooSoon
            );

            // Update last heartbeat time
            domain_info.last_heartbeat = current_block;
            
            // Extend domain expiry
            let new_expiry = current_block.saturating_add(T::ExpiryBlocks::get().into());
            <DomainExpiry<T>>::insert(&domain_name, new_expiry);
            
            // Update domain info
            <DomainMap<T>>::insert(&domain_name, domain_info);
            <ActiveDomains<T>>::insert(&domain_name, current_block);

            // Emit heartbeat event
            Self::deposit_event(Event::DomainHeartbeat {
                domain_name,
                maintainer: who,
                block_number: current_block,
            });

            Ok(())
        }

        // Report a domain with missed heartbeat
        #[pallet::call_index(7)]
        #[pallet::weight(<SubstrateWeight<T> as WeightInfo>::benchmark_report_missed_heartbeat())]
        pub fn report_missed_heartbeat(
            origin: OriginFor<T>,
            domain_name: DomainName<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure domain exists
            let domain_info = <DomainMap<T>>::get(&domain_name).ok_or(Error::<T>::DomainNotFound)?;
            
            // Get current block number
            let current_block = frame_system::Pallet::<T>::block_number();
            
            // Check if heartbeat is actually missed
            let heartbeat_deadline = domain_info.last_heartbeat.saturating_add(T::HeartbeatInterval::get().into());
            ensure!(current_block > heartbeat_deadline, Error::<T>::HeartbeatTooSoon);
            
            // Check if observation already exists
            ensure!(
                !HeartbeatObservations::<T>::contains_key(&domain_name, &who),
                Error::<T>::DuplicateHeartbeatObservation
            );

            // Insert heartbeat observation
            HeartbeatObservations::<T>::insert(&domain_name, &who, current_block);

            // Update heartbeat observation count
            let mut count = HeartbeatObservationCount::<T>::get(&domain_name);
            count = count.saturating_add(1);
            HeartbeatObservationCount::<T>::insert(&domain_name, count);

            // Emit heartbeat missed observed event
            Self::deposit_event(Event::HeartbeatMissedObserved {
                domain_name: domain_name.clone(),
                observer: who,
                count,
            });

            // Check if revocation threshold is reached
            if count >= T::RevocationThreshold::get() {
                // Mark domain as unavailable
                let mut updated_info = domain_info.clone();
                updated_info.available = false;
                <DomainMap<T>>::insert(&domain_name, updated_info);
                
                // Emit domain revoked by consensus event
                Self::deposit_event(Event::DomainRevokedByConsensus {
                    domain_name: domain_name.clone(),
                    observation_count: count,
                });
                
                log::info!(
                    "Domain {:?} revoked by consensus with {:?} observations",
                    domain_name,
                    count
                );
            }

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // Ensure that the domain has not expired
        pub(super) fn ensure_not_expired(domain_name: &DomainName<T>) -> DispatchResult {
            let current_block = frame_system::Pallet::<T>::block_number();
            
            // Check if domain exists
            let domain_info = <DomainMap<T>>::get(domain_name).ok_or(Error::<T>::DomainNotFound)?;
            
            // Check if domain is available
            ensure!(domain_info.available, Error::<T>::DomainExpired);
            
            // Check if domain has expired based on expiry block
            let expiry_block = <DomainExpiry<T>>::get(domain_name).ok_or(Error::<T>::DomainExpired)?;
            ensure!(current_block <= expiry_block, Error::<T>::DomainExpired);
            
            // Check if heartbeat is still valid
            let heartbeat_deadline = domain_info.last_heartbeat.saturating_add(T::HeartbeatInterval::get().into());
            if current_block > heartbeat_deadline {
                // Domain has missed heartbeats, mark it as expired
                let mut updated_info = domain_info.clone();
                updated_info.available = false;
                <DomainMap<T>>::insert(domain_name, updated_info);
                
                // Emit domain expired event
                Self::deposit_event(Event::DomainExpiredHeartbeat {
                    domain_name: domain_name.clone(),
                    maintainer: domain_info.creator,
                    last_heartbeat: domain_info.last_heartbeat,
                });
                
                return Err(Error::<T>::DomainExpired.into());
            }
            
            Ok(())
        }
    }
}
