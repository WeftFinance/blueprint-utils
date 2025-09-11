use scrypto::prelude::*;

pub trait CanBeChecked {
  fn is_a_rate(&self) -> bool;

  fn is_zero(&self) -> bool;

  fn is_positive(&self) -> bool;

  fn is_valid_percentage(&self) -> bool {
    self.is_a_rate()
  }

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
