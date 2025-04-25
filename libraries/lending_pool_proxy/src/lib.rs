//!
//! The lending pool act as a liquidity layer and grant access the liquidity pool via a badge.
//!
//! This module implement a struct used to call methods of the lending pool component.
//! it also hold the badge require to authenticate calls to the lending pool component involved in borrowing operations.
//!  
use anyhow::{bail, Result};
use common::prelude::CacheEntry;
use scrypto::prelude::*;
use utils::InstantUtils;

/// Lending pool proxy
/// A proxy struct to call protected or unprotected methods of the lending pool
#[derive(ScryptoSbor)]
pub struct LendingPoolProxy {
  /// Lending Market client badge required to have access to protected methods of the lending pool
  pub client_badges: KeyValueStore<ComponentAddress, NonFungibleVault>,

  pub registered_lending_pools: KeyValueStore<ResourceAddress, (ComponentAddress, bool)>,

  /// Cache for loan unit ratio (Updated on every transaction)
  pub loan_unit_ratio_cache: KeyValueStore<(ComponentAddress, ResourceAddress), CacheEntry<PreciseDecimal>>,
}

impl LendingPoolProxy {
  pub fn set_client_badge(&mut self, client_badge: NonFungibleBucket) -> (ComponentAddress, ResourceAddress) {
    let client_badge_address = client_badge.resource_address();
    let (lending_pool_component, badge_set) = *self.registered_lending_pools.get(&client_badge_address).unwrap();

    if self.client_badges.get(&lending_pool_component).is_none() {
      self
        .client_badges
        .insert(lending_pool_component, NonFungibleVault::with_bucket(client_badge))
    } else {
      self.client_badges.get_mut(&lending_pool_component).unwrap().put(client_badge)
    };

    if !badge_set {
      self.registered_lending_pools.insert(client_badge_address, (lending_pool_component, true));
    }

    (lending_pool_component, client_badge_address)
  }

  pub fn register_lending_pool(&mut self, lending_pool_address: ComponentAddress, client_badge_address: ResourceAddress) -> Result<()> {
    if let Some(value) = self.registered_lending_pools.get(&client_badge_address) {
      if value.1 {
        bail!("Client badge already registered");
      } else {
        self.registered_lending_pools.insert(client_badge_address, (lending_pool_address, false));
      }
    } else {
      self.registered_lending_pools.insert(client_badge_address, (lending_pool_address, false));
    }

    Ok(())
  }

  /// Get Loan unit ratio in batch for provided resources
  /// The response is cached for the current transaction
  pub fn get_loan_unit_ratio(
    &mut self,
    component: ComponentAddress,
    res_addresses: IndexSet<ResourceAddress>,
  ) -> IndexMap<ResourceAddress, PreciseDecimal> {
    let lending_pool: Global<AnyComponent> = component.into();

    let hash = Runtime::transaction_hash();
    let mut res = IndexMap::new();
    let mut res_address_to_update = IndexSet::new();

    for res_address in &res_addresses {
      if let Some(entry) = self.loan_unit_ratio_cache.get(&(component, *res_address)) {
        if entry.transaction_hash == hash {
          res.insert(*res_address, entry.cached_value);
        } else {
          res_address_to_update.insert(*res_address);
        }
      } else {
        res_address_to_update.insert(*res_address);
      }
    }

    if res_address_to_update.is_empty() {
      return res;
    }

    let new_values =
      lending_pool.call_raw::<IndexMap<ResourceAddress, Option<PreciseDecimal>>>("get_loan_unit_ratio", scrypto_args!(res_address_to_update));

    for (res_address, new_value) in new_values {
      if let Some(new_value) = new_value {
        self.loan_unit_ratio_cache.insert(
          (component, res_address),
          CacheEntry {
            transaction_hash: hash,
            cached_value: new_value,
            timestamp: Instant::now(),
          },
        );
        res.insert(res_address, new_value);
      }
    }

    res
  }

  /// A proxy method to call protected_borrow on the lending pool
  /// This method is protected by the client badge
  pub fn protected_borrow(&mut self, component: ComponentAddress, resources: IndexMap<ResourceAddress, Decimal>) -> Vec<(FungibleBucket, Decimal)> {
    let lending_pool: Global<AnyComponent> = component.into();

    let client_badge_ref = self.client_badges.get(&component).unwrap();

    let client_badge_proof = client_badge_ref.create_proof_of_non_fungibles(&client_badge_ref.non_fungible_local_ids(1));

    lending_pool.call_raw::<Vec<(FungibleBucket, Decimal)>>("protected_borrow", scrypto_args!(client_badge_proof, resources))
  }

  /// A proxy method to call protected_repay on the lending pool
  /// This method is protected by the client badge
  pub fn protected_repay(
    &mut self,
    component: ComponentAddress,
    resources: Vec<(FungibleBucket, Option<Decimal>)>,
  ) -> Vec<(Decimal, FungibleBucket, Decimal)> {
    let lending_pool: Global<AnyComponent> = component.into();

    let client_badge_ref = self.client_badges.get(&component).unwrap();

    let client_badge_proof = client_badge_ref.create_proof_of_non_fungibles(&client_badge_ref.non_fungible_local_ids(1));

    lending_pool.call_raw::<Vec<(Decimal, FungibleBucket, Decimal)>>("protected_repay", scrypto_args!(client_badge_proof, resources))
  }
}
