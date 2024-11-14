use anyhow::{anyhow, Result};
use scrypto::prelude::rust::hash::Hash;
use scrypto::prelude::*;
use utils::InstantUtils;

#[derive(ScryptoSbor, Clone)]
pub enum ConfigurationKey<K: ScryptoSbor + Hash + Copy> {
  Current(K),
  History(u64),
}

#[derive(ScryptoSbor, Clone)]
pub struct ConfigurationEntry<K: ScryptoSbor + Hash + Copy, C: ScryptoSbor> {
  key: K,
  entry: C,
  version: u64,
  expiration_time: Option<Instant>,
}

#[derive(ScryptoSbor)]
pub struct ConfigurationManager<K: ScryptoSbor + Hash + Copy, C: ScryptoSbor + Clone + Updatable<U>, U: ScryptoSbor> {
  // Config
  track_history: bool,
  get_config_error_message: String,
  default_expiration_time: Option<u64>,
  // State
  version_count: u64,
  entry_count: u16,
  entries: KeyValueStore<ConfigurationKey<K>, ConfigurationEntry<K, C>>,
  phantom_data: Option<U>,
}

impl<K: ScryptoSbor + Hash + Copy, C: ScryptoSbor + Clone + Updatable<U>, U: ScryptoSbor> ConfigurationManager<K, C, U> {
  pub fn new(
    track_history: bool,
    default_expiration_time: Option<u64>,
    get_config_error_message: String,
    entries: KeyValueStore<ConfigurationKey<K>, ConfigurationEntry<K, C>>,
  ) -> Self {
    Self {
      track_history,
      default_expiration_time,
      get_config_error_message,
      entries,
      entry_count: 0,
      version_count: 0,
      phantom_data: None,
    }
  }

  pub fn update_default_expiration(&mut self, new_default_expiration: Option<u64>) {
    self.default_expiration_time = new_default_expiration;
  }

  pub fn get_entry_count(&self) -> u16 {
    self.entry_count
  }

  pub fn new_entry(&mut self, key: K, entry: C) -> Result<()> {
    self.set_entry(key, entry, true)
  }

  pub fn update_entry(&mut self, key: K, update_inputs: U) -> Result<()> {
    let mut current_entry = self.get_current_entry(key)?;

    current_entry.update(update_inputs)?;
    current_entry.check()?;

    self.set_entry(key, current_entry, false)
  }

  pub fn set_entry_expired(&mut self, version: u64) -> Result<()> {
    if let Some(mut entry) = self.entries.get_mut(&ConfigurationKey::History(version)) {
      entry.expiration_time = Some(Instant::now());
      Ok(())
    } else {
      Err(anyhow!(self.get_config_error_message.clone()))
    }
  }

  pub fn get_current_entry(&self, key: K) -> Result<C> {
    self
      .entries
      .get(&ConfigurationKey::Current(key))
      .map(|e| e.entry.clone())
      .ok_or_else(|| anyhow!(self.get_config_error_message.clone()))
  }

  pub fn get_history_entry(&self, version: u64) -> Result<(C, bool)> {
    let entry = self
      .entries
      .get(&ConfigurationKey::History(version))
      .ok_or_else(|| anyhow!(self.get_config_error_message.clone()))?;

    let is_from_history = entry.expiration_time.map_or(true, |time| time > Instant::now());

    let returned_entry = if is_from_history {
      entry.entry.clone()
    } else {
      self.get_current_entry(entry.key)?
    };

    Ok((returned_entry, is_from_history))
  }

  pub fn get_current_version(&self, key: K) -> Result<u64> {
    self
      .entries
      .get(&ConfigurationKey::Current(key))
      .map(|e| e.version)
      .ok_or_else(|| anyhow!(self.get_config_error_message.clone()))
  }

  fn insert_entry(&mut self, config_key: ConfigurationKey<K>, key: K, entry: C, expiration_time: Option<Instant>) -> Result<()> {
    self.entries.insert(
      config_key,
      ConfigurationEntry {
        key,
        entry,
        version: self.version_count,
        expiration_time,
      },
    );
    Ok(())
  }

  fn set_entry(&mut self, key: K, entry: C, increment_version: bool) -> Result<()> {
    self.insert_entry(ConfigurationKey::Current(key), key, entry.clone(), None)?;

    if self.track_history {
      let expiration_time = self.default_expiration_time.map(|exp| Instant::now().add_seconds(exp as i64).unwrap());
      self.insert_entry(ConfigurationKey::History(self.version_count), key, entry, expiration_time)?;
      self.version_count += 1;
    }

    if increment_version {
      self.entry_count += 1;
    }

    Ok(())
  }
}

/// Trait for updatable items
pub trait Updatable<U> {
  fn update(&mut self, inputs: U) -> Result<()>;
  fn check(&self) -> Result<()>;
}
