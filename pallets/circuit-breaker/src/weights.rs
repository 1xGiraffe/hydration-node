// This file is part of HydraDX.

// Copyright (C) 2020-2023  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Autogenerated weights for `pallet_circuit_breaker`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-05-23, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bench-bot`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: `1024`

// Executed Command:
// ./target/release/hydradx
// benchmark
// pallet
// --wasm-execution=compiled
// --pallet
// *
// --extrinsic
// *
// --heap-pages
// 4096
// --steps
// 50
// --repeat
// 20
// --template=scripts/pallet-weight-template.hbs
// --output
// weights/

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_omnipool.
pub trait WeightInfo {
	fn on_finalize(m: u32, n: u32) -> Weight;
	fn on_finalize_single_liquidity_limit_entry() -> Weight;
	fn on_finalize_single_trade_limit_entry() -> Weight;
	fn on_finalize_empty() -> Weight;
	fn set_trade_volume_limit() -> Weight;
	fn set_add_liquidity_limit() -> Weight;
	fn set_remove_liquidity_limit() -> Weight;
	fn ensure_pool_state_change_limit() -> Weight;
	fn ensure_add_liquidity_limit() -> Weight;
	fn ensure_remove_liquidity_limit() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// The range of component `n` is `[0, 400]`.
	/// The range of component `m` is `[0, 400]`.
	fn on_finalize(n: u32, m: u32) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `64 + m * (113 ±0) + n * (56 ±0)`
		//  Estimated: `0`
		// Minimum execution time: 306_621_000 picoseconds.
		Weight::from_parts(308_251_000, 0)
			// Standard Error: 8_989
			.saturating_add(Weight::from_parts(270_702, 0).saturating_mul(n.into()))
			// Standard Error: 8_989
			.saturating_add(Weight::from_parts(1_049_170, 0).saturating_mul(m.into()))
	}
	fn on_finalize_single_liquidity_limit_entry() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `208`
		//  Estimated: `0`
		// Minimum execution time: 8_112_000 picoseconds.
		Weight::from_parts(8_381_000, 0)
	}
	fn on_finalize_single_trade_limit_entry() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `208`
		//  Estimated: `0`
		// Minimum execution time: 8_194_000 picoseconds.
		Weight::from_parts(8_399_000, 0)
	}
	fn on_finalize_empty() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `208`
		//  Estimated: `0`
		// Minimum execution time: 8_191_000 picoseconds.
		Weight::from_parts(8_373_000, 0)
	}
	/// Storage: `CircuitBreaker::TradeVolumeLimitPerAsset` (r:0 w:1)
	/// Proof: `CircuitBreaker::TradeVolumeLimitPerAsset` (`max_values`: None, `max_size`: Some(28), added: 2503, mode: `MaxEncodedLen`)
	fn set_trade_volume_limit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_818_000 picoseconds.
		Weight::from_parts(9_140_000, 0).saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `CircuitBreaker::LiquidityAddLimitPerAsset` (r:0 w:1)
	/// Proof: `CircuitBreaker::LiquidityAddLimitPerAsset` (`max_values`: None, `max_size`: Some(29), added: 2504, mode: `MaxEncodedLen`)
	fn set_add_liquidity_limit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_841_000 picoseconds.
		Weight::from_parts(9_081_000, 0).saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `CircuitBreaker::LiquidityRemoveLimitPerAsset` (r:0 w:1)
	/// Proof: `CircuitBreaker::LiquidityRemoveLimitPerAsset` (`max_values`: None, `max_size`: Some(29), added: 2504, mode: `MaxEncodedLen`)
	fn set_remove_liquidity_limit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_089_000 picoseconds.
		Weight::from_parts(9_231_000, 0).saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `CircuitBreaker::LiquidityAddLimitPerAsset` (r:1 w:0)
	/// Proof: `CircuitBreaker::LiquidityAddLimitPerAsset` (`max_values`: None, `max_size`: Some(29), added: 2504, mode: `MaxEncodedLen`)
	/// Storage: `CircuitBreaker::AllowedAddLiquidityAmountPerAsset` (r:1 w:1)
	/// Proof: `CircuitBreaker::AllowedAddLiquidityAmountPerAsset` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// Storage: `CircuitBreaker::LiquidityRemoveLimitPerAsset` (r:1 w:0)
	/// Proof: `CircuitBreaker::LiquidityRemoveLimitPerAsset` (`max_values`: None, `max_size`: Some(29), added: 2504, mode: `MaxEncodedLen`)
	/// Storage: `CircuitBreaker::AllowedRemoveLiquidityAmountPerAsset` (r:1 w:1)
	/// Proof: `CircuitBreaker::AllowedRemoveLiquidityAmountPerAsset` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	fn ensure_add_liquidity_limit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `262`
		//  Estimated: `3517`
		// Minimum execution time: 20_016_000 picoseconds.
		Weight::from_parts(20_252_000, 3517)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `CircuitBreaker::LiquidityAddLimitPerAsset` (r:1 w:0)
	/// Proof: `CircuitBreaker::LiquidityAddLimitPerAsset` (`max_values`: None, `max_size`: Some(29), added: 2504, mode: `MaxEncodedLen`)
	/// Storage: `CircuitBreaker::AllowedAddLiquidityAmountPerAsset` (r:1 w:1)
	/// Proof: `CircuitBreaker::AllowedAddLiquidityAmountPerAsset` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	/// Storage: `CircuitBreaker::LiquidityRemoveLimitPerAsset` (r:1 w:0)
	/// Proof: `CircuitBreaker::LiquidityRemoveLimitPerAsset` (`max_values`: None, `max_size`: Some(29), added: 2504, mode: `MaxEncodedLen`)
	/// Storage: `CircuitBreaker::AllowedRemoveLiquidityAmountPerAsset` (r:1 w:1)
	/// Proof: `CircuitBreaker::AllowedRemoveLiquidityAmountPerAsset` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
	fn ensure_remove_liquidity_limit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `208`
		//  Estimated: `3517`
		// Minimum execution time: 17_308_000 picoseconds.
		Weight::from_parts(17_647_000, 3517)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `CircuitBreaker::AllowedTradeVolumeLimitPerAsset` (r:2 w:2)
	/// Proof: `CircuitBreaker::AllowedTradeVolumeLimitPerAsset` (`max_values`: None, `max_size`: Some(68), added: 2543, mode: `MaxEncodedLen`)
	/// Storage: `CircuitBreaker::TradeVolumeLimitPerAsset` (r:2 w:0)
	/// Proof: `CircuitBreaker::TradeVolumeLimitPerAsset` (`max_values`: None, `max_size`: Some(28), added: 2503, mode: `MaxEncodedLen`)
	fn ensure_pool_state_change_limit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `208`
		//  Estimated: `6076`
		// Minimum execution time: 17_387_000 picoseconds.
		Weight::from_parts(17_667_000, 6076)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
}
