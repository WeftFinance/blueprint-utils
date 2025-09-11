use common::prelude::*;
use generate_config::GenerateConfig;
use scrypto::prelude::*;

generate_service_variants!(
  pub enum MarketService,
  (ScryptoSbor,ManifestSbor, Debug, Clone, Copy, PartialEq, Eq, Hash),

  // CreateCDP: creates a new CDP.
  CreateCDP,

  // UpdateCDP: updates an existing CDP.
  UpdateCDP,

  // BurnCDP: burns (destroys) a CDP.
  BurnCDP
);

#[test]
fn test_new_app_service_status() {
  let status = ServiceStatus::<MarketService>::new();

  for service in MarketService::variants() {
    assert!(status.check(&service));
  }
}

#[test]
fn test_set_status() {
  let mut status = ServiceStatus::new();

  for service in MarketService::variants() {
    // OperatingStatus::Enabled(true)

    assert!(status.set_status(service, true, StatusChangeType::ModeratorSet).is_ok());

    assert!(status.check(&service));

    // OperatingStatus::Enabled(false)

    assert!(status.set_status(service, false, StatusChangeType::ModeratorSet).is_ok());

    assert!(!status.check(&service));

    // OperatingStatus::AdminEnabled(true)

    assert!(status.set_status(service, true, StatusChangeType::AdminSetAndLock).is_ok());

    assert!(status.check(&service));

    // OperatingStatus::AdminEnabled(false)

    assert!(status.set_status(service, false, StatusChangeType::AdminSetAndLock).is_ok());

    assert!(!status.check(&service));
  }
}

#[test]
fn test_set_roles() {
  let mut status = ServiceStatus::new();

  for service in MarketService::variants() {
    // Moderator action on default status (Should succeed)
    assert!(status.set_status(service, true, StatusChangeType::ModeratorSet).is_ok());

    assert!(status.set_status(service, false, StatusChangeType::ModeratorSet).is_ok());

    // Admin not locking action on status set by moderator (Should succeed)
    assert!(status.set_status(service, false, StatusChangeType::AdminSetAndUnlock).is_ok());

    assert!(status.set_status(service, true, StatusChangeType::AdminSetAndUnlock).is_ok());

    // Moderator action on not locked status (Should succeed)
    assert!(status.set_status(service, true, StatusChangeType::ModeratorSet).is_ok());

    assert!(status.set_status(service, false, StatusChangeType::ModeratorSet).is_ok());

    // Admin overriding status (Should succeed)
    assert!(status.set_status(service, false, StatusChangeType::AdminSetAndLock).is_ok());

    assert!(status.set_status(service, true, StatusChangeType::AdminSetAndLock).is_ok());

    // Moderator overriding status (Should fail)
    assert!(status.set_status(service, true, StatusChangeType::ModeratorSet).is_err());

    assert!(status.set_status(service, false, StatusChangeType::ModeratorSet).is_err());
  }
}

#[derive(ScryptoSbor, ManifestSbor, Debug, Clone, PartialEq, Eq, Hash, GenerateConfig)]
struct TestConfig {
  #[check = "val.is_positive()"]
  amount: Decimal,
  #[check = "val.is_a_rate()"]
  rate: Decimal,
}

updatable_config!(TestConfig);

#[test]
fn test_updatable_config_macro() {
  let mut config = TestConfig {
    amount: Decimal::ONE,
    rate: dec!("0.5"),
  };

  assert!(config.check().is_ok());

  let updates = indexset![UpdateTestConfigInput::Amount(Decimal::from(10)), UpdateTestConfigInput::Rate(dec!("0.8"))];

  assert!(config.update(updates).is_ok());
  assert_eq!(config.amount, Decimal::from(10));
  assert_eq!(config.rate, dec!("0.8"));

  // Test validation methods work correctly
  assert!(config.amount.is_positive());
  assert!(config.rate.is_a_rate());
}
