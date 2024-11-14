use common::prelude::*;
use generate_config::GenerateConfig;
use scrypto::prelude::*;

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

#[test]
fn main2() {
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

  // println!("{:#?}", config);

  assert!(config.valuator_component == CONSENSUS_MANAGER);
  assert!(config.underlying_resources.contains("resource1"));

  config
    .update(indexset!(
      UpdateTestConfigInput::UnderlyingResources(UpdateSetInput::Remove("resource1".to_string())),
      UpdateTestConfigInput::ResourceMap("resource_key1".to_string(), Some(dec!(0.4))),
      UpdateTestConfigInput::ResourceMap("resource_key2".to_string(), None)
    ))
    .unwrap();

  // println!("{:#?}", config);

  assert!(config.resource_map.get("resource_key2").is_none());
  assert!(!config.underlying_resources.contains("resource1"));
  // assert!(config.check().is_ok());
}

trait CanBeChecked {
  fn is_a_rate(&self) -> bool;
}

impl CanBeChecked for &Decimal {
  fn is_a_rate(&self) -> bool {
    **self >= Decimal::ZERO && **self <= Decimal::ONE
  }
}
