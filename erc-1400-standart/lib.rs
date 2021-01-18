#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;
pub mod models;

#[ink::contract]
mod erc1400 {
    use super::*;
    use models::{ doc::*, error::*};

    use ink_prelude::{string::String, vec::Vec};
    use ink_storage::{collections::HashMap as StorageHashMap, Lazy };

    #[ink(storage)]
    pub struct Erc1400 {
        symbol: String,
        total_supply: Balance,
        balances: StorageHashMap<AccountId, Balance>,
        allow: StorageHashMap<AccountId, Balance>,
        documents: Vec<Document>,
        total_paritions: Vec<Hash>,
        partitions_of: StorageHashMap<AccountId, Vec<Hash>>,
        balance_of_partition: StorageHashMap<(AccountId, Hash), Balance >,
        owner: Lazy<AccountId>,
        authorized_operator: StorageHashMap<AccountId, bool>,
        controllers: StorageHashMap<AccountId, bool>,
        allow_by_partition: StorageHashMap<(AccountId, Hash), Balance >,
        authorized_operator_by_partition: StorageHashMap<(AccountId, Hash), bool>,
        controllers_by_partition: StorageHashMap<(AccountId, Hash), bool>
    }

    impl Erc1400 {
        #[ink(constructor)]
        pub fn new(token_symbol: String) -> Self {
            let caller = Self::env().caller();
            let mut controllers = StorageHashMap::new();
            controllers.insert(caller, true);

            Self { 
                symbol: token_symbol,
                total_supply: 0,
                balances: StorageHashMap::new(),
                allow: StorageHashMap::new(),
                documents: Vec::new(),
                total_paritions: Vec::new(),
                partitions_of: StorageHashMap::new(),
                balance_of_partition: StorageHashMap::new(),
                owner: Lazy::new(caller),
                authorized_operator: StorageHashMap::new(),
                controllers,
                allow_by_partition: StorageHashMap::new(),
                authorized_operator_by_partition: StorageHashMap::new(),
                controllers_by_partition: StorageHashMap::new()
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, token_holder: AccountId) -> Balance {
            self.balances.get(&token_holder).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn balance_of_by_partition(&self, token_holder: AccountId, partition: Hash) -> Balance {
            self.balance_of_partition.get(&(token_holder, partition)).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn list_of_partition(&self) -> Vec<Hash> {
            self.total_paritions.clone()
        }

        #[ink(message)]
        pub fn partion_of_token_holder(&self, token_holder: AccountId) -> Vec<Hash>{
            match self.partitions_of.get(&token_holder) {
                None => Vec::new(),
                Some(p) => {
                    p.clone()
                }
            }
        }

        #[ink(message)]
        pub fn symbol(&self) -> String {
            self.symbol.clone()
        }

        #[ink(message)]
        pub fn issue_by_partition(&mut self, partition: Hash, amount: Balance) -> Result<(), Error> {
            let caller = self.env().caller();
            if self.is_controllable() {
                self.total_supply += amount;

                let balance = self.balance_of(caller);
                self.balances.insert(caller, balance + amount);

                if self.is_partition(partition) == false {
                    self.total_paritions.push(partition);
                    let mut own_partition = self.partion_of_token_holder(caller);
                    own_partition.push(partition);
                    self.partitions_of.insert(caller, own_partition);
                };
                let p_balance = self.balance_of_partition.get(&(caller, partition)).copied().unwrap_or(0);
                self.balance_of_partition.insert((caller, partition), amount + p_balance);
                Ok(())
            }else {
                Err(Error::NotAllowed)
            }
        }

        #[ink(message)]
        pub fn set_controller(&mut self, controller: AccountId) -> Result<(), Error> {
            if self.only_owner() {
                self.controllers.insert(controller, true);
                Ok(())
            }else {
                return  Err(Error::NotAllowed);
            }
        }

        fn is_partition(&self, partition: Hash) -> bool {
            self.total_paritions.contains(&partition)
        }

        fn is_controllable(&self) -> bool {
            let caller = self.env().caller();
            self.controllers.get(&caller).copied().unwrap_or(false)
        }

        pub fn only_owner(&self) -> bool {
            let caller = self.env().caller();
            if caller == *self.owner {
                true
            } else {
                false
            }
        }
    }
}
