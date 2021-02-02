#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
pub mod error;

#[ink::contract]
mod token_lock {
    use super::*;
    use error::*;

    use ink_storage::{ collections::HashMap as StorageHashMap, Lazy };

    #[ink(storage)]
    pub struct TokenLock {
        lst_account: StorageHashMap<AccountId, bool>,
        amount: Lazy<Balance>,
        release_date: Lazy<u64>,
        owner: Lazy<AccountId>
    }

    impl TokenLock {
        /// amount: of token that each account will get
        /// lock_time: time of lock perior (minute)
        #[ink(constructor)]
        pub fn new(amount: Balance, lock_time: u64) -> Self {
            let release_date = Self::env().block_timestamp();
            let minute = lock_time * 60000;
            let caller = Self::env().caller();
            Self { 
                lst_account: StorageHashMap::new(),
                release_date: Lazy::new(release_date + minute),
                amount: Lazy::new(amount),
                owner: Lazy::new(caller),
            }
        }

        /// get token afer release date
        #[ink(message)]
        pub fn get_token(&mut self) -> Result<(), Error> {
            let now = self.env().block_timestamp();
            let caller = self.env().caller();

            if now < *self.release_date {
                return Err(Error::IsUnlockPerior)
            }else if self.allow(caller) == false {
                return  Err(Error::NoPermissionAllow);
            }else if *self.amount > self.env().balance() {
                return Err(Error::InsufficientFunds)
            };
            self.env()
                .transfer(caller, *self.amount)
                .map_err(|err| {
                    match err {
                        ink_env::Error::BelowSubsistenceThreshold => {
                            Error::BelowSubsistenceThreshold
                        }
                        _ => Error::TransferFailed,
                }
            })
        }
        
        /// add account that can get amount by owner
        #[ink(message)]
        pub fn add_allow_account(&mut self, account: AccountId)-> Result<(), Error>{
            let caller = self.env().caller();
            if *self.owner == caller {
                if *self.amount > self.env().balance() {
                    return Err(Error::InsufficientFunds)
                };
                self.lst_account.insert(account, true);
                Ok(())
            }else {
                Err(Error::NoPermissionAllow)
            }
        }

        /// user to change owner
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) {
            let caller = self.env().caller();
            if *self.owner == caller {
                *self.owner = new_owner;
            }
        }

        /// check time before release
        #[ink(message)]
        pub fn time_count_down(&self) -> u64 {
            let now = self.env().block_timestamp();
            let mut time = 0;
            if now < *self.release_date {
               time = *self.release_date - now;
            }
            time
        }

        /// check you allow to get token or not
        #[ink(message)]
        pub fn allow(&self, account: AccountId) -> bool{
            self.lst_account.get(&account).copied().unwrap_or(false)
        }
    }
}
