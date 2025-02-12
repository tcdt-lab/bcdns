//! Tests for the TLD Pallet

use super::*;
use crate::mock::{new_test_ext, RuntimeOrigin, Test, TldModule};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn register_domain_works() {
    new_test_ext().execute_with(|| {
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            "example".as_bytes().to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> = "{\"key\":\"value\"}"
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            "maintainer_id".as_bytes().to_vec().try_into().unwrap();
        let expiry = 100u64;

        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(1),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry
        ));

        assert!(DomainMap::<Test>::contains_key(&domain_name));
    });
}

#[test]
fn amend_chainspec_works() {
    new_test_ext().execute_with(|| {
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            "example".as_bytes().to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> = "{\"key\":\"value\"}"
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap();
        let new_chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            "{\"key\":\"new_value\"}"
                .as_bytes()
                .to_vec()
                .try_into()
                .unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            "maintainer_id".as_bytes().to_vec().try_into().unwrap();
        let expiry = 100u64;

        TldModule::register_domain(
            RuntimeOrigin::signed(1),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry,
        )
        .unwrap();
        assert_ok!(TldModule::amend_chainspec(
            RuntimeOrigin::signed(1),
            domain_name.clone(),
            new_chain_spec.clone()
        ));

        let domain_info = DomainMap::<Test>::get(&domain_name).unwrap();
        assert_eq!(domain_info.chain_spec, new_chain_spec);
    });
}

#[test]
fn revoke_domain_works() {
    new_test_ext().execute_with(|| {
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            "example".as_bytes().to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> = "{\"key\":\"value\"}"
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            "maintainer_id".as_bytes().to_vec().try_into().unwrap();
        let expiry = 100u64;

        TldModule::register_domain(
            RuntimeOrigin::signed(1),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry,
        )
        .unwrap();
        assert_ok!(TldModule::revoke_domain(
            RuntimeOrigin::signed(1),
            domain_name.clone()
        ));

        assert!(!DomainMap::<Test>::contains_key(&domain_name));
    });
}

#[test]
fn test_initiate_transfer() {
    new_test_ext().execute_with(|| {
        let caller = 1u64; // Account ID of caller
        let new_owner = 2u64; // Account ID of the new owner
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"example-domain".to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            b"example-spec".to_vec().try_into().unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            b"maintainer".to_vec().try_into().unwrap();
        let expiry = 200;

        // Register the domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry,
        ));

        // Ensure the domain exists
        assert!(DomainMap::<Test>::contains_key(&domain_name));

        // Initiate the transfer
        assert_ok!(TldModule::initiate_transfer(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            new_owner
        ));

        // Verify the transfer was added to `PendingTransfers`
        assert!(PendingTransfers::<Test>::contains_key(&domain_name));
        assert_eq!(
            PendingTransfers::<Test>::get(&domain_name).unwrap(),
            new_owner
        );
    });
}

#[test]
fn test_accept_transfer() {
    new_test_ext().execute_with(|| {
        let caller = 1u64; // Account ID of caller
        let new_owner = 2u64; // Account ID of the new owner
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"example-accept".to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            b"example-spec".to_vec().try_into().unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            b"maintainer".to_vec().try_into().unwrap();
        let expiry = 200;

        // Register the domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry,
        ));

        // Ensure the domain exists
        assert!(DomainMap::<Test>::contains_key(&domain_name));

        // Initiate the transfer
        assert_ok!(TldModule::initiate_transfer(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            new_owner
        ));

        // Accept the transfer
        assert_ok!(TldModule::accept_transfer(
            RuntimeOrigin::signed(new_owner),
            domain_name.clone(),
        ));

        // Verify the transfer was removed from `PendingTransfers`
        assert!(!PendingTransfers::<Test>::contains_key(&domain_name));

        // Verify the new owner was updated
        let domain_info = DomainMap::<Test>::get(&domain_name).unwrap();
        assert_eq!(domain_info.creator, new_owner);
    });
}

#[test]
fn test_revoke_transfer() {
    new_test_ext().execute_with(|| {
        let caller = 1u64; // Account ID of caller
        let new_owner = 2u64; // Account ID of the new owner
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"example-revoke".to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            b"example-spec".to_vec().try_into().unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            b"maintainer".to_vec().try_into().unwrap();
        let expiry = 200;

        // Register the domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry,
        ));

        // Ensure the domain exists
        assert!(DomainMap::<Test>::contains_key(&domain_name));

        // Initiate the transfer
        assert_ok!(TldModule::initiate_transfer(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            new_owner
        ));

        // Revoke the transfer
        assert_ok!(TldModule::revoke_transfer(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
        ));

        // Verify the transfer was removed from `PendingTransfers`
        assert!(!PendingTransfers::<Test>::contains_key(&domain_name));
    });
}

#[test]
fn test_initiate_transfer_no_permission() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;
        let not_creator = 3u64;
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"no-permission".to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            b"example-spec".to_vec().try_into().unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            b"maintainer".to_vec().try_into().unwrap();
        let expiry = 200;

        // Register the domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry,
        ));

        // Attempt to initiate a transfer with an invalid account
        assert_noop!(
            TldModule::initiate_transfer(
                RuntimeOrigin::signed(not_creator),
                domain_name.clone(),
                caller
            ),
            crate::Error::<Test>::NotDomainOwner
        );
    });
}

#[test]
fn test_accept_transfer_no_permission() {
    new_test_ext().execute_with(|| {
        let caller = 1u64;
        let new_owner = 2u64;
        let not_recipient = 3u64;
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"wrong-recipient".to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            b"example-spec".to_vec().try_into().unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            b"maintainer".to_vec().try_into().unwrap();
        let expiry = 200;

        // Register the domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
            expiry,
        ));

        // Initiate a transfer
        assert_ok!(TldModule::initiate_transfer(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            new_owner
        ));

        // Attempt to accept the transfer with an invalid account
        assert_noop!(
            TldModule::accept_transfer(RuntimeOrigin::signed(not_recipient), domain_name.clone(),),
            crate::Error::<Test>::NotTransferRecipient
        );
    });
}
