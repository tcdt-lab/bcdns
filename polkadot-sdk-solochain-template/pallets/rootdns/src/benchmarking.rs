//! Benchmarking setup for pallet-rootdns
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Rootdns;
use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use scale_info::prelude::vec;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn benchmark_register_tld() {
        let caller: T::AccountId = whitelisted_caller();
        let tld_name = vec![b'a'; T::MaxTLDNameLength::get() as usize];
        let chain_spec = vec![b'c'; T::MaxChainSpecSize::get() as usize];

        #[extrinsic_call]
        register_tld(
            RawOrigin::Signed(caller),
            tld_name.clone(),
            chain_spec.clone(),
        );

        assert!(TLDMap::<T>::contains_key(&tld_name));
    }

    #[benchmark]
    fn benchmark_remove_tld() {
        let caller: T::AccountId = whitelisted_caller();
        let tld_name = vec![b'a'; T::MaxTLDNameLength::get() as usize];
        let chain_spec = vec![b'c'; T::MaxChainSpecSize::get() as usize];
        let _ = Rootdns::<T>::register_tld(RawOrigin::Signed(caller.clone()).into(), tld_name.clone(), chain_spec);

        #[extrinsic_call]
        remove_tld(RawOrigin::Root, tld_name.clone());

        assert!(!TLDMap::<T>::contains_key(&tld_name));
    }

    impl_benchmark_test_suite!(Rootdns, mock::new_test_ext(), mock::Test);
}
