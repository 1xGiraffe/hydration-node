mod engine;
mod intents;

use crate as pallet_omnix;
use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::pallet_prelude::ConstU32;
use frame_support::traits::{ConstU128, ConstU64, Everything, Time};
use frame_support::{construct_runtime, parameter_types, PalletId};
use hydradx_traits::router::{AmountInAndOut, AssetPair, RouterT, Trade};
use orml_traits::{parameter_type_with_key, GetByKey};
use pallet_currencies::BasicCurrencyAdapter;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, BlockNumberProvider, IdentityLookup};
use sp_runtime::transaction_validity::TransactionPriority;
use sp_runtime::{BuildStorage, DispatchError, DispatchResult};

type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type AssetId = u32;
pub(crate) type AccountId = u64;
type NamedReserveIdentifier = [u8; 8];

use crate::types::{Balance, IntentId, Moment};

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

pub(crate) const LRNA: AssetId = 1;

pub const NOW: Moment = 1689844300000; // unix time in milliseconds

construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Currencies: pallet_currencies,
		Tokens: orml_tokens,
		OmniX: pallet_omnix,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeTask = RuntimeTask;
	type Nonce = u64;
	type Block = Block;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: AssetId| -> Balance {
		if *currency_id == LRNA{
			400_000_000
		}else{
			1
		}
	};
}

impl orml_tokens::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = i128;
	type CurrencyId = AssetId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type MaxLocks = ();
	type DustRemovalWhitelist = ();
	type MaxReserves = ();
	type ReserveIdentifier = NamedReserveIdentifier;
	type CurrencyHooks = ();
}

impl pallet_currencies::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Test, Balances, i128, u32>;
	type GetNativeCurrencyId = NativeCurrencyId;
	type WeightInfo = ();
}

parameter_types! {
	pub const HubAssetId: AssetId = LRNA;
	pub const MaxCallData: u32 = 4 * 1024 * 1024;
	pub const OmniXPalletId: PalletId = PalletId(*b"tstomnix");
	pub const MaxAllowdIntentDuration: Moment = 86_400_000; //1day
	pub const NativeCurrencyId: AssetId = 0;
}

pub struct DummyTimestampProvider;

impl Time for DummyTimestampProvider {
	type Moment = u64;

	fn now() -> Self::Moment {
		//TODO: perhaps use some static value which is possible to set as part of test
		NOW
	}
}

pub struct DummyOrder;

impl GetByKey<RuntimeCall, TransactionPriority> for DummyOrder {
	fn get(k: &RuntimeCall) -> TransactionPriority {
		0
	}
}

pub struct MockBlockNumberProvider;

impl BlockNumberProvider for MockBlockNumberProvider {
	type BlockNumber = u64;

	fn current_block_number() -> Self::BlockNumber {
		System::block_number()
	}
}

impl pallet_omnix::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssetId = AssetId;
	type HubAssetId = HubAssetId;
	type TimestampProvider = DummyTimestampProvider;
	type MaxAllowedIntentDeadline = MaxAllowdIntentDuration;
	type BlockNumberProvider = MockBlockNumberProvider;
	type Currency = Tokens;
	type ReservableCurrency = Currencies;
	type TradeExecutor = DummyTradeExecutor;
	type PalletId = OmniXPalletId;
	type MaxCallData = MaxCallData;
	type PriorityOrder = DummyOrder;
	type WeightInfo = ();
}

pub struct DummyTradeExecutor;

impl
	RouterT<
		RuntimeOrigin,
		AssetId,
		Balance,
		hydradx_traits::router::Trade<AssetId>,
		hydradx_traits::router::AmountInAndOut<Balance>,
	> for DummyTradeExecutor
{
	fn sell(
		_origin: RuntimeOrigin,
		_asset_in: AssetId,
		_asset_out: AssetId,
		_amount_in: Balance,
		_min_amount_out: Balance,
		_route: Vec<Trade<AssetId>>,
	) -> DispatchResult {
		unimplemented!()
	}

	fn sell_all(
		_origin: RuntimeOrigin,
		_asset_in: AssetId,
		_asset_out: AssetId,
		_min_amount_out: Balance,
		_route: Vec<Trade<AssetId>>,
	) -> DispatchResult {
		unimplemented!()
	}

	fn buy(
		_origin: RuntimeOrigin,
		_asset_in: AssetId,
		_asset_out: AssetId,
		_amount_out: Balance,
		_max_amount_in: Balance,
		_route: Vec<Trade<AssetId>>,
	) -> DispatchResult {
		unimplemented!()
	}

	fn calculate_sell_trade_amounts(
		_route: &[Trade<AssetId>],
		_amount_in: Balance,
	) -> Result<Vec<AmountInAndOut<Balance>>, DispatchError> {
		unimplemented!()
	}

	fn calculate_buy_trade_amounts(
		_route: &[Trade<AssetId>],
		_amount_out: Balance,
	) -> Result<Vec<AmountInAndOut<Balance>>, DispatchError> {
		unimplemented!()
	}

	fn set_route(
		_origin: RuntimeOrigin,
		_asset_pair: AssetPair<AssetId>,
		_route: Vec<Trade<AssetId>>,
	) -> DispatchResultWithPostInfo {
		unimplemented!()
	}

	fn force_insert_route(
		_origin: RuntimeOrigin,
		_asset_pair: AssetPair<AssetId>,
		_route: Vec<Trade<AssetId>>,
	) -> DispatchResultWithPostInfo {
		unimplemented!()
	}
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder {}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
		let mut r: sp_io::TestExternalities = t.into();

		r.execute_with(|| {
			System::set_block_number(1);
		});

		r
	}
}

pub(crate) fn get_intent_id(moment: Moment, increment: u64) -> IntentId {
	crate::Pallet::<Test>::get_intent_id(moment, increment)
}
