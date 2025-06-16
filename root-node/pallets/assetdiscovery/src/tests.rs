use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use sp_core::sr25519;
use sp_runtime::traits::BadOrigin;

// Helper function to convert a string to a ByteVector
fn to_bounded_vec<T: crate::Config>(s: &[u8]) -> BoundedVec<u8, T::MaxByteLength> {
    BoundedVec::<u8, T::MaxByteLength>::try_from(s.to_vec()).unwrap()
}

// Helper function to get test accounts
fn get_account(id: u8) -> sr25519::Public {
    sr25519::Public::from_raw([id; 32])
}

// Helper function to setup a domain with assets for testing
fn setup_domain_with_assets() -> (BoundedVec<u8, <Test as crate::Config>::MaxByteLength>, BoundedVec<u8, <Test as crate::Config>::MaxByteLength>) {
    let domain = to_bounded_vec::<Test>(b"example.test");
    let asset_hash = to_bounded_vec::<Test>(b"asset123");
    
    // Register the asset for the domain
    assert_ok!(
        AssetDiscovery::register_asset_for_domain(
            RuntimeOrigin::signed(get_account(1)),
            domain.clone(),
            asset_hash.clone()
        )
    );
    
    // Create the key for the pending request
    let mut key_bytes = vec![];
    key_bytes.append(&mut asset_hash.clone().to_vec());
    key_bytes.append(&mut domain.clone().to_vec());
    let key = to_bounded_vec::<Test>(&key_bytes);
    
    // Submit the verified domain
    let pending_request = AssetDiscovery::pending_requests(&key).unwrap();
    assert_ok!(
        AssetDiscovery::submit_verified_domain(
            RuntimeOrigin::root(),
            key.clone(),
            pending_request.clone()
        )
    );
    
    (domain, asset_hash)
}

#[test]
fn test_register_asset_for_domain() {
    new_test_ext().execute_with(|| {
        // Progress to block 1 so events get deposited
        System::set_block_number(1);
        
        let domain = to_bounded_vec::<Test>(b"example.test");
        let asset_hash = to_bounded_vec::<Test>(b"asset123");
        let account = get_account(1);
        
        // Register asset for domain
        assert_ok!(
            AssetDiscovery::register_asset_for_domain(
                RuntimeOrigin::signed(account.clone()),
                domain.clone(),
                asset_hash.clone()
            )
        );
        
        // Check that the pending request was stored correctly
        let mut key_bytes = vec![];
        key_bytes.append(&mut asset_hash.clone().to_vec());
        key_bytes.append(&mut domain.clone().to_vec());
        let key = to_bounded_vec::<Test>(&key_bytes);
        
        let pending_request = AssetDiscovery::pending_requests(&key).unwrap();
        assert_eq!(pending_request.domain, domain);
        assert_eq!(pending_request.asset_hash, asset_hash);
        
        // Assert that the correct event was deposited
        System::assert_has_event(
            Event::DomainValidationRequested(account, pending_request).into()
        );
    });
}

#[test]
fn test_submit_verified_domain() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let domain = to_bounded_vec::<Test>(b"example.test");
        let asset_hash = to_bounded_vec::<Test>(b"asset123");
        let account = get_account(1);
        
        // Register the asset for the domain
        assert_ok!(
            AssetDiscovery::register_asset_for_domain(
                RuntimeOrigin::signed(account.clone()),
                domain.clone(),
                asset_hash.clone()
            )
        );
        
        // Create the key for the pending request
        let mut key_bytes = vec![];
        key_bytes.append(&mut asset_hash.clone().to_vec());
        key_bytes.append(&mut domain.clone().to_vec());
        let key = to_bounded_vec::<Test>(&key_bytes);
        
        let pending_request = AssetDiscovery::pending_requests(&key).unwrap();
        assert_ok!(
            AssetDiscovery::submit_verified_domain(
                RuntimeOrigin::root(),
                key.clone(),
                pending_request.clone()
            )
        );
        
        // Check that the pending request was removed
        assert!(AssetDiscovery::pending_requests(&key).is_none());
        
        // Check that the domain was registered with the asset
        let provider_assets = AssetDiscovery::provider_assets(&domain).unwrap();
        assert!(provider_assets.assets.contains(&asset_hash));
        
        // Check that the asset has the domain as a provider
        let asset_providers = AssetDiscovery::asset_providers(&asset_hash).unwrap();
        assert!(asset_providers.providers.contains(&domain));
        
        // Assert that the correct event was deposited
        System::assert_has_event(
            Event::AssetRegisteredForDomain(domain, asset_hash, pending_request.timestamp).into()
        );
    });
}

#[test]
fn test_vote_for_domain_revocation() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Setup domain with assets
        let (domain, asset_hash) = setup_domain_with_assets();
        
        // Now vote for domain revocation
        assert_ok!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(1)),
                domain.clone()
            )
        );
        
        // Check that the vote was recorded
        let votes = AssetDiscovery::revocation_votes(&domain);
        assert_eq!(votes.len(), 1);
        assert!(votes.contains(&get_account(1)));
        
        // Assert that the correct event was deposited
        System::assert_has_event(
            Event::RevocationVoteSubmitted(get_account(1), domain.clone()).into()
        );
        
        // Add more votes to reach the threshold (3 votes)
        assert_ok!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(2)),
                domain.clone()
            )
        );
        
        assert_ok!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(3)),
                domain.clone()
            )
        );
        
        // Check that the domain was revoked after reaching the threshold
        assert!(AssetDiscovery::provider_assets(&domain).is_none());
        
        // Check that the asset no longer has the domain as a provider
        let asset_providers = AssetDiscovery::asset_providers(&asset_hash).unwrap();
        assert!(!asset_providers.providers.contains(&domain));
        
        // Assert that the correct event was deposited
        System::assert_has_event(
            Event::AssetProviderRevoked(domain.clone()).into()
        );
    });
}

#[test]
fn test_double_voting_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Setup domain with assets
        let (domain, _) = setup_domain_with_assets();
        
        // Vote once
        assert_ok!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(1)),
                domain.clone()
            )
        );
        
        // Try to vote again with the same account
        assert_noop!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(1)),
                domain.clone()
            ),
            Error::<Test>::AlreadyVoted
        );
    });
}

#[test]
fn test_remove_expired_pending_requests() {
    new_test_ext().execute_with(|| {
        // Set up initial block number
        System::set_block_number(1);
        
        let domain = to_bounded_vec::<Test>(b"example.test");
        let asset_hash = to_bounded_vec::<Test>(b"asset123");
        
        // Register the asset for the domain
        assert_ok!(
            AssetDiscovery::register_asset_for_domain(
                RuntimeOrigin::signed(get_account(1)),
                domain.clone(),
                asset_hash.clone()
            )
        );
        
        // Create the key for the pending request
        let mut key_bytes = vec![];
        key_bytes.append(&mut asset_hash.clone().to_vec());
        key_bytes.append(&mut domain.clone().to_vec());
        let key = to_bounded_vec::<Test>(&key_bytes);
        
        // Verify the pending request exists
        assert!(AssetDiscovery::pending_requests(&key).is_some());
        
        // Progress the block number past the expiration (current_block_number + REQUEST_LIFETIME)
        System::set_block_number(1002); 
        
        // Remove expired pending requests
        assert_ok!(
            AssetDiscovery::remove_expired_pending_requests(RuntimeOrigin::root())
        );
        
        // Check that the pending request was removed
        assert!(AssetDiscovery::pending_requests(&key).is_none());
        
        // Assert that the correct event was deposited
        System::assert_has_event(
            Event::ExpiredRequestsRemoved.into()
        );
    });
}

#[test]
fn test_non_existent_domain_revocation_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let non_existent_domain = to_bounded_vec::<Test>(b"nonexistent.test");
        
        // Try to vote for revocation of a non-existent domain
        assert_noop!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(1)),
                non_existent_domain
            ),
            Error::<Test>::DomainDoesNotExist
        );
    });
}

#[test]
fn test_unauthorized_access_fails() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let domain = to_bounded_vec::<Test>(b"example.test");
        let asset_hash = to_bounded_vec::<Test>(b"asset123");
        
        // Register the asset for the domain
        assert_ok!(
            AssetDiscovery::register_asset_for_domain(
                RuntimeOrigin::signed(get_account(1)),
                domain.clone(),
                asset_hash.clone()
            )
        );
        
        // Create the key for the pending request
        let mut key_bytes = vec![];
        key_bytes.append(&mut asset_hash.clone().to_vec());
        key_bytes.append(&mut domain.clone().to_vec());
        let key = to_bounded_vec::<Test>(&key_bytes);
        
        let pending_request = AssetDiscovery::pending_requests(&key).unwrap();
        
        // Try to submit verified domain with a non-signed origin
        assert_noop!(
            AssetDiscovery::submit_verified_domain(
                RuntimeOrigin::none(),
                key.clone(),
                pending_request.clone()
            ),
            BadOrigin
        );
        
        // Try to remove expired requests with a non-signed origin
        assert_noop!(
            AssetDiscovery::remove_expired_pending_requests(RuntimeOrigin::none()),
            BadOrigin
        );
    });
}

#[test]
fn test_revocation_votes_cleared_after_revocation() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Setup domain with assets
        let (domain, _) = setup_domain_with_assets();
        
        // Vote for domain revocation
        assert_ok!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(1)),
                domain.clone()
            )
        );
        
        // Check that the vote was recorded
        let votes = AssetDiscovery::revocation_votes(&domain);
        assert_eq!(votes.len(), 1);
        
        // Add more votes to reach the threshold (3 votes)
        assert_ok!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(2)),
                domain.clone()
            )
        );
        
        assert_ok!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(3)),
                domain.clone()
            )
        );
        
        // Check that the votes were cleared after revocation
        assert!(AssetDiscovery::revocation_votes(&domain).is_empty());
    });
}

#[test]
fn test_multiple_assets_for_domain() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        let domain = to_bounded_vec::<Test>(b"example.test");
        let asset_hash1 = to_bounded_vec::<Test>(b"asset123");
        let asset_hash2 = to_bounded_vec::<Test>(b"asset456");
        let account = get_account(1);
        
        // Register first asset for domain
        assert_ok!(
            AssetDiscovery::register_asset_for_domain(
                RuntimeOrigin::signed(account.clone()),
                domain.clone(),
                asset_hash1.clone()
            )
        );
        
        // Create the key for the first pending request
        let mut key_bytes1 = vec![];
        key_bytes1.append(&mut asset_hash1.clone().to_vec());
        key_bytes1.append(&mut domain.clone().to_vec());
        let key1 = to_bounded_vec::<Test>(&key_bytes1);
        
        // Submit the first verified domain
        let pending_request1 = AssetDiscovery::pending_requests(&key1).unwrap();
        assert_ok!(
            AssetDiscovery::submit_verified_domain(
                RuntimeOrigin::root(),
                key1.clone(),
                pending_request1.clone()
            )
        );
        
        // Register second asset for domain
        assert_ok!(
            AssetDiscovery::register_asset_for_domain(
                RuntimeOrigin::signed(account.clone()),
                domain.clone(),
                asset_hash2.clone()
            )
        );
        
        // Create the key for the second pending request
        let mut key_bytes2 = vec![];
        key_bytes2.append(&mut asset_hash2.clone().to_vec());
        key_bytes2.append(&mut domain.clone().to_vec());
        let key2 = to_bounded_vec::<Test>(&key_bytes2);
        
        // Submit the second verified domain
        let pending_request2 = AssetDiscovery::pending_requests(&key2).unwrap();
        assert_ok!(
            AssetDiscovery::submit_verified_domain(
                RuntimeOrigin::root(),
                key2.clone(),
                pending_request2.clone()
            )
        );
        
        // Check that the domain has both assets
        let provider_assets = AssetDiscovery::provider_assets(&domain).unwrap();
        assert!(provider_assets.assets.contains(&asset_hash1));
        assert!(provider_assets.assets.contains(&asset_hash2));
        
        // Check that both assets have the domain as a provider
        let asset_providers1 = AssetDiscovery::asset_providers(&asset_hash1).unwrap();
        assert!(asset_providers1.providers.contains(&domain));
        
        let asset_providers2 = AssetDiscovery::asset_providers(&asset_hash2).unwrap();
        assert!(asset_providers2.providers.contains(&domain));
        
        // Add votes to reach the revocation threshold (3 votes)
        assert_ok!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(1)),
                domain.clone()
            )
        );
        
        assert_ok!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(2)),
                domain.clone()
            )
        );
        
        assert_ok!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(3)),
                domain.clone()
            )
        );
        
        // Check that the domain was revoked
        assert!(AssetDiscovery::provider_assets(&domain).is_none());
        
        // Check that both assets no longer have the domain as a provider
        let asset_providers1_after = AssetDiscovery::asset_providers(&asset_hash1).unwrap();
        assert!(!asset_providers1_after.providers.contains(&domain));
        
        let asset_providers2_after = AssetDiscovery::asset_providers(&asset_hash2).unwrap();
        assert!(!asset_providers2_after.providers.contains(&domain));
    });
}

#[test]
fn test_cleanup_revoked_domains() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Setup multiple domains with assets
        let domain1 = to_bounded_vec::<Test>(b"example1.test");
        let domain2 = to_bounded_vec::<Test>(b"example2.test");
        let asset_hash = to_bounded_vec::<Test>(b"asset123");
        let account = get_account(1);
        
        // Register asset for first domain
        assert_ok!(
            AssetDiscovery::register_asset_for_domain(
                RuntimeOrigin::signed(account.clone()),
                domain1.clone(),
                asset_hash.clone()
            )
        );
        
        // Create the key for the first pending request
        let mut key_bytes1 = vec![];
        key_bytes1.append(&mut asset_hash.clone().to_vec());
        key_bytes1.append(&mut domain1.clone().to_vec());
        let key1 = to_bounded_vec::<Test>(&key_bytes1);
        
        // Submit the first verified domain
        let pending_request1 = AssetDiscovery::pending_requests(&key1).unwrap();
        assert_ok!(
            AssetDiscovery::submit_verified_domain(
                RuntimeOrigin::root(),
                key1.clone(),
                pending_request1.clone()
            )
        );
        
        // Register asset for second domain
        assert_ok!(
            AssetDiscovery::register_asset_for_domain(
                RuntimeOrigin::signed(account.clone()),
                domain2.clone(),
                asset_hash.clone()
            )
        );
        
        // Create the key for the second pending request
        let mut key_bytes2 = vec![];
        key_bytes2.append(&mut asset_hash.clone().to_vec());
        key_bytes2.append(&mut domain2.clone().to_vec());
        let key2 = to_bounded_vec::<Test>(&key_bytes2);
        
        // Submit the second verified domain
        let pending_request2 = AssetDiscovery::pending_requests(&key2).unwrap();
        assert_ok!(
            AssetDiscovery::submit_verified_domain(
                RuntimeOrigin::root(),
                key2.clone(),
                pending_request2.clone()
            )
        );
        
        // Create a list of domains to revoke
        let domains = vec![domain1.clone(), domain2.clone()];
        let bounded_domains = BoundedVec::<_, <Test as crate::Config>::MaxItems>::try_from(domains).unwrap();
        
        // Cleanup the revoked domains
        assert_ok!(
            AssetDiscovery::cleanup_revoked_domains(
                RuntimeOrigin::signed(account.clone()),
                bounded_domains
            )
        );
        
        // Check that both domains were revoked
        assert!(AssetDiscovery::provider_assets(&domain1).is_none());
        assert!(AssetDiscovery::provider_assets(&domain2).is_none());
        
        // Check that the asset no longer has either domain as a provider
        let asset_providers = AssetDiscovery::asset_providers(&asset_hash).unwrap();
        assert!(!asset_providers.providers.contains(&domain1));
        assert!(!asset_providers.providers.contains(&domain2));
    });
}

#[test]
fn test_too_many_votes_error() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        
        // Setup domain with assets
        let domain = to_bounded_vec::<Test>(b"example.test");
        let asset_hash = to_bounded_vec::<Test>(b"asset123");
        let account = get_account(1);
        
        // Register the asset for the domain
        assert_ok!(
            AssetDiscovery::register_asset_for_domain(
                RuntimeOrigin::signed(account.clone()),
                domain.clone(),
                asset_hash.clone()
            )
        );
        
        // Create the key for the pending request
        let mut key_bytes = vec![];
        key_bytes.append(&mut asset_hash.clone().to_vec());
        key_bytes.append(&mut domain.clone().to_vec());
        let key = to_bounded_vec::<Test>(&key_bytes);
        
        // Submit the verified domain
        let pending_request = AssetDiscovery::pending_requests(&key).unwrap();
        assert_ok!(
            AssetDiscovery::submit_verified_domain(
                RuntimeOrigin::root(),
                key.clone(),
                pending_request.clone()
            )
        );
        
        // Verify the domain exists
        assert!(AssetDiscovery::provider_assets(&domain).is_some());
        
        // Create a separate domain for testing TooManyVotes
        let test_domain = to_bounded_vec::<Test>(b"test-votes.example");
        let test_asset = to_bounded_vec::<Test>(b"test-asset");
        
        // Register the test asset for the test domain
        assert_ok!(
            AssetDiscovery::register_asset_for_domain(
                RuntimeOrigin::signed(account.clone()),
                test_domain.clone(),
                test_asset.clone()
            )
        );
        
        // Create the key for the test pending request
        let mut test_key_bytes = vec![];
        test_key_bytes.append(&mut test_asset.clone().to_vec());
        test_key_bytes.append(&mut test_domain.clone().to_vec());
        let test_key = to_bounded_vec::<Test>(&test_key_bytes);
        
        // Submit the verified test domain
        let test_pending_request = AssetDiscovery::pending_requests(&test_key).unwrap();
        assert_ok!(
            AssetDiscovery::submit_verified_domain(
                RuntimeOrigin::root(),
                test_key.clone(),
                test_pending_request.clone()
            )
        );
        
        // Verify the test domain exists
        assert!(AssetDiscovery::provider_assets(&test_domain).is_some());
        
        // Add 5 votes to the test domain
        let mut votes = frame_support::BoundedVec::<_, <Test as crate::Config>::MaxItems>::default();
        for i in 1..=5 {
            votes.try_push(get_account(i)).unwrap();
        }
        crate::RevocationVotes::<Test>::insert(&test_domain, votes);
        
        // Verify votes were added
        assert_eq!(AssetDiscovery::revocation_votes(&test_domain).len(), 5);
        
        // Add one more vote
        assert_noop!(
            AssetDiscovery::vote_for_domain_revocation(
                RuntimeOrigin::signed(get_account(6)),
                test_domain.clone()
            ),
            Error::<Test>::TooManyVotes
        );
    });
}
