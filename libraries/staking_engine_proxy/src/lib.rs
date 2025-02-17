pub mod amount_group;
pub mod staking_engine_config;

use amount_group::AmountGroup;
use scrypto::prelude::*;
use staking_engine_config::*;

#[derive(ScryptoSbor)]
pub struct StakingEngineProxy {
  pub staking_engine: Owned<AnyComponent>,
}
impl StakingEngineProxy {
  // Instantiate Oracle at given package address
  pub fn initialize_staking_engine(oracle_package_address: PackageAddress, pool_config: StakingEngineConfig) -> Self {
    info!("[OracleProxy] initialize_oracle() oracle_package_address = {:?}", oracle_package_address);
    let result = ScryptoVmV1Api::blueprint_call(oracle_package_address, "StakingEngine", "instantiate", scrypto_args!(pool_config));
    Self {
      staking_engine: scrypto_decode(&result).unwrap(),
    }
  }

  pub fn update_config(&mut self, config_input: IndexSet<UpdateStakingEngineConfigInput>) {
    self.staking_engine.call_raw::<()>("update_config", scrypto_args!(config_input));
  }

  pub fn distribute(&self, stream: u16, gains: IndexMap<ResourceAddress, Decimal>) {
    self.staking_engine.call_raw::<()>("distribute", scrypto_args!(stream, gains));
  }

  pub fn deposit(&mut self, amount: Decimal) -> Decimal {
    self.staking_engine.call_raw::<Decimal>("deposit", scrypto_args!(amount))
  }

  pub fn get_pool_empty(&self) -> bool {
    self.staking_engine.call_raw::<bool>("get_pool_empty", scrypto_args!())
  }

  pub fn get_pool_staked_amount(&self) -> Decimal {
    self.staking_engine.call_raw::<Decimal>("get_pool_staked_amount", scrypto_args!())
  }

  pub fn get_staked_amount(&self, id: NonFungibleLocalId) -> Option<Decimal> {
    self.staking_engine.call_raw::<Option<Decimal>>("get_staked_amount", scrypto_args!(id))
  }

  pub fn get_redeemable_value(&self, id: NonFungibleLocalId) -> Option<(Decimal, IndexMap<u16, AmountGroup<Decimal>>)> {
    self
      .staking_engine
      .call_raw::<Option<(Decimal, IndexMap<u16, AmountGroup<Decimal>>)>>("get_redeemable_value", scrypto_args!(id))
  }

  pub fn contribute(&mut self, id: NonFungibleLocalId, amount: Decimal) -> (Decimal, Decimal) {
    self
      .staking_engine
      .call_raw::<(Decimal, Decimal)>("contribute", scrypto_args!(id, amount))
  }

  pub fn redeem(&mut self, id: NonFungibleLocalId, amount: Option<Decimal>) -> Decimal {
    self.staking_engine.call_raw::<Decimal>("redeem", scrypto_args!(id, amount))
  }

  pub fn claim(&mut self, id: NonFungibleLocalId, streams: IndexSet<u16>) -> IndexMap<u16, AmountGroup<Decimal>> {
    self
      .staking_engine
      .call_raw::<IndexMap<u16, AmountGroup<Decimal>>>("claim", scrypto_args!(id, streams))
  }
}
