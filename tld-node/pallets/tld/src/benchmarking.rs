//! Benchmarking setup for pallet-tld
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as TldModule;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::BoundedVec;
use frame_support::traits::Get;
use frame_system::pallet_prelude::BlockNumberFor;
use frame_support::sp_runtime::SaturatedConversion;
use scale_info::prelude::vec::Vec;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn benchmark_register_domain() {
        let caller: T::AccountId = whitelisted_caller();
        let domain_name_raw: Vec<u8> = "example".as_bytes().to_vec();
        let domain_name: BoundedVec<u8, T::MaxDomainLength> = domain_name_raw.clone().try_into().unwrap();
        
        let chain_spec_raw: Vec<u8> = "{\"key\":\"value\"}".as_bytes().to_vec();
        let chain_spec: BoundedVec<u8, T::MaxChainSpecSize> = chain_spec_raw.clone().try_into().unwrap();
        
        let maintainer_raw: Vec<u8> = "maintainer_id".as_bytes().to_vec();
        let maintainer: BoundedVec<u8, T::MaxMaintainerSize> = maintainer_raw.clone().try_into().unwrap();

        #[extrinsic_call]
        register_domain(
            RawOrigin::Signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
        );

        assert!(DomainMap::<T>::contains_key(&domain_name));
    }

    #[benchmark]
    fn benchmark_amend_chainspec() {
        let caller: T::AccountId = whitelisted_caller();
        let domain_name_raw: Vec<u8> = "example".as_bytes().to_vec();
        let domain_name: BoundedVec<u8, T::MaxDomainLength> = domain_name_raw.clone().try_into().unwrap();
        
        let chain_spec_raw: Vec<u8> = "{\"key\":\"value\"}".as_bytes().to_vec();
        let chain_spec: BoundedVec<u8, T::MaxChainSpecSize> = chain_spec_raw.clone().try_into().unwrap();
        
        let new_chain_spec_raw: Vec<u8> = "{\"key\":\"new_value\"}".as_bytes().to_vec();
        let new_chain_spec: BoundedVec<u8, T::MaxChainSpecSize> = new_chain_spec_raw.clone().try_into().unwrap();
        
        let maintainer_raw: Vec<u8> = "maintainer_id".as_bytes().to_vec();
        let maintainer: BoundedVec<u8, T::MaxMaintainerSize> = maintainer_raw.clone().try_into().unwrap();

        TldModule::<T>::register_domain(
            RawOrigin::Signed(caller.clone()).into(),
            domain_name.clone(),
            chain_spec,
            maintainer,
        )
        .unwrap();

        #[extrinsic_call]
        amend_chainspec(
            RawOrigin::Signed(caller),
            domain_name.clone(),
            new_chain_spec.clone(),
        );

        let domain_info = DomainMap::<T>::get(&domain_name).unwrap();
        assert_eq!(domain_info.chain_spec, new_chain_spec);
    }

    #[benchmark]
    fn benchmark_revoke_domain() {
        let caller: T::AccountId = whitelisted_caller();
        let domain_name_raw: Vec<u8> = "example".as_bytes().to_vec();
        let domain_name: BoundedVec<u8, T::MaxDomainLength> = domain_name_raw.clone().try_into().unwrap();
        
        let chain_spec_raw: Vec<u8> = "{\"key\":\"value\"}".as_bytes().to_vec();
        let chain_spec: BoundedVec<u8, T::MaxChainSpecSize> = chain_spec_raw.clone().try_into().unwrap();
        
        let maintainer_raw: Vec<u8> = "maintainer_id".as_bytes().to_vec();
        let maintainer: BoundedVec<u8, T::MaxMaintainerSize> = maintainer_raw.clone().try_into().unwrap();

        TldModule::<T>::register_domain(
            RawOrigin::Signed(caller.clone()).into(),
            domain_name.clone(),
            chain_spec,
            maintainer,
        )
        .unwrap();

        #[extrinsic_call]
        revoke_domain(RawOrigin::Signed(caller), domain_name.clone());

        assert!(!DomainMap::<T>::contains_key(&domain_name));
    }

    #[benchmark]
    fn benchmark_initiate_transfer() {
        let caller: T::AccountId = whitelisted_caller();
        let new_owner: T::AccountId = account("new_owner", 0, 0);
        let domain_name_raw: Vec<u8> = "example".as_bytes().to_vec();
        let domain_name: BoundedVec<u8, T::MaxDomainLength> = domain_name_raw.clone().try_into().unwrap();
        
        let chain_spec_raw: Vec<u8> = "{\"key\":\"value\"}".as_bytes().to_vec();
        let chain_spec: BoundedVec<u8, T::MaxChainSpecSize> = chain_spec_raw.clone().try_into().unwrap();
        
        let maintainer_raw: Vec<u8> = "maintainer_id".as_bytes().to_vec();
        let maintainer: BoundedVec<u8, T::MaxMaintainerSize> = maintainer_raw.clone().try_into().unwrap();

        TldModule::<T>::register_domain(
            RawOrigin::Signed(caller.clone()).into(),
            domain_name.clone(),
            chain_spec,
            maintainer,
        )
        .unwrap();

        #[extrinsic_call]
        initiate_transfer(
            RawOrigin::Signed(caller),
            domain_name.clone(),
            new_owner.clone(),
        );

        // Check that the pending transfer is recorded in the appropriate storage
        assert!(PendingTransfers::<T>::contains_key(&domain_name));
        assert_eq!(PendingTransfers::<T>::get(&domain_name).unwrap(), new_owner);
    }

    #[benchmark]
    fn benchmark_accept_transfer() {
        let owner: T::AccountId = whitelisted_caller();
        let new_owner: T::AccountId = account("new_owner", 0, 0);
        let domain_name_raw: Vec<u8> = "example".as_bytes().to_vec();
        let domain_name: BoundedVec<u8, T::MaxDomainLength> = domain_name_raw.clone().try_into().unwrap();
        
        let chain_spec_raw: Vec<u8> = "{\"key\":\"value\"}".as_bytes().to_vec();
        let chain_spec: BoundedVec<u8, T::MaxChainSpecSize> = chain_spec_raw.clone().try_into().unwrap();
        
        let maintainer_raw: Vec<u8> = "maintainer_id".as_bytes().to_vec();
        let maintainer: BoundedVec<u8, T::MaxMaintainerSize> = maintainer_raw.clone().try_into().unwrap();

        TldModule::<T>::register_domain(
            RawOrigin::Signed(owner.clone()).into(),
            domain_name.clone(),
            chain_spec,
            maintainer,
        )
        .unwrap();

        TldModule::<T>::initiate_transfer(
            RawOrigin::Signed(owner).into(),
            domain_name.clone(),
            new_owner.clone(),
        )
        .unwrap();

        #[extrinsic_call]
        accept_transfer(RawOrigin::Signed(new_owner.clone()), domain_name.clone());

        let domain_info = DomainMap::<T>::get(&domain_name).unwrap();
        assert_eq!(domain_info.creator, new_owner);
        assert!(!PendingTransfers::<T>::contains_key(&domain_name));
    }

    #[benchmark]
    fn benchmark_revoke_transfer() {
        let owner: T::AccountId = whitelisted_caller();
        let new_owner: T::AccountId = account("new_owner", 0, 0);
        let domain_name_raw: Vec<u8> = "example".as_bytes().to_vec();
        let domain_name: BoundedVec<u8, T::MaxDomainLength> = domain_name_raw.clone().try_into().unwrap();
        
        let chain_spec_raw: Vec<u8> = "{\"key\":\"value\"}".as_bytes().to_vec();
        let chain_spec: BoundedVec<u8, T::MaxChainSpecSize> = chain_spec_raw.clone().try_into().unwrap();
        
        let maintainer_raw: Vec<u8> = "maintainer_id".as_bytes().to_vec();
        let maintainer: BoundedVec<u8, T::MaxMaintainerSize> = maintainer_raw.clone().try_into().unwrap();

        TldModule::<T>::register_domain(
            RawOrigin::Signed(owner.clone()).into(),
            domain_name.clone(),
            chain_spec,
            maintainer,
        )
        .unwrap();

        TldModule::<T>::initiate_transfer(
            RawOrigin::Signed(owner.clone()).into(),
            domain_name.clone(),
            new_owner,
        )
        .unwrap();

        #[extrinsic_call]
        revoke_transfer(RawOrigin::Signed(owner), domain_name.clone());

        assert!(!PendingTransfers::<T>::contains_key(&domain_name));
    }

    #[benchmark]
    fn benchmark_send_heartbeat() {
        let caller: T::AccountId = whitelisted_caller();
        let domain_name_raw: Vec<u8> = "example".as_bytes().to_vec();
        let domain_name: BoundedVec<u8, T::MaxDomainLength> = domain_name_raw.clone().try_into().unwrap();
        
        let chain_spec_raw: Vec<u8> = "{\"key\":\"value\"}".as_bytes().to_vec();
        let chain_spec: BoundedVec<u8, T::MaxChainSpecSize> = chain_spec_raw.clone().try_into().unwrap();
        
        let maintainer_raw: Vec<u8> = "maintainer_id".as_bytes().to_vec();
        let maintainer: BoundedVec<u8, T::MaxMaintainerSize> = maintainer_raw.clone().try_into().unwrap();

        // Register domain
        TldModule::<T>::register_domain(
            RawOrigin::Signed(caller.clone()).into(),
            domain_name.clone(),
            chain_spec.clone(),
            maintainer.clone(),
        )
        .unwrap();

        // Advance blocks to allow for a new heartbeat
        let interval_value = T::HeartbeatInterval::get().saturated_into::<u32>();
        let min_interval_u32 = interval_value / 10 + 1;
        for _ in 0..min_interval_u32 {
            frame_system::Pallet::<T>::set_block_number(
                frame_system::Pallet::<T>::block_number() + 1u32.saturated_into::<BlockNumberFor<T>>(),
            );
        }

        #[extrinsic_call]
        send_heartbeat(RawOrigin::Signed(caller), domain_name.clone());

        // Verify the domain info has been updated with a new heartbeat timestamp
        let domain_info = DomainMap::<T>::get(&domain_name).unwrap();
        assert_eq!(domain_info.last_heartbeat, frame_system::Pallet::<T>::block_number());
    }

    #[benchmark]
    fn benchmark_report_missed_heartbeat() {
        let maintainer: T::AccountId = whitelisted_caller();
        let observer: T::AccountId = account("observer", 0, 0);
        let domain_name_raw: Vec<u8> = "example".as_bytes().to_vec();
        let domain_name: BoundedVec<u8, T::MaxDomainLength> = domain_name_raw.clone().try_into().unwrap();
        
        let chain_spec_raw: Vec<u8> = "{\"key\":\"value\"}".as_bytes().to_vec();
        let chain_spec: BoundedVec<u8, T::MaxChainSpecSize> = chain_spec_raw.clone().try_into().unwrap();
        
        let maintainer_id_raw: Vec<u8> = "maintainer_id".as_bytes().to_vec();
        let maintainer_id: BoundedVec<u8, T::MaxMaintainerSize> = maintainer_id_raw.clone().try_into().unwrap();

        // Register domain
        TldModule::<T>::register_domain(
            RawOrigin::Signed(maintainer.clone()).into(),
            domain_name.clone(),
            chain_spec.clone(),
            maintainer_id.clone(),
        )
        .unwrap();

        // Advance blocks past the heartbeat interval to make the domain eligible for reporting
        let interval_value = T::HeartbeatInterval::get().saturated_into::<u32>();
        let interval_u32 = interval_value + 1;
        for _ in 0..interval_u32 {
            frame_system::Pallet::<T>::set_block_number(
                frame_system::Pallet::<T>::block_number() + 1u32.saturated_into::<BlockNumberFor<T>>(),
            );
        }

        #[extrinsic_call]
        report_missed_heartbeat(RawOrigin::Signed(observer), domain_name.clone());

        // Verify an observation has been recorded
        assert_eq!(HeartbeatObservationCount::<T>::get(&domain_name), 1);
    }

    impl_benchmark_test_suite!(TldModule, mock::new_test_ext(), mock::Test);
}
