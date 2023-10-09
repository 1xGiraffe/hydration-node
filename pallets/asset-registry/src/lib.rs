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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchError;
use frame_support::pallet_prelude::*;
use frame_support::sp_runtime::traits::CheckedAdd;
use frame_system::pallet_prelude::*;
use orml_traits::MultiCurrency;
use scale_info::TypeInfo;
use sp_arithmetic::traits::{BaseArithmetic, Zero};
use sp_std::convert::TryInto;
use sp_std::vec::Vec;

#[cfg(test)]
mod tests;

mod benchmarking;
//TODO
//pub mod migration;
mod types;
pub mod weights;

use weights::WeightInfo;

pub use types::AssetType;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

pub use crate::types::{AssetDetails, Balance};
use frame_support::BoundedVec;
use hydradx_traits::{
	registry::{Create, Inspect, Mutate},
	AssetKind, CreateRegistry, Registry, ShareTokenRegistry,
};

/// Default value of existential deposit. This value is used if existential deposit wasn't
/// provided.
pub const DEFAULT_ED: Balance = 1;

#[frame_support::pallet]
#[allow(clippy::too_many_arguments)]
pub mod pallet {
	use super::*;

	pub type AssetDetailsT<T> = AssetDetails<<T as Config>::AssetId, <T as Config>::StringLimit>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The origin which can work with asset-registry.
		type RegistryOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The origin which can update assets' detail.
		type UpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Asset type
		type AssetId: Parameter
			+ Member
			+ Default
			+ Copy
			+ BaseArithmetic
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;

		/// Asset location type
		type AssetNativeLocation: Parameter + Member + Default + MaxEncodedLen;

		/// Multi currency mechanism
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = Self::AssetId, Balance = Balance>;

		#[pallet::constant]
		type SequentialIdStartAt: Get<Self::AssetId>;

		/// Native Asset Id
		#[pallet::constant]
		type NativeAssetId: Get<Self::AssetId>;

		/// Storage fees for external asset creation.
		#[pallet::constant]
		type StorageFees: Get<Balance>;

		/// Storage fees for external asset creation.
		#[pallet::constant]
		type StorageFeesBeneficiary: Get<Self::AccountId>;

		/// The maximum length of a name or symbol stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;

		/// Weight information for the extrinsics
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::error]
	pub enum Error<T> {
		/// Asset ID is not available. This only happens when it reaches the MAX value of given id type.
		NoIdAvailable,

		/// Invalid asset name or symbol.
		AssetNotFound,

		/// Invalid asset name or symbol.
		TooLong,

		/// Asset ID is not registered in the asset-registry.
		AssetNotRegistered,

		/// Asset is already registered.
		AssetAlreadyRegistered,

		/// Incorrect number of assets provided to create shared asset.
		InvalidSharedAssetLen,

		/// Cannot update asset location
		CannotUpdateLocation,

		/// Selected asset id is out of reserved range.
		NotInReservedRange,

		/// Location already registered with different asset
		LocationAlreadyRegistered,

		/// Origin is forbidden to set/update value
		Forbidden,

		/// Balance too low
		InsufficientBalance,
	}

	#[pallet::type_value]
	/// Default value of NextAssetId if storage is empty. 1 is used to offset for native token
	/// which id is 0.
	pub fn DefaultNextAssetId<T: Config>() -> T::AssetId {
		1.into()
	}

	#[pallet::storage]
	#[pallet::getter(fn assets)]
	/// Details of an asset.
	pub type Assets<T: Config> = StorageMap<_, Twox64Concat, T::AssetId, AssetDetailsT<T>, OptionQuery>;

	#[pallet::storage]
	/// Next available asset id. This is sequential id assigned for each new registered asset.
	pub type NextAssetId<T: Config> = StorageValue<_, T::AssetId, ValueQuery, DefaultNextAssetId<T>>;

	#[pallet::storage]
	#[pallet::getter(fn asset_ids)]
	/// Mapping between asset name and asset id.
	pub type AssetIds<T: Config> =
		StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::StringLimit>, T::AssetId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn locations)]
	/// Native location of an asset.
	pub type AssetLocations<T: Config> = StorageMap<_, Twox64Concat, T::AssetId, T::AssetNativeLocation, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn location_assets)]
	/// Local asset for native location.
	pub type LocationAssets<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetNativeLocation, T::AssetId, OptionQuery>;

	#[allow(clippy::type_complexity)]
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		//asset_id, name, existential deposit, symbol, decimals, xcm_rate_limit, is_sufficient
		pub registered_assets: Vec<(
			Option<T::AssetId>,
			Option<Vec<u8>>,
			Balance,
			Option<Vec<u8>>,
			Option<u8>,
			Option<Balance>,
			bool,
		)>,
		pub native_asset_name: Vec<u8>,
		pub native_existential_deposit: Balance,
		pub native_symbol: Vec<u8>,
		pub native_decimals: u8,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig::<T> {
				registered_assets: vec![],
				native_asset_name: b"HDX".to_vec(),
				native_existential_deposit: DEFAULT_ED,
				native_symbol: b"HDX".to_vec(),
				native_decimals: 12,
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// Register native asset first
			// It is to make sure that native is registered as any other asset
			let native_asset_name = Pallet::<T>::to_bounded_name(self.native_asset_name.to_vec())
				.map_err(|_| panic!("Invalid native asset name!"))
				.unwrap();

			let native_symbol = Pallet::<T>::to_bounded_name(self.native_symbol.to_vec())
				.map_err(|_| panic!("Invalid native asset symbol!"))
				.unwrap();

			AssetIds::<T>::insert(&native_asset_name, T::NativeAssetId::get());
			let details = AssetDetails {
				name: Some(native_asset_name),
				asset_type: AssetType::Token,
				existential_deposit: self.native_existential_deposit,
				xcm_rate_limit: None,
				symbol: Some(native_symbol),
				decimals: Some(self.native_decimals),
				is_sufficient: true,
			};

			Assets::<T>::insert(T::NativeAssetId::get(), details);

			self.registered_assets
				.iter()
				.for_each(|(id, name, ed, symbol, decimals, xcm_rate_limit, is_sufficient)| {
					let bounded_name = name.as_ref().map(|name| {
						Pallet::<T>::to_bounded_name(name.to_vec())
							.map_err(|_| panic!("Invalid asset name!"))
							.unwrap()
					});
					let bounded_symbol = symbol.as_ref().map(|symbol| {
						Pallet::<T>::to_bounded_name(symbol.to_vec())
							.map_err(|_| panic!("Invalid symbol!"))
							.unwrap()
					});

					let details = AssetDetails {
						name: bounded_name,
						asset_type: AssetType::Token,
						existential_deposit: *ed,
						xcm_rate_limit: *xcm_rate_limit,
						symbol: bounded_symbol,
						decimals: *decimals,
						is_sufficient: *is_sufficient,
					};
					let _ = Pallet::<T>::do_register_asset(*id, &details, None)
						.map_err(|_| panic!("Failed to register asset"));
				})
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Asset was registered.
		Registered {
			asset_id: T::AssetId,
			asset_name: Option<BoundedVec<u8, T::StringLimit>>,
			asset_type: AssetType<T::AssetId>,
			existential_deposit: Balance,
			xcm_rate_limit: Option<Balance>,
			symbol: Option<BoundedVec<u8, T::StringLimit>>,
			decimals: Option<u8>,
			is_sufficient: bool,
		},

		/// Asset was updated.
		Updated {
			asset_id: T::AssetId,
			asset_name: Option<BoundedVec<u8, T::StringLimit>>,
			asset_type: AssetType<T::AssetId>,
			existential_deposit: Balance,
			xcm_rate_limit: Option<Balance>,
			symbol: Option<BoundedVec<u8, T::StringLimit>>,
			decimals: Option<u8>,
			is_sufficient: bool,
		},

		/// Native location set for an asset.
		LocationSet {
			asset_id: T::AssetId,
			location: T::AssetNativeLocation,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new asset.
		///
		/// New asset is given `NextAssetId` - sequential asset id
		///
		/// Asset's id is optional and it can't be used by another asset if it's provided.
		/// Provided `asset_id` must be from within reserved range.
		/// If `asset_id` is `None`, new asset is given id for sequential ids.
		///
		/// Asset's name is optional and it can't be used by another asset if it's provided.
		/// Adds mapping between `name` and assigned `asset_id` so asset id can be retrieved by name too (Note: this approach is used in AMM implementation (xyk))
		///
		/// Emits 'Registered` event when successful.
		#[allow(clippy::too_many_arguments)]
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::register())]
		pub fn register(
			origin: OriginFor<T>,
			asset_id: Option<T::AssetId>,
			name: Option<Vec<u8>>,
			asset_type: AssetType<T::AssetId>,
			existential_deposit: Option<Balance>,
			symbol: Option<Vec<u8>>,
			decimals: Option<u8>,
			location: Option<T::AssetNativeLocation>,
			xcm_rate_limit: Option<Balance>,
			is_sufficient: bool,
		) -> DispatchResult {
			T::RegistryOrigin::ensure_origin(origin)?;

			let bounded_name = if let Some(name) = name {
				let bounded_name = Self::to_bounded_name(name)?;
				Some(bounded_name)
			} else {
				None
			};

			let bounded_symbol = if let Some(symbol) = symbol {
				Some(Self::to_bounded_name(symbol)?)
			} else {
				None
			};

			let details = AssetDetails::new(
				bounded_name,
				asset_type,
				existential_deposit.unwrap_or(DEFAULT_ED),
				bounded_symbol,
				decimals,
				xcm_rate_limit,
				is_sufficient,
			);

			Self::do_register_asset(asset_id, &details, location)?;
			Ok(())
		}

		/// Update registered asset.
		///
		/// All parameteres are optional and value is not updated if param is `None`.
		///
		/// `decimals` - can be update by `UpdateOrigin` only if it wasn't set yet. Only
		/// `RegistryOrigin` can update `decimals` if it was previously set.
		///
		/// `location` - can be updated only by `RegistryOrigin`.
		///
		/// Emits `Updated` event when successful.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::update())]
		pub fn update(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			name: Option<Vec<u8>>,
			asset_type: Option<AssetType<T::AssetId>>,
			existential_deposit: Option<Balance>,
			xcm_rate_limit: Option<Balance>,
			is_sufficient: Option<bool>,
			symbol: Option<Vec<u8>>,
			decimals: Option<u8>,
			location: Option<T::AssetNativeLocation>,
		) -> DispatchResult {
			let is_registry_origin = match T::RegistryOrigin::ensure_origin(origin.clone()) {
				Ok(_) => true,
				Err(_) => {
					T::UpdateOrigin::ensure_origin(origin)?;

					false
				}
			};

			Assets::<T>::try_mutate(asset_id, |maybe_detail| -> DispatchResult {
				let mut details = maybe_detail.as_mut().ok_or(Error::<T>::AssetNotFound)?;

				let new_bounded_name = if let Some(n) = name {
					let new_name = Self::to_bounded_name(n)?;
					ensure!(Self::asset_ids(&new_name).is_none(), Error::<T>::AssetAlreadyRegistered);

					if let Some(old_name) = &details.name {
						AssetIds::<T>::remove(old_name);
					}

					if Some(new_name.clone()) != details.name {
						AssetIds::<T>::insert(&new_name, asset_id);
					}

					Some(new_name)
				} else {
					None
				};

				let bounded_symbol = if let Some(s) = symbol {
					Some(Self::to_bounded_name(s)?)
				} else {
					None
				};

				details.name = new_bounded_name.or_else(|| details.name.clone());
				details.asset_type = asset_type.unwrap_or(details.asset_type);
				details.existential_deposit = existential_deposit.unwrap_or(details.existential_deposit);
				details.xcm_rate_limit = xcm_rate_limit.or(details.xcm_rate_limit);
				details.is_sufficient = is_sufficient.unwrap_or(details.is_sufficient);
				details.symbol = bounded_symbol.or_else(|| details.symbol.clone());

				if decimals.is_some() {
					if details.decimals.is_none() {
						details.decimals = decimals;
					} else {
						//Only highest origin can change decimal if it was set previously.
						ensure!(is_registry_origin, Error::<T>::Forbidden);
						details.decimals = decimals;
					};
				}

				if let Some(loc) = location {
					//Only highest origin can update location.
					ensure!(is_registry_origin, Error::<T>::Forbidden);

					if let Some(old_location) = AssetLocations::<T>::take(asset_id) {
						LocationAssets::<T>::remove(&old_location);
					}
					Self::do_set_location(asset_id, loc)?;
				}

				Self::deposit_event(Event::Updated {
					asset_id,
					asset_name: details.name.clone(),
					asset_type: details.asset_type,
					existential_deposit: details.existential_deposit,
					xcm_rate_limit: details.xcm_rate_limit,
					symbol: details.symbol.clone(),
					decimals: details.decimals,
					is_sufficient: details.is_sufficient,
				});

				Ok(())
			})
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::register_external())]
		pub fn register_external(origin: OriginFor<T>, location: T::AssetNativeLocation) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if !T::StorageFees::get().is_zero() {
				ensure!(
					T::Currency::ensure_can_withdraw(T::NativeAssetId::get(), &who, T::StorageFees::get()).is_ok(),
					Error::<T>::InsufficientBalance
				);

				T::Currency::transfer(
					T::NativeAssetId::get(),
					&who,
					&T::StorageFeesBeneficiary::get(),
					T::StorageFees::get(),
				)?;
			}

			Self::do_register_asset(
				None,
				&AssetDetails::new(None, AssetType::External, DEFAULT_ED, None, None, None, false),
				Some(location),
			)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn next_asset_id() -> Option<T::AssetId> {
		NextAssetId::<T>::get().checked_add(&T::SequentialIdStartAt::get())
	}

	/// Convert Vec<u8> to BoundedVec so it respects the max set limit, otherwise return TooLong error
	//TODO: remove pub
	pub fn to_bounded_name(name: Vec<u8>) -> Result<BoundedVec<u8, T::StringLimit>, Error<T>> {
		name.try_into().map_err(|_| Error::<T>::TooLong)
	}

	fn do_set_location(asset_id: T::AssetId, location: T::AssetNativeLocation) -> Result<(), DispatchError> {
		ensure!(
			Self::location_assets(&location).is_none(),
			Error::<T>::LocationAlreadyRegistered
		);

		AssetLocations::<T>::insert(asset_id, &location);
		LocationAssets::<T>::insert(&location, asset_id);

		Self::deposit_event(Event::LocationSet { asset_id, location });

		Ok(())
	}

	fn do_register_asset(
		selected_asset_id: Option<T::AssetId>,
		details: &AssetDetails<T::AssetId, T::StringLimit>,
		location: Option<T::AssetNativeLocation>,
	) -> Result<T::AssetId, DispatchError> {
		let asset_id = if let Some(id) = selected_asset_id {
			ensure!(id < T::SequentialIdStartAt::get(), Error::<T>::NotInReservedRange);

			ensure!(!Assets::<T>::contains_key(id), Error::<T>::AssetAlreadyRegistered);

			id
		} else {
			NextAssetId::<T>::mutate(|value| -> Result<T::AssetId, DispatchError> {
				let next_asset_id = *value;
				*value = next_asset_id
					.checked_add(&T::AssetId::from(1))
					.ok_or(Error::<T>::NoIdAvailable)?;

				Ok(next_asset_id
					.checked_add(&T::SequentialIdStartAt::get())
					.ok_or(Error::<T>::NoIdAvailable)?)
			})?
		};

		Assets::<T>::insert(asset_id, details);
		if let Some(name) = details.name.as_ref() {
			ensure!(!AssetIds::<T>::contains_key(name), Error::<T>::AssetAlreadyRegistered);
			AssetIds::<T>::insert(name, asset_id);
		}

		if let Some(loc) = location {
			Self::do_set_location(asset_id, loc)?;
		}

		Self::deposit_event(Event::Registered {
			asset_id,
			asset_name: details.name.clone(),
			asset_type: details.asset_type,
			existential_deposit: details.existential_deposit,
			xcm_rate_limit: details.xcm_rate_limit,
			symbol: details.symbol.clone(),
			decimals: details.decimals,
			is_sufficient: details.is_sufficient,
		});

		Ok(asset_id)
	}

	/// Create asset for given name or return existing AssetId if such asset already exists.
	pub fn get_or_create_asset(
		name: Vec<u8>,
		asset_type: AssetType<T::AssetId>,
		existential_deposit: Balance,
		asset_id: Option<T::AssetId>,
		is_sufficient: bool,
	) -> Result<T::AssetId, DispatchError> {
		let bounded_name: BoundedVec<u8, T::StringLimit> = Self::to_bounded_name(name)?;

		if let Some(asset_id) = AssetIds::<T>::get(&bounded_name) {
			Ok(asset_id)
		} else {
			Self::do_register_asset(
				asset_id,
				&AssetDetails::new(
					Some(bounded_name),
					asset_type,
					existential_deposit,
					None,
					None,
					None,
					is_sufficient,
				),
				None,
			)
		}
	}

	/// Return location for given asset.
	pub fn asset_to_location(asset_id: T::AssetId) -> Option<T::AssetNativeLocation> {
		Self::locations(asset_id)
	}

	/// Return asset for given loation.
	pub fn location_to_asset(location: T::AssetNativeLocation) -> Option<T::AssetId> {
		Self::location_assets(location)
	}
}

impl<T: Config> Registry<T::AssetId, Vec<u8>, Balance, DispatchError> for Pallet<T> {
	fn exists(asset_id: T::AssetId) -> bool {
		Assets::<T>::contains_key(asset_id)
	}

	fn retrieve_asset(name: &Vec<u8>) -> Result<T::AssetId, DispatchError> {
		let bounded_name = Self::to_bounded_name(name.clone())?;
		if let Some(asset_id) = AssetIds::<T>::get(bounded_name) {
			Ok(asset_id)
		} else {
			Err(Error::<T>::AssetNotFound.into())
		}
	}

	fn retrieve_asset_type(asset_id: T::AssetId) -> Result<AssetKind, DispatchError> {
		let asset_details =
			Assets::<T>::get(asset_id).ok_or_else(|| Into::<DispatchError>::into(Error::<T>::AssetNotFound))?;
		Ok(asset_details.asset_type.into())
	}

	fn create_asset(name: &Vec<u8>, existential_deposit: Balance) -> Result<T::AssetId, DispatchError> {
		Self::get_or_create_asset(name.clone(), AssetType::Token, existential_deposit, None, false)
	}
}

impl<T: Config> ShareTokenRegistry<T::AssetId, Vec<u8>, Balance, DispatchError> for Pallet<T> {
	fn retrieve_shared_asset(name: &Vec<u8>, _assets: &[T::AssetId]) -> Result<T::AssetId, DispatchError> {
		Self::retrieve_asset(name)
	}

	fn create_shared_asset(
		name: &Vec<u8>,
		assets: &[T::AssetId],
		existential_deposit: Balance,
	) -> Result<T::AssetId, DispatchError> {
		ensure!(assets.len() == 2, Error::<T>::InvalidSharedAssetLen);
		Self::get_or_create_asset(
			name.clone(),
			AssetType::PoolShare(assets[0], assets[1]),
			existential_deposit,
			None,
			false,
		)
	}
}

use orml_traits::GetByKey;

// Return Existential deposit of an asset
impl<T: Config> GetByKey<T::AssetId, Balance> for Pallet<T> {
	fn get(k: &T::AssetId) -> Balance {
		if let Some(details) = Self::assets(k) {
			details.existential_deposit
		} else {
			// Asset does not exist - not supported
			Balance::max_value()
		}
	}
}

/// Allows querying the XCM rate limit for an asset by its id.
pub struct XcmRateLimitsInRegistry<T>(PhantomData<T>);
/// Allows querying the XCM rate limit for an asset by its id.
/// Both a unknown asset and an unset rate limit will return `None`.
impl<T: Config> GetByKey<T::AssetId, Option<Balance>> for XcmRateLimitsInRegistry<T> {
	fn get(k: &T::AssetId) -> Option<Balance> {
		Pallet::<T>::assets(k).and_then(|details| details.xcm_rate_limit)
	}
}

impl<T: Config> CreateRegistry<T::AssetId, Balance> for Pallet<T> {
	type Error = DispatchError;

	fn create_asset(name: &[u8], kind: AssetKind, existential_deposit: Balance) -> Result<T::AssetId, Self::Error> {
		let bounded_name = Some(Self::to_bounded_name(name.to_vec())?);

		Pallet::<T>::do_register_asset(
			None,
			&AssetDetails::new(bounded_name, kind.into(), existential_deposit, None, None, None, false),
			None,
		)
	}
}

impl<T: Config> Inspect<T::AssetNativeLocation, Balance> for Pallet<T> {
	type Error = DispatchError;
	type AssetId = T::AssetId;

	fn is_sufficient(id: Self::AssetId) -> bool {
		match Self::assets(id) {
			Some(a) => a.is_sufficient,
			None => false,
		}
	}
}

impl<T: Config> Mutate<T::AssetNativeLocation, Balance> for Pallet<T> {
	fn set_location(asset_id: Self::AssetId, location: T::AssetNativeLocation) -> Result<(), Self::Error> {
		Self::do_set_location(asset_id, location)
	}
}

impl<T: Config> Create<T::AssetNativeLocation, Balance> for Pallet<T> {
	fn register_asset(
		asset_id: Option<Self::AssetId>,
		name: Option<&[u8]>,
		kind: AssetKind,
		existential_deposit: Option<Balance>,
		symbol: Option<&[u8]>,
		decimals: Option<u8>,
		location: Option<T::AssetNativeLocation>,
		xcm_rate_limit: Option<Balance>,
		is_sufficient: bool,
	) -> Result<Self::AssetId, Self::Error> {
		let bounded_name = if let Some(name) = name {
			let bounded_name = Self::to_bounded_name(name.to_vec())?;
			Some(bounded_name)
		} else {
			None
		};

		let bounded_symbol = if let Some(symbol) = symbol {
			Some(Self::to_bounded_name(symbol.to_vec())?)
		} else {
			None
		};

		let details = AssetDetails::new(
			bounded_name,
			kind.into(),
			existential_deposit.unwrap_or(DEFAULT_ED),
			bounded_symbol,
			decimals,
			xcm_rate_limit,
			is_sufficient,
		);

		Self::do_register_asset(asset_id, &details, location)
	}
}

use hydradx_traits::InspectRegistry;
impl<T: Config> InspectRegistry<T::AssetId> for Pallet<T> {
	fn exists(asset_id: T::AssetId) -> bool {
		Assets::<T>::contains_key(asset_id)
	}

	fn decimals(asset_id: T::AssetId) -> Option<u8> {
		Assets::<T>::get(asset_id)?.decimals
	}
}
