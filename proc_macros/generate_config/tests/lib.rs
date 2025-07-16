use generate_config::GenerateConfig;
use scrypto::prelude::*;
use utils::prelude::*;

#[derive(Debug, GenerateConfig)]
pub struct TestConfig {
  pub valuator_component: ComponentAddress,
  pub valuator_method: String,
  pub is_enabled: bool,
  #[check = "val.is_a_rate()"]
  pub rate: Decimal,
  pub underlying_resources: BTreeSet<String>,
  pub resource_map: BTreeMap<String, Decimal>,
}

#[derive(Debug, GenerateConfig)]
pub struct SimpleConfig {
  pub name: String,
  pub value: u32,
}

#[derive(Debug, GenerateConfig)]
pub struct ValidationConfig {
  #[check = "val.is_positive()"]
  pub amount: Decimal,
  #[check = "val.is_valid_percentage()"]
  pub percentage: Decimal,
}

#[test]
fn test_basic_update_functionality() {
  let mut config = TestConfig {
    valuator_component: GENESIS_HELPER,
    valuator_method: "method".to_string(),
    is_enabled: true,
    rate: dec!(0.5),
    underlying_resources: BTreeSet::new(),
    resource_map: BTreeMap::new(),
  };

  let mut updates = IndexSet::new();
  updates.insert(UpdateTestConfigInput::ValuatorComponent(CONSENSUS_MANAGER));
  updates.insert(UpdateTestConfigInput::Rate(dec!(0.8)));
  updates.insert(UpdateTestConfigInput::UnderlyingResources(UpdateSetInput::Add("resource1".to_string())));
  updates.insert(UpdateTestConfigInput::ResourceMap("resource_key1".to_string(), Some(dec!(1.2))));
  updates.insert(UpdateTestConfigInput::ResourceMap("resource_key2".to_string(), Some(dec!(1.3))));

  config.update(updates).unwrap();

  assert_eq!(config.valuator_component, CONSENSUS_MANAGER);
  assert_eq!(config.rate, dec!(0.8));
  assert!(config.underlying_resources.contains("resource1"));
  assert_eq!(config.resource_map.get("resource_key1"), Some(&dec!(1.2)));
  assert_eq!(config.resource_map.get("resource_key2"), Some(&dec!(1.3)));

  config
    .update(indexset!(
      UpdateTestConfigInput::UnderlyingResources(UpdateSetInput::Remove("resource1".to_string())),
      UpdateTestConfigInput::ResourceMap("resource_key1".to_string(), Some(dec!(0.4))),
      UpdateTestConfigInput::ResourceMap("resource_key2".to_string(), None)
    ))
    .unwrap();

  assert!(!config.resource_map.contains_key("resource_key2"));
  assert!(!config.underlying_resources.contains("resource1"));
  assert_eq!(config.resource_map.get("resource_key1"), Some(&dec!(0.4)));
  assert!(config.check().is_ok());
}

#[test]
fn test_simple_config_update() {
  let mut config = SimpleConfig {
    name: "test".to_string(),
    value: 42,
  };

  let updates = indexset!(
    UpdateSimpleConfigInput::Name("updated_test".to_string()),
    UpdateSimpleConfigInput::Value(100)
  );

  config.update(updates).unwrap();

  assert_eq!(config.name, "updated_test");
  assert_eq!(config.value, 100);
}

#[test]
fn test_set_operations() {
  let mut config = TestConfig {
    valuator_component: GENESIS_HELPER,
    valuator_method: "method".to_string(),
    is_enabled: true,
    rate: dec!(0.5),
    underlying_resources: BTreeSet::new(),
    resource_map: BTreeMap::new(),
  };

  // Test adding to set
  config
    .update(indexset!(
      UpdateTestConfigInput::UnderlyingResources(UpdateSetInput::Add("resource1".to_string())),
      UpdateTestConfigInput::UnderlyingResources(UpdateSetInput::Add("resource2".to_string()))
    ))
    .unwrap();

  assert!(config.underlying_resources.contains("resource1"));
  assert!(config.underlying_resources.contains("resource2"));
  assert_eq!(config.underlying_resources.len(), 2);

  // Test removing from set
  config
    .update(indexset!(UpdateTestConfigInput::UnderlyingResources(UpdateSetInput::Remove(
      "resource1".to_string()
    ))))
    .unwrap();

  assert!(!config.underlying_resources.contains("resource1"));
  assert!(config.underlying_resources.contains("resource2"));
  assert_eq!(config.underlying_resources.len(), 1);

  // Test removing non-existent item (should not fail)
  config
    .update(indexset!(UpdateTestConfigInput::UnderlyingResources(UpdateSetInput::Remove(
      "non_existent".to_string()
    ))))
    .unwrap();

  assert_eq!(config.underlying_resources.len(), 1);
}

#[test]
fn test_map_operations() {
  let mut config = TestConfig {
    valuator_component: GENESIS_HELPER,
    valuator_method: "method".to_string(),
    is_enabled: true,
    rate: dec!(0.5),
    underlying_resources: BTreeSet::new(),
    resource_map: BTreeMap::new(),
  };

  // Test adding to map
  config
    .update(indexset!(
      UpdateTestConfigInput::ResourceMap("key1".to_string(), Some(dec!(1.0))),
      UpdateTestConfigInput::ResourceMap("key2".to_string(), Some(dec!(2.0)))
    ))
    .unwrap();

  assert_eq!(config.resource_map.get("key1"), Some(&dec!(1.0)));
  assert_eq!(config.resource_map.get("key2"), Some(&dec!(2.0)));
  assert_eq!(config.resource_map.len(), 2);

  // Test updating existing key
  config
    .update(indexset!(UpdateTestConfigInput::ResourceMap("key1".to_string(), Some(dec!(1.5)))))
    .unwrap();

  assert_eq!(config.resource_map.get("key1"), Some(&dec!(1.5)));
  assert_eq!(config.resource_map.len(), 2);

  // Test removing from map
  config
    .update(indexset!(UpdateTestConfigInput::ResourceMap("key1".to_string(), None)))
    .unwrap();

  assert!(!config.resource_map.contains_key("key1"));
  assert_eq!(config.resource_map.get("key2"), Some(&dec!(2.0)));
  assert_eq!(config.resource_map.len(), 1);

  // Test removing non-existent key (should not fail)
  config
    .update(indexset!(UpdateTestConfigInput::ResourceMap("non_existent".to_string(), None)))
    .unwrap();

  assert_eq!(config.resource_map.len(), 1);
}

#[test]
fn test_validation_success() {
  let mut config = ValidationConfig {
    amount: dec!(10.0),
    percentage: dec!(0.5),
  };

  // Valid updates should succeed
  config
    .update(indexset!(
      UpdateValidationConfigInput::Amount(dec!(20.0)),
      UpdateValidationConfigInput::Percentage(dec!(0.75))
    ))
    .unwrap();

  assert_eq!(config.amount, dec!(20.0));
  assert_eq!(config.percentage, dec!(0.75));
}

#[test]
fn test_validation_failure() {
  let mut config = ValidationConfig {
    amount: dec!(10.0),
    percentage: dec!(0.5),
  };

  // Invalid amount (negative)
  let result = config.update(indexset!(UpdateValidationConfigInput::Amount(dec!(-5.0))));
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("Invalid ValidationConfig::amount"));
  // Value is changed but validation failed, so it remains changed
  assert_eq!(config.amount, dec!(-5.0));

  // Reset to valid state
  config.amount = dec!(10.0);

  // Invalid percentage (> 1.0)
  let result = config.update(indexset!(UpdateValidationConfigInput::Percentage(dec!(1.5))));
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("Invalid ValidationConfig::percentage"));
  // Value is changed but validation failed, so it remains changed
  assert_eq!(config.percentage, dec!(1.5));

  // Amount should still be the reset value
  assert_eq!(config.amount, dec!(10.0));
}

#[test]
fn test_rate_validation() {
  let mut config = TestConfig {
    valuator_component: GENESIS_HELPER,
    valuator_method: "method".to_string(),
    is_enabled: true,
    rate: dec!(0.5),
    underlying_resources: BTreeSet::new(),
    resource_map: BTreeMap::new(),
  };

  // Valid rate should work
  config.update(indexset!(UpdateTestConfigInput::Rate(dec!(0.8)))).unwrap();
  assert_eq!(config.rate, dec!(0.8));

  // Invalid rate (> 1.0) should fail
  let result = config.update(indexset!(UpdateTestConfigInput::Rate(dec!(1.5))));
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("Invalid TestConfig::rate"));
  // Value is changed but validation failed, so it remains changed
  assert_eq!(config.rate, dec!(1.5));

  // Invalid rate (< 0.0) should fail
  let result = config.update(indexset!(UpdateTestConfigInput::Rate(dec!(-0.1))));
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("Invalid TestConfig::rate"));
  // Value is changed but validation failed, so it remains changed
  assert_eq!(config.rate, dec!(-0.1));
}

#[test]
fn test_empty_update_set() {
  let mut config = SimpleConfig {
    name: "test".to_string(),
    value: 42,
  };

  let original_name = config.name.clone();
  let original_value = config.value;

  // Empty update set should not change anything
  config.update(IndexSet::new()).unwrap();

  assert_eq!(config.name, original_name);
  assert_eq!(config.value, original_value);
}

#[test]
fn test_check_method_directly() {
  let valid_config = TestConfig {
    valuator_component: GENESIS_HELPER,
    valuator_method: "method".to_string(),
    is_enabled: true,
    rate: dec!(0.5),
    underlying_resources: BTreeSet::new(),
    resource_map: BTreeMap::new(),
  };

  assert!(valid_config.check().is_ok());

  let invalid_config = TestConfig {
    valuator_component: GENESIS_HELPER,
    valuator_method: "method".to_string(),
    is_enabled: true,
    rate: dec!(1.5), // Invalid rate
    underlying_resources: BTreeSet::new(),
    resource_map: BTreeMap::new(),
  };

  let result = invalid_config.check();
  assert!(result.is_err());
  assert!(result.unwrap_err().contains("Invalid TestConfig::rate"));
}

#[test]
fn test_multiple_validation_failures() {
  let mut config = ValidationConfig {
    amount: dec!(10.0),
    percentage: dec!(0.5),
  };

  // Try to update with multiple invalid values
  let result = config.update(indexset!(
    UpdateValidationConfigInput::Amount(dec!(-5.0)),
    UpdateValidationConfigInput::Percentage(dec!(1.5))
  ));

  assert!(result.is_err());
  // Should fail on first validation error (amount)
  assert!(result.unwrap_err().contains("Invalid ValidationConfig::amount"));

  // Values are changed but validation failed, so they remain changed
  assert_eq!(config.amount, dec!(-5.0));
  assert_eq!(config.percentage, dec!(1.5));
}

trait CanBeChecked {
  fn is_a_rate(&self) -> bool;
  // fn is_positive(&self) -> bool;
  fn is_valid_percentage(&self) -> bool;
}

impl CanBeChecked for &Decimal {
  fn is_a_rate(&self) -> bool {
    **self >= Decimal::ZERO && **self <= Decimal::ONE
  }

  // fn is_positive(&self) -> bool {
  //   **self > Decimal::ZERO
  // }

  fn is_valid_percentage(&self) -> bool {
    **self >= Decimal::ZERO && **self <= Decimal::ONE
  }
}
