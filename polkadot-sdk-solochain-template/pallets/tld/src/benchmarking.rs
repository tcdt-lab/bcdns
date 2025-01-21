//! Benchmarking setup for pallet-tld
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as TldModule;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use scale_info::prelude::vec::Vec;

#[benchmarks]
mod benchmarks {
    use super::*;
    use frame_system::pallet_prelude::BlockNumberFor;

    #[benchmark]
    fn benchmark_register_domain() {
        let caller: T::AccountId = whitelisted_caller();
        let domain_name: Vec<u8> = "example".as_bytes().to_vec().try_into().unwrap();
        let chain_spec: Vec<u8> = "{\"key\":\"value\"}"
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap();
        let maintainer: Vec<u8> = "maintainer_id".as_bytes().to_vec().try_into().unwrap();
        let expiry: BlockNumberFor<T> = 100u32.into();

        #[extrinsic_call]
        register_domain(
            RawOrigin::Signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry,
        );

        assert!(DomainMap::<T>::contains_key(&domain_name));
    }

    #[benchmark]
    fn benchmark_amend_chainspec() {
        let caller: T::AccountId = whitelisted_caller();
        let domain_name: Vec<u8> = "example".as_bytes().to_vec().try_into().unwrap();
        let chain_spec: Vec<u8> = "{\"key\":\"value\"}"
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap();
        let new_chain_spec: Vec<u8> = "{\"key\":\"new_value\"}"
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap();
        TldModule::<T>::register_domain(
            RawOrigin::Signed(caller.clone()).into(),
            domain_name.clone(),
            chain_spec,
            "maintainer_id".as_bytes().to_vec().try_into().unwrap(),
            100u32.into(),
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
        let domain_name: Vec<u8> = "example".as_bytes().to_vec().try_into().unwrap();
        let chain_spec: Vec<u8> = "{\"key\":\"value\"}"
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap();
        TldModule::<T>::register_domain(
            RawOrigin::Signed(caller.clone()).into(),
            domain_name.clone(),
            chain_spec,
            "maintainer_id".as_bytes().to_vec().try_into().unwrap(),
            100u32.into(),
        )
        .unwrap();

        #[extrinsic_call]
        revoke_domain(RawOrigin::Signed(caller), domain_name.clone());

        assert!(!DomainMap::<T>::contains_key(&domain_name));
    }

    // #[benchmark]
    // fn benchmark_transfer_domain() {
    //     let caller: T::AccountId = whitelisted_caller();
    //     let new_owner: T::AccountId = account("new_owner", 0, 0);
    //
    //     let domain_name = b"benchmark-domain".to_vec();
    //     let chain_spec = b"chain-spec".to_vec();
    //     let maintainer = b"maintainer".to_vec();
    //     let expiry: BlockNumberFor<T> = 100u32.into();
    //
    //     let _ = Pallet::<T>::register_domain(
    //         RawOrigin::Signed(caller.clone()).into(),
    //         domain_name.clone(),
    //         chain_spec,
    //         maintainer,
    //         expiry,
    //     );
    //
    //     #[extrinsic_call]
    //     transfer_domain(
    //         RawOrigin::Signed(caller.clone()),
    //         domain_name.clone(),
    //         new_owner.clone(),
    //     );
    //
    //     assert!(DomainMap::<T>::get(&domain_name).is_some());
    //     assert_eq!(
    //         DomainMap::<T>::get(&domain_name).unwrap().creator,
    //         new_owner
    //     );
    // }

    #[benchmark]
    fn benchmark_initiate_transfer() {
        let caller: T::AccountId = whitelisted_caller();
        let new_owner: T::AccountId = account("new_owner", 0, 0);

        let domain_name: Vec<u8> = "transfer-domain".as_bytes().to_vec().try_into().unwrap();
        let chain_spec: Vec<u8> = "{\"key\":\"value\"}"
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap();
        let maintainer: Vec<u8> = "maintainer".as_bytes().to_vec().try_into().unwrap();
        let expiry: BlockNumberFor<T> = 200u32.into();

        // Register the domain
        TldModule::<T>::register_domain(
            RawOrigin::Signed(caller.clone()).into(),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry,
        )
        .unwrap();

        #[extrinsic_call]
        initiate_transfer(
            RawOrigin::Signed(caller.clone()),
            domain_name.clone(),
            new_owner.clone(),
        );

        assert!(PendingTransfers::<T>::contains_key(&domain_name));
        assert_eq!(PendingTransfers::<T>::get(&domain_name).unwrap(), new_owner);
    }

    #[benchmark]
    fn benchmark_accept_transfer() {
        let caller: T::AccountId = whitelisted_caller();
        let new_owner: T::AccountId = account("new_owner", 0, 0);

        let domain_name: Vec<u8> = "accept-domain".as_bytes().to_vec().try_into().unwrap();
        let chain_spec: Vec<u8> = "{\"key\":\"value\"}"
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap();
        let maintainer: Vec<u8> = "maintainer".as_bytes().to_vec().try_into().unwrap();
        let expiry: BlockNumberFor<T> = 200u32.into();

        // Register the domain
        TldModule::<T>::register_domain(
            RawOrigin::Signed(caller.clone()).into(),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry,
        )
        .unwrap();

        // Initiate a transfer
        TldModule::<T>::initiate_transfer(
            RawOrigin::Signed(caller.clone()).into(),
            domain_name.clone(),
            new_owner.clone(),
        )
        .unwrap();

        #[extrinsic_call]
        accept_transfer(
            RawOrigin::Signed(new_owner.clone()),
            domain_name.clone().try_into().unwrap(),
        );

        let domain_info = DomainMap::<T>::get(&domain_name).unwrap();
        assert_eq!(domain_info.creator, new_owner);
        assert!(!PendingTransfers::<T>::contains_key(&domain_name));
    }

    #[benchmark]
    fn benchmark_revoke_transfer() {
        let caller: T::AccountId = whitelisted_caller();
        let new_owner: T::AccountId = account("new_owner", 0, 0);

        let domain_name: Vec<u8> = "revoke-domain".as_bytes().to_vec().try_into().unwrap();
        let chain_spec: Vec<u8> = "{\"key\":\"value\"}"
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap();
        let maintainer: Vec<u8> = "maintainer".as_bytes().to_vec().try_into().unwrap();
        let expiry: BlockNumberFor<T> = 200u32.into();

        // Register the domain
        TldModule::<T>::register_domain(
            RawOrigin::Signed(caller.clone()).into(),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry,
        )
        .unwrap();

        // Initiate a transfer
        TldModule::<T>::initiate_transfer(
            RawOrigin::Signed(caller.clone()).into(),
            domain_name.clone(),
            new_owner.clone(),
        )
        .unwrap();

        #[extrinsic_call]
        revoke_transfer(RawOrigin::Signed(caller.clone()), domain_name.clone());

        assert!(!PendingTransfers::<T>::contains_key(&domain_name));
    }

    impl_benchmark_test_suite!(TldModule, mock::new_test_ext(), mock::Test);
}
