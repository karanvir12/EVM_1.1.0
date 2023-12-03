// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Polkadot-specific RPCs implementation.

#![warn(missing_docs)]

// pub use fc_storage::overrides_handle;
use jsonrpsee::RpcModule;
use polkadot_primitives::{ AccountId, Balance, Block, BlockNumber, Hash, Nonce };
use sc_client_api::{AuxStore, UsageProvider};
use sc_consensus_beefy::communication::notification::{ BeefyBestBlockStream, BeefyVersionedFinalityProofStream };
use std::sync::Arc;
use sc_consensus_grandpa::FinalityProofProvider;
pub use sc_rpc::{ DenyUnsafe, SubscriptionTaskExecutor };
//use sp_api::ProvideRuntimeApi;
use consensus_common::SelectChain;
use sc_network_sync::SyncingService;
use sc_transaction_pool_api::TransactionPool;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{ Error as BlockChainError, HeaderBackend, HeaderMetadata };
use sp_consensus_babe::BabeApi;
use sp_keystore::KeystorePtr;

use fc_rpc::EthTask;
use sc_client_api::{ backend::{ Backend, StorageProvider }, client::BlockchainEvents };
use std::collections::BTreeMap;
//=============================================
use fc_rpc_core::types::{ FeeHistoryCache, FeeHistoryCacheLimit, FilterPool };
use sc_network::NetworkService;
use sc_transaction_pool::{ ChainApi, Pool };
use sp_runtime::{ testing::H256, traits::BlakeTwo256 };
// Frontier
use fc_rpc::{
	EthBlockDataCacheTask,
	OverrideHandle,
	RuntimeApiStorageOverride,
	SchemaV1Override,
	SchemaV2Override,
	SchemaV3Override,
	StorageOverride,
};
use fp_storage::EthereumStorageSchema;

use fp_rpc::{ ConvertTransaction, ConvertTransactionRuntimeApi, EthereumRuntimeRPCApi };

use sp_api::{ BlockT, HeaderT, ProvideRuntimeApi };

/// A type representing all RPC extensions.
pub type RpcExtension = RpcModule<()>;

/// Extra dependencies for BABE.
pub struct BabeDeps {
	/// A handle to the BABE worker for issuing requests.
	pub babe_worker_handle: sc_consensus_babe::BabeWorkerHandle<Block>,
	/// The keystore that manages the keys of the node.
	pub keystore: KeystorePtr,
}

pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<OverrideHandle<Block>>
	where
		C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
		C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
		C: Send + Sync + 'static,
		C::Api: sp_api::ApiExt<Block> +
			fp_rpc::EthereumRuntimeRPCApi<Block> +
			fp_rpc::ConvertTransactionRuntimeApi<Block>,
		BE: Backend<Block> + 'static,
		BE::State: sc_client_api::StateBackend<BlakeTwo256>
{
	let mut overrides_map = BTreeMap::new();
	overrides_map.insert(
		EthereumStorageSchema::V1,
		Box::new(SchemaV1Override::new(client.clone())) as Box<dyn StorageOverride<_>>
	);
	overrides_map.insert(
		EthereumStorageSchema::V2,
		Box::new(SchemaV2Override::new(client.clone())) as Box<dyn StorageOverride<_>>
	);
	overrides_map.insert(
		EthereumStorageSchema::V3,
		Box::new(SchemaV3Override::new(client.clone())) as Box<dyn StorageOverride<_>>
	);

	Arc::new(OverrideHandle {
		schemas: overrides_map,
		fallback: Box::new(fc_rpc::RuntimeApiStorageOverride::new(client)),
	})
}
/// Dependencies for GRANDPA
pub struct GrandpaDeps<B> {
	/// Voting round info.
	pub shared_voter_state: sc_consensus_grandpa::SharedVoterState,
	/// Authority set info.
	pub shared_authority_set: sc_consensus_grandpa::SharedAuthoritySet<Hash, BlockNumber>,
	/// Receives notifications about justification events from Grandpa.
	pub justification_stream: sc_consensus_grandpa::GrandpaJustificationStream<Block>,
	/// Executor to drive the subscription manager in the Grandpa RPC handler.
	pub subscription_executor: sc_rpc::SubscriptionTaskExecutor,
	/// Finality proof provider.
	pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Dependencies for BEEFY
pub struct BeefyDeps {
	/// Receives notifications about finality proof events from BEEFY.
	pub beefy_finality_proof_stream: BeefyVersionedFinalityProofStream<Block>,
	/// Receives notifications about best block events from BEEFY.
	pub beefy_best_block_stream: BeefyBestBlockStream<Block>,
	/// Executor to drive the subscription manager in the BEEFY RPC handler.
	pub subscription_executor: sc_rpc::SubscriptionTaskExecutor,
}
/// Avalailable frontier backend types.
#[derive(Debug, Copy, Clone, Default, clap::ValueEnum)]
pub enum BackendType {
	/// Either RocksDb or ParityDb as per inherited from the global backend settings.
	#[default]
	KeyValue,
	/// Sql database with custom log indexing.
	Sql,
}

/// Full client dependencies
pub struct FullDeps<C, P, SC, B, A: ChainApi> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// The [`SelectChain`] Strategy
	pub select_chain: SC,
	/// A copy of the chain spec.
	pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// BABE specific dependencies.
	pub babe: BabeDeps,
	/// GRANDPA specific dependencies.
	pub grandpa: GrandpaDeps<B>,
	/// BEEFY specific dependencies.
	pub beefy: BeefyDeps,
	/// Backend used by the node.
	//pub backend: Arc<B>,
	//pub backend: Arc<dyn fc_db::BackendReader<Block> + Send + Sync>,
	pub backend: Arc<dyn fc_api::Backend<Block>>,
	pub sync: Arc<SyncingService<Block>>,

	/// Graph pool instance.
	pub graph: Arc<Pool<A>>,
	/// The Node authority flag
	pub is_authority: bool,
	/// Whether to enable dev signer
	pub enable_dev_signer: bool,
	/// Network service
	pub network: Arc<NetworkService<Block, Hash>>,
	/// EthFilterApi pool.
	pub filter_pool: Option<FilterPool>,
	/// Maximum number of logs in a query.
	pub max_past_logs: u32,
	/// Fee history cache.
	pub fee_history_cache: FeeHistoryCache,
	/// Maximum fee history cache size.
	pub fee_history_cache_limit: FeeHistoryCacheLimit,
	/// Ethereum data access overrides.
	pub overrides: Arc<OverrideHandle<Block>>,
	/// Cache for Ethereum block data.
	pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
	pub execute_gas_limit_multiplier: u64,

	/// Mandated parent hashes for a given block hash.
	pub forced_parent_hashes: Option<BTreeMap<H256, H256>>,

	// /// Something that can create the inherent data providers for pending state
	// pub pending_create_inherent_data_providers: CIDP,
	// pub converter: Option<CT>,
}

#[derive(Default)]
//// The ethereum-compatibility configuration used to run a node.
#[derive(Clone, Debug, clap::Parser)]
pub struct EthConfiguration {
	/// Maximum number of logs in a query.
	#[arg(long, default_value = "10000")]
	pub max_past_logs: u32,

	/// Maximum fee history cache size.
	#[arg(long, default_value = "2048")]
	pub fee_history_limit: u64,

	#[arg(long)]
	pub enable_dev_signer: bool,

	/// The dynamic-fee pallet target gas price set by block author
	#[arg(long, default_value = "1")]
	pub target_gas_price: u64,

	/// Maximum allowed gas limit will be `block.gas_limit * execute_gas_limit_multiplier`
	/// when using eth_call/eth_estimateGas.
	#[arg(long, default_value = "10")]
	pub execute_gas_limit_multiplier: u64,

	/// Size in bytes of the LRU cache for block data.
	#[arg(long, default_value = "50")]
	pub eth_log_block_cache: usize,

	/// Size in bytes of the LRU cache for transactions statuses data.
	#[arg(long, default_value = "50")]
	pub eth_statuses_cache: usize,

	/// Sets the frontier backend type (KeyValue or Sql)
	#[arg(long, value_enum, ignore_case = true, default_value_t = BackendType::default())]
	pub frontier_backend_type: BackendType,

	// Sets the SQL backend's pool size.
	#[arg(long, default_value = "100")]
	pub frontier_sql_backend_pool_size: u32,

	/// Sets the SQL backend's query timeout in number of VM ops.
	#[arg(long, default_value = "10000000")]
	pub frontier_sql_backend_num_ops_timeout: u32,

	/// Sets the SQL backend's auxiliary thread limit.
	#[arg(long, default_value = "4")]
	pub frontier_sql_backend_thread_count: u32,

	/// Sets the SQL backend's query timeout in number of VM ops.
	/// Default value is 200MB.
	#[arg(long, default_value = "209715200")]
	pub frontier_sql_backend_cache_size: u64,
}

///
pub fn create_full<C, P, SC, B, A, BE, EC: fc_rpc::EthConfig<Block, C>>(
	deps: FullDeps<C, P, SC, B, A>,
	subscription_task_executor: SubscriptionTaskExecutor,
	pubsub_notification_sinks: Arc<fc_mapping_sync::EthereumBlockNotificationSinks<fc_mapping_sync::EthereumBlockNotification<Block>>>
)
	-> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
	where
		C: ProvideRuntimeApi<Block> +
			HeaderBackend<Block> +
			AuxStore +
			HeaderMetadata<Block, Error = BlockChainError> +
			Send +
			Sync +
			'static,
		C::Api: frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
		C::Api: mmr_rpc::MmrRuntimeApi<Block, <Block as sp_runtime::traits::Block>::Hash, BlockNumber>,
		C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
		C::Api: BabeApi<Block>,
		C::Api: BlockBuilder<Block>,
		P: TransactionPool + Sync + Send + 'static,
		SC: SelectChain<Block> + 'static,
		B: sc_client_api::Backend<Block> + Send + Sync + 'static,
        C: BlockchainEvents<Block> + 'static,
		B::State: sc_client_api::StateBackend<sp_runtime::traits::HashingFor<Block>>,
		A: ChainApi<Block = Block> + 'static,
		C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,
        C: sp_api::CallApiAt<Block> + ProvideRuntimeApi<Block>,
		C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
        P: TransactionPool<Block=Block> + 'static,
		C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + StorageProvider<Block, BE> + UsageProvider<Block>,
		BE: Backend<Block> + 'static
{
	use frame_rpc_system::{ System, SystemApiServer };
	//  use mmr_rpc::{Mmr, MmrApiServer};
	use pallet_transaction_payment_rpc::{ TransactionPayment, TransactionPaymentApiServer };
	use sc_consensus_babe_rpc::{ Babe, BabeApiServer };
	use sc_consensus_beefy_rpc::{ Beefy, BeefyApiServer };
	use sc_consensus_grandpa_rpc::{ Grandpa, GrandpaApiServer };
	use sc_sync_state_rpc::{ SyncState, SyncStateApiServer };
	//use substrate_state_trie_migration_rpc::{StateMigration, StateMigrationApiServer};
	use fc_rpc::{
		Eth,
		EthApiServer,
		EthDevSigner,
		EthFilter,
		EthFilterApiServer,
		EthPubSub,
		EthPubSubApiServer,
		EthSigner,
		Net,
		NetApiServer,
		Web3,
		Web3ApiServer,
	};
	pub use substrate_state_trie_migration_rpc::{ StateMigration, StateMigrationApiServer };

	let mut io = RpcModule::new(());
	let FullDeps {
		client,
		pool,
		select_chain,
		chain_spec,
		deny_unsafe,
		babe,
		grandpa,
		beefy,
		backend,
		is_authority,
		enable_dev_signer,
		network,
		sync,
		filter_pool,
		//backend,
		max_past_logs,
		fee_history_cache,
		fee_history_cache_limit,
		overrides,
		block_data_cache,
		execute_gas_limit_multiplier,
		//beefy,
		forced_parent_hashes,
		graph,
		// pending_create_inherent_data_providers,
		// converter,
	} = deps;

	let BabeDeps { babe_worker_handle, keystore } = babe;
	let GrandpaDeps {
		shared_voter_state,
		shared_authority_set,
		justification_stream,
		subscription_executor,
		finality_provider,
	} = grandpa;

	let pp = pool.clone();
	// let pbp = pool.clone();

	//io.merge(StateMigration::new(client.clone(), backend.clone(), deny_unsafe).into_rpc())?;
	io.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
	io.merge(TransactionPayment::new(client.clone()).into_rpc())?;

	let mut signers = Vec::new();
	if enable_dev_signer {
		signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
	}
	io.merge(Babe::new(client.clone(), babe_worker_handle.clone(), keystore, select_chain, deny_unsafe).into_rpc())?;
	io.merge(
		Grandpa::new(
			subscription_executor,
			shared_authority_set.clone(),
			shared_voter_state,
			justification_stream,
			finality_provider
		).into_rpc()
	)?;
	io.merge(SyncState::new(chain_spec, client.clone(), shared_authority_set, babe_worker_handle)?.into_rpc())?;

	io.merge(
		Beefy::<Block>
			::new(beefy.beefy_finality_proof_stream, beefy.beefy_best_block_stream, beefy.subscription_executor)?
			.into_rpc()
	)?;


	io.merge(
		Eth::<Block, C, P, polkadot_runtime::TransactionConverter, BE, A, EC>::new(
			client.clone(),
			pp,
			graph.clone(),
			Some(polkadot_runtime::TransactionConverter),
			sync.clone(),
			signers,
			overrides.clone(),
			backend.clone(),
			is_authority,
			block_data_cache.clone(),
			fee_history_cache,
			fee_history_cache_limit,
			execute_gas_limit_multiplier,
			forced_parent_hashes,
		)
			//  .replace_config::<EC>()
			.into_rpc()
	)?;

	if let Some(filter_pool) = filter_pool {
		io.merge(
			EthFilter::new(
				client.clone(),
				backend,
				graph.clone(),
				filter_pool,
				500_usize, // max stored filters
				max_past_logs,
				block_data_cache.clone()
			).into_rpc()
		)?;
	}

	io.merge(
		EthPubSub::new(
			pool,
			client.clone(),
			sync,
			subscription_task_executor,
			overrides,
			pubsub_notification_sinks
		).into_rpc()
	)?;


	

	io.merge(
		Net::new(
			client.clone(),
			network,
			// Whether to format the `peer_count` response as Hex (default) or not.
			true
		).into_rpc()
	)?;

	io.merge(Web3::new(client.clone()).into_rpc())?;

	Ok(io)
}
