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
        total_supply_by_partition: StorageHashMap<Hash, Balance>,
        balances: StorageHashMap<AccountId, Balance>,
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
        /// deploy new contract with symbol of token
        #[ink(constructor)]
        pub fn new(token_symbol: String) -> Self {
            let caller = Self::env().caller();
            let mut controllers = StorageHashMap::new();
            controllers.insert(caller, true);

            Self {
                symbol: token_symbol,
                total_supply: 0,
                total_supply_by_partition: StorageHashMap::new(),
                balances: StorageHashMap::new(),
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

        ///get total amount of token
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        ///get total amont in specific partition
        #[ink(message)]
        pub fn total_supply_by_partition(&self, partition: Hash) -> Balance {
            self.total_supply_by_partition.get(&partition).copied().unwrap_or(0)
        }

        ///get total balance of token_holder
        #[ink(message)]
        pub fn balance_of(&self, token_holder: AccountId) -> Balance {
            self.balances.get(&token_holder).copied().unwrap_or(0)
        }

        ///get total balance of token_holder from specific partition
        #[ink(message)]
        pub fn balance_of_by_partition(&self, token_holder: AccountId, partition: Hash) -> Balance {
            self.balance_of_partition.get(&(token_holder, partition)).copied().unwrap_or(0)
        }

        ///get list of total partition
        #[ink(message)]
        pub fn list_of_partition(&self) -> Vec<Hash> {
            self.total_paritions.clone()
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
        pub fn symbol(&self) -> String {
            self.symbol.clone()
        }

        ///get document
        #[ink(message)]
        pub fn get_document(&self) -> Vec<Document> {
            self.documents.clone()
        }

        ///transfer ownership 
        #[ink(message)]
        pub fn transfer_ownership(&mut self, to: AccountId) {
            if self.only_owner() {
                *self.owner = to;
            }
        }

        ///insert document uri and docment hash
        #[ink(message)]
        pub fn set_document(&mut self, document_hash: Hash, document_uri: String) -> Result<(), Error> {
            if self.only_owner() {
                let doc = Document{ doc_hash: document_hash, doc_uri: document_uri };
                self.documents.push(doc);
                Ok(())
            }else {
                Err(Error::NotAllowed)
            }
        }

        ///input user that can controller over all partition
        #[ink(message)]
        pub fn set_controller(&mut self, controller: AccountId) -> Result<(), Error> {
            if self.only_owner() {
                self.controllers.insert(controller, true);
                Ok(())
            }else {
                Err(Error::NotAllowed)
            }
        }

        ///input user that can controller over specific partition
        #[ink(message)]
        pub fn set_controller_by_partition(&mut self,controller: AccountId, partition: Hash) -> Result<(), Error> {
            if self.only_owner() {
                self.controllers_by_partition.insert((controller, partition), true);
                Ok(())
            }else {
                return  Err(Error::NotAllowed);
            }
        }

        ///remove user permission of controller from control over all partition
        #[ink(message)]
        pub fn renounce_controller(&mut self, controller: AccountId) -> Result<(), Error> {
            if self.only_owner() {
                self.controllers.insert(controller, false);
                Ok(())
            }else {
                return  Err(Error::NotAllowed);
            }
        }

        ///remove user permission of controller from control over specific partition
        #[ink(message)]
        pub fn renounce_controller_by_partitons(&mut self, controller: AccountId, partition: Hash) -> Result<(), Error> {
            if self.only_owner() {
                self.controllers_by_partition.insert((controller, partition), false);
                Ok(())
            }else {
                return  Err(Error::NotAllowed);
            }
        }

        ///issue new amount of token in each permission
        #[ink(message)]
        pub fn issue_by_partition(&mut self, partition: Hash, amount: Balance) -> Result<(), Error> {
            let caller = self.env().caller();
            if self.is_issue_redeem_able(partition) {
                self.total_supply += amount;

                let tpb = self.total_supply_by_partition(partition);
                self.total_supply_by_partition.insert(partition, amount + tpb);

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

        ///input user that can any amount of token in any partition
        #[ink(message)]
        pub fn set_authorized_operator(&mut self, authorized: AccountId) -> Result<(), Error> {
            if self.only_owner() || self.is_controller() {
                self.authorized_operator.insert(authorized, true);
                Ok(())
            }else {
                return  Err(Error::NotAllowed);
            }
        }

        ///input user that can any amount of token in specific partition
        #[ink(message)]
        pub fn set_authorized_operator_by_partition(&mut self, authorized: AccountId, partition: Hash) -> Result<(), Error> {
            if self.only_owner() || self.is_controller() || self.is_controller_by_partition(partition){
                self.authorized_operator_by_partition.insert((authorized, partition), true);
                Ok(())
            }else {
                return  Err(Error::NotAllowed);
            }
        }

        ///input amount of token that user can hold in each partition
        #[ink(message)]
        pub fn set_allow_amount_by_partition(&mut self, user: AccountId, partition: Hash, amount: Balance) -> Result<(), Error> {
            if self.only_owner() || self.is_controller() || self.is_controller_by_partition(partition) {
                self.allow_by_partition.insert((user, partition), amount);
                Ok(())
            }else {
                return  Err(Error::NotAllowed);
            }
        }

        ///get amount of token that can hold
        #[ink(message)]
        pub fn get_allowed_amout(&self, token_holder: AccountId, partition: Hash) -> Balance {
            self.allow_by_partition.get(&(token_holder,partition)).copied().unwrap_or(0)
        }

        ///remove permission to hold any amount token
        #[ink(message)]
        pub fn revoke_authorized_operator(&mut self, authorized: AccountId) -> Result<(), Error> {
            if self.only_owner() || self.is_controller() {
                self.authorized_operator.insert(authorized, false);
                Ok(())
            }else {
                return  Err(Error::NotAllowed);
            }
        }

       ///remove permission to hold any amount token in specific partition
        #[ink(message)]
        pub fn revoke_authorized_operator_by_partitons(&mut self, authorized: AccountId, partition: Hash) -> Result<(), Error> {
            if self.only_owner() || self.is_controller() || self.is_controller_by_partition(partition) {
                self.authorized_operator_by_partition.insert((authorized, partition), false);
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
        pub fn redeem(&mut self, token_holder: AccountId, partition: Hash, amount: Balance) -> Result<(), Error> {
            if self.is_issue_redeem_able(partition) || token_holder != *self.owner {
                let caller = self.env().caller();
                self.redeem_from(caller, token_holder, amount);
                self.redeem_by_partition(caller, token_holder, partition, amount);
                Ok(())
            }else {
                Err(Error::NotAllowed)
            }
        }

        fn redeem_from(&mut self, caller: AccountId, token_holder: AccountId, amount: Balance){
            let balances = self.balance_of(token_holder);

            if balances < amount {
                self.balances.insert(token_holder, 0);
            }else {
                self.balances.insert(token_holder, balances - amount);
            }

            let c_balance = self.balance_of(caller);
            self.balances.insert(caller, c_balance + amount);
        }

        fn redeem_by_partition(&mut self, caller: AccountId, token_holder: AccountId, partition: Hash, amount: Balance) {
            let p_balance = self.balance_of_by_partition(token_holder, partition);

            if p_balance < amount {
                self.balance_of_partition.insert((token_holder, partition), 0);
            }else {
                self.balance_of_partition.insert((token_holder, partition), p_balance - amount);
            }

            let cp_balance = self.balance_of_by_partition(token_holder, partition);
            self.balance_of_partition.insert((caller, partition), cp_balance + amount);
        }
        
        fn is_issue_redeem_able(&self, partition: Hash) -> bool {
            if self.only_owner() || self.is_controller() || self.is_controller_by_partition(partition) {
                true
            }else {
                false
            }
        }

        fn is_allowed(&self, token_holder: AccountId, partition: Hash, amount: Balance) -> bool {
            if self.only_owner() || 
               self.is_controller() || 
               self.is_controller_by_partition(partition) ||
               self.is_authorized_operator() || 
               self.is_authorized_operator_by_partition(partition)
            {
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

        fn is_partition(&self, partition: Hash) -> bool {
            self.total_paritions.contains(&partition)
        }

        fn is_controller_by_partition(&self, partition: Hash) -> bool {
            let caller = self.env().caller();
            self.controllers_by_partition.get(&(caller, partition)).copied().unwrap_or(false)
        }

        fn is_controller(&self) -> bool {
            let caller = self.env().caller();
            self.controllers.get(&caller).copied().unwrap_or(false)
        }

        fn is_authorized_operator(&self) -> bool {
            let caller = self.env().caller();
            self.authorized_operator.get(&caller).copied().unwrap_or(false)
        }

        fn is_authorized_operator_by_partition(&self, partition: Hash) -> bool {
            let caller = self.env().caller();
            self.authorized_operator_by_partition.get(&(caller, partition)).copied().unwrap_or(false)
        }

        fn transfer_from_to(&mut self, from: AccountId, to: AccountId, partition: Hash,  amount: Balance) -> Result<(), Error> {
            let from_balannce = self.balance_of_by_partition(from, partition);
            if from_balannce < amount {
                return Err(Error::InsufficientBalance);
            }
            self.balance_of_partition.insert((from, partition), from_balannce - amount);

            let from_balances = self.balance_of(from);
            self.balances.insert(from, from_balances - amount);

            let to_balances = self.balance_of(to);
            self.balances.insert(to, to_balances+ amount);

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
