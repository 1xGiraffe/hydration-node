use super::*;

use crate::types::AssetType;
use frame_support::error::BadOrigin;
use mock::{AssetId, Registry};
use polkadot_xcm::v3::{
	Junction::{self, Parachain},
	Junctions::X2,
	MultiLocation,
};
use pretty_assertions::assert_eq;

#[test]
fn register_should_work_when_all_params_are_provided() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_id = 1;
		let name = b"Test asset".to_vec();
		let symbol = b"TKN".to_vec();
		let decimals = 12;
		let xcm_rate_limit = 1_000;
		let ed = 10_000;
		let is_sufficient = true;

		let key = Junction::from(BoundedVec::try_from(asset_id.encode()).unwrap());
		let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));

		//Act
		assert_ok!(Registry::register(
			RuntimeOrigin::root(),
			Some(asset_id),
			Some(name.clone()),
			AssetType::Token,
			Some(ed),
			Some(symbol.clone()),
			Some(decimals),
			Some(asset_location.clone()),
			Some(xcm_rate_limit),
			is_sufficient
		));

		//Assert
		let bounded_name = Pallet::<Test>::try_into_bounded(Some(name)).unwrap();
		let bounded_symbol = Pallet::<Test>::try_into_bounded(Some(symbol)).unwrap();
		assert_eq!(
			Registry::assets(asset_id),
			Some(AssetDetails {
				name: bounded_name.clone(),
				asset_type: AssetType::Token,
				existential_deposit: ed,
				xcm_rate_limit: Some(xcm_rate_limit),
				symbol: bounded_symbol.clone(),
				decimals: Some(decimals),
				is_sufficient
			})
		);

		assert_eq!(Registry::asset_ids(bounded_name.clone().unwrap()), Some(asset_id));

		assert_eq!(Registry::location_assets(asset_location.clone()), Some(asset_id));
		assert_eq!(Registry::locations(asset_id), Some(asset_location.clone()));

		assert!(has_event(
			Event::<Test>::Registered {
				asset_id,
				asset_name: bounded_name,
				asset_type: AssetType::Token,
				existential_deposit: ed,
				xcm_rate_limit: Some(xcm_rate_limit),
				symbol: bounded_symbol,
				decimals: Some(decimals),
				is_sufficient
			}
			.into()
		));

		assert!(has_event(
			Event::<Test>::LocationSet {
				asset_id,
				location: asset_location
			}
			.into()
		));
	});
}

#[test]
fn register_should_work_when_only_required_params_were_provided() {
	ExtBuilder::default().build().execute_with(|| {
		let expected_id = Pallet::<Test>::next_asset_id().unwrap();
		let is_sufficient = true;

		//Act
		assert_ok!(Registry::register(
			RuntimeOrigin::root(),
			None,
			None,
			AssetType::Token,
			None,
			None,
			None,
			None,
			None,
			is_sufficient
		));

		//Assert
		assert_eq!(
			Registry::assets(expected_id),
			Some(AssetDetails {
				name: None,
				asset_type: AssetType::Token,
				existential_deposit: 1,
				xcm_rate_limit: None,
				symbol: None,
				decimals: None,
				is_sufficient
			})
		);

		assert!(has_event(
			Event::<Test>::Registered {
				asset_id: expected_id,
				asset_name: None,
				asset_type: AssetType::Token,
				existential_deposit: 1,
				xcm_rate_limit: None,
				symbol: None,
				decimals: None,
				is_sufficient
			}
			.into()
		));
	});
}

#[test]
fn register_should_not_work_when_asset_id_is_not_from_reserved_range() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_id: AssetId = Pallet::<Test>::next_asset_id().unwrap();
		let name = b"Test asset".to_vec();
		let symbol = b"TKN".to_vec();
		let decimals = 12;
		let xcm_rate_limit = 1_000;
		let ed = 10_000;
		let is_sufficient = true;

		let key = Junction::from(BoundedVec::try_from(asset_id.encode()).unwrap());
		let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));

		//Act
		assert_noop!(
			Registry::register(
				RuntimeOrigin::root(),
				Some(asset_id),
				Some(name),
				AssetType::Token,
				Some(ed),
				Some(symbol),
				Some(decimals),
				Some(asset_location),
				Some(xcm_rate_limit),
				is_sufficient
			),
			Error::<Test>::NotInReservedRange
		);
	});
}

#[test]
fn register_should_not_work_when_asset_id_is_already_used() {
	ExtBuilder::default()
		.with_assets(vec![
			(Some(1), Some(b"Tkn1".to_vec()), UNIT, None, None, None, true),
			(Some(2), Some(b"Tkn2".to_vec()), UNIT, None, None, None, true),
			(Some(3), Some(b"Tkn3".to_vec()), UNIT, None, None, None, true),
		])
		.build()
		.execute_with(|| {
			let asset_id = 1;
			let name = b"Test asset".to_vec();
			let symbol = b"TKN".to_vec();
			let decimals = 12;
			let xcm_rate_limit = 1_000;
			let ed = 10_000;
			let is_sufficient = true;

			let key = Junction::from(BoundedVec::try_from(asset_id.encode()).unwrap());
			let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));

			//Act
			assert_noop!(
				Registry::register(
					RuntimeOrigin::root(),
					Some(asset_id),
					Some(name),
					AssetType::Token,
					Some(ed),
					Some(symbol),
					Some(decimals),
					Some(asset_location),
					Some(xcm_rate_limit),
					is_sufficient
				),
				Error::<Test>::AssetAlreadyRegistered
			);
		});
}

#[test]
fn register_should_not_work_when_asset_name_is_already_used() {
	ExtBuilder::default()
		.with_assets(vec![
			(Some(1), Some(b"Tkn1".to_vec()), UNIT, None, None, None, true),
			(Some(2), Some(b"Tkn2".to_vec()), UNIT, None, None, None, true),
			(Some(3), Some(b"Tkn3".to_vec()), UNIT, None, None, None, true),
		])
		.build()
		.execute_with(|| {
			let asset_id = 4;
			let name = b"Tkn3".to_vec();
			let symbol = b"TKN".to_vec();
			let decimals = 12;
			let xcm_rate_limit = 1_000;
			let ed = 10_000;
			let is_sufficient = true;

			let key = Junction::from(BoundedVec::try_from(asset_id.encode()).unwrap());
			let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));

			//Act
			assert_noop!(
				Registry::register(
					RuntimeOrigin::root(),
					Some(asset_id),
					Some(name),
					AssetType::Token,
					Some(ed),
					Some(symbol),
					Some(decimals),
					Some(asset_location),
					Some(xcm_rate_limit),
					is_sufficient
				),
				Error::<Test>::AssetAlreadyRegistered
			);
		});
}

#[test]
fn register_should_not_work_when_asset_location_is_already_used() {
	ExtBuilder::default()
		.with_assets(vec![
			(Some(1), Some(b"Tkn1".to_vec()), UNIT, None, None, None, true),
			(Some(2), Some(b"Tkn2".to_vec()), UNIT, None, None, None, true),
			(Some(3), Some(b"Tkn3".to_vec()), UNIT, None, None, None, true),
		])
		.build()
		.execute_with(|| {
			//Arrange
			let asset_id = 4;

			let key = Junction::from(BoundedVec::try_from(asset_id.encode()).unwrap());
			let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));
			Pallet::<Test>::set_location(3, asset_location.clone()).unwrap();

			let name = b"Tkn4".to_vec();
			let symbol = b"TKN".to_vec();
			let decimals = 12;
			let xcm_rate_limit = 1_000;
			let ed = 10_000;
			let is_sufficient = true;

			//Act
			assert_noop!(
				Registry::register(
					RuntimeOrigin::root(),
					Some(asset_id),
					Some(name),
					AssetType::Token,
					Some(ed),
					Some(symbol),
					Some(decimals),
					Some(asset_location),
					Some(xcm_rate_limit),
					is_sufficient
				),
				Error::<Test>::LocationAlreadyRegistered
			);
		});
}

#[test]
fn register_should_not_work_when_origin_is_none() {
	ExtBuilder::default()
		.with_assets(vec![
			(Some(1), Some(b"Tkn1".to_vec()), UNIT, None, None, None, true),
			(Some(2), Some(b"Tkn2".to_vec()), UNIT, None, None, None, true),
			(Some(3), Some(b"Tkn3".to_vec()), UNIT, None, None, None, true),
		])
		.build()
		.execute_with(|| {
			//Arrange
			let asset_id = 4;

			let name = b"Tkn4".to_vec();
			let symbol = b"TKN".to_vec();
			let decimals = 12;
			let xcm_rate_limit = 1_000;
			let ed = 10_000;
			let is_sufficient = true;

			let key = Junction::from(BoundedVec::try_from(asset_id.encode()).unwrap());
			let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));

			//Act
			assert_noop!(
				Registry::register(
					RuntimeOrigin::none(),
					Some(asset_id),
					Some(name),
					AssetType::Token,
					Some(ed),
					Some(symbol),
					Some(decimals),
					Some(asset_location),
					Some(xcm_rate_limit),
					is_sufficient
				),
				BadOrigin
			);
		});
}

#[test]
fn register_should_not_work_when_origin_is_not_allowed() {
	ExtBuilder::default()
		.with_assets(vec![
			(Some(1), Some(b"Tkn1".to_vec()), UNIT, None, None, None, true),
			(Some(2), Some(b"Tkn2".to_vec()), UNIT, None, None, None, true),
			(Some(3), Some(b"Tkn3".to_vec()), UNIT, None, None, None, true),
		])
		.build()
		.execute_with(|| {
			//Arrange
			let asset_id = 4;

			let name = b"Tkn4".to_vec();
			let symbol = b"TKN".to_vec();
			let decimals = 12;
			let xcm_rate_limit = 1_000;
			let ed = 10_000;
			let is_sufficient = true;

			let key = Junction::from(BoundedVec::try_from(asset_id.encode()).unwrap());
			let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));

			//Act
			assert_noop!(
				Registry::register(
					RuntimeOrigin::signed(ALICE),
					Some(asset_id),
					Some(name),
					AssetType::Token,
					Some(ed),
					Some(symbol),
					Some(decimals),
					Some(asset_location),
					Some(xcm_rate_limit),
					is_sufficient
				),
				BadOrigin
			);
		});
}

#[test]
fn register_external_asset_should_work_when_location_is_provided() {
	ExtBuilder::default().build().execute_with(|| {
		let expected_id = Pallet::<Test>::next_asset_id().unwrap();

		let key = Junction::from(BoundedVec::try_from(528.encode()).unwrap());
		let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));

		let alice_balance = 10_000 * UNIT;
		Tokens::set_balance(RuntimeOrigin::root(), ALICE, NativeAssetId::get(), alice_balance, 0).unwrap();
		assert_eq!(Tokens::free_balance(NativeAssetId::get(), &TREASURY), 0);

		//Act
		assert_ok!(Registry::register_external(
			RuntimeOrigin::signed(ALICE),
			asset_location.clone()
		));

		//Assert
		assert_eq!(
			Registry::assets(expected_id),
			Some(AssetDetails {
				name: None,
				asset_type: AssetType::External,
				existential_deposit: crate::DEFAULT_ED,
				xcm_rate_limit: None,
				symbol: None,
				decimals: None,
				is_sufficient: false
			})
		);

		assert_eq!(Registry::location_assets(asset_location.clone()), Some(expected_id));
		assert_eq!(Registry::locations(expected_id), Some(asset_location.clone()));

		assert!(has_event(
			Event::<Test>::Registered {
				asset_id: expected_id,
				asset_name: None,
				asset_type: AssetType::External,
				existential_deposit: crate::DEFAULT_ED,
				xcm_rate_limit: None,
				symbol: None,
				decimals: None,
				is_sufficient: false
			}
			.into()
		));

		assert!(has_event(
			Event::<Test>::LocationSet {
				asset_id: expected_id,
				location: asset_location
			}
			.into()
		));

		assert_eq!(
			Tokens::free_balance(NativeAssetId::get(), &ALICE),
			alice_balance - StoreFees::get()
		);
		assert_eq!(Tokens::free_balance(NativeAssetId::get(), &TREASURY), StoreFees::get());
	});
}

#[test]
fn register_external_asset_should_not_work_when_location_is_already_used() {
	ExtBuilder::default().build().execute_with(|| {
		//Arrange
		let asset_id = 1;
		let name = b"Test asset".to_vec();
		let symbol = b"TKN".to_vec();
		let decimals = 12;
		let xcm_rate_limit = 1_000;
		let ed = 10_000;
		let is_sufficient = true;

		let key = Junction::from(BoundedVec::try_from(asset_id.encode()).unwrap());
		let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));

		assert_ok!(Registry::register(
			RuntimeOrigin::root(),
			Some(asset_id),
			Some(name),
			AssetType::Token,
			Some(ed),
			Some(symbol),
			Some(decimals),
			Some(asset_location.clone()),
			Some(xcm_rate_limit),
			is_sufficient
		));

		let alice_balance = 10_000 * UNIT;
		Tokens::set_balance(RuntimeOrigin::root(), ALICE, NativeAssetId::get(), alice_balance, 0).unwrap();
		assert_eq!(Tokens::free_balance(NativeAssetId::get(), &TREASURY), 0);

		//Act
		assert_noop!(
			Registry::register_external(RuntimeOrigin::signed(ALICE), asset_location),
			Error::<Test>::LocationAlreadyRegistered
		);
	});
}

#[test]
fn register_external_asset_should_not_work_when_user_cant_pay_storage_fees() {
	ExtBuilder::default().build().execute_with(|| {
		let key = Junction::from(BoundedVec::try_from(528.encode()).unwrap());
		let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));

		let alice_balance = 10_000 * UNIT;
		Tokens::set_balance(
			RuntimeOrigin::root(),
			ALICE,
			NativeAssetId::get(),
			StoreFees::get() - 1,
			alice_balance,
		)
		.unwrap();

		//Act
		assert_noop!(
			Registry::register_external(RuntimeOrigin::signed(ALICE), asset_location),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn register_should_fail_when_name_is_too_long() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_id = 1;
		let name = vec![97u8; <Test as crate::Config>::StringLimit::get() as usize + 1];
		let symbol = b"TKN".to_vec();
		let decimals = 12;
		let xcm_rate_limit = 1_000;
		let ed = 10_000;
		let is_sufficient = true;

		let key = Junction::from(BoundedVec::try_from(asset_id.encode()).unwrap());
		let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));

		//Act
		assert_noop!(Registry::register(
			RuntimeOrigin::root(),
			Some(asset_id),
			Some(name.clone()),
			AssetType::Token,
			Some(ed),
			Some(symbol.clone()),
			Some(decimals),
			Some(asset_location.clone()),
			Some(xcm_rate_limit),
			is_sufficient
		), Error::<Test>::TooLong);
	});
}

#[test]
fn register_should_fail_when_symbol_is_too_long() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_id = 1;
		let name = b"Test asset".to_vec();
		let symbol = vec![97u8; <Test as crate::Config>::StringLimit::get() as usize + 1];
		let decimals = 12;
		let xcm_rate_limit = 1_000;
		let ed = 10_000;
		let is_sufficient = true;

		let key = Junction::from(BoundedVec::try_from(asset_id.encode()).unwrap());
		let asset_location = AssetLocation(MultiLocation::new(0, X2(Parachain(200), key)));

		//Act
		assert_noop!(Registry::register(
			RuntimeOrigin::root(),
			Some(asset_id),
			Some(name.clone()),
			AssetType::Token,
			Some(ed),
			Some(symbol.clone()),
			Some(decimals),
			Some(asset_location.clone()),
			Some(xcm_rate_limit),
			is_sufficient
		), Error::<Test>::TooLong);
	});
}
