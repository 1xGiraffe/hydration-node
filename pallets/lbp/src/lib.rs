#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::sp_runtime::{
	app_crypto::sp_core::crypto::UncheckedFrom,
	traits::{Hash, Zero},
};
use frame_support::{dispatch::DispatchResult, ensure, traits::Get, transactional};
use frame_system::ensure_signed;
use orml_traits::{MultiCurrency, MultiCurrencyExtended, MultiReservableCurrency};
use primitives::{asset::AssetPair, Amount, AssetId, Balance};
use sp_runtime::{RuntimeDebug,
	traits::{CheckedAdd, CheckedSub}};
use sp_std::{marker::PhantomData, vec, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarking;

pub mod weights;
use weights::WeightInfo;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq)]
pub enum CurveType {
	Constant,
	Linear,
}

impl Default for CurveType {
	fn default() -> Self {
		CurveType::Linear
	}
}

type PoolId<T> = <T as frame_system::Config>::AccountId;
type LBPWeight = u128;
// type BalanceInfo = (Balance, AssetId);
// type AssetParams = (AssetId, Weight, Balance);
// type AssetWeights = (Weight, Weight);
// type AssetBalances = (Balance, Balance);

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, Encode, Decode, Copy, Clone, PartialEq, Eq, Default)]
pub struct Pool<BlockNumber> {
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub initial_weights: (LBPWeight, LBPWeight),
	pub final_weights: (LBPWeight, LBPWeight),
	pub curve: CurveType,
	pub pausable: bool,
}

type BalanceOf<T> = <<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::OriginFor;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Multi currency for transfer of currencies
		type MultiCurrency: MultiCurrencyExtended<Self::AccountId, CurrencyId = AssetId, Amount = Amount>
			+ MultiReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		/// Native Asset Id
		type NativeAssetId: Get<AssetId>;

		/// Mapping of asset pairs to unique pool identities
		type AssetPairPoolId: AssetPairPoolIdFor<AssetId, PoolId<Self>>;

		#[pallet::constant]
		/// Trading fee rate
		type PoolDeposit: Get<BalanceOf<Self>>;

		#[pallet::constant]
		/// Trading fee rate
		type SwapFee: Get<BalanceOf<Self>>;

		/// Weight information for the extrinsics.
		type WeightInfo: WeightInfo;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::error]
	pub enum Error<T> {
		/// Create pool errors
		CannotCreatePoolWithSameAssets,
		CannotCreatePoolWithZeroLiquidity,

		/// Update pool errors
		NotOwner,
		SaleStarted,
		SaleNotEnded,
		InvalidData,

		/// Add / Remove liquidity errors
		CannotAddZeroLiquidity,
		CannotRemoveZeroLiquidity,
		LiquidityOverflow,
		LiquidityUnderflow,

		/// Balance errors
		InsufficientAssetBalance,

		/// Pool existence errors
		TokenPoolNotFound,
		TokenPoolAlreadyExists,

		/// Calculation errors
		BlockNumberInvalid,
		MaxWeightExceeded,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Create new LBP pool
		/// who, asset a, asset b, amount_a, amount_b
		CreatePool(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Update the LBP pool
		/// who, pool id
		UpdatePool(T::AccountId, PoolId<T>),

		/// Add liquidity to the pool
		/// who, asset_a, asset_b, amount_a, amount_b
		AddLiquidity(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Remove liquidity from the pool
		/// who, asset_a, asset_b, shares
		RemoveLiquidity(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Destroy LBP pool
		/// who, asset a, asset b
		PoolDestroyed(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Sell token
		/// who, asset in, asset out, amount, sale price
		Sell(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		/// Buy token
		/// who, asset out, asset in, amount, buy price
		Buy(T::AccountId, AssetId, AssetId, BalanceOf<T>, BalanceOf<T>),

		Paused(T::AccountId),
		Unpaused(T::AccountId),
		WeightsUpdated(PoolId<T>, LBPWeight, LBPWeight),
	}

	#[pallet::storage]
	#[pallet::getter(fn pool_owner)]
	pub type PoolOwner<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_deposit)]
	pub type PoolDeposit<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, BalanceOf<T>, ValueQuery>;

	/// Asset pair for each pool. Assets are stored unordered.
	#[pallet::storage]
	#[pallet::getter(fn pool_assets)]
	pub type PoolAssets<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, (AssetId, AssetId), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_data)]
	pub type PoolData<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, Pool<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pool_balances)]
	pub type PoolBalances<T: Config> = StorageMap<_, Blake2_128Concat, PoolId<T>, (BalanceOf<T>, BalanceOf<T>), ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::create_pool())]
		#[transactional]
		pub fn create_pool(
			origin: OriginFor<T>,
			asset_a: AssetId,
			amount_a: BalanceOf<T>,
			asset_b: AssetId,
			amount_b: BalanceOf<T>,
			pool_data: Pool<T::BlockNumber>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				!amount_a.is_zero() || !amount_b.is_zero(),
				Error::<T>::CannotCreatePoolWithZeroLiquidity
			);

			ensure!(asset_a != asset_b, Error::<T>::CannotCreatePoolWithSameAssets);

			let asset_pair = AssetPair {
				asset_in: asset_a,
				asset_out: asset_b,
			};

			ensure!(!Self::exists(asset_pair), Error::<T>::TokenPoolAlreadyExists);

			ensure!(
				T::MultiCurrency::free_balance(asset_a, &who) >= amount_a,
				Error::<T>::InsufficientAssetBalance
			);

			ensure!(
				T::MultiCurrency::free_balance(asset_b, &who) >= amount_b,
				Error::<T>::InsufficientAssetBalance
			);

			Self::validate_pool_data(&pool_data)?;

			let pool_id = Self::get_pair_id(asset_pair);

			let deposit = T::PoolDeposit::get();

			T::MultiCurrency::reserve(T::NativeAssetId::get(), &who, deposit)?;
			<PoolDeposit<T>>::insert(&pool_id, &deposit);

			<PoolOwner<T>>::insert(&pool_id, &who);
			<PoolAssets<T>>::insert(&pool_id, &(asset_a, asset_b));
			<PoolData<T>>::insert(&pool_id, &pool_data);

			T::MultiCurrency::transfer(asset_a, &who, &pool_id, amount_a)?;
			T::MultiCurrency::transfer(asset_b, &who, &pool_id, amount_b)?;

			<PoolBalances<T>>::insert(&pool_id, &(amount_a, amount_b));

			Self::deposit_event(Event::CreatePool(who, asset_a, asset_b, amount_a, amount_b));

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::update_pool_data())]
		#[transactional]
		pub fn update_pool_data(
			origin: OriginFor<T>,
			pool_id: PoolId<T>,
			start: Option<T::BlockNumber>,
			end: Option<T::BlockNumber>,
			initial_weights: Option<(LBPWeight, LBPWeight)>,
			final_weights: Option<(LBPWeight, LBPWeight)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::test_pool_ownership(&who, &pool_id)?;

			let mut pool_data = Self::pool_data(&pool_id);

			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(pool_data.start == Zero::zero() || now < pool_data.start,
				Error::<T>::SaleStarted);


			if let Some(new_start) = start {
				pool_data.start = new_start;
			}

			if let Some(new_end) = end {
				pool_data.end = new_end;
			}

			if let Some((w1, w2)) = initial_weights {
				pool_data.initial_weights = (w1, w2);
			}

			if let Some((w1, w2)) = final_weights {
				pool_data.final_weights = (w1, w2);
			}

			Self::validate_pool_data(&pool_data)?;

			<PoolData<T>>::insert(&pool_id, &pool_data);
			Self::deposit_event(Event::UpdatePool(who, pool_id));

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolId<T>,
			amount_a: BalanceOf<T>,
			amount_b: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::test_pool_ownership(&who, &pool_id)?;

			ensure!(
				!amount_a.is_zero() || !amount_b.is_zero(),
				Error::<T>::CannotAddZeroLiquidity
			);

			let (mut reserve_a, mut reserve_b) = Self::pool_balances(&pool_id);
			let (asset_a, asset_b) = Self::pool_assets(&pool_id);

			let pool_data = Self::pool_data(&pool_id);

			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(pool_data.start == Zero::zero() || now < pool_data.start,
				Error::<T>::SaleStarted);

			if !amount_a.is_zero() {
				ensure!(
					T::MultiCurrency::free_balance(asset_a, &who) >= amount_a,
					Error::<T>::InsufficientAssetBalance
				);

				reserve_a = reserve_a
				.checked_add(&amount_a)
				.ok_or(Error::<T>::LiquidityOverflow)?;
			}

			if !amount_b.is_zero() {
				ensure!(
					T::MultiCurrency::free_balance(asset_b, &who) >= amount_b,
					Error::<T>::InsufficientAssetBalance
				);

				reserve_b = reserve_b
				.checked_add(&amount_b)
				.ok_or(Error::<T>::LiquidityOverflow)?;
			}

			T::MultiCurrency::transfer(asset_a, &who, &pool_id, amount_a)?;
			T::MultiCurrency::transfer(asset_b, &who, &pool_id, amount_b)?;

			<PoolBalances<T>>::insert(&pool_id, (reserve_a, reserve_b));
			Self::deposit_event(Event::AddLiquidity(pool_id, asset_a, asset_b, amount_a, amount_b));

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::add_liquidity())]
		#[transactional]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			pool_id: PoolId<T>,
			amount_a: BalanceOf<T>,
			amount_b: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::test_pool_ownership(&who, &pool_id)?;

			ensure!(
				!amount_a.is_zero() || !amount_b.is_zero(),
				Error::<T>::CannotRemoveZeroLiquidity
			);

			let (mut reserve_a, mut reserve_b) = Self::pool_balances(&pool_id);
			let (asset_a, asset_b) = Self::pool_assets(&pool_id);

			let pool_data = Self::pool_data(&pool_id);

			ensure!(!Self::is_sale_running(&pool_data),
				Error::<T>::SaleNotEnded);

			if !amount_a.is_zero() {
				reserve_a = reserve_a
				.checked_sub(&amount_a)
				.ok_or(Error::<T>::LiquidityUnderflow)?;
			}

			if !amount_b.is_zero() {
				reserve_b = reserve_b
				.checked_sub(&amount_b)
				.ok_or(Error::<T>::LiquidityUnderflow)?;
			}

			T::MultiCurrency::transfer(asset_a, &pool_id, &who, amount_a)?;
			T::MultiCurrency::transfer(asset_b, &pool_id, &who, amount_b)?;

			<PoolBalances<T>>::insert(&pool_id, (reserve_a, reserve_b));
			Self::deposit_event(Event::RemoveLiquidity(pool_id, asset_a, asset_b, amount_a, amount_b));

			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::destroy_pool())]
		#[transactional]
		pub fn destroy_pool(
			origin: OriginFor<T>,
			pool_id: PoolId<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			Self::test_pool_ownership(&who, &pool_id)?;

			let pool_data = Self::pool_data(&pool_id);
			let now = <frame_system::Pallet<T>>::block_number();
			ensure!(pool_data.start.is_zero() || pool_data.end < now, Error::<T>::SaleNotEnded);

			let (amount_a, amount_b) = Self::pool_balances(&pool_id);
			let (asset_a, asset_b) = Self::pool_assets(&pool_id);

			T::MultiCurrency::transfer(asset_a, &pool_id, &who, amount_a)?;
			T::MultiCurrency::transfer(asset_b, &pool_id, &who, amount_b)?;

			let deposit = Self::pool_deposit(&pool_id);
			T::MultiCurrency::unreserve(T::NativeAssetId::get(), &who, deposit);

			<PoolOwner<T>>::remove(&pool_id);
			<PoolDeposit<T>>::remove(&pool_id);
			<PoolAssets<T>>::remove(&pool_id);
			<PoolData<T>>::remove(&pool_id);
			<PoolBalances<T>>::remove(&pool_id);

			Self::deposit_event(Event::PoolDestroyed(pool_id, asset_a, asset_b, amount_a, amount_b));

			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn validate_pool_data(pool_data: &Pool<T::BlockNumber>) -> DispatchResult {
		let now = <frame_system::Pallet<T>>::block_number();
		ensure!(
			pool_data.start == Zero::zero() || now <= pool_data.start,
			Error::<T>::BlockNumberInvalid
		);
		ensure!(
			pool_data.end == Zero::zero() || pool_data.start < pool_data.end,
			Error::<T>::BlockNumberInvalid
		);
		ensure!(
			pool_data.initial_weights.0 <= 1_000_000 && pool_data.initial_weights.1 <= 1_000_000,
			Error::<T>::MaxWeightExceeded
		);
		ensure!(
			pool_data.final_weights.0 <= 1_000_000 && pool_data.final_weights.1 <= 1_000_000,
			Error::<T>::MaxWeightExceeded
		);

		Ok(())
	}

	fn test_pool_ownership(who: &T::AccountId, pool_id: &PoolId<T>) -> DispatchResult {
		ensure!(<PoolOwner<T>>::contains_key(&pool_id), Error::<T>::TokenPoolNotFound);

		let pool_owner = Self::pool_owner(&pool_id);
		ensure!(who == &pool_owner, Error::<T>::NotOwner);

		Ok(())
	}

	fn is_sale_running(pool_data: &Pool<T::BlockNumber>) -> bool {
		let now = <frame_system::Pallet<T>>::block_number();
		pool_data.start <= now && now <= pool_data.end
	}

	fn exists(assets: AssetPair) -> bool {
		let pair_account = T::AssetPairPoolId::from_assets(assets.asset_in, assets.asset_out);
		<PoolAssets<T>>::contains_key(&pair_account)
	}

	fn get_pair_id(assets: AssetPair) -> PoolId<T> {
		T::AssetPairPoolId::from_assets(assets.asset_in, assets.asset_out)
	}

	fn get_pool_assets(pool_id: &T::AccountId) -> Option<Vec<AssetId>> {
		match <PoolAssets<T>>::contains_key(pool_id) {
			true => {
				let assets = Self::pool_assets(pool_id);
				Some(vec![assets.0, assets.1])
			}
			false => None,
		}
	}
}

pub trait AssetPairPoolIdFor<AssetId: Sized, PoolId: Sized> {
	fn from_assets(asset_a: AssetId, asset_b: AssetId) -> PoolId;
}

pub struct AssetPairPoolId<T: Config>(PhantomData<T>);

impl<T: Config> AssetPairPoolIdFor<AssetId, PoolId<T>> for AssetPairPoolId<T>
where
	PoolId<T>: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	fn from_assets(asset_a: AssetId, asset_b: AssetId) -> PoolId<T> {
		let mut buf = Vec::new();
		buf.extend_from_slice(b"lbp");
		if asset_a < asset_b {
			buf.extend_from_slice(&asset_a.to_le_bytes());
			buf.extend_from_slice(&asset_b.to_le_bytes());
		} else {
			buf.extend_from_slice(&asset_b.to_le_bytes());
			buf.extend_from_slice(&asset_a.to_le_bytes());
		}
		PoolId::<T>::unchecked_from(T::Hashing::hash(&buf[..]))
	}
}
