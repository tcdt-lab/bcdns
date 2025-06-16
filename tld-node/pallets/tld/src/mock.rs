use crate::{self as pallet_tld};
use frame_support::pallet_prelude::ConstU32;
use frame_support::{
    derive_impl,
    traits::{ConstU16, ConstU64},
};
use sp_core::{
    sr25519::Signature,
    H256
};
use sp_runtime::{
    testing::TestXt,
    traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        TldModule: pallet_tld,
    }
);

// Add this to your mock.rs file
use sp_core::crypto::KeyTypeId;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"test");

pub mod crypto {
    use super::KEY_TYPE;
    use sp_runtime::app_crypto::{app_crypto, sr25519};
    use sp_runtime::{MultiSignature, MultiSigner};
    use sp_core::sr25519::Public as Sr25519Public;
    use frame_system::offchain::AppCrypto;
    
    // Define the crypto type for testing
    app_crypto!(sr25519, KEY_TYPE);
    
    // Use the types defined by the app_crypto macro
    pub type TestPublic = Public;    
    pub struct TestAuthId;
    
    // Implementation for MultiSignature
    impl AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
        type RuntimeAppPublic = TestPublic;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
    
    // Implementation for the specific types used in the pallet
    impl AppCrypto<<Sr25519Public as sp_runtime::traits::IdentifyAccount>::AccountId, sp_core::sr25519::Signature> for TestAuthId {
        type RuntimeAppPublic = TestPublic;
        type GenericSignature = sp_core::sr25519::Signature;
        type GenericPublic = sp_core::sr25519::Public;
    }
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = sp_core::sr25519::Public;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
type Extrinsic = TestXt<RuntimeCall, ()>;

impl frame_system::offchain::SigningTypes for Test {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
    RuntimeCall: From<LocalCall>,
{
    type OverarchingCall = RuntimeCall;
    type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
    RuntimeCall: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: RuntimeCall,
        _public: Self::Public,
        _account: AccountId,
        nonce: u64,
    ) -> Option<(RuntimeCall, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
        Some((call, (nonce, ())))
    }
}

impl pallet_tld::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_tld::weights::SubstrateWeight<Test>;
    type MaxDomainLength = ConstU32<16>;
    type MaxChainSpecSize = ConstU32<256>;
    type MaxMaintainerSize = ConstU32<16>;
    type ExpiryBlocks = ConstU32<1000>;
    type AuthorityId = crypto::TestAuthId;
    type RevocationThreshold = ConstU32<3>; // Require 3 observations before auto-revocation
    type HeartbeatInterval = ConstU32<100>; // Heartbeat required every 100 blocks
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}
