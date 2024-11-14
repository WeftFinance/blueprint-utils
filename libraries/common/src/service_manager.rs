use super::service_status::{ServiceStatus, ServiceVariantProvider, StatusChangeType};
use anyhow::{anyhow, Result};
use scrypto::prelude::rust::hash::Hash;
use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct ServiceManager<K: ScryptoSbor + Hash + Copy, T: ScryptoSbor + ServiceVariantProvider + Eq + Clone + Hash + Debug> {
  entries: KeyValueStore<K, ServiceStatus<T>>,
}

impl<K: ScryptoSbor + Hash + Copy, T: ScryptoSbor + ServiceVariantProvider + Eq + Clone + Hash + Debug> ServiceManager<K, T> {
  pub fn new(entries: KeyValueStore<K, ServiceStatus<T>>) -> Self {
    Self { entries }
  }

  pub fn new_entry(&mut self, key: K) -> Result<()> {
    self.entries.insert(key, ServiceStatus::new());
    Ok(())
  }

  pub fn check(&self, key: K, service: T) -> Result<bool> {
    let entry = self.entries.get(&key).ok_or_else(|| anyhow!("Service is not set"))?;
    Ok(entry.check(&service))
  }

  pub fn assert(&self, key: K, service: &T) -> Result<()> {
    let entry = self.entries.get(&key).ok_or_else(|| anyhow!("Service is not set"))?;
    entry.assert_active(service)
  }

  pub fn update(&mut self, key: K, service: T, new_status: bool, status_change_type: StatusChangeType) -> Result<()> {
    let mut entry = self.entries.get_mut(&key).ok_or_else(|| anyhow!("Service is not set"))?;
    entry.set_status(service, new_status, status_change_type)
  }
}
