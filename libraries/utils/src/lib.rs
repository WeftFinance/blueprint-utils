use anyhow::{anyhow, ensure, Result};
use scrypto::prelude::*;

pub fn check_lsu(input_lsu_address: ResourceAddress) -> Option<ComponentAddress> {
  let metadata: GlobalAddress = ResourceManager::from(input_lsu_address).get_metadata("validator").ok()??;

  let validator_address: ComponentAddress = metadata.try_into().ok()?;

  let validator: Global<Validator> = validator_address.into();

  let lsu_address: GlobalAddress = validator.get_metadata("pool_unit").ok()??;

  if input_lsu_address == ResourceAddress::try_from(lsu_address).unwrap() {
    Some(validator_address)
  } else {
    None
  }
}

pub fn check_claim_nft(input_claim_nft_address: ResourceAddress) -> Option<ComponentAddress> {
  let metadata: GlobalAddress = ResourceManager::from(input_claim_nft_address).get_metadata("validator").ok()??;

  let validator_address: ComponentAddress = metadata.try_into().ok()?;

  let validator: Global<Validator> = validator_address.into();

  let claim_nft_address: GlobalAddress = validator.get_metadata("claim_nft").ok()??;

  if input_claim_nft_address == ResourceAddress::try_from(claim_nft_address).unwrap() {
    Some(validator_address)
  } else {
    None
  }
}

pub fn check_recallable_resource(res_manager: ResourceManager) {
  // Check if the collateral pool supports recall
  if let Some(role) = res_manager.get_role("recaller") {
    res_manager.get_role("recaller_updater").map(|updater_role| {
      ensure!(
        role == AccessRule::DenyAll && updater_role == AccessRule::DenyAll,
        "Recallable assets are not supported"
      );
      Ok(())
    });
  }
}

pub trait CanBeChecked {
  fn is_a_rate(&self) -> bool;

  fn is_zero(&self) -> bool;

  fn is_positive(&self) -> bool;

  fn is_zero_or_positive(&self) -> bool {
    self.is_positive() || self.is_zero()
  }
}

impl CanBeChecked for Decimal {
  fn is_a_rate(&self) -> bool {
    *self >= Decimal::ZERO && *self <= Decimal::ONE
  }

  fn is_positive(&self) -> bool {
    Decimal::is_positive(self)
  }

  fn is_zero(&self) -> bool {
    Decimal::is_zero(self)
  }
}

impl CanBeChecked for IndexMap<ResourceAddress, Decimal> {
  fn is_a_rate(&self) -> bool {
    self.values().all(CanBeChecked::is_a_rate)
  }

  fn is_positive(&self) -> bool {
    self.values().all(CanBeChecked::is_positive)
  }

  fn is_zero(&self) -> bool {
    self.values().all(CanBeChecked::is_zero)
  }
}

#[macro_export]
macro_rules! create_event {
    ($name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        #[derive(ScryptoSbor, ScryptoEvent)]
        pub struct $name {
            $(pub $field: $type),*
        }
        impl  $name {
         pub fn emit(self) {
            Runtime::emit_event(self)
          }
        }
    };
    ($name:ident($data_type:ty)) => {
        #[derive(ScryptoSbor, ScryptoEvent)]
        pub struct $name (pub $data_type);
        impl $name {
         pub fn emit(self) {
            Runtime::emit_event(self)
          }
        }
    };
}

#[macro_export]
macro_rules! check_field_invalidity {
  ($config:ident, $get_default_config:ident, $field:ident, $valid_values:expr, $invalid_values:expr) => {
    paste::paste! {

        #[test]
        fn [<valid_ $field>]() {
          let default_config = $get_default_config();

          for valid_value in $valid_values {
              let config = $config {
                  $field: valid_value,
                  ..default_config.clone()
              };
             config.check().unwrap();
          }
        }

        #[test]
        fn [<invalid_ $field>]() {
          let default_config = $get_default_config();

          for invalid_value in $invalid_values {
              let config = $config {
                  $field: invalid_value,
                  ..default_config.clone()
              };
              assert!(config.check().is_err());
          }
        }
    }
  };
}

pub trait InstantUtils {
  fn now() -> Instant;
  fn checked_sub(&self, other: Instant) -> Option<u64>;
}

impl InstantUtils for Instant {
  fn now() -> Instant {
    Clock::current_time(TimePrecision::Minute)
  }

  fn checked_sub(&self, other: Instant) -> Option<u64> {
    self
      .seconds_since_unix_epoch
      .checked_sub(other.seconds_since_unix_epoch)
      .map(|x| x as u64)
  }
}

/// Helper function to safely truncate a precise decimal to decimal with the rounding strategy adopted as default
pub fn checked_truncate(amount: PreciseDecimal) -> Result<Decimal> {
  amount
    .checked_truncate(RoundingMode::ToNearestMidpointTowardZero)
    .ok_or(anyhow!("Truncation failed"))
}

/// Helper function to safely round a decimal to decimal with the rounding strategy adopted as default
pub fn checked_round(amount: Decimal, inner_precision: u8) -> Result<Decimal> {
  amount
    .checked_round(inner_precision, RoundingMode::ToNearestMidpointTowardZero)
    .ok_or(anyhow!("Rounding failed"))
}
