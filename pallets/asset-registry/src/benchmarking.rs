// This file is part of pallet-asset-registry.

// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::types::AssetDetails;
use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use orml_traits::MultiCurrencyExtended;

const UNIT: u128 = 1_000_000_000_000;

benchmarks! {
	 where_clause { where
		T::Currency: MultiCurrencyExtended<T::AccountId, Amount=i128>,
		T: crate::pallet::Config,
	}

	register {
		let asset_id= T::AssetId::from(3);
        let name = vec![97u8; T::StringLimit::get() as usize];
		let ed = 1_000_000_u128;
		let symbol = vec![97u8; T::StringLimit::get() as usize];
		let decimals = 12_u8;
		let location: T::AssetNativeLocation = Default::default();
		let xcm_rate_limit = 1_000_u128;
		let is_sufficient = true;

	}: _(RawOrigin::Root, Some(asset_id), Some(name.clone()), AssetType::Token, Some(ed), Some(symbol), Some(decimals), Some(location), Some(xcm_rate_limit), is_sufficient)
	verify {
		let b_name = Pallet::<T>::try_into_bounded(Some(name)).unwrap().unwrap();
		assert!(Pallet::<T>::asset_ids(b_name).is_some());

		assert!(Pallet::<T>::assets(asset_id).is_some());
	}

	update {
		let asset_id = T::AssetId::from(3);
		let name = vec![97u8; T::StringLimit::get() as usize];
		let ed = 1_000_000_u128;
		let symbol = vec![97u8; T::StringLimit::get() as usize];
		let decimals = 12_u8;
		let location: T::AssetNativeLocation = Default::default();
		let xcm_rate_limit = 1_000_u128;
		let is_sufficient = true;

		let _ = Pallet::<T>::register(RawOrigin::Root.into(), Some(asset_id), Some(name), AssetType::Token, Some(ed), Some(symbol), Some(decimals), Some(location), Some(xcm_rate_limit), is_sufficient);

		let new_name= vec![98u8; T::StringLimit::get() as usize];
		let new_type = AssetType::PoolShare(T::AssetId::from(10u8),T::AssetId::from(20u8));
		let new_ed = 1_000_000_u128;
		let new_xcm_rate_limit = 1_000_u128;
		let new_is_sufficient = false;
		let new_symbol = vec![98u8; T::StringLimit::get() as usize];
		let new_decimals = 12_u8;


	}: _(RawOrigin::Root, asset_id, Some(new_name.clone()), Some(new_type), Some(new_ed), Some(new_xcm_rate_limit), Some(new_is_sufficient), Some(new_symbol.clone()), Some(new_decimals), Some(Default::default()))
	verify {
		let b_name = Pallet::<T>::try_into_bounded(Some(new_name)).unwrap().unwrap();
		let b_symbol = Pallet::<T>::try_into_bounded(Some(new_symbol)).unwrap().unwrap();

		assert_eq!(Pallet::<T>::asset_ids(&b_name), Some(asset_id));

		assert_eq!(crate::Pallet::<T>::assets(asset_id), Some(AssetDetails {
			name: Some(b_name),
			asset_type: new_type,
			existential_deposit: new_ed,
			symbol: Some(b_symbol),
			decimals: Some(new_decimals),
			xcm_rate_limit: Some(xcm_rate_limit),
			is_sufficient: new_is_sufficient,
		}));
	}

	register_external {
		let caller: T::AccountId = account("caller", 0, 1);
		T::Currency::update_balance(T::NativeAssetId::get(), &caller, (100_000 * UNIT) as i128)?;

		let expected_asset_id = Pallet::<T>::next_asset_id().unwrap();
		let location: T::AssetNativeLocation = Default::default();

		assert!(Pallet::<T>::location_assets(location.clone()).is_none());
	}: _(RawOrigin::Signed(caller), location.clone())
	verify {
		assert_eq!(Pallet::<T>::locations(expected_asset_id), Some(location.clone()));
		assert_eq!(Pallet::<T>::location_assets(location), Some(expected_asset_id));

	}

	impl_benchmark_test_suite!(Pallet, crate::tests::mock::ExtBuilder::default().build(), crate::tests::mock::Test);
}
