//! Benchmarking setup for pallet-assetdiscovery
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as AssetDiscovery;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::U256;
use scale_info::prelude::vec::Vec;
use scale_info::prelude::vec;
use frame_support::BoundedVec;
use crate::pallet::{PendingRequest, AssetList, ProviderList, AssetProviders, ProviderAssets, RevocationVotes, PendingRequests};

// Helper function to create a byte vector of specified length
fn create_byte_vector<T: Config>(length: usize) -> BoundedVec<u8, T::MaxByteLength> {
    let mut data = Vec::with_capacity(length);
    for i in 0..length {
        data.push((i % 256) as u8);
    }
    data.try_into().unwrap_or_default()
}

// Helper function to create a domain name
fn create_domain_name<T: Config>(name: &[u8]) -> BoundedVec<u8, T::MaxByteLength> {
    name.to_vec().try_into().unwrap_or_default()
}

// Helper function to create a pending request
fn create_pending_request<T: Config>(
    requester: BoundedVec<u8, T::MaxByteLength>,
    domain: BoundedVec<u8, T::MaxByteLength>,
    asset_hash: BoundedVec<u8, T::MaxByteLength>
) -> PendingRequest<T> {
    PendingRequest {
        requester,
        domain,
        asset_hash,
        timestamp: U256::from(100u32),
    }
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn submit_verified_domain() {
        let caller: T::AccountId = whitelisted_caller();
        let requester = create_byte_vector::<T>(32);
        let domain = create_domain_name::<T>(b"example.com");
        let asset_hash = create_byte_vector::<T>(32);
        
        // Setup: Add the domain to ProviderAssets to simulate it exists
        let asset_list = AssetList { 
            assets: Vec::new().try_into().unwrap_or_default() 
        };
        ProviderAssets::<T>::insert(&domain, asset_list);
        
        // Create the pending request and add it to storage
        let pending_request = create_pending_request::<T>(
            requester.clone(),
            domain.clone(),
            asset_hash
        );
        
        // Add the pending request to storage using account_id as the key
        PendingRequests::<T>::insert(&requester, pending_request.clone());

        #[extrinsic_call]
        submit_verified_domain(RawOrigin::Signed(caller), requester, pending_request);
    }

    #[benchmark]
    fn register_asset_for_domain() {
        let caller: T::AccountId = whitelisted_caller();
        let domain = create_domain_name::<T>(b"example.com");
        let asset_hash = create_byte_vector::<T>(32);
        
        // Setup: Add the domain to ProviderAssets to simulate it exists
        let asset_list = AssetList { 
            assets: Vec::new().try_into().unwrap_or_default() 
        };
        ProviderAssets::<T>::insert(&domain, asset_list);

        #[extrinsic_call]
        register_asset_for_domain(RawOrigin::Signed(caller), domain, asset_hash);
    }

    #[benchmark]
    fn cleanup_revoked_domains() {
        let caller: T::AccountId = whitelisted_caller();
        let domain = create_domain_name::<T>(b"example.com");
        let domains = vec![domain.clone()];
        
        // Setup: Add the domain to ProviderAssets to simulate it exists
        let asset_hash = create_byte_vector::<T>(32);
        let asset_list = AssetList { 
            assets: vec![asset_hash.clone()].try_into().unwrap_or_default() 
        };
        ProviderAssets::<T>::insert(&domain, asset_list);
        
        // Setup: Add the asset to AssetProviders
        let provider_list = ProviderList {
            providers: vec![domain.clone()].try_into().unwrap_or_default()
        };
        AssetProviders::<T>::insert(&asset_hash, provider_list);

        #[extrinsic_call]
        cleanup_revoked_domains(RawOrigin::Signed(caller), domains.try_into().unwrap_or_default());
    }

    #[benchmark]
    fn remove_expired_pending_requests() {
        let domain = create_domain_name::<T>(b"example.com");
        let requester = create_byte_vector::<T>(32);
        let asset_hash = create_byte_vector::<T>(32);
        
        // Add some expired pending requests
        let pending_request = PendingRequest {
            requester,
            domain: domain.clone(),
            asset_hash,
            timestamp: U256::from(100u32), // Old timestamp to ensure it's expired
        };
        
        PendingRequests::<T>::insert(domain, pending_request);

        #[extrinsic_call]
        remove_expired_pending_requests(RawOrigin::Root);
    }

    #[benchmark]
    fn vote_for_domain_revocation() {
        let caller: T::AccountId = whitelisted_caller();
        let domain = create_domain_name::<T>(b"example.com");
        
        // Add domain to ProviderAssets
        let asset_hash = create_byte_vector::<T>(32);
        let asset_list = AssetList { 
            assets: vec![asset_hash.clone()].try_into().unwrap_or_default() 
        };
        ProviderAssets::<T>::insert(&domain, asset_list);
        
        // Initialize an empty vote list
        let empty_votes: Vec<T::AccountId> = Vec::new();
        let bounded_votes: BoundedVec<T::AccountId, T::MaxItems> = empty_votes.try_into().unwrap_or_default();
        RevocationVotes::<T>::insert(&domain, bounded_votes);

        #[extrinsic_call]
        vote_for_domain_revocation(RawOrigin::Signed(caller), domain);
    }

    impl_benchmark_test_suite!(AssetDiscovery, crate::mock::new_test_ext(), crate::mock::Test);
}
