// This file is part of hydradx-adapters.

// Copyright (C) 2022  Intergalactic, Limited (GIB).
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

use codec::FullCodec;
use frame_support::{
	sp_runtime::{
		traits::{AtLeast32BitUnsigned, Convert, Get, MaybeSerializeDeserialize, Saturating, Zero},
		ArithmeticError, DispatchError, DispatchResult, FixedPointNumber, FixedPointOperand, FixedU128,
		SaturatedConversion,
	},
	traits::{Contains, OriginTrait},
	weights::{Weight, WeightToFee},
};
use hydra_dx_math::{
	ema::EmaPrice,
	omnipool::types::BalanceUpdate,
	support::rational::{round_to_rational, Rounding},
};
use hydradx_traits::{
	liquidity_mining::PriceAdjustment, AggregatedOracle, AggregatedPriceOracle, NativePriceOracle,
	OnLiquidityChangedHandler, OnTradeHandler, OraclePeriod, PriceOracle,
};
use orml_xcm_support::{OnDepositFail, UnknownAsset as UnknownAssetT};
use pallet_circuit_breaker::WeightInfo;
use pallet_ema_oracle::{OnActivityHandler, OracleError, Price};
use pallet_omnipool::traits::{AssetInfo, ExternalPriceProvider, OmnipoolHooks};
use pallet_transaction_multi_payment::DepositFee;
use polkadot_xcm::latest::prelude::*;
use primitive_types::U128;
use primitives::{constants::chain::OMNIPOOL_SOURCE, AccountId, AssetId, Balance, BlockNumber, CollectionId};
use sp_std::{collections::btree_map::BTreeMap, fmt::Debug, marker::PhantomData};
use warehouse_liquidity_mining::GlobalFarmData;
use xcm_builder::TakeRevenue;
use xcm_executor::{
	traits::{Convert as MoreConvert, MatchesFungible, TransactAsset, WeightTrader},
	Assets,
};

pub mod inspect;

#[cfg(test)]
mod tests;

/// Weight trader that accepts multiple assets as weight fee payment.
///
/// It uses `WeightToFee` in combination with a `NativePriceOracle` to set the right price for weight.
/// Keeps track of the assets used to pay for weight and can refund them one by one (interface only
/// allows returning one asset per refund). Will pass any remaining assets on `Drop` to
/// `TakeRevenue`.
pub struct MultiCurrencyTrader<
	AssetId,
	Balance: FixedPointOperand + TryInto<u128>,
	Price: FixedPointNumber,
	ConvertWeightToFee: WeightToFee<Balance = Balance>,
	AcceptedCurrencyPrices: NativePriceOracle<AssetId, Price>,
	ConvertCurrency: Convert<MultiAsset, Option<AssetId>>,
	Revenue: TakeRevenue,
> {
	weight: Weight,
	paid_assets: BTreeMap<(MultiLocation, Price), u128>,
	_phantom: PhantomData<(
		AssetId,
		Balance,
		Price,
		ConvertWeightToFee,
		AcceptedCurrencyPrices,
		ConvertCurrency,
		Revenue,
	)>,
}

impl<
		AssetId,
		Balance: FixedPointOperand + TryInto<u128>,
		Price: FixedPointNumber,
		ConvertWeightToFee: WeightToFee<Balance = Balance>,
		AcceptedCurrencyPrices: NativePriceOracle<AssetId, Price>,
		ConvertCurrency: Convert<MultiAsset, Option<AssetId>>,
		Revenue: TakeRevenue,
	> MultiCurrencyTrader<AssetId, Balance, Price, ConvertWeightToFee, AcceptedCurrencyPrices, ConvertCurrency, Revenue>
{
	/// Get the asset id of the first asset in `payment` and try to determine its price via the
	/// price oracle.
	fn get_asset_and_price(&mut self, payment: &Assets) -> Option<(MultiLocation, Price)> {
		if let Some(asset) = payment.fungible_assets_iter().next() {
			ConvertCurrency::convert(asset.clone())
				.and_then(|currency| AcceptedCurrencyPrices::price(currency))
				.and_then(|price| match asset.id {
					Concrete(location) => Some((location, price)),
					_ => None,
				})
		} else {
			None
		}
	}
}

impl<
		AssetId,
		Balance: FixedPointOperand + TryInto<u128>,
		Price: FixedPointNumber,
		ConvertWeightToFee: WeightToFee<Balance = Balance>,
		AcceptedCurrencyPrices: NativePriceOracle<AssetId, Price>,
		ConvertCurrency: Convert<MultiAsset, Option<AssetId>>,
		Revenue: TakeRevenue,
	> WeightTrader
	for MultiCurrencyTrader<AssetId, Balance, Price, ConvertWeightToFee, AcceptedCurrencyPrices, ConvertCurrency, Revenue>
{
	fn new() -> Self {
		Self {
			weight: Default::default(),
			paid_assets: Default::default(),
			_phantom: PhantomData,
		}
	}

	/// Will try to buy weight with the first asset in `payment`.
	///
	/// This is a reasonable strategy as the `BuyExecution` XCM instruction only passes one asset
	/// per buy.
	/// The fee is determined by `ConvertWeightToFee` in combination with the price determined by
	/// `AcceptedCurrencyPrices`.
	fn buy_weight(&mut self, weight: Weight, payment: Assets) -> Result<Assets, XcmError> {
		log::trace!(
			target: "xcm::weight", "MultiCurrencyTrader::buy_weight weight: {:?}, payment: {:?}",
			weight, payment
		);
		let (asset_loc, price) = self.get_asset_and_price(&payment).ok_or(XcmError::AssetNotFound)?;
		let fee = ConvertWeightToFee::weight_to_fee(&weight);
		let converted_fee = price.checked_mul_int(fee).ok_or(XcmError::Overflow)?;
		let amount: u128 = converted_fee.try_into().map_err(|_| XcmError::Overflow)?;
		let required = (Concrete(asset_loc), amount).into();
		let unused = payment.checked_sub(required).map_err(|_| XcmError::TooExpensive)?;
		self.weight = self.weight.saturating_add(weight);
		let key = (asset_loc, price);
		match self.paid_assets.get_mut(&key) {
			Some(v) => v.saturating_accrue(amount),
			None => {
				self.paid_assets.insert(key, amount);
			}
		}
		Ok(unused)
	}

	/// Will refund up to `weight` from the first asset tracked by the trader.
	fn refund_weight(&mut self, weight: Weight) -> Option<MultiAsset> {
		log::trace!(
			target: "xcm::weight", "MultiCurrencyTrader::refund_weight weight: {:?}, paid_assets: {:?}",
			weight, self.paid_assets
		);
		let weight = weight.min(self.weight);
		self.weight -= weight; // Will not underflow because of `min()` above.
		let fee = ConvertWeightToFee::weight_to_fee(&weight);
		if let Some(((asset_loc, price), amount)) = self.paid_assets.iter_mut().next() {
			let converted_fee: u128 = price.saturating_mul_int(fee).saturated_into();
			let refund = converted_fee.min(*amount);
			*amount -= refund; // Will not underflow because of `min()` above.

			let refund_asset = *asset_loc;
			if amount.is_zero() {
				let key = (*asset_loc, *price);
				self.paid_assets.remove(&key);
			}
			Some((Concrete(refund_asset), refund).into())
		} else {
			None
		}
	}
}

/// We implement `Drop` so that when the weight trader is dropped at the end of XCM execution, the
/// generated revenue is stored on-chain. This is configurable via the `Revenue` generic.
impl<
		AssetId,
		Balance: FixedPointOperand + TryInto<u128>,
		Price: FixedPointNumber,
		ConvertWeightToFee: WeightToFee<Balance = Balance>,
		AcceptedCurrencyPrices: NativePriceOracle<AssetId, Price>,
		ConvertCurrency: Convert<MultiAsset, Option<AssetId>>,
		Revenue: TakeRevenue,
	> Drop
	for MultiCurrencyTrader<AssetId, Balance, Price, ConvertWeightToFee, AcceptedCurrencyPrices, ConvertCurrency, Revenue>
{
	fn drop(&mut self) {
		for ((asset_loc, _), amount) in self.paid_assets.iter() {
			Revenue::take_revenue((*asset_loc, *amount).into());
		}
	}
}

/// Implements `TakeRevenue` by sending the assets to the fee receiver, using an implementor of
/// `DepositFee`.
///
/// Note: Only supports concrete fungible assets.
pub struct ToFeeReceiver<AccountId, AssetId, Balance, Price, C, D, F>(
	PhantomData<(AccountId, AssetId, Balance, Price, C, D, F)>,
);
impl<
		AccountId,
		AssetId,
		Balance: AtLeast32BitUnsigned,
		Price,
		C: Convert<MultiLocation, Option<AssetId>>,
		D: DepositFee<AccountId, AssetId, Balance>,
		F: Get<AccountId>,
	> TakeRevenue for ToFeeReceiver<AccountId, AssetId, Balance, Price, C, D, F>
{
	fn take_revenue(asset: MultiAsset) {
		match asset {
			MultiAsset {
				id: Concrete(loc),
				fun: Fungibility::Fungible(amount),
			} => {
				C::convert(loc).and_then(|id| {
					let receiver = F::get();
					D::deposit_fee(&receiver, id, amount.saturated_into::<Balance>())
						.map_err(|e| log::trace!(target: "xcm::take_revenue", "Could not deposit fee: {:?}", e))
						.ok()
				});
			}
			_ => {
				debug_assert!(false, "Can only accept concrete fungible tokens as revenue.");
				log::trace!(target: "xcm::take_revenue", "Can only accept concrete fungible tokens as revenue.");
			}
		}
	}
}

/// Passes on trade and liquidity data from the omnipool to the oracle.
pub struct OmnipoolHookAdapter<Origin, Lrna, Runtime>(PhantomData<(Origin, Lrna, Runtime)>);

impl<Origin, Lrna, Runtime> OmnipoolHooks<Origin, AccountId, AssetId, Balance>
	for OmnipoolHookAdapter<Origin, Lrna, Runtime>
where
	Lrna: Get<AssetId>,
	Runtime: pallet_ema_oracle::Config
		+ pallet_circuit_breaker::Config
		+ frame_system::Config<RuntimeOrigin = Origin>
		+ pallet_staking::Config,
	<Runtime as frame_system::Config>::AccountId: From<AccountId>,
	<Runtime as pallet_staking::Config>::AssetId: From<AssetId>,
{
	type Error = DispatchError;

	fn on_liquidity_changed(origin: Origin, asset: AssetInfo<AssetId, Balance>) -> Result<Weight, Self::Error> {
		OnActivityHandler::<Runtime>::on_liquidity_changed(
			OMNIPOOL_SOURCE,
			asset.asset_id,
			Lrna::get(),
			*asset.delta_changes.delta_reserve,
			*asset.delta_changes.delta_hub_reserve,
			asset.after.reserve,
			asset.after.hub_reserve,
		)
		.map_err(|(_, e)| e)?;

		match asset.delta_changes.delta_reserve {
			BalanceUpdate::Increase(amount) => pallet_circuit_breaker::Pallet::<Runtime>::ensure_add_liquidity_limit(
				origin,
				asset.asset_id.into(),
				asset.before.reserve.into(),
				amount.into(),
			)?,
			BalanceUpdate::Decrease(amount) => {
				pallet_circuit_breaker::Pallet::<Runtime>::ensure_remove_liquidity_limit(
					origin,
					asset.asset_id.into(),
					asset.before.reserve.into(),
					amount.into(),
				)?
			}
		};

		Ok(Self::on_liquidity_changed_weight())
	}

	fn on_trade(
		_origin: Origin,
		asset_in: AssetInfo<AssetId, Balance>,
		asset_out: AssetInfo<AssetId, Balance>,
	) -> Result<Weight, Self::Error> {
		OnActivityHandler::<Runtime>::on_trade(
			OMNIPOOL_SOURCE,
			asset_in.asset_id,
			Lrna::get(),
			*asset_in.delta_changes.delta_reserve,
			*asset_in.delta_changes.delta_hub_reserve,
			asset_in.after.reserve,
			asset_in.after.hub_reserve,
		)
		.map_err(|(_, e)| e)?;

		OnActivityHandler::<Runtime>::on_trade(
			OMNIPOOL_SOURCE,
			Lrna::get(),
			asset_out.asset_id,
			*asset_out.delta_changes.delta_hub_reserve,
			*asset_out.delta_changes.delta_reserve,
			asset_out.after.hub_reserve,
			asset_out.after.reserve,
		)
		.map_err(|(_, e)| e)?;

		let amount_in = *asset_in.delta_changes.delta_reserve;
		let amount_out = *asset_out.delta_changes.delta_reserve;

		pallet_circuit_breaker::Pallet::<Runtime>::ensure_pool_state_change_limit(
			asset_in.asset_id.into(),
			asset_in.before.reserve.into(),
			amount_in.into(),
			asset_out.asset_id.into(),
			asset_out.before.reserve.into(),
			amount_out.into(),
		)?;

		Ok(Self::on_trade_weight())
	}

	fn on_hub_asset_trade(_origin: Origin, asset: AssetInfo<AssetId, Balance>) -> Result<Weight, Self::Error> {
		OnActivityHandler::<Runtime>::on_trade(
			OMNIPOOL_SOURCE,
			Lrna::get(),
			asset.asset_id,
			*asset.delta_changes.delta_hub_reserve,
			*asset.delta_changes.delta_reserve,
			asset.after.hub_reserve,
			asset.after.reserve,
		)
		.map_err(|(_, e)| e)?;

		let amount_out = *asset.delta_changes.delta_reserve;

		pallet_circuit_breaker::Pallet::<Runtime>::ensure_pool_state_change_limit(
			Lrna::get().into(),
			Balance::zero().into(),
			Balance::zero().into(),
			asset.asset_id.into(),
			asset.before.reserve.into(),
			amount_out.into(),
		)?;

		Ok(Self::on_trade_weight())
	}

	fn on_liquidity_changed_weight() -> Weight {
		let w1 = OnActivityHandler::<Runtime>::on_liquidity_changed_weight();
		let w2 = <Runtime as pallet_circuit_breaker::Config>::WeightInfo::ensure_add_liquidity_limit()
			.max(<Runtime as pallet_circuit_breaker::Config>::WeightInfo::ensure_remove_liquidity_limit());
		let w3 = <Runtime as pallet_circuit_breaker::Config>::WeightInfo::on_finalize_single_liquidity_limit_entry();
		w1.saturating_add(w2).saturating_add(w3)
	}

	fn on_trade_weight() -> Weight {
		let w1 = OnActivityHandler::<Runtime>::on_trade_weight().saturating_mul(2);
		let w2 = <Runtime as pallet_circuit_breaker::Config>::WeightInfo::ensure_pool_state_change_limit();
		let w3 = <Runtime as pallet_circuit_breaker::Config>::WeightInfo::on_finalize_single_trade_limit_entry();
		w1.saturating_add(w2).saturating_add(w3)
	}

	fn on_trade_fee(fee_account: AccountId, asset: AssetId, amount: Balance) -> Result<Balance, Self::Error> {
		pallet_staking::Pallet::<Runtime>::process_trade_fee(fee_account.into(), asset.into(), amount)
	}
}

/// Passes ema oracle price to the omnipool.
pub struct EmaOraclePriceAdapter<Period, Runtime>(PhantomData<(Period, Runtime)>);

impl<Period, Runtime> ExternalPriceProvider<AssetId, Price> for EmaOraclePriceAdapter<Period, Runtime>
where
	Period: Get<OraclePeriod>,
	Runtime: pallet_ema_oracle::Config + pallet_omnipool::Config,
{
	type Error = DispatchError;

	fn get_price(asset_a: AssetId, asset_b: AssetId) -> Result<Price, Self::Error> {
		let (price, _) =
			pallet_ema_oracle::Pallet::<Runtime>::get_price(asset_a, asset_b, Period::get(), OMNIPOOL_SOURCE)
				.map_err(|_| pallet_omnipool::Error::<Runtime>::PriceDifferenceTooHigh)?;
		Ok(price)
	}

	fn get_price_weight() -> Weight {
		pallet_ema_oracle::Pallet::<Runtime>::get_price_weight()
	}
}

pub struct OraclePriceProviderAdapterForOmnipool<AssetId, AggregatedPriceGetter, Lrna>(
	PhantomData<(AssetId, AggregatedPriceGetter, Lrna)>,
);

impl<AssetId, AggregatedPriceGetter, Lrna> PriceOracle<AssetId>
	for OraclePriceProviderAdapterForOmnipool<AssetId, AggregatedPriceGetter, Lrna>
where
	u32: From<AssetId>,
	AggregatedPriceGetter: AggregatedPriceOracle<AssetId, BlockNumber, EmaPrice, Error = OracleError>,
	Lrna: Get<AssetId>,
{
	type Price = EmaPrice;

	fn price(asset_a: AssetId, asset_b: AssetId, period: OraclePeriod) -> Option<EmaPrice> {
		let price_asset_a_lrna = AggregatedPriceGetter::get_price(asset_a, Lrna::get(), period, OMNIPOOL_SOURCE);

		let price_asset_a_lrna = match price_asset_a_lrna {
			Ok(price) => price.0,
			Err(OracleError::SameAsset) => EmaPrice::from(1),
			Err(_) => return None,
		};

		let price_lrna_asset_b = AggregatedPriceGetter::get_price(Lrna::get(), asset_b, period, OMNIPOOL_SOURCE);

		let price_lrna_asset_b = match price_lrna_asset_b {
			Ok(price) => price.0,
			Err(OracleError::SameAsset) => EmaPrice::from(1),
			Err(_) => return None,
		};

		let nominator = U128::full_mul(price_asset_a_lrna.n.into(), price_lrna_asset_b.n.into());
		let denominator = U128::full_mul(price_asset_a_lrna.d.into(), price_lrna_asset_b.d.into());

		let rational_as_u128 = round_to_rational((nominator, denominator), Rounding::Nearest);
		let price_in_ema_price = EmaPrice::new(rational_as_u128.0, rational_as_u128.1);

		Some(price_in_ema_price)
	}
}

pub struct PriceAdjustmentAdapter<Runtime, LMInstance>(PhantomData<(Runtime, LMInstance)>);

impl<Runtime, LMInstance> PriceAdjustment<GlobalFarmData<Runtime, LMInstance>>
	for PriceAdjustmentAdapter<Runtime, LMInstance>
where
	Runtime: warehouse_liquidity_mining::Config<LMInstance>
		+ pallet_ema_oracle::Config
		+ pallet_omnipool_liquidity_mining::Config,
{
	type Error = DispatchError;
	type PriceAdjustment = FixedU128;

	fn get(global_farm: &GlobalFarmData<Runtime, LMInstance>) -> Result<Self::PriceAdjustment, Self::Error> {
		let (price, _) = pallet_ema_oracle::Pallet::<Runtime>::get_price(
			global_farm.reward_currency.into(),
			global_farm.incentivized_asset.into(), //LRNA
			OraclePeriod::TenMinutes,
			OMNIPOOL_SOURCE,
		)
		.map_err(|_| pallet_omnipool_liquidity_mining::Error::<Runtime>::PriceAdjustmentNotAvailable)?;

		FixedU128::checked_from_rational(price.n, price.d).ok_or_else(|| ArithmeticError::Overflow.into())
	}
}

/// Asset transaction errors.
enum Error {
	/// Failed to match fungible.
	FailedToMatchFungible,
	/// `MultiLocation` to `AccountId` Conversion failed.
	AccountIdConversionFailed,
	/// `CurrencyId` conversion failed.
	CurrencyIdConversionFailed,
}

impl From<Error> for XcmError {
	fn from(e: Error) -> Self {
		match e {
			Error::FailedToMatchFungible => XcmError::FailedToTransactAsset("FailedToMatchFungible"),
			Error::AccountIdConversionFailed => XcmError::FailedToTransactAsset("AccountIdConversionFailed"),
			Error::CurrencyIdConversionFailed => XcmError::FailedToTransactAsset("CurrencyIdConversionFailed"),
		}
	}
}

/// The `TransactAsset` implementation, to handle `MultiAsset` deposit/withdraw, but reroutes deposits and transfers
/// to unsupported accounts to an alternative.
///
/// Note that teleport related functions are unimplemented.
///
/// Methods of `DepositFailureHandler` would be called on multi-currency deposit
/// errors.
///
/// If the asset is known, deposit/withdraw will be handled by `MultiCurrency`,
/// else by `UnknownAsset` if unknown.
///
/// Taken and modified from `orml_xcm_support`.
/// https://github.com/open-web3-stack/open-runtime-module-library/blob/4ae0372e2c624e6acc98305564b9d395f70814c0/xcm-support/src/currency_adapter.rs#L96-L202
#[allow(clippy::type_complexity)]
pub struct ReroutingMultiCurrencyAdapter<
	MultiCurrency,
	UnknownAsset,
	Match,
	AccountId,
	AccountIdConvert,
	CurrencyId,
	CurrencyIdConvert,
	DepositFailureHandler,
	RerouteFilter,
	RerouteDestination,
>(
	PhantomData<(
		MultiCurrency,
		UnknownAsset,
		Match,
		AccountId,
		AccountIdConvert,
		CurrencyId,
		CurrencyIdConvert,
		DepositFailureHandler,
		RerouteFilter,
		RerouteDestination,
	)>,
);

impl<
		MultiCurrency: orml_traits::MultiCurrency<AccountId, CurrencyId = CurrencyId>,
		UnknownAsset: UnknownAssetT,
		Match: MatchesFungible<MultiCurrency::Balance>,
		AccountId: sp_std::fmt::Debug + Clone,
		AccountIdConvert: MoreConvert<MultiLocation, AccountId>,
		CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug,
		CurrencyIdConvert: Convert<MultiAsset, Option<CurrencyId>>,
		DepositFailureHandler: OnDepositFail<CurrencyId, AccountId, MultiCurrency::Balance>,
		RerouteFilter: Contains<(CurrencyId, AccountId)>,
		RerouteDestination: Get<AccountId>,
	> TransactAsset
	for ReroutingMultiCurrencyAdapter<
		MultiCurrency,
		UnknownAsset,
		Match,
		AccountId,
		AccountIdConvert,
		CurrencyId,
		CurrencyIdConvert,
		DepositFailureHandler,
		RerouteFilter,
		RerouteDestination,
	>
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation, _context: &XcmContext) -> Result<(), XcmError> {
		match (
			AccountIdConvert::convert_ref(location),
			CurrencyIdConvert::convert(asset.clone()),
			Match::matches_fungible(asset),
		) {
			// known asset
			(Ok(who), Some(currency_id), Some(amount)) => {
				if RerouteFilter::contains(&(currency_id, who.clone())) {
					MultiCurrency::deposit(currency_id, &RerouteDestination::get(), amount)
						.or_else(|err| DepositFailureHandler::on_deposit_currency_fail(err, currency_id, &who, amount))
				} else {
					MultiCurrency::deposit(currency_id, &who, amount)
						.or_else(|err| DepositFailureHandler::on_deposit_currency_fail(err, currency_id, &who, amount))
				}
			}
			// unknown asset
			_ => UnknownAsset::deposit(asset, location)
				.or_else(|err| DepositFailureHandler::on_deposit_unknown_asset_fail(err, asset, location)),
		}
	}

	fn withdraw_asset(
		asset: &MultiAsset,
		location: &MultiLocation,
		_maybe_context: Option<&XcmContext>,
	) -> Result<Assets, XcmError> {
		UnknownAsset::withdraw(asset, location).or_else(|_| {
			let who = AccountIdConvert::convert_ref(location)
				.map_err(|_| XcmError::from(Error::AccountIdConversionFailed))?;
			let currency_id = CurrencyIdConvert::convert(asset.clone())
				.ok_or_else(|| XcmError::from(Error::CurrencyIdConversionFailed))?;
			let amount: MultiCurrency::Balance = Match::matches_fungible(asset)
				.ok_or_else(|| XcmError::from(Error::FailedToMatchFungible))?
				.saturated_into();
			MultiCurrency::withdraw(currency_id, &who, amount).map_err(|e| XcmError::FailedToTransactAsset(e.into()))
		})?;

		Ok(asset.clone().into())
	}

	fn transfer_asset(
		asset: &MultiAsset,
		from: &MultiLocation,
		to: &MultiLocation,
		_context: &XcmContext,
	) -> Result<Assets, XcmError> {
		let from_account =
			AccountIdConvert::convert_ref(from).map_err(|_| XcmError::from(Error::AccountIdConversionFailed))?;
		let to_account =
			AccountIdConvert::convert_ref(to).map_err(|_| XcmError::from(Error::AccountIdConversionFailed))?;
		let currency_id = CurrencyIdConvert::convert(asset.clone())
			.ok_or_else(|| XcmError::from(Error::CurrencyIdConversionFailed))?;
		let to_account = if RerouteFilter::contains(&(currency_id, to_account.clone())) {
			RerouteDestination::get()
		} else {
			to_account
		};
		let amount: MultiCurrency::Balance = Match::matches_fungible(asset)
			.ok_or_else(|| XcmError::from(Error::FailedToMatchFungible))?
			.saturated_into();
		MultiCurrency::transfer(currency_id, &from_account, &to_account, amount)
			.map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

		Ok(asset.clone().into())
	}
}

// Dynamic fees volume adapter
pub struct OracleVolume(Balance, Balance);

impl pallet_dynamic_fees::traits::Volume<Balance> for OracleVolume {
	fn amount_in(&self) -> Balance {
		self.0
	}

	fn amount_out(&self) -> Balance {
		self.1
	}
}

pub struct OracleAssetVolumeProvider<Runtime, Lrna, Period>(PhantomData<(Runtime, Lrna, Period)>);

impl<Runtime, Lrna, Period> pallet_dynamic_fees::traits::VolumeProvider<AssetId, Balance>
	for OracleAssetVolumeProvider<Runtime, Lrna, Period>
where
	Runtime: pallet_ema_oracle::Config,
	Lrna: Get<AssetId>,
	Period: Get<OraclePeriod>,
{
	type Volume = OracleVolume;

	fn asset_volume(asset_id: AssetId) -> Option<Self::Volume> {
		let entry =
			pallet_ema_oracle::Pallet::<Runtime>::get_entry(asset_id, Lrna::get(), Period::get(), OMNIPOOL_SOURCE)
				.ok()?;
		Some(OracleVolume(entry.volume.a_in, entry.volume.a_out))
	}

	fn asset_liquidity(asset_id: AssetId) -> Option<Balance> {
		let entry =
			pallet_ema_oracle::Pallet::<Runtime>::get_entry(asset_id, Lrna::get(), Period::get(), OMNIPOOL_SOURCE)
				.ok()?;
		Some(entry.liquidity.a)
	}
}

pub struct VestingInfo<Runtime>(PhantomData<Runtime>);

impl<Runtime> pallet_staking::traits::VestingDetails<AccountId, Balance> for VestingInfo<Runtime>
where
	Runtime: pallet_balances::Config<Balance = Balance>,
	AccountId: codec::EncodeLike<<Runtime as frame_system::Config>::AccountId>,
{
	fn locked(who: AccountId) -> Balance {
		let lock_id = orml_vesting::VESTING_LOCK_ID;

		if let Some(p) = pallet_balances::Locks::<Runtime>::get(who.clone())
			.iter()
			.find(|x| x.id == lock_id)
		{
			return p.amount;
		}

		Zero::zero()
	}
}

pub struct FreezableNFT<Runtime, Origin>(PhantomData<(Runtime, Origin)>);

impl<Runtime, Origin: OriginTrait<AccountId = AccountId>> pallet_staking::traits::Freeze<AccountId, CollectionId>
	for FreezableNFT<Runtime, Origin>
where
	Runtime: frame_system::Config<RuntimeOrigin = Origin> + pallet_uniques::Config<CollectionId = CollectionId>,
{
	fn freeze_collection(owner: AccountId, collection: CollectionId) -> DispatchResult {
		pallet_uniques::Pallet::<Runtime>::freeze_collection(Runtime::RuntimeOrigin::signed(owner), collection)
	}
}
