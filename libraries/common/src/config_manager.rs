use anyhow::{anyhow, ensure, Result};
use scrypto::prelude::rust::hash::Hash;
use scrypto::prelude::*;
use utils::{define_inner_error, InstantUtils};

define_inner_error! {
  KEY_AND_VERSION_MISMATCH,
  CONFIG_ENTRY_NOT_FOUND,
  CONFIG_VERSION_NOT_FOUND,
}

#[derive(ScryptoSbor, ManifestSbor, Clone)]
pub enum SetExpirationInput {
  None,
  Now,
  Instant(Instant),
}

#[derive(ScryptoSbor, Clone, PartialEq, Eq, Hash)]
pub enum ConfigurationKey<K: ScryptoSbor + Debug + Hash + Copy> {
  Current(K),
  History(u64),
}

#[derive(ScryptoSbor, Clone)]
pub struct ConfigurationEntry<K: ScryptoSbor + Debug + Hash + Copy, C: ScryptoSbor> {
  key: K,
  entry: C,
  version: u64,
  expiration_time: Option<Instant>,
}

/// Trait for updatable items
pub trait Updatable<U> {
  fn update(&mut self, inputs: U) -> Result<()>;
  fn check(&self) -> Result<()>;
}

#[derive(ScryptoSbor)]
pub struct ConfigurationManager<K, C, U>
where
  K: ScryptoSbor + Hash + Copy + Debug,
  C: ScryptoSbor + Clone + Updatable<U>,
  U: ScryptoSbor,
{
  // Config
  track_history: bool,
  default_expiration_time: Option<u64>,
  // State
  version_count: u64,
  entry_count: u16,
  entries: KeyValueStore<ConfigurationKey<K>, ConfigurationEntry<K, C>>,
  phantom_data: Option<U>,
}

impl<K, C, U> ConfigurationManager<K, C, U>
where
  K: ScryptoSbor + Debug + Hash + Copy + PartialEq + Eq,
  C: ScryptoSbor + Clone + Updatable<U>,
  U: ScryptoSbor,
{
  pub fn new(track_history: bool, entries: KeyValueStore<ConfigurationKey<K>, ConfigurationEntry<K, C>>) -> Self {
    Self {
      track_history,
      default_expiration_time: None,
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

  pub fn set_entry(&mut self, key: K, entry: C) -> Result<()> {
    self.set_entry_internal(key, entry)
  }

  pub fn update_entry(&mut self, key: K, update_inputs: U) -> Result<()> {
    let mut current_entry = self.get_current_entry(key)?;

    current_entry.update(update_inputs)?;

    self.set_entry_internal(key, current_entry)
  }

  pub fn set_entry_expiration(&mut self, key: K, version: u64, expiration_time: SetExpirationInput) -> Result<()> {
    match self.entries.get_mut(&ConfigurationKey::History(version)) {
      Some(mut entry) => {
        ensure!(entry.key == key, "{}|{:?} != {:?}", KEY_AND_VERSION_MISMATCH, key, entry.key);

        match expiration_time {
          SetExpirationInput::None => entry.expiration_time = None,
          SetExpirationInput::Now => entry.expiration_time = Some(Instant::now()),
          SetExpirationInput::Instant(expiration_time) => entry.expiration_time = Some(expiration_time),
        };

        Ok(())
      }
      None => Err(anyhow!("{}|{}", CONFIG_VERSION_NOT_FOUND, version)),
    }
  }

  pub fn get_current_version(&self, key: K) -> Result<u64> {
    self
      .entries
      .get(&ConfigurationKey::Current(key))
      .map(|e| e.version)
      .ok_or(anyhow!(CONFIG_ENTRY_NOT_FOUND))
  }

  pub fn get_current_entry(&self, key: K) -> Result<C> {
    self
      .entries
      .get(&ConfigurationKey::Current(key))
      .map(|e| e.entry.clone())
      .ok_or(anyhow!(CONFIG_ENTRY_NOT_FOUND))
  }

  pub fn get_history_entry(&self, key: K, version: u64) -> Result<(C, bool)> {
    let entry = self
      .entries
      .get(&ConfigurationKey::History(version))
      .ok_or(anyhow!(CONFIG_VERSION_NOT_FOUND))?;

    ensure!(entry.key == key, "{}|{:?} != {:?}", KEY_AND_VERSION_MISMATCH, key, entry.key);

    let is_from_history = entry.expiration_time.map_or(true, |time| time > Instant::now());

    let returned_entry = if is_from_history {
      entry.entry.clone()
    } else {
      self.get_current_entry(entry.key)?
    };

    Ok((returned_entry, is_from_history))
  }

  // ! LOCAL METHODS

  fn set_entry_internal(&mut self, key: K, entry: C) -> Result<()> {
    entry.check()?;

    if self.entries.get(&ConfigurationKey::Current(key)).is_none() {
      self.entry_count = self.entry_count.saturating_add(1);
    }

    let current_entry = ConfigurationEntry {
      key,
      entry: entry.clone(),
      version: self.version_count,
      expiration_time: None,
    };

    self.entries.insert(ConfigurationKey::Current(key), current_entry);

    if self.track_history {
      let expiration_time = self.default_expiration_time.map(|exp| Instant::now().add_seconds(exp as i64).unwrap());

      let history_entry = ConfigurationEntry {
        key,
        entry,
        version: self.version_count,
        expiration_time,
      };

      self.entries.insert(ConfigurationKey::History(self.version_count), history_entry);
      self.version_count = self.version_count.checked_add(1).unwrap();
    }

    Ok(())
  }
}
