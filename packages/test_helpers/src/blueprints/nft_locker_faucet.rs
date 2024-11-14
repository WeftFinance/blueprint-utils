use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct LockerNftData {
  data: IndexMap<ResourceAddress, Decimal>,
}

#[blueprint]
mod nft_locker_faucet {

  use indexmap::IndexMap;

  enable_method_auth! {
          methods {
                 create => restrict_to: [OWNER];
                 mint  => PUBLIC;
                 get_resource_amounts => PUBLIC;
                 get_eligible_resources_by_nft => PUBLIC;
          }
  }
  struct NftLockerFaucet {
    nfts: IndexMap<ResourceAddress, Vec<ResourceAddress>>,
    nfts_resource_lookup: IndexMap<Vec<ResourceAddress>, ResourceAddress>,
    nft_resources_manager: IndexMap<ResourceAddress, ResourceManager>,
    admin_role: OwnerRole,
  }

  impl NftLockerFaucet {
    pub fn instantiate(admin_role: OwnerRole) -> Global<NftLockerFaucet> {
      Self {
        admin_role: admin_role.clone(),
        nfts: IndexMap::new(),
        nfts_resource_lookup: IndexMap::new(),
        nft_resources_manager: IndexMap::new(),
      }
      .instantiate()
      .prepare_to_globalize(admin_role)
      .globalize()
    }

    pub fn create(&mut self, mut resources: IndexSet<ResourceAddress>) -> ResourceAddress {
      resources.sort();
      let resources_vec: Vec<ResourceAddress> = resources.into_iter().collect();
      assert!(!resources_vec.is_empty(), "At least 1 resource is required");
      assert!(!self.nfts_resource_lookup.contains_key(&resources_vec), "NFT already create");
      let resource_manager = ResourceBuilder::new_ruid_non_fungible::<LockerNftData>(self.admin_role.clone())
        .mint_roles(mint_roles! {
          minter => rule!(allow_all);
          minter_updater => rule!(deny_all);
        })
        .create_with_no_initial_supply();

      let nft_resource_address = resource_manager.address();

      self.nft_resources_manager.insert(nft_resource_address, resource_manager);

      self.nfts.insert(nft_resource_address, resources_vec.clone());

      self.nfts_resource_lookup.insert(resources_vec, nft_resource_address);

      nft_resource_address
    }

    pub fn mint(&mut self, nft_resource_address: ResourceAddress, nft_resource_amount: IndexMap<ResourceAddress, Decimal>) -> NonFungibleBucket {
      assert!(self.nfts.contains_key(&nft_resource_address), "Nft resource_address not found");

      let resources: Vec<ResourceAddress> = self.nfts.get(&nft_resource_address).unwrap().to_vec();

      nft_resource_amount.iter().for_each(|f| {
        if !resources.contains(f.0) {
          panic!("This resource is not Eligible for this NFT")
        }
      });

      let resource_manager = self.nft_resources_manager.get_mut(&nft_resource_address).unwrap();

      resource_manager
        .mint_ruid_non_fungible(LockerNftData { data: nft_resource_amount })
        .as_non_fungible()
    }

    pub fn get_resource_amounts(
      &self,
      resource_address: ResourceAddress,
      nft_ids: IndexSet<NonFungibleLocalId>,
    ) -> IndexMap<NonFungibleLocalId, IndexMap<ResourceAddress, Decimal>> {
      let locker_res_manager = self.nft_resources_manager.get(&resource_address).unwrap();

      let res_amount = nft_ids.iter().fold(IndexMap::new(), |mut res_amount, id| {
        let res_amount_entry: &mut IndexMap<ResourceAddress, Decimal> = res_amount.entry(id.clone()).or_default();

        let data: LockerNftData = locker_res_manager.get_non_fungible_data(id);

        for (res, amount) in data.data {
          let entry = res_amount_entry.entry(res).or_insert(Decimal::ZERO);

          *entry += amount;
        }

        res_amount
      });

      res_amount
    }

    pub fn get_eligible_resources_by_nft(&self, resource_address: ResourceAddress) -> Vec<ResourceAddress> {
      match self.nfts.get(&resource_address) {
        Some(resources) => resources.to_vec(),
        None => Vec::new(),
      }
    }
  }
}
