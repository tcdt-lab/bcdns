use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn register_tld_works() {
    new_test_ext().execute_with(|| {
        let tld_name = b"example".to_vec();
        let chain_spec = b"{\"chainSpec\":\"details\"}".to_vec();

        assert_ok!(RootDNSModule::register_tld(RuntimeOrigin::signed(1), tld_name.clone(), chain_spec.clone()));

        let stored = RootDNSModule::tld_map(tld_name.clone()).unwrap();
        assert_eq!(stored.chain_spec, chain_spec);
    });
}

#[test]
fn register_tld_fails_for_duplicate() {
    new_test_ext().execute_with(|| {
        let tld_name = b"example".to_vec();
        let chain_spec = b"{\"chainSpec\":\"details\"}".to_vec();

        assert_ok!(RootDNSModule::register_tld(RuntimeOrigin::signed(1), tld_name.clone(), chain_spec.clone()));
        assert_noop!(
            RootDNSModule::register_tld(RuntimeOrigin::signed(1), tld_name.clone(), chain_spec.clone()),
            Error::<Test>::TLDAlreadyRegistered
        );
    });
}

#[test]
fn remove_tld_works() {
    new_test_ext().execute_with(|| {
        let tld_name = b"example".to_vec();
        let chain_spec = b"{\"chainSpec\":\"details\"}".to_vec();

        assert_ok!(RootDNSModule::register_tld(RuntimeOrigin::signed(1), tld_name.clone(), chain_spec.clone()));
        assert_ok!(RootDNSModule::remove_tld(RuntimeOrigin::root(), tld_name.clone()));

        assert!(RootDNSModule::tld_map(tld_name).is_none());
    });
}

#[test]
fn remove_tld_fails_for_non_existent() {
    new_test_ext().execute_with(|| {
        let tld_name = b"nonexistent".to_vec();
        assert_noop!(
            RootDNSModule::remove_tld(RuntimeOrigin::root(), tld_name),
            Error::<Test>::TLDNotFound
        );
    });
}