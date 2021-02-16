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
        symbol: Vec<String>,
        list_paritions: Vec<Hash>,
        partition_symbol: StorageHashMap<String, Hash>,
        total_supply: Balance,
        total_supply_by_partition: StorageHashMap<Hash, Balance>,
        documents: StorageHashMap<Hash, Vec<Document>>,
        partitions_of: StorageHashMap<AccountId, Vec<Hash>>,
        balance_of_partition: StorageHashMap<(AccountId, Hash), Balance >,
        owner: Lazy<AccountId>,
        allow_by_partition: StorageHashMap<(AccountId, Hash), Balance >,
        controllers: StorageHashMap<(AccountId, Hash), bool>,
        is_issuable: StorageHashMap<Hash, bool>,
    }

    impl Erc1400 {
        /// deploy new contract with symbol of token
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();

            Self {
                symbol: Vec::new(),
                partition_symbol: StorageHashMap::new(),
                total_supply: 0,
                total_supply_by_partition: StorageHashMap::new(),
                documents: StorageHashMap::new(),
                list_paritions: Vec::new(),
                partitions_of: StorageHashMap::new(),
                balance_of_partition: StorageHashMap::new(),
                owner: Lazy::new(caller),
                controllers: StorageHashMap::new(),
                allow_by_partition: StorageHashMap::new(),
                is_issuable: StorageHashMap::new(),
            }
        }

        ///transfer ownership 
        #[ink(message)]
        pub fn transfer_ownership(&mut self, to: AccountId) {
            if self.only_owner() {
                *self.owner = to;
            }
        }

        ///get total token amount of all partition 
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        ///get total token amont in specific partition
        #[ink(message)]
        pub fn total_supply_by_partition(&self, partition: Hash) -> Balance {
            self.total_supply_by_partition.get(&partition).copied().unwrap_or(0)
        }

        ///get total balance of token_holder from specific partition
        #[ink(message)]
        pub fn balance_of_by_partition(&self, token_holder: AccountId, partition: Hash) -> Balance {
            self.balance_of_partition.get(&(token_holder, partition)).copied().unwrap_or(0)
        }

        ///get list of total partition of each token_holder
        #[ink(message)]
        pub fn partion_of_token_holder(&self, token_holder: AccountId) -> Vec<Hash>{
            match self.partitions_of.get(&token_holder) {
                None => Vec::new(),
                Some(p) => {
                    p.clone()
                }
            }
        }

        ///get symbol of token
        #[ink(message)]
        pub fn symbol(&self) -> Vec<String> {
            self.symbol.clone()
        }

        #[ink(message)]
        pub fn get_hash_by_symbol(&self, symbol: String) -> Hash {
            let hash = Hash::from([0; 32]);
            self.partition_symbol.get(&symbol).copied().unwrap_or(hash)
        }

        ///get document
        #[ink(message)]
        pub fn get_document(&self, partition: Hash) -> Vec<Document> {
            match self.documents.get(&partition) {
                None => Vec::new(),
                Some(d) => {
                    d.clone()
                }
            }
        }

        ///get amount of token that can hold
        #[ink(message)]
        pub fn get_allowed_amout(&self, token_holder: AccountId, partition: Hash) -> Balance {
            self.allow_by_partition.get(&(token_holder,partition)).copied().unwrap_or(0)
        }

        ///insert document uri and docment hash
        #[ink(message)]
        pub fn set_document(&mut self, partition: Hash, document_hash: Hash, document_uri: String) -> Result<(), Error> {
            if self.is_controller_by_partition(partition) {
                let mut all_doc = self.get_document(partition);
                let doc = Document{ doc_hash: document_hash, doc_uri: document_uri };
                all_doc.push(doc);
                self.documents.insert(partition, all_doc);
                Ok(())
            }else {
                Err(Error::NotAllowed)
            }
        }

        ///input user that can controller over specific partition
        #[ink(message)]
        pub fn set_controller_by_partition(&mut self,controller: AccountId, symbol: String, partition: Hash) -> Result<(), Error> {
            if self.only_owner() {
                if self.is_exit_partition(partition) == false {
                    self.controllers.insert((controller, partition), true);
                    self.list_paritions.push(partition);
                    self.is_issuable.insert(partition, true);
                    self.symbol.push(symbol.clone());
                    self.partition_symbol.insert(symbol.clone(), partition);
                    Ok(())
                }else {
                    return Err(Error::NotAllowed);
                }
            }else {
                return Err(Error::NotAllowed);
            }
        }

        ///issue new amount of token in each partition
        #[ink(message)]
        pub fn issue_by_partition(&mut self, partition: Hash, amount: Balance) -> Result<(), Error> {
            let caller = self.env().caller();
            if self.is_controller_by_partition(partition) && self.is_issuable(partition) {
                self.total_supply += amount;
                self.total_supply_by_partition.insert(partition, amount);

                self.balance_of_partition.insert((caller, partition), amount);

                let mut own_partition = self.partion_of_token_holder(caller);
                own_partition.push(partition);
                self.partitions_of.insert(caller, own_partition);
                self.is_issuable.insert(partition, false);
                Ok(())
            }else {
                Err(Error::NotAllowed)
            }
        }

        ///input amount of token that user can hold in each partition
        #[ink(message)]
        pub fn set_allow_amount_by_partition(&mut self, user: AccountId, partition: Hash, amount: Balance) -> Result<(), Error> {
            if self.is_controller_by_partition(partition) {
                self.allow_by_partition.insert((user, partition), amount);
                Ok(())
            }else {
                return  Err(Error::NotAllowed);
            }
        }

        ///transfer token to any token_holder that allow to hold
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, partition: Hash, amount: Balance) -> Result<(), Error>{
            let caller = self.env().caller();
            if self.is_allowed(to, partition, amount) {
                self.transfer_from_to(caller, to,partition, amount)?;
                Ok(())
            }else {
                Err(Error::NotAllowed)
            }
        }

        ///redeem token from token_holder
        #[ink(message)]
        pub fn redeem_from(&mut self, token_holder: AccountId, partition: Hash, amount: Balance) -> Result<(), Error> {
            if self.is_controller_by_partition(partition) {
                let caller = self.env().caller();
                self.redeem(caller, token_holder, amount, partition);
                Ok(())
            }else {
                Err(Error::NotAllowed)
            }
        }

        fn redeem(&mut self, caller: AccountId, token_holder: AccountId, amount: Balance, partition: Hash){

            let balance = self.balance_of_by_partition(token_holder, partition);

            if balance < amount {
                self.balance_of_partition.insert((token_holder, partition), 0);
            }else {
                self.balance_of_partition.insert((token_holder, partition), balance - amount);
            }

            let balance = self.balance_of_by_partition(caller, partition);
            self.balance_of_partition.insert((caller, partition), balance + amount);
        }

        fn is_allowed(&self, token_holder: AccountId, partition: Hash, amount: Balance) -> bool {
            if self.is_controller_by_partition(partition) {
                true
            }else {
                let alow_balance = self.get_allowed_amout(token_holder, partition);
                if alow_balance < amount {
                    false
                }else {
                    true
                }
            }
            
        }

        fn is_issuable(&self, partition: Hash) -> bool {
            self.is_issuable.get(&partition).copied().unwrap_or(false)
        }

        fn is_exit_partition(&self, partition: Hash) -> bool {
            self.list_paritions.contains(&partition)
        }

        fn is_controller_by_partition(&self, partition: Hash) -> bool {
            let caller = self.env().caller();
            self.controllers.get(&(caller, partition)).copied().unwrap_or(false)
        }

        fn transfer_from_to(&mut self, from: AccountId, to: AccountId, partition: Hash,  amount: Balance) -> Result<(), Error> {
            let from_balannce = self.balance_of_by_partition(from, partition);
            if from_balannce < amount {
                return Err(Error::InsufficientBalance);
            }
            self.balance_of_partition.insert((from, partition), from_balannce - amount);

            let to_balannce = self.balance_of_by_partition(to, partition);
            self.balance_of_partition.insert((to, partition), to_balannce + amount);

            Ok(())
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
