//! Tests for the RootDNS Pallet

use crate::{mock::*, Config, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn register_tld_works() {
    new_test_ext().execute_with(|| {
        let tld_name: BoundedVec<u8, <Test as Config>::MaxTLDNameLength> =
            BoundedVec::try_from(b"example".to_vec()).unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            BoundedVec::try_from(b"{\"chainSpec\":\"details\"}".to_vec()).unwrap();

        assert_ok!(RootDNSModule::register_tld(
            RuntimeOrigin::signed(1),
            tld_name.clone(),
            chain_spec.clone()
        ));

        let stored = RootDNSModule::tld_map(tld_name.clone()).unwrap();
        assert_eq!(stored.chain_spec, chain_spec);
    });
}

#[test]
fn register_tld_fails_for_duplicate() {
    new_test_ext().execute_with(|| {
        let tld_name: BoundedVec<u8, <Test as Config>::MaxTLDNameLength> =
            BoundedVec::try_from(b"example".to_vec()).unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            BoundedVec::try_from(b"{\"chainSpec\":\"details\"}".to_vec()).unwrap();

        assert_ok!(RootDNSModule::register_tld(
            RuntimeOrigin::signed(1),
            tld_name.clone(),
            chain_spec.clone()
        ));
        assert_noop!(
            RootDNSModule::register_tld(
                RuntimeOrigin::signed(1),
                tld_name.clone(),
                chain_spec.clone()
            ),
            Error::<Test>::TLDAlreadyRegistered
        );
    });
}

#[test]
fn remove_tld_works() {
    new_test_ext().execute_with(|| {
        let tld_name: BoundedVec<u8, <Test as Config>::MaxTLDNameLength> =
            BoundedVec::try_from(b"example".to_vec()).unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            BoundedVec::try_from(b"{\"chainSpec\":\"details\"}".to_vec()).unwrap();

        assert_ok!(RootDNSModule::register_tld(
            RuntimeOrigin::signed(1),
            tld_name.clone(),
            chain_spec.clone()
        ));
        assert_ok!(RootDNSModule::remove_tld(
            RuntimeOrigin::root(),
            tld_name.clone()
        ));

        assert!(RootDNSModule::tld_map(tld_name).is_none());
    });
}

#[test]
fn remove_tld_fails_for_non_existent() {
    new_test_ext().execute_with(|| {
        let tld_name: BoundedVec<u8, <Test as Config>::MaxTLDNameLength> =
            BoundedVec::try_from(b"nonexistent".to_vec()).unwrap();

        assert_noop!(
            RootDNSModule::remove_tld(RuntimeOrigin::root(), tld_name),
            Error::<Test>::TLDNotFound
        );
    });
}
