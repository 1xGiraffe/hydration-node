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


//! Autogenerated weights for `pallet_evm_accounts`
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

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weights for `pallet_evm_accounts`.
pub struct WeightInfo<T>(PhantomData<T>);

/// Weights for `pallet_evm_accounts` using the HydraDX node and recommended hardware.
pub struct HydraWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_evm_accounts::WeightInfo for HydraWeight<T> {
	/// Storage: `EVMAccounts::AccountExtension` (r:1 w:1)
	/// Proof: `EVMAccounts::AccountExtension` (`max_values`: None, `max_size`: Some(48), added: 2523, mode: `MaxEncodedLen`)
	/// Storage: `System::Account` (r:1 w:0)
	/// Proof: `System::Account` (`max_values`: None, `max_size`: Some(128), added: 2603, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::NextAssetId` (r:1 w:0)
	/// Proof: `AssetRegistry::NextAssetId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::LocationAssets` (r:1 w:0)
	/// Proof: `AssetRegistry::LocationAssets` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	/// Storage: `Tokens::Accounts` (r:1 w:0)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::Assets` (r:1 w:0)
	/// Proof: `AssetRegistry::Assets` (`max_values`: None, `max_size`: Some(125), added: 2600, mode: `MaxEncodedLen`)
	fn bind_evm_address() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `545`
		//  Estimated: `4087`
		// Minimum execution time: 32_109_000 picoseconds.
		Weight::from_parts(32_705_000, 4087)
			.saturating_add(T::DbWeight::get().reads(6_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `EVMAccounts::ContractDeployer` (r:0 w:1)
	/// Proof: `EVMAccounts::ContractDeployer` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn add_contract_deployer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_574_000 picoseconds.
		Weight::from_parts(9_760_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `EVMAccounts::ContractDeployer` (r:0 w:1)
	/// Proof: `EVMAccounts::ContractDeployer` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn remove_contract_deployer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_344_000 picoseconds.
		Weight::from_parts(9_633_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `EVMAccounts::ContractDeployer` (r:0 w:1)
	/// Proof: `EVMAccounts::ContractDeployer` (`max_values`: None, `max_size`: Some(36), added: 2511, mode: `MaxEncodedLen`)
	fn renounce_contract_deployer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 9_421_000 picoseconds.
		Weight::from_parts(9_685_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}