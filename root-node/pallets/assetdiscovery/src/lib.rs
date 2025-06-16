//! # Asset Discovery Pallet
//! 
//! ## Overview
//! 
//! The assetdiscovery pallet provides functionality for managing assets and their providers.
//! It allows users to register assets and link them to domains.
//! 
//! ## Features
//! - **Register assets**: Allows users to register assets with their unique identifiers and metadata.
//! - **Link assets to domains**: Enables users to link assets to specific domains.
//! 
//! ## Storage
//! - **`AssetProviders`**: Maps asset hashes to their providers (domains that serve the asset)
//!   - Key: Asset hash (ByteVector)
//!   - Value: List of provider domains (ProviderList)
//! - **`ProviderAssets`**: Maps domain names to the assets they provide
//!   - Key: Domain name (ByteVector)
//!   - Value: List of assets provided (AssetList)
//! - **`LastProcessedDomain`**: Tracks the last domain processed by the offchain worker for batch operations
//!   - Value: Domain name (ByteVector)
//! - **`PendingRequests`**: Stores pending asset registration requests awaiting verification
//!   - Key: Composite key of asset hash + domain (ByteVector)
//!   - Value: Pending request details (PendingRequest)
//! - **`RevocationVotes`**: Tracks votes for domain revocation, mapping domains to the accounts that voted
//!   - Key: Domain name (ByteVector)
//!   - Value: List of accounts that voted for revocation (ItemVector<AccountId>)
//! 	
//! ## Events
//! - **`AssetRegistered`**: Triggered when an asset is successfully registered.
//! - **`ProviderAdded`**: Triggered when a provider is added to an asset.
//! - **`ProviderRemoved`**: Triggered when a provider is removed from an asset.
//! - **`AssetRevoked`**: Triggered when an asset is revoked.
//! - **`ProviderRevoked`**: Triggered when a provider is revoked.
//! - **`ExpiredRequestsRemoved`**: Triggered when expired requests are removed.
//! - **`RevocationVoteSubmitted`**: Triggered when a revocation vote is submitted.
//! 
//! ## Errors
//! - **`ByteVectorTooLarge`**: The provided byte vector exceeds the maximum allowed size.
//! - **`DomainDoesNotExist`**: The specified domain does not exist.
//! - **`AlreadyVoted`**: The caller has already voted for revocation.
//! - **`TooManyVotes`**: The number of votes has exceeded the threshold.
//! 
//! ## Usage
//! 1. **Register an Asset**: Call `register_asset_for_domain` with a domain name, asset hash, and timestamp.
//! 2. **Submit a Verified Domain**: Call `submit_verified_domain` with the requester's account ID and the pending request.
//! 3. **Vote for Domain Revocation**: Call `vote_for_domain_revocation` with the domain name.
//! 4. **Remove Expired Requests**: Call `remove_expired_pending_requests`.
//! 
//! ## Note
//! Run `cargo doc --package pallet-assetdiscovery --open` to view the complete documentation for this pallet.

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

/// The authority ID type for the assetdiscovery pallet
pub type AuthorityId = sp_runtime::app_crypto::sr25519::Public;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use codec::Encode;
	use core::fmt;
	use core::str;
	use frame_support::{pallet_prelude::*, Deserialize, Serialize};
	use frame_system::{
		offchain::{
			AppCrypto, CreateSignedTransaction, SendSignedTransaction, 
			Signer
		},
		pallet_prelude::{BlockNumberFor, *},
	};
	use scale_info::prelude::{format, string::String, vec, vec::Vec};
	use sp_core::{offchain::Duration, U256};
	use sp_runtime::offchain::http;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config + scale_info::TypeInfo {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
		type PalletRootDNS: pallet_rootdns::Config;
		/// Maximum size for domain names and other byte vectors
		#[pallet::constant]
		type MaxByteLength: Get<u32>;
		/// Maximum number of items in a vector
		#[pallet::constant]
		type MaxItems: Get<u32>;
		/// The type of crypto to use for signing transactions from the offchain worker
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
		/// The lifetime of a pending request in blocks
		#[pallet::constant]
		type RequestLifetime: Get<u32>;
		/// The number of votes required to revoke a domain
		#[pallet::constant]
		type RevocationThreshold: Get<u32>;
	}

	const TLD_MODULE_PREFIX: &[u8] = b"TldModule";
	const TLD_STORAGE_PREFIX: &[u8] = b"DomainMap";
	const DOMAIN_BATCH_SIZE: usize = 10;

	// Type aliases for BoundedVec
	type ByteVector<T> = BoundedVec<u8, <T as Config>::MaxByteLength>;
	type ItemVector<T, Item> = BoundedVec<Item, <T as Config>::MaxItems>;

	/// Represents a pending request for asset registration
	#[derive(Encode, Decode, Clone, PartialEq, Default, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct PendingRequest<T: Config> {
		pub requester: ByteVector<T>,
		pub domain: ByteVector<T>,
		pub asset_hash: ByteVector<T>,
		pub timestamp: U256,
	}

	impl<T: Config> fmt::Debug for PendingRequest<T> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("PendingRequest")
				.field("requester", &self.requester)
				.field("domain", &self.domain)
				.field("asset_hash", &self.asset_hash)
				.field("timestamp", &self.timestamp)
				.finish()
		}
	}

	/// List of providers for a given asset
	#[derive(Encode, Decode, Clone, PartialEq, Default, TypeInfo)]
	pub struct ProviderList<T: Config> {
		pub providers: ItemVector<T, ByteVector<T>>,
	}

	/// List of assets associated with a provider
	#[derive(Encode, Decode, Clone, PartialEq, Default, TypeInfo)]
	pub struct AssetList<T: Config> {
		pub assets: ItemVector<T, ByteVector<T>>,
	}

	#[pallet::storage]
	#[pallet::getter(fn asset_providers)]
	pub(super) type AssetProviders<T: Config> =
		StorageMap<_, Blake2_128Concat, 
			ByteVector<T>,  // Asset hash
			ProviderList<T>,  // Provider domains
			OptionQuery
		>;

	#[pallet::storage]
	#[pallet::getter(fn provider_assets)]
	pub(super) type ProviderAssets<T: Config> =
		StorageMap<_, Blake2_128Concat, 
			ByteVector<T>,  // Domain
			AssetList<T>,  // Assets
			OptionQuery
		>;

	#[pallet::storage]
	#[pallet::getter(fn last_processed_domain)]
	pub(super) type LastProcessedDomain<T: Config> = 
		StorageValue<_, 
			ByteVector<T>,  // Last processed domain
			ValueQuery
		>;

	#[pallet::storage]
	#[pallet::getter(fn pending_requests)]
	pub(super) type PendingRequests<T: Config> =
		StorageMap<_, Blake2_128Concat, 
			ByteVector<T>,  // Composite key (assethash+domain)
			PendingRequest<T>,  // Request details
			OptionQuery
		>;

	#[pallet::storage]
	#[pallet::getter(fn revocation_votes)]
	pub type RevocationVotes<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ByteVector<T>,  // Domain
		ItemVector<T, T::AccountId>,  // Voters
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Emitted when a domain validation is requested
		DomainValidationRequested(
			T::AccountId,  // Account that requested the validation
			PendingRequest<T>,  // Details of the pending request
		),
		// Emitted when an asset is registered for a domain
		AssetRegisteredForDomain(
			ByteVector<T>,  // Domain name
			ByteVector<T>,  // Asset hash
			U256,  // Timestamp of registration
		),
		// Emitted when a domain is revoked as an asset provider
		AssetProviderRevoked(
			ByteVector<T>,  // Domain that was revoked
		),
		// Emitted when expired requests are removed during cleanup
		ExpiredRequestsRemoved,
		// Emitted when a vote for domain revocation is submitted
		RevocationVoteSubmitted(
			T::AccountId,  // Account that submitted the vote
			ByteVector<T>,  // Domain being voted for revocation
		),
	}

	#[pallet::error]
	pub enum Error<T> {
		RequestDoesNotExist,  // The pending request does not exist
		ByteVectorTooLarge,  // The provided byte vector exceeds the maximum allowed size
		DomainDoesNotExist,  // The specified domain does not exist
		AlreadyVoted,  // The account has already voted for domain revocation
		TooManyVotes,  // The number of votes has reached the maximum allowed
	}

	fn extract_tld(domain: &[u8]) -> Option<&[u8]> {
		if let Some(pos) = domain.iter().rposition(|&b| b == b'.') {
			Some(&domain[pos + 1..])
		} else {
			None
		}
	}

	#[derive(Debug, Deserialize)]
	struct Chainspec {
		#[serde(rename = "bootNodes")]
		boot_nodes: Vec<String>,
	}

	fn fetch_json_from_url(url: &str) -> Result<Chainspec, &'static str> {
		let request = http::Request::get(url);

		let response = match request.send() {
			Ok(resp) => {
				let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(5000));
				match resp.try_wait(deadline) {
					Ok(r) => match r {
						Ok(response) => response,
						Err(err) => {
							log::error!("Error sending request: {:?}", err);
							return Err("Failed to get response");
						},
					},
					Err(_) => {
						log::error!("Failed to wait for response");
						return Err("Failed to wait for response");
					},
				}
			},
			Err(_) => {
				log::error!("Failed to send HTTP request");
				return Err("Failed to send HTTP request");
			},
		};

		if response.code != 200 {
			return Err("Unexpected response code");
		}

		let mut body = Vec::new();
		response.body().for_each(|chunk| body.push(chunk));

		let body_str = str::from_utf8(&body).map_err(|_| "Invalid UTF-8")?;
		let chainspec: Chainspec =
			serde_json::from_str(body_str).map_err(|_| "Failed to parse JSON")?;

		Ok(chainspec)
	}

	// Helper function to convert a ByteVector to a Vec<u8> for storage key calculation
	fn blake2_128_concat_storage_key<T: Config>(domain: &ByteVector<T>) -> Vec<u8> {
		let hash = sp_core::hashing::blake2_128(domain.as_slice());
		let mut result = Vec::with_capacity(hash.len() + domain.len());
		result.extend_from_slice(&hash);
		result.extend(domain.clone());
		result
	}

	fn extract_rpc_endpoint(multiaddr: String) -> String {
		let parts: Vec<&str> = multiaddr.split('/').collect();
		let ip = if parts.len() > 2 { parts[2] } else { "" };
		let port = if parts.len() > 4 { parts[4] } else { "" };
		format!("http://{}:{}", ip, port)
	}

	#[derive(Serialize, Deserialize, Debug)]
	struct RPCResponse {
		jsonrpc: String,
		result: String,
		id: u32,
	}

	impl<T: Config> Pallet<T> {
		/// Fetches pending requests that have not expired
		fn fetch_pending_requests(
			batch_size: usize,
			current_time: U256,
		) -> Vec<(ByteVector<T>, PendingRequest<T>)> {
			PendingRequests::<T>::iter()
				.filter_map(|(account, data)| {
					if current_time <= data.timestamp {
						Some((
							account,
							PendingRequest {
								requester: data.requester,
								domain: data.domain,
								asset_hash: data.asset_hash,
								timestamp: data.timestamp,
							},
						))
					} else {
						None
					}
				})
				.take(batch_size)
				.collect()
		}

		/// Queries the TLD network to verify if a domain is available
		fn query_tld_network(domain: ByteVector<T>) -> bool {
			// Extract the TLD from the domain
			let tld = match extract_tld(&domain) {
				Some(tld) => tld,
				None => {
					log::error!("Failed to extract TLD from domain: {:?}", domain);
					return false;
				},
			};

			// Get the chainspec for the TLD
			let tld_info = match pallet_rootdns::Pallet::<T::PalletRootDNS>::get_chainspec_for_tld(tld) {
				Some(tld_info) => tld_info,
				None => {
					log::error!("No chainspec found for TLD: {:?}", tld);
					return false;
				},
			};

			// Fetch and parse the chainspec JSON
			let chainspec_str = match str::from_utf8(&tld_info.chain_spec) {
				Ok(s) => s,
				Err(e) => {
					log::error!("Invalid UTF-8 in chainspec: {:?}", e);
					return false;
				},
			};

			let spec = match fetch_json_from_url(chainspec_str) {
				Ok(chainspec) => chainspec,
				Err(e) => {
					log::error!("Failed to fetch chainspec JSON: {:?}", e);
					return false;
				},
			};

			// Calculate storage key
			let storage_key = Self::calculate_domain_storage_key(&domain);
			let key_hex = hex::encode(storage_key);
			
			// Prepare the request body template
			let request_body_template = format!(
				r#"{{
					"id": 1,
					"jsonrpc": "2.0",
					"method": "state_getStorage",
					"params": ["0x{}"]
				}}"#,
				key_hex
			);
			
			// Try each bootnode until one succeeds
			for bootnode in &spec.boot_nodes {
				let rpc_endpoint = extract_rpc_endpoint(bootnode.clone());
				log::info!("Trying bootnode with RPC endpoint: {}", rpc_endpoint);
				
				// Send the RPC request
				let request = match http::Request::post(&rpc_endpoint, vec![request_body_template.clone()])
					.add_header("content-type", "application/json")
					.send() {
						Ok(req) => req,
						Err(e) => {
							log::warn!("Failed to send HTTP request to {}: {:?}", rpc_endpoint, e);
							continue; // Try the next bootnode
						},
					};

				// Wait for the response with a timeout
				let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(5000));
				let response = match request.try_wait(deadline) {
					Ok(Ok(res)) => res,
					Ok(Err(e)) => {
						log::warn!("HTTP request to {} failed: {:?}", rpc_endpoint, e);
						continue; // Try the next bootnode
					},
					Err(_) => {
						log::warn!("Request to {} timed out", rpc_endpoint);
						continue; // Try the next bootnode
					},
				};

				// Process the response
				if response.code != 200 {
					log::warn!("Request to {} failed with code: {}", rpc_endpoint, response.code);
					continue; // Try the next bootnode
				}

				// Parse the response body
				let body = response.body().collect::<Vec<_>>();
				let body_str = match String::from_utf8(body) {
					Ok(s) => s,
					Err(e) => {
						log::warn!("Invalid UTF-8 in response from {}: {:?}", rpc_endpoint, e);
						continue; // Try the next bootnode
					},
				};

				let decoded_body = match serde_json::from_str::<RPCResponse>(&body_str) {
					Ok(body) => body,
					Err(e) => {
						log::warn!("Failed to parse JSON response from {}: {:?}", rpc_endpoint, e);
						continue; // Try the next bootnode
					},
				};

				// Decode the result
				if decoded_body.result.is_empty() {
					log::warn!("Empty result in response from {}", rpc_endpoint);
					continue; // Try the next bootnode
				}

				let result_bytes = match hex::decode(decoded_body.result.trim_start_matches("0x")) {
					Ok(bytes) => bytes,
					Err(e) => {
						log::warn!("Failed to decode hex result from {}: {:?}", rpc_endpoint, e);
						continue; // Try the next bootnode
					},
				};

				// Extract the 'available' field from the decoded data
				let mut decoder = &result_bytes[..];
				
				// Skip unnecessary fields
				if <Vec<u8> as codec::Decode>::decode(&mut decoder).is_err() {
					log::warn!("Failed to decode creator field from {}", rpc_endpoint);
					continue; // Try the next bootnode
				}
				
				if <Vec<u8> as codec::Decode>::decode(&mut decoder).is_err() {
					log::warn!("Failed to decode chain_spec field from {}", rpc_endpoint);
					continue; // Try the next bootnode
				}
				
				if <Vec<u8> as codec::Decode>::decode(&mut decoder).is_err() {
					log::warn!("Failed to decode maintainer field from {}", rpc_endpoint);
					continue; // Try the next bootnode
				}
				
				// Decode the `available` field 
				let available = match <bool as codec::Decode>::decode(&mut decoder) {
					Ok(available) => available,
					Err(e) => {
						log::warn!("Failed to decode available field from {}: {:?}", rpc_endpoint, e);
						continue; // Try the next bootnode
					},
				};
				
				// Successfully processed a response, return the result
				log::info!("Successfully queried bootnode at {}", rpc_endpoint);
				return !available;
			}
			
			// If we've tried all bootnodes and none succeeded, log an error and return false
			log::error!("Failed to connect to any bootnode for TLD: {:?}", tld);
			false
		}

		/// Calculates the storage key for a domain in the TLD network
		fn calculate_domain_storage_key(domain: &ByteVector<T>) -> Vec<u8> {
			let mut storage_key = Vec::new();
			
			// Add the TLD module prefix
			storage_key.extend(sp_core::hashing::twox_128(TLD_MODULE_PREFIX));
			
			// Add the TLD storage prefix
			storage_key.extend(sp_core::hashing::twox_128(TLD_STORAGE_PREFIX));
			
			// Add the domain-specific part of the key
			storage_key.extend(blake2_128_concat_storage_key::<T>(domain));
			
			storage_key
		}

		/// Retrieves a batch of domains for processing
		fn get_domain_batch(batch_size: usize) -> Result<Vec<ByteVector<T>>, &'static str> {
			let mut keys = Vec::new();
			let mut key_count = 0;
			let last_key = LastProcessedDomain::<T>::get();

			let iter = match last_key.is_empty() {
				true => ProviderAssets::<T>::iter(),
				false => ProviderAssets::<T>::iter_from(last_key.to_vec()),
			};

			for (key, _value) in iter {
				if key_count >= batch_size {
					break;
				}
				keys.push(key);
				key_count += 1;
			}

			if let Some(last_key) = keys.last() {
				LastProcessedDomain::<T>::put(last_key.clone());
			}

			Ok(keys)
		}

		/// Processes a batch of revoked domains
		fn process_revoked_domains(domains: Vec<ByteVector<T>>, signer: &Signer<T, T::AuthorityId>) {
			let mut revoked_domains = Vec::new();

			for domain in domains {
				let is_valid = Self::query_tld_network(domain.clone());
				if !is_valid {
					revoked_domains.push(domain.clone());
				}
			}

			if revoked_domains.is_empty() {
				return;
			}

			// Instead of directly revoking domains, submit votes for revocation
			for domain in revoked_domains {
				let call = Call::vote_for_domain_revocation { domain };
				let _ = signer.send_signed_transaction(|_account| call.clone());
			}
		}

		/// Processes expired pending requests
		fn process_expired_requests(current_time: U256, signer: &Signer<T, T::AuthorityId>) {
			if current_time % 10 == 0.into() {
				log::info!("Cleaning up expired requests");
				let call = Call::remove_expired_pending_requests { };
				let _ = signer.send_signed_transaction(|_account| call.clone());
			}
		}

		/// Processes domain verification by sending a signed transaction
		fn process_domain_verification(
			key: ByteVector<T>,
			pending_request: PendingRequest<T>,
			signer: &Signer<T, T::AuthorityId>,
		) {
			let call = Call::submit_verified_domain { key, pending_request };
			let _ = signer.send_signed_transaction(|_account| call.clone());
		}

		/// Verifies if a domain is associated with a given asset hash
		fn verify_domain_asset_hash(domain: ByteVector<T>, _asset_hash: ByteVector<T>) -> bool {
			let tld = match extract_tld(&domain) {
				Some(tld) => tld,
				None => return false,
			};

			let tld_info =
				match pallet_rootdns::Pallet::<T::PalletRootDNS>::get_chainspec_for_tld(tld) {
					Some(tld_info) => tld_info,
					None => return false,
				};

			let spec = match fetch_json_from_url(str::from_utf8(&tld_info.chain_spec).unwrap()) {
				Ok(chainspec) => chainspec,
				Err(_) => return false,
			};

			if spec.boot_nodes.is_empty() {
				log::error!("No boot nodes found in chainspec");
				return false;
			}

			let multiaddr = spec.boot_nodes[0].clone(); // TODO: just take the first bootnode for now
			let rpc_endpoint = extract_rpc_endpoint(multiaddr);
			// Storage key calculation
			let mut storage_key = vec![];
			storage_key.extend(sp_core::hashing::twox_128(TLD_MODULE_PREFIX));
			storage_key.extend(sp_core::hashing::twox_128(TLD_STORAGE_PREFIX));
			// Convert ByteVector<T> to Vec<u8> for encoding
			let domain_vec = domain.to_vec();
			let bounded_domain: ByteVector<T> = domain_vec.try_into().map_err(|_| {
				log::error!("Failed to convert domain to ByteVector");
				return false;
			}).unwrap_or_else(|_| ByteVector::<T>::default());
			
			storage_key.extend(blake2_128_concat_storage_key::<T>(&bounded_domain));

			let key_hex = hex::encode(storage_key);

			const REQUEST_BODY: &str = r#"
            {
                "id":1,
                "jsonrpc":"2.0",
                "method": "state_getStorage",
				"params": ["0x{}"]
            }"#;

			let request_body = REQUEST_BODY.replace("{}", &key_hex);

			let req = http::Request::post(&rpc_endpoint, vec![request_body])
				.add_header("content-type", "application/json")
				.send()
				.unwrap();

			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(5000));
			let res = match req.try_wait(deadline) {
				Ok(r) => match r {
					Ok(response) => response,
					Err(err) => {
						log::error!("Error sending request: {:?}", err);
						return false;
					},
				},
				Err(_) => {
					log::error!("Request timed out");
					return false;
				},
			};

			if res.code == 200 {
				let body = res.body().collect::<Vec<_>>();
				let body_str = String::from_utf8(body).unwrap();

				let decoded_body = serde_json::from_str::<RPCResponse>(&body_str).unwrap();
				let result_bytes =
					hex::decode(decoded_body.result.trim_start_matches("0x")).unwrap();

				let mut decoder = &result_bytes[..];
				
				// Skip the creator field (AccountId)
				let _ = <Vec<u8> as codec::Decode>::decode(&mut decoder)
					.expect("Failed to decode creator field");
					
				// Skip the chain_spec field (BoundedVec)
				let _ = <Vec<u8> as codec::Decode>::decode(&mut decoder)
					.expect("Failed to decode chain_spec field");
					
				// Skip the maintainer field (BoundedVec)
				let _ = <Vec<u8> as codec::Decode>::decode(&mut decoder)
					.expect("Failed to decode maintainer field");
					
				// Now decode the available field (bool)
				let available = <bool as codec::Decode>::decode(&mut decoder)
					.expect("Failed to decode available field");
				
				return !available;
			} else {
				log::info!("Request failed with code: {}", res.code);
			}
			false
		}

		/// Executes the revocation of a domain and removes it from all associated assets
		fn execute_domain_revocation(domain: ByteVector<T>) -> DispatchResult {
			// Get the provider assets
			if let Some(provider_assets) = ProviderAssets::<T>::get(&domain) {
				// For each asset, remove the domain from its providers
				for asset_hash in provider_assets.assets.iter() {
					if let Some(asset_providers) = AssetProviders::<T>::get(asset_hash) {
						// Create a new provider list without the current domain
						let mut new_providers = asset_providers.clone();
						new_providers.providers.retain(|p| p != &domain);
						
						// Update the asset providers
						AssetProviders::<T>::insert(
							asset_hash.clone(),
							ProviderList { providers: new_providers.providers },
						);
					}
				}
				
				// Remove the provider assets
				ProviderAssets::<T>::remove(&domain);
				
				// Clear the revocation votes
				RevocationVotes::<T>::remove(&domain);
				
				// Emit event
				Self::deposit_event(Event::AssetProviderRevoked(domain));
				
				Ok(())
			} else {
				Err(Error::<T>::DomainDoesNotExist.into())
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			log::info!("Offchain worker started at block: {:?}", block_number);

			// Create a signer for sending transactions
			let signer = Signer::<T, T::AuthorityId>::any_account();
			if !signer.can_sign() {
				log::error!("No local accounts available. Consider adding one via --alice or --bob.");
				return;
			}

			// Get the current block number
			let current_time: U256 = block_number.into();

			// Process expired requests
			Self::process_expired_requests(current_time, &signer);

			let pending_requests = Self::fetch_pending_requests(DOMAIN_BATCH_SIZE, current_time);
			
			for (key, pending_request) in pending_requests {
				let domain = pending_request.domain.clone();
				let asset_hash = pending_request.asset_hash.clone();

				// Check if the domain is valid and has the correct asset hash
				if Self::verify_domain_asset_hash(domain.clone(), asset_hash.clone()) {
					log::info!("Domain verified: {:?}", domain);
					Self::process_domain_verification(key, pending_request, &signer);
				}
			}

			// Process revoked domains
			if let Ok(domains) = Self::get_domain_batch(DOMAIN_BATCH_SIZE) {
				Self::process_revoked_domains(domains, &signer);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submits a verified domain and registers it in the system
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn submit_verified_domain(
			origin: OriginFor<T>,
			key: ByteVector<T>,
			pending_request: PendingRequest<T>,
		) -> DispatchResult {
			// Allow both signed and root origin for this call
			let _who = ensure_signed_or_root(origin)?;

			// Check if the pending request exists
			ensure!(<PendingRequests<T>>::contains_key(&key), Error::<T>::RequestDoesNotExist);

			PendingRequests::<T>::remove(key);

			match AssetProviders::<T>::get(&pending_request.asset_hash) {
				Some(provider_list) => {
					let mut providers = provider_list.providers.clone();
					providers.try_push(pending_request.domain.clone()).map_err(|_| Error::<T>::ByteVectorTooLarge)?;
					AssetProviders::<T>::insert(
						pending_request.asset_hash.clone(),
						ProviderList { providers },
					);
				},
				None => {
					// Create a new provider list with a single provider
					let mut providers = ItemVector::<T, ByteVector<T>>::default();
					providers.try_push(pending_request.domain.clone()).map_err(|_| Error::<T>::ByteVectorTooLarge)?;
					AssetProviders::<T>::insert(
						pending_request.asset_hash.clone(),
						ProviderList { providers },
					);
				},
			}

			match ProviderAssets::<T>::get(&pending_request.domain) {
				Some(asset_list) => {
					let mut assets = asset_list.assets.clone();
					assets.try_push(pending_request.asset_hash.clone()).map_err(|_| Error::<T>::ByteVectorTooLarge)?;
					ProviderAssets::<T>::insert(
						pending_request.domain.clone(),
						AssetList { assets },
					);
				},
				None => {
					// Create a new asset list with a single asset
					let mut assets = ItemVector::<T, ByteVector<T>>::default();
					assets.try_push(pending_request.asset_hash.clone()).map_err(|_| Error::<T>::ByteVectorTooLarge)?;
					ProviderAssets::<T>::insert(
						pending_request.domain.clone(),
						AssetList { assets },
					);
				},
			}

			// Emit event
			Self::deposit_event(Event::AssetRegisteredForDomain(
				pending_request.domain,
				pending_request.asset_hash,
				pending_request.timestamp,
			));

			Ok(())
		}

		/// Registers an asset for a domain and creates a pending request
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn register_asset_for_domain(
			origin: OriginFor<T>,
			domain: ByteVector<T>,
			asset_hash: ByteVector<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let lifetime = current_block_number.into() + T::RequestLifetime::get();

			let request = PendingRequest {
				requester: who.clone().encode().try_into().map_err(|_| Error::<T>::ByteVectorTooLarge)?,
				domain: domain.clone(),
				asset_hash: asset_hash.clone(),
				timestamp: lifetime,
			};

			let mut key_bytes = vec![];
			key_bytes.append(&mut asset_hash.clone().to_vec());
			key_bytes.append(&mut domain.clone().to_vec());
			let key: ByteVector<T> = key_bytes.try_into().map_err(|_| Error::<T>::ByteVectorTooLarge)?;

			// Request ocw to validate domain
			PendingRequests::<T>::insert(key.clone(), request.clone());

			// Emit event
			Self::deposit_event(Event::DomainValidationRequested(who, request));

			Ok(())
		}

		/// Cleans up revoked domains by removing them from the system
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn cleanup_revoked_domains(
			origin: OriginFor<T>,
			domains: ItemVector<T, ByteVector<T>>,
		) -> DispatchResult {
			// Allow both signed and root origin for this call
			let _who = ensure_signed_or_root(origin)?;

			for domain in domains {
				if let Some(asset_list) = ProviderAssets::<T>::get(&domain) {
					for asset in asset_list.assets {
						if let Some(provider_list) = AssetProviders::<T>::get(&asset) {
							// Create a new provider list without the current domain
							let new_providers: ItemVector<T, ByteVector<T>> = provider_list.providers
								.iter()
								.filter(|&x| x != &domain)
								.cloned()
								.collect::<Vec<_>>()
								.try_into()
								.map_err(|_| Error::<T>::ByteVectorTooLarge)?;
							
							AssetProviders::<T>::insert(
								asset.clone(),
								ProviderList { providers: new_providers },
							);
						}
					}
					ProviderAssets::<T>::remove(&domain);
					Self::deposit_event(Event::AssetProviderRevoked(domain));
				}
			}

			Ok(())
		}

		/// Removes expired pending requests from the system
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn remove_expired_pending_requests(
			origin: OriginFor<T>
		) -> DispatchResult {
			// Allow both signed and root origin for this call
			let _who = ensure_signed_or_root(origin)?;

			// Get current block number
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let current_time = current_block_number.into();

			// Get all pending requests
			let expired_requests: Vec<_> = PendingRequests::<T>::iter()
				.filter(|(_, data)| current_time > data.timestamp)
				.collect();

			// Remove expired requests
			for (key, _) in expired_requests {
				PendingRequests::<T>::remove(key);
			}

			Self::deposit_event(Event::ExpiredRequestsRemoved);

			Ok(())
		}

		/// Submits a vote for domain revocation and executes revocation if threshold is reached
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn vote_for_domain_revocation(
			origin: OriginFor<T>,
			domain: ByteVector<T>,
		) -> DispatchResult {
			// Only allow signed transactions
			let who = ensure_signed(origin)?;
			
			// Check if the domain exists
			ensure!(ProviderAssets::<T>::contains_key(&domain), Error::<T>::DomainDoesNotExist);
			
			// Get the current votes
			let mut votes = RevocationVotes::<T>::get(&domain);
			
			// Check if the account has already voted
			ensure!(!votes.contains(&who), Error::<T>::AlreadyVoted);
			
			// Add the vote
			votes.try_push(who.clone()).map_err(|_| Error::<T>::TooManyVotes)?;
			RevocationVotes::<T>::insert(&domain, votes.clone());
			
			// Emit event
			Self::deposit_event(Event::RevocationVoteSubmitted(who, domain.clone()));
			
			// Check if we've reached the threshold
			if votes.len() as u32 >= T::RevocationThreshold::get() {
				// Execute the revocation
				Self::execute_domain_revocation(domain)?;
			}
			
			Ok(())
		}
	}
}
