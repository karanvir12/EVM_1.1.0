// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

// Implemented by autogenerated benchmarking code.
pub trait WeightInfo {
	fn set_config_with_u32() -> Weight;
	fn set_config_with_weight() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	// Storage: XcmpQueue QueueConfig (r:1 w:1)
	fn set_config_with_u32() -> Weight {
		Weight::from_parts(2_717_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	// Storage: XcmpQueue QueueConfig (r:1 w:1)
	fn set_config_with_weight() -> Weight {
		Weight::from_parts(2_717_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

impl WeightInfo for () {
	// Storage: XcmpQueue QueueConfig (r:1 w:1)
	fn set_config_with_u32() -> Weight {
		Weight::from_parts(2_717_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	// Storage: XcmpQueue QueueConfig (r:1 w:1)
	fn set_config_with_weight() -> Weight {
		Weight::from_parts(2_717_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
