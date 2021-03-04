#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod attendant_token {

    use ink_prelude::{string::String, vec, vec::Vec};
    use ink_storage::collections::HashMap as StorageHashMap;
    use ink_storage::traits::{PackedLayout, SpreadLayout};

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidHash,
        CheckOutFail,
        CheckInFail,
        ChangeFail,
        PermissionDeny,
    }

    #[derive(Debug, scale::Encode, PackedLayout, scale::Decode, Clone, SpreadLayout)]
    #[cfg_attr(
        feature = "std",
        derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout)
    )]
    pub struct Attendant {
        time: u64,
        location: String,
    }

    #[ink(storage)]
    pub struct AttendantToken {
        token: StorageHashMap<AccountId, Balance>,
        attendant_hash: Hash,
        check_in: StorageHashMap<AccountId, Vec<Attendant>>,
        check_out: StorageHashMap<AccountId, Vec<Attendant>>,
        check_in_status: StorageHashMap<AccountId, bool>,
        check_in_time: StorageHashMap<AccountId, u64>,
        change_able: bool,
        owner: AccountId,
    }

    impl AttendantToken {
        #[ink(constructor)]
        pub fn new() -> Self {
            let hash = Hash::from([1; 32]);
            let caller = Self::env().caller();
            Self {
                token: StorageHashMap::new(),
                attendant_hash: hash,
                check_in: StorageHashMap::new(),
                check_out: StorageHashMap::new(),
                check_in_status: StorageHashMap::new(),
                check_in_time: StorageHashMap::new(),
                change_able: false,
                owner: caller,
            }
        }

        #[ink(message)]
        pub fn checked_in(&mut self, attendant_hash: Hash, location: String) -> Result<(), Error> {
            if attendant_hash != self.attendant_hash {
                return Err(Error::InvalidHash);
            } else {
                let caller = self.env().caller();

                if self.checked_in_status(caller) == true {
                    return Err(Error::CheckOutFail);
                };

                let now = self.env().block_timestamp();
                let info = Attendant {
                    time: now.clone(),
                    location: location.clone(),
                };
                self.check_in_time.insert(caller, now);

                let mut attendant_list = self.checked_in_list(caller);
                attendant_list.push(info);
                self.check_in.insert(caller, attendant_list);

                let token = self.get_token(caller);
                self.token.insert(caller, token + 8);

                self.check_in_status.insert(caller, true);
                if self.change_able {
                    self.generate_hash();
                };
                Ok(())
            }
        }

        #[ink(message)]
        pub fn checked_out(&mut self, attendant_hash: Hash, location: String) -> Result<(), Error> {
            if attendant_hash != self.attendant_hash {
                return Err(Error::InvalidHash);
            } else {
                let caller = self.env().caller();

                if self.checked_in_status(caller) == false {
                    return Err(Error::CheckOutFail);
                };

                let now = self.env().block_timestamp();
                let info = Attendant {
                    time: now,
                    location: location.clone(),
                };

                let mut attendant_list = self.checked_out_list(caller);
                attendant_list.push(info);
                self.check_out.insert(caller, attendant_list);

                let hour = 1000 * 60 * 60;
                if now - self.get_checked_time(caller) > hour * 8 {
                    let token = self.get_token(caller);
                    self.token.insert(caller, token + 2);
                }

                self.check_in_status.insert(caller, false);
                if self.change_able {
                    self.generate_hash();
                };
                Ok(())
            }
        }

        #[ink(message)]
        pub fn get_token(&self, attender: AccountId) -> Balance {
            self.token.get(&attender).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn checked_in_list(&self, attender: AccountId) -> Vec<Attendant> {
            match self.check_in.get(&attender) {
                None => Vec::new(),
                Some(l) => l.clone(),
            }
        }

        #[ink(message)]
        pub fn checked_out_list(&self, attender: AccountId) -> Vec<Attendant> {
            match self.check_out.get(&attender) {
                None => Vec::new(),
                Some(l) => l.clone(),
            }
        }

        #[ink(message)]
        pub fn checked_in_status(&self, attender: AccountId) -> bool {
            self.check_in_status
                .get(&attender)
                .copied()
                .unwrap_or(false)
        }

        #[ink(message)]
        pub fn is_changeable(&mut self, can: bool)-> Result<(), Error>  {
            let caller = self.env().caller();
            if caller == self.owner {
                self.change_able = can;
                Ok(())
            }else {
                Err(Error::PermissionDeny)
            }
            
        }

        #[ink(message)]
        pub fn get_hash(&self) -> Hash {
            self.attendant_hash
        }

        fn generate_hash(&mut self) {
            let x = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 12, 34, 56, 78, 90];
            let new_hash = self.env().random(&x);
            self.attendant_hash = new_hash;
        }

        fn get_checked_time(&self, attender: AccountId) -> u64 {
            self.check_in_time.get(&attender).copied().unwrap_or(0)
        }
    }
}
