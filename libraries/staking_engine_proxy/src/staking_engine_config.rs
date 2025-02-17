//! The staking engine config module is used to configure the staking engine running product.
//!
//! The precision define the number of digits and the number of decimal places to use for the staking engine running product.
//! if the running product falls below 10^-max_decimal_places, the pool running product will be scale up,
//! on the other hand, if the running product is above 10^max_digits, the pool running product will be scale down.
//! if max_digits or max_decimal_places are no set, the scaling up or down management will be disabled correspondingly.

use anyhow::Result;
use generate_config::GenerateConfig;
use scrypto::prelude::*;

#[derive(ScryptoSbor, Default, Clone, Debug, GenerateConfig)]
pub struct StakingEngineConfig {
  /// Defines the maximum number of digits allowed in the running product.
  #[check = "val.is_none() || (val.unwrap() > 0 && val.unwrap() <= 41)"]
  pub max_digits: Option<u8>,

  /// Defines the maximum number of decimal places allowed in the running product.
  #[check = "val.is_none() || (val.unwrap() > 0 && val.unwrap() <= 36)"]
  pub max_decimal_places: Option<u8>,

  /// Defines the maximum number of resources allowed to be distributed as rewards.
  /// This is to avoid possible state explosion.
  #[check = "val.is_none() || val.unwrap() > 0"]
  pub max_resource_count: Option<u32>,
}
