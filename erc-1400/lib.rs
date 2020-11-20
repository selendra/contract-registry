#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc_1400 {
    use ink_storage::{collections::HashMap as StorageHashMap, lazy::Lazy};

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        NotPermission,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct Erc1400 {
        owner: Lazy<AccountId>,
        total_supply: Lazy<Balance>,
        partitions: StorageHashMap<AccountId, bool>,
        balances: StorageHashMap<AccountId, Balance>,
    }

    impl Erc1400 {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let owner = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(owner, initial_supply);
            Self {
                owner: Lazy::new(owner),
                total_supply: Lazy::new(initial_supply),
                partitions: StorageHashMap::new(),
                balances,
            }
        }

        #[ink(message)]
        pub fn add_partition(&mut self, parititon: AccountId) {
            let caller = self.env().caller();
            if caller == *self.owner {
                self.partitions.insert(parititon, true);
            }
        }

        #[ink(message)]
        pub fn revoke_partition(&mut self, parititon: AccountId) {
            let caller = self.env().caller();
            if caller == *self.owner {
                self.partitions.insert(parititon, false);
            }
        }

        #[ink(message)]
        pub fn is_partition(&self, partition: AccountId) -> bool {
            match self.is_permission(partition) {
                Ok(_) => true,
                Err(_) => false,
            }
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            self.is_permission(caller)?;
            self.is_permission(to)?;
            self.transfer_from_to(caller, to, value)
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of_or_zero(&from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(from, from_balance - value);

            let to_balance = self.balance_of_or_zero(&to);
            self.balances.insert(to, to_balance + value);

            Ok(())
        }

        #[ink(message)]
        pub fn transfer_ownership(&mut self, to: AccountId) -> Result<()> {
            let caller = Self::env().caller();
            if caller == *self.owner {
                *self.owner = to;

                Ok(())
            } else {
                Err(Error::NotPermission)
            }
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        fn partition_or_not(&self, partition: &AccountId) -> bool {
            *self.partitions.get(partition).unwrap_or(&false)
        }

        fn is_permission(&self, partition: AccountId) -> Result<()> {
            if partition == *self.owner {
                Ok(())
            } else if self.partition_or_not(&partition) {
                Ok(())
            } else {
                Err(Error::NotPermission)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn new_work() {
            let contract = Erc1400::new(100);
            assert_eq!(AccountId::from([0x1; 32]), *contract.owner);
        }

        #[ink::test]
        fn add_partition_work() {
            let mut contract = Erc1400::new(100);
            contract.add_partition(AccountId::from([0x0; 32]));
            assert_eq!(contract.is_partition(AccountId::from([0x0; 32])), true);
            assert_eq!(contract.is_partition(AccountId::from([0x1; 32])), true)
        }

        #[ink::test]
        fn revoke_partition_work() {
            let mut contract = Erc1400::new(100);
            contract.add_partition(AccountId::from([0x0; 32]));
            assert_eq!(contract.is_partition(AccountId::from([0x0; 32])), true);
            contract.revoke_partition(AccountId::from([0x0; 32]));
            assert_eq!(contract.is_partition(AccountId::from([0x0; 32])), false)
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = Erc1400::new(100);
            contract.add_partition(AccountId::from([0x0; 32]));
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.transfer(AccountId::from([0x0; 32]), 10), Ok(()));
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
            assert_ne!(contract.transfer(AccountId::from([0x0; 32]), 100), Ok(()));
        }

        #[ink::test]
        fn transfer_ownership_works() {
            let mut contract = Erc1400::new(777);
            assert!(contract
                .transfer_ownership(AccountId::from([0x0; 32]))
                .is_ok())
        }
    }
}
