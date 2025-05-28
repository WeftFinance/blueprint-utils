//!
//! The lending market component is a client of the lending pool component.
//! The lending pool act as a liquidity layer and grant access the liquidity pool via a badge.
//!
//! This module implement a struct used to call methods of the lending pool component.
//! it also hold the badge require to authenticate calls to the lending pool component involved in lending operations.
//!  
use anyhow::Result;
use common::prelude::CacheEntry;
use scrypto::prelude::*;
use utils::InstantUtils;

/// Lending pool proxy
/// A proxy struct to call protected or unprotected methods of the lending pool
#[derive(ScryptoSbor)]
pub struct LendingPoolProxy {
  /// Attached lending pool component
  pub lending_pool: Global<AnyComponent>,

  /// Lending Market client badge required to have access to protected methods of the lending pool
  pub client_badge: NonFungibleVault,

  /// Cache for loan unit ratio (Updated on every transaction)
  pub loan_unit_ratio_cache: KeyValueStore<ResourceAddress, CacheEntry<PreciseDecimal>>,

  /// Cache for deposit unit ratio (Updated on every transaction)
  pub deposit_unit_ratio_cache: KeyValueStore<ResourceAddress, CacheEntry<PreciseDecimal>>,
}

impl LendingPoolProxy {
  pub fn get_deposit_unit_reverse_address(&mut self, res_address: ResourceAddress) -> Option<ResourceAddress> {
    self
      .lending_pool
      .call_raw::<Option<ResourceAddress>>("get_deposit_unit_reverse_address", scrypto_args!(res_address))
  }

  pub fn get_deposit_unit_address(&mut self, res_address: ResourceAddress) -> Option<ResourceAddress> {
    self
      .lending_pool
      .call_raw::<Option<ResourceAddress>>("get_deposit_unit_address", scrypto_args!(res_address))
  }

  /// Get Loan unit ratio in batch for provided resources
  /// The response is cached for the current transaction
  pub fn get_loan_unit_ratio(&mut self, res_addresses: IndexSet<ResourceAddress>) -> IndexMap<ResourceAddress, PreciseDecimal> {
    self.get_unit_ratios(res_addresses, true, "get_loan_unit_ratio")
  }

  /// Get Deposit unit ratio in batch for provided resources
  /// The response is cached for the current transaction
  pub fn get_deposit_unit_ratio(&mut self, res_addresses: IndexSet<ResourceAddress>) -> IndexMap<ResourceAddress, PreciseDecimal> {
    self.get_unit_ratios(res_addresses, false, "get_deposit_unit_ratio")
  }

  /// A proxy method to call protected_borrow on the lending pool
  /// This method is protected by the client badge
  pub fn protected_borrow(&mut self, resources: IndexMap<ResourceAddress, Decimal>) -> Vec<(FungibleBucket, Decimal)> {
    let client_badge_proof = self
      .client_badge
      .create_proof_of_non_fungibles(&self.client_badge.non_fungible_local_ids(1));

    self
      .lending_pool
      .call_raw::<Vec<(FungibleBucket, Decimal)>>("protected_borrow", scrypto_args!(client_badge_proof, resources))
  }

  /// A proxy method to call protected_repay on the lending pool
  /// This method is protected by the client badge
  pub fn protected_repay(&mut self, resources: Vec<(FungibleBucket, Option<Decimal>)>) -> Vec<(Decimal, FungibleBucket, Decimal)> {
    let client_badge_proof = self
      .client_badge
      .create_proof_of_non_fungibles(&self.client_badge.non_fungible_local_ids(1));

    self
      .client_badge
      .authorize_with_non_fungibles(&self.client_badge.non_fungible_local_ids(10), || {
        self
          .lending_pool
          .call_raw::<Vec<(Decimal, FungibleBucket, Decimal)>>("protected_repay", scrypto_args!(client_badge_proof, resources))
      })
  }

  // Private methods

  fn get_unit_ratios(
    &mut self,
    res_addresses: IndexSet<ResourceAddress>,
    loan_unit_ratio: bool,
    method_name: &str,
  ) -> IndexMap<ResourceAddress, PreciseDecimal> {
    let hash = Runtime::transaction_hash();
    let mut res = IndexMap::new();
    let mut res_address_to_update = IndexSet::new();

    let cache = if loan_unit_ratio {
      &mut self.loan_unit_ratio_cache
    } else {
      &mut self.deposit_unit_ratio_cache
    };

    for res_address in &res_addresses {
      if let Some(entry) = cache.get(res_address) {
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

    let new_values = self
      .lending_pool
      .call_raw::<IndexMap<ResourceAddress, Option<PreciseDecimal>>>(method_name, scrypto_args!(res_address_to_update));

    for (res_address, new_value) in new_values {
      if let Some(new_value) = new_value {
        cache.insert(
          res_address,
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
}
