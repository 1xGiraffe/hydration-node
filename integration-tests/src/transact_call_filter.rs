#![cfg(test)]
use crate::polkadot_test_net::*;

use frame_support::{assert_ok, weights::Weight};
use sp_runtime::codec::Encode;

use polkadot_xcm::latest::prelude::*;
use xcm_emulator::TestExt;

#[test]
fn allowed_transact_call_should_pass_filter() {
	// Arrange
	TestNet::reset();

	Hydra::execute_with(|| {
		assert_ok!(hydradx_runtime::Balances::transfer(
			hydradx_runtime::RuntimeOrigin::signed(ALICE.into()),
			parachain_reserve_account(),
			1_000 * UNITS,
		));
	});

	Acala::execute_with(|| {
		// allowed by SafeCallFilter and the runtime call filter
		let call = pallet_balances::Call::<hydradx_runtime::Runtime>::transfer {
			dest: BOB.into(),
			value: UNITS,
		};
		let message = Xcm(vec![
			WithdrawAsset(
				(
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(HYDRA_PARA_ID), GeneralIndex(0)),
					},
					900 * UNITS,
				)
					.into(),
			),
			BuyExecution {
				fees: (
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(HYDRA_PARA_ID), GeneralIndex(0)),
					},
					800 * UNITS,
				)
					.into(),
				weight_limit: Unlimited,
			},
			Transact {
				require_weight_at_most: Weight::from_parts(10_000_000_000, 0u64),
				origin_kind: OriginKind::SovereignAccount,
				call: hydradx_runtime::RuntimeCall::Balances(call).encode().into(),
			},
			ExpectTransactStatus(MaybeErrorCode::Success),
			RefundSurplus,
			DepositAsset {
				assets: All.into(),
				beneficiary: Junction::AccountId32 {
					id: parachain_reserve_account().into(),
					network: None,
				}
				.into(),
			},
		]);

		// Act
		assert_ok!(hydradx_runtime::PolkadotXcm::send_xcm(
			Here,
			MultiLocation::new(1, X1(Parachain(HYDRA_PARA_ID))),
			message
		));
	});

	Hydra::execute_with(|| {
		// Assert
		assert!(hydradx_runtime::System::events().iter().any(|r| matches!(
			r.event,
			hydradx_runtime::RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. })
		)));
		assert_eq!(
			hydradx_runtime::Balances::free_balance(&AccountId::from(BOB)),
			BOB_INITIAL_NATIVE_BALANCE + UNITS
		);
	});
}

#[test]
fn blocked_transact_calls_should_not_pass_filter() {
	// Arrange
	TestNet::reset();

	Hydra::execute_with(|| {
		assert_ok!(hydradx_runtime::Balances::transfer(
			hydradx_runtime::RuntimeOrigin::signed(ALICE.into()),
			parachain_reserve_account(),
			1_000 * UNITS,
		));
	});

	Acala::execute_with(|| {
		// filtered by SafeCallFilter
		let call = pallet_tips::Call::<hydradx_runtime::Runtime>::report_awesome {
			reason: vec![0, 10],
			who: BOB.into(),
		};
		let message = Xcm(vec![
			WithdrawAsset(
				(
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(HYDRA_PARA_ID), GeneralIndex(0)),
					},
					900 * UNITS,
				)
					.into(),
			),
			BuyExecution {
				fees: (
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(HYDRA_PARA_ID), GeneralIndex(0)),
					},
					800 * UNITS,
				)
					.into(),
				weight_limit: Unlimited,
			},
			Transact {
				require_weight_at_most: Weight::from_parts(10_000_000_000, 0u64),
				origin_kind: OriginKind::Native,
				call: hydradx_runtime::RuntimeCall::Tips(call).encode().into(),
			},
			ExpectTransactStatus(MaybeErrorCode::Success),
			RefundSurplus,
			DepositAsset {
				assets: All.into(),
				beneficiary: Junction::AccountId32 {
					id: parachain_reserve_account().into(),
					network: None,
				}
				.into(),
			},
		]);

		// Act
		assert_ok!(hydradx_runtime::PolkadotXcm::send_xcm(
			Here,
			MultiLocation::new(1, X1(Parachain(HYDRA_PARA_ID))),
			message
		));
	});

	Hydra::execute_with(|| {
		// Assert
		assert!(hydradx_runtime::System::events().iter().any(|r| matches!(
			r.event,
			hydradx_runtime::RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
				error: cumulus_primitives_core::XcmError::NoPermission,
				..
			})
		)));
	});
}

#[test]
fn safe_call_filter_should_respect_runtime_call_filter() {
	// Arrange
	TestNet::reset();

	Hydra::execute_with(|| {
		assert_ok!(hydradx_runtime::Balances::transfer(
			hydradx_runtime::RuntimeOrigin::signed(ALICE.into()),
			parachain_reserve_account(),
			1_000 * UNITS,
		));
	});

	Acala::execute_with(|| {
		// transfer to the Omnipool is filtered by the runtime call filter
		let call = pallet_balances::Call::<hydradx_runtime::Runtime>::transfer {
			dest: hydradx_runtime::Omnipool::protocol_account(),
			value: UNITS,
		};
		let message = Xcm(vec![
			WithdrawAsset(
				(
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(HYDRA_PARA_ID), GeneralIndex(0)),
					},
					900 * UNITS,
				)
					.into(),
			),
			BuyExecution {
				fees: (
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(HYDRA_PARA_ID), GeneralIndex(0)),
					},
					800 * UNITS,
				)
					.into(),
				weight_limit: Unlimited,
			},
			Transact {
				require_weight_at_most: Weight::from_parts(1_000_000_000, 2653u64),
				origin_kind: OriginKind::Native,
				call: hydradx_runtime::RuntimeCall::Balances(call).encode().into(),
			},
			ExpectTransactStatus(MaybeErrorCode::Success),
			RefundSurplus,
			DepositAsset {
				assets: All.into(),
				beneficiary: Junction::AccountId32 {
					id: parachain_reserve_account().into(),
					network: None,
				}
				.into(),
			},
		]);

		// Act
		assert_ok!(hydradx_runtime::PolkadotXcm::send_xcm(
			Here,
			MultiLocation::new(1, X1(Parachain(HYDRA_PARA_ID))),
			message
		));
	});

	Hydra::execute_with(|| {
		// Assert
		assert!(hydradx_runtime::System::events().iter().any(|r| matches!(
			r.event,
			hydradx_runtime::RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
				error: cumulus_primitives_core::XcmError::NoPermission,
				..
			})
		)));
	});
}

use hydradx_runtime::xcm_account_derivation::HashedDescriptionDescribeFamilyAllTerminal;
use orml_traits::MultiCurrency;
use xcm_executor::traits::Convert;
#[test]
fn asd() {
	// Arrange
	TestNet::reset();

	let xcm_interior_at_acala = X1(Junction::AccountId32 {
		network: None,
		id: evm_account().into(),
	});

	let xcm_interior_at_hydra = X2(
		Junction::Parachain(ACALA_PARA_ID),
		Junction::AccountId32 {
			network: None,
			id: evm_account().into(),
		},
	);

	let xcm_origin_at_hydra = MultiLocation {
		parents: 1,
		interior: xcm_interior_at_hydra,
	};

	let acala_account_id_at_hydra: AccountId =
		HashedDescriptionDescribeFamilyAllTerminal::convert_ref(xcm_origin_at_hydra).unwrap();

	Hydra::execute_with(|| {
		init_omnipool();

		assert_ok!(hydradx_runtime::Balances::transfer(
			hydradx_runtime::RuntimeOrigin::signed(ALICE.into()),
			acala_account_id_at_hydra.clone(),
			1_000 * UNITS,
		));
		let dai_balance =
			hydradx_runtime::Currencies::free_balance(DAI, &AccountId::from(acala_account_id_at_hydra.clone()));
		assert_eq!(dai_balance, 0);
	});

	Acala::execute_with(|| {
		// allowed by SafeCallFilter and the runtime call filter
		let call = pallet_balances::Call::<hydradx_runtime::Runtime>::transfer {
			dest: BOB.into(),
			value: UNITS,
		};

		let omni_sell =
			hydradx_runtime::RuntimeCall::Omnipool(pallet_omnipool::Call::<hydradx_runtime::Runtime>::sell {
				asset_in: HDX,
				asset_out: DAI,
				amount: UNITS,
				min_buy_amount: 0,
			});

		let message = Xcm(vec![
			WithdrawAsset(
				(
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(HYDRA_PARA_ID), GeneralIndex(0)),
					},
					900 * UNITS,
				)
					.into(),
			),
			BuyExecution {
				fees: (
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(HYDRA_PARA_ID), GeneralIndex(0)),
					},
					800 * UNITS,
				)
					.into(),
				weight_limit: Unlimited,
			},
			Transact {
				require_weight_at_most: Weight::from_parts(10_000_000_000, 0u64),
				origin_kind: OriginKind::SovereignAccount,
				call: omni_sell.encode().into(),
			},
			ExpectTransactStatus(MaybeErrorCode::Success),
			RefundSurplus,
			DepositAsset {
				assets: All.into(),
				beneficiary: Junction::AccountId32 {
					id: parachain_reserve_account().into(),
					network: None,
				}
				.into(),
			},
		]);

		// Act

		assert_ok!(hydradx_runtime::PolkadotXcm::send_xcm(
			xcm_interior_at_acala,
			MultiLocation::new(1, X1(Parachain(HYDRA_PARA_ID))),
			message
		));
	});

	Hydra::execute_with(|| {
		// Assert
		assert!(hydradx_runtime::System::events().iter().any(|r| matches!(
			r.event,
			hydradx_runtime::RuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. })
		)));
		/*assert_eq!(
			hydradx_runtime::Balances::free_balance(&AccountId::from(acala_account_id_at_hydra)),
			1_000 * UNITS - UNITS
		);*/

		let dai_balance = hydradx_runtime::Currencies::free_balance(DAI, &AccountId::from(acala_account_id_at_hydra));
		assert!(dai_balance > 0);
		assert_eq!(dai_balance, 26619890727267708);
	});
}
