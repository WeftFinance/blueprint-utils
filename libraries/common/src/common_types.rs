use scrypto::prelude::*;
use utils::CanBeChecked;

/// Define an update set
#[derive(ScryptoSbor, ManifestSbor, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpdateSetInput<T> {
  Add(T),
  Remove(T),
}

/// Define an empty badge
#[derive(ScryptoSbor, NonFungibleData)]
pub struct EmptyBadgeData {}

/// Define options for deposit limit
#[derive(ScryptoSbor, ManifestSbor, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DepositLimitType {
  /// No limit
  None,

  /// Limit by amount
  Amount(Decimal),

  /// Limit by a ratio of the resource total supply
  SupplyRatio(Decimal),
}
impl DepositLimitType {
  pub fn check(&self) -> bool {
    match self {
      DepositLimitType::None => true,
      DepositLimitType::Amount(amount) => amount.is_zero_or_positive(),
      DepositLimitType::SupplyRatio(ratio) => ratio.is_a_rate(),
    }
  }

  pub fn check_limit(&self, res_address: ResourceAddress, value: Decimal) -> bool {
    match self {
      DepositLimitType::None => true,
      DepositLimitType::Amount(amount) => value <= *amount,
      DepositLimitType::SupplyRatio(ratio) => {
        let res_manager: ResourceManager = res_address.into();
        if let Some(total_supply) = res_manager.total_supply() {
          value <= total_supply.checked_mul(*ratio).unwrap()
        } else {
          true
        }
      }
    }
  }
}

/// Define a cache entry
#[derive(ScryptoSbor, Clone)]
pub struct CacheEntry<T: Clone> {
  pub transaction_hash: Hash,
  pub cached_value: T,
  pub timestamp: Instant,
}
