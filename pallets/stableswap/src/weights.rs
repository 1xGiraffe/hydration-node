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

//! Autogenerated weights for pallet_stableswap
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-07-17, STEPS: 5, REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// target/release/hydradx
// benchmark
// pallet
// --pallet=pallet-stableswap
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --chain=dev
// --extrinsic=*
// --steps=5
// --repeat=20
// --output
// stableswap.rs
// --template
// .maintain/pallet-weight-template-no-back.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_stableswap.
pub trait WeightInfo {
	fn create_pool() -> Weight;
	fn add_liquidity() -> Weight;
	fn remove_liquidity_one_asset() -> Weight;
	fn sell() -> Weight;
	fn buy() -> Weight;
	fn set_asset_tradable_state() -> Weight;
	fn update_pool_fee() -> Weight;
	fn update_amplification() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn create_pool() -> Weight {
		Weight::from_ref_time(52_934_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(7 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	fn add_liquidity() -> Weight {
		Weight::from_ref_time(676_183_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(27 as u64))
			.saturating_add(RocksDbWeight::get().writes(13 as u64))
	}
	fn remove_liquidity_one_asset() -> Weight {
		Weight::from_ref_time(402_817_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(15 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}
	fn sell() -> Weight {
		Weight::from_ref_time(357_640_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(15 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}
	fn buy() -> Weight {
		Weight::from_ref_time(343_079_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(16 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	fn set_asset_tradable_state() -> Weight {
		Weight::from_ref_time(24_426_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	fn update_pool_fee() -> Weight {
		Weight::from_ref_time(0)
	}
	fn update_amplification() -> Weight {
		Weight::from_ref_time(24_353_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_pool() -> Weight {
		Weight::from_ref_time(52_934_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(7 as u64))
			.saturating_add(RocksDbWeight::get().writes(2 as u64))
	}
	fn add_liquidity() -> Weight {
		Weight::from_ref_time(676_183_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(27 as u64))
			.saturating_add(RocksDbWeight::get().writes(13 as u64))
	}
	fn remove_liquidity_one_asset() -> Weight {
		Weight::from_ref_time(402_817_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(15 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}
	fn sell() -> Weight {
		Weight::from_ref_time(357_640_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(15 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}
	fn buy() -> Weight {
		Weight::from_ref_time(343_079_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(16 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	fn set_asset_tradable_state() -> Weight {
		Weight::from_ref_time(24_426_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	fn update_pool_fee() -> Weight {
		Weight::from_ref_time(0)
	}
	fn update_amplification() -> Weight {
		Weight::from_ref_time(24_353_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(1 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
}
