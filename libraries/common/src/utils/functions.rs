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

pub fn check_recallable_resource(res_manager: ResourceManager) -> Result<()> {
  // If the resource defines recall roles, enforce they are DenyAll
  if let Some(recaller_role) = res_manager.get_role("recaller") {
    let updater_role = res_manager
      .get_role("recaller_updater")
      .ok_or_else(|| anyhow!("Recallable assets are not supported"))?;

    ensure!(
      recaller_role == AccessRule::DenyAll && updater_role == AccessRule::DenyAll,
      "Recallable assets are not supported"
    );
  }

  Ok(())
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
