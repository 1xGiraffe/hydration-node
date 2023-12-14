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

//! Autogenerated weights for `pallet_dca`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-12-12, STEPS: `10`, REPEAT: `30`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bench-bot`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// target/release/hydradx
// benchmark
// pallet
// --chain=dev
// --steps=10
// --repeat=30
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=.maintain/pallet-weight-template-no-back.hbs
// --pallet=pallet-dca
// --output=weights-1.1.0/dca.rs
// --extrinsic=*

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

use pallet_dca::weights::WeightInfo;

/// Weights for pallet_dca using the hydraDX node and recommended hardware.
pub struct HydraWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for HydraWeight<T> {
	/// Storage: `DCA::ScheduleIdsPerBlock` (r:12 w:2)
	/// Proof: `DCA::ScheduleIdsPerBlock` (`max_values`: None, `max_size`: Some(101), added: 2576, mode: `MaxEncodedLen`)
	/// Storage: `DCA::Schedules` (r:1 w:0)
	/// Proof: `DCA::Schedules` (`max_values`: None, `max_size`: Some(191), added: 2666, mode: `MaxEncodedLen`)
	/// Storage: `DCA::RemainingAmounts` (r:1 w:1)
	/// Proof: `DCA::RemainingAmounts` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Reserves` (r:1 w:1)
	/// Proof: `Balances::Reserves` (`max_values`: None, `max_size`: Some(1249), added: 3724, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `DCA::RetriesOnError` (r:0 w:1)
	/// Proof: `DCA::RetriesOnError` (`max_values`: None, `max_size`: Some(21), added: 2496, mode: `MaxEncodedLen`)
	fn on_initialize_with_buy_trade() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `38787`
		//  Estimated: `31902`
		// Minimum execution time: 234_825_000 picoseconds.
		Weight::from_parts(240_565_000, 31902)
			.saturating_add(T::DbWeight::get().reads(17_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: `DCA::ScheduleIdsPerBlock` (r:12 w:2)
	/// Proof: `DCA::ScheduleIdsPerBlock` (`max_values`: None, `max_size`: Some(101), added: 2576, mode: `MaxEncodedLen`)
	/// Storage: `DCA::Schedules` (r:1 w:0)
	/// Proof: `DCA::Schedules` (`max_values`: None, `max_size`: Some(191), added: 2666, mode: `MaxEncodedLen`)
	/// Storage: `DCA::RemainingAmounts` (r:1 w:1)
	/// Proof: `DCA::RemainingAmounts` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Reserves` (r:1 w:1)
	/// Proof: `Balances::Reserves` (`max_values`: None, `max_size`: Some(1249), added: 3724, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:2 w:2)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `DCA::RetriesOnError` (r:0 w:1)
	/// Proof: `DCA::RetriesOnError` (`max_values`: None, `max_size`: Some(21), added: 2496, mode: `MaxEncodedLen`)
	fn on_initialize_with_sell_trade() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `38787`
		//  Estimated: `31902`
		// Minimum execution time: 236_989_000 picoseconds.
		Weight::from_parts(240_871_000, 31902)
			.saturating_add(T::DbWeight::get().reads(17_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: `DCA::ScheduleIdsPerBlock` (r:1 w:0)
	/// Proof: `DCA::ScheduleIdsPerBlock` (`max_values`: None, `max_size`: Some(101), added: 2576, mode: `MaxEncodedLen`)
	fn on_initialize_with_empty_block() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1071`
		//  Estimated: `3566`
		// Minimum execution time: 18_241_000 picoseconds.
		Weight::from_parts(18_519_000, 3566).saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `DCA::ScheduleIdSequencer` (r:1 w:1)
	/// Proof: `DCA::ScheduleIdSequencer` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Reserves` (r:1 w:1)
	/// Proof: `Balances::Reserves` (`max_values`: None, `max_size`: Some(1249), added: 3724, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `DCA::ScheduleIdsPerBlock` (r:11 w:1)
	/// Proof: `DCA::ScheduleIdsPerBlock` (`max_values`: None, `max_size`: Some(101), added: 2576, mode: `MaxEncodedLen`)
	/// Storage: `DCA::RetriesOnError` (r:0 w:1)
	/// Proof: `DCA::RetriesOnError` (`max_values`: None, `max_size`: Some(21), added: 2496, mode: `MaxEncodedLen`)
	/// Storage: `DCA::Schedules` (r:0 w:1)
	/// Proof: `DCA::Schedules` (`max_values`: None, `max_size`: Some(191), added: 2666, mode: `MaxEncodedLen`)
	/// Storage: `DCA::ScheduleOwnership` (r:0 w:1)
	/// Proof: `DCA::ScheduleOwnership` (`max_values`: None, `max_size`: Some(60), added: 2535, mode: `MaxEncodedLen`)
	/// Storage: `DCA::RemainingAmounts` (r:0 w:1)
	/// Proof: `DCA::RemainingAmounts` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn schedule() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `3290`
		//  Estimated: `29326`
		// Minimum execution time: 140_204_000 picoseconds.
		Weight::from_parts(142_118_000, 29326)
			.saturating_add(T::DbWeight::get().reads(14_u64))
			.saturating_add(T::DbWeight::get().writes(8_u64))
	}
	/// Storage: `DCA::Schedules` (r:1 w:1)
	/// Proof: `DCA::Schedules` (`max_values`: None, `max_size`: Some(191), added: 2666, mode: `MaxEncodedLen`)
	/// Storage: `DCA::RemainingAmounts` (r:1 w:1)
	/// Proof: `DCA::RemainingAmounts` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	/// Storage: `Balances::Reserves` (r:1 w:1)
	/// Proof: `Balances::Reserves` (`max_values`: None, `max_size`: Some(1249), added: 3724, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:1)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `DCA::ScheduleIdsPerBlock` (r:1 w:1)
	/// Proof: `DCA::ScheduleIdsPerBlock` (`max_values`: None, `max_size`: Some(101), added: 2576, mode: `MaxEncodedLen`)
	/// Storage: `DCA::RetriesOnError` (r:0 w:1)
	/// Proof: `DCA::RetriesOnError` (`max_values`: None, `max_size`: Some(21), added: 2496, mode: `MaxEncodedLen`)
	/// Storage: `DCA::ScheduleOwnership` (r:0 w:1)
	/// Proof: `DCA::ScheduleOwnership` (`max_values`: None, `max_size`: Some(60), added: 2535, mode: `MaxEncodedLen`)
	fn terminate() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2451`
		//  Estimated: `4714`
		// Minimum execution time: 81_882_000 picoseconds.
		Weight::from_parts(82_424_000, 4714)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
}
