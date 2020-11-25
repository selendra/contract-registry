#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod distribute_sel {
    use ink_storage::{collections::HashMap as StorageHashMap, lazy::Lazy};


    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        TransferFailed,
        InsufficientFunds,
        OnlyOwner,
    }

    pub type Result<T> = core::result::Result<T, Error>;
    
    #[ink(storage)]
    pub struct DistributeSel {
        owner: Lazy<AccountId>,
        balances: StorageHashMap<AccountId, Balance>
    }

    impl DistributeSel {
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            Self { 
                owner: Lazy::new(caller), 
                balances: StorageHashMap::new()
            }
        }

        #[ink(message)]
        pub fn distribute(&mut self, account: AccountId, value: Balance) -> Result<()> {
            self.only_owner(self.env().caller())?;
            let balance = self.balance_of_or_zero(&account);
            self.balances.insert(account, value + balance);

            Ok(())
        }

        #[ink(message)]
        pub fn get_balance(&mut self, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            let balance = self.balance_of_or_zero(&caller);

            if balance < value {
                Err(Error::InsufficientFunds)
            }else{
                self.transfer_balance(caller, value)?;
                self.balances.insert(caller, balance - value);
                Ok(())
            }
            
        }

        fn transfer_balance(&mut self, to: AccountId, value: Balance) -> Result<()> {
            match self.env().transfer(to, value) {
                Ok(_) => Ok(()),
                Err(_) => Err(Error::TransferFailed)
            }    
        }
        
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        #[ink(message)]
        pub fn transfer_ownership(&mut self, to: AccountId) -> Result<()> {
            let caller = Self::env().caller();
            self.only_owner(caller)?;
            *self.owner = to;
            Ok(())
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        fn only_owner(&self, caller: AccountId) -> Result<()> {
            if *self.owner == caller {
                Ok(())
            } else {
                return Err(Error::OnlyOwner);
            }
        }
    }
}