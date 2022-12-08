use super::super::*;
use crate::{add_omnipool_token, create_subpool};
use pallet_omnipool::types::{AssetReserveState, Tradability};

//const ONE: u128 = 1;
const ALICE_INITIAL_LRNA_BALANCE: Balance = 500 * ONE;
const ALICE_INITIAL_ASSET_3_BALANCE: Balance = 1000 * ONE;
const ALICE_INITIAL_ASSET_5_BALANCE: Balance = 5000 * ONE;

const USDA: u32 = 3;
const USDB: u32 = 4;
const USDC: u32 = 5;
const R1: u32 = 6;
const R2: u32 = 7;

const SUBPOOL_ID: u32 = 100;

const ONE_MILLION: Balance = 1_000_000 * ONE;

const OMNIPOOL_INITIAL_HDX_BALANCE: Balance = 1_000_000 * ONE;
const OMNIPOOL_INITIAL_DAI_BALANCE: Balance = 1_000_000 * ONE;

#[test]
fn subpool_trades_should_work_correct_when_trade_stable_out_given_asset_in() {
	ExtBuilder::default()
		.with_registered_asset(USDA)
		.with_registered_asset(USDB)
		.with_registered_asset(USDC)
		.with_registered_asset(R1)
		.with_registered_asset(R2)
		.with_registered_asset(SUBPOOL_ID)
		.with_endowed_accounts(vec![
			(Omnipool::protocol_account(), HDX, OMNIPOOL_INITIAL_HDX_BALANCE),
			(Omnipool::protocol_account(), DAI, OMNIPOOL_INITIAL_DAI_BALANCE),
		])
		.add_endowed_accounts((Omnipool::protocol_account(), USDA, ONE_MILLION))
		.add_endowed_accounts((Omnipool::protocol_account(), USDB, ONE_MILLION))
		.add_endowed_accounts((Omnipool::protocol_account(), USDC, ONE_MILLION))
		.add_endowed_accounts((Omnipool::protocol_account(), R1, ONE_MILLION))
		.add_endowed_accounts((Omnipool::protocol_account(), R2, 3 * ONE_MILLION))
		.add_endowed_accounts((ALICE, R1, 10_000 * ONE))
		.add_endowed_accounts((LP1, 1_000, 5000 * ONE))
		.with_initial_pool(FixedU128::from_float(100.0), FixedU128::from_float(4.0))
		.build()
		.execute_with(|| {
			add_omnipool_token!(USDA, FixedU128::from_float(100.0));
			add_omnipool_token!(USDB, FixedU128::from_float(100.0));
			add_omnipool_token!(USDC, FixedU128::from_float(100.0));
			add_omnipool_token!(R1, FixedU128::from_float(50.0));
			add_omnipool_token!(R2, FixedU128::from_float(150.0));

			create_subpool!(SUBPOOL_ID, USDA, USDB);

			let usda_alice = Tokens::free_balance(USDA, &ALICE);

			assert_eq!(usda_alice, 0u128);

			let amount_to_sell = 1000 * ONE;
			assert_ok!(OmnipoolSubpools::sell(
				Origin::signed(ALICE),
				R1,
				USDA,
				amount_to_sell,
				0u128
			));

			let r1_state = Omnipool::load_asset_state(R1).unwrap();
			let subpool_state = Stableswap::get_pool(SUBPOOL_ID).unwrap();
			let subpool_share_state = Omnipool::load_asset_state(SUBPOOL_ID).unwrap();

			let usda_alice = Tokens::free_balance(USDA, &ALICE);
			let r1_alice = Tokens::free_balance(R1, &ALICE);

			assert_eq!(usda_alice, 499372810804422);
			assert_eq!(r1_alice, 9000000000000000);

			assert_eq!(
				r1_state,
				AssetReserveState {
					reserve: 1001000000000000000,
					hub_reserve: 49950049950049950050,
					shares: 1000000000000000000u128,
					protocol_shares: 0u128,
					cap: 1000000000000000000u128,
					tradable: Tradability::default()
				},
			);

			assert_eq!(
				subpool_state.balances::<Test>(),
				vec![999500627189195578, 1000000000000000000]
			);

			assert_eq!(
				subpool_share_state,
				AssetReserveState {
					reserve: 199950062421972534333,
					hub_reserve: 200049950049950049950,
					shares: 200000000000000000000u128,
					protocol_shares: 0u128,
					cap: 500000000000000000u128,
					tradable: Tradability::default(),
				},
			);
		});
}