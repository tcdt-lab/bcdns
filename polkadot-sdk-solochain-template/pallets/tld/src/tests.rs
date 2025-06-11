//! Tests for the TLD Pallet

use super::*;
use crate::mock::{new_test_ext, RuntimeOrigin, Test, TldModule};
use frame_support::{assert_noop, assert_ok, BoundedVec};

fn test_pub() -> sp_core::sr25519::Public {
	sp_core::sr25519::Public::from_raw([1u8; 32])
}

fn test_pub_new() -> sp_core::sr25519::Public {
	sp_core::sr25519::Public::from_raw([2u8; 32])
}


fn test_pub_not_recipient() -> sp_core::sr25519::Public {
	sp_core::sr25519::Public::from_raw([3u8; 32])
}

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

        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone(),
            chain_spec,
            maintainer
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

        TldModule::register_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone(),
            chain_spec,
            maintainer,
        )
        .unwrap();
        assert_ok!(TldModule::amend_chainspec(
            RuntimeOrigin::signed(test_pub()),
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

        TldModule::register_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone(),
            chain_spec,
            maintainer,
        )
        .unwrap();
        assert_ok!(TldModule::revoke_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone()
        ));

        assert!(!DomainMap::<Test>::contains_key(&domain_name));
    });
}

#[test]
fn test_initiate_transfer() {
    new_test_ext().execute_with(|| {
        let caller = test_pub(); // Account ID of caller
        let new_owner = test_pub_new(); // Account ID of the new owner
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"example-domain".to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            b"example-spec".to_vec().try_into().unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            b"maintainer".to_vec().try_into().unwrap();

        // Register the domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
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
        let caller = test_pub(); // Account ID of caller
        let new_owner = test_pub_new(); // Account ID of the new owner
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"example-accept".to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            b"example-spec".to_vec().try_into().unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            b"maintainer".to_vec().try_into().unwrap();

        // Register the domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
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
        let caller = test_pub(); // Account ID of caller
        let new_owner = test_pub_new(); // Account ID of the new owner
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"example-revoke".to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            b"example-spec".to_vec().try_into().unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            b"maintainer".to_vec().try_into().unwrap();

        // Register the domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
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
        let caller = test_pub();
        let not_creator = test_pub_new();
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"no-permission".to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            b"example-spec".to_vec().try_into().unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            b"maintainer".to_vec().try_into().unwrap();

        // Register the domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
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
        let caller = test_pub();
        let new_owner = test_pub_new();
        let not_recipient = test_pub_not_recipient();
        let domain_name: BoundedVec<u8, <Test as Config>::MaxDomainLength> =
            b"wrong-recipient".to_vec().try_into().unwrap();
        let chain_spec: BoundedVec<u8, <Test as Config>::MaxChainSpecSize> =
            b"example-spec".to_vec().try_into().unwrap();
        let maintainer: BoundedVec<u8, <Test as Config>::MaxMaintainerSize> =
            b"maintainer".to_vec().try_into().unwrap();

        // Register the domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(caller),
            domain_name.clone(),
            chain_spec,
            maintainer,
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

#[test]
fn test_send_heartbeat() {
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

        // Register domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone(),
            chain_spec,
            maintainer
        ));

        // Get initial domain info
        let initial_domain_info = DomainMap::<Test>::get(&domain_name).unwrap();
        let initial_heartbeat = initial_domain_info.last_heartbeat;

        // Advance blocks to allow for a new heartbeat
        let min_interval = 51; // Must be more than HeartbeatInterval / 2 (50 blocks)
        frame_system::Pallet::<Test>::set_block_number(
            frame_system::Pallet::<Test>::block_number() + min_interval
        );

        // Send heartbeat
        assert_ok!(TldModule::send_heartbeat(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone()
        ));

        // Verify heartbeat was updated
        let updated_domain_info = DomainMap::<Test>::get(&domain_name).unwrap();
        assert!(updated_domain_info.last_heartbeat > initial_heartbeat);
        assert_eq!(updated_domain_info.last_heartbeat, frame_system::Pallet::<Test>::block_number());
    });
}

#[test]
fn test_send_heartbeat_too_soon() {
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

        // Register domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone(),
            chain_spec,
            maintainer
        ));

        // Try to send heartbeat immediately (should fail due to rate limiting)
        assert_noop!(
            TldModule::send_heartbeat(RuntimeOrigin::signed(test_pub()), domain_name.clone()),
            Error::<Test>::HeartbeatTooSoon
        );
    });
}

#[test]
fn test_send_heartbeat_not_maintainer() {
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

        // Register domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone(),
            chain_spec,
            maintainer
        ));

        // Try to send heartbeat from non-maintainer account
        assert_noop!(
            TldModule::send_heartbeat(RuntimeOrigin::signed(test_pub_new()), domain_name.clone()),
            Error::<Test>::NotDomainMaintainer
        );
    });
}

#[test]
fn test_report_missed_heartbeat() {
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

        // Register domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone(),
            chain_spec,
            maintainer
        ));

        // Advance blocks past the heartbeat interval (using a large enough value)
        let interval = 100 + 1;
        for _ in 0..interval {
            frame_system::Pallet::<Test>::set_block_number(
                frame_system::Pallet::<Test>::block_number() + 1
            );
        }

        // Report missed heartbeat
        assert_ok!(TldModule::report_missed_heartbeat(
            RuntimeOrigin::signed(test_pub_new()),
            domain_name.clone()
        ));

        // Verify observation was recorded
        assert_eq!(HeartbeatObservationCount::<Test>::get(&domain_name), 1);
        assert!(HeartbeatObservations::<Test>::contains_key(&domain_name, &test_pub_new()));
    });
}

#[test]
fn test_report_missed_heartbeat_duplicate() {
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

        // Register domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone(),
            chain_spec,
            maintainer
        ));

        // Advance blocks past the heartbeat interval
        let interval = 100 + 1;
        for _ in 0..interval {
            frame_system::Pallet::<Test>::set_block_number(
                frame_system::Pallet::<Test>::block_number() + 1
            );
        }

        // Report missed heartbeat
        assert_ok!(TldModule::report_missed_heartbeat(
            RuntimeOrigin::signed(test_pub_new()),
            domain_name.clone()
        ));

        // Try to report again from the same account (should fail)
        assert_noop!(
            TldModule::report_missed_heartbeat(RuntimeOrigin::signed(test_pub_new()), domain_name.clone()),
            Error::<Test>::DuplicateHeartbeatObservation
        );
    });
}

#[test]
fn test_report_missed_heartbeat_no_missed() {
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

        // Register domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone(),
            chain_spec,
            maintainer
        ));

        // Try to report without missing heartbeat (should fail)
        assert_noop!(
            TldModule::report_missed_heartbeat(RuntimeOrigin::signed(test_pub_new()), domain_name.clone()),
            Error::<Test>::HeartbeatTooSoon
        );
    });
}

#[test]
fn test_threshold_revocation() {
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

        // Register domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone(),
            chain_spec,
            maintainer
        ));

        // Advance blocks past the heartbeat interval
        let interval = 100 + 1;
        for _ in 0..interval {
            frame_system::Pallet::<Test>::set_block_number(
                frame_system::Pallet::<Test>::block_number() + 1
            );
        }

        // Report from first observer
        assert_ok!(TldModule::report_missed_heartbeat(
            RuntimeOrigin::signed(test_pub_new()),
            domain_name.clone()
        ));

        // Domain should still be available
        let domain_info = DomainMap::<Test>::get(&domain_name).unwrap();
        assert!(domain_info.available);

        // Report from second observer
        assert_ok!(TldModule::report_missed_heartbeat(
            RuntimeOrigin::signed(test_pub_not_recipient()),
            domain_name.clone()
        ));

        // Domain should still be available (threshold is 3)
        let domain_info = DomainMap::<Test>::get(&domain_name).unwrap();
        assert!(domain_info.available);

        // Create a third observer account
        let test_pub_third = sp_core::sr25519::Public::from_raw([4u8; 32]);

        // Report from third observer
        assert_ok!(TldModule::report_missed_heartbeat(
            RuntimeOrigin::signed(test_pub_third),
            domain_name.clone()
        ));

        // Domain should now be unavailable (threshold reached)
        let domain_info = DomainMap::<Test>::get(&domain_name).unwrap();
        assert!(!domain_info.available);
    });
}

#[test]
fn test_domain_expiry_check() {
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

        // Register domain
        assert_ok!(TldModule::register_domain(
            RuntimeOrigin::signed(test_pub()),
            domain_name.clone(),
            chain_spec.clone(),
            maintainer
        ));

        // Advance blocks past the heartbeat interval
        let interval = 100 + 1;
        for _ in 0..interval {
            frame_system::Pallet::<Test>::set_block_number(
                frame_system::Pallet::<Test>::block_number() + 1
            );
        }

        // Try to amend chainspec (should fail due to missed heartbeat)
        assert_noop!(
            TldModule::amend_chainspec(
                RuntimeOrigin::signed(test_pub()),
                domain_name.clone(),
                chain_spec
            ),
            Error::<Test>::DomainExpired
        );
    });
}
