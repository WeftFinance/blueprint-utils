use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AuthBadgeData {}

#[derive(ScryptoSbor, Clone)]
pub struct PriceInfo {
  pub timestamp: i64,
  pub price: Decimal,
}

#[blueprint]
#[types(AuthBadgeData)]
mod resource_faucet {

  enable_method_auth! {
    roles {
      admin => updatable_by: [];
    },
    methods {
      deposit => restrict_to: [admin];
      withdraw => restrict_to: [admin];
      withdraw_xrd => restrict_to: [admin];
      get => PUBLIC;
      swap => PUBLIC;
    }
  }

  struct ResourceFaucet {
    admin_rule: AccessRule,
    collected_xrd: Vault,
    price_feed: Global<AnyComponent>,
    resources: KeyValueStore<ResourceAddress, Vault>,
  }
  impl ResourceFaucet {
    pub fn instantiate(price_feed: Global<AnyComponent>) -> NonFungibleBucket {
      let (admin_badge_address_reservation, admin_badge_address) = Runtime::allocate_non_fungible_address();

      let admin_rule = rule!(require(admin_badge_address));

      let admin_badge = ResourceBuilder::new_integer_non_fungible_with_registered_type::<AuthBadgeData>(OwnerRole::Fixed(admin_rule.clone()))
        .with_address(admin_badge_address_reservation)
        .metadata(metadata! {
          init{
            "name" => "Resource faucet Admin Badge", updatable;
          }
        })
        .mint_initial_supply([(IntegerNonFungibleLocalId::from(0), AuthBadgeData {})]);

      Self {
        admin_rule: admin_rule.clone(),
        price_feed,
        collected_xrd: Vault::new(XRD),
        resources: KeyValueStore::new(),
      }
      .instantiate()
      .prepare_to_globalize(OwnerRole::Fixed(admin_rule))
      .roles(roles! {
              admin => rule!(require(admin_badge.resource_address()));
      })
      .globalize();

      admin_badge
    }

    /// FAUCET : CREATE AND SUPPLY RESOURCES FOR TEST

    pub fn deposit(&mut self, resources: Bucket) {
      let res_address = resources.resource_address();

      if self.resources.get(&res_address).is_none() {
        self.resources.insert(res_address, Vault::with_bucket(resources));
      } else {
        self.resources.get_mut(&res_address).unwrap().put(resources);
      }
    }

    pub fn withdraw(&mut self, resource: ResourceAddress, amount: Option<Decimal>) -> Bucket {
      let mut vault = self.resources.get_mut(&resource).expect("Resource not found");

      if let Some(amount) = amount {
        vault.take_advanced(amount, WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointTowardZero))
      } else {
        vault.take_all()
      }
    }

    pub fn withdraw_xrd(&mut self) -> Bucket {
      self.collected_xrd.take_all()
    }

    pub fn get(&mut self, resource: ResourceAddress, xrd: Bucket) -> Bucket {
      assert!(xrd.resource_address() == XRD, "Provide XRD to get faucet tokens");

      let from_price = self.get_price(XRD);

      let to_price = self.get_price(resource);

      assert!(from_price.price > Decimal::ZERO, "XRD price is zero");

      let to_amount = xrd.amount() * (from_price.price / to_price.price);

      self.collected_xrd.put(xrd);

      self
        .resources
        .get_mut(&resource)
        .expect("Resource not found")
        .take_advanced(to_amount, WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointTowardZero))
    }

    /// TEST EXCHANGE

    pub fn swap(&mut self, from_bucket: Bucket, to_resource: ResourceAddress) -> Bucket {
      let from_resource = from_bucket.resource_address();

      let from_price = self.get_price(from_resource);
      let to_price = self.get_price(to_resource);

      assert!(from_price.price > Decimal::ZERO, "XRD price is zero");

      let to_amount = from_bucket.amount() * (from_price.price / to_price.price);

      if self.resources.get(&from_resource).is_none() {
        self.resources.insert(from_resource, Vault::with_bucket(from_bucket));
      } else {
        self.resources.get_mut(&from_resource).expect("Resource not found").put(from_bucket);
      }

      self
        .resources
        .get_mut(&to_resource)
        .expect("Resource not found")
        .take_advanced(to_amount, WithdrawStrategy::Rounded(RoundingMode::ToNearestMidpointTowardZero))
    }

    /// Local methods

    fn get_price(&mut self, resource: ResourceAddress) -> PriceInfo {
      self
        .price_feed
        .call_raw::<Option<PriceInfo>>("get_price", scrypto_args!(resource))
        .expect("Price not found")
    }
  }
}
